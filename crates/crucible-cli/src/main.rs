#![forbid(unsafe_code)]

use crucible_cli::{
    artifact_imports_table_sql, artifact_migration_checksum, artifact_migration_name,
    artifact_migration_sql, artifacts_table_sql, database_snapshot_is_exact,
    database_snapshot_is_exact_v1, decide_workspace_initialization, metadata_table_sql,
    migration_checksum, migration_table_sql, object_address_for_artifact,
    object_address_matches_id, parse_cli_args, prepare_artifact_publication,
    stored_artifact_is_exact, ArtifactStoreError, CliAction, CliParseError, DatabaseSnapshot,
    InitializationDecision, InitializationError, MigrationRecord, ObjectAddress, PathKind,
    PreparedArtifactPublication, StoredArtifactSnapshot, WorkspaceMetadata, WorkspaceSnapshot,
    MAX_CLI_ARGUMENTS, MAX_CLI_ARGUMENT_BYTES, MAX_LOCAL_ARTIFACT_BYTES, WORKSPACE_APPLICATION_ID,
    WORKSPACE_SCHEMA_VERSION,
};
use crucible_core::{ArtifactId, ArtifactRef};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::path::{Component, Path, PathBuf};
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostArgumentError {
    NonUtf8,
    TooMany,
    TooLong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostWorkspaceError {
    UnsafeRoot,
    Inspect,
    Publish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostWorkspaceAction {
    Inspect,
    Publish,
    MigrateV1,
}

#[derive(Debug)]
// The verified orchestration consumes this boundary result immediately; indirection would add a
// second allocator-backed host behavior solely to shrink a short-lived enum.
#[expect(clippy::large_enum_variant, reason = "short-lived trusted-boundary results avoid an additional host allocation")]
enum HostWorkspaceOutcome {
    Snapshot(WorkspaceSnapshot),
    Published,
    Migrated,
    Raced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InitCommandError {
    UnsafeRoot,
    OccupiedState,
    IncompatibleDatabase,
    Inspect,
    Publish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostArtifactAction {
    ReadSource,
    Publish,
    Load,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostArtifactError {
    UnsafeSource,
    TooLarge,
    Workspace,
    Publish,
    Load,
}

#[derive(Debug)]
enum HostArtifactOutcome {
    Source(Vec<u8>, String),
    Published,
    Snapshot(StoredArtifactSnapshot),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArtifactCommandError {
    UnsafeSource,
    TooLarge,
    InvalidId,
    Workspace,
    Publish,
    Integrity,
}

// CRUCIBLE-TCB: CLI-HOST-ARGS-001
#[verifier::external_body]
fn host_cli_args() -> (result: Result<Vec<String>, HostArgumentError>) {
    let mut arguments = Vec::new();
    for argument in std::env::args_os().skip(1) {
        if arguments.len() == MAX_CLI_ARGUMENTS {
            return Err(HostArgumentError::TooMany);
        }
        let argument = argument.into_string().map_err(|_| HostArgumentError::NonUtf8)?;
        if argument.len() > MAX_CLI_ARGUMENT_BYTES {
            return Err(HostArgumentError::TooLong);
        }
        arguments.push(argument);
    }
    Ok(arguments)
}

// CRUCIBLE-TCB: CLI-HOST-INIT-001
#[verifier::external_body]
fn host_workspace_action(root: &str, action: HostWorkspaceAction) -> (result: Result<
    HostWorkspaceOutcome,
    HostWorkspaceError,
>) {
    fn path_kind(path: &Path) -> PathKind {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => PathKind::Symlink,
            Ok(metadata) if metadata.is_dir() => PathKind::Directory,
            Ok(metadata) if metadata.is_file() => PathKind::File,
            Ok(_) => PathKind::Other,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => PathKind::Missing,
            Err(_) => PathKind::Other,
        }
    }

    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostWorkspaceError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostWorkspaceError::UnsafeRoot)?.join(path)
        };
        let mut normalized = PathBuf::new();
        for component in candidate.components() {
            match component {
                Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                    normalized.push(component.as_os_str());
                },
                Component::CurDir => {},
                Component::ParentDir => {
                    let _ = normalized.pop();
                },
            }
        }
        if !normalized.is_absolute() {
            return Err(HostWorkspaceError::UnsafeRoot);
        }
        Ok(normalized)
    }

    fn set_private_directory_permissions(path: &Path) -> Result<(), HostWorkspaceError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(
                |_| HostWorkspaceError::Publish,
            )?;
        }
        Ok(())
    }

    fn set_private_file_permissions(path: &Path) -> Result<(), HostWorkspaceError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(
                |_| HostWorkspaceError::Publish,
            )?;
        }
        Ok(())
    }

    fn safe_root(path: &Path) -> Result<PathBuf, HostWorkspaceError> {
        let absolute = lexical_absolute(path)?;
        let mut cursor = PathBuf::new();
        for component in absolute.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            match std::fs::symlink_metadata(&cursor) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err(HostWorkspaceError::UnsafeRoot);
                    }
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    match std::fs::create_dir(&cursor) {
                        Ok(()) => {},
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                                |_| HostWorkspaceError::UnsafeRoot,
                            )?;
                            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                                return Err(HostWorkspaceError::UnsafeRoot);
                            }
                        },
                        Err(_) => return Err(HostWorkspaceError::UnsafeRoot),
                    }
                },
                Err(_) => return Err(HostWorkspaceError::UnsafeRoot),
            }
        }
        let canonical = std::fs::canonicalize(&absolute).map_err(
            |_| HostWorkspaceError::UnsafeRoot,
        )?;
        #[cfg(unix)]
        if canonical != absolute {
            return Err(HostWorkspaceError::UnsafeRoot);
        }
        Ok(canonical)
    }

    fn directory_count(path: &Path) -> Result<u64, HostWorkspaceError> {
        let mut count = 0_u64;
        let entries = std::fs::read_dir(path).map_err(|_| HostWorkspaceError::Inspect)?;
        for entry in entries {
            entry.map_err(|_| HostWorkspaceError::Inspect)?;
            count = count.checked_add(1).ok_or(HostWorkspaceError::Inspect)?;
        }
        Ok(count)
    }

    fn query_i64(connection: &Connection, sql: &str) -> Option<i64> {
        connection.query_row(sql, [], |row| row.get(0)).ok()
    }

    fn query_bytes(connection: &Connection, sql: &str) -> Option<Vec<u8>> {
        connection.query_row(sql, [], |row| row.get::<_, String>(0)).ok().map(String::into_bytes)
    }

    fn query_object(connection: &Connection, name: &str) -> (Vec<u8>, Vec<u8>) {
        connection.query_row(
            "SELECT type, COALESCE(sql, '') FROM sqlite_schema WHERE name = ?1",
            [name],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).map(|(kind, sql)| (kind.into_bytes(), sql.into_bytes())).unwrap_or_else(
            |_| (Vec::new(), Vec::new()),
        )
    }

    fn inspect_database(path: &Path) -> Option<DatabaseSnapshot> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let connection = Connection::open_with_flags(path, flags).ok()?;
        connection.busy_timeout(std::time::Duration::from_secs(5)).ok()?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;").ok()?;
        connection.execute_batch("BEGIN DEFERRED;").ok()?;
        let application_id = query_i64(&connection, "PRAGMA application_id")?;
        let schema_version = query_i64(&connection, "PRAGMA user_version")?;
        let journal_mode = query_bytes(&connection, "PRAGMA journal_mode")?;
        let synchronous = query_i64(&connection, "PRAGMA synchronous")?;
        let quick_check = query_bytes(&connection, "PRAGMA quick_check")?;
        let schema_objects = query_i64(
            &connection,
            "SELECT COUNT(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
        )?;
        let schema_object_count = u64::try_from(schema_objects).ok()?;
        let (migrations_table_kind, migrations_table_sql) = query_object(
            &connection,
            "schema_migrations",
        );
        let (metadata_table_kind, metadata_table_sql) = query_object(
            &connection,
            "workspace_metadata",
        );
        let (artifacts_table_kind, artifacts_table_sql) = query_object(&connection, "artifacts");
        let (artifact_imports_table_kind, artifact_imports_table_sql) = query_object(
            &connection,
            "artifact_imports",
        );

        let migration_rows = query_i64(
            &connection,
            "SELECT COUNT(*) FROM schema_migrations",
        ).and_then(|count| u64::try_from(count).ok());
        let mut migration_statement = connection.prepare(
            "SELECT version, name, checksum FROM schema_migrations ORDER BY version LIMIT 3",
        ).ok()?;
        let migration_query = migration_statement.query_map(
            [],
            |row|
                {
                    Ok(
                        MigrationRecord {
                            version: row.get(0)?,
                            name: row.get::<_, String>(1)?.into_bytes(),
                            checksum: row.get::<_, String>(2)?.into_bytes(),
                        },
                    )
                },
        ).ok()?;
        let mut migrations = Vec::new();
        for migration in migration_query {
            migrations.push(migration.ok()?);
        }
        let metadata_rows = query_i64(
            &connection,
            "SELECT COUNT(*) FROM workspace_metadata",
        ).and_then(|count| u64::try_from(count).ok());
        let metadata = connection.query_row(
            "SELECT key, value FROM workspace_metadata ORDER BY key LIMIT 1",
            [],
            |row|
                {
                    Ok(
                        WorkspaceMetadata {
                            key: row.get::<_, String>(0)?.into_bytes(),
                            value: row.get::<_, String>(1)?.into_bytes(),
                        },
                    )
                },
        ).optional().ok().flatten();

        let snapshot = DatabaseSnapshot {
            application_id,
            schema_version,
            journal_mode,
            synchronous,
            quick_check,
            schema_object_count,
            migrations_table_kind,
            migrations_table_sql,
            metadata_table_kind,
            metadata_table_sql,
            artifacts_table_kind,
            artifacts_table_sql,
            artifact_imports_table_kind,
            artifact_imports_table_sql,
            migration_row_count: migration_rows.unwrap_or(u64::MAX),
            migrations,
            metadata_row_count: metadata_rows.unwrap_or(u64::MAX),
            metadata,
        };
        connection.execute_batch("COMMIT;").ok()?;
        Some(snapshot)
    }

    fn missing_snapshot() -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            root_kind: PathKind::Directory,
            state_kind: PathKind::Missing,
            state_entry_count: 0,
            corpus_kind: PathKind::Missing,
            corpus_entry_count: 0,
            seeds_kind: PathKind::Missing,
            interesting_kind: PathKind::Missing,
            coverage_kind: PathKind::Missing,
            regression_kind: PathKind::Missing,
            minimized_kind: PathKind::Missing,
            findings_kind: PathKind::Missing,
            objects_kind: PathKind::Missing,
            runs_kind: PathKind::Missing,
            reports_kind: PathKind::Missing,
            database_kind: PathKind::Missing,
            database_wal_kind: PathKind::Missing,
            database_shm_kind: PathKind::Missing,
            database: None,
        }
    }

    fn inspect_workspace(root: &Path) -> Result<WorkspaceSnapshot, HostWorkspaceError> {
        let state = root.join(".crucible");
        let state_kind = path_kind(&state);
        if state_kind == PathKind::Missing {
            return Ok(missing_snapshot());
        }
        if state_kind != PathKind::Directory {
            let mut snapshot = missing_snapshot();
            snapshot.state_kind = state_kind;
            return Ok(snapshot);
        }
        let corpus = state.join("corpus");
        let database_path = state.join("database.sqlite");
        let corpus_kind = path_kind(&corpus);
        let database_kind = path_kind(&database_path);
        let database = if database_kind == PathKind::File {
            inspect_database(&database_path)
        } else {
            None
        };
        Ok(
            WorkspaceSnapshot {
                root_kind: PathKind::Directory,
                state_kind,
                state_entry_count: directory_count(&state)?,
                corpus_kind,
                corpus_entry_count: if corpus_kind == PathKind::Directory {
                    directory_count(&corpus)?
                } else {
                    0
                },
                seeds_kind: path_kind(&corpus.join("seeds")),
                interesting_kind: path_kind(&corpus.join("interesting")),
                coverage_kind: path_kind(&corpus.join("coverage")),
                regression_kind: path_kind(&corpus.join("regression")),
                minimized_kind: path_kind(&corpus.join("minimized")),
                findings_kind: path_kind(&state.join("findings")),
                objects_kind: path_kind(&state.join("objects")),
                runs_kind: path_kind(&state.join("runs")),
                reports_kind: path_kind(&state.join("reports")),
                database_kind,
                database_wal_kind: path_kind(&state.join("database.sqlite-wal")),
                database_shm_kind: path_kind(&state.join("database.sqlite-shm")),
                database,
            },
        )
    }

    fn sync_directory(path: &Path) -> Result<(), HostWorkspaceError> {
        std::fs::File::open(path).and_then(|directory| directory.sync_all()).map_err(
            |_| HostWorkspaceError::Publish,
        )
    }

    fn build_staged_workspace(staging: &Path) -> Result<(), HostWorkspaceError> {
        std::fs::create_dir(staging).map_err(|_| HostWorkspaceError::Publish)?;
        set_private_directory_permissions(staging)?;
        for relative in [
            "corpus",
            "corpus/seeds",
            "corpus/interesting",
            "corpus/coverage",
            "corpus/regression",
            "corpus/minimized",
            "findings",
            "objects",
            "runs",
            "reports",
        ] {
            let directory = staging.join(relative);
            std::fs::create_dir(&directory).map_err(|_| HostWorkspaceError::Publish)?;
            set_private_directory_permissions(&directory)?;
        }

        let database_path = staging.join("database.sqlite");
        let mut connection = Connection::open(&database_path).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
                 PRAGMA synchronous = FULL;
                 PRAGMA journal_mode = WAL;",
        ).map_err(|_| HostWorkspaceError::Publish)?;
        let migrations_sql = String::from_utf8(migration_table_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let metadata_sql = String::from_utf8(metadata_table_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let artifacts_sql = String::from_utf8(artifacts_table_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let imports_sql = String::from_utf8(artifact_imports_table_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let checksum = String::from_utf8(migration_checksum()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let artifact_name = String::from_utf8(artifact_migration_name()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let artifact_checksum = String::from_utf8(artifact_migration_checksum()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let transaction = connection.transaction().map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute_batch(
            &format!("{migrations_sql};{metadata_sql};{artifacts_sql};{imports_sql};"),
        ).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (?1, ?2, ?3)",
            params![1, "initialize-workspace", checksum],
        ).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (?1, ?2, ?3)",
            params![2, artifact_name, artifact_checksum],
        ).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute(
            "INSERT INTO workspace_metadata(key, value) VALUES (?1, ?2)",
            params!["format", "crucible-workspace-v1"],
        ).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.pragma_update(None, "application_id", WORKSPACE_APPLICATION_ID).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        transaction.pragma_update(None, "user_version", WORKSPACE_SCHEMA_VERSION).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        transaction.commit().map_err(|_| HostWorkspaceError::Publish)?;
        connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        connection.close().map_err(|_| HostWorkspaceError::Publish)?;
        set_private_file_permissions(&database_path)?;
        std::fs::File::open(&database_path).and_then(|database| database.sync_all()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        for relative in [
            "corpus/seeds",
            "corpus/interesting",
            "corpus/coverage",
            "corpus/regression",
            "corpus/minimized",
            "corpus",
            "findings",
            "objects",
            "runs",
            "reports",
        ] {
            sync_directory(&staging.join(relative))?;
        }
        sync_directory(staging)
    }

    fn migrate_version_one_workspace(state: &Path) -> Result<bool, HostWorkspaceError> {
        let database_path = state.join("database.sqlite");
        let mut connection = Connection::open(&database_path).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA synchronous = FULL;
             PRAGMA busy_timeout = 5000;",
        ).map_err(|_| HostWorkspaceError::Publish)?;
        let migration_sql = String::from_utf8(artifact_migration_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let name = String::from_utf8(artifact_migration_name()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let checksum = String::from_utf8(artifact_migration_checksum()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let transaction = connection.transaction_with_behavior(
            rusqlite::TransactionBehavior::Immediate,
        ).map_err(|_| HostWorkspaceError::Publish)?;
        let current_version: i64 = transaction.query_row(
            "PRAGMA user_version",
            [],
            |row| row.get(0),
        ).map_err(|_| HostWorkspaceError::Publish)?;
        if current_version == WORKSPACE_SCHEMA_VERSION {
            transaction.rollback().map_err(|_| HostWorkspaceError::Publish)?;
            connection.close().map_err(|_| HostWorkspaceError::Publish)?;
            return Ok(false);
        }
        if current_version != 1 {
            return Err(HostWorkspaceError::Publish);
        }
        transaction.execute_batch(&migration_sql).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (2, ?1, ?2)",
            params![name, checksum],
        ).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.commit().map_err(|_| HostWorkspaceError::Publish)?;
        connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        connection.close().map_err(|_| HostWorkspaceError::Publish)?;
        set_private_file_permissions(&database_path)?;
        set_private_directory_permissions(state)?;
        for relative in [
            "corpus",
            "corpus/seeds",
            "corpus/interesting",
            "corpus/coverage",
            "corpus/regression",
            "corpus/minimized",
            "findings",
            "objects",
            "runs",
            "reports",
        ] {
            set_private_directory_permissions(&state.join(relative))?;
        }
        std::fs::File::open(&database_path).and_then(|database| database.sync_all()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        sync_directory(state)?;
        Ok(true)
    }

    let root = safe_root(Path::new(root))?;
    match action {
        HostWorkspaceAction::Inspect => {
            Ok(HostWorkspaceOutcome::Snapshot(inspect_workspace(&root)?))
        },
        HostWorkspaceAction::Publish => {
            let state = root.join(".crucible");
            if path_kind(&state) != PathKind::Missing {
                return Ok(HostWorkspaceOutcome::Raced);
            }
            let staging = root.join(format!(".crucible.init-{}", std::process::id()));
            if path_kind(&staging) != PathKind::Missing {
                return Err(HostWorkspaceError::Publish);
            }
            if let Err(error) = build_staged_workspace(&staging) {
                let _ = std::fs::remove_dir_all(&staging);
                return Err(error);
            }
            let staged_database = inspect_database(&staging.join("database.sqlite"));
            if staged_database.as_ref().is_none_or(
                |database| !database_snapshot_is_exact(database),
            ) {
                let _ = std::fs::remove_dir_all(&staging);
                return Err(HostWorkspaceError::Publish);
            }
            match std::fs::rename(&staging, &state) {
                Ok(()) => {
                    sync_directory(&root)?;
                    Ok(HostWorkspaceOutcome::Published)
                },
                Err(_) if path_kind(&state) != PathKind::Missing => {
                    let _ = std::fs::remove_dir_all(&staging);
                    Ok(HostWorkspaceOutcome::Raced)
                },
                Err(_) => {
                    let _ = std::fs::remove_dir_all(&staging);
                    Err(HostWorkspaceError::Publish)
                },
            }
        },
        HostWorkspaceAction::MigrateV1 => {
            let state = root.join(".crucible");
            let snapshot = inspect_workspace(&root)?;
            match snapshot.database.as_ref() {
                Some(database) if database_snapshot_is_exact(database) => {
                    return Ok(HostWorkspaceOutcome::Raced);
                },
                Some(database) if database_snapshot_is_exact_v1(database) => {},
                _ => return Ok(HostWorkspaceOutcome::Raced),
            }
            let migrated_by_this_process = migrate_version_one_workspace(&state)?;
            let migrated = inspect_database(&state.join("database.sqlite"));
            if migrated.as_ref().is_none_or(|database| !database_snapshot_is_exact(database)) {
                return Err(HostWorkspaceError::Publish);
            }
            if migrated_by_this_process {
                Ok(HostWorkspaceOutcome::Migrated)
            } else {
                Ok(HostWorkspaceOutcome::Raced)
            }
        },
    }
}

// CRUCIBLE-TCB: CLI-HOST-ARTIFACT-001
#[verifier::external_body]
fn host_artifact_action(
    action: HostArtifactAction,
    root: &str,
    subject: &str,
    address: Option<&ObjectAddress>,
    artifact: Option<&ArtifactRef>,
    contents: &[u8],
) -> (result: Result<HostArtifactOutcome, HostArtifactError>) {
    use std::io::{Read, Write};

    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostArtifactError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostArtifactError::Workspace)?.join(path)
        };
        let mut normalized = PathBuf::new();
        for component in candidate.components() {
            match component {
                Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                    normalized.push(component.as_os_str());
                },
                Component::CurDir => {},
                Component::ParentDir => {
                    let _ = normalized.pop();
                },
            }
        }
        if normalized.is_absolute() {
            Ok(normalized)
        } else {
            Err(HostArtifactError::Workspace)
        }
    }

    fn safe_existing_file(path: &Path) -> Result<PathBuf, HostArtifactError> {
        let absolute = lexical_absolute(path).map_err(|_| HostArtifactError::UnsafeSource)?;
        let mut cursor = PathBuf::new();
        for component in absolute.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                |_| HostArtifactError::UnsafeSource,
            )?;
            if metadata.file_type().is_symlink() {
                return Err(HostArtifactError::UnsafeSource);
            }
            if cursor == absolute {
                if !metadata.is_file() {
                    return Err(HostArtifactError::UnsafeSource);
                }
            } else if !metadata.is_dir() {
                return Err(HostArtifactError::UnsafeSource);
            }
        }
        let canonical = std::fs::canonicalize(&absolute).map_err(
            |_| HostArtifactError::UnsafeSource,
        )?;
        #[cfg(unix)]
        if canonical != absolute {
            return Err(HostArtifactError::UnsafeSource);
        }
        Ok(canonical)
    }

    fn safe_workspace(root: &Path) -> Result<PathBuf, HostArtifactError> {
        let absolute = lexical_absolute(root)?;
        let mut cursor = PathBuf::new();
        for component in absolute.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                |_| HostArtifactError::Workspace,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(HostArtifactError::Workspace);
            }
        }
        let canonical = std::fs::canonicalize(&absolute).map_err(|_| HostArtifactError::Workspace)?;
        #[cfg(unix)]
        if canonical != absolute {
            return Err(HostArtifactError::Workspace);
        }
        for relative in [".crucible", ".crucible/objects"] {
            let metadata = std::fs::symlink_metadata(canonical.join(relative)).map_err(
                |_| HostArtifactError::Workspace,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(HostArtifactError::Workspace);
            }
        }
        let database = std::fs::symlink_metadata(
            canonical.join(".crucible/database.sqlite"),
        ).map_err(|_| HostArtifactError::Workspace)?;
        if database.file_type().is_symlink() || !database.is_file() {
            return Err(HostArtifactError::Workspace);
        }
        Ok(canonical)
    }

    fn read_bounded(path: &Path) -> Result<Vec<u8>, HostArtifactError> {
        let metadata = std::fs::symlink_metadata(path).map_err(
            |_| HostArtifactError::UnsafeSource,
        )?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(HostArtifactError::UnsafeSource);
        }
        if metadata.len() > MAX_LOCAL_ARTIFACT_BYTES {
            return Err(HostArtifactError::TooLarge);
        }
        let file = std::fs::File::open(path).map_err(|_| HostArtifactError::UnsafeSource)?;
        let mut bytes = Vec::new();
        file.take(MAX_LOCAL_ARTIFACT_BYTES + 1).read_to_end(&mut bytes).map_err(
            |_| HostArtifactError::UnsafeSource,
        )?;
        if bytes.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES {
            return Err(HostArtifactError::TooLarge);
        }
        Ok(bytes)
    }

    fn set_private_directory(path: &Path) -> Result<(), HostArtifactError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(
                |_| HostArtifactError::Publish,
            )?;
        }
        Ok(())
    }

    fn set_private_file(path: &Path) -> Result<(), HostArtifactError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(
                |_| HostArtifactError::Publish,
            )?;
        }
        Ok(())
    }

    fn ensure_directory(path: &Path) -> Result<(), HostArtifactError> {
        match std::fs::create_dir(path) {
            Ok(()) => set_private_directory(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let metadata = std::fs::symlink_metadata(path).map_err(
                    |_| HostArtifactError::Publish,
                )?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    Err(HostArtifactError::Publish)
                } else {
                    Ok(())
                }
            },
            Err(_) => Err(HostArtifactError::Publish),
        }
    }

    fn sync_directory(path: &Path) -> Result<(), HostArtifactError> {
        std::fs::File::open(path).and_then(|directory| directory.sync_all()).map_err(
            |_| HostArtifactError::Publish,
        )
    }

    fn read_published_object(path: &Path) -> Result<Vec<u8>, HostArtifactError> {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| HostArtifactError::Load)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len()
            > MAX_LOCAL_ARTIFACT_BYTES {
            return Err(HostArtifactError::Load);
        }
        let file = std::fs::File::open(path).map_err(|_| HostArtifactError::Load)?;
        let mut bytes = Vec::new();
        file.take(MAX_LOCAL_ARTIFACT_BYTES + 1).read_to_end(&mut bytes).map_err(
            |_| HostArtifactError::Load,
        )?;
        if bytes.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES {
            Err(HostArtifactError::Load)
        } else {
            Ok(bytes)
        }
    }

    fn object_path(root: &Path, address: &ObjectAddress) -> PathBuf {
        root.join(".crucible/objects").join(&address.algorithm).join(&address.first).join(
            &address.second,
        ).join(&address.object_name)
    }

    fn publish_object(root: &Path, address: &ObjectAddress, contents: &[u8]) -> Result<
        (),
        HostArtifactError,
    > {
        let objects = root.join(".crucible/objects");
        let algorithm = objects.join(&address.algorithm);
        let first = algorithm.join(&address.first);
        let second = first.join(&address.second);
        ensure_directory(&algorithm)?;
        ensure_directory(&first)?;
        ensure_directory(&second)?;
        let target = second.join(&address.object_name);
        if target.exists() {
            return if read_published_object(&target)? == contents {
                Ok(())
            } else {
                Err(HostArtifactError::Publish)
            };
        }
        let mut temporary = None;
        for sequence in 0..16_u8 {
            let candidate = second.join(
                format!(
                ".{}.tmp-{}-{sequence}",
                address.object_name,
                std::process::id()
            ),
            );
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&candidate) {
                Ok(mut file) => {
                    set_private_file(&candidate)?;
                    file.write_all(contents).map_err(|_| HostArtifactError::Publish)?;
                    file.sync_all().map_err(|_| HostArtifactError::Publish)?;
                    temporary = Some(candidate);
                    break;
                },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
                Err(_) => return Err(HostArtifactError::Publish),
            }
        }
        let temporary = temporary.ok_or(HostArtifactError::Publish)?;
        if read_published_object(&temporary).map_err(|_| HostArtifactError::Publish)? != contents {
            let _ = std::fs::remove_file(&temporary);
            return Err(HostArtifactError::Publish);
        }
        match std::fs::hard_link(&temporary, &target) {
            Ok(()) => {},
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if read_published_object(&target).map_err(|_| HostArtifactError::Publish)?
                    != contents {
                    let _ = std::fs::remove_file(&temporary);
                    return Err(HostArtifactError::Publish);
                }
            },
            Err(_) => {
                let _ = std::fs::remove_file(&temporary);
                return Err(HostArtifactError::Publish);
            },
        }
        std::fs::remove_file(&temporary).map_err(|_| HostArtifactError::Publish)?;
        sync_directory(&second)?;
        sync_directory(&first)?;
        sync_directory(&algorithm)?;
        sync_directory(&objects)
    }

    let address = match (address, artifact) {
        (Some(address), Some(artifact)) if object_address_matches_id(&artifact.id, address) => {
            Some(address)
        },
        (Some(_), _) => return Err(HostArtifactError::Workspace),
        (None, _) => None,
    };
    match action {
        HostArtifactAction::ReadSource => {
            let source = safe_existing_file(Path::new(subject))?;
            let provenance = source.to_str().ok_or(HostArtifactError::UnsafeSource)?.to_owned();
            Ok(HostArtifactOutcome::Source(read_bounded(&source)?, provenance))
        },
        HostArtifactAction::Publish => {
            let root = safe_workspace(Path::new(root))?;
            let address = address.ok_or(HostArtifactError::Publish)?;
            let artifact = artifact.ok_or(HostArtifactError::Publish)?;
            if artifact.id.as_str() != format!("sha256:{}", address.object_name)
                || artifact.size_bytes != contents.len() as u64 || artifact.media_type.is_some() {
                return Err(HostArtifactError::Publish);
            }
            artifact.verify(contents).map_err(|_| HostArtifactError::Publish)?;
            publish_object(&root, address, contents)?;

            let database = root.join(".crucible/database.sqlite");
            let mut connection = Connection::open(&database).map_err(
                |_| HostArtifactError::Workspace,
            )?;
            connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(
                |_| HostArtifactError::Workspace,
            )?;
            connection.execute_batch(
                "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;",
            ).map_err(|_| HostArtifactError::Workspace)?;
            let transaction = connection.transaction_with_behavior(
                rusqlite::TransactionBehavior::Immediate,
            ).map_err(|_| HostArtifactError::Publish)?;
            transaction.execute(
                "INSERT INTO artifacts(id, algorithm, digest, size_bytes, media_type)
                 VALUES (?1, 'sha256', ?2, ?3, NULL)
                 ON CONFLICT(id) DO NOTHING",
                params![artifact.id.as_str(), address.object_name, artifact.size_bytes as i64],
            ).map_err(|_| HostArtifactError::Publish)?;
            let stored: (String, String, String, i64, Option<String>) = transaction.query_row(
                "SELECT id, algorithm, digest, size_bytes, media_type FROM artifacts WHERE id = ?1",
                [artifact.id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).map_err(|_| HostArtifactError::Publish)?;
            if stored.0 != artifact.id.as_str() || stored.1 != "sha256" || stored.2
                != address.object_name || stored.3 != artifact.size_bytes as i64
                || stored.4.is_some() {
                return Err(HostArtifactError::Publish);
            }
            transaction.execute(
                "INSERT INTO artifact_imports(artifact_id, source_path) VALUES (?1, ?2)
                 ON CONFLICT(artifact_id, source_path) DO NOTHING",
                params![artifact.id.as_str(), subject.as_bytes()],
            ).map_err(|_| HostArtifactError::Publish)?;
            transaction.commit().map_err(|_| HostArtifactError::Publish)?;
            connection.close().map_err(|_| HostArtifactError::Publish)?;
            Ok(HostArtifactOutcome::Published)
        },
        HostArtifactAction::Load => {
            let root = safe_workspace(Path::new(root))?;
            let address = address.ok_or(HostArtifactError::Load)?;
            let requested = artifact.ok_or(HostArtifactError::Load)?;
            let database = root.join(".crucible/database.sqlite");
            let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
            let connection = Connection::open_with_flags(database, flags).map_err(
                |_| HostArtifactError::Load,
            )?;
            let stored_record = connection.query_row(
                "SELECT id, algorithm, digest, size_bytes, media_type FROM artifacts WHERE id = ?1",
                [requested.id.as_str()],
                |row|
                    {
                        let size: i64 = row.get(3)?;
                        let size = u64::try_from(size).map_err(
                            |_| rusqlite::Error::IntegralValueOutOfRange(3, size),
                        )?;
                        Ok(
                            (
                                ArtifactRef {
                                    id: ArtifactId::new(row.get(0)?),
                                    size_bytes: size,
                                    media_type: row.get(4)?,
                                },
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                            ),
                        )
                    },
            ).optional().map_err(|_| HostArtifactError::Load)?;
            let (record, record_algorithm, record_digest) = match stored_record {
                Some((record, algorithm, digest)) => { (Some(record), Some(algorithm), Some(digest))
                },
                None => (None, None, None),
            };
            let matching_import_count = if subject.is_empty() {
                connection.query_row(
                    "SELECT COUNT(*) FROM artifact_imports WHERE artifact_id = ?1",
                    [requested.id.as_str()],
                    |row| row.get::<_, i64>(0),
                )
            } else {
                connection.query_row(
                    "SELECT COUNT(*) FROM artifact_imports WHERE artifact_id = ?1 AND source_path = ?2",
                    params![requested.id.as_str(), subject.as_bytes()],
                    |row| row.get::<_, i64>(0),
                )
            }.ok().and_then(|count| u64::try_from(count).ok()).unwrap_or(u64::MAX);
            let objects = root.join(".crucible/objects");
            let algorithm = objects.join(&address.algorithm);
            let first = algorithm.join(&address.first);
            let second = first.join(&address.second);
            for directory in [&algorithm, &first, &second] {
                let metadata = std::fs::symlink_metadata(directory).map_err(
                    |_| HostArtifactError::Load,
                )?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(HostArtifactError::Load);
                }
            }
            let target = object_path(&root, address);
            let metadata = std::fs::symlink_metadata(&target).ok();
            let object_is_file = metadata.as_ref().is_some_and(
                |metadata| !metadata.file_type().is_symlink() && metadata.is_file(),
            );
            let stored_contents = if object_is_file {
                read_published_object(&target)?
            } else {
                Vec::new()
            };
            Ok(
                HostArtifactOutcome::Snapshot(
                    StoredArtifactSnapshot {
                        object_is_file,
                        record,
                        record_algorithm,
                        record_digest,
                        contents: stored_contents,
                        matching_import_count,
                    },
                ),
            )
        },
    }
}

