#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits, decode_profile1,
    scan_profile1_block_scalars, scan_profile1_plain_scalars, scan_profile1_quoted_scalars,
    scan_profile1_structural_lexemes, AtomizeLimits, BlockScalarScanLimits, BomPolicy,
    DecodeLimits,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_empty_block_scan_is_proved_successful_and_semantically_valid() {
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
    proof {
        crucible_yaml::structural::lemma_empty_structural_scan_has_no_lexemes(
            atomized@,
            layout@,
            structural_limits@,
            structural@,
        );
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
    let plain_limits = canonical_plain_scalar_limits();
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
    match scan_profile1_block_scalars(
        &atomized,
        &layout,
        &structural,
        &quoted,
        &plain,
        block_limits,
    ) {
        Ok(_blocks) => {
            proof {
                assert(crucible_yaml::block::scan_profile1_block_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    block_limits@,
                ) == Ok(_blocks@));
                assert(crucible_yaml::block::block_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    _blocks@,
                ));
                crucible_yaml::block::lemma_block_well_formed_has_exact_ranges_and_content(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    _blocks@,
                );
                crucible_yaml::block::lemma_empty_block_scan_has_no_scalars(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    block_limits@,
                    _blocks@,
                );
            }
            assert(_blocks@.scalars.len() == 0);
        },
        Err(_) => assert(false),
    }
}

#[test]
fn forged_block_range_or_content_provenance_cannot_be_laundered() {
    proof {
        let atom = crucible_yaml::atom::LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Indicator(
                crucible_yaml::YamlIndicator::LiteralBlockScalar,
            ),
            code_point: 0x7c,
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
        let forged_content = crucible_yaml::block::BlockScalarContentScalarView {
            code_point: 0x20,
            source_atom_index: 9,
            byte_start: 0,
            byte_end: 99,
            origin: crucible_yaml::BlockScalarContentOrigin::FoldedLineBreak,
        };
        let forged_scalar = crucible_yaml::block::BlockScalarView {
            style: crucible_yaml::BlockScalarStyle::Literal,
            chomping: crucible_yaml::BlockChomping::Clip,
            explicit_indentation: None,
            parent_indentation: 0,
            content_indentation: 1,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            header_end_atom_index: 1,
            content_start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 0,
            byte_end: 9,
            content: Seq::empty().push(forged_content),
        };
        reveal(crucible_yaml::block::block_scalar_range_and_content_spec);
        assert(!crucible_yaml::block::block_scalar_range_and_content_spec(
            atomized.atoms,
            Seq::empty(),
            forged_scalar,
        ));
    }
}

#[test]
fn public_nonempty_literal_and_folded_provenance_contracts_are_nonvacuous() {
    proof {
        let direct = crucible_yaml::block::BlockScalarContentScalarView {
            code_point: 0x61,
            source_atom_index: 1,
            byte_start: 1,
            byte_end: 2,
            origin: crucible_yaml::BlockScalarContentOrigin::Direct,
        };
        let folded = crucible_yaml::block::BlockScalarContentScalarView {
            code_point: 0x20,
            source_atom_index: 2,
            byte_start: 2,
            byte_end: 4,
            origin: crucible_yaml::BlockScalarContentOrigin::FoldedLineBreak,
        };
        assert(crucible_yaml::block::content_provenance_code_point_spec(0x61, direct));
        assert(crucible_yaml::block::content_provenance_code_point_spec(0x0a, folded));
        assert(direct.source_atom_index < folded.source_atom_index);
    }
}

#[test]
fn public_provenance_rejects_header_indentation_and_literal_fold_origins() {
    proof {
        let position = |byte_offset: u64, line: u64, column: u64|
            crucible_yaml::utf8::SourcePositionView { byte_offset, line, column };
        let atom = |
            kind: crucible_yaml::LexicalAtomKind,
            code_point: u32,
            byte_offset: u64,
            line: u64,
            column: u64,
        |
            crucible_yaml::atom::LexicalAtomView {
                kind,
                code_point,
                span: crucible_yaml::utf8::SourceSpanView {
                    start: position(byte_offset, line, column),
                    end: position((byte_offset + 1) as u64, line, (column + 1) as u64),
                },
            };
        let atoms = Seq::empty().push(
            atom(
                crucible_yaml::LexicalAtomKind::Indicator(
                    crucible_yaml::YamlIndicator::LiteralBlockScalar,
                ),
                0x7c,
                0,
                0,
                0,
            ),
        ).push(atom(crucible_yaml::LexicalAtomKind::LineFeed, 0x0a, 1, 0, 1)).push(
            atom(crucible_yaml::LexicalAtomKind::Space, 0x20, 2, 1, 0),
        ).push(atom(crucible_yaml::LexicalAtomKind::LineFeed, 0x0a, 3, 1, 1));
        let header_line = crucible_yaml::layout::LayoutLineView {
            line_number: 0,
            start_atom_index: 0,
            content_atom_index: 0,
            end_atom_index: 1,
            terminated: true,
            indentation_columns: 0,
            byte_start: 0,
            content_byte_start: 0,
            byte_end: 2,
        };
        let body_line = crucible_yaml::layout::LayoutLineView {
            line_number: 1,
            start_atom_index: 2,
            content_atom_index: 3,
            end_atom_index: 3,
            terminated: true,
            indentation_columns: 1,
            byte_start: 2,
            content_byte_start: 3,
            byte_end: 4,
        };
        let lines = Seq::empty().push(header_line).push(body_line);
        let content = |
            code_point: u32,
            source_atom_index: u64,
            origin: crucible_yaml::BlockScalarContentOrigin,
        |
            crucible_yaml::block::BlockScalarContentScalarView {
                code_point,
                source_atom_index,
                byte_start: atoms[source_atom_index as int].span.start.byte_offset,
                byte_end: atoms[source_atom_index as int].span.end.byte_offset,
                origin,
            };
        let scalar = |item: crucible_yaml::block::BlockScalarContentScalarView|
            crucible_yaml::block::BlockScalarView {
                style: crucible_yaml::BlockScalarStyle::Literal,
                chomping: crucible_yaml::BlockChomping::Clip,
                explicit_indentation: None,
                parent_indentation: 0,
                content_indentation: 1,
                start_line_number: 0,
                end_line_number: 1,
                start_atom_index: 0,
                header_end_atom_index: 2,
                content_start_atom_index: 2,
                end_atom_index: 4,
                byte_start: 0,
                byte_end: 4,
                content: Seq::empty().push(item),
            };
        let header = scalar(content(0x7c, 0, crucible_yaml::BlockScalarContentOrigin::Direct));
        let indentation = scalar(content(0x20, 2, crucible_yaml::BlockScalarContentOrigin::Direct));
        let literal_fold = scalar(
            content(0x20, 3, crucible_yaml::BlockScalarContentOrigin::FoldedLineBreak),
        );
        reveal(crucible_yaml::block::block_content_scalar_provenance_spec);
        assert(!crucible_yaml::block::block_content_scalar_provenance_spec(
            atoms,
            lines,
            header,
            0,
        ));
        assert(!crucible_yaml::block::block_content_scalar_provenance_spec(
            atoms,
            lines,
            indentation,
            0,
        ));
        assert(!crucible_yaml::block::block_content_scalar_provenance_spec(
            atoms,
            lines,
            literal_fold,
            0,
        ));
    }
}

} // verus!
