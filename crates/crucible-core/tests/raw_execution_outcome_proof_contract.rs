#![allow(clippy::result_large_err)]

use crucible_core::{
    validate_raw_execution_outcome, RawExecutionOutcome, RawExecutionOutcomeLimits,
    RawExecutionOutcomeRejection, ValidatedRawExecutionOutcome,
};
#[allow(unused_imports)]
use vstd::prelude::*;

verus! {

#[allow(dead_code)]
fn executable_validation_has_the_exact_total_pure_result(
    outcome: RawExecutionOutcome,
    limits: RawExecutionOutcomeLimits,
) -> (result: Result<ValidatedRawExecutionOutcome, RawExecutionOutcomeRejection>)
    ensures
        match &result {
            Ok(validated) => {
                crucible_core::execution::validate_raw_execution_outcome_spec(outcome@, limits@)
                    == Ok(validated@)
            },
            Err(rejection) => {
                crucible_core::execution::validate_raw_execution_outcome_spec(outcome@, limits@)
                    == Err(rejection@.error) && rejection@.outcome == outcome@
            },
        },
{
    validate_raw_execution_outcome(outcome, limits)
}

proof fn successful_validation_exposes_public_semantics(
    outcome: crucible_core::execution::RawExecutionOutcomeView,
    limits: crucible_core::RawExecutionOutcomeLimitsView,
    validated: crucible_core::execution::RawExecutionOutcomeView,
)
    requires
        crucible_core::execution::validate_raw_execution_outcome_spec(outcome, limits) == Ok(
            validated,
        ),
    ensures
        crucible_core::execution::raw_execution_outcome_semantics_spec(validated),
        validated == outcome,
{
    crucible_core::execution::lemma_successful_raw_execution_validation_has_semantics(
        outcome,
        limits,
        validated,
    );
}

proof fn forged_validated_outcome_cannot_replace_the_input(
    outcome: crucible_core::execution::RawExecutionOutcomeView,
    forged: crucible_core::execution::RawExecutionOutcomeView,
    limits: crucible_core::RawExecutionOutcomeLimitsView,
)
    requires
        outcome != forged,
    ensures
        crucible_core::execution::validate_raw_execution_outcome_spec(outcome, limits) != Ok(
            forged,
        ),
{
    crucible_core::execution::lemma_raw_execution_validation_preserves_exact_input(outcome, limits);
}

proof fn a_forged_zero_process_identifier_is_not_semantically_valid(
    outcome: crucible_core::execution::RawExecutionOutcomeView,
    limits: crucible_core::RawExecutionOutcomeLimitsView,
)
    requires
        outcome.events.len() == 1,
        outcome.events[0] == (crucible_core::execution::RawExecutionEventView::ProcessCreated {
            logical_process: 0,
        }),
    ensures
        crucible_core::execution::validate_raw_execution_outcome_spec(outcome, limits) is Err,
{
    reveal(crucible_core::execution::validate_raw_execution_outcome_spec);
    reveal(crucible_core::execution::termination_validation_spec);
    reveal(crucible_core::execution::validate_raw_execution_events_spec);
    reveal(crucible_core::execution::event_validation_spec);
}

proof fn absolute_limits_clamp_unbounded_caller_requests() {
    assert(crucible_core::execution::effective_raw_execution_limit_spec(
        u64::MAX,
        crucible_core::MAX_RAW_EXECUTION_EVENTS,
    ) == crucible_core::MAX_RAW_EXECUTION_EVENTS);
    assert(crucible_core::execution::effective_raw_execution_limit_spec(
        u64::MAX,
        crucible_core::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    ) == crucible_core::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS);
    assert(crucible_core::execution::effective_raw_execution_limit_spec(
        u64::MAX,
        crucible_core::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    ) == crucible_core::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS);
    assert(crucible_core::execution::effective_raw_execution_limit_spec(
        u64::MAX,
        crucible_core::MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ) == crucible_core::MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD);
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(crucible_core::RAW_EXECUTION_OUTCOME_SCHEMA_VERSION, 1);
}
