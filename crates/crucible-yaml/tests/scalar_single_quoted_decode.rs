use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    decode_profile1_single_quoted_scalar_content, scan_profile1_quoted_scalars,
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

fn text(content: &crucible_yaml::DecodedScalarContent) -> String {
    content
        .content()
        .iter()
        .map(|item| char::from_u32(item.code_point()).expect("verified Unicode scalar"))
        .collect()
}

fn limits(max_content_code_points: u64) -> ScalarDecodeLimits {
    ScalarDecodeLimits::new(max_content_code_points)
}

#[test]
fn direct_unicode_and_doubled_quotes_decode_with_exact_provenance() {
    let bytes = "q: 'β here''s 😀'\n".as_bytes();
    let (atoms, _, _, quoted) = scan(bytes);
    let decoded = decode_profile1_single_quoted_scalar_content(
        &atoms,
        &quoted,
        0,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("single-quoted content is bounded");

    assert_eq!(decoded.style(), DecodedScalarStyle::SingleQuoted);
    assert_eq!(text(&decoded), "β here's 😀");
    let doubled = decoded
        .content()
        .iter()
        .find(|item| item.origin() == DecodedContentOrigin::SingleQuoteDoubled)
        .expect("the doubled quote emits one quote");
    assert_eq!(doubled.code_point(), u32::from('\''));
    assert_eq!(doubled.source_atom_end() - doubled.source_atom_start(), 2);
    assert_eq!(
        &bytes[doubled.byte_start() as usize..doubled.byte_end() as usize],
        b"''"
    );
}

#[test]
fn multiline_flow_folding_trims_boundary_white_and_preserves_empty_lines() {
    let bytes = b"q: 'one  \n  two\n\n    three'\n";
    let (atoms, _, _, quoted) = scan(bytes);
    let decoded = decode_profile1_single_quoted_scalar_content(
        &atoms,
        &quoted,
        0,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("multiline single-quoted content is bounded");

    assert_eq!(text(&decoded), "one two\nthree");
    let folded: Vec<_> = decoded
        .content()
        .iter()
        .filter(|item| item.origin() == DecodedContentOrigin::FoldedLineBreak)
        .collect();
    assert_eq!(folded.len(), 2);
    assert_eq!(folded[0].code_point(), u32::from(' '));
    assert_eq!(folded[1].code_point(), u32::from('\n'));
    for item in folded {
        assert_eq!(item.source_atom_end() - item.source_atom_start(), 1);
        assert_eq!(
            atoms.atoms()[item.source_atom_start() as usize].code_point(),
            u32::from('\n')
        );
    }
}

#[test]
fn empty_content_limits_and_style_errors_are_exact() {
    let (atoms, _, _, quoted) = scan(b"['', 'a''b', \"double\"]\n");
    let empty = decode_profile1_single_quoted_scalar_content(&atoms, &quoted, 0, limits(0))
        .expect("empty content consumes no output budget");
    assert!(empty.content().is_empty());

    let error = decode_profile1_single_quoted_scalar_content(&atoms, &quoted, 1, limits(1))
        .expect_err("the doubled quote is the second output code point");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ContentLimitExceeded);
    let second = &quoted.scalars()[1];
    assert_eq!(error.byte_offset(), second.byte_start() + 2);

    let error = decode_profile1_single_quoted_scalar_content(&atoms, &quoted, 2, limits(32))
        .expect_err("the selected scalar is double-quoted");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ScalarStyleMismatch);
    assert_eq!(error.byte_offset(), quoted.scalars()[2].byte_start());

    let error = decode_profile1_single_quoted_scalar_content(&atoms, &quoted, 3, limits(32))
        .expect_err("there is no fourth quoted scalar");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ScalarIndexOutOfRange);
    assert_eq!(error.byte_offset(), atoms.source_len_bytes());
}
