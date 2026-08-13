#![expect(
    clippy::single_match,
    reason = "explicit match arms carry branch-specific Verus assertions"
)]

use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, scan_profile1_block_scalars,
    scan_profile1_completed_tokens, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, BomPolicy, DecodeLimits,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_empty_completed_scan_is_proved_successful_partitioned_and_balanced() {
    let input: &[u8] = &[];
    let decode_limits = DecodeLimits::new(0, 0);
    proof {
        reveal_with_fuel(crucible_yaml::utf8::profile1_decodable_tail_spec, 1);
        assert(crucible_yaml::utf8::profile1_decodable_spec(
            input@,
            decode_limits@,
            BomPolicy::AllowAndStrip,
        ));
        crucible_yaml::utf8::lemma_profile1_decodable_is_ok(
            input@,
            decode_limits@,
            BomPolicy::AllowAndStrip,
        );
    }
    let decoded = match decode_profile1(input, decode_limits, BomPolicy::AllowAndStrip) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    let atom_limits = AtomizeLimits::new(0);
    proof {
        crucible_yaml::atom::lemma_atomize_within_limits(decoded@, atom_limits@);
    }
    let atomized = match atomize_profile1(&decoded, atom_limits) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    proof {
        crucible_yaml::atom::lemma_atomized_correspondence_preserves_validity(decoded@, atomized@);
        crucible_yaml::atom::lemma_atomized_well_formed_is_intrinsic(decoded@, atomized@);
    }
    let layout_limits = canonical_structural_layout_limits();
    proof {
        crucible_yaml::layout::lemma_short_atom_stream_fits_layout_limits(
            atomized@,
            layout_limits@,
        );
    }
    let layout = match analyze_profile1_layout(&atomized, layout_limits) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    proof {
        crucible_yaml::layout::lemma_empty_layout_has_no_lines(atomized@, layout_limits@, layout@);
    }
    let structural_limits = canonical_structural_scan_limits();
    proof {
        crucible_yaml::structural::lemma_short_well_formed_input_fits_structural_scan_limits(
            atomized@,
            layout@,
            structural_limits@,
        );
    }
    let structural = match scan_profile1_structural_lexemes(&atomized, &layout, structural_limits) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    let quoted_limits = canonical_quoted_scalar_limits();
    proof {
        crucible_yaml::structural::lemma_empty_structural_scan_has_no_lexemes(
            atomized@,
            layout@,
            structural_limits@,
            structural@,
        );
        crucible_yaml::quoted::lemma_empty_input_fits_quoted_scalar_scan_limits(
            atomized@,
            layout@,
            structural@,
            quoted_limits@,
        );
    }
    let quoted = match scan_profile1_quoted_scalars(
        &atomized,
        &layout,
        &structural,
        quoted_limits,
    ) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    let plain_limits = canonical_plain_scalar_limits();
    proof {
        crucible_yaml::quoted::lemma_empty_quoted_scan_has_no_scalars(
            atomized@,
            layout@,
            structural@,
            quoted_limits@,
            quoted@,
        );
        crucible_yaml::plain::lemma_empty_input_fits_plain_scalar_scan_limits(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain_limits@,
        );
    }
    let plain = match scan_profile1_plain_scalars(
        &atomized,
        &layout,
        &structural,
        &quoted,
        plain_limits,
    ) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    let block_limits = canonical_block_scalar_limits();
    proof {
        crucible_yaml::plain::lemma_empty_plain_scan_has_no_scalars(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain_limits@,
            plain@,
        );
        crucible_yaml::block::lemma_empty_input_fits_block_scalar_scan_limits(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block_limits@,
        );
    }
    let block = match scan_profile1_block_scalars(
        &atomized,
        &layout,
        &structural,
        &quoted,
        &plain,
        block_limits,
    ) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    let token_limits = canonical_completed_token_limits();
    proof {
        crucible_yaml::token::lemma_empty_input_fits_completed_token_limits(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
            token_limits@,
        );
        reveal(crucible_yaml::token::completed_token_empty_canonical_inputs_spec);
        assert(crucible_yaml::token::completed_token_empty_canonical_inputs_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
        ));
    }
    match scan_profile1_completed_tokens(
        &atomized,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        token_limits,
    ) {
        Ok(_tokens) => {
            proof {
                assert(crucible_yaml::token::scan_profile1_completed_tokens_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    block@,
                    token_limits@,
                ) == Ok(_tokens@));
                assert(crucible_yaml::token::completed_token_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    block@,
                    _tokens@,
                ));
                assert(crucible_yaml::token::completed_token_partition_spec(atomized@, _tokens@));
                assert(crucible_yaml::token::completed_token_flow_balanced_spec(_tokens@.tokens));
                crucible_yaml::token::lemma_completed_tokens_well_formed_has_exact_formation_and_limits(
                atomized@, layout@, structural@, quoted@, plain@, block@, _tokens@);
                assert(crucible_yaml::token::completed_token_source_corresponds_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    block@,
                    _tokens@,
                ));
                assert(crucible_yaml::token::completed_token_absolute_limits_spec(_tokens@));
                reveal(crucible_yaml::token::completed_token_partition_spec);
                assert(layout@.source_len_bytes == 0);
                let forged_layout = crucible_yaml::layout::LayoutSourceView {
                    source_len_bytes: 1,
                    ..layout@
                };
                assert forall|candidate_limits: crucible_yaml::token::CompletedTokenLimitsView|
                    crucible_yaml::token::scan_profile1_completed_tokens_spec(
                        atomized@,
                        forged_layout,
                        structural@,
                        quoted@,
                        plain@,
                        block@,
                        candidate_limits,
                    ).is_err() by {
                    assert(forged_layout != layout@);
                    crucible_yaml::token::lemma_completed_tokens_reject_noncanonical_layout(
                        atomized@,
                        layout@,
                        forged_layout,
                        structural@,
                        quoted@,
                        plain@,
                        block@,
                        candidate_limits,
                    );
                }
                crucible_yaml::token::lemma_noncanonical_layout_cannot_correspond_to_completed_tokens(
                atomized@, layout@, forged_layout, structural@, quoted@, plain@, block@, _tokens@);
                assert(!crucible_yaml::token::completed_token_source_corresponds_spec(
                    atomized@,
                    forged_layout,
                    structural@,
                    quoted@,
                    plain@,
                    block@,
                    _tokens@,
                ));

                let forged_structural = crucible_yaml::structural::StructuralLexemeSourceView {
                    source_len_bytes: if structural@.source_len_bytes == 0 {
                        1
                    } else {
                        0
                    },
                    ..structural@
                };
                assert(forged_structural != structural@);
                crucible_yaml::token::lemma_noncanonical_structural_cannot_correspond_to_completed_tokens(

                    atomized@,
                    layout@,
                    structural@,
                    forged_structural,
                    quoted@,
                    plain@,
                    block@,
                    _tokens@,
                );
                assert(!crucible_yaml::token::completed_token_source_corresponds_spec(
                    atomized@,
                    layout@,
                    forged_structural,
                    quoted@,
                    plain@,
                    block@,
                    _tokens@,
                ));

                let forged_quoted = crucible_yaml::quoted::QuotedScalarSourceView {
                    source_len_bytes: if quoted@.source_len_bytes == 0 {
                        1
                    } else {
                        0
                    },
                    ..quoted@
                };
                assert(forged_quoted != quoted@);
                crucible_yaml::token::lemma_noncanonical_quoted_cannot_correspond_to_completed_tokens(
                atomized@, layout@, structural@, quoted@, forged_quoted, plain@, block@, _tokens@);
                assert(!crucible_yaml::token::completed_token_source_corresponds_spec(
                    atomized@,
                    layout@,
                    structural@,
                    forged_quoted,
                    plain@,
                    block@,
                    _tokens@,
                ));

                let forged_plain = crucible_yaml::plain::PlainScalarSourceView {
                    source_len_bytes: if plain@.source_len_bytes == 0 {
                        1
                    } else {
                        0
                    },
                    ..plain@
                };
                assert(forged_plain != plain@);
                crucible_yaml::token::lemma_noncanonical_plain_cannot_correspond_to_completed_tokens(
                atomized@, layout@, structural@, quoted@, plain@, forged_plain, block@, _tokens@);
                assert(!crucible_yaml::token::completed_token_source_corresponds_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    forged_plain,
                    block@,
                    _tokens@,
                ));

                let forged_block = crucible_yaml::block::BlockScalarSourceView {
                    source_len_bytes: if block@.source_len_bytes == 0 {
                        1
                    } else {
                        0
                    },
                    ..block@
                };
                assert(forged_block != block@);
                crucible_yaml::token::lemma_noncanonical_block_cannot_correspond_to_completed_tokens(
                atomized@, layout@, structural@, quoted@, plain@, block@, forged_block, _tokens@);
                assert(!crucible_yaml::token::completed_token_source_corresponds_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    forged_block,
                    _tokens@,
                ));
            }
            assert(_tokens@.tokens.len() == 0);
        },
        Err(_) => assert(false),
    }
}