// CRUCIBLE-TCB: CLI-HOST-COMPLETE-001
#[verifier::external_body]
fn host_complete(success: bool, message: &[u8]) {
    use std::io::Write;

    if success {
        let _ = std::io::stdout().write_all(message);
    } else {
        let _ = std::io::stderr().write_all(message);
        std::process::exit(1);
    }
}

fn map_policy_error(error: InitializationError) -> (mapped: InitCommandError) {
    match error {
        InitializationError::UnsafeRoot => InitCommandError::UnsafeRoot,
        InitializationError::OccupiedState => InitCommandError::OccupiedState,
        InitializationError::IncompatibleDatabase => InitCommandError::IncompatibleDatabase,
    }
}

fn inspect_workspace(root: &str) -> (result: Result<WorkspaceSnapshot, InitCommandError>) {
    match host_workspace_action(root, HostWorkspaceAction::Inspect) {
        Ok(HostWorkspaceOutcome::Snapshot(snapshot)) => Ok(snapshot),
        Err(HostWorkspaceError::UnsafeRoot) => Err(InitCommandError::UnsafeRoot),
        Err(HostWorkspaceError::Inspect) => Err(InitCommandError::Inspect),
        Err(HostWorkspaceError::Publish)
        | Ok(HostWorkspaceOutcome::Published)
        | Ok(HostWorkspaceOutcome::Migrated)
        | Ok(HostWorkspaceOutcome::Raced) => Err(InitCommandError::Inspect),
    }
}

