use crucible_cli::{
    build_local_raw_observation, CapturedOutput, LocalExecutionEvidence, LocalTermination,
};
use crucible_core::{
    CompletionDisposition, ContentDigest, HarnessTerminationReason, RawExecutionEvent,
    RunAttemptId, RunId, TerminationRecord,
};

fn artifact(bytes: &[u8]) -> crucible_core::ArtifactRef {
    crucible_core::ArtifactRef {
        id: ContentDigest::from_bytes(bytes)
            .expect("small artifact hashes")
            .into_artifact_id(),
        size_bytes: bytes.len() as u64,
        media_type: None,
    }
}

#[test]
fn timeout_and_exact_stream_accounting_become_portable_raw_facts() {
    let stdout = CapturedOutput::new(b"retained".to_vec(), 4);
    let stderr = CapturedOutput::new(Vec::new(), 0);
    let evidence =
        LocalExecutionEvidence::new(LocalTermination::Timeout, stdout, stderr, 0, 125_000_000)
            .expect("valid host evidence");
    let observation = build_local_raw_observation(
        RunId::new(String::from("run-1")),
        RunAttemptId::new(String::from("attempt-1")),
        &evidence,
        artifact(evidence.stdout().retained()),
        artifact(evidence.stderr().retained()),
    )
    .expect("validated raw observation");
    let raw = observation.observation();
    assert_eq!(raw.outcome().completion(), CompletionDisposition::Completed);
    assert_eq!(
        raw.outcome().termination(),
        &Some(TerminationRecord::HarnessTerminated {
            reason: HarnessTerminationReason::Timeout,
        })
    );
    assert!(raw
        .outcome()
        .events()
        .contains(&RawExecutionEvent::TimeoutThresholdReached));
    assert!(raw.stdout().truncated());
    assert_eq!(raw.stdout().retained_bytes(), 8);
    assert_eq!(raw.stdout().discarded_bytes(), 4);
    assert!(!raw.stderr().truncated());
    assert_eq!(raw.wall_time().seconds(), 0);
    assert_eq!(raw.wall_time().nanoseconds(), 125_000_000);
}

#[test]
fn retained_artifact_mismatch_is_rejected_before_observation_admission() {
    let evidence = LocalExecutionEvidence::new(
        LocalTermination::ExitCode(7),
        CapturedOutput::new(b"abc".to_vec(), 0),
        CapturedOutput::new(Vec::new(), 0),
        0,
        1,
    )
    .expect("valid host evidence");
    assert!(build_local_raw_observation(
        RunId::new(String::from("run-1")),
        RunAttemptId::new(String::from("attempt-1")),
        &evidence,
        artifact(b"wrong"),
        artifact(b""),
    )
    .is_err());
}
