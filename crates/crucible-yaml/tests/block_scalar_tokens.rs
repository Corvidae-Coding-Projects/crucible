use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    scan_profile1_block_scalars, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource, BlockChomping,
    BlockScalarContentOrigin, BlockScalarErrorKind, BlockScalarScanLimits, BlockScalarSource,
    BlockScalarStyle, BomPolicy, DecodeLimits, LayoutSource, PlainScalarScanLimits,
    PlainScalarSource, QuotedScalarScanLimits, QuotedScalarSource, StructuralLexemeSource,
    StructuralScanLimits, MAX_PROFILE1_BLOCK_SCALARS,
    MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_PLAIN_SCALARS,
    MAX_PROFILE1_PLAIN_SCALAR_ATOMS, MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_QUOTED_SCALAR_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_STRUCTURAL_LEXEMES,
    MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
};

fn atomize(input: &[u8]) -> AtomizedSource {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 bytes");
    atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS))
        .expect("decoded source fits the profile atom cap")
}

fn layout(source: &AtomizedSource) -> LayoutSource {
    analyze_profile1_layout(source, canonical_structural_layout_limits())
        .expect("source has canonical bounded line layout")
}

fn structural(source: &AtomizedSource, layout: &LayoutSource) -> StructuralLexemeSource {
    scan_profile1_structural_lexemes(
        source,
        layout,
        StructuralScanLimits::new(MAX_PROFILE1_STRUCTURAL_LEXEMES),
    )
    .expect("source has a canonical structural-candidate partition")
}

fn quoted(
    source: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
) -> QuotedScalarSource {
    scan_profile1_quoted_scalars(
        source,
        layout,
        structural,
        QuotedScalarScanLimits::new(
            MAX_PROFILE1_QUOTED_SCALARS,
            MAX_PROFILE1_QUOTED_SCALAR_ATOMS,
        ),
    )
    .expect("quoted scalar evidence is canonical")
}

fn plain(
    source: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
    quoted: &QuotedScalarSource,
) -> PlainScalarSource {
    scan_profile1_plain_scalars(
        source,
        layout,
        structural,
        quoted,
        PlainScalarScanLimits::new(MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS),
    )
    .expect("plain scalar evidence is canonical")
}

fn profile_limits() -> BlockScalarScanLimits {
    BlockScalarScanLimits::new(
        MAX_PROFILE1_BLOCK_SCALARS,
        MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
        MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS,
        MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
    )
}

fn upstream(
    input: &[u8],
) -> (
    AtomizedSource,
    LayoutSource,
    StructuralLexemeSource,
    QuotedScalarSource,
    PlainScalarSource,
) {
    let atoms = atomize(input);
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);
    let quotes = quoted(&atoms, &lines, &candidates);
    let plains = plain(&atoms, &lines, &candidates, &quotes);
    (atoms, lines, candidates, quotes, plains)
}

fn scan(input: &[u8]) -> (AtomizedSource, BlockScalarSource) {
    let (atoms, lines, candidates, quotes, plains) = upstream(input);
    let blocks = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        &plains,
        profile_limits(),
    )
    .expect("block scalars are valid and bounded");
    (atoms, blocks)
}

fn content(source: &BlockScalarSource, index: usize) -> String {
    source.scalars()[index]
        .content()
        .iter()
        .map(|scalar| char::from_u32(scalar.code_point()).expect("verified Unicode scalar value"))
        .collect()
}

fn error(input: &[u8]) -> (BlockScalarErrorKind, u64) {
    let (atoms, lines, candidates, quotes, plains) = upstream(input);
    let error = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        &plains,
        profile_limits(),
    )
    .expect_err("fixture is an invalid block scalar");
    (error.kind(), error.byte_offset())
}

