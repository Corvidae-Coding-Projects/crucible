use crucible_yaml::{
    decode_profile1, BomPolicy, DecodeErrorKind, DecodeLimits, DecodedScalar, DecodedSource,
    SourcePosition, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_SOURCE_BYTES,
};

fn limits(source_bytes: u64, scalars: u64) -> DecodeLimits {
    DecodeLimits::new(source_bytes, scalars)
}

fn decode(input: &[u8]) -> DecodedSource {
    decode_profile1(
        input,
        limits(1_048_576, 1_048_576),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 UTF-8")
}

fn point(position: SourcePosition) -> (u64, u64, u64) {
    (position.byte_offset(), position.line(), position.column())
}

fn scalar(scalar: &DecodedScalar) -> (u32, (u64, u64, u64), (u64, u64, u64)) {
    (
        scalar.code_point(),
        point(scalar.span().start()),
        point(scalar.span().end()),
    )
}

#[test]
fn ascii_and_multibyte_scalars_retain_exact_original_byte_spans() {
    let source = decode("aé😀z".as_bytes());
    assert_eq!(source.profile_version(), 1);
    assert_eq!(source.transformation_version(), 1);
    assert_eq!(source.source_len_bytes(), 8);
    assert_eq!(source.bom_bytes(), 0);
    assert_eq!(source.scalars().len(), 4);
    assert_eq!(scalar(&source.scalars()[0]), (0x61, (0, 0, 0), (1, 0, 1)));
    assert_eq!(scalar(&source.scalars()[1]), (0xe9, (1, 0, 1), (3, 0, 2)));
    assert_eq!(
        scalar(&source.scalars()[2]),
        (0x1f600, (3, 0, 2), (7, 0, 3))
    );
    assert_eq!(scalar(&source.scalars()[3]), (0x7a, (7, 0, 3), (8, 0, 4)));
}

#[test]
fn leading_bom_policy_is_explicit_and_nonleading_bom_is_data() {
    let with_bom = decode(&[0xef, 0xbb, 0xbf, b'x']);
    assert_eq!(with_bom.bom_bytes(), 3);
    assert_eq!(with_bom.source_len_bytes(), 4);
    assert_eq!(scalar(&with_bom.scalars()[0]), (0x78, (3, 0, 0), (4, 0, 1)));

    let only_bom = decode(&[0xef, 0xbb, 0xbf]);
    assert!(only_bom.scalars().is_empty());

    let forbidden = decode_profile1(&[0xef, 0xbb, 0xbf, b'x'], limits(16, 16), BomPolicy::Forbid)
        .expect_err("forbidden BOM");
    assert_eq!(forbidden.kind(), DecodeErrorKind::ForbiddenByteOrderMark);
    assert_eq!(forbidden.byte_offset(), 0);

    let nonleading = decode(&[b'x', 0xef, 0xbb, 0xbf]);
    assert_eq!(nonleading.scalars()[1].code_point(), 0xfeff);
}

#[test]
fn line_endings_are_normalized_without_losing_source_spans() {
    let source = decode(b"a\r\nb\rc\n");
    let actual: Vec<_> = source.scalars().iter().map(scalar).collect();
    assert_eq!(
        actual,
        vec![
            (0x61, (0, 0, 0), (1, 0, 1)),
            (0x0a, (1, 0, 1), (3, 1, 0)),
            (0x62, (3, 1, 0), (4, 1, 1)),
            (0x0a, (4, 1, 1), (5, 2, 0)),
            (0x63, (5, 2, 0), (6, 2, 1)),
            (0x0a, (6, 2, 1), (7, 3, 0)),
        ]
    );
}

#[test]
fn malformed_utf8_has_stable_typed_diagnostics() {
    let cases: &[(&[u8], DecodeErrorKind, u64)] = &[
        (&[0x80], DecodeErrorKind::UnexpectedContinuationByte, 0),
        (&[0xc0, 0x80], DecodeErrorKind::OverlongEncoding, 0),
        (&[0xe2, 0x82], DecodeErrorKind::TruncatedSequence, 2),
        (
            &[0xe2, 0x28, 0xa1],
            DecodeErrorKind::InvalidContinuationByte,
            1,
        ),
        (&[0xe0, 0x9f, 0x80], DecodeErrorKind::OverlongEncoding, 0),
        (&[0xed, 0xa0, 0x80], DecodeErrorKind::SurrogateCodePoint, 0),
        (
            &[0xf0, 0x8f, 0x80, 0x80],
            DecodeErrorKind::OverlongEncoding,
            0,
        ),
        (
            &[0xf4, 0x90, 0x80, 0x80],
            DecodeErrorKind::CodePointOutOfRange,
            0,
        ),
        (&[0xf8], DecodeErrorKind::InvalidLeadingByte, 0),
        (&[0xc2], DecodeErrorKind::TruncatedSequence, 1),
        (&[0xe2, 0x28], DecodeErrorKind::InvalidContinuationByte, 1),
        (&[0xe0, 0x9f], DecodeErrorKind::OverlongEncoding, 0),
        (&[0xed, 0xa0], DecodeErrorKind::SurrogateCodePoint, 0),
        (&[0xf4, 0x90], DecodeErrorKind::CodePointOutOfRange, 0),
        (&[0xf0, 0x90, 0x80], DecodeErrorKind::TruncatedSequence, 3),
        (
            &[0xe1, 0x80, 0x7f],
            DecodeErrorKind::InvalidContinuationByte,
            2,
        ),
        (
            &[0xf1, 0x80, 0x80, 0x7f],
            DecodeErrorKind::InvalidContinuationByte,
            3,
        ),
        (
            &[0xf5, 0x80, 0x80, 0x80],
            DecodeErrorKind::CodePointOutOfRange,
            0,
        ),
        (&[0xff], DecodeErrorKind::InvalidLeadingByte, 0),
        (&[0xef, 0xbb], DecodeErrorKind::TruncatedSequence, 2),
    ];
    for (input, expected_kind, expected_offset) in cases {
        let error = decode_profile1(input, limits(64, 64), BomPolicy::AllowAndStrip)
            .expect_err("malformed UTF-8");
        assert_eq!(&error.kind(), expected_kind, "input {input:?}");
        assert_eq!(error.byte_offset(), *expected_offset, "input {input:?}");
    }
}

#[test]
fn every_utf8_width_and_scalar_boundary_decodes_canonically() {
    let bytes = [
        0x7f, 0xc2, 0x80, 0xdf, 0xbf, 0xe0, 0xa0, 0x80, 0xed, 0x9f, 0xbf, 0xee, 0x80, 0x80, 0xef,
        0xbf, 0xbf, 0xf0, 0x90, 0x80, 0x80, 0xf4, 0x8f, 0xbf, 0xbf,
    ];
    let source = decode(&bytes);
    let code_points: Vec<_> = source
        .scalars()
        .iter()
        .map(DecodedScalar::code_point)
        .collect();
    assert_eq!(
        code_points,
        vec![0x7f, 0x80, 0x7ff, 0x800, 0xd7ff, 0xe000, 0xffff, 0x10000, 0x10ffff]
    );
    let mut expected_start = 0;
    for decoded in source.scalars() {
        assert_eq!(decoded.span().start().byte_offset(), expected_start);
        assert!(decoded.span().end().byte_offset() > expected_start);
        expected_start = decoded.span().end().byte_offset();
    }
    assert_eq!(expected_start, bytes.len() as u64);
}

#[test]
fn resource_limits_reject_before_unbounded_decoding() {
    let source_error = decode_profile1(b"abc", limits(2, 3), BomPolicy::AllowAndStrip)
        .expect_err("source-byte limit");
    assert_eq!(
        source_error.kind(),
        DecodeErrorKind::SourceByteLimitExceeded
    );
    assert_eq!(source_error.byte_offset(), 2);

    let scalar_error = decode_profile1(b"abc", limits(3, 2), BomPolicy::AllowAndStrip)
        .expect_err("decoded-scalar limit");
    assert_eq!(
        scalar_error.kind(),
        DecodeErrorKind::DecodedScalarLimitExceeded
    );
    assert_eq!(scalar_error.byte_offset(), 2);

    let empty = decode_profile1(b"", limits(0, 0), BomPolicy::AllowAndStrip)
        .expect("empty input fits zero limits");
    assert!(empty.scalars().is_empty());
}

#[test]
fn cross_stage_diagnostic_precedence_is_stable() {
    let source_before_bom =
        decode_profile1(&[0xef, 0xbb, 0xbf, b'x'], limits(2, 0), BomPolicy::Forbid)
            .expect_err("source cap precedes BOM policy");
    assert_eq!(
        source_before_bom.kind(),
        DecodeErrorKind::SourceByteLimitExceeded
    );
    assert_eq!(source_before_bom.byte_offset(), 2);

    let bom_before_scalar =
        decode_profile1(&[0xef, 0xbb, 0xbf, b'x'], limits(4, 0), BomPolicy::Forbid)
            .expect_err("BOM policy precedes scalar cap");
    assert_eq!(
        bom_before_scalar.kind(),
        DecodeErrorKind::ForbiddenByteOrderMark
    );
    assert_eq!(bom_before_scalar.byte_offset(), 0);

    let scalar_before_utf8 = decode_profile1(&[0x80], limits(1, 0), BomPolicy::AllowAndStrip)
        .expect_err("scalar cap precedes decoding the next malformed byte");
    assert_eq!(
        scalar_before_utf8.kind(),
        DecodeErrorKind::DecodedScalarLimitExceeded
    );
    assert_eq!(scalar_before_utf8.byte_offset(), 0);
}

#[test]
fn profile_caps_apply_even_when_a_caller_requests_unbounded_limits() {
    assert_eq!(MAX_PROFILE1_SOURCE_BYTES, 16 * 1024 * 1024);
    assert_eq!(MAX_PROFILE1_DECODED_SCALARS, 1024 * 1024);
    let oversized = vec![b'a'; MAX_PROFILE1_SOURCE_BYTES as usize + 1];
    let error = decode_profile1(
        &oversized,
        limits(u64::MAX, u64::MAX),
        BomPolicy::AllowAndStrip,
    )
    .expect_err("profile source cap");
    assert_eq!(error.kind(), DecodeErrorKind::SourceByteLimitExceeded);
    assert_eq!(error.byte_offset(), MAX_PROFILE1_SOURCE_BYTES);

    let at_scalar_cap = vec![b'a'; MAX_PROFILE1_DECODED_SCALARS as usize];
    let decoded = decode_profile1(
        &at_scalar_cap,
        limits(u64::MAX, u64::MAX),
        BomPolicy::AllowAndStrip,
    )
    .expect("the exact scalar cap is accepted");
    assert_eq!(decoded.scalars().len() as u64, MAX_PROFILE1_DECODED_SCALARS);
    drop(decoded);

    let above_scalar_cap = vec![b'a'; MAX_PROFILE1_DECODED_SCALARS as usize + 1];
    let error = decode_profile1(
        &above_scalar_cap,
        limits(u64::MAX, u64::MAX),
        BomPolicy::AllowAndStrip,
    )
    .expect_err("profile scalar cap");
    assert_eq!(error.kind(), DecodeErrorKind::DecodedScalarLimitExceeded);
    assert_eq!(error.byte_offset(), MAX_PROFILE1_DECODED_SCALARS);
}

#[test]
fn acceptance_matches_the_independent_standard_library_oracle_for_all_two_byte_inputs() {
    for first in u8::MIN..=u8::MAX {
        for second in u8::MIN..=u8::MAX {
            let input = [first, second];
            let standard_accepts = std::str::from_utf8(&input).is_ok();
            let crucible_accepts =
                decode_profile1(&input, limits(2, 2), BomPolicy::AllowAndStrip).is_ok();
            assert_eq!(
                crucible_accepts, standard_accepts,
                "two-byte input {input:02x?}"
            );
        }
    }
}

#[test]
fn every_unicode_scalar_matches_an_independent_encoder_and_span_model() {
    const CHUNK_SCALARS: usize = 65_536;
    let mut bytes = Vec::new();
    let mut expected = Vec::new();

    let check_chunk = |bytes: &[u8], expected: &[(u32, u64, u64, u64, u64, u64, u64)]| {
        let decoded = decode_profile1(
            bytes,
            limits(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
            BomPolicy::AllowAndStrip,
        )
        .expect("standard-library-encoded Unicode scalars");
        assert_eq!(decoded.scalars().len(), expected.len());
        for (actual, expected) in decoded.scalars().iter().zip(expected) {
            assert_eq!(
                scalar(actual),
                (
                    expected.0,
                    (expected.1, expected.2, expected.3),
                    (expected.4, expected.5, expected.6),
                )
            );
        }
    };

    let mut byte_offset = 0u64;
    let mut line = 0u64;
    let mut column = 0u64;
    for code_point in 0..=0x10ffff {
        let Some(character) = char::from_u32(code_point) else {
            continue;
        };
        let mut encoded = [0u8; 4];
        let encoded = character.encode_utf8(&mut encoded).as_bytes();
        let start = (byte_offset, line, column);
        bytes.extend_from_slice(encoded);
        byte_offset += encoded.len() as u64;
        let normalized = if code_point == 0x0d { 0x0a } else { code_point };
        if normalized == 0x0a {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
        expected.push((
            normalized,
            start.0,
            start.1,
            start.2,
            byte_offset,
            line,
            column,
        ));

        if expected.len() == CHUNK_SCALARS {
            check_chunk(&bytes, &expected);
            bytes.clear();
            expected.clear();
            byte_offset = 0;
            line = 0;
            column = 0;
        }
    }
    if !expected.is_empty() {
        check_chunk(&bytes, &expected);
    }
}

#[test]
fn decoding_is_deterministic_and_single_byte_inputs_are_bounded() {
    let input = b"alpha\r\nbeta: \xf0\x9f\x98\x80\n";
    assert_eq!(
        decode_profile1(input, limits(128, 128), BomPolicy::AllowAndStrip),
        decode_profile1(input, limits(128, 128), BomPolicy::AllowAndStrip)
    );

    for byte in u8::MIN..=u8::MAX {
        let result = decode_profile1(&[byte], limits(1, 1), BomPolicy::AllowAndStrip);
        if byte <= 0x7f {
            assert!(result.is_ok(), "ASCII byte {byte:#x}");
        } else {
            let error = result.expect_err("an isolated non-ASCII byte is incomplete or invalid");
            let expected_offset = if (0xc2..=0xf4).contains(&byte) { 1 } else { 0 };
            assert_eq!(error.byte_offset(), expected_offset, "byte {byte:#x}");
        }
    }
}
