#![allow(unused_imports)]

use crucible_yaml::atom::{LexicalAtomKind, LexicalAtomView};
use crucible_yaml::token::{
    CompletedTokenPartKind, CompletedTokenPartView, CompletedTokenSourceView, CompletedTokenView,
};
use crucible_yaml::utf8::{SourcePositionView, SourceSpanView};
use crucible_yaml::{
    CompletedTokenKind, CstDocumentView, CstMappingEntryView, CstNodeKind, CstNodeStyle,
    CstNodeView, CstSequenceEntryView, CstSourceView, CstSyntaxOwnerKind, CstSyntaxOwnerView,
    CstWarningKind, CstWarningView,
};
use vstd::prelude::*;

verus! {

#[test]
fn pure_parser_task_stack_initialization_and_pop_are_exact() {
    proof {
        let first = crucible_yaml::cst::cst_node_task_spec(1, 9, true, 4);
        let second = crucible_yaml::cst::cst_node_task_spec(3, 7, false, 3);
        reveal(crucible_yaml::cst::cst_initial_parse_tasks_spec);
        reveal(crucible_yaml::cst::cst_pop_parse_task_spec);
        let initial = crucible_yaml::cst::cst_initial_parse_tasks_spec(1, 9, true, 4);
        assert(initial == Seq::empty().push(first));
        assert(crucible_yaml::cst::cst_pop_parse_task_spec(initial.push(second)) == Some(
            (initial, second),
        ));
        assert(crucible_yaml::cst::cst_pop_parse_task_spec(initial) == Some((Seq::empty(), first)));
        assert(crucible_yaml::cst::cst_pop_parse_task_spec(Seq::empty()).is_none());
    }
}

#[test]
fn pure_parser_parent_frame_resumption_is_exact() {
    proof {
        let first = crucible_yaml::cst::cst_node_task_spec(1, 9, true, 4);
        let resumed = crucible_yaml::cst::ParseTaskView { state: 2, cursor: 5, ..first };
        let tasks = Seq::empty().push(first);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        assert(crucible_yaml::cst::cst_resume_parse_task_spec(tasks, resumed) == tasks.push(
            resumed,
        ));
        assert(crucible_yaml::cst::cst_resume_parse_task_spec(Seq::empty(), resumed) == seq![
            resumed,
        ]);
    }
}

#[test]
fn pure_parser_completed_child_channel_is_exact_and_consuming() {
    proof {
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 4, next_token: 9 };
        reveal(crucible_yaml::cst::cst_store_completed_node_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        let stored = crucible_yaml::cst::cst_store_completed_node_spec(child);
        assert(stored == Some(child));
        assert(crucible_yaml::cst::cst_take_completed_node_spec(stored) == Some((None, child)));
        assert(crucible_yaml::cst::cst_take_completed_node_spec(None).is_none());
    }
}

#[test]
fn pure_parser_terminal_state_is_exact_and_total() {
    proof {
        let parsed = crucible_yaml::cst::ParsedNodeView { node_index: 2, next_token: 7 };
        let internal = Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        );
        reveal(crucible_yaml::cst::cst_finish_parse_node_spec);
        assert(crucible_yaml::cst::cst_finish_parse_node_spec(Seq::empty(), Some(parsed), 1, 8, 3)
            == Ok(parsed));
        assert(crucible_yaml::cst::cst_finish_parse_node_spec(
            seq![crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4)],
            Some(parsed),
            1,
            8,
            3,
        ) == internal);
        assert(crucible_yaml::cst::cst_finish_parse_node_spec(Seq::empty(), None, 1, 8, 3)
            == internal);
        assert(crucible_yaml::cst::cst_finish_parse_node_spec(
            Seq::empty(),
            Some(crucible_yaml::cst::ParsedNodeView { next_token: 9, ..parsed }),
            1,
            8,
            3,
        ) == internal);
        assert(crucible_yaml::cst::cst_finish_parse_node_spec(
            Seq::empty(),
            Some(crucible_yaml::cst::ParsedNodeView { node_index: 3, ..parsed }),
            1,
            8,
            3,
        ) == internal);
    }
}

#[test]
fn pure_parser_fuel_initialization_and_consumption_are_exact() {
    proof {
        reveal(crucible_yaml::cst::cst_initial_parse_fuel_spec);
        reveal(crucible_yaml::cst::cst_consume_parse_fuel_spec);
        let initial = crucible_yaml::cst::cst_initial_parse_fuel_spec();
        assert(initial == crucible_yaml::cst::cst_initial_parse_fuel_spec());
        crucible_yaml::cst::lemma_consume_parse_fuel_strictly_decreases(2);
        assert(crucible_yaml::cst::cst_consume_parse_fuel_spec(2) == Ok(1u64));
        assert(crucible_yaml::cst::cst_consume_parse_fuel_spec(1) == Ok(0u64));
        assert(crucible_yaml::cst::cst_consume_parse_fuel_spec(0) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_frame_state_domains_and_updates_are_exact() {
    proof {
        let base = crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4);
        let flow = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            ..base
        };
        reveal(crucible_yaml::cst::cst_parse_task_state_is_valid_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        assert(crucible_yaml::cst::cst_parse_task_state_is_valid_spec(
            crucible_yaml::cst::ParseTaskKind::Node,
            0,
        ));
        assert(!crucible_yaml::cst::cst_parse_task_state_is_valid_spec(
            crucible_yaml::cst::ParseTaskKind::Node,
            1,
        ));
        assert(crucible_yaml::cst::cst_parse_task_state_is_valid_spec(
            crucible_yaml::cst::ParseTaskKind::FlowSequence,
            4,
        ));
        assert(!crucible_yaml::cst::cst_parse_task_state_is_valid_spec(
            crucible_yaml::cst::ParseTaskKind::FlowSequence,
            5,
        ));
        assert(crucible_yaml::cst::cst_parse_task_state_is_valid_spec(
            crucible_yaml::cst::ParseTaskKind::BlockMapping,
            6,
        ));
        let resumed = crucible_yaml::cst::cst_task_set_state_spec(flow, 4);
        assert(resumed.state == 4);
        assert(resumed.kind == flow.kind && resumed.cursor == flow.cursor);
        assert(resumed.pending_sequence == flow.pending_sequence);
    }
}

#[test]
fn pure_parser_frame_cursor_bounds_and_updates_are_exact() {
    proof {
        let base = crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4);
        let out_of_bounds = crucible_yaml::cst::ParseTaskView { cursor: 9, ..base };
        let invalid_range = crucible_yaml::cst::ParseTaskView { token_start: 9, ..base };
        reveal(crucible_yaml::cst::cst_parse_task_cursor_is_in_bounds_spec);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        assert(crucible_yaml::cst::cst_parse_task_cursor_is_in_bounds_spec(base));
        assert(!crucible_yaml::cst::cst_parse_task_cursor_is_in_bounds_spec(out_of_bounds));
        assert(!crucible_yaml::cst::cst_parse_task_cursor_is_in_bounds_spec(invalid_range));
        let advanced = crucible_yaml::cst::cst_task_set_cursor_spec(base, 6);
        assert(advanced.cursor == 6);
        assert(advanced.kind == base.kind && advanced.state == base.state);
        assert(advanced.end == base.end && advanced.pending_mapping == base.pending_mapping);
    }
}

#[test]
fn pure_parser_entry_scratch_updates_are_exact() {
    proof {
        let base = crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4);
        reveal(crucible_yaml::cst::cst_task_begin_keyed_entry_spec);
        reveal(crucible_yaml::cst::cst_task_begin_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_key_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_colon_spec);
        reveal(crucible_yaml::cst::cst_task_set_entry_end_spec);
        reveal(crucible_yaml::cst::cst_task_extend_node_end_spec);
        let keyed = crucible_yaml::cst::cst_task_begin_keyed_entry_spec(base, 3, true);
        assert(keyed.entry_token_start == 3 && keyed.explicit_key);
        assert(keyed.kind == base.kind && keyed.cursor == base.cursor);
        let keyed = crucible_yaml::cst::cst_task_set_key_node_spec(keyed, 7);
        let keyed = crucible_yaml::cst::cst_task_set_colon_spec(keyed, 4);
        let keyed = crucible_yaml::cst::cst_task_set_entry_end_spec(keyed, 6);
        assert(keyed.key_node_index == 7 && keyed.colon_token == 4);
        assert(keyed.entry_token_start == 3 && keyed.entry_token_end == 6);
        let extended = crucible_yaml::cst::cst_task_extend_node_end_spec(keyed, 6);
        assert(extended.node_token_end == 6);
        assert(crucible_yaml::cst::cst_task_extend_node_end_spec(extended, 4) == extended);
        let sequence = crucible_yaml::cst::cst_task_begin_sequence_entry_spec(base, 2);
        assert(sequence.entry_token_start == 2 && sequence.entry_token_end == 3);
        assert(sequence.pending_sequence == base.pending_sequence);
    }
}

