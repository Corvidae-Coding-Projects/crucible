#[expect(
    unused_imports,
    reason = "these variants are referenced only inside Verus proof code"
)]
use crucible_yaml::{DecodedContentOrigin, QuotedScalarStyle};
use vstd::prelude::*;

verus! {

#[test]
fn pure_unicode_escape_decodes_to_one_exact_provenance_record() {
    proof {
        let quote = crucible_yaml::quoted::QuotedScalarView {
            style: QuotedScalarStyle::Double,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 8,
            byte_start: 0,
            byte_end: 8,
        };
        let atoms = Seq::new(
            8,
            |index: int|
                crucible_yaml::atom::LexicalAtomView {
                    kind: if index == 0 || index == 7 {
                        crucible_yaml::LexicalAtomKind::Indicator(
                            crucible_yaml::YamlIndicator::DoubleQuotedScalar,
                        )
                    } else {
                        crucible_yaml::LexicalAtomKind::Content
                    },
                    code_point: seq![0x22u32, 0x5c, 0x75, 0x30, 0x33, 0x42, 0x32, 0x22][index],
                    span: crucible_yaml::utf8::SourceSpanView {
                        start: crucible_yaml::utf8::SourcePositionView {
                            byte_offset: index as u64,
                            line: 0,
                            column: index as u64,
                        },
                        end: crucible_yaml::utf8::SourcePositionView {
                            byte_offset: (index + 1) as u64,
                            line: 0,
                            column: (index + 1) as u64,
                        },
                    },
                },
        );
        reveal(crucible_yaml::scalar_decode::decode_double_quoted_content_spec);
        reveal(crucible_yaml::scalar_decode::double_quoted_scalar_range_matches_atoms_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::decode_double_quoted_loop_spec, 3);
        reveal(crucible_yaml::scalar_decode::double_quoted_step_spec);
        reveal(crucible_yaml::scalar_decode::scalar_atom_white_spec);
        reveal(crucible_yaml::scalar_decode::simple_double_escape_value_spec);
        reveal(crucible_yaml::scalar_decode::double_escape_width_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::double_hex_value_tail_spec, 6);
        reveal(crucible_yaml::scalar_decode::scalar_hex_digit_value_spec);
        reveal(crucible_yaml::scalar_decode::decoded_unicode_scalar_spec);
        reveal(crucible_yaml::scalar_decode::double_escape_content_spec);
        reveal(crucible_yaml::scalar_decode::prepend_decoded_content_result_spec);
        assert(crucible_yaml::scalar_decode::decode_double_quoted_content_spec(
            atoms,
            quote,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 1 },
        ) == Ok(
            crucible_yaml::scalar_decode::DecodedScalarContentView {
                style: crucible_yaml::scalar_decode::DecodedScalarStyle::DoubleQuoted,
                content: seq![
                    crucible_yaml::scalar_decode::DecodedContentScalarView {
                        code_point: 0x03b2,
                        source_atom_start: 1,
                        source_atom_end: 7,
                        byte_start: 1,
                        byte_end: 7,
                        origin: DecodedContentOrigin::DoubleQuotedEscape,
                    },
                ],
            },
        ));
    }
}

