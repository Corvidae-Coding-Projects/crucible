use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, decode_profile1, AtomizeLimits, AtomizedSource,
    BomPolicy, DecodeLimits, LayoutErrorKind, LayoutLimits, LayoutLine, LayoutSource,
    LexicalAtomKind, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_INDENTATION_COLUMNS,
    MAX_PROFILE1_LAYOUT_LINES, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
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

fn line_fact(line: &LayoutLine) -> (u64, u64, u64, u64, bool, u64, u64, u64, u64) {
    (
        line.line_number(),
        line.start_atom_index(),
        line.content_atom_index(),
        line.end_atom_index(),
        line.is_terminated(),
        line.indentation_columns(),
        line.byte_start(),
        line.content_byte_start(),
        line.byte_end(),
    )
}

#[test]
fn lines_preserve_atom_ranges_indentation_and_original_byte_offsets() {
    let bytes = b"\xef\xbb\xbf  key\r\n\n    tail";
    let atomized = atomize(bytes);
    let analyzed = layout(&atomized);

    assert_eq!(analyzed.profile_version(), 1);
    assert_eq!(analyzed.input_transformation_version(), 1);
    assert_eq!(analyzed.transformation_version(), 1);
    assert_eq!(analyzed.source_len_bytes(), 19);
    assert_eq!(analyzed.bom_bytes(), 3);
    assert_eq!(analyzed.lines().len(), 3);
    assert_eq!(
        analyzed.lines().iter().map(line_fact).collect::<Vec<_>>(),
        vec![
            (0, 0, 2, 5, true, 2, 3, 5, 10),
            (1, 6, 6, 6, true, 0, 10, 10, 11),
            (2, 7, 11, 15, false, 4, 11, 15, 19),
        ]
    );
}

#[test]
fn leading_tabs_are_preserved_for_contextual_lexing_and_lowered_indent_caps_are_exact() {
    for (bytes, expected_indentation, expected_content_atom) in [
        (&b"\tkey"[..], 0, 0),
        (&b" \tkey"[..], 1, 1),
        (&b"\n  \t"[..], 2, 3),
    ] {
        let atomized = atomize(bytes);
        let analyzed = layout(&atomized);
        let line = analyzed.lines().last().expect("non-phantom tab line");
        assert_eq!(line.indentation_columns(), expected_indentation);
        assert_eq!(line.content_atom_index(), expected_content_atom);
        assert_eq!(
            atomized.atoms()[expected_content_atom as usize].kind(),
            LexicalAtomKind::Tab
        );
    }

    let block_scalar = atomize(b"block: |\n  \tprintf\n");
    let analyzed = layout(&block_scalar);
    assert_eq!(analyzed.lines().len(), 2);
    assert_eq!(
        line_fact(&analyzed.lines()[1]),
        (1, 9, 11, 18, true, 2, 9, 11, 19)
    );
    assert_eq!(block_scalar.atoms()[11].kind(), LexicalAtomKind::Tab);

    let body_tab = atomize(b"key\tvalue");
    assert_eq!(layout(&body_tab).lines()[0].indentation_columns(), 0);

    let three_spaces = atomize(b"   x");
    let error = analyze_profile1_layout(&three_spaces, LayoutLimits::new(1, 2))
        .expect_err("the third indentation space exceeds a two-column caller cap");
    assert_eq!(error.kind(), LayoutErrorKind::IndentationLimitExceeded);
    assert_eq!(error.byte_offset(), 2);

    let exact = atomize(b"  x");
    assert_eq!(
        analyze_profile1_layout(&exact, LayoutLimits::new(1, 2))
            .expect("the exact indentation cap")
            .lines()[0]
            .indentation_columns(),
        2
    );
}

#[test]
fn indentation_boundaries_multibyte_offsets_and_nonphantom_tails_are_exact() {
    let exact_bytes = vec![b' '; MAX_PROFILE1_INDENTATION_COLUMNS as usize];
    let exact = atomize(&exact_bytes);
    let analyzed = analyze_profile1_layout(&exact, LayoutLimits::new(u64::MAX, u64::MAX))
        .expect("the absolute indentation boundary is accepted");
    assert_eq!(analyzed.lines()[0].indentation_columns(), 4096);
    assert_eq!(analyzed.lines()[0].content_atom_index(), 4096);

    let exceeded_bytes = vec![b' '; MAX_PROFILE1_INDENTATION_COLUMNS as usize + 1];
    let exceeded = atomize(&exceeded_bytes);
    let error = analyze_profile1_layout(&exceeded, LayoutLimits::new(u64::MAX, u64::MAX))
        .expect_err("the first space beyond the absolute indentation cap is rejected");
    assert_eq!(error.kind(), LayoutErrorKind::IndentationLimitExceeded);
    assert_eq!(error.byte_offset(), 4096);

    let multibyte = atomize("é\n  β".as_bytes());
    assert_eq!(
        layout(&multibyte)
            .lines()
            .iter()
            .map(line_fact)
            .collect::<Vec<_>>(),
        vec![
            (0, 0, 0, 1, true, 0, 0, 0, 3),
            (1, 2, 4, 5, false, 2, 3, 5, 7)
        ]
    );
    let multibyte_error = atomize("é\n   β".as_bytes());
    let error = analyze_profile1_layout(&multibyte_error, LayoutLimits::new(2, 2))
        .expect_err("the multibyte prefix does not distort the exact byte error offset");
    assert_eq!(error.kind(), LayoutErrorKind::IndentationLimitExceeded);
    assert_eq!(error.byte_offset(), 5);

    let bom_only = layout(&atomize(b"\xef\xbb\xbf"));
    assert_eq!(bom_only.source_len_bytes(), 3);
    assert_eq!(bom_only.bom_bytes(), 3);
    assert!(bom_only.lines().is_empty());

    let all_spaces = layout(&atomize(b"   "));
    assert_eq!(
        all_spaces.lines().iter().map(line_fact).collect::<Vec<_>>(),
        vec![(0, 0, 3, 3, false, 3, 0, 3, 3)]
    );
}

