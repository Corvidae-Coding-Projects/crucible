#[expect(
    unused_imports,
    reason = "the import is consumed only by Verus proof code"
)]
use crucible_core::execution::{RawExecutionEventView, RawExecutionOutcomeView};
use crucible_core::observation::*;
use crucible_core::{CompletionDisposition, RawExecutionOutcome, RunAttemptId, RunId};
use vstd::prelude::*;

verus! {

proof fn absolute_observation_limits_cannot_be_raised() {
    let requested = RawObservationLimitsView {
        outcome_limits: crucible_core::execution::RawExecutionOutcomeLimitsView {
            max_events: u64::MAX,
            max_extension_namespace_code_points: u64::MAX,
            max_extension_media_type_code_points: u64::MAX,
            max_extension_payload_bytes_per_record: u64::MAX,
        },
        max_identity_code_points: u64::MAX,
        max_resource_extensions: u64::MAX,
        max_extensions: u64::MAX,
        max_extension_namespace_code_points: u64::MAX,
        max_extension_media_type_code_points: u64::MAX,
        max_extension_payload_bytes_per_record: u64::MAX,
    };
    assert(effective_raw_observation_identity_limit_spec(requested.max_identity_code_points)
        == MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS);
    assert(effective_raw_observation_resource_extension_limit_spec(
        requested.max_resource_extensions,
    ) == MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS);
    assert(effective_raw_observation_extension_limit_spec(requested.max_extensions)
        == MAX_RAW_OBSERVATION_EXTENSIONS);
}

proof fn successful_validation_exposes_exact_public_semantics(
    source: RawObservationView,
    limits: RawObservationLimitsView,
) {
    let result = validate_raw_observation_spec(source, limits);
    if let Ok(validated) = result {
        lemma_successful_raw_observation_validation_has_semantics(source, limits);
        assert(validated == source);
        assert(raw_observation_semantics_with_limits_spec(validated, limits));
    }
}

proof fn pure_success_preserves_the_exact_observation(
    source: RawObservationView,
    limits: RawObservationLimitsView,
) {
    lemma_raw_observation_validation_preserves_exact_input(source, limits);
}

#[expect(
    dead_code,
    reason = "this executable wrapper exists solely for Verus to check rejection ownership"
)]
fn executable_rejection_retains_the_exact_owned_observation(
    source: RawObservation,
    limits: RawObservationLimits,
) {
    let ghost before = source@;
    match validate_raw_observation(source, limits) {
        Ok(_) => {},
        Err(_rejection) => assert(_rejection@.observation == before),
    }
}

proof fn caller_lowered_nested_event_limit_cannot_be_laundered_by_the_outer_record(
    source: RawObservationView,
) {
    let outcome = RawExecutionOutcomeView {
        completion: CompletionDisposition::Completed,
        termination: None,
        events: seq![RawExecutionEventView::TimeoutThresholdReached],
    };
    let outcome_limits = crucible_core::execution::RawExecutionOutcomeLimitsView {
        max_events: 0,
        max_extension_namespace_code_points:
            crucible_core::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        max_extension_media_type_code_points:
            crucible_core::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        max_extension_payload_bytes_per_record:
            crucible_core::MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    };
    assert(crucible_core::execution::validate_raw_execution_outcome_spec(outcome, outcome_limits)
        != Ok(outcome)) by {
        reveal(crucible_core::execution::validate_raw_execution_outcome_spec);
    }
    let limits = RawObservationLimitsView {
        outcome_limits,
        max_identity_code_points: MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS,
        max_resource_extensions: MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS,
        max_extensions: MAX_RAW_OBSERVATION_EXTENSIONS,
        max_extension_namespace_code_points:
            crucible_core::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        max_extension_media_type_code_points:
            crucible_core::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        max_extension_payload_bytes_per_record:
            crucible_core::MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    };
    if source.outcome == outcome {
        lemma_raw_observation_semantics_enforces_supplied_outcome_limits(source, limits);
        assert(!raw_observation_semantics_with_limits_spec(source, limits));
    }
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    let _ = RunId::new(String::from("run"));
    let _ = RunAttemptId::new(String::from("attempt"));
    let _ = RawExecutionOutcome::new(CompletionDisposition::Completed, None, Vec::new());
}