#[test]
fn pure_escaped_break_retains_only_the_following_empty_line_break() {
    proof {
        let quote = crucible_yaml::quoted::QuotedScalarView {
            style: QuotedScalarStyle::Double,
            start_line_number: 0,
            end_line_number: 2,
            start_atom_index: 0,
            end_atom_index: 6,
            byte_start: 0,
            byte_end: 6,
        };
        let atoms = Seq::new(
            6,
            |index: int|
                crucible_yaml::atom::LexicalAtomView {
                    kind: if index == 0 || index == 5 {
                        crucible_yaml::LexicalAtomKind::Indicator(
                            crucible_yaml::YamlIndicator::DoubleQuotedScalar,
                        )
                    } else if index == 2 || index == 3 {
                        crucible_yaml::LexicalAtomKind::LineFeed
                    } else {
                        crucible_yaml::LexicalAtomKind::Content
                    },
                    code_point: seq![0x22u32, 0x5c, 0x0a, 0x0a, 0x78, 0x22][index],
                    span: crucible_yaml::utf8::SourceSpanView {
                        start: crucible_yaml::utf8::SourcePositionView {
                            byte_offset: index as u64,
                            line: if index <= 2 {
                                0
                            } else if index == 3 {
                                1
                            } else {
                                2
                            },
                            column: 0,
                        },
                        end: crucible_yaml::utf8::SourcePositionView {
                            byte_offset: (index + 1) as u64,
                            line: if index < 2 {
                                0
                            } else if index == 2 {
                                1
                            } else {
                                2
                            },
                            column: 0,
                        },
                    },
                },
        );
        reveal(crucible_yaml::scalar_decode::decode_double_quoted_content_spec);
        reveal(crucible_yaml::scalar_decode::double_quoted_scalar_range_matches_atoms_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::decode_double_quoted_loop_spec, 4);
        reveal(crucible_yaml::scalar_decode::double_quoted_step_spec);
        reveal(crucible_yaml::scalar_decode::scalar_atom_white_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::single_quoted_break_group_spec, 4);
        reveal_with_fuel(crucible_yaml::scalar_decode::skip_scalar_white_spec, 4);
        reveal(crucible_yaml::scalar_decode::escaped_break_content_spec);
        reveal(crucible_yaml::scalar_decode::direct_atom_content_spec);
        reveal(crucible_yaml::scalar_decode::prepend_decoded_content_result_spec);
        let expected_break = crucible_yaml::scalar_decode::DecodedContentScalarView {
            code_point: 0x0a,
            source_atom_start: 3,
            source_atom_end: 4,
            byte_start: 3,
            byte_end: 4,
            origin: DecodedContentOrigin::EscapedLineBreak,
        };
        let expected_direct = crucible_yaml::scalar_decode::DecodedContentScalarView {
            code_point: 0x78,
            source_atom_start: 4,
            source_atom_end: 5,
            byte_start: 4,
            byte_end: 5,
            origin: DecodedContentOrigin::Direct,
        };
        assert(crucible_yaml::scalar_decode::single_quoted_break_group_spec(atoms, 2, 5, 3) == (
            seq![2int, 3int],
            4,
        ));
        assert(crucible_yaml::scalar_decode::escaped_break_content_spec(atoms, seq![2int, 3int])
            == seq![expected_break]);
        assert(crucible_yaml::scalar_decode::double_quoted_step_spec(atoms, 1, 5, 2) == Ok(
            (seq![expected_break], 4),
        ));
        assert(crucible_yaml::scalar_decode::double_quoted_step_spec(atoms, 4, 5, 1) == Ok(
            (seq![expected_direct], 5),
        ));
        assert(crucible_yaml::scalar_decode::decode_double_quoted_loop_spec(atoms, 5, 5, 0, 3)
            == Ok(Seq::empty()));
        assert(crucible_yaml::scalar_decode::decode_double_quoted_loop_spec(atoms, 4, 5, 1, 4)
            == Ok(seq![expected_direct]));
        assert(seq![expected_break] + seq![expected_direct] =~= seq![
            expected_break,
            expected_direct,
        ]);
        assert(crucible_yaml::scalar_decode::decode_double_quoted_loop_spec(atoms, 1, 5, 2, 5)
            == Ok(seq![expected_break, expected_direct]));
        assert(crucible_yaml::scalar_decode::decode_double_quoted_content_spec(
            atoms,
            quote,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 2 },
        ) == Ok(
            crucible_yaml::scalar_decode::DecodedScalarContentView {
                style: crucible_yaml::scalar_decode::DecodedScalarStyle::DoubleQuoted,
                content: seq![expected_break, expected_direct],
            },
        ));
    }
}

} // verus!
