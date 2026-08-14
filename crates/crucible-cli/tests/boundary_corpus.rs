use crucible_core::{ArtifactId, ArtifactRef};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MAX_FIXTURE_BYTES: u64 = 8_192;

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/boundaries")
}

fn read(relative: &str) -> Vec<u8> {
    let path = corpus_root().join(relative);
    let metadata = std::fs::symlink_metadata(&path).expect("fixture metadata");
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    assert!(
        metadata.len() <= MAX_FIXTURE_BYTES,
        "oversized fixture: {relative}"
    );
    std::fs::read(path).expect("fixture bytes")
}

fn key_values(bytes: &[u8]) -> BTreeMap<String, String> {
    let text = std::str::from_utf8(bytes).expect("UTF-8 fixture");
    text.lines()
        .map(|line| line.split_once('=').expect("key=value fixture"))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

#[test]
fn every_declared_boundary_fixture_is_bounded_and_rejected_or_contained() {
    let resource_yaml = read("yaml/resource-limit.yaml");
    assert!(resource_yaml
        .windows(11)
        .any(|window| window == b"&recursive "));
    assert!(resource_yaml
        .windows(10)
        .any(|window| window == b"*recursive"));

    let duplicate_yaml = String::from_utf8(read("yaml/duplicate-and-cycle.yaml")).unwrap();
    assert_eq!(duplicate_yaml.matches("duplicate:").count(), 2);
    assert!(duplicate_yaml.contains("&cycle [*cycle]"));

    let corrupt = String::from_utf8(read("storage/corrupt-record.txt")).unwrap();
    let fields = corrupt.split_whitespace().collect::<Vec<_>>();
    assert_eq!(fields.len(), 3);
    let declared_size = fields[1].parse::<u64>().expect("declared size");
    let artifact = ArtifactRef {
        id: ArtifactId::new(fields[0].to_owned()),
        size_bytes: declared_size,
        media_type: None,
    };
    assert!(artifact.verify(fields[2].as_bytes()).is_err());

    let interrupted = String::from_utf8(read("storage/interrupted-publication.txt")).unwrap();
    assert!(interrupted.contains("verified-object-present"));
    assert!(interrupted.contains("database-reference-absent"));
    assert!(interrupted.contains("lease-status-active"));

    let plugin = String::from_utf8(read("plugin/protocol-violation.jsonl")).unwrap();
    let packets = plugin
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON packet"))
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 2);
    assert_ne!(packets[0]["schema"], "crucible.plugin.packet.v1");
    assert!(packets[0]["declared_bytes"].as_u64().unwrap() > 67_108_864);
    assert_eq!(packets[1]["stalled"], true);
    assert_eq!(packets[1]["terminal_record"], false);

    let scenario = key_values(&read("scenario/cancel.trace"));
    assert_eq!(scenario.get("event").map(String::as_str), Some("cancel"));
    assert_eq!(
        scenario.get("expected").map(String::as_str),
        Some("participant-cleaned")
    );

    let vm = key_values(&read("vm/escape-attempt.txt"));
    assert_eq!(vm.get("operation").map(String::as_str), Some("read"));
    assert!(vm.get("path").unwrap().starts_with("/host/"));
    assert_eq!(vm.get("expected").map(String::as_str), Some("not-found"));

    let hostile = String::from_utf8(read("hostile/prompt-injection.txt")).unwrap();
    assert!(hostile.contains("TARGET_ONLY_PROMPT_INJECTION_DO_NOT_LOG"));
    assert!(hostile.contains("hostile evidence"));

    let unregistered = String::from_utf8(read("proof/unregistered-boundary.rs.txt")).unwrap();
    assert!(unregistered.contains("#[verifier::external_body]"));
    assert!(!unregistered.contains("CRUCIBLE-TCB:"));
    let proof_failures = String::from_utf8(read("proof/failure-modes.tsv")).unwrap();
    for classification in [
        "proof-timeout",
        "proof-solver-failure",
        "proof-cache-identity-mismatch",
    ] {
        assert!(proof_failures.contains(classification));
    }
}
