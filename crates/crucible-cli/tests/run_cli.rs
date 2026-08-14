use crucible_core::{
    canonical_raw_observation_codec_limits, decode_raw_observation, CompletionDisposition,
    HarnessTerminationReason, RawExecutionEvent, TerminationRecord,
};
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
        let root =
            std::env::temp_dir().join(format!("crucible-run-{}-{sequence}", std::process::id()));
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

    fn write_configuration(
        &self,
        command: &Path,
        arguments: &str,
        timeout_ms: u64,
        output_mb: u64,
    ) -> PathBuf {
        let template = r#"version: 1
language: {profile: crucible-yaml-1}
project: {name: run-fixture}
target: {adapter: cli, command: "@COMMAND@", args: @ARGUMENTS@}
execution: {timeout_ms: @TIMEOUT@, memory_mb: 128, max_processes: 4, max_output_mb: @OUTPUT@, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}
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
            .replace("@ARGUMENTS@", arguments)
            .replace("@TIMEOUT@", &timeout_ms.to_string())
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

    fn object_bytes(&self, artifact_id: &str) -> Vec<u8> {
        let digest = artifact_id
            .strip_prefix("sha256:")
            .expect("canonical SHA-256 artifact ID");
        std::fs::read(
            self.root
                .join(".crucible/objects/sha256")
                .join(&digest[0..2])
                .join(&digest[2..4])
                .join(digest),
        )
        .expect("read stored object")
    }

    fn decoded_observation(&self) -> crucible_core::ValidatedRawObservation {
        let connection = self.database();
        let observation_id: String = connection
            .query_row(
                "SELECT observation_artifact_id FROM observations",
                [],
                |row| row.get(0),
            )
            .expect("observation row");
        decode_raw_observation(
            self.object_bytes(&observation_id),
            canonical_raw_observation_codec_limits(),
        )
        .expect("decode persisted observation")
    }
}

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn run_command_executes_directly_and_persists_an_immutable_nonzero_exit_observation() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target(
        "target.sh",
        b"#!/bin/sh\nprintf '%s' \"$1\"\nprintf stderr-evidence >&2\nexit 7\n",
    );
    let configuration = workspace.write_configuration(
        &target,
        r#"["$(printf shell-interpolation-must-not-run)"]"#,
        2_000,
        1,
    );
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run_id = String::from_utf8(output.stdout).expect("UTF-8 run ID");
    assert!(run_id.starts_with("run-"));
    assert!(run_id.ends_with('\n'));

    let connection = workspace.database();
    let row: (String, String, String, String, i64, i64) = connection
        .query_row(
            "SELECT a.status, c.network_policy, c.isolation_backend,
                    c.output_capture_status, o.completion_tag, o.termination_tag
             FROM run_attempts a
             JOIN run_effective_controls c ON c.run_id = a.run_id
             JOIN observations o ON o.attempt_id = a.id",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("persisted run records");
    assert_eq!(row.0, "observed");
    assert_eq!(row.1, "none");
    assert_eq!(row.2, "linux-bubblewrap-prlimit-v1");
    assert_eq!(row.3, "drain-and-discard");
    assert_eq!(row.4, 1);
    assert_eq!(row.5, 1);

    let decoded = workspace.decoded_observation();
    let observation = decoded.observation();
    assert_eq!(
        observation.outcome().completion(),
        CompletionDisposition::Completed
    );
    assert_eq!(
        observation.outcome().termination(),
        &Some(TerminationRecord::ExitCode { code: 7 })
    );
    assert_eq!(
        workspace.object_bytes(observation.stdout().artifact().id.as_str()),
        b"$(printf shell-interpolation-must-not-run)"
    );
    assert_eq!(
        workspace.object_bytes(observation.stderr().artifact().id.as_str()),
        b"stderr-evidence"
    );
    assert!(!observation.stdout().truncated());
    assert!(!observation.stderr().truncated());

    let artifact_rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))
        .expect("artifact count");
    assert!(artifact_rows >= 6);
}

