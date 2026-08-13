#[expect(
    unused_imports,
    reason = "these variants are referenced only inside Verus proof code"
)]
use crucible_yaml::{CstNodeKind, CstNodeStyle, CstScalarDecodeErrorKind};
use vstd::prelude::*;

verus! {

#[test]
fn pure_empty_cst_node_decodes_without_fabricating_content_or_a_token() {
    proof {
        let node = crucible_yaml::cst::CstNodeView {
            kind: CstNodeKind::Empty,
            style: CstNodeStyle::Empty,
            token_start: 3,
            token_end: 3,
            byte_start: 8,
            byte_end: 8,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: Some(3),
            empty_anchor_byte: Some(8),
        };
        reveal(crucible_yaml::resolve_scalar_node::decode_cst_node_scalar_spec);
        assert(crucible_yaml::resolve_scalar_node::decode_cst_node_scalar_spec(
            arbitrary(),
            arbitrary(),
            arbitrary(),
            arbitrary(),
            Seq::empty(),
            node,
            0,
            crucible_yaml::resolve_scalar_node::CstScalarDecodeLimitsView {
                max_content_code_points: 0,
            },
        ) == Ok(
            Some(
                crucible_yaml::resolve_scalar_node::DecodedCstScalarView {
                    node_index: 0,
                    token_index: None,
                    style: CstNodeStyle::Empty,
                    decoded: None,
                },
            ),
        ));
    }
}

} // verus!
