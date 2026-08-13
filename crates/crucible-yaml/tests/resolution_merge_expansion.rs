use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_duplicate_key_limits, canonical_merge_expansion_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_key_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    compose_profile1_canonical_structural_keys, decode_profile1, expand_profile1_merge_keys,
    parse_profile1_cst, reject_profile1_duplicate_keys, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AnchorAliasLimits, AtomizeLimits, AtomizedSource,
    BlockScalarSource, BomPolicy, CompletedTokenSource, CstNodeKind, CstSource, DecodeLimits,
    DuplicateFreeStructuralKeySource, ExpandedSemanticGraphSource, MergeExpansionError,
    MergeExpansionErrorKind, MergeExpansionLimits, PlainScalarSource, QuotedScalarSource,
    MAX_PROFILE1_ALIAS_BINDINGS, MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES, MAX_PROFILE1_EXPANDED_REFERENCES,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_MERGE_MAPPING_RECORDS, MAX_PROFILE1_MERGE_SOURCES,
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
    Parsed {
        atoms,
        quoted,
        plain,
        block,
        tokens,
        cst,
    }
}

fn duplicate_free(parsed: &Parsed) -> DuplicateFreeStructuralKeySource {
    let structural = compose_profile1_canonical_structural_keys(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
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
    reject_profile1_duplicate_keys(structural, canonical_duplicate_key_limits()).unwrap()
}

fn expand(
    input: &[u8],
    limits: MergeExpansionLimits,
) -> Result<ExpandedSemanticGraphSource, MergeExpansionError> {
    let parsed = parse(input);
    expand_profile1_merge_keys(duplicate_free(&parsed), limits)
}

fn mapping_entries(
    source: &ExpandedSemanticGraphSource,
    mapping_node_index: u64,
) -> Vec<(u64, u64, u64, bool)> {
    let record = source
        .mappings()
        .iter()
        .find(|record| record.node_index() == mapping_node_index)
        .unwrap();
    source.entries()[record.entry_start() as usize..record.entry_end() as usize]
        .iter()
        .map(|entry| {
            (
                entry.key_node_index(),
                entry.value_node_index(),
                entry.source_mapping_node_index(),
                entry.inherited(),
            )
        })
        .collect()
}

#[test]
fn plain_merge_key_expands_aliases_and_quoted_spelling_remains_ordinary() {
    let parsed = parse(
        b"base: &base {a: one, b: two}\nresult: {<<: *base, c: three}\nquoted: {\"<<\": ordinary}\n",
    );
    let roots = parsed.cst.documents()[0].root_node_index() as usize;
    let root_entries = &parsed.cst.mapping_entries()[parsed.cst.nodes()[roots].entry_start()
        as usize
        ..parsed.cst.nodes()[roots].entry_end() as usize];
    let result_node = root_entries[1].value_node_index();
    let quoted_node = root_entries[2].value_node_index();
    let source =
        expand_profile1_merge_keys(duplicate_free(&parsed), canonical_merge_expansion_limits())
            .unwrap();

    let result = mapping_entries(&source, result_node);
    assert_eq!(result.len(), 3);
    assert!(!result[0].3, "explicit entries precede inherited entries");
    assert!(result[1].3 && result[2].3);
    assert_eq!(result[1].2, root_entries[0].value_node_index());
    assert_eq!(result[2].2, root_entries[0].value_node_index());

    let quoted = mapping_entries(&source, quoted_node);
    assert_eq!(quoted.len(), 1);
    assert!(!quoted[0].3, "quoted << is an ordinary key");
}

#[test]
fn merge_sequence_uses_earlier_sources_and_explicit_receivers_override_all_inherited_values() {
    let parsed = parse(
        b"left: &left {a: left, shared: left}\nright: &right {b: right, shared: right}\nresult: {before: kept, <<: [*left, *right], shared: explicit, after: kept}\n",
    );
    let root = parsed.cst.documents()[0].root_node_index() as usize;
    let root_entries = &parsed.cst.mapping_entries()[parsed.cst.nodes()[root].entry_start() as usize
        ..parsed.cst.nodes()[root].entry_end() as usize];
    let result_node = root_entries[2].value_node_index();
    let source =
        expand_profile1_merge_keys(duplicate_free(&parsed), canonical_merge_expansion_limits())
            .unwrap();
    let result = mapping_entries(&source, result_node);

    assert_eq!(result.len(), 5);
    assert!(result[..3].iter().all(|entry| !entry.3));
    assert!(result[3..].iter().all(|entry| entry.3));
    assert_eq!(result[3].2, root_entries[0].value_node_index());
    assert_eq!(result[4].2, root_entries[1].value_node_index());
}

#[test]
fn explicit_merge_tag_and_nested_merges_are_expanded_without_materializing_trees() {
    let source = expand(
        b"base: &base {a: one}\nmid: &mid {!!merge ignored: *base, b: two}\ntop: {<<: *mid, c: three}\n",
        canonical_merge_expansion_limits(),
    )
    .unwrap();
    assert!(source.mappings().iter().any(|mapping| {
        mapping_entries(&source, mapping.node_index())
            .iter()
            .filter(|entry| entry.3)
            .count()
            >= 2
    }));
    assert!(source.expanded_reference_count() > source.entries().len() as u64);
}

#[test]
fn direct_mapping_merge_values_are_supported_without_alias_materialization() {
    let source = expand(
        b"result: {<<: {a: one, b: two}, c: three}\n",
        canonical_merge_expansion_limits(),
    )
    .unwrap();
    let result = source
        .mappings()
        .iter()
        .find(|mapping| mapping_entries(&source, mapping.node_index()).len() == 3)
        .unwrap();
    let entries = mapping_entries(&source, result.node_index());
    assert!(!entries[0].3);
    assert!(entries[1].3 && entries[2].3);
}

#[test]
fn full_tree_reference_accounting_handles_deep_collection_cursor_work_exactly() {
    let input = b"[[[[[[[[x]]]]]]]]\n";
    let canonical = canonical_merge_expansion_limits();
    let source = expand(
        input,
        MergeExpansionLimits::new(
            canonical.max_mappings(),
            canonical.max_expanded_mapping_entries(),
            9,
            canonical.max_merge_sources(),
        ),
    )
    .expect("eight sequences and one scalar are nine materialized node occurrences");
    assert_eq!(source.expanded_reference_count(), 9);

    let error = expand(
        input,
        MergeExpansionLimits::new(
            canonical.max_mappings(),
            canonical.max_expanded_mapping_entries(),
            8,
            canonical.max_merge_sources(),
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        MergeExpansionErrorKind::ExpandedReferenceLimitExceeded
    );
    assert_eq!(error.byte_offset(), 8);
}

#[test]
fn invalid_merge_shapes_have_typed_exact_value_diagnostics() {
    for (input, expected_offset) in [
        (&b"{<<: scalar}\n"[..], 5u64),
        (&b"{<<: [{a: one}, scalar]}\n"[..], 16u64),
    ] {
        let error = expand(input, canonical_merge_expansion_limits()).unwrap_err();
        assert_eq!(error.kind(), MergeExpansionErrorKind::InvalidMergeValue);
        assert_eq!(error.byte_offset(), expected_offset);
    }
}

#[test]
fn merge_caps_are_independent_exact_and_intrinsic_errors_precede_them() {
    let input = b"base: &base {a: one}\nresult: {<<: *base, b: two}\n";
    let full = expand(input, canonical_merge_expansion_limits()).unwrap();
    let mappings = full.mappings().len() as u64;
    let entries = full.entries().len() as u64;
    let references = full.expanded_reference_count();
    let sources = full.merge_source_count();

    expand(
        input,
        MergeExpansionLimits::new(mappings, entries, references, sources),
    )
    .expect("all exact accepted cap boundaries are inclusive");

    for (limits, expected) in [
        (
            MergeExpansionLimits::new(0, entries, references, sources),
            MergeExpansionErrorKind::MappingLimitExceeded,
        ),
        (
            MergeExpansionLimits::new(mappings, 0, references, sources),
            MergeExpansionErrorKind::ExpandedMappingEntryLimitExceeded,
        ),
        (
            MergeExpansionLimits::new(mappings, entries, 0, sources),
            MergeExpansionErrorKind::ExpandedReferenceLimitExceeded,
        ),
        (
            MergeExpansionLimits::new(mappings, entries, references, 0),
            MergeExpansionErrorKind::MergeSourceLimitExceeded,
        ),
    ] {
        assert_eq!(expand(input, limits).unwrap_err().kind(), expected);
    }

    let invalid = b"{<<: scalar}\n";
    let error = expand(invalid, MergeExpansionLimits::new(0, 0, 0, 0)).unwrap_err();
    assert_eq!(error.kind(), MergeExpansionErrorKind::InvalidMergeValue);

    let canonical = canonical_merge_expansion_limits();
    assert_eq!(canonical.max_mappings(), MAX_PROFILE1_MERGE_MAPPING_RECORDS);
    assert_eq!(
        canonical.max_expanded_mapping_entries(),
        MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES
    );
    assert_eq!(
        canonical.max_expanded_references(),
        MAX_PROFILE1_EXPANDED_REFERENCES
    );
    assert_eq!(canonical.max_merge_sources(), MAX_PROFILE1_MERGE_SOURCES);
    assert_eq!(canonical.max_mappings(), 1_048_576);
    assert_eq!(canonical.max_expanded_mapping_entries(), 1_048_576);
    assert_eq!(canonical.max_expanded_references(), 1_048_576);
    assert_eq!(canonical.max_merge_sources(), 1_048_576);
}

#[test]
fn every_mapping_is_retained_even_without_merges() {
    let parsed = parse(b"[{a: one}, {b: two}]\n");
    let expected = parsed
        .cst
        .nodes()
        .iter()
        .filter(|node| node.kind() == CstNodeKind::Mapping)
        .count();
    let source =
        expand_profile1_merge_keys(duplicate_free(&parsed), canonical_merge_expansion_limits())
            .unwrap();
    assert_eq!(source.mappings().len(), expected);
    assert_eq!(source.merge_source_count(), 0);
}
