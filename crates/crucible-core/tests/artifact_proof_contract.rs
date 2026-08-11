#![allow(unused_imports)]

use crucible_core::{sha256, DigestDecodeError, HashError, Sha256Digest};
use vstd::prelude::*;

verus! {

#[test]
#[allow(unused_variables)]
fn executable_hash_is_tied_to_the_pure_specification() {
    match sha256(b"abc") {
        Ok(digest) => assert(digest@ == crucible_core::artifact::sha256_spec(b"abc"@)),
        Err(HashError::InputTooLong) => assert(false),
    }
}

#[test]
fn every_canonical_digest_encoding_is_accepted_by_the_decoder() {
    let digest = Sha256Digest { bytes: [0u8;32] };
    let encoded = digest.to_hex();
    proof {
        crucible_core::artifact::lemma_hex_encode_is_canonical(digest@);
    }
    assert(crucible_core::artifact::canonical_sha256_hex_spec(encoded@));
    match Sha256Digest::from_hex(encoded) {
        Ok(_) => {},
        Err(DigestDecodeError::WrongLength) | Err(DigestDecodeError::NonCanonicalHex) => {
            assert(false)
        },
    }
}

} // verus!