fn initialize_workspace(root: &str) -> (result: Result<(), InitCommandError>) {
    let initial = inspect_workspace(root)?;
    match decide_workspace_initialization(&initial) {
        Ok(InitializationDecision::Reuse) => Ok(()),
        Err(error) => Err(map_policy_error(error)),
        Ok(InitializationDecision::MigrateV1) => {
            match host_workspace_action(root, HostWorkspaceAction::MigrateV1) {
                Ok(HostWorkspaceOutcome::Migrated) | Ok(HostWorkspaceOutcome::Raced) => {},
                Ok(HostWorkspaceOutcome::Snapshot(_))
                | Ok(HostWorkspaceOutcome::Published)
                | Err(HostWorkspaceError::Publish) => {
                    return Err(InitCommandError::Publish);
                },
                Err(HostWorkspaceError::UnsafeRoot) => {
                    return Err(InitCommandError::UnsafeRoot);
                },
                Err(HostWorkspaceError::Inspect) => return Err(InitCommandError::Inspect),
            }
            let completed = inspect_workspace(root)?;
            match decide_workspace_initialization(&completed) {
                Ok(InitializationDecision::Reuse) => Ok(()),
                Ok(InitializationDecision::Create) | Ok(InitializationDecision::MigrateV1) => {
                    Err(InitCommandError::Publish)
                },
                Err(error) => Err(map_policy_error(error)),
            }
        },
        Ok(InitializationDecision::Create) => {
            match host_workspace_action(root, HostWorkspaceAction::Publish) {
                Ok(HostWorkspaceOutcome::Published) | Ok(HostWorkspaceOutcome::Raced) => {},
                Ok(HostWorkspaceOutcome::Snapshot(_))
                | Ok(HostWorkspaceOutcome::Migrated)
                | Err(HostWorkspaceError::Publish) => {
                    return Err(InitCommandError::Publish);
                },
                Err(HostWorkspaceError::UnsafeRoot) => {
                    return Err(InitCommandError::UnsafeRoot);
                },
                Err(HostWorkspaceError::Inspect) => return Err(InitCommandError::Inspect),
            }
            let completed = inspect_workspace(root)?;
            match decide_workspace_initialization(&completed) {
                Ok(InitializationDecision::Reuse) => Ok(()),
                Ok(InitializationDecision::Create) | Ok(InitializationDecision::MigrateV1) => {
                    Err(InitCommandError::Publish)
                },
                Err(error) => Err(map_policy_error(error)),
            }
        },
    }
}

