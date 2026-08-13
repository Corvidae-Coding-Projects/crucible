use crucible_cli::{
    admit_run_store_transition, RunAttemptStatus, RunStoreTransition, RunStoreTransitionError,
};

#[test]
fn immutable_attempt_transitions_are_admitted_by_verified_policy() {
    assert_eq!(
        admit_run_store_transition(RunAttemptStatus::Reserved, RunStoreTransition::AttachTarget),
        Ok(RunAttemptStatus::TargetPrepared)
    );
    assert_eq!(
        admit_run_store_transition(
            RunAttemptStatus::TargetPrepared,
            RunStoreTransition::RecordObservation,
        ),
        Ok(RunAttemptStatus::Observed)
    );
    assert_eq!(
        admit_run_store_transition(
            RunAttemptStatus::TargetPrepared,
            RunStoreTransition::RecordHarnessFailure,
        ),
        Ok(RunAttemptStatus::HarnessFailure)
    );
}

#[test]
fn terminal_attempts_cannot_be_rewritten_or_laundered() {
    assert_eq!(
        admit_run_store_transition(
            RunAttemptStatus::Observed,
            RunStoreTransition::RecordHarnessFailure,
        ),
        Err(RunStoreTransitionError::InvalidTransition)
    );
    assert_eq!(
        admit_run_store_transition(
            RunAttemptStatus::HarnessFailure,
            RunStoreTransition::RecordObservation,
        ),
        Err(RunStoreTransitionError::InvalidTransition)
    );
}
