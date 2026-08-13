use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_duplicate_key_limits, canonical_lowering_limits, canonical_merge_expansion_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_key_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    compose_profile1_canonical_structural_keys, decode_profile1, expand_profile1_merge_keys,
    lower_profile1_canonical_graph, parse_profile1_cst, reject_profile1_duplicate_keys,
    scan_profile1_block_scalars, scan_profile1_completed_tokens, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AnchorAliasLimits,
    AtomizeLimits, BomPolicy, CanonicalLoweringErrorKind, CanonicalLoweringLimits,
    CanonicalYamlNodeKind, DecodeLimits, ExpandedSemanticGraphSource, SemanticNodeKind,
    MAX_PROFILE1_ALIAS_BINDINGS, MAX_PROFILE1_ANCHOR_DECLARATIONS,
    MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS, MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES,
    MAX_PROFILE1_CANONICAL_NODES, MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
};

fn expanded(input: &[u8]) -> ExpandedSemanticGraphSource {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .unwrap();
    let atoms = atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS)).unwrap();
    let layout = analyze_profile1_layout(&atoms, canonical_structural_layout_limits()).unwrap();
    let structural =
        scan_profile1_structural_lexemes(&atoms, &layout, canonical_structural_scan_limits())
            .unwrap();
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &layout,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .unwrap();
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .unwrap();
    let block = scan_profile1_block_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    )
    .unwrap();
    let tokens = scan_profile1_completed_tokens(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    )
    .unwrap();
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
    .unwrap();
    let structural_keys = compose_profile1_canonical_structural_keys(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        canonical_semantic_topology_limits(),
        canonical_semantic_scalar_table_limits(),
        AnchorAliasLimits::new(
            MAX_PROFILE1_ANCHOR_DECLARATIONS,
            MAX_PROFILE1_ALIAS_BINDINGS,
        ),
        canonical_semantic_node_table_limits(),
        canonical_alias_cycle_limits(),
        canonical_scalar_key_limits(),
        canonical_structural_key_limits(),
    )
    .unwrap();
    let duplicate_free =
        reject_profile1_duplicate_keys(structural_keys, canonical_duplicate_key_limits()).unwrap();
    expand_profile1_merge_keys(duplicate_free, canonical_merge_expansion_limits()).unwrap()
}

fn input_node_kinds(source: &crucible_yaml::CanonicalYamlGraphSource) -> Vec<SemanticNodeKind> {
    source
        .input()
        .input()
        .structural_keys()
        .scalar_keys()
        .graph()
        .node_table()
        .nodes()
        .iter()
        .map(|node| node.kind())
        .collect()
}

fn resolve_input_node(source: &crucible_yaml::CanonicalYamlGraphSource, mut index: u64) -> u64 {
    let slots = source
        .input()
        .input()
        .structural_keys()
        .scalar_keys()
        .graph()
        .node_table()
        .nodes();
    for _ in 0..slots.len() {
        if slots[index as usize].kind() != SemanticNodeKind::Alias {
            return index;
        }
        index = slots[index as usize].alias_target_node_index().unwrap();
    }
    panic!("the authenticated acyclic graph must resolve every alias")
}

#[test]
fn lowering_resolves_aliases_and_projects_effective_merge_edges_without_tree_materialization() {
    let source = lower_profile1_canonical_graph(
        expanded(
            b"base: &base {a: one}\nalias: *base\nresult: {<<: *base, b: two}\nsequence: [*base]\n",
        ),
        canonical_lowering_limits(),
    )
    .unwrap();
    let input_kinds = input_node_kinds(&source);

    assert_eq!(source.nodes().len() as u64, source.input_node_count());
    assert_eq!(
        source.input().expanded_reference_count(),
        source.expanded_reference_count()
    );
    for (index, node) in source.nodes().iter().enumerate() {
        assert_eq!(node.source_node_index(), index as u64);
        assert_ne!(
            input_kinds[node.resolved_node_index() as usize],
            SemanticNodeKind::Alias
        );
    }

    let alias = source
        .nodes()
        .iter()
        .find(|node| input_kinds[node.source_node_index() as usize] == SemanticNodeKind::Alias)
        .unwrap();
    let target = &source.nodes()[alias.resolved_node_index() as usize];
    assert_eq!(alias.kind(), CanonicalYamlNodeKind::Mapping);
    assert_eq!(
        (alias.edge_start(), alias.edge_end()),
        (target.edge_start(), target.edge_end())
    );

    assert!(source
        .mapping_entries()
        .iter()
        .any(|entry| entry.inherited()));
    assert!(source.mapping_entries().iter().all(|entry| {
        input_kinds[entry.key_node_index() as usize] != SemanticNodeKind::Alias
            && input_kinds[entry.value_node_index() as usize] != SemanticNodeKind::Alias
    }));
    assert!(source.sequence_entries().iter().all(|entry| {
        input_kinds[entry.value_node_index() as usize] != SemanticNodeKind::Alias
    }));

    let input_entries = source.input().entries();
    assert_eq!(source.mapping_entries().len(), input_entries.len());
    for (lowered, input) in source.mapping_entries().iter().zip(input_entries) {
        assert_eq!(
            lowered.source_mapping_node_index(),
            input.source_mapping_node_index()
        );
        assert_eq!(lowered.source_edge_index(), input.source_edge_index());
        assert_eq!(lowered.inherited(), input.inherited());
        assert_eq!(
            lowered.key_node_index(),
            resolve_input_node(&source, input.key_node_index())
        );
        assert_eq!(
            lowered.value_node_index(),
            resolve_input_node(&source, input.value_node_index())
        );
    }

    let topology_edges = source
        .input()
        .input()
        .structural_keys()
        .scalar_keys()
        .graph()
        .node_table()
        .topology()
        .sequence_edges();
    assert_eq!(source.sequence_entries().len(), topology_edges.len());
    for (lowered, input) in source.sequence_entries().iter().zip(topology_edges) {
        assert_eq!(lowered.source_edge_index(), input.cst_entry_index());
        assert_eq!(
            lowered.value_node_index(),
            resolve_input_node(&source, input.child_node_index())
        );
    }
}

