use crucible_core::{
    canonical_raw_execution_outcome_codec_limits, canonical_raw_execution_outcome_limits,
    decode_raw_execution_outcome, encode_raw_execution_outcome, validate_raw_execution_outcome,
    ArtifactRef, CompletionDisposition, LogicalProcessId, RawExecutionEvent, RawExecutionOutcome,
    RawExecutionOutcomeCodecErrorKind, RawExecutionOutcomeCodecLimits, ResourceKind,
    TerminationRecord, VersionedExtensionRef, MAX_RAW_EXECUTION_EVENTS,
    MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS, MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
};

fn artifact(contents: &[u8], media_type: Option<&str>) -> ArtifactRef {
    ArtifactRef::from_bytes(contents, media_type.map(String::from)).unwrap()
}

fn validated(outcome: RawExecutionOutcome) -> crucible_core::ValidatedRawExecutionOutcome {
    validate_raw_execution_outcome(
        outcome,
        crucible_core::canonical_raw_execution_outcome_limits(),
    )
    .unwrap()
}

#[test]
fn empty_outcome_has_exact_canonical_golden_bytes() {
    let value = validated(RawExecutionOutcome::new(
        CompletionDisposition::Completed,
        None,
        Vec::new(),
    ));
    let encoded =
        encode_raw_execution_outcome(&value, MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES).unwrap();
    assert_eq!(
        encoded,
        vec![
            b'C', b'R', b'X', b'O', 0, 1, // magic and schema version
            0, 1, // CompletionDisposition::Completed
            0, 0, 0, 0, 0, 0, 0, 0, // zero events
            0, // no termination
        ]
    );

    let decoded =
        decode_raw_execution_outcome(encoded, canonical_raw_execution_outcome_codec_limits())
            .unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn complex_outcome_round_trips_through_real_bytes_without_information_loss() {
    let process = LogicalProcessId::new(9).unwrap();
    let value = validated(RawExecutionOutcome::new(
        CompletionDisposition::Cancelled,
        Some(TerminationRecord::UnixSignal {
            signal: 15,
            core_dumped: false,
        }),
        vec![
            RawExecutionEvent::TimeoutThresholdReached,
            RawExecutionEvent::ResourceThresholdReached {
                resource: ResourceKind::Memory,
            },
            RawExecutionEvent::DeadlockSuspected,
            RawExecutionEvent::LivelockSuspected,
            RawExecutionEvent::WatchdogTriggered,
            RawExecutionEvent::ProcessCreated {
                logical_process: process,
            },
            RawExecutionEvent::PlatformSpecific(VersionedExtensionRef::new(
                String::from("org.example.β"),
                7,
                artifact(b"extension-payload", Some("application/example+β")),
            )),
            RawExecutionEvent::ProcessExited {
                logical_process: process,
            },
        ],
    ));
    let encoded =
        encode_raw_execution_outcome(&value, MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES).unwrap();
    let decoded =
        decode_raw_execution_outcome(encoded, canonical_raw_execution_outcome_codec_limits())
            .unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn every_termination_variant_round_trips_with_its_payload() {
    let terminations = vec![
        TerminationRecord::ExitCode { code: -9 },
        TerminationRecord::UnixSignal {
            signal: 11,
            core_dumped: true,
        },
        TerminationRecord::WindowsException {
            status: 0xc000_0005,
        },
        TerminationRecord::EmbeddedReset {
            cause: crucible_core::ResetCause::Brownout,
        },
        TerminationRecord::HarnessTerminated {
            reason: crucible_core::HarnessTerminationReason::CleanupFailure,
        },
        TerminationRecord::PlatformSpecific(VersionedExtensionRef::new(
            String::from("org.example.termination"),
            44,
            artifact(b"termination-extension", Some("application/x-termination")),
        )),
    ];
    for termination in terminations {
        let value = validated(RawExecutionOutcome::new(
            CompletionDisposition::Incomplete,
            Some(termination),
            Vec::new(),
        ));
        let encoded =
            encode_raw_execution_outcome(&value, MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES).unwrap();
        let decoded =
            decode_raw_execution_outcome(encoded, canonical_raw_execution_outcome_codec_limits())
                .unwrap();
        assert_eq!(decoded, value);
    }
}

fn empty_golden() -> Vec<u8> {
    vec![
        b'C', b'R', b'X', b'O', 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

fn one_event_prefix() -> Vec<u8> {
    let mut bytes = empty_golden();
    bytes[15] = 1;
    bytes
}

#[test]
fn malformed_encodings_fail_with_exact_typed_offsets_and_preserve_input() {
    let mut cases = Vec::new();
    cases.push((Vec::new(), RawExecutionOutcomeCodecErrorKind::Truncated, 0));
    let mut bad_magic = empty_golden();
    bad_magic[0] = b'X';
    cases.push((
        bad_magic,
        RawExecutionOutcomeCodecErrorKind::InvalidMagic,
        0,
    ));
    let mut bad_completion = empty_golden();
    bad_completion[7] = 99;
    cases.push((
        bad_completion,
        RawExecutionOutcomeCodecErrorKind::UnknownCompletionTag,
        6,
    ));
    let mut bad_option = empty_golden();
    bad_option[16] = 2;
    cases.push((
        bad_option,
        RawExecutionOutcomeCodecErrorKind::InvalidOptionTag,
        16,
    ));
    let mut trailing = empty_golden();
    trailing.push(0);
    cases.push((
        trailing,
        RawExecutionOutcomeCodecErrorKind::TrailingBytes,
        17,
    ));

    for (encoded, kind, byte_offset) in cases {
        let expected = encoded.clone();
        let rejection =
            decode_raw_execution_outcome(encoded, canonical_raw_execution_outcome_codec_limits())
                .unwrap_err();
        assert_eq!(rejection.error().kind(), kind);
        assert_eq!(rejection.error().byte_offset(), byte_offset);
        assert_eq!(rejection.encoded(), expected.as_slice());
        assert_eq!(rejection.into_encoded(), expected);
    }
}

#[test]
fn future_schema_and_input_cap_rejections_preserve_the_exact_opaque_bytes() {
    let value = validated(RawExecutionOutcome::new(
        CompletionDisposition::Completed,
        None,
        Vec::new(),
    ));
    let encode_error = encode_raw_execution_outcome(&value, 16).unwrap_err();
    assert_eq!(
        encode_error.kind(),
        RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded
    );
    assert_eq!(encode_error.byte_offset(), 16);

    let mut future = empty_golden();
    future[5] = 2;
    let expected = future.clone();
    let rejection =
        decode_raw_execution_outcome(future, canonical_raw_execution_outcome_codec_limits())
            .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeCodecErrorKind::UnsupportedSchemaVersion
    );
    assert_eq!(rejection.into_encoded(), expected);

    let oversized = empty_golden();
    let limits = RawExecutionOutcomeCodecLimits::new(
        (oversized.len() - 1) as u64,
        canonical_raw_execution_outcome_limits(),
    );
    let rejection = decode_raw_execution_outcome(oversized.clone(), limits).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded
    );
    assert_eq!(rejection.into_encoded(), oversized);
}

#[test]
fn malformed_nested_tags_utf8_and_boole_have_exact_offsets() {
    let mut unknown_event = one_event_prefix();
    unknown_event.extend_from_slice(&[0, 99]);

    let mut unknown_resource = one_event_prefix();
    unknown_resource.extend_from_slice(&[0, 2, 0, 99]);

    let mut invalid_utf8 = one_event_prefix();
    invalid_utf8.extend_from_slice(&[0, 8]);
    invalid_utf8.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    invalid_utf8.push(0xff);

    let mut invalid_bool = empty_golden();
    invalid_bool[16] = 1;
    invalid_bool.extend_from_slice(&[0, 2, 0, 0, 0, 1, 2]);

    let cases = vec![
        (
            unknown_event,
            RawExecutionOutcomeCodecErrorKind::UnknownEventTag,
            17,
        ),
        (
            unknown_resource,
            RawExecutionOutcomeCodecErrorKind::UnknownResourceKindTag,
            19,
        ),
        (
            invalid_utf8,
            RawExecutionOutcomeCodecErrorKind::InvalidUtf8,
            27,
        ),
        (
            invalid_bool,
            RawExecutionOutcomeCodecErrorKind::InvalidBoolean,
            23,
        ),
    ];
    for (encoded, expected_kind, expected_offset) in cases {
        let rejection =
            decode_raw_execution_outcome(encoded, canonical_raw_execution_outcome_codec_limits())
                .unwrap_err();
        assert_eq!(rejection.error().kind(), expected_kind);
        assert_eq!(rejection.error().byte_offset(), expected_offset);
    }
}

#[test]
fn declared_caps_are_checked_before_nested_allocation_or_semantics() {
    let mut over_event_cap_and_bad_option = one_event_prefix();
    over_event_cap_and_bad_option[16] = 2;
    let limits = RawExecutionOutcomeCodecLimits::new(
        MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
        crucible_core::RawExecutionOutcomeLimits::new(
            0,
            MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            7,
        ),
    );
    let rejection =
        decode_raw_execution_outcome(over_event_cap_and_bad_option, limits).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeCodecErrorKind::DeclaredEventLimitExceeded
    );
    assert_eq!(rejection.error().byte_offset(), 8);

    let mut over_namespace_cap = one_event_prefix();
    over_namespace_cap.extend_from_slice(&[0, 8]);
    over_namespace_cap.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 5]);
    let rejection = decode_raw_execution_outcome(
        over_namespace_cap,
        RawExecutionOutcomeCodecLimits::new(
            MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
            crucible_core::RawExecutionOutcomeLimits::new(
                MAX_RAW_EXECUTION_EVENTS,
                1,
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
                7,
            ),
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded
    );
    assert_eq!(rejection.error().byte_offset(), 19);
    assert_eq!(rejection.error().code_point_index(), Some(1));

    let value = validated(RawExecutionOutcome::new(
        CompletionDisposition::Completed,
        None,
        vec![RawExecutionEvent::PlatformSpecific(
            VersionedExtensionRef::new(
                String::from("x"),
                1,
                ArtifactRef {
                    id: artifact(b"payload-eight", None).id,
                    size_bytes: 8,
                    media_type: None,
                },
            ),
        )],
    ));
    let encoded =
        encode_raw_execution_outcome(&value, MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES).unwrap();
    let rejection = decode_raw_execution_outcome(
        encoded,
        RawExecutionOutcomeCodecLimits::new(
            MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
            crucible_core::RawExecutionOutcomeLimits::new(
                MAX_RAW_EXECUTION_EVENTS,
                MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
                7,
            ),
        ),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawExecutionOutcomeCodecErrorKind::DeclaredPayloadLimitExceeded
    );
    assert_eq!(rejection.error().event_index(), Some(0));
}

