use crucible_cli::{parse_cli_args, CliAction, ReportFormat, MAX_CLI_ARGUMENTS};
use std::process::Command;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn documented_command_families_have_distinct_verified_actions() {
    assert_eq!(
        parse_cli_args(&args(&["build", "crucible.yaml"])),
        Ok(CliAction::Build("crucible.yaml".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["fuzz", "crucible.yaml"])),
        Ok(CliAction::Fuzz("crucible.yaml".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["replay", "BUG-000143"])),
        Ok(CliAction::Replay("BUG-000143".into(), ".".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["minimize", "BUG-000143", "workspace"])),
        Ok(CliAction::Minimize("BUG-000143".into(), "workspace".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["findings"])),
        Ok(CliAction::Findings(".".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["findings", "workspace"])),
        Ok(CliAction::Findings("workspace".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&[
            "verify",
            "BUG-000143",
            "--patch",
            "candidate.diff"
        ])),
        Ok(CliAction::VerifyFinding(
            "BUG-000143".into(),
            "candidate.diff".into(),
            ".".into()
        ))
    );
    assert_eq!(
        parse_cli_args(&args(&["capabilities"])),
        Ok(CliAction::Capabilities(".".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["proof", "workspace"])),
        Ok(CliAction::Proof("workspace".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["tcb"])),
        Ok(CliAction::Tcb(".".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["plugins"])),
        Ok(CliAction::Plugins(".".into()))
    );
}

#[test]
fn report_formats_are_explicit_and_workspace_aware() {
    for (name, format) in [
        ("human", ReportFormat::Human),
        ("json", ReportFormat::Json),
        ("jsonl", ReportFormat::JsonLines),
        ("sarif", ReportFormat::Sarif),
        ("junit", ReportFormat::Junit),
        ("evidence", ReportFormat::EvidenceGraph),
        ("bundle", ReportFormat::BundleManifest),
    ] {
        assert_eq!(
            parse_cli_args(&args(&["report", "BUG-000143", "--format", name])),
            Ok(CliAction::Report("BUG-000143".into(), format, ".".into()))
        );
        assert_eq!(
            parse_cli_args(&args(&[
                "report",
                "BUG-000143",
                "--format",
                name,
                "workspace"
            ])),
            Ok(CliAction::Report(
                "BUG-000143".into(),
                format,
                "workspace".into()
            ))
        );
    }
}

#[test]
fn storage_maintenance_is_explicit_and_argument_count_remains_bounded() {
    assert_eq!(
        parse_cli_args(&args(&["artifact", "check"])),
        Ok(CliAction::ArtifactCheck(".".into()))
    );
    assert_eq!(
        parse_cli_args(&args(&["artifact", "gc", "workspace"])),
        Ok(CliAction::ArtifactGc("workspace".into()))
    );
    assert_eq!(MAX_CLI_ARGUMENTS, 5);
    assert!(parse_cli_args(&args(&["report", "x", "--format", "yaml"])).is_err());
    assert!(parse_cli_args(&args(&["verify", "x", "candidate.diff"])).is_err());
}

#[test]
fn rejected_arguments_describe_every_documented_command_family() {
    let output = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .arg("unsupported")
        .output()
        .expect("execute CLI");
    assert!(!output.status.success());
    let usage = String::from_utf8(output.stderr).expect("usage is UTF-8");
    for command in [
        "crucible init",
        "crucible artifact import",
        "crucible artifact verify",
        "crucible artifact check",
        "crucible artifact gc",
        "crucible build",
        "crucible run",
        "crucible fuzz",
        "crucible replay",
        "crucible minimize",
        "crucible findings",
        "crucible inspect",
        "crucible verify",
        "crucible report",
        "crucible config validate",
        "crucible config canonicalize",
        "crucible capabilities",
        "crucible proof",
        "crucible tcb",
        "crucible plugins",
    ] {
        assert!(usage.contains(command), "usage omitted {command}: {usage}");
    }
}