#[test]
fn pure_parser_stack_rejects_invalid_frames_and_preserves_valid_ones() {
    proof {
        let base = crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4);
        let invalid_state = crucible_yaml::cst::ParseTaskView { state: 1, ..base };
        let invalid_cursor = crucible_yaml::cst::ParseTaskView { cursor: 9, ..base };
        reveal(crucible_yaml::cst::cst_parse_task_is_valid_spec);
        reveal(crucible_yaml::cst::cst_parse_task_stack_is_valid_spec);
        assert(crucible_yaml::cst::cst_parse_task_is_valid_spec(base));
        assert(!crucible_yaml::cst::cst_parse_task_is_valid_spec(invalid_state));
        assert(!crucible_yaml::cst::cst_parse_task_is_valid_spec(invalid_cursor));
        let initial = crucible_yaml::cst::cst_initial_parse_tasks_spec(1, 8, true, 4);
        crucible_yaml::cst::lemma_cst_initial_parse_tasks_are_valid(1, 8, true, 4);
        assert(crucible_yaml::cst::cst_parse_task_stack_is_valid_spec(initial));
        let resumed = crucible_yaml::cst::cst_resume_parse_task_spec(initial, base);
        crucible_yaml::cst::lemma_cst_resume_preserves_valid_stack(initial, base);
        assert(crucible_yaml::cst::cst_parse_task_stack_is_valid_spec(resumed));
        crucible_yaml::cst::lemma_cst_pop_preserves_valid_stack(resumed);
        let popped = crucible_yaml::cst::cst_pop_parse_task_spec(resumed).unwrap();
        assert(crucible_yaml::cst::cst_parse_task_stack_is_valid_spec(popped.0));
        assert(crucible_yaml::cst::cst_parse_task_is_valid_spec(popped.1));
        let forged = seq![invalid_state];
        assert(forged[0] == invalid_state);
        assert(!crucible_yaml::cst::cst_parse_task_is_valid_spec(forged[0]));
        assert(!crucible_yaml::cst::cst_parse_task_stack_is_valid_spec(forged));
    }
}

#[test]
fn pure_parser_machine_initialization_is_exact() {
    proof {
        reveal(crucible_yaml::cst::cst_initial_parse_machine_spec);
        let machine = crucible_yaml::cst::cst_initial_parse_machine_spec(1, 8, true, 4);
        assert(machine.tasks == crucible_yaml::cst::cst_initial_parse_tasks_spec(1, 8, true, 4));
        assert(machine.completed.is_none());
        assert(machine.fuel == crucible_yaml::cst::cst_initial_parse_fuel_spec());
        crucible_yaml::cst::lemma_cst_initial_parse_tasks_are_valid(1, 8, true, 4);
        assert(crucible_yaml::cst::cst_parse_task_stack_is_valid_spec(machine.tasks));
    }
}

#[test]
fn pure_parser_machine_channels_and_fuel_are_exact() {
    proof {
        let initial = crucible_yaml::cst::cst_initial_parse_machine_spec(1, 8, true, 4);
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 3, next_token: 6 };
        reveal(crucible_yaml::cst::cst_machine_store_completed_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_machine_pop_task_spec);
        reveal(crucible_yaml::cst::cst_machine_consume_fuel_spec);
        let stored = crucible_yaml::cst::cst_machine_store_completed_spec(initial, child);
        assert(stored.completed == Some(child));
        assert(stored.tasks == initial.tasks && stored.fuel == initial.fuel);
        let taken = crucible_yaml::cst::cst_machine_take_completed_spec(stored).unwrap();
        assert(taken.0.completed.is_none() && taken.1 == child);
        assert(taken.0.tasks == initial.tasks && taken.0.fuel == initial.fuel);
        let task = crucible_yaml::cst::cst_node_task_spec(2, 7, false, 3);
        let resumed = crucible_yaml::cst::cst_machine_resume_task_spec(taken.0, task);
        assert(resumed.tasks == taken.0.tasks.push(task));
        assert(resumed.completed == taken.0.completed && resumed.fuel == taken.0.fuel);
        let popped = crucible_yaml::cst::cst_machine_pop_task_spec(resumed).unwrap();
        assert(popped.0.tasks == taken.0.tasks && popped.1 == task);
        assert(popped.0.completed == resumed.completed && popped.0.fuel == resumed.fuel);
        let two = crucible_yaml::cst::ParseMachineView { fuel: 2, ..popped.0 };
        match crucible_yaml::cst::cst_machine_consume_fuel_spec(two) {
            Ok(one) => {
                assert(one.fuel == 1 && one.tasks == two.tasks && one.completed == two.completed);
            },
            Err(_) => assert(false),
        }
    }
}

#[test]
fn pure_parser_machine_dispatch_and_terminal_result_are_total() {
    proof {
        let initial = crucible_yaml::cst::cst_initial_parse_machine_spec(1, 8, true, 4);
        let ready = crucible_yaml::cst::ParseMachineView { fuel: 2, ..initial };
        reveal(crucible_yaml::cst::cst_machine_begin_step_spec);
        reveal(crucible_yaml::cst::cst_machine_finish_spec);
        match crucible_yaml::cst::cst_machine_begin_step_spec(ready) {
            Ok((next, task)) => {
                assert(task == crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4));
                assert(next.tasks.len() == 0 && next.completed.is_none() && next.fuel == 1);
            },
            Err(_) => assert(false),
        }
        let exhausted = crucible_yaml::cst::ParseMachineView { fuel: 0, ..initial };
        assert(crucible_yaml::cst::cst_machine_begin_step_spec(exhausted) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
        let parsed = crucible_yaml::cst::ParsedNodeView { node_index: 2, next_token: 7 };
        let terminal = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(parsed),
            fuel: 1,
        };
        assert(crucible_yaml::cst::cst_machine_finish_spec(terminal, 1, 8, 3) == Ok(parsed));
    }
}

#[test]
fn pure_parser_dispatched_frame_bounds_are_exact() {
    proof {
        let valid = crucible_yaml::cst::cst_node_task_spec(1, 8, true, 4);
        let end_past_tokens = crucible_yaml::cst::ParseTaskView { end: 9, ..valid };
        let cursor_past_end = crucible_yaml::cst::ParseTaskView { cursor: 9, ..valid };
        let start_past_end = crucible_yaml::cst::ParseTaskView { token_start: 9, ..valid };
        let internal = Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        );
        reveal(crucible_yaml::cst::cst_validate_dispatched_task_spec);
        assert(crucible_yaml::cst::cst_validate_dispatched_task_spec(valid, 8) == Ok(()));
        assert(crucible_yaml::cst::cst_validate_dispatched_task_spec(end_past_tokens, 8)
            == internal);
        assert(crucible_yaml::cst::cst_validate_dispatched_task_spec(cursor_past_end, 9)
            == internal);
        assert(crucible_yaml::cst::cst_validate_dispatched_task_spec(start_past_end, 9)
            == internal);
    }
}

