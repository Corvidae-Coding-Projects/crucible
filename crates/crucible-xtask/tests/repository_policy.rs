use crucible_xtask::{
    parse_approvals, parse_ledger, reconcile_boundaries, validate_toolchain_lock, SourceFile,
    SourceOrigin,
};
use std::fs;
use std::path::{Path, PathBuf};

fn visit(root: &Path, directory: &Path, sources: &mut Vec<SourceFile>) {
    for entry in fs::read_dir(directory).expect("read workspace directory") {
        let entry = entry.expect("read entry");
        let path = entry.path();
        let file_type = entry.file_type().expect("file type");
        let relative = path
            .strip_prefix(root)
            .expect("relative path")
            .to_str()
            .expect("UTF-8 path")
            .replace('\\', "/")
            .into_bytes();
        if file_type.is_symlink() {
            sources.push(SourceFile::new(relative, Vec::new(), SourceOrigin::Symlink));
        } else if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !matches!(
                name.as_ref(),
                ".git" | ".crucible" | "node_modules" | "objects" | "target" | "target-verus"
            ) {
                visit(root, &path, sources);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(SourceFile::new(
                relative,
                fs::read(path).expect("source bytes"),
                SourceOrigin::Workspace,
            ));
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn checked_in_repository_satisfies_strict_tcb_and_toolchain_policy() {
    let root = workspace_root();
    let mut sources = Vec::new();
    visit(&root, &root, &mut sources);
    let ledger = parse_ledger(&fs::read(root.join("tcb/ledger.tsv")).expect("ledger"))
        .expect("valid ledger");
    let approvals = parse_approvals(&fs::read(root.join("tcb/approved.tsv")).expect("approvals"))
        .expect("valid approvals");
    reconcile_boundaries(&sources, &ledger, &approvals).expect("strict TCB reconciliation");
    validate_toolchain_lock(
        &fs::read(root.join("tools/verus-toolchain.lock")).expect("toolchain lock"),
    )
    .expect("exact toolchain lock");
}

#[test]
fn code_ci_preserves_the_cold_verification_evidence_until_upload() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/code.yml"))
        .expect("code workflow");
    for step in [
        "- name: Reproduce all Verus proofs",
        "- name: Seal cold proof evidence",
        "- name: Validate machine-readable reports",
        "- name: Upload proof and trusted-boundary evidence",
    ] {
        assert_eq!(
            workflow.matches(step).count(),
            1,
            "evidence steps must each occur exactly once: {step}"
        );
    }
    assert_eq!(
        workflow.matches("cargo xtask verify --all").count(),
        1,
        "a second incremental verification overwrites the cold proof-output report"
    );
    let (before_verify, after_verify) = workflow
        .split_once("- name: Reproduce all Verus proofs")
        .expect("unique proof-reproduction step");
    assert!(!before_verify.contains("- name: Seal cold proof evidence"));
    let (verify, after_seal) = after_verify
        .split_once("- name: Seal cold proof evidence")
        .expect("proof output is sealed immediately after reproduction");
    assert!(verify.contains("cargo xtask verify --all"));
    let (seal, after_validate) = after_seal
        .split_once("- name: Validate machine-readable reports")
        .expect("sealed evidence is validated later");
    let (validate, upload) = after_validate
        .split_once("- name: Upload proof and trusted-boundary evidence")
        .expect("validation precedes a real upload step");
    assert!(!upload.contains("- name: Upload proof and trusted-boundary evidence"));

    assert!(workflow.contains("EVIDENCE_ROOT:"));
    assert!(seal.contains("$GITHUB_RUN_ID"));
    assert!(seal.contains("$GITHUB_RUN_ATTEMPT"));
    assert!(seal.contains("$GITHUB_SHA"));
    assert!(seal.contains("git rev-parse 'HEAD^{tree}'"));
    assert!(seal.contains("tools/verus-toolchain.lock"));
    assert!(seal.contains("verification.stdout.log"));
    assert!(seal.contains("verification.stderr.log"));
    assert!(seal.contains("SHA256SUMS"));
    assert!(seal.contains("chmod -R a-w"));

    assert!(validate.contains("sha256sum --check SHA256SUMS"));
    assert!(validate.contains("test -s verification.stdout.log"));
    assert!(validate.contains("test -f verification.stderr.log"));
    assert!(validate.contains(".head_sha == $head_sha"));
    assert!(!validate.contains("cargo xtask verify"));
    assert!(upload.contains("path: target/crucible/evidence/"));
}
