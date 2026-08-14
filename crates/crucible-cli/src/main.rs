#![forbid(unsafe_code)]

use crucible_cli::{
    admit_run_store_transition, artifact_imports_table_sql, artifact_migration_checksum,
    artifact_migration_name, artifact_migration_sql, artifacts_table_sql,
    authenticate_artifact_contents, authenticate_artifact_preview, build_local_raw_observation,
    canonical_configuration_limits, canonical_local_capability_probe_report,
    classify_raw_local_execution, database_snapshot_is_exact, database_snapshot_is_exact_v1,
    database_snapshot_is_exact_v2, database_snapshot_is_exact_v3, decide_workspace_initialization,
    domain_migration_checksum, domain_migration_name, domain_migration_sql,
    encode_local_target_arguments, evaluate_process_exit_oracle,
    inspection_observation_codec_limits, local_capability_manifest, metadata_table_sql,
    migration_checksum, migration_table_sql, object_address_for_artifact,
    object_address_matches_id, parse_cli_args, prepare_artifact_publication,
    prepare_local_execution, render_run_inspection_report, run_migration_checksum,
    run_migration_name, run_migration_sql, stored_artifact_is_exact, target_build_manifest,
    validate_configuration, validate_local_capability_probe, validate_run_inspection,
    ArtifactStoreError, CapturedOutput, CliAction, CliParseError, ConfigurationError,
    ConfigurationErrorKind, DatabaseSnapshot, InitializationDecision, InitializationError,
    InspectionArtifactError, InspectionControls, InspectionHarnessFailure, InspectionObservation,
    InspectionPreviews, InspectionReportError, InspectionStatus, InspectionTarget,
    InspectionValidationError, LocalExecutionClassificationError, LocalExecutionPlan,
    LocalNetworkPolicy, LocalOracleVerdict, LocalRunPlanError, LocalRuntimeIdentity,
    LocalTermination, MigrationRecord, ObjectAddress, PathKind, PreparedArtifactPublication,
    RawLocalExecution, ReportFormat, ReservedRun, RunAttemptStatus, RunInspectionSnapshot,
    RunStoreTransition, StoredArtifactSnapshot, WorkspaceMetadata, WorkspaceSnapshot,
    MAX_CLI_ARGUMENTS, MAX_CLI_ARGUMENT_BYTES, MAX_CONFIGURATION_SOURCE_BYTES,
    MAX_INSPECTION_OBSERVATION_BYTES, MAX_LOCAL_ARGUMENT_WIRE_BYTES, MAX_LOCAL_ARTIFACT_BYTES,
    MAX_LOCAL_CONTROL_STATUS_BYTES, MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES,
    WORKSPACE_APPLICATION_ID, WORKSPACE_SCHEMA_VERSION,
};
use crucible_core::{
    decode_raw_observation, derive_replay_seeds, encode_raw_observation, ArtifactId, ArtifactRef,
    ContentDigest, PersistenceRetentionPolicy, MAX_GC_CANDIDATES, MAX_PERSISTENCE_BATCH_BYTES,
    MAX_PERSISTENCE_BATCH_ITEMS,
};
use rusqlite::limits::Limit;
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
    MigrateV3,
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
enum HostStorageAction {
    Check,
    Collect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostStorageError {
    UnsafeWorkspace,
    Integrity,
    WorkLimit,
    ActiveLease,
    Persistence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StorageMaintenanceReport {
    verified: u64,
    orphaned: u64,
    temporary: u64,
    collected: u64,
    preserved: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostInspectionError {
    Workspace,
    NotFound,
    InvalidEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectionCommandError {
    Workspace,
    NotFound,
    InvalidEvidence,
    ArtifactIntegrity,
    Observation,
    Report,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostDomainReadAction {
    Findings,
    Report,
    Capabilities,
    Proof,
    Tcb,
    Plugins,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostDomainReadError {
    Workspace,
    NotFound,
    InvalidEvidence,
    OutputLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostFindingAction {
    Replay,
    Minimize,
    RegisterPatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostFindingError {
    Workspace,
    NotFound,
    Integrity,
    Execution,
    NoMinimizableInput,
    Persist,
    VerificationInconclusive,
    OutputLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostLogAction {
    Initialize,
    Event,
}

const MAX_DOMAIN_REPORT_BYTES: usize = 1_048_576;

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
    RecordBuild,
    Reserve,
    AttachTarget,
    AggregateSuccess,
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
        let domain_sql = String::from_utf8(domain_migration_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let domain_name = String::from_utf8(domain_migration_name()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let domain_checksum = String::from_utf8(domain_migration_checksum()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let transaction = connection.transaction().map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute_batch(
            &format!(
                "{migrations_sql};{metadata_sql};{artifacts_sql};{imports_sql};{run_sql};{domain_sql}"
            ),
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
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (?1, ?2, ?3)",
            params![4, domain_name, domain_checksum],
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
        let domain_sql = String::from_utf8(domain_migration_sql()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let domain_name = String::from_utf8(domain_migration_name()).map_err(
            |_| HostWorkspaceError::Publish,
        )?;
        let domain_checksum = String::from_utf8(domain_migration_checksum()).map_err(
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
        if current_version != expected_version || (expected_version != 1 && expected_version != 2
            && expected_version != 3) {
            return Err(HostWorkspaceError::Publish);
        }
        if current_version == 1 {
            transaction.execute_batch(&migration_sql).map_err(|_| HostWorkspaceError::Publish)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, name, checksum) VALUES (2, ?1, ?2)",
                params![name, checksum],
            ).map_err(|_| HostWorkspaceError::Publish)?;
        }
        if current_version <= 2 {
            transaction.execute_batch(&run_sql).map_err(|_| HostWorkspaceError::Publish)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, name, checksum) VALUES (3, ?1, ?2)",
                params![run_name, run_checksum],
            ).map_err(|_| HostWorkspaceError::Publish)?;
        }
        transaction.execute_batch(&domain_sql).map_err(|_| HostWorkspaceError::Publish)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (4, ?1, ?2)",
            params![domain_name, domain_checksum],
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
        HostWorkspaceAction::MigrateV1
        | HostWorkspaceAction::MigrateV2
        | HostWorkspaceAction::MigrateV3 => {
            let expected_version = match action {
                HostWorkspaceAction::MigrateV1 => 1,
                HostWorkspaceAction::MigrateV2 => 2,
                HostWorkspaceAction::MigrateV3 => 3,
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
                Some(database) if expected_version == 3 && database_snapshot_is_exact_v3(
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
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(
                |_| HostArtifactError::Publish,
            )?.as_secs();
            let now = i64::try_from(now).map_err(|_| HostArtifactError::Publish)?;
            let lease_id =
                format!(
                "lease-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(
                    |_| HostArtifactError::Publish,
                )?.as_nanos(),
            );
            let owner_identity = format!("artifact-publication:{}", artifact.id.as_str());
            let lease_transaction = connection.transaction_with_behavior(
                rusqlite::TransactionBehavior::Immediate,
            ).map_err(|_| HostArtifactError::Publish)?;
            lease_transaction.execute(
                "INSERT INTO storage_generations(id, status, created_epoch)
                 SELECT COALESCE((SELECT MAX(id) FROM storage_generations), 0) + 1, 'open', ?1
                 WHERE NOT EXISTS(SELECT 1 FROM storage_generations WHERE status = 'open')",
                [now],
            ).map_err(|_| HostArtifactError::Publish)?;
            let generation_id: i64 = lease_transaction.query_row(
                "SELECT id FROM storage_generations WHERE status = 'open' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            ).map_err(|_| HostArtifactError::Publish)?;
            lease_transaction.execute(
                "INSERT INTO storage_leases(
                    id, generation_id, artifact_id, owner_identity, status, expires_epoch
                 ) VALUES (?1, ?2, NULL, ?3, 'active', ?4)",
                params![lease_id, generation_id, owner_identity, now.saturating_add(300)],
            ).map_err(|_| HostArtifactError::Publish)?;
            lease_transaction.commit().map_err(|_| HostArtifactError::Publish)?;

            publish_object(&root, address, contents)?;

            let transaction = connection.transaction_with_behavior(
                rusqlite::TransactionBehavior::Immediate,
            ).map_err(|_| HostArtifactError::Publish)?;
            let active_lease: i64 = transaction.query_row(
                "SELECT COUNT(*) FROM storage_leases
                 WHERE id = ?1 AND generation_id = ?2 AND owner_identity = ?3 AND status = 'active'",
                params![lease_id, generation_id, owner_identity],
                |row| row.get(0),
            ).map_err(|_| HostArtifactError::Publish)?;
            if active_lease != 1 {
                return Err(HostArtifactError::Publish);
            }
            let item_count = if subject.is_empty() {
                1_u64
            } else {
                2_u64
            };
            let encoded_bytes = (artifact.id.as_str().len() as u64).checked_add(
                address.object_name.len() as u64,
            ).and_then(|value| value.checked_add(subject.len() as u64)).and_then(
                |value| value.checked_add(32),
            ).ok_or(HostArtifactError::Publish)?;
            if item_count > MAX_PERSISTENCE_BATCH_ITEMS || encoded_bytes
                > MAX_PERSISTENCE_BATCH_BYTES {
                return Err(HostArtifactError::Publish);
            }
            let batch_id = format!("batch-{lease_id}");
            transaction.execute(
                "INSERT INTO persistence_batches(
                    id, campaign_id, retention_policy, status, item_count,
                    encoded_bytes, generation_id
                 ) VALUES (?1, NULL, 'retain-every-run', 'open', ?2, ?3, ?4)",
                params![batch_id, item_count as i64, encoded_bytes as i64, generation_id],
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
            transaction.execute(
                "UPDATE persistence_batches SET status = 'committing'
                 WHERE id = ?1 AND status = 'open'",
                [batch_id.as_str()],
            ).map_err(|_| HostArtifactError::Publish)?;
            transaction.execute(
                "UPDATE storage_leases SET artifact_id = ?1, status = 'released'
                 WHERE id = ?2 AND status = 'active'",
                params![artifact.id.as_str(), lease_id],
            ).map_err(|_| HostArtifactError::Publish)?;
            transaction.execute(
                "UPDATE persistence_batches SET status = 'committed'
                 WHERE id = ?1 AND status = 'committing'",
                [batch_id.as_str()],
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

// CRUCIBLE-TCB: CLI-HOST-STORAGE-001
#[verifier::external_body]
fn host_storage_maintenance(root: &str, action: HostStorageAction) -> (result: Result<
    StorageMaintenanceReport,
    HostStorageError,
>) {
    use std::io::Read;

    struct ObjectEntry {
        path: PathBuf,
        artifact_id: Option<String>,
    }

    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostStorageError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostStorageError::UnsafeWorkspace)?.join(path)
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
                        return Err(HostStorageError::UnsafeWorkspace);
                    }
                },
            }
        }
        if normalized.is_absolute() {
            Ok(normalized)
        } else {
            Err(HostStorageError::UnsafeWorkspace)
        }
    }

    fn require_directory(path: &Path) -> Result<(), HostStorageError> {
        let metadata = std::fs::symlink_metadata(path).map_err(
            |_| HostStorageError::UnsafeWorkspace,
        )?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            Err(HostStorageError::UnsafeWorkspace)
        } else {
            Ok(())
        }
    }

    fn safe_workspace(root: &Path) -> Result<PathBuf, HostStorageError> {
        let absolute = lexical_absolute(root)?;
        let mut cursor = PathBuf::new();
        for component in absolute.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            require_directory(&cursor)?;
        }
        let canonical = std::fs::canonicalize(&absolute).map_err(
            |_| HostStorageError::UnsafeWorkspace,
        )?;
        #[cfg(unix)]
        if canonical != absolute {
            return Err(HostStorageError::UnsafeWorkspace);
        }
        require_directory(&canonical.join(".crucible"))?;
        require_directory(&canonical.join(".crucible/objects"))?;
        let database = canonical.join(".crucible/database.sqlite");
        let metadata = std::fs::symlink_metadata(database).map_err(
            |_| HostStorageError::UnsafeWorkspace,
        )?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(HostStorageError::UnsafeWorkspace);
        }
        Ok(canonical)
    }

    fn lowercase_hex(value: &str, length: usize) -> bool {
        value.len() == length && value.bytes().all(
            |byte| byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte <= b'f'),
        )
    }

    fn increment_work(work: &mut usize) -> Result<(), HostStorageError> {
        *work = work.checked_add(1).ok_or(HostStorageError::WorkLimit)?;
        if *work > MAX_GC_CANDIDATES {
            Err(HostStorageError::WorkLimit)
        } else {
            Ok(())
        }
    }

    fn scan_objects(root: &Path) -> Result<Vec<ObjectEntry>, HostStorageError> {
        let algorithm = root.join(".crucible/objects/sha256");
        match std::fs::symlink_metadata(&algorithm) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(HostStorageError::UnsafeWorkspace),
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(HostStorageError::UnsafeWorkspace);
            },
            Ok(_) => {},
        }
        let mut work = 0usize;
        let mut objects = Vec::new();
        for first_entry in std::fs::read_dir(&algorithm).map_err(|_| HostStorageError::Integrity)? {
            increment_work(&mut work)?;
            let first_entry = first_entry.map_err(|_| HostStorageError::Integrity)?;
            let first_name = first_entry.file_name().into_string().map_err(
                |_| HostStorageError::Integrity,
            )?;
            let first_type = first_entry.file_type().map_err(|_| HostStorageError::Integrity)?;
            if first_type.is_symlink() || !first_type.is_dir() || !lowercase_hex(&first_name, 2) {
                return Err(HostStorageError::Integrity);
            }
            for second_entry in std::fs::read_dir(first_entry.path()).map_err(
                |_| HostStorageError::Integrity,
            )? {
                increment_work(&mut work)?;
                let second_entry = second_entry.map_err(|_| HostStorageError::Integrity)?;
                let second_name = second_entry.file_name().into_string().map_err(
                    |_| HostStorageError::Integrity,
                )?;
                let second_type = second_entry.file_type().map_err(
                    |_| HostStorageError::Integrity,
                )?;
                if second_type.is_symlink() || !second_type.is_dir() || !lowercase_hex(
                    &second_name,
                    2,
                ) {
                    return Err(HostStorageError::Integrity);
                }
                for object_entry in std::fs::read_dir(second_entry.path()).map_err(
                    |_| HostStorageError::Integrity,
                )? {
                    increment_work(&mut work)?;
                    let object_entry = object_entry.map_err(|_| HostStorageError::Integrity)?;
                    let name = object_entry.file_name().into_string().map_err(
                        |_| HostStorageError::Integrity,
                    )?;
                    let file_type = object_entry.file_type().map_err(
                        |_| HostStorageError::Integrity,
                    )?;
                    if file_type.is_symlink() || !file_type.is_file() {
                        return Err(HostStorageError::Integrity);
                    }
                    let artifact_id = if lowercase_hex(&name, 64) && name.starts_with(&first_name)
                        && name[2..].starts_with(&second_name) {
                        Some(format!("sha256:{name}"))
                    } else if name.starts_with('.') && name.contains(".tmp-") {
                        None
                    } else {
                        return Err(HostStorageError::Integrity);
                    };
                    objects.push(ObjectEntry { path: object_entry.path(), artifact_id });
                }
            }
        }
        Ok(objects)
    }

    fn read_bounded(path: &Path) -> Result<Vec<u8>, HostStorageError> {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| HostStorageError::Integrity)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len()
            > MAX_LOCAL_ARTIFACT_BYTES {
            return Err(HostStorageError::Integrity);
        }
        let file = std::fs::File::open(path).map_err(|_| HostStorageError::Integrity)?;
        let mut bytes = Vec::new();
        file.take(MAX_LOCAL_ARTIFACT_BYTES + 1).read_to_end(&mut bytes).map_err(
            |_| HostStorageError::Integrity,
        )?;
        if bytes.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES {
            Err(HostStorageError::Integrity)
        } else {
            Ok(bytes)
        }
    }

    let root = safe_workspace(Path::new(root))?;
    let mut connection = Connection::open(root.join(".crucible/database.sqlite")).map_err(
        |_| HostStorageError::UnsafeWorkspace,
    )?;
    connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(
        |_| HostStorageError::Persistence,
    )?;
    connection.set_limit(Limit::SQLITE_LIMIT_LENGTH, 1_048_576).map_err(
        |_| HostStorageError::Persistence,
    )?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;").map_err(
        |_| HostStorageError::Persistence,
    )?;
    let application_id: i64 = connection.query_row(
        "PRAGMA application_id",
        [],
        |row| row.get(0),
    ).map_err(|_| HostStorageError::UnsafeWorkspace)?;
    let schema_version: i64 = connection.query_row(
        "PRAGMA user_version",
        [],
        |row| row.get(0),
    ).map_err(|_| HostStorageError::UnsafeWorkspace)?;
    let quick_check: String = connection.query_row(
        "PRAGMA quick_check",
        [],
        |row| row.get(0),
    ).map_err(|_| HostStorageError::Integrity)?;
    if application_id != WORKSPACE_APPLICATION_ID || schema_version != WORKSPACE_SCHEMA_VERSION {
        return Err(HostStorageError::UnsafeWorkspace);
    }
    if quick_check != "ok" {
        return Err(HostStorageError::Integrity);
    }
    let transaction = connection.transaction_with_behavior(
        rusqlite::TransactionBehavior::Immediate,
    ).map_err(|_| HostStorageError::Persistence)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(
        |_| HostStorageError::Persistence,
    )?.as_secs();
    let now = i64::try_from(now).map_err(|_| HostStorageError::Persistence)?;
    transaction.execute(
        "UPDATE storage_leases SET status = 'released'
         WHERE status = 'active' AND expires_epoch <= ?1",
        [now],
    ).map_err(|_| HostStorageError::Persistence)?;
    let active_leases: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM storage_leases WHERE status = 'active'",
        [],
        |row| row.get(0),
    ).map_err(|_| HostStorageError::Persistence)?;
    if active_leases != 0 {
        return Err(HostStorageError::ActiveLease);
    }
    let mut database_artifacts = Vec::new();
    {
        let mut statement = transaction.prepare(
            "SELECT id, algorithm, digest, size_bytes, media_type FROM artifacts ORDER BY id",
        ).map_err(|_| HostStorageError::Persistence)?;
        let mut rows = statement.query([]).map_err(|_| HostStorageError::Persistence)?;
        while let Some(row) = rows.next().map_err(|_| HostStorageError::Persistence)? {
            if database_artifacts.len() == MAX_GC_CANDIDATES {
                return Err(HostStorageError::WorkLimit);
            }
            let id: String = row.get(0).map_err(|_| HostStorageError::Integrity)?;
            let algorithm: String = row.get(1).map_err(|_| HostStorageError::Integrity)?;
            let digest: String = row.get(2).map_err(|_| HostStorageError::Integrity)?;
            let size: i64 = row.get(3).map_err(|_| HostStorageError::Integrity)?;
            let media_type: Option<String> = row.get(4).map_err(|_| HostStorageError::Integrity)?;
            if algorithm != "sha256" || !lowercase_hex(&digest, 64) || id
                != format!("sha256:{digest}") || size < 0 || media_type.is_some() {
                return Err(HostStorageError::Integrity);
            }
            database_artifacts.push((id, size as u64));
        }
    }

    let objects = scan_objects(&root)?;
    let mut orphaned = 0u64;
    let mut temporary = 0u64;
    for object in objects.iter() {
        match &object.artifact_id {
            Some(id) => {
                let bytes = read_bounded(&object.path)?;
                let computed = ContentDigest::from_bytes(bytes.as_slice()).map_err(
                    |_| HostStorageError::Integrity,
                )?.into_artifact_id();
                if computed.as_str() != id {
                    return Err(HostStorageError::Integrity);
                }
                match database_artifacts.iter().find(|(referenced_id, _)| referenced_id == id) {
                    Some((_, size)) if *size == bytes.len() as u64 => {},
                    Some(_) => return Err(HostStorageError::Integrity),
                    None => orphaned += 1,
                }
            },
            None => temporary += 1,
        }
    }
    if !database_artifacts.iter().all(
        |database_entry|
            objects.iter().any(|object| object.artifact_id.as_ref() == Some(&database_entry.0)),
    ) {
        return Err(HostStorageError::Integrity);
    }
    let mut collected = 0u64;
    if action == HostStorageAction::Collect {
        transaction.execute(
            "INSERT INTO storage_generations(id, status, created_epoch)
             SELECT COALESCE((SELECT MAX(id) FROM storage_generations), 0) + 1, 'open', ?1
             WHERE NOT EXISTS(SELECT 1 FROM storage_generations WHERE status = 'open')",
            [now],
        ).map_err(|_| HostStorageError::Persistence)?;
        let generation_id: i64 = transaction.query_row(
            "SELECT id FROM storage_generations WHERE status = 'open' ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).map_err(|_| HostStorageError::Persistence)?;
        transaction.execute(
            "UPDATE storage_generations SET status = 'collecting'
             WHERE id = ?1 AND status = 'open'",
            [generation_id],
        ).map_err(|_| HostStorageError::Persistence)?;
        for object in objects.iter() {
            let should_collect = match &object.artifact_id {
                Some(id) => !database_artifacts.iter().any(
                    |(referenced_id, _)| referenced_id == id,
                ),
                None => true,
            };
            if should_collect {
                std::fs::remove_file(&object.path).map_err(|_| HostStorageError::Persistence)?;
                collected += 1;
            }
        }
        transaction.execute(
            "UPDATE storage_generations SET status = 'collected' WHERE id = ?1",
            [generation_id],
        ).map_err(|_| HostStorageError::Persistence)?;
        transaction.execute(
            "INSERT INTO storage_generations(id, status, created_epoch)
             VALUES (?1, 'open', ?2)",
            params![generation_id.checked_add(1).ok_or(HostStorageError::Persistence)?, now],
        ).map_err(|_| HostStorageError::Persistence)?;
    }
    transaction.commit().map_err(|_| HostStorageError::Persistence)?;
    Ok(
        StorageMaintenanceReport {
            verified: database_artifacts.len() as u64,
            orphaned,
            temporary,
            collected,
            preserved: database_artifacts.len() as u64,
        },
    )
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
        ).is_err() || transport.len() < 32 || transport.len() as u64 > MAX_LOCAL_ARGUMENT_WIRE_BYTES
            + 32 {
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

        fn capture_supervisor_stream<R: Read>(mut stream: R, limit: u64, nonce: [u8; 32]) -> Result<
            CapturedSupervisorStream,
            HostLocalRunError,
        > {
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
                limit.checked_add(160).ok_or(HostLocalRunError::Capture)?,
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
                let mut retained_index = 0usize;
                while retained_index < keep {
                    raw_retained.push(chunk[retained_index]);
                    retained_index += 1;
                }

                let overlap_len = overlap.len();
                let mut scan = overlap;
                let mut chunk_index = 0usize;
                while chunk_index < count {
                    scan.push(chunk[chunk_index]);
                    chunk_index += 1;
                }
                let scan_base = total.checked_sub(overlap_len as u64).ok_or(
                    HostLocalRunError::Capture,
                )?;
                let mut index = 0_usize;
                while index.checked_add(header_len + 1).is_some_and(|end| end <= scan.len()) {
                    if scan[index..].starts_with(&header[0..header_len]) {
                        let tag = scan[index + header_len];
                        let (record_len, termination) = match tag {
                            b'B' => (header_len + 1, None),
                            b'E' if index + header_len + 9 <= scan.len() => {
                                let bytes: [u8; 8] = scan[index + header_len + 1..index + header_len
                                    + 9].try_into().map_err(|_| HostLocalRunError::Capture)?;
                                (
                                    header_len + 9,
                                    Some(LocalTermination::ExitCode(i64::from_be_bytes(bytes))),
                                )
                            },
                            b'S' if index + header_len + 6 <= scan.len() => {
                                let bytes: [u8; 4] = scan[index + header_len + 1..index + header_len
                                    + 5].try_into().map_err(|_| HostLocalRunError::Capture)?;
                                let signal = i32::from_be_bytes(bytes);
                                let core = scan[index + header_len + 5];
                                if signal <= 0 || signal >= 128 || core > 1 {
                                    index += 1;
                                    continue;
                                }
                                (
                                    header_len + 6,
                                    Some(
                                        LocalTermination::UnixSignal {
                                            signal,
                                            core_dumped: core == 1,
                                        },
                                    ),
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
                (Some((_, start_end)), Some((position, end, termination))) if position
                    >= start_end => Some((position, end, termination)),
                _ => None,
            };
            let mut ranges = Vec::new();
            if let Some(range) = start_range {
                ranges.push(range);
            }
            if let Some((start, end, _)) = &accepted_end {
                ranges.push((*start, *end));
            }
            let removed = ranges.iter().try_fold(
                0_u64,
                |sum, (start, end)|
                    { sum.checked_add(end - start).ok_or(HostLocalRunError::Capture) },
            )?;
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
            Ok(
                CapturedSupervisorStream {
                    target_output: CapturedOutput::new(retained, discarded),
                    control_status,
                    target_started: start_range.is_some(),
                    target_termination,
                },
            )
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
                    let mut command = execution_command(plan, control.descriptor(), None, None)?;
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
            let mut nonce = [0_u8;32];
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
            if transport.write_all(&nonce).is_err() || transport.write_all(
                target_argument_wire,
            ).is_err() || transport.flush().is_err() {
                let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
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
            let supervisor_capture = stderr_thread.join().map_err(|_| HostLocalRunError::Capture)??;
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
    project_name: &[u32],
    retention_policy: PersistenceRetentionPolicy,
    oracle_verdict: LocalOracleVerdict,
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
        HostRunStoreAction::RecordBuild => {
            let effective_configuration = effective_configuration.ok_or(
                HostRunStoreError::Persist,
            )?;
            let capability_manifest = capability_manifest.ok_or(HostRunStoreError::Persist)?;
            let target_artifact = target_artifact.ok_or(HostRunStoreError::Persist)?;
            let target_manifest = target_manifest.ok_or(HostRunStoreError::Persist)?;
            if configuration_digest.len() != 71 || project_name.is_empty() || project_name.len()
                > 4_096 {
                return Err(HostRunStoreError::Persist);
            }
            let mut decoded_project_name = String::new();
            for code_point in project_name {
                decoded_project_name.push(
                    char::from_u32(*code_point).ok_or(HostRunStoreError::Persist)?,
                );
            }
            transaction.execute(
                "INSERT INTO capability_manifests(artifact_id, backend, platform)
                 VALUES (?1, 'linux-bubblewrap-prlimit-v1', 'linux')
                 ON CONFLICT(artifact_id) DO NOTHING",
                [capability_manifest.id.as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let project_id = format!("project-{configuration_digest}");
            transaction.execute(
                "INSERT INTO projects(id, name, configuration_artifact_id)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO NOTHING",
                params![project_id, decoded_project_name, effective_configuration.id.as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let persisted_project_id: String = transaction.query_row(
                "SELECT id FROM projects WHERE name = ?1",
                [decoded_project_name.as_str()],
                |row| row.get(0),
            ).map_err(|_| HostRunStoreError::Persist)?;
            let target_build_id = format!("target-build-{}", target_manifest.id.as_str());
            let target_id = format!("target-{target_build_id}");
            transaction.execute(
                "INSERT INTO targets(id, project_id, adapter, configuration_artifact_id)
                 VALUES (?1, ?2, 'cli', ?3)
                 ON CONFLICT(id) DO NOTHING",
                params![target_id, persisted_project_id, effective_configuration.id.as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
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
            let source_snapshot_id = format!("source-snapshot-{}", target_artifact.id.as_str());
            transaction.execute(
                "INSERT INTO source_snapshots(id, project_id, artifact_id, identity_digest)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    source_snapshot_id,
                    persisted_project_id,
                    target_artifact.id.as_str(),
                    target_artifact.id.as_str(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let recipe_id = format!("build-recipe-{}", target_manifest.id.as_str());
            transaction.execute(
                "INSERT INTO build_recipes(
                    id, target_id, source_snapshot_id, recipe_artifact_id, identity_digest
                 ) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    recipe_id,
                    target_id,
                    source_snapshot_id,
                    effective_configuration.id.as_str(),
                    configuration_digest,
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let execution_sequence = next_sequence(&transaction, "build-execution")?;
            if execution_sequence <= 0 {
                return Err(HostRunStoreError::Persist);
            }
            let execution_id = format!("build-execution-{execution_sequence:020}");
            transaction.execute(
                "INSERT INTO build_executions(
                    id, build_recipe_id, status, log_artifact_id, output_target_build_id
                 ) VALUES (?1, ?2, 'succeeded', NULL, ?3)",
                params![execution_id, recipe_id, target_build_id],
            ).map_err(|_| HostRunStoreError::Persist)?;
            HostRunStoreOutcome::Updated
        },
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
        HostRunStoreAction::AggregateSuccess => {
            let reservation = reservation.ok_or(HostRunStoreError::Persist)?;
            if retention_policy != PersistenceRetentionPolicy::HighThroughput || oracle_verdict
                != LocalOracleVerdict::Pass || project_name.is_empty() || project_name.len()
                > 4_096 {
                return Err(HostRunStoreError::Persist);
            }
            if admit_persisted_transition(
                &transaction,
                reservation,
                RunStoreTransition::RecordObservation,
            )? != RunAttemptStatus::Observed {
                return Err(HostRunStoreError::Conflict);
            }
            let mut decoded_project_name = String::new();
            for code_point in project_name {
                decoded_project_name.push(
                    char::from_u32(*code_point).ok_or(HostRunStoreError::Persist)?,
                );
            }
            let run_identity: (String, String, String, String) = transaction.query_row(
                "SELECT effective_configuration_artifact_id, seed, target_build_id,
                        configuration_digest
                 FROM runs WHERE id = ?1",
                [reservation.run_id().as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            ).map_err(|_| HostRunStoreError::Persist)?;
            let campaign_seed = run_identity.1.parse::<u64>().map_err(
                |_| HostRunStoreError::Persist,
            )?;
            let replay_seeds = derive_replay_seeds(campaign_seed);
            let project_id = format!("project-{}", run_identity.3);
            transaction.execute(
                "INSERT INTO projects(id, name, configuration_artifact_id)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO NOTHING",
                params![project_id, decoded_project_name, run_identity.0],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let persisted_project_id: String = transaction.query_row(
                "SELECT id FROM projects WHERE name = ?1",
                [decoded_project_name.as_str()],
                |row| row.get(0),
            ).map_err(|_| HostRunStoreError::Persist)?;
            let target_id = format!("target-{}", run_identity.2);
            transaction.execute(
                "INSERT INTO targets(id, project_id, adapter, configuration_artifact_id)
                 VALUES (?1, ?2, 'cli', ?3)
                 ON CONFLICT(id) DO NOTHING",
                params![target_id, persisted_project_id, run_identity.0],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let campaign_id = format!("campaign-{}", reservation.run_id().as_str());
            transaction.execute(
                "INSERT INTO campaigns(
                    id, project_id, configuration_artifact_id, retention_policy, status,
                    campaign_seed, scheduling_seed, fault_seed
                 ) VALUES (?1, ?2, ?3, 'aggregate-checkpoints', 'completed', ?4, ?5, ?6)",
                params![
                    campaign_id,
                    persisted_project_id,
                    run_identity.0,
                    replay_seeds.campaign.to_string(),
                    replay_seeds.scheduling.to_string(),
                    replay_seeds.fault.to_string(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let experiment_id = format!("experiment-{}", reservation.run_id().as_str());
            transaction.execute(
                "INSERT INTO experiments(id, campaign_id, kind, experiment_seed, status)
                 VALUES (?1, ?2, 'fuzz', ?3, 'completed')",
                params![experiment_id, campaign_id, replay_seeds.experiment.to_string()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let engine_allocations = [
                ("coverage-fuzzing", 35_i64),
                ("property-testing", 20_i64),
                ("stateful-testing", 15_i64),
                ("metamorphic-testing", 10_i64),
                ("fault-injection", 10_i64),
                ("symbolic-testing", 5_i64),
                ("miscellaneous", 5_i64),
            ];
            for (engine_class, allocated_slots) in engine_allocations {
                let executions = if engine_class == "coverage-fuzzing" {
                    1_i64
                } else {
                    0_i64
                };
                transaction.execute(
                    "INSERT INTO engine_stats(
                        campaign_id, epoch, engine_class, engine_seed, executions,
                        cpu_seconds, cpu_nanoseconds, new_coverage, new_findings,
                        unique_states, minimized_findings, mutation_score_improvement,
                        new_oracle_failures, corpus_quality_improvement, provenance_credit,
                        allocated_slots
                     ) VALUES (?1, 0, ?2, ?3, ?4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ?5)",
                    params![
                        campaign_id,
                        engine_class,
                        replay_seeds.engine.to_string(),
                        executions,
                        allocated_slots,
                    ],
                ).map_err(|_| HostRunStoreError::Persist)?;
            }
            let attempt_deletes = transaction.execute(
                "DELETE FROM run_attempts WHERE id = ?1 AND run_id = ?2
                 AND status = 'target_prepared'",
                params![reservation.attempt_id().as_str(), reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let control_deletes = transaction.execute(
                "DELETE FROM run_effective_controls WHERE run_id = ?1",
                [reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let run_deletes = transaction.execute(
                "DELETE FROM runs WHERE id = ?1",
                [reservation.run_id().as_str()],
            ).map_err(|_| HostRunStoreError::Persist)?;
            if attempt_deletes != 1 || control_deletes != 1 || run_deletes != 1 {
                return Err(HostRunStoreError::Conflict);
            }
            HostRunStoreOutcome::Updated
        },
        HostRunStoreAction::RecordObservation => {
            let reservation = reservation.ok_or(HostRunStoreError::Persist)?;
            let observation = observation_artifact.ok_or(HostRunStoreError::Persist)?;
            let stdout = stdout_artifact.ok_or(HostRunStoreError::Persist)?;
            let stderr = stderr_artifact.ok_or(HostRunStoreError::Persist)?;
            if completion_tag == 0 || termination_tag == 0 || project_name.is_empty()
                || project_name.len() > 4_096 {
                return Err(HostRunStoreError::Persist);
            }
            let mut decoded_project_name = String::new();
            for code_point in project_name {
                let scalar = char::from_u32(*code_point).ok_or(HostRunStoreError::Persist)?;
                decoded_project_name.push(scalar);
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
            let run_identity: (String, String, String, String, String) = transaction.query_row(
                "SELECT effective_configuration_artifact_id,
                        capability_manifest_artifact_id, seed, target_build_id,
                        configuration_digest
                 FROM runs WHERE id = ?1",
                [reservation.run_id().as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).map_err(|_| HostRunStoreError::Persist)?;
            let campaign_seed = run_identity.2.parse::<u64>().map_err(
                |_| HostRunStoreError::Persist,
            )?;
            let replay_seeds = derive_replay_seeds(campaign_seed);
            let retention = match retention_policy {
                PersistenceRetentionPolicy::ManagedReplay => "retain-every-run",
                PersistenceRetentionPolicy::HighThroughput => "aggregate-checkpoints",
            };

            let project_id = format!("project-{}", run_identity.4);
            transaction.execute(
                "INSERT INTO projects(id, name, configuration_artifact_id)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO NOTHING",
                params![project_id, decoded_project_name, run_identity.0],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let persisted_project_id: String = transaction.query_row(
                "SELECT id FROM projects WHERE name = ?1",
                [decoded_project_name.as_str()],
                |row| row.get(0),
            ).map_err(|_| HostRunStoreError::Persist)?;
            let target_id = format!("target-{}", run_identity.3);
            transaction.execute(
                "INSERT INTO targets(id, project_id, adapter, configuration_artifact_id)
                 VALUES (?1, ?2, 'cli', ?3)
                 ON CONFLICT(id) DO NOTHING",
                params![target_id, persisted_project_id, run_identity.0],
            ).map_err(|_| HostRunStoreError::Persist)?;

            let campaign_id = format!("campaign-{}", reservation.run_id().as_str());
            transaction.execute(
                "INSERT INTO campaigns(
                    id, project_id, configuration_artifact_id, retention_policy, status,
                    campaign_seed, scheduling_seed, fault_seed
                 ) VALUES (?1, ?2, ?3, ?4, 'completed', ?5, ?6, ?7)",
                params![
                    campaign_id,
                    persisted_project_id,
                    run_identity.0,
                    retention,
                    replay_seeds.campaign.to_string(),
                    replay_seeds.scheduling.to_string(),
                    replay_seeds.fault.to_string(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            let experiment_id = format!("experiment-{}", reservation.run_id().as_str());
            let experiment_kind = if retention_policy
                == PersistenceRetentionPolicy::HighThroughput {
                "fuzz"
            } else {
                "local-run"
            };
            transaction.execute(
                "INSERT INTO experiments(id, campaign_id, kind, experiment_seed, status)
                 VALUES (?1, ?2, ?3, ?4, 'completed')",
                params![
                    experiment_id,
                    campaign_id,
                    experiment_kind,
                    replay_seeds.experiment.to_string(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;
            transaction.execute(
                "INSERT INTO run_replay_metadata(
                    run_id, schema_version, campaign_seed, engine_seed, experiment_seed,
                    scheduling_seed, fault_seed, engine_seed_status,
                    engine_checkpoint_artifact_id, generated_schedule_artifact_id,
                    fault_trace_artifact_id, environment_artifact_id,
                    failure_predicate_artifact_id, engine_version
                 ) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, 'supported', NULL, NULL, NULL,
                    ?7, ?8, 'crucible-local-run-v1')",
                params![
                    reservation.run_id().as_str(),
                    replay_seeds.campaign.to_string(),
                    replay_seeds.engine.to_string(),
                    replay_seeds.experiment.to_string(),
                    replay_seeds.scheduling.to_string(),
                    replay_seeds.fault.to_string(),
                    run_identity.1,
                    run_identity.0,
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;

            let engine_allocations = [
                ("coverage-fuzzing", 35_i64),
                ("property-testing", 20_i64),
                ("stateful-testing", 15_i64),
                ("metamorphic-testing", 10_i64),
                ("fault-injection", 10_i64),
                ("symbolic-testing", 5_i64),
                ("miscellaneous", 5_i64),
            ];
            for (engine_class, allocated_slots) in engine_allocations {
                transaction.execute(
                    "INSERT INTO engine_stats(
                        campaign_id, epoch, engine_class, engine_seed, executions,
                        cpu_seconds, cpu_nanoseconds, new_coverage, new_findings,
                        unique_states, minimized_findings, mutation_score_improvement,
                        new_oracle_failures, corpus_quality_improvement, provenance_credit,
                        allocated_slots
                     ) VALUES (?1, 0, ?2, ?3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ?4)",
                    params![
                        campaign_id,
                        engine_class,
                        replay_seeds.engine.to_string(),
                        allocated_slots,
                    ],
                ).map_err(|_| HostRunStoreError::Persist)?;
            }

            let oracle_id = format!("oracle-{}", reservation.attempt_id().as_str());
            let verdict = match oracle_verdict {
                LocalOracleVerdict::Pass => "pass",
                LocalOracleVerdict::Fail => "fail",
            };
            transaction.execute(
                "INSERT INTO oracle_verdicts(
                    id, attempt_id, oracle_id, verdict, facts_artifact_id,
                    hypothesis_artifact_id
                 ) VALUES (?1, ?2, 'process-exit-predicate-v1', ?3, ?4, NULL)",
                params![
                    oracle_id,
                    reservation.attempt_id().as_str(),
                    verdict,
                    observation.id.as_str(),
                ],
            ).map_err(|_| HostRunStoreError::Persist)?;

            if oracle_verdict == LocalOracleVerdict::Fail {
                let existing_finding = transaction.query_row(
                    "SELECT id FROM findings
                     WHERE project_id = ?1 AND kind = 'target-defect/process-exit'
                       AND canonical_predicate_artifact_id = ?2
                     ORDER BY id LIMIT 1",
                    params![persisted_project_id, run_identity.0],
                    |row| row.get::<_, String>(0),
                ).optional().map_err(|_| HostRunStoreError::Persist)?;
                let (finding_id, is_original) = match existing_finding {
                    Some(id) => (id, 0_i64),
                    None => {
                        let finding_sequence = next_sequence(&transaction, "finding")?;
                        if finding_sequence <= 0 {
                            return Err(HostRunStoreError::Persist);
                        }
                        let id = format!("BUG-{finding_sequence:06}");
                        transaction.execute(
                            "INSERT INTO findings(
                                id, project_id, kind, status, canonical_predicate_artifact_id
                             ) VALUES (?1, ?2, 'target-defect/process-exit', 'open', ?3)",
                            params![id, persisted_project_id, run_identity.0],
                        ).map_err(|_| HostRunStoreError::Persist)?;
                        (id, 1_i64)
                    },
                };
                let instance_sequence = next_sequence(&transaction, "finding-instance")?;
                if instance_sequence <= 0 {
                    return Err(HostRunStoreError::Persist);
                }
                let instance_id = format!("finding-instance-{instance_sequence:020}");
                transaction.execute(
                    "INSERT INTO finding_instances(
                        id, finding_id, run_attempt_id, observation_id,
                        predicate_artifact_id, is_original
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        instance_id,
                        finding_id,
                        reservation.attempt_id().as_str(),
                        observation_id,
                        run_identity.0,
                        is_original,
                    ],
                ).map_err(|_| HostRunStoreError::Persist)?;
                transaction.execute(
                    "INSERT INTO finding_campaigns(finding_id, campaign_id, first_instance_id)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(finding_id, campaign_id) DO NOTHING",
                    params![finding_id, campaign_id, instance_id],
                ).map_err(|_| HostRunStoreError::Persist)?;

                let generation_id: i64 = transaction.query_row(
                    "SELECT id FROM storage_generations WHERE status = 'open'
                     ORDER BY id DESC LIMIT 1",
                    [],
                    |row| row.get(0),
                ).map_err(|_| HostRunStoreError::Persist)?;
                let target_artifacts: (String, String) = transaction.query_row(
                    "SELECT target_artifact_id, manifest_artifact_id
                     FROM target_builds WHERE id = ?1",
                    [run_identity.3.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                ).map_err(|_| HostRunStoreError::Persist)?;
                let rooted_artifacts = [
                    observation.id.as_str(),
                    stdout.id.as_str(),
                    stderr.id.as_str(),
                    run_identity.0.as_str(),
                    run_identity.1.as_str(),
                    target_artifacts.0.as_str(),
                    target_artifacts.1.as_str(),
                ];
                for artifact_id in rooted_artifacts {
                    transaction.execute(
                        "INSERT INTO artifact_roots(
                            artifact_id, root_kind, root_id, generation_id
                         ) VALUES (?1, 'original-finding', ?2, ?3)
                         ON CONFLICT(artifact_id, root_kind, root_id) DO NOTHING",
                        params![artifact_id, finding_id, generation_id],
                    ).map_err(|_| HostRunStoreError::Persist)?;
                }
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

// CRUCIBLE-TCB: CLI-HOST-INSPECT-001
#[verifier::external_body]
fn host_inspection_snapshot(root: &str, requested_run_id: &str) -> (result: Result<
    RunInspectionSnapshot,
    HostInspectionError,
>) {
    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostInspectionError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostInspectionError::Workspace)?.join(path)
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
                        return Err(HostInspectionError::Workspace);
                    }
                },
            }
        }
        if normalized.is_absolute() {
            Ok(normalized)
        } else {
            Err(HostInspectionError::Workspace)
        }
    }

    fn safe_directory(path: &Path) -> Result<(), HostInspectionError> {
        let mut cursor = PathBuf::new();
        for component in path.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                |_| HostInspectionError::Workspace,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(HostInspectionError::Workspace);
            }
        }
        Ok(())
    }

    fn artifact(row: &rusqlite::Row<'_>, start: usize) -> rusqlite::Result<ArtifactRef> {
        let raw_size: i64 = row.get(start + 1)?;
        let size_bytes = u64::try_from(raw_size).map_err(
            |_| rusqlite::Error::IntegralValueOutOfRange(start + 1, raw_size),
        )?;
        Ok(
            ArtifactRef {
                id: ArtifactId::new(row.get(start)?),
                size_bytes,
                media_type: row.get(start + 2)?,
            },
        )
    }

    fn optional_artifact(row: &rusqlite::Row<'_>, start: usize) -> rusqlite::Result<
        Option<ArtifactRef>,
    > {
        match row.get::<_, Option<String>>(start)? {
            Some(id) => {
                let raw_size: i64 = row.get(start + 1)?;
                let size_bytes = u64::try_from(raw_size).map_err(
                    |_| rusqlite::Error::IntegralValueOutOfRange(start + 1, raw_size),
                )?;
                Ok(
                    Some(
                        ArtifactRef {
                            id: ArtifactId::new(id),
                            size_bytes,
                            media_type: row.get(start + 2)?,
                        },
                    ),
                )
            },
            None => Ok(None),
        }
    }

    let root = lexical_absolute(Path::new(root))?;
    safe_directory(&root)?;
    let state = root.join(".crucible");
    safe_directory(&state)?;
    let database = state.join("database.sqlite");
    let metadata = std::fs::symlink_metadata(&database).map_err(
        |_| HostInspectionError::Workspace,
    )?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(HostInspectionError::Workspace);
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let connection = Connection::open_with_flags(database, flags).map_err(
        |_| HostInspectionError::Workspace,
    )?;
    connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(
        |_| HostInspectionError::Workspace,
    )?;
    connection.set_limit(Limit::SQLITE_LIMIT_LENGTH, 1_048_576).map_err(
        |_| HostInspectionError::Workspace,
    )?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA query_only = ON;").map_err(
        |_| HostInspectionError::Workspace,
    )?;
    let application_id: i64 = connection.query_row(
        "PRAGMA application_id",
        [],
        |row| row.get(0),
    ).map_err(|_| HostInspectionError::Workspace)?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(
        |_| HostInspectionError::Workspace,
    )?;
    if application_id != WORKSPACE_APPLICATION_ID || version != WORKSPACE_SCHEMA_VERSION {
        return Err(HostInspectionError::Workspace);
    }
    let run_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM runs WHERE id = ?1",
        [requested_run_id],
        |row| row.get(0),
    ).map_err(|_| HostInspectionError::InvalidEvidence)?;
    if run_count == 0 {
        return Err(HostInspectionError::NotFound);
    }
    if run_count != 1 {
        return Err(HostInspectionError::InvalidEvidence);
    }
    let attempt_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM run_attempts WHERE run_id = ?1",
        [requested_run_id],
        |row| row.get(0),
    ).map_err(|_| HostInspectionError::InvalidEvidence)?;
    if attempt_count != 1 {
        return Err(HostInspectionError::InvalidEvidence);
    }
    let snapshot = connection.query_row(
        "SELECT
            r.id, a.id, a.status,
            source.id, source.size_bytes, source.media_type,
            effective.id, effective.size_bytes, effective.media_type,
            r.configuration_digest, r.target_build_id,
            target.id, target.size_bytes, target.media_type,
            target_manifest.id, target_manifest.size_bytes, target_manifest.media_type,
            b.identity_digest,
            capability.id, capability.size_bytes, capability.media_type,
            r.seed,
            controls.timeout_ms, controls.memory_bytes, controls.max_processes,
            controls.max_stream_bytes, controls.network_policy, controls.isolation_backend,
            controls.output_capture_status,
            observation.id, observation.size_bytes, observation.media_type,
            stdout.id, stdout.size_bytes, stdout.media_type,
            stderr.id, stderr.size_bytes, stderr.media_type,
            o.completion_tag, o.termination_tag,
            h.kind,
            detail.id, detail.size_bytes, detail.media_type
         FROM runs r
         JOIN run_attempts a ON a.run_id = r.id
         JOIN run_effective_controls controls ON controls.run_id = r.id
         JOIN artifacts source ON source.id = r.configuration_source_artifact_id
         JOIN artifacts effective ON effective.id = r.effective_configuration_artifact_id
         JOIN artifacts capability ON capability.id = r.capability_manifest_artifact_id
         LEFT JOIN target_builds b ON b.id = r.target_build_id
         LEFT JOIN artifacts target ON target.id = b.target_artifact_id
         LEFT JOIN artifacts target_manifest ON target_manifest.id = b.manifest_artifact_id
         LEFT JOIN observations o ON o.attempt_id = a.id
         LEFT JOIN artifacts observation ON observation.id = o.observation_artifact_id
         LEFT JOIN artifacts stdout ON stdout.id = o.stdout_artifact_id
         LEFT JOIN artifacts stderr ON stderr.id = o.stderr_artifact_id
         LEFT JOIN harness_failures h ON h.attempt_id = a.id
         LEFT JOIN artifacts detail ON detail.id = h.detail_artifact_id
         WHERE r.id = ?1",
        [requested_run_id],
        |row|
            {
                let status_text: String = row.get(2)?;
                let status = match status_text.as_str() {
                    "reserved" => InspectionStatus::Reserved,
                    "target_prepared" => InspectionStatus::TargetPrepared,
                    "observed" => InspectionStatus::Observed,
                    "harness_failure" => InspectionStatus::HarnessFailure,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                let target = match row.get::<_, Option<String>>(10)? {
                    Some(build_id) => {
                        Some(
                            InspectionTarget {
                                build_id,
                                target_artifact: artifact(row, 11)?,
                                manifest_artifact: artifact(row, 14)?,
                                identity_digest: row.get(17)?,
                            },
                        )
                    },
                    None => None,
                };
                let observation = match optional_artifact(row, 29)? {
                    Some(observation_artifact) => {
                        let raw_completion: i64 = row.get(38)?;
                        let completion_tag = u16::try_from(raw_completion).map_err(
                            |_| rusqlite::Error::IntegralValueOutOfRange(38, raw_completion),
                        )?;
                        let raw_termination: i64 = row.get(39)?;
                        let termination_tag = u16::try_from(raw_termination).map_err(
                            |_| rusqlite::Error::IntegralValueOutOfRange(39, raw_termination),
                        )?;
                        Some(
                            InspectionObservation {
                                artifact: observation_artifact,
                                stdout_artifact: artifact(row, 32)?,
                                stderr_artifact: artifact(row, 35)?,
                                completion_tag,
                                termination_tag,
                            },
                        )
                    },
                    None => None,
                };
                let harness_failure = match row.get::<_, Option<String>>(40)? {
                    Some(kind) => {
                        Some(
                            InspectionHarnessFailure {
                                kind,
                                detail_artifact: optional_artifact(row, 41)?,
                            },
                        )
                    },
                    None => None,
                };
                Ok(
                    RunInspectionSnapshot {
                        run_id: row.get(0)?,
                        attempt_id: row.get(1)?,
                        status,
                        configuration_source: artifact(row, 3)?,
                        effective_configuration: artifact(row, 6)?,
                        configuration_digest: row.get(9)?,
                        target,
                        capability_manifest: artifact(row, 18)?,
                        seed: row.get(21)?,
                        controls: InspectionControls {
                            timeout_ms: row.get(22)?,
                            memory_bytes: row.get(23)?,
                            max_processes: row.get(24)?,
                            max_stream_bytes: row.get(25)?,
                            network_policy: row.get(26)?,
                            isolation_backend: row.get(27)?,
                            output_capture_status: row.get(28)?,
                        },
                        observation,
                        harness_failure,
                    },
                )
            },
    ).map_err(|_| HostInspectionError::InvalidEvidence)?;
    connection.close().map_err(|_| HostInspectionError::Workspace)?;
    Ok(snapshot)
}

// CRUCIBLE-TCB: CLI-HOST-DOMAIN-READ-001
#[verifier::external_body]
fn host_domain_read(
    action: HostDomainReadAction,
    root: &str,
    subject: &str,
    format: ReportFormat,
) -> (result: Result<Vec<u8>, HostDomainReadError>)
    ensures
        match &result {
            Ok(output) => output@.len() <= MAX_DOMAIN_REPORT_BYTES,
            Err(_) => true,
        },
{
    #[derive(Clone)]
    struct FindingReport {
        finding_id: String,
        project: String,
        kind: String,
        status: String,
        predicate_artifact: String,
        instance_id: String,
        attempt_id: String,
        observation_id: String,
        run_id: String,
        target_build_id: String,
        capability_artifact: String,
        oracle_id: String,
        oracle_kind: String,
        oracle_verdict: String,
        facts_artifact: String,
        hypothesis_artifact: Option<String>,
        campaign_seed: String,
        engine_seed: String,
        experiment_seed: String,
        scheduling_seed: String,
        fault_seed: String,
        engine_seed_status: String,
        engine_version: String,
        timeout_ms: String,
        memory_bytes: String,
        max_processes: String,
        max_stream_bytes: String,
        network_policy: String,
        isolation_backend: String,
        output_capture_status: String,
        reproduction: Option<(i64, i64, String, i64, i64, i64)>,
        rooted_artifacts: Vec<String>,
    }

    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostDomainReadError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostDomainReadError::Workspace)?.join(path)
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
                        return Err(HostDomainReadError::Workspace);
                    }
                },
            }
        }
        if normalized.is_absolute() {
            Ok(normalized)
        } else {
            Err(HostDomainReadError::Workspace)
        }
    }

    fn require_safe_directory(path: &Path) -> Result<(), HostDomainReadError> {
        let mut cursor = PathBuf::new();
        for component in path.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                |_| HostDomainReadError::Workspace,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(HostDomainReadError::Workspace);
            }
        }
        Ok(())
    }

    fn open_database(root: &str) -> Result<Connection, HostDomainReadError> {
        let root = lexical_absolute(Path::new(root))?;
        require_safe_directory(&root)?;
        let state = root.join(".crucible");
        require_safe_directory(&state)?;
        let database = state.join("database.sqlite");
        let metadata = std::fs::symlink_metadata(&database).map_err(
            |_| HostDomainReadError::Workspace,
        )?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(HostDomainReadError::Workspace);
        }
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let connection = Connection::open_with_flags(database, flags).map_err(
            |_| HostDomainReadError::Workspace,
        )?;
        connection.set_limit(Limit::SQLITE_LIMIT_LENGTH, 1_048_576).map_err(
            |_| HostDomainReadError::Workspace,
        )?;
        connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(
            |_| HostDomainReadError::Workspace,
        )?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; BEGIN DEFERRED;",
        ).map_err(|_| HostDomainReadError::Workspace)?;
        let identity: (i64, i64, String) = connection.query_row(
            "SELECT (SELECT application_id FROM pragma_application_id),
                    (SELECT user_version FROM pragma_user_version),
                    (SELECT quick_check FROM pragma_quick_check)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|_| HostDomainReadError::Workspace)?;
        if identity.0 != WORKSPACE_APPLICATION_ID || identity.1 != WORKSPACE_SCHEMA_VERSION
            || identity.2 != "ok" {
            return Err(HostDomainReadError::Workspace);
        }
        Ok(connection)
    }

    fn bounded(mut output: Vec<u8>) -> Result<Vec<u8>, HostDomainReadError> {
        if output.len() > MAX_DOMAIN_REPORT_BYTES {
            Err(HostDomainReadError::OutputLimit)
        } else {
            if output.last() != Some(&b'\n') {
                output.push(b'\n');
            }
            if output.len() > MAX_DOMAIN_REPORT_BYTES {
                Err(HostDomainReadError::OutputLimit)
            } else {
                Ok(output)
            }
        }
    }

    fn json_bytes(value: &serde_json::Value) -> Result<Vec<u8>, HostDomainReadError> {
        serde_json::to_vec(value).map_err(|_| HostDomainReadError::InvalidEvidence).and_then(
            bounded,
        )
    }

    fn json_object(entries: &[(&str, serde_json::Value)]) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        for (key, value) in entries {
            object.insert((*key).to_owned(), value.clone());
        }
        serde_json::Value::Object(object)
    }

    fn json_text(value: &str) -> serde_json::Value {
        serde_json::Value::String(value.to_owned())
    }

    fn json_array(values: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::Value::Array(values)
    }

    fn json_fact_node(id: &str, kind: &str) -> serde_json::Value {
        let mut node = serde_json::Map::new();
        node.insert("id".to_owned(), json_text(id));
        node.insert("kind".to_owned(), json_text(kind));
        node.insert("fact".to_owned(), serde_json::Value::Bool(true));
        serde_json::Value::Object(node)
    }

    fn json_edge(from: &str, to: &str, relation: &str) -> serde_json::Value {
        let mut edge = serde_json::Map::new();
        edge.insert("from".to_owned(), json_text(from));
        edge.insert("to".to_owned(), json_text(to));
        edge.insert("relation".to_owned(), json_text(relation));
        serde_json::Value::Object(edge)
    }

    fn xml_escape(value: &str) -> String {
        let mut escaped = String::new();
        for character in value.chars() {
            let code_point = character as u32;
            if code_point == 38 {
                escaped.push_str("&amp;");
            } else if code_point == 60 {
                escaped.push_str("&lt;");
            } else if code_point == 62 {
                escaped.push_str("&gt;");
            } else if code_point == 34 {
                escaped.push_str("&quot;");
            } else if code_point == 39 {
                escaped.push_str("&apos;");
            } else {
                escaped.push(character);
            }
        }
        escaped
    }

    fn load_finding(connection: &Connection, subject: &str) -> Result<
        FindingReport,
        HostDomainReadError,
    > {
        fn decode_finding(row: &rusqlite::Row<'_>) -> rusqlite::Result<FindingReport> {
            Ok(
                FindingReport {
                    finding_id: row.get(0)?,
                    project: row.get(1)?,
                    kind: row.get(2)?,
                    status: row.get(3)?,
                    predicate_artifact: row.get(4)?,
                    instance_id: row.get(5)?,
                    attempt_id: row.get(6)?,
                    observation_id: row.get(7)?,
                    run_id: row.get(8)?,
                    target_build_id: row.get(9)?,
                    capability_artifact: row.get(10)?,
                    oracle_id: row.get(11)?,
                    oracle_kind: row.get(12)?,
                    oracle_verdict: row.get(13)?,
                    facts_artifact: row.get(14)?,
                    hypothesis_artifact: row.get(15)?,
                    campaign_seed: row.get(16)?,
                    engine_seed: row.get(17)?,
                    experiment_seed: row.get(18)?,
                    scheduling_seed: row.get(19)?,
                    fault_seed: row.get(20)?,
                    engine_seed_status: row.get(21)?,
                    engine_version: row.get(22)?,
                    timeout_ms: row.get(23)?,
                    memory_bytes: row.get(24)?,
                    max_processes: row.get(25)?,
                    max_stream_bytes: row.get(26)?,
                    network_policy: row.get(27)?,
                    isolation_backend: row.get(28)?,
                    output_capture_status: row.get(29)?,
                    reproduction: None,
                    rooted_artifacts: Vec::new(),
                },
            )
        }

        fn decode_reproduction(row: &rusqlite::Row<'_>) -> rusqlite::Result<
            (i64, i64, String, i64, i64, i64),
        > {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        }

        fn decode_artifact_id(row: &rusqlite::Row<'_>) -> rusqlite::Result<String> {
            row.get(0)
        }

        if subject.is_empty() || subject.len() > 4_096 {
            return Err(HostDomainReadError::NotFound);
        }
        let report_result = connection.query_row(
            "SELECT f.id, p.name, f.kind, f.status, f.canonical_predicate_artifact_id,
                    fi.id, fi.run_attempt_id, COALESCE(fi.observation_id, ''),
                    r.id, r.target_build_id, r.capability_manifest_artifact_id,
                    ov.id, ov.oracle_id, ov.verdict, ov.facts_artifact_id,
                    ov.hypothesis_artifact_id,
                    rm.campaign_seed, rm.engine_seed, rm.experiment_seed,
                    rm.scheduling_seed, rm.fault_seed, rm.engine_seed_status,
                    rm.engine_version, c.timeout_ms, c.memory_bytes, c.max_processes,
                    c.max_stream_bytes, c.network_policy, c.isolation_backend,
                    c.output_capture_status
             FROM findings f
             JOIN projects p ON p.id = f.project_id
             JOIN finding_instances fi ON fi.finding_id = f.id
             JOIN run_attempts ra ON ra.id = fi.run_attempt_id
             JOIN runs r ON r.id = ra.run_id
             JOIN oracle_verdicts ov ON ov.attempt_id = ra.id
             JOIN run_replay_metadata rm ON rm.run_id = r.id
             JOIN run_effective_controls c ON c.run_id = r.id
             WHERE f.id = ?1
            ORDER BY fi.is_original DESC, fi.id, ov.id LIMIT 1",
            [subject],
            decode_finding,
        ).optional();
        let optional_report = report_result.map_err(|_| HostDomainReadError::InvalidEvidence)?;
        let mut report = optional_report.ok_or(HostDomainReadError::NotFound)?;
        let reproduction_result = connection.query_row(
            "SELECT attempt_count, observed_failures, classification,
                    environment_equivalent, schedule_replayed_exactly,
                    fault_trace_replayed_exactly
             FROM reproduction_samples WHERE finding_id = ?1
            ORDER BY id DESC LIMIT 1",
            [subject],
            decode_reproduction,
        ).optional();
        report.reproduction = reproduction_result.map_err(
            |_| HostDomainReadError::InvalidEvidence,
        )?;
        let statement_result = connection.prepare(
            "SELECT artifact_id FROM artifact_roots
             WHERE root_kind = 'original-finding' AND root_id = ?1
             ORDER BY artifact_id LIMIT 129",
        );
        let mut statement = statement_result.map_err(|_| HostDomainReadError::InvalidEvidence)?;
        let rows_result = statement.query_map([subject], decode_artifact_id);
        let rows = rows_result.map_err(|_| HostDomainReadError::InvalidEvidence)?;
        for row in rows {
            if report.rooted_artifacts.len() == 128 {
                return Err(HostDomainReadError::OutputLimit);
            }
            report.rooted_artifacts.push(row.map_err(|_| HostDomainReadError::InvalidEvidence)?);
        }
        Ok(report)
    }

    fn facts(report: &FindingReport) -> serde_json::Value {
        let reproduction = match &report.reproduction {
            Some((attempts, failures, classification, environment, schedule, faults)) => {
                json_object(
                    &[
                        ("attempts", serde_json::Value::from(*attempts)),
                        ("observed_failures", serde_json::Value::from(*failures)),
                        ("classification", json_text(classification)),
                        ("environment_equivalent", serde_json::Value::Bool(*environment == 1)),
                        ("schedule_replayed_exactly", serde_json::Value::Bool(*schedule == 1)),
                        ("fault_trace_replayed_exactly", serde_json::Value::Bool(*faults == 1)),
                        ("determinism_proven", serde_json::Value::Bool(false)),
                    ],
                )
            },
            None => json_object(
                &[
                    ("attempts", serde_json::Value::from(0)),
                    ("observed_failures", serde_json::Value::from(0)),
                    ("classification", json_text("not-sampled")),
                    ("determinism_proven", serde_json::Value::Bool(false)),
                ],
            ),
        };
        let hypothesis = match &report.hypothesis_artifact {
            Some(value) => json_text(value),
            None => serde_json::Value::Null,
        };
        let oracle = json_object(
            &[
                ("verdict_id", json_text(&report.oracle_id)),
                ("identity", json_text(&report.oracle_kind)),
                ("verdict", json_text(&report.oracle_verdict)),
                ("facts_artifact_id", json_text(&report.facts_artifact)),
                ("hypothesis_artifact_id", hypothesis),
            ],
        );
        let controls = json_object(
            &[
                ("timeout_ms", json_text(&report.timeout_ms)),
                ("memory_bytes", json_text(&report.memory_bytes)),
                ("max_processes", json_text(&report.max_processes)),
                ("max_stream_bytes", json_text(&report.max_stream_bytes)),
                ("network_policy", json_text(&report.network_policy)),
                ("isolation_backend", json_text(&report.isolation_backend)),
                ("output_capture_status", json_text(&report.output_capture_status)),
            ],
        );
        let seeds = json_object(
            &[
                ("campaign", json_text(&report.campaign_seed)),
                ("engine", json_text(&report.engine_seed)),
                ("experiment", json_text(&report.experiment_seed)),
                ("scheduling", json_text(&report.scheduling_seed)),
                ("fault", json_text(&report.fault_seed)),
                ("engine_seed_status", json_text(&report.engine_seed_status)),
                ("engine_version", json_text(&report.engine_version)),
            ],
        );
        json_object(
            &[
                ("finding_id", json_text(&report.finding_id)),
                ("project", json_text(&report.project)),
                ("class", json_text(&report.kind)),
                ("status", json_text(&report.status)),
                ("instance_id", json_text(&report.instance_id)),
                ("attempt_id", json_text(&report.attempt_id)),
                ("run_id", json_text(&report.run_id)),
                ("observation_id", json_text(&report.observation_id)),
                ("target_build_id", json_text(&report.target_build_id)),
                ("capability_artifact_id", json_text(&report.capability_artifact)),
                ("predicate_artifact_id", json_text(&report.predicate_artifact)),
                ("oracle", oracle),
                ("controls", controls),
                ("reproduction", reproduction),
                ("seeds", seeds),
                ("scenario_trace", json_array(Vec::new())),
            ],
        )
    }

    fn render_evidence_graph(report: &FindingReport) -> Result<Vec<u8>, HostDomainReadError> {
        let mut nodes = Vec::new();
        nodes.push(json_fact_node(&report.finding_id, "finding"));
        nodes.push(json_fact_node(&report.oracle_id, "oracle-verdict"));
        nodes.push(json_fact_node(&report.observation_id, "observation"));
        for id in &report.rooted_artifacts {
            nodes.push(json_fact_node(id, "artifact"));
        }
        let edges =
            vec![
            json_edge(&report.observation_id, &report.oracle_id, "evaluated-by"),
            json_edge(&report.oracle_id, &report.finding_id, "instantiates"),
        ];
        let mut graph = serde_json::Map::new();
        graph.insert("schema".to_owned(), json_text("crucible.evidence-graph.v1"));
        graph.insert("nodes".to_owned(), json_array(nodes));
        graph.insert("edges".to_owned(), json_array(edges));
        graph.insert("hypotheses".to_owned(), json_array(Vec::new()));
        json_bytes(&serde_json::Value::Object(graph))
    }

    fn render_bundle_manifest(report: &FindingReport, report_facts: serde_json::Value) -> Result<
        Vec<u8>,
        HostDomainReadError,
    > {
        let mut artifact_values = Vec::new();
        for artifact in &report.rooted_artifacts {
            artifact_values.push(json_text(artifact));
        }
        let mut unsigned_object = serde_json::Map::new();
        unsigned_object.insert("schema".to_owned(), json_text("crucible.evidence-bundle.v1"));
        unsigned_object.insert("finding_id".to_owned(), json_text(&report.finding_id));
        unsigned_object.insert("facts".to_owned(), report_facts);
        unsigned_object.insert("hypotheses".to_owned(), json_array(Vec::new()));
        unsigned_object.insert("artifacts".to_owned(), json_array(artifact_values));
        unsigned_object.insert(
            "signature_scope".to_owned(),
            json_text("exact-manifest-and-provenance"),
        );
        unsigned_object.insert(
            "hypothesis_truth_attested".to_owned(),
            serde_json::Value::Bool(false),
        );
        let unsigned = serde_json::Value::Object(unsigned_object);
        let unsigned_bytes = serde_json::to_vec(&unsigned).map_err(
            |_| HostDomainReadError::InvalidEvidence,
        )?;
        let digest = ContentDigest::from_bytes(&unsigned_bytes).map_err(
            |_| HostDomainReadError::InvalidEvidence,
        )?.into_artifact_id();
        let mut artifact_values = Vec::new();
        for artifact in &report.rooted_artifacts {
            artifact_values.push(json_text(artifact));
        }
        let mut manifest = serde_json::Map::new();
        manifest.insert("schema".to_owned(), json_text("crucible.evidence-bundle.v1"));
        manifest.insert("finding_id".to_owned(), json_text(&report.finding_id));
        manifest.insert("facts".to_owned(), unsigned["facts"].clone());
        manifest.insert("hypotheses".to_owned(), json_array(Vec::new()));
        manifest.insert("artifacts".to_owned(), json_array(artifact_values));
        manifest.insert("seeds".to_owned(), unsigned["facts"]["seeds"].clone());
        manifest.insert("unsigned_payload_digest".to_owned(), json_text(digest.as_str()));
        manifest.insert("signature".to_owned(), serde_json::Value::Null);
        manifest.insert("signature_scope".to_owned(), json_text("exact-manifest-and-provenance"));
        manifest.insert("hypothesis_truth_attested".to_owned(), serde_json::Value::Bool(false));
        json_bytes(&serde_json::Value::Object(manifest))
    }

    fn render_capabilities(connection: &Connection) -> Result<Vec<u8>, HostDomainReadError> {
        let mut manifests = Vec::new();
        let mut statement = connection.prepare(
            "SELECT artifact_id, backend, platform FROM capability_manifests
             ORDER BY artifact_id LIMIT 1025",
        ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
        let rows = statement.query_map(
            [],
            |row|
                { Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
                },
        ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
        for row in rows {
            if manifests.len() == 1_024 {
                return Err(HostDomainReadError::OutputLimit);
            }
            let (artifact_id, backend, platform) = row.map_err(
                |_| HostDomainReadError::InvalidEvidence,
            )?;
            manifests.push(
                json_object(
                    &[
                        ("artifact_id", json_text(&artifact_id)),
                        ("backend", json_text(&backend)),
                        ("platform", json_text(&platform)),
                    ],
                ),
            );
        }
        let platform_status = if cfg!(target_os = "linux") {
            "supported"
        } else {
            "unsupported-by-platform"
        };
        let features =
            vec![
            json_object(&[
                ("name", json_text("sqlite-filesystem-storage")),
                ("status", json_text("supported")),
            ]),
            json_object(&[
                ("name", json_text("transactional-server-remote-storage-identity")),
                ("status", json_text("specified")),
            ]),
            json_object(&[
                ("name", json_text("linux-cli-execution")),
                ("status", json_text(platform_status)),
            ]),
            json_object(&[
                ("name", json_text("signed-evidence-bundles")),
                ("status", json_text("not-configured")),
                ("reason", json_text("no signing key was supplied")),
            ]),
        ];
        json_bytes(
            &json_object(
                &[
                    ("schema", json_text("crucible.capabilities.v1")),
                    ("features", json_array(features)),
                    ("recorded_manifests", json_array(manifests)),
                ],
            ),
        )
    }

    fn render_proof_report(connection: &Connection) -> Result<Vec<u8>, HostDomainReadError> {
        let mut rows_output = Vec::new();
        let mut statement = connection.prepare(
            "SELECT pa.id, pa.artifact_id, pa.proof_kind,
                    pa.trusted_boundary_digest, vr.id, vr.status
             FROM proof_artifacts pa
             JOIN verification_runs vr ON vr.id = pa.verification_run_id
             ORDER BY pa.id LIMIT 1025",
        ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
        let rows = statement.query_map(
            [],
            |row|
                {
                    Ok(
                        (
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                        ),
                    )
                },
        ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
        for row in rows {
            if rows_output.len() == 1_024 {
                return Err(HostDomainReadError::OutputLimit);
            }
            let (id, artifact_id, kind, boundary_digest, verification_run, status) = row.map_err(
                |_| HostDomainReadError::InvalidEvidence,
            )?;
            rows_output.push(
                json_object(
                    &[
                        ("id", json_text(&id)),
                        ("artifact_id", json_text(&artifact_id)),
                        ("kind", json_text(&kind)),
                        ("trusted_boundary_digest", json_text(&boundary_digest)),
                        ("verification_run_id", json_text(&verification_run)),
                        ("status", json_text(&status)),
                    ],
                ),
            );
        }
        json_bytes(
            &json_object(
                &[
                    ("schema", json_text("crucible.proof-report.v1")),
                    ("proof_artifacts", json_array(rows_output)),
                    ("reproduction_command", json_text("cargo xtask verify --all")),
                ],
            ),
        )
    }

    fn render_tcb_report() -> Result<Vec<u8>, HostDomainReadError> {
        let policy = json_object(
            &[
                ("deny_unregistered", serde_json::Value::Bool(true)),
                ("deny_unapproved_growth", serde_json::Value::Bool(true)),
            ],
        );
        let boundary_names = [
            "XTASK-HOST-ARGS-001",
            "XTASK-HOST-SNAPSHOT-001",
            "XTASK-HOST-PROBE-001",
            "XTASK-HOST-COMMAND-001",
            "XTASK-HOST-REPORTS-001",
            "XTASK-HOST-WRITE-001",
            "XTASK-HOST-COMPLETE-001",
            "CLI-HOST-ARGS-001",
            "CLI-HOST-INIT-001",
            "CLI-HOST-ARTIFACT-001",
            "CLI-HOST-CONFIG-001",
            "CLI-HOST-LOCAL-SUPERVISOR-001",
            "CLI-HOST-LOCAL-RUN-001",
            "CLI-HOST-RUN-STORE-001",
            "CLI-HOST-INSPECT-001",
            "CLI-HOST-STORAGE-001",
            "CLI-HOST-DOMAIN-READ-001",
            "CLI-HOST-FINDING-001",
            "CLI-HOST-LOG-001",
            "CLI-HOST-COMPLETE-001",
            "CORE-HOST-UTF8-001",
        ];
        let mut boundaries = Vec::new();
        for name in boundary_names {
            boundaries.push(json_text(name));
        }
        json_bytes(
            &json_object(
                &[
                    ("schema", json_text("crucible.tcb-report.v1")),
                    ("policy", policy),
                    (
                        "audit_command",
                        json_text(
                            "cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth",
                        ),
                    ),
                    ("boundaries", json_array(boundaries)),
                ],
            ),
        )
    }

    fn render_plugin_report(connection: &Connection) -> Result<Vec<u8>, HostDomainReadError> {
        let mut plugins = Vec::new();
        let mut statement = connection.prepare(
            "SELECT id, manifest_artifact_id, capability_manifest_artifact_id,
                    implementation_artifact_id, status
             FROM plugin_identities ORDER BY id LIMIT 1025",
        ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
        let rows = statement.query_map(
            [],
            |row|
                {
                    Ok(
                        (
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                        ),
                    )
                },
        ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
        for row in rows {
            if plugins.len() == 1_024 {
                return Err(HostDomainReadError::OutputLimit);
            }
            let (id, manifest, capabilities, implementation, status) = row.map_err(
                |_| HostDomainReadError::InvalidEvidence,
            )?;
            plugins.push(
                json_object(
                    &[
                        ("id", json_text(&id)),
                        ("manifest_artifact_id", json_text(&manifest)),
                        ("capability_manifest_artifact_id", json_text(&capabilities)),
                        ("implementation_artifact_id", json_text(&implementation)),
                        ("status", json_text(&status)),
                    ],
                ),
            );
        }
        json_bytes(
            &json_object(
                &[
                    ("schema", json_text("crucible.plugin-report.v1")),
                    ("plugins", json_array(plugins)),
                ],
            ),
        )
    }

    fn render_human_report(report: &FindingReport) -> Result<Vec<u8>, HostDomainReadError> {
        let reproduction = match &report.reproduction {
            Some((attempts, failures, classification, _, _, _)) => {
                let mut text = failures.to_string();
                text.push('/');
                text.push_str(&attempts.to_string());
                text.push_str(" under recorded controls (");
                text.push_str(classification);
                text.push_str("; determinism is not proven)");
                text
            },
            None => String::from("not sampled under recorded controls; determinism is not proven"),
        };
        let fields = [
            &report.finding_id,
            &report.project,
            &report.target_build_id,
            &report.kind,
            &report.status,
            &reproduction,
            &report.isolation_backend,
            &report.network_policy,
            &report.output_capture_status,
            &report.oracle_kind,
            &report.oracle_verdict,
            &report.predicate_artifact,
            &report.facts_artifact,
        ];
        let labels = [
            "",
            "\n\nObserved facts:\n  Target: ",
            "\n  Target build: ",
            "\n  Class: ",
            "\n  Status: ",
            "\n  Reproduction: ",
            "\n  Isolation: ",
            ", ",
            ", ",
            "\n  Oracle: ",
            " / ",
            "\n  Failure predicate: ",
            "\n  Facts artifact: ",
        ];
        let mut output = String::new();
        for index in 0..fields.len() {
            output.push_str(labels[index]);
            output.push_str(fields[index]);
        }
        output.push_str("\n\nHypotheses:\n  none recorded\n");
        bounded(output.into_bytes())
    }

    fn render_json_report(report_facts: serde_json::Value) -> Result<Vec<u8>, HostDomainReadError> {
        json_bytes(
            &json_object(
                &[
                    ("schema", json_text("crucible.finding-report.v1")),
                    ("facts", report_facts),
                    ("hypotheses", json_array(Vec::new())),
                ],
            ),
        )
    }

    fn render_json_lines_report(report_facts: serde_json::Value) -> Result<
        Vec<u8>,
        HostDomainReadError,
    > {
        let fact_record = json_object(
            &[
                ("schema", json_text("crucible.finding-report.v1")),
                ("record", json_text("facts")),
                ("value", report_facts),
            ],
        );
        let mut output = serde_json::to_vec(&fact_record).map_err(
            |_| HostDomainReadError::InvalidEvidence,
        )?;
        output.push(b'\n');
        let hypothesis_record = json_object(
            &[
                ("schema", json_text("crucible.finding-report.v1")),
                ("record", json_text("hypotheses")),
                ("value", json_array(Vec::new())),
            ],
        );
        output.extend_from_slice(
            &serde_json::to_vec(&hypothesis_record).map_err(
                |_| HostDomainReadError::InvalidEvidence,
            )?,
        );
        bounded(output)
    }

    fn render_sarif_report(report: &FindingReport, report_facts: serde_json::Value) -> Result<
        Vec<u8>,
        HostDomainReadError,
    > {
        let driver = json_object(
            &[
                ("name", json_text("Crucible")),
                ("semanticVersion", json_text(env!("CARGO_PKG_VERSION"))),
            ],
        );
        let tool = json_object(&[("driver", driver)]);
        let mut message_text = report.oracle_verdict.clone();
        message_text.push_str(" observed for ");
        message_text.push_str(&report.finding_id);
        let message = json_object(&[("text", json_text(&message_text))]);
        let properties = json_object(
            &[("facts", report_facts), ("hypotheses", json_array(Vec::new()))],
        );
        let result = json_object(
            &[
                ("ruleId", json_text(&report.kind)),
                ("level", json_text("error")),
                ("message", message),
                ("properties", properties),
            ],
        );
        let run = json_object(&[("tool", tool), ("results", json_array(vec![result]))]);
        let sarif_schema = json_text("https://json.schemastore.org/sarif-2.1.0.json");
        json_bytes(
            &json_object(
                &[
                    ("$schema", sarif_schema),
                    ("version", json_text("2.1.0")),
                    ("runs", json_array(vec![run])),
                ],
            ),
        )
    }

    fn render_junit_report(report: &FindingReport) -> Result<Vec<u8>, HostDomainReadError> {
        let id = xml_escape(&report.finding_id);
        let project = xml_escape(&report.project);
        let kind = xml_escape(&report.kind);
        let mut output = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><testsuites tests=\"1\" failures=\"1\"><testsuite name=\"Crucible\" tests=\"1\" failures=\"1\"><testcase classname=\"",
        );
        output.push_str(&project);
        output.push_str("\" name=\"");
        output.push_str(&id);
        output.push_str("\"><failure type=\"");
        output.push_str(&kind);
        output.push_str(
            "\">observed oracle failure; hypotheses: none recorded; determinism is not proven</failure></testcase></testsuite></testsuites>",
        );
        bounded(output.into_bytes())
    }

    let connection = open_database(root)?;
    let output = match action {
        HostDomainReadAction::Findings => {
            let mut statement = connection.prepare(
                "SELECT f.id, p.name, f.kind, f.status, COUNT(fi.id)
                 FROM findings f JOIN projects p ON p.id = f.project_id
                 LEFT JOIN finding_instances fi ON fi.finding_id = f.id
                 GROUP BY f.id, p.name, f.kind, f.status
                 ORDER BY f.id LIMIT 1025",
            ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
            let rows = statement.query_map(
                [],
                |row|
                    {
                        Ok(
                            (
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, String>(3)?,
                                row.get::<_, i64>(4)?,
                            ),
                        )
                    },
            ).map_err(|_| HostDomainReadError::InvalidEvidence)?;
            let mut output = Vec::new();
            let mut count = 0usize;
            for row in rows {
                if count == 1_024 {
                    return Err(HostDomainReadError::OutputLimit);
                }
                let (id, project, kind, status, instances) = row.map_err(
                    |_| HostDomainReadError::InvalidEvidence,
                )?;
                output.extend_from_slice(
                    format!("{id}\t{status}\t{kind}\t{project}\tinstances={instances}\n").as_bytes(),
                );
                count += 1;
            }
            if count == 0 {
                output.extend_from_slice(b"no findings\n");
            }
            bounded(output)?
        },
        HostDomainReadAction::Report => {
            let report = load_finding(&connection, subject)?;
            let report_facts = facts(&report);
            match format {
                ReportFormat::Human => render_human_report(&report)?,
                ReportFormat::Json => render_json_report(report_facts)?,
                ReportFormat::JsonLines => render_json_lines_report(report_facts)?,
                ReportFormat::Sarif => render_sarif_report(&report, report_facts)?,
                ReportFormat::Junit => render_junit_report(&report)?,
                ReportFormat::EvidenceGraph => render_evidence_graph(&report)?,
                ReportFormat::BundleManifest => { render_bundle_manifest(&report, report_facts)? },
            }
        },
        HostDomainReadAction::Capabilities => render_capabilities(&connection)?,
        HostDomainReadAction::Proof => render_proof_report(&connection)?,
        HostDomainReadAction::Tcb => render_tcb_report()?,
        HostDomainReadAction::Plugins => render_plugin_report(&connection)?,
    };
    connection.execute_batch("COMMIT;").map_err(|_| HostDomainReadError::Workspace)?;
    connection.close().map_err(|_| HostDomainReadError::Workspace)?;
    Ok(output)
}

// CRUCIBLE-TCB: CLI-HOST-FINDING-001
#[verifier::external_body]
fn host_finding_action(
    action: HostFindingAction,
    root: &str,
    subject: &str,
    patch: Option<&ArtifactRef>,
) -> (result: Result<Vec<u8>, HostFindingError>)
    ensures
        match &result {
            Ok(output) => output@.len() <= MAX_DOMAIN_REPORT_BYTES,
            Err(_) => true,
        },
{
    fn lexical_absolute(path: &Path) -> Result<PathBuf, HostFindingError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(|_| HostFindingError::Workspace)?.join(path)
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
                        return Err(HostFindingError::Workspace);
                    }
                },
            }
        }
        if normalized.is_absolute() {
            Ok(normalized)
        } else {
            Err(HostFindingError::Workspace)
        }
    }

    fn require_safe_directory(path: &Path) -> Result<(), HostFindingError> {
        let mut cursor = PathBuf::new();
        for component in path.components() {
            cursor.push(component.as_os_str());
            if matches!(component, Component::Prefix(_) | Component::RootDir) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&cursor).map_err(
                |_| HostFindingError::Workspace,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(HostFindingError::Workspace);
            }
        }
        Ok(())
    }

    fn open_database(root: &Path, read_only: bool) -> Result<Connection, HostFindingError> {
        require_safe_directory(root)?;
        let state = root.join(".crucible");
        require_safe_directory(&state)?;
        let database = state.join("database.sqlite");
        let metadata = std::fs::symlink_metadata(&database).map_err(
            |_| HostFindingError::Workspace,
        )?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(HostFindingError::Workspace);
        }
        let connection = if read_only {
            Connection::open_with_flags(
                database,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
        } else {
            Connection::open(database)
        }.map_err(|_| HostFindingError::Workspace)?;
        connection.set_limit(Limit::SQLITE_LIMIT_LENGTH, 1_048_576).map_err(
            |_| HostFindingError::Workspace,
        )?;
        connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(
            |_| HostFindingError::Workspace,
        )?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;").map_err(
            |_| HostFindingError::Workspace,
        )?;
        let application_id: i64 = connection.query_row(
            "PRAGMA application_id",
            [],
            |row| row.get(0),
        ).map_err(|_| HostFindingError::Workspace)?;
        let version: i64 = connection.query_row(
            "PRAGMA user_version",
            [],
            |row| row.get(0),
        ).map_err(|_| HostFindingError::Workspace)?;
        let quick: String = connection.query_row(
            "PRAGMA quick_check",
            [],
            |row| row.get(0),
        ).map_err(|_| HostFindingError::Workspace)?;
        if application_id != WORKSPACE_APPLICATION_ID || version != WORKSPACE_SCHEMA_VERSION
            || quick != "ok" {
            return Err(HostFindingError::Workspace);
        }
        Ok(connection)
    }

    fn next_sequence(transaction: &rusqlite::Transaction<'_>, name: &str) -> Result<
        i64,
        HostFindingError,
    > {
        transaction.query_row(
            "INSERT INTO id_sequences(name, next_value) VALUES (?1, 1)
             ON CONFLICT(name) DO UPDATE SET next_value = next_value + 1
             RETURNING next_value",
            [name],
            |row| row.get(0),
        ).map_err(|_| HostFindingError::Persist)
    }

    fn bounded(mut output: Vec<u8>) -> Result<Vec<u8>, HostFindingError> {
        if output.last() != Some(&b'\n') {
            output.push(b'\n');
        }
        if output.len() > MAX_DOMAIN_REPORT_BYTES {
            Err(HostFindingError::OutputLimit)
        } else {
            Ok(output)
        }
    }

    if subject.is_empty() || subject.len() > 4_096 {
        return Err(HostFindingError::NotFound);
    }
    let root = lexical_absolute(Path::new(root))?;
    match action {
        HostFindingAction::Replay => {
            let connection = open_database(&root, true)?;
            let original: (String, i64, String, Option<String>, Option<String>, String) =
                connection.query_row(
                "SELECT r.effective_configuration_artifact_id, a.size_bytes,
                            rm.environment_artifact_id, rm.generated_schedule_artifact_id,
                            rm.fault_trace_artifact_id, fi.run_attempt_id
                     FROM findings f
                     JOIN finding_instances fi ON fi.finding_id = f.id AND fi.is_original = 1
                     JOIN run_attempts ra ON ra.id = fi.run_attempt_id
                     JOIN runs r ON r.id = ra.run_id
                     JOIN artifacts a ON a.id = r.effective_configuration_artifact_id
                     JOIN run_replay_metadata rm ON rm.run_id = r.id
                     WHERE f.id = ?1 ORDER BY fi.id LIMIT 1",
                [subject],
                |row|
                    Ok(
                        (
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                        ),
                    ),
            ).optional().map_err(|_| HostFindingError::Integrity)?.ok_or(
                HostFindingError::NotFound,
            )?;
            connection.close().map_err(|_| HostFindingError::Workspace)?;
            if original.1 < 0 || original.1 as u64 > MAX_CONFIGURATION_SOURCE_BYTES {
                return Err(HostFindingError::Integrity);
            }
            let artifact_id = ArtifactId::new(original.0.clone());
            let address = object_address_for_artifact(&artifact_id).map_err(
                |_| HostFindingError::Integrity,
            )?;
            let object_directory = root.join(".crucible").join("objects").join(
                &address.algorithm,
            ).join(&address.first).join(&address.second);
            require_safe_directory(&object_directory)?;
            let object_path = object_directory.join(&address.object_name);
            let metadata = std::fs::symlink_metadata(&object_path).map_err(
                |_| HostFindingError::Integrity,
            )?;
            if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len()
                != original.1 as u64 {
                return Err(HostFindingError::Integrity);
            }
            use std::io::{Read, Write};
            let file = std::fs::File::open(&object_path).map_err(|_| HostFindingError::Integrity)?;
            let mut contents = Vec::new();
            file.take(MAX_CONFIGURATION_SOURCE_BYTES + 1).read_to_end(&mut contents).map_err(
                |_| HostFindingError::Integrity,
            )?;
            if contents.len() as u64 != original.1 as u64 || ContentDigest::from_bytes(
                &contents,
            ).map_err(|_| HostFindingError::Integrity)?.into_artifact_id().as_str() != original.0 {
                return Err(HostFindingError::Integrity);
            }
            let replay_name =
                format!(
                ".crucible-replay-{}-{}.yaml",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(
                    |_| HostFindingError::Execution,
                )?.as_nanos(),
            );
            let replay_path = root.join(replay_name);
            let mut replay_file = std::fs::OpenOptions::new().write(true).create_new(true).open(
                &replay_path,
            ).map_err(|_| HostFindingError::Execution)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                replay_file.set_permissions(std::fs::Permissions::from_mode(0o600)).map_err(
                    |_| HostFindingError::Execution,
                )?;
            }
            replay_file.write_all(&contents).and_then(|()| replay_file.sync_all()).map_err(
                |_| HostFindingError::Execution,
            )?;
            drop(replay_file);
            drop(contents);
            let executable = std::env::current_exe().map_err(|_| HostFindingError::Execution)?;
            let status_result = std::process::Command::new(executable).arg("run").arg(
                &replay_path,
            ).current_dir(&root).stdin(std::process::Stdio::null()).stdout(
                std::process::Stdio::null(),
            ).stderr(std::process::Stdio::inherit()).status();
            let removed = std::fs::remove_file(&replay_path);
            if removed.is_err() {
                return Err(HostFindingError::Execution);
            }
            let status = status_result.map_err(|_| HostFindingError::Execution)?;
            if !status.success() {
                return Err(HostFindingError::Execution);
            }
            let mut connection = open_database(&root, false)?;
            let transaction = connection.transaction_with_behavior(
                rusqlite::TransactionBehavior::Immediate,
            ).map_err(|_| HostFindingError::Persist)?;
            let replayed: (String, String, String, Option<String>, Option<String>) =
                transaction.query_row(
                "SELECT fi.run_attempt_id, ov.verdict, rm.environment_artifact_id,
                            rm.generated_schedule_artifact_id, rm.fault_trace_artifact_id
                     FROM finding_instances fi
                     JOIN run_attempts ra ON ra.id = fi.run_attempt_id
                     JOIN runs r ON r.id = ra.run_id
                     JOIN oracle_verdicts ov ON ov.attempt_id = ra.id
                     JOIN run_replay_metadata rm ON rm.run_id = r.id
                     WHERE fi.finding_id = ?1
                     ORDER BY fi.id DESC LIMIT 1",
                [subject],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).map_err(|_| HostFindingError::Integrity)?;
            if replayed.0 == original.5 {
                return Err(HostFindingError::Execution);
            }
            let observed = if replayed.1 == "fail" {
                1_i64
            } else {
                0_i64
            };
            let environment_equivalent = if replayed.2 == original.2 {
                1_i64
            } else {
                0_i64
            };
            let schedule_exact = if original.3.is_some() && replayed.3 == original.3 {
                1_i64
            } else {
                0_i64
            };
            let fault_exact = if original.4.is_some() && replayed.4 == original.4 {
                1_i64
            } else {
                0_i64
            };
            let sample_id = format!("reproduction-{subject}");
            let prior: Option<(i64, i64, i64, i64, i64)> = transaction.query_row(
                "SELECT attempt_count, observed_failures, environment_equivalent,
                        schedule_replayed_exactly, fault_trace_replayed_exactly
                 FROM reproduction_samples WHERE id = ?1",
                [sample_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).optional().map_err(|_| HostFindingError::Persist)?;
            let (attempts, failures, environments, schedules, faults) = match prior {
                Some((attempts, failures, environments, schedules, faults)) => (
                    attempts.checked_add(1).ok_or(HostFindingError::Persist)?,
                    failures.checked_add(observed).ok_or(HostFindingError::Persist)?,
                    environments.min(environment_equivalent),
                    schedules.min(schedule_exact),
                    faults.min(fault_exact),
                ),
                None => (1_i64, observed, environment_equivalent, schedule_exact, fault_exact),
            };
            if attempts > 1_000_000 {
                return Err(HostFindingError::Persist);
            }
            let classification = if failures == attempts {
                "stable-under-recorded-controls"
            } else if failures == 0 {
                "not-observed-in-sample"
            } else {
                "intermittent-under-recorded-controls"
            };
            let evidence_artifact: String = transaction.query_row(
                "SELECT ov.facts_artifact_id FROM oracle_verdicts ov
                 WHERE ov.attempt_id = ?1 ORDER BY ov.id LIMIT 1",
                [replayed.0.as_str()],
                |row| row.get(0),
            ).map_err(|_| HostFindingError::Integrity)?;
            transaction.execute(
                "INSERT INTO reproduction_samples(
                    id, finding_id, promise, attempt_count, observed_failures,
                    environment_equivalent, schedule_replayed_exactly,
                    fault_trace_replayed_exactly, classification, evidence_artifact_id
                 ) VALUES (?1, ?2, 'finding', ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                    attempt_count = excluded.attempt_count,
                    observed_failures = excluded.observed_failures,
                    environment_equivalent = excluded.environment_equivalent,
                    schedule_replayed_exactly = excluded.schedule_replayed_exactly,
                    fault_trace_replayed_exactly = excluded.fault_trace_replayed_exactly,
                    classification = excluded.classification,
                    evidence_artifact_id = excluded.evidence_artifact_id",
                params![
                    sample_id,
                    subject,
                    attempts,
                    failures,
                    environments,
                    schedules,
                    faults,
                    classification,
                    evidence_artifact,
                ],
            ).map_err(|_| HostFindingError::Persist)?;
            transaction.commit().map_err(|_| HostFindingError::Persist)?;
            connection.close().map_err(|_| HostFindingError::Persist)?;
            bounded(
                format!(
                "{subject}: {failures}/{attempts} under recorded controls ({classification}; determinism is not proven)"
            ).into_bytes(),
            )
        },
        HostFindingAction::Minimize => {
            let connection = open_database(&root, true)?;
            let finding_exists: i64 = connection.query_row(
                "SELECT COUNT(*) FROM findings WHERE id = ?1",
                [subject],
                |row| row.get(0),
            ).map_err(|_| HostFindingError::Integrity)?;
            if finding_exists != 1 {
                return Err(HostFindingError::NotFound);
            }
            let inputs: i64 = connection.query_row(
                "SELECT COUNT(*) FROM corpus_entries ce
                 JOIN finding_campaigns fc ON fc.campaign_id = ce.campaign_id
                 WHERE fc.finding_id = ?1 AND ce.state IN ('seed','interesting','coverage','regression','minimized')",
                [subject],
                |row| row.get(0),
            ).map_err(|_| HostFindingError::Integrity)?;
            connection.close().map_err(|_| HostFindingError::Workspace)?;
            if inputs == 0 {
                Err(HostFindingError::NoMinimizableInput)
            } else {
                Err(HostFindingError::Execution)
            }
        },
        HostFindingAction::RegisterPatch => {
            let patch = patch.ok_or(HostFindingError::Integrity)?;
            let mut connection = open_database(&root, false)?;
            let transaction = connection.transaction_with_behavior(
                rusqlite::TransactionBehavior::Immediate,
            ).map_err(|_| HostFindingError::Persist)?;
            let finding: Option<String> = transaction.query_row(
                "SELECT rm.campaign_seed
                 FROM findings f
                 JOIN finding_instances fi ON fi.finding_id = f.id AND fi.is_original = 1
                 JOIN run_attempts ra ON ra.id = fi.run_attempt_id
                 JOIN run_replay_metadata rm ON rm.run_id = ra.run_id
                 WHERE f.id = ?1 ORDER BY fi.id LIMIT 1",
                [subject],
                |row| row.get(0),
            ).optional().map_err(|_| HostFindingError::Integrity)?;
            let seed = finding.ok_or(HostFindingError::NotFound)?;
            let patch_sequence = next_sequence(&transaction, "patch")?;
            let verification_sequence = next_sequence(&transaction, "verification-run")?;
            if patch_sequence <= 0 || verification_sequence <= 0 {
                return Err(HostFindingError::Persist);
            }
            let patch_id = format!("PATCH-{patch_sequence:05}");
            let verification_id = format!("verification-{verification_sequence:020}");
            transaction.execute(
                "INSERT INTO patches(id, finding_id, patch_artifact_id, producer_identity, status)
                 VALUES (?1, ?2, ?3, 'crucible-cli-submission-v1', 'candidate')",
                params![patch_id, subject, patch.id.as_str()],
            ).map_err(|_| HostFindingError::Persist)?;
            transaction.execute(
                "INSERT INTO verification_runs(id, patch_id, status, evidence_artifact_id, seed)
                 VALUES (?1, ?2, 'inconclusive', ?3, ?4)",
                params![verification_id, patch_id, patch.id.as_str(), seed],
            ).map_err(|_| HostFindingError::Persist)?;
            let generation: i64 = transaction.query_row(
                "SELECT id FROM storage_generations WHERE status = 'open'
                 ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            ).map_err(|_| HostFindingError::Persist)?;
            transaction.execute(
                "INSERT INTO artifact_roots(artifact_id, root_kind, root_id, generation_id)
                 VALUES (?1, 'manual', ?2, ?3)
                 ON CONFLICT(artifact_id, root_kind, root_id) DO NOTHING",
                params![patch.id.as_str(), patch_id, generation],
            ).map_err(|_| HostFindingError::Persist)?;
            transaction.commit().map_err(|_| HostFindingError::Persist)?;
            connection.close().map_err(|_| HostFindingError::Persist)?;
            Err(HostFindingError::VerificationInconclusive)
        },
    }
}

// CRUCIBLE-TCB: CLI-HOST-LOG-001
#[verifier::external_body]
#[expect(
    clippy::too_many_arguments,
    reason = "the structured event boundary receives every correlation identity explicitly"
)]
fn host_structured_log(
    action: HostLogAction,
    event: &str,
    campaign_id: &str,
    engine_id: &str,
    experiment_id: &str,
    run_id: &str,
    run_attempt_id: &str,
    target_id: &str,
    target_build_id: &str,
    worker_id: &str,
    finding_id: &str,
) {
    match action {
        HostLogAction::Initialize => {
            if std::env::var_os("CRUCIBLE_LOG").as_deref() == Some(std::ffi::OsStr::new("json")) {
                let initialized = tracing_subscriber::fmt().json().with_ansi(false).with_target(
                    true,
                ).with_writer(std::io::stderr).with_max_level(tracing::Level::INFO).try_init();
                if initialized.is_ok() {
                    tracing::event!(
                        target: "crucible",
                        tracing::Level::INFO,
                        event = "process-started",
                        severity = "INFO",
                    );
                }
            }
        },
        HostLogAction::Event => {
            tracing::event!(
                target: "crucible",
                tracing::Level::INFO,
                event = event,
                campaign_id = campaign_id,
                engine_id = engine_id,
                experiment_id = experiment_id,
                run_id = run_id,
                run_attempt_id = run_attempt_id,
                scenario_id = "",
                scenario_step_id = "",
                participant_id = "",
                target_id = target_id,
                target_build_id = target_build_id,
                worker_id = worker_id,
                finding_id = finding_id,
                proof_artifact_id = "",
                trusted_boundary_id = "",
                severity = "INFO",
            );
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
        | Ok(InitializationDecision::MigrateV2)
        | Ok(InitializationDecision::MigrateV3) => Err(InitCommandError::Publish),
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
        Ok(InitializationDecision::MigrateV3) => {
            complete_workspace_migration(root, HostWorkspaceAction::MigrateV3)
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
                | Ok(InitializationDecision::MigrateV2)
                | Ok(InitializationDecision::MigrateV3) => Err(InitCommandError::Publish),
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
        Ok(InitializationDecision::MigrateV1)
        | Ok(InitializationDecision::MigrateV2)
        | Ok(InitializationDecision::MigrateV3) => { initialize_workspace(root).is_ok() },
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

fn inspection_artifact_snapshot(root: &str, artifact: &ArtifactRef) -> (result: Result<
    StoredArtifactSnapshot,
    InspectionCommandError,
>) {
    let address = object_address_for_artifact(&artifact.id).map_err(
        |_error| InspectionCommandError::ArtifactIntegrity,
    )?;
    match host_artifact_action(
        HostArtifactAction::Load,
        root,
        "",
        Some(&address),
        Some(artifact),
        &[],
    ) {
        Ok(HostArtifactOutcome::Snapshot(snapshot)) => Ok(snapshot),
        Ok(HostArtifactOutcome::Source(_, _)) | Ok(HostArtifactOutcome::Published) => {
            Err(InspectionCommandError::ArtifactIntegrity)
        },
        Err(HostArtifactError::Workspace) => Err(InspectionCommandError::Workspace),
        Err(HostArtifactError::UnsafeSource)
        | Err(HostArtifactError::TooLarge)
        | Err(HostArtifactError::Publish)
        | Err(HostArtifactError::Load) => Err(InspectionCommandError::ArtifactIntegrity),
    }
}

fn inspection_preview(root: &str, artifact: &ArtifactRef) -> (result: Result<
    crucible_cli::AuthenticatedArtifactPreview,
    InspectionCommandError,
>) {
    let snapshot = inspection_artifact_snapshot(root, artifact)?;
    authenticate_artifact_preview(artifact, snapshot).map_err(
        |error|
            match error {
                InspectionArtifactError::TooLarge | InspectionArtifactError::Integrity => {
                    InspectionCommandError::ArtifactIntegrity
                },
            },
    )
}

fn authenticate_inspection_artifact(root: &str, artifact: &ArtifactRef, limit: u64) -> Result<
    Vec<u8>,
    InspectionCommandError,
> {
    if artifact.size_bytes > limit {
        return Err(InspectionCommandError::ArtifactIntegrity);
    }
    let snapshot = inspection_artifact_snapshot(root, artifact)?;
    authenticate_artifact_contents(artifact, snapshot, limit).map_err(
        |error|
            match error {
                InspectionArtifactError::TooLarge | InspectionArtifactError::Integrity => {
                    InspectionCommandError::ArtifactIntegrity
                },
            },
    )
}

fn inspect_run(run_id: &str, root: &str) -> (result: Result<Vec<u8>, InspectionCommandError>) {
    if !artifact_workspace_is_ready(root) {
        return Err(InspectionCommandError::Workspace);
    }
    let host_snapshot = host_inspection_snapshot(root, run_id).map_err(
        |error|
            match error {
                HostInspectionError::Workspace => InspectionCommandError::Workspace,
                HostInspectionError::NotFound => InspectionCommandError::NotFound,
                HostInspectionError::InvalidEvidence => InspectionCommandError::InvalidEvidence,
            },
    )?;
    let inspection = validate_run_inspection(run_id, host_snapshot).map_err(
        |error|
            match error {
                InspectionValidationError::IdentityMismatch
                | InspectionValidationError::InvalidMetadata
                | InspectionValidationError::InvalidState => InspectionCommandError::InvalidEvidence,
            },
    )?;
    let snapshot = inspection.snapshot();

    let configuration_source = inspection_preview(root, &snapshot.configuration_source)?;
    let effective_configuration = inspection_preview(root, &snapshot.effective_configuration)?;
    {
        let _capability = authenticate_inspection_artifact(
            root,
            &snapshot.capability_manifest,
            MAX_LOCAL_ARTIFACT_BYTES,
        )?;
    }
    if let Some(target) = &snapshot.target {
        {
            let _target_contents = authenticate_inspection_artifact(
                root,
                &target.target_artifact,
                MAX_LOCAL_ARTIFACT_BYTES,
            )?;
        }
        {
            let _manifest_contents = authenticate_inspection_artifact(
                root,
                &target.manifest_artifact,
                MAX_LOCAL_ARTIFACT_BYTES,
            )?;
        }
    }
    if let Some(failure) = &snapshot.harness_failure {
        if let Some(artifact) = &failure.detail_artifact {
            let _detail = authenticate_inspection_artifact(
                root,
                artifact,
                MAX_LOCAL_ARTIFACT_BYTES,
            )?;
        }
    }
    let (decoded_observation, stdout, stderr) = match &snapshot.observation {
        Some(record) => {
            let encoded = authenticate_inspection_artifact(
                root,
                &record.artifact,
                MAX_INSPECTION_OBSERVATION_BYTES,
            )?;
            let decoded = decode_raw_observation(
                encoded,
                inspection_observation_codec_limits(),
            ).map_err(|_error| InspectionCommandError::Observation)?;
            let stdout = inspection_preview(root, &record.stdout_artifact)?;
            let stderr = inspection_preview(root, &record.stderr_artifact)?;
            (Some(decoded), Some(stdout), Some(stderr))
        },
        None => (None, None, None),
    };
    let previews = InspectionPreviews {
        configuration_source,
        effective_configuration,
        stdout,
        stderr,
    };
    render_run_inspection_report(&inspection, decoded_observation.as_ref(), &previews).map_err(
        |error|
            match error {
                InspectionReportError::EvidenceMismatch => InspectionCommandError::InvalidEvidence,
                InspectionReportError::ReportTooLarge => InspectionCommandError::Report,
            },
    )
}

fn complete_inspection_error(error: InspectionCommandError) {
    let message: &[u8] = match error {
        InspectionCommandError::Workspace => {
            b"crucible inspect: workspace is missing, incompatible, or unsafe\n"
        },
        InspectionCommandError::NotFound => b"crucible inspect: run was not found\n",
        InspectionCommandError::InvalidEvidence => {
            b"crucible inspect: persisted evidence is inconsistent\n"
        },
        InspectionCommandError::ArtifactIntegrity => {
            b"crucible inspect: artifact integrity failure\n"
        },
        InspectionCommandError::Observation => { b"crucible inspect: observation decoding failed\n"
        },
        InspectionCommandError::Report => b"crucible inspect: report limit exceeded\n",
    };
    host_complete(false, message)
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

fn prefixed_identifier(prefix: &str, value: &str) -> (result: String) {
    let mut result = String::new();
    result.push_str(prefix);
    result.push_str(value);
    result
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

fn storage_maintenance_output(report: StorageMaintenanceReport) -> (output: Vec<u8>) {
    let mut output = Vec::new();
    append_bytes(&mut output, b"verified=");
    append_decimal_u64(&mut output, report.verified);
    append_bytes(&mut output, b" orphaned=");
    append_decimal_u64(&mut output, report.orphaned);
    append_bytes(&mut output, b" temporary=");
    append_decimal_u64(&mut output, report.temporary);
    append_bytes(&mut output, b" collected=");
    append_decimal_u64(&mut output, report.collected);
    append_bytes(&mut output, b" preserved=");
    append_decimal_u64(&mut output, report.preserved);
    output.push(b'\n');
    output
}

fn run_storage_maintenance(root: &str, action: HostStorageAction) {
    match host_storage_maintenance(root, action) {
        Ok(report) => {
            let output = storage_maintenance_output(report);
            host_complete(true, output.as_slice());
        },
        Err(HostStorageError::UnsafeWorkspace) => {
            host_complete(false, b"crucible artifact: unsafe workspace\n");
        },
        Err(HostStorageError::Integrity) => {
            host_complete(false, b"crucible artifact: storage integrity failure\n");
        },
        Err(HostStorageError::WorkLimit) => {
            host_complete(false, b"crucible artifact: storage scan exceeds the 4096-entry limit\n");
        },
        Err(HostStorageError::ActiveLease) => {
            host_complete(false, b"crucible artifact gc: active publication lease\n");
        },
        Err(HostStorageError::Persistence) => {
            host_complete(false, b"crucible artifact: storage maintenance failed\n");
        },
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

fn record_build(
    root: &str,
    effective_configuration: &ArtifactRef,
    configuration_digest: &str,
    capability_manifest: &ArtifactRef,
    target: &ArtifactRef,
    target_manifest: &ArtifactRef,
    project_name: &[u32],
) -> (result: Result<(), RunCommandError>) {
    match host_run_store_action(
        HostRunStoreAction::RecordBuild,
        root,
        None,
        None,
        Some(effective_configuration),
        configuration_digest,
        Some(capability_manifest),
        None,
        0,
        Some(target),
        Some(target_manifest),
        None,
        None,
        None,
        0,
        0,
        "",
        project_name,
        PersistenceRetentionPolicy::ManagedReplay,
        LocalOracleVerdict::Pass,
    ) {
        Ok(HostRunStoreOutcome::Updated) => Ok(()),
        Ok(HostRunStoreOutcome::Reserved(_)) | Err(_) => Err(RunCommandError::Persistence),
    }
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
        &[],
        PersistenceRetentionPolicy::ManagedReplay,
        LocalOracleVerdict::Pass,
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
        &[],
        PersistenceRetentionPolicy::ManagedReplay,
        LocalOracleVerdict::Pass,
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
        &[],
        PersistenceRetentionPolicy::ManagedReplay,
        LocalOracleVerdict::Pass,
    ) {
        Ok(HostRunStoreOutcome::Updated) => Ok(()),
        Ok(HostRunStoreOutcome::Reserved(_)) | Err(_) => Err(RunCommandError::Persistence),
    }
}

fn record_fuzz_success(root: &str, reservation: &ReservedRun, project_name: &[u32]) -> Result<
    (),
    RunCommandError,
> {
    match host_run_store_action(
        HostRunStoreAction::AggregateSuccess,
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
        "",
        project_name,
        PersistenceRetentionPolicy::HighThroughput,
        LocalOracleVerdict::Pass,
    ) {
        Ok(HostRunStoreOutcome::Updated) => Ok(()),
        Ok(HostRunStoreOutcome::Reserved(_)) | Err(_) => Err(RunCommandError::Persistence),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "this boundary forwards one complete authenticated observation without an intermediate duplicate"
)]
fn record_run_observation(
    root: &str,
    reservation: &ReservedRun,
    observation: &ArtifactRef,
    stdout: &ArtifactRef,
    stderr: &ArtifactRef,
    completion_tag: u16,
    termination_tag: u16,
    project_name: &[u32],
    retention_policy: PersistenceRetentionPolicy,
    oracle_verdict: LocalOracleVerdict,
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
        project_name,
        retention_policy,
        oracle_verdict,
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

fn campaign_id_output(reservation: &ReservedRun) -> (output: Vec<u8>) {
    let mut output = Vec::new();
    append_bytes(&mut output, b"campaign-");
    append_bytes(&mut output, reservation.run_id().as_str().as_bytes_vec().as_slice());
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

fn complete_build_error(error: RunCommandError) {
    let message: &[u8] = match error {
        RunCommandError::Workspace => {
            b"crucible build: workspace is missing, incompatible, or unsafe\n"
        },
        RunCommandError::Artifact => b"crucible build: artifact publication failed\n",
        RunCommandError::TargetPreparation => b"crucible build: target preparation failed\n",
        RunCommandError::CapabilityUnavailable => {
            b"crucible build: required build capability is unavailable\n"
        },
        RunCommandError::Persistence => b"crucible build: build persistence failed\n",
        RunCommandError::Execution | RunCommandError::Observation => {
            b"crucible build: build identity construction failed\n"
        },
    };
    host_complete(false, message);
}

fn build_local_configuration(path: &str) {
    let contents = match host_read_configuration(path) {
        Ok(value) => value,
        Err(HostConfigError::UnsafeSource) => {
            host_complete(false, b"crucible build: unsafe configuration source\n");
            return;
        },
        Err(HostConfigError::TooLarge) => {
            host_complete(false, b"crucible build: configuration exceeds the source limit\n");
            return;
        },
        Err(HostConfigError::Read) => {
            host_complete(false, b"crucible build: could not read configuration\n");
            return;
        },
        #[cfg(not(unix))]
        Err(HostConfigError::UnsupportedPlatform) => {
            complete_build_error(RunCommandError::CapabilityUnavailable);
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
        Err(_) => {
            complete_build_error(RunCommandError::CapabilityUnavailable);
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
        Ok(_) | Err(_) => {
            complete_build_error(RunCommandError::Workspace);
            return;
        },
    };
    if !artifact_workspace_is_ready(root.as_str()) {
        complete_build_error(RunCommandError::Workspace);
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
        Ok(_) | Err(_) => {
            complete_build_error(RunCommandError::CapabilityUnavailable);
            return;
        },
    };
    let probe = match validate_local_capability_probe(&plan, probe_report) {
        Ok(value) if value.available() => value,
        Ok(_) | Err(_) => {
            complete_build_error(RunCommandError::CapabilityUnavailable);
            return;
        },
    };
    let host_runtime = match host_runtime {
        Some(runtime) => runtime,
        None => {
            complete_build_error(RunCommandError::CapabilityUnavailable);
            return;
        },
    };
    let probe_publication = match publish_run_generated_artifact(root.as_str(), probe.report()) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
            return;
        },
    };
    let source_publication = match prepare_run_artifact(contents.as_slice()) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
            return;
        },
    };
    if publish_artifact(root.as_str(), path, &source_publication, contents.as_slice()).is_err() {
        complete_build_error(RunCommandError::Artifact);
        return;
    }
    let effective_publication = match publish_run_generated_artifact(
        root.as_str(),
        validated.canonical_bytes(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
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
            complete_build_error(error);
            return;
        },
    };
    let harness_publication = match publish_run_generated_artifact(
        root.as_str(),
        host_runtime.harness_contents.as_slice(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
            return;
        },
    };
    let bubblewrap_publication = match publish_run_generated_artifact(
        root.as_str(),
        host_runtime.bubblewrap_contents.as_slice(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
            return;
        },
    };
    let prlimit_publication = match publish_run_generated_artifact(
        root.as_str(),
        host_runtime.prlimit_contents.as_slice(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
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
            complete_build_error(RunCommandError::Execution);
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
        Ok(_) | Err(_) => {
            complete_build_error(RunCommandError::TargetPreparation);
            return;
        },
    };
    let target_publication = match prepare_run_artifact(target_contents.as_slice()) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
            return;
        },
    };
    if publish_artifact(
        root.as_str(),
        target_provenance.as_str(),
        &target_publication,
        target_contents.as_slice(),
    ).is_err() {
        complete_build_error(RunCommandError::Artifact);
        return;
    }
    let target_manifest_bytes = target_build_manifest(
        &target_publication.artifact,
        &runtime_identity,
    );
    let target_manifest_publication = match publish_run_generated_artifact(
        root.as_str(),
        target_manifest_bytes.as_slice(),
    ) {
        Ok(value) => value,
        Err(error) => {
            complete_build_error(error);
            return;
        },
    };
    let configuration_identity = ContentDigest::Sha256(validated.digest()).into_artifact_id();
    if record_build(
        root.as_str(),
        &effective_publication.artifact,
        configuration_identity.as_str(),
        &capability_publication.artifact,
        &target_publication.artifact,
        &target_manifest_publication.artifact,
        validated.execution().project_name(),
    ).is_err() {
        complete_build_error(RunCommandError::Persistence);
        return;
    }
    let target_build_id = prefixed_identifier(
        "target-build-",
        target_manifest_publication.artifact.id.as_str(),
    );
    let target_id = prefixed_identifier("target-", target_build_id.as_str());
    host_structured_log(
        HostLogAction::Event,
        "build-completed",
        "",
        "local-cli",
        "",
        "",
        "",
        target_id.as_str(),
        target_build_id.as_str(),
        "local-process",
        "",
    );
    let mut output = Vec::new();
    append_bytes(&mut output, b"target-build-");
    append_bytes(
        &mut output,
        target_manifest_publication.artifact.id.as_str().as_bytes_vec().as_slice(),
    );
    output.push(b'\n');
    host_complete(true, output.as_slice());
}

fn run_local_configuration(path: &str, retention_policy: PersistenceRetentionPolicy) {
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
    let oracle_verdict = evaluate_process_exit_oracle(
        evidence.termination(),
        validated.execution().allowed_exit_codes(),
        validated.execution().timeout_is_failure(),
    );
    if retention_policy == PersistenceRetentionPolicy::HighThroughput && oracle_verdict
        == LocalOracleVerdict::Pass {
        if record_fuzz_success(
            root.as_str(),
            &reservation,
            validated.execution().project_name(),
        ).is_err() {
            persist_failure_then_complete(
                root.as_str(),
                &reservation,
                "EvidencePersistence",
                RunCommandError::Persistence,
            );
            return;
        }
        let output = campaign_id_output(&reservation);
        let campaign_id = prefixed_identifier("campaign-", reservation.run_id().as_str());
        let experiment_id = prefixed_identifier("experiment-", reservation.run_id().as_str());
        let target_build_id = prefixed_identifier(
            "target-build-",
            target_manifest_publication.artifact.id.as_str(),
        );
        let target_id = prefixed_identifier("target-", target_build_id.as_str());
        host_structured_log(
            HostLogAction::Event,
            "fuzz-success-aggregated",
            campaign_id.as_str(),
            "coverage-fuzzing",
            experiment_id.as_str(),
            reservation.run_id().as_str(),
            reservation.attempt_id().as_str(),
            target_id.as_str(),
            target_build_id.as_str(),
            "local-process",
            "",
        );
        host_complete(true, output.as_slice());
        return;
    }
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
        validated.execution().project_name(),
        retention_policy,
        oracle_verdict,
    ).is_err() {
        persist_failure_then_complete(
            root.as_str(),
            &reservation,
            "EvidencePersistence",
            RunCommandError::Persistence,
        );
        return;
    }
    let campaign_id = prefixed_identifier("campaign-", reservation.run_id().as_str());
    let experiment_id = prefixed_identifier("experiment-", reservation.run_id().as_str());
    let target_build_id = prefixed_identifier(
        "target-build-",
        target_manifest_publication.artifact.id.as_str(),
    );
    let target_id = prefixed_identifier("target-", target_build_id.as_str());
    host_structured_log(
        HostLogAction::Event,
        "run-observed",
        campaign_id.as_str(),
        "local-cli",
        experiment_id.as_str(),
        reservation.run_id().as_str(),
        reservation.attempt_id().as_str(),
        target_id.as_str(),
        target_build_id.as_str(),
        "local-process",
        "",
    );
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

fn complete_domain_read_error(command: &[u8], error: HostDomainReadError) {
    let detail: &[u8] = match error {
        HostDomainReadError::Workspace => b": workspace is missing, incompatible, or unsafe\n",
        HostDomainReadError::NotFound => b": finding not found\n",
        HostDomainReadError::InvalidEvidence => b": stored evidence is invalid\n",
        HostDomainReadError::OutputLimit => b": report exceeds the bounded output limit\n",
    };
    let mut output = Vec::new();
    append_bytes(&mut output, b"crucible ");
    append_bytes(&mut output, command);
    append_bytes(&mut output, detail);
    host_complete(false, output.as_slice());
}

fn run_domain_read(
    action: HostDomainReadAction,
    root: &str,
    subject: &str,
    format: ReportFormat,
    command: &[u8],
) {
    match host_domain_read(action, root, subject, format) {
        Ok(output) => host_complete(true, output.as_slice()),
        Err(error) => complete_domain_read_error(command, error),
    }
}

fn complete_finding_error(action: HostFindingAction, error: HostFindingError) {
    let message: &[u8] = match error {
        HostFindingError::Workspace => {
            b"crucible: workspace is missing, incompatible, or unsafe\n"
        },
        HostFindingError::NotFound => b"crucible: finding not found\n",
        HostFindingError::Integrity => b"crucible: finding evidence integrity failure\n",
        HostFindingError::Execution => match action {
            HostFindingAction::Replay => b"crucible replay: recorded execution failed\n",
            HostFindingAction::Minimize => {
                b"crucible minimize: recorded input delivery cannot preserve the predicate\n"
            },
            HostFindingAction::RegisterPatch => {
                b"crucible verify: verification execution failed\n"
            },
        },
        HostFindingError::NoMinimizableInput => {
            b"crucible minimize: finding has no minimizable input artifact\n"
        },
        HostFindingError::Persist => b"crucible: finding evidence persistence failed\n",
        HostFindingError::VerificationInconclusive => {
            b"crucible verify: verification inconclusive: finding has no recorded source snapshot and build recipe\n"
        },
        HostFindingError::OutputLimit => b"crucible: command output exceeds the bounded limit\n",
    };
    host_complete(false, message);
}

fn run_finding_action(action: HostFindingAction, root: &str, subject: &str) {
    match host_finding_action(action, root, subject, None) {
        Ok(output) => {
            host_structured_log(
                HostLogAction::Event,
                "finding-operation-completed",
                "",
                "local-cli",
                "",
                "",
                "",
                "",
                "",
                "local-process",
                subject,
            );
            host_complete(true, output.as_slice());
        },
        Err(error) => complete_finding_error(action, error),
    }
}

fn verify_finding(subject: &str, patch_path: &str, root: &str) {
    if let Err(error) = host_domain_read(
        HostDomainReadAction::Report,
        root,
        subject,
        ReportFormat::Human,
    ) {
        let mapped = match error {
            HostDomainReadError::NotFound => HostFindingError::NotFound,
            HostDomainReadError::Workspace => HostFindingError::Workspace,
            HostDomainReadError::InvalidEvidence => HostFindingError::Integrity,
            HostDomainReadError::OutputLimit => HostFindingError::OutputLimit,
        };
        complete_finding_error(HostFindingAction::RegisterPatch, mapped);
        return;
    }
    let publication = match import_artifact(patch_path, root) {
        Ok(publication) => publication,
        Err(error) => {
            complete_artifact_error(error);
            return;
        },
    };
    match host_finding_action(
        HostFindingAction::RegisterPatch,
        root,
        subject,
        Some(&publication.artifact),
    ) {
        Ok(output) => host_complete(true, output.as_slice()),
        Err(error) => complete_finding_error(HostFindingAction::RegisterPatch, error),
    }
}

fn complete_usage() {
    host_complete(
        false,
        b"usage: crucible init [path]\n\
       crucible artifact import <file> [workspace]\n\
       crucible artifact verify <artifact-id> [workspace]\n\
       crucible artifact check [workspace]\n\
       crucible artifact gc [workspace]\n\
       crucible build <configuration>\n\
       crucible run <configuration>\n\
       crucible fuzz <configuration>\n\
       crucible replay <finding-id> [workspace]\n\
       crucible minimize <finding-id> [workspace]\n\
       crucible findings [workspace]\n\
       crucible inspect <run-id> [workspace]\n\
       crucible verify <finding-id> --patch <file> [workspace]\n\
       crucible report <finding-id> --format <human|json|jsonl|sarif|junit|evidence|bundle> [workspace]\n\
       crucible config validate <file>\n\
       crucible config canonicalize <file>\n\
       crucible capabilities [workspace]\n\
       crucible proof [workspace]\n\
       crucible tcb [workspace]\n\
       crucible plugins [workspace]\n",
    );
}

fn main() {
    host_structured_log(HostLogAction::Initialize, "", "", "", "", "", "", "", "", "", "");
    let arguments = match host_cli_args() {
        Ok(arguments) => arguments,
        Err(HostArgumentError::NonUtf8) => {
            host_complete(false, b"crucible: path is not valid UTF-8\n");
            return;
        },
        Err(HostArgumentError::TooMany) => {
            complete_usage();
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
            complete_usage();
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
        CliAction::Build(path) => build_local_configuration(path.as_str()),
        CliAction::Run(path) => {
            run_local_configuration(path.as_str(), PersistenceRetentionPolicy::ManagedReplay)
        },
        CliAction::Fuzz(path) => {
            run_local_configuration(path.as_str(), PersistenceRetentionPolicy::HighThroughput)
        },
        CliAction::Replay(subject, root) => run_finding_action(
            HostFindingAction::Replay,
            root.as_str(),
            subject.as_str(),
        ),
        CliAction::Minimize(subject, root) => run_finding_action(
            HostFindingAction::Minimize,
            root.as_str(),
            subject.as_str(),
        ),
        CliAction::VerifyFinding(subject, patch, root) => verify_finding(
            subject.as_str(),
            patch.as_str(),
            root.as_str(),
        ),
        CliAction::Findings(root) => run_domain_read(
            HostDomainReadAction::Findings,
            root.as_str(),
            "",
            ReportFormat::Human,
            b"findings",
        ),
        CliAction::Report(subject, format, root) => run_domain_read(
            HostDomainReadAction::Report,
            root.as_str(),
            subject.as_str(),
            format,
            b"report",
        ),
        CliAction::Capabilities(root) => run_domain_read(
            HostDomainReadAction::Capabilities,
            root.as_str(),
            "",
            ReportFormat::Json,
            b"capabilities",
        ),
        CliAction::Proof(root) => run_domain_read(
            HostDomainReadAction::Proof,
            root.as_str(),
            "",
            ReportFormat::Json,
            b"proof",
        ),
        CliAction::Tcb(root) => run_domain_read(
            HostDomainReadAction::Tcb,
            root.as_str(),
            "",
            ReportFormat::Json,
            b"tcb",
        ),
        CliAction::Plugins(root) => run_domain_read(
            HostDomainReadAction::Plugins,
            root.as_str(),
            "",
            ReportFormat::Json,
            b"plugins",
        ),
        CliAction::ArtifactCheck(root) => {
            run_storage_maintenance(root.as_str(), HostStorageAction::Check)
        },
        CliAction::ArtifactGc(root) => {
            run_storage_maintenance(root.as_str(), HostStorageAction::Collect)
        },
        CliAction::Inspect(run_id, root) => match inspect_run(run_id.as_str(), root.as_str()) {
            Ok(report) => host_complete(true, report.as_slice()),
            Err(error) => complete_inspection_error(error),
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
        CliAction::ConfigValidate(path) => run_configuration(path.as_str(), false),
        CliAction::ConfigCanonicalize(path) => run_configuration(path.as_str(), true),
    }
}

} // verus!