#[test]
fn pure_parser_node_frame_empty_transition_is_exact_and_total() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(0, limits, 0);
        let task = crucible_yaml::cst::cst_node_task_spec(0, 0, false, 4);
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_node_store_empty_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_empty_node_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_machine_store_completed_spec);
        reveal(crucible_yaml::cst::cst_store_completed_node_spec);
        let result = crucible_yaml::cst::cst_step_node_task_spec(
            Seq::empty(),
            Seq::empty(),
            task,
            machine,
            builder,
            4,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder.nodes.len() == 1);
        assert(next_builder.nodes[0].kind == CstNodeKind::Empty);
        assert(next_machine.completed == Some(
            crucible_yaml::cst::ParsedNodeView { node_index: 0, next_token: 0 },
        ));
        let invalid = crucible_yaml::cst::ParseTaskView { end: 1, ..task };
        assert(crucible_yaml::cst::cst_step_node_task_spec(
            Seq::empty(),
            Seq::empty(),
            invalid,
            machine,
            builder,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_sequence_state_zero_transition_is_exact_and_total() {
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            state: 0,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_sequence_state_zero_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_begin_keyed_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        reveal(crucible_yaml::cst::cst_machine_push_task_spec);
        reveal(crucible_yaml::cst::cst_push_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_sequence_state_zero_spec(
            seq![token],
            task,
            machine,
            builder,
            4,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.tasks.len() == 2);
        assert(next_machine.tasks[0].kind == crucible_yaml::cst::ParseTaskKind::FlowSequence);
        assert(next_machine.tasks[0].state == 1);
        assert(next_machine.tasks[1] == crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4));
        let invalid = crucible_yaml::cst::ParseTaskView { state: 1, ..task };
        assert(crucible_yaml::cst::cst_step_flow_sequence_state_zero_spec(
            seq![token],
            invalid,
            machine,
            builder,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_mapping_state_zero_transition_is_exact_and_total() {
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowMapping,
            state: 0,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_state_zero_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_begin_keyed_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        reveal(crucible_yaml::cst::cst_machine_push_task_spec);
        reveal(crucible_yaml::cst::cst_push_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_mapping_state_zero_spec(
            seq![token],
            task,
            machine,
            builder,
            4,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.tasks.len() == 2);
        assert(next_machine.tasks[0].kind == crucible_yaml::cst::ParseTaskKind::FlowMapping);
        assert(next_machine.tasks[0].state == 1);
        assert(next_machine.tasks[1] == crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4));
        let invalid = crucible_yaml::cst::ParseTaskView { state: 1, ..task };
        assert(crucible_yaml::cst::cst_step_flow_mapping_state_zero_spec(
            seq![token],
            invalid,
            machine,
            builder,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_sequence_completed_key_transition_is_exact_and_total() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(0, limits, 0);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 0, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            state: 1,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 0, next_token: 0 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_sequence_state_one_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_key_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_sequence_state_one_spec(
            Seq::empty(),
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].kind == crucible_yaml::cst::ParseTaskKind::FlowSequence);
        assert(next_machine.tasks[0].state == 2);
        assert(next_machine.tasks[0].key_node_index == 0);
        assert(next_machine.tasks[0].cursor == 0);
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_flow_sequence_state_one_spec(
            Seq::empty(),
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_mapping_completed_key_transition_is_exact_and_total() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(0, limits, 0);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 0, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowMapping,
            state: 1,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 0, next_token: 0 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_state_one_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_key_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_mapping_state_one_spec(
            Seq::empty(),
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].kind == crucible_yaml::cst::ParseTaskKind::FlowMapping);
        assert(next_machine.tasks[0].state == 2);
        assert(next_machine.tasks[0].key_node_index == 0);
        assert(next_machine.tasks[0].cursor == 0);
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_flow_mapping_state_one_spec(
            Seq::empty(),
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_mapping_key_decision_transition_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 2,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let empty = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let key_node = CstNodeView {
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
        let builder = crucible_yaml::cst::CstBuilderView {
            nodes: seq![key_node],
            syntax_owner_slots: seq![Some(CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 0,
            })],
            ..empty
        };
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowMapping,
            state: 2,
            cursor: 1,
            key_node_index: 0,
            entry_token_start: 0,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_state_two_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_finish_entry_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_empty_node_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_task_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_mapping_state_two_spec(
            seq![key],
            task,
            machine,
            builder,
            4,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder.nodes.len() == 2);
        assert(next_builder.nodes[1].kind == CstNodeKind::Empty);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 4);
        assert(next_machine.tasks[0].pending_mapping.len() == 1);
        assert(next_machine.tasks[0].pending_mapping[0].key_node_index == 0);
        assert(next_machine.tasks[0].pending_mapping[0].value_node_index == 1);
        assert(next_machine.tasks[0].pending_mapping[0].token_start == 0);
        assert(next_machine.tasks[0].pending_mapping[0].token_end == 1);
        assert(next_machine.tasks[0].pending_mapping[0].mapping_value_token.is_none());
        let invalid = crucible_yaml::cst::ParseTaskView { state: 1, ..task };
        assert(crucible_yaml::cst::cst_step_flow_mapping_state_two_spec(
            seq![key],
            invalid,
            machine,
            builder,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_mapping_completed_value_transition_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: None,
            ..key
        };
        let value = CompletedTokenView {
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            scalar_index: Some(1),
            ..key
        };
        let tokens = seq![key, colon, value];
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 2,
            max_sequence_entries: 1,
            max_mapping_entries: 2,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(3, limits, 3);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 3, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowMapping,
            state: 3,
            cursor: 2,
            key_node_index: 0,
            entry_token_start: 0,
            colon_token: 1,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 1, next_token: 3 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_state_three_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 4);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_finish_entry_spec);
        reveal(crucible_yaml::cst::cst_task_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_mapping_state_three_spec(
            tokens,
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 4);
        assert(next_machine.tasks[0].cursor == 3);
        assert(next_machine.tasks[0].pending_mapping.len() == 1);
        assert(next_machine.tasks[0].pending_mapping[0].key_node_index == 0);
        assert(next_machine.tasks[0].pending_mapping[0].value_node_index == 1);
        assert(next_machine.tasks[0].pending_mapping[0].token_start == 0);
        assert(next_machine.tasks[0].pending_mapping[0].token_end == 3);
        assert(next_machine.tasks[0].pending_mapping[0].mapping_value_token == Some(1));
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_flow_mapping_state_three_spec(
            tokens,
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_sequence_key_decision_transition_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 2,
            max_sequence_entries: 2,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let empty = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let key_node = CstNodeView {
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
        let builder = crucible_yaml::cst::CstBuilderView {
            nodes: seq![key_node],
            syntax_owner_slots: seq![Some(CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 0,
            })],
            ..empty
        };
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            state: 2,
            cursor: 1,
            key_node_index: 0,
            entry_token_start: 0,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_sequence_state_two_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_step_flow_sequence_finish_entry_spec);
        reveal(crucible_yaml::cst::cst_task_push_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_sequence_state_two_spec(
            seq![key],
            task,
            machine,
            builder,
            4,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 4);
        assert(next_machine.tasks[0].pending_sequence.len() == 1);
        assert(next_machine.tasks[0].pending_sequence[0].node_index == 0);
        assert(next_machine.tasks[0].pending_sequence[0].token_start == 0);
        assert(next_machine.tasks[0].pending_sequence[0].token_end == 1);
        let invalid = crucible_yaml::cst::ParseTaskView { state: 1, ..task };
        assert(crucible_yaml::cst::cst_step_flow_sequence_state_two_spec(
            seq![key],
            invalid,
            machine,
            builder,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_sequence_completed_value_transition_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: None,
            ..key
        };
        let value = CompletedTokenView {
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            scalar_index: Some(1),
            ..key
        };
        let tokens = seq![key, colon, value];
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 4,
            max_sequence_entries: 2,
            max_mapping_entries: 2,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let empty = crucible_yaml::cst::cst_empty_builder_spec(3, limits, 3);
        let key_node = CstNodeView {
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
        let value_node = CstNodeView {
            token_start: 2,
            token_end: 3,
            byte_start: 2,
            byte_end: 3,
            scalar_or_alias_token: Some(2),
            ..key_node
        };
        let builder = crucible_yaml::cst::CstBuilderView {
            nodes: seq![key_node, value_node],
            syntax_owner_slots: seq![
                Some(CstSyntaxOwnerView {
                    token_index: 0,
                    kind: CstSyntaxOwnerKind::NodeContent,
                    record_index: 0,
                }),
                None,
                Some(CstSyntaxOwnerView {
                    token_index: 2,
                    kind: CstSyntaxOwnerKind::NodeContent,
                    record_index: 1,
                }),
            ],
            ..empty
        };
        let base = crucible_yaml::cst::cst_node_task_spec(0, 3, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            state: 3,
            cursor: 2,
            key_node_index: 0,
            entry_token_start: 0,
            colon_token: 1,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 1, next_token: 3 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_sequence_state_three_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_single_pair_mapping_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_claim_mapping_entry_references_spec);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_task_push_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_sequence_state_three_spec(
            tokens,
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder.nodes.len() == 3);
        assert(next_builder.nodes[2].kind == CstNodeKind::Mapping);
        assert(next_builder.mapping_entries.len() == 1);
        assert(next_builder.mapping_entries[0].key_node_index == 0);
        assert(next_builder.mapping_entries[0].value_node_index == 1);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 4);
        assert(next_machine.tasks[0].cursor == 3);
        assert(next_machine.tasks[0].pending_sequence.len() == 1);
        assert(next_machine.tasks[0].pending_sequence[0].node_index == 2);
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_flow_sequence_state_three_spec(
            tokens,
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_sequence_delimiter_transition_is_exact_and_total() {
    proof {
        let comma = CompletedTokenView {
            kind: CompletedTokenKind::FlowEntry,
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
        let scalar = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: Some(0),
            ..comma
        };
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(2, limits, 2);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 2, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            state: 4,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_sequence_state_four_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_task_push_flow_entry_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 3);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_sequence_state_four_spec(
            seq![comma, scalar],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 0);
        assert(next_machine.tasks[0].cursor == 1);
        assert(next_machine.tasks[0].flow_entry_tokens == seq![0u64]);
        assert(crucible_yaml::cst::cst_step_flow_sequence_state_four_spec(
            seq![scalar, scalar],
            task,
            machine,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::MissingFlowEntry,
                byte_offset: 1,
            },
        ));
    }
}

#[test]
fn pure_parser_flow_mapping_delimiter_transition_is_exact_and_total() {
    proof {
        let comma = CompletedTokenView {
            kind: CompletedTokenKind::FlowEntry,
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
        let scalar = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: Some(0),
            ..comma
        };
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(2, limits, 2);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 2, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowMapping,
            state: 4,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_flow_mapping_state_four_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_task_push_flow_entry_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 3);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_flow_mapping_state_four_spec(
            seq![comma, scalar],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 0);
        assert(next_machine.tasks[0].cursor == 1);
        assert(next_machine.tasks[0].flow_entry_tokens == seq![0u64]);
        assert(crucible_yaml::cst::cst_step_flow_mapping_state_four_spec(
            seq![scalar, scalar],
            task,
            machine,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::MissingFlowEntry,
                byte_offset: 1,
            },
        ));
        let invalid = crucible_yaml::cst::ParseTaskView { state: 3, ..task };
        assert(crucible_yaml::cst::cst_step_flow_mapping_state_four_spec(
            seq![comma, scalar],
            invalid,
            machine,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_block_sequence_entry_transition_is_exact_and_total() {
    proof {
        let position = SourcePositionView { byte_offset: 0, line: 0, column: 0 };
        let dash_atom = LexicalAtomView {
            kind: LexicalAtomKind::Indicator(crucible_yaml::YamlIndicator::BlockSequenceEntry),
            code_point: 0x2d,
            span: SourceSpanView { start: position, end: position },
        };
        let dash = CompletedTokenView {
            kind: CompletedTokenKind::BlockSequenceEntry,
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::BlockSequence,
            state: 0,
            indentation: 0,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_block_sequence_state_zero_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 3);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_token_column_spec);
        reveal(crucible_yaml::cst::cst_task_begin_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_empty_node_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_task_push_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_block_sequence_state_zero_spec(
            seq![dash_atom],
            seq![dash],
            task,
            machine,
            builder,
            4,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder.nodes.len() == 1);
        assert(next_builder.nodes[0].kind == CstNodeKind::Empty);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 2);
        assert(next_machine.tasks[0].pending_sequence.len() == 1);
        assert(next_machine.tasks[0].pending_sequence[0].node_index == 0);
        assert(next_machine.tasks[0].pending_sequence[0].token_start == 0);
        assert(next_machine.tasks[0].pending_sequence[0].token_end == 1);
        assert(next_machine.tasks[0].pending_sequence[0].indicator_token == Some(0));
        let invalid = crucible_yaml::cst::ParseTaskView { state: 1, ..task };
        assert(crucible_yaml::cst::cst_step_block_sequence_state_zero_spec(
            seq![dash_atom],
            seq![dash],
            invalid,
            machine,
            builder,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_block_sequence_completed_child_transition_is_exact_and_total() {
    proof {
        let dash = CompletedTokenView {
            kind: CompletedTokenKind::BlockSequenceEntry,
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::BlockSequence,
            state: 1,
            entry_token_start: 0,
            entry_token_end: 1,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 0, next_token: 1 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_block_sequence_state_one_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_entry_end_spec);
        reveal(crucible_yaml::cst::cst_task_extend_node_end_spec);
        reveal(crucible_yaml::cst::cst_task_push_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_block_sequence_state_one_spec(
            seq![dash],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 2);
        assert(next_machine.tasks[0].cursor == 1);
        assert(next_machine.tasks[0].node_token_end == 1);
        assert(next_machine.tasks[0].pending_sequence.len() == 1);
        assert(next_machine.tasks[0].pending_sequence[0].node_index == 0);
        assert(next_machine.tasks[0].pending_sequence[0].token_start == 0);
        assert(next_machine.tasks[0].pending_sequence[0].token_end == 1);
        assert(next_machine.tasks[0].pending_sequence[0].indicator_token == Some(0));
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_block_sequence_state_one_spec(
            seq![dash],
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_block_sequence_continuation_transition_is_exact_and_total() {
    proof {
        let dash = CompletedTokenView {
            kind: CompletedTokenKind::BlockSequenceEntry,
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::BlockSequence,
            state: 2,
            cursor: 1,
            indentation: 0,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_block_sequence_state_two_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 2);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_block_sequence_state_two_spec(
            Seq::empty(),
            seq![dash],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 0);
        assert(next_machine.tasks[0].cursor == 1);
        let invalid = crucible_yaml::cst::ParseTaskView { state: 1, ..task };
        assert(crucible_yaml::cst::cst_step_block_sequence_state_two_spec(
            Seq::empty(),
            seq![dash],
            invalid,
            machine,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_block_mapping_completed_key_transition_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: None,
            ..key
        };
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(2, limits, 2);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 2, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::BlockMapping,
            state: 1,
            colon_token: 1,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 0, next_token: 1 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_block_mapping_state_one_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_key_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_block_mapping_state_one_spec(
            seq![key, colon],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 2);
        assert(next_machine.tasks[0].cursor == 0);
        assert(next_machine.tasks[0].key_node_index == 0);
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_block_mapping_state_one_spec(
            seq![key, colon],
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_block_mapping_unpaired_explicit_key_handoff_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::BlockMapping,
            state: 4,
            explicit_key: true,
            ..base
        };
        let child = crucible_yaml::cst::ParsedNodeView { node_index: 0, next_token: 1 };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: Some(child),
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_block_mapping_state_four_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_machine_take_completed_spec);
        reveal(crucible_yaml::cst::cst_take_completed_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_cursor_spec);
        reveal(crucible_yaml::cst::cst_task_set_key_node_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_block_mapping_state_four_spec(
            seq![key],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder == builder);
        assert(next_machine.completed.is_none());
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 5);
        assert(next_machine.tasks[0].cursor == 1);
        assert(next_machine.tasks[0].key_node_index == 0);
        let missing = crucible_yaml::cst::ParseMachineView { completed: None, ..machine };
        assert(crucible_yaml::cst::cst_step_block_mapping_state_four_spec(
            seq![key],
            task,
            missing,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_block_mapping_unpaired_explicit_key_entry_is_exact_and_total() {
    proof {
        let key = CompletedTokenView {
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
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 2,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let empty = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 1);
        let key_node = CstNodeView {
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
        let builder = crucible_yaml::cst::CstBuilderView {
            nodes: seq![key_node],
            syntax_owner_slots: seq![Some(CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 0,
            })],
            ..empty
        };
        let base = crucible_yaml::cst::cst_node_task_spec(0, 1, false, 4);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::BlockMapping,
            state: 5,
            cursor: 1,
            entry_token_start: 0,
            key_node_index: 0,
            explicit_key: true,
            ..base
        };
        let machine = crucible_yaml::cst::ParseMachineView {
            tasks: Seq::empty(),
            completed: None,
            fuel: 1,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_step_block_mapping_state_five_spec);
        reveal(crucible_yaml::cst::cst_step_node_internal_error_spec);
        reveal(crucible_yaml::cst::cst_empty_node_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_task_set_entry_end_spec);
        reveal(crucible_yaml::cst::cst_task_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_task_set_state_spec);
        reveal(crucible_yaml::cst::cst_machine_resume_task_spec);
        reveal(crucible_yaml::cst::cst_resume_parse_task_spec);
        let result = crucible_yaml::cst::cst_step_block_mapping_state_five_spec(
            seq![key],
            task,
            machine,
            builder,
        );
        assert(result.is_ok());
        let (next_machine, next_builder) = match result {
            Ok(value) => value,
            Err(_) => (machine, builder),
        };
        assert(next_builder.nodes.len() == 2);
        assert(next_builder.nodes[1].kind == CstNodeKind::Empty);
        assert(next_machine.tasks.len() == 1);
        assert(next_machine.tasks[0].state == 6);
        assert(next_machine.tasks[0].entry_token_end == 1);
        assert(next_machine.tasks[0].pending_mapping.len() == 1);
        assert(next_machine.tasks[0].pending_mapping[0].key_node_index == 0);
        assert(next_machine.tasks[0].pending_mapping[0].value_node_index == 1);
        assert(next_machine.tasks[0].pending_mapping[0].token_start == 0);
        assert(next_machine.tasks[0].pending_mapping[0].token_end == 1);
        assert(next_machine.tasks[0].pending_mapping[0].explicit_key_token == Some(0));
        assert(next_machine.tasks[0].pending_mapping[0].mapping_value_token.is_none());
        let invalid = crucible_yaml::cst::ParseTaskView { state: 4, ..task };
        assert(crucible_yaml::cst::cst_step_block_mapping_state_five_spec(
            seq![key],
            invalid,
            machine,
            builder,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_parser_pending_table_appends_are_exact() {
    proof {
        let task = crucible_yaml::cst::cst_node_task_spec(1, 8, true, 3);
        let sequence = CstSequenceEntryView {
            node_index: 0,
            token_start: 1,
            token_end: 2,
            indicator_token: Some(1),
        };
        let mapping = CstMappingEntryView {
            key_node_index: 0,
            value_node_index: 1,
            token_start: 2,
            token_end: 5,
            explicit_key_token: Some(2),
            mapping_value_token: Some(4),
        };
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_task_push_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_task_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_task_push_flow_entry_spec);
        let after_sequence = crucible_yaml::cst::cst_task_push_sequence_entry_spec(task, sequence);
        assert(after_sequence.pending_sequence == Seq::empty().push(sequence));
        assert(after_sequence.pending_mapping.len() == 0);
        let after_mapping = crucible_yaml::cst::cst_task_push_mapping_entry_spec(
            after_sequence,
            mapping,
        );
        assert(after_mapping.pending_mapping == Seq::empty().push(mapping));
        let after_flow = crucible_yaml::cst::cst_task_push_flow_entry_spec(after_mapping, 6);
        assert(after_flow.flow_entry_tokens == Seq::empty().push(6));
        assert(after_flow.token_start == task.token_start);
        assert(after_flow.cursor == task.cursor && after_flow.end == task.end);
        assert(after_flow.depth_left == task.depth_left);
    }
}

#[test]
fn pure_collection_frame_specialization_is_exact() {
    proof {
        let base = crucible_yaml::cst::cst_node_task_spec(2, 9, true, 4);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_specialize_parse_task_spec);
        let result = crucible_yaml::cst::cst_specialize_parse_task_spec(
            base,
            crucible_yaml::cst::ParseTaskKind::FlowMapping,
            2,
            4,
            3,
            7,
            Some(2),
            Some(3),
            5,
            11,
        );
        assert(result.is_ok());
        let specialized = match result {
            Ok(task) => task,
            Err(_) => base,
        };
        assert(specialized.kind == crucible_yaml::cst::ParseTaskKind::FlowMapping);
        assert(specialized.state == 0);
        assert(specialized.token_start == 2 && specialized.cursor == 4);
        assert(specialized.opener == 3 && specialized.indentation == 7);
        assert(specialized.depth_left == 3);
        assert(specialized.anchor_property_token == Some(2));
        assert(specialized.tag_property_token == Some(3));
        assert(specialized.node_token_end == 5);
        assert(specialized.end == 9 && specialized.allow_block_mapping);
        let exhausted = crucible_yaml::cst::ParseTaskView { depth_left: 0, ..base };
        assert(crucible_yaml::cst::cst_specialize_parse_task_spec(
            exhausted,
            crucible_yaml::cst::ParseTaskKind::BlockSequence,
            2,
            3,
            3,
            2,
            None,
            None,
            4,
            17,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::DepthLimitExceeded,
                byte_offset: 17,
            },
        ));
    }
}

#[test]
fn pure_node_property_scan_is_exact() {
    proof {
        let anchor = CompletedTokenView {
            kind: CompletedTokenKind::AnchorProperty,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 2,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let separation = CompletedTokenView {
            kind: CompletedTokenKind::Separation,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 2,
            byte_end: 3,
            ..anchor
        };
        let tag = CompletedTokenView {
            kind: CompletedTokenKind::TagProperty,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 3,
            byte_end: 5,
            ..anchor
        };
        let scalar = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 3,
            end_atom_index: 4,
            byte_start: 6,
            byte_end: 7,
            scalar_index: Some(0),
            ..anchor
        };
        let tokens = Seq::empty().push(anchor).push(separation).push(tag).push(separation).push(
            scalar,
        );
        reveal_with_fuel(crucible_yaml::cst::cst_scan_node_properties_from_spec, 8);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 8);
        reveal(crucible_yaml::cst::cst_token_is_trivia_spec);
        let result = crucible_yaml::cst::cst_scan_node_properties_from_spec(
            tokens,
            0,
            5,
            None,
            None,
            6,
        );
        assert(result == Ok(
            crucible_yaml::cst::NodePropertiesView {
                next_token: 4,
                anchor_property_token: Some(0),
                tag_property_token: Some(2),
            },
        ));
        let duplicate = Seq::empty().push(anchor).push(separation).push(
            CompletedTokenView { byte_start: 3, byte_end: 5, ..anchor },
        );
        assert(crucible_yaml::cst::cst_scan_node_properties_from_spec(
            duplicate,
            0,
            3,
            None,
            None,
            4,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::DuplicateAnchorProperty,
                byte_offset: 3,
            },
        ));
        let unseparated = Seq::empty().push(anchor).push(scalar);
        assert(crucible_yaml::cst::cst_scan_node_properties_from_spec(
            unseparated,
            0,
            2,
            None,
            None,
            3,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::MissingPropertySeparation,
                byte_offset: 6,
            },
        ));
    }
}

#[test]
fn pure_leaf_node_constructors_are_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let property = CompletedTokenView {
            kind: CompletedTokenKind::AnchorProperty,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 2,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let scalar = CompletedTokenView {
            kind: CompletedTokenKind::DoubleQuotedScalar,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 3,
            byte_end: 6,
            scalar_index: Some(0),
            ..property
        };
        let property_tokens = Seq::empty().push(property);
        let scalar_tokens = property_tokens.push(scalar);
        let property_builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 2);
        let scalar_builder = crucible_yaml::cst::cst_empty_builder_spec(2, limits, 6);
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_finish_property_empty_node_spec);
        reveal(crucible_yaml::cst::cst_finish_alias_node_spec);
        reveal(crucible_yaml::cst::cst_finish_scalar_node_spec);
        reveal(crucible_yaml::cst::cst_scalar_style_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        let property_result = crucible_yaml::cst::cst_finish_property_empty_node_spec(
            property_builder,
            property_tokens,
            0,
            1,
            Some(0),
            None,
        );
        assert(property_result.is_ok());
        let (property_built, property_parsed) = match property_result {
            Ok(value) => value,
            Err(_) => (
                property_builder,
                crucible_yaml::cst::ParsedNodeView { node_index: 99, next_token: 99 },
            ),
        };
        let property_node = property_built.nodes[0];
        assert(property_parsed.node_index == 0 && property_parsed.next_token == 1);
        assert(property_node.kind == CstNodeKind::Empty);
        assert(property_node.anchor_property_token == Some(0));
        assert(property_node.empty_anchor_token == Some(1));
        assert(property_node.empty_anchor_byte == Some(2));
        let scalar_result = crucible_yaml::cst::cst_finish_scalar_node_spec(
            scalar_builder,
            scalar_tokens,
            0,
            1,
            Some(0),
            None,
        );
        assert(scalar_result.is_ok());
        let (scalar_built, scalar_parsed) = match scalar_result {
            Ok(value) => value,
            Err(_) => (
                scalar_builder,
                crucible_yaml::cst::ParsedNodeView { node_index: 99, next_token: 99 },
            ),
        };
        let scalar_node = scalar_built.nodes[0];
        assert(scalar_parsed.node_index == 0 && scalar_parsed.next_token == 2);
        assert(scalar_node.kind == CstNodeKind::Scalar);
        assert(scalar_node.style == CstNodeStyle::DoubleQuoted);
        assert(scalar_node.token_start == 0 && scalar_node.token_end == 2);
        assert(scalar_node.byte_start == 0 && scalar_node.byte_end == 6);
        assert(scalar_node.anchor_property_token == Some(0));
        assert(scalar_node.scalar_or_alias_token == Some(1));
        let alias = CompletedTokenView {
            kind: CompletedTokenKind::Alias,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 4,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let alias_tokens = Seq::empty().push(alias);
        let alias_builder = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 4);
        let alias_result = crucible_yaml::cst::cst_finish_alias_node_spec(
            alias_builder,
            alias_tokens,
            0,
            0,
        );
        assert(alias_result.is_ok());
        let (alias_built, alias_parsed) = match alias_result {
            Ok(value) => value,
            Err(_) => (
                alias_builder,
                crucible_yaml::cst::ParsedNodeView { node_index: 99, next_token: 99 },
            ),
        };
        let alias_node = alias_built.nodes[0];
        assert(alias_parsed.node_index == 0 && alias_parsed.next_token == 1);
        assert(alias_node.kind == CstNodeKind::Alias && alias_node.style == CstNodeStyle::Alias);
        assert(alias_node.scalar_or_alias_token == Some(0));
        assert(alias_node.anchor_property_token.is_none()
            && alias_node.tag_property_token.is_none());
    }
}

#[test]
fn pure_compact_single_pair_mapping_is_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 3,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let key_token = CompletedTokenView {
            kind: CompletedTokenKind::ExplicitMappingKey,
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
        let value_token = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            ..key_token
        };
        let tokens = Seq::empty().push(key_token).push(value_token);
        let child0 = CstNodeView {
            kind: CstNodeKind::Empty,
            style: CstNodeStyle::Empty,
            token_start: 0,
            token_end: 0,
            byte_start: 0,
            byte_end: 0,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: Some(0),
            empty_anchor_byte: Some(0),
        };
        let child1 = CstNodeView {
            token_start: 1,
            token_end: 1,
            byte_start: 1,
            byte_end: 1,
            empty_anchor_token: Some(1),
            empty_anchor_byte: Some(1),
            ..child0
        };
        let empty_builder = crucible_yaml::cst::cst_empty_builder_spec(2, limits, 2);
        let initial = crucible_yaml::cst::CstBuilderView {
            nodes: Seq::empty().push(child0).push(child1),
            ..empty_builder
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_single_pair_mapping_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_claim_mapping_entry_references_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        let result = crucible_yaml::cst::cst_single_pair_mapping_spec(
            initial,
            tokens,
            0,
            1,
            0,
            2,
            Some(0),
            Some(1),
        );
        assert(result.is_ok());
        let (built, node_index) = match result {
            Ok(value) => value,
            Err(_) => (initial, 99),
        };
        assert(node_index == 2);
        assert(built.mapping_entries.len() == 1 && built.nodes.len() == 3);
        let entry = built.mapping_entries[0];
        assert(entry.key_node_index == 0 && entry.value_node_index == 1);
        assert(entry.explicit_key_token == Some(0) && entry.mapping_value_token == Some(1));
        let node = built.nodes[2];
        assert(node.kind == CstNodeKind::Mapping && node.style == CstNodeStyle::FlowPair);
        assert(node.token_start == 0 && node.token_end == 2);
        assert(node.byte_start == 0 && node.byte_end == 2);
        assert(node.entry_start == 0 && node.entry_end == 1);
        assert(crucible_yaml::cst::cst_single_pair_mapping_spec(
            built,
            tokens,
            0,
            1,
            0,
            2,
            Some(0),
            Some(1),
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::MappingEntryLimitExceeded,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_flow_mapping_completion_is_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 3,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let token0 = CompletedTokenView {
            kind: CompletedTokenKind::FlowMappingStart,
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
        let token1 = CompletedTokenView {
            kind: CompletedTokenKind::ExplicitMappingKey,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            ..token0
        };
        let token2 = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            ..token0
        };
        let token3 = CompletedTokenView {
            kind: CompletedTokenKind::FlowEntry,
            start_atom_index: 3,
            end_atom_index: 4,
            byte_start: 3,
            byte_end: 4,
            ..token0
        };
        let token4 = CompletedTokenView {
            kind: CompletedTokenKind::FlowMappingEnd,
            start_atom_index: 4,
            end_atom_index: 5,
            byte_start: 4,
            byte_end: 5,
            ..token0
        };
        let tokens = Seq::empty().push(token0).push(token1).push(token2).push(token3).push(token4);
        let empty0 = CstNodeView {
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
            empty_anchor_byte: Some(1),
        };
        let empty1 = CstNodeView {
            token_start: 2,
            token_end: 2,
            byte_start: 2,
            byte_end: 2,
            empty_anchor_token: Some(2),
            empty_anchor_byte: Some(2),
            ..empty0
        };
        let empty_builder = crucible_yaml::cst::cst_empty_builder_spec(5, limits, 5);
        let initial = crucible_yaml::cst::CstBuilderView {
            nodes: Seq::empty().push(empty0).push(empty1),
            ..empty_builder
        };
        let mapping_entry = CstMappingEntryView {
            key_node_index: 0,
            value_node_index: 1,
            token_start: 1,
            token_end: 3,
            explicit_key_token: Some(1),
            mapping_value_token: Some(2),
        };
        let base = crucible_yaml::cst::cst_node_task_spec(0, 5, false, 1);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowMapping,
            cursor: 5,
            opener: 0,
            pending_mapping: Seq::empty().push(mapping_entry),
            flow_entry_tokens: Seq::empty().push(3),
            ..base
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_finish_iterative_mapping_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_push_mapping_entries_from_spec, 3);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_flow_entries_from_spec, 3);
        reveal(crucible_yaml::cst::cst_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_claim_mapping_entry_references_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        let result = crucible_yaml::cst::cst_finish_iterative_mapping_spec(
            tokens,
            task,
            Some(4),
            initial,
        );
        assert(result.is_ok());
        let (built, parsed) = match result {
            Ok(value) => value,
            Err(_) => (
                initial,
                crucible_yaml::cst::ParsedNodeView { node_index: 99, next_token: 99 },
            ),
        };
        assert(parsed.node_index == 2 && parsed.next_token == 5);
        assert(built.mapping_entries == Seq::empty().push(mapping_entry));
        assert(built.nodes.len() == 3);
        let node = built.nodes[2];
        assert(node.kind == CstNodeKind::Mapping && node.style == CstNodeStyle::Flow);
        assert(node.token_start == 0 && node.token_end == 5);
        assert(node.byte_start == 0 && node.byte_end == 5);
        assert(node.collection_start_token == Some(0));
        assert(node.collection_end_token == Some(4));
        assert(node.entry_start == 0 && node.entry_end == 1);
        assert(built.syntax_owner_slots[0] == Some(
            CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                record_index: 2,
            },
        ));
        assert(built.syntax_owner_slots[1] == Some(
            CstSyntaxOwnerView {
                token_index: 1,
                kind: CstSyntaxOwnerKind::MappingEntryIndicator,
                record_index: 0,
            },
        ));
        assert(built.syntax_owner_slots[2] == Some(
            CstSyntaxOwnerView {
                token_index: 2,
                kind: CstSyntaxOwnerKind::MappingEntryIndicator,
                record_index: 0,
            },
        ));
        assert(built.syntax_owner_slots[3] == Some(
            CstSyntaxOwnerView {
                token_index: 3,
                kind: CstSyntaxOwnerKind::FlowEntryIndicator,
                record_index: 2,
            },
        ));
        assert(built.syntax_owner_slots[4] == Some(
            CstSyntaxOwnerView {
                token_index: 4,
                kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                record_index: 2,
            },
        ));
        assert(crucible_yaml::cst::cst_finish_iterative_mapping_spec(tokens, task, Some(5), initial)
            == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_flow_sequence_completion_is_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let token0 = CompletedTokenView {
            kind: CompletedTokenKind::FlowSequenceStart,
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
        let token1 = CompletedTokenView {
            kind: CompletedTokenKind::FlowEntry,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            ..token0
        };
        let token2 = CompletedTokenView {
            kind: CompletedTokenKind::FlowSequenceEnd,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            ..token0
        };
        let tokens = Seq::empty().push(token0).push(token1).push(token2);
        let initial = crucible_yaml::cst::cst_empty_builder_spec(3, limits, 3);
        let base = crucible_yaml::cst::cst_node_task_spec(0, 3, false, 1);
        let task = crucible_yaml::cst::ParseTaskView {
            kind: crucible_yaml::cst::ParseTaskKind::FlowSequence,
            cursor: 3,
            opener: 0,
            flow_entry_tokens: Seq::empty().push(1),
            ..base
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_finish_iterative_sequence_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_push_sequence_entries_from_spec, 2);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_flow_entries_from_spec, 3);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        let result = crucible_yaml::cst::cst_finish_iterative_sequence_spec(
            tokens,
            task,
            Some(2),
            initial,
        );
        assert(result.is_ok());
        let (built, parsed) = match result {
            Ok(value) => value,
            Err(_) => (
                initial,
                crucible_yaml::cst::ParsedNodeView { node_index: 99, next_token: 99 },
            ),
        };
        assert(parsed.node_index == 0 && parsed.next_token == 3);
        assert(built.nodes.len() == 1);
        let node = built.nodes[0];
        assert(node.kind == CstNodeKind::Sequence && node.style == CstNodeStyle::Flow);
        assert(node.token_start == 0 && node.token_end == 3);
        assert(node.byte_start == 0 && node.byte_end == 3);
        assert(node.collection_start_token == Some(0));
        assert(node.collection_end_token == Some(2));
        assert(built.syntax_owner_slots[0] == Some(
            CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                record_index: 0,
            },
        ));
        assert(built.syntax_owner_slots[1] == Some(
            CstSyntaxOwnerView {
                token_index: 1,
                kind: CstSyntaxOwnerKind::FlowEntryIndicator,
                record_index: 0,
            },
        ));
        assert(built.syntax_owner_slots[2] == Some(
            CstSyntaxOwnerView {
                token_index: 2,
                kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                record_index: 0,
            },
        ));
        assert(crucible_yaml::cst::cst_finish_iterative_sequence_spec(
            tokens,
            task,
            Some(3),
            initial,
        ) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_empty_node_construction_is_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let initial = crucible_yaml::cst::cst_empty_builder_spec(0, limits, 13);
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_empty_node_spec);
        reveal(crucible_yaml::cst::cst_byte_at_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        let result = crucible_yaml::cst::cst_empty_node_spec(initial, Seq::empty(), 0);
        assert(result.is_ok());
        let (built, node_index) = match result {
            Ok(value) => value,
            Err(_) => (initial, 99),
        };
        assert(node_index == 0);
        assert(built.nodes.len() == 1);
        let node = built.nodes[0];
        assert(node.kind == CstNodeKind::Empty && node.style == CstNodeStyle::Empty);
        assert(node.token_start == 0 && node.token_end == 0);
        assert(node.byte_start == 13 && node.byte_end == 13);
        assert(node.empty_anchor_token == Some(0) && node.empty_anchor_byte == Some(13));
        assert(node.anchor_property_token.is_none() && node.tag_property_token.is_none());
        assert(node.scalar_or_alias_token.is_none());
        assert(node.collection_start_token.is_none() && node.collection_end_token.is_none());
        assert(node.entry_start == 0 && node.entry_end == 0);
        assert(crucible_yaml::cst::cst_empty_node_spec(built, Seq::empty(), 0) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::NodeLimitExceeded,
                byte_offset: 13,
            },
        ));
    }
}

#[test]
fn pure_parser_frame_initialization_and_bounded_push_are_exact() {
    proof {
        let task = crucible_yaml::cst::cst_node_task_spec(2, 7, true, 4);
        reveal(crucible_yaml::cst::cst_node_task_spec);
        reveal(crucible_yaml::cst::cst_push_parse_task_spec);
        assert(task.kind == crucible_yaml::cst::ParseTaskKind::Node);
        assert(task.token_start == 2 && task.cursor == 2 && task.end == 7 && task.opener == 2);
        assert(task.depth_left == 4 && task.allow_block_mapping);
        assert(task.pending_sequence.len() == 0);
        assert(task.pending_mapping.len() == 0);
        assert(task.flow_entry_tokens.len() == 0);
        let pushed = crucible_yaml::cst::cst_push_parse_task_spec(Seq::empty(), task, 0, 9);
        assert(pushed == Ok(Seq::empty().push(task)));
        let full = Seq::empty().push(task).push(task);
        assert(crucible_yaml::cst::cst_push_parse_task_spec(full, task, 0, 11) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::DepthLimitExceeded,
                byte_offset: 11,
            },
        ));
    }
}

#[test]
fn pure_builder_directive_and_depth_metadata_are_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 4,
        };
        let initial = crucible_yaml::cst::cst_empty_builder_spec(0, limits, 9);
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_record_directive_spec);
        reveal(crucible_yaml::cst::cst_observe_depth_spec);
        let recorded = crucible_yaml::cst::cst_record_directive_spec(initial, 3);
        assert(recorded.is_ok());
        let after_directive = match recorded {
            Ok(builder) => builder,
            Err(_) => initial,
        };
        assert(after_directive.directive_count == 1);
        assert(crucible_yaml::cst::cst_record_directive_spec(after_directive, 7) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::DirectiveLimitExceeded,
                byte_offset: 7,
            },
        ));
        let deeper = crucible_yaml::cst::cst_observe_depth_spec(after_directive, 3);
        assert(deeper.maximum_depth == 3);
        let shallower = crucible_yaml::cst::cst_observe_depth_spec(deeper, 2);
        assert(shallower == deeper);
    }
}

#[test]
fn pure_directive_and_tag_handle_helpers_are_exact() {
    proof {
        let position = SourcePositionView { byte_offset: 0, line: 0, column: 0 };
        let span = SourceSpanView { start: position, end: position };
        let bang = LexicalAtomView {
            kind: LexicalAtomKind::Indicator(crucible_yaml::atom::YamlIndicator::Tag),
            code_point: 0x21,
            span,
        };
        let letter = LexicalAtomView { kind: LexicalAtomKind::Content, code_point: 0x65, span };
        let atoms = Seq::empty().push(bang).push(bang).push(bang).push(letter).push(bang);
        let parameter = CompletedTokenPartView {
            kind: CompletedTokenPartKind::DirectiveParameter,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 0,
            byte_end: 1,
        };
        let named = CompletedTokenPartView {
            kind: CompletedTokenPartKind::TagHandle,
            start_atom_index: 2,
            end_atom_index: 5,
            byte_start: 2,
            byte_end: 5,
        };
        let token = CompletedTokenView {
            kind: CompletedTokenKind::TagProperty,
            start_line_number: 0,
            end_line_number: 0,
            start_atom_index: 2,
            end_atom_index: 5,
            byte_start: 2,
            byte_end: 5,
            scalar_index: None,
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty().push(parameter).push(named),
        };
        reveal(crucible_yaml::cst::cst_part_of_kind_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_part_of_kind_from_spec, 4);
        reveal(crucible_yaml::cst::cst_atom_ranges_equal_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_atom_ranges_equal_from_spec, 8);
        reveal(crucible_yaml::cst::cst_tag_handle_is_default_spec);
        reveal(crucible_yaml::cst::cst_first_undeclared_tag_handle_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_first_undeclared_tag_handle_from_spec, 3);
        reveal(crucible_yaml::cst::cst_tag_property_handle_is_undeclared_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_tag_handle_declared_from_spec, 8);
        assert(crucible_yaml::cst::cst_part_of_kind_spec(token, CompletedTokenPartKind::TagHandle)
            == Some(named));
        assert(crucible_yaml::cst::cst_atom_ranges_equal_spec(atoms, 0, 1, 1, 2));
        let primary = CompletedTokenPartView { start_atom_index: 0, end_atom_index: 1, ..named };
        let secondary = CompletedTokenPartView { start_atom_index: 0, end_atom_index: 2, ..named };
        let forged_suffix = CompletedTokenPartView {
            start_atom_index: 3,
            end_atom_index: 5,
            ..named
        };
        assert(crucible_yaml::cst::cst_tag_handle_is_default_spec(atoms, primary));
        assert(crucible_yaml::cst::cst_tag_handle_is_default_spec(atoms, secondary));
        assert(!crucible_yaml::cst::cst_tag_handle_is_default_spec(atoms, named));
        assert(!crucible_yaml::cst::cst_tag_handle_is_default_spec(atoms, forged_suffix));
        let tokens = Seq::empty().push(token);
        assert(crucible_yaml::cst::cst_first_undeclared_tag_handle_spec(
            atoms,
            tokens,
            0,
            1,
            Seq::empty(),
        ) == Some(0));
        assert(crucible_yaml::cst::cst_first_undeclared_tag_handle_spec(
            atoms,
            tokens,
            0,
            1,
            Seq::empty().push((2u64, 5u64)),
        ) == None);
    }
}

#[test]
fn pure_builder_entry_warning_and_document_appends_are_exact() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let initial = crucible_yaml::cst::cst_empty_builder_spec(3, limits, 12);
        let sequence = CstSequenceEntryView {
            node_index: 0,
            token_start: 0,
            token_end: 1,
            indicator_token: Some(0),
        };
        let mapping = CstMappingEntryView {
            key_node_index: 0,
            value_node_index: 0,
            token_start: 1,
            token_end: 3,
            explicit_key_token: Some(1),
            mapping_value_token: Some(2),
        };
        let warning = CstWarningView {
            kind: CstWarningKind::ReservedDirective,
            document_index: 0,
            token_index: 0,
            byte_offset: 0,
        };
        let document = CstDocumentView {
            token_start: 0,
            token_end: 3,
            byte_start: 0,
            byte_end: 12,
            prefix_token_start: 0,
            prefix_token_end: 0,
            directive_start: 0,
            directive_end: 0,
            explicit_start_token_start: 0,
            explicit_start_token_end: 0,
            root_token_start: 0,
            root_token_end: 3,
            explicit_end_token_start: 3,
            explicit_end_token_end: 3,
            suffix_token_start: 3,
            suffix_token_end: 3,
            root_node_index: 0,
            explicit_start_token: None,
            explicit_end_token: None,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_push_sequence_entry_spec);
        reveal(crucible_yaml::cst::cst_push_mapping_entry_spec);
        reveal(crucible_yaml::cst::cst_claim_mapping_entry_references_spec);
        reveal(crucible_yaml::cst::cst_push_warning_spec);
        reveal(crucible_yaml::cst::cst_push_document_spec);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        let after_sequence_result = crucible_yaml::cst::cst_push_sequence_entry_spec(
            initial,
            sequence,
            0,
        );
        let after_sequence = match after_sequence_result {
            Ok(builder) => builder,
            Err(_) => initial,
        };
        assert(after_sequence_result.is_ok());
        assert(after_sequence.sequence_entries == Seq::empty().push(sequence));
        assert(after_sequence.syntax_owner_slots[0].is_some());
        let after_mapping_result = crucible_yaml::cst::cst_push_mapping_entry_spec(
            after_sequence,
            mapping,
            4,
        );
        let after_mapping = match after_mapping_result {
            Ok(builder) => builder,
            Err(_) => after_sequence,
        };
        assert(after_mapping_result.is_ok());
        assert(after_mapping.mapping_entries == Seq::empty().push(mapping));
        assert(after_mapping.syntax_owner_slots[1].is_some());
        assert(after_mapping.syntax_owner_slots[2].is_some());
        let after_warning = match crucible_yaml::cst::cst_push_warning_spec(
            after_mapping,
            warning,
        ) {
            Ok(builder) => builder,
            Err(_) => after_mapping,
        };
        let after_document = match crucible_yaml::cst::cst_push_document_spec(
            after_warning,
            document,
        ) {
            Ok(builder) => builder,
            Err(_) => after_warning,
        };
        assert(after_document.warnings == Seq::empty().push(warning));
        assert(after_document.documents == Seq::empty().push(document));
        assert(crucible_yaml::cst::cst_push_warning_spec(after_document, warning) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::WarningLimitExceeded,
                byte_offset: 0,
            },
        ));
        assert(crucible_yaml::cst::cst_push_document_spec(after_document, document) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::DocumentLimitExceeded,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_builder_node_append_claims_every_reference_and_honors_the_first_limit() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let initial = crucible_yaml::cst::cst_empty_builder_spec(2, limits, 9);
        let node = CstNodeView {
            kind: CstNodeKind::Scalar,
            style: CstNodeStyle::Plain,
            token_start: 0,
            token_end: 2,
            byte_start: 0,
            byte_end: 9,
            anchor_property_token: Some(0),
            tag_property_token: None,
            scalar_or_alias_token: Some(1),
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        };
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_push_node_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_claim_node_references_spec, 7);
        reveal(crucible_yaml::cst::cst_claim_optional_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        let appended_result = crucible_yaml::cst::cst_push_node_spec(initial, node, 0);
        let appended = match appended_result {
            Ok(result) => result.0,
            Err(_) => initial,
        };
        assert(appended_result.is_ok());
        assert(appended.nodes == Seq::empty().push(node));
        assert(appended.syntax_owner_slots[0] == Some(
            CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeProperty,
                record_index: 0,
            },
        ));
        assert(appended.syntax_owner_slots[1] == Some(
            CstSyntaxOwnerView {
                token_index: 1,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 0,
            },
        ));
        assert(crucible_yaml::cst::cst_push_node_spec(appended, node, 9) == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::NodeLimitExceeded,
                byte_offset: 9,
            },
        ));
    }
}

