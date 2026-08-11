#![forbid(unsafe_code)]

use vstd::prelude::*;

mod tcb;
mod toolchain;

pub use tcb::{
    parse_approvals, parse_ledger, reconcile_boundaries, scan_boundaries, ApprovalEntry,
    AuditError, AuditSummary, BoundaryKind, BoundaryOccurrence, LedgerEntry, SourceFile,
    SourceOrigin,
};
pub use toolchain::{
    validate_tool_probes, validate_toolchain_lock, ToolName, ToolProbe, ToolchainError,
};

macro_rules! define_cli_literal {
    ($exec_name:ident, $spec_name:ident, [$($character:literal),+ $(,)?]) => {
        verus! {

        #[doc(hidden)]
        pub open spec fn $spec_name() -> Seq<char> {
            seq![$($character),+]
        }

        #[doc(hidden)]
        pub fn $exec_name() -> (value: String)
            ensures
                value@ == $spec_name(),
        {
            let mut value = String::new();
            $(value.push($character);)+
            value
        }

        } // verus!
    };
}

define_cli_literal!(
    verify_literal,
    verify_literal_spec,
    ['v', 'e', 'r', 'i', 'f', 'y']
);
define_cli_literal!(all_literal, all_literal_spec, ['-', '-', 'a', 'l', 'l']);
define_cli_literal!(
    format_literal,
    format_literal_spec,
    ['f', 'o', 'r', 'm', 'a', 't']
);
define_cli_literal!(
    check_literal,
    check_literal_spec,
    ['-', '-', 'c', 'h', 'e', 'c', 'k']
);
define_cli_literal!(
    tcb_audit_literal,
    tcb_audit_literal_spec,
    ['t', 'c', 'b', '-', 'a', 'u', 'd', 'i', 't']
);
define_cli_literal!(
    deny_unregistered_literal,
    deny_unregistered_literal_spec,
    [
        '-', '-', 'd', 'e', 'n', 'y', '-', 'u', 'n', 'r', 'e', 'g', 'i', 's', 't', 'e', 'r', 'e',
        'd',
    ]
);
define_cli_literal!(
    deny_unapproved_growth_literal,
    deny_unapproved_growth_literal_spec,
    [
        '-', '-', 'd', 'e', 'n', 'y', '-', 'u', 'n', 'a', 'p', 'p', 'r', 'o', 'v', 'e', 'd', '-',
        'g', 'r', 'o', 'w', 't', 'h',
    ]
);

verus! {

broadcast use vstd::string::group_string_axioms;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    VerifyAll,
    TcbAuditStrict,
    FormatCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    UnsupportedArguments,
}

pub open spec fn parse_args_spec(args: Seq<Seq<char>>) -> Result<Action, ParseError> {
    if args.len() == 2 && args[0] == verify_literal_spec() && args[1] == all_literal_spec() {
        Ok(Action::VerifyAll)
    } else if args.len() == 2 && args[0] == format_literal_spec() && args[1]
        == check_literal_spec() {
        Ok(Action::FormatCheck)
    } else if args.len() == 3 && args[0] == tcb_audit_literal_spec() && args[1]
        == deny_unregistered_literal_spec() && args[2] == deny_unapproved_growth_literal_spec() {
        Ok(Action::TcbAuditStrict)
    } else {
        Err(ParseError::UnsupportedArguments)
    }
}

pub fn parse_args(args: &[String]) -> (action: Result<Action, ParseError>)
    ensures
        action == parse_args_spec(args.deep_view()),
{
    let verify = verify_literal();
    let all = all_literal();
    let tcb_audit = tcb_audit_literal();
    let format = format_literal();
    let check = check_literal();
    let deny_unregistered = deny_unregistered_literal();
    let deny_unapproved_growth = deny_unapproved_growth_literal();

    if args.len() == 2 && args[0] == verify && args[1] == all {
        Ok(Action::VerifyAll)
    } else if args.len() == 2 && args[0] == format && args[1] == check {
        Ok(Action::FormatCheck)
    } else if args.len() == 3 && args[0] == tcb_audit && args[1] == deny_unregistered && args[2]
        == deny_unapproved_growth {
        Ok(Action::TcbAuditStrict)
    } else {
        Err(ParseError::UnsupportedArguments)
    }
}

} // verus!
