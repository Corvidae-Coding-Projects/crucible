use crucible_cli::{
    database_snapshot_is_exact, database_snapshot_is_exact_v1, decide_workspace_initialization,
    DatabaseSnapshot, InitializationDecision, InitializationError, MigrationRecord, PathKind,
    WorkspaceMetadata, WorkspaceSnapshot,
};
#[expect(
    unused_imports,
    reason = "Verus requires the prelude crate marker even when this Rust test names no vstd item"
)]
use vstd::prelude::*;

fn exact_database() -> DatabaseSnapshot {
    DatabaseSnapshot {
        application_id: 0x4352_5543,
        schema_version: 2,
        journal_mode: b"wal".to_vec(),
        synchronous: 2,
        quick_check: b"ok".to_vec(),
        schema_object_count: 4,
        migrations_table_kind: b"table".to_vec(),
        migrations_table_sql: crucible_cli::migration_table_sql(),
        metadata_table_kind: b"table".to_vec(),
        metadata_table_sql: crucible_cli::metadata_table_sql(),
        artifacts_table_kind: b"table".to_vec(),
        artifacts_table_sql: crucible_cli::artifacts_table_sql(),
        artifact_imports_table_kind: b"table".to_vec(),
        artifact_imports_table_sql: crucible_cli::artifact_imports_table_sql(),
        migration_row_count: 2,
        migrations: vec![
            MigrationRecord {
                version: 1,
                name: b"initialize-workspace".to_vec(),
                checksum: crucible_cli::migration_checksum(),
            },
            MigrationRecord {
                version: 2,
                name: b"add-artifact-store".to_vec(),
                checksum: crucible_cli::artifact_migration_checksum(),
            },
        ],
        metadata_row_count: 1,
        metadata: Some(WorkspaceMetadata {
            key: b"format".to_vec(),
            value: b"crucible-workspace-v1".to_vec(),
        }),
    }
}

fn exact_version_one_database() -> DatabaseSnapshot {
    let mut database = exact_database();
    database.schema_version = 1;
    database.schema_object_count = 2;
    database.artifacts_table_kind.clear();
    database.artifacts_table_sql.clear();
    database.artifact_imports_table_kind.clear();
    database.artifact_imports_table_sql.clear();
    database.migration_row_count = 1;
    database.migrations.pop();
    database
}

fn exact_workspace(database: Option<DatabaseSnapshot>) -> WorkspaceSnapshot {
    WorkspaceSnapshot {
        root_kind: PathKind::Directory,
        state_kind: PathKind::Directory,
        state_entry_count: 8,
        corpus_kind: PathKind::Directory,
        corpus_entry_count: 5,
        seeds_kind: PathKind::Directory,
        interesting_kind: PathKind::Directory,
        coverage_kind: PathKind::Directory,
        regression_kind: PathKind::Directory,
        minimized_kind: PathKind::Directory,
        findings_kind: PathKind::Directory,
        objects_kind: PathKind::Directory,
        runs_kind: PathKind::Directory,
        reports_kind: PathKind::Directory,
        database_kind: PathKind::File,
        database_wal_kind: PathKind::File,
        database_shm_kind: PathKind::File,
        database,
    }
}

#[test]
fn exact_policy_distinguishes_create_reuse_and_occupied_state() {
    let mut missing = exact_workspace(None);
    missing.state_kind = PathKind::Missing;
    missing.state_entry_count = 0;
    missing.corpus_kind = PathKind::Missing;
    missing.corpus_entry_count = 0;
    missing.database_kind = PathKind::Missing;
    missing.database_wal_kind = PathKind::Missing;
    missing.database_shm_kind = PathKind::Missing;
    assert_eq!(
        decide_workspace_initialization(&missing),
        Ok(InitializationDecision::Create)
    );

    let existing = exact_workspace(Some(exact_database()));
    assert_eq!(
        decide_workspace_initialization(&existing),
        Ok(InitializationDecision::Reuse)
    );

    let version_one = exact_workspace(Some(exact_version_one_database()));
    assert!(database_snapshot_is_exact_v1(
        version_one.database.as_ref().expect("version-one database")
    ));
    assert_eq!(
        decide_workspace_initialization(&version_one),
        Ok(InitializationDecision::MigrateV1)
    );

    let mut occupied = exact_workspace(None);
    occupied.state_entry_count = 1;
    occupied.database_kind = PathKind::Missing;
    assert_eq!(
        decide_workspace_initialization(&occupied),
        Err(InitializationError::OccupiedState)
    );
}

#[test]
fn exact_policy_rejects_database_laundering() {
    let exact = exact_database();
    assert!(database_snapshot_is_exact(&exact));

    let mut extra_migration = exact_database();
    extra_migration.migration_row_count = 3;
    extra_migration.migrations.push(MigrationRecord {
        version: 3,
        name: b"foreign".to_vec(),
        checksum: b"foreign".to_vec(),
    });
    assert!(!database_snapshot_is_exact(&extra_migration));

    let mut extra_metadata = exact_database();
    extra_metadata.metadata_row_count = 2;
    extra_metadata.metadata = Some(WorkspaceMetadata {
        key: b"foreign".to_vec(),
        value: b"state".to_vec(),
    });
    assert!(!database_snapshot_is_exact(&extra_metadata));

    let mut foreign_schema = exact_database();
    foreign_schema.schema_object_count = 3;
    assert!(!database_snapshot_is_exact(&foreign_schema));

    let mut forged_view = exact_database();
    forged_view.migrations_table_kind = b"view".to_vec();
    assert!(!database_snapshot_is_exact(&forged_view));
}

#[test]
fn artifact_migration_checksum_authenticates_the_exact_script() {
    let script = crucible_cli::artifact_migration_sql();
    let digest = crucible_core::sha256(&script).expect("migration script is bounded");
    let expected = format!("sha256:{}", digest.to_hex());
    assert_eq!(
        String::from_utf8(crucible_cli::artifact_migration_checksum()).expect("checksum is UTF-8"),
        expected
    );
}

#[test]
fn path_and_layout_failures_have_stable_precedence() {
    let mut snapshot = exact_workspace(Some(exact_database()));
    snapshot.root_kind = PathKind::Symlink;
    snapshot.state_entry_count = 99;
    assert_eq!(
        decide_workspace_initialization(&snapshot),
        Err(InitializationError::UnsafeRoot)
    );

    snapshot.root_kind = PathKind::Directory;
    snapshot.state_kind = PathKind::Symlink;
    assert_eq!(
        decide_workspace_initialization(&snapshot),
        Err(InitializationError::OccupiedState)
    );
}
