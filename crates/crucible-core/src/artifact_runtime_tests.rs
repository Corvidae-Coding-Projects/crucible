use super::sha256;

#[test]
fn empty_artifact_digest_unit_vector() {
    let digest = sha256(b"").expect("empty input is within the SHA-256 length bound");
    assert_eq!(
        digest.to_hex(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}
