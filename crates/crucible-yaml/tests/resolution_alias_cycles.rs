use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, compose_profile1_semantic_node_table, decode_profile1,
    parse_profile1_cst, resolve_profile1_alias_cycles, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AliasCycleErrorKind, AliasCycleLimits, AnchorAliasLimits,
    AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy, CompletedTokenSource, CstNodeKind,
    CstSource, DecodeLimits, PlainScalarSource, QuotedScalarSource, SemanticNodeKind,
    MAX_PROFILE1_ALIAS_BINDINGS, MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SEMANTIC_DEPTH, MAX_PROFILE1_SEMANTIC_WORK_STACK,
    MAX_PROFILE1_SOURCE_BYTES,
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

fn anchor_limits() -> AnchorAliasLimits {
    AnchorAliasLimits::new(
        MAX_PROFILE1_ANCHOR_DECLARATIONS,
        MAX_PROFILE1_ALIAS_BINDINGS,
    )
}

fn resolve(
    parsed: &Parsed,
    limits: AliasCycleLimits,
) -> Result<crucible_yaml::AcyclicSemanticGraphSource, crucible_yaml::AliasCycleError> {
    resolve_profile1_alias_cycles(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        canonical_semantic_scalar_table_limits(),
        anchor_limits(),
        canonical_semantic_node_table_limits(),
        limits,
    )
}

#[test]
fn acyclic_alias_sharing_retains_one_owned_node_table_and_visits_every_node_once() {
    let parsed = parse(b"base: &base [one]\nleft: *base\nright: *base\nnested: {copy: *base}\n");
    let expected = compose_profile1_semantic_node_table(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        canonical_semantic_scalar_table_limits(),
        anchor_limits(),
        canonical_semantic_node_table_limits(),
    )
    .expect("the pre-cycle node table is exact");
    let graph = resolve(&parsed, canonical_alias_cycle_limits()).expect("the graph is acyclic");

    assert_eq!(graph.node_table(), &expected);
    assert_eq!(graph.input_node_count(), expected.nodes().len() as u64);
    assert_eq!(
        graph.input_alias_count(),
        expected.alias_redirects().len() as u64
    );
    assert_eq!(graph.visit_order().len(), expected.nodes().len());
    assert_eq!(graph.node_depths().len(), expected.nodes().len());
    assert_eq!(graph.visit_states().len(), expected.nodes().len());
    assert!(graph
        .visit_states()
        .iter()
        .all(|state| *state == crucible_yaml::SemanticVisitState::Complete));
    assert!(graph.max_depth_observed() >= 1);
    assert!(graph.max_depth_observed() <= MAX_PROFILE1_SEMANTIC_DEPTH);
    assert_eq!(
        graph.deepest_path().len() as u64,
        graph.max_depth_observed()
    );
    assert_eq!(
        graph.node_depths()[graph.deepest_path()[0] as usize],
        graph.max_depth_observed()
    );
    assert_eq!(
        graph.node_depths()[*graph.deepest_path().last().expect("nonempty path") as usize],
        1
    );
    for pair in graph.deepest_path().windows(2) {
        assert!(pair[1] < pair[0]);
        assert_eq!(
            graph.node_depths()[pair[0] as usize],
            graph.node_depths()[pair[1] as usize] + 1
        );
    }

    let mut visits = graph.visit_order().to_vec();
    visits.sort_unstable();
    assert_eq!(
        visits,
        (0..expected.nodes().len() as u64).collect::<Vec<_>>()
    );

    let redirects = expected.alias_redirects();
    assert_eq!(redirects.len(), 3);
    assert!(redirects
        .iter()
        .all(|redirect| redirect.target_node_index() == redirects[0].target_node_index()));
    for redirect in redirects {
        let slot = &expected.nodes()[redirect.alias_node_index() as usize];
        assert_eq!(slot.kind(), SemanticNodeKind::Alias);
        assert_eq!(
            slot.alias_target_node_index(),
            Some(redirect.target_node_index())
        );
    }
}

#[test]
fn direct_and_nested_cycles_fail_at_the_alias_edge_that_closes_the_path() {
    for input in [&b"&a [*a]\n"[..], &b"&a {child: [&b [*a]]}\n"[..]] {
        let parsed = parse(input);
        let table = compose_profile1_semantic_node_table(
            &parsed.atoms,
            &parsed.quoted,
            &parsed.plain,
            &parsed.block,
            &parsed.tokens,
            &parsed.cst,
            canonical_semantic_topology_limits(),
            canonical_semantic_scalar_table_limits(),
            anchor_limits(),
            canonical_semantic_node_table_limits(),
        )
        .expect("cycle rejection is deliberately later than node-table composition");
        let redirect = table
            .alias_redirects()
            .last()
            .expect("the closing alias has an exact redirect");
        let error = resolve(&parsed, canonical_alias_cycle_limits())
            .expect_err("direct and indirect ancestor cycles are rejected");
        assert_eq!(error.kind(), AliasCycleErrorKind::AliasCycle);
        assert_eq!(error.byte_offset(), redirect.name_byte_start());
        assert_eq!(error.alias_node_index(), Some(redirect.alias_node_index()));
        assert_eq!(
            error.target_node_index(),
            Some(redirect.target_node_index())
        );
        assert!(redirect.target_node_index() > redirect.alias_node_index());
    }
}

#[test]
fn cycle_errors_precede_lowered_traversal_caps_and_caps_name_the_first_excluded_node() {
    let cyclic = parse(b"&a [*a]\n");
    let error = resolve(&cyclic, AliasCycleLimits::new(0, 0))
        .expect_err("an intrinsic cycle precedes caller resource caps");
    assert_eq!(error.kind(), AliasCycleErrorKind::AliasCycle);

    let parsed = parse(b"[[value]]\n");
    let root = parsed.cst.documents()[0].root_node_index() as usize;
    assert_eq!(parsed.cst.nodes()[root].kind(), CstNodeKind::Sequence);
    let child = parsed.cst.nodes()[root].entry_start() as usize;
    let child_node = parsed.cst.sequence_entries()[child].node_index() as usize;

    let error = resolve(
        &parsed,
        AliasCycleLimits::new(1, MAX_PROFILE1_SEMANTIC_WORK_STACK),
    )
    .expect_err("the nested collection exceeds depth one");
    assert_eq!(
        error.kind(),
        AliasCycleErrorKind::SemanticDepthLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        parsed.cst.nodes()[child_node].byte_start()
    );

    let error = resolve(
        &parsed,
        AliasCycleLimits::new(MAX_PROFILE1_SEMANTIC_DEPTH, 1),
    )
    .expect_err("the nested collection exceeds a one-frame work stack");
    assert_eq!(error.kind(), AliasCycleErrorKind::WorkStackLimitExceeded);
    assert_eq!(
        error.byte_offset(),
        parsed.cst.nodes()[child_node].byte_start()
    );
}

#[test]
fn node_table_authentication_errors_are_preserved_without_partial_graph_output() {
    let parsed = parse(b"[one]\n");
    let foreign = parse(b"different\n");
    let error = resolve_profile1_alias_cycles(
        &parsed.atoms,
        &foreign.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        canonical_semantic_topology_limits(),
        canonical_semantic_scalar_table_limits(),
        anchor_limits(),
        canonical_semantic_node_table_limits(),
        canonical_alias_cycle_limits(),
    )
    .expect_err("foreign scalar evidence cannot enter the owned graph");
    assert!(matches!(
        error.kind(),
        AliasCycleErrorKind::NodeTable(crucible_yaml::SemanticNodeTableErrorKind::ScalarTable(_))
    ));
}
