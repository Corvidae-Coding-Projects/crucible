use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy,
    CompletedTokenErrorKind, CompletedTokenKind, CompletedTokenLimits, CompletedTokenPartKind,
    CompletedTokenSource, DecodeLimits, LayoutSource, PlainScalarErrorKind, PlainScalarSource,
    QuotedScalarSource, StructuralLexemeSource, MAX_PROFILE1_COMPLETED_TOKENS,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_FLOW_DEPTH, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES,
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

fn upstream(
    input: &[u8],
) -> (
    AtomizedSource,
    LayoutSource,
    StructuralLexemeSource,
    QuotedScalarSource,
    PlainScalarSource,
    BlockScalarSource,
) {
    let atoms = atomize(input);
    let lines = analyze_profile1_layout(&atoms, canonical_structural_layout_limits())
        .expect("canonical line layout");
    let structural =
        scan_profile1_structural_lexemes(&atoms, &lines, canonical_structural_scan_limits())
            .expect("canonical structural candidates");
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .expect("canonical quoted scalars");
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .expect("canonical plain scalars");
    let block = scan_profile1_block_scalars(
        &atoms,
        &lines,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    )
    .expect("canonical block scalars");
    (atoms, lines, structural, quoted, plain, block)
}

fn scan(input: &[u8]) -> (AtomizedSource, CompletedTokenSource) {
    let (atoms, lines, structural, quoted, plain, block) = upstream(input);
    let tokens = scan_profile1_completed_tokens(
        &atoms,
        &lines,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    )
    .expect("valid completed token stream");
    (atoms, tokens)
}

fn kinds(tokens: &CompletedTokenSource) -> Vec<CompletedTokenKind> {
    tokens.tokens().iter().map(|token| token.kind()).collect()
}

fn raw<'a>(input: &'a [u8], tokens: &CompletedTokenSource, index: usize) -> &'a [u8] {
    let token = &tokens.tokens()[index];
    &input[token.byte_start() as usize..token.byte_end() as usize]
}

#[test]
fn mixed_stream_has_complete_final_kinds_and_an_exact_lossless_partition() {
    let input = b"%YAML 1.2 # version\n%TAG !e! tag:example.com,2000:app/\n---\nroot: &a !e!node\n  flow: [\"x,#\", *a]\n  literal: |-\n    raw # []\n...\n";
    let (atoms, tokens) = scan(input);

    assert_eq!(tokens.profile_version(), 1);
    assert_eq!(tokens.transformation_version(), 1);
    assert_eq!(tokens.input_atom_count(), atoms.atoms().len() as u64);
    assert!(kinds(&tokens).contains(&CompletedTokenKind::YamlDirective));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::TagDirective));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::DirectivesEnd));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::AnchorProperty));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::TagProperty));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::FlowSequenceStart));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::DoubleQuotedScalar));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::Alias));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::LiteralBlockScalar));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::DocumentEnd));

    let mut next_atom = 0;
    let mut next_byte = atoms.bom_bytes();
    let mut reconstructed = Vec::new();
    for token in tokens.tokens() {
        assert_eq!(token.start_atom_index(), next_atom);
        assert_eq!(token.byte_start(), next_byte);
        assert!(token.start_atom_index() < token.end_atom_index());
        assert!(token.byte_start() < token.byte_end());
        reconstructed
            .extend_from_slice(&input[token.byte_start() as usize..token.byte_end() as usize]);
        next_atom = token.end_atom_index();
        next_byte = token.byte_end();
    }
    assert_eq!(next_atom, atoms.atoms().len() as u64);
    assert_eq!(next_byte, input.len() as u64);
    assert_eq!(reconstructed, input);
}

