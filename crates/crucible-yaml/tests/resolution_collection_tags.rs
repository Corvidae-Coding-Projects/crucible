use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, parse_profile1_cst,
    resolve_profile1_cst_node_collection_tag, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, AtomizedSource, BomPolicy,
    CollectionTagErrorKind, CollectionTagLimits, CompletedTokenSource, CstNodeKind, CstSource,
    DecodeLimits, ResolvedCollectionTag, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS, MAX_PROFILE1_SOURCE_BYTES,
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

fn unlimited() -> CollectionTagLimits {
    CollectionTagLimits::new(MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS)
}

fn resolve_root(input: &[u8]) -> crucible_yaml::ResolvedCollection {
    let (atoms, tokens, cst) = parse(input);
    let root = cst.documents()[0].root_node_index();
    resolve_profile1_cst_node_collection_tag(&atoms, &tokens, &cst, root, unlimited())
        .expect("valid collection tag resolves")
        .expect("root is a collection")
}

fn explicit_tag_text(value: &crucible_yaml::ResolvedCollection) -> Option<String> {
    value.explicit_tag().map(|tag| {
        tag.content()
            .iter()
            .map(|point| char::from_u32(point.code_point()).expect("Unicode tag point"))
            .collect()
    })
}

#[test]
fn implicit_non_specific_and_explicit_core_collection_tags_are_exact() {
    let implicit_sequence = resolve_root(b"[one]\n");
    assert_eq!(implicit_sequence.kind(), CstNodeKind::Sequence);
    assert_eq!(implicit_sequence.tag(), ResolvedCollectionTag::CoreSequence);
    assert!(implicit_sequence.explicit_tag().is_none());

    let explicit_sequence = resolve_root(b"!!seq [one]\n");
    assert_eq!(explicit_sequence.tag(), ResolvedCollectionTag::CoreSequence);
    assert_eq!(
        explicit_tag_text(&explicit_sequence).as_deref(),
        Some("tag:yaml.org,2002:seq")
    );

    let non_specific_mapping = resolve_root(b"! {one: two}\n");
    assert_eq!(non_specific_mapping.kind(), CstNodeKind::Mapping);
    assert_eq!(
        non_specific_mapping.tag(),
        ResolvedCollectionTag::CoreMapping
    );
    assert_eq!(
        explicit_tag_text(&non_specific_mapping).as_deref(),
        Some("!")
    );

    let explicit_mapping = resolve_root(b"!!map {one: two}\n");
    assert_eq!(explicit_mapping.tag(), ResolvedCollectionTag::CoreMapping);
}

#[test]
fn custom_collection_tags_are_lossless_and_scalars_are_left_to_the_scalar_path() {
    let local = resolve_root(b"!local {one: two}\n");
    assert_eq!(local.tag(), ResolvedCollectionTag::CustomLocal);
    assert_eq!(explicit_tag_text(&local).as_deref(), Some("!local"));

    let global = resolve_root(b"!<tag:example.com,2026:sequence> [one]\n");
    assert_eq!(global.tag(), ResolvedCollectionTag::CustomGlobal);
    assert_eq!(
        explicit_tag_text(&global).as_deref(),
        Some("tag:example.com,2026:sequence")
    );

    let (atoms, tokens, cst) = parse(b"scalar\n");
    let root = cst.documents()[0].root_node_index();
    assert!(
        resolve_profile1_cst_node_collection_tag(&atoms, &tokens, &cst, root, unlimited())
            .expect("a scalar is not a collection-tag error")
            .is_none()
    );
}

#[test]
fn kind_mismatches_indices_and_tag_limits_report_exact_typed_errors() {
    for input in [
        &b"!!map [one]\n"[..],
        &b"!!seq {one: two}\n"[..],
        &b"!!str [one]\n"[..],
    ] {
        let (atoms, tokens, cst) = parse(input);
        let root = cst.documents()[0].root_node_index();
        let error =
            resolve_profile1_cst_node_collection_tag(&atoms, &tokens, &cst, root, unlimited())
                .expect_err("the explicit standard tag is incompatible with the collection kind");
        assert_eq!(
            error.kind(),
            CollectionTagErrorKind::CollectionTagKindMismatch
        );
        assert!(error.byte_offset() < input.len() as u64);
    }

    let (atoms, tokens, cst) = parse(b"!long [one]\n");
    let root = cst.documents()[0].root_node_index();
    let error = resolve_profile1_cst_node_collection_tag(
        &atoms,
        &tokens,
        &cst,
        root,
        CollectionTagLimits::new(1),
    )
    .expect_err("the second resolved tag code point is excluded");
    assert_eq!(
        error.kind(),
        CollectionTagErrorKind::TagResolution(
            crucible_yaml::TagResolutionErrorKind::TagCodePointLimitExceeded
        )
    );

    let error = resolve_profile1_cst_node_collection_tag(
        &atoms,
        &tokens,
        &cst,
        cst.nodes().len() as u64,
        unlimited(),
    )
    .expect_err("the node index is checked by the authenticated tag producer");
    assert_eq!(
        error.kind(),
        CollectionTagErrorKind::TagResolution(
            crucible_yaml::TagResolutionErrorKind::NodeIndexOutOfRange
        )
    );
    assert_eq!(error.byte_offset(), atoms.source_len_bytes());
}
