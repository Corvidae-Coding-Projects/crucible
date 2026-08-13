use crucible_cli::{
    object_address_for_artifact, object_address_matches_id, prepare_artifact_publication,
    stored_artifact_is_exact, ObjectAddress, PreparedArtifactPublication, StoredArtifactSnapshot,
};
use crucible_core::{ArtifactId, ArtifactRef};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_publication_plan_has_the_pure_address(contents: &[u8]) -> (result: Result<
    PreparedArtifactPublication,
    crucible_cli::ArtifactStoreError,
>)
    ensures
        match &result {
            Ok(publication) => crucible_cli::prepared_artifact_publication_spec(
                contents@,
                publication@,
            ),
            Err(
                crucible_cli::ArtifactStoreError::InputTooLong,
            ) => !crucible_core::artifact::sha256_input_supported(contents@.len() as nat),
            Err(_) => false,
        },
{
    prepare_artifact_publication(contents)
}

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_address_has_only_canonical_components(id: &ArtifactId) -> (result: Result<
    ObjectAddress,
    crucible_cli::ArtifactStoreError,
>)
    ensures
        match &result {
            Ok(address) => crucible_cli::object_address_spec(id@, address@),
            Err(_) => !crucible_core::artifact::canonical_sha256_artifact_id_spec(id@),
        },
{
    object_address_for_artifact(id)
}

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_address_matcher_has_exact_pure_meaning(
    id: &ArtifactId,
    address: &ObjectAddress,
) -> (matches: bool)
    ensures
        matches == crucible_cli::object_address_spec(id@, address@),
{
    object_address_matches_id(id, address)
}

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_stored_snapshot_check_has_exact_pure_meaning(
    expected: &ArtifactRef,
    require_import: bool,
    snapshot: &StoredArtifactSnapshot,
) -> (exact: bool)
    ensures
        exact == crucible_cli::stored_artifact_is_exact_spec(expected@, require_import, snapshot@),
{
    stored_artifact_is_exact(expected, require_import, snapshot)
}

proof fn a_wrong_database_digest_cannot_be_laundered_as_an_exact_snapshot(
    expected: crucible_core::artifact::ArtifactRefView,
    require_import: bool,
    snapshot: crucible_cli::StoredArtifactSnapshotView,
)
    requires
        snapshot.record_digest != Some(expected.id.skip(7)),
    ensures
        !crucible_cli::stored_artifact_is_exact_spec(expected, require_import, snapshot),
{
}

proof fn a_canonical_address_cannot_contain_path_separators(
    id: Seq<char>,
    address: crucible_cli::ObjectAddressView,
)
    requires
        crucible_cli::object_address_spec(id, address),
    ensures
        forall|index: int|
            0 <= index < address.object_name.len() ==> address.object_name[index] != '/'
                && address.object_name[index] != '\\',
{
    crucible_cli::lemma_object_address_has_no_path_separators(id, address);
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(crucible_cli::MAX_LOCAL_ARTIFACT_BYTES, 64 * 1024 * 1024);
}
