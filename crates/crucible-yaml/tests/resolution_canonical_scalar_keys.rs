use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, compose_profile1_canonical_scalar_keys, decode_profile1,
    parse_profile1_cst, scan_profile1_block_scalars, scan_profile1_completed_tokens,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AnchorAliasLimits, AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy,
    CanonicalScalarKeyErrorKind, CanonicalScalarKeyLimits, CompletedTokenSource, CstSource,
    DecodeLimits, PlainScalarSource, QuotedScalarSource, MAX_PROFILE1_ALIAS_BINDINGS,
    MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
    MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
    MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
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

fn compose(
    parsed: &Parsed,
    limits: CanonicalScalarKeyLimits,
) -> Result<crucible_yaml::CanonicalScalarKeySource, crucible_yaml::CanonicalScalarKeyError> {
    compose_profile1_canonical_scalar_keys(
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
        canonical_alias_cycle_limits(),
        limits,
    )
}

fn value_nodes(parsed: &Parsed) -> Vec<u64> {
    let root = parsed.cst.documents()[0].root_node_index() as usize;
    let node = &parsed.cst.nodes()[root];
    parsed.cst.mapping_entries()[node.entry_start() as usize..node.entry_end() as usize]
        .iter()
        .map(|entry| entry.value_node_index())
        .collect()
}

#[test]
fn normalized_scalar_values_have_exact_style_independent_collision_free_keys() {
    let parsed = parse(
        b"a: 1\nb: 01\nc: \"1\"\nd: !!str 1\ne: TRUE\nf: true\ng: null\nh: ~\n\
          i: 1.0\nj: 10e-1\nk: !custom x\nl: !custom \"x\"\nm: !other x\n",
    );
    let nodes = value_nodes(&parsed);
    let source = compose(&parsed, canonical_scalar_key_limits())
        .expect("every scalar receives a canonical semantic key");

    assert_eq!(source.input_node_count(), parsed.cst.nodes().len() as u64);
    assert_eq!(source.records().len(), nodes.len() * 2);
    assert_eq!(
        source.total_key_bytes(),
        source
            .records()
            .iter()
            .map(|record| record.bytes().len() as u64)
            .sum::<u64>()
    );

    let key = |node_index: u64| {
        source
            .records()
            .iter()
            .find(|record| record.node_index() == node_index)
            .expect("scalar key record")
            .bytes()
            .iter()
            .map(|byte| byte.value())
            .collect::<Vec<_>>()
    };

    assert_eq!(key(nodes[0]), key(nodes[1]));
    assert_ne!(key(nodes[0]), key(nodes[2]));
    assert_eq!(key(nodes[2]), key(nodes[3]));
    assert_eq!(key(nodes[4]), key(nodes[5]));
    assert_eq!(key(nodes[6]), key(nodes[7]));
    assert_eq!(key(nodes[8]), key(nodes[9]));
    assert_eq!(key(nodes[10]), key(nodes[11]));
    assert_ne!(key(nodes[10]), key(nodes[12]));

    for record in source.records() {
        assert!(!record.bytes().is_empty());
        assert!(record
            .bytes()
            .iter()
            .all(|byte| byte.source_byte_offset() <= parsed.atoms.source_len_bytes()));
    }
}

#[test]
fn independent_record_key_and_aggregate_caps_report_exact_source_anchors() {
    let parsed = parse(b"first: one\nsecond: two\n");
    let scalar_nodes: Vec<_> = parsed
        .cst
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (node.kind() == crucible_yaml::CstNodeKind::Scalar).then_some(index)
        })
        .collect();

    let error = compose(
        &parsed,
        CanonicalScalarKeyLimits::new(
            0,
            MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
            MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
        ),
    )
    .expect_err("the first scalar record is excluded");
    assert_eq!(
        error.kind(),
        CanonicalScalarKeyErrorKind::RecordLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        parsed.cst.nodes()[scalar_nodes[0]].byte_start()
    );

    let error = compose(
        &parsed,
        CanonicalScalarKeyLimits::new(
            MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS,
            0,
            MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
        ),
    )
    .expect_err("the first generated byte is excluded");
    assert_eq!(
        error.kind(),
        CanonicalScalarKeyErrorKind::KeyByteLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        parsed.cst.nodes()[scalar_nodes[0]].byte_start()
    );

    let error = compose(
        &parsed,
        CanonicalScalarKeyLimits::new(
            MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS,
            MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
            1,
        ),
    )
    .expect_err("the second aggregate byte is excluded");
    assert_eq!(
        error.kind(),
        CanonicalScalarKeyErrorKind::TotalKeyByteLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        parsed.cst.nodes()[scalar_nodes[0]].byte_start()
    );
}

#[test]
fn upstream_acyclic_graph_authentication_is_preserved() {
    let parsed = parse(b"value: one\n");
    let foreign = parse(b"different\n");
    let error = compose_profile1_canonical_scalar_keys(
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
        canonical_scalar_key_limits(),
    )
    .expect_err("foreign scalar evidence cannot enter canonical identity");
    assert!(matches!(
        error.kind(),
        CanonicalScalarKeyErrorKind::AliasCycle(crucible_yaml::AliasCycleErrorKind::NodeTable(_))
    ));
}