fn artifact_workspace_is_ready(root: &str) -> (ready: bool) {
    let snapshot = match inspect_workspace(root) {
        Ok(snapshot) => snapshot,
        Err(_) => return false,
    };
    match decide_workspace_initialization(&snapshot) {
        Ok(InitializationDecision::Reuse) => true,
        Ok(InitializationDecision::MigrateV1) => initialize_workspace(root).is_ok(),
        Ok(InitializationDecision::Create) | Err(_) => false,
    }
}

fn map_host_artifact_error(error: HostArtifactError) -> (mapped: ArtifactCommandError) {
    match error {
        HostArtifactError::UnsafeSource => ArtifactCommandError::UnsafeSource,
        HostArtifactError::TooLarge => ArtifactCommandError::TooLarge,
        HostArtifactError::Workspace => ArtifactCommandError::Workspace,
        HostArtifactError::Publish => ArtifactCommandError::Publish,
        HostArtifactError::Load => ArtifactCommandError::Integrity,
    }
}

fn read_artifact_source(source: &str) -> (result: Result<(Vec<u8>, String), ArtifactCommandError>)
    ensures
        match &result {
            Ok((contents, _)) => contents@.len() <= MAX_LOCAL_ARTIFACT_BYTES,
            Err(_) => true,
        },
{
    match host_artifact_action(HostArtifactAction::ReadSource, "", source, None, None, &[]) {
        Ok(HostArtifactOutcome::Source(contents, provenance)) => {
            if contents.len() as u128 > MAX_LOCAL_ARTIFACT_BYTES as u128 {
                Err(ArtifactCommandError::TooLarge)
            } else {
                Ok((contents, provenance))
            }
        },
        Ok(HostArtifactOutcome::Published) | Ok(HostArtifactOutcome::Snapshot(_)) => {
            Err(ArtifactCommandError::UnsafeSource)
        },
        Err(error) => Err(map_host_artifact_error(error)),
    }
}