#[test]
fn literal_and_folded_styles_emit_complete_normalized_content() {
    let bytes = b"literal: |\n  line one\n  line two\nfolded: >\n  line one\n  line two\n";
    let (atoms, blocks) = scan(bytes);
    assert_eq!(blocks.profile_version(), 1);
    assert_eq!(blocks.input_transformation_version(), 1);
    assert_eq!(blocks.layout_transformation_version(), 1);
    assert_eq!(blocks.structural_transformation_version(), 1);
    assert_eq!(blocks.quoted_transformation_version(), 1);
    assert_eq!(blocks.plain_transformation_version(), 1);
    assert_eq!(blocks.transformation_version(), 1);
    assert_eq!(blocks.input_atom_count(), atoms.atoms().len() as u64);
    assert_eq!(blocks.scalars().len(), 2);

    let literal = &blocks.scalars()[0];
    assert_eq!(literal.style(), BlockScalarStyle::Literal);
    assert_eq!(literal.chomping(), BlockChomping::Clip);
    assert_eq!(literal.explicit_indentation(), None);
    assert_eq!(literal.content_indentation(), 2);
    assert_eq!(content(&blocks, 0), "line one\nline two\n");

    let folded = &blocks.scalars()[1];
    assert_eq!(folded.style(), BlockScalarStyle::Folded);
    assert_eq!(folded.chomping(), BlockChomping::Clip);
    assert_eq!(content(&blocks, 1), "line one line two\n");
    assert!(folded
        .content()
        .iter()
        .any(|scalar| scalar.origin() == BlockScalarContentOrigin::FoldedLineBreak));
}

#[test]
fn header_modifiers_work_in_both_orders_and_preserve_more_indentation() {
    let bytes = b"first: |2-\n    indented\nsecond: >+2\n    folded\n    line\n\nnext: ok\n";
    let (_, blocks) = scan(bytes);
    assert_eq!(blocks.scalars().len(), 2);

    let first = &blocks.scalars()[0];
    assert_eq!(first.explicit_indentation(), Some(2));
    assert_eq!(first.content_indentation(), 2);
    assert_eq!(first.chomping(), BlockChomping::Strip);
    assert_eq!(content(&blocks, 0), "  indented");

    let second = &blocks.scalars()[1];
    assert_eq!(second.explicit_indentation(), Some(2));
    assert_eq!(second.chomping(), BlockChomping::Keep);
    assert_eq!(content(&blocks, 1), "  folded\n  line\n\n");

    let reverse = b"first: |-2\n  exact\nsecond: |2+\n  exact\n";
    let (_, reverse_blocks) = scan(reverse);
    assert_eq!(reverse_blocks.scalars()[0].explicit_indentation(), Some(2));
    assert_eq!(reverse_blocks.scalars()[0].chomping(), BlockChomping::Strip);
    assert_eq!(reverse_blocks.scalars()[1].explicit_indentation(), Some(2));
    assert_eq!(reverse_blocks.scalars()[1].chomping(), BlockChomping::Keep);
}

#[test]
fn strip_clip_and_keep_apply_exact_trailing_line_rules() {
    let bytes = b"strip: |-\n  text\n\nclip: |\n  text\n\nkeep: |+\n  text\n\nnext: end\n";
    let (_, blocks) = scan(bytes);
    assert_eq!(blocks.scalars().len(), 3);
    assert_eq!(content(&blocks, 0), "text");
    assert_eq!(content(&blocks, 1), "text\n");
    assert_eq!(content(&blocks, 2), "text\n\n");
}

#[test]
fn folded_empty_and_more_indented_lines_are_never_flattened() {
    let bytes = b"value: >\n  folded\n  line\n\n    more\n    indented\n  final\n";
    let (_, blocks) = scan(bytes);
    assert_eq!(
        content(&blocks, 0),
        "folded line\n\n  more\n  indented\nfinal\n"
    );
}

#[test]
fn auto_indentation_handles_leading_and_all_empty_content() {
    let bytes = b"auto: |\n\n    text\nempty: |+\n  \n    \nnext: ok\n";
    let (_, blocks) = scan(bytes);
    assert_eq!(blocks.scalars().len(), 2);
    assert_eq!(blocks.scalars()[0].content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "\ntext\n");
    assert_eq!(blocks.scalars()[1].content_indentation(), 4);
    assert_eq!(content(&blocks, 1), "\n\n");
}

