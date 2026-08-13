use vstd::prelude::*;

verus! {

proof fn successful_duplicate_check_has_exact_public_semantics(
    input: crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeySourceView,
    limits: crucible_yaml::resolve_duplicate_key::DuplicateKeyLimitsView,
    output: crucible_yaml::resolve_duplicate_key::DuplicateFreeStructuralKeySourceView,
)
    requires
        crucible_yaml::resolve_duplicate_key::reject_profile1_duplicate_keys_spec(input, limits)
            == Ok(output),
    ensures
        crucible_yaml::resolve_duplicate_key::duplicate_free_structural_key_source_well_formed_spec(
            input,
            limits,
            output,
        ),
{
    crucible_yaml::resolve_duplicate_key::lemma_duplicate_key_success_is_well_formed(
        input,
        limits,
        output,
    );
}

proof fn public_duplicate_free_semantics_authenticate_the_total_result(
    input: crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeySourceView,
    limits: crucible_yaml::resolve_duplicate_key::DuplicateKeyLimitsView,
    output: crucible_yaml::resolve_duplicate_key::DuplicateFreeStructuralKeySourceView,
)
    requires
        crucible_yaml::resolve_duplicate_key::duplicate_free_structural_key_source_well_formed_spec(
            input,
            limits,
            output,
        ),
    ensures
        crucible_yaml::resolve_duplicate_key::reject_profile1_duplicate_keys_spec(input, limits)
            == Ok(output),
{
    crucible_yaml::resolve_duplicate_key::lemma_duplicate_key_well_formed_authenticates_exact_result(
    input, limits, output);
}

proof fn equal_key_bytes_with_different_provenance_cannot_launder_pairwise_distinctness() {
    let records = seq![
        crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView {
            node_index: 0,
            byte_start: 4,
            bytes: seq![
                crucible_yaml::resolve_canonical_scalar_key::CanonicalKeyByteView {
                    value: 0x51,
                    source_byte_offset: 4,
                },
            ],
        },
        crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView {
            node_index: 1,
            byte_start: 40,
            bytes: seq![
                crucible_yaml::resolve_canonical_scalar_key::CanonicalKeyByteView {
                    value: 0x51,
                    source_byte_offset: 40,
                },
            ],
        },
    ];
    let edges = seq![
        crucible_yaml::resolve_topology::SemanticMappingEdgeView {
            cst_entry_index: 0,
            key_node_index: 0,
            value_node_index: 0,
            token_start: 0,
            token_end: 1,
        },
        crucible_yaml::resolve_topology::SemanticMappingEdgeView {
            cst_entry_index: 1,
            key_node_index: 1,
            value_node_index: 1,
            token_start: 1,
            token_end: 2,
        },
    ];

    reveal(crucible_yaml::resolve_duplicate_key::mapping_keys_pairwise_distinct_spec);
    reveal(crucible_yaml::resolve_duplicate_key::mapping_key_indices_equal_spec);
    crucible_yaml::resolve_canonical_structural_key::lemma_equal_single_canonical_byte_values_compare_equal(
    0x51, 4, 40);
    assert(crucible_yaml::resolve_duplicate_key::mapping_key_indices_equal_spec(
        0,
        1,
        edges,
        records,
        0,
    ) == Ok(true));
    crucible_yaml::resolve_duplicate_key::lemma_equal_mapping_key_pair_is_not_distinct(
        0,
        2,
        0,
        1,
        edges,
        records,
        0,
    );
}

} // verus!
