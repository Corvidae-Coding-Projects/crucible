use crucible_cli::{
    artifact_imports_table_sql, artifacts_table_sql, domain_migration_checksum,
    domain_migration_name, domain_migration_sql, domain_schema_digest, metadata_table_sql,
    migration_table_sql, run_migration_sql, ENGINE_EVENT_SCHEMA_VERSION,
    EVIDENCE_BUNDLE_SCHEMA_VERSION, REPORT_SCHEMA_VERSION, WORKSPACE_SCHEMA_VERSION,
};
use crucible_core::ContentDigest;
use rusqlite::Connection;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

const CONCEPTUAL_TABLES: [&str; 34] = [
    "projects",
    "targets",
    "target_builds",
    "source_snapshots",
    "build_recipes",
    "build_executions",
    "deployments",
    "campaigns",
    "experiments",
    "scenarios",
    "scenario_participants",
    "scenario_steps",
    "scenario_edges",
    "runs",
    "run_attempts",
    "observations",
    "oracle_verdicts",
    "findings",
    "finding_instances",
    "finding_campaigns",
    "finding_transitions",
    "artifacts",
    "evidence_nodes",
    "provenance_edges",
    "coverage_records",
    "corpus_entries",
    "patches",
    "verification_runs",
    "proof_artifacts",
    "trusted_boundaries",
    "plugin_identities",
    "capability_manifests",
    "engine_stats",
    "run_effective_controls",
];

fn computed_domain_schema_digest() -> String {
    let connection = Connection::open_in_memory().expect("open in-memory database");
    let scripts = [
        migration_table_sql(),
        metadata_table_sql(),
        artifacts_table_sql(),
        artifact_imports_table_sql(),
        run_migration_sql(),
        domain_migration_sql(),
    ];
    for script in scripts {
        connection
            .execute_batch(&String::from_utf8(script).expect("schema SQL is UTF-8"))
            .expect("apply schema SQL");
    }
    let mut statement = connection
        .prepare(
            "SELECT name, type, COALESCE(sql, '') FROM sqlite_schema
             WHERE name NOT LIKE 'sqlite_%'
               AND name NOT IN ('schema_migrations','workspace_metadata','artifacts','artifact_imports')
             ORDER BY name",
        )
        .expect("prepare schema signature");
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .expect("query schema signature");
    let mut signature = Vec::new();
    for row in rows {
        let (name, kind, sql) = row.expect("schema row");
        for value in [name.as_bytes(), kind.as_bytes(), sql.as_bytes()] {
            signature.extend_from_slice(&(value.len() as u64).to_be_bytes());
            signature.extend_from_slice(value);
        }
    }
    ContentDigest::from_bytes(&signature)
        .expect("bounded schema signature")
        .into_artifact_id()
        .as_str()
        .to_owned()
}

#[test]
fn domain_schema_digest_authenticates_every_non_base_schema_object() {
    let migration = ContentDigest::from_bytes(&domain_migration_sql())
        .expect("bounded migration SQL")
        .into_artifact_id()
        .as_str()
        .to_owned();
    assert_eq!(
        String::from_utf8(domain_migration_checksum()).expect("digest is UTF-8"),
        migration,
        "computed migration digest: {migration}"
    );
    let computed = computed_domain_schema_digest();
    assert_eq!(
        String::from_utf8(domain_schema_digest()).expect("digest is UTF-8"),
        computed,
        "computed digest: {computed}"
    );
}

#[test]
fn schema_four_covers_the_complete_domain_and_storage_coordination_model() {
    assert_eq!(WORKSPACE_SCHEMA_VERSION, 4);
    assert_eq!(domain_migration_name(), b"add-domain-storage-model");
    assert!(domain_migration_checksum().starts_with(b"sha256:"));
    let sql = String::from_utf8(domain_migration_sql()).expect("migration SQL is UTF-8");
    for table in CONCEPTUAL_TABLES {
        if !matches!(
            table,
            "target_builds"
                | "runs"
                | "run_attempts"
                | "observations"
                | "artifacts"
                | "capability_manifests"
                | "run_effective_controls"
        ) {
            assert!(
                sql.contains(&format!("CREATE TABLE {table}")),
                "missing {table}"
            );
        }
    }
    for storage_table in [
        "storage_generations",
        "storage_leases",
        "artifact_roots",
        "persistence_batches",
    ] {
        assert!(
            sql.contains(&format!("CREATE TABLE {storage_table}")),
            "missing {storage_table}"
        );
    }
    assert!(sql.contains("retention_policy"));
    assert!(sql.contains("scheduling_seed"));
    assert!(sql.contains("fault_seed"));
    for replay_table in [
        "run_replay_metadata",
        "reproduction_samples",
        "minimization_steps",
    ] {
        assert!(
            sql.contains(&format!("CREATE TABLE {replay_table}")),
            "missing {replay_table}"
        );
    }
    assert!(sql.contains("engine_seed_status"));
    assert!(
        sql.contains("encoded_bytes INTEGER NOT NULL CHECK(encoded_bytes BETWEEN 0 AND 67108864)")
    );
    assert_eq!(ENGINE_EVENT_SCHEMA_VERSION, 1);
    assert_eq!(EVIDENCE_BUNDLE_SCHEMA_VERSION, 1);
    assert_eq!(REPORT_SCHEMA_VERSION, 1);
    assert!(sql.contains("PRAGMA user_version = 4"));

    let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = std::env::temp_dir().join(format!(
        "crucible-storage-schema-{}-{sequence}",
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
        4
    );
    for table in CONCEPTUAL_TABLES {
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get::<_, i64>(0),
                )
                .expect("table lookup"),
            1,
            "missing installed table {table}"
        );
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 4 AND name = 'add-domain-storage-model'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("migration row"),
        1
    );
    std::fs::remove_dir_all(root).expect("remove temporary workspace");
}