#[test]
fn leading_empty_and_later_content_indentation_errors_have_exact_offsets() {
    let leading = b"key: |\n    \n  text\n";
    let leading_line = leading
        .windows(5)
        .position(|window| window == b"    \n")
        .unwrap();
    assert_eq!(
        error(leading),
        (
            BlockScalarErrorKind::InvalidLeadingEmptyIndentation,
            (leading_line + 2) as u64,
        )
    );

    let explicit = b"key: |2\n text\n";
    let text = explicit
        .windows(4)
        .position(|window| window == b"text")
        .unwrap();
    assert_eq!(
        error(explicit),
        (BlockScalarErrorKind::InvalidBlockIndentation, text as u64)
    );

    let later = b"key: |\n    first\n  second\n";
    let second = later
        .windows(6)
        .position(|window| window == b"second")
        .unwrap();
    assert_eq!(
        error(later),
        (BlockScalarErrorKind::InvalidBlockIndentation, second as u64)
    );
}

#[test]
fn tabs_are_content_only_after_required_space_indentation() {
    let valid = b"block: |\n \tprintf 'ok'\n detected\n";
    let (_, blocks) = scan(valid);
    assert_eq!(blocks.scalars()[0].content_indentation(), 1);
    assert_eq!(content(&blocks, 0), "\tprintf 'ok'\ndetected\n");

    let invalid = b"block: |2\n \tbad\n";
    let tab = invalid.iter().position(|byte| *byte == b'\t').unwrap();
    assert_eq!(
        error(invalid),
        (BlockScalarErrorKind::TabInIndentation, tab as u64)
    );
}

#[test]
fn block_content_keeps_comment_and_indicator_spellings_raw() {
    let bytes = b"value: |\n  # not a comment\n  [not, flow]: {still raw}\nnext: end\n";
    let (_, blocks) = scan(bytes);
    assert_eq!(
        content(&blocks, 0),
        "# not a comment\n[not, flow]: {still raw}\n"
    );
}

#[test]
fn block_content_rejects_a_document_bom_at_the_exact_byte() {
    let bytes = b"value: |\n  before\xef\xbb\xbfafter\n";
    let offset = bytes
        .windows(3)
        .position(|window| window == b"\xef\xbb\xbf")
        .expect("fixture BOM");
    assert_eq!(
        error(bytes),
        (BlockScalarErrorKind::InvalidBlockCharacter, offset as u64)
    );
}

#[test]
fn headers_reject_zero_duplicates_text_nonseparated_comments_and_eof() {
    let fixtures: &[(&[u8], BlockScalarErrorKind, usize)] = &[
        (
            b"key: |0\n  x\n",
            BlockScalarErrorKind::InvalidIndentationIndicator,
            6,
        ),
        (
            b"key: |++\n  x\n",
            BlockScalarErrorKind::InvalidBlockHeader,
            7,
        ),
        (
            b"key: |2+3\n  x\n",
            BlockScalarErrorKind::InvalidBlockHeader,
            8,
        ),
        (
            b"key: |#comment\n  x\n",
            BlockScalarErrorKind::InvalidBlockHeader,
            6,
        ),
        (
            b"key: | text\n  x\n",
            BlockScalarErrorKind::InvalidBlockHeader,
            7,
        ),
        (
            b"key: |",
            BlockScalarErrorKind::MissingBlockHeaderLineBreak,
            6,
        ),
    ];
    for (bytes, kind, offset) in fixtures {
        assert_eq!(error(bytes), (*kind, *offset as u64), "fixture: {bytes:?}");
    }
}

#[test]
fn quoted_plain_property_and_flow_indicators_are_not_block_scalars() {
    let bytes = b"quoted: \"| not block\"\nplain: a|b\nflow: [a|b, c>d]\ntagged: !!str |\n  real\n";
    let (_, blocks) = scan(bytes);
    assert_eq!(blocks.scalars().len(), 1);
    assert_eq!(content(&blocks, 0), "real\n");
}

