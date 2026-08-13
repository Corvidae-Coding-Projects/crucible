use crucible_core::{
    decode_raw_execution_outcome, encode_raw_execution_outcome, RawExecutionOutcomeCodecError,
    RawExecutionOutcomeCodecLimits, RawExecutionOutcomeCodecRejection,
    ValidatedRawExecutionOutcome,
};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_decoder_exposes_semantics_and_preserves_every_rejected_byte(
    encoded: Vec<u8>,
    limits: RawExecutionOutcomeCodecLimits,
) -> (result: Result<ValidatedRawExecutionOutcome, RawExecutionOutcomeCodecRejection>)
    ensures
        match &result {
            Ok(outcome) => crucible_core::execution::raw_execution_outcome_semantics_spec(outcome@),
            Err(rejection) => rejection@.encoded == encoded@,
        },
{
    decode_raw_execution_outcome(encoded, limits)
}

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_encoder_never_exceeds_the_clamped_absolute_limit(
    outcome: &ValidatedRawExecutionOutcome,
    requested_limit: u64,
) -> (result: Result<Vec<u8>, RawExecutionOutcomeCodecError>)
    ensures
        match &result {
            Ok(encoded) => encoded@.len()
                <= crucible_core::execution_codec::effective_raw_execution_encoded_limit_spec(
                requested_limit,
            ),
            Err(error) => error@.kind
                == crucible_core::RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
        },
{
    encode_raw_execution_outcome(outcome, requested_limit)
}

proof fn codec_error_stable_tag_endpoints_are_fixed() {
    assert(crucible_core::execution_codec::raw_execution_outcome_codec_error_kind_stable_tag_spec(
        crucible_core::RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
    ) == 1);
    assert(crucible_core::execution_codec::raw_execution_outcome_codec_error_kind_stable_tag_spec(
        crucible_core::RawExecutionOutcomeCodecErrorKind::SemanticValidationFailed,
    ) == 21);
}

proof fn encoded_byte_limit_cannot_be_raised() {
    assert(crucible_core::execution_codec::effective_raw_execution_encoded_limit_spec(u64::MAX)
        == crucible_core::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES);
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(
        crucible_core::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
        134_217_728
    );
}
