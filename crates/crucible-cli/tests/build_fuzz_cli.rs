use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
#[expect(
    unused_imports,
    reason = "Verus requires the prelude crate marker even when this black-box test names no vstd item"
)]
use vstd::prelude::*;

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct Workspace {
    root: PathBuf,
}

impl Workspace {
    fn new(label: &str) -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "crucible-build-fuzz-{label}-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create root");
        let initialized = invoke(&root, &["init", root.to_str().expect("UTF-8 root")]);
        assert!(initialized.status.success());
        Self { root }
    }

    fn configuration(&self, project: &str, target_body: &[u8]) -> PathBuf {
        let target = self.root.join("target.sh");
        std::fs::write(&target, target_body).expect("write target");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700))
                .expect("make target executable");
        }
        let configuration = self.root.join("crucible.yaml");
        std::fs::write(
            &configuration,
            format!(
                r#"version: 1
language: {{profile: crucible-yaml-1}}
project: {{name: {project}}}
target: {{adapter: cli, command: "{}", args: []}}
execution: {{timeout_ms: 2000, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}}
oracles: {{process_exit: {{allowed_codes: [0], timeout_is_failure: true}}}}
inputs: {{corpus: []}}
engines: {{fuzz: {{enabled: true, modes: [managed], native_backends: []}}, property: {{enabled: false}}, differential: {{enabled: false}}, metamorphic: {{enabled: false}}, fault: {{enabled: false}}, concurrency: {{enabled: false}}, symbolic: {{enabled: false}}, mutation: {{enabled: false}}}}
sanitizers: {{address: false, undefined: false, thread: false, memory: false, leak: false}}
campaign: {{duration: 1s, workers: 1, seed: 41}}
storage: {{root: .crucible}}
verification: {{verus: {{required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}}}
"#,
                target.display()
            ),
        )
        .expect("write configuration");
        configuration
    }

    fn connection(&self) -> Connection {
        Connection::open(self.root.join(".crucible/database.sqlite")).expect("open database")
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).expect("remove workspace");
    }
}

fn invoke(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("run crucible")
}

#[test]
fn build_records_a_prebuilt_cli_target_without_executing_it() {
    let workspace = Workspace::new("build");
    let marker = workspace.root.join("executed");
    let configuration = workspace.configuration(
        "build-fixture",
        format!("#!/bin/sh\nprintf executed > '{}'\n", marker.display()).as_bytes(),
    );
    let output = invoke(
        &workspace.root,
        &[
            "build",
            configuration.to_str().expect("UTF-8 configuration"),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("target-build-sha256:"));
    assert!(!marker.exists(), "build executed the target");
    let connection = workspace.connection();
    for (table, expected) in [
        ("projects", 1_i64),
        ("targets", 1),
        ("source_snapshots", 1),
        ("build_recipes", 1),
        ("build_executions", 1),
        ("target_builds", 1),
        ("runs", 0),
        ("run_attempts", 0),
    ] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            expected,
            "unexpected {table} count"
        );
    }
    assert_eq!(
        connection
            .query_row("SELECT status FROM build_executions", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "succeeded"
    );
}

#[test]
fn successful_high_throughput_fuzzing_keeps_aggregates_not_transient_runs() {
    let workspace = Workspace::new("fuzz");
    let configuration = workspace.configuration("fuzz-fixture", b"#!/bin/sh\nexit 0\n");
    let output = invoke(
        &workspace.root,
        &["fuzz", configuration.to_str().expect("UTF-8 configuration")],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("campaign-run-"));
    let connection = workspace.connection();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM campaigns WHERE retention_policy = 'aggregate-checkpoints'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT executions FROM engine_stats WHERE engine_class = 'coverage-fuzzing'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    for table in [
        "runs",
        "run_attempts",
        "observations",
        "oracle_verdicts",
        "run_replay_metadata",
    ] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0,
            "transient {table} row was retained"
        );
    }
}

#[test]
#[ignore = "nightly and weekly depth tier"]
fn extended_campaign() {
    const EXECUTIONS: i64 = 16;
    let workspace = Workspace::new("extended-campaign");
    let configuration = workspace.configuration("extended-fuzz", b"#!/bin/sh\nexit 0\n");
    for _ in 0..EXECUTIONS {
        let output = invoke(
            &workspace.root,
            &["fuzz", configuration.to_str().expect("UTF-8 configuration")],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let connection = workspace.connection();
    assert_eq!(
        connection
            .query_row(
                "SELECT SUM(executions) FROM engine_stats WHERE engine_class = 'coverage-fuzzing'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        EXECUTIONS
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM campaigns", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        EXECUTIONS
    );
    for table in ["runs", "run_attempts", "observations", "oracle_verdicts"] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0,
            "extended campaign retained transient {table} rows"
        );
    }
}
