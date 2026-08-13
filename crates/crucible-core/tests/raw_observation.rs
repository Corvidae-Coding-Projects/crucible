use crucible_core::{
    canonical_raw_execution_outcome_limits, canonical_raw_observation_limits,
    validate_raw_execution_outcome, validate_raw_observation, ArtifactId, ArtifactRef,
    CapturedStreamRef, CompletionDisposition, CoverageProviderId, CoverageRef, FaultTrace,
    RawExecutionEvent, RawExecutionOutcome, RawExecutionOutcomeErrorKind,
    RawExecutionOutcomeLimits, RawExecutionOutcomeLocation, RawObservation,
    RawObservationErrorKind, RawObservationLimits, RawObservationLocation, RecordedDuration,
    RecordedDurationError, ResourceSnapshot, RunAttemptId, RunId, ScheduleTrace, StateDigest,
    TargetBuildId, VersionedExtensionRef,
};

fn artifact(contents: &[u8], media_type: Option<&str>) -> ArtifactRef {
    ArtifactRef::from_bytes(contents, media_type.map(String::from)).unwrap()
}

fn validated_outcome() -> crucible_core::ValidatedRawExecutionOutcome {
    validate_raw_execution_outcome(
        RawExecutionOutcome::new(CompletionDisposition::Completed, None, Vec::new()),
        canonical_raw_execution_outcome_limits(),
    )
    .unwrap()
}

fn stream(contents: &[u8]) -> CapturedStreamRef {
    CapturedStreamRef::new(
        artifact(contents, Some("application/octet-stream")),
        false,
        contents.len() as u64,
        0,
    )
}

fn empty_resources() -> ResourceSnapshot {
    ResourceSnapshot::new(None, None, None, None, None, None, Vec::new())
}

fn candidate(
    run_id: RunId,
    stdout: CapturedStreamRef,
    stderr: CapturedStreamRef,
    resources: ResourceSnapshot,
    coverage: Option<CoverageRef>,
    fault_trace: Option<FaultTrace>,
    extensions: Vec<VersionedExtensionRef>,
) -> RawObservation {
    RawObservation::new(
        run_id,
        RunAttemptId::new(String::from("attempt-1")),
        validated_outcome().into_inner(),
        stdout,
        stderr,
        RecordedDuration::new(2, 3).unwrap(),
        None,
        None,
        resources,
        coverage,
        None,
        None,
        fault_trace,
        extensions,
    )
}

fn minimal_observation() -> RawObservation {
    candidate(
        RunId::new(String::from("run-1")),
        stream(b"stdout"),
        stream(b"stderr"),
        empty_resources(),
        None,
        None,
        Vec::new(),
    )
}

