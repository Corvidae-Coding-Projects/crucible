use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AtomizeLimits, AtomizedSource, BomPolicy, DecodeLimits, LayoutSource, PlainScalarErrorKind,
    PlainScalarScanLimits, PlainScalarSource, QuotedScalarScanLimits, QuotedScalarSource,
    StructuralLexemeSource, StructuralScanLimits, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS,
    MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_QUOTED_SCALAR_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
    MAX_PROFILE1_STRUCTURAL_LEXEMES,
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

fn scan(
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
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        PlainScalarScanLimits::new(MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS),
    )
    .expect("plain scalars are valid and bounded");
    (atoms, lines, candidates, quotes, plain)
}

fn raw_ranges<'a>(bytes: &'a [u8], source: &PlainScalarSource) -> Vec<&'a [u8]> {
    source
        .scalars()
        .iter()
        .map(|scalar| &bytes[scalar.byte_start() as usize..scalar.byte_end() as usize])
        .collect()
}

#[test]
fn block_and_flow_plain_character_rules_have_exact_ranges() {
    let bytes = b"plain: value\nseq: [-123, http://example.com/foo#bar, ::vector, two:three]\n";
    let (atoms, lines, candidates, quotes, plain) = scan(bytes);

    assert_eq!(plain.profile_version(), 1);
    assert_eq!(plain.input_transformation_version(), 1);
    assert_eq!(plain.layout_transformation_version(), 1);
    assert_eq!(plain.structural_transformation_version(), 1);
    assert_eq!(plain.quoted_transformation_version(), 1);
    assert_eq!(plain.transformation_version(), 1);
    assert_eq!(plain.source_len_bytes(), bytes.len() as u64);
    assert_eq!(plain.bom_bytes(), 0);
    assert_eq!(plain.input_atom_count(), atoms.atoms().len() as u64);
    assert_eq!(plain.input_line_count(), lines.lines().len() as u64);
    assert_eq!(
        plain.input_structural_lexeme_count(),
        candidates.lexemes().len() as u64
    );
    assert_eq!(
        plain.input_quoted_scalar_count(),
        quotes.scalars().len() as u64
    );
    assert_eq!(
        raw_ranges(bytes, &plain),
        vec![
            &b"plain"[..],
            &b"value"[..],
            &b"seq"[..],
            &b"-123"[..],
            &b"http://example.com/foo#bar"[..],
            &b"::vector"[..],
            &b"two:three"[..],
        ]
    );
}

#[test]
fn multiline_plain_ranges_include_internal_presentation_but_trim_the_tail() {
    let bytes = b"key: first line\n  second line\n  \tthird line  \nnext: end\n";
    let (_, _, _, _, plain) = scan(bytes);
    assert_eq!(
        raw_ranges(bytes, &plain),
        vec![
            &b"key"[..],
            &b"first line\n  second line\n  \tthird line"[..],
            &b"next"[..],
            &b"end"[..],
        ]
    );
    let multiline = &plain.scalars()[1];
    assert_eq!(multiline.start_line_number(), 0);
    assert_eq!(multiline.end_line_number(), 2);
}

#[test]
fn quoted_and_block_regions_are_not_reclassified_as_plain_content() {
    let bytes = b"literal: |\n  raw: \"not quoted\" # still raw\nfolded: >\n  [also, raw]\nplain: a \"quote\" and [brackets]\nreal: \"x\"\n";
    let (_, _, _, quotes, plain) = scan(bytes);
    assert_eq!(quotes.scalars().len(), 1);
    assert_eq!(
        raw_ranges(bytes, &plain),
        vec![
            &b"literal"[..],
            &b"folded"[..],
            &b"plain"[..],
            &b"a \"quote\" and [brackets]"[..],
            &b"real"[..],
        ]
    );
}

#[test]
fn contextual_tabs_distinguish_indentation_from_separation_and_scalar_content() {
    for (bytes, offset) in [
        (&b"\troot"[..], 0),
        (&b"key:\n\tvalue\n"[..], 5),
        (&b"key: first\n\tcontinued\n"[..], 11),
    ] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let quotes = quoted(&atoms, &lines, &candidates);
        let error = scan_profile1_plain_scalars(
            &atoms,
            &lines,
            &candidates,
            &quotes,
            PlainScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("the tab occurs before required structural indentation");
        assert_eq!(error.kind(), PlainScalarErrorKind::TabInIndentation);
        assert_eq!(error.byte_offset(), offset, "fixture: {bytes:?}");
    }

    let bytes = b"key: first\n  \tcontinued\nblock: |\n  \tprintf 'ok'\nflow: [one,\n\t two]\n";
    let (_, _, _, _, plain) = scan(bytes);
    assert!(raw_ranges(bytes, &plain)
        .iter()
        .any(|range| *range == &b"first\n  \tcontinued"[..]));
    assert!(raw_ranges(bytes, &plain).contains(&&b"two"[..]));
}

