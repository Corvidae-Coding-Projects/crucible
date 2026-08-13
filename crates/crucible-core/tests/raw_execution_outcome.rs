use crucible_core::{
    canonical_raw_execution_outcome_limits, validate_raw_execution_outcome, ArtifactId,
    ArtifactRef, CompletionDisposition, HarnessTerminationReason, LogicalProcessId,
    LogicalProcessIdError, RawExecutionEvent, RawExecutionOutcome, RawExecutionOutcomeErrorKind,
    RawExecutionOutcomeLimits, RawExecutionOutcomeLocation, ResourceKind, TerminationRecord,
    VersionedExtensionRef, MAX_RAW_EXECUTION_EVENTS,
    MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
};

fn artifact(contents: &[u8]) -> ArtifactRef {
    ArtifactRef::from_bytes(contents, Some(String::from("application/octet-stream"))).unwrap()
}

fn extension(namespace: &str) -> VersionedExtensionRef {
    VersionedExtensionRef::new(String::from(namespace), 1, artifact(b"extension"))
}

fn assert_exact_stable_tags(tags: &[u16]) {
    for (index, tag) in tags.iter().enumerate() {
        assert_eq!(*tag, index as u16 + 1);
    }
}

#[test]
fn complete_portable_outcome_validates_without_coercing_any_fact() {
    let process = LogicalProcessId::new(7).unwrap();
    let outcome = RawExecutionOutcome::new(
        CompletionDisposition::Completed,
        Some(TerminationRecord::ExitCode { code: 0 }),
        vec![
            RawExecutionEvent::ProcessCreated {
                logical_process: process,
            },
            RawExecutionEvent::TimeoutThresholdReached,
            RawExecutionEvent::ResourceThresholdReached {
                resource: ResourceKind::Memory,
            },
            RawExecutionEvent::DeadlockSuspected,
            RawExecutionEvent::LivelockSuspected,
            RawExecutionEvent::WatchdogTriggered,
            RawExecutionEvent::ProcessExited {
                logical_process: process,
            },
            RawExecutionEvent::PlatformSpecific(extension("org.example.runtime-event")),
        ],
    );

    let validated =
        validate_raw_execution_outcome(outcome, canonical_raw_execution_outcome_limits()).unwrap();
    assert_eq!(validated.outcome().events().len(), 8);
    assert_eq!(
        validated.outcome().completion(),
        CompletionDisposition::Completed
    );

    assert_eq!(
        validated.into_inner().termination(),
        &Some(TerminationRecord::ExitCode { code: 0 })
    );
}

#[test]
fn completion_termination_and_detected_conditions_remain_independent_facts() {
    let outcome = RawExecutionOutcome::new(
        CompletionDisposition::Cancelled,
        Some(TerminationRecord::HarnessTerminated {
            reason: HarnessTerminationReason::Cancellation,
        }),
        vec![
            RawExecutionEvent::TimeoutThresholdReached,
            RawExecutionEvent::ResourceThresholdReached {
                resource: ResourceKind::WallTime,
            },
        ],
    );
    let validated =
        validate_raw_execution_outcome(outcome, canonical_raw_execution_outcome_limits()).unwrap();
    assert_eq!(validated.outcome().events().len(), 2);
    assert_eq!(
        validated.outcome().termination(),
        &Some(TerminationRecord::HarnessTerminated {
            reason: HarnessTerminationReason::Cancellation,
        })
    );
}

#[test]
fn every_platform_termination_form_is_representable_without_unix_coercion() {
    let terminations = vec![
        TerminationRecord::ExitCode { code: -7 },
        TerminationRecord::UnixSignal {
            signal: 9,
            core_dumped: true,
        },
        TerminationRecord::WindowsException {
            status: 0xc000_0005,
        },
        TerminationRecord::EmbeddedReset {
            cause: crucible_core::ResetCause::Watchdog,
        },
        TerminationRecord::HarnessTerminated {
            reason: HarnessTerminationReason::MemoryLimit,
        },
        TerminationRecord::PlatformSpecific(extension("org.example.termination")),
    ];

    for termination in terminations {
        let outcome = RawExecutionOutcome::new(
            CompletionDisposition::Incomplete,
            Some(termination),
            Vec::new(),
        );
        assert!(
            validate_raw_execution_outcome(outcome, canonical_raw_execution_outcome_limits(),)
                .is_ok()
        );
    }
}

