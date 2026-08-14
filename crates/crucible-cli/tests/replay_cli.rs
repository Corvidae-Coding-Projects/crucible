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

struct ReplayWorkspace {
    root: PathBuf,
}

impl ReplayWorkspace {
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "crucible-replay-cli-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create root");
        let initialized = run(&root, &["init", root.to_str().expect("UTF-8 root")]);
        assert!(initialized.status.success());
        let target = root.join("fails.sh");
        std::fs::write(&target, b"#!/bin/sh\nexit 9\n").expect("write target");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700))
                .expect("make target executable");
        }
        let configuration = root.join("crucible.yaml");
        std::fs::write(
            &configuration,
            format!(
                r#"version: 1
language: {{profile: crucible-yaml-1}}
project: {{name: replay-fixture}}
target: {{adapter: cli, command: "{}", args: []}}
execution: {{timeout_ms: 2000, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}}
oracles: {{process_exit: {{allowed_codes: [0], timeout_is_failure: true}}}}
inputs: {{corpus: []}}
engines: {{fuzz: {{enabled: false, modes: [], native_backends: []}}, property: {{enabled: false}}, differential: {{enabled: false}}, metamorphic: {{enabled: false}}, fault: {{enabled: false}}, concurrency: {{enabled: false}}, symbolic: {{enabled: false}}, mutation: {{enabled: false}}}}
sanitizers: {{address: false, undefined: false, thread: false, memory: false, leak: false}}
campaign: {{duration: 1s, workers: 1, seed: 29}}
storage: {{root: .crucible}}
verification: {{verus: {{required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}}}
"#,
                target.display()
            ),
        )
        .expect("write configuration");
        let first = run(
            &root,
            &["run", configuration.to_str().expect("UTF-8 configuration")],
        );
        assert!(
            first.status.success(),
            "{}",
            String::from_utf8_lossy(&first.stderr)
        );
        Self { root }
    }

    fn connection(&self) -> Connection {
        Connection::open(self.root.join(".crucible/database.sqlite")).expect("open database")
    }
}

impl Drop for ReplayWorkspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).expect("remove workspace");
    }
}

fn run(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("run crucible")
}

#[test]
fn finding_replay_reexecutes_recorded_configuration_and_records_observed_rate() {
    let workspace = ReplayWorkspace::new();
    let root = workspace.root.to_str().expect("UTF-8 root");
    let replay = run(&workspace.root, &["replay", "BUG-000001", root]);
    assert!(
        replay.status.success(),
        "{}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let output = String::from_utf8(replay.stdout).expect("UTF-8 replay report");
    assert!(output.contains("BUG-000001"));
    assert!(output.contains("1/1"));
    assert!(output.contains("stable-under-recorded-controls"));
    assert!(output.contains("determinism is not proven"));

    let connection = workspace.connection();
    let sample: (String, i64, i64, i64, i64, i64, String) = connection
        .query_row(
            "SELECT promise, attempt_count, observed_failures, environment_equivalent,
                    schedule_replayed_exactly, fault_trace_replayed_exactly, classification
             FROM reproduction_samples",
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
        .expect("reproduction sample");
    assert_eq!(sample.0, "finding");
    assert_eq!((sample.1, sample.2), (1, 1));
    assert_eq!(sample.3, 1);
    assert_eq!(sample.4, 0);
    assert_eq!(sample.5, 0);
    assert_eq!(sample.6, "stable-under-recorded-controls");
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM finding_instances WHERE finding_id = 'BUG-000001'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        2
    );
}

#[test]
fn minimize_refuses_to_claim_progress_without_a_declared_input_artifact() {
    let workspace = ReplayWorkspace::new();
    let output = run(
        &workspace.root,
        &[
            "minimize",
            "BUG-000001",
            workspace.root.to_str().expect("UTF-8 root"),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no minimizable input artifact"));
    assert_eq!(
        workspace
            .connection()
            .query_row("SELECT COUNT(*) FROM minimization_steps", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn verify_preserves_patch_evidence_and_reports_missing_build_recipe_as_inconclusive() {
    let workspace = ReplayWorkspace::new();
    let patch = workspace.root.join("candidate.diff");
    std::fs::write(
        &patch,
        b"--- a/fails.sh\n+++ b/fails.sh\n@@ -1,2 +1,2 @@\n #!/bin/sh\n-exit 9\n+exit 0\n",
    )
    .expect("write patch");
    let output = run(
        &workspace.root,
        &[
            "verify",
            "BUG-000001",
            "--patch",
            patch.to_str().expect("UTF-8 patch"),
            workspace.root.to_str().expect("UTF-8 root"),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "verification inconclusive: finding has no recorded source snapshot and build recipe"
    ));
    let connection = workspace.connection();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM patches WHERE finding_id = 'BUG-000001' AND status = 'candidate'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM verification_runs WHERE status = 'inconclusive'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM artifact_roots WHERE root_kind = 'manual'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
}