#[test]
fn every_document_root_and_scalar_identity_is_retained_in_the_owned_canonical_graph() {
    let source = lower_profile1_canonical_graph(
        expanded(b"---\n!local first\n...\n---\n[second, 0x10, true, null]\n"),
        canonical_lowering_limits(),
    )
    .unwrap();

    assert_eq!(source.document_roots().len(), 2);
    assert_eq!(source.document_roots()[0].document_index(), 0);
    assert_eq!(source.document_roots()[1].document_index(), 1);
    assert!(source.document_roots().iter().all(|root| {
        root.value_node_index()
            == source.nodes()[root.source_node_index() as usize].resolved_node_index()
    }));
    assert!(source.nodes().iter().any(|node| {
        node.kind() == CanonicalYamlNodeKind::Scalar && node.scalar_index().is_some()
    }));
    assert_eq!(
        source
            .input()
            .input()
            .structural_keys()
            .scalar_keys()
            .graph()
            .node_table()
            .scalars()
            .scalars()
            .len(),
        5
    );
}

#[test]
fn all_lowering_caps_are_independent_exact_and_cannot_raise_the_profile_maximum() {
    let canonical = canonical_lowering_limits();
    assert_eq!(MAX_PROFILE1_CANONICAL_NODES, 1_048_576);
    assert_eq!(MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES, 1_048_576);
    assert_eq!(MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES, 1_048_576);
    assert_eq!(MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS, 1_048_576);

    let cases = [
        (
            b"x\n".as_slice(),
            CanonicalLoweringLimits::new(
                0,
                canonical.max_sequence_entries(),
                canonical.max_mapping_entries(),
                canonical.max_document_roots(),
            ),
            CanonicalLoweringErrorKind::NodeLimitExceeded,
            0,
        ),
        (
            b"[x]\n".as_slice(),
            CanonicalLoweringLimits::new(
                canonical.max_nodes(),
                0,
                canonical.max_mapping_entries(),
                canonical.max_document_roots(),
            ),
            CanonicalLoweringErrorKind::SequenceEntryLimitExceeded,
            1,
        ),
        (
            b"a: b\n".as_slice(),
            CanonicalLoweringLimits::new(
                canonical.max_nodes(),
                canonical.max_sequence_entries(),
                0,
                canonical.max_document_roots(),
            ),
            CanonicalLoweringErrorKind::MappingEntryLimitExceeded,
            0,
        ),
        (
            b"x\n".as_slice(),
            CanonicalLoweringLimits::new(
                canonical.max_nodes(),
                canonical.max_sequence_entries(),
                canonical.max_mapping_entries(),
                0,
            ),
            CanonicalLoweringErrorKind::DocumentRootLimitExceeded,
            0,
        ),
    ];
    for (input, limits, expected_kind, expected_offset) in cases {
        let error = lower_profile1_canonical_graph(expanded(input), limits).unwrap_err();
        assert_eq!(error.kind(), expected_kind);
        assert_eq!(error.byte_offset(), expected_offset);
    }

    let exact_input = b"a: [b]\n";
    let probe =
        lower_profile1_canonical_graph(expanded(exact_input), canonical_lowering_limits()).unwrap();
    let exact = CanonicalLoweringLimits::new(
        probe.nodes().len() as u64,
        probe.sequence_entries().len() as u64,
        probe.mapping_entries().len() as u64,
        probe.document_roots().len() as u64,
    );
    let accepted = lower_profile1_canonical_graph(expanded(exact_input), exact).unwrap();
    assert_eq!(accepted.nodes().len(), probe.nodes().len());

    let raised = CanonicalLoweringLimits::new(u64::MAX, u64::MAX, u64::MAX, u64::MAX);
    assert_eq!(raised.max_nodes(), u64::MAX);
    let tiny = lower_profile1_canonical_graph(expanded(b"x\n"), raised).unwrap();
    assert_eq!(tiny.nodes().len(), 1);
}