#[test]
fn timeout_terminates_the_process_tree_and_is_persisted_as_a_target_fact() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target(
        "timeout.sh",
        b"#!/bin/sh\n/bin/sh -c 'while :; do :; done' crucible-timeout-descendant &\nwait\n",
    );
    let configuration = workspace.write_configuration(&target, "[]", 100, 1);
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let decoded = workspace.decoded_observation();
    let observation = decoded.observation();
    assert_eq!(
        observation.outcome().completion(),
        CompletionDisposition::Completed
    );
    assert_eq!(
        observation.outcome().termination(),
        &Some(TerminationRecord::HarnessTerminated {
            reason: HarnessTerminationReason::Timeout,
        })
    );
    assert!(observation
        .outcome()
        .events()
        .contains(&RawExecutionEvent::TimeoutThresholdReached));
    fn tagged_descendant_exists() -> bool {
        let entries = std::fs::read_dir("/proc").expect("read procfs");
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name
                .to_string_lossy()
                .bytes()
                .all(|byte| byte.is_ascii_digit())
            {
                let command_line = std::fs::read(entry.path().join("cmdline")).unwrap_or_default();
                if command_line
                    .windows(b"crucible-timeout-descendant".len())
                    .any(|window| window == b"crucible-timeout-descendant")
                {
                    return true;
                }
            }
        }
        false
    }
    for _ in 0..20 {
        if !tagged_descendant_exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("tagged timed-out descendant remained alive");
}

#[test]
fn target_signal_is_persisted_as_a_native_signal_not_a_wrapper_exit_code() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("signal.sh", b"#!/bin/sh\nkill -SEGV $$\n");
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let decoded = workspace.decoded_observation();
    assert!(matches!(
        decoded.observation().outcome().termination(),
        Some(TerminationRecord::UnixSignal {
            signal: 11,
            core_dumped: _,
        })
    ));
}

#[test]
fn target_cannot_reach_the_host_backed_supervisor_control_transport() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target(
        "control-isolation.sh",
        br#"#!/bin/sh
if printf forged > /crucible-control/target-forged 2>/dev/null; then
    exit 42
fi
if [ -e /proc/1/fd ]; then
    exit 43
fi
exit 0
"#,
    );
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let decoded = workspace.decoded_observation();
    assert_eq!(
        decoded.observation().outcome().termination(),
        &Some(TerminationRecord::ExitCode { code: 0 })
    );
    assert!(!workspace.root.join(".crucible/runs/target-forged").exists());
}

#[test]
fn target_cannot_trace_its_nondumpable_supervisor() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target(
        "ptrace-parent.py",
        br#"#!/usr/bin/python3
import ctypes
import os
import sys

PTRACE_ATTACH = 16
PTRACE_DETACH = 17
libc = ctypes.CDLL(None, use_errno=True)
parent = os.getppid()
if libc.ptrace(PTRACE_ATTACH, parent, 0, 0) == 0:
    libc.ptrace(PTRACE_DETACH, parent, 0, 0)
    sys.exit(44)
sys.exit(0)
"#,
    );
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let decoded = workspace.decoded_observation();
    assert_eq!(
        decoded.observation().outcome().termination(),
        &Some(TerminationRecord::ExitCode { code: 0 })
    );
}

#[test]
fn output_limit_keeps_draining_and_records_exact_retained_and_discarded_counts() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("output.sh", b"#!/bin/sh\nhead -c 1048580 /dev/zero\n");
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let decoded = workspace.decoded_observation();
    let stdout = decoded.observation().stdout();
    assert!(stdout.truncated());
    assert_eq!(stdout.retained_bytes(), 1_048_576);
    assert_eq!(stdout.discarded_bytes(), 4);
    assert_eq!(
        workspace.object_bytes(stdout.artifact().id.as_str()).len(),
        1_048_576
    );
}