#[test]
fn directive_property_alias_and_tag_parts_retain_exact_payload_ranges() {
    let input = b"%YAML 12.34\n%TAG !e! tag:example.com,2000:app/\n%FUTURE one two\n---\n&e !<tag:yaml.org,2002:str> !e!node *e\n";
    let (_, tokens) = scan(input);

    let yaml = tokens
        .tokens()
        .iter()
        .find(|token| token.kind() == CompletedTokenKind::YamlDirective)
        .expect("YAML directive");
    assert_eq!(yaml.yaml_version(), Some((12, 34)));
    assert_eq!(raw(input, &tokens, 0), b"%YAML 12.34");
    assert_eq!(
        yaml.parts()
            .iter()
            .map(|part| part.kind())
            .collect::<Vec<_>>(),
        vec![
            CompletedTokenPartKind::DirectiveName,
            CompletedTokenPartKind::YamlMajor,
            CompletedTokenPartKind::YamlMinor,
        ]
    );

    let tag_directive = tokens
        .tokens()
        .iter()
        .find(|token| token.kind() == CompletedTokenKind::TagDirective)
        .expect("TAG directive");
    assert_eq!(tag_directive.parts().len(), 3);
    let tag_raw = &input[tag_directive.byte_start() as usize..tag_directive.byte_end() as usize];
    assert_eq!(tag_raw, b"%TAG !e! tag:example.com,2000:app/");

    let reserved = tokens
        .tokens()
        .iter()
        .find(|token| token.kind() == CompletedTokenKind::ReservedDirective)
        .expect("reserved directive");
    assert_eq!(reserved.parts().len(), 3);
    assert_eq!(
        reserved
            .parts()
            .iter()
            .map(|part| part.kind())
            .collect::<Vec<_>>(),
        vec![
            CompletedTokenPartKind::DirectiveName,
            CompletedTokenPartKind::DirectiveParameter,
            CompletedTokenPartKind::DirectiveParameter,
        ]
    );

    for (kind, expected) in [
        (CompletedTokenKind::AnchorProperty, &b"&e"[..]),
        (
            CompletedTokenKind::VerbatimTagProperty,
            &b"!<tag:yaml.org,2002:str>"[..],
        ),
        (CompletedTokenKind::TagProperty, &b"!e!node"[..]),
        (CompletedTokenKind::Alias, &b"*e"[..]),
    ] {
        let token = tokens
            .tokens()
            .iter()
            .find(|token| token.kind() == kind)
            .expect("expected property token");
        assert_eq!(
            &input[token.byte_start() as usize..token.byte_end() as usize],
            expected
        );
        assert!(!token.parts().is_empty());
    }
}

#[test]
fn scalar_internal_punctuation_and_document_like_text_never_leak_into_outer_tokens() {
    let input = b"plain: a # comment\nquoted: \"[,] # --- ...\"\nblock: |\n  %TAG !x! y,[] # raw\nnext: done\n";
    let (_, tokens) = scan(input);
    assert_eq!(
        tokens
            .tokens()
            .iter()
            .filter(|token| token.kind() == CompletedTokenKind::Comment)
            .count(),
        1
    );
    assert_eq!(
        tokens
            .tokens()
            .iter()
            .filter(|token| token.kind() == CompletedTokenKind::FlowEntry)
            .count(),
        0
    );
    assert_eq!(
        tokens
            .tokens()
            .iter()
            .filter(|token| matches!(
                token.kind(),
                CompletedTokenKind::YamlDirective | CompletedTokenKind::TagDirective
            ))
            .count(),
        0
    );
}

#[test]
fn flow_nesting_is_typed_balanced_and_caller_bounded() {
    let (_, balanced) = scan(b"root: [{a: [b, c]}]\n");
    assert_eq!(balanced.maximum_flow_depth(), 3);

    for (input, kind, offset) in [
        (&b"[a}"[..], CompletedTokenErrorKind::MismatchedFlowEnd, 2),
        (
            &b"[a"[..],
            CompletedTokenErrorKind::UnclosedFlowCollection,
            2,
        ),
    ] {
        let (atoms, lines, structural, quoted, plain, block) = upstream(input);
        let error = scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("invalid flow nesting");
        assert_eq!(error.kind(), kind, "fixture: {input:?}");
        assert_eq!(error.byte_offset(), offset, "fixture: {input:?}");
    }

    let input = b"[[x]]";
    let (atoms, lines, structural, quoted, plain, block) = upstream(input);
    let error = scan_profile1_completed_tokens(
        &atoms,
        &lines,
        &structural,
        &quoted,
        &plain,
        &block,
        CompletedTokenLimits::new(u64::MAX, 1),
    )
    .expect_err("the second opener exceeds the caller flow-depth bound");
    assert_eq!(
        error.kind(),
        CompletedTokenErrorKind::FlowDepthLimitExceeded
    );
    assert_eq!(error.byte_offset(), 1);
}