#[test]
fn crlf_provenance_and_raw_token_ranges_remain_exact() {
    let bytes = b"key: >\r\n  alpha\r\n  beta\r\nnext: ok\r\n";
    let (_, blocks) = scan(bytes);
    let scalar = &blocks.scalars()[0];
    assert_eq!(content(&blocks, 0), "alpha beta\n");
    assert_eq!(
        &bytes[scalar.byte_start() as usize..scalar.byte_end() as usize],
        b">\r\n  alpha\r\n  beta\r\n"
    );
    let folded = scalar
        .content()
        .iter()
        .find(|item| item.origin() == BlockScalarContentOrigin::FoldedLineBreak)
        .expect("one source line break is folded");
    assert_eq!(
        &bytes[folded.byte_start() as usize..folded.byte_end() as usize],
        b"\r\n"
    );
}

#[test]
fn all_four_resource_caps_are_exact_and_syntax_has_local_precedence() {
    assert_eq!(MAX_PROFILE1_BLOCK_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS);
    assert_eq!(
        MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
        MAX_PROFILE1_LEXICAL_ATOMS
    );
    assert_eq!(
        MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS,
        MAX_PROFILE1_LEXICAL_ATOMS
    );
    assert_eq!(
        MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
        MAX_PROFILE1_LEXICAL_ATOMS
    );

    let bytes = b"first: |-\n  a\nsecond: |-\n  b\n";
    let (atoms, lines, candidates, quotes, plains) = upstream(bytes);
    let first_indicator = bytes.iter().position(|byte| *byte == b'|').unwrap();
    let second_indicator = bytes.iter().rposition(|byte| *byte == b'|').unwrap();
    let first_a = bytes.iter().position(|byte| *byte == b'a').unwrap();
    let second_b = bytes.iter().rposition(|byte| *byte == b'b').unwrap();

    let cases = [
        (
            BlockScalarScanLimits::new(0, u64::MAX, u64::MAX, u64::MAX),
            BlockScalarErrorKind::ScalarLimitExceeded,
            first_indicator,
        ),
        (
            BlockScalarScanLimits::new(1, u64::MAX, u64::MAX, u64::MAX),
            BlockScalarErrorKind::ScalarLimitExceeded,
            second_indicator,
        ),
        (
            BlockScalarScanLimits::new(u64::MAX, 2, u64::MAX, u64::MAX),
            BlockScalarErrorKind::PresentationAtomLimitExceeded,
            first_indicator + 2,
        ),
        (
            BlockScalarScanLimits::new(u64::MAX, u64::MAX, 0, u64::MAX),
            BlockScalarErrorKind::ScalarContentLimitExceeded,
            first_a,
        ),
        (
            BlockScalarScanLimits::new(u64::MAX, u64::MAX, u64::MAX, 1),
            BlockScalarErrorKind::TotalContentLimitExceeded,
            second_b,
        ),
    ];
    for (limits, kind, offset) in cases {
        let error =
            scan_profile1_block_scalars(&atoms, &lines, &candidates, &quotes, &plains, limits)
                .expect_err("the selected cap excludes this fixture");
        assert_eq!(error.kind(), kind);
        assert_eq!(error.byte_offset(), offset as u64);
    }

    let malformed = b"key: |0\n  x\n";
    let (atoms, lines, candidates, quotes, plains) = upstream(malformed);
    let error = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        &plains,
        BlockScalarScanLimits::new(0, 0, 0, 0),
    )
    .expect_err("syntax precedes local caps");
    assert_eq!(
        error.kind(),
        BlockScalarErrorKind::InvalidIndentationIndicator
    );
}