#[test]
fn comments_mapping_delimiters_and_flow_punctuation_terminate_without_becoming_content() {
    let bytes = b"outside: Up, up, and away! # comment\nflow: [one, two: three, foo#bar]\n";
    let (_, _, _, _, plain) = scan(bytes);
    assert_eq!(
        raw_ranges(bytes, &plain),
        vec![
            &b"outside"[..],
            &b"Up, up, and away!"[..],
            &b"flow"[..],
            &b"one"[..],
            &b"two"[..],
            &b"three"[..],
            &b"foo#bar"[..],
        ]
    );
}

#[test]
fn properties_aliases_and_empty_continuation_lines_preserve_plain_boundaries() {
    let bytes = b"anchored: &name value\nalias: *name\ntagged: !!str text\nmulti: first\n\n  second\nnext: end\n";
    let (_, _, _, _, plain) = scan(bytes);
    assert_eq!(
        raw_ranges(bytes, &plain),
        vec![
            &b"anchored"[..],
            &b"value"[..],
            &b"alias"[..],
            &b"tagged"[..],
            &b"text"[..],
            &b"multi"[..],
            &b"first\n\n  second"[..],
            &b"next"[..],
            &b"end"[..],
        ]
    );
}

#[test]
fn verbatim_tag_uri_punctuation_never_becomes_plain_content() {
    for (bytes, expected) in [
        (&b"!<tag:yaml.org,2002:str> foo\n"[..], vec![&b"foo"[..]]),
        (
            &b"tagged: !<tag:yaml.org,2002:str> foo\n"[..],
            vec![&b"tagged"[..], &b"foo"[..]],
        ),
        (
            &b"tagged: !<tag:example.test/a[b],c> value\n"[..],
            vec![&b"tagged"[..], &b"value"[..]],
        ),
    ] {
        let (_, _, _, _, plain) = scan(bytes);
        assert_eq!(raw_ranges(bytes, &plain), expected, "fixture: {bytes:?}");
    }
}

#[test]
fn flow_mapping_colons_are_excluded_even_when_structurally_coalesced() {
    for (bytes, expected) in [
        (&b"flow: {\"a\":b}\n"[..], vec![&b"flow"[..], &b"b"[..]]),
        (&b"flow: {a:}\n"[..], vec![&b"flow"[..], &b"a"[..]]),
        (
            &b"flow: {url: http://example.test}\n"[..],
            vec![&b"flow"[..], &b"url"[..], &b"http://example.test"[..]],
        ),
    ] {
        let (_, _, _, _, plain) = scan(bytes);
        assert_eq!(raw_ranges(bytes, &plain), expected, "fixture: {bytes:?}");
    }
}

#[test]
fn tab_only_prefix_cannot_bypass_contextual_indentation_validation() {
    for bytes in [&b"\t[foo]\n"[..], &b"\t{a: b}\n"[..], &b"\t\"foo\"\n"[..]] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let quotes = quoted(&atoms, &lines, &candidates);
        let error = scan_profile1_plain_scalars(
            &atoms,
            &lines,
            &candidates,
            &quotes,
            PlainScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("a tab cannot supply required root indentation");
        assert_eq!(error.kind(), PlainScalarErrorKind::TabInIndentation);
        assert_eq!(error.byte_offset(), 0, "fixture: {bytes:?}");
    }
}

#[test]
fn digits_inside_block_header_comments_do_not_set_indentation() {
    let bytes = b"key: | # comment 9\n  text\nnext: ok\n";
    let (_, _, _, _, plain) = scan(bytes);
    assert_eq!(
        raw_ranges(bytes, &plain),
        vec![&b"key"[..], &b"next"[..], &b"ok"[..]]
    );
}