fn publish_artifact(
    root: &str,
    source: &str,
    publication: &PreparedArtifactPublication,
    contents: &[u8],
) -> (result: Result<(), ArtifactCommandError>) {
    match host_artifact_action(
        HostArtifactAction::Publish,
        root,
        source,
        Some(&publication.address),
        Some(&publication.artifact),
        contents,
    ) {
        Ok(HostArtifactOutcome::Published) => {},
        Ok(HostArtifactOutcome::Source(_, _)) | Ok(HostArtifactOutcome::Snapshot(_)) => {
            return Err(ArtifactCommandError::Publish);
        },
        Err(error) => return Err(map_host_artifact_error(error)),
    }
    let snapshot = match host_artifact_action(
        HostArtifactAction::Load,
        root,
        source,
        Some(&publication.address),
        Some(&publication.artifact),
        &[],
    ) {
        Ok(HostArtifactOutcome::Snapshot(snapshot)) => snapshot,
        Ok(HostArtifactOutcome::Source(_, _)) | Ok(HostArtifactOutcome::Published) => {
            return Err(ArtifactCommandError::Integrity);
        },
        Err(error) => return Err(map_host_artifact_error(error)),
    };
    if stored_artifact_is_exact(&publication.artifact, true, &snapshot) {
        Ok(())
    } else {
        Err(ArtifactCommandError::Integrity)
    }
}

