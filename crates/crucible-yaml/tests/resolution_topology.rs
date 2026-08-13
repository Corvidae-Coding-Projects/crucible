use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, compose_profile1_semantic_topology, decode_profile1,
    parse_profile1_cst, scan_profile1_block_scalars, scan_profile1_completed_tokens,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AtomizeLimits, AtomizedSource, BomPolicy, CompletedTokenSource, CstNodeKind, CstSource,
    DecodeLimits, SemanticTopologyErrorKind, SemanticTopologyLimits, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS,
    MAX_PROFILE1_SEMANTIC_MAPPING_EDGES, MAX_PROFILE1_SEMANTIC_NODES,
    MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES, MAX_PROFILE1_SOURCE_BYTES,
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

fn unlimited() -> SemanticTopologyLimits {
    SemanticTopologyLimits::new(
        MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS,
        MAX_PROFILE1_SEMANTIC_NODES,
        MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES,
        MAX_PROFILE1_SEMANTIC_MAPPING_EDGES,
    )
}

#[test]
fn mixed_documents_nodes_and_collection_edges_retain_exact_cst_identity() {
    let input = b"root:\n  - [one, two]\n  - {key: value}\n---\n[three]\n";
    let (atoms, tokens, cst) = parse(input);
    let topology = compose_profile1_semantic_topology(&atoms, &tokens, &cst, unlimited())
        .expect("valid CST composes exact topology");

    assert_eq!(topology.document_roots().len(), cst.documents().len());
    assert_eq!(topology.nodes().len(), cst.nodes().len());
    assert_eq!(
        topology.sequence_edges().len(),
        cst.sequence_entries().len()
    );
    assert_eq!(topology.mapping_edges().len(), cst.mapping_entries().len());

    for (index, root) in topology.document_roots().iter().enumerate() {
        assert_eq!(root.document_index(), index as u64);
        assert_eq!(root.node_index(), cst.documents()[index].root_node_index());
        assert_eq!(root.byte_start(), cst.documents()[index].byte_start());
    }
    for (index, node) in topology.nodes().iter().enumerate() {
        let source = &cst.nodes()[index];
        assert_eq!(node.cst_node_index(), index as u64);
        assert_eq!(node.kind(), source.kind());
        assert_eq!(node.byte_start(), source.byte_start());
        assert_eq!(node.byte_end(), source.byte_end());
        assert_eq!(node.edge_start(), source.entry_start());
        assert_eq!(node.edge_end(), source.entry_end());
    }
    for (index, edge) in topology.sequence_edges().iter().enumerate() {
        let source = &cst.sequence_entries()[index];
        assert_eq!(edge.cst_entry_index(), index as u64);
        assert_eq!(edge.child_node_index(), source.node_index());
        assert_eq!(edge.token_start(), source.token_start());
        assert_eq!(edge.token_end(), source.token_end());
    }
    for (index, edge) in topology.mapping_edges().iter().enumerate() {
        let source = &cst.mapping_entries()[index];
        assert_eq!(edge.cst_entry_index(), index as u64);
        assert_eq!(edge.key_node_index(), source.key_node_index());
        assert_eq!(edge.value_node_index(), source.value_node_index());
        assert_eq!(edge.token_start(), source.token_start());
        assert_eq!(edge.token_end(), source.token_end());
    }

    assert!(topology
        .nodes()
        .iter()
        .any(|node| node.kind() == CstNodeKind::Sequence && node.edge_end() > node.edge_start()));
    assert!(topology
        .nodes()
        .iter()
        .any(|node| node.kind() == CstNodeKind::Mapping && node.edge_end() > node.edge_start()));
}

#[test]
fn each_topology_cap_rejects_the_first_excluded_source_record() {
    let (atoms, tokens, cst) = parse(b"[one, two]\n");
    for (limits, kind, expected_offset) in [
        (
            SemanticTopologyLimits::new(0, u64::MAX, u64::MAX, u64::MAX),
            SemanticTopologyErrorKind::DocumentRootLimitExceeded,
            cst.documents()[0].byte_start(),
        ),
        (
            SemanticTopologyLimits::new(u64::MAX, 0, u64::MAX, u64::MAX),
            SemanticTopologyErrorKind::NodeLimitExceeded,
            cst.nodes()[0].byte_start(),
        ),
        (
            SemanticTopologyLimits::new(u64::MAX, u64::MAX, 0, u64::MAX),
            SemanticTopologyErrorKind::SequenceEdgeLimitExceeded,
            tokens.tokens()[cst.sequence_entries()[0].token_start() as usize].byte_start(),
        ),
        (
            SemanticTopologyLimits::new(u64::MAX, u64::MAX, u64::MAX, 0),
            SemanticTopologyErrorKind::MappingEdgeLimitExceeded,
            atoms.source_len_bytes(),
        ),
    ] {
        let result = compose_profile1_semantic_topology(&atoms, &tokens, &cst, limits);
        if kind == SemanticTopologyErrorKind::MappingEdgeLimitExceeded {
            assert!(result.is_ok(), "the fixture has no mapping edge to exclude");
        } else {
            let error = result.expect_err("the first record exceeds its lowered cap");
            assert_eq!(error.kind(), kind);
            assert_eq!(error.byte_offset(), expected_offset);
        }
    }
}

#[test]
fn mapping_edge_limit_and_cross_source_authentication_are_exact() {
    let (atoms, tokens, cst) = parse(b"{one: two}\n");
    let expected = tokens.tokens()[cst.mapping_entries()[0].token_start() as usize].byte_start();
    let error = compose_profile1_semantic_topology(
        &atoms,
        &tokens,
        &cst,
        SemanticTopologyLimits::new(u64::MAX, u64::MAX, u64::MAX, 0),
    )
    .expect_err("the first mapping edge exceeds a zero cap");
    assert_eq!(
        error.kind(),
        SemanticTopologyErrorKind::MappingEdgeLimitExceeded
    );
    assert_eq!(error.byte_offset(), expected);

    let (other_atoms, _, _) = parse(b"[different]\n");
    let error = compose_profile1_semantic_topology(&other_atoms, &tokens, &cst, unlimited())
        .expect_err("completed tokens must authenticate against the atom source");
    assert_eq!(
        error.kind(),
        SemanticTopologyErrorKind::InputCompletedTokenMismatch
    );
}