#[test]
fn forged_gap_overlap_and_byte_endpoint_cannot_satisfy_the_public_partition() {
    proof {
        let first_atom = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
            code_point: 0x61,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 0,
                    line: 0,
                    column: 0,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 1, line: 0, column: 1 },
            },
        };
        let second_atom = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::LineFeed,
            code_point: 0x0a,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 1,
                    line: 0,
                    column: 1,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 2, line: 1, column: 0 },
            },
        };
        let atomized = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            atoms: Seq::empty().push(first_atom).push(second_atom),
        };
        let first = crucible_yaml::token::CompletedTokenView {
            kind: crucible_yaml::CompletedTokenKind::PlainScalar,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
            scalar_index: Some(0),
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let line_feed = crucible_yaml::token::CompletedTokenView {
            kind: crucible_yaml::CompletedTokenKind::LineFeed,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let valid = crucible_yaml::token::CompletedTokenSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            structural_transformation_version: 1,
            quoted_transformation_version: 1,
            plain_transformation_version: 1,
            block_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            input_atom_count: 2,
            maximum_flow_depth: 0,
            tokens: Seq::empty().push(first).push(line_feed),
        };
        reveal(crucible_yaml::token::completed_token_partition_spec);
        reveal(crucible_yaml::token::completed_token_sequence_partition_spec);
        reveal(crucible_yaml::token::completed_token_range_spec);
        assert(crucible_yaml::token::completed_token_partition_spec(atomized, valid));

        let gap = crucible_yaml::token::CompletedTokenSourceView {
            tokens: Seq::empty().push(line_feed),
            ..valid
        };
        assert(!crucible_yaml::token::completed_token_partition_spec(atomized, gap));

        let overlap = crucible_yaml::token::CompletedTokenSourceView {
            tokens: Seq::empty().push(first).push(first),
            ..valid
        };
        assert(!crucible_yaml::token::completed_token_partition_spec(atomized, overlap));

        let bad_endpoint_token = crucible_yaml::token::CompletedTokenView { byte_end: 2, ..first };
        let bad_endpoint = crucible_yaml::token::CompletedTokenSourceView {
            tokens: Seq::empty().push(bad_endpoint_token).push(line_feed),
            ..valid
        };
        assert(!crucible_yaml::token::completed_token_range_spec(
            atomized.atoms,
            bad_endpoint_token,
        )) by {
            assert(bad_endpoint_token.byte_end == 2);
            assert(atomized.atoms[0].span.end.byte_offset == 1);
        }
        assert(!crucible_yaml::token::completed_token_sequence_partition_spec(
            atomized.atoms,
            bad_endpoint.tokens,
        )) by {
            if crucible_yaml::token::completed_token_sequence_partition_spec(
                atomized.atoms,
                bad_endpoint.tokens,
            ) {
                assert(bad_endpoint.tokens[0] == bad_endpoint_token);
                assert(crucible_yaml::token::completed_token_range_spec(
                    atomized.atoms,
                    bad_endpoint.tokens[0],
                ));
                assert(false);
            }
        }
        assert(!crucible_yaml::token::completed_token_partition_spec(atomized, bad_endpoint));
    }
}

