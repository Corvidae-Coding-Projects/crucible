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
