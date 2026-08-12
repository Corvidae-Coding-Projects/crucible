#![allow(unused_imports)]

use crucible_yaml::{DecodedContentOrigin, QuotedScalarStyle};
use vstd::prelude::*;

verus! {

#[test]
fn pure_doubled_quote_decodes_to_one_exact_provenance_record() {
    proof {
        let quote = crucible_yaml::quoted::QuotedScalarView {
            style: QuotedScalarStyle::Single,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 4,
            byte_start: 0,
            byte_end: 4,
        };
        let atoms = seq![
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::SingleQuotedScalar,
                ),
                code_point: 0x27,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 0,
                        line: 0,
                        column: 0,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 1,
                        line: 0,
                        column: 1,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::SingleQuotedScalar,
                ),
                code_point: 0x27,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 1,
                        line: 0,
                        column: 1,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 2,
                        line: 0,
                        column: 2,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::SingleQuotedScalar,
                ),
                code_point: 0x27,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 2,
                        line: 0,
                        column: 2,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 3,
                        line: 0,
                        column: 3,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::SingleQuotedScalar,
                ),
                code_point: 0x27,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 3,
                        line: 0,
                        column: 3,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 4,
                        line: 0,
                        column: 4,
                    },
                },
            },
        ];
        reveal(crucible_yaml::scalar_decode::decode_single_quoted_content_spec);
        reveal(crucible_yaml::scalar_decode::quoted_scalar_range_matches_atoms_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::decode_single_quoted_loop_spec, 3);
        reveal(crucible_yaml::scalar_decode::single_quoted_step_spec);
        reveal(crucible_yaml::scalar_decode::scalar_atom_white_spec);
        reveal(crucible_yaml::scalar_decode::doubled_quote_content_spec);
        reveal(crucible_yaml::scalar_decode::prepend_decoded_content_result_spec);
        assert(crucible_yaml::scalar_decode::decode_single_quoted_content_spec(
            atoms,
            quote,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 1 },
        ) == Ok(
            crucible_yaml::scalar_decode::DecodedScalarContentView {
                style: crucible_yaml::scalar_decode::DecodedScalarStyle::SingleQuoted,
                content: seq![
                    crucible_yaml::scalar_decode::DecodedContentScalarView {
                        code_point: 0x27,
                        source_atom_start: 1,
                        source_atom_end: 3,
                        byte_start: 1,
                        byte_end: 3,
                        origin: DecodedContentOrigin::SingleQuoteDoubled,
                    },
                ],
            },
        ));
    }
}

} // verus!