#[test]
fn codec_error_tags_are_the_exact_version_one_mapping() {
    let tags = [
        RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
        RawExecutionOutcomeCodecErrorKind::Truncated,
        RawExecutionOutcomeCodecErrorKind::InvalidMagic,
        RawExecutionOutcomeCodecErrorKind::UnsupportedSchemaVersion,
        RawExecutionOutcomeCodecErrorKind::UnknownCompletionTag,
        RawExecutionOutcomeCodecErrorKind::InvalidOptionTag,
        RawExecutionOutcomeCodecErrorKind::UnknownTerminationTag,
        RawExecutionOutcomeCodecErrorKind::UnknownResetCauseTag,
        RawExecutionOutcomeCodecErrorKind::UnknownHarnessTerminationReasonTag,
        RawExecutionOutcomeCodecErrorKind::UnknownEventTag,
        RawExecutionOutcomeCodecErrorKind::UnknownResourceKindTag,
        RawExecutionOutcomeCodecErrorKind::InvalidBoolean,
        RawExecutionOutcomeCodecErrorKind::InvalidUtf8,
        RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded,
        RawExecutionOutcomeCodecErrorKind::DeclaredEventLimitExceeded,
        RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded,
        RawExecutionOutcomeCodecErrorKind::DeclaredMediaTypeLimitExceeded,
        RawExecutionOutcomeCodecErrorKind::DeclaredPayloadLimitExceeded,
        RawExecutionOutcomeCodecErrorKind::InvalidLogicalProcessId,
        RawExecutionOutcomeCodecErrorKind::TrailingBytes,
        RawExecutionOutcomeCodecErrorKind::SemanticValidationFailed,
    ];
    for (index, kind) in tags.into_iter().enumerate() {
        assert_eq!(kind.stable_tag(), index as u16 + 1);
    }
}
