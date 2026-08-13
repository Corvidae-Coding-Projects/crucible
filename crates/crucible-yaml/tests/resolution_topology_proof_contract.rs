#[expect(
    unused_imports,
    reason = "the node-kind variants are referenced only inside Verus proof code"
)]
use crucible_yaml::CstNodeKind;
use vstd::prelude::*;

verus! {

proof fn external_consumers_can_extract_authenticated_cst_and_exact_projection(
    completed: crucible_yaml::token::CompletedTokenSourceView,
    cst: crucible_yaml::cst::CstSourceView,
    topology: crucible_yaml::resolve_topology::SemanticTopologySourceView,
)
    requires
        crucible_yaml::resolve_topology::semantic_topology_source_well_formed_spec(
            completed,
            cst,
            topology,
        ),
    ensures
        crucible_yaml::cst::cst_public_semantics_spec(completed, cst),
        topology == crucible_yaml::resolve_topology::semantic_topology_exact_source_spec(
            completed,
            cst,
        ),
{
    crucible_yaml::resolve_topology::lemma_semantic_topology_well_formed_authenticates_cst(
        completed,
        cst,
        topology,
    );
}

#[test]
fn pure_topology_node_projection_retains_index_kind_and_edge_interval() {
    proof {
        let node = crucible_yaml::cst::CstNodeView {
            kind: CstNodeKind::Sequence,
            style: crucible_yaml::CstNodeStyle::Flow,
            token_start: 4,
            token_end: 9,
            byte_start: 7,
            byte_end: 15,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: Some(4),
            collection_end_token: Some(8),
            entry_start: 2,
            entry_end: 5,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        reveal(crucible_yaml::resolve_topology::semantic_topology_node_spec);
        assert(crucible_yaml::resolve_topology::semantic_topology_node_spec(node, 11)
            == crucible_yaml::resolve_topology::SemanticTopologyNodeView {
            cst_node_index: 11,
            kind: CstNodeKind::Sequence,
            byte_start: 7,
            byte_end: 15,
            edge_start: 2,
            edge_end: 5,
        });
    }
}

} // verus!
