use crucible_cli::{parse_cli_args, CliAction, MAX_CONFIGURATION_SOURCE_BYTES};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

fn temporary_path(kind: &str) -> PathBuf {
    let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "crucible-config-{kind}-{}-{sequence}",
        std::process::id()
    ))
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn new(contents: &[u8]) -> Self {
        let path = temporary_path("file.yaml");
        std::fs::write(&path, contents).expect("write configuration fixture");
        Self { path }
    }

    fn sparse(length: u64) -> Self {
        let path = temporary_path("large.yaml");
        let file = std::fs::File::create(&path).expect("create sparse configuration fixture");
        file.set_len(length)
            .expect("size sparse configuration fixture");
        Self { path }
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new() -> Self {
        let path = temporary_path("directory");
        std::fs::create_dir(&path).expect("create configuration fixture directory");
        Self { path }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.path);
    }
}

fn config_validate(path: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["config", "validate"])
        .arg(path)
        .output()
        .expect("run config validate")
}

const MINIMAL_CONFIGURATION: &[u8] = b"version: 1\nlanguage: {profile: crucible-yaml-1}\nproject: {name: demo}\ntarget: {adapter: cli, command: ./demo, args: []}\nexecution: {timeout_ms: 1, memory_mb: 1, max_processes: 1, max_output_mb: 1, network: false, required_capabilities: []}\noracles: {process_exit: {allowed_codes: [0], timeout_is_failure: true}}\ninputs: {corpus: []}\nengines: {fuzz: {enabled: false, modes: [], native_backends: []}, property: {enabled: false}, differential: {enabled: false}, metamorphic: {enabled: false}, fault: {enabled: false}, concurrency: {enabled: false}, symbolic: {enabled: false}, mutation: {enabled: false}}\nsanitizers: {address: false, undefined: false, thread: false, memory: false, leak: false}\ncampaign: {duration: 1s, workers: 1, seed: 0}\nstorage: {root: .crucible}\nverification: {verus: {required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}\n";

#[test]
fn verified_argument_parser_exposes_both_configuration_commands() {
    let validate = parse_cli_args(&[
        String::from("config"),
        String::from("validate"),
        String::from("crucible.yaml"),
    ])
    .unwrap();
    assert_eq!(
        validate,
        CliAction::ConfigValidate(String::from("crucible.yaml"))
    );

    let canonicalize = parse_cli_args(&[
        String::from("config"),
        String::from("canonicalize"),
        String::from("crucible.yaml"),
    ])
    .unwrap();
    assert_eq!(
        canonicalize,
        CliAction::ConfigCanonicalize(String::from("crucible.yaml"))
    );
}

#[test]
fn configuration_commands_run_end_to_end_and_report_typed_failures() {
    let valid = TemporaryFile::new(MINIMAL_CONFIGURATION);
    let validation = config_validate(&valid.path);
    assert!(
        validation.status.success(),
        "{}",
        String::from_utf8_lossy(&validation.stderr)
    );
    let digest = String::from_utf8(validation.stdout).unwrap();
    assert!(digest.starts_with("sha256:"));
    assert_eq!(digest.trim().len(), 71);

    let canonicalization = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["config", "canonicalize"])
        .arg(&valid.path)
        .output()
        .expect("run config canonicalize");
    assert!(canonicalization.status.success());
    assert!(canonicalization.stdout.starts_with(b"{\"version\":1,"));

    let invalid = TemporaryFile::new(b"version: [1]\n");
    let rejection = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["config", "validate"])
        .arg(&invalid.path)
        .output()
        .expect("run rejected config validate");
    assert!(!rejection.status.success());
    assert!(String::from_utf8_lossy(&rejection.stderr)
        .starts_with("crucible config: WrongValueKind at byte "));
}

#[test]
fn configuration_reader_rejects_non_files_symlinks_and_oversized_sources() {
    let directory = TemporaryDirectory::new();
    let directory_rejection = config_validate(&directory.path);
    assert!(!directory_rejection.status.success());
    assert_eq!(
        directory_rejection.stderr,
        b"crucible config: unsafe configuration source\n"
    );

    let oversized = TemporaryFile::sparse(MAX_CONFIGURATION_SOURCE_BYTES + 1);
    let oversized_rejection = config_validate(&oversized.path);
    assert!(!oversized_rejection.status.success());
    assert_eq!(
        oversized_rejection.stderr,
        b"crucible config: SourceByteLimitExceeded at byte 16777216\n"
    );

    #[cfg(unix)]
    {
        let valid = TemporaryFile::new(MINIMAL_CONFIGURATION);
        let link_path = temporary_path("link.yaml");
        std::os::unix::fs::symlink(&valid.path, &link_path)
            .expect("create configuration symlink fixture");
        let symlink_rejection = config_validate(&link_path);
        let _ = std::fs::remove_file(&link_path);
        assert!(!symlink_rejection.status.success());
        assert_eq!(
            symlink_rejection.stderr,
            b"crucible config: unsafe configuration source\n"
        );
    }
}

#[test]
fn configuration_reader_uses_no_follow_admission_or_fails_closed() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("read CLI source policy fixture");
    let configuration_reader = source
        .split(concat!("// CRUCIBLE-", "TCB: CLI-HOST-CONFIG-001"))
        .nth(1)
        .unwrap()
        .split(concat!("// CRUCIBLE-", "TCB: CLI-HOST-COMPLETE-001"))
        .next()
        .unwrap();

    #[cfg(unix)]
    {
        let unix_reader = configuration_reader
            .split("#[cfg(unix)]")
            .nth(1)
            .unwrap()
            .split("#[cfg(not(unix))]")
            .next()
            .unwrap();
        assert!(unix_reader.contains("rustix::fs::openat"));
        assert!(unix_reader.contains("OFlags::NOFOLLOW"));
        assert!(!unix_reader.contains("File::open"));
    }

    let non_unix_reader = configuration_reader
        .split("#[cfg(not(unix))]")
        .nth(1)
        .unwrap();
    assert!(non_unix_reader.contains("Err(HostConfigError::UnsupportedPlatform)"));
    assert!(!non_unix_reader.contains("File::open"));
}

#[test]
fn cli_canonicalization_preserves_declared_empty_values_and_emits_json_unicode() {
    let source = String::from_utf8(MINIMAL_CONFIGURATION.to_vec())
        .unwrap()
        .replace("project: {name: demo}", "project: {name: \"😀\"}")
        .replace("required_capabilities: []", "required_capabilities: [\"\"]")
        .replace("allowed_codes: [0]", "allowed_codes: []")
        .replace("corpus: []", "corpus: [\"\"]");
    let configuration = TemporaryFile::new(source.as_bytes());
    let output = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["config", "canonicalize"])
        .arg(&configuration.path)
        .output()
        .expect("run config canonicalize");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let decoded: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("canonical output must be accepted by an independent JSON parser");
    assert_eq!(decoded["project"]["name"], "😀");
    assert_eq!(decoded["execution"]["required_capabilities"][0], "");
    assert_eq!(
        decoded["oracles"]["process_exit"]["allowed_codes"],
        serde_json::json!([])
    );
    assert_eq!(decoded["inputs"]["corpus"][0], "");
}
