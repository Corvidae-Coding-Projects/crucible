use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, parse_profile1_cst,
    resolve_profile1_cst_node_scalar_value, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy,
    CompletedTokenSource, CstSource, DecodeLimits, PlainScalarSource, QuotedScalarSource,
    ResolvedScalarTag, ResolvedScalarValue, ScalarValueErrorKind, ScalarValueLimits,
    MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS, MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    MAX_PROFILE1_CORE_INTEGER_LIMBS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS, MAX_PROFILE1_SOURCE_BYTES,
};

struct Parsed {
    atoms: AtomizedSource,
    quoted: QuotedScalarSource,
    plain: PlainScalarSource,
    block: BlockScalarSource,
    tokens: CompletedTokenSource,
    cst: CstSource,
}

fn parse(input: &[u8]) -> Parsed {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 bytes");
    let atoms = atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS))
        .expect("bounded atom source");
    let layout = analyze_profile1_layout(&atoms, canonical_structural_layout_limits())
        .expect("canonical layout");
    let structural =
        scan_profile1_structural_lexemes(&atoms, &layout, canonical_structural_scan_limits())
            .expect("canonical structural candidates");
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &layout,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .expect("canonical quoted scalars");
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .expect("canonical plain scalars");
    let block = scan_profile1_block_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    )
    .expect("canonical block scalars");
    let tokens = scan_profile1_completed_tokens(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    )
    .expect("canonical completed tokens");
    let cst = parse_profile1_cst(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        &tokens,
        canonical_cst_limits(),
    )
    .expect("canonical CST");
    Parsed {
        atoms,
        quoted,
        plain,
        block,
        tokens,
        cst,
    }
}

fn unlimited() -> ScalarValueLimits {
    ScalarValueLimits::new(
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
        MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
        MAX_PROFILE1_CORE_INTEGER_LIMBS,
        MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
        MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    )
}

fn resolve_root(input: &[u8]) -> crucible_yaml::ResolvedScalar {
    let parsed = parse(input);
    let root = parsed.cst.documents()[0].root_node_index();
    resolve_profile1_cst_node_scalar_value(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        root,
        unlimited(),
    )
    .expect("valid scalar resolves")
    .expect("root is a scalar")
}

fn explicit_tag_text(value: &crucible_yaml::ResolvedScalar) -> Option<String> {
    value.explicit_tag().map(|tag| {
        tag.content()
            .iter()
            .map(|point| char::from_u32(point.code_point()).expect("Unicode tag code point"))
            .collect()
    })
}

#[test]
fn implicit_core_and_non_plain_scalar_resolution_is_exact() {
    assert!(matches!(
        resolve_root(b"null\n").value(),
        ResolvedScalarValue::Null
    ));
    assert!(matches!(
        resolve_root(b"TRUE\n").value(),
        ResolvedScalarValue::Boolean(true)
    ));

    let integer = resolve_root(b"0x2A\n");
    assert_eq!(integer.tag(), ResolvedScalarTag::CoreInteger);
    match integer.value() {
        ResolvedScalarValue::Integer(value) => {
            assert!(!value.negative());
            assert_eq!(value.limbs(), &[42]);
        }
        other => panic!("expected integer, got {other:?}"),
    }

    let finite = resolve_root(b"1.20e1\n");
    match finite.value() {
        ResolvedScalarValue::FiniteFloat(value) => {
            assert_eq!(value.coefficient_digits_le(), &[2, 1]);
            assert!(!value.exponent_negative());
            assert_eq!(value.exponent_digits_le(), &[0]);
        }
        other => panic!("expected finite float, got {other:?}"),
    }
    assert!(matches!(
        resolve_root(b"-.Inf\n").value(),
        ResolvedScalarValue::NegativeInfinity
    ));
    assert!(matches!(
        resolve_root(b".NaN\n").value(),
        ResolvedScalarValue::NotANumber
    ));

    for input in [&b"yes\n"[..], &b"\"true\"\n"[..], &b"|\n  null\n"[..]] {
        let value = resolve_root(input);
        assert_eq!(value.tag(), ResolvedScalarTag::CoreString);
        assert!(matches!(value.value(), ResolvedScalarValue::String));
    }
    assert!(matches!(
        resolve_root(b"---\n").value(),
        ResolvedScalarValue::Null
    ));
}

