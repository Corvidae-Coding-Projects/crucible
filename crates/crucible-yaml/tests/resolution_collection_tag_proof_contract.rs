#![allow(unused_imports)]

use crucible_yaml::{CstNodeKind, CstNodeStyle, ResolvedCollectionTag};
use vstd::prelude::*;

verus! {

#[test]
fn pure_implicit_sequence_tag_is_exact_and_node_index_bound() {
    proof {
        let node = crucible_yaml::cst::CstNodeView {
            kind: CstNodeKind::Sequence,
            style: CstNodeStyle::Flow,
            token_start: 0,
            token_end: 2,
            byte_start: 0,
            byte_end: 2,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: Some(0),
            collection_end_token: Some(1),
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        reveal(crucible_yaml::resolve_collection_tag::resolve_collection_tag_spec);
        assert(crucible_yaml::resolve_collection_tag::resolve_collection_tag_spec(None, node, 7)
            == Ok(
            crucible_yaml::resolve_collection_tag::ResolvedCollectionView {
                node_index: 7,
                kind: CstNodeKind::Sequence,
                tag: ResolvedCollectionTag::CoreSequence,
                explicit_tag: None,
            },
        ));
    }
}

} // verus!
