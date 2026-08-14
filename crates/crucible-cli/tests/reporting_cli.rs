use serde_json::Value;
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
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "crucible-reporting-cli-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create root");
        let initialized = command(&root, &["init", root.to_str().expect("UTF-8 root")]);
        assert!(
            initialized.status.success(),
            "{}",
            String::from_utf8_lossy(&initialized.stderr)
        );
        Self { root }
    }

    fn create_finding(&self) -> String {
        let target = self.root.join("fails.sh");
        std::fs::write(&target, b"#!/bin/sh\nexit 7\n").expect("write target");
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
project: {{name: report-fixture}}
target: {{adapter: cli, command: "{}", args: []}}
execution: {{timeout_ms: 2000, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}}
oracles: {{process_exit: {{allowed_codes: [0], timeout_is_failure: true}}}}
inputs: {{corpus: []}}
engines: {{fuzz: {{enabled: false, modes: [], native_backends: []}}, property: {{enabled: false}}, differential: {{enabled: false}}, metamorphic: {{enabled: false}}, fault: {{enabled: false}}, concurrency: {{enabled: false}}, symbolic: {{enabled: false}}, mutation: {{enabled: false}}}}
sanitizers: {{address: false, undefined: false, thread: false, memory: false, leak: false}}
campaign: {{duration: 1s, workers: 1, seed: 11}}
storage: {{root: .crucible}}
verification: {{verus: {{required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}}}
"#,
                target.display()
            ),
        )
        .expect("write configuration");
        let run = command(
            &self.root,
            &["run", configuration.to_str().expect("UTF-8 configuration")],
        );
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        "BUG-000001".to_owned()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).expect("remove workspace");
    }
}

fn command(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("run crucible")
}

fn parse_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}

#[test]
fn reports_distinguish_facts_and_hypotheses_in_every_declared_format() {
    let workspace = Workspace::new();
    let finding = workspace.create_finding();
    let root = workspace.root.to_str().expect("UTF-8 root");

    let findings = command(&workspace.root, &["findings", root]);
    assert!(findings.status.success());
    let findings = String::from_utf8(findings.stdout).expect("UTF-8 findings");
    assert!(findings.contains(&finding));
    assert!(findings.contains("target-defect/process-exit"));

    let human = command(
        &workspace.root,
        &["report", &finding, "--format", "human", root],
    );
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    let human = String::from_utf8(human.stdout).expect("UTF-8 human report");
    assert!(human.contains("Observed facts:"));
    assert!(human.contains("Hypotheses:"));
    assert!(human.contains("none recorded"));
    assert!(human.contains("determinism is not proven"));

    let json = parse_json(&command(
        &workspace.root,
        &["report", &finding, "--format", "json", root],
    ));
    assert_eq!(json["schema"], "crucible.finding-report.v1");
    assert_eq!(json["facts"]["finding_id"], finding);
    assert!(json["hypotheses"]
        .as_array()
        .expect("hypotheses array")
        .is_empty());

    let jsonl = command(
        &workspace.root,
        &["report", &finding, "--format", "jsonl", root],
    );
    assert!(jsonl.status.success());
    let lines = jsonl
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty());
    for line in lines {
        let _: Value = serde_json::from_slice(line).expect("valid JSONL record");
    }

    let sarif = parse_json(&command(
        &workspace.root,
        &["report", &finding, "--format", "sarif", root],
    ));
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(
        sarif["runs"][0]["results"][0]["ruleId"],
        "target-defect/process-exit"
    );

    let junit = command(
        &workspace.root,
        &["report", &finding, "--format", "junit", root],
    );
    assert!(junit.status.success());
    let junit = String::from_utf8(junit.stdout).expect("UTF-8 JUnit");
    assert!(junit.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(junit.contains("<failure"));

    let graph = parse_json(&command(
        &workspace.root,
        &["report", &finding, "--format", "evidence", root],
    ));
    assert_eq!(graph["schema"], "crucible.evidence-graph.v1");
    assert!(graph["nodes"].as_array().expect("nodes").len() >= 3);

    let bundle = parse_json(&command(
        &workspace.root,
        &["report", &finding, "--format", "bundle", root],
    ));
    assert_eq!(bundle["schema"], "crucible.evidence-bundle.v1");
    assert_eq!(bundle["finding_id"], finding);
    assert_eq!(bundle["signature"], Value::Null);
    assert_eq!(bundle["signature_scope"], "exact-manifest-and-provenance");
    assert_eq!(bundle["hypothesis_truth_attested"], false);
    assert_eq!(bundle["seeds"]["campaign"], "11");
}

#[test]
fn workspace_introspection_commands_emit_bounded_versioned_json() {
    let workspace = Workspace::new();
    let root = workspace.root.to_str().expect("UTF-8 root");
    for (command_name, schema) in [
        ("capabilities", "crucible.capabilities.v1"),
        ("proof", "crucible.proof-report.v1"),
        ("tcb", "crucible.tcb-report.v1"),
        ("plugins", "crucible.plugin-report.v1"),
    ] {
        let output = command(&workspace.root, &[command_name, root]);
        assert!(output.stdout.len() <= 1_048_576);
        let json = parse_json(&output);
        assert_eq!(json["schema"], schema);
        if command_name == "tcb" {
            let boundaries = json["boundaries"].as_array().expect("TCB boundaries");
            assert_eq!(boundaries.len(), 21);
            let mut names = boundaries
                .iter()
                .map(|value| value.as_str().expect("boundary name"))
                .collect::<Vec<_>>();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), 21);
            assert!(names.contains(&"CORE-HOST-UTF8-001"));
            assert!(names.contains(&"XTASK-HOST-COMMAND-001"));
            assert!(names.contains(&"CLI-HOST-DOMAIN-READ-001"));
        }
    }
}

#[test]
fn report_rejects_unknown_findings_without_inventing_evidence() {
    let workspace = Workspace::new();
    let output = command(
        &workspace.root,
        &[
            "report",
            "BUG-999999",
            "--format",
            "json",
            workspace.root.to_str().expect("UTF-8 root"),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("finding not found"));
    assert!(output.stdout.is_empty());
}
