use crucible_xtask::{parse_args, Action, ParseError};
use vstd::prelude::*;

verus! {

broadcast use vstd::string::group_string_axioms;

#[allow(clippy::vec_init_then_push)]
fn format_arguments() -> (args: Vec<String>)
    ensures
        crucible_xtask::parse_args_spec(args.deep_view()) == Ok(Action::FormatCheck),
{
    let mut format = String::new();
    format.push('f');
    format.push('o');
    format.push('r');
    format.push('m');
    format.push('a');
    format.push('t');

    let mut check = String::new();
    check.push('-');
    check.push('-');
    check.push('c');
    check.push('h');
    check.push('e');
    check.push('c');
    check.push('k');

    let mut args = Vec::new();
    args.push(format);
    args.push(check);
    assert(args.deep_view().len() == 2);
    assert(args.deep_view()[0] == crucible_xtask::format_literal_spec());
    assert(args.deep_view()[1] == crucible_xtask::check_literal_spec());
    proof {
        reveal(crucible_xtask::parse_args_spec);
        reveal(crucible_xtask::format_literal_spec);
        reveal(crucible_xtask::check_literal_spec);
    }
    args
}

#[test]
#[allow(clippy::vec_init_then_push)]
fn parses_required_verification_interface() {
    let mut args = Vec::new();
    args.push(crucible_xtask::verify_literal());
    args.push(crucible_xtask::all_literal());
    assert(args.deep_view()[0] == crucible_xtask::verify_literal_spec());
    assert(args.deep_view()[1] == crucible_xtask::all_literal_spec());
    reveal(crucible_xtask::parse_args_spec);
    reveal(crucible_xtask::verify_literal_spec);
    reveal(crucible_xtask::all_literal_spec);
    assert(crucible_xtask::parse_args_spec(args.deep_view()) == Ok(Action::VerifyAll));
    let action = parse_args(&args);
    match action {
        Ok(Action::VerifyAll) => {},
        _ => vstd::pervasive::unreached(),
    }
}

#[test]
#[allow(clippy::vec_init_then_push)]
fn parses_required_tcb_audit_interface() {
    let mut args = Vec::new();
    args.push(crucible_xtask::tcb_audit_literal());
    args.push(crucible_xtask::deny_unregistered_literal());
    args.push(crucible_xtask::deny_unapproved_growth_literal());
    assert(args.deep_view()[0] == crucible_xtask::tcb_audit_literal_spec());
    assert(args.deep_view()[1] == crucible_xtask::deny_unregistered_literal_spec());
    assert(args.deep_view()[2] == crucible_xtask::deny_unapproved_growth_literal_spec());
    reveal(crucible_xtask::parse_args_spec);
    reveal(crucible_xtask::tcb_audit_literal_spec);
    reveal(crucible_xtask::deny_unregistered_literal_spec);
    reveal(crucible_xtask::deny_unapproved_growth_literal_spec);
    assert(crucible_xtask::parse_args_spec(args.deep_view()) == Ok(Action::TcbAuditStrict));
    let action = parse_args(&args);
    match action {
        Ok(Action::TcbAuditStrict) => {},
        _ => vstd::pervasive::unreached(),
    }
}

#[test]
fn parses_platform_safe_formatter_interface() {
    let args = format_arguments();
    let action = parse_args(&args);
    match action {
        Ok(Action::FormatCheck) => {},
        _ => vstd::pervasive::unreached(),
    }
}

#[test]
fn rejects_scope_weakening_variants() {
    let args: Vec<String> = Vec::new();
    assert(crucible_xtask::parse_args_spec(args.deep_view()) == Err(
        ParseError::UnsupportedArguments,
    ));
    let action = parse_args(&args);
    match action {
        Err(ParseError::UnsupportedArguments) => {},
        _ => vstd::pervasive::unreached(),
    }
}

} // verus!
