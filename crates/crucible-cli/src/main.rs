#![forbid(unsafe_code)]

use crucible_cli::{
    admit_run_store_transition, artifact_imports_table_sql, artifact_migration_checksum,
    artifact_migration_name, artifact_migration_sql, artifacts_table_sql,
    build_local_raw_observation, canonical_configuration_limits,
    canonical_local_capability_probe_report, classify_raw_local_execution,
    database_snapshot_is_exact, database_snapshot_is_exact_v1, database_snapshot_is_exact_v2,
    decide_workspace_initialization, encode_local_target_arguments, local_capability_manifest,
    metadata_table_sql, migration_checksum, migration_table_sql, object_address_for_artifact,
    object_address_matches_id, parse_cli_args, prepare_artifact_publication,
    prepare_local_execution, run_migration_checksum, run_migration_name, run_migration_sql,
    stored_artifact_is_exact, target_build_manifest, validate_configuration,
    validate_local_capability_probe, ArtifactStoreError, CapturedOutput, CliAction, CliParseError,
    ConfigurationError, ConfigurationErrorKind, DatabaseSnapshot, InitializationDecision,
    InitializationError, LocalExecutionClassificationError, LocalExecutionPlan, LocalNetworkPolicy,
    LocalRunPlanError, LocalRuntimeIdentity, LocalTermination, MigrationRecord, ObjectAddress,
    PathKind, PreparedArtifactPublication, RawLocalExecution, ReservedRun, RunAttemptStatus,
    RunStoreTransition, StoredArtifactSnapshot, WorkspaceMetadata, WorkspaceSnapshot,
    MAX_CLI_ARGUMENTS, MAX_CLI_ARGUMENT_BYTES, MAX_CONFIGURATION_SOURCE_BYTES,
    MAX_LOCAL_ARGUMENT_WIRE_BYTES, MAX_LOCAL_ARTIFACT_BYTES, MAX_LOCAL_CONTROL_STATUS_BYTES,
    MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES, WORKSPACE_APPLICATION_ID, WORKSPACE_SCHEMA_VERSION,
};
use crucible_core::{encode_raw_observation, ArtifactId, ArtifactRef, ContentDigest};
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
    MigrateV2,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostConfigError {
    UnsafeSource,
    TooLarge,
    Read,
    #[cfg(not(unix))]
    UnsupportedPlatform,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostLocalRunAction {
    ResolveWorkspace,
    ProbeCapabilities,
    ReadTarget,
    Execute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostLocalRunError {
    UnsafeConfigurationPath,
    Workspace,
    TargetPreparation,
    CapabilityUnavailable,
    Spawn,
    Capture,
    Cleanup,
}

#[derive(Debug)]
struct HostLocalRuntime {
    platform: String,
    architecture: String,
    kernel_release: String,
    bubblewrap_version: String,
    prlimit_version: String,
    harness_contents: Vec<u8>,
    bubblewrap_contents: Vec<u8>,
    prlimit_contents: Vec<u8>,
}

#[derive(Debug)]
enum HostLocalRunOutcome {
    WorkspaceRoot(String),
    CapabilityProbe(Vec<u8>, Option<HostLocalRuntime>),
    Target(Vec<u8>, String),
    Executed(RawLocalExecution),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostRunStoreAction {
    Reserve,
    AttachTarget,
    RecordObservation,
    RecordHarnessFailure,
}

#[derive(Debug)]
enum HostRunStoreOutcome {
    Reserved(ReservedRun),
    Updated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostRunStoreError {
    Workspace,
    Conflict,
    Persist,
}

spec fn host_local_runtime_well_formed_spec(runtime: &HostLocalRuntime) -> bool {
    0 < runtime.platform@.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < runtime.architecture@.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < runtime.kernel_release@.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < runtime.bubblewrap_version@.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < runtime.prlimit_version@.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES
        && runtime.harness_contents@.len() <= MAX_LOCAL_ARTIFACT_BYTES
        && runtime.bubblewrap_contents@.len() <= MAX_LOCAL_ARTIFACT_BYTES
        && runtime.prlimit_contents@.len() <= MAX_LOCAL_ARTIFACT_BYTES
}

spec fn host_local_run_result_shape_spec(
    action: HostLocalRunAction,
    plan: crucible_cli::LocalExecutionPlanView,
    result: &Result<HostLocalRunOutcome, HostLocalRunError>,
) -> bool {
    match result {
        Err(_) => true,
        Ok(HostLocalRunOutcome::WorkspaceRoot(root)) => {
            action == HostLocalRunAction::ResolveWorkspace && root@.len() > 0
        },
        Ok(HostLocalRunOutcome::CapabilityProbe(report, runtime)) => {
            action == HostLocalRunAction::ProbeCapabilities && match runtime {
                Some(identity) => host_local_runtime_well_formed_spec(identity) && report@
                    == crucible_cli::canonical_local_capability_probe_report_spec(plan, true),
                None => report@ == crucible_cli::canonical_local_capability_probe_report_spec(
                    plan,
                    false,
                ),
            }
        },
        Ok(HostLocalRunOutcome::Target(contents, provenance)) => {
            action == HostLocalRunAction::ReadTarget && contents@.len() <= MAX_LOCAL_ARTIFACT_BYTES
                && provenance@.len() > 0
        },
        Ok(HostLocalRunOutcome::Executed(raw)) => {
            action == HostLocalRunAction::Execute
                && crucible_cli::raw_local_execution_well_formed_spec(raw@)
                && raw@.stdout.retained.len() <= plan.max_stream_bytes && raw@.stderr.retained.len()
                <= plan.max_stream_bytes
        },
    }
}

spec fn host_run_store_result_shape_spec(
    action: HostRunStoreAction,
    result: &Result<HostRunStoreOutcome, HostRunStoreError>,
) -> bool {
    match result {
        Err(_) => true,
        Ok(HostRunStoreOutcome::Reserved(reservation)) => {
            action == HostRunStoreAction::Reserve && crucible_cli::reserved_run_well_formed_spec(
                reservation@,
            )
        },
        Ok(HostRunStoreOutcome::Updated) => action != HostRunStoreAction::Reserve,
    }
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

    fn query_run_schema_digest(connection: &Connection) -> Option<Vec<u8>> {
        let mut statement = connection.prepare(
            "SELECT name, type, COALESCE(sql, '') FROM sqlite_schema
             WHERE name NOT LIKE 'sqlite_%'
               AND name NOT IN ('schema_migrations','workspace_metadata','artifacts','artifact_imports')
             ORDER BY name",
        ).ok()?;
        let rows = statement.query_map(
            [],
            |row|
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        ).ok()?;
        let mut signature = Vec::new();
        let mut row_count = 0_u64;
        for row in rows {
            let (name, kind, sql) = row.ok()?;
            row_count = row_count.checked_add(1)?;
            for value in [name.as_bytes(), kind.as_bytes(), sql.as_bytes()] {
                signature.extend_from_slice(&(value.len() as u64).to_be_bytes());
                signature.extend_from_slice(value);
            }
        }
        if row_count == 0 {
            Some(Vec::new())
        } else {
            Some(
                ContentDigest::from_bytes(
                    signature.as_slice(),
                ).ok()?.into_artifact_id().as_str().as_bytes().to_vec(),
            )
        }
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
        let run_schema_digest = query_run_schema_digest(&connection)?;

        let migration_rows = query_i64(
            &connection,
            "SELECT COUNT(*) FROM schema_migrations",
        ).and_then(|count| u64::try_from(count).ok());
        let mut migration_statement = connection.prepare(
            "SELECT version, name, checksum FROM schema_migrations ORDER BY version LIMIT 4",
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
            run_schema_digest,
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
        let run_sql = String::from_utf8(run_migration_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let run_name = String::from_utf8(run_migration_name()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let run_checksum = String::from_utf8(run_migration_checksum()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let transaction = connection.transaction().map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute_batch(
            &format!("{migrations_sql};{metadata_sql};{artifacts_sql};{imports_sql};{run_sql}"),
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
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (?1, ?2, ?3)",
            params![3, run_name, run_checksum],
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

    fn migrate_workspace(state: &Path, expected_version: i64) -> Result<bool, HostWorkspaceError> {
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
        let run_sql = String::from_utf8(run_migration_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let run_name = String::from_utf8(run_migration_name()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let run_checksum = String::from_utf8(run_migration_checksum()).map_err(
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
        if current_version != expected_version || (expected_version != 1 && expected_version != 2) {
            return Err(HostWorkspaceError::Publish);
        }
        if current_version == 1 {
            transaction.execute_batch(&migration_sql).map_err(|_| HostWorkspaceError::Publish)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, name, checksum) VALUES (2, ?1, ?2)",
                params![name, checksum],
            ).map_err(|_| HostWorkspaceError::Publish)?;
        }
        transaction.execute_batch(&run_sql).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (3, ?1, ?2)",
            params![run_name, run_checksum],
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
        HostWorkspaceAction::MigrateV1 | HostWorkspaceAction::MigrateV2 => {
            let expected_version = match action {
                HostWorkspaceAction::MigrateV1 => 1,
                HostWorkspaceAction::MigrateV2 => 2,
                HostWorkspaceAction::Inspect | HostWorkspaceAction::Publish => {
                    return Err(HostWorkspaceError::Publish);
                },
            };
            let state = root.join(".crucible");
            let snapshot = inspect_workspace(&root)?;
            match snapshot.database.as_ref() {
                Some(database) if database_snapshot_is_exact(database) => {
                    return Ok(HostWorkspaceOutcome::Raced);
                },
                Some(database) if expected_version == 1 && database_snapshot_is_exact_v1(
                    database,
                ) => {},
                Some(database) if expected_version == 2 && database_snapshot_is_exact_v2(
                    database,
                ) => {},
                _ => return Ok(HostWorkspaceOutcome::Raced),
            }
            let migrated_by_this_process = migrate_workspace(&state, expected_version)?;
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
            if !subject.is_empty() {
                transaction.execute(
                    "INSERT INTO artifact_imports(artifact_id, source_path) VALUES (?1, ?2)
                     ON CONFLICT(artifact_id, source_path) DO NOTHING",
                    params![artifact.id.as_str(), subject.as_bytes()],
                ).map_err(|_| HostArtifactError::Publish)?;
            }
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

// CRUCIBLE-TCB: CLI-HOST-CONFIG-001
#[verifier::external_body]
fn host_read_configuration(path: &str) -> (result: Result<Vec<u8>, HostConfigError>) {
    use std::io::Read;

    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostConfigError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostConfigError::UnsafeSource)?.join(path)
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
            Err(HostConfigError::UnsafeSource)
        }
    }

    let absolute = lexical_absolute(Path::new(path))?;
    #[cfg(unix)]
    {
        use rustix::fs::{FileType, Mode, OFlags};

        let components: Vec<_> = absolute.components().filter_map(
            |component|
                match component {
                    Component::Normal(value) => Some(value),
                    _ => None,
                },
        ).collect();
        if components.is_empty() {
            return Err(HostConfigError::UnsafeSource);
        }
        let directory_flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW
            | OFlags::CLOEXEC;
        let mut directory = rustix::fs::open("/", directory_flags, Mode::empty()).map_err(
            |_| HostConfigError::UnsafeSource,
        )?;
        let mut final_file = None;
        for (index, component) in components.iter().enumerate() {
            let is_final = index + 1 == components.len();
            let flags = if is_final {
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK
            } else {
                directory_flags
            };
            let opened = rustix::fs::openat(&directory, *component, flags, Mode::empty()).map_err(
                |_| HostConfigError::UnsafeSource,
            )?;
            let stat = rustix::fs::fstat(&opened).map_err(|_| HostConfigError::UnsafeSource)?;
            let file_type = FileType::from_raw_mode(stat.st_mode);
            if is_final {
                if !file_type.is_file() {
                    return Err(HostConfigError::UnsafeSource);
                }
                let length = u64::try_from(stat.st_size).map_err(|_| HostConfigError::Read)?;
                if length > MAX_CONFIGURATION_SOURCE_BYTES {
                    return Err(HostConfigError::TooLarge);
                }
                final_file = Some(opened);
            } else {
                if !file_type.is_dir() {
                    return Err(HostConfigError::UnsafeSource);
                }
                directory = opened;
            }
        }
        let descriptor = final_file.ok_or(HostConfigError::UnsafeSource)?;
        let file = std::fs::File::from(descriptor);
        let mut contents = Vec::new();
        file.take(MAX_CONFIGURATION_SOURCE_BYTES + 1).read_to_end(&mut contents).map_err(
            |_| HostConfigError::Read,
        )?;
        if contents.len() as u64 > MAX_CONFIGURATION_SOURCE_BYTES {
            return Err(HostConfigError::TooLarge);
        }
        Ok(contents)
    }
    #[cfg(not(unix))]
    {
        let _ = absolute;
        Err(HostConfigError::UnsupportedPlatform)
    }
}

// CRUCIBLE-TCB: CLI-HOST-LOCAL-SUPERVISOR-001
#[verifier::external_body]
fn host_internal_local_supervisor()
    ensures
        false,
{
    #[cfg(target_os = "linux")]
    {
        use std::io::{Read, Write};
        use std::os::unix::process::ExitStatusExt;
        use std::process::{Command, Stdio};

        fn terminate(code: i32) -> ! {
            std::process::exit(code)
        }

        fn fail(marker: &[u8]) -> ! {
            let mut stderr = std::io::stderr().lock();
            let _ = stderr.write_all(marker);
            let _ = stderr.flush();
            terminate(125)
        }

        fn status_record(nonce: &[u8; 32], tag: u8, payload: &[u8]) -> Vec<u8> {
            let mut record = Vec::with_capacity(27 + nonce.len() + payload.len());
            record.extend_from_slice(b"\0CRUCIBLE-SUPERVISOR-V1\0");
            record.extend_from_slice(nonce);
            record.push(tag);
            record.extend_from_slice(payload);
            record
        }

        fn write_status(record: &[u8]) -> Result<(), ()> {
            let mut stderr = std::io::stderr().lock();
            stderr.write_all(record).map_err(|_| ())?;
            stderr.flush().map_err(|_| ())
        }

        fn read_u64(bytes: &[u8], cursor: &mut usize) -> Option<u64> {
            let end = cursor.checked_add(8)?;
            let field = bytes.get(*cursor..end)?;
            let array: [u8; 8] = field.try_into().ok()?;
            *cursor = end;
            Some(u64::from_be_bytes(array))
        }

        fn decode_arguments(bytes: &[u8]) -> Option<Vec<String>> {
            let magic = b"CRUCIBLE-ARGV-V1\n";
            if !bytes.starts_with(magic) {
                return None;
            }
            let mut cursor = magic.len();
            let count = usize::try_from(read_u64(bytes, &mut cursor)?).ok()?;
            if count > MAX_CLI_ARGUMENTS.saturating_mul(262_144) {
                return None;
            }
            let mut arguments = Vec::new();
            arguments.try_reserve(count).ok()?;
            for _ in 0..count {
                let length = usize::try_from(read_u64(bytes, &mut cursor)?).ok()?;
                let end = cursor.checked_add(length)?;
                let field = bytes.get(cursor..end)?;
                let argument = String::from_utf8(field.to_vec()).ok()?;
                if argument.as_bytes().contains(&0) {
                    return None;
                }
                arguments.push(argument);
                cursor = end;
            }
            if cursor == bytes.len() {
                Some(arguments)
            } else {
                None
            }
        }

        let mut transport = Vec::new();
        if std::io::stdin().take(MAX_LOCAL_ARGUMENT_WIRE_BYTES + 33).read_to_end(
            &mut transport,
        ).is_err() || transport.len() < 32
            || transport.len() as u64 > MAX_LOCAL_ARGUMENT_WIRE_BYTES + 32 {
            fail(b"supervisor-arguments-invalid\n");
        }
        let nonce: [u8; 32] = match transport[0..32].try_into() {
            Ok(value) => value,
            Err(_) => fail(b"supervisor-arguments-invalid\n"),
        };
        let arguments = match decode_arguments(&transport[32..]) {
            Some(value) => value,
            None => fail(b"supervisor-arguments-invalid\n"),
        };
        if rustix::process::set_dumpable_behavior(
            rustix::process::DumpableBehavior::NotDumpable,
        ).is_err() {
            fail(b"supervisor-nondumpable-failed\n");
        }
        let mut child = match Command::new("/crucible-target").args(arguments).stdin(
            Stdio::null(),
        ).stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn() {
            Ok(value) => value,
            Err(_) => fail(b"target-start-failed\n"),
        };
        if write_status(status_record(&nonce, b'B', &[]).as_slice()).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            terminate(125);
        }
        let target_status = match child.wait() {
            Ok(value) => value,
            Err(_) => fail(b"target-wait-failed\n"),
        };
        if let Some(code) = target_status.code() {
            let payload = i64::from(code).to_be_bytes();
            if write_status(status_record(&nonce, b'E', &payload).as_slice()).is_err() {
                terminate(125);
            }
            terminate(code)
        }
        if let Some(signal) = target_status.signal() {
            let core = if target_status.core_dumped() {
                1
            } else {
                0
            };
            let mut payload = Vec::with_capacity(5);
            payload.extend_from_slice(&signal.to_be_bytes());
            payload.push(core);
            if write_status(status_record(&nonce, b'S', payload.as_slice()).as_slice()).is_err() {
                terminate(125);
            }
            terminate(128_i32.saturating_add(signal))
        }
        fail(b"target-status-unavailable\n")
    }
    #[cfg(not(target_os = "linux"))]
    { std::process::exit(125) }
}

// CRUCIBLE-TCB: CLI-HOST-LOCAL-RUN-001
#[verifier::external_body]
fn host_local_run_action(
    action: HostLocalRunAction,
    configuration_path: &str,
    workspace_root: &str,
    plan: &LocalExecutionPlan,
    target_contents: &[u8],
    target_argument_wire: &[u8],
) -> (result: Result<HostLocalRunOutcome, HostLocalRunError>)
    requires
        crucible_cli::local_execution_plan_well_formed_spec(plan@),
        action == HostLocalRunAction::Execute ==> target_contents@.len()
            <= MAX_LOCAL_ARTIFACT_BYTES,
        action == HostLocalRunAction::Execute
            ==> crucible_cli::local_target_argument_wire_shape_spec(target_argument_wire@),
    ensures
        host_local_run_result_shape_spec(action, plan@, &result),
{
    #[cfg(target_os = "linux")]
    {
        use std::io::{Read, Write};
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::process::{CommandExt, ExitStatusExt};
        use std::process::{Command, Stdio};

        fn code_points_to_string(points: &[u32]) -> Result<String, HostLocalRunError> {
            let mut value = String::new();
            for point in points {
                let character = char::from_u32(*point).ok_or(HostLocalRunError::TargetPreparation)?;
                if character == '\0' {
                    return Err(HostLocalRunError::TargetPreparation);
                }
                value.push(character);
            }
            Ok(value)
        }

        fn lexical_absolute(path: &Path) -> Result<PathBuf, HostLocalRunError> {
            let candidate = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir().map_err(
                    |_| HostLocalRunError::UnsafeConfigurationPath,
                )?.join(path)
            };
            let mut normalized = PathBuf::new();
            for component in candidate.components() {
                match component {
                    Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                        normalized.push(component.as_os_str());
                    },
                    Component::CurDir => {},
                    Component::ParentDir => {
                        if !normalized.pop() {
                            return Err(HostLocalRunError::UnsafeConfigurationPath);
                        }
                    },
                }
            }
            if normalized.is_absolute() {
                Ok(normalized)
            } else {
                Err(HostLocalRunError::UnsafeConfigurationPath)
            }
        }

        fn require_safe_directory(path: &Path) -> Result<(), HostLocalRunError> {
            let mut cursor = PathBuf::new();
            for component in path.components() {
                cursor.push(component.as_os_str());
                if matches!(component, Component::Prefix(_) | Component::RootDir) {
                    continue;
                }
                let metadata = std::fs::symlink_metadata(&cursor).map_err(
                    |_| HostLocalRunError::Workspace,
                )?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(HostLocalRunError::Workspace);
                }
            }
            Ok(())
        }

        fn configuration_parent(path: &str) -> Result<PathBuf, HostLocalRunError> {
            let absolute = lexical_absolute(Path::new(path))?;
            let metadata = std::fs::symlink_metadata(&absolute).map_err(
                |_| HostLocalRunError::UnsafeConfigurationPath,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(HostLocalRunError::UnsafeConfigurationPath);
            }
            let parent = absolute.parent().ok_or(HostLocalRunError::UnsafeConfigurationPath)?;
            require_safe_directory(parent).map_err(|_| HostLocalRunError::UnsafeConfigurationPath)?;
            Ok(parent.to_path_buf())
        }

        fn read_target(configuration_path: &str, plan: &LocalExecutionPlan) -> Result<
            (Vec<u8>, String),
            HostLocalRunError,
        > {
            use rustix::fs::{FileType, Mode, OFlags};

            let parent = configuration_parent(configuration_path)?;
            let configured = code_points_to_string(plan.target_command())?;
            let candidate = Path::new(configured.as_str());
            let absolute = if candidate.is_absolute() {
                lexical_absolute(candidate)?
            } else {
                lexical_absolute(parent.join(candidate).as_path())?
            };
            let components: Vec<_> = absolute.components().filter_map(
                |component|
                    match component {
                        Component::Normal(value) => Some(value),
                        _ => None,
                    },
            ).collect();
            if components.is_empty() {
                return Err(HostLocalRunError::TargetPreparation);
            }
            let directory_flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW
                | OFlags::CLOEXEC;
            let mut directory = rustix::fs::open("/", directory_flags, Mode::empty()).map_err(
                |_| HostLocalRunError::TargetPreparation,
            )?;
            let mut final_file = None;
            for (index, component) in components.iter().enumerate() {
                let is_final = index + 1 == components.len();
                let flags = if is_final {
                    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK
                } else {
                    directory_flags
                };
                let opened = rustix::fs::openat(
                    &directory,
                    *component,
                    flags,
                    Mode::empty(),
                ).map_err(|_| HostLocalRunError::TargetPreparation)?;
                let stat = rustix::fs::fstat(&opened).map_err(
                    |_| HostLocalRunError::TargetPreparation,
                )?;
                let file_type = FileType::from_raw_mode(stat.st_mode);
                if is_final {
                    if !file_type.is_file() {
                        return Err(HostLocalRunError::TargetPreparation);
                    }
                    let length = u64::try_from(stat.st_size).map_err(
                        |_| HostLocalRunError::TargetPreparation,
                    )?;
                    if length > MAX_LOCAL_ARTIFACT_BYTES {
                        return Err(HostLocalRunError::TargetPreparation);
                    }
                    final_file = Some(opened);
                } else {
                    if !file_type.is_dir() {
                        return Err(HostLocalRunError::TargetPreparation);
                    }
                    directory = opened;
                }
            }
            let descriptor = final_file.ok_or(HostLocalRunError::TargetPreparation)?;
            let file = std::fs::File::from(descriptor);
            let mut contents = Vec::new();
            file.take(MAX_LOCAL_ARTIFACT_BYTES + 1).read_to_end(&mut contents).map_err(
                |_| HostLocalRunError::TargetPreparation,
            )?;
            if contents.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES {
                return Err(HostLocalRunError::TargetPreparation);
            }
            let provenance = absolute.to_str().ok_or(
                HostLocalRunError::TargetPreparation,
            )?.to_owned();
            Ok((contents, provenance))
        }

        fn capture_stream<R: Read>(mut stream: R, limit: u64) -> Result<
            CapturedOutput,
            HostLocalRunError,
        > {
            let mut retained = Vec::new();
            let mut discarded = 0_u64;
            let mut chunk = [0_u8;16 * 1024];
            loop {
                let count = stream.read(&mut chunk).map_err(|_| HostLocalRunError::Capture)?;
                if count == 0 {
                    break;
                }
                let retained_len = retained.len() as u64;
                let available = limit.saturating_sub(retained_len);
                let keep = usize::try_from(available.min(count as u64)).map_err(
                    |_| HostLocalRunError::Capture,
                )?;
                retained.extend_from_slice(&chunk[0..keep]);
                discarded =
                discarded.checked_add((count - keep) as u64).ok_or(HostLocalRunError::Capture)?;
            }
            Ok(CapturedOutput::new(retained, discarded))
        }

        struct CapturedSupervisorStream {
            target_output: CapturedOutput,
            control_status: Vec<u8>,
            target_started: bool,
            target_termination: Option<LocalTermination>,
        }

        fn capture_supervisor_stream<R: Read>(
            mut stream: R,
            limit: u64,
            nonce: [u8; 32],
        ) -> Result<CapturedSupervisorStream, HostLocalRunError> {
            fn record(nonce: &[u8; 32], tag: u8, payload: &[u8]) -> Vec<u8> {
                let magic = b"\0CRUCIBLE-SUPERVISOR-V1\0";
                let mut value = Vec::with_capacity(magic.len() + nonce.len() + 1 + payload.len());
                value.extend_from_slice(magic);
                value.extend_from_slice(nonce);
                value.push(tag);
                value.extend_from_slice(payload);
                value
            }

            let header = record(&nonce, 0, &[]);
            let header_len = header.len() - 1;
            let retain_cap = usize::try_from(
                limit.checked_add(160).ok_or(
                    HostLocalRunError::Capture,
                )?,
            ).map_err(|_| HostLocalRunError::Capture)?;
            let mut raw_retained = Vec::new();
            let mut overlap = Vec::new();
            let mut total = 0_u64;
            let mut start_range: Option<(u64, u64)> = None;
            let mut end_record: Option<(u64, u64, LocalTermination)> = None;
            let mut chunk = [0_u8;16 * 1024];
            loop {
                let count = stream.read(&mut chunk).map_err(|_| HostLocalRunError::Capture)?;
                if count == 0 {
                    break;
                }
                let keep = retain_cap.saturating_sub(raw_retained.len()).min(count);
                raw_retained.extend_from_slice(&chunk[..keep]);

                let overlap_len = overlap.len();
                let mut scan = overlap;
                scan.extend_from_slice(&chunk[..count]);
                let scan_base = total.checked_sub(overlap_len as u64).ok_or(
                    HostLocalRunError::Capture,
                )?;
                let mut index = 0_usize;
                while index.checked_add(header_len + 1).is_some_and(
                    |end| end <= scan.len(),
                ) {
                    if scan[index..].starts_with(&header[..header_len]) {
                        let tag = scan[index + header_len];
                        let (record_len, termination) = match tag {
                            b'B' => (header_len + 1, None),
                            b'E' if index + header_len + 9 <= scan.len() => {
                                let bytes: [u8; 8] = scan[
                                    index + header_len + 1..index + header_len + 9
                                ].try_into().map_err(|_| HostLocalRunError::Capture)?;
                                (
                                    header_len + 9,
                                    Some(LocalTermination::ExitCode(i64::from_be_bytes(bytes))),
                                )
                            },
                            b'S' if index + header_len + 6 <= scan.len() => {
                                let bytes: [u8; 4] = scan[
                                    index + header_len + 1..index + header_len + 5
                                ].try_into().map_err(|_| HostLocalRunError::Capture)?;
                                let signal = i32::from_be_bytes(bytes);
                                let core = scan[index + header_len + 5];
                                if signal <= 0 || signal >= 128 || core > 1 {
                                    index += 1;
                                    continue;
                                }
                                (
                                    header_len + 6,
                                    Some(LocalTermination::UnixSignal {
                                        signal,
                                        core_dumped: core == 1,
                                    }),
                                )
                            },
                            _ => {
                                index += 1;
                                continue;
                            },
                        };
                        let position = scan_base.checked_add(index as u64).ok_or(
                            HostLocalRunError::Capture,
                        )?;
                        let end = position.checked_add(record_len as u64).ok_or(
                            HostLocalRunError::Capture,
                        )?;
                        if tag == b'B' {
                            if start_range.is_none() {
                                start_range = Some((position, end));
                            }
                        } else if end_record.as_ref().is_none_or(
                            |(previous, _, _)| position > *previous,
                        ) {
                            end_record = termination.map(|value| (position, end, value));
                        }
                        index = index.saturating_add(record_len);
                    } else {
                        index += 1;
                    }
                }
                let overlap_start = scan.len().saturating_sub(79);
                overlap = scan[overlap_start..].to_vec();
                total = total.checked_add(count as u64).ok_or(HostLocalRunError::Capture)?;
            }

            let accepted_end = match (start_range, end_record) {
                (Some((_, start_end)), Some((position, end, termination)))
                    if position >= start_end => Some((position, end, termination)),
                _ => None,
            };
            let mut ranges = Vec::new();
            if let Some(range) = start_range {
                ranges.push(range);
            }
            if let Some((start, end, _)) = &accepted_end {
                ranges.push((*start, *end));
            }
            let removed = ranges.iter().try_fold(0_u64, |sum, (start, end)| {
                sum.checked_add(end - start).ok_or(HostLocalRunError::Capture)
            })?;
            let target_total = total.checked_sub(removed).ok_or(HostLocalRunError::Capture)?;
            let mut retained = Vec::new();
            for (index, byte) in raw_retained.into_iter().enumerate() {
                if retained.len() as u64 >= limit {
                    break;
                }
                let position = index as u64;
                if ranges.iter().any(|(start, end)| *start <= position && position < *end) {
                    continue;
                }
                retained.push(byte);
            }
            let discarded = target_total.checked_sub(retained.len() as u64).ok_or(
                HostLocalRunError::Capture,
            )?;
            let mut control_status = Vec::new();
            if start_range.is_some() {
                control_status.extend_from_slice(record(&nonce, b'B', &[]).as_slice());
            }
            let target_termination = accepted_end.map(|(_, _, value)| value);
            match &target_termination {
                Some(LocalTermination::ExitCode(code)) => control_status.extend_from_slice(
                    record(&nonce, b'E', &code.to_be_bytes()).as_slice(),
                ),
                Some(LocalTermination::UnixSignal { signal, core_dumped }) => {
                    let mut payload = Vec::with_capacity(5);
                    payload.extend_from_slice(&signal.to_be_bytes());
                    payload.push(u8::from(*core_dumped));
                    control_status.extend_from_slice(record(&nonce, b'S', &payload).as_slice());
                },
                Some(LocalTermination::Timeout) | None => {},
                Some(_) => return Err(HostLocalRunError::Capture),
            }
            Ok(CapturedSupervisorStream {
                target_output: CapturedOutput::new(retained, discarded),
                control_status,
                target_started: start_range.is_some(),
                target_termination,
            })
        }

        struct TemporaryControlStatus {
            path: PathBuf,
            file: std::fs::File,
        }

        impl TemporaryControlStatus {
            fn create(directory: &Path, label: &str) -> Result<Self, HostLocalRunError> {
                for sequence in 0..32_u8 {
                    let path = directory.join(
                        format!(
                        ".{label}-{}-{sequence}",
                        std::process::id(),
                    ),
                    );
                    match std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(
                        &path,
                    ) {
                        Ok(file) => {
                            let control = Self { path, file };
                            std::fs::set_permissions(
                                &control.path,
                                std::fs::Permissions::from_mode(0o600),
                            ).map_err(|_| HostLocalRunError::Spawn)?;
                            rustix::io::fcntl_setfd(
                                &control.file,
                                rustix::io::FdFlags::empty(),
                            ).map_err(|_| HostLocalRunError::Spawn)?;
                            return Ok(control);
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
                        Err(_) => return Err(HostLocalRunError::Spawn),
                    }
                }
                Err(HostLocalRunError::Spawn)
            }

            fn descriptor(&self) -> std::os::fd::RawFd {
                use std::os::fd::AsRawFd;
                self.file.as_raw_fd()
            }

            fn read_bounded(&mut self) -> Result<Vec<u8>, HostLocalRunError> {
                use std::io::Seek;
                self.file.seek(std::io::SeekFrom::Start(0)).map_err(
                    |_| HostLocalRunError::Capture,
                )?;
                let mut status = Vec::new();
                (&self.file).take(MAX_LOCAL_CONTROL_STATUS_BYTES + 1).read_to_end(
                    &mut status,
                ).map_err(|_| HostLocalRunError::Capture)?;
                if status.len() as u64 > MAX_LOCAL_CONTROL_STATUS_BYTES {
                    return Err(HostLocalRunError::Capture);
                }
                Ok(status)
            }
        }

        impl Drop for TemporaryControlStatus {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.path);
            }
        }

        fn status_contains(status: &[u8], expected: &[u8]) -> bool {
            status.windows(expected.len()).any(|window| window == expected)
        }

        fn status_has_child(status: &[u8]) -> bool {
            status_contains(status, b"\"child-pid\":")
        }

        fn status_has_exit_code(status: &[u8], code: i32) -> bool {
            let expected = format!("\"exit-code\": {code}");
            status_contains(status, expected.as_bytes())
        }

        fn execution_command(
            plan: &LocalExecutionPlan,
            status_descriptor: std::os::fd::RawFd,
            materialized_target: Option<&str>,
            supervisor_executable: Option<&str>,
        ) -> Result<Command, HostLocalRunError> {
            let mut command = Command::new("/usr/bin/bwrap");
            command.arg("--json-status-fd");
            command.arg(status_descriptor.to_string());
            command.args(
                [
                    "--die-with-parent",
                    "--new-session",
                    "--unshare-all",
                    "--cap-drop",
                    "ALL",
                    "--tmpfs",
                    "/",
                    "--ro-bind",
                    "/usr",
                    "/usr",
                    "--symlink",
                    "usr/bin",
                    "/bin",
                    "--symlink",
                    "usr/sbin",
                    "/sbin",
                    "--symlink",
                    "usr/lib",
                    "/lib",
                    "--symlink",
                    "usr/lib64",
                    "/lib64",
                    "--dev",
                    "/dev",
                    "--tmpfs",
                    "/tmp",
                    "--dir",
                    "/work",
                    "--chdir",
                    "/work",
                    "--clearenv",
                    "--setenv",
                    "HOME",
                    "/work",
                    "--setenv",
                    "TMPDIR",
                    "/tmp",
                    "--setenv",
                    "LC_ALL",
                    "C",
                    "--setenv",
                    "TZ",
                    "UTC",
                ],
            );
            if plan.network_policy() == LocalNetworkPolicy::UnrestrictedHost {
                command.arg("--share-net");
            }
            match (materialized_target, supervisor_executable) {
                (Some(target_path), Some(supervisor_path)) => {
                    command.args(
                        [
                            "--ro-bind",
                            target_path,
                            "/crucible-target",
                            "--ro-bind",
                            supervisor_path,
                            "/crucible-supervisor",
                            "--",
                            "/usr/bin/prlimit",
                        ],
                    );
                    command.arg(format!("--as={}", plan.memory_bytes()));
                    command.arg(format!("--nproc={}", plan.max_processes()));
                    command.arg(format!("--fsize={}", plan.max_stream_bytes()));
                    command.args(
                        ["--", "/crucible-supervisor", "__crucible-internal-local-supervisor-v1"],
                    );
                },
                (None, None) => {
                    command.args(["--", "/usr/bin/prlimit"]);
                    command.arg(format!("--as={}", plan.memory_bytes()));
                    command.arg(format!("--nproc={}", plan.max_processes()));
                    command.arg(format!("--fsize={}", plan.max_stream_bytes()));
                    command.args(["--", "/bin/true"]);
                },
                _ => return Err(HostLocalRunError::TargetPreparation),
            }
            command.stdin(Stdio::null());
            command.process_group(0);
            Ok(command)
        }

        fn read_regular_file(path: &Path) -> Result<Vec<u8>, HostLocalRunError> {
            let metadata = std::fs::symlink_metadata(path).map_err(
                |_| HostLocalRunError::CapabilityUnavailable,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(HostLocalRunError::CapabilityUnavailable);
            }
            let file = std::fs::File::open(path).map_err(
                |_| HostLocalRunError::CapabilityUnavailable,
            )?;
            let mut contents = Vec::new();
            file.take(MAX_LOCAL_ARTIFACT_BYTES + 1).read_to_end(&mut contents).map_err(
                |_| HostLocalRunError::CapabilityUnavailable,
            )?;
            if contents.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES {
                return Err(HostLocalRunError::CapabilityUnavailable);
            }
            Ok(contents)
        }

        fn clean_runtime_text(bytes: Vec<u8>) -> Result<String, HostLocalRunError> {
            let value = String::from_utf8(bytes).map_err(
                |_| HostLocalRunError::CapabilityUnavailable,
            )?;
            let value = value.trim().to_owned();
            if value.is_empty() || value.len() as u64 > MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES
                || value.bytes().any(
                |byte|
                    {
                        let printable_ascii = byte >= 0x20 && byte != 0x7f && byte & 0x80 == 0;
                        !printable_ascii || byte == b'='
                    },
            ) {
                return Err(HostLocalRunError::CapabilityUnavailable);
            }
            Ok(value)
        }

        fn bounded_command_stdout(path: &Path) -> Result<String, HostLocalRunError> {
            let mut child = Command::new(path).arg("--version").stdin(Stdio::null()).stdout(
                Stdio::piped(),
            ).stderr(Stdio::null()).spawn().map_err(|_| HostLocalRunError::CapabilityUnavailable)?;
            let stdout = child.stdout.take().ok_or(HostLocalRunError::CapabilityUnavailable)?;
            let mut bytes = Vec::new();
            stdout.take(MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES + 1).read_to_end(&mut bytes).map_err(
                |_| HostLocalRunError::CapabilityUnavailable,
            )?;
            let status = child.wait().map_err(|_| HostLocalRunError::CapabilityUnavailable)?;
            if !status.success() || bytes.len() as u64 > MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES {
                return Err(HostLocalRunError::CapabilityUnavailable);
            }
            clean_runtime_text(bytes)
        }

        fn probe_capabilities(root: &str, plan: &LocalExecutionPlan) -> Result<
            (Vec<u8>, Option<HostLocalRuntime>),
            HostLocalRunError,
        > {
            let root = lexical_absolute(Path::new(root)).map_err(|_| HostLocalRunError::Workspace)?;
            require_safe_directory(&root)?;
            let state = root.join(".crucible");
            let runs = state.join("runs");
            require_safe_directory(&state)?;
            require_safe_directory(&runs)?;

            let collected = (|| -> Result<HostLocalRuntime, HostLocalRunError>
                {
                    let bubblewrap_path = Path::new("/usr/bin/bwrap");
                    let prlimit_path = Path::new("/usr/bin/prlimit");
                    let harness_path = std::env::current_exe().map_err(
                        |_| HostLocalRunError::CapabilityUnavailable,
                    )?;
                    let harness_contents = read_regular_file(harness_path.as_path())?;
                    let bubblewrap_contents = read_regular_file(bubblewrap_path)?;
                    let prlimit_contents = read_regular_file(prlimit_path)?;
                    let bubblewrap_version = bounded_command_stdout(bubblewrap_path)?;
                    let prlimit_version = bounded_command_stdout(prlimit_path)?;
                    let kernel_file = std::fs::File::open("/proc/sys/kernel/osrelease").map_err(
                        |_| HostLocalRunError::CapabilityUnavailable,
                    )?;
                    let mut kernel_bytes = Vec::new();
                    kernel_file.take(MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES + 1).read_to_end(
                        &mut kernel_bytes,
                    ).map_err(|_| HostLocalRunError::CapabilityUnavailable)?;
                    if kernel_bytes.len() as u64 > MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES {
                        return Err(HostLocalRunError::CapabilityUnavailable);
                    }
                    let kernel_release = clean_runtime_text(kernel_bytes)?;
                    let platform = String::from("linux");
                    let architecture = clean_runtime_text(
                        std::env::consts::ARCH.as_bytes().to_vec(),
                    )?;

                    let mut control = TemporaryControlStatus::create(
                        runs.as_path(),
                        "probe-status",
                    )?;
                    let mut command = execution_command(
                        plan,
                        control.descriptor(),
                        None,
                        None,
                    )?;
                    command.stdout(Stdio::null()).stderr(Stdio::null());
                    let status = command.status().map_err(
                        |_| HostLocalRunError::CapabilityUnavailable,
                    )?;
                    let control_status = control.read_bounded()?;
                    if !status.success() || !status_has_exit_code(control_status.as_slice(), 0) {
                        return Err(HostLocalRunError::CapabilityUnavailable);
                    }
                    Ok(
                        HostLocalRuntime {
                            platform,
                            architecture,
                            kernel_release,
                            bubblewrap_version,
                            prlimit_version,
                            harness_contents,
                            bubblewrap_contents,
                            prlimit_contents,
                        },
                    )
                })();
            let runtime = collected.ok();
            let report = canonical_local_capability_probe_report(plan, runtime.is_some());
            Ok((report, runtime))
        }

        struct TemporaryTarget(PathBuf);

        impl Drop for TemporaryTarget {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }

        fn execute_target(
            root: &str,
            plan: &LocalExecutionPlan,
            target_contents: &[u8],
            target_argument_wire: &[u8],
        ) -> Result<RawLocalExecution, HostLocalRunError> {
            if target_contents.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES || !(
            target_contents.starts_with(b"\x7fELF") || target_contents.starts_with(b"#!")) {
                return Err(HostLocalRunError::TargetPreparation);
            }
            if target_argument_wire.len() as u64 > MAX_LOCAL_ARGUMENT_WIRE_BYTES
                || !target_argument_wire.starts_with(b"CRUCIBLE-ARGV-V1\n") {
                return Err(HostLocalRunError::TargetPreparation);
            }
            let root = lexical_absolute(Path::new(root)).map_err(|_| HostLocalRunError::Workspace)?;
            require_safe_directory(&root)?;
            let state = root.join(".crucible");
            let runs = state.join("runs");
            require_safe_directory(&state)?;
            require_safe_directory(&runs)?;
            let mut temporary = None;
            for sequence in 0..32_u8 {
                let candidate = runs.join(format!(".target-{}-{sequence}", std::process::id()));
                match std::fs::OpenOptions::new().write(true).create_new(true).open(&candidate) {
                    Ok(mut file) => {
                        let target = TemporaryTarget(candidate);
                        std::fs::set_permissions(
                            &target.0,
                            std::fs::Permissions::from_mode(0o700),
                        ).map_err(|_| HostLocalRunError::TargetPreparation)?;
                        file.write_all(target_contents).map_err(
                            |_| HostLocalRunError::TargetPreparation,
                        )?;
                        file.sync_all().map_err(|_| HostLocalRunError::TargetPreparation)?;
                        temporary = Some(target);
                        break;
                    },
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
                    Err(_) => return Err(HostLocalRunError::TargetPreparation),
                }
            }
            let temporary = temporary.ok_or(HostLocalRunError::TargetPreparation)?;
            let target_path = temporary.0.to_str().ok_or(HostLocalRunError::TargetPreparation)?;
            let supervisor_path = std::env::current_exe().map_err(
                |_| HostLocalRunError::TargetPreparation,
            )?;
            let supervisor_path = supervisor_path.to_str().ok_or(
                HostLocalRunError::TargetPreparation,
            )?;
            let mut bwrap_control = TemporaryControlStatus::create(
                runs.as_path(),
                "target-status",
            )?;
            let mut nonce = [0_u8; 32];
            std::fs::File::open("/dev/urandom").and_then(
                |mut source| source.read_exact(&mut nonce),
            ).map_err(|_| HostLocalRunError::TargetPreparation)?;
            let mut command = execution_command(
                plan,
                bwrap_control.descriptor(),
                Some(target_path),
                Some(supervisor_path),
            )?;
            command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
            let started = std::time::Instant::now();
            let timeout = std::time::Duration::from_millis(plan.timeout_ms());
            let deadline = started.checked_add(timeout).ok_or(HostLocalRunError::Spawn)?;
            let mut child = command.spawn().map_err(|_| HostLocalRunError::Spawn)?;
            let group = rustix::process::Pid::from_child(&child);
            let mut transport = match child.stdin.take() {
                Some(stream) => stream,
                None => {
                    let _ = rustix::process::kill_process_group(
                        group,
                        rustix::process::Signal::KILL,
                    );
                    let _ = child.wait();
                    return Err(HostLocalRunError::Spawn);
                },
            };
            let stdout = match child.stdout.take() {
                Some(stream) => stream,
                None => {
                    let _ = rustix::process::kill_process_group(
                        group,
                        rustix::process::Signal::KILL,
                    );
                    let _ = child.wait();
                    return Err(HostLocalRunError::Capture);
                },
            };
            let stderr = match child.stderr.take() {
                Some(stream) => stream,
                None => {
                    let _ = rustix::process::kill_process_group(
                        group,
                        rustix::process::Signal::KILL,
                    );
                    let _ = child.wait();
                    return Err(HostLocalRunError::Capture);
                },
            };
            let stream_limit = plan.max_stream_bytes();
            let stdout_thread = match std::thread::Builder::new().name(
                String::from("crucible-stdout-drain"),
            ).spawn(move || capture_stream(stdout, stream_limit)) {
                Ok(thread) => thread,
                Err(_) => {
                    let _ = rustix::process::kill_process_group(
                        group,
                        rustix::process::Signal::KILL,
                    );
                    let _ = child.wait();
                    return Err(HostLocalRunError::Capture);
                },
            };
            let stderr_thread = match std::thread::Builder::new().name(
                String::from("crucible-stderr-drain"),
            ).spawn(move || capture_supervisor_stream(stderr, stream_limit, nonce)) {
                Ok(thread) => thread,
                Err(_) => {
                    let _ = rustix::process::kill_process_group(
                        group,
                        rustix::process::Signal::KILL,
                    );
                    let _ = child.wait();
                    let _ = stdout_thread.join();
                    return Err(HostLocalRunError::Capture);
                },
            };
            if transport.write_all(&nonce).is_err()
                || transport.write_all(target_argument_wire).is_err()
                || transport.flush().is_err() {
                let _ = rustix::process::kill_process_group(
                    group,
                    rustix::process::Signal::KILL,
                );
                let _ = child.wait();
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Err(HostLocalRunError::Spawn);
            }
            drop(transport);
            let mut timed_out = false;
            let status = loop {
                match child.try_wait() {
                    Ok(Some(status)) => break status,
                    Ok(None) if std::time::Instant::now() >= deadline => {
                        timed_out = true;
                        if rustix::process::kill_process_group(
                            group,
                            rustix::process::Signal::KILL,
                        ).is_err() {
                            let _ = child.kill();
                            let _ = child.wait();
                            let _ = stdout_thread.join();
                            let _ = stderr_thread.join();
                            return Err(HostLocalRunError::Cleanup);
                        }
                        break child.wait().map_err(|_| HostLocalRunError::Cleanup)?;
                    },
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(2)),
                    Err(_) => {
                        let _ = rustix::process::kill_process_group(
                            group,
                            rustix::process::Signal::KILL,
                        );
                        let _ = child.wait();
                        let _ = stdout_thread.join();
                        let _ = stderr_thread.join();
                        return Err(HostLocalRunError::Spawn);
                    },
                }
            };
            let stdout = stdout_thread.join().map_err(|_| HostLocalRunError::Capture)??;
            let supervisor_capture = stderr_thread.join().map_err(
                |_| HostLocalRunError::Capture,
            )??;
            let stderr = supervisor_capture.target_output;
            let elapsed = started.elapsed();
            let wrapper_termination = if timed_out {
                LocalTermination::Timeout
            } else if let Some(code) = status.code() {
                LocalTermination::ExitCode(code as i64)
            } else if let Some(signal) = status.signal() {
                LocalTermination::UnixSignal { signal, core_dumped: status.core_dumped() }
            } else {
                return Err(HostLocalRunError::Spawn);
            };
            let bwrap_status = bwrap_control.read_bounded()?;
            let wrapper_authenticated = if timed_out {
                status_has_child(bwrap_status.as_slice())
            } else if let Some(code) = status.code() {
                status_has_exit_code(bwrap_status.as_slice(), code)
            } else {
                false
            };
            let target_started = supervisor_capture.target_started && wrapper_authenticated;
            let target_termination = if timed_out {
                None
            } else {
                supervisor_capture.target_termination
            };
            let mut control_status = bwrap_status;
            control_status.extend_from_slice(b"\n--crucible-supervisor-status--\n");
            control_status.extend_from_slice(supervisor_capture.control_status.as_slice());
            if control_status.len() as u64 > MAX_LOCAL_CONTROL_STATUS_BYTES {
                return Err(HostLocalRunError::Capture);
            }
            RawLocalExecution::new(
                wrapper_termination,
                target_termination,
                target_started,
                stdout,
                stderr,
                elapsed.as_secs(),
                elapsed.subsec_nanos(),
                control_status,
            ).map_err(|_| HostLocalRunError::Capture)
        }

        match action {
            HostLocalRunAction::ResolveWorkspace => {
                let parent = configuration_parent(configuration_path)?;
                let root = parent.to_str().ok_or(
                    HostLocalRunError::UnsafeConfigurationPath,
                )?.to_owned();
                Ok(HostLocalRunOutcome::WorkspaceRoot(root))
            },
            HostLocalRunAction::ProbeCapabilities => {
                let (report, runtime) = probe_capabilities(workspace_root, plan)?;
                Ok(HostLocalRunOutcome::CapabilityProbe(report, runtime))
            },
            HostLocalRunAction::ReadTarget => {
                let (contents, provenance) = read_target(configuration_path, plan)?;
                Ok(HostLocalRunOutcome::Target(contents, provenance))
            },
            HostLocalRunAction::Execute => Ok(
                HostLocalRunOutcome::Executed(
                    execute_target(workspace_root, plan, target_contents, target_argument_wire)?,
                ),
            ),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (
            action,
            configuration_path,
            workspace_root,
            plan,
            target_contents,
            target_argument_wire,
        );
        Err(HostLocalRunError::CapabilityUnavailable)
    }
}

// CRUCIBLE-TCB: CLI-HOST-RUN-STORE-001
#[verifier::external_body]
#[expect(
    clippy::too_many_arguments,
    reason = "the host persistence boundary receives every immutable evidence reference explicitly"
)]
fn host_run_store_action(
    action: HostRunStoreAction,
    root: &str,
    reservation: Option<&ReservedRun>,
    configuration_source: Option<&ArtifactRef>,
    effective_configuration: Option<&ArtifactRef>,
    configuration_digest: &str,
    capability_manifest: Option<&ArtifactRef>,
    plan: Option<&LocalExecutionPlan>,
    seed: u64,
    target_artifact: Option<&ArtifactRef>,
    target_manifest: Option<&ArtifactRef>,
    observation_artifact: Option<&ArtifactRef>,
    stdout_artifact: Option<&ArtifactRef>,
    stderr_artifact: Option<&ArtifactRef>,
    completion_tag: u16,
    termination_tag: u16,
    failure_kind: &str,
) -> (result: Result<HostRunStoreOutcome, HostRunStoreError>)
    ensures
        host_run_store_result_shape_spec(action, &result),
{
    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostRunStoreError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostRunStoreError::Workspace)?.join(path)
        };
        let mut normalized = PathBuf::new();
        for component in candidate.components() {
            match component {
                Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                    normalized.push(component.as_os_str());
                },
                Component::CurDir => {},
                Component::ParentDir => {
                    if !normalized.pop() {
                        return Err(HostRunStoreError::Workspace);
                    }
                },
            }
        }
        if normalized.is_absolute() {
            Ok(normalized)
        } else {
            Err(HostRunStoreError::Workspace)
        }
    }

    fn require_safe_directory(path: &Path) -> Result<(), HostRunStoreError> {
        let mut cursor = PathBuf::new();
        for component in path.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                |_| HostRunStoreError::Workspace,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(HostRunStoreError::Workspace);
            }
        }
        Ok(())
    }

    fn open_database(root: &str) -> Result<Connection, HostRunStoreError> {
        let root = lexical_absolute(Path::new(root))?;
        require_safe_directory(&root)?;
        let state = root.join(".crucible");
        require_safe_directory(&state)?;
        let database = state.join("database.sqlite");
        let metadata = std::fs::symlink_metadata(&database).map_err(
            |_| HostRunStoreError::Workspace,
        )?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(HostRunStoreError::Workspace);
        }
        let connection = Connection::open(database).map_err(|_| HostRunStoreError::Workspace)?;
        connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(
            |_| HostRunStoreError::Workspace,
        )?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;").map_err(
            |_| HostRunStoreError::Workspace,
        )?;
        let application_id: i64 = connection.query_row(
            "PRAGMA application_id",
            [],
            |row| row.get(0),
        ).map_err(|_| HostRunStoreError::Workspace)?;
        let version: i64 = connection.query_row(
            "PRAGMA user_version",
            [],
            |row| row.get(0),
        ).map_err(|_| HostRunStoreError::Workspace)?;
        let quick: String = connection.query_row(
            "PRAGMA quick_check",
            [],
            |row| row.get(0),
        ).map_err(|_| HostRunStoreError::Workspace)?;
        if application_id != WORKSPACE_APPLICATION_ID || version != WORKSPACE_SCHEMA_VERSION
            || quick != "ok" {
            return Err(HostRunStoreError::Workspace);
        }
        Ok(connection)
    }

    fn next_sequence(transaction: &rusqlite::Transaction<'_>, name: &str) -> Result<
        i64,
        HostRunStoreError,
    > {
        transaction.query_row(
            "INSERT INTO id_sequences(name, next_value) VALUES (?1, 1)
             ON CONFLICT(name) DO UPDATE SET next_value = next_value + 1
             RETURNING next_value",
            [name],
            |row| row.get(0),
        ).map_err(|_| HostRunStoreError::Persist)
    }

    fn admit_persisted_transition(
        transaction: &rusqlite::Transaction<'_>,
        reservation: &ReservedRun,
        transition: RunStoreTransition,
    ) -> Result<RunAttemptStatus, HostRunStoreError> {
        let status: String = transaction.query_row(
            "SELECT status FROM run_attempts WHERE id = ?1 AND run_id = ?2",
            params![reservation.attempt_id().as_str(), reservation.run_id().as_str()],
            |row| row.get(0),
        ).map_err(|_| HostRunStoreError::Conflict)?;
        let current = match status.as_str() {
            "reserved" => RunAttemptStatus::Reserved,
            "target_prepared" => RunAttemptStatus::TargetPrepared,
            "observed" => RunAttemptStatus::Observed,
            "harness_failure" => RunAttemptStatus::HarnessFailure,
            _ => return Err(HostRunStoreError::Conflict),
        };
        admit_run_store_transition(current, transition).map_err(|_| HostRunStoreError::Conflict)
    }

    let mut connection = open_database(root)?;
    let transaction = connection.transaction_with_behavior(
        rusqlite::TransactionBehavior::Immediate,
    ).map_err(|_| HostRunStoreError::Persist)?;
    let outcome = match action {
        HostRunStoreAction::Reserve => {
            let configuration_source = configuration_source.ok_or(HostRunStoreError::Persist)?;
            let effective_configuration = effective_configuration.ok_or(
                HostRunStoreError::Persist,
            )?;
            let capability_manifest = capability_manifest.ok_or(HostRunStoreError::Persist)?;
            let plan = plan.ok_or(HostRunStoreError::Persist)?;
            if configuration_digest.len() != 71 {
                return Err(HostRunStoreError::Persist);
            }
            let run_sequence = next_sequence(&transaction, "run")?;
            let attempt_sequence = next_sequence(&transaction, "run-attempt")?;
            if run_sequence <= 0 || attempt_sequence <= 0 {
                return Err(HostRunStoreError::Persist);
            }
            let run_id = format!("run-{run_sequence:020}");
            let attempt_id = format!("attempt-{attempt_sequence:020}");
            let reservation = ReservedRun::new(run_id.clone(), attempt_id.clone()).map_err(
                |_| HostRunStoreError::Persist,
            )?;
            transaction.execute(
                "INSERT INTO capability_manifests(artifact_id, backend, platform)
                 VALUES (?1, 'linux-bubblewrap-prlimit-v1', 'linux')
                 ON CONFLICT(artifact_id) DO NOTHING",
                [capability_manifest.id.as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let manifest_matches: i64 = transaction.query_row(
                "SELECT COUNT(*) FROM capability_manifests
                 WHERE artifact_id = ?1 AND backend = 'linux-bubblewrap-prlimit-v1'
                   AND platform = 'linux'",
                [capability_manifest.id.as_str()],
                |row| row.get(0),
            ).map_err(|_| HostRunStoreError::Persist)?;
            if manifest_matches != 1 {
                return Err(HostRunStoreError::Conflict);
            }
            transaction.execute(
                "INSERT INTO runs(
                    id, configuration_source_artifact_id, effective_configuration_artifact_id,
                    configuration_digest, target_build_id, capability_manifest_artifact_id, seed
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
                params![
                    run_id,
                    configuration_source.id.as_str(),
                    effective_configuration.id.as_str(),
                    configuration_digest,
                    capability_manifest.id.as_str(),
                    seed.to_string(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let network_policy = match plan.network_policy() {
                LocalNetworkPolicy::None => "none",
                LocalNetworkPolicy::UnrestrictedHost => "unrestricted-host",
                _ => return Err(HostRunStoreError::Persist),
            };
            transaction.execute(
                "INSERT INTO run_effective_controls(
                    run_id, timeout_ms, memory_bytes, max_processes, max_stream_bytes,
                    network_policy, isolation_backend, output_capture_status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                    'linux-bubblewrap-prlimit-v1', 'drain-and-discard')",
                params![
                    reservation.run_id().as_str(),
                    plan.timeout_ms().to_string(),
                    plan.memory_bytes().to_string(),
                    plan.max_processes().to_string(),
                    plan.max_stream_bytes().to_string(),
                    network_policy,
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            transaction.execute(
                "INSERT INTO run_attempts(id, run_id, ordinal, status)
                 VALUES (?1, ?2, 1, 'reserved')",
                params![reservation.attempt_id().as_str(), reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            HostRunStoreOutcome::Reserved(reservation)
        },
        HostRunStoreAction::AttachTarget => {
            let reservation = reservation.ok_or(HostRunStoreError::Persist)?;
            let target_artifact = target_artifact.ok_or(HostRunStoreError::Persist)?;
            let target_manifest = target_manifest.ok_or(HostRunStoreError::Persist)?;
            if admit_persisted_transition(
                &transaction,
                reservation,
                RunStoreTransition::AttachTarget,
            )? != RunAttemptStatus::TargetPrepared {
                return Err(HostRunStoreError::Conflict);
            }
            let target_build_id = format!("target-build-{}", target_manifest.id.as_str());
            transaction.execute(
                "INSERT INTO target_builds(
                    id, target_artifact_id, manifest_artifact_id, identity_digest
                 ) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    target_build_id,
                    target_artifact.id.as_str(),
                    target_manifest.id.as_str(),
                    target_manifest.id.as_str(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let exact_build: i64 = transaction.query_row(
                "SELECT COUNT(*) FROM target_builds
                 WHERE id = ?1 AND target_artifact_id = ?2 AND manifest_artifact_id = ?3
                   AND identity_digest = ?4",
                params![
                    target_build_id,
                    target_artifact.id.as_str(),
                    target_manifest.id.as_str(),
                    target_manifest.id.as_str(),
                ],
                |row| row.get(0),
            ).map_err(|_| HostRunStoreError::Persist)?;
            if exact_build != 1 {
                return Err(HostRunStoreError::Conflict);
            }
            let run_updates = transaction.execute(
                "UPDATE runs SET target_build_id = ?1
                 WHERE id = ?2 AND target_build_id IS NULL",
                params![target_build_id, reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let attempt_updates = transaction.execute(
                "UPDATE run_attempts SET status = 'target_prepared'
                 WHERE id = ?1 AND run_id = ?2 AND status = 'reserved'",
                params![reservation.attempt_id().as_str(), reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            if run_updates != 1 || attempt_updates != 1 {
                return Err(HostRunStoreError::Conflict);
            }
            HostRunStoreOutcome::Updated
        },
        HostRunStoreAction::RecordObservation => {
            let reservation = reservation.ok_or(HostRunStoreError::Persist)?;
            let observation = observation_artifact.ok_or(HostRunStoreError::Persist)?;
            let stdout = stdout_artifact.ok_or(HostRunStoreError::Persist)?;
            let stderr = stderr_artifact.ok_or(HostRunStoreError::Persist)?;
            if completion_tag == 0 || termination_tag == 0 {
                return Err(HostRunStoreError::Persist);
            }
            if admit_persisted_transition(
                &transaction,
                reservation,
                RunStoreTransition::RecordObservation,
            )? != RunAttemptStatus::Observed {
                return Err(HostRunStoreError::Conflict);
            }
            let observation_id = format!("observation-{}", reservation.attempt_id().as_str());
            transaction.execute(
                "INSERT INTO observations(
                    id, attempt_id, observation_artifact_id, stdout_artifact_id,
                    stderr_artifact_id, completion_tag, termination_tag
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    observation_id,
                    reservation.attempt_id().as_str(),
                    observation.id.as_str(),
                    stdout.id.as_str(),
                    stderr.id.as_str(),
                    completion_tag as i64,
                    termination_tag as i64,
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let updates = transaction.execute(
                "UPDATE run_attempts SET status = 'observed'
                 WHERE id = ?1 AND run_id = ?2 AND status = 'target_prepared'",
                params![reservation.attempt_id().as_str(), reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            if updates != 1 {
                return Err(HostRunStoreError::Conflict);
            }
            HostRunStoreOutcome::Updated
        },
        HostRunStoreAction::RecordHarnessFailure => {
            let reservation = reservation.ok_or(HostRunStoreError::Persist)?;
            if failure_kind.is_empty() {
                return Err(HostRunStoreError::Persist);
            }
            if admit_persisted_transition(
                &transaction,
                reservation,
                RunStoreTransition::RecordHarnessFailure,
            )? != RunAttemptStatus::HarnessFailure {
                return Err(HostRunStoreError::Conflict);
            }
            let updates = transaction.execute(
                "UPDATE run_attempts SET status = 'harness_failure'
                 WHERE id = ?1 AND run_id = ?2
                   AND status IN ('reserved','target_prepared')",
                params![reservation.attempt_id().as_str(), reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            if updates != 1 {
                return Err(HostRunStoreError::Conflict);
            }
            transaction.execute(
                "INSERT INTO harness_failures(attempt_id, kind, detail_artifact_id)
                 VALUES (?1, ?2, NULL)",
                params![reservation.attempt_id().as_str(), failure_kind],
            ).map_err(|_| HostRunStoreError::Persist)?;
            HostRunStoreOutcome::Updated
        },
    };
    transaction.commit().map_err(|_| HostRunStoreError::Persist)?;
    connection.close().map_err(|_| HostRunStoreError::Persist)?;
    Ok(outcome)
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

fn complete_workspace_migration(root: &str, action: HostWorkspaceAction) -> (result: Result<
    (),
    InitCommandError,
>) {
    match host_workspace_action(root, action) {
        Ok(HostWorkspaceOutcome::Migrated) | Ok(HostWorkspaceOutcome::Raced) => {},
        Ok(HostWorkspaceOutcome::Snapshot(_))
        | Ok(HostWorkspaceOutcome::Published)
        | Err(HostWorkspaceError::Publish) => return Err(InitCommandError::Publish),
        Err(HostWorkspaceError::UnsafeRoot) => return Err(InitCommandError::UnsafeRoot),
        Err(HostWorkspaceError::Inspect) => return Err(InitCommandError::Inspect),
    }
    let completed = inspect_workspace(root)?;
    match decide_workspace_initialization(&completed) {
        Ok(InitializationDecision::Reuse) => Ok(()),
        Ok(InitializationDecision::Create)
        | Ok(InitializationDecision::MigrateV1)
        | Ok(InitializationDecision::MigrateV2) => Err(InitCommandError::Publish),
        Err(error) => Err(map_policy_error(error)),
    }
}

fn initialize_workspace(root: &str) -> (result: Result<(), InitCommandError>) {
    let initial = inspect_workspace(root)?;
    match decide_workspace_initialization(&initial) {
        Ok(InitializationDecision::Reuse) => Ok(()),
        Err(error) => Err(map_policy_error(error)),
        Ok(InitializationDecision::MigrateV1) => {
            complete_workspace_migration(root, HostWorkspaceAction::MigrateV1)
        },
        Ok(InitializationDecision::MigrateV2) => {
            complete_workspace_migration(root, HostWorkspaceAction::MigrateV2)
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
                Ok(InitializationDecision::Create)
                | Ok(InitializationDecision::MigrateV1)
                | Ok(InitializationDecision::MigrateV2) => Err(InitCommandError::Publish),
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
        Ok(InitializationDecision::MigrateV1) | Ok(InitializationDecision::MigrateV2) => {
            initialize_workspace(root).is_ok()
        },
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
    if stored_artifact_is_exact(&publication.artifact, !source.is_empty(), &snapshot) {
        Ok(())
    } else {
        Err(ArtifactCommandError::Integrity)
    }
}

fn publish_generated_artifact(
    root: &str,
    publication: &PreparedArtifactPublication,
    contents: &[u8],
) -> (result: Result<(), ArtifactCommandError>) {
    publish_artifact(root, "", publication, contents)
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

fn append_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    let mut index = 0;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
        decreases bytes.len() - index,
    {
        output.push(bytes[index]);
        index += 1;
    }
}

fn append_decimal_u64(output: &mut Vec<u8>, mut value: u64) {
    if value == 0 {
        output.push(b'0');
        return;
    }
    let mut reversed = Vec::new();
    while value > 0
        decreases value,
    {
        reversed.push(b'0' + (value % 10) as u8);
        value /= 10;
    }
    let mut index = reversed.len();
    while index > 0
        invariant
            index <= reversed.len(),
        decreases index,
    {
        index -= 1;
        output.push(reversed[index]);
    }
}

fn configuration_error_label(kind: ConfigurationErrorKind) -> (label: Vec<u8>) {
    let literal: &[u8] = match kind {
        ConfigurationErrorKind::SourceByteLimitExceeded => b"SourceByteLimitExceeded",
        ConfigurationErrorKind::YamlSyntax => b"YamlSyntax",
        ConfigurationErrorKind::ExpectedSingleDocument => b"ExpectedSingleDocument",
        ConfigurationErrorKind::ExpectedRootMapping => b"ExpectedRootMapping",
        ConfigurationErrorKind::UnknownField => b"UnknownField",
        ConfigurationErrorKind::MissingRequiredField => b"MissingRequiredField",
        ConfigurationErrorKind::WrongValueKind => b"WrongValueKind",
        ConfigurationErrorKind::UnsupportedSchemaVersion => b"UnsupportedSchemaVersion",
        ConfigurationErrorKind::InvalidLanguageProfile => b"InvalidLanguageProfile",
        ConfigurationErrorKind::InvalidTargetAdapter => b"InvalidTargetAdapter",
        ConfigurationErrorKind::InvalidFieldValue => b"InvalidFieldValue",
        ConfigurationErrorKind::IntegerOutOfRange => b"IntegerOutOfRange",
        ConfigurationErrorKind::DuplicateSequenceValue => b"DuplicateSequenceValue",
        ConfigurationErrorKind::CrossFieldInvariant => b"CrossFieldInvariant",
        ConfigurationErrorKind::TypedNodeLimitExceeded => b"TypedNodeLimitExceeded",
        ConfigurationErrorKind::CanonicalByteLimitExceeded => b"CanonicalByteLimitExceeded",
        ConfigurationErrorKind::DepthLimitExceeded => b"DepthLimitExceeded",
        ConfigurationErrorKind::WorkLimitExceeded => b"WorkLimitExceeded",
        ConfigurationErrorKind::HashInputTooLong => b"HashInputTooLong",
        ConfigurationErrorKind::InternalInvariantViolation => b"InternalInvariantViolation",
        _ => b"UnknownConfigurationError",
    };
    vstd::slice::slice_to_vec(literal)
}

fn configuration_error_output(error: &ConfigurationError) -> (output: Vec<u8>) {
    let mut output = vstd::slice::slice_to_vec(b"crucible config: ");
    let label = configuration_error_label(error.kind());
    append_bytes(&mut output, label.as_slice());
    append_bytes(&mut output, b" at byte ");
    append_decimal_u64(&mut output, error.byte_offset());
    output.push(b'\n');
    output
}

fn configuration_digest_output(digest: crucible_core::Sha256Digest) -> (output: Vec<u8>) {
    let mut output = vstd::slice::slice_to_vec(b"sha256:");
    let hex = digest.to_hex();
    let bytes = hex.as_str().as_bytes_vec();
    append_bytes(&mut output, bytes.as_slice());
    output.push(b'\n');
    output
}

fn complete_configuration_error(error: ConfigurationError) {
    let output = configuration_error_output(&error);
    host_complete(false, output.as_slice());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunCommandError {
    Workspace,
    Artifact,
    TargetPreparation,
    CapabilityUnavailable,
    Execution,
    Persistence,
    Observation,
}

fn prepare_run_artifact(contents: &[u8]) -> (result: Result<
    PreparedArtifactPublication,
    RunCommandError,
>) {
    match prepare_artifact_publication(contents) {
        Ok(publication) => Ok(publication),
        Err(_) => Err(RunCommandError::Artifact),
    }
}

fn publish_run_generated_artifact(root: &str, contents: &[u8]) -> (result: Result<
    PreparedArtifactPublication,
    RunCommandError,
>) {
    let publication = prepare_run_artifact(contents)?;
    match publish_generated_artifact(root, &publication, contents) {
        Ok(()) => {},
        Err(_host_error) => return Err(RunCommandError::Artifact),
    }
    Ok(publication)
}

fn reserve_run(
    root: &str,
    configuration_source: &ArtifactRef,
    effective_configuration: &ArtifactRef,
    configuration_digest: &str,
    capability_manifest: &ArtifactRef,
    plan: &LocalExecutionPlan,
    seed: u64,
) -> (result: Result<ReservedRun, RunCommandError>)
    requires
        crucible_cli::local_execution_plan_well_formed_spec(plan@),
    ensures
        result is Ok ==> crucible_cli::reserved_run_well_formed_spec(result.unwrap()@),
{
    match host_run_store_action(
        HostRunStoreAction::Reserve,
        root,
        None,
        Some(configuration_source),
        Some(effective_configuration),
        configuration_digest,
        Some(capability_manifest),
        Some(plan),
        seed,
        None,
        None,
        None,
        None,
        None,
        0,
        0,
        "",
    ) {
        Ok(HostRunStoreOutcome::Reserved(reservation)) => Ok(reservation),
        Ok(HostRunStoreOutcome::Updated) | Err(_) => Err(RunCommandError::Persistence),
    }
}

fn attach_run_target(
    root: &str,
    reservation: &ReservedRun,
    target: &ArtifactRef,
    manifest: &ArtifactRef,
) -> (result: Result<(), RunCommandError>) {
    match host_run_store_action(
        HostRunStoreAction::AttachTarget,
        root,
        Some(reservation),
        None,
        None,
        "",
        None,
        None,
        0,
        Some(target),
        Some(manifest),
        None,
        None,
        None,
        0,
        0,
        "",
    ) {
        Ok(HostRunStoreOutcome::Updated) => Ok(()),
        Ok(HostRunStoreOutcome::Reserved(_)) | Err(_) => Err(RunCommandError::Persistence),
    }
}

fn record_run_harness_failure(root: &str, reservation: &ReservedRun, kind: &str) -> (result: Result<
    (),
    RunCommandError,
>) {
    match host_run_store_action(
        HostRunStoreAction::RecordHarnessFailure,
        root,
        Some(reservation),
        None,
        None,
        "",
        None,
        None,
        0,
        None,
        None,
        None,
        None,
        None,
        0,
        0,
        kind,
    ) {
        Ok(HostRunStoreOutcome::Updated) => Ok(()),
        Ok(HostRunStoreOutcome::Reserved(_)) | Err(_) => Err(RunCommandError::Persistence),
    }
}

fn record_run_observation(
    root: &str,
    reservation: &ReservedRun,
    observation: &ArtifactRef,
    stdout: &ArtifactRef,
    stderr: &ArtifactRef,
    completion_tag: u16,
    termination_tag: u16,
) -> (result: Result<(), RunCommandError>) {
    match host_run_store_action(
        HostRunStoreAction::RecordObservation,
        root,
        Some(reservation),
        None,
        None,
        "",
        None,
        None,
        0,
        None,
        None,
        Some(observation),
        Some(stdout),
        Some(stderr),
        completion_tag,
        termination_tag,
        "",
    ) {
        Ok(HostRunStoreOutcome::Updated) => Ok(()),
        Ok(HostRunStoreOutcome::Reserved(_)) | Err(_) => Err(RunCommandError::Persistence),
    }
}

fn run_id_output(reservation: &ReservedRun) -> (output: Vec<u8>) {
    let bytes = reservation.run_id().as_str().as_bytes_vec();
    let mut output = Vec::new();
    append_bytes(&mut output, bytes.as_slice());
    output.push(b'\n');
    output
}

fn complete_run_error(error: RunCommandError) {
    let message: &[u8] = match error {
        RunCommandError::Workspace => {
            b"crucible run: workspace is missing, incompatible, or unsafe\n"
        },
        RunCommandError::Artifact => b"crucible run: artifact publication failed\n",
        RunCommandError::TargetPreparation => b"crucible run: target preparation failed\n",
        RunCommandError::CapabilityUnavailable => {
            b"crucible run: required execution capability is unavailable\n"
        },
        RunCommandError::Execution => b"crucible run: target execution failed\n",
        RunCommandError::Persistence => b"crucible run: evidence persistence failed\n",
        RunCommandError::Observation => b"crucible run: observation construction failed\n",
    };
    host_complete(false, message)
}

fn persist_failure_then_complete(
    root: &str,
    reservation: &ReservedRun,
    kind: &str,
    error: RunCommandError,
) {
    if record_run_harness_failure(root, reservation, kind).is_err() {
        complete_run_error(RunCommandError::Persistence);
    } else {
        complete_run_error(error);
    }
}

fn run_local_configuration(path: &str) {
    let contents = match host_read_configuration(path) {
        Ok(value) => value,
        Err(HostConfigError::UnsafeSource) => {
            host_complete(false, b"crucible run: unsafe configuration source\n");
            return;
        },
        Err(HostConfigError::TooLarge) => {
            host_complete(false, b"crucible run: configuration exceeds the source limit\n");
            return;
        },
        Err(HostConfigError::Read) => {
            host_complete(false, b"crucible run: could not read configuration\n");
            return;
        },
        #[cfg(not(unix))]
        Err(HostConfigError::UnsupportedPlatform) => {
            host_complete(false, b"crucible run: configuration platform unsupported\n");
            return;
        },
    };
    let validated = match validate_configuration(
        contents.as_slice(),
        canonical_configuration_limits(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_configuration_error(error);
            return;
        },
    };
    let plan = match prepare_local_execution(validated.execution()) {
        Ok(plan) => plan,
        Err(LocalRunPlanError::RequiredCapabilityUnavailable { .. }) => {
            complete_run_error(RunCommandError::CapabilityUnavailable);
            return;
        },
        Err(LocalRunPlanError::UnsupportedPlatform) => {
            complete_run_error(RunCommandError::CapabilityUnavailable);
            return;
        },
        Err(LocalRunPlanError::ArithmeticOverflow)
        | Err(LocalRunPlanError::OutputLimitTooLarge)
        | Err(LocalRunPlanError::UnsupportedStorageLayout) => {
            complete_run_error(RunCommandError::Execution);
            return;
        },
        Err(_) => {
            complete_run_error(RunCommandError::Execution);
            return;
        },
    };
    let target_argument_wire = match encode_local_target_arguments(&plan) {
        Ok(value) => value,
        Err(_) => {
            complete_run_error(RunCommandError::Execution);
            return;
        },
    };
    let root = match host_local_run_action(
        HostLocalRunAction::ResolveWorkspace,
        path,
        "",
        &plan,
        &[],
        &[],
    ) {
        Ok(HostLocalRunOutcome::WorkspaceRoot(root)) => root,
        Ok(HostLocalRunOutcome::CapabilityProbe(_, _))
        | Ok(HostLocalRunOutcome::Target(_, _))
        | Ok(HostLocalRunOutcome::Executed(_))
        | Err(_) => {
            complete_run_error(RunCommandError::Workspace);
            return;
        },
    };
    if !artifact_workspace_is_ready(root.as_str()) {
        complete_run_error(RunCommandError::Workspace);
        return;
    }
    let (probe_report, host_runtime) = match host_local_run_action(
        HostLocalRunAction::ProbeCapabilities,
        path,
        root.as_str(),
        &plan,
        &[],
        &[],
    ) {
        Ok(HostLocalRunOutcome::CapabilityProbe(report, runtime)) => (report, runtime),
        Ok(HostLocalRunOutcome::WorkspaceRoot(_))
        | Ok(HostLocalRunOutcome::Target(_, _))
        | Ok(HostLocalRunOutcome::Executed(_))
        | Err(_) => {
            complete_run_error(RunCommandError::CapabilityUnavailable);
            return;
        },
    };
    let probe = match validate_local_capability_probe(&plan, probe_report) {
        Ok(value) => value,
        Err(_) => {
            complete_run_error(RunCommandError::CapabilityUnavailable);
            return;
        },
    };
    let probe_publication = match publish_run_generated_artifact(root.as_str(), probe.report()) {
        Ok(value) => value,
        Err(error) => {
            complete_run_error(error);
            return;
        },
    };
    let source_publication = match prepare_run_artifact(contents.as_slice()) {
        Ok(value) => value,
        Err(error) => {
            complete_run_error(error);
            return;
        },
    };
    if publish_artifact(root.as_str(), path, &source_publication, contents.as_slice()).is_err() {
        complete_run_error(RunCommandError::Artifact);
        return;
    }
    let effective_publication = match publish_run_generated_artifact(
        root.as_str(),
        validated.canonical_bytes(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_run_error(error);
            return;
        },
    };
    let capability_bytes = local_capability_manifest(&plan, &probe, &probe_publication.artifact);
    let capability_publication = match publish_run_generated_artifact(
        root.as_str(),
        capability_bytes.as_slice(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_run_error(error);
            return;
        },
    };
    let configuration_identity = ContentDigest::Sha256(validated.digest()).into_artifact_id();
    let reservation = match reserve_run(
        root.as_str(),
        &source_publication.artifact,
        &effective_publication.artifact,
        configuration_identity.as_str(),
        &capability_publication.artifact,
        &plan,
        validated.execution().campaign_seed(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_run_error(error);
            return;
        },
    };
    if !probe.available() {
        persist_failure_then_complete(
            root.as_str(),
            &reservation,
            "CapabilityUnavailable",
            RunCommandError::CapabilityUnavailable,
        );
        return;
    }
    let host_runtime = match host_runtime {
        Some(value) => value,
        None => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "CapabilityProbeMismatch",
                RunCommandError::CapabilityUnavailable,
            );
            return;
        },
    };
    let harness_publication = match publish_run_generated_artifact(
        root.as_str(),
        host_runtime.harness_contents.as_slice(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    let bubblewrap_publication = match publish_run_generated_artifact(
        root.as_str(),
        host_runtime.bubblewrap_contents.as_slice(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    let prlimit_publication = match publish_run_generated_artifact(
        root.as_str(),
        host_runtime.prlimit_contents.as_slice(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    let runtime_identity = match LocalRuntimeIdentity::new(
        host_runtime.platform,
        host_runtime.architecture,
        host_runtime.kernel_release,
        host_runtime.bubblewrap_version,
        host_runtime.prlimit_version,
        harness_publication.artifact,
        bubblewrap_publication.artifact,
        prlimit_publication.artifact,
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "RuntimeIdentity",
                RunCommandError::Execution,
            );
            return;
        },
    };
    let (target_contents, target_provenance) = match host_local_run_action(
        HostLocalRunAction::ReadTarget,
        path,
        root.as_str(),
        &plan,
        &[],
        &[],
    ) {
        Ok(HostLocalRunOutcome::Target(contents, provenance)) => (contents, provenance),
        Ok(HostLocalRunOutcome::WorkspaceRoot(_))
        | Ok(HostLocalRunOutcome::CapabilityProbe(_, _))
        | Ok(HostLocalRunOutcome::Executed(_))
        | Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "TargetPreparation",
                RunCommandError::TargetPreparation,
            );
            return;
        },
    };
    let target_publication = match prepare_run_artifact(target_contents.as_slice()) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "TargetPreparation",
                RunCommandError::TargetPreparation,
            );
            return;
        },
    };
    if publish_artifact(
        root.as_str(),
        target_provenance.as_str(),
        &target_publication,
        target_contents.as_slice(),
    ).is_err() {
        persist_failure_then_complete(
            root.as_str(),
            &reservation,
            "EvidencePersistence",
            RunCommandError::Artifact,
        );
        return;
    }
    assert(crucible_cli::local_runtime_identity_well_formed_spec(runtime_identity@));
    let target_manifest_bytes = target_build_manifest(
        &target_publication.artifact,
        &runtime_identity,
    );
    let target_manifest_publication = match publish_run_generated_artifact(
        root.as_str(),
        target_manifest_bytes.as_slice(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    if attach_run_target(
        root.as_str(),
        &reservation,
        &target_publication.artifact,
        &target_manifest_publication.artifact,
    ).is_err() {
        persist_failure_then_complete(
            root.as_str(),
            &reservation,
            "EvidencePersistence",
            RunCommandError::Persistence,
        );
        return;
    }
    let raw_execution = match host_local_run_action(
        HostLocalRunAction::Execute,
        path,
        root.as_str(),
        &plan,
        target_contents.as_slice(),
        target_argument_wire.as_slice(),
    ) {
        Ok(HostLocalRunOutcome::Executed(raw)) => raw,
        Ok(HostLocalRunOutcome::WorkspaceRoot(_))
        | Ok(HostLocalRunOutcome::CapabilityProbe(_, _))
        | Ok(HostLocalRunOutcome::Target(_, _)) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "ExecutionBoundary",
                RunCommandError::Execution,
            );
            return;
        },
        Err(HostLocalRunError::TargetPreparation) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "TargetPreparation",
                RunCommandError::TargetPreparation,
            );
            return;
        },
        Err(HostLocalRunError::CapabilityUnavailable) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "CapabilityUnavailable",
                RunCommandError::CapabilityUnavailable,
            );
            return;
        },
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "ExecutionBoundary",
                RunCommandError::Execution,
            );
            return;
        },
    };
    let evidence = match classify_raw_local_execution(raw_execution) {
        Ok(value) => value,
        Err(LocalExecutionClassificationError::TargetDidNotStart) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "TargetPreparation",
                RunCommandError::TargetPreparation,
            );
            return;
        },
        Err(LocalExecutionClassificationError::StatusMismatch)
        | Err(LocalExecutionClassificationError::InvalidEvidence)
        | Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "ExecutionBoundary",
                RunCommandError::Execution,
            );
            return;
        },
    };
    let stdout_publication = match publish_run_generated_artifact(
        root.as_str(),
        evidence.stdout().retained(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    let stderr_publication = match publish_run_generated_artifact(
        root.as_str(),
        evidence.stderr().retained(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    let observation = match build_local_raw_observation(
        reservation.run_id().clone(),
        reservation.attempt_id().clone(),
        &evidence,
        stdout_publication.artifact,
        stderr_publication.artifact,
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "ObservationConstruction",
                RunCommandError::Observation,
            );
            return;
        },
    };
    let completion_tag = observation.observation().outcome().completion().stable_tag();
    let termination_tag = match observation.observation().outcome().termination() {
        Some(termination) => termination.stable_tag(),
        None => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "ObservationConstruction",
                RunCommandError::Observation,
            );
            return;
        },
    };
    let encoded_observation = match encode_raw_observation(
        &observation,
        crucible_core::MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "ObservationEncoding",
                RunCommandError::Observation,
            );
            return;
        },
    };
    let observation_publication = match publish_run_generated_artifact(
        root.as_str(),
        encoded_observation.as_slice(),
    ) {
        Ok(value) => value,
        Err(_) => {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Artifact,
            );
            return;
        },
    };
    let raw = observation.observation();
    if record_run_observation(
        root.as_str(),
        &reservation,
        &observation_publication.artifact,
        raw.stdout().artifact(),
        raw.stderr().artifact(),
        completion_tag,
        termination_tag,
    ).is_err() {
        persist_failure_then_complete(
            root.as_str(),
            &reservation,
            "EvidencePersistence",
            RunCommandError::Persistence,
        );
        return;
    }
    let output = run_id_output(&reservation);
    host_complete(true, output.as_slice());
}

fn run_configuration(path: &str, canonicalize: bool) {
    let contents = match host_read_configuration(path) {
        Ok(value) => value,
        Err(HostConfigError::UnsafeSource) => {
            host_complete(false, b"crucible config: unsafe configuration source\n");
            return;
        },
        Err(HostConfigError::TooLarge) => {
            host_complete(false, b"crucible config: SourceByteLimitExceeded at byte 16777216\n");
            return;
        },
        Err(HostConfigError::Read) => {
            host_complete(false, b"crucible config: could not read configuration\n");
            return;
        },
        #[cfg(not(unix))]
        Err(HostConfigError::UnsupportedPlatform) => {
            host_complete(false, b"crucible config: configuration platform unsupported\n");
            return;
        },
    };
    let validated = match validate_configuration(
        contents.as_slice(),
        canonical_configuration_limits(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_configuration_error(error);
            return;
        },
    };
    if canonicalize {
        host_complete(true, validated.canonical_bytes());
    } else {
        let output = configuration_digest_output(validated.digest());
        host_complete(true, output.as_slice());
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
                b"usage: crucible init [path]\n       crucible run <configuration>\n       crucible artifact import <file> [workspace]\n       crucible artifact verify <artifact-id> [workspace]\n       crucible config validate <file>\n       crucible config canonicalize <file>\n",
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
                b"usage: crucible init [path]\n       crucible run <configuration>\n       crucible artifact import <file> [workspace]\n       crucible artifact verify <artifact-id> [workspace]\n       crucible config validate <file>\n       crucible config canonicalize <file>\n",
            );
            return;
        },
    };
    match action {
        CliAction::InternalLocalSupervisor => host_internal_local_supervisor(),
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
        CliAction::Run(path) => run_local_configuration(path.as_str()),
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
        CliAction::ConfigValidate(path) => run_configuration(path.as_str(), false),
        CliAction::ConfigCanonicalize(path) => run_configuration(path.as_str(), true),
    }
}

} // verus!
