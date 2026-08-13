#[expect(
    unused_imports,
    reason = "these variants are referenced only inside Verus proof code"
)]
use crucible_yaml::{ResolvedTagKind, ResolvedTagOrigin, TagResolutionErrorKind};
use vstd::prelude::*;

verus! {

#[test]
fn pure_tag_finalization_preserves_percent_escape_spelling_exactly() {
    proof {
        let content = seq![
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x21,
                source_atom_index: 4,
                byte_start: 7,
                byte_end: 8,
                origin: ResolvedTagOrigin::TagSuffix,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x78,
                source_atom_index: 5,
                byte_start: 8,
                byte_end: 9,
                origin: ResolvedTagOrigin::TagSuffix,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x25,
                source_atom_index: 6,
                byte_start: 9,
                byte_end: 10,
                origin: ResolvedTagOrigin::TagSuffix,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x34,
                source_atom_index: 7,
                byte_start: 10,
                byte_end: 11,
                origin: ResolvedTagOrigin::TagSuffix,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x31,
                source_atom_index: 8,
                byte_start: 11,
                byte_end: 12,
                origin: ResolvedTagOrigin::TagSuffix,
            },
        ];
        reveal(crucible_yaml::resolve_tag::finalize_resolved_tag_property_spec);
        reveal(crucible_yaml::resolve_tag::effective_tag_code_point_limit_spec);
        reveal(crucible_yaml::resolve_tag::global_tag_uri_spec);
        assert(crucible_yaml::resolve_tag::finalize_resolved_tag_property_spec(
            content,
            2,
            5,
            crucible_yaml::resolve_tag::TagResolutionLimitsView { max_tag_code_points: 5 },
        ) == Ok(
            crucible_yaml::resolve_tag::ResolvedTagPropertyView {
                kind: ResolvedTagKind::Local,
                token_index: 2,
                content,
            },
        ));
    }
}

#[test]
fn pure_tag_limit_and_invalid_global_uri_have_exact_precedence() {
    proof {
        let invalid = seq![
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x24,
                source_atom_index: 2,
                byte_start: 2,
                byte_end: 3,
                origin: ResolvedTagOrigin::VerbatimPayload,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x3a,
                source_atom_index: 3,
                byte_start: 3,
                byte_end: 4,
                origin: ResolvedTagOrigin::VerbatimPayload,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x3f,
                source_atom_index: 4,
                byte_start: 4,
                byte_end: 5,
                origin: ResolvedTagOrigin::VerbatimPayload,
            },
        ];
        reveal(crucible_yaml::resolve_tag::finalize_resolved_tag_property_spec);
        reveal(crucible_yaml::resolve_tag::effective_tag_code_point_limit_spec);
        reveal(crucible_yaml::resolve_tag::global_tag_uri_spec);
        reveal_with_fuel(crucible_yaml::resolve_tag::tag_uri_scheme_tail_spec, 5);
        assert(crucible_yaml::resolve_tag::finalize_resolved_tag_property_spec(
            invalid,
            0,
            0,
            crucible_yaml::resolve_tag::TagResolutionLimitsView { max_tag_code_points: 0 },
        ) == Err(
            crucible_yaml::resolve_tag::TagResolutionErrorView {
                kind: TagResolutionErrorKind::InvalidGlobalTagUri,
                byte_offset: 2,
            },
        ));

        let valid = seq![
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x21,
                source_atom_index: 9,
                byte_start: 9,
                byte_end: 10,
                origin: ResolvedTagOrigin::VerbatimPayload,
            },
            crucible_yaml::resolve_tag::ResolvedTagCodePointView {
                code_point: 0x78,
                source_atom_index: 10,
                byte_start: 10,
                byte_end: 11,
                origin: ResolvedTagOrigin::VerbatimPayload,
            },
        ];
        assert(crucible_yaml::resolve_tag::finalize_resolved_tag_property_spec(
            valid,
            4,
            7,
            crucible_yaml::resolve_tag::TagResolutionLimitsView { max_tag_code_points: 1 },
        ) == Err(
            crucible_yaml::resolve_tag::TagResolutionErrorView {
                kind: TagResolutionErrorKind::TagCodePointLimitExceeded,
                byte_offset: 10,
            },
        ));
    }
}

} // verus!
