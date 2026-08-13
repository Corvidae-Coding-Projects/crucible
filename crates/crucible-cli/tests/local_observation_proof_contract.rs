use crucible_cli::{
    build_local_raw_observation, CapturedOutput, LocalExecutionEvidence, LocalObservationError,
    LocalTermination,
};
use crucible_core::{ArtifactRef, RunAttemptId, RunId, ValidatedRawObservation};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "wrapper exposes the executable observation contract to Verus")]
fn accepted_host_evidence_cannot_launder_an_invalid_raw_observation(
    run_id: RunId,
    attempt_id: RunAttemptId,
    evidence: &LocalExecutionEvidence,
    stdout: ArtifactRef,
    stderr: ArtifactRef,
) -> (result: Result<ValidatedRawObservation, LocalObservationError>)
    requires
        run_id@.len() > 0,
        attempt_id@.len() > 0,
        crucible_cli::local_execution_evidence_well_formed_spec(evidence@),
        stdout@ == (crucible_core::artifact::ArtifactRefView {
            id: stdout@.id,
            size_bytes: evidence@.stdout.retained.len() as u64,
            media_type: None,
        }),
        stderr@ == (crucible_core::artifact::ArtifactRefView {
            id: stderr@.id,
            size_bytes: evidence@.stderr.retained.len() as u64,
            media_type: None,
        }),
    ensures
        result is Ok ==> crucible_core::observation::raw_observation_semantics_spec(
            result.unwrap()@,
        ),
{
    build_local_raw_observation(run_id, attempt_id, evidence, stdout, stderr)
}

#[expect(dead_code, reason = "proof fixture is consumed by Verus rather than the Rust test runner")]
fn constructor_rejects_nonportable_nanoseconds(stdout: CapturedOutput, stderr: CapturedOutput) {
    let _result = LocalExecutionEvidence::new(
        LocalTermination::ExitCode(0),
        stdout,
        stderr,
        0,
        1_000_000_000,
    );
    assert(_result is Err);
}

} // verus!
fn main() {}
