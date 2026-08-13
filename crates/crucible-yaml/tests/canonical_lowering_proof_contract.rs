#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_canonical_lowering_has_exact_public_semantics(
    input: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: crucible_yaml::lower::CanonicalLoweringLimitsView,
    output: crucible_yaml::lower::CanonicalYamlGraphSourceView,
)
    requires
        crucible_yaml::lower::lower_profile1_canonical_graph_spec(input, limits) == Ok(output),
    ensures
        crucible_yaml::lower::canonical_yaml_graph_source_well_formed_spec(input, limits, output),
{
    crucible_yaml::lower::lemma_canonical_lowering_success_is_well_formed(input, limits, output);
}

proof fn authenticated_canonical_graph_owns_the_exact_merge_expansion(
    input: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: crucible_yaml::lower::CanonicalLoweringLimitsView,
    output: crucible_yaml::lower::CanonicalYamlGraphSourceView,
)
    requires
        crucible_yaml::lower::canonical_yaml_graph_source_well_formed_spec(input, limits, output),
    ensures
        crucible_yaml::lower::canonical_yaml_graph_source_preserves_input_identity_spec(
            input,
            output,
        ),
{
    crucible_yaml::lower::lemma_authenticated_canonical_graph_preserves_input_identity(
        input,
        limits,
        output,
    );
}

proof fn canonical_lowering_rejects_output_substitution(
    input: crucible_yaml::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: crucible_yaml::lower::CanonicalLoweringLimitsView,
    first: crucible_yaml::lower::CanonicalYamlGraphSourceView,
    second: crucible_yaml::lower::CanonicalYamlGraphSourceView,
)
    requires
        crucible_yaml::lower::canonical_yaml_graph_source_well_formed_spec(input, limits, first),
        crucible_yaml::lower::canonical_yaml_graph_source_well_formed_spec(input, limits, second),
    ensures
        first == second,
{
    crucible_yaml::lower::lemma_authenticated_canonical_lowering_is_unique(
        input,
        limits,
        first,
        second,
    );
}

} // verus!
