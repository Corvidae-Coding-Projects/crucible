#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_quoted_scalar_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits, decode_profile1,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AtomizeLimits, BomPolicy, DecodeLimits, PlainScalarScanLimits,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_empty_plain_scan_is_proved_successful_and_semantically_valid() {
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
    proof {
        assert(atomized@.atoms.len() == 0);
        assert(structural@.lexemes.len() == 0) by {
            crucible_yaml::structural::lemma_structural_well_formed_has_exact_partition(
                atomized@,
                layout@,
                structural@,
            );
            reveal(crucible_yaml::structural::structural_lexeme_partition_spec);
        }
    }
    let quote_limits = canonical_quoted_scalar_limits();
    proof {
        crucible_yaml::quoted::lemma_empty_input_fits_quoted_scalar_scan_limits(
            atomized@,
            layout@,
            structural@,
            quote_limits@,
        );
    }
    let quoted = match scan_profile1_quoted_scalars(&atomized, &layout, &structural, quote_limits) {
        Ok(source) => source,
        Err(_) => {
            assert(false);
            return;
        },
    };
    let plain_limits = PlainScalarScanLimits::new(0, 0);
    proof {
        crucible_yaml::quoted::lemma_empty_quoted_scan_has_no_scalars(
            atomized@,
            layout@,
            structural@,
            quote_limits@,
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
    match scan_profile1_plain_scalars(&atomized, &layout, &structural, &quoted, plain_limits) {
        Ok(_plain) => {
            proof {
                assert(crucible_yaml::plain::scan_profile1_plain_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain_limits@,
                ) == Ok(_plain@));
                assert(crucible_yaml::plain::plain_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    _plain@,
                ));
                crucible_yaml::plain::lemma_plain_well_formed_has_exact_ranges(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    _plain@,
                );
                assert(crucible_yaml::plain::plain_scalar_ranges_well_formed_spec(
                    atomized@,
                    _plain@,
                ));
            }
            assert(_plain@.scalars.len() == 0);
        },
        Err(_) => assert(false),
    }
}

#[test]
fn forged_plain_range_cannot_be_laundered_by_semantic_validity() {
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
        let atomized = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 1,
            bom_bytes: 0,
            atoms: Seq::empty().push(atom),
        };
        let forged_scalar = crucible_yaml::plain::PlainScalarView {
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 2,
            byte_start: 0,
            byte_end: 9,
        };
        let forged_plain = crucible_yaml::plain::PlainScalarSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            structural_transformation_version: 1,
            quoted_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 1,
            bom_bytes: 0,
            input_atom_count: 1,
            input_line_count: 1,
            input_structural_lexeme_count: 1,
            input_quoted_scalar_count: 0,
            scalars: Seq::empty().push(forged_scalar),
        };
        reveal(crucible_yaml::plain::plain_scalar_ranges_well_formed_spec);
        reveal(crucible_yaml::plain::plain_scalar_sequence_ranges_spec);
        reveal(crucible_yaml::plain::plain_scalar_range_spec);
        assert(!crucible_yaml::plain::plain_scalar_range_spec(atomized.atoms, forged_scalar));
        assert(!crucible_yaml::plain::plain_scalar_sequence_ranges_spec(
            atomized.atoms,
            forged_plain.scalars,
        )) by {
            if crucible_yaml::plain::plain_scalar_sequence_ranges_spec(
                atomized.atoms,
                forged_plain.scalars,
            ) {
                assert(forged_plain.scalars[0] == forged_scalar);
                assert(crucible_yaml::plain::plain_scalar_range_spec(
                    atomized.atoms,
                    forged_plain.scalars[0],
                ));
                assert(false);
            }
        }
        assert(!crucible_yaml::plain::plain_scalar_ranges_well_formed_spec(atomized, forged_plain));
        assert forall|
            layout: crucible_yaml::layout::LayoutSourceView,
            structural: crucible_yaml::structural::StructuralLexemeSourceView,
            quoted: crucible_yaml::quoted::QuotedScalarSourceView,
        |
            !crucible_yaml::plain::plain_scalar_source_well_formed_spec(
                atomized,
                layout,
                structural,
                quoted,
                forged_plain,
            ) by {
            if crucible_yaml::plain::plain_scalar_source_well_formed_spec(
                atomized,
                layout,
                structural,
                quoted,
                forged_plain,
            ) {
                crucible_yaml::plain::lemma_plain_well_formed_has_exact_ranges(
                    atomized,
                    layout,
                    structural,
                    quoted,
                    forged_plain,
                );
                assert(false);
            }
        }
    }
}

#[test]
fn public_plain_range_contract_is_nonvacuous_for_two_ordered_scalars() {
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
        let space = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Space,
            code_point: 0x20,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 1,
                    line: 0,
                    column: 1,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 2, line: 0, column: 2 },
            },
        };
        let second_atom = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
            code_point: 0x62,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 2,
                    line: 0,
                    column: 2,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 3, line: 0, column: 3 },
            },
        };
        let atoms = Seq::empty().push(first_atom).push(space).push(second_atom);
        let first = crucible_yaml::plain::PlainScalarView {
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
        };
        let second = crucible_yaml::plain::PlainScalarView {
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
        };
        let atomized = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 3,
            bom_bytes: 0,
            atoms,
        };
        let plain = crucible_yaml::plain::PlainScalarSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            structural_transformation_version: 1,
            quoted_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 3,
            bom_bytes: 0,
            input_atom_count: 3,
            input_line_count: 1,
            input_structural_lexeme_count: 3,
            input_quoted_scalar_count: 0,
            scalars: Seq::empty().push(first).push(second),
        };
        reveal(crucible_yaml::plain::plain_scalar_ranges_well_formed_spec);
        reveal(crucible_yaml::plain::plain_scalar_sequence_ranges_spec);
        reveal(crucible_yaml::plain::plain_scalar_range_spec);
        assert(crucible_yaml::plain::plain_scalar_range_spec(atoms, first));
        assert(crucible_yaml::plain::plain_scalar_range_spec(atoms, second));
        assert(first.end_atom_index <= second.start_atom_index);
        assert(first.byte_end <= second.byte_start);
        assert forall|index: int|
            0 <= index < plain.scalars.len() implies crucible_yaml::plain::plain_scalar_range_spec(
            atoms,
            #[trigger] plain.scalars[index],
        ) && (index > 0 ==> plain.scalars[index - 1].end_atom_index
            <= plain.scalars[index].start_atom_index && plain.scalars[index - 1].byte_end
            <= plain.scalars[index].byte_start) by {
            if index == 0 {
                assert(plain.scalars[index] == first);
            } else {
                assert(index == 1);
                assert(plain.scalars[index] == second);
                assert(plain.scalars[index - 1] == first);
            }
        }
        assert(crucible_yaml::plain::plain_scalar_ranges_well_formed_spec(atomized, plain));
    }
}

} // verus!
