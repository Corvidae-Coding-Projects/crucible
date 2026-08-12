use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource,
    BomPolicy, DecodeLimits, LayoutSource, QuotedScalarErrorKind, QuotedScalarScanLimits,
    QuotedScalarSource, QuotedScalarStyle, StructuralLexemeSource, StructuralScanLimits,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_QUOTED_SCALARS,
    MAX_PROFILE1_QUOTED_SCALAR_ATOMS, MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_STRUCTURAL_LEXEMES,
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

fn scan(
    input: &[u8],
) -> (
    AtomizedSource,
    LayoutSource,
    StructuralLexemeSource,
    QuotedScalarSource,
) {
    let atoms = atomize(input);
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(
            MAX_PROFILE1_QUOTED_SCALARS,
            MAX_PROFILE1_QUOTED_SCALAR_ATOMS,
        ),
    )
    .expect("quoted scalars are valid and bounded");
    (atoms, lines, candidates, quoted)
}

#[test]
fn single_and_double_quoted_scalars_have_exact_raw_ranges_and_styles() {
    let bytes = b"single: 'here''s # [,]'\ndouble: \"a, # \\\" \\x41 \\u03B2 \\U0001F600\"\n";
    let (atoms, lines, candidates, quoted) = scan(bytes);

    assert_eq!(quoted.profile_version(), 1);
    assert_eq!(quoted.input_transformation_version(), 1);
    assert_eq!(quoted.layout_transformation_version(), 1);
    assert_eq!(quoted.structural_transformation_version(), 1);
    assert_eq!(quoted.transformation_version(), 1);
    assert_eq!(quoted.source_len_bytes(), bytes.len() as u64);
    assert_eq!(quoted.bom_bytes(), 0);
    assert_eq!(quoted.input_atom_count(), atoms.atoms().len() as u64);
    assert_eq!(quoted.input_line_count(), lines.lines().len() as u64);
    assert_eq!(
        quoted.input_structural_lexeme_count(),
        candidates.lexemes().len() as u64
    );
    assert_eq!(quoted.scalars().len(), 2);

    let single_start = bytes.iter().position(|byte| *byte == b'\'').unwrap();
    let single_end = bytes[single_start + 1..]
        .windows(2)
        .position(|window| window == b"'\n")
        .map(|index| single_start + 1 + index + 1)
        .unwrap();
    let single = &quoted.scalars()[0];
    assert_eq!(single.style(), QuotedScalarStyle::Single);
    assert_eq!(single.start_line_number(), 0);
    assert_eq!(single.end_line_number(), 0);
    assert_eq!(single.byte_start(), single_start as u64);
    assert_eq!(single.byte_end(), single_end as u64);
    assert_eq!(&bytes[single_start..single_end], b"'here''s # [,]'");

    let double_start = bytes.iter().position(|byte| *byte == b'"').unwrap();
    let double = &quoted.scalars()[1];
    assert_eq!(double.style(), QuotedScalarStyle::Double);
    assert_eq!(double.start_line_number(), 1);
    assert_eq!(double.end_line_number(), 1);
    assert_eq!(double.byte_start(), double_start as u64);
    assert_eq!(double.byte_end(), (bytes.len() - 1) as u64);
    assert_eq!(double.end_atom_index(), atoms.atoms().len() as u64 - 1);
    assert!(single.end_atom_index() <= double.start_atom_index());
    assert!(single.byte_end() <= double.byte_start());
}

#[test]
fn empty_multibyte_and_every_profile_escape_form_are_bounded_losslessly() {
    let (_, _, _, empty) = scan(b"[\"\", '']\n");
    assert_eq!(empty.scalars().len(), 2);
    assert_eq!(
        empty.scalars()[0].byte_end() - empty.scalars()[0].byte_start(),
        2
    );
    assert_eq!(
        empty.scalars()[1].byte_end() - empty.scalars()[1].byte_start(),
        2
    );

    let multibyte_bytes = "q: 'β😀'\n".as_bytes();
    let (_, _, _, multibyte) = scan(multibyte_bytes);
    assert_eq!(multibyte.scalars()[0].byte_start(), 3);
    assert_eq!(multibyte.scalars()[0].byte_end(), 11);

    let escape_bytes = b"q: \"\\0\\a\\b\\t\\\t\\n\\v\\f\\r\\e\\ \\\"\\/\\\\\\N\\_\\L\\P\\x00\\uD7FF\\U00010000\"\n";
    let (_, _, _, escaped) = scan(escape_bytes);
    assert_eq!(escaped.scalars().len(), 1);
    assert_eq!(escaped.scalars()[0].byte_start(), 3);
    assert_eq!(
        escaped.scalars()[0].byte_end(),
        (escape_bytes.len() - 1) as u64
    );
}

