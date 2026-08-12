use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, decode_profile1, scan_profile1_structural_lexemes,
    AtomizeLimits, AtomizedSource, BomPolicy, DecodeLimits, LayoutLimits, LayoutSource,
    StructuralCandidateRole as StructuralLexemeKind, StructuralLexeme, StructuralScanErrorKind,
    StructuralScanLimits, YamlIndicator, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_INDENTATION_COLUMNS, MAX_PROFILE1_LAYOUT_LINES, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_STRUCTURAL_LEXEMES,
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
    analyze_profile1_layout(
        source,
        LayoutLimits::new(MAX_PROFILE1_LAYOUT_LINES, MAX_PROFILE1_INDENTATION_COLUMNS),
    )
    .expect("source has valid bounded line layout")
}

fn scan(
    input: &[u8],
) -> (
    AtomizedSource,
    LayoutSource,
    crucible_yaml::StructuralLexemeSource,
) {
    let atoms = atomize(input);
    let lines = layout(&atoms);
    let lexemes = scan_profile1_structural_lexemes(
        &atoms,
        &lines,
        StructuralScanLimits::new(MAX_PROFILE1_STRUCTURAL_LEXEMES),
    )
    .expect("source has a valid bounded structural partition");
    (atoms, lines, lexemes)
}

fn fact(lexeme: &StructuralLexeme) -> (StructuralLexemeKind, u64, u64, u64, u64, u64) {
    (
        lexeme.candidate_role(),
        lexeme.line_number(),
        lexeme.start_atom_index(),
        lexeme.end_atom_index(),
        lexeme.byte_start(),
        lexeme.byte_end(),
    )
}

#[test]
fn directives_markers_comments_flow_and_content_have_exact_lossless_ranges() {
    let bytes = b"\xef\xbb\xbf%YAML 1.2\r\n---\nkey: [a, b] # note\n...\n";
    let (atoms, lines, scanned) = scan(bytes);

    assert_eq!(scanned.profile_version(), 1);
    assert_eq!(scanned.input_transformation_version(), 1);
    assert_eq!(scanned.layout_transformation_version(), 1);
    assert_eq!(scanned.transformation_version(), 1);
    assert_eq!(scanned.source_len_bytes(), bytes.len() as u64);
    assert_eq!(scanned.bom_bytes(), 3);
    assert_eq!(scanned.input_atom_count(), atoms.atoms().len() as u64);
    assert_eq!(scanned.input_line_count(), lines.lines().len() as u64);
    assert_eq!(
        scanned.lexemes().iter().map(fact).collect::<Vec<_>>(),
        vec![
            (StructuralLexemeKind::Directive, 0, 0, 9, 3, 12),
            (StructuralLexemeKind::LineFeed, 0, 9, 10, 12, 14),
            (StructuralLexemeKind::DocumentStart, 1, 10, 13, 14, 17),
            (StructuralLexemeKind::LineFeed, 1, 13, 14, 17, 18),
            (StructuralLexemeKind::Content, 2, 14, 17, 18, 21),
            (
                StructuralLexemeKind::Indicator(YamlIndicator::MappingValue),
                2,
                17,
                18,
                21,
                22,
            ),
            (StructuralLexemeKind::Separation, 2, 18, 19, 22, 23),
            (StructuralLexemeKind::FlowSequenceStart, 2, 19, 20, 23, 24),
            (StructuralLexemeKind::Content, 2, 20, 21, 24, 25),
            (StructuralLexemeKind::FlowEntry, 2, 21, 22, 25, 26),
            (StructuralLexemeKind::Separation, 2, 22, 23, 26, 27),
            (StructuralLexemeKind::Content, 2, 23, 24, 27, 28),
            (StructuralLexemeKind::FlowSequenceEnd, 2, 24, 25, 28, 29),
            (StructuralLexemeKind::Separation, 2, 25, 26, 29, 30),
            (StructuralLexemeKind::Comment, 2, 26, 32, 30, 36),
            (StructuralLexemeKind::LineFeed, 2, 32, 33, 36, 37),
            (StructuralLexemeKind::DocumentEnd, 3, 33, 36, 37, 40),
            (StructuralLexemeKind::LineFeed, 3, 36, 37, 40, 41),
        ]
    );
}

