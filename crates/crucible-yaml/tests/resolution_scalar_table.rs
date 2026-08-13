use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, compose_profile1_semantic_scalar_table, decode_profile1,
    parse_profile1_cst, scan_profile1_block_scalars, scan_profile1_completed_tokens,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy, CompletedTokenSource, CstSource,
    DecodeLimits, PlainScalarSource, QuotedScalarSource, ResolvedScalarTag, ResolvedScalarValue,
    SemanticScalarTableErrorKind, SemanticScalarTableLimits,
    MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS, MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    MAX_PROFILE1_CORE_INTEGER_LIMBS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS, MAX_PROFILE1_SEMANTIC_NODES, MAX_PROFILE1_SOURCE_BYTES,
    MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
};

fn parse(
    input: &[u8],
) -> (
    AtomizedSource,
    QuotedScalarSource,
    PlainScalarSource,
    BlockScalarSource,
    CompletedTokenSource,
    CstSource,
) {
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
    (atoms, quoted, plain, block, tokens, cst)
}

fn unlimited() -> SemanticScalarTableLimits {
    SemanticScalarTableLimits::new(
        MAX_PROFILE1_SEMANTIC_NODES,
        MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
        MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
        MAX_PROFILE1_CORE_INTEGER_LIMBS,
        MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
        MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    )
}

fn decoded_text(scalar: &crucible_yaml::ResolvedScalar) -> String {
    scalar
        .presentation()
        .decoded()
        .expect("nonempty scalar presentation")
        .content()
        .iter()
        .map(|point| char::from_u32(point.code_point()).expect("decoded Unicode scalar"))
        .collect()
}

#[test]
fn scalar_table_retains_exact_cst_indices_values_and_total_content() {
    let input = b"--- null\n--- true\n--- 0x2a\n--- 1.50\n--- \"hello\"\n--- |\n  block\n--- []\n";
    let (atoms, quoted, plain, block, tokens, cst) = parse(input);
    let table = compose_profile1_semantic_scalar_table(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        unlimited(),
    )
    .expect("every scalar node composes in CST order");

    assert_eq!(table.input_node_count(), cst.nodes().len() as u64);
    assert_eq!(table.scalars().len(), 6);
    assert_eq!(table.total_content_code_points(), 4 + 4 + 4 + 4 + 5 + 6);

    assert_eq!(table.scalars()[0].tag(), ResolvedScalarTag::CoreNull);
    assert!(matches!(
        table.scalars()[0].value(),
        ResolvedScalarValue::Null
    ));
    assert_eq!(table.scalars()[1].tag(), ResolvedScalarTag::CoreBoolean);
    assert!(matches!(
        table.scalars()[1].value(),
        ResolvedScalarValue::Boolean(true)
    ));
    assert_eq!(table.scalars()[2].tag(), ResolvedScalarTag::CoreInteger);
    match table.scalars()[2].value() {
        ResolvedScalarValue::Integer(value) => {
            assert!(!value.negative());
            assert_eq!(value.limbs(), &[42]);
        }
        other => panic!("expected canonical integer, got {other:?}"),
    }
    assert_eq!(table.scalars()[3].tag(), ResolvedScalarTag::CoreFloat);
    match table.scalars()[3].value() {
        ResolvedScalarValue::FiniteFloat(value) => {
            assert!(!value.negative());
            assert_eq!(value.coefficient_digits_le(), &[5, 1]);
            assert!(value.exponent_negative());
            assert_eq!(value.exponent_digits_le(), &[1]);
        }
        other => panic!("expected canonical finite float, got {other:?}"),
    }
    for (scalar, expected_text) in table.scalars()[4..].iter().zip(["hello", "block\n"]) {
        assert_eq!(scalar.tag(), ResolvedScalarTag::CoreString);
        assert!(matches!(scalar.value(), ResolvedScalarValue::String));
        assert_eq!(decoded_text(scalar), expected_text);
    }
    for pair in table.scalars().windows(2) {
        assert!(pair[0].node_index() < pair[1].node_index());
    }
    for scalar in table.scalars() {
        assert_eq!(
            cst.nodes()[scalar.node_index() as usize].kind(),
            crucible_yaml::CstNodeKind::Scalar
        );
    }
}

#[test]
fn record_and_aggregate_caps_fail_at_the_exact_first_excluded_scalar() {
    let (atoms, quoted, plain, block, tokens, cst) = parse(b"[one, two]\n");
    let first_scalar_node = cst
        .nodes()
        .iter()
        .position(|node| node.kind() == crucible_yaml::CstNodeKind::Scalar)
        .expect("fixture has a scalar");
    let second_scalar_node = cst
        .nodes()
        .iter()
        .enumerate()
        .skip(first_scalar_node + 1)
        .find(|(_, node)| node.kind() == crucible_yaml::CstNodeKind::Scalar)
        .map(|(index, _)| index)
        .expect("fixture has a second scalar");

    let error = compose_profile1_semantic_scalar_table(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        SemanticScalarTableLimits::new(
            1,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ),
    )
    .expect_err("the second scalar record exceeds a one-record cap");
    assert_eq!(
        error.kind(),
        SemanticScalarTableErrorKind::ScalarLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        cst.nodes()[second_scalar_node].byte_start()
    );

    let error = compose_profile1_semantic_scalar_table(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        SemanticScalarTableLimits::new(
            u64::MAX,
            4,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ),
    )
    .expect_err("the second code point of the second scalar exceeds the aggregate cap");
    assert_eq!(
        error.kind(),
        SemanticScalarTableErrorKind::TotalContentLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        cst.nodes()[second_scalar_node].byte_start() + 1
    );
}

#[test]
fn cross_source_authentication_precedes_scalar_table_population() {
    let (atoms, quoted, plain, block, tokens, cst) = parse(b"value\n");
    let (other_atoms, _, _, _, _, _) = parse(b"different\n");
    let error = compose_profile1_semantic_scalar_table(
        &other_atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        unlimited(),
    )
    .expect_err("atom and producer identities must remain authenticated");
    assert_eq!(
        error.kind(),
        SemanticScalarTableErrorKind::ScalarValue(
            crucible_yaml::ScalarValueErrorKind::ScalarDecode(
                crucible_yaml::CstScalarDecodeErrorKind::InputCompletedTokenMismatch,
            ),
        )
    );

    let table = compose_profile1_semantic_scalar_table(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        unlimited(),
    )
    .expect("matching sources remain valid");
    assert_eq!(table.scalars().len(), 1);
}
