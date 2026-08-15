//! Verified admission and exact effective-control projection for the local CLI executor.
use crate::{EffectiveExecutionConfiguration, MAX_LOCAL_ARTIFACT_BYTES};
use crucible_core::{
    canonical_raw_observation_limits, validate_raw_observation, ArtifactRef, CapturedStreamRef,
    CompletionDisposition, HarnessTerminationReason, RawExecutionEvent, RawExecutionOutcome,
    RawObservation, RecordedDuration, ResourceSnapshot, RunAttemptId, RunId, TargetAdapterIdentity,
    TargetAdapterKind, TargetBuildId, TargetId, TargetInstanceLifecycle, TargetLifecycleAction,
    TargetLifecycleError, TerminationRecord, ValidatedRawObservation,
};
#[expect(
    unused_imports,
    reason = "the sequence-extensionality macro is consumed only by Verus proof erasure"
)]
use vstd::assert_seqs_equal;
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

verus! {

pub const BYTES_PER_MEBIBYTE: u64 = 1_048_576;

pub const MAX_LOCAL_CONTROL_STATUS_BYTES: u64 = 65_536;

pub const MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES: u64 = 4_096;

pub const MAX_LOCAL_ARGUMENT_WIRE_BYTES: u64 = MAX_LOCAL_ARTIFACT_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalExecutionBackend {
    LinuxBubblewrapPrlimitV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalNetworkPolicy {
    None,
    UnrestrictedHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputCapturePolicy {
    DrainAndDiscard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalRunPlanError {
    UnsupportedPlatform,
    ArithmeticOverflow,
    OutputLimitTooLarge,
    UnsupportedStorageLayout,
    RequiredCapabilityUnavailable { index: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalTermination {
    ExitCode(i64),
    UnixSignal { signal: i32, core_dumped: bool },
    Timeout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalOracleVerdict {
    Pass,
    Fail,
}

pub open spec fn allowed_exit_code_spec(allowed: Seq<i64>, code: i64) -> bool {
    exists|index: int| 0 <= index < allowed.len() && allowed[index] == code
}

pub open spec fn process_exit_oracle_spec(
    termination: LocalTermination,
    allowed_exit_codes: Seq<i64>,
    timeout_is_failure: bool,
) -> LocalOracleVerdict {
    match termination {
        LocalTermination::ExitCode(code) => if allowed_exit_code_spec(allowed_exit_codes, code) {
            LocalOracleVerdict::Pass
        } else {
            LocalOracleVerdict::Fail
        },
        LocalTermination::UnixSignal { .. } => LocalOracleVerdict::Fail,
        LocalTermination::Timeout => if timeout_is_failure {
            LocalOracleVerdict::Fail
        } else {
            LocalOracleVerdict::Pass
        },
    }
}

fn allowed_exit_code(allowed: &[i64], code: i64) -> (accepted: bool)
    ensures
        accepted == allowed_exit_code_spec(allowed@, code),
{
    let mut index = 0usize;
    while index < allowed.len()
        invariant
            index <= allowed.len(),
            forall|prior: int| 0 <= prior < index ==> allowed@[prior] != code,
        decreases allowed.len() - index,
    {
        if allowed[index] == code {
            assert(allowed_exit_code_spec(allowed@, code));
            return true;
        }
        index += 1;
    }
    assert(!allowed_exit_code_spec(allowed@, code));
    false
}

pub fn evaluate_process_exit_oracle(
    termination: LocalTermination,
    allowed_exit_codes: &[i64],
    timeout_is_failure: bool,
) -> (verdict: LocalOracleVerdict)
    ensures
        verdict == process_exit_oracle_spec(termination, allowed_exit_codes@, timeout_is_failure),
{
    match termination {
        LocalTermination::ExitCode(code) => if allowed_exit_code(allowed_exit_codes, code) {
            LocalOracleVerdict::Pass
        } else {
            LocalOracleVerdict::Fail
        },
        LocalTermination::UnixSignal { .. } => LocalOracleVerdict::Fail,
        LocalTermination::Timeout => if timeout_is_failure {
            LocalOracleVerdict::Fail
        } else {
            LocalOracleVerdict::Pass
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalExecutionEvidenceError {
    RetainedOutputTooLarge,
    NanosecondsOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawLocalExecutionError {
    InvalidTermination,
    RetainedOutputTooLarge,
    StatusTooLarge,
    NanosecondsOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalExecutionClassificationError {
    TargetDidNotStart,
    StatusMismatch,
    InvalidEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalCapabilityProbeError {
    ReportMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalRuntimeIdentityError {
    EmptyField,
    InvalidField,
    FieldTooLong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalArgumentWireError {
    InvalidCodePoint,
    TooLarge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunAttemptStatus {
    Reserved,
    TargetPrepared,
    Observed,
    HarnessFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunStoreTransition {
    AttachTarget,
    RecordObservation,
    RecordHarnessFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunStoreTransitionError {
    InvalidTransition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalObservationError {
    StdoutArtifactMismatch,
    StderrArtifactMismatch,
    InvalidDuration,
    InvalidObservation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReservedRunError {
    EmptyRunId,
    EmptyAttemptId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReservedRun {
    run_id: RunId,
    attempt_id: RunAttemptId,
}

#[verifier::ext_equal]
pub struct ReservedRunView {
    pub run_id: Seq<char>,
    pub attempt_id: Seq<char>,
}

impl View for ReservedRun {
    type V = ReservedRunView;

    closed spec fn view(&self) -> ReservedRunView {
        ReservedRunView { run_id: self.run_id@, attempt_id: self.attempt_id@ }
    }
}

pub open spec fn reserved_run_well_formed_spec(reservation: ReservedRunView) -> bool {
    reservation.run_id.len() > 0 && reservation.attempt_id.len() > 0
}

impl ReservedRun {
    pub fn new(run_id: String, attempt_id: String) -> (result: Result<Self, ReservedRunError>)
        ensures
            match &result {
                Ok(reservation) => reserved_run_well_formed_spec(reservation@),
                Err(ReservedRunError::EmptyRunId) => run_id@.len() == 0,
                Err(ReservedRunError::EmptyAttemptId) => run_id@.len() > 0 && attempt_id@.len()
                    == 0,
            },
    {
        if run_id.is_empty() {
            return Err(ReservedRunError::EmptyRunId);
        }
        if attempt_id.is_empty() {
            return Err(ReservedRunError::EmptyAttemptId);
        }
        Ok(Self { run_id: RunId::new(run_id), attempt_id: RunAttemptId::new(attempt_id) })
    }

    pub fn run_id(&self) -> (value: &RunId)
        ensures
            value@ == self@.run_id,
    {
        &self.run_id
    }

    pub fn attempt_id(&self) -> (value: &RunAttemptId)
        ensures
            value@ == self@.attempt_id,
    {
        &self.attempt_id
    }
}

pub open spec fn run_store_transition_spec(
    current: RunAttemptStatus,
    transition: RunStoreTransition,
) -> Option<RunAttemptStatus> {
    match (current, transition) {
        (RunAttemptStatus::Reserved, RunStoreTransition::AttachTarget) => {
            Some(RunAttemptStatus::TargetPrepared)
        },
        (RunAttemptStatus::Reserved, RunStoreTransition::RecordHarnessFailure)
        | (RunAttemptStatus::TargetPrepared, RunStoreTransition::RecordHarnessFailure) => {
            Some(RunAttemptStatus::HarnessFailure)
        },
        (RunAttemptStatus::TargetPrepared, RunStoreTransition::RecordObservation) => {
            Some(RunAttemptStatus::Observed)
        },
        _ => None,
    }
}

pub fn admit_run_store_transition(
    current: RunAttemptStatus,
    transition: RunStoreTransition,
) -> (result: Result<RunAttemptStatus, RunStoreTransitionError>)
    ensures
        match result {
            Ok(next) => run_store_transition_spec(current, transition) == Some(next),
            Err(RunStoreTransitionError::InvalidTransition) => {
                run_store_transition_spec(current, transition) is None
            },
        },
{
    match (current, transition) {
        (RunAttemptStatus::Reserved, RunStoreTransition::AttachTarget) => {
            Ok(RunAttemptStatus::TargetPrepared)
        },
        (RunAttemptStatus::Reserved, RunStoreTransition::RecordHarnessFailure)
        | (RunAttemptStatus::TargetPrepared, RunStoreTransition::RecordHarnessFailure) => {
            Ok(RunAttemptStatus::HarnessFailure)
        },
        (RunAttemptStatus::TargetPrepared, RunStoreTransition::RecordObservation) => {
            Ok(RunAttemptStatus::Observed)
        },
        _ => Err(RunStoreTransitionError::InvalidTransition),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CapturedOutput {
    retained: Vec<u8>,
    discarded: u64,
}

#[verifier::ext_equal]
pub struct CapturedOutputView {
    pub retained: Seq<u8>,
    pub discarded: u64,
}

impl View for CapturedOutput {
    type V = CapturedOutputView;

    closed spec fn view(&self) -> CapturedOutputView {
        CapturedOutputView { retained: self.retained@, discarded: self.discarded }
    }
}

impl CapturedOutput {
    pub fn new(retained: Vec<u8>, discarded: u64) -> (output: Self)
        ensures
            output@ == (CapturedOutputView { retained: retained@, discarded }),
    {
        Self { retained, discarded }
    }

    pub fn retained(&self) -> (value: &[u8])
        ensures
            value@ == self@.retained,
    {
        self.retained.as_slice()
    }

    pub fn discarded(&self) -> (value: u64)
        ensures
            value == self@.discarded,
    {
        self.discarded
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalExecutionEvidence {
    termination: LocalTermination,
    stdout: CapturedOutput,
    stderr: CapturedOutput,
    wall_seconds: u64,
    wall_nanoseconds: u32,
}

#[verifier::ext_equal]
pub struct LocalExecutionEvidenceView {
    pub termination: LocalTermination,
    pub stdout: CapturedOutputView,
    pub stderr: CapturedOutputView,
    pub wall_seconds: u64,
    pub wall_nanoseconds: u32,
}

impl View for LocalExecutionEvidence {
    type V = LocalExecutionEvidenceView;

    closed spec fn view(&self) -> LocalExecutionEvidenceView {
        LocalExecutionEvidenceView {
            termination: self.termination,
            stdout: self.stdout@,
            stderr: self.stderr@,
            wall_seconds: self.wall_seconds,
            wall_nanoseconds: self.wall_nanoseconds,
        }
    }
}

pub open spec fn local_execution_evidence_well_formed_spec(
    evidence: LocalExecutionEvidenceView,
) -> bool {
    evidence.stdout.retained.len() <= MAX_LOCAL_ARTIFACT_BYTES && evidence.stderr.retained.len()
        <= MAX_LOCAL_ARTIFACT_BYTES && evidence.wall_nanoseconds < 1_000_000_000
}

impl LocalExecutionEvidence {
    pub fn new(
        termination: LocalTermination,
        stdout: CapturedOutput,
        stderr: CapturedOutput,
        wall_seconds: u64,
        wall_nanoseconds: u32,
    ) -> (result: Result<Self, LocalExecutionEvidenceError>)
        ensures
            wall_nanoseconds >= 1_000_000_000 ==> result is Err,
            stdout@.retained.len() > MAX_LOCAL_ARTIFACT_BYTES || stderr@.retained.len()
                > MAX_LOCAL_ARTIFACT_BYTES ==> result is Err,
            match &result {
                Ok(evidence) => local_execution_evidence_well_formed_spec(evidence@) && evidence@
                    == (LocalExecutionEvidenceView {
                    termination,
                    stdout: stdout@,
                    stderr: stderr@,
                    wall_seconds,
                    wall_nanoseconds,
                }),
                Err(LocalExecutionEvidenceError::RetainedOutputTooLarge) => stdout@.retained.len()
                    > MAX_LOCAL_ARTIFACT_BYTES || stderr@.retained.len() > MAX_LOCAL_ARTIFACT_BYTES,
                Err(LocalExecutionEvidenceError::NanosecondsOutOfRange) => wall_nanoseconds
                    >= 1_000_000_000,
            },
    {
        if stdout.retained.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES || stderr.retained.len() as u64
            > MAX_LOCAL_ARTIFACT_BYTES {
            return Err(LocalExecutionEvidenceError::RetainedOutputTooLarge);
        }
        if wall_nanoseconds >= 1_000_000_000 {
            return Err(LocalExecutionEvidenceError::NanosecondsOutOfRange);
        }
        Ok(Self { termination, stdout, stderr, wall_seconds, wall_nanoseconds })
    }

    pub fn termination(&self) -> (value: LocalTermination)
        ensures
            value == self@.termination,
    {
        self.termination
    }

    pub fn stdout(&self) -> (value: &CapturedOutput)
        ensures
            value@ == self@.stdout,
    {
        &self.stdout
    }

    pub fn stderr(&self) -> (value: &CapturedOutput)
        ensures
            value@ == self@.stderr,
    {
        &self.stderr
    }

    pub fn wall_seconds(&self) -> (value: u64)
        ensures
            value == self@.wall_seconds,
    {
        self.wall_seconds
    }

    pub fn wall_nanoseconds(&self) -> (value: u32)
        ensures
            value == self@.wall_nanoseconds,
    {
        self.wall_nanoseconds
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawLocalExecution {
    wrapper_termination: LocalTermination,
    target_termination: Option<LocalTermination>,
    target_started: bool,
    stdout: CapturedOutput,
    stderr: CapturedOutput,
    wall_seconds: u64,
    wall_nanoseconds: u32,
    control_status: Vec<u8>,
}

#[verifier::ext_equal]
pub struct RawLocalExecutionView {
    pub wrapper_termination: LocalTermination,
    pub target_termination: Option<LocalTermination>,
    pub target_started: bool,
    pub stdout: CapturedOutputView,
    pub stderr: CapturedOutputView,
    pub wall_seconds: u64,
    pub wall_nanoseconds: u32,
    pub control_status: Seq<u8>,
}

impl View for RawLocalExecution {
    type V = RawLocalExecutionView;

    closed spec fn view(&self) -> RawLocalExecutionView {
        RawLocalExecutionView {
            wrapper_termination: self.wrapper_termination,
            target_termination: self.target_termination,
            target_started: self.target_started,
            stdout: self.stdout@,
            stderr: self.stderr@,
            wall_seconds: self.wall_seconds,
            wall_nanoseconds: self.wall_nanoseconds,
            control_status: self.control_status@,
        }
    }
}

pub open spec fn raw_local_execution_well_formed_spec(raw: RawLocalExecutionView) -> bool {
    raw.stdout.retained.len() <= MAX_LOCAL_ARTIFACT_BYTES && raw.stderr.retained.len()
        <= MAX_LOCAL_ARTIFACT_BYTES && raw.control_status.len() <= MAX_LOCAL_CONTROL_STATUS_BYTES
        && raw.wall_nanoseconds < 1_000_000_000
}

pub open spec fn local_execution_classification_spec(raw: RawLocalExecutionView) -> Option<
    LocalExecutionEvidenceView,
> {
    if !raw.target_started {
        None
    } else {
        let authenticated_termination = match raw.wrapper_termination {
            LocalTermination::Timeout => Some(LocalTermination::Timeout),
            _ => if target_termination_matches_wrapper_spec(
                raw.target_termination,
                raw.wrapper_termination,
            ) {
                raw.target_termination
            } else {
                None
            },
        };
        match authenticated_termination {
            Some(termination) => Some(
                LocalExecutionEvidenceView {
                    termination,
                    stdout: raw.stdout,
                    stderr: raw.stderr,
                    wall_seconds: raw.wall_seconds,
                    wall_nanoseconds: raw.wall_nanoseconds,
                },
            ),
            None => None,
        }
    }
}

pub open spec fn target_termination_matches_wrapper_spec(
    target: Option<LocalTermination>,
    wrapper: LocalTermination,
) -> bool {
    match (target, wrapper) {
        (
            Some(LocalTermination::ExitCode(target_code)),
            LocalTermination::ExitCode(wrapper_code),
        ) => { target_code == wrapper_code },
        (
            Some(LocalTermination::UnixSignal { signal, core_dumped: _ }),
            LocalTermination::ExitCode(wrapper_code),
        ) => 0 < signal < 128 && wrapper_code == (128 + signal) as i64,
        _ => false,
    }
}

fn target_termination_matches_wrapper(
    target: Option<LocalTermination>,
    wrapper: LocalTermination,
) -> (matches: bool)
    ensures
        matches == target_termination_matches_wrapper_spec(target, wrapper),
{
    match (target, wrapper) {
        (
            Some(LocalTermination::ExitCode(target_code)),
            LocalTermination::ExitCode(wrapper_code),
        ) => { target_code == wrapper_code },
        (
            Some(LocalTermination::UnixSignal { signal, core_dumped: _ }),
            LocalTermination::ExitCode(wrapper_code),
        ) => signal > 0 && signal < 128 && wrapper_code == (128 + signal) as i64,
        _ => false,
    }
}

impl RawLocalExecution {
    #[expect(
        clippy::too_many_arguments,
        reason = "every raw host status field is explicit at the verified classification boundary"
    )]
    pub fn new(
        wrapper_termination: LocalTermination,
        target_termination: Option<LocalTermination>,
        target_started: bool,
        stdout: CapturedOutput,
        stderr: CapturedOutput,
        wall_seconds: u64,
        wall_nanoseconds: u32,
        control_status: Vec<u8>,
    ) -> (result: Result<Self, RawLocalExecutionError>)
        ensures
            match &result {
                Ok(raw) => raw_local_execution_well_formed_spec(raw@),
                Err(_) => true,
            },
    {
        if stdout.retained.len() as u64 > MAX_LOCAL_ARTIFACT_BYTES || stderr.retained.len() as u64
            > MAX_LOCAL_ARTIFACT_BYTES {
            return Err(RawLocalExecutionError::RetainedOutputTooLarge);
        }
        if control_status.len() as u64 > MAX_LOCAL_CONTROL_STATUS_BYTES {
            return Err(RawLocalExecutionError::StatusTooLarge);
        }
        if wall_nanoseconds >= 1_000_000_000 {
            return Err(RawLocalExecutionError::NanosecondsOutOfRange);
        }
        Ok(
            Self {
                wrapper_termination,
                target_termination,
                target_started,
                stdout,
                stderr,
                wall_seconds,
                wall_nanoseconds,
                control_status,
            },
        )
    }
}

pub fn classify_raw_local_execution(raw: RawLocalExecution) -> (result: Result<
    LocalExecutionEvidence,
    LocalExecutionClassificationError,
>)
    requires
        raw_local_execution_well_formed_spec(raw@),
    ensures
        match &result {
            Ok(evidence) => local_execution_evidence_well_formed_spec(evidence@)
                && local_execution_classification_spec(raw@) == Some(evidence@),
            Err(_) => local_execution_classification_spec(raw@) is None,
        },
{
    let ghost raw_view = raw@;
    if !raw.target_started {
        assert(local_execution_classification_spec(raw_view) is None);
        return Err(LocalExecutionClassificationError::TargetDidNotStart);
    }
    let authenticated_termination = match raw.wrapper_termination {
        LocalTermination::Timeout => LocalTermination::Timeout,
        _ => {
            if !target_termination_matches_wrapper(
                raw.target_termination,
                raw.wrapper_termination,
            ) {
                assert(!target_termination_matches_wrapper_spec(
                    raw.target_termination,
                    raw.wrapper_termination,
                ));
                assert(local_execution_classification_spec(raw_view) is None);
                return Err(LocalExecutionClassificationError::StatusMismatch);
            }
            match raw.target_termination {
                Some(termination) => termination,
                None => {
                    assert(false);
                    return Err(LocalExecutionClassificationError::StatusMismatch);
                },
            }
        },
    };
    match LocalExecutionEvidence::new(
        authenticated_termination,
        raw.stdout,
        raw.stderr,
        raw.wall_seconds,
        raw.wall_nanoseconds,
    ) {
        Ok(evidence) => {
            assert(local_execution_classification_spec(raw_view) == Some(evidence@));
            Ok(evidence)
        },
        Err(LocalExecutionEvidenceError::RetainedOutputTooLarge) => {
            assert(false);
            Err(LocalExecutionClassificationError::InvalidEvidence)
        },
        Err(LocalExecutionEvidenceError::NanosecondsOutOfRange) => {
            assert(false);
            Err(LocalExecutionClassificationError::InvalidEvidence)
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalExecutionPlan {
    command: Vec<u32>,
    arguments: Vec<Vec<u32>>,
    timeout_ms: u64,
    memory_bytes: u64,
    max_processes: u64,
    max_stream_bytes: u64,
    network_policy: LocalNetworkPolicy,
    backend: LocalExecutionBackend,
    output_capture_policy: OutputCapturePolicy,
}

#[verifier::ext_equal]
pub struct LocalExecutionPlanView {
    pub command: Seq<u32>,
    pub arguments: Seq<Seq<u32>>,
    pub timeout_ms: u64,
    pub memory_bytes: u64,
    pub max_processes: u64,
    pub max_stream_bytes: u64,
    pub network_policy: LocalNetworkPolicy,
    pub backend: LocalExecutionBackend,
    pub output_capture_policy: OutputCapturePolicy,
}

impl View for LocalExecutionPlan {
    type V = LocalExecutionPlanView;

    closed spec fn view(&self) -> LocalExecutionPlanView {
        LocalExecutionPlanView {
            command: self.command@,
            arguments: crate::configuration::configuration_text_sequence_views_spec(
                self.arguments@,
            ),
            timeout_ms: self.timeout_ms,
            memory_bytes: self.memory_bytes,
            max_processes: self.max_processes,
            max_stream_bytes: self.max_stream_bytes,
            network_policy: self.network_policy,
            backend: self.backend,
            output_capture_policy: self.output_capture_policy,
        }
    }
}

pub open spec fn ascii_code_points_spec(value: Seq<u8>) -> Seq<u32> {
    Seq::new(value.len(), |index: int| value[index] as u32)
}

pub open spec fn local_capability_supported_spec(
    capability: Seq<u32>,
    network_enabled: bool,
) -> bool {
    capability == ascii_code_points_spec(b"process_group_termination"@) || capability
        == ascii_code_points_spec(b"resource_limits"@) || capability == ascii_code_points_spec(
        b"private_working_directory"@,
    ) || capability == ascii_code_points_spec(b"bounded_output_capture"@) || capability
        == ascii_code_points_spec(b"wall_clock_timeout"@) || capability == ascii_code_points_spec(
        b"controlled_environment"@,
    ) || capability == ascii_code_points_spec(b"memory_limit"@) || capability
        == ascii_code_points_spec(b"process_count_limit"@) || capability == ascii_code_points_spec(
        b"file_size_limit"@,
    ) || (!network_enabled && capability == ascii_code_points_spec(b"network_isolation"@))
}

pub open spec fn all_local_capabilities_supported_spec(
    capabilities: Seq<Seq<u32>>,
    network_enabled: bool,
) -> bool {
    forall|index: int|
        0 <= index < capabilities.len() ==> local_capability_supported_spec(
            capabilities[index],
            network_enabled,
        )
}

pub open spec fn local_execution_plan_well_formed_spec(plan: LocalExecutionPlanView) -> bool {
    plan.command.len() > 0 && plan.timeout_ms > 0 && plan.memory_bytes > 0 && plan.max_processes > 0
        && plan.max_stream_bytes > 0 && plan.max_stream_bytes <= MAX_LOCAL_ARTIFACT_BYTES
        && plan.backend == LocalExecutionBackend::LinuxBubblewrapPrlimitV1
        && plan.output_capture_policy == OutputCapturePolicy::DrainAndDiscard
}

pub open spec fn local_execution_plan_matches_configuration_spec(
    configuration: crate::configuration::EffectiveExecutionConfigurationView,
    plan: LocalExecutionPlanView,
) -> bool {
    plan.command == configuration.command && plan.arguments == configuration.arguments
        && plan.timeout_ms == configuration.timeout_ms && plan.memory_bytes as int
        == configuration.memory_mb as int * BYTES_PER_MEBIBYTE as int && plan.max_processes
        == configuration.max_processes && plan.max_stream_bytes as int
        == configuration.max_output_mb as int * BYTES_PER_MEBIBYTE as int && plan.network_policy
        == if configuration.network_enabled {
        LocalNetworkPolicy::UnrestrictedHost
    } else {
        LocalNetworkPolicy::None
    } && all_local_capabilities_supported_spec(
        configuration.required_capabilities,
        configuration.network_enabled,
    ) && configuration.storage_root == ascii_code_points_spec(b".crucible"@)
}

fn code_points_equal_ascii(value: &[u32], ascii: &[u8]) -> (equal: bool)
    ensures
        equal == (value@ == ascii_code_points_spec(ascii@)),
{
    if value.len() != ascii.len() {
        return false;
    }
    let mut index = 0usize;
    while index < value.len()
        invariant
            index <= value.len(),
            value.len() == ascii.len(),
            forall|prior: int|
                0 <= prior < index ==> value@[prior] == ascii_code_points_spec(ascii@)[prior],
        decreases value.len() - index,
    {
        if value[index] != ascii[index] as u32 {
            return false;
        }
        index += 1;
    }
    assert(value@ =~= ascii_code_points_spec(ascii@));
    true
}

fn local_capability_supported(capability: &[u32], network_enabled: bool) -> (supported: bool)
    ensures
        supported == local_capability_supported_spec(capability@, network_enabled),
{
    code_points_equal_ascii(capability, b"process_group_termination") || code_points_equal_ascii(
        capability,
        b"resource_limits",
    ) || code_points_equal_ascii(capability, b"private_working_directory")
        || code_points_equal_ascii(capability, b"bounded_output_capture")
        || code_points_equal_ascii(capability, b"wall_clock_timeout") || code_points_equal_ascii(
        capability,
        b"controlled_environment",
    ) || code_points_equal_ascii(capability, b"memory_limit") || code_points_equal_ascii(
        capability,
        b"process_count_limit",
    ) || code_points_equal_ascii(capability, b"file_size_limit") || (!network_enabled
        && code_points_equal_ascii(capability, b"network_isolation"))
}

fn clone_code_points(source: &[u32]) -> (copy: Vec<u32>)
    ensures
        copy@ == source@,
{
    let mut copy: Vec<u32> = Vec::new();
    let mut index = 0usize;
    while index < source.len()
        invariant
            index <= source.len(),
            copy@ == source@.subrange(0, index as int),
        decreases source.len() - index,
    {
        copy.push(source[index]);
        index += 1;
    }
    copy
}

fn clone_code_point_sequences(source: &[Vec<u32>]) -> (copy: Vec<Vec<u32>>)
    ensures
        crate::configuration::configuration_text_sequence_views_spec(copy@)
            == crate::configuration::configuration_text_sequence_views_spec(source@),
{
    let mut copy: Vec<Vec<u32>> = Vec::new();
    let mut index = 0usize;
    while index < source.len()
        invariant
            index <= source.len(),
            copy.len() == index,
            forall|prior: int| 0 <= prior < index ==> copy@[prior]@ == source@[prior]@,
        decreases source.len() - index,
    {
        copy.push(clone_code_points(source[index].as_slice()));
        index += 1;
    }
    assert(crate::configuration::configuration_text_sequence_views_spec(copy@)
        =~= crate::configuration::configuration_text_sequence_views_spec(source@));
    copy
}

impl LocalExecutionPlan {
    pub fn target_command(&self) -> (value: &[u32])
        ensures
            value@ == self@.command,
    {
        self.command.as_slice()
    }

    pub fn target_arguments(&self) -> (value: &[Vec<u32>])
        ensures
            crate::configuration::configuration_text_sequence_views_spec(value@) == self@.arguments,
    {
        self.arguments.as_slice()
    }

    pub fn timeout_ms(&self) -> (value: u64)
        ensures
            value == self@.timeout_ms,
    {
        self.timeout_ms
    }

    pub fn memory_bytes(&self) -> (value: u64)
        ensures
            value == self@.memory_bytes,
    {
        self.memory_bytes
    }

    pub fn max_processes(&self) -> (value: u64)
        ensures
            value == self@.max_processes,
    {
        self.max_processes
    }

    pub fn max_stream_bytes(&self) -> (value: u64)
        ensures
            value == self@.max_stream_bytes,
    {
        self.max_stream_bytes
    }

    pub fn network_policy(&self) -> (value: LocalNetworkPolicy)
        ensures
            value == self@.network_policy,
    {
        self.network_policy
    }

    pub fn backend(&self) -> (value: LocalExecutionBackend)
        ensures
            value == self@.backend,
    {
        self.backend
    }

    pub fn output_capture_policy(&self) -> (value: OutputCapturePolicy)
        ensures
            value == self@.output_capture_policy,
    {
        self.output_capture_policy
    }
}

pub open spec fn local_target_argument_wire_shape_spec(wire: Seq<u8>) -> bool {
    wire.len() <= MAX_LOCAL_ARGUMENT_WIRE_BYTES && wire.len() >= 25 && wire.subrange(0, 17)
        == b"CRUCIBLE-ARGV-V1\n"@
}

proof fn lemma_append_preserves_prefix(left: Seq<u8>, right: Seq<u8>, end: int)
    requires
        0 <= end <= left.len(),
    ensures
        (left + right).subrange(0, end) == left.subrange(0, end),
{
    assert_seqs_equal!(
        (left + right).subrange(0, end) == left.subrange(0, end),
        index => {
            assert((left + right)[index] == left[index]);
        }
    );
}

fn push_argument_wire_byte(output: &mut Vec<u8>, byte: u8) -> (result: Result<
    (),
    LocalArgumentWireError,
>)
    ensures
        match result {
            Ok(()) => final(output)@ == old(output)@.push(byte) && final(output)@.len()
                <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
            Err(LocalArgumentWireError::TooLarge) => old(output)@.len()
                >= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
            Err(_) => false,
        },
{
    if output.len() as u64 >= MAX_LOCAL_ARGUMENT_WIRE_BYTES {
        return Err(LocalArgumentWireError::TooLarge);
    }
    output.push(byte);
    Ok(())
}

fn append_argument_wire_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> (result: Result<
    (),
    LocalArgumentWireError,
>)
    requires
        old(output)@.len() <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
    ensures
        result is Ok ==> final(output)@ == old(output)@ + bytes@ && final(output)@.len()
            <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
{
    let ghost initial = output@;
    let mut index = 0usize;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
            output@ == initial + bytes@.subrange(0, index as int),
            output@.len() <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
        decreases bytes.len() - index,
    {
        push_argument_wire_byte(output, bytes[index])?;
        index += 1;
    }
    assert(bytes@.subrange(0, bytes@.len() as int) == bytes@);
    assert(output@ =~= initial + bytes@);
    Ok(())
}

fn append_argument_wire_u64(output: &mut Vec<u8>, value: u64) -> (result: Result<
    (),
    LocalArgumentWireError,
>)
    requires
        old(output)@.len() <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
    ensures
        result is Ok ==> final(output)@.len() == old(output)@.len() + 8 && final(output)@.len()
            <= MAX_LOCAL_ARGUMENT_WIRE_BYTES && final(output)@.subrange(
            0,
            old(output)@.len() as int,
        ) == old(output)@,
{
    let ghost initial = output@;
    push_argument_wire_byte(output, (value >> 56) as u8)?;
    push_argument_wire_byte(output, ((value >> 48) & 0xff) as u8)?;
    push_argument_wire_byte(output, ((value >> 40) & 0xff) as u8)?;
    push_argument_wire_byte(output, ((value >> 32) & 0xff) as u8)?;
    push_argument_wire_byte(output, ((value >> 24) & 0xff) as u8)?;
    push_argument_wire_byte(output, ((value >> 16) & 0xff) as u8)?;
    push_argument_wire_byte(output, ((value >> 8) & 0xff) as u8)?;
    push_argument_wire_byte(output, (value & 0xff) as u8)?;
    assert(output@.subrange(0, initial.len() as int) == initial) by {
        assert_seqs_equal!(output@.subrange(0, initial.len() as int) == initial);
    };
    Ok(())
}

fn encode_argument_code_points(points: &[u32]) -> (result: Result<
    Vec<u8>,
    LocalArgumentWireError,
>) {
    let mut output = Vec::new();
    let mut index = 0usize;
    while index < points.len()
        invariant
            index <= points.len(),
            output@.len() <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
        decreases points.len() - index,
    {
        let point = points[index];
        if point <= 0x7f {
            push_argument_wire_byte(&mut output, point as u8)?;
        } else if point <= 0x7ff {
            assert(0xc0 + point / 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0xc0 + point / 0x40) as u8)?;
            assert(0x80 + point % 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0x80 + point % 0x40) as u8)?;
        } else if point <= 0xffff {
            let surrogate = point >> 11 == 0x1b;
            if surrogate {
                return Err(LocalArgumentWireError::InvalidCodePoint);
            }
            assert(0xe0 + point / 0x1000 <= 0xff);
            push_argument_wire_byte(&mut output, (0xe0 + point / 0x1000) as u8)?;
            assert(0x80 + (point / 0x40) % 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0x80 + (point / 0x40) % 0x40) as u8)?;
            assert(0x80 + point % 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0x80 + point % 0x40) as u8)?;
        } else if point <= 0x10ffff {
            assert(0xf0 + point / 0x40000 <= 0xff);
            push_argument_wire_byte(&mut output, (0xf0 + point / 0x40000) as u8)?;
            assert(0x80 + (point / 0x1000) % 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0x80 + (point / 0x1000) % 0x40) as u8)?;
            assert(0x80 + (point / 0x40) % 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0x80 + (point / 0x40) % 0x40) as u8)?;
            assert(0x80 + point % 0x40 <= 0xff);
            push_argument_wire_byte(&mut output, (0x80 + point % 0x40) as u8)?;
        } else {
            return Err(LocalArgumentWireError::InvalidCodePoint);
        }
        index += 1;
    }
    Ok(output)
}

pub fn encode_local_target_arguments(plan: &LocalExecutionPlan) -> (result: Result<
    Vec<u8>,
    LocalArgumentWireError,
>)
    requires
        local_execution_plan_well_formed_spec(plan@),
    ensures
        result is Ok ==> local_target_argument_wire_shape_spec(result.unwrap()@),
{
    let mut output = vstd::slice::slice_to_vec(b"CRUCIBLE-ARGV-V1\n");
    let ghost header = output@;
    append_argument_wire_u64(&mut output, plan.arguments.len() as u64)?;
    assert(output@.len() == 25);
    assert(header == b"CRUCIBLE-ARGV-V1\n"@);
    assert(output@.subrange(0, 17) == b"CRUCIBLE-ARGV-V1\n"@);
    let mut index = 0usize;
    while index < plan.arguments.len()
        invariant
            index <= plan.arguments.len(),
            output@.len() <= MAX_LOCAL_ARGUMENT_WIRE_BYTES,
            output@.len() >= 25,
            output@.subrange(0, 17) == b"CRUCIBLE-ARGV-V1\n"@,
        decreases plan.arguments.len() - index,
    {
        let argument = encode_argument_code_points(plan.arguments[index].as_slice())?;
        let ghost before_length = output@;
        append_argument_wire_u64(&mut output, argument.len() as u64)?;
        assert(output@.subrange(0, before_length.len() as int) == before_length);
        assert(output@.subrange(0, 17) == b"CRUCIBLE-ARGV-V1\n"@);
        let ghost before_argument = output@;
        append_argument_wire_bytes(&mut output, argument.as_slice())?;
        proof {
            lemma_append_preserves_prefix(before_argument, argument@, 17);
        }
        assert(output@.subrange(0, 17) == b"CRUCIBLE-ARGV-V1\n"@);
        index += 1;
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedLocalCapabilityProbe {
    report: Vec<u8>,
    available: bool,
    network_policy: LocalNetworkPolicy,
}

#[verifier::ext_equal]
pub struct ValidatedLocalCapabilityProbeView {
    pub report: Seq<u8>,
    pub available: bool,
    pub network_policy: LocalNetworkPolicy,
}

impl View for ValidatedLocalCapabilityProbe {
    type V = ValidatedLocalCapabilityProbeView;

    closed spec fn view(&self) -> ValidatedLocalCapabilityProbeView {
        ValidatedLocalCapabilityProbeView {
            report: self.report@,
            available: self.available,
            network_policy: self.network_policy,
        }
    }
}

pub open spec fn canonical_local_capability_probe_report_spec(
    plan: LocalExecutionPlanView,
    available: bool,
) -> Seq<u8> {
    match (available, plan.network_policy) {
        (
            true,
            LocalNetworkPolicy::None,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=available\nnetwork=isolated\nworking_directory=private\nenvironment=controlled\nrlimits=applied\nprocess_group=available\n"@,
        (
            true,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=available\nnetwork=shared-by-configuration\nworking_directory=private\nenvironment=controlled\nrlimits=applied\nprocess_group=available\n"@,
        (
            false,
            LocalNetworkPolicy::None,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=unavailable\nnetwork=unavailable\nworking_directory=unavailable\nenvironment=unavailable\nrlimits=unavailable\nprocess_group=unavailable\n"@,
        (
            false,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=unavailable\nnetwork=shared-by-configuration\nworking_directory=unavailable\nenvironment=unavailable\nrlimits=unavailable\nprocess_group=unavailable\n"@,
    }
}

pub open spec fn local_capability_probe_matches_plan_spec(
    probe: ValidatedLocalCapabilityProbeView,
    plan: LocalExecutionPlanView,
) -> bool {
    probe.network_policy == plan.network_policy && probe.report
        == canonical_local_capability_probe_report_spec(plan, probe.available)
}

pub fn canonical_local_capability_probe_report(
    plan: &LocalExecutionPlan,
    available: bool,
) -> (report: Vec<u8>)
    ensures
        report@ == canonical_local_capability_probe_report_spec(plan@, available),
{
    let literal: &[u8] = match (available, plan.network_policy) {
        (
            true,
            LocalNetworkPolicy::None,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=available\nnetwork=isolated\nworking_directory=private\nenvironment=controlled\nrlimits=applied\nprocess_group=available\n",
        (
            true,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=available\nnetwork=shared-by-configuration\nworking_directory=private\nenvironment=controlled\nrlimits=applied\nprocess_group=available\n",
        (
            false,
            LocalNetworkPolicy::None,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=unavailable\nnetwork=unavailable\nworking_directory=unavailable\nenvironment=unavailable\nrlimits=unavailable\nprocess_group=unavailable\n",
        (
            false,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"crucible-linux-capability-probe-v1\nstack=bubblewrap-prlimit\nresult=unavailable\nnetwork=shared-by-configuration\nworking_directory=unavailable\nenvironment=unavailable\nrlimits=unavailable\nprocess_group=unavailable\n",
    };
    vstd::slice::slice_to_vec(literal)
}

fn bytes_equal(left: &[u8], right: &[u8]) -> (equal: bool)
    ensures
        equal == (left@ == right@),
{
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0usize;
    while index < left.len()
        invariant
            index <= left.len(),
            left.len() == right.len(),
            forall|prior: int| 0 <= prior < index ==> left@[prior] == right@[prior],
        decreases left.len() - index,
    {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    assert(left@ =~= right@);
    true
}

pub fn validate_local_capability_probe(plan: &LocalExecutionPlan, report: Vec<u8>) -> (result:
    Result<ValidatedLocalCapabilityProbe, LocalCapabilityProbeError>)
    requires
        local_execution_plan_well_formed_spec(plan@),
    ensures
        match &result {
            Ok(probe) => local_capability_probe_matches_plan_spec(probe@, plan@),
            Err(LocalCapabilityProbeError::ReportMismatch) => !(report@
                == canonical_local_capability_probe_report_spec(plan@, true) || report@
                == canonical_local_capability_probe_report_spec(plan@, false)),
        },
{
    let available_report = canonical_local_capability_probe_report(plan, true);
    if bytes_equal(report.as_slice(), available_report.as_slice()) {
        return Ok(
            ValidatedLocalCapabilityProbe {
                report,
                available: true,
                network_policy: plan.network_policy,
            },
        );
    }
    let unavailable_report = canonical_local_capability_probe_report(plan, false);
    if bytes_equal(report.as_slice(), unavailable_report.as_slice()) {
        return Ok(
            ValidatedLocalCapabilityProbe {
                report,
                available: false,
                network_policy: plan.network_policy,
            },
        );
    }
    Err(LocalCapabilityProbeError::ReportMismatch)
}

impl ValidatedLocalCapabilityProbe {
    pub fn available(&self) -> (available: bool)
        ensures
            available == self@.available,
    {
        self.available
    }

    pub fn report(&self) -> (report: &[u8])
        ensures
            report@ == self@.report,
    {
        self.report.as_slice()
    }
}

pub open spec fn local_capability_manifest_prefix_spec(
    probe: ValidatedLocalCapabilityProbeView,
    plan: LocalExecutionPlanView,
) -> Seq<u8> {
    match (probe.available, plan.network_policy) {
        (
            true,
            LocalNetworkPolicy::None,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":1,\"probe_evidence_artifact\":\""@,
        (
            true,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":1,\"probe_evidence_artifact\":\""@,
        (
            false,
            LocalNetworkPolicy::None,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":0,\"probe_evidence_artifact\":\""@,
        (
            false,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":0,\"probe_evidence_artifact\":\""@,
    }
}

pub open spec fn local_capability_manifest_controls_spec(
    probe: ValidatedLocalCapabilityProbeView,
    plan: LocalExecutionPlanView,
) -> Seq<u8> {
    match (probe.available, plan.network_policy) {
        (
            true,
            LocalNetworkPolicy::None,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"enforced:pipe-drain-and-discard-v1\",\"controlled_environment\":\"enforced:bubblewrap-clearenv-v1\",\"file_size_limit\":\"enforced:prlimit-rlimit-fsize\",\"memory_limit\":\"enforced:prlimit-rlimit-as\",\"network_isolation\":\"enforced:bubblewrap-net-namespace\",\"private_working_directory\":\"enforced:bubblewrap-tmpfs\",\"process_count_limit\":\"enforced:prlimit-rlimit-nproc\",\"process_group_termination\":\"enforced:unix-process-group-kill\",\"resource_limits\":\"enforced:prlimit-v1\",\"wall_clock_timeout\":\"enforced:monotonic-process-group-watchdog-v1\"}}\n"@,
        (
            true,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"enforced:pipe-drain-and-discard-v1\",\"controlled_environment\":\"enforced:bubblewrap-clearenv-v1\",\"file_size_limit\":\"enforced:prlimit-rlimit-fsize\",\"memory_limit\":\"enforced:prlimit-rlimit-as\",\"network_isolation\":\"available-but-disabled:configuration\",\"private_working_directory\":\"enforced:bubblewrap-tmpfs\",\"process_count_limit\":\"enforced:prlimit-rlimit-nproc\",\"process_group_termination\":\"enforced:unix-process-group-kill\",\"resource_limits\":\"enforced:prlimit-v1\",\"wall_clock_timeout\":\"enforced:monotonic-process-group-watchdog-v1\"}}\n"@,
        (
            false,
            LocalNetworkPolicy::None,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"unavailable:probe-failed\",\"controlled_environment\":\"unavailable:probe-failed\",\"file_size_limit\":\"unavailable:probe-failed\",\"memory_limit\":\"unavailable:probe-failed\",\"network_isolation\":\"unavailable:probe-failed\",\"private_working_directory\":\"unavailable:probe-failed\",\"process_count_limit\":\"unavailable:probe-failed\",\"process_group_termination\":\"unavailable:probe-failed\",\"resource_limits\":\"unavailable:probe-failed\",\"wall_clock_timeout\":\"unavailable:probe-failed\"}}\n"@,
        (
            false,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"unavailable:probe-failed\",\"controlled_environment\":\"unavailable:probe-failed\",\"file_size_limit\":\"unavailable:probe-failed\",\"memory_limit\":\"unavailable:probe-failed\",\"network_isolation\":\"available-but-disabled:configuration\",\"private_working_directory\":\"unavailable:probe-failed\",\"process_count_limit\":\"unavailable:probe-failed\",\"process_group_termination\":\"unavailable:probe-failed\",\"resource_limits\":\"unavailable:probe-failed\",\"wall_clock_timeout\":\"unavailable:probe-failed\"}}\n"@,
    }
}

pub open spec fn local_capability_manifest_spec(
    plan: LocalExecutionPlanView,
    probe: ValidatedLocalCapabilityProbeView,
    probe_artifact: crucible_core::artifact::ArtifactRefView,
) -> Seq<u8> {
    local_capability_manifest_prefix_spec(probe, plan) + vstd::utf8::encode_utf8(probe_artifact.id)
        + local_capability_manifest_controls_spec(probe, plan)
}

pub fn local_capability_manifest(
    plan: &LocalExecutionPlan,
    probe: &ValidatedLocalCapabilityProbe,
    probe_artifact: &ArtifactRef,
) -> (manifest: Vec<u8>)
    requires
        local_execution_plan_well_formed_spec(plan@),
        local_capability_probe_matches_plan_spec(probe@, plan@),
    ensures
        manifest@ == local_capability_manifest_spec(plan@, probe@, probe_artifact@),
{
    let literal: &[u8] = match (probe.available, plan.network_policy) {
        (
            true,
            LocalNetworkPolicy::None,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":1,\"probe_evidence_artifact\":\"",
        (
            true,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":1,\"probe_evidence_artifact\":\"",
        (
            false,
            LocalNetworkPolicy::None,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":0,\"probe_evidence_artifact\":\"",
        (
            false,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"{\"schema_version\":1,\"backend\":\"linux-bubblewrap-prlimit-v1\",\"platform\":\"linux\",\"isolation_tier\":0,\"probe_evidence_artifact\":\"",
    };
    let mut manifest = vstd::slice::slice_to_vec(literal);
    let evidence_id = probe_artifact.id.as_str().as_bytes_vec();
    append_manifest_bytes(&mut manifest, evidence_id.as_slice());
    let controls: &[u8] = match (probe.available, plan.network_policy) {
        (
            true,
            LocalNetworkPolicy::None,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"enforced:pipe-drain-and-discard-v1\",\"controlled_environment\":\"enforced:bubblewrap-clearenv-v1\",\"file_size_limit\":\"enforced:prlimit-rlimit-fsize\",\"memory_limit\":\"enforced:prlimit-rlimit-as\",\"network_isolation\":\"enforced:bubblewrap-net-namespace\",\"private_working_directory\":\"enforced:bubblewrap-tmpfs\",\"process_count_limit\":\"enforced:prlimit-rlimit-nproc\",\"process_group_termination\":\"enforced:unix-process-group-kill\",\"resource_limits\":\"enforced:prlimit-v1\",\"wall_clock_timeout\":\"enforced:monotonic-process-group-watchdog-v1\"}}\n",
        (
            true,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"enforced:pipe-drain-and-discard-v1\",\"controlled_environment\":\"enforced:bubblewrap-clearenv-v1\",\"file_size_limit\":\"enforced:prlimit-rlimit-fsize\",\"memory_limit\":\"enforced:prlimit-rlimit-as\",\"network_isolation\":\"available-but-disabled:configuration\",\"private_working_directory\":\"enforced:bubblewrap-tmpfs\",\"process_count_limit\":\"enforced:prlimit-rlimit-nproc\",\"process_group_termination\":\"enforced:unix-process-group-kill\",\"resource_limits\":\"enforced:prlimit-v1\",\"wall_clock_timeout\":\"enforced:monotonic-process-group-watchdog-v1\"}}\n",
        (
            false,
            LocalNetworkPolicy::None,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"unavailable:probe-failed\",\"controlled_environment\":\"unavailable:probe-failed\",\"file_size_limit\":\"unavailable:probe-failed\",\"memory_limit\":\"unavailable:probe-failed\",\"network_isolation\":\"unavailable:probe-failed\",\"private_working_directory\":\"unavailable:probe-failed\",\"process_count_limit\":\"unavailable:probe-failed\",\"process_group_termination\":\"unavailable:probe-failed\",\"resource_limits\":\"unavailable:probe-failed\",\"wall_clock_timeout\":\"unavailable:probe-failed\"}}\n",
        (
            false,
            LocalNetworkPolicy::UnrestrictedHost,
        ) => b"\",\"capabilities\":{\"bounded_output_capture\":\"unavailable:probe-failed\",\"controlled_environment\":\"unavailable:probe-failed\",\"file_size_limit\":\"unavailable:probe-failed\",\"memory_limit\":\"unavailable:probe-failed\",\"network_isolation\":\"available-but-disabled:configuration\",\"private_working_directory\":\"unavailable:probe-failed\",\"process_count_limit\":\"unavailable:probe-failed\",\"process_group_termination\":\"unavailable:probe-failed\",\"resource_limits\":\"unavailable:probe-failed\",\"wall_clock_timeout\":\"unavailable:probe-failed\"}}\n",
    };
    append_manifest_bytes(&mut manifest, controls);
    manifest
}

fn append_manifest_bytes(output: &mut Vec<u8>, value: &[u8])
    ensures
        final(output)@ == old(output)@ + value@,
{
    let ghost initial = output@;
    let mut index = 0usize;
    while index < value.len()
        invariant
            index <= value.len(),
            output@ == initial + value@.subrange(0, index as int),
        decreases value.len() - index,
    {
        output.push(value[index]);
        index += 1;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalRuntimeIdentity {
    platform: Vec<u8>,
    architecture: Vec<u8>,
    kernel_release: Vec<u8>,
    bubblewrap_version: Vec<u8>,
    prlimit_version: Vec<u8>,
    harness_artifact: ArtifactRef,
    bubblewrap_artifact: ArtifactRef,
    prlimit_artifact: ArtifactRef,
}

#[verifier::ext_equal]
pub struct LocalRuntimeIdentityView {
    pub platform: Seq<u8>,
    pub architecture: Seq<u8>,
    pub kernel_release: Seq<u8>,
    pub bubblewrap_version: Seq<u8>,
    pub prlimit_version: Seq<u8>,
    pub harness_artifact: crucible_core::artifact::ArtifactRefView,
    pub bubblewrap_artifact: crucible_core::artifact::ArtifactRefView,
    pub prlimit_artifact: crucible_core::artifact::ArtifactRefView,
}

impl View for LocalRuntimeIdentity {
    type V = LocalRuntimeIdentityView;

    closed spec fn view(&self) -> LocalRuntimeIdentityView {
        LocalRuntimeIdentityView {
            platform: self.platform@,
            architecture: self.architecture@,
            kernel_release: self.kernel_release@,
            bubblewrap_version: self.bubblewrap_version@,
            prlimit_version: self.prlimit_version@,
            harness_artifact: self.harness_artifact@,
            bubblewrap_artifact: self.bubblewrap_artifact@,
            prlimit_artifact: self.prlimit_artifact@,
        }
    }
}

pub open spec fn local_runtime_identity_well_formed_spec(
    identity: LocalRuntimeIdentityView,
) -> bool {
    0 < identity.platform.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < identity.architecture.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < identity.kernel_release.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < identity.bubblewrap_version.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES && 0
        < identity.prlimit_version.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES
}

fn local_runtime_text_is_safe(value: &[u8]) -> (safe: bool) {
    if value.is_empty() || value.len() as u64 > MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES {
        return false;
    }
    let mut index = 0usize;
    while index < value.len()
        invariant
            index <= value.len(),
        decreases value.len() - index,
    {
        let byte = value[index];
        let printable_ascii = byte >= 0x20 && byte != 0x7f && byte & 0x80 == 0;
        if !printable_ascii || byte == b'=' {
            return false;
        }
        index += 1;
    }
    true
}

impl LocalRuntimeIdentity {
    #[expect(
        clippy::too_many_arguments,
        reason = "runtime identity keeps each behaviorally relevant tool and platform field explicit"
    )]
    pub fn new(
        platform: String,
        architecture: String,
        kernel_release: String,
        bubblewrap_version: String,
        prlimit_version: String,
        harness_artifact: ArtifactRef,
        bubblewrap_artifact: ArtifactRef,
        prlimit_artifact: ArtifactRef,
    ) -> (result: Result<Self, LocalRuntimeIdentityError>)
        ensures
            result is Ok ==> local_runtime_identity_well_formed_spec(result.unwrap()@),
    {
        let platform = platform.as_str().as_bytes_vec();
        let architecture = architecture.as_str().as_bytes_vec();
        let kernel_release = kernel_release.as_str().as_bytes_vec();
        let bubblewrap_version = bubblewrap_version.as_str().as_bytes_vec();
        let prlimit_version = prlimit_version.as_str().as_bytes_vec();
        validate_local_runtime_text(platform.as_slice())?;
        validate_local_runtime_text(architecture.as_slice())?;
        validate_local_runtime_text(kernel_release.as_slice())?;
        validate_local_runtime_text(bubblewrap_version.as_slice())?;
        validate_local_runtime_text(prlimit_version.as_slice())?;
        Ok(
            Self {
                platform,
                architecture,
                kernel_release,
                bubblewrap_version,
                prlimit_version,
                harness_artifact,
                bubblewrap_artifact,
                prlimit_artifact,
            },
        )
    }

    pub fn harness_artifact(&self) -> (artifact: &ArtifactRef)
        ensures
            artifact@ == self@.harness_artifact,
    {
        &self.harness_artifact
    }

    pub fn bubblewrap_artifact(&self) -> (artifact: &ArtifactRef)
        ensures
            artifact@ == self@.bubblewrap_artifact,
    {
        &self.bubblewrap_artifact
    }

    pub fn prlimit_artifact(&self) -> (artifact: &ArtifactRef)
        ensures
            artifact@ == self@.prlimit_artifact,
    {
        &self.prlimit_artifact
    }
}

fn validate_local_runtime_text(value: &[u8]) -> (result: Result<(), LocalRuntimeIdentityError>)
    ensures
        result is Ok ==> 0 < value@.len() <= MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES,
{
    if value.is_empty() {
        return Err(LocalRuntimeIdentityError::EmptyField);
    }
    if value.len() as u64 > MAX_LOCAL_RUNTIME_IDENTITY_TEXT_BYTES {
        return Err(LocalRuntimeIdentityError::FieldTooLong);
    }
    if !local_runtime_text_is_safe(value) {
        return Err(LocalRuntimeIdentityError::InvalidField);
    }
    Ok(())
}

pub open spec fn manifest_with_field_spec(output: Seq<u8>, name: Seq<u8>, value: Seq<u8>) -> Seq<
    u8,
> {
    (output + name + value).push(b'\n')
}

fn append_manifest_field(output: &mut Vec<u8>, name: &[u8], value: &[u8])
    ensures
        final(output)@ == manifest_with_field_spec(old(output)@, name@, value@),
{
    let ghost initial = output@;
    append_manifest_bytes(output, name);
    append_manifest_bytes(output, value);
    output.push(b'\n');
    assert(output@ == manifest_with_field_spec(initial, name@, value@));
}

pub open spec fn manifest_with_artifact_spec(
    output: Seq<u8>,
    name: Seq<u8>,
    artifact: crucible_core::artifact::ArtifactRefView,
) -> Seq<u8> {
    (output + name + vstd::utf8::encode_utf8(artifact.id)).push(b'\n')
}

fn append_manifest_artifact(output: &mut Vec<u8>, name: &[u8], artifact: &ArtifactRef)
    ensures
        final(output)@ == manifest_with_artifact_spec(old(output)@, name@, artifact@),
{
    let ghost initial = output@;
    append_manifest_bytes(output, name);
    let bytes = artifact.id.as_str().as_bytes_vec();
    append_manifest_bytes(output, bytes.as_slice());
    output.push(b'\n');
    assert(output@ == manifest_with_artifact_spec(initial, name@, artifact@));
}

pub open spec fn target_build_manifest_spec(
    target: crucible_core::artifact::ArtifactRefView,
    runtime: LocalRuntimeIdentityView,
) -> Seq<u8> {
    let manifest =
        b"crucible-target-build-manifest-v1\nadapter=cli-materialized-executable-v1\ntarget_artifact="@;
    let manifest = manifest + vstd::utf8::encode_utf8(target.id);
    let manifest = manifest + b"\n"@;
    let manifest = manifest_with_field_spec(manifest, b"platform="@, runtime.platform);
    let manifest = manifest_with_field_spec(manifest, b"architecture="@, runtime.architecture);
    let manifest = manifest_with_field_spec(manifest, b"kernel_release="@, runtime.kernel_release);
    let manifest = manifest_with_field_spec(
        manifest,
        b"bubblewrap_version="@,
        runtime.bubblewrap_version,
    );
    let manifest = manifest_with_field_spec(
        manifest,
        b"prlimit_version="@,
        runtime.prlimit_version,
    );
    let manifest = manifest_with_artifact_spec(
        manifest,
        b"harness_artifact="@,
        runtime.harness_artifact,
    );
    let manifest = manifest_with_artifact_spec(
        manifest,
        b"bubblewrap_artifact="@,
        runtime.bubblewrap_artifact,
    );
    let manifest = manifest_with_artifact_spec(
        manifest,
        b"prlimit_artifact="@,
        runtime.prlimit_artifact,
    );
    manifest + b"mount_policy=minimal-host-usr-read-only-v1\nunresolved_host_runtime=true\n"@
}

pub fn target_build_manifest(target: &ArtifactRef, runtime: &LocalRuntimeIdentity) -> (manifest:
    Vec<u8>)
    requires
        local_runtime_identity_well_formed_spec(runtime@),
    ensures
        manifest@ == target_build_manifest_spec(target@, runtime@),
{
    let mut manifest = vstd::slice::slice_to_vec(
        b"crucible-target-build-manifest-v1\nadapter=cli-materialized-executable-v1\ntarget_artifact=",
    );
    let id = target.id.as_str().as_bytes_vec();
    append_manifest_bytes(&mut manifest, id.as_slice());
    append_manifest_bytes(&mut manifest, b"\n");
    append_manifest_field(&mut manifest, b"platform=", runtime.platform.as_slice());
    append_manifest_field(&mut manifest, b"architecture=", runtime.architecture.as_slice());
    append_manifest_field(&mut manifest, b"kernel_release=", runtime.kernel_release.as_slice());
    append_manifest_field(
        &mut manifest,
        b"bubblewrap_version=",
        runtime.bubblewrap_version.as_slice(),
    );
    append_manifest_field(&mut manifest, b"prlimit_version=", runtime.prlimit_version.as_slice());
    append_manifest_artifact(&mut manifest, b"harness_artifact=", &runtime.harness_artifact);
    append_manifest_artifact(&mut manifest, b"bubblewrap_artifact=", &runtime.bubblewrap_artifact);
    append_manifest_artifact(&mut manifest, b"prlimit_artifact=", &runtime.prlimit_artifact);
    append_manifest_bytes(
        &mut manifest,
        b"mount_policy=minimal-host-usr-read-only-v1\nunresolved_host_runtime=true\n",
    );
    assert(manifest@ =~= target_build_manifest_spec(target@, runtime@));
    manifest
}

pub fn prepare_local_execution(configuration: &EffectiveExecutionConfiguration) -> (result: Result<
    LocalExecutionPlan,
    LocalRunPlanError,
>)
    requires
        crate::effective_execution_configuration_well_formed_spec(configuration@),
    ensures
        match &result {
            Ok(plan) => local_execution_plan_well_formed_spec(plan@)
                && local_execution_plan_matches_configuration_spec(configuration@, plan@),
            Err(_) => true,
        },
{
    #[cfg(not(target_os = "linux"))]
    {
        let _ = configuration;
        return Err(LocalRunPlanError::UnsupportedPlatform);
    }

    let memory_mb = configuration.memory_mb();
    if !code_points_equal_ascii(configuration.storage_root(), b".crucible") {
        return Err(LocalRunPlanError::UnsupportedStorageLayout);
    }
    if memory_mb > u64::MAX / BYTES_PER_MEBIBYTE {
        return Err(LocalRunPlanError::ArithmeticOverflow);
    }
    let output_mb = configuration.max_output_mb();
    if output_mb > u64::MAX / BYTES_PER_MEBIBYTE {
        return Err(LocalRunPlanError::ArithmeticOverflow);
    }
    let memory_bytes = memory_mb * BYTES_PER_MEBIBYTE;
    let max_stream_bytes = output_mb * BYTES_PER_MEBIBYTE;
    if max_stream_bytes > MAX_LOCAL_ARTIFACT_BYTES {
        return Err(LocalRunPlanError::OutputLimitTooLarge);
    }
    let network_enabled = configuration.network_enabled();
    let capabilities = configuration.required_capabilities();
    let mut index = 0usize;
    while index < capabilities.len()
        invariant
            index <= capabilities.len(),
            forall|prior: int|
                0 <= prior < index ==> local_capability_supported_spec(
                    crate::configuration::configuration_text_sequence_views_spec(
                        capabilities@,
                    )[prior],
                    network_enabled,
                ),
        decreases capabilities.len() - index,
    {
        proof {
            reveal(crate::configuration::configuration_text_sequence_views_spec);
            assert(crate::configuration::configuration_text_sequence_views_spec(
                capabilities@,
            )[index as int] == capabilities@[index as int]@);
        }
        if !local_capability_supported(capabilities[index].as_slice(), network_enabled) {
            return Err(LocalRunPlanError::RequiredCapabilityUnavailable { index: index as u64 });
        }
        assert(local_capability_supported_spec(
            crate::configuration::configuration_text_sequence_views_spec(
                capabilities@,
            )[index as int],
            network_enabled,
        ));
        index += 1;
    }
    let network_policy = if network_enabled {
        LocalNetworkPolicy::UnrestrictedHost
    } else {
        LocalNetworkPolicy::None
    };
    let plan = LocalExecutionPlan {
        command: clone_code_points(configuration.target_command()),
        arguments: clone_code_point_sequences(configuration.target_arguments()),
        timeout_ms: configuration.timeout_ms(),
        memory_bytes,
        max_processes: configuration.max_processes(),
        max_stream_bytes,
        network_policy,
        backend: LocalExecutionBackend::LinuxBubblewrapPrlimitV1,
        output_capture_policy: OutputCapturePolicy::DrainAndDiscard,
    };
    assert(all_local_capabilities_supported_spec(
        crate::configuration::configuration_text_sequence_views_spec(capabilities@),
        network_enabled,
    ));
    assert(configuration@.required_capabilities
        == crate::configuration::configuration_text_sequence_views_spec(capabilities@));
    assert(network_enabled == configuration@.network_enabled);
    assert(local_execution_plan_well_formed_spec(plan@));
    assert(local_execution_plan_matches_configuration_spec(configuration@, plan@));
    Ok(plan)
}

pub fn build_local_raw_observation(
    run_id: RunId,
    attempt_id: RunAttemptId,
    evidence: &LocalExecutionEvidence,
    stdout_artifact: ArtifactRef,
    stderr_artifact: ArtifactRef,
) -> (result: Result<ValidatedRawObservation, LocalObservationError>)
    ensures
        result is Ok ==> crucible_core::observation::raw_observation_semantics_spec(
            result.unwrap()@,
        ),
{
    if stdout_artifact.size_bytes != evidence.stdout.retained.len() as u64
        || stdout_artifact.media_type.is_some() {
        return Err(LocalObservationError::StdoutArtifactMismatch);
    }
    if stderr_artifact.size_bytes != evidence.stderr.retained.len() as u64
        || stderr_artifact.media_type.is_some() {
        return Err(LocalObservationError::StderrArtifactMismatch);
    }
    let mut events = Vec::new();
    let termination = match evidence.termination {
        LocalTermination::ExitCode(code) => TerminationRecord::ExitCode { code },
        LocalTermination::UnixSignal { signal, core_dumped } => {
            TerminationRecord::UnixSignal { signal, core_dumped }
        },
        LocalTermination::Timeout => {
            events.push(RawExecutionEvent::TimeoutThresholdReached);
            TerminationRecord::HarnessTerminated { reason: HarnessTerminationReason::Timeout }
        },
    };
    let outcome = RawExecutionOutcome::new(
        CompletionDisposition::Completed,
        Some(termination),
        events,
    );
    let wall_time = match RecordedDuration::new(evidence.wall_seconds, evidence.wall_nanoseconds) {
        Ok(value) => value,
        Err(_) => return Err(LocalObservationError::InvalidDuration),
    };
    let stdout = CapturedStreamRef::new(
        stdout_artifact,
        evidence.stdout.discarded > 0,
        evidence.stdout.retained.len() as u64,
        evidence.stdout.discarded,
    );
    let stderr = CapturedStreamRef::new(
        stderr_artifact,
        evidence.stderr.discarded > 0,
        evidence.stderr.retained.len() as u64,
        evidence.stderr.discarded,
    );
    let resources = ResourceSnapshot::new(None, None, None, None, None, None, Vec::new());
    let observation = RawObservation::new(
        run_id,
        attempt_id,
        outcome,
        stdout,
        stderr,
        wall_time,
        None,
        None,
        resources,
        None,
        None,
        None,
        None,
        Vec::new(),
    );
    let limits = canonical_raw_observation_limits();
    match validate_raw_observation(observation, limits) {
        Ok(validated) => {
            proof {
                reveal(crucible_core::observation::raw_observation_semantics_spec);
                assert(crucible_core::observation::raw_observation_semantics_spec(validated@));
            }
            Ok(validated)
        },
        Err(_) => Err(LocalObservationError::InvalidObservation),
    }
}

pub fn prepare_local_cli_target_instance(
    plan: &LocalExecutionPlan,
    target_id: TargetId,
    target_build_id: TargetBuildId,
    owner_attempt_id: RunAttemptId,
    instance_ordinal: u64,
) -> (result: Result<TargetInstanceLifecycle, TargetLifecycleError>)
    requires
        local_execution_plan_well_formed_spec(plan@),
    ensures
        match &result {
            Ok(instance) => {
                crucible_core::target_adapter::target_instance_lifecycle_well_formed_spec(instance@)
                    && instance@.adapter.kind == TargetAdapterKind::Cli && instance@.adapter.version
                    == 1 && instance@.target_id == target_id@ && instance@.target_build_id
                    == target_build_id@ && instance@.owner_attempt_id == owner_attempt_id@
                    && instance@.instance_ordinal == instance_ordinal && instance@.state
                    == crucible_core::TargetLifecycleState::Prepared
            },
            Err(_) => true,
        },
{
    let adapter_kind = match plan.backend() {
        LocalExecutionBackend::LinuxBubblewrapPrlimitV1 => TargetAdapterKind::Cli,
    };
    let adapter = TargetAdapterIdentity::new(adapter_kind, 1)?;
    let allocated = TargetInstanceLifecycle::new(
        adapter,
        target_id,
        target_build_id,
        owner_attempt_id,
        instance_ordinal,
    )?;
    crucible_core::advance_target_instance_lifecycle(
        allocated,
        TargetLifecycleAction::PrepareSucceeded,
    )
}

} // verus!
