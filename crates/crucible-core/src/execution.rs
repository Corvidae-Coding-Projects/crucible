use crate::artifact::{ArtifactIdParseError, ArtifactRef, ArtifactRefView, ContentDigest};
use crate::parse_artifact_id;
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

verus! {

pub const RAW_EXECUTION_OUTCOME_SCHEMA_VERSION: u16 = 1;

pub const MAX_RAW_EXECUTION_EVENTS: u64 = 1_048_576;

pub const MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS: u64 = 1_048_576;

pub const MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS: u64 = 1_048_576;

pub const MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD: u64 = 1_099_511_627_776;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawExecutionOutcomeLimits {
    max_events: u64,
    max_extension_namespace_code_points: u64,
    max_extension_media_type_code_points: u64,
    max_extension_payload_bytes_per_record: u64,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeLimitsView {
    pub max_events: u64,
    pub max_extension_namespace_code_points: u64,
    pub max_extension_media_type_code_points: u64,
    pub max_extension_payload_bytes_per_record: u64,
}

impl View for RawExecutionOutcomeLimits {
    type V = RawExecutionOutcomeLimitsView;

    closed spec fn view(&self) -> RawExecutionOutcomeLimitsView {
        RawExecutionOutcomeLimitsView {
            max_events: self.max_events,
            max_extension_namespace_code_points: self.max_extension_namespace_code_points,
            max_extension_media_type_code_points: self.max_extension_media_type_code_points,
            max_extension_payload_bytes_per_record: self.max_extension_payload_bytes_per_record,
        }
    }
}

impl RawExecutionOutcomeLimits {
    pub fn new(
        max_events: u64,
        max_extension_namespace_code_points: u64,
        max_extension_media_type_code_points: u64,
        max_extension_payload_bytes_per_record: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (RawExecutionOutcomeLimitsView {
                max_events,
                max_extension_namespace_code_points,
                max_extension_media_type_code_points,
                max_extension_payload_bytes_per_record,
            }),
    {
        Self {
            max_events,
            max_extension_namespace_code_points,
            max_extension_media_type_code_points,
            max_extension_payload_bytes_per_record,
        }
    }

    pub fn max_events(&self) -> (value: u64)
        ensures
            value == self@.max_events,
    {
        self.max_events
    }

    pub fn max_extension_namespace_code_points(&self) -> (value: u64)
        ensures
            value == self@.max_extension_namespace_code_points,
    {
        self.max_extension_namespace_code_points
    }

    pub fn max_extension_media_type_code_points(&self) -> (value: u64)
        ensures
            value == self@.max_extension_media_type_code_points,
    {
        self.max_extension_media_type_code_points
    }

    pub fn max_extension_payload_bytes_per_record(&self) -> (value: u64)
        ensures
            value == self@.max_extension_payload_bytes_per_record,
    {
        self.max_extension_payload_bytes_per_record
    }
}

pub fn canonical_raw_execution_outcome_limits() -> (limits: RawExecutionOutcomeLimits)
    ensures
        limits@ == (RawExecutionOutcomeLimitsView {
            max_events: MAX_RAW_EXECUTION_EVENTS,
            max_extension_namespace_code_points: MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            max_extension_media_type_code_points:
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            max_extension_payload_bytes_per_record:
                MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        }),
{
    RawExecutionOutcomeLimits::new(
        MAX_RAW_EXECUTION_EVENTS,
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    )
}

pub open spec fn effective_raw_execution_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_raw_execution_limit(requested: u64, absolute: u64) -> (effective: u64)
    ensures
        effective == effective_raw_execution_limit_spec(requested, absolute),
        effective <= absolute,
        effective <= requested,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompletionDisposition {
    Completed,
    Cancelled,
    Incomplete,
}

pub open spec fn completion_disposition_stable_tag_spec(value: CompletionDisposition) -> u16 {
    match value {
        CompletionDisposition::Completed => 1,
        CompletionDisposition::Cancelled => 2,
        CompletionDisposition::Incomplete => 3,
    }
}

impl CompletionDisposition {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == completion_disposition_stable_tag_spec(self),
    {
        match self {
            CompletionDisposition::Completed => 1,
            CompletionDisposition::Cancelled => 2,
            CompletionDisposition::Incomplete => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResetCause {
    PowerOn,
    Watchdog,
    Software,
    Brownout,
    External,
    Unknown,
}

pub open spec fn reset_cause_stable_tag_spec(value: ResetCause) -> u16 {
    match value {
        ResetCause::PowerOn => 1,
        ResetCause::Watchdog => 2,
        ResetCause::Software => 3,
        ResetCause::Brownout => 4,
        ResetCause::External => 5,
        ResetCause::Unknown => 6,
    }
}

impl ResetCause {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == reset_cause_stable_tag_spec(self),
    {
        match self {
            ResetCause::PowerOn => 1,
            ResetCause::Watchdog => 2,
            ResetCause::Software => 3,
            ResetCause::Brownout => 4,
            ResetCause::External => 5,
            ResetCause::Unknown => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HarnessTerminationReason {
    Timeout,
    Cancellation,
    CpuTimeLimit,
    MemoryLimit,
    ProcessCountLimit,
    FileSizeLimit,
    OutputLimit,
    CleanupFailure,
}

pub open spec fn harness_termination_reason_stable_tag_spec(
    value: HarnessTerminationReason,
) -> u16 {
    match value {
        HarnessTerminationReason::Timeout => 1,
        HarnessTerminationReason::Cancellation => 2,
        HarnessTerminationReason::CpuTimeLimit => 3,
        HarnessTerminationReason::MemoryLimit => 4,
        HarnessTerminationReason::ProcessCountLimit => 5,
        HarnessTerminationReason::FileSizeLimit => 6,
        HarnessTerminationReason::OutputLimit => 7,
        HarnessTerminationReason::CleanupFailure => 8,
    }
}

impl HarnessTerminationReason {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == harness_termination_reason_stable_tag_spec(self),
    {
        match self {
            HarnessTerminationReason::Timeout => 1,
            HarnessTerminationReason::Cancellation => 2,
            HarnessTerminationReason::CpuTimeLimit => 3,
            HarnessTerminationReason::MemoryLimit => 4,
            HarnessTerminationReason::ProcessCountLimit => 5,
            HarnessTerminationReason::FileSizeLimit => 6,
            HarnessTerminationReason::OutputLimit => 7,
            HarnessTerminationReason::CleanupFailure => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceKind {
    WallTime,
    CpuTime,
    Memory,
    ProcessCount,
    FileSize,
    StandardOutput,
    StandardError,
}

pub open spec fn resource_kind_stable_tag_spec(value: ResourceKind) -> u16 {
    match value {
        ResourceKind::WallTime => 1,
        ResourceKind::CpuTime => 2,
        ResourceKind::Memory => 3,
        ResourceKind::ProcessCount => 4,
        ResourceKind::FileSize => 5,
        ResourceKind::StandardOutput => 6,
        ResourceKind::StandardError => 7,
    }
}

impl ResourceKind {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == resource_kind_stable_tag_spec(self),
    {
        match self {
            ResourceKind::WallTime => 1,
            ResourceKind::CpuTime => 2,
            ResourceKind::Memory => 3,
            ResourceKind::ProcessCount => 4,
            ResourceKind::FileSize => 5,
            ResourceKind::StandardOutput => 6,
            ResourceKind::StandardError => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LogicalProcessIdError {
    Zero,
}

impl LogicalProcessIdError {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == 1,
    {
        match self {
            LogicalProcessIdError::Zero => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct LogicalProcessId(u64);

impl View for LogicalProcessId {
    type V = u64;

    closed spec fn view(&self) -> u64 {
        self.0
    }
}

impl LogicalProcessId {
    pub fn new(value: u64) -> (result: Result<Self, LogicalProcessIdError>)
        ensures
            match result {
                Ok(id) => value > 0 && id@ == value,
                Err(LogicalProcessIdError::Zero) => value == 0,
            },
    {
        if value == 0 {
            Err(LogicalProcessIdError::Zero)
        } else {
            Ok(Self(value))
        }
    }

    pub fn value(&self) -> (value: u64)
        ensures
            value == self@,
    {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VersionedExtensionRef {
    namespace: String,
    schema_version: u32,
    payload: ArtifactRef,
}

#[verifier::ext_equal]
pub struct VersionedExtensionRefView {
    pub namespace: Seq<char>,
    pub schema_version: u32,
    pub payload: ArtifactRefView,
}

impl View for VersionedExtensionRef {
    type V = VersionedExtensionRefView;

    closed spec fn view(&self) -> VersionedExtensionRefView {
        VersionedExtensionRefView {
            namespace: self.namespace@,
            schema_version: self.schema_version,
            payload: self.payload@,
        }
    }
}

impl VersionedExtensionRef {
    pub fn new(namespace: String, schema_version: u32, payload: ArtifactRef) -> (extension: Self)
        ensures
            extension@ == (VersionedExtensionRefView {
                namespace: namespace@,
                schema_version,
                payload: payload@,
            }),
    {
        Self { namespace, schema_version, payload }
    }

    pub fn namespace(&self) -> (value: &str)
        ensures
            value@ == self@.namespace,
    {
        self.namespace.as_str()
    }

    pub fn schema_version(&self) -> (value: u32)
        ensures
            value == self@.schema_version,
    {
        self.schema_version
    }

    pub fn payload(&self) -> (value: &ArtifactRef)
        ensures
            value@ == self@.payload,
    {
        &self.payload
    }
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminationRecord {
    ExitCode { code: i64 },
    UnixSignal { signal: i32, core_dumped: bool },
    WindowsException { status: u32 },
    EmbeddedReset { cause: ResetCause },
    HarnessTerminated { reason: HarnessTerminationReason },
    PlatformSpecific(VersionedExtensionRef),
}

#[verifier::ext_equal]
pub enum TerminationRecordView {
    ExitCode { code: i64 },
    UnixSignal { signal: i32, core_dumped: bool },
    WindowsException { status: u32 },
    EmbeddedReset { cause: ResetCause },
    HarnessTerminated { reason: HarnessTerminationReason },
    PlatformSpecific(VersionedExtensionRefView),
}

impl View for TerminationRecord {
    type V = TerminationRecordView;

    open spec fn view(&self) -> TerminationRecordView {
        match self {
            TerminationRecord::ExitCode { code } => TerminationRecordView::ExitCode { code: *code },
            TerminationRecord::UnixSignal { signal, core_dumped } => {
                TerminationRecordView::UnixSignal { signal: *signal, core_dumped: *core_dumped }
            },
            TerminationRecord::WindowsException { status } => {
                TerminationRecordView::WindowsException { status: *status }
            },
            TerminationRecord::EmbeddedReset { cause } => {
                TerminationRecordView::EmbeddedReset { cause: *cause }
            },
            TerminationRecord::HarnessTerminated { reason } => {
                TerminationRecordView::HarnessTerminated { reason: *reason }
            },
            TerminationRecord::PlatformSpecific(extension) => {
                TerminationRecordView::PlatformSpecific(extension@)
            },
        }
    }
}

pub open spec fn termination_record_stable_tag_spec(value: TerminationRecordView) -> u16 {
    match value {
        TerminationRecordView::ExitCode { .. } => 1,
        TerminationRecordView::UnixSignal { .. } => 2,
        TerminationRecordView::WindowsException { .. } => 3,
        TerminationRecordView::EmbeddedReset { .. } => 4,
        TerminationRecordView::HarnessTerminated { .. } => 5,
        TerminationRecordView::PlatformSpecific(_) => 6,
    }
}

impl TerminationRecord {
    pub fn stable_tag(&self) -> (tag: u16)
        ensures
            tag == termination_record_stable_tag_spec(self@),
    {
        match self {
            TerminationRecord::ExitCode { .. } => 1,
            TerminationRecord::UnixSignal { .. } => 2,
            TerminationRecord::WindowsException { .. } => 3,
            TerminationRecord::EmbeddedReset { .. } => 4,
            TerminationRecord::HarnessTerminated { .. } => 5,
            TerminationRecord::PlatformSpecific(_) => 6,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawExecutionEvent {
    TimeoutThresholdReached,
    ResourceThresholdReached { resource: ResourceKind },
    DeadlockSuspected,
    LivelockSuspected,
    WatchdogTriggered,
    ProcessCreated { logical_process: LogicalProcessId },
    ProcessExited { logical_process: LogicalProcessId },
    PlatformSpecific(VersionedExtensionRef),
}

#[verifier::ext_equal]
pub enum RawExecutionEventView {
    TimeoutThresholdReached,
    ResourceThresholdReached { resource: ResourceKind },
    DeadlockSuspected,
    LivelockSuspected,
    WatchdogTriggered,
    ProcessCreated { logical_process: u64 },
    ProcessExited { logical_process: u64 },
    PlatformSpecific(VersionedExtensionRefView),
}

impl View for RawExecutionEvent {
    type V = RawExecutionEventView;

    open spec fn view(&self) -> RawExecutionEventView {
        match self {
            RawExecutionEvent::TimeoutThresholdReached => {
                RawExecutionEventView::TimeoutThresholdReached
            },
            RawExecutionEvent::ResourceThresholdReached { resource } => {
                RawExecutionEventView::ResourceThresholdReached { resource: *resource }
            },
            RawExecutionEvent::DeadlockSuspected => RawExecutionEventView::DeadlockSuspected,
            RawExecutionEvent::LivelockSuspected => RawExecutionEventView::LivelockSuspected,
            RawExecutionEvent::WatchdogTriggered => RawExecutionEventView::WatchdogTriggered,
            RawExecutionEvent::ProcessCreated { logical_process } => {
                RawExecutionEventView::ProcessCreated { logical_process: logical_process@ }
            },
            RawExecutionEvent::ProcessExited { logical_process } => {
                RawExecutionEventView::ProcessExited { logical_process: logical_process@ }
            },
            RawExecutionEvent::PlatformSpecific(extension) => {
                RawExecutionEventView::PlatformSpecific(extension@)
            },
        }
    }
}

pub open spec fn raw_execution_event_stable_tag_spec(value: RawExecutionEventView) -> u16 {
    match value {
        RawExecutionEventView::TimeoutThresholdReached => 1,
        RawExecutionEventView::ResourceThresholdReached { .. } => 2,
        RawExecutionEventView::DeadlockSuspected => 3,
        RawExecutionEventView::LivelockSuspected => 4,
        RawExecutionEventView::WatchdogTriggered => 5,
        RawExecutionEventView::ProcessCreated { .. } => 6,
        RawExecutionEventView::ProcessExited { .. } => 7,
        RawExecutionEventView::PlatformSpecific(_) => 8,
    }
}

impl RawExecutionEvent {
    pub fn stable_tag(&self) -> (tag: u16)
        ensures
            tag == raw_execution_event_stable_tag_spec(self@),
    {
        match self {
            RawExecutionEvent::TimeoutThresholdReached => 1,
            RawExecutionEvent::ResourceThresholdReached { .. } => 2,
            RawExecutionEvent::DeadlockSuspected => 3,
            RawExecutionEvent::LivelockSuspected => 4,
            RawExecutionEvent::WatchdogTriggered => 5,
            RawExecutionEvent::ProcessCreated { .. } => 6,
            RawExecutionEvent::ProcessExited { .. } => 7,
            RawExecutionEvent::PlatformSpecific(_) => 8,
        }
    }
}

pub open spec fn raw_execution_event_views_spec(events: Seq<RawExecutionEvent>) -> Seq<
    RawExecutionEventView,
> {
    Seq::new(events.len(), |index: int| events[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawExecutionOutcome {
    completion: CompletionDisposition,
    termination: Option<TerminationRecord>,
    events: Vec<RawExecutionEvent>,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeView {
    pub completion: CompletionDisposition,
    pub termination: Option<TerminationRecordView>,
    pub events: Seq<RawExecutionEventView>,
}

impl View for RawExecutionOutcome {
    type V = RawExecutionOutcomeView;

    closed spec fn view(&self) -> RawExecutionOutcomeView {
        RawExecutionOutcomeView {
            completion: self.completion,
            termination: match &self.termination {
                Some(termination) => Some(termination@),
                None => None,
            },
            events: raw_execution_event_views_spec(self.events@),
        }
    }
}

impl RawExecutionOutcome {
    pub fn new(
        completion: CompletionDisposition,
        termination: Option<TerminationRecord>,
        events: Vec<RawExecutionEvent>,
    ) -> (outcome: Self)
        ensures
            outcome@ == (RawExecutionOutcomeView {
                completion,
                termination: match &termination {
                    Some(termination) => Some(termination@),
                    None => None,
                },
                events: raw_execution_event_views_spec(events@),
            }),
    {
        Self { completion, termination, events }
    }

    pub fn completion(&self) -> (value: CompletionDisposition)
        ensures
            value == self@.completion,
    {
        self.completion
    }

    pub fn termination(&self) -> (value: &Option<TerminationRecord>)
        ensures
            match (value, self@.termination) {
                (Some(actual), Some(expected)) => actual@ == expected,
                (None, None) => true,
                _ => false,
            },
    {
        &self.termination
    }

    pub fn events(&self) -> (value: &[RawExecutionEvent])
        ensures
            raw_execution_event_views_spec(value@) == self@.events,
    {
        self.events.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawExecutionOutcomeLocation {
    Termination,
    Event(u64),
}

pub open spec fn raw_execution_outcome_location_stable_tag_spec(
    value: RawExecutionOutcomeLocation,
) -> u16 {
    match value {
        RawExecutionOutcomeLocation::Termination => 1,
        RawExecutionOutcomeLocation::Event(_) => 2,
    }
}

impl RawExecutionOutcomeLocation {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == raw_execution_outcome_location_stable_tag_spec(self),
    {
        match self {
            RawExecutionOutcomeLocation::Termination => 1,
            RawExecutionOutcomeLocation::Event(_) => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawExecutionOutcomeErrorKind {
    EventLimitExceeded,
    ExtensionNamespaceLimitExceeded,
    ExtensionMediaTypeLimitExceeded,
    ExtensionPayloadLimitExceeded,
    EmptyExtensionNamespace,
    ZeroExtensionSchemaVersion,
    MalformedExtensionArtifact,
    UnsupportedExtensionArtifactAlgorithm,
    EmptyExtensionMediaType,
    InvalidUnixSignal,
    InvalidLogicalProcessId,
}

pub open spec fn raw_execution_outcome_error_kind_stable_tag_spec(
    value: RawExecutionOutcomeErrorKind,
) -> u16 {
    match value {
        RawExecutionOutcomeErrorKind::EventLimitExceeded => 1,
        RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded => 2,
        RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded => 3,
        RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded => 4,
        RawExecutionOutcomeErrorKind::EmptyExtensionNamespace => 5,
        RawExecutionOutcomeErrorKind::ZeroExtensionSchemaVersion => 6,
        RawExecutionOutcomeErrorKind::MalformedExtensionArtifact => 7,
        RawExecutionOutcomeErrorKind::UnsupportedExtensionArtifactAlgorithm => 8,
        RawExecutionOutcomeErrorKind::EmptyExtensionMediaType => 9,
        RawExecutionOutcomeErrorKind::InvalidUnixSignal => 10,
        RawExecutionOutcomeErrorKind::InvalidLogicalProcessId => 11,
    }
}

impl RawExecutionOutcomeErrorKind {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == raw_execution_outcome_error_kind_stable_tag_spec(self),
    {
        match self {
            RawExecutionOutcomeErrorKind::EventLimitExceeded => 1,
            RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded => 2,
            RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded => 3,
            RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded => 4,
            RawExecutionOutcomeErrorKind::EmptyExtensionNamespace => 5,
            RawExecutionOutcomeErrorKind::ZeroExtensionSchemaVersion => 6,
            RawExecutionOutcomeErrorKind::MalformedExtensionArtifact => 7,
            RawExecutionOutcomeErrorKind::UnsupportedExtensionArtifactAlgorithm => 8,
            RawExecutionOutcomeErrorKind::EmptyExtensionMediaType => 9,
            RawExecutionOutcomeErrorKind::InvalidUnixSignal => 10,
            RawExecutionOutcomeErrorKind::InvalidLogicalProcessId => 11,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawExecutionOutcomeError {
    kind: RawExecutionOutcomeErrorKind,
    location: RawExecutionOutcomeLocation,
    extension_code_point_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeErrorView {
    pub kind: RawExecutionOutcomeErrorKind,
    pub location: RawExecutionOutcomeLocation,
    pub extension_code_point_index: Option<u64>,
}

impl View for RawExecutionOutcomeError {
    type V = RawExecutionOutcomeErrorView;

    closed spec fn view(&self) -> RawExecutionOutcomeErrorView {
        RawExecutionOutcomeErrorView {
            kind: self.kind,
            location: self.location,
            extension_code_point_index: self.extension_code_point_index,
        }
    }
}

impl RawExecutionOutcomeError {
    fn new(
        kind: RawExecutionOutcomeErrorKind,
        location: RawExecutionOutcomeLocation,
        extension_code_point_index: Option<u64>,
    ) -> (error: Self)
        ensures
            error@ == (RawExecutionOutcomeErrorView { kind, location, extension_code_point_index }),
    {
        Self { kind, location, extension_code_point_index }
    }

    pub fn kind(&self) -> (value: RawExecutionOutcomeErrorKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn location(&self) -> (value: RawExecutionOutcomeLocation)
        ensures
            value == self@.location,
    {
        self.location
    }

    pub fn extension_code_point_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.extension_code_point_index,
    {
        self.extension_code_point_index
    }
}

pub open spec fn raw_execution_error_spec(
    kind: RawExecutionOutcomeErrorKind,
    location: RawExecutionOutcomeLocation,
    extension_code_point_index: Option<u64>,
) -> RawExecutionOutcomeErrorView {
    RawExecutionOutcomeErrorView { kind, location, extension_code_point_index }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExtensionMetadataUsage {
    namespace_code_points: u64,
    media_type_code_points: u64,
}

#[verifier::ext_equal]
pub struct ExtensionMetadataUsageView {
    pub namespace_code_points: u64,
    pub media_type_code_points: u64,
}

impl View for ExtensionMetadataUsage {
    type V = ExtensionMetadataUsageView;

    closed spec fn view(&self) -> ExtensionMetadataUsageView {
        ExtensionMetadataUsageView {
            namespace_code_points: self.namespace_code_points,
            media_type_code_points: self.media_type_code_points,
        }
    }
}

pub open spec fn extension_validation_spec(
    extension: VersionedExtensionRefView,
    location: RawExecutionOutcomeLocation,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    used: ExtensionMetadataUsageView,
) -> Result<ExtensionMetadataUsageView, RawExecutionOutcomeErrorView> {
    if used.namespace_code_points > namespace_limit {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded,
                location,
                Some(0),
            ),
        )
    } else if used.media_type_code_points > media_type_limit {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded,
                location,
                Some(0),
            ),
        )
    } else if extension.namespace.len() == 0 {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::EmptyExtensionNamespace,
                location,
                None,
            ),
        )
    } else if extension.schema_version == 0 {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::ZeroExtensionSchemaVersion,
                location,
                None,
            ),
        )
    } else if extension.namespace.len() > namespace_limit - used.namespace_code_points {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded,
                location,
                Some((namespace_limit - used.namespace_code_points) as u64),
            ),
        )
    } else if extension.payload.size_bytes > payload_limit {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded,
                location,
                None,
            ),
        )
    } else if crate::artifact::malformed_artifact_id_spec(extension.payload.id) {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::MalformedExtensionArtifact,
                location,
                None,
            ),
        )
    } else if crate::artifact::unsupported_artifact_algorithm_spec(extension.payload.id) {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::UnsupportedExtensionArtifactAlgorithm,
                location,
                None,
            ),
        )
    } else if extension.payload.media_type is Some && extension.payload.media_type.unwrap().len()
        == 0 {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::EmptyExtensionMediaType,
                location,
                None,
            ),
        )
    } else if extension.payload.media_type is Some && extension.payload.media_type.unwrap().len()
        > media_type_limit - used.media_type_code_points {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded,
                location,
                Some((media_type_limit - used.media_type_code_points) as u64),
            ),
        )
    } else {
        Ok(
            ExtensionMetadataUsageView {
                namespace_code_points: (used.namespace_code_points as int
                    + extension.namespace.len()) as u64,
                media_type_code_points: if extension.payload.media_type is Some {
                    (used.media_type_code_points as int
                        + extension.payload.media_type.unwrap().len()) as u64
                } else {
                    used.media_type_code_points
                },
            },
        )
    }
}

fn validate_extension(
    extension: &VersionedExtensionRef,
    location: RawExecutionOutcomeLocation,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    used: ExtensionMetadataUsage,
) -> (result: Result<ExtensionMetadataUsage, RawExecutionOutcomeError>)
    requires
        used@.namespace_code_points <= namespace_limit,
        used@.media_type_code_points <= media_type_limit,
        namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ensures
        match result {
            Ok(total) => extension_validation_spec(
                extension@,
                location,
                namespace_limit,
                media_type_limit,
                payload_limit,
                used@,
            ) == Ok(total@),
            Err(error) => extension_validation_spec(
                extension@,
                location,
                namespace_limit,
                media_type_limit,
                payload_limit,
                used@,
            ) == Err(error@),
        },
{
    let namespace_length = extension.namespace.as_str().unicode_len();
    if namespace_length == 0 {
        return Err(
            RawExecutionOutcomeError::new(
                RawExecutionOutcomeErrorKind::EmptyExtensionNamespace,
                location,
                None,
            ),
        );
    }
    if extension.schema_version == 0 {
        return Err(
            RawExecutionOutcomeError::new(
                RawExecutionOutcomeErrorKind::ZeroExtensionSchemaVersion,
                location,
                None,
            ),
        );
    }
    let remaining = namespace_limit - used.namespace_code_points;
    if namespace_length as u64 > remaining {
        return Err(
            RawExecutionOutcomeError::new(
                RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded,
                location,
                Some(remaining),
            ),
        );
    }
    if extension.payload.size_bytes > payload_limit {
        return Err(
            RawExecutionOutcomeError::new(
                RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded,
                location,
                None,
            ),
        );
    }
    match parse_artifact_id(&extension.payload.id) {
        Ok(ContentDigest::Sha256(_digest)) => proof {
            crate::artifact::lemma_artifact_id_spec_is_canonical(_digest@);
        },
        Err(ArtifactIdParseError::MalformedArtifactId) => {
            return Err(
                RawExecutionOutcomeError::new(
                    RawExecutionOutcomeErrorKind::MalformedExtensionArtifact,
                    location,
                    None,
                ),
            );
        },
        Err(ArtifactIdParseError::UnsupportedAlgorithm) => {
            return Err(
                RawExecutionOutcomeError::new(
                    RawExecutionOutcomeErrorKind::UnsupportedExtensionArtifactAlgorithm,
                    location,
                    None,
                ),
            );
        },
    }
    let mut next_media_type_code_points = used.media_type_code_points;
    if let Some(media_type) = &extension.payload.media_type {
        let media_type_length = media_type.as_str().unicode_len();
        if media_type_length == 0 {
            return Err(
                RawExecutionOutcomeError::new(
                    RawExecutionOutcomeErrorKind::EmptyExtensionMediaType,
                    location,
                    None,
                ),
            );
        }
        let media_remaining = media_type_limit - used.media_type_code_points;
        if media_type_length as u64 > media_remaining {
            return Err(
                RawExecutionOutcomeError::new(
                    RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded,
                    location,
                    Some(media_remaining),
                ),
            );
        }
        next_media_type_code_points = used.media_type_code_points + media_type_length as u64;
    }
    Ok(
        ExtensionMetadataUsage {
            namespace_code_points: used.namespace_code_points + namespace_length as u64,
            media_type_code_points: next_media_type_code_points,
        },
    )
}

pub open spec fn termination_validation_spec(
    termination: Option<TerminationRecordView>,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
) -> Result<ExtensionMetadataUsageView, RawExecutionOutcomeErrorView> {
    let empty_usage = ExtensionMetadataUsageView {
        namespace_code_points: 0,
        media_type_code_points: 0,
    };
    match termination {
        None => Ok(empty_usage),
        Some(TerminationRecordView::UnixSignal { signal, .. }) => if signal <= 0 {
            Err(
                raw_execution_error_spec(
                    RawExecutionOutcomeErrorKind::InvalidUnixSignal,
                    RawExecutionOutcomeLocation::Termination,
                    None,
                ),
            )
        } else {
            Ok(empty_usage)
        },
        Some(TerminationRecordView::PlatformSpecific(extension)) => extension_validation_spec(
            extension,
            RawExecutionOutcomeLocation::Termination,
            namespace_limit,
            media_type_limit,
            payload_limit,
            empty_usage,
        ),
        Some(_) => Ok(empty_usage),
    }
}

fn validate_termination(
    termination: &Option<TerminationRecord>,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
) -> (result: Result<ExtensionMetadataUsage, RawExecutionOutcomeError>)
    requires
        namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ensures
        match result {
            Ok(total) => termination_validation_spec(
                match termination {
                    Some(termination) => Some(termination@),
                    None => None,
                },
                namespace_limit,
                media_type_limit,
                payload_limit,
            ) == Ok(total@),
            Err(error) => termination_validation_spec(
                match termination {
                    Some(termination) => Some(termination@),
                    None => None,
                },
                namespace_limit,
                media_type_limit,
                payload_limit,
            ) == Err(error@),
        },
{
    let empty_usage = ExtensionMetadataUsage {
        namespace_code_points: 0,
        media_type_code_points: 0,
    };
    match termination {
        Some(TerminationRecord::UnixSignal { signal, .. }) => {
            if *signal <= 0 {
                Err(
                    RawExecutionOutcomeError::new(
                        RawExecutionOutcomeErrorKind::InvalidUnixSignal,
                        RawExecutionOutcomeLocation::Termination,
                        None,
                    ),
                )
            } else {
                Ok(empty_usage)
            }
        },
        Some(TerminationRecord::PlatformSpecific(extension)) => validate_extension(
            extension,
            RawExecutionOutcomeLocation::Termination,
            namespace_limit,
            media_type_limit,
            payload_limit,
            empty_usage,
        ),
        _ => Ok(empty_usage),
    }
}

pub open spec fn event_validation_spec(
    event: RawExecutionEventView,
    location: RawExecutionOutcomeLocation,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    used: ExtensionMetadataUsageView,
) -> Result<ExtensionMetadataUsageView, RawExecutionOutcomeErrorView> {
    match event {
        RawExecutionEventView::ProcessCreated { logical_process }
        | RawExecutionEventView::ProcessExited { logical_process } => if logical_process == 0 {
            Err(
                raw_execution_error_spec(
                    RawExecutionOutcomeErrorKind::InvalidLogicalProcessId,
                    location,
                    None,
                ),
            )
        } else {
            Ok(used)
        },
        RawExecutionEventView::PlatformSpecific(extension) => extension_validation_spec(
            extension,
            location,
            namespace_limit,
            media_type_limit,
            payload_limit,
            used,
        ),
        _ => Ok(used),
    }
}

fn validate_event(
    event: &RawExecutionEvent,
    event_index: u64,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    used: ExtensionMetadataUsage,
) -> (result: Result<ExtensionMetadataUsage, RawExecutionOutcomeError>)
    requires
        used@.namespace_code_points <= namespace_limit,
        used@.media_type_code_points <= media_type_limit,
        namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ensures
        match result {
            Ok(total) => event_validation_spec(
                event@,
                RawExecutionOutcomeLocation::Event(event_index),
                namespace_limit,
                media_type_limit,
                payload_limit,
                used@,
            ) == Ok(total@),
            Err(error) => event_validation_spec(
                event@,
                RawExecutionOutcomeLocation::Event(event_index),
                namespace_limit,
                media_type_limit,
                payload_limit,
                used@,
            ) == Err(error@),
        },
{
    match event {
        RawExecutionEvent::ProcessCreated { logical_process }
        | RawExecutionEvent::ProcessExited { logical_process } => {
            if logical_process.value() == 0 {
                Err(
                    RawExecutionOutcomeError::new(
                        RawExecutionOutcomeErrorKind::InvalidLogicalProcessId,
                        RawExecutionOutcomeLocation::Event(event_index),
                        None,
                    ),
                )
            } else {
                Ok(used)
            }
        },
        RawExecutionEvent::PlatformSpecific(extension) => validate_extension(
            extension,
            RawExecutionOutcomeLocation::Event(event_index),
            namespace_limit,
            media_type_limit,
            payload_limit,
            used,
        ),
        _ => Ok(used),
    }
}

pub open spec fn validate_raw_execution_events_spec(
    events: Seq<RawExecutionEventView>,
    index: nat,
    event_limit: u64,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    used: ExtensionMetadataUsageView,
) -> Result<ExtensionMetadataUsageView, RawExecutionOutcomeErrorView>
    decreases events.len() - index,
{
    if index >= events.len() {
        Ok(used)
    } else if index >= event_limit {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::EventLimitExceeded,
                RawExecutionOutcomeLocation::Event(index as u64),
                None,
            ),
        )
    } else {
        match event_validation_spec(
            events[index as int],
            RawExecutionOutcomeLocation::Event(index as u64),
            namespace_limit,
            media_type_limit,
            payload_limit,
            used,
        ) {
            Err(error) => Err(error),
            Ok(next_used) => validate_raw_execution_events_spec(
                events,
                index + 1,
                event_limit,
                namespace_limit,
                media_type_limit,
                payload_limit,
                next_used,
            ),
        }
    }
}

pub open spec fn validate_raw_execution_outcome_spec(
    outcome: RawExecutionOutcomeView,
    limits: RawExecutionOutcomeLimitsView,
) -> Result<RawExecutionOutcomeView, RawExecutionOutcomeErrorView> {
    let event_limit = effective_raw_execution_limit_spec(
        limits.max_events,
        MAX_RAW_EXECUTION_EVENTS,
    );
    let namespace_limit = effective_raw_execution_limit_spec(
        limits.max_extension_namespace_code_points,
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    );
    let media_type_limit = effective_raw_execution_limit_spec(
        limits.max_extension_media_type_code_points,
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    );
    let payload_limit = effective_raw_execution_limit_spec(
        limits.max_extension_payload_bytes_per_record,
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    );
    if outcome.events.len() > event_limit {
        Err(
            raw_execution_error_spec(
                RawExecutionOutcomeErrorKind::EventLimitExceeded,
                RawExecutionOutcomeLocation::Event(event_limit),
                None,
            ),
        )
    } else {
        match termination_validation_spec(
            outcome.termination,
            namespace_limit,
            media_type_limit,
            payload_limit,
        ) {
            Err(error) => Err(error),
            Ok(used) => match validate_raw_execution_events_spec(
                outcome.events,
                0,
                event_limit,
                namespace_limit,
                media_type_limit,
                payload_limit,
                used,
            ) {
                Err(error) => Err(error),
                Ok(_) => Ok(outcome),
            },
        }
    }
}

pub open spec fn raw_execution_outcome_semantics_spec(outcome: RawExecutionOutcomeView) -> bool {
    exists|event_limit: u64, namespace_limit: u64, media_type_limit: u64, payload_limit: u64|
        #![auto]
        event_limit <= MAX_RAW_EXECUTION_EVENTS && namespace_limit
            <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS && media_type_limit
            <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS && payload_limit
            <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD
            && validate_raw_execution_outcome_spec(
            outcome,
            RawExecutionOutcomeLimitsView {
                max_events: event_limit,
                max_extension_namespace_code_points: namespace_limit,
                max_extension_media_type_code_points: media_type_limit,
                max_extension_payload_bytes_per_record: payload_limit,
            },
        ) == Ok(outcome)
}

proof fn lemma_raw_execution_events_error_lifts_to_outcome(
    outcome: RawExecutionOutcomeView,
    limits: RawExecutionOutcomeLimitsView,
    event_limit: u64,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    termination_used: ExtensionMetadataUsageView,
    error: RawExecutionOutcomeErrorView,
)
    requires
        event_limit == effective_raw_execution_limit_spec(
            limits.max_events,
            MAX_RAW_EXECUTION_EVENTS,
        ),
        namespace_limit == effective_raw_execution_limit_spec(
            limits.max_extension_namespace_code_points,
            MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        ),
        media_type_limit == effective_raw_execution_limit_spec(
            limits.max_extension_media_type_code_points,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        ),
        payload_limit == effective_raw_execution_limit_spec(
            limits.max_extension_payload_bytes_per_record,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ),
        outcome.events.len() <= event_limit,
        termination_validation_spec(
            outcome.termination,
            namespace_limit,
            media_type_limit,
            payload_limit,
        ) == Ok(termination_used),
        validate_raw_execution_events_spec(
            outcome.events,
            0,
            event_limit,
            namespace_limit,
            media_type_limit,
            payload_limit,
            termination_used,
        ) == Err(error),
    ensures
        validate_raw_execution_outcome_spec(outcome, limits) == Err(error),
{
    reveal(validate_raw_execution_outcome_spec);
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedRawExecutionOutcome {
    outcome: RawExecutionOutcome,
}

impl View for ValidatedRawExecutionOutcome {
    type V = RawExecutionOutcomeView;

    closed spec fn view(&self) -> RawExecutionOutcomeView {
        self.outcome@
    }
}

impl ValidatedRawExecutionOutcome {
    pub fn outcome(&self) -> (value: &RawExecutionOutcome)
        ensures
            value@ == self@,
    {
        &self.outcome
    }

    pub fn into_inner(self) -> (value: RawExecutionOutcome)
        ensures
            value@ == self@,
    {
        self.outcome
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawExecutionOutcomeRejection {
    error: RawExecutionOutcomeError,
    outcome: RawExecutionOutcome,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeRejectionView {
    pub error: RawExecutionOutcomeErrorView,
    pub outcome: RawExecutionOutcomeView,
}

impl View for RawExecutionOutcomeRejection {
    type V = RawExecutionOutcomeRejectionView;

    closed spec fn view(&self) -> RawExecutionOutcomeRejectionView {
        RawExecutionOutcomeRejectionView { error: self.error@, outcome: self.outcome@ }
    }
}

impl RawExecutionOutcomeRejection {
    pub fn error(&self) -> (value: &RawExecutionOutcomeError)
        ensures
            value@ == self@.error,
    {
        &self.error
    }

    pub fn outcome(&self) -> (value: &RawExecutionOutcome)
        ensures
            value@ == self@.outcome,
    {
        &self.outcome
    }

    pub fn into_parts(self) -> (value: (RawExecutionOutcomeError, RawExecutionOutcome))
        ensures
            value.0@ == self@.error,
            value.1@ == self@.outcome,
    {
        (self.error, self.outcome)
    }
}

// A rejection retains the exact owned outcome so callers can persist or replay rejected evidence.
// Boxing would add a mandatory error-path allocation without reducing the retained information.
#[expect(
    clippy::result_large_err,
    reason = "rejections retain the exact owned outcome for deterministic replay"
)]
pub fn validate_raw_execution_outcome(
    outcome: RawExecutionOutcome,
    limits: RawExecutionOutcomeLimits,
) -> (result: Result<ValidatedRawExecutionOutcome, RawExecutionOutcomeRejection>)
    ensures
        match &result {
            Ok(validated) => validate_raw_execution_outcome_spec(outcome@, limits@) == Ok(
                validated@,
            ),
            Err(rejection) => validate_raw_execution_outcome_spec(outcome@, limits@) == Err(
                rejection@.error,
            ) && rejection@.outcome == outcome@,
        },
{
    let event_limit = effective_raw_execution_limit(limits.max_events, MAX_RAW_EXECUTION_EVENTS);
    let namespace_limit = effective_raw_execution_limit(
        limits.max_extension_namespace_code_points,
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    );
    let media_type_limit = effective_raw_execution_limit(
        limits.max_extension_media_type_code_points,
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    );
    let payload_limit = effective_raw_execution_limit(
        limits.max_extension_payload_bytes_per_record,
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    );
    proof {
        reveal(<RawExecutionOutcomeLimits as View>::view);
        assert(event_limit == effective_raw_execution_limit_spec(
            limits@.max_events,
            MAX_RAW_EXECUTION_EVENTS,
        ));
        assert(namespace_limit == effective_raw_execution_limit_spec(
            limits@.max_extension_namespace_code_points,
            MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        ));
        assert(media_type_limit == effective_raw_execution_limit_spec(
            limits@.max_extension_media_type_code_points,
            MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        ));
        assert(payload_limit == effective_raw_execution_limit_spec(
            limits@.max_extension_payload_bytes_per_record,
            MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        ));
    }
    if outcome.events.len() as u64 > event_limit {
        let error = RawExecutionOutcomeError::new(
            RawExecutionOutcomeErrorKind::EventLimitExceeded,
            RawExecutionOutcomeLocation::Event(event_limit),
            None,
        );
        proof {
            reveal(validate_raw_execution_outcome_spec);
            assert(outcome@.events.len() > event_limit);
        }
        return Err(RawExecutionOutcomeRejection { error, outcome });
    }
    let termination_used = match validate_termination(
        &outcome.termination,
        namespace_limit,
        media_type_limit,
        payload_limit,
    ) {
        Ok(total) => total,
        Err(error) => return Err(RawExecutionOutcomeRejection { error, outcome }),
    };
    let mut used = termination_used;
    let ghost outcome_view = outcome@;
    let mut index = 0usize;
    while index < outcome.events.len()
        invariant
            index <= outcome.events.len(),
            event_limit <= MAX_RAW_EXECUTION_EVENTS,
            namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
            event_limit == effective_raw_execution_limit_spec(
                limits@.max_events,
                MAX_RAW_EXECUTION_EVENTS,
            ),
            namespace_limit == effective_raw_execution_limit_spec(
                limits@.max_extension_namespace_code_points,
                MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            ),
            media_type_limit == effective_raw_execution_limit_spec(
                limits@.max_extension_media_type_code_points,
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            ),
            payload_limit == effective_raw_execution_limit_spec(
                limits@.max_extension_payload_bytes_per_record,
                MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
            ),
            outcome.events.len() as u64 <= event_limit,
            outcome_view.events.len() <= event_limit,
            used@.namespace_code_points <= namespace_limit,
            used@.media_type_code_points <= media_type_limit,
            outcome@ == outcome_view,
            termination_validation_spec(
                outcome_view.termination,
                namespace_limit,
                media_type_limit,
                payload_limit,
            ) == Ok(termination_used@),
            validate_raw_execution_events_spec(
                outcome_view.events,
                0,
                event_limit,
                namespace_limit,
                media_type_limit,
                payload_limit,
                termination_used@,
            ) == validate_raw_execution_events_spec(
                outcome_view.events,
                index as nat,
                event_limit,
                namespace_limit,
                media_type_limit,
                payload_limit,
                used@,
            ),
        decreases outcome.events.len() - index,
    {
        match validate_event(
            &outcome.events[index],
            index as u64,
            namespace_limit,
            media_type_limit,
            payload_limit,
            used,
        ) {
            Ok(total) => {
                proof {
                    reveal(validate_raw_execution_events_spec);
                }
                used = total;
            },
            Err(error) => {
                proof {
                    reveal(validate_raw_execution_events_spec);
                    reveal(validate_raw_execution_outcome_spec);
                    assert(event_validation_spec(
                        outcome_view.events[index as int],
                        RawExecutionOutcomeLocation::Event(index as u64),
                        namespace_limit,
                        media_type_limit,
                        payload_limit,
                        used@,
                    ) == Err(error@));
                    assert(validate_raw_execution_events_spec(
                        outcome_view.events,
                        index as nat,
                        event_limit,
                        namespace_limit,
                        media_type_limit,
                        payload_limit,
                        used@,
                    ) == Err(error@));
                    assert(validate_raw_execution_events_spec(
                        outcome_view.events,
                        0,
                        event_limit,
                        namespace_limit,
                        media_type_limit,
                        payload_limit,
                        termination_used@,
                    ) == Err(error@));
                    lemma_raw_execution_events_error_lifts_to_outcome(
                        outcome_view,
                        limits@,
                        event_limit,
                        namespace_limit,
                        media_type_limit,
                        payload_limit,
                        termination_used@,
                        error@,
                    );
                }
                return Err(RawExecutionOutcomeRejection { error, outcome });
            },
        }
        index += 1;
    }
    proof {
        reveal(validate_raw_execution_events_spec);
        reveal(validate_raw_execution_outcome_spec);
    }
    Ok(ValidatedRawExecutionOutcome { outcome })
}

pub proof fn lemma_raw_execution_validation_preserves_exact_input(
    outcome: RawExecutionOutcomeView,
    limits: RawExecutionOutcomeLimitsView,
)
    ensures
        forall|validated: RawExecutionOutcomeView|
            validate_raw_execution_outcome_spec(outcome, limits) == Ok(validated) ==> validated
                == outcome,
{
    reveal(validate_raw_execution_outcome_spec);
}

pub proof fn lemma_successful_raw_execution_validation_has_semantics(
    outcome: RawExecutionOutcomeView,
    limits: RawExecutionOutcomeLimitsView,
    validated: RawExecutionOutcomeView,
)
    requires
        validate_raw_execution_outcome_spec(outcome, limits) == Ok(validated),
    ensures
        raw_execution_outcome_semantics_spec(validated),
        validated == outcome,
{
    lemma_raw_execution_validation_preserves_exact_input(outcome, limits);
    assert(validated == outcome);
    let event_limit = effective_raw_execution_limit_spec(
        limits.max_events,
        MAX_RAW_EXECUTION_EVENTS,
    );
    let namespace_limit = effective_raw_execution_limit_spec(
        limits.max_extension_namespace_code_points,
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    );
    let media_type_limit = effective_raw_execution_limit_spec(
        limits.max_extension_media_type_code_points,
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    );
    let payload_limit = effective_raw_execution_limit_spec(
        limits.max_extension_payload_bytes_per_record,
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    );
    assert(event_limit <= MAX_RAW_EXECUTION_EVENTS);
    assert(namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS);
    assert(media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS);
    assert(payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD);
    assert(validate_raw_execution_outcome_spec(
        outcome,
        RawExecutionOutcomeLimitsView {
            max_events: event_limit,
            max_extension_namespace_code_points: namespace_limit,
            max_extension_media_type_code_points: media_type_limit,
            max_extension_payload_bytes_per_record: payload_limit,
        },
    ) == Ok(outcome));
    reveal(raw_execution_outcome_semantics_spec);
}

} // verus!