#[test]
fn pure_builder_claim_transition_is_exact_and_rejects_duplicate_ownership() {
    proof {
        let limits = crucible_yaml::CstLimitsView {
            max_documents: 1,
            max_nodes: 1,
            max_sequence_entries: 1,
            max_mapping_entries: 1,
            max_directives: 1,
            max_warnings: 1,
            max_depth: 1,
        };
        let initial = crucible_yaml::cst::cst_empty_builder_spec(1, limits, 7);
        reveal(crucible_yaml::cst::cst_empty_builder_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_token_spec);
        reveal(crucible_yaml::cst::cst_claim_syntax_owner_slots_spec);
        assert(initial.syntax_owner_slots.len() == 1);
        assert(initial.syntax_owner_slots[0].is_none());
        let claimed_result = crucible_yaml::cst::cst_claim_syntax_token_spec(
            initial,
            0,
            CstSyntaxOwnerKind::NodeContent,
            2,
        );
        let claimed = match claimed_result {
            Ok(builder) => builder,
            Err(_) => initial,
        };
        assert(claimed_result.is_ok());
        assert(claimed.syntax_owner_slots[0] == Some(
            CstSyntaxOwnerView {
                token_index: 0,
                kind: CstSyntaxOwnerKind::NodeContent,
                record_index: 2,
            },
        ));
        let duplicate = crucible_yaml::cst::cst_claim_syntax_token_spec(
            claimed,
            0,
            CstSyntaxOwnerKind::NodeProperty,
            3,
        );
        assert(duplicate == Err(
            crucible_yaml::CstErrorView {
                kind: crucible_yaml::CstErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        ));
    }
}

#[test]
fn pure_block_property_boundary_and_collection_permission_are_exact() {
    proof {
        let line_zero = SourcePositionView { byte_offset: 0, line: 0, column: 2 };
        let line_one = SourcePositionView { byte_offset: 4, line: 1, column: 2 };
        let ordinary_atom = LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
            code_point: 0x61,
            span: SourceSpanView { start: line_zero, end: line_zero },
        };
        let next_line_atom = LexicalAtomView {
            span: SourceSpanView { start: line_one, end: line_one },
            ..ordinary_atom
        };
        let colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
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
        let property = CompletedTokenView {
            kind: CompletedTokenKind::AnchorProperty,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            ..colon
        };
        let line_feed = CompletedTokenView {
            kind: CompletedTokenKind::LineFeed,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            ..colon
        };
        let next_node = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_line_number: 1,
            end_line_number: 1,
            start_atom_index: 3,
            end_atom_index: 4,
            byte_start: 4,
            byte_end: 5,
            scalar_index: Some(0),
            ..colon
        };
        let atoms = Seq::empty().push(ordinary_atom).push(ordinary_atom).push(ordinary_atom).push(
            next_line_atom,
        );
        let tokens = Seq::empty().push(colon).push(property).push(line_feed).push(next_node);
        reveal(crucible_yaml::cst::cst_block_property_only_end_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_block_property_only_end_from_spec, 5);
        reveal(crucible_yaml::cst::cst_block_value_allows_collection_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_block_value_allows_collection_from_spec, 5);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 5);
        reveal(crucible_yaml::cst::cst_token_column_spec);
        assert(crucible_yaml::cst::cst_block_property_only_end_spec(atoms, tokens, 1, 4, 2, 4)
            == Some(2));
        assert(crucible_yaml::cst::cst_block_value_allows_collection_spec(tokens, 1, 4, 0, 4));
    }
}

