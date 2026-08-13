use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, parse_profile1_cst,
    resolve_profile1_anchor_aliases, scan_profile1_block_scalars, scan_profile1_completed_tokens,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AnchorAliasErrorKind, AnchorAliasLimits, AtomizeLimits, AtomizedSource, BomPolicy,
    CompletedTokenKind, CompletedTokenSource, CstSource, DecodeLimits, MAX_PROFILE1_ALIAS_BINDINGS,
    MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES,
};

fn parse(input: &[u8]) -> (AtomizedSource, CompletedTokenSource, CstSource) {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 bytes");
    let atoms = atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS))
        .expect("bounded atom source");
    let layout = analyze_profile1_layout(&atoms, canonical_structural_layout_limits())
        .expect("canonical layout");
    let structural =
        scan_profile1_structural_lexemes(&atoms, &layout, canonical_structural_scan_limits())
            .expect("canonical structural candidates");
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &layout,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .expect("canonical quoted scalars");
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .expect("canonical plain scalars");
    let block = scan_profile1_block_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    )
    .expect("canonical block scalars");
    let tokens = scan_profile1_completed_tokens(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    )
    .expect("canonical completed tokens");
    let cst = parse_profile1_cst(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        &tokens,
        canonical_cst_limits(),
    )
    .expect("canonical CST");
    (atoms, tokens, cst)
}

fn unlimited() -> AnchorAliasLimits {
    AnchorAliasLimits::new(
        MAX_PROFILE1_ANCHOR_DECLARATIONS,
        MAX_PROFILE1_ALIAS_BINDINGS,
    )
}

fn token_indices(tokens: &CompletedTokenSource, kind: CompletedTokenKind) -> Vec<u64> {
    tokens
        .tokens()
        .iter()
        .enumerate()
        .filter_map(|(index, token)| (token.kind() == kind).then_some(index as u64))
        .collect()
}

fn owner_node(cst: &CstSource, token_index: u64) -> u64 {
    cst.syntax_owners()[token_index as usize]
        .as_ref()
        .expect("syntax token has an owner")
        .record_index()
}

#[test]
fn aliases_bind_to_the_latest_exact_preceding_anchor_in_their_document() {
    let input =
        b"root:\n  first: &kappa old\n  before: *kappa\n  second: &kappa new\n  after: *kappa\n";
    let (atoms, tokens, cst) = parse(input);
    let anchor_tokens = token_indices(&tokens, CompletedTokenKind::AnchorProperty);
    let alias_tokens = token_indices(&tokens, CompletedTokenKind::Alias);
    assert_eq!(anchor_tokens.len(), 2);
    assert_eq!(alias_tokens.len(), 2);

    let resolved = resolve_profile1_anchor_aliases(&atoms, &tokens, &cst, unlimited())
        .expect("every alias has a preceding declaration");
    assert_eq!(resolved.anchors().len(), 2);
    assert_eq!(resolved.aliases().len(), 2);

    for (index, expected_anchor) in anchor_tokens.iter().enumerate() {
        let binding = &resolved.aliases()[index];
        assert_eq!(binding.alias_token_index(), alias_tokens[index]);
        assert_eq!(binding.target_anchor_index(), index as u64);
        assert_eq!(
            binding.target_node_index(),
            owner_node(&cst, *expected_anchor)
        );
        assert_eq!(
            binding.name_byte_start(),
            tokens.tokens()[alias_tokens[index] as usize].byte_start() + 1
        );
    }
}

#[test]
fn a_collection_anchor_is_visible_to_a_descendant_alias_before_parent_completion() {
    let input = b"root: &self [*self]\n";
    let (atoms, tokens, cst) = parse(input);
    let anchor_token = token_indices(&tokens, CompletedTokenKind::AnchorProperty)[0];
    let alias_token = token_indices(&tokens, CompletedTokenKind::Alias)[0];

    let resolved = resolve_profile1_anchor_aliases(&atoms, &tokens, &cst, unlimited())
        .expect("presentation order, not CST completion order, controls visibility");
    let binding = &resolved.aliases()[0];
    assert_eq!(binding.target_node_index(), owner_node(&cst, anchor_token));
    assert_eq!(binding.alias_node_index(), owner_node(&cst, alias_token));
    assert!(binding.target_node_index() > binding.alias_node_index());
}

#[test]
fn document_reset_missing_aliases_and_caller_limits_have_exact_precedence() {
    let cross_document = b"--- &x first\n...\n--- *x\n";
    let (atoms, tokens, cst) = parse(cross_document);
    let alias_token = token_indices(&tokens, CompletedTokenKind::Alias)[0];
    let error = resolve_profile1_anchor_aliases(
        &atoms,
        &tokens,
        &cst,
        AnchorAliasLimits::new(MAX_PROFILE1_ANCHOR_DECLARATIONS, 0),
    )
    .expect_err("a prior document must not provide an alias binding");
    assert_eq!(error.kind(), AnchorAliasErrorKind::UnresolvedAlias);
    assert_eq!(
        error.byte_offset(),
        tokens.tokens()[alias_token as usize].byte_start()
    );

    let valid = b"root:\n  declaration: &x value\n  use: *x\n";
    let (atoms, tokens, cst) = parse(valid);
    let anchor_token = token_indices(&tokens, CompletedTokenKind::AnchorProperty)[0];
    let alias_token = token_indices(&tokens, CompletedTokenKind::Alias)[0];
    let error = resolve_profile1_anchor_aliases(
        &atoms,
        &tokens,
        &cst,
        AnchorAliasLimits::new(0, MAX_PROFILE1_ALIAS_BINDINGS),
    )
    .expect_err("the first anchor exceeds a zero declaration limit");
    assert_eq!(error.kind(), AnchorAliasErrorKind::AnchorLimitExceeded);
    assert_eq!(
        error.byte_offset(),
        tokens.tokens()[anchor_token as usize].byte_start()
    );

    let error = resolve_profile1_anchor_aliases(
        &atoms,
        &tokens,
        &cst,
        AnchorAliasLimits::new(MAX_PROFILE1_ANCHOR_DECLARATIONS, 0),
    )
    .expect_err("the first resolved alias exceeds a zero alias limit");
    assert_eq!(error.kind(), AnchorAliasErrorKind::AliasLimitExceeded);
    assert_eq!(
        error.byte_offset(),
        tokens.tokens()[alias_token as usize].byte_start()
    );
}

#[test]
fn unicode_names_and_input_authentication_are_exact() {
    let input = "root:\n  declaration: &π value\n  use: *π\n".as_bytes();
    let (atoms, tokens, cst) = parse(input);
    let resolved = resolve_profile1_anchor_aliases(&atoms, &tokens, &cst, unlimited())
        .expect("exact Unicode names match");
    assert_eq!(resolved.anchors().len(), 1);
    assert_eq!(resolved.aliases().len(), 1);
    assert_eq!(
        resolved.anchors()[0].name_byte_end() - resolved.anchors()[0].name_byte_start(),
        2
    );

    let (other_atoms, other_tokens, other_cst) = parse(b"different: [shape, and, length]\n");
    let error = resolve_profile1_anchor_aliases(&other_atoms, &tokens, &cst, unlimited())
        .expect_err("completed tokens authenticate their atom source");
    assert_eq!(
        error.kind(),
        AnchorAliasErrorKind::InputCompletedTokenMismatch
    );

    let error = resolve_profile1_anchor_aliases(&atoms, &tokens, &other_cst, unlimited())
        .expect_err("the CST authenticates its completed-token source");
    assert_eq!(error.kind(), AnchorAliasErrorKind::InputCstMismatch);
    let _ = other_tokens;
}
