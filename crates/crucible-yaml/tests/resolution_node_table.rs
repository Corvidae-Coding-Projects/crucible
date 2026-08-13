use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, compose_profile1_semantic_node_table,
    compose_profile1_semantic_scalar_table, compose_profile1_semantic_topology, decode_profile1,
    parse_profile1_cst, resolve_profile1_anchor_aliases, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AnchorAliasLimits, AnchorAliasSource, AtomizeLimits,
    AtomizedSource, BlockScalarSource, BomPolicy, CompletedTokenSource, CstNodeKind, CstSource,
    DecodeLimits, PlainScalarSource, QuotedScalarSource, ResolvedCollectionTag, SemanticNodeKind,
    SemanticNodeTableErrorKind, SemanticNodeTableLimits, SemanticScalarTableLimits,
    SemanticScalarTableSource, SemanticTopologySource, MAX_PROFILE1_ALIAS_BINDINGS,
    MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS, MAX_PROFILE1_SEMANTIC_COLLECTIONS,
    MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES, MAX_PROFILE1_SOURCE_BYTES,
};

struct Parsed {
    atoms: AtomizedSource,
    quoted: QuotedScalarSource,
    plain: PlainScalarSource,
    block: BlockScalarSource,
    tokens: CompletedTokenSource,
    cst: CstSource,
}

fn parse(input: &[u8]) -> Parsed {
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
    Parsed {
        atoms,
        quoted,
        plain,
        block,
        tokens,
        cst,
    }
}

fn foundations(
    parsed: &Parsed,
) -> (
    SemanticTopologySource,
    SemanticScalarTableSource,
    AnchorAliasSource,
) {
    let topology = compose_profile1_semantic_topology(
        &parsed.atoms,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
    )
    .expect("canonical topology");
    let scalars = compose_profile1_semantic_scalar_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_scalar_table_limits(),
    )
    .expect("canonical scalar table");
    let anchors = resolve_profile1_anchor_aliases(
        &parsed.atoms,
        &parsed.tokens,
        &parsed.cst,
        AnchorAliasLimits::new(
            MAX_PROFILE1_ANCHOR_DECLARATIONS,
            MAX_PROFILE1_ALIAS_BINDINGS,
        ),
    )
    .expect("canonical anchor and alias bindings");
    (topology, scalars, anchors)
}

fn limits() -> SemanticNodeTableLimits {
    SemanticNodeTableLimits::new(
        MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES,
        MAX_PROFILE1_SEMANTIC_COLLECTIONS,
        MAX_PROFILE1_ALIAS_BINDINGS,
        MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
    )
}

fn scalar_limits() -> SemanticScalarTableLimits {
    canonical_semantic_scalar_table_limits()
}

fn anchor_limits() -> AnchorAliasLimits {
    AnchorAliasLimits::new(
        MAX_PROFILE1_ANCHOR_DECLARATIONS,
        MAX_PROFILE1_ALIAS_BINDINGS,
    )
}

#[test]
fn every_cst_node_gets_one_exact_value_or_redirect_slot() {
    let parsed = parse(b"base: &base [one, 2]\ncopy: *base\nnested: {flag: true}\n");
    let (topology, scalars, anchors) = foundations(&parsed);
    let table = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        limits(),
    )
    .expect("every node has an exact semantic slot");

    assert_eq!(table.nodes().len(), parsed.cst.nodes().len());
    assert_eq!(table.topology(), &topology);
    assert_eq!(table.scalars(), &scalars);
    assert_eq!(table.anchors(), &anchors);
    assert_eq!(table.input_node_count(), topology.nodes().len() as u64);
    assert_eq!(table.input_scalar_count(), scalars.scalars().len() as u64);
    assert_eq!(table.input_anchor_count(), anchors.anchors().len() as u64);
    assert_eq!(table.input_alias_count(), anchors.aliases().len() as u64);
    assert_eq!(table.alias_redirects().len(), anchors.aliases().len());

    for (index, (slot, cst_node)) in table
        .nodes()
        .iter()
        .zip(parsed.cst.nodes().iter())
        .enumerate()
    {
        assert_eq!(slot.cst_node_index(), index as u64);
        assert_eq!(slot.token_start(), cst_node.token_start());
        assert_eq!(slot.token_end(), cst_node.token_end());
        assert_eq!(slot.byte_start(), cst_node.byte_start());
        assert_eq!(slot.byte_end(), cst_node.byte_end());
        assert_eq!(
            slot.anchor_property_token(),
            cst_node.anchor_property_token()
        );
        assert_eq!(slot.tag_property_token(), cst_node.tag_property_token());
        assert_eq!(slot.edge_start(), cst_node.entry_start());
        assert_eq!(slot.edge_end(), cst_node.entry_end());

        match slot.kind() {
            SemanticNodeKind::Scalar => {
                assert!(matches!(
                    cst_node.kind(),
                    CstNodeKind::Scalar | CstNodeKind::Empty
                ));
                let scalar =
                    &scalars.scalars()[slot.value_index().expect("scalar record") as usize];
                assert_eq!(scalar.node_index(), index as u64);
                assert_eq!(slot.alias_target_node_index(), None);
            }
            SemanticNodeKind::Sequence | SemanticNodeKind::Mapping => {
                let collection =
                    &table.collections()[slot.value_index().expect("collection record") as usize];
                assert_eq!(collection.node_index(), index as u64);
                assert_eq!(collection.kind(), cst_node.kind());
                assert_eq!(
                    collection.tag(),
                    if slot.kind() == SemanticNodeKind::Sequence {
                        ResolvedCollectionTag::CoreSequence
                    } else {
                        ResolvedCollectionTag::CoreMapping
                    }
                );
                assert_eq!(slot.alias_target_node_index(), None);
            }
            SemanticNodeKind::Alias => {
                assert_eq!(cst_node.kind(), CstNodeKind::Alias);
                assert_eq!(slot.value_index(), None);
                let target = slot
                    .alias_target_node_index()
                    .expect("alias slot has its exact target");
                assert_eq!(
                    parsed.cst.nodes()[target as usize].kind(),
                    CstNodeKind::Sequence
                );
            }
        }
    }

    assert_eq!(table.alias_redirects().len(), 1);
    let redirect = &table.alias_redirects()[0];
    let binding = &anchors.aliases()[0];
    assert_eq!(redirect.binding_index(), 0);
    assert_eq!(redirect.document_index(), binding.document_index());
    assert_eq!(redirect.alias_node_index(), binding.alias_node_index());
    assert_eq!(redirect.alias_token_index(), binding.alias_token_index());
    assert_eq!(
        redirect.target_anchor_index(),
        binding.target_anchor_index()
    );
    assert_eq!(redirect.target_node_index(), binding.target_node_index());
    assert_eq!(
        redirect.name_start_atom_index(),
        binding.name_start_atom_index()
    );
    assert_eq!(
        redirect.name_end_atom_index(),
        binding.name_end_atom_index()
    );
    assert_eq!(redirect.name_byte_start(), binding.name_byte_start());
    assert_eq!(redirect.name_byte_end(), binding.name_byte_end());
}

