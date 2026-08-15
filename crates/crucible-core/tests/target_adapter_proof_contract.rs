use crucible_core::{
    advance_target_instance_lifecycle, TargetInstanceLifecycle, TargetLifecycleAction,
    TargetLifecycleError,
};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "wrapper exposes lifecycle transition correspondence to Verus")]
fn accepted_transition_matches_the_total_lifecycle_model(
    lifecycle: TargetInstanceLifecycle,
    action: TargetLifecycleAction,
) -> (result: Result<TargetInstanceLifecycle, TargetLifecycleError>)
    requires
        crucible_core::target_adapter::target_instance_lifecycle_well_formed_spec(lifecycle@),
    ensures
        match &result {
            Ok(next) => crucible_core::target_adapter::target_lifecycle_transition_spec(
                lifecycle@.state,
                action,
            ) == Some(next@.state)
                && crucible_core::target_adapter::same_target_instance_identity_spec(
                lifecycle@,
                next@,
            ),
            Err(TargetLifecycleError::InvalidTransition) => {
                crucible_core::target_adapter::target_lifecycle_transition_spec(
                    lifecycle@.state,
                    action,
                ) is None
            },
            Err(_) => false,
        },
{
    advance_target_instance_lifecycle(lifecycle, action)
}

proof fn execution_requires_prepared_and_reset_before_reuse() {
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::Prepared,
        TargetLifecycleAction::BeginExecute,
    ) == Some(crucible_core::TargetLifecycleState::Executing));
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::Executing,
        TargetLifecycleAction::FinishExecute,
    ) == Some(crucible_core::TargetLifecycleState::ResetRequired));
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::ResetRequired,
        TargetLifecycleAction::BeginExecute,
    ) is None);
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::ResetRequired,
        TargetLifecycleAction::ResetSucceeded,
    ) == Some(crucible_core::TargetLifecycleState::Prepared));
}

proof fn uncertain_reset_discards_and_terminal_states_are_absorbing() {
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::ResetRequired,
        TargetLifecycleAction::ResetUncertain,
    ) == Some(crucible_core::TargetLifecycleState::Discarded));
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::Discarded,
        TargetLifecycleAction::PrepareSucceeded,
    ) is None);
    assert(crucible_core::target_adapter::target_lifecycle_transition_spec(
        crucible_core::TargetLifecycleState::Cleaned,
        TargetLifecycleAction::BeginExecute,
    ) is None);
}

} // verus!
fn main() {}
