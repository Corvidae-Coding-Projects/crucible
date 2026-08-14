use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
#[expect(
    unused_imports,
    reason = "Verus requires the prelude crate marker even when this black-box test names no vstd item"
)]
use vstd::prelude::*;

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[test]
fn json_tracing_has_correlated_fields_and_never_copies_target_output() {
    let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "crucible-logging-cli-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create root");
    let initialized = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["init", root.to_str().expect("UTF-8 root")])
        .current_dir(&root)
        .output()
        .expect("initialize");
    assert!(initialized.status.success());
    let target = root.join("target.sh");
    let target_only = "TARGET_ONLY_PROMPT_INJECTION_DO_NOT_LOG";
    std::fs::write(&target, format!("#!/bin/sh\nprintf '{target_only}'\n")).expect("write target");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700))
            .expect("make target executable");
    }
    let configuration: PathBuf = root.join("crucible.yaml");
    std::fs::write(
        &configuration,
        format!(
            r#"version: 1
language: {{profile: crucible-yaml-1}}
project: {{name: logging-fixture}}
target: {{adapter: cli, command: "{}", args: []}}
execution: {{timeout_ms: 2000, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}}
oracles: {{process_exit: {{allowed_codes: [0], timeout_is_failure: true}}}}
inputs: {{corpus: []}}
engines: {{fuzz: {{enabled: false, modes: [], native_backends: []}}, property: {{enabled: false}}, differential: {{enabled: false}}, metamorphic: {{enabled: false}}, fault: {{enabled: false}}, concurrency: {{enabled: false}}, symbolic: {{enabled: false}}, mutation: {{enabled: false}}}}
sanitizers: {{address: false, undefined: false, thread: false, memory: false, leak: false}}
campaign: {{duration: 1s, workers: 1, seed: 53}}
storage: {{root: .crucible}}
verification: {{verus: {{required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}}}
"#,
            target.display()
        ),
    )
    .expect("write configuration");
    let output = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["run", configuration.to_str().expect("UTF-8 configuration")])
        .env("CRUCIBLE_LOG", "json")
        .current_dir(&root)
        .output()
        .expect("run with logging");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 logs");
    assert!(!stderr.contains(target_only));
    let records = stderr
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("one JSON event per line"))
        .collect::<Vec<_>>();
    assert!(!records.is_empty(), "captured stderr: {stderr:?}");
    assert!(records.iter().all(|record| record["timestamp"].is_string()));
    let correlated = records
        .iter()
        .find(|record| {
            record["fields"]["run_id"].is_string() && record["fields"]["run_attempt_id"].is_string()
        })
        .expect("correlated run event");
    assert_eq!(correlated["fields"]["engine_id"], "local-cli");
    assert_eq!(correlated["fields"]["worker_id"], "local-process");
    assert_eq!(correlated["fields"]["severity"], "INFO");
    assert!(correlated["fields"]["target_build_id"].is_string());

    std::fs::remove_dir_all(root).expect("remove workspace");
}
