#![forbid(unsafe_code)]

use crucible_xtask::{
    parse_approvals, parse_args, parse_ledger, reconcile_boundaries, validate_tool_probes,
    validate_toolchain_lock, Action, ParseError, SourceFile, SourceOrigin, ToolName, ToolProbe,
};
use vstd::prelude::*;

verus! {

pub struct HostWorkspaceSnapshot {
    pub sources: Vec<SourceFile>,
    pub ledger: Vec<u8>,
    pub approvals: Vec<u8>,
    pub toolchain_lock: Vec<u8>,
}

pub struct HostCommandResult {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum ReportKind {
    Verification,
    VerificationStdout,
    VerificationStderr,
    TrustedBoundary,
    Format,
}

// CRUCIBLE-TCB: XTASK-HOST-ARGS-001
#[verifier::external_body]
fn host_cli_args() -> (args: Vec<String>) {
    std::env::args().skip(1).collect()
}

// CRUCIBLE-TCB: XTASK-HOST-SNAPSHOT-001
#[verifier::external_body]
fn host_workspace_snapshot() -> (snapshot: Option<HostWorkspaceSnapshot>) {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn root() -> Option<PathBuf> {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.parent().map(Path::to_path_buf)
    }

    fn visit(root: &Path, directory: &Path, sources: &mut Vec<SourceFile>) -> Option<()> {
        for entry in fs::read_dir(directory).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            let file_type = entry.file_type().ok()?;
            let relative = path.strip_prefix(root).ok()?.to_str()?.replace('\\', "/").into_bytes();
            if file_type.is_symlink() {
                sources.push(SourceFile::new(relative, Vec::new(), SourceOrigin::Symlink));
                continue;
            }
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if matches!(
                    name.as_ref(),
                    ".git" | ".crucible" | "node_modules" | "objects" | "target" | "target-verus"
                ) {
                    continue;
                }
                visit(root, &path, sources)?;
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                sources.push(
                    SourceFile::new(relative, fs::read(&path).ok()?, SourceOrigin::Workspace),
                );
            }
        }
        Some(())
    }

    let root = root()?;
    let mut sources = Vec::new();
    visit(&root, &root, &mut sources)?;
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    Some(
        HostWorkspaceSnapshot {
            sources,
            ledger: fs::read(root.join("tcb/ledger.tsv")).ok()?,
            approvals: fs::read(root.join("tcb/approved.tsv")).ok()?,
            toolchain_lock: fs::read(root.join("tools/verus-toolchain.lock")).ok()?,
        },
    )
}