#[test]
fn scenario_topology() {
    let connection = Connection::open_in_memory().expect("open database");
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable foreign keys");
    for script in [
        migration_table_sql(),
        metadata_table_sql(),
        artifacts_table_sql(),
        artifact_imports_table_sql(),
        run_migration_sql(),
        domain_migration_sql(),
    ] {
        connection
            .execute_batch(&String::from_utf8(script).expect("schema SQL is UTF-8"))
            .expect("apply schema");
    }
    let artifact_id = format!("sha256:{}", "a".repeat(64));
    connection
        .execute(
            "INSERT INTO artifacts(id, algorithm, digest, size_bytes, media_type)
             VALUES (?1, 'sha256', ?2, 1, 'application/octet-stream')",
            [&artifact_id, &"a".repeat(64)],
        )
        .expect("artifact");
    connection
        .execute(
            "INSERT INTO projects(id, name, configuration_artifact_id) VALUES ('project', 'project', ?1)",
            [&artifact_id],
        )
        .expect("project");
    connection
        .execute(
            "INSERT INTO target_builds(id, target_artifact_id, manifest_artifact_id, identity_digest)
             VALUES ('build', ?1, ?1, ?1)",
            [&artifact_id],
        )
        .expect("target build");
    connection
        .execute(
            "INSERT INTO campaigns(id, project_id, configuration_artifact_id, retention_policy,
                    status, campaign_seed, scheduling_seed, fault_seed)
             VALUES ('campaign', 'project', ?1, 'retain-every-run', 'active', '1', '2', '3')",
            [&artifact_id],
        )
        .expect("campaign");
    for scenario in ["scenario-a", "scenario-b"] {
        connection
            .execute(
                "INSERT INTO scenarios(id, campaign_id, definition_artifact_id, status)
                 VALUES (?1, 'campaign', ?2, 'active')",
                [scenario, &artifact_id],
            )
            .expect("scenario");
    }
    connection
        .execute(
            "INSERT INTO scenario_participants(id, scenario_id, target_build_id, role)
             VALUES ('participant-a', 'scenario-a', 'build', 'server')",
            [],
        )
        .expect("participant");
    for (id, scenario, ordinal) in [
        ("step-a1", "scenario-a", 0),
        ("step-a2", "scenario-a", 1),
        ("step-b1", "scenario-b", 0),
    ] {
        connection
            .execute(
                "INSERT INTO scenario_steps(id, scenario_id, participant_id, ordinal, action_artifact_id)
                 VALUES (?1, ?2, NULL, ?3, ?4)",
                rusqlite::params![id, scenario, ordinal, artifact_id],
            )
            .expect("scenario step");
    }
    connection
        .execute(
            "INSERT INTO scenario_edges(scenario_id, from_step_id, to_step_id, relation)
             VALUES ('scenario-a', 'step-a1', 'step-a2', 'before')",
            [],
        )
        .expect("valid scenario edge");
    assert!(
        connection
            .execute(
                "INSERT INTO scenario_edges(scenario_id, from_step_id, to_step_id, relation)
                 VALUES ('scenario-a', 'step-a1', 'step-a1', 'cancel')",
                [],
            )
            .is_err(),
        "self-edge was admitted"
    );
    assert!(
        connection
            .execute(
                "INSERT INTO scenario_edges(scenario_id, from_step_id, to_step_id, relation)
                 VALUES ('scenario-a', 'step-a1', 'step-b1', 'data')",
                [],
            )
            .is_err(),
        "an edge crossed scenario ownership"
    );
}
