#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_scalar_key_composition_has_exact_public_semantics(
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
    key_limits: crucible_yaml::resolve_canonical_scalar_key::CanonicalScalarKeyLimitsView,
    source: crucible_yaml::resolve_canonical_scalar_key::CanonicalScalarKeySourceView,
)
    requires
        crucible_yaml::resolve_canonical_scalar_key::compose_profile1_canonical_scalar_keys_spec(
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
            key_limits,
        ) == Ok(source),
    ensures
        crucible_yaml::resolve_canonical_scalar_key::canonical_scalar_key_source_well_formed_spec(
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
            key_limits,
            source,
        ),
{
    crucible_yaml::resolve_canonical_scalar_key::lemma_canonical_scalar_key_success_is_well_formed(
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
        key_limits,
        source,
    );
}

proof fn public_scalar_key_semantics_authenticate_the_total_result(
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
    key_limits: crucible_yaml::resolve_canonical_scalar_key::CanonicalScalarKeyLimitsView,
    source: crucible_yaml::resolve_canonical_scalar_key::CanonicalScalarKeySourceView,
)
    requires
        crucible_yaml::resolve_canonical_scalar_key::canonical_scalar_key_source_well_formed_spec(
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
            key_limits,
            source,
        ),
    ensures
        crucible_yaml::resolve_canonical_scalar_key::compose_profile1_canonical_scalar_keys_spec(
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
            key_limits,
        ) == Ok(source),
{
    crucible_yaml::resolve_canonical_scalar_key::lemma_canonical_scalar_key_well_formed_authenticates_exact_result(

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
        key_limits,
        source,
    );
}

} // verus!