#[test]
fn comment_and_indicator_recognition_uses_yaml_lookaround_context() {
    let (_, _, scanned) = scan(b"url: https://x/#frag # comment\n");
    assert_eq!(
        scanned.lexemes().iter().map(fact).collect::<Vec<_>>(),
        vec![
            (StructuralLexemeKind::Content, 0, 0, 3, 0, 3),
            (
                StructuralLexemeKind::Indicator(YamlIndicator::MappingValue),
                0,
                3,
                4,
                3,
                4,
            ),
            (StructuralLexemeKind::Separation, 0, 4, 5, 4, 5),
            (StructuralLexemeKind::Content, 0, 5, 20, 5, 20),
            (StructuralLexemeKind::Separation, 0, 20, 21, 20, 21),
            (StructuralLexemeKind::Comment, 0, 21, 30, 21, 30),
            (StructuralLexemeKind::LineFeed, 0, 30, 31, 30, 31),
        ]
    );

    let (_, _, markers) = scan(b" ---\n---x\n...x\n");
    assert_eq!(
        markers.lexemes().iter().map(fact).collect::<Vec<_>>(),
        vec![
            (StructuralLexemeKind::Indentation, 0, 0, 1, 0, 1),
            (StructuralLexemeKind::Content, 0, 1, 4, 1, 4),
            (StructuralLexemeKind::LineFeed, 0, 4, 5, 4, 5),
            (StructuralLexemeKind::Content, 1, 5, 9, 5, 9),
            (StructuralLexemeKind::LineFeed, 1, 9, 10, 9, 10),
            (StructuralLexemeKind::Content, 2, 10, 14, 10, 14),
            (StructuralLexemeKind::LineFeed, 2, 14, 15, 14, 15),
        ]
    );
}

#[test]
fn block_content_tabs_and_multibyte_content_are_preserved_without_reclassification() {
    let (atoms, _, scanned) = scan("block: |\n  \tprintf(β)\n".as_bytes());
    assert_eq!(
        scanned.lexemes().iter().map(fact).collect::<Vec<_>>(),
        vec![
            (StructuralLexemeKind::Content, 0, 0, 5, 0, 5),
            (
                StructuralLexemeKind::Indicator(YamlIndicator::MappingValue),
                0,
                5,
                6,
                5,
                6,
            ),
            (StructuralLexemeKind::Separation, 0, 6, 7, 6, 7),
            (
                StructuralLexemeKind::Indicator(YamlIndicator::LiteralBlockScalar),
                0,
                7,
                8,
                7,
                8,
            ),
            (StructuralLexemeKind::LineFeed, 0, 8, 9, 8, 9),
            (StructuralLexemeKind::Indentation, 1, 9, 11, 9, 11),
            (StructuralLexemeKind::Content, 1, 11, 21, 11, 22),
            (StructuralLexemeKind::LineFeed, 1, 21, 22, 22, 23),
        ]
    );
    assert_eq!(atoms.atoms()[11].code_point(), u32::from(b'\t'));
}

#[test]
fn lexeme_caps_are_all_or_error_at_the_first_excluded_range() {
    assert_eq!(MAX_PROFILE1_STRUCTURAL_LEXEMES, MAX_PROFILE1_LEXICAL_ATOMS);
    let atoms = atomize(b"a b\n");
    let lines = layout(&atoms);

    let error = scan_profile1_structural_lexemes(&atoms, &lines, StructuralScanLimits::new(2))
        .expect_err("the third lexeme exceeds the caller-lowered cap");
    assert_eq!(error.kind(), StructuralScanErrorKind::LexemeLimitExceeded);
    assert_eq!(error.byte_offset(), 2);

    let accepted = scan_profile1_structural_lexemes(&atoms, &lines, StructuralScanLimits::new(4))
        .expect("the exact four-lexeme boundary is accepted");
    assert_eq!(accepted.lexemes().len(), 4);

    let empty_atoms = atomize(b"");
    let empty_layout = layout(&empty_atoms);
    let empty =
        scan_profile1_structural_lexemes(&empty_atoms, &empty_layout, StructuralScanLimits::new(0))
            .expect("an empty source needs no lexeme capacity");
    assert!(empty.lexemes().is_empty());

    let bom_atoms = atomize(b"\xef\xbb\xbfa");
    let bom_layout = layout(&bom_atoms);
    let bom_error =
        scan_profile1_structural_lexemes(&bom_atoms, &bom_layout, StructuralScanLimits::new(0))
            .expect_err("the first post-BOM atom exceeds a zero lexeme cap");
    assert_eq!(
        bom_error.kind(),
        StructuralScanErrorKind::LexemeLimitExceeded
    );
    assert_eq!(bom_error.byte_offset(), 3);

    let bom_only_atoms = atomize(b"\xef\xbb\xbf");
    let bom_only_layout = layout(&bom_only_atoms);
    let bom_only = scan_profile1_structural_lexemes(
        &bom_only_atoms,
        &bom_only_layout,
        StructuralScanLimits::new(0),
    )
    .expect("a stripped-BOM-only source needs no lexeme capacity");
    assert_eq!(bom_only.bom_bytes(), 3);
    assert!(bom_only.lexemes().is_empty());

    let multibyte_atoms = atomize("α β".as_bytes());
    let multibyte_layout = layout(&multibyte_atoms);
    let multibyte_error = scan_profile1_structural_lexemes(
        &multibyte_atoms,
        &multibyte_layout,
        StructuralScanLimits::new(2),
    )
    .expect_err("the beta content candidate is the first excluded lexeme");
    assert_eq!(
        multibyte_error.kind(),
        StructuralScanErrorKind::LexemeLimitExceeded
    );
    assert_eq!(multibyte_error.byte_offset(), 3);
}

