use crucible_core::{
    parse_artifact_id, sha256, ArtifactId, ArtifactIdParseError, ArtifactIdentityError,
    ArtifactRef, ContentDigest, DigestAlgorithm, DigestDecodeError, Sha256Digest,
};
use std::string::String;

fn assert_digest(input: &[u8], expected: &str) {
    let digest = sha256(input).expect("test vector is within the SHA-256 length bound");
    assert_eq!(digest.to_hex(), expected);
}

#[test]
fn nist_sha256_vectors_cover_empty_single_and_multi_block_messages() {
    assert_digest(
        b"",
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert_digest(
        b"abc",
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    assert_digest(
        b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    );
}

#[test]
fn nist_million_byte_vector_exercises_long_block_folding() {
    let input = vec![b'a'; 1_000_000];
    assert_digest(
        &input,
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
    );
}

#[test]
fn canonical_digest_codec_round_trips_and_rejects_noncanonical_text() {
    let canonical =
        String::from("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let digest = Sha256Digest::from_hex(canonical.clone()).expect("canonical digest must decode");
    assert_eq!(digest.to_hex(), canonical);

    let uppercase =
        String::from("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD");
    assert_eq!(
        Sha256Digest::from_hex(uppercase),
        Err(DigestDecodeError::NonCanonicalHex)
    );
    assert_eq!(
        Sha256Digest::from_hex(String::from("00")),
        Err(DigestDecodeError::WrongLength)
    );
    assert_eq!(
        Sha256Digest::from_hex(String::from(
            "ga7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )),
        Err(DigestDecodeError::NonCanonicalHex)
    );
}

#[test]
fn artifact_reference_is_content_derived_and_checks_size_and_digest() {
    let media_type = String::from("text/plain");
    let artifact = ArtifactRef::from_bytes(b"abc", Some(media_type.clone()))
        .expect("small input is within the SHA-256 length bound");

    assert_eq!(artifact.size_bytes, 3);
    assert_eq!(artifact.media_type, Some(media_type));
    assert_eq!(
        artifact.id.clone().into_inner(),
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(artifact.verify(b"abc"), Ok(()));
    assert_eq!(
        artifact.verify(b"abd"),
        Err(ArtifactIdentityError::DigestMismatch)
    );

    let wrong_size = ArtifactRef {
        id: artifact.id,
        size_bytes: 4,
        media_type: artifact.media_type,
    };
    assert_eq!(
        wrong_size.verify(b"abc"),
        Err(ArtifactIdentityError::SizeMismatch)
    );
}

#[test]
fn content_digest_dispatch_is_explicit_and_algorithm_labeled() {
    let digest =
        ContentDigest::from_bytes(b"abc").expect("small input is within the SHA-256 length bound");
    assert_eq!(digest.algorithm(), DigestAlgorithm::Sha256);
    assert_eq!(
        digest.into_artifact_id().into_inner(),
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn artifact_id_parser_and_verifier_dispatch_typed_algorithm_failures() {
    let valid = ArtifactRef::from_bytes(b"abc", None)
        .expect("small input is within the SHA-256 length bound");
    assert_eq!(
        parse_artifact_id(&valid.id)
            .expect("generated artifact IDs are canonical")
            .algorithm(),
        DigestAlgorithm::Sha256
    );

    let unsupported = ArtifactId::new(String::from(
        "sha512:ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
         2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
    ));
    assert_eq!(
        parse_artifact_id(&unsupported),
        Err(ArtifactIdParseError::UnsupportedAlgorithm)
    );
    assert_eq!(
        ArtifactRef {
            id: unsupported,
            size_bytes: 3,
            media_type: None
        }
        .verify(b"abc"),
        Err(ArtifactIdentityError::UnsupportedAlgorithm)
    );

    for malformed in [
        String::from("sha256:00"),
        String::from("sha256:BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"),
        String::from("sha256-without-a-separator"),
    ] {
        let id = ArtifactId::new(malformed);
        assert_eq!(
            parse_artifact_id(&id),
            Err(ArtifactIdParseError::MalformedArtifactId)
        );
        assert_eq!(
            ArtifactRef {
                id,
                size_bytes: 3,
                media_type: None
            }
            .verify(b"abc"),
            Err(ArtifactIdentityError::MalformedArtifactId)
        );
    }
}

fn decode_test_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0, "fixture hex must contain whole bytes");
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("fixture hex is ASCII");
            u8::from_str_radix(text, 16).expect("fixture contains hexadecimal bytes")
        })
        .collect()
}

#[test]
fn checksum_pinned_nist_cavp_short_messages_cover_every_padding_boundary() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/nist-cavp/SHA256ShortMsg.rsp");
    let fixture =
        std::fs::read_to_string(fixture_path).expect("checked-in CAVP fixture is readable");
    assert_digest(
        fixture.as_bytes(),
        "294ecec26959357405a621121bbfb01db4d45b9e834624b2d71aedd94ffde019",
    );
    let mut length_bits = None;
    let mut message = None;
    let mut count = 0;
    let mut covered_lengths = Vec::new();

    for line in fixture.lines() {
        if let Some(value) = line.strip_prefix("Len = ") {
            length_bits = Some(value.parse::<usize>().expect("CAVP length is decimal"));
        } else if let Some(value) = line.strip_prefix("Msg = ") {
            message = Some(value);
        } else if let Some(expected) = line.strip_prefix("MD = ") {
            let bits = length_bits.take().expect("digest follows a length");
            let encoded_message = message.take().expect("digest follows a message");
            assert_eq!(bits % 8, 0, "fixture is byte-oriented");
            let bytes = if bits == 0 {
                Vec::new()
            } else {
                decode_test_hex(encoded_message)
            };
            assert_eq!(bytes.len(), bits / 8);
            assert_digest(&bytes, expected);
            covered_lengths.push(bytes.len());
            count += 1;
        }
    }

    assert_eq!(
        count, 65,
        "the complete NIST SHA-256 ShortMsg member is exercised"
    );
    for boundary in [55, 56, 63, 64] {
        assert!(
            covered_lengths.contains(&boundary),
            "missing {boundary}-byte boundary vector"
        );
    }
}

#[test]
fn independently_cross_checked_sixty_five_byte_binary_vector() {
    let input: Vec<u8> = (0..=64).collect();
    assert_digest(
        &input,
        "4bfd2c8b6f1eec7a2afeb48b934ee4b2694182027e6d0fc075074f2fabb31781",
    );
}
