use crucible_cli::{canonical_configuration_limits, validate_configuration};

const CONFIGURATION: &str = r#"version: 1
language: {profile: crucible-yaml-1}
project: {name: projected-target}
target: {adapter: cli, command: /opt/target, args: [alpha, "β"]}
execution: {timeout_ms: 250, memory_mb: 128, max_processes: 4, max_output_mb: 2, network: false, required_capabilities: [process_group_termination, resource_limits]}
oracles: {process_exit: {allowed_codes: [0, -7], timeout_is_failure: true}}
inputs: {corpus: [seed-a, "种子"]}
engines: {fuzz: {enabled: false, modes: [], native_backends: []}, property: {enabled: false}, differential: {enabled: false}, metamorphic: {enabled: false}, fault: {enabled: false}, concurrency: {enabled: false}, symbolic: {enabled: false}, mutation: {enabled: false}}
sanitizers: {address: false, undefined: false, thread: false, memory: false, leak: false}
campaign: {duration: 1s, workers: 1, seed: 18446744073709551615}
storage: {root: .crucible}
verification: {verus: {required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}
"#;

fn points(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

#[test]
fn validated_configuration_exposes_the_exact_execution_facing_projection() {
    let validated =
        validate_configuration(CONFIGURATION.as_bytes(), canonical_configuration_limits())
            .expect("configuration validates");
    let execution = validated.execution();

    assert_eq!(execution.project_name(), points("projected-target"));
    assert_eq!(execution.target_command(), points("/opt/target"));
    assert_eq!(execution.target_arguments(), [points("alpha"), points("β")]);
    assert_eq!(execution.timeout_ms(), 250);
    assert_eq!(execution.memory_mb(), 128);
    assert_eq!(execution.max_processes(), 4);
    assert_eq!(execution.max_output_mb(), 2);
    assert!(!execution.network_enabled());
    assert_eq!(
        execution.required_capabilities(),
        [
            points("process_group_termination"),
            points("resource_limits")
        ]
    );
    assert_eq!(execution.allowed_exit_codes(), [0, -7]);
    assert!(execution.timeout_is_failure());
    assert_eq!(execution.corpus(), [points("seed-a"), points("种子")]);
    assert_eq!(execution.campaign_seed(), u64::MAX);
    assert_eq!(execution.storage_root(), points(".crucible"));
}
