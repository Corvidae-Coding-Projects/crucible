use crucible_cli::{evaluate_process_exit_oracle, LocalOracleVerdict, LocalTermination};

#[test]
fn process_exit_oracle_uses_the_versioned_configured_predicate() {
    assert_eq!(
        evaluate_process_exit_oracle(LocalTermination::ExitCode(0), &[0], true),
        LocalOracleVerdict::Pass
    );
    assert_eq!(
        evaluate_process_exit_oracle(LocalTermination::ExitCode(7), &[0], true),
        LocalOracleVerdict::Fail
    );
    assert_eq!(
        evaluate_process_exit_oracle(LocalTermination::Timeout, &[0], true),
        LocalOracleVerdict::Fail
    );
    assert_eq!(
        evaluate_process_exit_oracle(LocalTermination::Timeout, &[0], false),
        LocalOracleVerdict::Pass
    );
    assert_eq!(
        evaluate_process_exit_oracle(
            LocalTermination::UnixSignal {
                signal: 11,
                core_dumped: false,
            },
            &[0],
            false,
        ),
        LocalOracleVerdict::Fail
    );
}