fn import_artifact(source: &str, root: &str) -> (result: Result<
    PreparedArtifactPublication,
    ArtifactCommandError,
>)
    ensures
        match &result {
            Ok(publication) => crucible_core::artifact::canonical_sha256_artifact_id_spec(
                publication.artifact.id@,
            ) && crucible_cli::object_address_spec(publication.artifact.id@, publication.address@),
            Err(_) => true,
        },
{
    if !artifact_workspace_is_ready(root) {
        return Err(ArtifactCommandError::Workspace);
    }
    let (contents, provenance) = read_artifact_source(source)?;
    let publication = match prepare_artifact_publication(&contents) {
        Ok(publication) => publication,
        Err(ArtifactStoreError::InputTooLong) => return Err(ArtifactCommandError::TooLarge),
        Err(ArtifactStoreError::MalformedArtifactId)
        | Err(ArtifactStoreError::UnsupportedAlgorithm)
        | Err(ArtifactStoreError::IntegrityMismatch) => {
            return Err(ArtifactCommandError::Integrity);
        },
    };
    publish_artifact(root, provenance.as_str(), &publication, &contents)?;
    Ok(publication)
}

fn verify_artifact(id: String, root: &str) -> (result: Result<ArtifactId, ArtifactCommandError>)
    ensures
        match &result {
            Ok(id) => crucible_core::artifact::canonical_sha256_artifact_id_spec(id@),
            Err(_) => true,
        },
{
    if !artifact_workspace_is_ready(root) {
        return Err(ArtifactCommandError::Workspace);
    }
    let requested_id = ArtifactId::new(id);
    let address = match object_address_for_artifact(&requested_id) {
        Ok(address) => address,
        Err(ArtifactStoreError::MalformedArtifactId)
        | Err(ArtifactStoreError::UnsupportedAlgorithm) => {
            return Err(ArtifactCommandError::InvalidId);
        },
        Err(ArtifactStoreError::InputTooLong) | Err(ArtifactStoreError::IntegrityMismatch) => {
            return Err(ArtifactCommandError::InvalidId);
        },
    };
    let request = ArtifactRef { id: requested_id, size_bytes: 0, media_type: None };
    let snapshot = match host_artifact_action(
        HostArtifactAction::Load,
        root,
        "",
        Some(&address),
        Some(&request),
        &[],
    ) {
        Ok(HostArtifactOutcome::Snapshot(snapshot)) => snapshot,
        Ok(HostArtifactOutcome::Source(_, _)) | Ok(HostArtifactOutcome::Published) => {
            return Err(ArtifactCommandError::Integrity);
        },
        Err(error) => return Err(map_host_artifact_error(error)),
    };
    if snapshot.contents.len() as u128 > MAX_LOCAL_ARTIFACT_BYTES as u128 {
        return Err(ArtifactCommandError::Integrity);
    }
    let expected = ArtifactRef {
        id: request.id,
        size_bytes: snapshot.contents.len() as u64,
        media_type: None,
    };
    if stored_artifact_is_exact(&expected, false, &snapshot) {
        Ok(expected.id)
    } else {
        Err(ArtifactCommandError::Integrity)
    }
}