#[test]
fn every_upstream_authentication_mismatch_is_distinct_typed_and_all_or_error() {
    let bytes = b"key: |\n  value\n";
    let atoms = atomize(bytes);
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);
    let quotes = quoted(&atoms, &lines, &candidates);
    let plains = plain(&atoms, &lines, &candidates, &quotes);

    let other = b"longer: |\n value\n";
    let (other_atoms, other_lines, other_candidates, other_quotes, other_plains) = upstream(other);
    assert_ne!(atoms.atoms().len(), other_atoms.atoms().len());
    let error = scan_profile1_block_scalars(
        &atoms,
        &other_lines,
        &candidates,
        &quotes,
        &plains,
        profile_limits(),
    )
    .expect_err("layout evidence belongs to a different atom source");
    assert_eq!(error.kind(), BlockScalarErrorKind::InputLayoutMismatch);
    assert_eq!(error.byte_offset(), 0);

    let error = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &other_candidates,
        &quotes,
        &plains,
        profile_limits(),
    )
    .expect_err("structural evidence belongs to a different atom source");
    assert_eq!(error.kind(), BlockScalarErrorKind::InputStructuralMismatch);
    assert_eq!(error.byte_offset(), 0);

    let error = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &candidates,
        &other_quotes,
        &plains,
        profile_limits(),
    )
    .expect_err("quoted evidence belongs to a different atom source");
    assert_eq!(error.kind(), BlockScalarErrorKind::InputQuotedMismatch);
    assert_eq!(error.byte_offset(), 0);

    let error = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        &other_plains,
        profile_limits(),
    )
    .expect_err("plain evidence belongs to a different atom source");
    assert_eq!(error.kind(), BlockScalarErrorKind::InputPlainMismatch);
    assert_eq!(error.byte_offset(), 0);
    assert_eq!(other_lines.transformation_version(), 1);
    assert_eq!(other_candidates.transformation_version(), 1);
    assert_eq!(other_quotes.transformation_version(), 1);
}

#[test]
fn compact_block_collection_context_sets_parent_indentation_and_exact_boundaries() {
    let (_, blocks) = scan(b"- key: |\n    x\n  other: y\n");
    let scalar = &blocks.scalars()[0];
    assert_eq!(scalar.parent_indentation(), 2);
    assert_eq!(scalar.content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "x\n");
    assert_eq!(scalar.byte_end(), 15);

    let (_, blocks) = scan(b"- key: |2\n    x\n");
    let scalar = &blocks.scalars()[0];
    assert_eq!(scalar.parent_indentation(), 2);
    assert_eq!(scalar.content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "x\n");

    let (_, blocks) = scan(b"- ? |2\n      x\n  : y\n");
    let scalar = &blocks.scalars()[0];
    assert_eq!(scalar.parent_indentation(), 2);
    assert_eq!(scalar.content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "  x\n");
    assert_eq!(scalar.byte_end(), 15);

    let (_, blocks) = scan(b"- - |2\n    x\n");
    let scalar = &blocks.scalars()[0];
    assert_eq!(scalar.parent_indentation(), 2);
    assert_eq!(scalar.content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "x\n");

    let (_, blocks) = scan(b"-    key: |2\n       x\n");
    assert_eq!(blocks.scalars()[0].parent_indentation(), 5);
    assert_eq!(blocks.scalars()[0].content_indentation(), 7);
    assert_eq!(content(&blocks, 0), "x\n");

    let (_, blocks) = scan(b"- &anchor key: |2\n    x\n");
    assert_eq!(blocks.scalars()[0].parent_indentation(), 2);
    assert_eq!(blocks.scalars()[0].content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "x\n");

    let (_, blocks) = scan(b"? key: |2\n    x\n");
    assert_eq!(blocks.scalars()[0].parent_indentation(), 2);
    assert_eq!(blocks.scalars()[0].content_indentation(), 4);
    assert_eq!(content(&blocks, 0), "x\n");

    let (_, blocks) = scan(b"- key: |2\n  x\n");
    assert_eq!(blocks.scalars()[0].parent_indentation(), 2);
    assert_eq!(blocks.scalars()[0].content(), &[]);
    assert_eq!(blocks.scalars()[0].byte_end(), 10);
}
