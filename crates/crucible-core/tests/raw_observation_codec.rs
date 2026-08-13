use crucible_core::{
    canonical_raw_execution_outcome_limits, canonical_raw_observation_codec_limits,
    canonical_raw_observation_limits, decode_raw_observation, encode_raw_observation,
    validate_raw_execution_outcome, validate_raw_observation, ArtifactRef, CapturedStreamRef,
    CompletionDisposition, CoverageProviderId, CoverageRef, FaultTrace, RawExecutionEvent,
    RawExecutionOutcome, RawExecutionOutcomeErrorKind, RawExecutionOutcomeLimits,
    RawExecutionOutcomeLocation, RawObservation, RawObservationCodecErrorKind,
    RawObservationCodecLimits, RawObservationLimits, RawObservationLocation, RecordedDuration,
    ResourceSnapshot, RunAttemptId, RunId, ScheduleTrace, StateDigest, TargetBuildId,
    VersionedExtensionRef, MAX_RAW_OBSERVATION_ENCODED_BYTES,
};

fn artifact(contents: &[u8]) -> ArtifactRef {
    ArtifactRef::from_bytes(contents, None).unwrap()
}

fn media_artifact(contents: &[u8], media_type: &str) -> ArtifactRef {
    ArtifactRef::from_bytes(contents, Some(String::from(media_type))).unwrap()
}

const COMPLETE_V1_GOLDEN_HEX: &str = "43524f420001000000000000000772756e2de585a8000000000000000a617474656d70742dceb200000000000000114352584f00010003000000000000000000000000000000000600000000000000477368613235363a36336434326432363135366663633736316535376461343132386539383831643562646633626639333366306636653963393364366532366239623930616537000100000000000000060000000000000013000000000000000600000000000000477368613235363a37653662373130623736353430346363636261643965656463666637363135666333376232363964366462313263643831613538626535343164393330383363000000000000000000060000000000000000000000000000000c000001590100000000000000070000037a01000000000001000001000000000000000201000000000000000801000000000000000b01000000000000001101000000000000001701000000000000001d000000000000000100000000000000176f72672e6578616d706c652e7265736f757263652ecebb00000002000000000000001200000000000000477368613235363a35623731636438656363303738393636356533323636383866363931303335663262373030336561323962623631643362646332366130316133316363636662000100000000000000096c6c766d2e65646765000000000000000631392e312e3700000000000000096275696c642de585a800000000000000147368613235363a666561747572652d7370616365000000000000000800000000000000477368613235363a6333613330393162396433323236376430623331373565653134663730613165306233643732393264306130666134353032306365643566623736346436323000000000000000000500000000000000640100000000000000116f72672e6578616d706c652e737461746500000004000000000000000500000000000000477368613235363a34626136393733356361353337363565643661373039656462353663366561323336623731393361336232396136623339306333343666306634333430653465000100000000000000146f72672e6578616d706c652e7363686564756c6500000005000000000000000800000000000000477368613235363a3566373230303565383163346563626435616133353838653830653364616630646630363863653063663536333839343435303861303136356665613630396100000000000000001f010100000000000000116f72672e6578616d706c652e6661756c7400000006000000000000000500000000000000477368613235363a6631633536326561653332663963633261393764633964373638623839643162336639333761326139356538313265393462646330333361303735643761316600000000000000000a0000000000000004000000000000000200000000000000010000000000000003010000000000000001000000000000001b6f72672e6578616d706c652e6f62736572766174696f6e2ee585a800000003000000000000001500000000000000477368613235363a6532643365616161373961366265663066363464303430316339333336326531633431313130643735363033646331316233636437653331663839623438373600";

const COMPLETE_V1_MEDIA_OPTION_HEX: &str =
    "0100000000000000216170706c69636174696f6e2f766e642e6372756369626c652e636f766572616765";

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap();
            let low = (pair[1] as char).to_digit(16).unwrap();
            ((high << 4) | low) as u8
        })
        .collect()
}

fn validated_observation() -> crucible_core::ValidatedRawObservation {
    let outcome = validate_raw_execution_outcome(
        RawExecutionOutcome::new(CompletionDisposition::Completed, None, Vec::new()),
        canonical_raw_execution_outcome_limits(),
    )
    .unwrap();
    validate_raw_observation(
        RawObservation::new(
            RunId::new(String::from("r")),
            RunAttemptId::new(String::from("a")),
            outcome.into_inner(),
            CapturedStreamRef::new(artifact(b""), false, 0, 0),
            CapturedStreamRef::new(artifact(b""), false, 0, 0),
            RecordedDuration::new(0, 0).unwrap(),
            None,
            None,
            ResourceSnapshot::new(None, None, None, None, None, None, Vec::new()),
            None,
            None,
            None,
            None,
            Vec::new(),
        ),
        canonical_raw_observation_limits(),
    )
    .unwrap()
}

