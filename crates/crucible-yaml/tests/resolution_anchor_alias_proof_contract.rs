use vstd::prelude::*;

verus! {

#[test]
fn pure_latest_anchor_lookup_uses_exact_name_document_and_presentation_order() {
    proof {
        let atoms = seq![0x78u32, 0x79u32];
        let anchors = seq![
            crucible_yaml::resolve_anchor::AnchorDeclarationView {
                document_index: 0,
                node_index: 9,
                token_index: 2,
                name_start_atom_index: 0,
                name_end_atom_index: 1,
                name_byte_start: 4,
                name_byte_end: 5,
            },
            crucible_yaml::resolve_anchor::AnchorDeclarationView {
                document_index: 0,
                node_index: 11,
                token_index: 7,
                name_start_atom_index: 0,
                name_end_atom_index: 1,
                name_byte_start: 14,
                name_byte_end: 15,
            },
            crucible_yaml::resolve_anchor::AnchorDeclarationView {
                document_index: 1,
                node_index: 13,
                token_index: 9,
                name_start_atom_index: 0,
                name_end_atom_index: 1,
                name_byte_start: 18,
                name_byte_end: 19,
            },
        ];
        reveal_with_fuel(crucible_yaml::resolve_anchor::latest_matching_anchor_spec, 5);
        reveal(crucible_yaml::resolve_anchor::anchor_name_ranges_match_spec);
        assert(crucible_yaml::resolve_anchor::latest_matching_anchor_spec(
            atoms,
            anchors,
            0,
            8,
            0,
            1,
            3,
        ) == Some(1));
        assert(crucible_yaml::resolve_anchor::latest_matching_anchor_spec(
            atoms,
            anchors,
            1,
            9,
            0,
            1,
            3,
        ) == None);
    }
}

#[test]
fn pure_anchor_and_alias_limits_are_independently_lowerable() {
    proof {
        reveal(crucible_yaml::resolve_anchor::effective_anchor_limit_spec);
        reveal(crucible_yaml::resolve_anchor::effective_alias_limit_spec);
        assert(crucible_yaml::resolve_anchor::effective_anchor_limit_spec(
            crucible_yaml::resolve_anchor::AnchorAliasLimitsView {
                max_anchors: 0,
                max_aliases: u64::MAX,
            },
        ) == 0);
        assert(crucible_yaml::resolve_anchor::effective_alias_limit_spec(
            crucible_yaml::resolve_anchor::AnchorAliasLimitsView {
                max_anchors: u64::MAX,
                max_aliases: 0,
            },
        ) == 0);
    }
}

} // verus!