fn artifact_output(prefix: &[u8], id: &ArtifactId) -> (output: Vec<u8>) {
    let mut output = vstd::slice::slice_to_vec(prefix);
    let bytes = id.as_str().as_bytes_vec();
    let mut index = 0;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
        decreases bytes.len() - index,
    {
        output.push(bytes[index]);
        index += 1;
    }
    output.push(b'\n');
    output
}

fn complete_artifact_error(error: ArtifactCommandError) {
    match error {
        ArtifactCommandError::UnsafeSource => {
            host_complete(false, b"crucible artifact import: unsafe artifact source\n")
        },
        ArtifactCommandError::TooLarge => host_complete(
            false,
            b"crucible artifact import: source exceeds the 67108864-byte import limit\n",
        ),
        ArtifactCommandError::InvalidId => {
            host_complete(false, b"crucible artifact verify: invalid artifact ID\n")
        },
        ArtifactCommandError::Workspace => host_complete(
            false,
            b"crucible artifact: workspace is missing, incompatible, or unsafe\n",
        ),
        ArtifactCommandError::Publish => {
            host_complete(false, b"crucible artifact import: publication failed\n")
        },
        ArtifactCommandError::Integrity => {
            host_complete(false, b"crucible artifact: artifact integrity check failed\n")
        },
    }
}

