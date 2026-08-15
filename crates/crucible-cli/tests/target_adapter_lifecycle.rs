use crucible_cli::{
    canonical_configuration_limits, prepare_local_cli_target_instance, prepare_local_execution,
    validate_configuration,
};
use crucible_core::{
    advance_target_instance_lifecycle, RunAttemptId, TargetBuildId, TargetId,
    TargetLifecycleAction, TargetLifecycleState,
};

const CONFIGURATION: &str = r#"version: 1
language: {profile: crucible-yaml-1}
project: {name: adapter-lifecycle}
target: {adapter: cli, command: ./target, args: []}
execution: {timeout_ms: 250, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}
oracles: {process_exit: {allowed_codes: [0], timeout_is_failure: true}}
inputs: {corpus: []}
engines: {fuzz: {enabled: false, modes: [], native_backends: []}, property: {enabled: false}, differential: {enabled: false}, metamorphic: {enabled: false}, fault: {enabled: false}, concurrency: {enabled: false}, symbolic: {enabled: false}, mutation: {enabled: false}}
sanitizers: {address: false, undefined: false, thread: false, memory: false, leak: false}
campaign: {duration: 1s, workers: 1, seed: 7}
storage: {root: .crucible}
verification: {verus: {required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}
"#;

#[test]
fn linux_cli_plan_prepares_one_identity_bound_exclusive_instance() {
    let configuration =
        validate_configuration(CONFIGURATION.as_bytes(), canonical_configuration_limits())
            .expect("configuration validates");
    let plan = prepare_local_execution(configuration.execution()).expect("local plan");
    let prepared = prepare_local_cli_target_instance(
        &plan,
        TargetId::new(String::from("target-lifecycle")),
        TargetBuildId::new(String::from("target-build-lifecycle")),
        RunAttemptId::new(String::from("attempt-lifecycle")),
        1,
    )
    .expect("CLI target prepares");
    assert_eq!(prepared.state(), TargetLifecycleState::Prepared);
    assert_eq!(prepared.target_id().as_str(), "target-lifecycle");
    assert_eq!(
        prepared.target_build_id().as_str(),
        "target-build-lifecycle"
    );
    assert_eq!(prepared.owner_attempt_id().as_str(), "attempt-lifecycle");

    let executing =
        advance_target_instance_lifecycle(prepared, TargetLifecycleAction::BeginExecute)
            .expect("execution begins");
    let reset_required =
        advance_target_instance_lifecycle(executing, TargetLifecycleAction::FinishExecute)
            .expect("execution finishes");
    let cleaned =
        advance_target_instance_lifecycle(reset_required, TargetLifecycleAction::CleanupSucceeded)
            .expect("stateless CLI instance is cleaned instead of pooled");
    assert_eq!(cleaned.state(), TargetLifecycleState::Cleaned);
}