#[test]
fn pure_explicit_mapping_lookahead_uses_exact_cross_line_indentation() {
    proof {
        let first_position = SourcePositionView { byte_offset: 0, line: 0, column: 2 };
        let second_position = SourcePositionView { byte_offset: 4, line: 1, column: 4 };
        let first_atom = LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
            code_point: 0x61,
            span: SourceSpanView { start: first_position, end: first_position },
        };
        let second_atom = LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Indicator(
                crucible_yaml::YamlIndicator::MappingValue,
            ),
            code_point: 0x3a,
            span: SourceSpanView { start: second_position, end: second_position },
        };
        let key = CompletedTokenView {
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
        let colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_line_number: 1,
            end_line_number: 1,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 4,
            byte_end: 5,
            scalar_index: None,
            ..key
        };
        let atoms = Seq::empty().push(first_atom).push(second_atom);
        let tokens = Seq::empty().push(key).push(colon);
        reveal(crucible_yaml::cst::cst_find_explicit_mapping_value_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_find_explicit_mapping_value_from_spec, 4);
        reveal(crucible_yaml::cst::cst_find_mapping_value_on_line_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_find_mapping_value_on_line_from_spec, 4);
        reveal(crucible_yaml::cst::cst_token_column_spec);
        assert(crucible_yaml::cst::cst_find_explicit_mapping_value_spec(atoms, tokens, 0, 2, 4, 3)
            == Some(1));
        assert(crucible_yaml::cst::cst_find_explicit_mapping_value_spec(
            atoms,
            tokens,
            0,
            2,
            3,
            3,
        ).is_none());
    }
}

