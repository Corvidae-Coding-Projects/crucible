use crucible_cli::{
    canonical_configuration_limits, canonical_local_capability_probe_report,
    encode_local_target_arguments, local_capability_manifest, parse_cli_args,
    prepare_local_execution, target_build_manifest, validate_configuration,
    validate_local_capability_probe, CliAction, LocalExecutionBackend, LocalNetworkPolicy,
    LocalRunPlanError, LocalRuntimeIdentity, OutputCapturePolicy, ReservedRun,
};
use crucible_core::ArtifactRef;

const CONFIGURATION: &str = r#"version: 1
language: {profile: crucible-yaml-1}
project: {name: run-plan}
target: {adapter: cli, command: ./target, args: [alpha, beta]}
execution: {timeout_ms: 250, memory_mb: 128, max_processes: 4, max_output_mb: 2, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}
oracles: {process_exit: {allowed_codes: [0], timeout_is_failure: true}}
inputs: {corpus: []}
engines: {fuzz: {enabled: false, modes: [], native_backends: []}, property: {enabled: false}, differential: {enabled: false}, metamorphic: {enabled: false}, fault: {enabled: false}, concurrency: {enabled: false}, symbolic: {enabled: false}, mutation: {enabled: false}}
sanitizers: {address: false, undefined: false, thread: false, memory: false, leak: false}
campaign: {duration: 1s, workers: 1, seed: 7}
storage: {root: .crucible}
verification: {verus: {required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}
"#;

fn plan_for(source: &str) -> Result<crucible_cli::LocalExecutionPlan, LocalRunPlanError> {
    let configuration = validate_configuration(source.as_bytes(), canonical_configuration_limits())
        .expect("configuration fixture validates");
    prepare_local_execution(configuration.execution())
}

#[test]
fn execution_plan_expands_controls_without_losing_requested_values() {
    let plan = plan_for(CONFIGURATION).expect("supported local execution plan");
    assert_eq!(
        plan.backend(),
        LocalExecutionBackend::LinuxBubblewrapPrlimitV1
    );
    assert_eq!(plan.network_policy(), LocalNetworkPolicy::None);
    assert_eq!(
        plan.output_capture_policy(),
        OutputCapturePolicy::DrainAndDiscard
    );
    assert_eq!(plan.timeout_ms(), 250);
    assert_eq!(plan.memory_bytes(), 128 * 1_048_576);
    assert_eq!(plan.max_processes(), 4);
    assert_eq!(plan.max_stream_bytes(), 2 * 1_048_576);
    assert_eq!(
        plan.target_command(),
        &[46, 47, 116, 97, 114, 103, 101, 116]
    );
    assert_eq!(
        plan.target_arguments(),
        &[vec![97, 108, 112, 104, 97], vec![98, 101, 116, 97]]
    );
}

#[test]
fn supervisor_argument_wire_preserves_exact_direct_argv_boundaries() {
    let plan = plan_for(CONFIGURATION).expect("supported local execution plan");
    let mut expected = b"CRUCIBLE-ARGV-V1\n".to_vec();
    expected.extend_from_slice(&2_u64.to_be_bytes());
    expected.extend_from_slice(&5_u64.to_be_bytes());
    expected.extend_from_slice(b"alpha");
    expected.extend_from_slice(&4_u64.to_be_bytes());
    expected.extend_from_slice(b"beta");
    assert_eq!(
        encode_local_target_arguments(&plan).expect("bounded exact argv wire"),
        expected,
    );
}

#[test]
fn unknown_or_disabled_required_capabilities_fail_closed_at_the_exact_index() {
    let unknown = CONFIGURATION.replace(
        "process_group_termination, resource_limits, network_isolation, private_working_directory",
        "process_group_termination, future_isolation",
    );
    assert_eq!(
        plan_for(&unknown),
        Err(LocalRunPlanError::RequiredCapabilityUnavailable { index: 1 }),
    );

    let enabled_network = CONFIGURATION.replace("network: false", "network: true");
    assert_eq!(
        plan_for(&enabled_network),
        Err(LocalRunPlanError::RequiredCapabilityUnavailable { index: 2 }),
    );
}

#[test]
fn per_stream_capture_cannot_exceed_the_local_artifact_admission_limit() {
    let oversized = CONFIGURATION.replace("max_output_mb: 2", "max_output_mb: 65");
    assert_eq!(
        plan_for(&oversized),
        Err(LocalRunPlanError::OutputLimitTooLarge)
    );
}

#[test]
fn persisted_manifests_bind_the_backend_policy_and_target_artifact() {
    let plan = plan_for(CONFIGURATION).expect("supported local execution plan");
    let report = canonical_local_capability_probe_report(&plan, true);
    let probe = validate_local_capability_probe(&plan, report).expect("successful exact probe");
    let probe_artifact = artifact('b');
    let capability_manifest =
        String::from_utf8(local_capability_manifest(&plan, &probe, &probe_artifact))
            .expect("manifest is canonical UTF-8");
    assert!(capability_manifest.contains("linux-bubblewrap-prlimit-v1"));
    assert!(capability_manifest.contains("network_isolation\":\"enforced"));
    assert!(capability_manifest.contains(probe_artifact.id.as_str()));

    let target = crucible_core::ArtifactRef {
        id: crucible_core::ContentDigest::from_bytes(b"target")
            .expect("hash target")
            .into_artifact_id(),
        size_bytes: 6,
        media_type: None,
    };
    let runtime = LocalRuntimeIdentity::new(
        String::from("linux"),
        String::from("x86_64"),
        String::from("6.0-review"),
        String::from("bubblewrap 1"),
        String::from("prlimit 1"),
        artifact('c'),
        artifact('d'),
        artifact('e'),
    )
    .expect("bounded runtime identity");
    let target_manifest = String::from_utf8(target_build_manifest(&target, &runtime))
        .expect("target manifest is UTF-8");
    assert!(target_manifest.contains(target.id.as_str()));
    assert!(target_manifest.contains("cli-materialized-executable-v1"));
    assert!(!target_manifest.contains("runtime_snapshot="));
    assert!(target_manifest.contains("unresolved_host_runtime=true"));

    assert!(ReservedRun::new(String::new(), String::from("attempt-1")).is_err());
    assert!(ReservedRun::new(String::from("run-1"), String::new()).is_err());
}

fn artifact(hex: char) -> ArtifactRef {
    let bytes = [hex as u8];
    ArtifactRef {
        id: crucible_core::ContentDigest::from_bytes(&bytes)
            .expect("hash fixture")
            .into_artifact_id(),
        size_bytes: 1,
        media_type: None,
    }
}

#[test]
fn unavailable_probe_cannot_generate_an_enforced_manifest() {
    let plan = plan_for(CONFIGURATION).expect("supported local execution plan");
    let report = canonical_local_capability_probe_report(&plan, false);
    let probe = validate_local_capability_probe(&plan, report).expect("exact unavailable probe");
    assert!(!probe.available());
    let probe_artifact = artifact('f');
    let manifest = String::from_utf8(local_capability_manifest(&plan, &probe, &probe_artifact))
        .expect("manifest UTF-8");
    assert!(manifest.contains("unavailable:probe-failed"));
    assert!(!manifest.contains("network_isolation\":\"enforced"));
}

#[test]
fn the_internal_supervisor_entry_is_an_exact_verified_cli_action() {
    assert_eq!(
        parse_cli_args(&[String::from("__crucible-internal-local-supervisor-v1")]),
        Ok(CliAction::InternalLocalSupervisor),
    );
}