#[test]
fn complete_observation_retains_every_raw_field_without_coercion() {
    let coverage = CoverageRef::new(
        CoverageProviderId::new(String::from("llvm.edge")),
        String::from("19.1.7"),
        TargetBuildId::new(String::from("build-7")),
        String::from("sha256:feature-space"),
        artifact(b"coverage", Some("application/vnd.crucible.coverage")),
        4,
        91,
    );
    let state = StateDigest::new(
        String::from("com.example.state"),
        2,
        artifact(b"state", Some("application/vnd.crucible.state")),
    );
    let schedule = ScheduleTrace::new(
        String::from("org.example.schedule"),
        3,
        artifact(b"schedule", None),
        17,
        true,
    );
    let fault = FaultTrace::new(
        String::from("org.example.fault"),
        4,
        artifact(b"fault", None),
        9,
        3,
        2,
        1,
        3,
        true,
    );
    let resource_extension = VersionedExtensionRef::new(
        String::from("com.example.resources"),
        1,
        artifact(b"resource-extension", None),
    );
    let resources = ResourceSnapshot::new(
        Some(2),
        Some(8),
        Some(11),
        Some(23),
        Some(100),
        Some(200),
        vec![resource_extension],
    );
    let extension = VersionedExtensionRef::new(
        String::from("com.example.observation"),
        8,
        artifact(b"observation-extension", None),
    );
    let observation = RawObservation::new(
        RunId::new(String::from("run-全")),
        RunAttemptId::new(String::from("attempt-β")),
        validated_outcome().into_inner(),
        CapturedStreamRef::new(artifact(b"out", None), true, 3, 99),
        stream(b"err"),
        RecordedDuration::new(7, 8).unwrap(),
        Some(RecordedDuration::new(5, 6).unwrap()),
        Some(4_096),
        resources,
        Some(coverage),
        Some(state),
        Some(schedule),
        Some(fault),
        vec![extension],
    );

    let validated = validate_raw_observation(observation, canonical_raw_observation_limits())
        .expect("complete raw observation");
    let value = validated.observation();
    assert_eq!(value.run_id().as_str(), "run-全");
    assert_eq!(value.attempt_id().as_str(), "attempt-β");
    assert_eq!(value.stdout().discarded_bytes(), 99);
    assert_eq!(value.wall_time().nanoseconds(), 8);
    assert_eq!(value.cpu_time().as_ref().unwrap().seconds(), 5);
    assert_eq!(value.peak_rss_bytes(), Some(4_096));
    assert_eq!(value.resources().extensions().len(), 1);
    assert_eq!(value.coverage().as_ref().unwrap().new_features(), 4);
    assert_eq!(value.state_digest().as_ref().unwrap().schema_version(), 2);
    assert_eq!(value.schedule_trace().as_ref().unwrap().decisions(), 17);
    assert_eq!(value.fault_trace().as_ref().unwrap().applied(), 3);
    assert_eq!(value.extensions().len(), 1);
}

#[test]
fn durations_streams_coverage_and_fault_accounting_have_typed_exact_failures() {
    assert_eq!(
        RecordedDuration::new(1, 1_000_000_000).unwrap_err(),
        RecordedDurationError::NanosecondsOutOfRange
    );

    let bad_stream = CapturedStreamRef::new(artifact(b"abc", None), false, 2, 0);
    let observation = candidate(
        RunId::new(String::from("r")),
        bad_stream,
        stream(b""),
        empty_resources(),
        None,
        None,
        Vec::new(),
    );
    let rejection =
        validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::RetainedByteCountMismatch
    );
    assert_eq!(rejection.error().location(), RawObservationLocation::Stdout);

    let observation = candidate(
        RunId::new(String::from("r")),
        CapturedStreamRef::new(artifact(b"abc", None), false, 3, 1),
        stream(b""),
        empty_resources(),
        None,
        None,
        Vec::new(),
    );
    let rejection =
        validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::TruncationFlagMismatch
    );

    let observation = candidate(
        RunId::new(String::from("r")),
        stream(b""),
        stream(b""),
        empty_resources(),
        Some(CoverageRef::new(
            CoverageProviderId::new(String::from("provider")),
            String::from("1"),
            TargetBuildId::new(String::from("build")),
            String::from("features"),
            artifact(b"coverage", None),
            8,
            7,
        )),
        None,
        Vec::new(),
    );
    let rejection =
        validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::CoverageCountMismatch
    );
    assert_eq!(
        rejection.error().location(),
        RawObservationLocation::Coverage
    );

    let observation = candidate(
        RunId::new(String::from("r")),
        stream(b""),
        stream(b""),
        empty_resources(),
        None,
        Some(FaultTrace::new(
            String::from("org.example.fault"),
            1,
            artifact(b"fault", None),
            4,
            2,
            2,
            1,
            0,
            true,
        )),
        Vec::new(),
    );
    let rejection =
        validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::FaultTraceCountMismatch
    );
    assert_eq!(
        rejection.error().location(),
        RawObservationLocation::FaultTrace
    );
}