#[test]
fn independently_lowered_caps_report_the_exact_first_excluded_record() {
    let parsed = parse(b"[]\n");
    let (_topology, _scalars, _anchors) = foundations(&parsed);
    let error = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        SemanticNodeTableLimits::new(0, u64::MAX, u64::MAX, u64::MAX),
    )
    .expect_err("the root is the first excluded semantic node");
    assert_eq!(error.kind(), SemanticNodeTableErrorKind::NodeLimitExceeded);
    assert_eq!(error.byte_offset(), parsed.cst.nodes()[0].byte_start());

    let error = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        SemanticNodeTableLimits::new(u64::MAX, 0, u64::MAX, u64::MAX),
    )
    .expect_err("the root is the first excluded collection record");
    assert_eq!(
        error.kind(),
        SemanticNodeTableErrorKind::CollectionLimitExceeded
    );
    assert_eq!(error.byte_offset(), parsed.cst.nodes()[0].byte_start());

    let parsed = parse(b"value: &a one\ncopy: *a\n");
    let (_topology, _scalars, anchors) = foundations(&parsed);
    let error = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        SemanticNodeTableLimits::new(u64::MAX, u64::MAX, 0, u64::MAX),
    )
    .expect_err("the first alias redirect exceeds its independent cap");
    assert_eq!(
        error.kind(),
        SemanticNodeTableErrorKind::AliasRedirectLimitExceeded
    );
    assert_eq!(error.byte_offset(), anchors.aliases()[0].name_byte_start());

    let parsed = parse(b"!long [one]\n");
    let error = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        SemanticNodeTableLimits::new(u64::MAX, u64::MAX, u64::MAX, 1),
    )
    .expect_err("the second collection-tag code point exceeds its nested cap");
    assert_eq!(
        error.kind(),
        SemanticNodeTableErrorKind::CollectionTag(
            crucible_yaml::CollectionTagErrorKind::TagResolution(
                crucible_yaml::TagResolutionErrorKind::TagCodePointLimitExceeded,
            ),
        )
    );
    assert_eq!(error.byte_offset(), 1);
}

#[test]
fn every_raw_producer_is_authenticated_before_owned_population() {
    let parsed = parse(b"[one]\n");
    let other = parse(b"different\n");

    let error = compose_profile1_semantic_node_table(
        &other.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        limits(),
    )
    .expect_err("foreign atoms cannot authenticate the completed stream");
    assert_eq!(
        error.kind(),
        SemanticNodeTableErrorKind::InputCompletedTokenMismatch
    );

    let error = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &other.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        limits(),
    )
    .expect_err("foreign quoted evidence cannot populate owned scalar values");
    assert_eq!(
        error.kind(),
        SemanticNodeTableErrorKind::ScalarTable(
            crucible_yaml::SemanticScalarTableErrorKind::ScalarValue(
                crucible_yaml::ScalarValueErrorKind::ScalarDecode(
                    crucible_yaml::CstScalarDecodeErrorKind::InputQuotedMismatch,
                ),
            ),
        )
    );

    let error = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &other.cst,
        canonical_semantic_topology_limits(),
        scalar_limits(),
        anchor_limits(),
        limits(),
    )
    .expect_err("foreign CST evidence cannot be mixed with the completed stream");
    assert_eq!(error.kind(), SemanticNodeTableErrorKind::InputCstMismatch);
}
