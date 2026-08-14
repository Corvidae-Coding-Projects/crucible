use crucible_cli::{parse_cli_args, CliAction, MAX_INSPECTION_REPORT_BYTES};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct TemporaryWorkspace {
    root: PathBuf,
}

impl TemporaryWorkspace {
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "crucible-inspect-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create temporary workspace root");
        let init = Command::new(env!("CARGO_BIN_EXE_crucible"))
            .arg("init")
            .arg(&root)
            .output()
            .expect("initialize workspace");
        assert!(
            init.status.success(),
            "{}",
            String::from_utf8_lossy(&init.stderr)
        );
        Self { root }
    }

    fn database(&self) -> Connection {
        Connection::open(self.root.join(".crucible/database.sqlite")).expect("open database")
    }

    fn write_target(&self, name: &str, source: &[u8]) -> PathBuf {
        let path = self.root.join(name);
        std::fs::write(&path, source).expect("write target fixture");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                .expect("make target executable");
        }
        path
    }

    fn write_configuration(&self, command: &Path, output_mb: u64) -> PathBuf {
        let template = r#"version: 1
language: {profile: crucible-yaml-1}
project: {name: inspect-fixture}
target: {adapter: cli, command: "@COMMAND@", args: []}
execution: {timeout_ms: 2000, memory_mb: 128, max_processes: 4, max_output_mb: @OUTPUT@, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}
oracles: {process_exit: {allowed_codes: [0], timeout_is_failure: true}}
inputs: {corpus: []}
engines: {fuzz: {enabled: false, modes: [], native_backends: []}, property: {enabled: false}, differential: {enabled: false}, metamorphic: {enabled: false}, fault: {enabled: false}, concurrency: {enabled: false}, symbolic: {enabled: false}, mutation: {enabled: false}}
sanitizers: {address: false, undefined: false, thread: false, memory: false, leak: false}
campaign: {duration: 1s, workers: 1, seed: 7}
storage: {root: .crucible}
verification: {verus: {required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}
"#;
        let contents = template
            .replace("@COMMAND@", command.to_str().expect("UTF-8 fixture path"))
            .replace("@OUTPUT@", &output_mb.to_string());
        let path = self.root.join("crucible.yaml");
        std::fs::write(&path, contents).expect("write configuration");
        path
    }

    fn run(&self, configuration: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_crucible"))
            .arg("run")
            .arg(configuration)
            .current_dir(&self.root)
            .output()
            .expect("run Crucible")
    }

    fn inspect(&self, run_id: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_crucible"))
            .arg("inspect")
            .arg(run_id)
            .current_dir(&self.root)
            .output()
            .expect("inspect Crucible run")
    }

    fn object_path(&self, artifact_id: &str) -> PathBuf {
        let digest = artifact_id
            .strip_prefix("sha256:")
            .expect("canonical SHA-256 artifact ID");
        self.root
            .join(".crucible/objects/sha256")
            .join(&digest[0..2])
            .join(&digest[2..4])
            .join(digest)
    }
}

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn successful_run_id(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("run ID is UTF-8")
        .trim()
        .to_owned()
}

#[test]
fn verified_argument_parser_exposes_bounded_run_inspection() {
    assert_eq!(
        parse_cli_args(&[String::from("inspect"), String::from("run-0001")]),
        Ok(CliAction::Inspect(
            String::from("run-0001"),
            String::from(".")
        ))
    );
    assert_eq!(
        parse_cli_args(&[
            String::from("inspect"),
            String::from("run-0001"),
            String::from("workspace"),
        ]),
        Ok(CliAction::Inspect(
            String::from("run-0001"),
            String::from("workspace")
        ))
    );
}

