use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    decode_profile1_block_scalar_content, scan_profile1_block_scalars, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AtomizeLimits,
    BlockScalarContentOrigin, BlockScalarScanLimits, BlockScalarSource, BomPolicy, DecodeLimits,
    DecodedContentOrigin, DecodedScalarStyle, PlainScalarScanLimits, QuotedScalarScanLimits,
    ScalarDecodeErrorKind, ScalarDecodeLimits, StructuralScanLimits, MAX_PROFILE1_BLOCK_SCALARS,
    MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS,
    MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_QUOTED_SCALAR_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
    MAX_PROFILE1_STRUCTURAL_LEXEMES, MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
};

fn scan(input: &[u8]) -> BlockScalarSource {
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
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &quoted,
        PlainScalarScanLimits::new(MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS),
    )
    .expect("canonical plain scalars");
    scan_profile1_block_scalars(
        &atoms,
        &lines,
        &candidates,
        &quoted,
        &plain,
        BlockScalarScanLimits::new(
            MAX_PROFILE1_BLOCK_SCALARS,
            MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
            MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS,
            MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
        ),
    )
    .expect("canonical block scalars")
}

fn limits(max_content_code_points: u64) -> ScalarDecodeLimits {
    ScalarDecodeLimits::new(max_content_code_points)
}

#[test]
fn literal_and_folded_content_copy_exact_normalized_values_and_provenance() {
    let blocks = scan(b"literal: |\n  one\n  two\nfolded: >\n  one\n  two\n");
    let literal = decode_profile1_block_scalar_content(
        &blocks,
        0,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("literal content is bounded");
    let folded = decode_profile1_block_scalar_content(
        &blocks,
        1,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("folded content is bounded");

    assert_eq!(literal.style(), DecodedScalarStyle::LiteralBlock);
    assert_eq!(folded.style(), DecodedScalarStyle::FoldedBlock);
    let literal_text: String = literal
        .content()
        .iter()
        .map(|item| char::from_u32(item.code_point()).expect("verified Unicode scalar"))
        .collect();
    let folded_text: String = folded
        .content()
        .iter()
        .map(|item| char::from_u32(item.code_point()).expect("verified Unicode scalar"))
        .collect();
    assert_eq!(literal_text, "one\ntwo\n");
    assert_eq!(folded_text, "one two\n");

    for (source, decoded) in blocks.scalars()[1].content().iter().zip(folded.content()) {
        assert_eq!(decoded.code_point(), source.code_point());
        assert_eq!(decoded.source_atom_start(), source.source_atom_index());
        assert_eq!(decoded.source_atom_end(), source.source_atom_index() + 1);
        assert_eq!(decoded.byte_start(), source.byte_start());
        assert_eq!(decoded.byte_end(), source.byte_end());
        assert_eq!(
            decoded.origin(),
            match source.origin() {
                BlockScalarContentOrigin::Direct => DecodedContentOrigin::Direct,
                BlockScalarContentOrigin::FoldedLineBreak => {
                    DecodedContentOrigin::FoldedLineBreak
                }
            }
        );
    }
}

#[test]
fn block_decode_limits_and_indices_have_exact_diagnostics() {
    let blocks = scan(b"value: |\n  a\n  b\n");
    let source = &blocks.scalars()[0].content()[2];
    let error = decode_profile1_block_scalar_content(&blocks, 0, limits(2))
        .expect_err("the third normalized code point is excluded");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ContentLimitExceeded);
    assert_eq!(error.byte_offset(), source.byte_start());

    let error = decode_profile1_block_scalar_content(&blocks, 1, limits(8))
        .expect_err("there is no second block scalar");
    assert_eq!(error.kind(), ScalarDecodeErrorKind::ScalarIndexOutOfRange);
    assert_eq!(error.byte_offset(), blocks.source_len_bytes());
}

#[test]
fn empty_block_content_decodes_to_an_empty_semantic_string() {
    let blocks = scan(b"value: |-\n");
    let decoded = decode_profile1_block_scalar_content(&blocks, 0, limits(0))
        .expect("empty content consumes no output budget");
    assert_eq!(decoded.style(), DecodedScalarStyle::LiteralBlock);
    assert!(decoded.content().is_empty());
}
