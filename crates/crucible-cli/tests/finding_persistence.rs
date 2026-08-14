use crucible_core::derive_replay_seeds;
use rusqlite::Connection;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[test]
fn failed_oracle_creates_a_rooted_finding_and_complete_replay_metadata() {
    let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "crucible-finding-persistence-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create workspace");
    let init = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .arg("init")
        .arg(&root)
        .output()
        .expect("initialize");
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let target = root.join("fails.sh");
    std::fs::write(&target, b"#!/bin/sh\nexit 7\n").expect("write target");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700))
            .expect("make executable");
    }
    let configuration = root.join("crucible.yaml");
    std::fs::write(
        &configuration,
        format!(
            r#"version: 1
language: {{profile: crucible-yaml-1}}
project: {{name: persisted-finding}}
target: {{adapter: cli, command: "{}", args: []}}
execution: {{timeout_ms: 2000, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}}
oracles: {{process_exit: {{allowed_codes: [0], timeout_is_failure: true}}}}
inputs: {{corpus: []}}
engines: {{fuzz: {{enabled: false, modes: [], native_backends: []}}, property: {{enabled: false}}, differential: {{enabled: false}}, metamorphic: {{enabled: false}}, fault: {{enabled: false}}, concurrency: {{enabled: false}}, symbolic: {{enabled: false}}, mutation: {{enabled: false}}}}
sanitizers: {{address: false, undefined: false, thread: false, memory: false, leak: false}}
campaign: {{duration: 1s, workers: 1, seed: 7}}
storage: {{root: .crucible}}
verification: {{verus: {{required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}}}
"#,
            target.display()
        ),
    )
    .expect("write configuration");
    let run = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .arg("run")
        .arg(&configuration)
        .current_dir(&root)
        .output()
        .expect("execute run");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let connection = Connection::open(root.join(".crucible/database.sqlite")).expect("database");
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM oracle_verdicts WHERE verdict = 'fail'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    let finding: (String, String, String) = connection
        .query_row("SELECT id, kind, status FROM findings", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .expect("finding");
    assert!(finding.0.starts_with("BUG-"));
    assert_eq!(finding.1, "target-defect/process-exit");
    assert_eq!(finding.2, "open");
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM finding_instances WHERE is_original = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM artifact_roots WHERE root_kind = 'original-finding'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap()
            >= 3
    );

    let recorded: (String, String, String, String, String, String, i64) = connection
        .query_row(
            "SELECT campaign_seed, engine_seed, experiment_seed, scheduling_seed, fault_seed,
                    engine_seed_status, schema_version FROM run_replay_metadata",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .expect("replay metadata");
    let seeds = derive_replay_seeds(7);
    assert_eq!(recorded.0, seeds.campaign.to_string());
    assert_eq!(recorded.1, seeds.engine.to_string());
    assert_eq!(recorded.2, seeds.experiment.to_string());
    assert_eq!(recorded.3, seeds.scheduling.to_string());
    assert_eq!(recorded.4, seeds.fault.to_string());
    assert_eq!(recorded.5, "supported");
    assert_eq!(recorded.6, 1);
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM engine_stats", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        7
    );

    drop(connection);
    std::fs::remove_dir_all(root).expect("remove workspace");
}
