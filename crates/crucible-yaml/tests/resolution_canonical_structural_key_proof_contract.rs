#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_structural_key_composition_has_exact_public_semantics(
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
    scalar_key_limits: crucible_yaml::resolve_canonical_scalar_key::CanonicalScalarKeyLimitsView,
    structural_limits:
        crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeyLimitsView,
    source: crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeySourceView,
)
    requires
        crucible_yaml::resolve_canonical_structural_key::compose_profile1_canonical_structural_keys_spec(

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
            scalar_key_limits,
            structural_limits,
        ) == Ok(source),
    ensures
        crucible_yaml::resolve_canonical_structural_key::canonical_structural_key_source_well_formed_spec(

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
            scalar_key_limits,
            structural_limits,
            source,
        ),
{
    crucible_yaml::resolve_canonical_structural_key::lemma_canonical_structural_key_success_is_well_formed(

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
        scalar_key_limits,
        structural_limits,
        source,
    );
}

proof fn public_structural_key_semantics_authenticate_the_total_result(
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
    scalar_key_limits: crucible_yaml::resolve_canonical_scalar_key::CanonicalScalarKeyLimitsView,
    structural_limits:
        crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeyLimitsView,
    source: crucible_yaml::resolve_canonical_structural_key::CanonicalStructuralKeySourceView,
)
    requires
        crucible_yaml::resolve_canonical_structural_key::canonical_structural_key_source_well_formed_spec(

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
            scalar_key_limits,
            structural_limits,
            source,
        ),
    ensures
        crucible_yaml::resolve_canonical_structural_key::compose_profile1_canonical_structural_keys_spec(

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
            scalar_key_limits,
            structural_limits,
        ) == Ok(source),
{
    crucible_yaml::resolve_canonical_structural_key::lemma_canonical_structural_key_well_formed_authenticates_exact_result(

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
        scalar_key_limits,
        structural_limits,
        source,
    );
}

} // verus!
