use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_key_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    compose_profile1_canonical_structural_keys, decode_profile1, parse_profile1_cst,
    scan_profile1_block_scalars, scan_profile1_completed_tokens, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AnchorAliasLimits,
    AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy, CanonicalStructuralKeyErrorKind,
    CanonicalStructuralKeyLimits, CompletedTokenSource, CstSource, DecodeLimits, PlainScalarSource,
    QuotedScalarSource, MAX_PROFILE1_ALIAS_BINDINGS, MAX_PROFILE1_ANCHOR_DECLARATIONS,
    MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES, MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_MAPPING_SORT_ENTRIES,
    MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
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
    limits: CanonicalStructuralKeyLimits,
) -> Result<crucible_yaml::CanonicalStructuralKeySource, crucible_yaml::CanonicalStructuralKeyError>
{
    compose_profile1_canonical_structural_keys(
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
        canonical_scalar_key_limits(),
        limits,
    )
}

fn key(source: &crucible_yaml::CanonicalStructuralKeySource, node_index: u64) -> Vec<u8> {
    source.records()[node_index as usize]
        .bytes()
        .iter()
        .map(|byte| byte.value())
        .collect()
}

#[test]
fn collections_have_exact_alias_transparent_and_order_aware_structural_identities() {
    let parsed = parse(
        b"---\n{a: 1, b: [true, null]}\n---\nb:\n  - TRUE\n  - ~\na: 01\n\
          ---\n[1, 2]\n---\n[2, 1]\n---\n!one [x]\n---\n!one [\"x\"]\n---\n!two [x]\n",
    );
    let source = compose(&parsed, canonical_structural_key_limits())
        .expect("every semantic node receives one structural identity");
    let roots: Vec<_> = parsed
        .cst
        .documents()
        .iter()
        .map(|document| document.root_node_index())
        .collect();

    assert_eq!(source.records().len(), parsed.cst.nodes().len());
    assert!(source
        .records()
        .iter()
        .enumerate()
        .all(|(index, record)| record.node_index() == index as u64));
    assert_eq!(key(&source, roots[0]), key(&source, roots[1]));
    assert_ne!(key(&source, roots[2]), key(&source, roots[3]));
    assert_eq!(key(&source, roots[4]), key(&source, roots[5]));
    assert_ne!(key(&source, roots[4]), key(&source, roots[6]));
    assert_eq!(
        source.total_key_bytes(),
        source
            .records()
            .iter()
            .map(|record| record.bytes().len() as u64)
            .sum::<u64>()
    );

    let aliases = parse(b"root: {original: &a [1, 2], copy: *a}\n");
    let alias_source = compose(&aliases, canonical_structural_key_limits()).unwrap();
    let root = aliases.cst.documents()[0].root_node_index() as usize;
    let root_node = &aliases.cst.nodes()[root];
    let nested =
        aliases.cst.mapping_entries()[root_node.entry_start() as usize].value_node_index() as usize;
    let nested_node = &aliases.cst.nodes()[nested];
    let values: Vec<_> = aliases.cst.mapping_entries()
        [nested_node.entry_start() as usize..nested_node.entry_end() as usize]
        .iter()
        .map(|entry| entry.value_node_index())
        .collect();
    assert_eq!(key(&alias_source, values[0]), key(&alias_source, values[1]));
}

#[test]
fn custom_collection_tag_code_points_retain_exact_source_provenance() {
    let parsed = parse(b"!one [x]\n");
    let source = compose(&parsed, canonical_structural_key_limits()).unwrap();
    let root = parsed.cst.documents()[0].root_node_index() as usize;
    let bytes = source.records()[root].bytes();

    // Six collection-header bytes, one local-tag marker, and one eight-byte tag length precede
    // the four-byte big-endian encoding of every resolved tag code point.
    assert!(bytes.len() >= 31);
    let sources: Vec<_> = bytes[15..31]
        .iter()
        .map(|byte| byte.source_byte_offset())
        .collect();
    assert_eq!(
        sources,
        vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]
    );
}

#[test]
fn nested_collection_keys_and_mapping_entry_order_use_exact_recursive_identity() {
    let parsed = parse(b"---\n? [a, b]\n: {x: 1, y: 2}\n---\n? [\"a\", b]\n: {y: 02, x: 01}\n");
    let source = compose(&parsed, canonical_structural_key_limits()).unwrap();
    let roots: Vec<_> = parsed
        .cst
        .documents()
        .iter()
        .map(|document| document.root_node_index())
        .collect();
    assert_eq!(key(&source, roots[0]), key(&source, roots[1]));
}

