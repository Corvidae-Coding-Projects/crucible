#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_composition_yields_exact_public_table_semantics(
    atomized: crucible_yaml::atom::AtomizedSourceView,
    quoted: crucible_yaml::quoted::QuotedScalarSourceView,
    plain: crucible_yaml::plain::PlainScalarSourceView,
    block: crucible_yaml::block::BlockScalarSourceView,
    completed: crucible_yaml::token::CompletedTokenSourceView,
    cst: crucible_yaml::cst::CstSourceView,
    limits: crucible_yaml::resolve_scalar_table::SemanticScalarTableLimitsView,
    source: crucible_yaml::resolve_scalar_table::SemanticScalarTableSourceView,
)
    requires
        crucible_yaml::cst::cst_public_semantics_spec(completed, cst),
        crucible_yaml::resolve_scalar_table::compose_profile1_semantic_scalar_table_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
        ) == Ok(source),
    ensures
        crucible_yaml::resolve_scalar_table::semantic_scalar_table_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
            source,
        ),
{
    crucible_yaml::resolve_scalar_table::lemma_semantic_scalar_table_success_is_well_formed(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        limits,
        source,
    );
}

proof fn scalar_table_well_formedness_authenticates_values_order_and_aggregate(
    atomized: crucible_yaml::atom::AtomizedSourceView,
    quoted: crucible_yaml::quoted::QuotedScalarSourceView,
    plain: crucible_yaml::plain::PlainScalarSourceView,
    block: crucible_yaml::block::BlockScalarSourceView,
    completed: crucible_yaml::token::CompletedTokenSourceView,
    cst: crucible_yaml::cst::CstSourceView,
    limits: crucible_yaml::resolve_scalar_table::SemanticScalarTableLimitsView,
    source: crucible_yaml::resolve_scalar_table::SemanticScalarTableSourceView,
)
    requires
        crucible_yaml::resolve_scalar_table::semantic_scalar_table_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
            source,
        ),
    ensures
        crucible_yaml::resolve_scalar_table::compose_profile1_semantic_scalar_table_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
        ) == Ok(source),
        crucible_yaml::resolve_scalar_table::semantic_scalar_table_scalar_node_indices_spec(
            source.scalars,
        ) == crucible_yaml::resolve_scalar_table::semantic_scalar_table_expected_node_indices_spec(
            cst.nodes,
        ),
        source.total_content_code_points
            == crucible_yaml::resolve_scalar_table::semantic_scalar_table_total_content_spec(
            source.scalars,
        ),
{
    crucible_yaml::resolve_scalar_table::lemma_semantic_scalar_table_well_formed_authenticates_exact_composition(
    atomized, quoted, plain, block, completed, cst, limits, source);
    crucible_yaml::resolve_scalar_table::lemma_semantic_scalar_table_well_formed_is_exact(
        cst,
        source,
    );
}

} // verus!
