#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, BomPolicy, DecodeLimits,
    QuotedScalarScanLimits,
};
use vstd::prelude::*;

verus! {

pub open spec fn fixture_atom(
    kind: crucible_yaml::LexicalAtomKind,
    code_point: u32,
    byte_start: u64,
    byte_end: u64,
    start_line: u64,
    end_line: u64,
    start_column: u64,
    end_column: u64,
) -> crucible_yaml::atom::LexicalAtomView {
    crucible_yaml::atom::LexicalAtomView {
        kind,
        code_point,
        span: crucible_yaml::utf8::SourceSpanView {
            start: crucible_yaml::utf8::SourcePositionView {
                byte_offset: byte_start,
                line: start_line,
                column: start_column,
            },
            end: crucible_yaml::utf8::SourcePositionView {
                byte_offset: byte_end,
                line: end_line,
                column: end_column,
            },
        },
    }
}

#[test]
fn executable_quoted_scan_has_an_exact_total_pure_result_and_semantic_validity() {
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
            assert(crucible_yaml::structural::structural_lexeme_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
            ));
            crucible_yaml::structural::lemma_structural_well_formed_has_exact_partition(
                atomized@,
                layout@,
                structural@,
            );
            reveal(crucible_yaml::structural::structural_lexeme_partition_spec);
        }
    }
    let quote_limits = QuotedScalarScanLimits::new(0, 0);
    proof {
        crucible_yaml::quoted::lemma_empty_input_fits_quoted_scalar_scan_limits(
            atomized@,
            layout@,
            structural@,
            quote_limits@,
        );
    }
    match scan_profile1_quoted_scalars(&atomized, &layout, &structural, quote_limits) {
        Ok(_quoted) => {
            proof {
                assert(crucible_yaml::quoted::scan_profile1_quoted_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quote_limits@,
                ) == Ok(_quoted@));
                assert(crucible_yaml::quoted::quoted_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    _quoted@,
                ));
                crucible_yaml::quoted::lemma_quoted_well_formed_has_exact_ranges(
                    atomized@,
                    layout@,
                    structural@,
                    _quoted@,
                );
                assert(crucible_yaml::quoted::quoted_scalar_ranges_well_formed_spec(
                    atomized@,
                    _quoted@,
                ));
            }
            assert(_quoted@.scalars.len() == 0);
        },
        Err(_) => assert(false),
    }
}

#[test]
fn public_range_contract_is_nonvacuous_for_single_and_multiline_double_scalars() {
    proof {
        let atoms = Seq::empty().push(
            fixture_atom(
                crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::SingleQuotedScalar,
                ),
                0x27,
                0,
                1,
                0,
                0,
                0,
                1,
            ),
        ).push(fixture_atom(crucible_yaml::LexicalAtomKind::Content, 0x61, 1, 2, 0, 0, 1, 2)).push(
            fixture_atom(
                crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::SingleQuotedScalar,
                ),
                0x27,
                2,
                3,
                0,
                0,
                2,
                3,
            ),
        ).push(fixture_atom(crucible_yaml::LexicalAtomKind::Space, 0x20, 3, 4, 0, 0, 3, 4)).push(
            fixture_atom(
                crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::DoubleQuotedScalar,
                ),
                0x22,
                4,
                5,
                0,
                0,
                4,
                5,
            ),
        ).push(fixture_atom(crucible_yaml::LexicalAtomKind::Content, 0x62, 5, 6, 0, 0, 5, 6)).push(
            fixture_atom(crucible_yaml::LexicalAtomKind::LineFeed, 0x0a, 6, 7, 0, 1, 6, 0),
        ).push(fixture_atom(crucible_yaml::LexicalAtomKind::Content, 0x63, 7, 8, 1, 1, 0, 1)).push(
            fixture_atom(
                crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::DoubleQuotedScalar,
                ),
                0x22,
                8,
                9,
                1,
                1,
                1,
                2,
            ),
        );
        let single = crucible_yaml::quoted::QuotedScalarView {
            style: crucible_yaml::QuotedScalarStyle::Single,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 3,
            byte_start: 0,
            byte_end: 3,
        };
        let multiline_double = crucible_yaml::quoted::QuotedScalarView {
            style: crucible_yaml::QuotedScalarStyle::Double,
            start_line_number: 0,
            end_line_number: 1,
            start_atom_index: 4,
            end_atom_index: 9,
            byte_start: 4,
            byte_end: 9,
        };
        let atomized = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 9,
            bom_bytes: 0,
            atoms,
        };
        let quoted = crucible_yaml::quoted::QuotedScalarSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            structural_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 9,
            bom_bytes: 0,
            input_atom_count: 9,
            input_line_count: 2,
            input_structural_lexeme_count: 9,
            scalars: Seq::empty().push(single).push(multiline_double),
        };
        reveal(crucible_yaml::quoted::quoted_scalar_ranges_well_formed_spec);
        reveal(crucible_yaml::quoted::quoted_scalar_sequence_ranges_spec);
        reveal(crucible_yaml::quoted::quoted_scalar_range_spec);
        assert(crucible_yaml::quoted::quoted_scalar_range_spec(atoms, single));
        assert(crucible_yaml::quoted::quoted_scalar_range_spec(atoms, multiline_double));
        assert(single.end_atom_index <= multiline_double.start_atom_index);
        assert(single.byte_end <= multiline_double.byte_start);
        assert forall|index: int|
            0 <= index
                < quoted.scalars.len() implies crucible_yaml::quoted::quoted_scalar_range_spec(
            atoms,
            #[trigger] quoted.scalars[index],
        ) && (index > 0 ==> quoted.scalars[index - 1].end_atom_index
            <= quoted.scalars[index].start_atom_index && quoted.scalars[index - 1].byte_end
            <= quoted.scalars[index].byte_start) by {
            if index == 0 {
                assert(quoted.scalars[index] == single);
            } else {
                assert(index == 1);
                assert(quoted.scalars[index] == multiline_double);
                assert(quoted.scalars[index - 1] == single);
            }
        }
        assert(crucible_yaml::quoted::quoted_scalar_ranges_well_formed_spec(atomized, quoted));
    }
}

} // verus!
