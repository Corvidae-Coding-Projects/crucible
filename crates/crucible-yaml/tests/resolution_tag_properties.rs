use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, parse_profile1_cst,
    resolve_profile1_node_tag_property, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource, BomPolicy,
    CompletedTokenSource, CstSource, DecodeLimits, ResolvedTagKind, ResolvedTagOrigin,
    TagResolutionErrorKind, TagResolutionLimits, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS, MAX_PROFILE1_SOURCE_BYTES,
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

fn node_with_tag_spelling(
    input: &[u8],
    tokens: &CompletedTokenSource,
    cst: &CstSource,
    spelling: &[u8],
) -> u64 {
    cst.nodes()
        .iter()
        .enumerate()
        .find_map(|(node_index, node)| {
            let token_index = node.tag_property_token()? as usize;
            let token = &tokens.tokens()[token_index];
            let raw = &input[token.byte_start() as usize..token.byte_end() as usize];
            (raw == spelling).then_some(node_index as u64)
        })
        .expect("tagged node exists")
}

fn tag_text(tag: &crucible_yaml::ResolvedTagProperty) -> String {
    tag.content()
        .iter()
        .map(|item| char::from_u32(item.code_point()).expect("tag character is Unicode"))
        .collect()
}

fn unlimited() -> TagResolutionLimits {
    TagResolutionLimits::new(MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS)
}

#[test]
fn default_named_local_and_verbatim_tags_resolve_without_percent_decoding() {
    let input = b"%TAG !e! tag:example.com,2026:app/\n---\nvalues:\n  - !!str text\n  - !local value\n  - !e!thing custom\n  - !<tag:example.com,2026:item%20one> verbatim\n  - !<!private%20tag> local-verbatim\n";
    let (atoms, tokens, cst) = parse(input);

    for (raw, kind, expected) in [
        (
            &b"!!str"[..],
            ResolvedTagKind::Global,
            "tag:yaml.org,2002:str",
        ),
        (&b"!local"[..], ResolvedTagKind::Local, "!local"),
        (
            &b"!e!thing"[..],
            ResolvedTagKind::Global,
            "tag:example.com,2026:app/thing",
        ),
        (
            &b"!<tag:example.com,2026:item%20one>"[..],
            ResolvedTagKind::Global,
            "tag:example.com,2026:item%20one",
        ),
        (
            &b"!<!private%20tag>"[..],
            ResolvedTagKind::Local,
            "!private%20tag",
        ),
    ] {
        let node_index = node_with_tag_spelling(input, &tokens, &cst, raw);
        let resolved =
            resolve_profile1_node_tag_property(&atoms, &tokens, &cst, node_index, unlimited())
                .expect("valid explicit tag resolves")
                .expect("node has an explicit tag");
        assert_eq!(resolved.kind(), kind, "fixture: {raw:?}");
        assert_eq!(tag_text(&resolved), expected, "fixture: {raw:?}");
    }

    let escaped_node =
        node_with_tag_spelling(input, &tokens, &cst, b"!<tag:example.com,2026:item%20one>");
    let escaped =
        resolve_profile1_node_tag_property(&atoms, &tokens, &cst, escaped_node, unlimited())
            .expect("valid tag")
            .expect("explicit tag");
    assert!(escaped
        .content()
        .iter()
        .all(|item| item.origin() == ResolvedTagOrigin::VerbatimPayload));
    assert_eq!(
        escaped
            .content()
            .iter()
            .filter(|item| item.code_point() == u32::from('%'))
            .count(),
        1,
        "the percent escape is preserved as three presentation characters",
    );
}

#[test]
fn tag_directive_bindings_reset_and_may_change_local_or_global_identity() {
    let input = b"%TAG !e! tag:first.example,2026:\n--- !e!item one\n...\n%TAG !e! !second-\n--- !e!item two\n";
    let (atoms, tokens, cst) = parse(input);
    let tagged_nodes: Vec<_> = cst
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.tag_property_token().map(|_| index as u64))
        .collect();
    assert_eq!(tagged_nodes.len(), 2);

    let first =
        resolve_profile1_node_tag_property(&atoms, &tokens, &cst, tagged_nodes[0], unlimited())
            .expect("first document tag resolves")
            .expect("explicit tag");
    assert_eq!(first.kind(), ResolvedTagKind::Global);
    assert_eq!(tag_text(&first), "tag:first.example,2026:item");

    let second =
        resolve_profile1_node_tag_property(&atoms, &tokens, &cst, tagged_nodes[1], unlimited())
            .expect("second document tag resolves")
            .expect("explicit tag");
    assert_eq!(second.kind(), ResolvedTagKind::Local);
    assert_eq!(tag_text(&second), "!second-item");
}

#[test]
fn invalid_global_uri_limits_absent_tags_and_indices_have_exact_results() {
    let input =
        b"bad: !<$:?> value\nplain: value\nescaped: !<tag:x:%41> kept\ndefault: !!str text\n";
    let (atoms, tokens, cst) = parse(input);

    let bad_node = node_with_tag_spelling(input, &tokens, &cst, b"!<$:?>");
    let error = resolve_profile1_node_tag_property(&atoms, &tokens, &cst, bad_node, unlimited())
        .expect_err("a global verbatim tag must be an absolute URI");
    assert_eq!(error.kind(), TagResolutionErrorKind::InvalidGlobalTagUri);
    let bad_token = &tokens.tokens()[cst.nodes()[bad_node as usize]
        .tag_property_token()
        .expect("tag token") as usize];
    assert_eq!(error.byte_offset(), bad_token.byte_start() + 2);

    let escaped_node = node_with_tag_spelling(input, &tokens, &cst, b"!<tag:x:%41>");
    let escaped =
        resolve_profile1_node_tag_property(&atoms, &tokens, &cst, escaped_node, unlimited())
            .expect("valid URI tag")
            .expect("explicit tag");
    assert_eq!(tag_text(&escaped), "tag:x:%41");
    let excluded = &escaped.content()[6];
    assert_eq!(excluded.code_point(), u32::from('%'));
    let error = resolve_profile1_node_tag_property(
        &atoms,
        &tokens,
        &cst,
        escaped_node,
        TagResolutionLimits::new(6),
    )
    .expect_err("the preserved percent sign is the first excluded tag character");
    assert_eq!(
        error.kind(),
        TagResolutionErrorKind::TagCodePointLimitExceeded
    );
    assert_eq!(error.byte_offset(), excluded.byte_start());

    let untagged = cst
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| node.tag_property_token().is_none())
        .map(|(index, _)| index as u64)
        .expect("fixture has untagged nodes");
    assert!(resolve_profile1_node_tag_property(
        &atoms,
        &tokens,
        &cst,
        untagged,
        TagResolutionLimits::new(0),
    )
    .expect("an absent property consumes no explicit-tag budget")
    .is_none());

    let error = resolve_profile1_node_tag_property(
        &atoms,
        &tokens,
        &cst,
        cst.nodes().len() as u64,
        unlimited(),
    )
    .expect_err("node index is out of range");
    assert_eq!(error.kind(), TagResolutionErrorKind::NodeIndexOutOfRange);
    assert_eq!(error.byte_offset(), atoms.source_len_bytes());
}
