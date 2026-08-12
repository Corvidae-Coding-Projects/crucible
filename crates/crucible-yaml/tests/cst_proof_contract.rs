#![allow(unused_imports)]

use crucible_yaml::token::{CompletedTokenSourceView, CompletedTokenView};
use crucible_yaml::{
    CompletedTokenKind, CstDocumentView, CstMappingEntryView, CstNodeKind, CstNodeStyle,
    CstNodeView, CstSequenceEntryView, CstSourceView, CstSyntaxOwnerKind, CstSyntaxOwnerView,
    CstWarningKind, CstWarningView,
};
use vstd::prelude::*;

verus! {

#[test]
fn a_collection_cannot_launder_a_self_or_forward_child_reference() {
    proof {
        let collection = CstNodeView {
            kind: CstNodeKind::Sequence,
            style: CstNodeStyle::Flow,
            token_start: 0,
            token_end: 2,
            byte_start: 0,
            byte_end: 2,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: Some(0),
            collection_end_token: Some(1),
            entry_start: 0,
            entry_end: 1,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let self_entry = CstSequenceEntryView {
            node_index: 0,
            token_start: 1,
            token_end: 1,
            indicator_token: None,
        };
        let nodes = Seq::empty().push(collection);
        let entries = Seq::empty().push(self_entry);
        assert(nodes.len() == 1);
        assert(entries.len() == 1);
        assert(nodes[0].kind == CstNodeKind::Sequence);
        assert(nodes[0].entry_start == 0);
        assert(nodes[0].entry_end == 1);
        assert(entries[0].node_index == 0);
        assert(!(entries[0].node_index < 0));
        reveal(crucible_yaml::cst::cst_child_before_parent_spec);
        assert(!crucible_yaml::cst::cst_child_before_parent_spec(nodes, entries, Seq::empty()));
    }
}

#[test]
fn one_entry_cannot_be_owned_by_two_collection_ranges() {
    proof {
        let first = CstNodeView {
            kind: CstNodeKind::Sequence,
            style: CstNodeStyle::Flow,
            token_start: 0,
            token_end: 2,
            byte_start: 0,
            byte_end: 2,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: Some(0),
            collection_end_token: Some(1),
            entry_start: 0,
            entry_end: 1,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let second = CstNodeView {
            token_start: 2,
            token_end: 4,
            byte_start: 2,
            byte_end: 4,
            ..first
        };
        let nodes = Seq::empty().push(first).push(second);
        reveal_with_fuel(crucible_yaml::cst::cst_entry_table_partition_from_spec, 3);
        reveal(crucible_yaml::cst::cst_entry_tables_uniquely_owned_spec);
        assert(!crucible_yaml::cst::cst_entry_tables_uniquely_owned_spec(nodes, 1, 0));
    }
}

#[test]
fn property_substitution_and_forged_empty_anchor_cannot_satisfy_exact_node_identity() {
    proof {
        let token = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
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
        let tokens = Seq::empty().push(token);
        let forged_property = CstNodeView {
            kind: CstNodeKind::Scalar,
            style: CstNodeStyle::Plain,
            token_start: 0,
            token_end: 1,
            byte_start: 0,
            byte_end: 1,
            anchor_property_token: Some(0),
            tag_property_token: None,
            scalar_or_alias_token: Some(0),
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let forged_empty = CstNodeView {
            kind: CstNodeKind::Empty,
            style: CstNodeStyle::Empty,
            token_start: 1,
            token_end: 1,
            byte_start: 1,
            byte_end: 1,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: Some(1),
            empty_anchor_byte: Some(0),
        };
        reveal(crucible_yaml::cst::cst_node_token_identity_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        assert(!crucible_yaml::cst::cst_node_token_identity_spec(tokens, 1, forged_property));
        assert(!crucible_yaml::cst::cst_node_token_identity_spec(tokens, 1, forged_empty));
    }
}

#[test]
fn cross_document_warning_and_nonincreasing_roots_cannot_launder_directive_state() {
    proof {
        let reserved = CompletedTokenView {
            kind: CompletedTokenKind::ReservedDirective,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 5,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let marker = CompletedTokenView {
            kind: CompletedTokenKind::DirectivesEnd,
            start_line_number: 1,
            end_line_number: 1,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 5,
            byte_end: 8,
            ..reserved
        };
        let scalar = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_line_number: 2,
            end_line_number: 2,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 8,
            byte_end: 9,
            scalar_index: Some(0),
            ..reserved
        };
        let tokens = Seq::empty().push(reserved).push(marker).push(scalar);
        let first = CstDocumentView {
            token_start: 0,
            token_end: 2,
            byte_start: 0,
            byte_end: 8,
            prefix_token_start: 0,
            prefix_token_end: 0,
            directive_start: 0,
            directive_end: 1,
            explicit_start_token_start: 1,
            explicit_start_token_end: 2,
            root_token_start: 2,
            root_token_end: 2,
            explicit_end_token_start: 2,
            explicit_end_token_end: 2,
            suffix_token_start: 2,
            suffix_token_end: 2,
            root_node_index: 0,
            explicit_start_token: Some(1),
            explicit_end_token: None,
        };
        let second = CstDocumentView {
            token_start: 2,
            token_end: 3,
            byte_start: 8,
            byte_end: 9,
            prefix_token_start: 2,
            prefix_token_end: 2,
            directive_start: 2,
            directive_end: 2,
            explicit_start_token_start: 2,
            explicit_start_token_end: 2,
            root_token_start: 2,
            root_token_end: 3,
            explicit_end_token_start: 3,
            explicit_end_token_end: 3,
            suffix_token_start: 3,
            suffix_token_end: 3,
            root_node_index: 1,
            explicit_start_token: None,
            explicit_end_token: None,
        };
        let documents = Seq::empty().push(first).push(second);
        let placeholder = CstNodeView {
            kind: CstNodeKind::Scalar,
            style: CstNodeStyle::Plain,
            token_start: 0,
            token_end: 1,
            byte_start: 0,
            byte_end: 5,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: Some(0),
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let nodes = Seq::empty().push(placeholder).push(placeholder);
        let leaked = CstWarningView {
            kind: CstWarningKind::ReservedDirective,
            document_index: 1,
            token_index: 0,
            byte_offset: 0,
        };
        reveal(crucible_yaml::cst::cst_warning_record_spec);
        assert(!crucible_yaml::cst::cst_warning_record_spec(tokens, documents, leaked));
        reveal(crucible_yaml::cst::cst_warnings_ordered_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_warnings_ordered_from_spec, 3);
        assert(!crucible_yaml::cst::cst_warnings_ordered_spec(
            tokens,
            documents,
            Seq::empty().push(leaked),
        ));
        reveal(crucible_yaml::cst::cst_documents_and_warnings_ordered_spec);
        assert(!crucible_yaml::cst::cst_documents_and_warnings_ordered_spec(
            tokens,
            9,
            documents,
            nodes,
            Seq::empty().push(leaked),
        ));

        let repeated_root = CstDocumentView { root_node_index: 0, ..second };
        let repeated_documents = Seq::empty().push(first).push(repeated_root);
        reveal(crucible_yaml::cst::cst_documents_ordered_spec);
        assert(!crucible_yaml::cst::cst_documents_ordered_spec(
            tokens,
            9,
            repeated_documents,
            nodes,
        ));
    }
}

#[test]
fn duplicate_scalar_reference_and_unowned_syntax_cannot_launder_ownership() {
    proof {
        let token = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
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
        let node = CstNodeView {
            kind: CstNodeKind::Scalar,
            style: CstNodeStyle::Plain,
            token_start: 0,
            token_end: 1,
            byte_start: 0,
            byte_end: 1,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: Some(0),
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let owner = CstSyntaxOwnerView {
            token_index: 0,
            kind: CstSyntaxOwnerKind::NodeContent,
            record_index: 0,
        };
        let owners = Seq::empty().push(Some(owner));
        reveal(crucible_yaml::cst::cst_owner_at_spec);
        reveal(crucible_yaml::cst::cst_node_references_owned_spec);
        assert(crucible_yaml::cst::cst_node_references_owned_spec(node, owners, 0));
        assert(!crucible_yaml::cst::cst_node_references_owned_spec(node, owners, 1));

        let nodes = Seq::empty().push(node).push(node);
        reveal(crucible_yaml::cst::cst_references_have_exact_owners_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_node_references_owned_from_spec, 4);
        reveal_with_fuel(crucible_yaml::cst::cst_document_references_owned_from_spec, 2);
        reveal_with_fuel(crucible_yaml::cst::cst_sequence_references_owned_from_spec, 2);
        reveal_with_fuel(crucible_yaml::cst::cst_mapping_references_owned_from_spec, 2);
        assert(!crucible_yaml::cst::cst_references_have_exact_owners_spec(
            Seq::empty(),
            nodes,
            Seq::empty(),
            Seq::empty(),
            owners,
        ));

        reveal_with_fuel(crucible_yaml::cst::cst_owner_slots_valid_from_spec, 3);
        assert(!crucible_yaml::cst::cst_owner_slots_valid_from_spec(
            Seq::empty().push(token),
            Seq::empty(),
            Seq::empty().push(node),
            Seq::empty(),
            Seq::empty(),
            Seq::empty().push(None),
            0,
            2,
        ));
    }
}

#[test]
fn document_regions_and_root_range_cannot_be_forged() {
    proof {
        let token = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
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
        let node = CstNodeView {
            kind: CstNodeKind::Scalar,
            style: CstNodeStyle::Plain,
            token_start: 0,
            token_end: 1,
            byte_start: 0,
            byte_end: 1,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: Some(0),
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let valid = CstDocumentView {
            token_start: 0,
            token_end: 1,
            byte_start: 0,
            byte_end: 1,
            prefix_token_start: 0,
            prefix_token_end: 0,
            directive_start: 0,
            directive_end: 0,
            explicit_start_token_start: 0,
            explicit_start_token_end: 0,
            root_token_start: 0,
            root_token_end: 1,
            explicit_end_token_start: 1,
            explicit_end_token_end: 1,
            suffix_token_start: 1,
            suffix_token_end: 1,
            root_node_index: 0,
            explicit_start_token: None,
            explicit_end_token: None,
        };
        let forged_region = CstDocumentView { prefix_token_end: 1, ..valid };
        let forged_root = CstDocumentView { root_token_start: 1, ..valid };
        reveal(crucible_yaml::cst::cst_document_record_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        assert(crucible_yaml::cst::cst_document_record_spec(
            Seq::empty().push(token),
            1,
            Seq::empty().push(node),
            valid,
        ));
        assert(!crucible_yaml::cst::cst_document_record_spec(
            Seq::empty().push(token),
            1,
            Seq::empty().push(node),
            forged_region,
        ));
        assert(!crucible_yaml::cst::cst_document_record_spec(
            Seq::empty().push(token),
            1,
            Seq::empty().push(node),
            forged_root,
        ));
    }
}

#[test]
fn nonempty_two_document_block_and_flow_fixture_satisfies_public_semantics() {
    proof {
        let base = CompletedTokenView {
            kind: CompletedTokenKind::DirectivesEnd,
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
        let t1 = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: Some(0),
            ..base
        };
        let t2 = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            ..base
        };
        let t3 = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 3,
            end_atom_index: 4,
            byte_start: 3,
            byte_end: 4,
            scalar_index: Some(1),
            ..base
        };
        let t4 = CompletedTokenView {
            kind: CompletedTokenKind::DocumentEnd,
            start_atom_index: 4,
            end_atom_index: 5,
            byte_start: 4,
            byte_end: 5,
            ..base
        };
        let t5 = CompletedTokenView {
            start_atom_index: 5,
            end_atom_index: 6,
            byte_start: 5,
            byte_end: 6,
            ..base
        };
        let t6 = CompletedTokenView {
            kind: CompletedTokenKind::FlowSequenceStart,
            start_atom_index: 6,
            end_atom_index: 7,
            byte_start: 6,
            byte_end: 7,
            ..base
        };
        let t7 = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 7,
            end_atom_index: 8,
            byte_start: 7,
            byte_end: 8,
            scalar_index: Some(2),
            ..base
        };
        let t8 = CompletedTokenView {
            kind: CompletedTokenKind::FlowSequenceEnd,
            start_atom_index: 8,
            end_atom_index: 9,
            byte_start: 8,
            byte_end: 9,
            ..base
        };
        let token_views = Seq::empty().push(base).push(t1).push(t2).push(t3).push(t4).push(t5).push(
            t6,
        ).push(t7).push(t8);
        let token_source = CompletedTokenSourceView {
            profile_version: 1,
            input_transformation_version: 1,
            layout_transformation_version: 1,
            structural_transformation_version: 1,
            quoted_transformation_version: 1,
            plain_transformation_version: 1,
            block_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 9,
            bom_bytes: 0,
            input_atom_count: 9,
            maximum_flow_depth: 1,
            tokens: token_views,
        };
        let key = CstNodeView {
            kind: CstNodeKind::Scalar,
            style: CstNodeStyle::Plain,
            token_start: 1,
            token_end: 2,
            byte_start: 1,
            byte_end: 2,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: Some(1),
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        let value = CstNodeView {
            token_start: 3,
            token_end: 4,
            byte_start: 3,
            byte_end: 4,
            scalar_or_alias_token: Some(3),
            ..key
        };
        let mapping = CstNodeView {
            kind: CstNodeKind::Mapping,
            style: CstNodeStyle::Block,
            token_start: 1,
            token_end: 4,
            byte_start: 1,
            byte_end: 4,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 1,
            ..key
        };
        let item = CstNodeView {
            token_start: 7,
            token_end: 8,
            byte_start: 7,
            byte_end: 8,
            scalar_or_alias_token: Some(7),
            ..key
        };
        let sequence = CstNodeView {
            kind: CstNodeKind::Sequence,
            style: CstNodeStyle::Flow,
            token_start: 6,
            token_end: 9,
            byte_start: 6,
            byte_end: 9,
            scalar_or_alias_token: None,
            collection_start_token: Some(6),
            collection_end_token: Some(8),
            entry_start: 0,
            entry_end: 1,
            ..key
        };
        let nodes = Seq::empty().push(key).push(value).push(mapping).push(item).push(sequence);
        let mapping_entries = Seq::empty().push(
            CstMappingEntryView {
                key_node_index: 0,
                value_node_index: 1,
                token_start: 1,
                token_end: 4,
                explicit_key_token: None,
                mapping_value_token: Some(2),
            },
        );
        let sequence_entries = Seq::empty().push(
            CstSequenceEntryView {
                node_index: 3,
                token_start: 7,
                token_end: 8,
                indicator_token: None,
            },
        );
        let first_document = CstDocumentView {
            token_start: 0,
            token_end: 5,
            byte_start: 0,
            byte_end: 5,
            prefix_token_start: 0,
            prefix_token_end: 0,
            directive_start: 0,
            directive_end: 0,
            explicit_start_token_start: 0,
            explicit_start_token_end: 1,
            root_token_start: 1,
            root_token_end: 4,
            explicit_end_token_start: 4,
            explicit_end_token_end: 5,
            suffix_token_start: 5,
            suffix_token_end: 5,
            root_node_index: 2,
            explicit_start_token: Some(0),
            explicit_end_token: Some(4),
        };
        let second_document = CstDocumentView {
            token_start: 5,
            token_end: 9,
            byte_start: 5,
            byte_end: 9,
            prefix_token_start: 5,
            prefix_token_end: 5,
            directive_start: 5,
            directive_end: 5,
            explicit_start_token_start: 5,
            explicit_start_token_end: 6,
            root_token_start: 6,
            root_token_end: 9,
            explicit_end_token_start: 9,
            explicit_end_token_end: 9,
            suffix_token_start: 9,
            suffix_token_end: 9,
            root_node_index: 4,
            explicit_start_token: Some(5),
            explicit_end_token: None,
        };
        let syntax_owners = Seq::empty()
            .push(Some(CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::DocumentStartMarker,
                record_index: 0,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 1,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 0,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 2,
                kind: CstSyntaxOwnerKind::MappingEntryIndicator,
                record_index: 0,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 3,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 1,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 4,
                kind: CstSyntaxOwnerKind::DocumentEndMarker,
                record_index: 0,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 5,
                kind: CstSyntaxOwnerKind::DocumentStartMarker,
                record_index: 1,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 6,
                kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                record_index: 4,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 7,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 3,
            }))
            .push(Some(CstSyntaxOwnerView {
                token_index: 8,
                kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                record_index: 4,
            }));
        let source = CstSourceView {
            profile_version: 1,
            input_token_transformation_version: 1,
            transformation_version: 1,
            source_len_bytes: 9,
            input_token_count: 9,
            directive_count: 0,
            maximum_depth: 1,
            documents: Seq::empty().push(first_document).push(second_document),
            nodes,
            sequence_entries,
            mapping_entries,
            warnings: Seq::empty(),
            syntax_owners,
        };

        reveal(crucible_yaml::cst::cst_public_semantics_spec);
        reveal(crucible_yaml::cst::cst_source_respects_limits_spec);
        reveal(crucible_yaml::cst::cst_effective_limit_spec);
        reveal(crucible_yaml::cst::cst_child_before_parent_spec);
        reveal(crucible_yaml::cst::cst_entry_tables_uniquely_owned_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_entry_table_partition_from_spec, 7);
        reveal(crucible_yaml::cst::cst_nodes_have_exact_token_identity_spec);
        reveal(crucible_yaml::cst::cst_node_token_identity_spec);
        reveal(crucible_yaml::cst::cst_style_matches_token_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_entry_ranges_spec);
        reveal(crucible_yaml::cst::cst_documents_and_warnings_ordered_spec);
        reveal(crucible_yaml::cst::cst_documents_ordered_spec);
        reveal(crucible_yaml::cst::cst_document_record_spec);
        reveal(crucible_yaml::cst::cst_warnings_ordered_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_warnings_ordered_from_spec, 2);
        reveal(crucible_yaml::cst::cst_exact_syntax_ownership_spec);
        reveal(crucible_yaml::cst::cst_references_have_exact_owners_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_owner_slots_valid_from_spec, 11);
        reveal_with_fuel(crucible_yaml::cst::cst_document_references_owned_from_spec, 4);
        reveal_with_fuel(crucible_yaml::cst::cst_node_references_owned_from_spec, 7);
        reveal_with_fuel(crucible_yaml::cst::cst_sequence_references_owned_from_spec, 3);
        reveal_with_fuel(crucible_yaml::cst::cst_mapping_references_owned_from_spec, 3);
        reveal(crucible_yaml::cst::cst_syntax_owner_record_spec);
        reveal(crucible_yaml::cst::cst_document_references_owned_spec);
        reveal(crucible_yaml::cst::cst_node_references_owned_spec);
        reveal(crucible_yaml::cst::cst_sequence_entry_reference_owned_spec);
        reveal(crucible_yaml::cst::cst_mapping_entry_references_owned_spec);
        reveal(crucible_yaml::cst::cst_owner_at_spec);
        assert(crucible_yaml::cst::cst_public_semantics_spec(token_source, source));
    }
}

} // verus!
