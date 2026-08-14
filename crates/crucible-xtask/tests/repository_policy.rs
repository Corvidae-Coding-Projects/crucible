use crucible_xtask::{
    parse_approvals, parse_ledger, reconcile_boundaries, scan_boundaries, validate_toolchain_lock,
    SourceFile, SourceOrigin,
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

fn contains_allow_attribute(contents: &[u8]) -> bool {
    contents.split(|byte| *byte == b'\n').any(|line| {
        let compact: Vec<u8> = line
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect();
        compact.starts_with(&[35, 91, 97, 108, 108, 111, 119, 40])
            || compact.starts_with(&[35, 33, 91, 97, 108, 108, 111, 119, 40])
    })
}

#[test]
fn rust_sources_forbid_unchecked_allow_attributes() {
    let root = workspace_root();
    let mut sources = Vec::new();
    visit(&root, &root, &mut sources);
    let offenders: Vec<String> = sources
        .iter()
        .filter(|source| contains_allow_attribute(&source.contents))
        .map(|source| String::from_utf8_lossy(&source.path).into_owned())
        .collect();
    assert!(
        offenders.is_empty(),
        "replace broad allow attributes with better code or a narrow reasoned expect: {offenders:?}"
    );
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
    let occurrences = scan_boundaries(&sources).expect("scan boundaries");
    for occurrence in &occurrences {
        let approval = approvals
            .iter()
            .find(|approval| approval.id == occurrence.id)
            .expect("matching approval");
        let source = sources
            .iter()
            .find(|source| source.path == occurrence.source_path)
            .expect("matching source");
        assert_eq!(
            approval.occurrence_line,
            occurrence.line,
            "boundary line: {:?}",
            String::from_utf8_lossy(&occurrence.id)
        );
        assert_eq!(
            approval.source_bytes,
            source.contents.len(),
            "boundary bytes: {:?}",
            String::from_utf8_lossy(&occurrence.id)
        );
        assert_eq!(
            approval.source_lines,
            source
                .contents
                .iter()
                .filter(|byte| **byte == b'\n')
                .count()
                + 1,
            "boundary lines: {:?}",
            String::from_utf8_lossy(&occurrence.id)
        );
    }
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

#[test]
fn canonical_verification_is_single_threaded_and_reports_the_cap() {
    let xtask = fs::read_to_string(workspace_root().join("crates/crucible-xtask/src/main.rs"))
        .expect("xtask source");
    let verify = xtask
        .split_once("Action::VerifyAll => {")
        .expect("verification action")
        .1;
    let separator = verify
        .find("command_argument(b\"--\")")
        .expect("cargo/verus separator");
    let thread_flag = verify
        .find("command_argument(b\"--num-threads\")")
        .expect("single-thread verifier flag");
    let thread_count = verify
        .find("command_argument(b\"1\")")
        .expect("single verifier thread");
    assert!(separator < thread_flag && thread_flag < thread_count);
    assert!(verify.contains("--num-threads 1"));
}

#[test]
fn all_ci_tiers_are_explicit_resource_bounded_and_fail_closed() {
    let root = workspace_root();
    let every_commit =
        fs::read_to_string(root.join(".github/workflows/code.yml")).expect("every-commit workflow");
    let nightly =
        fs::read_to_string(root.join(".github/workflows/nightly.yml")).expect("nightly workflow");
    let weekly =
        fs::read_to_string(root.join(".github/workflows/weekly.yml")).expect("weekly workflow");

    for (name, workflow) in [
        ("every commit", every_commit.as_str()),
        ("nightly", nightly.as_str()),
        ("weekly", weekly.as_str()),
    ] {
        assert!(
            workflow.contains("CARGO_BUILD_JOBS: 1"),
            "{name} build jobs"
        );
        assert!(
            workflow.contains("RUST_TEST_THREADS: 1"),
            "{name} test threads"
        );
        assert!(workflow.contains("timeout-minutes:"), "{name} timeout");
        assert!(
            !workflow.contains("continue-on-error"),
            "{name} must fail closed"
        );
    }

    for required in [
        "cargo xtask verify --all",
        "cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test --all --locked -- --test-threads=1",
        "cargo test -p crucible-cli --test harness_selftest -- --test-threads=1",
        "cargo test -p crucible-cli --test build_fuzz_cli -- --test-threads=1",
        "cargo test -p crucible-core --locked -- --test-threads=1",
        "cargo test -p crucible-yaml --locked -- --test-threads=1",
    ] {
        assert!(every_commit.contains(required), "Tier 1 omitted {required}");
    }

    assert!(nightly.contains("cron: '17 3 * * *'"));
    for required in [
        "sanitizer",
        "extended_campaign",
        "metamorphic",
        "differential",
        "mutation",
        "storage_maintenance",
        "boundary_corpus",
        "cargo xtask verify --all",
    ] {
        assert!(nightly.contains(required), "Tier 2 omitted {required}");
    }

    assert!(weekly.contains("cron: '29 4 * * 0'"));
    for required in [
        "symbolic",
        "concurrency",
        "extended_campaign",
        "soak",
        "aarch64-unknown-linux-gnu",
        "cargo xtask verify --all",
        "cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth",
        "scenario_topology",
    ] {
        assert!(weekly.contains(required), "Tier 3 omitted {required}");
    }
}