#[test]
fn forged_unbalanced_flow_sequence_is_rejected_by_the_public_balance_predicate() {
    proof {
        let opener = crucible_yaml::token::CompletedTokenView {
            kind: crucible_yaml::CompletedTokenKind::FlowSequenceStart,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        reveal(crucible_yaml::token::completed_token_flow_balanced_spec);
        reveal(crucible_yaml::token::completed_token_flow_prefix_spec);
        reveal(crucible_yaml::token::completed_token_flow_stack_after_kind_spec);
        assert(!crucible_yaml::token::completed_token_flow_balanced_spec(Seq::empty().push(opener)))
            by {
            if crucible_yaml::token::completed_token_flow_balanced_spec(Seq::empty().push(opener)) {
                let tokens = Seq::empty().push(opener);
                let states = choose|states: Seq<Seq<crucible_yaml::CompletedTokenKind>>|
                    states.len() == 2 && states[0].len() == 0 && states[1] == Seq::<
                        crucible_yaml::CompletedTokenKind,
                    >::empty() && forall|index: int|
                        0 <= index < 1
                            ==> crucible_yaml::token::completed_token_flow_stack_after_kind_spec(
                            #[trigger] states[index],
                            tokens[index].kind,
                        ) == Some(states[index + 1]);
                assert(states[0] == Seq::<crucible_yaml::CompletedTokenKind>::empty());
                assert(crucible_yaml::token::completed_token_flow_stack_after_kind_spec(
                    states[0],
                    opener.kind,
                ) == Some(states[1]));
                assert(states[1] == Seq::empty().push(
                    crucible_yaml::CompletedTokenKind::FlowSequenceStart,
                ));
                assert(false);
            }
        }

        let wrong_closer = crucible_yaml::token::CompletedTokenView {
            kind: crucible_yaml::CompletedTokenKind::FlowMappingEnd,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            ..opener
        };
        let mismatched = Seq::empty().push(opener).push(wrong_closer);
        assert(!crucible_yaml::token::completed_token_flow_balanced_spec(mismatched)) by {
            if crucible_yaml::token::completed_token_flow_balanced_spec(mismatched) {
                let states = choose|states: Seq<Seq<crucible_yaml::CompletedTokenKind>>|
                    states.len() == 3 && states[0].len() == 0 && states[2].len() == 0 && forall|
                        index: int,
                    |
                        0 <= index < 2
                            ==> crucible_yaml::token::completed_token_flow_stack_after_kind_spec(
                            #[trigger] states[index],
                            mismatched[index].kind,
                        ) == Some(states[index + 1]);
                assert(states[0] == Seq::<crucible_yaml::CompletedTokenKind>::empty());
                assert(states[1] == Seq::empty().push(
                    crucible_yaml::CompletedTokenKind::FlowSequenceStart,
                ));
                assert(crucible_yaml::token::completed_token_flow_stack_after_kind_spec(
                    states[1],
                    crucible_yaml::CompletedTokenKind::FlowMappingEnd,
                ) == None);
                assert(false);
            }
        }
    }
}

#[test]
fn forged_parts_scalar_substitution_and_unformed_tokens_are_rejected() {
    proof {
        let atom = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
            code_point: 0x61,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 0,
                    line: 0,
                    column: 0,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 1, line: 0, column: 1 },
            },
        };
        let atoms = Seq::empty().push(atom);
        let scalar = crucible_yaml::plain::PlainScalarView {
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
        };
        let valid = crucible_yaml::token::CompletedTokenView {
            kind: crucible_yaml::CompletedTokenKind::PlainScalar,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
            scalar_index: Some(0),
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let forged_index = crucible_yaml::token::CompletedTokenView {
            scalar_index: Some(1),
            ..valid
        };
        reveal(crucible_yaml::token::completed_token_scalar_identity_spec);
        assert(crucible_yaml::token::completed_token_scalar_identity_spec(
            Seq::empty(),
            Seq::empty().push(scalar),
            Seq::empty(),
            valid,
        ));
        assert(!crucible_yaml::token::completed_token_scalar_identity_spec(
            Seq::empty(),
            Seq::empty().push(scalar),
            Seq::empty(),
            forged_index,
        ));

        let leaked_part = crucible_yaml::token::CompletedTokenPartView {
            kind: crucible_yaml::CompletedTokenPartKind::TagSuffix,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
        };
        let forged_parts = crucible_yaml::token::CompletedTokenView {
            parts: Seq::empty().push(leaked_part),
            ..valid
        };
        reveal(crucible_yaml::token::completed_token_parts_schema_spec);
        assert(!crucible_yaml::token::completed_token_parts_schema_spec(forged_parts));

        reveal(crucible_yaml::token::completed_token_exact_formation_sequence_spec);
        reveal(crucible_yaml::token::completed_token_exact_formation_spec);
        assert(!crucible_yaml::token::completed_token_exact_formation_spec(
            Seq::empty(),
            Seq::empty(),
            Seq::empty(),
            Seq::empty(),
            valid,
        ));
        let unformed = Seq::empty().push(valid);
        assert(!crucible_yaml::token::completed_token_exact_formation_sequence_spec(
            Seq::empty(),
            Seq::empty(),
            Seq::empty(),
            Seq::empty(),
            unformed,
        )) by {
            if crucible_yaml::token::completed_token_exact_formation_sequence_spec(
                Seq::empty(),
                Seq::empty(),
                Seq::empty(),
                Seq::empty(),
                unformed,
            ) {
                assert(unformed[0] == valid);
                assert(crucible_yaml::token::completed_token_exact_formation_spec(
                    Seq::empty(),
                    Seq::empty(),
                    Seq::empty(),
                    Seq::empty(),
                    unformed[0],
                ));
                assert(false);
            }
        }
    }
}