#[test]
fn malformed_directives_properties_and_aliases_report_the_first_exact_bad_offset() {
    for (input, kind, offset) in [
        (
            &b"%YAML one\n---\n"[..],
            CompletedTokenErrorKind::InvalidYamlDirective,
            6,
        ),
        (
            &b"%TAG !e!\n---\n"[..],
            CompletedTokenErrorKind::InvalidTagDirective,
            8,
        ),
        (
            &b"---\n& value\n"[..],
            CompletedTokenErrorKind::EmptyAnchorName,
            5,
        ),
        (
            &b"---\n* value\n"[..],
            CompletedTokenErrorKind::EmptyAliasName,
            5,
        ),
        (
            &b"---\n!<tag:broken value\n"[..],
            CompletedTokenErrorKind::UnterminatedVerbatimTag,
            16,
        ),
        (
            &b"---\n!e! value\n"[..],
            CompletedTokenErrorKind::EmptyTagSuffix,
            7,
        ),
        (
            &b"---\n!tag%G0 value\n"[..],
            CompletedTokenErrorKind::InvalidTagPercentEscape,
            9,
        ),
    ] {
        let (atoms, lines, structural, quoted, plain, block) = upstream(input);
        let error = scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("malformed completed-token spelling");
        assert_eq!(error.kind(), kind, "fixture: {input:?}");
        assert_eq!(error.byte_offset(), offset, "fixture: {input:?}");
    }
}

#[test]
fn tag_character_classes_match_yaml_1_2_2_at_exact_offsets() {
    for input in [
        &b"!<tag:example!foo> value\n"[..],
        &b"%TAG !e! tag:example.com,2000:app/\n---\n"[..],
        &b"!e!name%20with%20escapes value\n"[..],
    ] {
        scan(input);
    }

    for (input, kind, offset) in [
        (
            &b"%TAG !e! bad{prefix\n"[..],
            CompletedTokenErrorKind::InvalidTagDirective,
            12,
        ),
        (
            &b"!<bad{uri> value\n"[..],
            CompletedTokenErrorKind::InvalidVerbatimTag,
            5,
        ),
        (
            &b"!bad\\uri value\n"[..],
            CompletedTokenErrorKind::InvalidTagCharacter,
            4,
        ),
        (
            "!café value\n".as_bytes(),
            CompletedTokenErrorKind::InvalidTagCharacter,
            4,
        ),
        (
            "!<tag:café> value\n".as_bytes(),
            CompletedTokenErrorKind::InvalidVerbatimTag,
            9,
        ),
        (
            "%TAG !e! tag:café\n---\n".as_bytes(),
            CompletedTokenErrorKind::InvalidTagDirective,
            16,
        ),
    ] {
        let (atoms, lines, structural, quoted, plain, block) = upstream(input);
        let error = scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("tag spelling must use the exact YAML character class");
        assert_eq!(error.kind(), kind, "fixture: {input:?}");
        assert_eq!(error.byte_offset(), offset, "fixture: {input:?}");
    }
}

