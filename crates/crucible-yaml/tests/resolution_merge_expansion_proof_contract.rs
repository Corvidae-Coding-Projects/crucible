use vstd::prelude::*;

verus! {

proof fn successful_merge_expansion_has_exact_public_semantics(
    input: crucible_yaml::resolve_duplicate_key::DuplicateFreeStructuralKeySourceView,
    limits: crucible_yaml::resolve_merge::MergeExpansionLimitsView,
    output: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
)
    requires
        crucible_yaml::resolve_merge::expand_profile1_merge_keys_spec(input, limits) == Ok(output),
    ensures
        crucible_yaml::resolve_merge::expanded_semantic_graph_source_well_formed_spec(
            input,
            limits,
            output,
        ),
{
    crucible_yaml::resolve_merge::lemma_merge_expansion_success_is_well_formed(
        input,
        limits,
        output,
    );
}

proof fn public_merge_semantics_authenticate_the_total_result(
    input: crucible_yaml::resolve_duplicate_key::DuplicateFreeStructuralKeySourceView,
    limits: crucible_yaml::resolve_merge::MergeExpansionLimitsView,
    output: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
)
    requires
        crucible_yaml::resolve_merge::expanded_semantic_graph_source_well_formed_spec(
            input,
            limits,
            output,
        ),
    ensures
        crucible_yaml::resolve_merge::expand_profile1_merge_keys_spec(input, limits) == Ok(output),
{
    crucible_yaml::resolve_merge::lemma_merge_expansion_well_formed_authenticates_exact_result(
        input,
        limits,
        output,
    );
}

proof fn authenticated_merge_expansion_cannot_substitute_its_owned_input(
    input: crucible_yaml::resolve_duplicate_key::DuplicateFreeStructuralKeySourceView,
    limits: crucible_yaml::resolve_merge::MergeExpansionLimitsView,
    output: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
)
    requires
        crucible_yaml::resolve_merge::expanded_semantic_graph_source_well_formed_spec(
            input,
            limits,
            output,
        ),
    ensures
        crucible_yaml::resolve_merge::expanded_semantic_graph_source_preserves_input_identity_spec(
            input,
            output,
        ),
{
    crucible_yaml::resolve_merge::lemma_authenticated_merge_expansion_preserves_input_identity(
        input,
        limits,
        output,
    );
}

proof fn distinct_outputs_cannot_both_authenticate_for_one_input(
    input: crucible_yaml::resolve_duplicate_key::DuplicateFreeStructuralKeySourceView,
    limits: crucible_yaml::resolve_merge::MergeExpansionLimitsView,
    first: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
    second: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
)
    requires
        crucible_yaml::resolve_merge::expanded_semantic_graph_source_well_formed_spec(
            input,
            limits,
            first,
        ),
        crucible_yaml::resolve_merge::expanded_semantic_graph_source_well_formed_spec(
            input,
            limits,
            second,
        ),
    ensures
        first == second,
{
    crucible_yaml::resolve_merge::lemma_authenticated_merge_expansion_is_unique(
        input,
        limits,
        first,
        second,
    );
}

} // verus!
