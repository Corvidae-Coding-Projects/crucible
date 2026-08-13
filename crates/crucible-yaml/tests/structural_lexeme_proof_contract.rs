#![expect(
    clippy::single_match,
    reason = "explicit match arms carry branch-specific Verus assertions"
)]

use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_structural_layout_limits, decode_profile1,
    scan_profile1_structural_lexemes, AtomizeLimits, BomPolicy, DecodeLimits, StructuralScanLimits,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_structural_scan_has_an_exact_total_pure_result_and_semantic_validity() {
    let input: &[u8] = &[0x61, 0x3a, 0x20, 0x5b, 0x62, 0x2c, 0x20, 0x63, 0x5d, 0x0a];
    let decode_limits = DecodeLimits::new(16, 16);
    proof {
        reveal_with_fuel(crucible_yaml::utf8::profile1_decodable_tail_spec, 11);
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
        crucible_yaml::atom::lemma_atomized_correspondence_preserves_validity(decoded@, atomized@);
        crucible_yaml::atom::lemma_atomized_well_formed_is_intrinsic(decoded@, atomized@);
        assert(crucible_yaml::atom::atomized_source_intrinsically_well_formed_spec(atomized@));
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
    assert(crucible_yaml::layout::layout_source_well_formed_spec(atomized@, layout@));
    let scan_limits = StructuralScanLimits::new(16);
    proof {
        assert(crucible_yaml::layout::analyze_profile1_layout_spec(
            atomized@,
            crucible_yaml::structural::canonical_layout_limits_spec(),
        ) == Ok(layout@));
        crucible_yaml::structural::lemma_short_well_formed_input_fits_structural_scan_limits(
            atomized@,
            layout@,
            scan_limits@,
        );
    }
    match scan_profile1_structural_lexemes(&atomized, &layout, scan_limits) {
        Ok(_lexemes) => {
            assert(crucible_yaml::structural::scan_profile1_structural_lexemes_spec(
                atomized@,
                layout@,
                scan_limits@,
            ) == Ok(_lexemes@));
            assert(crucible_yaml::structural::structural_lexeme_source_well_formed_spec(
                atomized@,
                layout@,
                _lexemes@,
            ));
            proof {
                crucible_yaml::structural::lemma_structural_well_formed_has_exact_partition(
                    atomized@,
                    layout@,
                    _lexemes@,
                );
            }
            assert(crucible_yaml::structural::structural_lexeme_partition_spec(
                atomized@,
                layout@,
                _lexemes@,
            ));
            assert(_lexemes@.lexemes.len() <= scan_limits@.max_lexemes);
            assert(_lexemes@.input_atom_count == atomized@.atoms.len());
            assert(_lexemes@.input_line_count == layout@.lines.len());
        },
        Err(_) => assert(false),
    }
}

#[test]
fn forged_structural_views_cannot_launder_an_invalid_layout_or_atom_source() {
    proof {
        let invalid_atom = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
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
        let forged_structural = crucible_yaml::structural::StructuralLexemeSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            input_atom_count: 1,
            input_line_count: 0,
            lexemes: Seq::empty(),
        };
        if crucible_yaml::structural::structural_lexeme_source_well_formed_spec(
            invalid_atomized,
            forged_layout,
            forged_structural,
        ) {
            crucible_yaml::structural::lemma_structural_well_formed_requires_layout(
                invalid_atomized,
                forged_layout,
                forged_structural,
            );
            assert(false);
        }
        assert(!crucible_yaml::structural::structural_lexeme_source_well_formed_spec(
            invalid_atomized,
            forged_layout,
            forged_structural,
        ));
    }
}

} // verus!