#[test]
fn flow_unsafe_plain_initial_indicators_have_typed_exact_errors() {
    for (bytes, offset) in [
        (&b"flow: [:]\n"[..], 7),
        (&b"flow: [?]\n"[..], 7),
        (&b"flow: [-]\n"[..], 7),
    ] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let quotes = quoted(&atoms, &lines, &candidates);
        let error = scan_profile1_plain_scalars(
            &atoms,
            &lines,
            &candidates,
            &quotes,
            PlainScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("the plain start lacks a flow-safe lookahead");
        assert_eq!(error.kind(), PlainScalarErrorKind::InvalidPlainStart);
        assert_eq!(error.byte_offset(), offset, "fixture: {bytes:?}");
    }

    for bytes in [&b"[:]\n"[..], &b"[?]\n"[..], &b"[-]\n"[..]] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let quotes = quoted(&atoms, &lines, &candidates);
        let zero_cap_error = scan_profile1_plain_scalars(
            &atoms,
            &lines,
            &candidates,
            &quotes,
            PlainScalarScanLimits::new(0, 0),
        )
        .expect_err("invalid syntax takes precedence over a zero scalar cap");
        assert_eq!(
            zero_cap_error.kind(),
            PlainScalarErrorKind::InvalidPlainStart
        );
        assert_eq!(zero_cap_error.byte_offset(), 1, "fixture: {bytes:?}");
    }
}

#[test]
fn reserved_starts_and_raw_forbidden_characters_have_typed_exact_offsets() {
    for (bytes, kind, offset) in [
        (
            &b"key: @reserved\n"[..],
            PlainScalarErrorKind::ReservedIndicator,
            5,
        ),
        (
            &b"key: `reserved\n"[..],
            PlainScalarErrorKind::ReservedIndicator,
            5,
        ),
        (
            &b"key: raw\0value\n"[..],
            PlainScalarErrorKind::InvalidPlainCharacter,
            8,
        ),
        (
            &b"key: raw\x1fvalue\n"[..],
            PlainScalarErrorKind::InvalidPlainCharacter,
            8,
        ),
    ] {
        let atoms = atomize(bytes);
        let lines = layout(&atoms);
        let candidates = structural(&atoms, &lines);
        let quotes = quoted(&atoms, &lines, &candidates);
        let error = scan_profile1_plain_scalars(
            &atoms,
            &lines,
            &candidates,
            &quotes,
            PlainScalarScanLimits::new(u64::MAX, u64::MAX),
        )
        .expect_err("fixture has a typed plain-scanner failure");
        assert_eq!(error.kind(), kind, "fixture: {bytes:?}");
        assert_eq!(error.byte_offset(), offset, "fixture: {bytes:?}");
    }
}

#[test]
fn count_and_atom_caps_are_exact_all_or_error_boundaries() {
    assert_eq!(MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS);
    assert_eq!(MAX_PROFILE1_PLAIN_SCALAR_ATOMS, MAX_PROFILE1_LEXICAL_ATOMS);
    let bytes = b"first: second";
    let atoms = atomize(bytes);
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);
    let quotes = quoted(&atoms, &lines, &candidates);

    let count_error = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        PlainScalarScanLimits::new(1, u64::MAX),
    )
    .expect_err("second scalar exceeds the count cap");
    assert_eq!(
        count_error.kind(),
        PlainScalarErrorKind::ScalarLimitExceeded
    );
    assert_eq!(count_error.byte_offset(), 7);

    let atom_error = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        PlainScalarScanLimits::new(2, 5),
    )
    .expect_err("second has six atoms");
    assert_eq!(
        atom_error.kind(),
        PlainScalarErrorKind::ScalarAtomLimitExceeded
    );
    assert_eq!(atom_error.byte_offset(), 12);

    let exact = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &quotes,
        PlainScalarScanLimits::new(2, 6),
    )
    .expect("both exact limits are admitted");
    assert_eq!(exact.scalars().len(), 2);
}

#[test]
fn bom_metadata_and_quoted_authentication_are_exact() {
    let bytes = b"\xef\xbb\xbfkey: value";
    let (_, _, _, _, plain) = scan(bytes);
    assert_eq!(plain.bom_bytes(), 3);
    assert_eq!(plain.scalars()[0].byte_start(), 3);

    let atoms = atomize(b"key: value\n");
    let lines = layout(&atoms);
    let candidates = structural(&atoms, &lines);
    let other_atoms = atomize(b"other: \"quoted\"\n");
    let other_lines = layout(&other_atoms);
    let other_candidates = structural(&other_atoms, &other_lines);
    let other_quotes = quoted(&other_atoms, &other_lines, &other_candidates);
    let error = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &candidates,
        &other_quotes,
        PlainScalarScanLimits::new(u64::MAX, u64::MAX),
    )
    .expect_err("quoted evidence is not canonical for this source");
    assert_eq!(error.kind(), PlainScalarErrorKind::InputQuotedMismatch);
    assert_eq!(error.byte_offset(), 0);
}