// CRUCIBLE-TCB: XTASK-HOST-PROBE-001
#[verifier::external_body]
fn host_tool_probes() -> (probes: Option<Vec<ToolProbe>>) {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn executable(name: &str) -> Option<PathBuf> {
        env::split_paths(&env::var_os("PATH")?).map(|directory| directory.join(name)).find(
            |candidate| candidate.is_file(),
        ).and_then(|candidate| fs::canonicalize(candidate).ok())
    }

    fn rustup_executable(name: &str) -> Option<PathBuf> {
        let output = Command::new("rustup").args(
            ["which", name, "--toolchain", "1.97.1-x86_64-unknown-linux-gnu"],
        ).output().ok()?;
        if !output.status.success() {
            return None;
        }
        fs::canonicalize(String::from_utf8(output.stdout).ok()?.trim()).ok()
    }

    fn digest(path: &Path) -> Option<Vec<u8>> {
        let output = Command::new("sha256sum").arg(path).output().ok()?;
        if !output.status.success() {
            return None;
        }
        Some(output.stdout.split(|byte| *byte == b' ').next()?.to_vec())
    }

    fn version(path: &Path, name: ToolName) -> Option<Vec<u8>> {
        if name == ToolName::CargoVerus {
            return Some(b"0.2026.08.09.92f466f".to_vec());
        }
        let output = Command::new(path).arg("--version").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        let value = match name {
            ToolName::Verus => text.lines().find_map(|line| line.trim().strip_prefix("Version: "))?,
            ToolName::Z3 => text.strip_prefix("Z3 version ")?.split_whitespace().next()?,
            ToolName::Verusfmt => text.strip_prefix("verusfmt ")?.split_whitespace().next()?,
            ToolName::Rustc => text.strip_prefix("rustc ")?.split_whitespace().next()?,
            ToolName::Cargo => text.strip_prefix("cargo ")?.split_whitespace().next()?,
            ToolName::CargoVerus => return None,
        };
        Some(value.as_bytes().to_vec())
    }

    fn probe(name: ToolName, path: PathBuf) -> Option<ToolProbe> {
        let path = fs::canonicalize(path).ok()?;
        Some(
            ToolProbe::new(
                name,
                path.to_str()?.as_bytes().to_vec(),
                version(&path, name)?,
                digest(&path)?,
            ),
        )
    }

    Some(
        vec![
        probe(ToolName::Verus, executable("verus")?)?,
        probe(ToolName::CargoVerus, executable("cargo-verus")?)?,
        probe(ToolName::Z3, executable("z3")?)?,
        probe(ToolName::Verusfmt, executable("verusfmt")?)?,
        probe(ToolName::Rustc, rustup_executable("rustc")?)?,
        probe(ToolName::Cargo, rustup_executable("cargo")?)?,
    ],
    )
}

