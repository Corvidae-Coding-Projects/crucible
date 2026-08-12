use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    decode_profile1_double_quoted_scalar_content, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource, BomPolicy, DecodeLimits,
    DecodedContentOrigin, DecodedScalarStyle, LayoutSource, QuotedScalarScanLimits,
    QuotedScalarSource, ScalarDecodeErrorKind, ScalarDecodeLimits, StructuralLexemeSource,
    StructuralScanLimits, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_QUOTED_SCALAR_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
    MAX_PROFILE1_STRUCTURAL_LEXEMES,
};

fn scan(
    input: &[u8],
) -> (
    AtomizedSource,
    LayoutSource,
    StructuralLexemeSource,
    QuotedScalarSource,
) {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 bytes");
    let atoms = atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS))
        .expect("bounded atoms");
    let lines = analyze_profile1_layout(&atoms, canonical_structural_layout_limits())
        .expect("canonical layout");
    let candidates = scan_profile1_structural_lexemes(
        &atoms,
        &lines,
        StructuralScanLimits::new(MAX_PROFILE1_STRUCTURAL_LEXEMES),
    )
    .expect("canonical structural candidates");
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(
            MAX_PROFILE1_QUOTED_SCALARS,
            MAX_PROFILE1_QUOTED_SCALAR_ATOMS,
        ),
    )
    .expect("canonical quoted scalars");
    (atoms, lines, candidates, quoted)
}

fn limits(max_content_code_points: u64) -> ScalarDecodeLimits {
    ScalarDecodeLimits::new(max_content_code_points)
}

fn code_points(content: &crucible_yaml::DecodedScalarContent) -> Vec<u32> {
    content
        .content()
        .iter()
        .map(crucible_yaml::DecodedContentScalar::code_point)
        .collect()
}

#[test]
fn every_simple_and_hex_escape_decodes_with_exact_provenance() {
    let bytes = b"q: \"\\0\\a\\b\\t\\\t\\n\\v\\f\\r\\e\\ \\\"\\/\\\\\\N\\_\\L\\P\\x41\\u03B2\\U0001F600\"\n";
    let (atoms, _, _, quoted) = scan(bytes);
    let decoded = decode_profile1_double_quoted_scalar_content(
        &atoms,
        &quoted,
        0,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("every YAML 1.2.2 double-quoted escape decodes");

    assert_eq!(decoded.style(), DecodedScalarStyle::DoubleQuoted);
    assert_eq!(
        code_points(&decoded),
        vec![
            0x00, 0x07, 0x08, 0x09, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x1b, 0x20, 0x22, 0x2f, 0x5c,
            0x85, 0xa0, 0x2028, 0x2029, 0x41, 0x03b2, 0x1f600,
        ]
    );
    let expected_atom_widths = [
        2u64, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 6, 10,
    ];
    for (item, expected_width) in decoded.content().iter().zip(expected_atom_widths) {
        assert_eq!(item.origin(), DecodedContentOrigin::DoubleQuotedEscape);
        assert_eq!(
            item.source_atom_end() - item.source_atom_start(),
            expected_width
        );
        assert_eq!(
            atoms.atoms()[item.source_atom_start() as usize].code_point(),
            u32::from(b'\\')
        );
        assert_eq!(bytes[item.byte_start() as usize], b'\\');
    }
}

#[test]
fn ordinary_and_escaped_line_breaks_have_distinct_exact_semantics() {
    let (atoms, _, _, quoted) = scan(b"q: \"folded  \n  next\n\n    blank\"\n");
    let folded = decode_profile1_double_quoted_scalar_content(
        &atoms,
        &quoted,
        0,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("ordinary double-quoted line breaks fold like other flow scalars");
    assert_eq!(
        code_points(&folded),
        "folded next\nblank"
            .chars()
            .map(u32::from)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        folded
            .content()
            .iter()
            .filter(|item| item.origin() == DecodedContentOrigin::FoldedLineBreak)
            .count(),
        2
    );

    let bytes = b"q: \"kept \t\\\n  next\\\n\n  after\"\n";
    let (atoms, _, _, quoted) = scan(bytes);
    let escaped = decode_profile1_double_quoted_scalar_content(
        &atoms,
        &quoted,
        0,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("escaped breaks discard the first break and retain empty-line breaks");
    assert_eq!(
        code_points(&escaped),
        "kept \tnext\nafter"
            .chars()
            .map(u32::from)
            .collect::<Vec<_>>()
    );
    let derived: Vec<_> = escaped
        .content()
        .iter()
        .filter(|item| item.origin() == DecodedContentOrigin::EscapedLineBreak)
        .collect();
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].code_point(), u32::from('\n'));
    assert_eq!(
        derived[0].source_atom_end() - derived[0].source_atom_start(),
        1
    );
    assert_eq!(bytes[derived[0].byte_start() as usize], b'\n');
    let derived_index = escaped
        .content()
        .iter()
        .position(|item| item.origin() == DecodedContentOrigin::EscapedLineBreak)
        .expect("one escaped-break-derived line feed");
    let error = decode_profile1_double_quoted_scalar_content(
        &atoms,
        &quoted,
        0,
        limits(derived_index as u64),
    )
    .expect_err("the derived line feed is the first excluded output record");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ContentLimitExceeded);
    assert_eq!(error.byte_offset(), derived[0].byte_start());
}

#[test]
fn empty_content_limits_style_and_indices_are_exact() {
    let (atoms, _, _, quoted) = scan(b"[\"\", \"\\u03B2\", 'single']\n");
    let empty = decode_profile1_double_quoted_scalar_content(&atoms, &quoted, 0, limits(0))
        .expect("empty content consumes no output budget");
    assert!(empty.content().is_empty());

    let error = decode_profile1_double_quoted_scalar_content(&atoms, &quoted, 1, limits(0))
        .expect_err("the escape is the first excluded output code point");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ContentLimitExceeded);
    assert_eq!(error.byte_offset(), quoted.scalars()[1].byte_start() + 1);

    let error = decode_profile1_double_quoted_scalar_content(&atoms, &quoted, 2, limits(32))
        .expect_err("the selected scalar is single-quoted");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ScalarStyleMismatch);
    assert_eq!(error.byte_offset(), quoted.scalars()[2].byte_start());

    let error = decode_profile1_double_quoted_scalar_content(&atoms, &quoted, 3, limits(32))
        .expect_err("there is no fourth quoted scalar");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ScalarIndexOutOfRange);
    assert_eq!(error.byte_offset(), atoms.source_len_bytes());
}