#[test]
fn event_and_extension_limits_are_exact_independent_and_cannot_be_raised() {
    assert_eq!(MAX_RAW_EXECUTION_EVENTS, 1_048_576);
    assert_eq!(MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS, 1_048_576);
    let process = LogicalProcessId::new(1).unwrap();
    let events = vec![
        RawExecutionEvent::ProcessCreated {
            logical_process: process,
        },
        RawExecutionEvent::ProcessExited {
            logical_process: process,
        },
    ];
    let error = validate_raw_execution_outcome(
        RawExecutionOutcome::new(CompletionDisposition::Completed, None, events),
        RawExecutionOutcomeLimits::new(
            1,
            MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.error().kind(),
        RawExecutionOutcomeErrorKind::EventLimitExceeded
    );
    assert_eq!(
        error.error().location(),
        RawExecutionOutcomeLocation::Event(1)
    );

    let error = validate_raw_execution_outcome(
        RawExecutionOutcome::new(
            CompletionDisposition::Completed,
            None,
            vec![RawExecutionEvent::PlatformSpecific(extension("abcd"))],
        ),
        RawExecutionOutcomeLimits::new(
            MAX_RAW_EXECUTION_EVENTS,
            3,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.error().kind(),
        RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded
    );
    assert_eq!(
        error.error().location(),
        RawExecutionOutcomeLocation::Event(0)
    );
    assert_eq!(error.error().extension_code_point_index(), Some(3));

    let raised = RawExecutionOutcomeLimits::new(u64::MAX, u64::MAX, u64::MAX, u64::MAX);
    assert!(validate_raw_execution_outcome(
        RawExecutionOutcome::new(
            CompletionDisposition::Completed,
            None,
            vec![RawExecutionEvent::PlatformSpecific(extension("abcd"))],
        ),
        raised,
    )
    .is_ok());
}

#[test]
fn invalid_extensions_return_the_exact_unconsumed_outcome_with_typed_location() {
    let malformed = VersionedExtensionRef::new(String::new(), 0, artifact(b"bad-extension"));
    let original = RawExecutionOutcome::new(
        CompletionDisposition::Incomplete,
        Some(TerminationRecord::PlatformSpecific(malformed)),
        vec![RawExecutionEvent::DeadlockSuspected],
    );
    let rejection =
        validate_raw_execution_outcome(original, canonical_raw_execution_outcome_limits())
            .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeErrorKind::EmptyExtensionNamespace
    );
    assert_eq!(
        rejection.error().location(),
        RawExecutionOutcomeLocation::Termination
    );
    let (error, returned) = rejection.into_parts();
    assert_eq!(
        error.kind(),
        RawExecutionOutcomeErrorKind::EmptyExtensionNamespace
    );
    assert_eq!(returned.events().len(), 1);
    assert!(matches!(
        returned.termination(),
        Some(TerminationRecord::PlatformSpecific(extension))
            if extension.schema_version() == 0 && extension.namespace().is_empty()
    ));
}

#[test]
fn extension_and_signal_validation_errors_are_distinct_and_have_stable_precedence() {
    let cases = vec![
        (
            VersionedExtensionRef::new(String::from("org.example.zero"), 0, artifact(b"zero")),
            RawExecutionOutcomeErrorKind::ZeroExtensionSchemaVersion,
        ),
        (
            VersionedExtensionRef::new(
                String::from("org.example.malformed"),
                1,
                ArtifactRef {
                    id: ArtifactId::new(String::from("malformed")),
                    size_bytes: 0,
                    media_type: Some(String::from("application/octet-stream")),
                },
            ),
            RawExecutionOutcomeErrorKind::MalformedExtensionArtifact,
        ),
        (
            VersionedExtensionRef::new(
                String::from("org.example.algorithm"),
                1,
                ArtifactRef {
                    id: ArtifactId::new(String::from("blake3:abcd")),
                    size_bytes: 0,
                    media_type: Some(String::from("application/octet-stream")),
                },
            ),
            RawExecutionOutcomeErrorKind::UnsupportedExtensionArtifactAlgorithm,
        ),
        (
            VersionedExtensionRef::new(
                String::from("org.example.media"),
                1,
                ArtifactRef {
                    id: artifact(b"empty-media").id,
                    size_bytes: b"empty-media".len() as u64,
                    media_type: Some(String::new()),
                },
            ),
            RawExecutionOutcomeErrorKind::EmptyExtensionMediaType,
        ),
    ];

    for (extension, expected) in cases {
        let rejection = validate_raw_execution_outcome(
            RawExecutionOutcome::new(
                CompletionDisposition::Incomplete,
                None,
                vec![RawExecutionEvent::PlatformSpecific(extension)],
            ),
            canonical_raw_execution_outcome_limits(),
        )
        .unwrap_err();
        assert_eq!(rejection.error().kind(), expected);
        assert_eq!(
            rejection.error().location(),
            RawExecutionOutcomeLocation::Event(0)
        );
    }

    for signal in [0, -1] {
        let rejection = validate_raw_execution_outcome(
            RawExecutionOutcome::new(
                CompletionDisposition::Incomplete,
                Some(TerminationRecord::UnixSignal {
                    signal,
                    core_dumped: false,
                }),
                Vec::new(),
            ),
            canonical_raw_execution_outcome_limits(),
        )
        .unwrap_err();
        assert_eq!(
            rejection.error().kind(),
            RawExecutionOutcomeErrorKind::InvalidUnixSignal
        );
        assert_eq!(
            rejection.error().location(),
            RawExecutionOutcomeLocation::Termination
        );
    }
}

#[test]
fn namespace_accounting_is_aggregate_across_termination_and_events() {
    let rejection = validate_raw_execution_outcome(
        RawExecutionOutcome::new(
            CompletionDisposition::Incomplete,
            Some(TerminationRecord::PlatformSpecific(extension("ab"))),
            vec![RawExecutionEvent::PlatformSpecific(extension("cdef"))],
        ),
        RawExecutionOutcomeLimits::new(
            MAX_RAW_EXECUTION_EVENTS,
            5,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded
    );
    assert_eq!(
        rejection.error().location(),
        RawExecutionOutcomeLocation::Event(0)
    );
    assert_eq!(rejection.error().extension_code_point_index(), Some(3));
}

#[test]
fn event_count_preflight_has_stable_precedence_over_nested_validation() {
    let rejection = validate_raw_execution_outcome(
        RawExecutionOutcome::new(
            CompletionDisposition::Incomplete,
            Some(TerminationRecord::UnixSignal {
                signal: 0,
                core_dumped: false,
            }),
            vec![RawExecutionEvent::DeadlockSuspected],
        ),
        RawExecutionOutcomeLimits::new(
            0,
            MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeErrorKind::EventLimitExceeded
    );
    assert_eq!(
        rejection.error().location(),
        RawExecutionOutcomeLocation::Event(0)
    );
}

#[test]
fn zero_process_ids_fail_with_a_stable_typed_error() {
    assert_eq!(LogicalProcessId::new(0), Err(LogicalProcessIdError::Zero));
}

#[test]
fn extension_payload_and_inline_media_limits_are_exact_and_typed() {
    assert_eq!(
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        1_048_576
    );
    assert_eq!(
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        1_099_511_627_776
    );

    let payload_limit = 7;
    for size_bytes in [payload_limit, payload_limit + 1] {
        let payload = ArtifactRef {
            id: artifact(b"payload-limit").id,
            size_bytes,
            media_type: None,
        };
        let result = validate_raw_execution_outcome(
            RawExecutionOutcome::new(
                CompletionDisposition::Completed,
                None,
                vec![RawExecutionEvent::PlatformSpecific(
                    VersionedExtensionRef::new(String::from("org.example.payload"), 1, payload),
                )],
            ),
            RawExecutionOutcomeLimits::new(
                MAX_RAW_EXECUTION_EVENTS,
                MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
                payload_limit,
            ),
        );
        if size_bytes == payload_limit {
            assert!(result.is_ok());
        } else {
            let rejection = result.unwrap_err();
            assert_eq!(
                rejection.error().kind(),
                RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded
            );
            assert_eq!(
                rejection.error().location(),
                RawExecutionOutcomeLocation::Event(0)
            );
        }
    }

    let media_payload = |media_type: &str| ArtifactRef {
        id: artifact(b"media-limit").id,
        size_bytes: 0,
        media_type: Some(String::from(media_type)),
    };
    let rejection = validate_raw_execution_outcome(
        RawExecutionOutcome::new(
            CompletionDisposition::Completed,
            Some(TerminationRecord::PlatformSpecific(
                VersionedExtensionRef::new(String::from("a"), 1, media_payload("β")),
            )),
            vec![RawExecutionEvent::PlatformSpecific(
                VersionedExtensionRef::new(String::from("b"), 1, media_payload("cd")),
            )],
        ),
        RawExecutionOutcomeLimits::new(
            MAX_RAW_EXECUTION_EVENTS,
            MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            2,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded
    );
    assert_eq!(
        rejection.error().location(),
        RawExecutionOutcomeLocation::Event(0)
    );
    assert_eq!(rejection.error().extension_code_point_index(), Some(1));
}

#[test]
fn stable_tags_are_exact_across_each_versioned_enum_vocabulary() {
    assert_exact_stable_tags(&[
        CompletionDisposition::Completed.stable_tag(),
        CompletionDisposition::Cancelled.stable_tag(),
        CompletionDisposition::Incomplete.stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        crucible_core::ResetCause::PowerOn.stable_tag(),
        crucible_core::ResetCause::Watchdog.stable_tag(),
        crucible_core::ResetCause::Software.stable_tag(),
        crucible_core::ResetCause::Brownout.stable_tag(),
        crucible_core::ResetCause::External.stable_tag(),
        crucible_core::ResetCause::Unknown.stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        HarnessTerminationReason::Timeout.stable_tag(),
        HarnessTerminationReason::Cancellation.stable_tag(),
        HarnessTerminationReason::CpuTimeLimit.stable_tag(),
        HarnessTerminationReason::MemoryLimit.stable_tag(),
        HarnessTerminationReason::ProcessCountLimit.stable_tag(),
        HarnessTerminationReason::FileSizeLimit.stable_tag(),
        HarnessTerminationReason::OutputLimit.stable_tag(),
        HarnessTerminationReason::CleanupFailure.stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        ResourceKind::WallTime.stable_tag(),
        ResourceKind::CpuTime.stable_tag(),
        ResourceKind::Memory.stable_tag(),
        ResourceKind::ProcessCount.stable_tag(),
        ResourceKind::FileSize.stable_tag(),
        ResourceKind::StandardOutput.stable_tag(),
        ResourceKind::StandardError.stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        TerminationRecord::ExitCode { code: 0 }.stable_tag(),
        TerminationRecord::UnixSignal {
            signal: 9,
            core_dumped: false,
        }
        .stable_tag(),
        TerminationRecord::WindowsException { status: 0 }.stable_tag(),
        TerminationRecord::EmbeddedReset {
            cause: crucible_core::ResetCause::Unknown,
        }
        .stable_tag(),
        TerminationRecord::HarnessTerminated {
            reason: HarnessTerminationReason::Timeout,
        }
        .stable_tag(),
        TerminationRecord::PlatformSpecific(extension("org.example.termination-tag")).stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        RawExecutionEvent::TimeoutThresholdReached.stable_tag(),
        RawExecutionEvent::ResourceThresholdReached {
            resource: ResourceKind::Memory,
        }
        .stable_tag(),
        RawExecutionEvent::DeadlockSuspected.stable_tag(),
        RawExecutionEvent::LivelockSuspected.stable_tag(),
        RawExecutionEvent::WatchdogTriggered.stable_tag(),
        RawExecutionEvent::ProcessCreated {
            logical_process: LogicalProcessId::new(1).unwrap(),
        }
        .stable_tag(),
        RawExecutionEvent::ProcessExited {
            logical_process: LogicalProcessId::new(1).unwrap(),
        }
        .stable_tag(),
        RawExecutionEvent::PlatformSpecific(extension("org.example.event-tag")).stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        RawExecutionOutcomeLocation::Termination.stable_tag(),
        RawExecutionOutcomeLocation::Event(0).stable_tag(),
    ]);
    assert_exact_stable_tags(&[
        RawExecutionOutcomeErrorKind::EventLimitExceeded.stable_tag(),
        RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded.stable_tag(),
        RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded.stable_tag(),
        RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded.stable_tag(),
        RawExecutionOutcomeErrorKind::EmptyExtensionNamespace.stable_tag(),
        RawExecutionOutcomeErrorKind::ZeroExtensionSchemaVersion.stable_tag(),
        RawExecutionOutcomeErrorKind::MalformedExtensionArtifact.stable_tag(),
        RawExecutionOutcomeErrorKind::UnsupportedExtensionArtifactAlgorithm.stable_tag(),
        RawExecutionOutcomeErrorKind::EmptyExtensionMediaType.stable_tag(),
        RawExecutionOutcomeErrorKind::InvalidUnixSignal.stable_tag(),
        RawExecutionOutcomeErrorKind::InvalidLogicalProcessId.stable_tag(),
    ]);
    assert_eq!(CompletionDisposition::Completed.stable_tag(), 1);
    assert_eq!(CompletionDisposition::Cancelled.stable_tag(), 2);
    assert_eq!(CompletionDisposition::Incomplete.stable_tag(), 3);
    assert_eq!(
        RawExecutionOutcomeErrorKind::EventLimitExceeded.stable_tag(),
        1
    );
    assert_eq!(
        RawExecutionOutcomeErrorKind::InvalidLogicalProcessId.stable_tag(),
        11
    );
    assert_eq!(LogicalProcessIdError::Zero.stable_tag(), 1);
}