fn main() {
    let arguments = match host_cli_args() {
        Ok(arguments) => arguments,
        Err(HostArgumentError::NonUtf8) => {
            host_complete(false, b"crucible: path is not valid UTF-8\n");
            return;
        },
        Err(HostArgumentError::TooMany) => {
            host_complete(
                false,
                b"usage: crucible init [path]\n       crucible artifact import <file> [workspace]\n       crucible artifact verify <artifact-id> [workspace]\n",
            );
            return;
        },
        Err(HostArgumentError::TooLong) => {
            host_complete(false, b"crucible: argument exceeds the 4096-byte limit\n");
            return;
        },
    };
    let action = match parse_cli_args(&arguments) {
        Ok(action) => action,
        Err(CliParseError::UnsupportedArguments) => {
            host_complete(
                false,
                b"usage: crucible init [path]\n       crucible artifact import <file> [workspace]\n       crucible artifact verify <artifact-id> [workspace]\n",
            );
            return;
        },
    };
    match action {
        CliAction::Init(root) => match initialize_workspace(root.as_str()) {
            Ok(()) => host_complete(true, b"initialized Crucible workspace\n"),
            Err(InitCommandError::UnsafeRoot) => {
                host_complete(false, b"crucible init: unsafe workspace root\n")
            },
            Err(InitCommandError::OccupiedState) => {
                host_complete(false, b"crucible init: managed state path is occupied\n")
            },
            Err(InitCommandError::IncompatibleDatabase) => {
                host_complete(false, b"crucible init: incompatible workspace database\n")
            },
            Err(InitCommandError::Inspect) => {
                host_complete(false, b"crucible init: could not inspect workspace\n")
            },
            Err(InitCommandError::Publish) => {
                host_complete(false, b"crucible init: could not publish workspace\n")
            },
        },
        CliAction::ArtifactImport(source, root) => {
            match import_artifact(source.as_str(), root.as_str()) {
                Ok(publication) => {
                    let output = artifact_output(&[], &publication.artifact.id);
                    host_complete(true, output.as_slice());
                },
                Err(error) => complete_artifact_error(error),
            }
        },
        CliAction::ArtifactVerify(id, root) => match verify_artifact(id, root.as_str()) {
            Ok(id) => {
                let output = artifact_output(b"verified ", &id);
                host_complete(true, output.as_slice());
            },
            Err(error) => complete_artifact_error(error),
        },
    }
}

} // verus!