fn complete_validated_observation() -> crucible_core::ValidatedRawObservation {
    let outcome = validate_raw_execution_outcome(
        RawExecutionOutcome::new(CompletionDisposition::Incomplete, None, Vec::new()),
        canonical_raw_execution_outcome_limits(),
    )
    .unwrap()
    .into_inner();
    let resource_extension = VersionedExtensionRef::new(
        String::from("org.example.resource.λ"),
        2,
        artifact(b"resource extension"),
    );
    let extension = VersionedExtensionRef::new(
        String::from("org.example.observation.全"),
        3,
        artifact(b"observation extension"),
    );
    let observation = RawObservation::new(
        RunId::new(String::from("run-全")),
        RunAttemptId::new(String::from("attempt-β")),
        outcome,
        CapturedStreamRef::new(artifact(b"stdout"), true, 6, 19),
        CapturedStreamRef::new(artifact(b"stderr"), false, 6, 0),
        RecordedDuration::new(12, 345).unwrap(),
        Some(RecordedDuration::new(7, 890).unwrap()),
        Some(65_536),
        ResourceSnapshot::new(
            Some(2),
            Some(8),
            Some(11),
            Some(17),
            Some(23),
            Some(29),
            vec![resource_extension],
        ),
        Some(CoverageRef::new(
            CoverageProviderId::new(String::from("llvm.edge")),
            String::from("19.1.7"),
            TargetBuildId::new(String::from("build-全")),
            String::from("sha256:feature-space"),
            media_artifact(b"coverage", "application/vnd.crucible.coverage"),
            5,
            100,
        )),
        Some(StateDigest::new(
            String::from("org.example.state"),
            4,
            artifact(b"state"),
        )),
        Some(ScheduleTrace::new(
            String::from("org.example.schedule"),
            5,
            artifact(b"schedule"),
            31,
            true,
        )),
        Some(FaultTrace::new(
            String::from("org.example.fault"),
            6,
            artifact(b"fault"),
            10,
            4,
            2,
            1,
            3,
            true,
        )),
        vec![extension],
    );
    validate_raw_observation(observation, canonical_raw_observation_limits()).unwrap()
}

#[test]
fn observation_round_trips_through_canonical_bytes_and_has_exact_header() {
    let value = validated_observation();
    let encoded = encode_raw_observation(&value, MAX_RAW_OBSERVATION_ENCODED_BYTES).unwrap();
    assert_eq!(&encoded[..6], b"CROB\0\x01");
    assert_eq!(&encoded[6..14], &[0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(encoded[14], b'r');
    let decoded =
        decode_raw_observation(encoded.clone(), canonical_raw_observation_codec_limits()).unwrap();
    assert_eq!(decoded, value);
    let reencoded = encode_raw_observation(&decoded, MAX_RAW_OBSERVATION_ENCODED_BYTES).unwrap();
    assert_eq!(reencoded, encoded);
}

#[test]
fn every_optional_resource_and_extension_field_round_trips_without_information_loss() {
    let value = complete_validated_observation();
    let encoded = encode_raw_observation(&value, MAX_RAW_OBSERVATION_ENCODED_BYTES).unwrap();
    let mut golden = decode_hex(COMPLETE_V1_GOLDEN_HEX);
    golden.splice(657..658, decode_hex(COMPLETE_V1_MEDIA_OPTION_HEX));
    assert_eq!(encoded, golden);
    let decoded =
        decode_raw_observation(encoded.clone(), canonical_raw_observation_codec_limits()).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(
        encode_raw_observation(&decoded, MAX_RAW_OBSERVATION_ENCODED_BYTES).unwrap(),
        encoded
    );
}

#[test]
fn future_schema_input_cap_and_trailing_bytes_preserve_the_exact_rejected_input() {
    let encoded =
        encode_raw_observation(&validated_observation(), MAX_RAW_OBSERVATION_ENCODED_BYTES)
            .unwrap();

    let mut future = encoded.clone();
    future[4] = 0;
    future[5] = 2;
    let rejection =
        decode_raw_observation(future.clone(), canonical_raw_observation_codec_limits())
            .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::UnsupportedSchemaVersion
    );
    assert_eq!(rejection.encoded(), future.as_slice());

    let rejection = decode_raw_observation(
        encoded.clone(),
        RawObservationCodecLimits::new(5, canonical_raw_observation_limits()),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::EncodedByteLimitExceeded
    );
    assert_eq!(rejection.encoded(), encoded.as_slice());

    let mut trailing = encoded;
    trailing.push(0xff);
    let rejection =
        decode_raw_observation(trailing.clone(), canonical_raw_observation_codec_limits())
            .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::TrailingBytes
    );
    assert_eq!(rejection.encoded(), trailing.as_slice());
}

