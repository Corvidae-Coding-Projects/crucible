#![allow(unused_imports)]

use crucible_yaml::DecodedContentOrigin;
use vstd::prelude::*;

verus! {

#[test]
fn pure_plain_flow_fold_has_exact_records_and_ranges() {
    proof {
        let atoms = seq![
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Content,
                code_point: 0x61,
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
                kind: crucible_yaml::LexicalAtomKind::LineFeed,
                code_point: 0x0a,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 1,
                        line: 0,
                        column: 1,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 2,
                        line: 1,
                        column: 0,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Space,
                code_point: 0x20,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 2,
                        line: 1,
                        column: 0,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 3,
                        line: 1,
                        column: 1,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Space,
                code_point: 0x20,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 3,
                        line: 1,
                        column: 1,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 4,
                        line: 1,
                        column: 2,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Content,
                code_point: 0x62,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 4,
                        line: 1,
                        column: 2,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 5,
                        line: 1,
                        column: 3,
                    },
                },
            },
        ];
        let plain = crucible_yaml::plain::PlainScalarView {
            start_line_number: 0,
            end_line_number: 1,
            start_atom_index: 0,
            end_atom_index: 5,
            byte_start: 0,
            byte_end: 5,
        };
        reveal(crucible_yaml::scalar_decode::decode_plain_content_spec);
        reveal(crucible_yaml::scalar_decode::plain_scalar_range_matches_atoms_spec);
        reveal(crucible_yaml::plain::plain_scalar_range_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::decode_plain_loop_spec, 8);
        reveal(crucible_yaml::scalar_decode::plain_step_spec);
        reveal(crucible_yaml::scalar_decode::scalar_atom_white_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::skip_scalar_white_spec, 8);
        reveal_with_fuel(crucible_yaml::scalar_decode::single_quoted_break_group_spec, 8);
        reveal(crucible_yaml::scalar_decode::folded_break_content_spec);
        reveal(crucible_yaml::scalar_decode::direct_atom_content_spec);
        reveal(crucible_yaml::scalar_decode::prepend_decoded_content_result_spec);
        let direct_a = crucible_yaml::scalar_decode::DecodedContentScalarView {
            code_point: 0x61,
            source_atom_start: 0,
            source_atom_end: 1,
            byte_start: 0,
            byte_end: 1,
            origin: DecodedContentOrigin::Direct,
        };
        let folded_space = crucible_yaml::scalar_decode::DecodedContentScalarView {
            code_point: 0x20,
            source_atom_start: 1,
            source_atom_end: 2,
            byte_start: 1,
            byte_end: 2,
            origin: DecodedContentOrigin::FoldedLineBreak,
        };
        let direct_b = crucible_yaml::scalar_decode::DecodedContentScalarView {
            code_point: 0x62,
            source_atom_start: 4,
            source_atom_end: 5,
            byte_start: 4,
            byte_end: 5,
            origin: DecodedContentOrigin::Direct,
        };
        let expected = seq![direct_a, folded_space, direct_b];
        assert(crucible_yaml::plain::plain_scalar_range_spec(atoms, plain));
        assert(crucible_yaml::scalar_decode::plain_scalar_range_matches_atoms_spec(atoms, plain));
        assert(crucible_yaml::scalar_decode::plain_step_spec(atoms, 0, 5, 3) == Ok(
            (seq![direct_a], 1),
        ));
        assert(crucible_yaml::scalar_decode::single_quoted_break_group_spec(atoms, 1, 5, 4) == (
            seq![1int],
            4,
        ));
        assert(crucible_yaml::scalar_decode::plain_step_spec(atoms, 1, 5, 2) == Ok(
            (seq![folded_space], 4),
        ));
        assert(crucible_yaml::scalar_decode::plain_step_spec(atoms, 4, 5, 1) == Ok(
            (seq![direct_b], 5),
        ));
        assert(crucible_yaml::scalar_decode::decode_plain_loop_spec(atoms, 5, 5, 0, 2) == Ok(
            Seq::empty(),
        ));
        assert(crucible_yaml::scalar_decode::decode_plain_loop_spec(atoms, 4, 5, 1, 3) == Ok(
            seq![direct_b],
        ));
        assert(seq![folded_space] + seq![direct_b] =~= seq![folded_space, direct_b]);
        assert(crucible_yaml::scalar_decode::decode_plain_loop_spec(atoms, 1, 5, 2, 4) == Ok(
            seq![folded_space, direct_b],
        ));
        assert(seq![direct_a] + seq![folded_space, direct_b] =~= expected);
        assert(crucible_yaml::scalar_decode::decode_plain_loop_spec(atoms, 0, 5, 3, 5) == Ok(
            expected,
        ));
        assert(crucible_yaml::scalar_decode::decode_plain_content_spec(
            atoms,
            plain,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 3 },
        ) == Ok(
            crucible_yaml::scalar_decode::DecodedScalarContentView {
                style: crucible_yaml::scalar_decode::DecodedScalarStyle::Plain,
                content: expected,
            },
        ));
    }
}

#[test]
fn pure_plain_cap_fails_at_the_folded_record_source_break() {
    proof {
        let atoms = seq![
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Content,
                code_point: 0x61,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 7,
                        line: 0,
                        column: 0,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 8,
                        line: 0,
                        column: 1,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::LineFeed,
                code_point: 0x0a,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 8,
                        line: 0,
                        column: 1,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 9,
                        line: 1,
                        column: 0,
                    },
                },
            },
            crucible_yaml::atom::LexicalAtomView {
                kind: crucible_yaml::LexicalAtomKind::Content,
                code_point: 0x62,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 9,
                        line: 1,
                        column: 0,
                    },
                    end: crucible_yaml::utf8::SourcePositionView {
                        byte_offset: 10,
                        line: 1,
                        column: 1,
                    },
                },
            },
        ];
        let plain = crucible_yaml::plain::PlainScalarView {
            start_line_number: 0,
            end_line_number: 1,
            start_atom_index: 0,
            end_atom_index: 3,
            byte_start: 7,
            byte_end: 10,
        };
        reveal(crucible_yaml::scalar_decode::decode_plain_content_spec);
        reveal(crucible_yaml::scalar_decode::plain_scalar_range_matches_atoms_spec);
        reveal(crucible_yaml::plain::plain_scalar_range_spec);
        reveal(crucible_yaml::scalar_decode::effective_scalar_content_limit_spec);
        reveal_with_fuel(crucible_yaml::scalar_decode::decode_plain_loop_spec, 3);
        reveal(crucible_yaml::scalar_decode::plain_step_spec);
        reveal(crucible_yaml::scalar_decode::scalar_atom_white_spec);
        reveal(crucible_yaml::scalar_decode::single_quoted_break_group_spec);
        reveal(crucible_yaml::scalar_decode::folded_break_content_spec);
        reveal(crucible_yaml::scalar_decode::direct_atom_content_spec);
        reveal(crucible_yaml::scalar_decode::prepend_decoded_content_result_spec);
        assert(crucible_yaml::scalar_decode::decode_plain_content_spec(
            atoms,
            plain,
            crucible_yaml::scalar_decode::ScalarDecodeLimitsView { max_content_code_points: 1 },
        ) == Err(
            crucible_yaml::scalar_decode::ScalarDecodeErrorView {
                kind: crucible_yaml::ScalarDecodeErrorKind::ContentLimitExceeded,
                byte_offset: 8,
            },
        ));
    }
}

} // verus!