#[test]
fn pure_mapping_value_lookahead_skips_nested_flow_indicators() {
    proof {
        let base = CompletedTokenView {
            kind: CompletedTokenKind::FlowMappingStart,
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
        let nested_colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            ..base
        };
        let flow_end = CompletedTokenView {
            kind: CompletedTokenKind::FlowMappingEnd,
            start_atom_index: 2,
            end_atom_index: 3,
            byte_start: 2,
            byte_end: 3,
            ..base
        };
        let top_level_colon = CompletedTokenView {
            kind: CompletedTokenKind::MappingValue,
            start_atom_index: 3,
            end_atom_index: 4,
            byte_start: 3,
            byte_end: 4,
            ..base
        };
        let tokens = Seq::empty().push(base).push(nested_colon).push(flow_end).push(
            top_level_colon,
        );
        reveal(crucible_yaml::cst::cst_find_mapping_value_on_line_spec);
        reveal_with_fuel(crucible_yaml::cst::cst_find_mapping_value_on_line_from_spec, 6);
        assert(crucible_yaml::cst::cst_find_mapping_value_on_line_spec(tokens, 0, 4, 5) == Some(3));
    }
}

#[test]
fn pure_position_helpers_fix_exact_byte_line_and_column_results() {
    proof {
        let position = SourcePositionView { byte_offset: 4, line: 2, column: 3 };
        let atom = LexicalAtomView {
            kind: crucible_yaml::LexicalAtomKind::Content,
            code_point: 0x61,
            span: SourceSpanView { start: position, end: position },
        };
        let token = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_line_number: 2,
            end_line_number: 2,
            start_atom_index: 0,
            end_atom_index: 1,
            byte_start: 4,
            byte_end: 5,
            scalar_index: Some(0),
            yaml_major: None,
            yaml_minor: None,
            parts: Seq::empty(),
        };
        let other_line = CompletedTokenView { start_line_number: 3, ..token };
        let tokens = Seq::empty().push(token);
        let atoms = Seq::empty().push(atom);
        reveal(crucible_yaml::cst::cst_same_line_spec);
        reveal(crucible_yaml::cst::cst_token_column_spec);
        assert(crucible_yaml::cst::cst_same_line_spec(token, token));
        assert(!crucible_yaml::cst::cst_same_line_spec(token, other_line));
        assert(crucible_yaml::cst::cst_byte_at_spec(tokens, 0, 9) == 4);
        assert(crucible_yaml::cst::cst_byte_at_spec(tokens, 1, 9) == 9);
        assert(crucible_yaml::cst::cst_token_column_spec(atoms, token) == 3);
        assert(crucible_yaml::cst::cst_token_column_spec(Seq::empty(), token) == 0);
    }
}

