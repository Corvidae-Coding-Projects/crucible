//! Verified content-addressed object-store policy for the local CLI.
use crucible_core::artifact::ContentDigest;
use crucible_core::{parse_artifact_id, ArtifactId, ArtifactIdParseError, ArtifactRef, HashError};
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

verus! {

pub const MAX_LOCAL_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactStoreError {
    InputTooLong,
    MalformedArtifactId,
    UnsupportedAlgorithm,
    IntegrityMismatch,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectAddress {
    pub algorithm: String,
    pub first: String,
    pub second: String,
    pub object_name: String,
}

#[verifier::ext_equal]
pub struct ObjectAddressView {
    pub algorithm: Seq<char>,
    pub first: Seq<char>,
    pub second: Seq<char>,
    pub object_name: Seq<char>,
}

impl View for ObjectAddress {
    type V = ObjectAddressView;

    open spec fn view(&self) -> ObjectAddressView {
        ObjectAddressView {
            algorithm: self.algorithm@,
            first: self.first@,
            second: self.second@,
            object_name: self.object_name@,
        }
    }
}

pub open spec fn sha256_label_spec() -> Seq<char> {
    seq!['s', 'h', 'a', '2', '5', '6']
}

pub open spec fn object_address_spec(id: Seq<char>, address: ObjectAddressView) -> bool {
    crucible_core::artifact::canonical_sha256_artifact_id_spec(id) && address.algorithm
        == sha256_label_spec() && address.object_name == id.skip(7) && address.first
        == address.object_name.subrange(0, 2) && address.second == address.object_name.subrange(
        2,
        4,
    )
}

fn sha256_label() -> (label: String)
    ensures
        label@ == sha256_label_spec(),
{
    let mut label = String::new();
    label.push('s');
    label.push('h');
    label.push('a');
    label.push('2');
    label.push('5');
    label.push('6');
    label
}

pub fn object_address_for_artifact(id: &ArtifactId) -> (result: Result<
    ObjectAddress,
    ArtifactStoreError,
>)
    ensures
        match &result {
            Ok(address) => object_address_spec(id@, address@),
            Err(
                ArtifactStoreError::MalformedArtifactId,
            ) => !crucible_core::artifact::canonical_sha256_artifact_id_spec(id@),
            Err(
                ArtifactStoreError::UnsupportedAlgorithm,
            ) => !crucible_core::artifact::canonical_sha256_artifact_id_spec(id@),
            Err(ArtifactStoreError::InputTooLong)
            | Err(ArtifactStoreError::IntegrityMismatch) => false,
        },
{
    let digest = match parse_artifact_id(id) {
        Ok(ContentDigest::Sha256(digest)) => digest,
        Err(ArtifactIdParseError::MalformedArtifactId) => {
            return Err(ArtifactStoreError::MalformedArtifactId);
        },
        Err(ArtifactIdParseError::UnsupportedAlgorithm) => {
            return Err(ArtifactStoreError::UnsupportedAlgorithm);
        },
        Ok(_) | Err(_) => {
            return Err(ArtifactStoreError::UnsupportedAlgorithm);
        },
    };
    let object_name = digest.to_hex();
    proof {
        crucible_core::artifact::lemma_hex_encode_is_canonical(digest@);
    }
    assert(object_name@.len() == 64);
    let mut first = String::new();
    first.push(object_name.get_char(0));
    first.push(object_name.get_char(1));
    let mut second = String::new();
    second.push(object_name.get_char(2));
    second.push(object_name.get_char(3));
    let address = ObjectAddress { algorithm: sha256_label(), first, second, object_name };
    assert(crucible_core::artifact::canonical_sha256_artifact_id_spec(id@));
    assert(address.algorithm@ == sha256_label_spec());
    assert(address.object_name@ == id@.skip(7));
    assert(address.first@ == address.object_name@.subrange(0, 2));
    assert(address.second@ == address.object_name@.subrange(2, 4));
    assert(object_address_spec(id@, address@));
    Ok(address)
}

#[derive(Debug, PartialEq, Eq)]
pub struct PreparedArtifactPublication {
    pub artifact: ArtifactRef,
    pub address: ObjectAddress,
}

#[verifier::ext_equal]
pub struct PreparedArtifactPublicationView {
    pub artifact: crucible_core::artifact::ArtifactRefView,
    pub address: ObjectAddressView,
}

impl View for PreparedArtifactPublication {
    type V = PreparedArtifactPublicationView;

    open spec fn view(&self) -> PreparedArtifactPublicationView {
        PreparedArtifactPublicationView { artifact: self.artifact@, address: self.address@ }
    }
}

pub open spec fn prepared_artifact_publication_spec(
    contents: Seq<u8>,
    publication: PreparedArtifactPublicationView,
) -> bool {
    crucible_core::artifact::sha256_input_supported(contents.len() as nat)
        && publication.artifact.id == crucible_core::artifact::artifact_id_spec(
        crucible_core::artifact::sha256_spec(contents),
    ) && publication.artifact.size_bytes as nat == contents.len()
        && publication.artifact.media_type is None && object_address_spec(
        publication.artifact.id,
        publication.address,
    )
}

pub fn prepare_artifact_publication(contents: &[u8]) -> (result: Result<
    PreparedArtifactPublication,
    ArtifactStoreError,
>)
    ensures
        match &result {
            Ok(publication) => prepared_artifact_publication_spec(contents@, publication@),
            Err(
                ArtifactStoreError::InputTooLong,
            ) => !crucible_core::artifact::sha256_input_supported(contents@.len() as nat),
            Err(_) => false,
        },
{
    let digest = match ContentDigest::from_bytes(contents) {
        Ok(digest) => digest,
        Err(HashError::InputTooLong) => {
            return Err(ArtifactStoreError::InputTooLong);
        },
    };
    let artifact = ArtifactRef {
        id: digest.into_artifact_id(),
        size_bytes: contents.len() as u64,
        media_type: None,
    };
    proof {
        crucible_core::artifact::lemma_artifact_id_spec_is_canonical(
            crucible_core::artifact::sha256_spec(contents@),
        );
    }
    let address = match object_address_for_artifact(&artifact.id) {
        Ok(address) => address,
        Err(ArtifactStoreError::MalformedArtifactId)
        | Err(ArtifactStoreError::UnsupportedAlgorithm)
        | Err(ArtifactStoreError::InputTooLong)
        | Err(ArtifactStoreError::IntegrityMismatch) => {
            assert(false);
            return Err(ArtifactStoreError::IntegrityMismatch);
        },
    };
    let publication = PreparedArtifactPublication { artifact, address };
    assert(prepared_artifact_publication_spec(contents@, publication@));
    Ok(publication)
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredArtifactSnapshot {
    pub object_is_file: bool,
    pub record: Option<ArtifactRef>,
    pub record_algorithm: Option<String>,
    pub record_digest: Option<String>,
    pub contents: Vec<u8>,
    pub matching_import_count: u64,
}

#[verifier::ext_equal]
pub struct StoredArtifactSnapshotView {
    pub object_is_file: bool,
    pub record: Option<crucible_core::artifact::ArtifactRefView>,
    pub record_algorithm: Option<Seq<char>>,
    pub record_digest: Option<Seq<char>>,
    pub contents: Seq<u8>,
    pub matching_import_count: u64,
}

impl View for StoredArtifactSnapshot {
    type V = StoredArtifactSnapshotView;

    open spec fn view(&self) -> StoredArtifactSnapshotView {
        StoredArtifactSnapshotView {
            object_is_file: self.object_is_file,
            record: match &self.record {
                Some(record) => Some(record@),
                None => None,
            },
            record_algorithm: match &self.record_algorithm {
                Some(algorithm) => Some(algorithm@),
                None => None,
            },
            record_digest: match &self.record_digest {
                Some(digest) => Some(digest@),
                None => None,
            },
            contents: self.contents@,
            matching_import_count: self.matching_import_count,
        }
    }
}

pub open spec fn stored_artifact_is_exact_spec(
    expected: crucible_core::artifact::ArtifactRefView,
    require_import: bool,
    snapshot: StoredArtifactSnapshotView,
) -> bool {
    snapshot.object_is_file && match snapshot.record {
        Some(record) => artifact_ref_equal_spec(record, expected),
        None => false,
    } && snapshot.record_algorithm == Some(sha256_label_spec()) && snapshot.record_digest == Some(
        expected.id.skip(7),
    ) && expected.size_bytes as nat == snapshot.contents.len()
        && crucible_core::artifact::sha256_input_supported(snapshot.contents.len() as nat)
        && expected.id == crucible_core::artifact::artifact_id_spec(
        crucible_core::artifact::sha256_spec(snapshot.contents),
    ) && if require_import {
        snapshot.matching_import_count == 1
    } else {
        true
    }
}

pub open spec fn artifact_ref_equal_spec(
    left: crucible_core::artifact::ArtifactRefView,
    right: crucible_core::artifact::ArtifactRefView,
) -> bool {
    left.id == right.id && left.size_bytes == right.size_bytes && left.media_type
        == right.media_type
}

#[expect(clippy::ptr_arg, reason = "owned-string views preserve the exact vstd equality proof surface")]
fn same_string(left: &String, right: &String) -> (same: bool)
    ensures
        same == (left@ == right@),
{
    left.clone() == right.clone()
}

fn same_optional_string(left: &Option<String>, right: &Option<String>) -> (same: bool)
    ensures
        same == match (left, right) {
            (Some(left), Some(right)) => left@ == right@,
            (None, None) => true,
            _ => false,
        },
{
    match (left, right) {
        (Some(left), Some(right)) => same_string(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_artifact_ref(left: &ArtifactRef, right: &ArtifactRef) -> (same: bool)
    ensures
        same == artifact_ref_equal_spec(left@, right@),
{
    same_string(&left.id.0, &right.id.0) && left.size_bytes == right.size_bytes
        && same_optional_string(&left.media_type, &right.media_type)
}

fn same_object_address(left: &ObjectAddress, right: &ObjectAddress) -> (same: bool)
    ensures
        same == (left@ == right@),
{
    let same = same_string(&left.algorithm, &right.algorithm) && same_string(
        &left.first,
        &right.first,
    ) && same_string(&left.second, &right.second) && same_string(
        &left.object_name,
        &right.object_name,
    );
    if same {
        assert(left@ =~= right@);
    }
    same
}

pub fn object_address_matches_id(id: &ArtifactId, address: &ObjectAddress) -> (matches: bool)
    ensures
        matches == object_address_spec(id@, address@),
{
    match object_address_for_artifact(id) {
        Ok(expected) => {
            let matches = same_object_address(&expected, address);
            if matches {
                assert(expected@ == address@);
                assert(object_address_spec(id@, address@));
            }
            matches
        },
        Err(_) => false,
    }
}

pub fn stored_artifact_is_exact(
    expected: &ArtifactRef,
    require_import: bool,
    snapshot: &StoredArtifactSnapshot,
) -> (exact: bool)
    ensures
        exact == stored_artifact_is_exact_spec(expected@, require_import, snapshot@),
{
    if !snapshot.object_is_file {
        return false;
    }
    let record = match &snapshot.record {
        Some(record) => record,
        None => return false,
    };
    if !same_artifact_ref(record, expected) {
        return false;
    }
    let algorithm = match &snapshot.record_algorithm {
        Some(algorithm) => algorithm,
        None => return false,
    };
    let stored_digest = match &snapshot.record_digest {
        Some(digest) => digest,
        None => return false,
    };
    if require_import && snapshot.matching_import_count != 1 {
        return false;
    }
    if expected.size_bytes as u128 != snapshot.contents.len() as u128 {
        return false;
    }
    assert(expected.size_bytes as int == snapshot.contents@.len());
    let digest = match ContentDigest::from_bytes(&snapshot.contents) {
        Ok(digest) => digest,
        Err(HashError::InputTooLong) => return false,
    };
    let actual_id = digest.into_artifact_id();
    if !same_string(&expected.id.0, &actual_id.0) {
        return false;
    }
    proof {
        crucible_core::artifact::lemma_artifact_id_spec_is_canonical(
            crucible_core::artifact::sha256_spec(snapshot.contents@),
        );
    }
    assert(crucible_core::artifact::canonical_sha256_artifact_id_spec(expected.id@));
    let address = match object_address_for_artifact(&expected.id) {
        Ok(address) => address,
        Err(_) => {
            assert(false);
            return false;
        },
    };
    if !same_string(algorithm, &address.algorithm) || !same_string(
        stored_digest,
        &address.object_name,
    ) {
        return false;
    }
    assert(address.algorithm@ == sha256_label_spec());
    assert(address.object_name@ == expected.id@.skip(7));
    true
}

pub proof fn lemma_object_address_has_no_path_separators(id: Seq<char>, address: ObjectAddressView)
    requires
        object_address_spec(id, address),
    ensures
        forall|index: int|
            0 <= index < address.object_name.len() ==> address.object_name[index] != '/'
                && address.object_name[index] != '\\',
{
    assert forall|index: int|
        0 <= index < address.object_name.len() implies address.object_name[index] != '/'
        && address.object_name[index] != '\\' by {
        let character = address.object_name[index];
        assert(crucible_core::artifact::lowercase_hex_value_spec(character) is Some);
        match character {
            '0'
            | '1'
            | '2'
            | '3'
            | '4'
            | '5'
            | '6'
            | '7'
            | '8'
            | '9'
            | 'a'
            | 'b'
            | 'c'
            | 'd'
            | 'e'
            | 'f' => {},
            _ => {},
        }
    };
}

} // verus!
