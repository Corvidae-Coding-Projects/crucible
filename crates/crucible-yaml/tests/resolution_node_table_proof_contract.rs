#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_owned_node_table_composition_has_exact_public_semantics(
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
    source: crucible_yaml::resolve_node_table::SemanticNodeTableSourceView,
)
    requires
        crucible_yaml::cst::cst_public_semantics_spec(completed, cst),
        crucible_yaml::resolve_node_table::compose_profile1_semantic_node_table_spec(
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
        ) == Ok(source),
    ensures
        crucible_yaml::resolve_node_table::semantic_node_table_source_well_formed_spec(
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
            source,
        ),
{
    crucible_yaml::resolve_node_table::lemma_semantic_node_table_success_is_well_formed(
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
        source,
    );
}

proof fn public_semantics_authenticate_the_exact_owned_aggregate(
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
    source: crucible_yaml::resolve_node_table::SemanticNodeTableSourceView,
)
    requires
        crucible_yaml::resolve_node_table::semantic_node_table_source_well_formed_spec(
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
            source,
        ),
    ensures
        crucible_yaml::resolve_node_table::compose_profile1_semantic_node_table_spec(
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
        ) == Ok(source),
        crucible_yaml::cst::cst_public_semantics_spec(completed, cst),
{
    crucible_yaml::resolve_node_table::lemma_semantic_node_table_well_formed_authenticates_exact_composition(

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
        source,
    );
}

} // verus!
