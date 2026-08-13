#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crucible_core::observation_codec::*;
use vstd::prelude::*;

verus! {

proof fn encoded_cap_cannot_be_raised() {
    assert(effective_raw_observation_encoded_limit_spec(u64::MAX)
        == MAX_RAW_OBSERVATION_ENCODED_BYTES);
}

proof fn decoder_rejection_retains_exact_bytes(
    encoded: Seq<u8>,
    limits: RawObservationCodecLimitsView,
    rejection: RawObservationCodecRejectionView,
) {
    if raw_observation_decode_contract_spec(encoded, limits, Err(rejection)) {
        lemma_raw_observation_decode_contract_rejection_preserves_bytes(encoded, limits, rejection);
        assert(rejection.encoded == encoded);
    }
}

proof fn decoder_success_contract_exposes_bounded_observation_semantics(
    encoded: Seq<u8>,
    limits: RawObservationCodecLimitsView,
    observation: crucible_core::observation::RawObservationView,
) {
    if raw_observation_decode_contract_spec(encoded, limits, Ok(observation)) {
        lemma_raw_observation_decode_contract_success_has_semantics(encoded, limits, observation);
        assert(crucible_core::observation::raw_observation_semantics_with_limits_spec(
            observation,
            limits.observation_limits,
        ));
    }
}

} // verus!
#[test]
fn proof_contract_is_compiled() {}