#[test]
fn explicit_standard_non_specific_and_custom_tags_are_preserved_and_enforced() {
    let integer = resolve_root(b"!!int \"42\"\n");
    assert_eq!(integer.tag(), ResolvedScalarTag::CoreInteger);
    assert_eq!(
        explicit_tag_text(&integer).as_deref(),
        Some("tag:yaml.org,2002:int")
    );
    assert!(matches!(integer.value(), ResolvedScalarValue::Integer(_)));

    assert!(matches!(
        resolve_root(b"!!null \"~\"\n").value(),
        ResolvedScalarValue::Null
    ));
    assert!(matches!(
        resolve_root(b"!!bool 'false'\n").value(),
        ResolvedScalarValue::Boolean(false)
    ));
    assert!(matches!(
        resolve_root(b"!!float \"1.50\"\n").value(),
        ResolvedScalarValue::FiniteFloat(_)
    ));
    assert!(matches!(
        resolve_root(b"!!float '.Inf'\n").value(),
        ResolvedScalarValue::PositiveInfinity
    ));

    let forced_string = resolve_root(b"! true\n");
    assert_eq!(forced_string.tag(), ResolvedScalarTag::CoreString);
    assert_eq!(explicit_tag_text(&forced_string).as_deref(), Some("!"));
    assert!(matches!(forced_string.value(), ResolvedScalarValue::String));

    let custom = resolve_root(b"!thing 42\n");
    assert_eq!(custom.tag(), ResolvedScalarTag::CustomLocal);
    assert_eq!(explicit_tag_text(&custom).as_deref(), Some("!thing"));
    assert!(matches!(custom.value(), ResolvedScalarValue::String));

    let custom_global = resolve_root(b"!<tag:example.com,2026:scalar> value\n");
    assert_eq!(custom_global.tag(), ResolvedScalarTag::CustomGlobal);
    assert_eq!(
        explicit_tag_text(&custom_global).as_deref(),
        Some("tag:example.com,2026:scalar")
    );

    let empty_string = resolve_root(b"!!str\n");
    assert_eq!(empty_string.tag(), ResolvedScalarTag::CoreString);
    assert!(matches!(empty_string.value(), ResolvedScalarValue::String));
    assert!(empty_string.presentation().decoded().is_none());
}

#[test]
fn explicit_tag_value_compatibility_and_nested_limits_have_exact_errors() {
    for (input, expected) in [
        (
            &b"!!int nope\n"[..],
            ScalarValueErrorKind::InvalidExplicitScalarTagValue,
        ),
        (
            &b"!!seq scalar\n"[..],
            ScalarValueErrorKind::ScalarTagKindMismatch,
        ),
        (
            &b"!!map scalar\n"[..],
            ScalarValueErrorKind::ScalarTagKindMismatch,
        ),
        (
            &b"!!bool yes\n"[..],
            ScalarValueErrorKind::InvalidExplicitScalarTagValue,
        ),
    ] {
        let parsed = parse(input);
        let root = parsed.cst.documents()[0].root_node_index();
        let error = resolve_profile1_cst_node_scalar_value(
            &parsed.atoms,
            &parsed.quoted,
            &parsed.plain,
            &parsed.block,
            &parsed.tokens,
            &parsed.cst,
            root,
            unlimited(),
        )
        .expect_err("invalid explicit scalar tag use is rejected");
        assert_eq!(error.kind(), expected);
        assert!(error.byte_offset() < input.len() as u64);
    }

    let input = b"1000000000\n";
    let parsed = parse(input);
    let root = parsed.cst.documents()[0].root_node_index();
    let error = resolve_profile1_cst_node_scalar_value(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        root,
        ScalarValueLimits::new(
            MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
            MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
            1,
            MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
            MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
        ),
    )
    .expect_err("the second canonical integer limb is caller-excluded");
    assert_eq!(
        error.kind(),
        ScalarValueErrorKind::IntegerMagnitudeLimitExceeded
    );
    assert_eq!(error.byte_offset(), 9);

    let parsed = parse(b"\"four\"\n");
    let root = parsed.cst.documents()[0].root_node_index();
    let error = resolve_profile1_cst_node_scalar_value(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        root,
        ScalarValueLimits::new(
            3,
            MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
            MAX_PROFILE1_CORE_INTEGER_LIMBS,
            MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
            MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
        ),
    )
    .expect_err("the fourth decoded code point is excluded");
    assert_eq!(
        error.kind(),
        ScalarValueErrorKind::ScalarDecode(
            crucible_yaml::CstScalarDecodeErrorKind::ContentLimitExceeded
        )
    );
    assert_eq!(error.byte_offset(), 4);

    let parsed = parse(b"!long value\n");
    let root = parsed.cst.documents()[0].root_node_index();
    let error = resolve_profile1_cst_node_scalar_value(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        root,
        ScalarValueLimits::new(
            MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
            1,
            MAX_PROFILE1_CORE_INTEGER_LIMBS,
            MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
            MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
        ),
    )
    .expect_err("the second resolved tag code point is excluded");
    assert_eq!(
        error.kind(),
        ScalarValueErrorKind::TagResolution(
            crucible_yaml::TagResolutionErrorKind::TagCodePointLimitExceeded
        )
    );
}