#[test]
fn nested_outcome_limits_and_exact_first_excluded_diagnostics_are_preserved() {
    let canonical = canonical_raw_observation_limits();
    let outcome = RawExecutionOutcome::new(
        CompletionDisposition::Completed,
        None,
        vec![RawExecutionEvent::PlatformSpecific(
            VersionedExtensionRef::new(String::from("abcd"), 1, artifact(b"nested", None)),
        )],
    );
    let observation = RawObservation::new(
        RunId::new(String::from("run")),
        RunAttemptId::new(String::from("attempt")),
        outcome,
        stream(b""),
        stream(b""),
        RecordedDuration::new(0, 0).unwrap(),
        None,
        None,
        empty_resources(),
        None,
        None,
        None,
        None,
        Vec::new(),
    );
    let limits = RawObservationLimits::new(
        RawExecutionOutcomeLimits::new(
            1,
            3,
            canonical
                .outcome_limits()
                .max_extension_media_type_code_points(),
            canonical
                .outcome_limits()
                .max_extension_payload_bytes_per_record(),
        ),
        canonical.max_identity_code_points(),
        canonical.max_resource_extensions(),
        canonical.max_extensions(),
        canonical.max_extension_namespace_code_points(),
        canonical.max_extension_media_type_code_points(),
        canonical.max_extension_payload_bytes_per_record(),
    );
    let rejection = validate_raw_observation(observation, limits).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::InvalidOutcome
    );
    assert_eq!(
        rejection.error().location(),
        RawObservationLocation::Outcome
    );
    assert_eq!(
        rejection.error().outcome_error_kind(),
        Some(RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded)
    );
    assert_eq!(
        rejection.error().outcome_error_location(),
        Some(RawExecutionOutcomeLocation::Event(0))
    );
    assert_eq!(rejection.error().code_point_index(), Some(3));
}

#[test]
fn inline_media_budget_counts_stream_and_typed_artifact_records() {
    let canonical = canonical_raw_observation_limits();
    let observation = candidate(
        RunId::new(String::from("r")),
        CapturedStreamRef::new(artifact(b"", Some("x")), false, 0, 0),
        CapturedStreamRef::new(artifact(b"", None), false, 0, 0),
        empty_resources(),
        None,
        None,
        Vec::new(),
    );
    let limits = RawObservationLimits::new(
        canonical.outcome_limits(),
        canonical.max_identity_code_points(),
        canonical.max_resource_extensions(),
        canonical.max_extensions(),
        canonical.max_extension_namespace_code_points(),
        0,
        canonical.max_extension_payload_bytes_per_record(),
    );
    let rejection = validate_raw_observation(observation, limits).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::ExtensionMediaTypeLimitExceeded
    );
    assert_eq!(rejection.error().location(), RawObservationLocation::Stdout);
    assert_eq!(rejection.error().code_point_index(), Some(0));
}