#[test]
fn truncation_invalid_utf8_and_nested_outcome_failures_have_exact_typed_offsets() {
    let encoded =
        encode_raw_observation(&validated_observation(), MAX_RAW_OBSERVATION_ENCODED_BYTES)
            .unwrap();
    let truncated = encoded[..13].to_vec();
    let rejection =
        decode_raw_observation(truncated.clone(), canonical_raw_observation_codec_limits())
            .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::Truncated
    );
    assert_eq!(rejection.error().byte_offset(), truncated.len() as u64);

    let mut invalid_utf8 = encoded.clone();
    invalid_utf8[14] = 0xff;
    let rejection =
        decode_raw_observation(invalid_utf8, canonical_raw_observation_codec_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::InvalidUtf8
    );
    assert_eq!(rejection.error().byte_offset(), 14);

    let nested_length_offset = 24usize;
    let nested_start = nested_length_offset + 8;
    let mut bad_nested = encoded;
    bad_nested[nested_start] = b'X';
    let rejection =
        decode_raw_observation(bad_nested, canonical_raw_observation_codec_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::NestedOutcomeRejected
    );
    assert_eq!(rejection.error().byte_offset(), nested_start as u64);
    assert_eq!(rejection.error().nested_error_tag(), Some(3));
}

#[test]
fn declared_counts_are_rejected_before_nested_record_allocation() {
    let mut encoded =
        encode_raw_observation(&validated_observation(), MAX_RAW_OBSERVATION_ENCODED_BYTES)
            .unwrap();
    assert_eq!(encoded.len(), 299);
    let resource_extension_count = 279usize;
    encoded[resource_extension_count..resource_extension_count + 8]
        .copy_from_slice(&u64::MAX.to_be_bytes());
    let rejection =
        decode_raw_observation(encoded, canonical_raw_observation_codec_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::DeclaredResourceExtensionLimitExceeded
    );
    assert_eq!(
        rejection.error().byte_offset(),
        resource_extension_count as u64
    );

    let mut encoded =
        encode_raw_observation(&validated_observation(), MAX_RAW_OBSERVATION_ENCODED_BYTES)
            .unwrap();
    let extension_count = 291usize;
    encoded[extension_count..extension_count + 8].copy_from_slice(&u64::MAX.to_be_bytes());
    let rejection =
        decode_raw_observation(encoded, canonical_raw_observation_codec_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::DeclaredExtensionLimitExceeded
    );
    assert_eq!(rejection.error().byte_offset(), extension_count as u64);
}

#[test]
fn codec_error_tags_are_an_exact_version_one_compatibility_mapping() {
    use RawObservationCodecErrorKind::*;
    let cases = [
        (EncodedByteLimitExceeded, 1),
        (Truncated, 2),
        (InvalidMagic, 3),
        (UnsupportedSchemaVersion, 4),
        (InvalidUtf8, 5),
        (StringLengthLimitExceeded, 6),
        (NestedOutcomeRejected, 7),
        (UnknownBooleanTag, 8),
        (InvalidDuration, 9),
        (DeclaredResourceExtensionLimitExceeded, 10),
        (DeclaredExtensionLimitExceeded, 11),
        (SemanticValidationFailed, 12),
        (TrailingBytes, 13),
        (InvalidOptionTag, 14),
    ];
    for (kind, tag) in cases {
        assert_eq!(kind.stable_tag(), tag);
    }
}