#[test]
fn preparation_failure_is_persisted_as_a_harness_failure_not_a_target_outcome() {
    let workspace = TemporaryWorkspace::new();
    let missing = workspace.root.join("missing-target");
    let configuration = workspace.write_configuration(&missing, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("target preparation failed"));

    let connection = workspace.database();
    let row: (String, String, i64) = connection
        .query_row(
            "SELECT a.status, h.kind,
                    (SELECT COUNT(*) FROM observations WHERE attempt_id = a.id)
             FROM run_attempts a
             JOIN harness_failures h ON h.attempt_id = a.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("persisted harness failure");
    assert_eq!(
        row,
        ("harness_failure".into(), "TargetPreparation".into(), 0)
    );
}

#[test]
fn missing_shebang_interpreter_is_a_harness_failure_not_a_target_exit() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target(
        "invalid-interpreter.sh",
        b"#!/definitely/missing/crucible-interpreter\nexit 0\n",
    );
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("target preparation failed"));

    let connection = workspace.database();
    let row: (String, String, i64) = connection
        .query_row(
            "SELECT a.status, h.kind,
                    (SELECT COUNT(*) FROM observations WHERE attempt_id = a.id)
             FROM run_attempts a
             JOIN harness_failures h ON h.attempt_id = a.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("persisted invalid-executable harness failure");
    assert_eq!(
        row,
        ("harness_failure".into(), "TargetPreparation".into(), 0)
    );
}

#[test]
fn malformed_elf_is_a_harness_failure_not_a_target_exit() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("malformed-elf", b"\x7fELFjunk");
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("target preparation failed"));

    let connection = workspace.database();
    let row: (String, String, i64) = connection
        .query_row(
            "SELECT a.status, h.kind,
                    (SELECT COUNT(*) FROM observations WHERE attempt_id = a.id)
             FROM run_attempts a
             JOIN harness_failures h ON h.attempt_id = a.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("persisted malformed-executable harness failure");
    assert_eq!(
        row,
        ("harness_failure".into(), "TargetPreparation".into(), 0)
    );
}

#[test]
fn capability_and_target_manifests_bind_probe_and_runtime_evidence() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("identity.sh", b"#!/bin/sh\nexit 0\n");
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let output = workspace.run(&configuration);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let connection = workspace.database();
    let (capability_id, target_build_id, target_id, manifest_id, identity_digest): (
        String,
        String,
        String,
        String,
        String,
    ) = connection
        .query_row(
            "SELECT r.capability_manifest_artifact_id, b.id, b.target_artifact_id,
                    b.manifest_artifact_id, b.identity_digest
             FROM runs r JOIN target_builds b ON b.id = r.target_build_id",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("persisted runtime-bound identity");
    let capability = String::from_utf8(workspace.object_bytes(&capability_id))
        .expect("capability manifest UTF-8");
    assert!(capability.contains("probe_evidence_artifact"));
    let target_manifest =
        String::from_utf8(workspace.object_bytes(&manifest_id)).expect("target manifest UTF-8");
    assert!(!target_manifest.contains("runtime_snapshot="));
    assert!(target_manifest.contains("harness_artifact="));
    assert!(target_manifest.contains("bubblewrap_artifact="));
    assert!(target_manifest.contains("prlimit_artifact="));
    assert!(target_manifest.contains("unresolved_host_runtime=true"));
    assert!(target_manifest.contains(&target_id));
    assert_eq!(identity_digest, manifest_id);
    assert_eq!(target_build_id, format!("target-build-{manifest_id}"));
}

#[test]
fn identical_target_and_runtime_inputs_reuse_one_target_build_identity_across_runs() {
    let workspace = TemporaryWorkspace::new();
    let target = workspace.write_target("stable-identity.sh", b"#!/bin/sh\nexit 0\n");
    let configuration = workspace.write_configuration(&target, "[]", 2_000, 1);
    let first = workspace.run(&configuration);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let second = workspace.run(&configuration);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let connection = workspace.database();
    let counts: (i64, i64) = connection
        .query_row(
            "SELECT COUNT(*), COUNT(DISTINCT target_build_id) FROM runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("run/build identity counts");
    assert_eq!(counts, (2, 1));
    let target_build_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM target_builds", [], |row| row.get(0))
        .expect("target build count");
    assert_eq!(target_build_count, 1);
}
