use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    decode_profile1_plain_scalar_content, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource,
    BomPolicy, DecodeLimits, DecodedContentOrigin, DecodedScalarStyle, LayoutSource,
    PlainScalarScanLimits, PlainScalarSource, QuotedScalarScanLimits, QuotedScalarSource,
    ScalarDecodeErrorKind, ScalarDecodeLimits, StructuralLexemeSource, StructuralScanLimits,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS,
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
    PlainScalarSource,
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
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &quoted,
        PlainScalarScanLimits::new(MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS),
    )
    .expect("canonical plain scalars");
    (atoms, lines, candidates, quoted, plain)
}

fn limits(max_content_code_points: u64) -> ScalarDecodeLimits {
    ScalarDecodeLimits::new(max_content_code_points)
}

fn text(content: &crucible_yaml::DecodedScalarContent) -> String {
    content
        .content()
        .iter()
        .map(|item| char::from_u32(item.code_point()).expect("verified Unicode scalar"))
        .collect()
}

#[test]
fn multiline_unicode_and_interior_white_decode_with_exact_provenance() {
    let bytes = b"key: \xce\xb2 one  \n  two\n\n    three \t four\nnext: done\n";
    let (atoms, _, _, _, plain) = scan(bytes);
    let decoded = decode_profile1_plain_scalar_content(
        &atoms,
        &plain,
        1,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("multiline plain content is bounded");

    assert_eq!(decoded.style(), DecodedScalarStyle::Plain);
    assert_eq!(text(&decoded), "\u{03b2} one two\nthree \t four");
    let direct_beta = decoded
        .content()
        .first()
        .expect("the scalar has direct content");
    assert_eq!(direct_beta.code_point(), u32::from('\u{03b2}'));
    assert_eq!(direct_beta.origin(), DecodedContentOrigin::Direct);
    assert_eq!(
        direct_beta.source_atom_end() - direct_beta.source_atom_start(),
        1
    );
    assert_eq!(
        &bytes[direct_beta.byte_start() as usize..direct_beta.byte_end() as usize],
        "\u{03b2}".as_bytes()
    );

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
fn first_excluded_direct_and_folded_records_report_exact_source_bytes() {
    let bytes = b"key: first  \n  second\n";
    let (atoms, _, _, _, plain) = scan(bytes);
    let scalar = &plain.scalars()[1];

    let direct_error = decode_profile1_plain_scalar_content(&atoms, &plain, 1, limits(0))
        .expect_err("the first direct code point exceeds a zero cap");
    assert_eq!(
        direct_error.kind(),
        ScalarDecodeErrorKind::ContentLimitExceeded
    );
    assert_eq!(direct_error.byte_offset(), scalar.byte_start());

    let unrestricted = decode_profile1_plain_scalar_content(
        &atoms,
        &plain,
        1,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("the fixture is bounded");
    let folded_index = unrestricted
        .content()
        .iter()
        .position(|item| item.origin() == DecodedContentOrigin::FoldedLineBreak)
        .expect("the physical line feed emits one folded space");
    let folded = &unrestricted.content()[folded_index];
    let folded_error =
        decode_profile1_plain_scalar_content(&atoms, &plain, 1, limits(folded_index as u64))
            .expect_err("the folded space is the first excluded record");
    assert_eq!(
        folded_error.kind(),
        ScalarDecodeErrorKind::ContentLimitExceeded
    );
    assert_eq!(folded_error.byte_offset(), folded.byte_start());
    assert_eq!(bytes[folded.byte_start() as usize], b'\n');
}

#[test]
fn quotes_embedded_in_plain_content_remain_direct_content() {
    let bytes = b"key: a \"quote\" and 'apostrophe'\n";
    let (atoms, _, _, _, plain) = scan(bytes);
    let decoded = decode_profile1_plain_scalar_content(
        &atoms,
        &plain,
        1,
        limits(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS),
    )
    .expect("quotes inside an authenticated plain range are not quote delimiters");

    assert_eq!(text(&decoded), "a \"quote\" and 'apostrophe'");
    assert!(decoded
        .content()
        .iter()
        .all(|item| item.origin() == DecodedContentOrigin::Direct));
}

#[test]
fn source_authentication_and_scalar_indices_are_exact() {
    let (atoms, _, _, _, plain) = scan(b"key: value\n");
    let (other_atoms, _, _, _, _) = scan(b"other: source\n");

    let mismatch = decode_profile1_plain_scalar_content(&other_atoms, &plain, 0, limits(32))
        .expect_err("plain evidence from another atom stream is rejected");
    assert_eq!(mismatch.kind(), ScalarDecodeErrorKind::InputPlainMismatch);
    assert_eq!(mismatch.byte_offset(), other_atoms.bom_bytes());

    let out_of_range = decode_profile1_plain_scalar_content(&atoms, &plain, 2, limits(32))
        .expect_err("there is no third plain scalar");
    assert_eq!(
        out_of_range.kind(),
        ScalarDecodeErrorKind::ScalarIndexOutOfRange
    );
    assert_eq!(out_of_range.byte_offset(), atoms.source_len_bytes());
}
