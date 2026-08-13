use crucible_cli::{
    classify_raw_local_execution, local_capability_manifest, target_build_manifest,
    validate_local_capability_probe, LocalCapabilityProbeError, LocalExecutionClassificationError,
    LocalExecutionEvidence, LocalExecutionPlan, LocalRuntimeIdentity, RawLocalExecution,
    ValidatedLocalCapabilityProbe,
};
use crucible_core::ArtifactRef;
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "wrapper exposes raw host classification correspondence to Verus")]
fn accepted_raw_execution_is_semantically_well_formed(raw: RawLocalExecution) -> (result: Result<
    LocalExecutionEvidence,
    LocalExecutionClassificationError,
>)
    requires
        crucible_cli::raw_local_execution_well_formed_spec(raw@),
    ensures
        result is Ok ==> crucible_cli::local_execution_evidence_well_formed_spec(result.unwrap()@),
        result is Ok ==> crucible_cli::local_execution_classification_spec(raw@) == Some(
            result.unwrap()@,
        ),
{
    classify_raw_local_execution(raw)
}

#[expect(dead_code, reason = "wrapper exposes capability report authentication to Verus")]
fn accepted_probe_matches_the_exact_plan(plan: &LocalExecutionPlan, report: Vec<u8>) -> (result:
    Result<ValidatedLocalCapabilityProbe, LocalCapabilityProbeError>)
    requires
        crucible_cli::local_execution_plan_well_formed_spec(plan@),
    ensures
        result is Ok ==> crucible_cli::local_capability_probe_matches_plan_spec(
            result.unwrap()@,
            plan@,
        ),
{
    validate_local_capability_probe(plan, report)
}

#[expect(dead_code, reason = "wrapper exposes byte-exact capability-manifest authentication")]
fn capability_manifest_binds_the_probe_and_its_artifact(
    plan: &LocalExecutionPlan,
    probe: &ValidatedLocalCapabilityProbe,
    artifact: &ArtifactRef,
) -> (manifest: Vec<u8>)
    requires
        crucible_cli::local_execution_plan_well_formed_spec(plan@),
        crucible_cli::local_capability_probe_matches_plan_spec(probe@, plan@),
    ensures
        manifest@ == crucible_cli::local_capability_manifest_spec(plan@, probe@, artifact@),
{
    local_capability_manifest(plan, probe, artifact)
}

#[expect(dead_code, reason = "wrapper exposes byte-exact target/runtime-manifest authentication")]
fn target_manifest_binds_every_runtime_identity(
    target: &ArtifactRef,
    runtime: &LocalRuntimeIdentity,
) -> (manifest: Vec<u8>)
    requires
        crucible_cli::local_runtime_identity_well_formed_spec(runtime@),
    ensures
        manifest@ == crucible_cli::target_build_manifest_spec(target@, runtime@),
{
    target_build_manifest(target, runtime)
}

proof fn terminal_run_states_are_absorbing() {
    assert(crucible_cli::run_store_transition_spec(
        crucible_cli::RunAttemptStatus::Observed,
        crucible_cli::RunStoreTransition::RecordHarnessFailure,
    ) is None);
    assert(crucible_cli::run_store_transition_spec(
        crucible_cli::RunAttemptStatus::HarnessFailure,
        crucible_cli::RunStoreTransition::RecordObservation,
    ) is None);
}

} // verus!
fn main() {}