#[test]
fn forged_absolute_limits_cannot_be_laundered() {
    proof {
        let over_count = crucible_yaml::token::CompletedTokenSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            structural_transformation_version: 1,
            quoted_transformation_version: 1,
            plain_transformation_version: 1,
            block_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 0,
            bom_bytes: 0,
            input_atom_count: 0,
            maximum_flow_depth: 4097u64,
            tokens: Seq::empty(),
        };
        reveal(crucible_yaml::token::completed_token_absolute_limits_spec);
        assert(!crucible_yaml::token::completed_token_absolute_limits_spec(over_count));
    }
}

#[test]
fn truncated_trivia_run_is_not_maximal() {
    proof {
        let first = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Space,
            code_point: 0x20,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 0,
                    line: 0,
                    column: 0,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 1, line: 0, column: 1 },
            },
        };
        let second = crucible_yaml::atom::LexicalAtomView {
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 1,
                    line: 0,
                    column: 1,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 2, line: 0, column: 2 },
            },
            ..first
        };
        let truncated = crucible_yaml::token::CompletedTokenView {
            kind: crucible_yaml::CompletedTokenKind::Indentation,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        reveal(crucible_yaml::token::completed_token_trivia_maximal_spec);
        reveal(crucible_yaml::token::token_is_space_or_tab_spec);
        assert(!crucible_yaml::token::completed_token_trivia_maximal_spec(
            Seq::empty().push(first).push(second),
            truncated,
        ));
    }
}

} // verus!