#[test]
fn semantic_media_rejection_preserves_exact_location_index_and_wire_offset() {
    let outcome = validate_raw_execution_outcome(
        RawExecutionOutcome::new(CompletionDisposition::Completed, None, Vec::new()),
        canonical_raw_execution_outcome_limits(),
    )
    .unwrap()
    .into_inner();
    let media = "application/vnd.example.unique-media";
    let observation = validate_raw_observation(
        RawObservation::new(
            RunId::new(String::from("r")),
            RunAttemptId::new(String::from("a")),
            outcome,
            CapturedStreamRef::new(
                ArtifactRef::from_bytes(b"", Some(String::from(media))).unwrap(),
                false,
                0,
                0,
            ),
            CapturedStreamRef::new(artifact(b""), false, 0, 0),
            RecordedDuration::new(0, 0).unwrap(),
            None,
            None,
            ResourceSnapshot::new(None, None, None, None, None, None, Vec::new()),
            None,
            None,
            None,
            None,
            Vec::new(),
        ),
        canonical_raw_observation_limits(),
    )
    .unwrap();
    let encoded = encode_raw_observation(&observation, MAX_RAW_OBSERVATION_ENCODED_BYTES).unwrap();
    let media_start = encoded
        .windows(media.len())
        .position(|window| window == media.as_bytes())
        .expect("unique media spelling");
    let media_length_offset = media_start - 8;
    let canonical = canonical_raw_observation_limits();
    let limits = RawObservationLimits::new(
        canonical.outcome_limits(),
        canonical.max_identity_code_points(),
        canonical.max_resource_extensions(),
        canonical.max_extensions(),
        canonical.max_extension_namespace_code_points(),
        0,
        canonical.max_extension_payload_bytes_per_record(),
    );
    let rejection = decode_raw_observation(
        encoded,
        RawObservationCodecLimits::new(MAX_RAW_OBSERVATION_ENCODED_BYTES, limits),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::SemanticValidationFailed
    );
    assert_eq!(rejection.error().byte_offset(), media_length_offset as u64);
    assert_eq!(
        rejection.error().semantic_error_kind(),
        Some(crucible_core::RawObservationErrorKind::ExtensionMediaTypeLimitExceeded)
    );
    assert_eq!(
        rejection.error().semantic_error_location(),
        Some(RawObservationLocation::Stdout)
    );
    assert_eq!(rejection.error().code_point_index(), Some(0));
}

#[test]
fn nested_semantic_rejection_preserves_the_full_typed_outcome_diagnostic() {
    let outcome = validate_raw_execution_outcome(
        RawExecutionOutcome::new(
            CompletionDisposition::Completed,
            None,
            vec![RawExecutionEvent::PlatformSpecific(
                VersionedExtensionRef::new(String::from("abcd"), 1, artifact(b"nested")),
            )],
        ),
        canonical_raw_execution_outcome_limits(),
    )
    .unwrap()
    .into_inner();
    let canonical = canonical_raw_observation_limits();
    let observation = validate_raw_observation(
        RawObservation::new(
            RunId::new(String::from("r")),
            RunAttemptId::new(String::from("a")),
            outcome,
            CapturedStreamRef::new(artifact(b""), false, 0, 0),
            CapturedStreamRef::new(artifact(b""), false, 0, 0),
            RecordedDuration::new(0, 0).unwrap(),
            None,
            None,
            ResourceSnapshot::new(None, None, None, None, None, None, Vec::new()),
            None,
            None,
            None,
            None,
            Vec::new(),
        ),
        canonical,
    )
    .unwrap();
    let encoded = encode_raw_observation(&observation, MAX_RAW_OBSERVATION_ENCODED_BYTES).unwrap();
    let lowered = RawObservationLimits::new(
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
    let rejection = decode_raw_observation(
        encoded,
        RawObservationCodecLimits::new(MAX_RAW_OBSERVATION_ENCODED_BYTES, lowered),
    )
    .unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::NestedOutcomeRejected
    );
    assert_eq!(rejection.error().nested_error_tag(), Some(16));
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
fn option_tags_are_distinct_from_boolean_spellings() {
    let mut encoded =
        encode_raw_observation(&validated_observation(), MAX_RAW_OBSERVATION_ENCODED_BYTES)
            .unwrap();
    let coverage_option_offset = 287usize;
    encoded[coverage_option_offset] = 2;
    let rejection =
        decode_raw_observation(encoded, canonical_raw_observation_codec_limits()).unwrap_err();
    assert_eq!(
        rejection.error().kind(),
        RawObservationCodecErrorKind::InvalidOptionTag
    );
    assert_eq!(
        rejection.error().byte_offset(),
        coverage_option_offset as u64
    );
}