#[test]
fn inspect_reports_complete_persisted_run_facts_and_no_invented_hypotheses() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("visible.sh", b"#!/bin/sh\nprintf 'visible\\n'\n");
    let configuration = workspace.write_configuration(&target, 1);
    let run_id = successful_run_id(workspace.run(&configuration));

    let inspected = workspace.inspect(&run_id);
    assert!(
        inspected.status.success(),
        "{}",
        String::from_utf8_lossy(&inspected.stderr)
    );
    assert!(inspected.stdout.len() <= MAX_INSPECTION_REPORT_BYTES as usize);
    let report = String::from_utf8(inspected.stdout).expect("human report is UTF-8");
    for fact in [
        format!("run: {run_id}"),
        String::from("attempt: attempt-00000000000000000001"),
        String::from("status: observed"),
        String::from("configuration-source: sha256:"),
        String::from("effective-configuration: sha256:"),
        String::from("configuration-digest: sha256:"),
        String::from("target-build: target-build-sha256:"),
        String::from("capability-manifest: sha256:"),
        String::from("seed: 7"),
        String::from("controls.timeout-ms: 2000"),
        String::from("controls.memory-bytes: 134217728"),
        String::from("controls.max-processes: 4"),
        String::from("controls.max-stream-bytes: 1048576"),
        String::from("controls.network-policy: none"),
        String::from("controls.isolation-backend: linux-bubblewrap-prlimit-v1"),
        String::from("controls.output-capture-status: drain-and-discard"),
        String::from("observation: sha256:"),
        String::from("stdout.retained-bytes: 8"),
        String::from("stdout.discarded-bytes: 0"),
        String::from("stdout.truncated: false"),
        String::from("stdout.preview-hex: 76697369626c650a"),
        String::from("stderr.retained-bytes: 0"),
        String::from("harness-failure: none"),
        String::from("hypotheses: none"),
    ] {
        assert!(
            report.contains(&fact),
            "missing report fact: {fact}\n{report}"
        );
    }
}

#[test]
fn inspect_bounds_large_stream_rendering_and_preserves_exact_accounting() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("large.sh", b"#!/bin/sh\nhead -c 1048580 /dev/zero\n");
    let configuration = workspace.write_configuration(&target, 1);
    let run_id = successful_run_id(workspace.run(&configuration));

    let inspected = workspace.inspect(&run_id);
    assert!(inspected.status.success());
    assert!(inspected.stdout.len() <= MAX_INSPECTION_REPORT_BYTES as usize);
    let report = String::from_utf8(inspected.stdout).expect("human report is UTF-8");
    assert!(report.contains("stdout.retained-bytes: 1048576"));
    assert!(report.contains("stdout.discarded-bytes: 4"));
    assert!(report.contains("stdout.truncated: true"));
    assert!(report.contains("stdout.preview-omitted-bytes: 1044480"));
}

#[test]
fn inspect_keeps_harness_failures_disjoint_from_target_observations() {
    let workspace = TemporaryWorkspace::new();
    let missing = workspace.root.join("missing-target");
    let configuration = workspace.write_configuration(&missing, 1);
    let run = workspace.run(&configuration);
    assert!(!run.status.success());
    let run_id: String = workspace
        .database()
        .query_row("SELECT id FROM runs", [], |row| row.get(0))
        .expect("reserved run ID");

    let inspected = workspace.inspect(&run_id);
    assert!(inspected.status.success());
    let report = String::from_utf8(inspected.stdout).expect("human report is UTF-8");
    assert!(report.contains("status: harness_failure"));
    assert!(report.contains("observation: none"));
    assert!(report.contains("harness-failure: TargetPreparation"));
    assert!(report.contains("hypotheses: none"));
}

#[test]
fn inspect_rejects_corrupted_reachable_evidence() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("visible.sh", b"#!/bin/sh\nprintf 'visible\\n'\n");
    let configuration = workspace.write_configuration(&target, 1);
    let run_id = successful_run_id(workspace.run(&configuration));
    let stdout_id: String = workspace
        .database()
        .query_row("SELECT stdout_artifact_id FROM observations", [], |row| {
            row.get(0)
        })
        .expect("stdout artifact ID");
    std::fs::write(workspace.object_path(&stdout_id), b"corrupt\n")
        .expect("corrupt reachable evidence");

    let inspected = workspace.inspect(&run_id);
    assert!(!inspected.status.success());
    assert!(String::from_utf8_lossy(&inspected.stderr).contains("artifact integrity failure"));
    assert!(inspected.stdout.is_empty());
}
