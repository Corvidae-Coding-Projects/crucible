use crucible_core::{
    advance_target_instance_lifecycle, RunAttemptId, TargetAdapterIdentity, TargetAdapterKind,
    TargetBuildId, TargetId, TargetInstanceLifecycle, TargetLifecycleAction, TargetLifecycleError,
    TargetLifecycleState, MAX_TARGET_INSTANCE_ORDINAL,
};

fn allocated() -> TargetInstanceLifecycle {
    let adapter = TargetAdapterIdentity::new(TargetAdapterKind::Cli, 1)
        .expect("versioned CLI adapter identity");
    TargetInstanceLifecycle::new(
        adapter,
        TargetId::new(String::from("target-alpha")),
        TargetBuildId::new(String::from("target-build-alpha")),
        RunAttemptId::new(String::from("attempt-alpha")),
        1,
    )
    .expect("bounded target instance identity")
}

fn reset_required() -> TargetInstanceLifecycle {
    let prepared =
        advance_target_instance_lifecycle(allocated(), TargetLifecycleAction::PrepareSucceeded)
            .expect("prepare succeeds");
    let executing =
        advance_target_instance_lifecycle(prepared, TargetLifecycleAction::BeginExecute)
            .expect("execution begins");
    advance_target_instance_lifecycle(executing, TargetLifecycleAction::FinishExecute)
        .expect("execution finishes")
}

#[test]
fn lifecycle_retains_immutable_adapter_build_and_instance_identity() {
    let lifecycle = reset_required();
    assert_eq!(lifecycle.adapter().kind(), TargetAdapterKind::Cli);
    assert_eq!(lifecycle.adapter().version(), 1);
    assert_eq!(lifecycle.target_id().as_str(), "target-alpha");
    assert_eq!(lifecycle.target_build_id().as_str(), "target-build-alpha");
    assert_eq!(lifecycle.instance_ordinal(), 1);
    assert_eq!(lifecycle.owner_attempt_id().as_str(), "attempt-alpha");
    assert_eq!(lifecycle.state(), TargetLifecycleState::ResetRequired);
}

#[test]
fn prepared_instance_is_exclusive_and_cannot_execute_twice_without_reset() {
    let prepared =
        advance_target_instance_lifecycle(allocated(), TargetLifecycleAction::PrepareSucceeded)
            .expect("prepare succeeds");
    let executing =
        advance_target_instance_lifecycle(prepared, TargetLifecycleAction::BeginExecute)
            .expect("first execution begins");
    assert_eq!(executing.state(), TargetLifecycleState::Executing);

    let reset_required =
        advance_target_instance_lifecycle(executing, TargetLifecycleAction::FinishExecute)
            .expect("execution finishes");
    assert_eq!(
        advance_target_instance_lifecycle(reset_required, TargetLifecycleAction::BeginExecute,),
        Err(TargetLifecycleError::InvalidTransition)
    );
}

#[test]
fn successful_reset_reuses_the_same_identity_and_uncertain_reset_discards_it() {
    let reset =
        advance_target_instance_lifecycle(reset_required(), TargetLifecycleAction::ResetSucceeded)
            .expect("confident reset returns to prepared");
    assert_eq!(reset.state(), TargetLifecycleState::Prepared);
    assert_eq!(reset.target_build_id().as_str(), "target-build-alpha");

    let discarded =
        advance_target_instance_lifecycle(reset_required(), TargetLifecycleAction::ResetUncertain)
            .expect("uncertain reset is an admitted discard");
    assert_eq!(discarded.state(), TargetLifecycleState::Discarded);
    assert_eq!(
        advance_target_instance_lifecycle(discarded, TargetLifecycleAction::PrepareSucceeded,),
        Err(TargetLifecycleError::InvalidTransition)
    );
}

#[test]
fn cleanup_is_typed_and_terminal_while_uncertain_cleanup_discards() {
    let prepared =
        advance_target_instance_lifecycle(allocated(), TargetLifecycleAction::PrepareSucceeded)
            .expect("prepare succeeds");
    let cleaned =
        advance_target_instance_lifecycle(prepared, TargetLifecycleAction::CleanupSucceeded)
            .expect("cleanup succeeds");
    assert_eq!(cleaned.state(), TargetLifecycleState::Cleaned);
    assert_eq!(
        advance_target_instance_lifecycle(cleaned, TargetLifecycleAction::BeginExecute),
        Err(TargetLifecycleError::InvalidTransition)
    );

    let discarded = advance_target_instance_lifecycle(
        reset_required(),
        TargetLifecycleAction::CleanupUncertain,
    )
    .expect("uncertain cleanup discards the instance");
    assert_eq!(discarded.state(), TargetLifecycleState::Discarded);
}

#[test]
fn failed_preparation_discards_and_identity_admission_is_bounded() {
    let discarded =
        advance_target_instance_lifecycle(allocated(), TargetLifecycleAction::PrepareFailed)
            .expect("failed preparation discards the allocation");
    assert_eq!(discarded.state(), TargetLifecycleState::Discarded);

    let adapter = TargetAdapterIdentity::new(TargetAdapterKind::Cli, 1).unwrap();
    assert_eq!(
        TargetInstanceLifecycle::new(
            adapter,
            TargetId::new(String::new()),
            TargetBuildId::new(String::from("build")),
            RunAttemptId::new(String::new()),
            1,
        ),
        Err(TargetLifecycleError::EmptyTargetId)
    );
    let adapter = TargetAdapterIdentity::new(TargetAdapterKind::Cli, 1).unwrap();
    assert_eq!(
        TargetInstanceLifecycle::new(
            adapter,
            TargetId::new(String::from("target")),
            TargetBuildId::new(String::new()),
            RunAttemptId::new(String::from("attempt")),
            1,
        ),
        Err(TargetLifecycleError::EmptyTargetBuildId)
    );
    let adapter = TargetAdapterIdentity::new(TargetAdapterKind::Cli, 1).unwrap();
    assert_eq!(
        TargetInstanceLifecycle::new(
            adapter,
            TargetId::new(String::from("target")),
            TargetBuildId::new(String::from("build")),
            RunAttemptId::new(String::new()),
            1,
        ),
        Err(TargetLifecycleError::EmptyOwnerAttemptId)
    );
    let adapter = TargetAdapterIdentity::new(TargetAdapterKind::Cli, 1).unwrap();
    assert_eq!(
        TargetInstanceLifecycle::new(
            adapter,
            TargetId::new(String::from("target")),
            TargetBuildId::new(String::from("build")),
            RunAttemptId::new(String::from("attempt")),
            MAX_TARGET_INSTANCE_ORDINAL + 1,
        ),
        Err(TargetLifecycleError::InstanceOrdinalOutOfRange)
    );
    assert_eq!(
        TargetAdapterIdentity::new(TargetAdapterKind::Cli, 0),
        Err(TargetLifecycleError::InvalidAdapterVersion)
    );
}
