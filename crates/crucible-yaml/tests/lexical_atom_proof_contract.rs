#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{
    atomize_profile1, decode_profile1, AtomizeErrorKind, AtomizeLimits, BomPolicy, DecodeLimits,
    LexicalAtomKind, YamlIndicator,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_atomization_has_an_exact_total_pure_result_and_public_views() {
    let input: &[u8] = &[0x2d, 0x20, 0x61, 0x0d, 0x0a];
    let decode_limits = DecodeLimits::new(16, 16);
    proof {
        reveal_with_fuel(crucible_yaml::utf8::profile1_decodable_tail_spec, 6);
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
    match atomize_profile1(&decoded, atom_limits) {
        Ok(_atomized) => {
            assert(crucible_yaml::atom::atomize_profile1_spec(decoded@, atom_limits@) == Ok(
                _atomized@,
            ));
            assert(_atomized@.atoms.len() == decoded@.scalars.len());
            assert(_atomized@.atoms == crucible_yaml::atom::lexical_atoms_for_scalars_spec(
                decoded@.scalars,
            ));
            assert(crucible_yaml::atom::lexical_atom_kind_spec(0x2d) == LexicalAtomKind::Indicator(
                YamlIndicator::BlockSequenceEntry,
            ));
            assert(_atomized@.atoms.len() <= crucible_yaml::MAX_PROFILE1_LEXICAL_ATOMS);
            assert(crucible_yaml::atom::atomized_source_corresponds_spec(decoded@, _atomized@));
            proof {
                crucible_yaml::atom::lemma_atomized_correspondence_preserves_validity(
                    decoded@,
                    _atomized@,
                );
            }
            assert(crucible_yaml::atom::atomized_source_well_formed_spec(decoded@, _atomized@));
        },
        Err(_) => assert(false),
    }
}

#[test]
fn atom_limit_total_pure_result_has_the_exact_first_excluded_span() {
    let _atom_limits = AtomizeLimits::new(2);
    proof {
        let scalar0 = crucible_yaml::utf8::DecodedScalarView {
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
        let scalar1 = crucible_yaml::utf8::DecodedScalarView {
            code_point: 0x62,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 1,
                    line: 0,
                    column: 1,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 2, line: 0, column: 2 },
            },
        };
        let scalar2 = crucible_yaml::utf8::DecodedScalarView {
            code_point: 0x63,
            span: crucible_yaml::utf8::SourceSpanView {
                start: crucible_yaml::utf8::SourcePositionView {
                    byte_offset: 2,
                    line: 0,
                    column: 2,
                },
                end: crucible_yaml::utf8::SourcePositionView { byte_offset: 3, line: 0, column: 3 },
            },
        };
        let decoded = crucible_yaml::utf8::DecodedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 3,
            bom_bytes: 0,
            scalars: Seq::empty().push(scalar0).push(scalar1).push(scalar2),
        };
        crucible_yaml::atom::lemma_atomize_limit_error(decoded, _atom_limits@);
        assert(crucible_yaml::atom::atomize_profile1_spec(decoded, _atom_limits@) == Err(
            crucible_yaml::atom::AtomizeErrorView {
                kind: AtomizeErrorKind::AtomLimitExceeded,
                byte_offset: 2,
            },
        ));
    }
}

#[test]
fn invalid_decoded_ghost_views_are_not_semantically_well_formed_atom_sources() {
    proof {
        let invalid_scalar = crucible_yaml::utf8::DecodedScalarView {
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
        let decoded = crucible_yaml::utf8::DecodedSourceView {
            profile_version: 999,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            scalars: Seq::empty().push(invalid_scalar),
        };
        let atomized = crucible_yaml::atom::AtomizedSourceView {
            profile_version: 1,
            transformation_version: 1,
            source_len_bytes: 2,
            bom_bytes: 0,
            atoms: crucible_yaml::atom::lexical_atoms_for_scalars_spec(decoded.scalars),
        };
        if crucible_yaml::atom::atomized_source_well_formed_spec(decoded, atomized) {
            crucible_yaml::atom::lemma_atomized_well_formed_scalar_is_normalized(
                decoded,
                atomized,
                0,
            );
            assert(!crucible_yaml::utf8::normalized_scalar_view_spec(invalid_scalar));
            assert(false);
        }
        assert(!crucible_yaml::atom::atomized_source_well_formed_spec(decoded, atomized));
    }
}

} // verus!
