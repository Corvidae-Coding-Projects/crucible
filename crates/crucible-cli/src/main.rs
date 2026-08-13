#![forbid(unsafe_code)]

use crucible_cli::{
    database_snapshot_is_exact, decide_workspace_initialization, metadata_table_sql,
    migration_checksum, migration_table_sql, parse_cli_args, CliAction, CliParseError,
    DatabaseSnapshot, InitializationDecision, InitializationError, MigrationRecord, PathKind,
    WorkspaceMetadata, WorkspaceSnapshot, MAX_CLI_ARGUMENTS, MAX_CLI_ARGUMENT_BYTES,
    WORKSPACE_APPLICATION_ID, WORKSPACE_SCHEMA_VERSION,
};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::path::{Component, Path, PathBuf};
use vstd::prelude::*;

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
}

#[derive(Debug)]
// The verified orchestration consumes this boundary result immediately; indirection would add a
// second allocator-backed host behavior solely to shrink a short-lived enum.
#[allow(clippy::large_enum_variant)]
enum HostWorkspaceOutcome {
    Snapshot(WorkspaceSnapshot),
    Published,
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
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;").ok()?;
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

        let migration_rows = query_i64(
            &connection,
            "SELECT COUNT(*) FROM schema_migrations",
        ).and_then(|count| u64::try_from(count).ok());
        let migration = connection.query_row(
            "SELECT version, name, checksum FROM schema_migrations ORDER BY version LIMIT 1",
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
        ).optional().ok().flatten();
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

        Some(
            DatabaseSnapshot {
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
                migration_row_count: migration_rows.unwrap_or(u64::MAX),
                migration,
                metadata_row_count: metadata_rows.unwrap_or(u64::MAX),
                metadata,
            },
        )
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
        let checksum = String::from_utf8(migration_checksum()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let transaction = connection.transaction().map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute_batch(&format!("{migrations_sql};{metadata_sql};")).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (?1, ?2, ?3)",
            params![WORKSPACE_SCHEMA_VERSION, "initialize-workspace", checksum],
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
        | Ok(HostWorkspaceOutcome::Raced) => Err(InitCommandError::Inspect),
    }
}

fn initialize_workspace(root: &str) -> (result: Result<(), InitCommandError>) {
    let initial = inspect_workspace(root)?;
    match decide_workspace_initialization(&initial) {
        Ok(InitializationDecision::Reuse) => Ok(()),
        Err(error) => Err(map_policy_error(error)),
        Ok(InitializationDecision::Create) => {
            match host_workspace_action(root, HostWorkspaceAction::Publish) {
                Ok(HostWorkspaceOutcome::Published) | Ok(HostWorkspaceOutcome::Raced) => {},
                Ok(HostWorkspaceOutcome::Snapshot(_)) | Err(HostWorkspaceError::Publish) => {
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
                Ok(InitializationDecision::Create) => Err(InitCommandError::Publish),
                Err(error) => Err(map_policy_error(error)),
            }
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
            host_complete(false, b"usage: crucible init [path]\n");
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
            host_complete(false, b"usage: crucible init [path]\n");
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
    }
}

} // verus!
