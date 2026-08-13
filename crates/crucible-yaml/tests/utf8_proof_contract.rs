#![expect(
    clippy::single_match,
    reason = "explicit match arms carry branch-specific Verus assertions"
)]

#[expect(
    unused_imports,
    reason = "the error kind is referenced only inside Verus proof code"
)]
use crucible_yaml::DecodeErrorKind;
use crucible_yaml::{decode_profile1, BomPolicy, DecodeLimits};
use vstd::prelude::*;

verus! {

#[test]
fn executable_decoder_is_tied_to_bytes_spans_and_profile_bounds() {
    let input: &[u8] = &[0x61, 0x0d, 0x0a, 0xc3, 0xa9];
    let limits = DecodeLimits::new(128, 128);
    proof {
        reveal_with_fuel(crucible_yaml::utf8::profile1_decodable_tail_spec, 6);
        assert(crucible_yaml::utf8::profile1_decodable_spec(
            input@,
            limits@,
            BomPolicy::AllowAndStrip,
        ));
        crucible_yaml::utf8::lemma_profile1_decodable_is_ok(
            input@,
            limits@,
            BomPolicy::AllowAndStrip,
        );
    }
    match decode_profile1(input, limits, BomPolicy::AllowAndStrip) {
        Ok(_source) => {
            assert(crucible_yaml::utf8::decode_profile1_spec(
                input@,
                limits@,
                BomPolicy::AllowAndStrip,
            ) == Ok(_source@));
            assert(_source@.source_len_bytes == 5);
            assert(_source@.scalars.len() <= 128);
            assert(crucible_yaml::utf8::decoded_source_well_formed_spec(_source@));
            assert(crucible_yaml::utf8::decoded_source_matches_input_spec(input@, _source@));
        },
        Err(_) => assert(false),
    }
}

#[test]
fn forbidden_bom_cannot_appear_in_a_successful_decode() {
    let input: &[u8] = &[0xef, 0xbb, 0xbf, 0x70, 0x6c, 0x61, 0x69, 0x6e];
    let limits = DecodeLimits::new(16, 16);
    proof {
        crucible_yaml::utf8::lemma_profile1_forbidden_bom_error(input@, limits@);
    }
    match decode_profile1(input, limits, BomPolicy::Forbid) {
        Err(_error) => {
            assert(_error@.kind == DecodeErrorKind::ForbiddenByteOrderMark);
            assert(_error@.byte_offset == 0);
            assert(crucible_yaml::utf8::decode_profile1_spec(input@, limits@, BomPolicy::Forbid)
                == Err(_error@));
        },
        Ok(_) => assert(false),
    }
}

#[test]
fn malformed_and_limited_inputs_have_exact_pure_results() {
    let malformed: &[u8] = &[0xe2, 0x28];
    let limits = DecodeLimits::new(16, 16);
    proof {
        crucible_yaml::utf8::lemma_profile1_non_bom_first_error(
            malformed@,
            limits@,
            BomPolicy::AllowAndStrip,
            DecodeErrorKind::InvalidContinuationByte,
            1,
        );
    }
    match decode_profile1(malformed, limits, BomPolicy::AllowAndStrip) {
        Err(_error) => {
            assert(_error@.kind == DecodeErrorKind::InvalidContinuationByte);
            assert(_error@.byte_offset == 1);
            assert(crucible_yaml::utf8::decode_profile1_spec(
                malformed@,
                limits@,
                BomPolicy::AllowAndStrip,
            ) == Err(_error@));
        },
        Ok(_) => assert(false),
    }

    let limited: &[u8] = &[0x78];
    let zero = DecodeLimits::new(0, 0);
    proof {
        crucible_yaml::utf8::lemma_profile1_source_limit_error(
            limited@,
            zero@,
            BomPolicy::AllowAndStrip,
        );
    }
    match decode_profile1(limited, zero, BomPolicy::AllowAndStrip) {
        Err(_error) => {
            assert(_error@.kind == DecodeErrorKind::SourceByteLimitExceeded);
            assert(_error@.byte_offset == 0);
            assert(crucible_yaml::utf8::decode_profile1_spec(
                limited@,
                zero@,
                BomPolicy::AllowAndStrip,
            ) == Err(_error@));
        },
        Ok(_) => assert(false),
    }
}

} // verus!
