use crucible_cli::{
    classify_raw_local_execution, CapturedOutput, LocalExecutionClassificationError,
    LocalTermination, RawLocalExecution,
};

fn raw(
    status: &[u8],
    wrapper: LocalTermination,
    target: Option<LocalTermination>,
    target_started: bool,
) -> RawLocalExecution {
    RawLocalExecution::new(
        wrapper,
        target,
        target_started,
        CapturedOutput::new(Vec::new(), 0),
        CapturedOutput::new(Vec::new(), 0),
        0,
        1,
        status.to_vec(),
    )
    .expect("bounded raw host result")
}

#[test]
fn wrapper_exit_without_terminal_target_status_is_not_a_target_outcome() {
    let execution = raw(
        b"{ \"child-pid\": 12 }\n",
        LocalTermination::ExitCode(1),
        None,
        false,
    );
    assert_eq!(
        classify_raw_local_execution(execution),
        Err(LocalExecutionClassificationError::TargetDidNotStart)
    );
}

#[test]
fn matching_terminal_status_is_admitted_as_the_target_exit() {
    let execution = raw(
        b"{ \"child-pid\": 12 }\n{ \"exit-code\": 7 }\n",
        LocalTermination::ExitCode(7),
        Some(LocalTermination::ExitCode(7)),
        true,
    );
    let evidence = classify_raw_local_execution(execution).expect("target started and exited");
    assert_eq!(evidence.termination(), LocalTermination::ExitCode(7));
}

#[test]
fn contradictory_wrapper_and_terminal_status_is_rejected() {
    let execution = raw(
        b"{ \"child-pid\": 12 }\n{ \"exit-code\": 0 }\n",
        LocalTermination::ExitCode(7),
        Some(LocalTermination::ExitCode(0)),
        true,
    );
    assert_eq!(
        classify_raw_local_execution(execution),
        Err(LocalExecutionClassificationError::StatusMismatch)
    );
}

#[test]
fn authenticated_native_signal_is_not_coerced_to_the_wrapper_exit_code() {
    let execution = raw(
        b"{ \"child-pid\": 12 }\n{ \"exit-code\": 139 }\n",
        LocalTermination::ExitCode(139),
        Some(LocalTermination::UnixSignal {
            signal: 11,
            core_dumped: true,
        }),
        true,
    );
    let evidence = classify_raw_local_execution(execution).expect("authenticated target signal");
    assert_eq!(
        evidence.termination(),
        LocalTermination::UnixSignal {
            signal: 11,
            core_dumped: true
        },
    );
}
