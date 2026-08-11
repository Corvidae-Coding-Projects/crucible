use std::path::{Path, PathBuf};
use std::process::Command;
#[allow(unused_imports)]
use vstd::prelude::*;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crucible-xtask"))
        .args(arguments)
        .current_dir(workspace_root())
        .output()
        .expect("xtask process")
}

#[test]
fn public_xtask_interfaces_run_end_to_end() {
    let format = run(&["format", "--check"]);
    assert!(
        format.status.success(),
        "{}",
        String::from_utf8_lossy(&format.stderr)
    );
    assert!(workspace_root()
        .join("target/crucible/reports/format.json")
        .is_file());

    let audit = run(&[
        "tcb-audit",
        "--deny-unregistered",
        "--deny-unapproved-growth",
    ]);
    assert!(
        audit.status.success(),
        "{}",
        String::from_utf8_lossy(&audit.stderr)
    );
    assert!(workspace_root()
        .join("target/crucible/reports/tcb-audit.json")
        .is_file());
    let audit_report =
        std::fs::read_to_string(workspace_root().join("target/crucible/reports/tcb-audit.json"))
            .expect("TCB report");
    assert!(audit_report.contains("\"trusted_boundary_entries\":7"));
    assert!(audit_report.contains("\"external_body_entries\":7"));
    assert!(audit_report.contains("\"assumption_entries\":0"));
    assert!(audit_report.contains("\"axiom_entries\":0"));
    assert!(audit_report.contains("\"unsafe_entries\":0"));
    assert!(audit_report.contains("\"foreign_entries\":0"));

    let invalid = run(&["verify"]);
    assert!(!invalid.status.success());
    assert!(
        String::from_utf8_lossy(&invalid.stderr).starts_with("usage: cargo xtask format --check")
    );
}