#[test]
fn structural_record_key_total_and_sort_caps_are_independent_and_exact() {
    let parsed = parse(b"{first: one, second: two}\n");
    let first_node_byte = parsed.cst.nodes()[0].byte_start();

    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            0,
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_MAPPING_SORT_ENTRIES,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::RecordLimitExceeded
    );
    assert_eq!(error.byte_offset(), first_node_byte);

    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
            0,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_MAPPING_SORT_ENTRIES,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::KeyByteLimitExceeded
    );
    assert_eq!(error.byte_offset(), first_node_byte);

    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
            0,
            MAX_PROFILE1_MAPPING_SORT_ENTRIES,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::TotalKeyByteLimitExceeded
    );
    assert_eq!(error.byte_offset(), first_node_byte);

    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            0,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::MappingSortLimitExceeded
    );
    let root = parsed.cst.documents()[0].root_node_index() as usize;
    assert_eq!(error.byte_offset(), parsed.cst.nodes()[root].byte_start());
}

#[test]
fn every_mapping_permutation_and_equal_key_pair_order_has_one_identity() {
    let parsed = parse(
        b"---\n{a: 1, b: 2, c: 3}\n---\n{a: 1, c: 3, b: 2}\n\
          ---\n{b: 2, a: 1, c: 3}\n---\n{b: 2, c: 3, a: 1}\n\
          ---\n{c: 3, a: 1, b: 2}\n---\n{c: 3, b: 2, a: 1}\n\
          ---\n{a: 1, a: 2}\n---\n{a: 2, a: 1}\n",
    );
    let source = compose(&parsed, canonical_structural_key_limits()).unwrap();
    let roots: Vec<_> = parsed
        .cst
        .documents()
        .iter()
        .map(|document| document.root_node_index())
        .collect();

    for root in &roots[1..6] {
        assert_eq!(key(&source, roots[0]), key(&source, *root));
    }
    // Duplicate rejection is the next machine. This layer deliberately canonicalizes the full
    // entry multiset independently of presentation order, including equal-key/different-value
    // pairs, so the later diagnostic cannot depend on source permutation.
    assert_eq!(key(&source, roots[6]), key(&source, roots[7]));
}

#[test]
fn exact_structural_cap_boundaries_accept_the_last_included_unit() {
    let parsed = parse(b"{first: one, second: [two, three]}\n");
    let full = compose(&parsed, canonical_structural_key_limits()).unwrap();
    let record_count = full.records().len() as u64;
    let largest_key = full
        .records()
        .iter()
        .map(|record| record.bytes().len() as u64)
        .max()
        .unwrap();
    let total = full.total_key_bytes();

    compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            record_count,
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            2,
        ),
    )
    .expect("the exact record boundary is inclusive");
    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            record_count - 1,
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            2,
        ),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::RecordLimitExceeded
    );
    assert_eq!(
        error.byte_offset(),
        parsed.cst.nodes()[(record_count - 1) as usize].byte_start()
    );

    compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(
            record_count,
            largest_key,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            2,
        ),
    )
    .expect("the exact per-key byte boundary is inclusive");
    let first_oversized = full
        .records()
        .iter()
        .find(|record| record.bytes().len() as u64 > largest_key - 1)
        .unwrap();
    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(record_count, largest_key - 1, total, 2),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::KeyByteLimitExceeded
    );
    assert_eq!(error.byte_offset(), first_oversized.byte_start());

    compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(record_count, largest_key, total, 2),
    )
    .expect("the exact aggregate byte boundary is inclusive");
    let mut cumulative = 0u64;
    let first_total_excluded = full
        .records()
        .iter()
        .find(|record| {
            cumulative += record.bytes().len() as u64;
            cumulative > total - 1
        })
        .unwrap();
    let error = compose(
        &parsed,
        CanonicalStructuralKeyLimits::new(record_count, largest_key, total - 1, 2),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        CanonicalStructuralKeyErrorKind::TotalKeyByteLimitExceeded
    );
    assert_eq!(error.byte_offset(), first_total_excluded.byte_start());

    let empty = parse(b"{}\n");
    compose(
        &empty,
        CanonicalStructuralKeyLimits::new(
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
            0,
        ),
    )
    .expect("zero mapping entries require zero sorting slots");
}
