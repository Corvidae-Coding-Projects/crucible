#[expect(
    unused_imports,
    reason = "these variants are referenced only inside Verus proof code"
)]
use crucible_yaml::{CstNodeKind, CstNodeStyle, ResolvedScalarTag};
use vstd::prelude::*;

verus! {

#[test]
fn pure_empty_node_resolves_to_core_null_without_fabricated_content() {
    proof {
        let decoded = crucible_yaml::resolve_scalar_node::DecodedCstScalarView {
            node_index: 0,
            token_index: None,
            style: CstNodeStyle::Empty,
            decoded: None,
        };
        let node = crucible_yaml::cst::CstNodeView {
            kind: CstNodeKind::Empty,
            style: CstNodeStyle::Empty,
            token_start: 0,
            token_end: 0,
            byte_start: 0,
            byte_end: 0,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: Some(0),
            empty_anchor_byte: Some(0),
        };
        reveal(crucible_yaml::resolve_scalar_value::resolve_decoded_scalar_value_spec);
        assert(crucible_yaml::resolve_scalar_value::resolve_decoded_scalar_value_spec(
            decoded,
            None,
            node,
            0,
            crucible_yaml::resolve_scalar_value::ScalarValueLimitsView {
                max_content_code_points: 0,
                max_tag_code_points: 0,
                max_integer_limbs: 0,
                max_float_coefficient_digits: 0,
                max_float_exponent_digits: 0,
            },
        ) == Ok(
            crucible_yaml::resolve_scalar_value::ResolvedScalarView {
                node_index: 0,
                tag: ResolvedScalarTag::CoreNull,
                explicit_tag: None,
                presentation: decoded,
                value: crucible_yaml::resolve_scalar_value::ResolvedScalarValueView::Null,
            },
        ));

        let forged = crucible_yaml::resolve_scalar_node::DecodedCstScalarView {
            node_index: 1,
            token_index: None,
            style: CstNodeStyle::Empty,
            decoded: None,
        };
        assert(crucible_yaml::resolve_scalar_value::resolve_decoded_scalar_value_spec(
            forged,
            None,
            node,
            0,
            crucible_yaml::resolve_scalar_value::ScalarValueLimitsView {
                max_content_code_points: 0,
                max_tag_code_points: 0,
                max_integer_limbs: 0,
                max_float_coefficient_digits: 0,
                max_float_exponent_digits: 0,
            },
        ) == Err(
            crucible_yaml::resolve_scalar_value::ScalarValueErrorView {
                kind: crucible_yaml::ScalarValueErrorKind::InvalidScalarPresentation,
                byte_offset: 0,
            },
        ));
    }
}

} // verus!