#[test]
fn identity_resource_and_extension_limits_are_independent_exact_and_cannot_be_raised() {
    let canonical = canonical_raw_observation_limits();
    assert!(canonical.max_identity_code_points() < u64::MAX);
    assert!(canonical.max_resource_extensions() < u64::MAX);
    assert!(canonical.max_extensions() < u64::MAX);

    let limits = RawObservationLimits::new(
        canonical.outcome_limits(),
        1,
        0,
        0,
        canonical.max_extension_namespace_code_points(),
        canonical.max_extension_media_type_code_points(),
        canonical.max_extension_payload_bytes_per_record(),
    );
    let rejection = validate_raw_observation(minimal_observation(), limits).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::IdentityLimitExceeded
    );
    assert_eq!(rejection.error().location(), RawObservationLocation::RunId);
    assert_eq!(rejection.error().code_point_index(), Some(1));

    let resource_extension =
        VersionedExtensionRef::new(String::from("resource"), 1, artifact(b"x", None));
    let observation = candidate(
        RunId::new(String::from("r")),
        stream(b""),
        stream(b""),
        ResourceSnapshot::new(None, None, None, None, None, None, vec![resource_extension]),
        None,
        None,
        Vec::new(),
    );
    let rejection = validate_raw_observation(
        observation,
        RawObservationLimits::new(
            canonical.outcome_limits(),
            canonical.max_identity_code_points(),
            0,
            canonical.max_extensions(),
            canonical.max_extension_namespace_code_points(),
            canonical.max_extension_media_type_code_points(),
            canonical.max_extension_payload_bytes_per_record(),
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::ResourceExtensionLimitExceeded
    );
    assert_eq!(
        rejection.error().location(),
        RawObservationLocation::ResourceExtension(0)
    );

    let extension =
        VersionedExtensionRef::new(String::from("observation"), 1, artifact(b"x", None));
    let observation = candidate(
        RunId::new(String::from("r")),
        stream(b""),
        stream(b""),
        empty_resources(),
        None,
        None,
        vec![extension],
    );
    let rejection = validate_raw_observation(
        observation,
        RawObservationLimits::new(
            canonical.outcome_limits(),
            canonical.max_identity_code_points(),
            canonical.max_resource_extensions(),
            0,
            canonical.max_extension_namespace_code_points(),
            canonical.max_extension_media_type_code_points(),
            canonical.max_extension_payload_bytes_per_record(),
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::ExtensionLimitExceeded
    );
    assert_eq!(
        rejection.error().location(),
        RawObservationLocation::Extension(0)
    );

    let raised = RawObservationLimits::new(
        canonical.outcome_limits(),
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
    );
    validate_raw_observation(minimal_observation(), raised).expect("absolute limits clamp");
}

#[test]
fn empty_identity_and_malformed_artifact_records_never_become_validated_observations() {
    let observation = candidate(
        RunId::new(String::new()),
        stream(b""),
        stream(b""),
        empty_resources(),
        None,
        None,
        Vec::new(),
    );
    let rejection =
        validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::EmptyIdentity
    );
    assert_eq!(rejection.error().location(), RawObservationLocation::RunId);

    let malformed = ArtifactRef {
        id: ArtifactId::new(String::from("not-an-artifact-id")),
        size_bytes: 0,
        media_type: None,
    };
    let observation = candidate(
        RunId::new(String::from("r")),
        stream(b""),
        CapturedStreamRef::new(malformed, false, 0, 0),
        empty_resources(),
        None,
        None,
        Vec::new(),
    );
    let rejection =
        validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationErrorKind::MalformedArtifactId
    );
    assert_eq!(rejection.error().location(), RawObservationLocation::Stderr);
}

#[test]
fn observation_error_and_location_tags_are_exact_version_one_mappings() {
    use RawObservationErrorKind::*;
    let errors = [
        EmptyIdentity,
        IdentityLimitExceeded,
        InvalidOutcome,
        RetainedByteCountMismatch,
        TruncationFlagMismatch,
        ArtifactPayloadLimitExceeded,
        MalformedArtifactId,
        UnsupportedArtifactAlgorithm,
        EmptyMediaType,
        ResourceExtensionLimitExceeded,
        ExtensionLimitExceeded,
        EmptyExtensionNamespace,
        ZeroExtensionSchemaVersion,
        ExtensionNamespaceLimitExceeded,
        ExtensionMediaTypeLimitExceeded,
        EmptyCoverageProvider,
        EmptyCoverageProviderVersion,
        EmptyCoverageTargetBuild,
        EmptyFeatureSetDigest,
        CoverageCountMismatch,
        EmptyStateNamespace,
        ZeroStateSchemaVersion,
        ZeroScheduleSchemaVersion,
        ZeroFaultSchemaVersion,
        FaultTraceCountMismatch,
        InvalidDuration,
        EmptyScheduleNamespace,
        EmptyFaultNamespace,
    ];
    for (index, kind) in errors.into_iter().enumerate() {
        assert_eq!(kind.stable_tag(), index as u16 + 1);
    }

    let locations = [
        RawObservationLocation::RunId,
        RawObservationLocation::AttemptId,
        RawObservationLocation::Outcome,
        RawObservationLocation::Stdout,
        RawObservationLocation::Stderr,
        RawObservationLocation::WallTime,
        RawObservationLocation::CpuTime,
        RawObservationLocation::Resources,
        RawObservationLocation::ResourceExtension(99),
        RawObservationLocation::Coverage,
        RawObservationLocation::StateDigest,
        RawObservationLocation::ScheduleTrace,
        RawObservationLocation::FaultTrace,
        RawObservationLocation::Extension(99),
    ];
    for (index, location) in locations.into_iter().enumerate() {
        assert_eq!(location.stable_tag(), index as u16 + 1);
    }
}