#[test]
fn terminal_escape_and_partial_hex_errors_report_end_of_input() {
    for (bytes, kind) in [
        (&b"q: \"bad \\"[..], QuotedScalarErrorKind::InvalidEscape),
        (&b"q: \"\\u12"[..], QuotedScalarErrorKind::InvalidHexDigit),
    ] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let error = scan_profile1_quoted_scalars(
            &atoms,
            &lines,
            &candidates,
            QuotedScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("an escape ending at EOF is incomplete");
        assert_eq!(error.kind(), kind);
        assert_eq!(error.byte_offset(), bytes.len() as u64);
    }
}

#[test]
fn forbidden_raw_characters_are_rejected_while_escape_boundaries_remain_exact() {
    for (bytes, offset) in [
        (&b"q: \"nul \0\"\n"[..], 8),
        (&b"q: 'unit \x1f'\n"[..], 9),
        (&b"q: \"delete \x7f\"\n"[..], 11),
        (&b"q: \"control \xc2\x9f\"\n"[..], 12),
        (&b"q: \"nonchar \xef\xbf\xbe\"\n"[..], 12),
    ] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let error = scan_profile1_quoted_scalars(
            &atoms,
            &lines,
            &candidates,
            QuotedScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("raw YAML-forbidden characters are not quoted content");
        assert_eq!(error.kind(), QuotedScalarErrorKind::InvalidQuotedCharacter);
        assert_eq!(error.byte_offset(), offset, "fixture: {bytes:?}");
    }

    for bytes in [&b"q: \"\\uD800\"\n"[..], &b"q: \"\\uDFFF\"\n"[..]] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let error = scan_profile1_quoted_scalars(
            &atoms,
            &lines,
            &candidates,
            QuotedScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("escaped surrogates are not Unicode scalar values");
        assert_eq!(error.kind(), QuotedScalarErrorKind::InvalidEscapedCodePoint);
        assert_eq!(error.byte_offset(), 4);
    }

    let (_, _, _, upper) = scan(b"q: \"\\U0010FFFF\"\n");
    assert_eq!(upper.scalars().len(), 1);
}

#[test]
fn multiline_quotes_preserve_crlf_tabs_and_escaped_line_break_ranges() {
    let bytes = b"q: \"folded\r\n \tline\\\r\n  next\"\r\n";
    let (atoms, _, _, quoted) = scan(bytes);
    let scalar = &quoted.scalars()[0];

    assert_eq!(scalar.style(), QuotedScalarStyle::Double);
    assert_eq!(scalar.start_line_number(), 0);
    assert_eq!(scalar.end_line_number(), 2);
    assert_eq!(scalar.byte_start(), 3);
    assert_eq!(scalar.byte_end(), (bytes.len() - 2) as u64);
    assert_eq!(
        atoms.atoms()[scalar.start_atom_index() as usize].code_point(),
        u32::from(b'"'),
    );
    assert_eq!(
        atoms.atoms()[scalar.end_atom_index() as usize - 1].code_point(),
        u32::from(b'"'),
    );
}

