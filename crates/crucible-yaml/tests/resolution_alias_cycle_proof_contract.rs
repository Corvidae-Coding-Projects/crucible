#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_cycle_resolution_has_exact_public_acyclic_semantics(
    atomized: crucible_yaml::atom::AtomizedSourceView,
    quoted: crucible_yaml::quoted::QuotedScalarSourceView,
    plain: crucible_yaml::plain::PlainScalarSourceView,
    block: crucible_yaml::block::BlockScalarSourceView,
    completed: crucible_yaml::token::CompletedTokenSourceView,
    cst: crucible_yaml::cst::CstSourceView,
    topology_limits: crucible_yaml::resolve_topology::SemanticTopologyLimitsView,
    scalar_limits: crucible_yaml::resolve_scalar_table::SemanticScalarTableLimitsView,
    anchor_limits: crucible_yaml::resolve_anchor::AnchorAliasLimitsView,
    node_limits: crucible_yaml::resolve_node_table::SemanticNodeTableLimitsView,
    cycle_limits: crucible_yaml::resolve_alias_cycle::AliasCycleLimitsView,
    source: crucible_yaml::resolve_alias_cycle::AcyclicSemanticGraphSourceView,
)
    requires
        crucible_yaml::cst::cst_public_semantics_spec(completed, cst),
        crucible_yaml::resolve_alias_cycle::resolve_profile1_alias_cycles_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
        ) == Ok(source),
    ensures
        crucible_yaml::resolve_alias_cycle::acyclic_semantic_graph_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
            source,
        ),
        crucible_yaml::resolve_alias_cycle::semantic_graph_edges_strictly_decrease_spec(
            source.node_table,
        ),
{
    crucible_yaml::resolve_alias_cycle::lemma_alias_cycle_success_is_well_formed_and_acyclic(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
        cycle_limits,
        source,
    );
}

proof fn public_acyclic_semantics_authenticate_the_exact_total_result(
    atomized: crucible_yaml::atom::AtomizedSourceView,
    quoted: crucible_yaml::quoted::QuotedScalarSourceView,
    plain: crucible_yaml::plain::PlainScalarSourceView,
    block: crucible_yaml::block::BlockScalarSourceView,
    completed: crucible_yaml::token::CompletedTokenSourceView,
    cst: crucible_yaml::cst::CstSourceView,
    topology_limits: crucible_yaml::resolve_topology::SemanticTopologyLimitsView,
    scalar_limits: crucible_yaml::resolve_scalar_table::SemanticScalarTableLimitsView,
    anchor_limits: crucible_yaml::resolve_anchor::AnchorAliasLimitsView,
    node_limits: crucible_yaml::resolve_node_table::SemanticNodeTableLimitsView,
    cycle_limits: crucible_yaml::resolve_alias_cycle::AliasCycleLimitsView,
    source: crucible_yaml::resolve_alias_cycle::AcyclicSemanticGraphSourceView,
)
    requires
        crucible_yaml::resolve_alias_cycle::acyclic_semantic_graph_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
            source,
        ),
    ensures
        crucible_yaml::resolve_alias_cycle::resolve_profile1_alias_cycles_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
        ) == Ok(source),
{
    crucible_yaml::resolve_alias_cycle::lemma_acyclic_semantic_graph_well_formed_authenticates_exact_result(

        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
        cycle_limits,
        source,
    );
}

#[test]
fn forged_nondecreasing_alias_redirect_is_not_acyclic() {
    proof {
        let forged = crucible_yaml::resolve_node_table::SemanticAliasRedirectView {
            binding_index: 0,
            document_index: 0,
            alias_node_index: 3,
            alias_token_index: 4,
            target_anchor_index: 0,
            target_node_index: 7,
            name_start_atom_index: 4,
            name_end_atom_index: 5,
            name_byte_start: 4,
            name_byte_end: 5,
        };
        let redirects = seq![forged];
        reveal(crucible_yaml::resolve_alias_cycle::alias_redirect_targets_decrease_spec);
        assert(redirects.len() == 1);
        assert(redirects[0] == forged);
        assert(forged.target_node_index >= forged.alias_node_index);
        assert(!(forall|index: int|
            0 <= index < redirects.len() ==> #[trigger] redirects[index].target_node_index
                < redirects[index].alias_node_index));
        assert(!crucible_yaml::resolve_alias_cycle::alias_redirect_targets_decrease_spec(
            redirects,
        ));
    }
}

} // verus!
