//! Verified content digests and immutable artifact identity.
//!
//! The SHA-256 implementation follows NIST FIPS 180-4 and is checked against checksum-pinned NIST
//! CAVP vectors. Its executable padding, schedule, rounds, block fold, output, canonical codec, and
//! algorithm-labeled artifact-ID parser are related to pure Verus specs.
use crate::ArtifactId;
#[allow(unused_imports)]
use vstd::assert_seqs_equal;
use vstd::prelude::*;
use vstd::string::{StrSliceExecFns, StringExecFns};

verus! {

pub const SHA256_DIGEST_BYTES: usize = 32;

pub const SHA256_HEX_CHARACTERS: usize = 64;

pub const MAX_SHA256_INPUT_BYTES: u64 = u64::MAX / 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashError {
    InputTooLong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigestDecodeError {
    WrongLength,
    NonCanonicalHex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactIdentityError {
    InputTooLong,
    SizeMismatch,
    MalformedArtifactId,
    UnsupportedAlgorithm,
    DigestMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArtifactIdParseError {
    MalformedArtifactId,
    UnsupportedAlgorithm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sha256Digest {
    pub bytes: [u8; SHA256_DIGEST_BYTES],
}

impl View for Sha256Digest {
    type V = Seq<u8>;

    open spec fn view(&self) -> Seq<u8> {
        self.bytes@
    }
}

pub open spec fn hex_character_spec(value: u8) -> char
    recommends
        value < 16,
{
    match value {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        _ => 'f',
    }
}

pub open spec fn lowercase_hex_value_spec(character: char) -> Option<u8> {
    match character {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' => Some(10),
        'b' => Some(11),
        'c' => Some(12),
        'd' => Some(13),
        'e' => Some(14),
        'f' => Some(15),
        _ => None,
    }
}

pub open spec fn canonical_sha256_hex_spec(encoded: Seq<char>) -> bool {
    encoded.len() == SHA256_HEX_CHARACTERS && forall|index: int|
        0 <= index < encoded.len() ==> lowercase_hex_value_spec(#[trigger] encoded[index]) is Some
}

pub open spec fn hex_prefix_spec(bytes: Seq<u8>, count: nat) -> Seq<char>
    recommends
        count <= bytes.len(),
    decreases count,
{
    if count == 0 {
        Seq::empty()
    } else {
        let prior = hex_prefix_spec(bytes, (count - 1) as nat);
        let byte = bytes[(count - 1) as int];
        prior.push(hex_character_spec(byte >> 4)).push(hex_character_spec(byte & 0x0f))
    }
}

pub open spec fn hex_encode_spec(bytes: Seq<u8>) -> Seq<char> {
    hex_prefix_spec(bytes, bytes.len() as nat)
}

proof fn lemma_hex_prefix_update_after(bytes: Seq<u8>, changed: int, value: u8, count: nat)
    requires
        count <= changed,
        changed < bytes.len(),
        count <= bytes.len(),
    ensures
        hex_prefix_spec(bytes.update(changed, value), count) == hex_prefix_spec(bytes, count),
    decreases count,
{
    if count > 0 {
        lemma_hex_prefix_update_after(bytes, changed, value, (count - 1) as nat);
        reveal_with_fuel(hex_prefix_spec, 2);
    }
}

proof fn lemma_lowercase_hex_inverse(character: char, value: u8)
    requires
        lowercase_hex_value_spec(character) == Some(value),
    ensures
        value < 16,
        hex_character_spec(value) == character,
{
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
}

proof fn lemma_hex_character_is_lowercase(value: u8)
    requires
        value < 16,
    ensures
        lowercase_hex_value_spec(hex_character_spec(value)) == Some(value),
{
    match value {
        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => {},
        _ => {},
    }
}

proof fn lemma_hex_prefix_is_lowercase(bytes: Seq<u8>, count: nat)
    requires
        count <= bytes.len(),
    ensures
        hex_prefix_spec(bytes, count).len() == count * 2,
        forall|index: int|
            0 <= index < hex_prefix_spec(bytes, count).len() ==> lowercase_hex_value_spec(
                #[trigger] hex_prefix_spec(bytes, count)[index],
            ) is Some,
    decreases count,
{
    if count > 0 {
        lemma_hex_prefix_is_lowercase(bytes, (count - 1) as nat);
        let byte = bytes[(count - 1) as int];
        assert(byte >> 4 < 16) by (bit_vector);
        assert(byte & 0x0f < 16) by (bit_vector);
        lemma_hex_character_is_lowercase(byte >> 4);
        lemma_hex_character_is_lowercase(byte & 0x0f);
        reveal_with_fuel(hex_prefix_spec, 2);
    }
}

pub proof fn lemma_hex_encode_is_canonical(bytes: Seq<u8>)
    requires
        bytes.len() == SHA256_DIGEST_BYTES,
    ensures
        canonical_sha256_hex_spec(hex_encode_spec(bytes)),
{
    lemma_hex_prefix_is_lowercase(bytes, bytes.len() as nat);
}

fn hex_character(value: u8) -> (character: char)
    requires
        value < 16,
    ensures
        character == hex_character_spec(value),
{
    match value {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        _ => 'f',
    }
}

fn lowercase_hex_value(character: char) -> (value: Option<u8>)
    ensures
        value == lowercase_hex_value_spec(character),
{
    match character {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' => Some(10),
        'b' => Some(11),
        'c' => Some(12),
        'd' => Some(13),
        'e' => Some(14),
        'f' => Some(15),
        _ => None,
    }
}

impl Sha256Digest {
    pub fn as_bytes(&self) -> (bytes: &[u8; SHA256_DIGEST_BYTES])
        ensures
            bytes@ == self@,
    {
        &self.bytes
    }

    pub fn to_hex(&self) -> (encoded: String)
        ensures
            encoded@ == hex_encode_spec(self@),
    {
        let mut encoded = String::new();
        let mut index = 0;
        while index < SHA256_DIGEST_BYTES
            invariant
                index <= SHA256_DIGEST_BYTES,
                self@.len() == SHA256_DIGEST_BYTES,
                encoded@ == hex_prefix_spec(self@, index as nat),
            decreases SHA256_DIGEST_BYTES - index,
        {
            let byte = self.bytes[index];
            assert(byte >> 4 < 16) by (bit_vector);
            assert(byte & 0x0f < 16) by (bit_vector);
            encoded.push(hex_character(byte >> 4));
            encoded.push(hex_character(byte & 0x0f));
            index += 1;
            reveal_with_fuel(hex_prefix_spec, 2);
        }
        encoded
    }

    pub fn from_hex(encoded: String) -> (result: Result<Self, DigestDecodeError>)
        ensures
            match result {
                Ok(digest) => canonical_sha256_hex_spec(encoded@) && digest@.len()
                    == SHA256_DIGEST_BYTES && hex_encode_spec(digest@) == encoded@,
                Err(DigestDecodeError::WrongLength) => encoded@.len() != SHA256_HEX_CHARACTERS,
                Err(DigestDecodeError::NonCanonicalHex) => encoded@.len() == SHA256_HEX_CHARACTERS
                    && !canonical_sha256_hex_spec(encoded@),
            },
    {
        let text = encoded.as_str();
        let length = text.unicode_len();
        if length != SHA256_HEX_CHARACTERS {
            return Err(DigestDecodeError::WrongLength);
        }
        let mut bytes = [0u8;SHA256_DIGEST_BYTES];
        let mut index = 0;
        while index < SHA256_DIGEST_BYTES
            invariant
                index <= SHA256_DIGEST_BYTES,
                text@.len() == SHA256_HEX_CHARACTERS,
                text@ == encoded@,
                hex_prefix_spec(bytes@, index as nat) == text@.take((index * 2) as int),
                forall|prior: int|
                    0 <= prior < index * 2 ==> lowercase_hex_value_spec(
                        #[trigger] text@[prior],
                    ) is Some,
            decreases SHA256_DIGEST_BYTES - index,
        {
            let ghost prior_bytes = bytes@;
            assert(index < SHA256_DIGEST_BYTES);
            assert(index * 2 + 1 < SHA256_HEX_CHARACTERS);
            let high = match lowercase_hex_value(text.get_char(index * 2)) {
                Some(value) => value,
                None => {
                    assert(!canonical_sha256_hex_spec(encoded@));
                    return Err(DigestDecodeError::NonCanonicalHex);
                },
            };
            let low = match lowercase_hex_value(text.get_char(index * 2 + 1)) {
                Some(value) => value,
                None => {
                    assert(!canonical_sha256_hex_spec(encoded@));
                    return Err(DigestDecodeError::NonCanonicalHex);
                },
            };
            let assembled = (high << 4) | low;
            bytes[index] = assembled;
            proof {
                lemma_lowercase_hex_inverse(text@[index * 2], high);
                lemma_lowercase_hex_inverse(text@[index * 2 + 1], low);
                lemma_hex_prefix_update_after(prior_bytes, index as int, assembled, index as nat);
                text@.lemma_take_succ_push((index * 2) as int);
                text@.lemma_take_succ_push((index * 2 + 1) as int);
            }
            assert((((high << 4) | low) >> 4) == high) by (bit_vector)
                requires
                    high < 16,
                    low < 16,
            ;
            assert((((high << 4) | low) & 0x0f) == low) by (bit_vector)
                requires
                    high < 16,
                    low < 16,
            ;
            index += 1;
            reveal_with_fuel(hex_prefix_spec, 2);
        }
        assert(canonical_sha256_hex_spec(encoded@));
        Ok(Self { bytes })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DigestAlgorithm {
    Sha256,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentDigest {
    Sha256(Sha256Digest),
}

pub enum ContentDigestView {
    Sha256(Seq<u8>),
}

impl View for ContentDigest {
    type V = ContentDigestView;

    open spec fn view(&self) -> ContentDigestView {
        match self {
            ContentDigest::Sha256(digest) => ContentDigestView::Sha256(digest@),
        }
    }
}

impl ContentDigest {
    pub fn from_bytes(contents: &[u8]) -> (result: Result<Self, HashError>)
        ensures
            match result {
                Ok(digest) => sha256_input_supported(contents@.len() as nat) && digest@
                    == ContentDigestView::Sha256(sha256_spec(contents@)),
                Err(HashError::InputTooLong) => !sha256_input_supported(contents@.len() as nat),
            },
    {
        match sha256(contents) {
            Ok(digest) => Ok(ContentDigest::Sha256(digest)),
            Err(error) => Err(error),
        }
    }

    pub fn algorithm(&self) -> (algorithm: DigestAlgorithm)
        ensures
            algorithm == DigestAlgorithm::Sha256,
    {
        match self {
            ContentDigest::Sha256(_) => DigestAlgorithm::Sha256,
        }
    }

    pub fn into_artifact_id(self) -> (id: ArtifactId)
        ensures
            match self@ {
                ContentDigestView::Sha256(digest) => id@ == artifact_id_spec(digest),
            },
    {
        match self {
            ContentDigest::Sha256(digest) => artifact_id_for_digest(&digest),
        }
    }
}

pub open spec fn lowercase_ascii_letter_spec(character: char) -> bool {
    'a' <= character <= 'z'
}

pub open spec fn algorithm_label_character_spec(character: char) -> bool {
    lowercase_ascii_letter_spec(character) || ('0' <= character <= '9') || character == '-'
}

pub open spec fn sha256_algorithm_label_spec(text: Seq<char>) -> bool {
    text.len() >= 7 && text[0] == 's' && text[1] == 'h' && text[2] == 'a' && text[3] == '2'
        && text[4] == '5' && text[5] == '6' && text[6] == ':'
}

pub open spec fn canonical_algorithm_label_at_spec(text: Seq<char>, colon: int) -> bool {
    0 < colon < text.len() && lowercase_ascii_letter_spec(text[0]) && text[colon] == ':' && forall|
        index: int,
    |
        1 <= index < colon ==> algorithm_label_character_spec(#[trigger] text[index])
}

pub open spec fn canonical_algorithm_label_spec(text: Seq<char>) -> bool {
    exists|colon: int| canonical_algorithm_label_at_spec(text, colon)
}

pub open spec fn canonical_sha256_artifact_id_spec(text: Seq<char>) -> bool {
    sha256_algorithm_label_spec(text) && canonical_sha256_hex_spec(text.skip(7))
}

pub open spec fn unsupported_artifact_algorithm_spec(text: Seq<char>) -> bool {
    canonical_algorithm_label_spec(text) && !sha256_algorithm_label_spec(text)
}

pub open spec fn malformed_artifact_id_spec(text: Seq<char>) -> bool {
    !canonical_sha256_artifact_id_spec(text) && !unsupported_artifact_algorithm_spec(text)
}

// Direct comparisons keep the parser inside Verus's verified primitive surface;
// `RangeInclusive::contains` would add iterator/library machinery to this check.
#[allow(clippy::manual_range_contains)]
fn is_lowercase_ascii_letter(character: char) -> (is_letter: bool)
    ensures
        is_letter == lowercase_ascii_letter_spec(character),
{
    character >= 'a' && character <= 'z'
}

#[allow(clippy::manual_range_contains)]
fn is_algorithm_label_character(character: char) -> (is_character: bool)
    ensures
        is_character == algorithm_label_character_spec(character),
{
    is_lowercase_ascii_letter(character) || (character >= '0' && character <= '9') || character
        == '-'
}

fn has_sha256_algorithm_label(text: &str, length: usize) -> (has_label: bool)
    requires
        length as nat == text@.len(),
    ensures
        has_label == sha256_algorithm_label_spec(text@),
{
    length >= 7 && text.get_char(0) == 's' && text.get_char(1) == 'h' && text.get_char(2) == 'a'
        && text.get_char(3) == '2' && text.get_char(4) == '5' && text.get_char(5) == '6'
        && text.get_char(6) == ':'
}

fn has_canonical_algorithm_label(text: &str, length: usize) -> (has_label: bool)
    requires
        length as nat == text@.len(),
    ensures
        has_label == canonical_algorithm_label_spec(text@),
{
    if length == 0 {
        assert(!canonical_algorithm_label_spec(text@));
        return false;
    }
    if !is_lowercase_ascii_letter(text.get_char(0)) {
        assert(!canonical_algorithm_label_spec(text@));
        return false;
    }
    let mut index = 1;
    while index < length
        invariant
            1 <= index <= length,
            length as nat == text@.len(),
            lowercase_ascii_letter_spec(text@[0]),
            forall|prior: int|
                1 <= prior < index ==> algorithm_label_character_spec(#[trigger] text@[prior])
                    && text@[prior] != ':',
        decreases length - index,
    {
        let character = text.get_char(index);
        if character == ':' {
            assert(canonical_algorithm_label_at_spec(text@, index as int));
            assert(canonical_algorithm_label_spec(text@));
            return true;
        }
        if !is_algorithm_label_character(character) {
            assert(!canonical_algorithm_label_spec(text@)) by {
                if canonical_algorithm_label_spec(text@) {
                    let colon = choose|colon: int| canonical_algorithm_label_at_spec(text@, colon);
                    if colon < index {
                        assert(text@[colon] != ':');
                    } else if colon == index {
                        assert(text@[colon] != ':');
                    } else {
                        assert(algorithm_label_character_spec(text@[index as int]));
                    }
                }
            };
            return false;
        }
        index += 1;
    }
    assert(!canonical_algorithm_label_spec(text@)) by {
        if canonical_algorithm_label_spec(text@) {
            let colon = choose|colon: int| canonical_algorithm_label_at_spec(text@, colon);
            assert(text@[colon] != ':');
        }
    };
    false
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArtifactRef {
    pub id: ArtifactId,
    pub size_bytes: u64,
    pub media_type: Option<String>,
}

fn artifact_id_for_digest(digest: &Sha256Digest) -> (id: ArtifactId)
    ensures
        id@ == artifact_id_spec(digest@),
{
    let hexadecimal = digest.to_hex();
    let mut canonical = String::new();
    canonical.push('s');
    canonical.push('h');
    canonical.push('a');
    canonical.push('2');
    canonical.push('5');
    canonical.push('6');
    canonical.push(':');
    canonical.append(hexadecimal.as_str());
    ArtifactId::new(canonical)
}

pub fn parse_artifact_id(id: &ArtifactId) -> (result: Result<ContentDigest, ArtifactIdParseError>)
    ensures
        match result {
            Ok(ContentDigest::Sha256(digest)) => id@ == artifact_id_spec(digest@),
            Err(ArtifactIdParseError::UnsupportedAlgorithm) => unsupported_artifact_algorithm_spec(
                id@,
            ),
            Err(ArtifactIdParseError::MalformedArtifactId) => malformed_artifact_id_spec(id@),
        },
{
    let text = id.as_str();
    let length = text.unicode_len();
    let is_sha256 = has_sha256_algorithm_label(text, length);
    if is_sha256 {
        let mut encoded_digest = String::new();
        let mut index = 7;
        while index < length
            invariant
                7 <= index <= length,
                length as nat == text@.len(),
                text@ == id@,
                sha256_algorithm_label_spec(text@),
                encoded_digest@ == text@.subrange(7, index as int),
            decreases length - index,
        {
            encoded_digest.push(text.get_char(index));
            index += 1;
        }
        assert(encoded_digest@ == text@.skip(7));
        match Sha256Digest::from_hex(encoded_digest) {
            Ok(digest) => {
                assert(canonical_sha256_artifact_id_spec(id@));
                proof {
                    assert_seqs_equal!(id@ == artifact_id_spec(digest@));
                }
                Ok(ContentDigest::Sha256(digest))
            },
            Err(DigestDecodeError::WrongLength) => {
                assert(malformed_artifact_id_spec(id@));
                Err(ArtifactIdParseError::MalformedArtifactId)
            },
            Err(DigestDecodeError::NonCanonicalHex) => {
                assert(malformed_artifact_id_spec(id@));
                Err(ArtifactIdParseError::MalformedArtifactId)
            },
        }
    } else if has_canonical_algorithm_label(text, length) {
        assert(unsupported_artifact_algorithm_spec(id@));
        Err(ArtifactIdParseError::UnsupportedAlgorithm)
    } else {
        assert(malformed_artifact_id_spec(id@));
        Err(ArtifactIdParseError::MalformedArtifactId)
    }
}

impl ArtifactRef {
    pub fn from_bytes(contents: &[u8], media_type: Option<String>) -> (result: Result<
        Self,
        ArtifactIdentityError,
    >)
        ensures
            match result {
                Ok(artifact) => sha256_input_supported(contents@.len() as nat) && artifact.id@
                    == artifact_id_spec(sha256_spec(contents@)) && artifact.size_bytes as nat
                    == contents@.len() && artifact.media_type.deep_view() == media_type.deep_view(),
                Err(ArtifactIdentityError::InputTooLong) => !sha256_input_supported(
                    contents@.len() as nat,
                ),
                Err(ArtifactIdentityError::SizeMismatch)
                | Err(ArtifactIdentityError::MalformedArtifactId)
                | Err(ArtifactIdentityError::UnsupportedAlgorithm)
                | Err(ArtifactIdentityError::DigestMismatch) => false,
            },
    {
        let digest = match ContentDigest::from_bytes(contents) {
            Ok(digest) => digest,
            Err(HashError::InputTooLong) => return Err(ArtifactIdentityError::InputTooLong),
        };
        Ok(Self { id: digest.into_artifact_id(), size_bytes: contents.len() as u64, media_type })
    }

    pub fn verify(&self, contents: &[u8]) -> (result: Result<(), ArtifactIdentityError>)
        ensures
            result is Ok ==> self.size_bytes as nat == contents@.len() && sha256_input_supported(
                contents@.len() as nat,
            ) && self.id@ == artifact_id_spec(sha256_spec(contents@)),
            result == Err(ArtifactIdentityError::SizeMismatch) ==> self.size_bytes as nat
                != contents@.len(),
            result == Err(ArtifactIdentityError::MalformedArtifactId) ==> self.size_bytes as nat
                == contents@.len() && malformed_artifact_id_spec(self.id@),
            result == Err(ArtifactIdentityError::UnsupportedAlgorithm) ==> self.size_bytes as nat
                == contents@.len() && unsupported_artifact_algorithm_spec(self.id@),
            result == Err(ArtifactIdentityError::InputTooLong) ==> self.size_bytes as nat
                == contents@.len() && canonical_sha256_artifact_id_spec(self.id@)
                && !sha256_input_supported(contents@.len() as nat),
            result == Err(ArtifactIdentityError::DigestMismatch) ==> self.size_bytes as nat
                == contents@.len() && canonical_sha256_artifact_id_spec(self.id@)
                && sha256_input_supported(contents@.len() as nat) && self.id@ != artifact_id_spec(
                sha256_spec(contents@),
            ),
    {
        if self.size_bytes as u128 != contents.len() as u128 {
            return Err(ArtifactIdentityError::SizeMismatch);
        }
        assert(self.size_bytes as int == contents.len() as int);
        let stored_digest = match parse_artifact_id(&self.id) {
            Ok(digest) => digest,
            Err(ArtifactIdParseError::MalformedArtifactId) => {
                return Err(ArtifactIdentityError::MalformedArtifactId);
            },
            Err(ArtifactIdParseError::UnsupportedAlgorithm) => {
                return Err(ArtifactIdentityError::UnsupportedAlgorithm);
            },
        };
        match stored_digest {
            ContentDigest::Sha256(expected) => {
                proof {
                    lemma_artifact_id_spec_is_canonical(expected@);
                }
                let actual = match sha256(contents) {
                    Ok(digest) => digest,
                    Err(HashError::InputTooLong) => {
                        return Err(ArtifactIdentityError::InputTooLong);
                    },
                };
                let expected_hex = expected.to_hex();
                let actual_hex = actual.to_hex();
                proof {
                    lemma_artifact_id_spec_is_canonical(actual@);
                }
                if actual_hex != expected_hex {
                    assert(self.id@ != artifact_id_spec(actual@)) by {
                        if self.id@ == artifact_id_spec(actual@) {
                            assert(self.id@.skip(7) == artifact_id_spec(actual@).skip(7));
                            assert(hex_encode_spec(expected@) == hex_encode_spec(actual@));
                        }
                    };
                    return Err(ArtifactIdentityError::DigestMismatch);
                }
            },
        }
        Ok(())
    }
}

pub const INITIAL_STATE: [u32; 8] = [
    0x6a09e667,
    0xbb67ae85,
    0x3c6ef372,
    0xa54ff53a,
    0x510e527f,
    0x9b05688c,
    0x1f83d9ab,
    0x5be0cd19,
];

pub const ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98,
    0x71374491,
    0xb5c0fbcf,
    0xe9b5dba5,
    0x3956c25b,
    0x59f111f1,
    0x923f82a4,
    0xab1c5ed5,
    0xd807aa98,
    0x12835b01,
    0x243185be,
    0x550c7dc3,
    0x72be5d74,
    0x80deb1fe,
    0x9bdc06a7,
    0xc19bf174,
    0xe49b69c1,
    0xefbe4786,
    0x0fc19dc6,
    0x240ca1cc,
    0x2de92c6f,
    0x4a7484aa,
    0x5cb0a9dc,
    0x76f988da,
    0x983e5152,
    0xa831c66d,
    0xb00327c8,
    0xbf597fc7,
    0xc6e00bf3,
    0xd5a79147,
    0x06ca6351,
    0x14292967,
    0x27b70a85,
    0x2e1b2138,
    0x4d2c6dfc,
    0x53380d13,
    0x650a7354,
    0x766a0abb,
    0x81c2c92e,
    0x92722c85,
    0xa2bfe8a1,
    0xa81a664b,
    0xc24b8b70,
    0xc76c51a3,
    0xd192e819,
    0xd6990624,
    0xf40e3585,
    0x106aa070,
    0x19a4c116,
    0x1e376c08,
    0x2748774c,
    0x34b0bcb5,
    0x391c0cb3,
    0x4ed8aa4a,
    0x5b9cca4f,
    0x682e6ff3,
    0x748f82ee,
    0x78a5636f,
    0x84c87814,
    0x8cc70208,
    0x90befffa,
    0xa4506ceb,
    0xbef9a3f7,
    0xc67178f2,
];

pub open spec fn rotate_right_spec(value: u32, count: u32) -> u32
    recommends
        0 < count < 32,
{
    (value >> count) | (value << (32 - count))
}

pub open spec fn add2_spec(left: u32, right: u32) -> u32 {
    ((((left as nat) + (right as nat)) as u64) & 0xffff_ffffu64) as u32
}

pub open spec fn add4_spec(first: u32, second: u32, third: u32, fourth: u32) -> u32 {
    add2_spec(add2_spec(first, second), add2_spec(third, fourth))
}

pub open spec fn add5_spec(first: u32, second: u32, third: u32, fourth: u32, fifth: u32) -> u32 {
    add2_spec(add4_spec(first, second, third, fourth), fifth)
}

pub open spec fn choice_word_spec(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ ((!x) & z)
}

pub open spec fn majority_spec(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

pub open spec fn big_sigma_zero_spec(value: u32) -> u32 {
    rotate_right_spec(value, 2) ^ rotate_right_spec(value, 13) ^ rotate_right_spec(value, 22)
}

pub open spec fn big_sigma_one_spec(value: u32) -> u32 {
    rotate_right_spec(value, 6) ^ rotate_right_spec(value, 11) ^ rotate_right_spec(value, 25)
}

pub open spec fn small_sigma_zero_spec(value: u32) -> u32 {
    rotate_right_spec(value, 7) ^ rotate_right_spec(value, 18) ^ (value >> 3)
}

pub open spec fn small_sigma_one_spec(value: u32) -> u32 {
    rotate_right_spec(value, 17) ^ rotate_right_spec(value, 19) ^ (value >> 10)
}

pub open spec fn message_word_spec(block: Seq<u8>, word: nat) -> u32
    recommends
        block.len() == 64,
        word < 16,
{
    let offset = word * 4;
    ((block[offset as int] as u32) << 24) | ((block[(offset + 1) as int] as u32) << 16) | ((block[(
    offset + 2) as int] as u32) << 8) | (block[(offset + 3) as int] as u32)
}

pub open spec fn schedule_word_spec(block: Seq<u8>, index: nat) -> u32
    recommends
        block.len() == 64,
        index < 64,
    decreases index,
{
    if index < 16 {
        message_word_spec(block, index)
    } else {
        add4_spec(
            small_sigma_one_spec(schedule_word_spec(block, (index - 2) as nat)),
            schedule_word_spec(block, (index - 7) as nat),
            small_sigma_zero_spec(schedule_word_spec(block, (index - 15) as nat)),
            schedule_word_spec(block, (index - 16) as nat),
        )
    }
}

pub open spec fn message_schedule_spec(block: Seq<u8>) -> Seq<u32>
    recommends
        block.len() == 64,
{
    Seq::new(64, |index: int| schedule_word_spec(block, index as nat))
}

pub open spec fn round_spec(working: Seq<u32>, constant: u32, word: u32) -> Seq<u32>
    recommends
        working.len() == 8,
{
    let temporary_one = add5_spec(
        working[7],
        big_sigma_one_spec(working[4]),
        choice_word_spec(working[4], working[5], working[6]),
        constant,
        word,
    );
    let temporary_two = add2_spec(
        big_sigma_zero_spec(working[0]),
        majority_spec(working[0], working[1], working[2]),
    );
    seq![
        add2_spec(temporary_one, temporary_two),
        working[0],
        working[1],
        working[2],
        add2_spec(working[3], temporary_one),
        working[4],
        working[5],
        working[6],
    ]
}

pub open spec fn rounds_spec(initial: Seq<u32>, schedule: Seq<u32>, count: nat) -> Seq<u32>
    recommends
        initial.len() == 8,
        schedule.len() == 64,
        count <= 64,
    decreases count,
{
    if count == 0 {
        initial
    } else {
        let prior = rounds_spec(initial, schedule, (count - 1) as nat);
        round_spec(prior, ROUND_CONSTANTS@[(count - 1) as int], schedule[(count - 1) as int])
    }
}

pub open spec fn combined_state_spec(initial: Seq<u32>, working: Seq<u32>) -> Seq<u32>
    recommends
        initial.len() == 8,
        working.len() == 8,
{
    seq![
        add2_spec(initial[0], working[0]),
        add2_spec(initial[1], working[1]),
        add2_spec(initial[2], working[2]),
        add2_spec(initial[3], working[3]),
        add2_spec(initial[4], working[4]),
        add2_spec(initial[5], working[5]),
        add2_spec(initial[6], working[6]),
        add2_spec(initial[7], working[7]),
    ]
}

pub open spec fn compress_spec(state: Seq<u32>, block: Seq<u8>) -> Seq<u32>
    recommends
        state.len() == 8,
        block.len() == 64,
{
    let schedule = message_schedule_spec(block);
    let working = rounds_spec(state, schedule, 64);
    combined_state_spec(state, working)
}

pub open spec fn sha256_input_supported(input_length: nat) -> bool {
    input_length <= MAX_SHA256_INPUT_BYTES as nat
}

pub open spec fn padding_zero_bytes_spec(input_length: nat) -> nat {
    let remainder = input_length % 64;
    if remainder < 56 {
        (55 - remainder) as nat
    } else {
        (119 - remainder) as nat
    }
}

pub open spec fn padded_length_spec(input_length: nat) -> nat {
    input_length + 1 + padding_zero_bytes_spec(input_length) + 8
}

pub open spec fn block_count_spec(input_length: nat) -> nat {
    let quotient = input_length / 64;
    if input_length % 64 < 56 {
        quotient + 1
    } else {
        quotient + 2
    }
}

pub open spec fn padded_byte_spec(input: Seq<u8>, position: int) -> u8
    recommends
        sha256_input_supported(input.len() as nat),
        0 <= position < padded_length_spec(input.len() as nat),
{
    let total_length = padded_length_spec(input.len() as nat);
    let bit_length = (input.len() * 8) as u64;
    if position < input.len() {
        input[position]
    } else if position == input.len() {
        0x80
    } else if position < total_length - 8 {
        0
    } else {
        let length_byte = position - (total_length - 8);
        ((bit_length >> (((7 - length_byte) * 8) as u32)) & 0xff) as u8
    }
}

pub open spec fn padded_block_spec(input: Seq<u8>, block_index: nat) -> Seq<u8>
    recommends
        sha256_input_supported(input.len() as nat),
        block_index < block_count_spec(input.len() as nat),
{
    Seq::new(64, |offset: int| padded_byte_spec(input, block_index * 64 + offset))
}

pub open spec fn hash_blocks_spec(input: Seq<u8>, count: nat) -> Seq<u32>
    recommends
        sha256_input_supported(input.len() as nat),
        count <= block_count_spec(input.len() as nat),
    decreases count,
{
    if count == 0 {
        INITIAL_STATE@
    } else {
        compress_spec(
            hash_blocks_spec(input, (count - 1) as nat),
            padded_block_spec(input, (count - 1) as nat),
        )
    }
}

pub open spec fn low_byte_spec(value: u32) -> u8 {
    (value & 0xff) as u8
}

pub open spec fn state_digest_byte_spec(state: Seq<u32>, index: nat) -> u8
    recommends
        state.len() == 8,
        index < SHA256_DIGEST_BYTES,
{
    let word = state[(index / 4) as int];
    let position = index % 4;
    let shift: u32 = if position == 0 {
        24u32
    } else if position == 1 {
        16u32
    } else if position == 2 {
        8u32
    } else {
        0u32
    };
    low_byte_spec(word >> shift)
}

pub open spec fn state_digest_spec(state: Seq<u32>) -> Seq<u8>
    recommends
        state.len() == 8,
{
    Seq::new(SHA256_DIGEST_BYTES as nat, |index: int| state_digest_byte_spec(state, index as nat))
}

pub open spec fn sha256_spec(input: Seq<u8>) -> Seq<u8>
    recommends
        sha256_input_supported(input.len() as nat),
{
    state_digest_spec(hash_blocks_spec(input, block_count_spec(input.len() as nat)))
}

pub open spec fn artifact_id_spec(digest: Seq<u8>) -> Seq<char> {
    seq!['s', 'h', 'a', '2', '5', '6', ':'] + hex_encode_spec(digest)
}

proof fn lemma_artifact_id_spec_is_canonical(digest: Seq<u8>)
    requires
        digest.len() == SHA256_DIGEST_BYTES,
    ensures
        canonical_sha256_artifact_id_spec(artifact_id_spec(digest)),
        artifact_id_spec(digest).skip(7) == hex_encode_spec(digest),
{
    lemma_hex_encode_is_canonical(digest);
    assert(artifact_id_spec(digest).skip(7) == hex_encode_spec(digest));
}

// Keep the primitive shifts explicit so Verus proves this implementation against
// `rotate_right_spec` instead of delegating the cryptographic operation to a
// library intrinsic.
#[allow(clippy::manual_rotate)]
fn rotate_right(value: u32, count: u32) -> (rotated: u32)
    requires
        0 < count < 32,
    ensures
        rotated == rotate_right_spec(value, count),
{
    (value >> count) | (value << (32 - count))
}

fn add2(left: u32, right: u32) -> (sum: u32)
    ensures
        sum == add2_spec(left, right),
{
    let wide = (left as u64) + (right as u64);
    let reduced = wide & 0xffff_ffff;
    assert(reduced <= u32::MAX) by (bit_vector)
        requires
            reduced == wide & 0xffff_ffffu64,
    ;
    reduced as u32
}

fn add4(first: u32, second: u32, third: u32, fourth: u32) -> (sum: u32)
    ensures
        sum == add4_spec(first, second, third, fourth),
{
    add2(add2(first, second), add2(third, fourth))
}

fn add5(first: u32, second: u32, third: u32, fourth: u32, fifth: u32) -> (sum: u32)
    ensures
        sum == add5_spec(first, second, third, fourth, fifth),
{
    add2(add4(first, second, third, fourth), fifth)
}

fn choice_word(x: u32, y: u32, z: u32) -> (value: u32)
    ensures
        value == choice_word_spec(x, y, z),
{
    (x & y) ^ ((!x) & z)
}

fn majority(x: u32, y: u32, z: u32) -> (value: u32)
    ensures
        value == majority_spec(x, y, z),
{
    (x & y) ^ (x & z) ^ (y & z)
}

fn big_sigma_zero(value: u32) -> (result: u32)
    ensures
        result == big_sigma_zero_spec(value),
{
    rotate_right(value, 2) ^ rotate_right(value, 13) ^ rotate_right(value, 22)
}

fn big_sigma_one(value: u32) -> (result: u32)
    ensures
        result == big_sigma_one_spec(value),
{
    rotate_right(value, 6) ^ rotate_right(value, 11) ^ rotate_right(value, 25)
}

fn small_sigma_zero(value: u32) -> (result: u32)
    ensures
        result == small_sigma_zero_spec(value),
{
    rotate_right(value, 7) ^ rotate_right(value, 18) ^ (value >> 3)
}

fn small_sigma_one(value: u32) -> (result: u32)
    ensures
        result == small_sigma_one_spec(value),
{
    rotate_right(value, 17) ^ rotate_right(value, 19) ^ (value >> 10)
}

fn message_word(block: &[u8; 64], word: usize) -> (value: u32)
    requires
        word < 16,
    ensures
        value == message_word_spec(block@, word as nat),
{
    let offset = word * 4;
    ((block[offset] as u32) << 24) | ((block[offset + 1] as u32) << 16) | ((block[offset
        + 2] as u32) << 8) | (block[offset + 3] as u32)
}

fn compression_round(working: &[u32; 8], constant: u32, word: u32) -> (next: [u32; 8])
    ensures
        next@ == round_spec(working@, constant, word),
{
    let temporary_one = add5(
        working[7],
        big_sigma_one(working[4]),
        choice_word(working[4], working[5], working[6]),
        constant,
        word,
    );
    let temporary_two = add2(
        big_sigma_zero(working[0]),
        majority(working[0], working[1], working[2]),
    );
    [
        add2(temporary_one, temporary_two),
        working[0],
        working[1],
        working[2],
        add2(working[3], temporary_one),
        working[4],
        working[5],
        working[6],
    ]
}

fn combine_state(initial: &[u32; 8], working: &[u32; 8]) -> (combined: [u32; 8])
    ensures
        combined@ == combined_state_spec(initial@, working@),
{
    [
        add2(initial[0], working[0]),
        add2(initial[1], working[1]),
        add2(initial[2], working[2]),
        add2(initial[3], working[3]),
        add2(initial[4], working[4]),
        add2(initial[5], working[5]),
        add2(initial[6], working[6]),
        add2(initial[7], working[7]),
    ]
}

fn compress(state: &mut [u32; 8], block: &[u8; 64])
    ensures
        final(state)@ == compress_spec(old(state)@, block@),
{
    let ghost initial_state = state@;
    let mut schedule = [0u32;64];
    let mut index = 0;
    while index < 16
        invariant
            index <= 16,
            block@.len() == 64,
            forall|prior: int|
                0 <= prior < index ==> schedule@[prior] == schedule_word_spec(block@, prior as nat),
        decreases 16 - index,
    {
        schedule[index] = message_word(block, index);
        index += 1;
    }
    while index < 64
        invariant
            16 <= index <= 64,
            block@.len() == 64,
            forall|prior: int|
                0 <= prior < index ==> schedule@[prior] == schedule_word_spec(block@, prior as nat),
        decreases 64 - index,
    {
        assert(schedule@[index - 2] == schedule_word_spec(block@, (index - 2) as nat));
        assert(schedule@[index - 7] == schedule_word_spec(block@, (index - 7) as nat));
        assert(schedule@[index - 15] == schedule_word_spec(block@, (index - 15) as nat));
        assert(schedule@[index - 16] == schedule_word_spec(block@, (index - 16) as nat));
        schedule[index] = add4(
            small_sigma_one(schedule[index - 2]),
            schedule[index - 7],
            small_sigma_zero(schedule[index - 15]),
            schedule[index - 16],
        );
        reveal_with_fuel(schedule_word_spec, 2);
        index += 1;
    }

    assert(message_schedule_spec(block@).len() == 64);
    proof {
        assert_seqs_equal!(schedule@ == message_schedule_spec(block@));
    }

    let mut working = *state;
    assert(working@ == initial_state);
    index = 0;
    while index < 64
        invariant
            index <= 64,
            schedule@ == message_schedule_spec(block@),
            schedule@.len() == 64,
            initial_state.len() == 8,
            working@ == rounds_spec(initial_state, schedule@, index as nat),
        decreases 64 - index,
    {
        working = compression_round(&working, ROUND_CONSTANTS[index], schedule[index]);
        index += 1;
        reveal_with_fuel(rounds_spec, 2);
    }
    *state = combine_state(state, &working);
}

fn padded_byte(input: &[u8], position: u128, total_length: u128, bit_length: u64) -> (byte: u8)
    requires
        sha256_input_supported(input@.len() as nat),
        input.len() as u128 + 9 <= total_length,
        total_length as nat == padded_length_spec(input@.len() as nat),
        position < total_length,
        bit_length == (input.len() as u64) * 8,
    ensures
        byte == padded_byte_spec(input@, position as int),
{
    if position < input.len() as u128 {
        input[position as usize]
    } else if position == input.len() as u128 {
        0x80
    } else if position < total_length - 8 {
        0
    } else {
        let length_byte = position - (total_length - 8);
        ((bit_length >> ((7 - length_byte) * 8)) & 0xff) as u8
    }
}

fn low_byte(value: u32) -> (byte: u8)
    ensures
        byte == low_byte_spec(value),
{
    assert(value & 0xff <= u8::MAX) by (bit_vector);
    (value & 0xff) as u8
}

fn digest_from_state(state: &[u32; 8]) -> (bytes: [u8; SHA256_DIGEST_BYTES])
    ensures
        bytes@ == state_digest_spec(state@),
{
    let mut bytes = [0u8;SHA256_DIGEST_BYTES];
    let mut index = 0;
    while index < SHA256_DIGEST_BYTES
        invariant
            index <= SHA256_DIGEST_BYTES,
            state@.len() == 8,
            forall|prior: int|
                0 <= prior < index ==> bytes@[prior] == state_digest_byte_spec(
                    state@,
                    prior as nat,
                ),
        decreases SHA256_DIGEST_BYTES - index,
    {
        let word_index = index / 4;
        let position = index % 4;
        let shift = if position == 0 {
            24
        } else if position == 1 {
            16
        } else if position == 2 {
            8
        } else {
            0
        };
        bytes[index] = low_byte(state[word_index] >> shift);
        index += 1;
    }
    proof {
        assert_seqs_equal!(bytes@ == state_digest_spec(state@));
    }
    bytes
}

pub fn sha256(input: &[u8]) -> (result: Result<Sha256Digest, HashError>)
    ensures
        match result {
            Ok(digest) => sha256_input_supported(input@.len() as nat) && digest@ == sha256_spec(
                input@,
            ),
            Err(HashError::InputTooLong) => !sha256_input_supported(input@.len() as nat),
        },
{
    let input_length_wide = input.len() as u128;
    if input_length_wide > MAX_SHA256_INPUT_BYTES as u128 {
        return Err(HashError::InputTooLong);
    }
    assert(sha256_input_supported(input@.len() as nat));
    let remainder = input.len() % 64;
    let zero_bytes = if remainder < 56 {
        55 - remainder
    } else {
        119 - remainder
    };
    let total_length = input_length_wide + 1 + zero_bytes as u128 + 8;
    assert(zero_bytes as nat == padding_zero_bytes_spec(input@.len() as nat));
    assert(total_length as nat == padded_length_spec(input@.len() as nat));
    let bit_length = (input.len() as u64) * 8;
    proof {
        vstd::arithmetic::div_mod::lemma_fundamental_div_mod(input.len() as int, 64);
    }
    let quotient = input.len() / 64;
    assert(input.len() as int == 64 * ((input.len() as int) / 64) + ((input.len() as int) % 64));
    assert((quotient as int) == (input.len() as int) / 64);
    assert((remainder as int) == (input.len() as int) % 64);
    assert(input.len() as int == 64 * (quotient as int) + remainder as int);
    let block_count = if remainder < 56 {
        quotient + 1
    } else {
        quotient + 2
    };
    assert(block_count as nat == block_count_spec(input@.len() as nat));
    assert(total_length as int == (block_count as int) * 64) by (nonlinear_arith)
        requires
            input.len() as int == 64 * (quotient as int) + remainder as int,
            remainder < 64,
            remainder < 56 ==> zero_bytes == 55 - remainder,
            remainder >= 56 ==> zero_bytes == 119 - remainder,
            block_count == if remainder < 56 {
                quotient + 1
            } else {
                quotient + 2
            },
            total_length == input.len() as u128 + 1 + zero_bytes as u128 + 8,
    ;
    let mut state = INITIAL_STATE;
    reveal_with_fuel(hash_blocks_spec, 2);
    assert(state@ == hash_blocks_spec(input@, 0));
    let mut block_index = 0;
    while block_index < block_count
        invariant
            block_index <= block_count,
            total_length as int == (block_count as int) * 64,
            total_length as nat == padded_length_spec(input@.len() as nat),
            block_count as nat == block_count_spec(input@.len() as nat),
            sha256_input_supported(input@.len() as nat),
            input.len() as u128 + 9 <= total_length,
            bit_length == (input.len() as u64) * 8,
            state@ == hash_blocks_spec(input@, block_index as nat),
        decreases block_count - block_index,
    {
        let mut block = [0u8;64];
        let mut offset = 0;
        while offset < 64
            invariant
                offset <= 64,
                block_index < block_count,
                total_length as int == (block_count as int) * 64,
                total_length as nat == padded_length_spec(input@.len() as nat),
                block_count as nat == block_count_spec(input@.len() as nat),
                sha256_input_supported(input@.len() as nat),
                input.len() as u128 + 9 <= total_length,
                bit_length == (input.len() as u64) * 8,
                forall|prior: int|
                    0 <= prior < offset ==> #[trigger] block@[prior] == padded_block_spec(
                        input@,
                        block_index as nat,
                    )[prior],
            decreases 64 - offset,
        {
            let position = (block_index as u128) * 64 + offset as u128;
            block[offset] = padded_byte(input, position, total_length, bit_length);
            offset += 1;
        }
        proof {
            assert_seqs_equal!(block@ == padded_block_spec(input@, block_index as nat));
        }
        compress(&mut state, &block);
        block_index += 1;
        reveal_with_fuel(hash_blocks_spec, 2);
    }
    let bytes = digest_from_state(&state);
    Ok(Sha256Digest { bytes })
}

} // verus!
