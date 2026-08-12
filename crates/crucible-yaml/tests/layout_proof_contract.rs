#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, decode_profile1, AtomizeLimits, BomPolicy,
    DecodeLimits, LayoutErrorKind, LayoutLimits, LexicalAtomKind,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_layout_has_an_exact_total_pure_result_and_semantic_validity() {
    let input: &[u8] = &[0x20, 0x20, 0x61, 0x0d, 0x0a, 0x62];
    let decode_limits = DecodeLimits::new(16, 16);
    proof {
        reveal_with_fuel(crucible_yaml::utf8::profile1_decodable_tail_spec, 7);
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
    let atom_limits = AtomizeLimits::new(16);
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
        assert(crucible_yaml::atom::atomized_source_well_formed_spec(decoded@, atomized@));
        crucible_yaml::atom::lemma_atomized_well_formed_is_intrinsic(decoded@, atomized@);
    }
    let layout_limits = LayoutLimits::new(16, 16);
    proof {
        crucible_yaml::layout::lemma_short_atom_stream_fits_layout_limits(
            atomized@,
            layout_limits@,
        );
    }
    match analyze_profile1_layout(&atomized, layout_limits) {
        Ok(_layout) => {
            assert(crucible_yaml::layout::analyze_profile1_layout_spec(atomized@, layout_limits@)
                == Ok(_layout@));
            assert(_layout@.lines.len() <= layout_limits@.max_lines);
            assert(_layout@.lines.len() <= crucible_yaml::MAX_PROFILE1_LAYOUT_LINES);
            assert(crucible_yaml::layout::layout_source_well_formed_spec(atomized@, _layout@));
            assert(_layout@.profile_version == 1);
            assert(_layout@.transformation_version == 1);
            assert(_layout@.input_transformation_version == atomized@.transformation_version);
            assert(_layout@.source_len_bytes == atomized@.source_len_bytes);
            assert(_layout@.bom_bytes == atomized@.bom_bytes);
        },
        Err(_error) => {
            assert(crucible_yaml::layout::analyze_profile1_layout_spec(atomized@, layout_limits@)
                == Err(_error@));
            assert(false);
        },
    }
}

#[test]
fn forged_over_cap_and_intrinsically_invalid_atom_views_are_rejected_by_the_pure_contract() {
    proof {
        let ordinary_atom = crucible_yaml::atom::LexicalAtomView {
            kind: LexicalAtomKind::Content,
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
        let rejected_atom = crucible_yaml::atom::LexicalAtomView {
            kind: LexicalAtomKind::Content,
            code_point: 0x62,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 4_242,
                    line: 0,
                    column: 0,
                },
                end: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 4_243,
                    line: 0,
                    column: 1,
                },
            },
        };
        let over_cap = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 4_243,
            bom_bytes: 0,
            atoms: Seq::new(
                (crucible_yaml::MAX_PROFILE1_LEXICAL_ATOMS + 1) as nat,
                |index: int|
                    if index == crucible_yaml::MAX_PROFILE1_LEXICAL_ATOMS as int {
                        rejected_atom
                    } else {
                        ordinary_atom
                    },
            ),
        };
        let limits = crucible_yaml::layout::LayoutLimitsView {
            max_lines: 0,
            max_indentation_columns: 0,
        };
        assert(over_cap.atoms.len() == crucible_yaml::MAX_PROFILE1_LEXICAL_ATOMS + 1);
        assert(over_cap.atoms[crucible_yaml::MAX_PROFILE1_LEXICAL_ATOMS as int] == rejected_atom);
        crucible_yaml::layout::lemma_layout_input_atom_limit_error(over_cap, limits);
        assert(crucible_yaml::layout::analyze_profile1_layout_spec(over_cap, limits) == Err(
            crucible_yaml::layout::LayoutErrorView {
                kind: LayoutErrorKind::InputAtomLimitExceeded,
                byte_offset: 4_242,
            },
        ));

        let invalid_atom = crucible_yaml::atom::LexicalAtomView {
            kind: LexicalAtomKind::Content,
            code_point: 0xd800,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 2,
                    line: 0,
                    column: 2,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 1, line: 0, column: 1 },
            },
        };
        let invalid_atomized = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            atoms: Seq::empty().push(invalid_atom),
        };
        if crucible_yaml::atom::atomized_source_intrinsically_well_formed_spec(invalid_atomized) {
            crucible_yaml::atom::lemma_intrinsic_atomized_scalar_is_normalized(invalid_atomized, 0);
            assert(!crucible_yaml::utf8::normalized_scalar_view_spec(
                crucible_yaml::utf8::DecodedScalarView {
                    code_point: invalid_atom.code_point,
                    span: invalid_atom.span,
                },
            ));
            assert(false);
        }
        assert(!crucible_yaml::atom::atomized_source_intrinsically_well_formed_spec(
            invalid_atomized,
        ));
        let forged_layout = crucible_yaml::layout::LayoutSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            lines: Seq::empty(),
        };
        if crucible_yaml::layout::layout_source_well_formed_spec(invalid_atomized, forged_layout) {
            crucible_yaml::layout::lemma_layout_well_formed_requires_intrinsic_atom_source(
                invalid_atomized,
                forged_layout,
            );
            assert(false);
        }
        assert(!crucible_yaml::layout::layout_source_well_formed_spec(
            invalid_atomized,
            forged_layout,
        ));
    }
}

} // verus!