#[test]
fn pure_parser_helpers_fix_scalar_classification_and_trivia_progress() {
    proof {
        let trivia = CompletedTokenView {
            kind: CompletedTokenKind::Separation,
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
        let scalar = CompletedTokenView {
            kind: CompletedTokenKind::PlainScalar,
            start_atom_index: 1,
            end_atom_index: 2,
            byte_start: 1,
            byte_end: 2,
            scalar_index: Some(0),
            ..trivia
        };
        let tokens = Seq::empty().push(trivia).push(scalar);
        reveal_with_fuel(crucible_yaml::cst::cst_skip_trivia_spec, 4);
        assert(crucible_yaml::cst::cst_skip_trivia_spec(tokens, 0, 2, 3) == 1);
        reveal(crucible_yaml::cst::cst_token_is_scalar_spec);
        reveal(crucible_yaml::cst::cst_scalar_style_spec);
        assert(crucible_yaml::cst::cst_token_is_scalar_spec(CompletedTokenKind::PlainScalar));
        assert(crucible_yaml::cst::cst_scalar_style_spec(CompletedTokenKind::PlainScalar)
            == CstNodeStyle::Plain);
    }
}

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
        let syntax_owners = Seq::empty().push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 0,
                    kind: CstSyntaxOwnerKind::DocumentStartMarker,
                    record_index: 0,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 1,
                    kind: CstSyntaxOwnerKind::NodeContent,
                    record_index: 0,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 2,
                    kind: CstSyntaxOwnerKind::MappingEntryIndicator,
                    record_index: 0,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 3,
                    kind: CstSyntaxOwnerKind::NodeContent,
                    record_index: 1,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 4,
                    kind: CstSyntaxOwnerKind::DocumentEndMarker,
                    record_index: 0,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 5,
                    kind: CstSyntaxOwnerKind::DocumentStartMarker,
                    record_index: 1,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 6,
                    kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                    record_index: 4,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 7,
                    kind: CstSyntaxOwnerKind::NodeContent,
                    record_index: 3,
                },
            ),
        ).push(
            Some(
                CstSyntaxOwnerView {
                    token_index: 8,
                    kind: CstSyntaxOwnerKind::NodeCollectionIndicator,
                    record_index: 4,
                },
            ),
        );
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
