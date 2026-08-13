use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_duplicate_key_limits, canonical_plain_scalar_limits, canonical_quoted_scalar_limits,
    canonical_scalar_key_limits, canonical_semantic_node_table_limits,
    canonical_semantic_scalar_table_limits, canonical_semantic_topology_limits,
    canonical_structural_key_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, compose_profile1_canonical_structural_keys, decode_profile1,
    parse_profile1_cst, reject_profile1_duplicate_keys, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AnchorAliasLimits, AtomizeLimits, AtomizedSource,
    BlockScalarSource, BomPolicy, CanonicalStructuralKeySource, CompletedTokenSource, CstSource,
    DecodeLimits, DuplicateFreeStructuralKeySource, DuplicateKeyError, DuplicateKeyErrorKind,
    DuplicateKeyLimits, PlainScalarSource, QuotedScalarSource, MAX_PROFILE1_ALIAS_BINDINGS,
    MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS, MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
};
use std::fmt::Write as _;

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

fn structural_keys(parsed: &Parsed) -> CanonicalStructuralKeySource {
    compose_profile1_canonical_structural_keys(
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
    .unwrap()
}

fn reject(
    input: &[u8],
    limits: DuplicateKeyLimits,
) -> Result<DuplicateFreeStructuralKeySource, DuplicateKeyError> {
    let parsed = parse(input);
    reject_profile1_duplicate_keys(structural_keys(&parsed), limits)
}

fn duplicate_error(input: &[u8]) -> DuplicateKeyError {
    reject(input, canonical_duplicate_key_limits()).expect_err("duplicate key must be rejected")
}

#[test]
fn scalar_keys_compare_by_resolved_tag_and_canonical_semantic_value() {
    for (input, expected_offset) in [
        (&b"{1: first, 01: second}\n"[..], 11u64),
        (&b"{true: first, TRUE: second}\n"[..], 14u64),
        (&b"{\"x\": first, x: second}\n"[..], 13u64),
        (&b"{\"x\": first, !!str x: second}\n"[..], 13u64),
        (&b"{ : first, : second}\n"[..], 11u64),
    ] {
        let error = duplicate_error(input);
        assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
        assert_eq!(error.byte_offset(), expected_offset);
    }

    reject(
        b"{1: integer, \"1\": string, !one x: local, !two x: other}\n",
        canonical_duplicate_key_limits(),
    )
    .expect("different resolved tags or values remain distinct");
}

#[test]
fn the_first_later_duplicate_in_mapping_source_order_is_diagnostic() {
    let input = b"{a: zero, b: one, b: two, a: three}\n";
    let second_b = input
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte == b'b')
        .nth(1)
        .unwrap()
        .0 as u64;
    let error = duplicate_error(input);
    assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
    assert_eq!(error.byte_offset(), second_b);
}

#[test]
fn the_earliest_later_duplicate_wins_across_nested_mapping_node_order() {
    let input = b"a: one\na: {b: one, b: two}\n";
    let second_a = input
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte == b'a')
        .nth(1)
        .unwrap()
        .0 as u64;
    let error = duplicate_error(input);
    assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
    assert_eq!(error.byte_offset(), second_a);
}

#[test]
fn a_wide_mapping_is_checked_iteratively_and_a_late_duplicate_is_exact() {
    let mut input = String::from("{");
    for index in 0..257u64 {
        if index != 0 {
            input.push_str(", ");
        }
        write!(&mut input, "k{index:03}: {index}").unwrap();
    }
    input.push_str("}\n");
    let source = reject(input.as_bytes(), canonical_duplicate_key_limits()).unwrap();
    assert_eq!(source.checked_mapping_count(), 1);
    assert_eq!(source.checked_mapping_entry_count(), 257);

    let insert = input.len() - 2;
    input.insert_str(insert, ", k000: duplicate");
    let expected = input.rfind("k000").unwrap() as u64;
    let error = duplicate_error(input.as_bytes());
    assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
    assert_eq!(error.byte_offset(), expected);
}

#[test]
fn aliases_and_recursive_collection_keys_use_exact_structural_equality() {
    let alias = b"? &key [a, b]\n: first\n? *key\n: second\n";
    let error = duplicate_error(alias);
    assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
    assert_eq!(error.byte_offset(), 24);

    let mapping = b"? {a: 1, b: 2}\n: first\n? {b: 02, a: 01}\n: second\n";
    let error = duplicate_error(mapping);
    assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
    assert_eq!(error.byte_offset(), 25);

    reject(
        b"? [a, b]\n: first\n? [b, a]\n: second\n? !one [x]\n: local\n? !two [x]\n: other\n",
        canonical_duplicate_key_limits(),
    )
    .expect("sequence order and complete collection tag identity remain significant");
}

#[test]
fn duplicate_scope_is_one_mapping_and_success_retains_exact_accounting() {
    let source = reject(
        b"- {a: one, b: two}\n- {a: three}\n",
        canonical_duplicate_key_limits(),
    )
    .expect("equal keys in separate mappings are not duplicates");
    assert_eq!(source.checked_mapping_count(), 2);
    assert_eq!(source.checked_mapping_entry_count(), 3);
    assert_eq!(
        source.structural_keys().records().len(),
        source.structural_keys().input_node_count() as usize
    );
}

#[test]
fn duplicate_diagnostics_precede_a_limit_excluding_the_same_key() {
    let duplicate = b"{a: one, a: two}\n";
    let error = reject(duplicate, DuplicateKeyLimits::new(0, 1)).unwrap_err();
    assert_eq!(error.kind(), DuplicateKeyErrorKind::DuplicateExplicitKey);
    assert_eq!(error.byte_offset(), 9);

    let unique = b"{a: one, b: two}\n";
    let error = reject(
        unique,
        DuplicateKeyLimits::new(MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS, 1),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        DuplicateKeyErrorKind::MappingEntryLimitExceeded
    );
    assert_eq!(error.byte_offset(), 9);
}

#[test]
fn mapping_and_entry_limits_are_independent_exact_and_cannot_raise_absolute_caps() {
    let parsed = parse(b"{a: one, b: two}\n");
    let structural = structural_keys(&parsed);
    let root = parsed.cst.documents()[0].root_node_index() as usize;
    let root_byte = parsed.cst.nodes()[root].byte_start();
    let error =
        reject_profile1_duplicate_keys(structural, DuplicateKeyLimits::new(0, 2)).unwrap_err();
    assert_eq!(error.kind(), DuplicateKeyErrorKind::MappingLimitExceeded);
    assert_eq!(error.byte_offset(), root_byte);

    let accepted = reject(b"{a: one, b: two}\n", DuplicateKeyLimits::new(1, 2))
        .expect("the exact accepted boundary is inclusive");
    assert_eq!(accepted.checked_mapping_count(), 1);
    assert_eq!(accepted.checked_mapping_entry_count(), 2);

    let raised = DuplicateKeyLimits::new(u64::MAX, u64::MAX);
    assert_eq!(
        raised.max_mappings(),
        u64::MAX,
        "the request is retained while execution clamps it to the profile cap"
    );
    assert_eq!(raised.max_mapping_entries(), u64::MAX);
    assert!(MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS < raised.max_mappings());
    assert!(MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES < raised.max_mapping_entries());
}
