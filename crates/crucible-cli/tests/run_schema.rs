use crucible_cli::{
    run_migration_checksum, run_migration_name, run_migration_sql, WORKSPACE_SCHEMA_VERSION,
};
use rusqlite::Connection;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[test]
fn workspace_schema_four_preserves_the_versioned_run_evidence_tables() {
    assert_eq!(WORKSPACE_SCHEMA_VERSION, 4);
    assert_eq!(run_migration_name(), b"add-local-run-evidence");
    assert!(run_migration_checksum().starts_with(b"sha256:"));
    let migration = String::from_utf8(run_migration_sql()).expect("migration SQL is UTF-8");
    for table in [
        "id_sequences",
        "target_builds",
        "capability_manifests",
        "runs",
        "run_attempts",
        "run_effective_controls",
        "observations",
        "harness_failures",
    ] {
        assert!(migration.contains(&format!("CREATE TABLE {table}")));
    }

    let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = std::env::temp_dir().join(format!(
        "crucible-run-schema-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create root");
    let output = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .arg("init")
        .arg(&root)
        .output()
        .expect("run init");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let connection =
        Connection::open(root.join(".crucible/database.sqlite")).expect("open workspace database");
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .expect("user version"),
        4,
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 3 AND name = 'add-local-run-evidence'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("migration row"),
        1,
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name IN ('id_sequences','target_builds','capability_manifests','runs','run_attempts','run_effective_controls','observations','harness_failures')",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("run tables"),
        8,
    );
    drop(connection);
    std::fs::remove_dir_all(root).expect("remove temporary workspace");
}
