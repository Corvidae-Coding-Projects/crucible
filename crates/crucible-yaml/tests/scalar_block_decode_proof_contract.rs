#![allow(unused_imports)]

use crucible_yaml::{BlockScalarContentOrigin, BlockScalarStyle, DecodedContentOrigin};
use vstd::prelude::*;

verus! {

#[test]
fn pure_block_content_copy_has_exact_value_and_provenance() {
    proof {
        reveal(crucible_yaml::scalar_decode::decode_block_content_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        reveal(crucible_yaml::scalar_decode::decoded_block_style_spec);
        reveal(crucible_yaml::scalar_decode::decoded_block_content_spec);
        reveal(crucible_yaml::scalar_decode::decoded_block_content_prefix_spec);
        reveal(crucible_yaml::scalar_decode::decoded_block_content_item_spec);
        reveal(crucible_yaml::scalar_decode::decoded_block_origin_spec);
        let source = seq![
            crucible_yaml::block::BlockScalarContentScalarView {
                code_point: 0x61,
                source_atom_index: 3,
                byte_start: 7,
                byte_end: 8,
                origin: BlockScalarContentOrigin::Direct,
            },
        ];
        let expected = crucible_yaml::scalar_decode::DecodedContentScalarView {
            code_point: 0x61,
            source_atom_start: 3,
            source_atom_end: 4,
            byte_start: 7,
            byte_end: 8,
            origin: DecodedContentOrigin::Direct,
        };
        assert(crucible_yaml::scalar_decode::decoded_block_content_item_spec(source[0])
            == expected);
        assert(crucible_yaml::scalar_decode::decoded_block_content_prefix_spec(source, 1) =~= seq![
            expected,
        ]);
        assert(crucible_yaml::scalar_decode::decoded_block_content_spec(source) == seq![expected]);
        assert(crucible_yaml::scalar_decode::decode_block_content_spec(
            BlockScalarStyle::Literal,
            source,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 1 },
        ) == Ok(
            crucible_yaml::scalar_decode::DecodedScalarContentView {
                style: crucible_yaml::scalar_decode::DecodedScalarStyle::LiteralBlock,
                content: seq![expected],
            },
        ));
    }
}

#[test]
fn pure_block_content_limit_reports_the_first_excluded_provenance() {
    proof {
        reveal(crucible_yaml::scalar_decode::decode_block_content_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        let source = seq![
            crucible_yaml::block::BlockScalarContentScalarView {
                code_point: 0x20,
                source_atom_index: 9,
                byte_start: 17,
                byte_end: 18,
                origin: BlockScalarContentOrigin::FoldedLineBreak,
            },
        ];
        assert(crucible_yaml::scalar_decode::decode_block_content_spec(
            BlockScalarStyle::Folded,
            source,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 0 },
        ) == Err(
            crucible_yaml::scalar_decode::ScalarDecodeErrorView {
                kind: crucible_yaml::scalar_decode::ScalarDecodeErrorKind::ContentLimitExceeded,
                byte_offset: 17,
            },
        ));
    }
}

} // verus!