#[test]
fn all_space_lines_have_exact_terminated_and_unterminated_candidate_ranges() {
    let (_, _, terminated) = scan(b"   \n");
    assert_eq!(
        terminated.lexemes().iter().map(fact).collect::<Vec<_>>(),
        vec![
            (StructuralLexemeKind::Indentation, 0, 0, 3, 0, 3),
            (StructuralLexemeKind::LineFeed, 0, 3, 4, 3, 4),
        ],
    );

    let (_, _, unterminated) = scan(b"   ");
    assert_eq!(
        unterminated.lexemes().iter().map(fact).collect::<Vec<_>>(),
        vec![(StructuralLexemeKind::Indentation, 0, 0, 3, 0, 3)],
    );
}

#[test]
fn candidate_roles_retain_every_quoted_and_block_scalar_byte_for_contextual_scanning() {
    let bytes = b"quoted: \"a, # b\"\nblock: |\n  [x, # y]\n";
    let (atoms, _, scanned) = scan(bytes);

    let reconstructed = scanned
        .lexemes()
        .iter()
        .flat_map(|lexeme| {
            bytes[lexeme.byte_start() as usize..lexeme.byte_end() as usize]
                .iter()
                .copied()
        })
        .collect::<Vec<_>>();
    assert_eq!(reconstructed, bytes);
    assert_eq!(
        scanned.lexemes().first().map(StructuralLexeme::byte_start),
        Some(0)
    );
    assert_eq!(
        scanned.lexemes().last().map(StructuralLexeme::byte_end),
        Some(bytes.len() as u64)
    );
    assert_eq!(
        scanned
            .lexemes()
            .iter()
            .map(|lexeme| lexeme.end_atom_index() - lexeme.start_atom_index())
            .sum::<u64>(),
        atoms.atoms().len() as u64,
    );
    assert!(scanned.lexemes().iter().any(|lexeme| {
        lexeme.candidate_role() == StructuralLexemeKind::Comment
            && &bytes[lexeme.byte_start() as usize..lexeme.byte_end() as usize] == b"# b\""
    }));
    assert!(scanned.lexemes().iter().any(|lexeme| {
        lexeme.candidate_role() == StructuralLexemeKind::FlowEntry
            && &bytes[lexeme.byte_start() as usize..lexeme.byte_end() as usize] == b","
    }));
}

#[test]
fn a_layout_from_a_different_atom_stream_is_rejected_before_scanning() {
    let atoms = atomize(b"x\n");
    let other_atoms = atomize(b" y\n");
    let other_layout = layout(&other_atoms);

    let error = scan_profile1_structural_lexemes(
        &atoms,
        &other_layout,
        StructuralScanLimits::new(MAX_PROFILE1_STRUCTURAL_LEXEMES),
    )
    .expect_err("the line layout is not canonical for this atom stream");
    assert_eq!(error.kind(), StructuralScanErrorKind::InputLayoutMismatch);
    assert_eq!(error.byte_offset(), 0);
}

#[test]
fn structural_partition_is_deterministic_monotonic_and_covers_every_atom_once() {
    let mut bytes = Vec::new();
    for line in 0..4096u64 {
        bytes.extend(std::iter::repeat_n(b' ', (line % 11) as usize));
        if line % 17 == 0 {
            bytes.extend_from_slice(b"--- # document");
        } else {
            bytes.extend_from_slice(b"key: [https://x/#frag, value] # comment");
        }
        bytes.push(b'\n');
    }
    let (atoms, lines, first) = scan(&bytes);
    let second =
        scan_profile1_structural_lexemes(&atoms, &lines, StructuralScanLimits::new(u64::MAX))
            .expect("deterministic rescan");
    assert_eq!(first, second);

    let mut next_atom = 0u64;
    let mut next_byte = 0u64;
    for lexeme in first.lexemes() {
        assert_eq!(lexeme.start_atom_index(), next_atom);
        assert_eq!(lexeme.byte_start(), next_byte);
        assert!(lexeme.start_atom_index() < lexeme.end_atom_index());
        assert!(lexeme.byte_start() < lexeme.byte_end());
        next_atom = lexeme.end_atom_index();
        next_byte = lexeme.byte_end();
    }
    assert_eq!(next_atom, atoms.atoms().len() as u64);
    assert_eq!(next_byte, atoms.source_len_bytes());
}
