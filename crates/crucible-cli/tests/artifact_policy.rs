use crucible_cli::{
    object_address_for_artifact, object_address_matches_id, prepare_artifact_publication,
    stored_artifact_is_exact, ObjectAddress, StoredArtifactSnapshot,
};
use crucible_core::{ArtifactId, ArtifactRef};
#[allow(unused_imports)]
use vstd::prelude::*;

const ABC_ID: &str = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[test]
fn verified_publication_plan_derives_a_safe_canonical_object_address() {
    let publication = prepare_artifact_publication(b"abc").expect("prepare artifact");
    assert_eq!(publication.artifact.id.as_str(), ABC_ID);
    assert_eq!(publication.artifact.size_bytes, 3);
    assert_eq!(
        publication.address,
        ObjectAddress {
            algorithm: String::from("sha256"),
            first: String::from("ba"),
            second: String::from("78"),
            object_name: String::from(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
        }
    );
    assert!(object_address_matches_id(
        &publication.artifact.id,
        &publication.address
    ));
}

#[test]
fn malformed_ids_never_produce_an_object_address() {
    for id in [
        "../outside",
        "sha256:../../outside",
        "sha256:BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
        "blake3:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    ] {
        assert!(object_address_for_artifact(&ArtifactId::new(String::from(id))).is_err());
    }
}

#[test]
fn stored_snapshot_requires_identity_bytes_and_provenance_to_agree() {
    let publication = prepare_artifact_publication(b"abc").expect("prepare artifact");
    let exact = StoredArtifactSnapshot {
        object_is_file: true,
        record: Some(ArtifactRef {
            id: ArtifactId::new(String::from(ABC_ID)),
            size_bytes: 3,
            media_type: None,
        }),
        record_algorithm: Some(String::from("sha256")),
        record_digest: Some(String::from(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )),
        contents: b"abc".to_vec(),
        matching_import_count: 1,
    };
    assert!(stored_artifact_is_exact(
        &publication.artifact,
        true,
        &exact
    ));

    let mut corrupt = exact;
    corrupt.contents = b"abd".to_vec();
    assert!(!stored_artifact_is_exact(
        &publication.artifact,
        true,
        &corrupt
    ));
    corrupt.contents = b"abc".to_vec();
    corrupt.record_digest = Some("0".repeat(64));
    assert!(!stored_artifact_is_exact(
        &publication.artifact,
        true,
        &corrupt
    ));
    corrupt.record_digest = Some(String::from(
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    ));
    corrupt.matching_import_count = 0;
    assert!(!stored_artifact_is_exact(
        &publication.artifact,
        true,
        &corrupt
    ));
}