#[test]
fn directives_and_document_boms_require_a_true_column_zero_prefix() {
    for (input, offset) in [(&b" %YAML 1.2\n"[..], 1), (&b"  %TAG !e! tag:ok/\n"[..], 2)] {
        let (atoms, lines, structural, quoted, plain, block) = upstream(input);
        let error = scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("indented document-prefix syntax is invalid");
        assert_eq!(error.byte_offset(), offset, "fixture: {input:?}");
    }

    let indented_bom = b"  \xef\xbb\xbf%YAML 1.2\n";
    let atoms = atomize(indented_bom);
    let lines = analyze_profile1_layout(&atoms, canonical_structural_layout_limits())
        .expect("canonical line layout");
    let structural =
        scan_profile1_structural_lexemes(&atoms, &lines, canonical_structural_scan_limits())
            .expect("canonical structural candidates");
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &lines,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .expect("canonical quoted scalars");
    let error = scan_profile1_plain_scalars(
        &atoms,
        &lines,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .expect_err("a BOM after same-line indentation is invalid before completed token formation");
    assert_eq!(error.kind(), PlainScalarErrorKind::InvalidPlainCharacter);
    assert_eq!(error.byte_offset(), 2);

    let input = b"...\n\xef\xbb\xbf%YAML 1.2\n---\n";
    let (_, tokens) = scan(input);
    assert!(kinds(&tokens).contains(&CompletedTokenKind::DocumentByteOrderMark));
    assert!(kinds(&tokens).contains(&CompletedTokenKind::YamlDirective));
}

#[test]
fn bom_is_rejected_in_properties_aliases_and_reserved_directives_but_allowed_in_quotes() {
    for (input, expected_kind, offset) in [
        (
            &b"&a\xef\xbb\xbf value\n"[..],
            CompletedTokenErrorKind::InvalidAnchorCharacter,
            2,
        ),
        (
            &b"*a\xef\xbb\xbf\n"[..],
            CompletedTokenErrorKind::InvalidAliasCharacter,
            2,
        ),
        (
            &b"%FUTURE a\xef\xbb\xbf\n"[..],
            CompletedTokenErrorKind::InvalidDirectiveCharacter,
            9,
        ),
    ] {
        let (atoms, lines, structural, quoted, plain, block) = upstream(input);
        let error = scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("a BOM inside an unquoted YAML production is invalid");
        assert_eq!(error.kind(), expected_kind, "fixture: {input:?}");
        assert_eq!(error.byte_offset(), offset, "fixture: {input:?}");
    }

    let quoted = b"value: \"before\xef\xbb\xbfafter\"\n";
    let (_, tokens) = scan(quoted);
    assert!(kinds(&tokens).contains(&CompletedTokenKind::DoubleQuotedScalar));
}

#[test]
fn token_caps_empty_bom_and_nonleading_document_bom_have_exact_behavior() {
    assert_eq!(MAX_PROFILE1_COMPLETED_TOKENS, MAX_PROFILE1_LEXICAL_ATOMS);
    assert_eq!(MAX_PROFILE1_FLOW_DEPTH, 4096);
    assert!(scan(b"").1.tokens().is_empty());
    let bom_only = scan(b"\xef\xbb\xbf").1;
    assert!(bom_only.tokens().is_empty());
    assert_eq!(bom_only.bom_bytes(), 3);

    let input = b"a: b\n";
    let (atoms, lines, structural, quoted, plain, block) = upstream(input);
    let error = scan_profile1_completed_tokens(
        &atoms,
        &lines,
        &structural,
        &quoted,
        &plain,
        &block,
        CompletedTokenLimits::new(1, u64::MAX),
    )
    .expect_err("the mapping value indicator is the second token");
    assert_eq!(error.kind(), CompletedTokenErrorKind::TokenLimitExceeded);
    assert_eq!(error.byte_offset(), 1);

    let input = b"...\n\xef\xbb\xbf---\n";
    let (_, tokens) = scan(input);
    assert!(kinds(&tokens).contains(&CompletedTokenKind::DocumentByteOrderMark));
}

#[test]
fn mismatched_upstream_evidence_has_a_distinct_error_for_each_transformation() {
    let input = b"key: value\n";
    let (atoms, lines, structural, quoted, plain, block) = upstream(input);
    let (other_atoms, other_lines, other_structural, other_quoted, other_plain, other_block) =
        upstream(b"other: [value]\n");

    let cases = [
        scan_profile1_completed_tokens(
            &atoms,
            &other_lines,
            &structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("layout mismatch")
        .kind(),
        scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &other_structural,
            &quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("structural mismatch")
        .kind(),
        scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &other_quoted,
            &plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("quoted mismatch")
        .kind(),
        scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &other_plain,
            &block,
            canonical_completed_token_limits(),
        )
        .expect_err("plain mismatch")
        .kind(),
        scan_profile1_completed_tokens(
            &atoms,
            &lines,
            &structural,
            &quoted,
            &plain,
            &other_block,
            canonical_completed_token_limits(),
        )
        .expect_err("block mismatch")
        .kind(),
    ];
    assert_eq!(
        cases,
        [
            CompletedTokenErrorKind::InputLayoutMismatch,
            CompletedTokenErrorKind::InputStructuralMismatch,
            CompletedTokenErrorKind::InputQuotedMismatch,
            CompletedTokenErrorKind::InputPlainMismatch,
            CompletedTokenErrorKind::InputBlockMismatch,
        ]
    );
    let _ = other_atoms;
}