#[test]
fn line_caps_reject_at_the_first_atom_of_the_first_excluded_line() {
    let empty = atomize(b"");
    assert!(analyze_profile1_layout(&empty, LayoutLimits::new(0, 0))
        .expect("empty input contains no layout lines")
        .lines()
        .is_empty());

    for (bytes, maximum_lines, expected_offset) in [
        (&b"a"[..], 0, 0),
        (&b"\n\n"[..], 1, 1),
        (&b"a\nb\nc"[..], 2, 4),
        (&b"\xef\xbb\xbfx"[..], 0, 3),
    ] {
        let atomized = atomize(bytes);
        let error = analyze_profile1_layout(
            &atomized,
            LayoutLimits::new(maximum_lines, MAX_PROFILE1_INDENTATION_COLUMNS),
        )
        .expect_err("source exceeds its caller-lowered line cap");
        assert_eq!(error.kind(), LayoutErrorKind::LineLimitExceeded);
        assert_eq!(error.byte_offset(), expected_offset);
    }

    let one_terminated_line = atomize(b"\n");
    let accepted = analyze_profile1_layout(&one_terminated_line, LayoutLimits::new(1, 0))
        .expect("one line terminated by LF");
    assert_eq!(accepted.lines().len(), 1);
    assert!(accepted.lines()[0].is_terminated());
}

#[test]
fn absolute_caps_are_fixed_and_the_maximum_line_boundary_is_executable() {
    assert_eq!(MAX_PROFILE1_LAYOUT_LINES, 1024 * 1024);
    assert_eq!(MAX_PROFILE1_LAYOUT_LINES, MAX_PROFILE1_LEXICAL_ATOMS);
    assert_eq!(MAX_PROFILE1_INDENTATION_COLUMNS, 4096);

    let bytes = vec![b'\n'; MAX_PROFILE1_LAYOUT_LINES as usize];
    let atomized = atomize(&bytes);
    let analyzed = analyze_profile1_layout(&atomized, LayoutLimits::new(u64::MAX, u64::MAX))
        .expect("the absolute line boundary is accepted");
    assert_eq!(analyzed.lines().len() as u64, MAX_PROFILE1_LAYOUT_LINES);
    assert_eq!(
        line_fact(analyzed.lines().last().expect("maximum final line")),
        (
            MAX_PROFILE1_LAYOUT_LINES - 1,
            MAX_PROFILE1_LAYOUT_LINES - 1,
            MAX_PROFILE1_LAYOUT_LINES - 1,
            MAX_PROFILE1_LAYOUT_LINES - 1,
            true,
            0,
            MAX_PROFILE1_LAYOUT_LINES - 1,
            MAX_PROFILE1_LAYOUT_LINES - 1,
            MAX_PROFILE1_LAYOUT_LINES,
        )
    );
}

#[test]
fn generated_layout_is_deterministic_monotonic_and_covers_every_atom_once() {
    let mut bytes = Vec::new();
    for line_number in 0..4096u64 {
        bytes.extend(std::iter::repeat_n(b' ', (line_number % 17) as usize));
        bytes.extend_from_slice(b"key\t#[]{}value");
        match line_number % 3 {
            0 => bytes.push(b'\n'),
            1 => bytes.push(b'\r'),
            _ => bytes.extend_from_slice(b"\r\n"),
        }
    }
    bytes.extend_from_slice(b"final");

    let atomized = atomize(&bytes);
    let first = layout(&atomized);
    let second = layout(&atomized);
    assert_eq!(first, second);
    assert_eq!(first.lines().len(), 4097);

    let mut next_atom = 0u64;
    let mut next_byte = 0u64;
    for (expected_line, line) in first.lines().iter().enumerate() {
        assert_eq!(line.line_number(), expected_line as u64);
        assert_eq!(line.start_atom_index(), next_atom);
        assert_eq!(line.byte_start(), next_byte);
        assert!(line.start_atom_index() <= line.content_atom_index());
        assert!(line.content_atom_index() <= line.end_atom_index());

        for atom in
            &atomized.atoms()[line.start_atom_index() as usize..line.content_atom_index() as usize]
        {
            assert_eq!(atom.kind(), LexicalAtomKind::Space);
        }

        next_atom = line.end_atom_index() + u64::from(line.is_terminated());
        next_byte = line.byte_end();
    }
    assert_eq!(next_atom, atomized.atoms().len() as u64);
    assert_eq!(next_byte, atomized.source_len_bytes());
}