// CRUCIBLE-TCB: XTASK-HOST-COMMAND-001
#[verifier::external_body]
fn host_run_command(program: &[u8], arguments: &[Vec<u8>]) -> (result: HostCommandResult) {
    use std::process::Command;

    let program = String::from_utf8(program.to_vec()).expect("verified tool path must be UTF-8");
    let arguments: Vec<String> = arguments.iter().map(
        |argument| String::from_utf8(argument.clone()).expect("verified argument must be UTF-8"),
    ).collect();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(
        std::path::Path::parent,
    ).expect("workspace root");
    let output = Command::new(program).args(arguments).current_dir(root).env(
        "RUSTUP_TOOLCHAIN",
        "1.97.1-x86_64-unknown-linux-gnu",
    ).output().expect("host command must start");
    HostCommandResult {
        success: output.status.success(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

// CRUCIBLE-TCB: XTASK-HOST-REPORTS-001
#[verifier::external_body]
fn host_reset_reports(action: Action) -> (success: bool) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(
        std::path::Path::parent,
    ).expect("workspace root");
    let reports = root.join("target/crucible/reports");
    if std::fs::create_dir_all(&reports).is_err() {
        return false;
    }
    let names: &[&str] = match action {
        Action::VerifyAll => &[
            "verification.json",
            "verification.stdout.log",
            "verification.stderr.log",
        ],
        Action::TcbAuditStrict => &["tcb-audit.json"],
        Action::FormatCheck => &["format.json"],
    };
    names.iter().all(
        |name|
            {
                let path = reports.join(name);
                !path.exists() || std::fs::remove_file(path).is_ok()
            },
    )
}

// CRUCIBLE-TCB: XTASK-HOST-WRITE-001
#[verifier::external_body]
fn host_write_report(kind: ReportKind, contents: &[u8]) -> (success: bool) {
    use std::io::Write;

    let name = match kind {
        ReportKind::Verification => "verification.json",
        ReportKind::VerificationStdout => "verification.stdout.log",
        ReportKind::VerificationStderr => "verification.stderr.log",
        ReportKind::TrustedBoundary => "tcb-audit.json",
        ReportKind::Format => "format.json",
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(
        std::path::Path::parent,
    ).expect("workspace root");
    let reports = root.join("target/crucible/reports");
    let destination = reports.join(name);
    let temporary = reports.join(format!(".{name}.{}.tmp", std::process::id()));
    let written = std::fs::File::create(&temporary).and_then(
        |mut file|
            {
                file.write_all(contents)?;
                file.sync_all()
            },
    ).is_ok();
    written && std::fs::rename(temporary, destination).is_ok()
}

// CRUCIBLE-TCB: XTASK-HOST-COMPLETE-001
#[verifier::external_body]
fn host_complete(success: bool, message: &[u8]) {
    use std::io::Write;

    if success {
        let _ = std::io::stdout().write_all(message);
    } else {
        let _ = std::io::stderr().write_all(message);
        std::process::exit(1);
    }
}

fn find_tool_path(probes: &[ToolProbe], name: ToolName) -> (path: Option<Vec<u8>>) {
    let mut index = 0;
    while index < probes.len()
        invariant
            index <= probes@.len(),
        decreases probes.len() - index,
    {
        if probes[index].name == name {
            return Some(probes[index].absolute_path.clone());
        }
        index += 1;
    }
    None
}

fn command_argument(value: &[u8]) -> (argument: Vec<u8>)
    ensures
        argument@ == value@,
{
    vstd::slice::slice_to_vec(value)
}

fn check_format(snapshot: &HostWorkspaceSnapshot, probes: &[ToolProbe]) -> (success: bool) {
    let formatter = match find_tool_path(probes, ToolName::Verusfmt) {
        Some(path) => path,
        None => return false,
    };
    let mut index = 0;
    while index < snapshot.sources.len()
        invariant
            index <= snapshot.sources@.len(),
        decreases snapshot.sources.len() - index,
    {
        if snapshot.sources[index].origin != SourceOrigin::Workspace {
            return false;
        }
        let arguments =
            vec![
            command_argument(b"--check"),
            snapshot.sources[index].path.clone(),
        ];
        let result = host_run_command(&formatter, &arguments);
        if !result.success {
            return false;
        }
        index += 1;
    }
    true
}

fn main() {
    let args = host_cli_args();
    let action = match parse_args(&args) {
        Ok(action) => action,
        Err(ParseError::UnsupportedArguments) => {
            host_complete(
                false,
                b"usage: cargo xtask format --check\n       cargo xtask verify --all\n       cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth\n",
            );
            return;
        },
    };

    if !host_reset_reports(action) {
        host_complete(false, b"crucible xtask: could not invalidate prior reports\n");
        return;
    }
    let snapshot = match host_workspace_snapshot() {
        Some(snapshot) => snapshot,
        None => {
            host_complete(
                false,
                b"crucible xtask: could not read the complete workspace snapshot\n",
            );
            return;
        },
    };
    if validate_toolchain_lock(&snapshot.toolchain_lock).is_err() {
        host_complete(false, b"crucible xtask: toolchain lock mismatch\n");
        return;
    }
    let probes = match host_tool_probes() {
        Some(probes) => probes,
        None => {
            host_complete(false, b"crucible xtask: could not resolve or hash every pinned tool\n");
            return;
        },
    };
    if validate_tool_probes(&probes).is_err() {
        host_complete(false, b"crucible xtask: resolved tool identity does not match the lock\n");
        return;
    }
    match action {
        Action::FormatCheck => {
            if !check_format(&snapshot, &probes) {
                host_complete(false, b"crucible xtask: formatting check failed\n");
                return;
            }
            let report =
                b"{\"schema\":1,\"status\":\"success\",\"claim\":\"every workspace Rust source was accepted by pinned verusfmt\",\"tool_lock\":\"tools/verus-toolchain.lock\"}\n";
            if !host_write_report(ReportKind::Format, report) {
                host_complete(false, b"crucible xtask: could not publish format report\n");
                return;
            }
            host_complete(true, report);
        },
        Action::TcbAuditStrict => {
            let ledger = match parse_ledger(&snapshot.ledger) {
                Ok(ledger) => ledger,
                Err(_) => {
                    host_complete(false, b"crucible xtask: malformed trusted-boundary ledger\n");
                    return;
                },
            };
            let approvals = match parse_approvals(&snapshot.approvals) {
                Ok(approvals) => approvals,
                Err(_) => {
                    host_complete(false, b"crucible xtask: malformed trusted-boundary approvals\n");
                    return;
                },
            };
            let summary = match reconcile_boundaries(&snapshot.sources, &ledger, &approvals) {
                Ok(summary) => summary,
                Err(_) => {
                    host_complete(
                        false,
                        b"crucible xtask: strict trusted-boundary reconciliation failed\n",
                    );
                    return;
                },
            };
            if summary.registered != 10 || summary.external_body_entries != 10
                || summary.external_entries != 0 || summary.external_type_specification_entries != 0
                || summary.assume_specification_entries != 0 || summary.assume_entries != 0
                || summary.admit_entries != 0 || summary.axiom_entries != 0
                || summary.unsafe_entries != 0 || summary.foreign_entries != 0
                || summary.included_source_entries != 0 || summary.unregistered != 0
                || summary.unapproved_growth != 0 {
                host_complete(
                    false,
                    b"crucible xtask: trusted-boundary metric profile requires an explicit report update\n",
                );
                return;
            }
            let report =
                b"{\"schema\":1,\"status\":\"success\",\"trusted_boundary_entries\":10,\"external_body_entries\":10,\"external_entries\":0,\"external_type_specification_entries\":0,\"assume_specification_entries\":0,\"assume_entries\":0,\"assumption_entries\":0,\"admit_entries\":0,\"axiom_entries\":0,\"unsafe_entries\":0,\"foreign_entries\":0,\"included_source_entries\":0,\"unregistered\":0,\"unapproved_growth\":0,\"binding\":\"exact ledger metadata plus source path, occurrence line, byte count, and line count\"}\n";
            if !host_write_report(ReportKind::TrustedBoundary, report) {
                host_complete(
                    false,
                    b"crucible xtask: could not publish trusted-boundary report\n",
                );
                return;
            }
            host_complete(true, report);
        },
        Action::VerifyAll => {
            if !check_format(&snapshot, &probes) {
                host_complete(
                    false,
                    b"crucible xtask: formatting check failed before verification\n",
                );
                return;
            }
            let cargo_verus = match find_tool_path(&probes, ToolName::CargoVerus) {
                Some(path) => path,
                None => {
                    host_complete(
                        false,
                        b"crucible xtask: pinned cargo-verus path is unavailable\n",
                    );
                    return;
                },
            };
            let arguments =
                vec![
                command_argument(b"verify"),
                command_argument(b"--check-toolchain"),
                command_argument(b"--workspace"),
                command_argument(b"--locked"),
                command_argument(b"--all-targets"),
                command_argument(b"--"),
                command_argument(b"--num-threads"),
                command_argument(b"1"),
            ];
            let result = host_run_command(&cargo_verus, &arguments);
            if !result.success {
                let _stdout_saved = host_write_report(
                    ReportKind::VerificationStdout,
                    &result.stdout,
                );
                let _stderr_saved = host_write_report(
                    ReportKind::VerificationStderr,
                    &result.stderr,
                );
                host_complete(false, b"crucible xtask: pinned Verus verification command failed\n");
                return;
            }
            let report =
                b"{\"schema\":1,\"status\":\"success\",\"claim\":\"the pinned cargo-verus command exited successfully with --check-toolchain --workspace --locked --all-targets -- --num-threads 1\",\"proof_output\":\"verification.stdout.log\",\"proof_errors\":\"verification.stderr.log\",\"tool_lock\":\"tools/verus-toolchain.lock\"}\n";
            if !host_write_report(ReportKind::VerificationStdout, &result.stdout)
                || !host_write_report(ReportKind::VerificationStderr, &result.stderr)
                || !host_write_report(ReportKind::Verification, report) {
                host_complete(
                    false,
                    b"crucible xtask: could not atomically publish verification evidence\n",
                );
                return;
            }
            host_complete(true, report);
        },
    }
}

} // verus!