#[test]
fn quotes_embedded_in_plain_content_do_not_start_quoted_scalars() {
    let (_, _, _, plain) = scan(b"foo\"bar baz'qux\n");
    assert!(plain.scalars().is_empty());

    let (_, _, _, flow) = scan(b"[\"x\", 'y']\n");
    assert_eq!(flow.scalars().len(), 2);
    assert_eq!(flow.scalars()[0].style(), QuotedScalarStyle::Double);
    assert_eq!(flow.scalars()[1].style(), QuotedScalarStyle::Single);

    let (_, _, _, json_flow) = scan(br#"{"a":"b"}"#);
    assert_eq!(json_flow.scalars().len(), 2);
    assert_eq!(json_flow.scalars()[0].byte_start(), 1);
    assert_eq!(json_flow.scalars()[1].byte_start(), 5);
}

#[test]
fn quotes_inside_plain_multiline_and_block_scalar_regions_remain_raw_content() {
    for bytes in [
        &b"key: a \"quoted\" tail\n"[..],
        &b"key: a \"unterminated\n"[..],
        &b"key: first line\n  \"quoted\" continuation\nnext: value\n"[..],
    ] {
        let (_, _, _, quoted) = scan(bytes);
        assert!(quoted.scalars().is_empty(), "fixture: {bytes:?}");
    }

    let bytes = b"literal: |\n  \"raw\"\nfolded: >\n  'also raw'\nnext: \"real\"\n";
    let (_, _, _, quoted) = scan(bytes);
    assert_eq!(quoted.scalars().len(), 1);
    assert_eq!(quoted.scalars()[0].style(), QuotedScalarStyle::Double);
    assert_eq!(
        &bytes[quoted.scalars()[0].byte_start() as usize..quoted.scalars()[0].byte_end() as usize],
        b"\"real\"",
    );
}

#[test]
fn invalid_escape_hex_code_point_and_unterminated_errors_have_exact_offsets() {
    for (bytes, kind, offset) in [
        (
            &b"q: \"bad \\c\"\n"[..],
            QuotedScalarErrorKind::InvalidEscape,
            9,
        ),
        (
            &b"q: \"bad \\u0x00\"\n"[..],
            QuotedScalarErrorKind::InvalidHexDigit,
            11,
        ),
        (
            &b"q: \"bad \\U00110000\"\n"[..],
            QuotedScalarErrorKind::InvalidEscapedCodePoint,
            8,
        ),
        (
            &b"q: \"unterminated"[..],
            QuotedScalarErrorKind::UnterminatedQuotedScalar,
            16,
        ),
        (
            &b"q: 'unterminated"[..],
            QuotedScalarErrorKind::UnterminatedQuotedScalar,
            16,
        ),
    ] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let error = scan_profile1_quoted_scalars(
            &atoms,
            &lines,
            &candidates,
            QuotedScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("fixture is not a valid quoted-scalar stream");
        assert_eq!(error.kind(), kind, "fixture: {bytes:?}");
        assert_eq!(error.byte_offset(), offset, "fixture: {bytes:?}");
    }
}

#[test]
fn quoted_scalar_caps_are_all_or_error_at_the_first_excluded_atom_or_scalar() {
    assert_eq!(MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS);
    assert_eq!(MAX_PROFILE1_QUOTED_SCALAR_ATOMS, MAX_PROFILE1_LEXICAL_ATOMS);
    let bytes = b"\"abcd\" 'z'";
    let atoms = atomize(bytes);
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);

    let atom_error = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(2, 5),
    )
    .expect_err("the closing double quote is the sixth scalar atom");
    assert_eq!(
        atom_error.kind(),
        QuotedScalarErrorKind::ScalarAtomLimitExceeded
    );
    assert_eq!(atom_error.byte_offset(), 5);

    let scalar_error = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(1, 6),
    )
    .expect_err("the single-quoted scalar is the second scalar");
    assert_eq!(
        scalar_error.kind(),
        QuotedScalarErrorKind::ScalarLimitExceeded
    );
    assert_eq!(scalar_error.byte_offset(), 7);

    let accepted = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(2, 6),
    )
    .expect("both exact limits are accepted");
    assert_eq!(accepted.scalars().len(), 2);
}

#[test]
fn bom_and_zero_cap_precedence_are_exact() {
    let bytes = b"\xef\xbb\xbf\"x\"";
    let (_, _, _, quoted) = scan(bytes);
    assert_eq!(quoted.bom_bytes(), 3);
    assert_eq!(quoted.scalars()[0].byte_start(), 3);
    assert_eq!(quoted.scalars()[0].byte_end(), 6);

    let atoms = atomize(b"\"x\"");
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);
    let scalar_error = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(0, 0),
    )
    .expect_err("the scalar-count cap takes precedence at a quote start");
    assert_eq!(
        scalar_error.kind(),
        QuotedScalarErrorKind::ScalarLimitExceeded
    );
    assert_eq!(scalar_error.byte_offset(), 0);

    let atom_error = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &candidates,
        QuotedScalarScanLimits::new(1, 0),
    )
    .expect_err("a zero scalar-atom cap rejects the opening quote");
    assert_eq!(
        atom_error.kind(),
        QuotedScalarErrorKind::ScalarAtomLimitExceeded
    );
    assert_eq!(atom_error.byte_offset(), 0);
}

#[test]
fn a_structural_partition_from_another_source_is_rejected_before_quote_scanning() {
    let atoms = atomize(b"\"x\"\n");
    let lines = layout(&atoms);
    let other_atoms = atomize(b"\"different\"\n");
    let other_lines = layout(&other_atoms);
    let other_candidates = structural(&other_atoms, &other_lines);

    let error = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &other_candidates,
        QuotedScalarScanLimits::new(u64::MAX, u64::MAX),
    )
    .expect_err("candidate evidence is not canonical for the supplied atoms");
    assert_eq!(error.kind(), QuotedScalarErrorKind::InputStructuralMismatch);
    assert_eq!(error.byte_offset(), 0);
}
