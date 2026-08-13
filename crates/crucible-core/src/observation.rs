//! Immutable, portable raw observations and semantic admission.
use crate::artifact::{
    parse_artifact_id, ArtifactIdParseError, ArtifactRef, ArtifactRefView, ContentDigest,
};
use crate::execution::{
    validate_raw_execution_outcome, RawExecutionOutcome, RawExecutionOutcomeErrorKind,
    RawExecutionOutcomeLimits, RawExecutionOutcomeLimitsView, RawExecutionOutcomeLocation,
    RawExecutionOutcomeView, VersionedExtensionRef, VersionedExtensionRefView,
    MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
};
use crate::{CoverageProviderId, RunAttemptId, RunId, TargetBuildId};
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

verus! {

pub const RAW_OBSERVATION_SCHEMA_VERSION: u16 = 1;

pub const MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS: u64 = 65_536;

pub const MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS: u64 = 65_536;

pub const MAX_RAW_OBSERVATION_EXTENSIONS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordedDurationError {
    NanosecondsOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordedDuration {
    seconds: u64,
    nanoseconds: u32,
}

#[verifier::ext_equal]
pub struct RecordedDurationView {
    pub seconds: u64,
    pub nanoseconds: u32,
}

impl View for RecordedDuration {
    type V = RecordedDurationView;

    closed spec fn view(&self) -> RecordedDurationView {
        RecordedDurationView { seconds: self.seconds, nanoseconds: self.nanoseconds }
    }
}

impl RecordedDuration {
    pub fn new(seconds: u64, nanoseconds: u32) -> (result: Result<Self, RecordedDurationError>)
        ensures
            match result {
                Ok(duration) => duration@ == (RecordedDurationView { seconds, nanoseconds })
                    && nanoseconds < 1_000_000_000,
                Err(RecordedDurationError::NanosecondsOutOfRange) => nanoseconds >= 1_000_000_000,
            },
    {
        if nanoseconds >= 1_000_000_000 {
            Err(RecordedDurationError::NanosecondsOutOfRange)
        } else {
            Ok(Self { seconds, nanoseconds })
        }
    }

    pub fn seconds(&self) -> (value: u64)
        ensures
            value == self@.seconds,
    {
        self.seconds
    }

    pub fn nanoseconds(&self) -> (value: u32)
        ensures
            value == self@.nanoseconds,
    {
        self.nanoseconds
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CapturedStreamRef {
    artifact: ArtifactRef,
    truncated: bool,
    retained_bytes: u64,
    discarded_bytes: u64,
}

#[verifier::ext_equal]
pub struct CapturedStreamRefView {
    pub artifact: ArtifactRefView,
    pub truncated: bool,
    pub retained_bytes: u64,
    pub discarded_bytes: u64,
}

impl View for CapturedStreamRef {
    type V = CapturedStreamRefView;

    closed spec fn view(&self) -> CapturedStreamRefView {
        CapturedStreamRefView {
            artifact: self.artifact@,
            truncated: self.truncated,
            retained_bytes: self.retained_bytes,
            discarded_bytes: self.discarded_bytes,
        }
    }
}

impl CapturedStreamRef {
    pub fn new(
        artifact: ArtifactRef,
        truncated: bool,
        retained_bytes: u64,
        discarded_bytes: u64,
    ) -> (stream: Self)
        ensures
            stream@ == (CapturedStreamRefView {
                artifact: artifact@,
                truncated,
                retained_bytes,
                discarded_bytes,
            }),
    {
        Self { artifact, truncated, retained_bytes, discarded_bytes }
    }

    pub fn artifact(&self) -> (value: &ArtifactRef)
        ensures
            value@ == self@.artifact,
    {
        &self.artifact
    }

    pub fn truncated(&self) -> (value: bool)
        ensures
            value == self@.truncated,
    {
        self.truncated
    }

    pub fn retained_bytes(&self) -> (value: u64)
        ensures
            value == self@.retained_bytes,
    {
        self.retained_bytes
    }

    pub fn discarded_bytes(&self) -> (value: u64)
        ensures
            value == self@.discarded_bytes,
    {
        self.discarded_bytes
    }
}

pub open spec fn observation_extension_views_spec(extensions: Seq<VersionedExtensionRef>) -> Seq<
    VersionedExtensionRefView,
>
    decreases extensions.len(),
{
    if extensions.len() == 0 {
        Seq::empty()
    } else {
        observation_extension_views_spec(extensions.drop_last()).push(extensions.last()@)
    }
}

pub proof fn lemma_observation_extension_views_properties(extensions: Seq<VersionedExtensionRef>)
    ensures
        observation_extension_views_spec(extensions).len() == extensions.len(),
        forall|index: int|
            0 <= index < extensions.len() ==> #[trigger] observation_extension_views_spec(
                extensions,
            )[index] == extensions[index]@,
    decreases extensions.len(),
{
    if extensions.len() > 0 {
        lemma_observation_extension_views_properties(extensions.drop_last());
        reveal(observation_extension_views_spec);
    } else {
        reveal(observation_extension_views_spec);
    }
}

proof fn lemma_observation_extension_view_at(extensions: Seq<VersionedExtensionRef>, index: int)
    requires
        0 <= index < extensions.len(),
    ensures
        observation_extension_views_spec(extensions)[index] == extensions[index]@,
{
    lemma_observation_extension_views_properties(extensions);
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResourceSnapshot {
    process_count: Option<u64>,
    thread_count: Option<u64>,
    open_file_count: Option<u64>,
    handle_count: Option<u64>,
    read_bytes: Option<u64>,
    written_bytes: Option<u64>,
    extensions: Vec<VersionedExtensionRef>,
}

#[verifier::ext_equal]
pub struct ResourceSnapshotView {
    pub process_count: Option<u64>,
    pub thread_count: Option<u64>,
    pub open_file_count: Option<u64>,
    pub handle_count: Option<u64>,
    pub read_bytes: Option<u64>,
    pub written_bytes: Option<u64>,
    pub extensions: Seq<VersionedExtensionRefView>,
}

impl View for ResourceSnapshot {
    type V = ResourceSnapshotView;

    closed spec fn view(&self) -> ResourceSnapshotView {
        ResourceSnapshotView {
            process_count: self.process_count,
            thread_count: self.thread_count,
            open_file_count: self.open_file_count,
            handle_count: self.handle_count,
            read_bytes: self.read_bytes,
            written_bytes: self.written_bytes,
            extensions: observation_extension_views_spec(self.extensions@),
        }
    }
}

impl ResourceSnapshot {
    pub fn new(
        process_count: Option<u64>,
        thread_count: Option<u64>,
        open_file_count: Option<u64>,
        handle_count: Option<u64>,
        read_bytes: Option<u64>,
        written_bytes: Option<u64>,
        extensions: Vec<VersionedExtensionRef>,
    ) -> (snapshot: Self)
        ensures
            snapshot@ == (ResourceSnapshotView {
                process_count,
                thread_count,
                open_file_count,
                handle_count,
                read_bytes,
                written_bytes,
                extensions: observation_extension_views_spec(extensions@),
            }),
    {
        Self {
            process_count,
            thread_count,
            open_file_count,
            handle_count,
            read_bytes,
            written_bytes,
            extensions,
        }
    }

    pub fn process_count(&self) -> (value: Option<u64>) {
        self.process_count
    }

    pub fn thread_count(&self) -> (value: Option<u64>) {
        self.thread_count
    }

    pub fn open_file_count(&self) -> (value: Option<u64>) {
        self.open_file_count
    }

    pub fn handle_count(&self) -> (value: Option<u64>) {
        self.handle_count
    }

    pub fn read_bytes(&self) -> (value: Option<u64>) {
        self.read_bytes
    }

    pub fn written_bytes(&self) -> (value: Option<u64>) {
        self.written_bytes
    }

    pub fn extensions(&self) -> (value: &[VersionedExtensionRef])
        ensures
            observation_extension_views_spec(value@) == self@.extensions,
    {
        self.extensions.as_slice()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CoverageRef {
    provider: CoverageProviderId,
    provider_version: String,
    target_build: TargetBuildId,
    feature_set_digest: String,
    artifact: ArtifactRef,
    new_features: u64,
    total_features: u64,
}

#[verifier::ext_equal]
pub struct CoverageRefView {
    pub provider: Seq<char>,
    pub provider_version: Seq<char>,
    pub target_build: Seq<char>,
    pub feature_set_digest: Seq<char>,
    pub artifact: ArtifactRefView,
    pub new_features: u64,
    pub total_features: u64,
}

impl View for CoverageRef {
    type V = CoverageRefView;

    closed spec fn view(&self) -> CoverageRefView {
        CoverageRefView {
            provider: self.provider@,
            provider_version: self.provider_version@,
            target_build: self.target_build@,
            feature_set_digest: self.feature_set_digest@,
            artifact: self.artifact@,
            new_features: self.new_features,
            total_features: self.total_features,
        }
    }
}

impl CoverageRef {
    pub fn new(
        provider: CoverageProviderId,
        provider_version: String,
        target_build: TargetBuildId,
        feature_set_digest: String,
        artifact: ArtifactRef,
        new_features: u64,
        total_features: u64,
    ) -> (coverage: Self)
        ensures
            coverage@ == (CoverageRefView {
                provider: provider@,
                provider_version: provider_version@,
                target_build: target_build@,
                feature_set_digest: feature_set_digest@,
                artifact: artifact@,
                new_features,
                total_features,
            }),
    {
        Self {
            provider,
            provider_version,
            target_build,
            feature_set_digest,
            artifact,
            new_features,
            total_features,
        }
    }

    pub fn provider(&self) -> (value: &CoverageProviderId) {
        &self.provider
    }

    pub fn provider_version(&self) -> (value: &str) {
        self.provider_version.as_str()
    }

    pub fn target_build(&self) -> (value: &TargetBuildId) {
        &self.target_build
    }

    pub fn feature_set_digest(&self) -> (value: &str) {
        self.feature_set_digest.as_str()
    }

    pub fn artifact(&self) -> (value: &ArtifactRef) {
        &self.artifact
    }

    pub fn new_features(&self) -> (value: u64) {
        self.new_features
    }

    pub fn total_features(&self) -> (value: u64) {
        self.total_features
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StateDigest {
    namespace: String,
    schema_version: u32,
    artifact: ArtifactRef,
}

#[verifier::ext_equal]
pub struct StateDigestView {
    pub namespace: Seq<char>,
    pub schema_version: u32,
    pub artifact: ArtifactRefView,
}

impl View for StateDigest {
    type V = StateDigestView;

    closed spec fn view(&self) -> StateDigestView {
        StateDigestView {
            namespace: self.namespace@,
            schema_version: self.schema_version,
            artifact: self.artifact@,
        }
    }
}

impl StateDigest {
    pub fn new(namespace: String, schema_version: u32, artifact: ArtifactRef) -> (value: Self)
        ensures
            value@ == (StateDigestView {
                namespace: namespace@,
                schema_version,
                artifact: artifact@,
            }),
    {
        Self { namespace, schema_version, artifact }
    }

    pub fn namespace(&self) -> (value: &str) {
        self.namespace.as_str()
    }

    pub fn schema_version(&self) -> (value: u32) {
        self.schema_version
    }

    pub fn artifact(&self) -> (value: &ArtifactRef) {
        &self.artifact
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScheduleTrace {
    namespace: String,
    schema_version: u32,
    artifact: ArtifactRef,
    decisions: u64,
    complete: bool,
}

#[verifier::ext_equal]
pub struct ScheduleTraceView {
    pub namespace: Seq<char>,
    pub schema_version: u32,
    pub artifact: ArtifactRefView,
    pub decisions: u64,
    pub complete: bool,
}

impl View for ScheduleTrace {
    type V = ScheduleTraceView;

    closed spec fn view(&self) -> ScheduleTraceView {
        ScheduleTraceView {
            namespace: self.namespace@,
            schema_version: self.schema_version,
            artifact: self.artifact@,
            decisions: self.decisions,
            complete: self.complete,
        }
    }
}

impl ScheduleTrace {
    pub fn new(
        namespace: String,
        schema_version: u32,
        artifact: ArtifactRef,
        decisions: u64,
        complete: bool,
    ) -> (value: Self)
        ensures
            value@ == (ScheduleTraceView {
                namespace: namespace@,
                schema_version,
                artifact: artifact@,
                decisions,
                complete,
            }),
    {
        Self { namespace, schema_version, artifact, decisions, complete }
    }

    pub fn namespace(&self) -> (value: &str) {
        self.namespace.as_str()
    }

    pub fn schema_version(&self) -> (value: u32) {
        self.schema_version
    }

    pub fn artifact(&self) -> (value: &ArtifactRef) {
        &self.artifact
    }

    pub fn decisions(&self) -> (value: u64) {
        self.decisions
    }

    pub fn complete(&self) -> (value: bool) {
        self.complete
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FaultTrace {
    namespace: String,
    schema_version: u32,
    artifact: ArtifactRef,
    reached: u64,
    applied: u64,
    skipped: u64,
    shadowed: u64,
    rejected: u64,
    complete: bool,
}

#[verifier::ext_equal]
pub struct FaultTraceView {
    pub namespace: Seq<char>,
    pub schema_version: u32,
    pub artifact: ArtifactRefView,
    pub reached: u64,
    pub applied: u64,
    pub skipped: u64,
    pub shadowed: u64,
    pub rejected: u64,
    pub complete: bool,
}

impl View for FaultTrace {
    type V = FaultTraceView;

    closed spec fn view(&self) -> FaultTraceView {
        FaultTraceView {
            namespace: self.namespace@,
            schema_version: self.schema_version,
            artifact: self.artifact@,
            reached: self.reached,
            applied: self.applied,
            skipped: self.skipped,
            shadowed: self.shadowed,
            rejected: self.rejected,
            complete: self.complete,
        }
    }
}

impl FaultTrace {
    // This constructor is the explicit boundary for a persisted fault summary: requiring every
    // counter prevents defaults from silently changing the meaning of an admitted trace.
    #[expect(
    clippy::too_many_arguments,
    reason = "every persisted fault counter is an explicit required input"
)]
    pub fn new(
        namespace: String,
        schema_version: u32,
        artifact: ArtifactRef,
        reached: u64,
        applied: u64,
        skipped: u64,
        shadowed: u64,
        rejected: u64,
        complete: bool,
    ) -> (value: Self)
        ensures
            value@ == (FaultTraceView {
                namespace: namespace@,
                schema_version,
                artifact: artifact@,
                reached,
                applied,
                skipped,
                shadowed,
                rejected,
                complete,
            }),
    {
        Self {
            namespace,
            schema_version,
            artifact,
            reached,
            applied,
            skipped,
            shadowed,
            rejected,
            complete,
        }
    }

    pub fn namespace(&self) -> (value: &str) {
        self.namespace.as_str()
    }

    pub fn schema_version(&self) -> (value: u32) {
        self.schema_version
    }

    pub fn artifact(&self) -> (value: &ArtifactRef) {
        &self.artifact
    }

    pub fn reached(&self) -> (value: u64) {
        self.reached
    }

    pub fn applied(&self) -> (value: u64) {
        self.applied
    }

    pub fn skipped(&self) -> (value: u64) {
        self.skipped
    }

    pub fn shadowed(&self) -> (value: u64) {
        self.shadowed
    }

    pub fn rejected(&self) -> (value: u64) {
        self.rejected
    }

    pub fn complete(&self) -> (value: bool) {
        self.complete
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawObservation {
    run_id: RunId,
    attempt_id: RunAttemptId,
    outcome: RawExecutionOutcome,
    stdout: CapturedStreamRef,
    stderr: CapturedStreamRef,
    wall_time: RecordedDuration,
    cpu_time: Option<RecordedDuration>,
    peak_rss_bytes: Option<u64>,
    resources: ResourceSnapshot,
    coverage: Option<CoverageRef>,
    state_digest: Option<StateDigest>,
    schedule_trace: Option<ScheduleTrace>,
    fault_trace: Option<FaultTrace>,
    extensions: Vec<VersionedExtensionRef>,
}

#[verifier::ext_equal]
pub struct RawObservationView {
    pub run_id: Seq<char>,
    pub attempt_id: Seq<char>,
    pub outcome: RawExecutionOutcomeView,
    pub stdout: CapturedStreamRefView,
    pub stderr: CapturedStreamRefView,
    pub wall_time: RecordedDurationView,
    pub cpu_time: Option<RecordedDurationView>,
    pub peak_rss_bytes: Option<u64>,
    pub resources: ResourceSnapshotView,
    pub coverage: Option<CoverageRefView>,
    pub state_digest: Option<StateDigestView>,
    pub schedule_trace: Option<ScheduleTraceView>,
    pub fault_trace: Option<FaultTraceView>,
    pub extensions: Seq<VersionedExtensionRefView>,
}

impl View for RawObservation {
    type V = RawObservationView;

    closed spec fn view(&self) -> RawObservationView {
        RawObservationView {
            run_id: self.run_id@,
            attempt_id: self.attempt_id@,
            outcome: self.outcome@,
            stdout: self.stdout@,
            stderr: self.stderr@,
            wall_time: self.wall_time@,
            cpu_time: match &self.cpu_time {
                Some(value) => Some(value@),
                None => None,
            },
            peak_rss_bytes: self.peak_rss_bytes,
            resources: self.resources@,
            coverage: match &self.coverage {
                Some(value) => Some(value@),
                None => None,
            },
            state_digest: match &self.state_digest {
                Some(value) => Some(value@),
                None => None,
            },
            schedule_trace: match &self.schedule_trace {
                Some(value) => Some(value@),
                None => None,
            },
            fault_trace: match &self.fault_trace {
                Some(value) => Some(value@),
                None => None,
            },
            extensions: observation_extension_views_spec(self.extensions@),
        }
    }
}

impl RawObservation {
    // Every field is required at the immutable observation boundary. A positional constructor is
    // intentionally exhaustive so schema additions fail at callers instead of inheriting defaults.
    #[expect(
    clippy::too_many_arguments,
    reason = "every immutable observation field is an explicit required input"
)]
    pub fn new(
        run_id: RunId,
        attempt_id: RunAttemptId,
        outcome: RawExecutionOutcome,
        stdout: CapturedStreamRef,
        stderr: CapturedStreamRef,
        wall_time: RecordedDuration,
        cpu_time: Option<RecordedDuration>,
        peak_rss_bytes: Option<u64>,
        resources: ResourceSnapshot,
        coverage: Option<CoverageRef>,
        state_digest: Option<StateDigest>,
        schedule_trace: Option<ScheduleTrace>,
        fault_trace: Option<FaultTrace>,
        extensions: Vec<VersionedExtensionRef>,
    ) -> (observation: Self)
        ensures
            observation@ == (RawObservationView {
                run_id: run_id@,
                attempt_id: attempt_id@,
                outcome: outcome@,
                stdout: stdout@,
                stderr: stderr@,
                wall_time: wall_time@,
                cpu_time: match &cpu_time {
                    Some(value) => Some(value@),
                    None => None,
                },
                peak_rss_bytes,
                resources: resources@,
                coverage: match &coverage {
                    Some(value) => Some(value@),
                    None => None,
                },
                state_digest: match &state_digest {
                    Some(value) => Some(value@),
                    None => None,
                },
                schedule_trace: match &schedule_trace {
                    Some(value) => Some(value@),
                    None => None,
                },
                fault_trace: match &fault_trace {
                    Some(value) => Some(value@),
                    None => None,
                },
                extensions: observation_extension_views_spec(extensions@),
            }),
    {
        Self {
            run_id,
            attempt_id,
            outcome,
            stdout,
            stderr,
            wall_time,
            cpu_time,
            peak_rss_bytes,
            resources,
            coverage,
            state_digest,
            schedule_trace,
            fault_trace,
            extensions,
        }
    }

    pub fn run_id(&self) -> (value: &RunId) {
        &self.run_id
    }

    pub fn attempt_id(&self) -> (value: &RunAttemptId) {
        &self.attempt_id
    }

    pub fn outcome(&self) -> (value: &RawExecutionOutcome) {
        &self.outcome
    }

    pub fn stdout(&self) -> (value: &CapturedStreamRef) {
        &self.stdout
    }

    pub fn stderr(&self) -> (value: &CapturedStreamRef) {
        &self.stderr
    }

    pub fn wall_time(&self) -> (value: &RecordedDuration) {
        &self.wall_time
    }

    pub fn cpu_time(&self) -> (value: &Option<RecordedDuration>) {
        &self.cpu_time
    }

    pub fn peak_rss_bytes(&self) -> (value: Option<u64>) {
        self.peak_rss_bytes
    }

    pub fn resources(&self) -> (value: &ResourceSnapshot) {
        &self.resources
    }

    pub fn coverage(&self) -> (value: &Option<CoverageRef>) {
        &self.coverage
    }

    pub fn state_digest(&self) -> (value: &Option<StateDigest>) {
        &self.state_digest
    }

    pub fn schedule_trace(&self) -> (value: &Option<ScheduleTrace>) {
        &self.schedule_trace
    }

    pub fn fault_trace(&self) -> (value: &Option<FaultTrace>) {
        &self.fault_trace
    }

    pub fn extensions(&self) -> (value: &[VersionedExtensionRef]) {
        self.extensions.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawObservationLimits {
    outcome_limits: RawExecutionOutcomeLimits,
    max_identity_code_points: u64,
    max_resource_extensions: u64,
    max_extensions: u64,
    max_extension_namespace_code_points: u64,
    max_extension_media_type_code_points: u64,
    max_extension_payload_bytes_per_record: u64,
}

#[verifier::ext_equal]
pub struct RawObservationLimitsView {
    pub outcome_limits: RawExecutionOutcomeLimitsView,
    pub max_identity_code_points: u64,
    pub max_resource_extensions: u64,
    pub max_extensions: u64,
    pub max_extension_namespace_code_points: u64,
    pub max_extension_media_type_code_points: u64,
    pub max_extension_payload_bytes_per_record: u64,
}

impl View for RawObservationLimits {
    type V = RawObservationLimitsView;

    closed spec fn view(&self) -> RawObservationLimitsView {
        RawObservationLimitsView {
            outcome_limits: self.outcome_limits@,
            max_identity_code_points: self.max_identity_code_points,
            max_resource_extensions: self.max_resource_extensions,
            max_extensions: self.max_extensions,
            max_extension_namespace_code_points: self.max_extension_namespace_code_points,
            max_extension_media_type_code_points: self.max_extension_media_type_code_points,
            max_extension_payload_bytes_per_record: self.max_extension_payload_bytes_per_record,
        }
    }
}

impl RawObservationLimits {
    pub fn new(
        outcome_limits: RawExecutionOutcomeLimits,
        max_identity_code_points: u64,
        max_resource_extensions: u64,
        max_extensions: u64,
        max_extension_namespace_code_points: u64,
        max_extension_media_type_code_points: u64,
        max_extension_payload_bytes_per_record: u64,
    ) -> (limits: Self) {
        Self {
            outcome_limits,
            max_identity_code_points,
            max_resource_extensions,
            max_extensions,
            max_extension_namespace_code_points,
            max_extension_media_type_code_points,
            max_extension_payload_bytes_per_record,
        }
    }

    pub fn outcome_limits(&self) -> (value: RawExecutionOutcomeLimits) {
        self.outcome_limits
    }

    pub fn max_identity_code_points(&self) -> (value: u64) {
        self.max_identity_code_points
    }

    pub fn max_resource_extensions(&self) -> (value: u64) {
        self.max_resource_extensions
    }

    pub fn max_extensions(&self) -> (value: u64) {
        self.max_extensions
    }

    pub fn max_extension_namespace_code_points(&self) -> (value: u64) {
        self.max_extension_namespace_code_points
    }

    pub fn max_extension_media_type_code_points(&self) -> (value: u64) {
        self.max_extension_media_type_code_points
    }

    pub fn max_extension_payload_bytes_per_record(&self) -> (value: u64) {
        self.max_extension_payload_bytes_per_record
    }
}

pub fn canonical_raw_observation_limits() -> (limits: RawObservationLimits) {
    RawObservationLimits::new(
        crate::execution::canonical_raw_execution_outcome_limits(),
        MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS,
        MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS,
        MAX_RAW_OBSERVATION_EXTENSIONS,
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    )
}

pub open spec fn effective_raw_observation_identity_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS {
        requested
    } else {
        MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS
    }
}

pub open spec fn effective_raw_observation_resource_extension_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS {
        requested
    } else {
        MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS
    }
}

pub open spec fn effective_raw_observation_extension_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_OBSERVATION_EXTENSIONS {
        requested
    } else {
        MAX_RAW_OBSERVATION_EXTENSIONS
    }
}

pub open spec fn effective_raw_observation_namespace_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS {
        requested
    } else {
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS
    }
}

pub open spec fn effective_raw_observation_media_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS {
        requested
    } else {
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS
    }
}

pub open spec fn effective_raw_observation_payload_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD {
        requested
    } else {
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit == if requested < absolute {
            requested
        } else {
            absolute
        },
        limit <= requested,
        limit <= absolute,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawObservationLocation {
    RunId,
    AttemptId,
    Outcome,
    Stdout,
    Stderr,
    WallTime,
    CpuTime,
    Resources,
    ResourceExtension(u64),
    Coverage,
    StateDigest,
    ScheduleTrace,
    FaultTrace,
    Extension(u64),
}

pub open spec fn raw_observation_location_stable_tag_spec(location: RawObservationLocation) -> u16 {
    match location {
        RawObservationLocation::RunId => 1,
        RawObservationLocation::AttemptId => 2,
        RawObservationLocation::Outcome => 3,
        RawObservationLocation::Stdout => 4,
        RawObservationLocation::Stderr => 5,
        RawObservationLocation::WallTime => 6,
        RawObservationLocation::CpuTime => 7,
        RawObservationLocation::Resources => 8,
        RawObservationLocation::ResourceExtension(_) => 9,
        RawObservationLocation::Coverage => 10,
        RawObservationLocation::StateDigest => 11,
        RawObservationLocation::ScheduleTrace => 12,
        RawObservationLocation::FaultTrace => 13,
        RawObservationLocation::Extension(_) => 14,
    }
}

impl RawObservationLocation {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == raw_observation_location_stable_tag_spec(self),
    {
        match self {
            Self::RunId => 1,
            Self::AttemptId => 2,
            Self::Outcome => 3,
            Self::Stdout => 4,
            Self::Stderr => 5,
            Self::WallTime => 6,
            Self::CpuTime => 7,
            Self::Resources => 8,
            Self::ResourceExtension(_) => 9,
            Self::Coverage => 10,
            Self::StateDigest => 11,
            Self::ScheduleTrace => 12,
            Self::FaultTrace => 13,
            Self::Extension(_) => 14,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawObservationErrorKind {
    EmptyIdentity,
    IdentityLimitExceeded,
    InvalidOutcome,
    RetainedByteCountMismatch,
    TruncationFlagMismatch,
    ArtifactPayloadLimitExceeded,
    MalformedArtifactId,
    UnsupportedArtifactAlgorithm,
    EmptyMediaType,
    ResourceExtensionLimitExceeded,
    ExtensionLimitExceeded,
    EmptyExtensionNamespace,
    ZeroExtensionSchemaVersion,
    ExtensionNamespaceLimitExceeded,
    ExtensionMediaTypeLimitExceeded,
    EmptyCoverageProvider,
    EmptyCoverageProviderVersion,
    EmptyCoverageTargetBuild,
    EmptyFeatureSetDigest,
    CoverageCountMismatch,
    EmptyStateNamespace,
    ZeroStateSchemaVersion,
    ZeroScheduleSchemaVersion,
    ZeroFaultSchemaVersion,
    FaultTraceCountMismatch,
    InvalidDuration,
    EmptyScheduleNamespace,
    EmptyFaultNamespace,
}

pub open spec fn raw_observation_error_kind_stable_tag_spec(kind: RawObservationErrorKind) -> u16 {
    match kind {
        RawObservationErrorKind::EmptyIdentity => 1,
        RawObservationErrorKind::IdentityLimitExceeded => 2,
        RawObservationErrorKind::InvalidOutcome => 3,
        RawObservationErrorKind::RetainedByteCountMismatch => 4,
        RawObservationErrorKind::TruncationFlagMismatch => 5,
        RawObservationErrorKind::ArtifactPayloadLimitExceeded => 6,
        RawObservationErrorKind::MalformedArtifactId => 7,
        RawObservationErrorKind::UnsupportedArtifactAlgorithm => 8,
        RawObservationErrorKind::EmptyMediaType => 9,
        RawObservationErrorKind::ResourceExtensionLimitExceeded => 10,
        RawObservationErrorKind::ExtensionLimitExceeded => 11,
        RawObservationErrorKind::EmptyExtensionNamespace => 12,
        RawObservationErrorKind::ZeroExtensionSchemaVersion => 13,
        RawObservationErrorKind::ExtensionNamespaceLimitExceeded => 14,
        RawObservationErrorKind::ExtensionMediaTypeLimitExceeded => 15,
        RawObservationErrorKind::EmptyCoverageProvider => 16,
        RawObservationErrorKind::EmptyCoverageProviderVersion => 17,
        RawObservationErrorKind::EmptyCoverageTargetBuild => 18,
        RawObservationErrorKind::EmptyFeatureSetDigest => 19,
        RawObservationErrorKind::CoverageCountMismatch => 20,
        RawObservationErrorKind::EmptyStateNamespace => 21,
        RawObservationErrorKind::ZeroStateSchemaVersion => 22,
        RawObservationErrorKind::ZeroScheduleSchemaVersion => 23,
        RawObservationErrorKind::ZeroFaultSchemaVersion => 24,
        RawObservationErrorKind::FaultTraceCountMismatch => 25,
        RawObservationErrorKind::InvalidDuration => 26,
        RawObservationErrorKind::EmptyScheduleNamespace => 27,
        RawObservationErrorKind::EmptyFaultNamespace => 28,
    }
}

impl RawObservationErrorKind {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == raw_observation_error_kind_stable_tag_spec(self),
    {
        match self {
            Self::EmptyIdentity => 1,
            Self::IdentityLimitExceeded => 2,
            Self::InvalidOutcome => 3,
            Self::RetainedByteCountMismatch => 4,
            Self::TruncationFlagMismatch => 5,
            Self::ArtifactPayloadLimitExceeded => 6,
            Self::MalformedArtifactId => 7,
            Self::UnsupportedArtifactAlgorithm => 8,
            Self::EmptyMediaType => 9,
            Self::ResourceExtensionLimitExceeded => 10,
            Self::ExtensionLimitExceeded => 11,
            Self::EmptyExtensionNamespace => 12,
            Self::ZeroExtensionSchemaVersion => 13,
            Self::ExtensionNamespaceLimitExceeded => 14,
            Self::ExtensionMediaTypeLimitExceeded => 15,
            Self::EmptyCoverageProvider => 16,
            Self::EmptyCoverageProviderVersion => 17,
            Self::EmptyCoverageTargetBuild => 18,
            Self::EmptyFeatureSetDigest => 19,
            Self::CoverageCountMismatch => 20,
            Self::EmptyStateNamespace => 21,
            Self::ZeroStateSchemaVersion => 22,
            Self::ZeroScheduleSchemaVersion => 23,
            Self::ZeroFaultSchemaVersion => 24,
            Self::FaultTraceCountMismatch => 25,
            Self::InvalidDuration => 26,
            Self::EmptyScheduleNamespace => 27,
            Self::EmptyFaultNamespace => 28,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawObservationError {
    kind: RawObservationErrorKind,
    location: RawObservationLocation,
    code_point_index: Option<u64>,
    outcome_error_kind: Option<RawExecutionOutcomeErrorKind>,
    outcome_error_location: Option<RawExecutionOutcomeLocation>,
}

#[verifier::ext_equal]
pub struct RawObservationErrorView {
    pub kind: RawObservationErrorKind,
    pub location: RawObservationLocation,
    pub code_point_index: Option<u64>,
    pub outcome_error_kind: Option<RawExecutionOutcomeErrorKind>,
    pub outcome_error_location: Option<RawExecutionOutcomeLocation>,
}

impl View for RawObservationError {
    type V = RawObservationErrorView;

    closed spec fn view(&self) -> RawObservationErrorView {
        RawObservationErrorView {
            kind: self.kind,
            location: self.location,
            code_point_index: self.code_point_index,
            outcome_error_kind: self.outcome_error_kind,
            outcome_error_location: self.outcome_error_location,
        }
    }
}

impl RawObservationError {
    fn new(
        kind: RawObservationErrorKind,
        location: RawObservationLocation,
        code_point_index: Option<u64>,
    ) -> (error: Self)
        ensures
            error@ == (RawObservationErrorView {
                kind,
                location,
                code_point_index,
                outcome_error_kind: None,
                outcome_error_location: None,
            }),
    {
        Self {
            kind,
            location,
            code_point_index,
            outcome_error_kind: None,
            outcome_error_location: None,
        }
    }

    fn outcome(
        kind: RawExecutionOutcomeErrorKind,
        location: RawExecutionOutcomeLocation,
        code_point_index: Option<u64>,
    ) -> (error: Self)
        ensures
            error@ == (RawObservationErrorView {
                kind: RawObservationErrorKind::InvalidOutcome,
                location: RawObservationLocation::Outcome,
                code_point_index,
                outcome_error_kind: Some(kind),
                outcome_error_location: Some(location),
            }),
    {
        Self {
            kind: RawObservationErrorKind::InvalidOutcome,
            location: RawObservationLocation::Outcome,
            code_point_index,
            outcome_error_kind: Some(kind),
            outcome_error_location: Some(location),
        }
    }

    pub fn kind(&self) -> (value: RawObservationErrorKind) {
        self.kind
    }

    pub fn location(&self) -> (value: RawObservationLocation) {
        self.location
    }

    pub fn code_point_index(&self) -> (value: Option<u64>) {
        self.code_point_index
    }

    pub fn outcome_error_kind(&self) -> (value: Option<RawExecutionOutcomeErrorKind>) {
        self.outcome_error_kind
    }

    pub fn outcome_error_location(&self) -> (value: Option<RawExecutionOutcomeLocation>) {
        self.outcome_error_location
    }
}

pub open spec fn artifact_reference_semantics_spec(
    artifact: ArtifactRefView,
    payload_limit: u64,
) -> bool {
    crate::artifact::canonical_sha256_artifact_id_spec(artifact.id) && artifact.size_bytes
        <= payload_limit && (artifact.media_type is None || artifact.media_type.unwrap().len() > 0)
}

pub open spec fn captured_stream_semantics_spec(
    stream: CapturedStreamRefView,
    payload_limit: u64,
) -> bool {
    artifact_reference_semantics_spec(stream.artifact, payload_limit) && stream.retained_bytes
        == stream.artifact.size_bytes && stream.truncated == (stream.discarded_bytes > 0)
}

pub open spec fn extension_record_semantics_spec(
    extension: VersionedExtensionRefView,
    payload_limit: u64,
) -> bool {
    extension.namespace.len() > 0 && extension.schema_version > 0
        && artifact_reference_semantics_spec(extension.payload, payload_limit)
}

pub open spec fn extensions_semantics_spec(
    extensions: Seq<VersionedExtensionRefView>,
    payload_limit: u64,
) -> bool {
    forall|index: int|
        0 <= index < extensions.len() ==> extension_record_semantics_spec(
            #[trigger] extensions[index],
            payload_limit,
        )
}

pub open spec fn extension_namespace_total_spec(extensions: Seq<VersionedExtensionRefView>) -> nat
    decreases extensions.len(),
{
    if extensions.len() == 0 {
        0
    } else {
        extension_namespace_total_spec(extensions.drop_last()) + extensions.last().namespace.len()
    }
}

pub open spec fn extension_media_total_spec(extensions: Seq<VersionedExtensionRefView>) -> nat
    decreases extensions.len(),
{
    if extensions.len() == 0 {
        0
    } else {
        extension_media_total_spec(extensions.drop_last())
            + if extensions.last().payload.media_type is Some {
            extensions.last().payload.media_type.unwrap().len()
        } else {
            0
        }
    }
}

pub open spec fn artifact_media_type_len_spec(artifact: ArtifactRefView) -> nat {
    if artifact.media_type is Some {
        artifact.media_type.unwrap().len()
    } else {
        0
    }
}

pub open spec fn non_extension_media_total_spec(observation: RawObservationView) -> nat {
    artifact_media_type_len_spec(observation.stdout.artifact) + artifact_media_type_len_spec(
        observation.stderr.artifact,
    ) + if observation.coverage is Some {
        artifact_media_type_len_spec(observation.coverage.unwrap().artifact)
    } else {
        0
    } + if observation.state_digest is Some {
        artifact_media_type_len_spec(observation.state_digest.unwrap().artifact)
    } else {
        0
    } + if observation.schedule_trace is Some {
        artifact_media_type_len_spec(observation.schedule_trace.unwrap().artifact)
    } else {
        0
    } + if observation.fault_trace is Some {
        artifact_media_type_len_spec(observation.fault_trace.unwrap().artifact)
    } else {
        0
    }
}

proof fn lemma_extension_namespace_total_push(
    extensions: Seq<VersionedExtensionRefView>,
    extension: VersionedExtensionRefView,
)
    ensures
        extension_namespace_total_spec(extensions.push(extension))
            == extension_namespace_total_spec(extensions) + extension.namespace.len(),
{
    reveal(extension_namespace_total_spec);
    assert(extensions.push(extension).drop_last() == extensions);
    assert(extensions.push(extension).last() == extension);
}

proof fn lemma_extension_media_total_push(
    extensions: Seq<VersionedExtensionRefView>,
    extension: VersionedExtensionRefView,
)
    ensures
        extension_media_total_spec(extensions.push(extension)) == extension_media_total_spec(
            extensions,
        ) + if extension.payload.media_type is Some {
            extension.payload.media_type.unwrap().len()
        } else {
            0
        },
{
    reveal(extension_media_total_spec);
    assert(extensions.push(extension).drop_last() == extensions);
    assert(extensions.push(extension).last() == extension);
}

pub open spec fn fault_trace_counts_match_spec(trace: FaultTraceView) -> bool {
    trace.applied as int + trace.skipped as int + trace.shadowed as int + trace.rejected as int
        == trace.reached as int
}

pub open spec fn raw_observation_semantics_with_limits_spec(
    observation: RawObservationView,
    limits: RawObservationLimitsView,
) -> bool {
    let identity_limit = effective_raw_observation_identity_limit_spec(
        limits.max_identity_code_points,
    );
    let resource_limit = effective_raw_observation_resource_extension_limit_spec(
        limits.max_resource_extensions,
    );
    let extension_limit = effective_raw_observation_extension_limit_spec(limits.max_extensions);
    let namespace_limit = effective_raw_observation_namespace_limit_spec(
        limits.max_extension_namespace_code_points,
    );
    let media_limit = effective_raw_observation_media_limit_spec(
        limits.max_extension_media_type_code_points,
    );
    let payload_limit = effective_raw_observation_payload_limit_spec(
        limits.max_extension_payload_bytes_per_record,
    );
    let identity_total = observation.run_id.len() + observation.attempt_id.len()
        + if observation.coverage is Some {
        let coverage = observation.coverage.unwrap();
        coverage.provider.len() + coverage.provider_version.len() + coverage.target_build.len()
            + coverage.feature_set_digest.len()
    } else {
        0
    } + if observation.state_digest is Some {
        observation.state_digest.unwrap().namespace.len()
    } else {
        0
    } + if observation.schedule_trace is Some {
        observation.schedule_trace.unwrap().namespace.len()
    } else {
        0
    } + if observation.fault_trace is Some {
        observation.fault_trace.unwrap().namespace.len()
    } else {
        0
    };
    observation.run_id.len() > 0 && observation.attempt_id.len() > 0 && identity_total
        <= identity_limit && observation.resources.extensions.len() <= resource_limit
        && observation.extensions.len() <= extension_limit && captured_stream_semantics_spec(
        observation.stdout,
        payload_limit,
    ) && captured_stream_semantics_spec(observation.stderr, payload_limit)
        && observation.wall_time.nanoseconds < 1_000_000_000 && (observation.cpu_time is None
        || observation.cpu_time.unwrap().nanoseconds < 1_000_000_000) && extensions_semantics_spec(
        observation.resources.extensions,
        payload_limit,
    ) && extensions_semantics_spec(observation.extensions, payload_limit)
        && extension_namespace_total_spec(observation.resources.extensions)
        + extension_namespace_total_spec(observation.extensions) <= namespace_limit
        && non_extension_media_total_spec(observation) + extension_media_total_spec(
        observation.resources.extensions,
    ) + extension_media_total_spec(observation.extensions) <= media_limit && (
    observation.coverage is None || {
        let coverage = observation.coverage.unwrap();
        coverage.provider.len() > 0 && coverage.provider_version.len() > 0
            && coverage.target_build.len() > 0 && coverage.feature_set_digest.len() > 0
            && coverage.new_features <= coverage.total_features
            && artifact_reference_semantics_spec(coverage.artifact, payload_limit)
    }) && (observation.state_digest is None || {
        let state = observation.state_digest.unwrap();
        state.namespace.len() > 0 && state.schema_version > 0 && artifact_reference_semantics_spec(
            state.artifact,
            payload_limit,
        )
    }) && (observation.schedule_trace is None || {
        let trace = observation.schedule_trace.unwrap();
        trace.namespace.len() > 0 && trace.schema_version > 0 && artifact_reference_semantics_spec(
            trace.artifact,
            payload_limit,
        )
    }) && (observation.fault_trace is None || {
        let trace = observation.fault_trace.unwrap();
        trace.namespace.len() > 0 && trace.schema_version > 0 && fault_trace_counts_match_spec(
            trace,
        ) && artifact_reference_semantics_spec(trace.artifact, payload_limit)
    }) && crate::execution::validate_raw_execution_outcome_spec(
        observation.outcome,
        limits.outcome_limits,
    ) == Ok(observation.outcome)
}

pub open spec fn validate_raw_observation_spec(
    observation: RawObservationView,
    limits: RawObservationLimitsView,
) -> Result<RawObservationView, ()> {
    if raw_observation_semantics_with_limits_spec(observation, limits) {
        Ok(observation)
    } else {
        Err(())
    }
}

pub open spec fn raw_observation_semantics_spec(observation: RawObservationView) -> bool {
    raw_observation_semantics_with_limits_spec(
        observation,
        RawObservationLimitsView {
            outcome_limits: RawExecutionOutcomeLimitsView {
                max_events: crate::execution::MAX_RAW_EXECUTION_EVENTS,
                max_extension_namespace_code_points:
                    MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
                max_extension_media_type_code_points:
                    MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
                max_extension_payload_bytes_per_record:
                    MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
            },
            max_identity_code_points: MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS,
            max_resource_extensions: MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS,
            max_extensions: MAX_RAW_OBSERVATION_EXTENSIONS,
            max_extension_namespace_code_points: MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            max_extension_media_type_code_points:
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            max_extension_payload_bytes_per_record:
                MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        },
    )
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedRawObservation {
    observation: RawObservation,
}

impl View for ValidatedRawObservation {
    type V = RawObservationView;

    closed spec fn view(&self) -> RawObservationView {
        self.observation@
    }
}

impl ValidatedRawObservation {
    pub fn observation(&self) -> (value: &RawObservation)
        ensures
            value@ == self@,
    {
        &self.observation
    }

    pub fn into_inner(self) -> (value: RawObservation)
        ensures
            value@ == self@,
    {
        self.observation
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawObservationRejection {
    error: RawObservationError,
    observation: RawObservation,
}

#[verifier::ext_equal]
pub struct RawObservationRejectionView {
    pub error: RawObservationErrorView,
    pub observation: RawObservationView,
}

impl View for RawObservationRejection {
    type V = RawObservationRejectionView;

    closed spec fn view(&self) -> RawObservationRejectionView {
        RawObservationRejectionView { error: self.error@, observation: self.observation@ }
    }
}

impl RawObservationRejection {
    pub fn error(&self) -> (value: &RawObservationError) {
        &self.error
    }

    pub fn observation(&self) -> (value: &RawObservation) {
        &self.observation
    }

    pub fn into_parts(self) -> (value: (RawObservationError, RawObservation)) {
        (self.error, self.observation)
    }
}

fn validate_artifact_reference(
    artifact: &ArtifactRef,
    location: RawObservationLocation,
    payload_limit: u64,
) -> (result: Result<(), RawObservationError>)
    ensures
        result is Ok ==> artifact_reference_semantics_spec(artifact@, payload_limit),
{
    if artifact.size_bytes > payload_limit {
        return Err(
            RawObservationError::new(
                RawObservationErrorKind::ArtifactPayloadLimitExceeded,
                location,
                None,
            ),
        );
    }
    match parse_artifact_id(&artifact.id) {
        Ok(ContentDigest::Sha256(_digest)) => proof {
            crate::artifact::lemma_artifact_id_spec_is_canonical(_digest@);
        },
        Err(ArtifactIdParseError::MalformedArtifactId) => return Err(
            RawObservationError::new(RawObservationErrorKind::MalformedArtifactId, location, None),
        ),
        Err(ArtifactIdParseError::UnsupportedAlgorithm) => return Err(
            RawObservationError::new(
                RawObservationErrorKind::UnsupportedArtifactAlgorithm,
                location,
                None,
            ),
        ),
    }
    if let Some(media_type) = &artifact.media_type {
        if media_type.as_str().unicode_len() == 0 {
            return Err(
                RawObservationError::new(RawObservationErrorKind::EmptyMediaType, location, None),
            );
        }
    }
    Ok(())
}

fn validate_stream(
    stream: &CapturedStreamRef,
    location: RawObservationLocation,
    payload_limit: u64,
) -> (result: Result<(), RawObservationError>)
    ensures
        result is Ok ==> captured_stream_semantics_spec(stream@, payload_limit),
{
    validate_artifact_reference(&stream.artifact, location, payload_limit)?;
    if stream.retained_bytes != stream.artifact.size_bytes {
        return Err(
            RawObservationError::new(
                RawObservationErrorKind::RetainedByteCountMismatch,
                location,
                None,
            ),
        );
    }
    if stream.truncated != (stream.discarded_bytes > 0) {
        return Err(
            RawObservationError::new(
                RawObservationErrorKind::TruncationFlagMismatch,
                location,
                None,
            ),
        );
    }
    Ok(())
}

fn account_artifact_media_type(
    artifact: &ArtifactRef,
    location: RawObservationLocation,
    used: &mut u64,
    limit: u64,
) -> (result: Result<(), RawObservationError>)
    requires
        *old(used) <= limit,
    ensures
        result is Ok ==> *final(used) == *old(used) + artifact_media_type_len_spec(artifact@),
        result is Ok ==> *final(used) <= limit,
{
    if let Some(media_type) = &artifact.media_type {
        let length = media_type.as_str().unicode_len() as u64;
        let remaining = limit - *used;
        if length > remaining {
            return Err(
                RawObservationError::new(
                    RawObservationErrorKind::ExtensionMediaTypeLimitExceeded,
                    location,
                    Some(remaining),
                ),
            );
        }
        *used += length;
    }
    Ok(())
}

fn validate_extension_record(
    extension: &VersionedExtensionRef,
    location: RawObservationLocation,
    payload_limit: u64,
) -> (result: Result<(), RawObservationError>)
    ensures
        result is Ok ==> extension_record_semantics_spec(extension@, payload_limit),
{
    if extension.namespace().unicode_len() == 0 {
        return Err(
            RawObservationError::new(
                RawObservationErrorKind::EmptyExtensionNamespace,
                location,
                None,
            ),
        );
    }
    if extension.schema_version() == 0 {
        return Err(
            RawObservationError::new(
                RawObservationErrorKind::ZeroExtensionSchemaVersion,
                location,
                None,
            ),
        );
    }
    validate_artifact_reference(extension.payload(), location, payload_limit)
}

fn validate_identity_part(
    value: &str,
    location: RawObservationLocation,
    used: &mut u64,
    limit: u64,
    empty_kind: RawObservationErrorKind,
) -> (result: Result<(), RawObservationError>)
    requires
        *old(used) <= limit,
    ensures
        result is Ok ==> value@.len() > 0 && *final(used) == *old(used) + value@.len()
            && *final(used) <= limit,
{
    let length = value.unicode_len() as u64;
    if length == 0 {
        return Err(RawObservationError::new(empty_kind, location, None));
    }
    let remaining = limit - *used;
    if length > remaining {
        return Err(
            RawObservationError::new(
                RawObservationErrorKind::IdentityLimitExceeded,
                location,
                Some(remaining),
            ),
        );
    }
    *used += length;
    Ok(())
}

// Rejections deliberately retain the exact owned input for deterministic persistence and replay.
// Boxing would trade this lint for a mandatory error-path allocation without reducing retained data.
#[expect(
    clippy::result_large_err,
    reason = "rejections retain the exact owned observation for deterministic replay"
)]
// Verus expands the shorthand destructuring below into explicit field patterns before rustc sees it.
#[expect(
    non_shorthand_field_patterns,
    reason = "Verus macro expansion emits explicit field patterns before rustc linting"
)]
pub fn validate_raw_observation(
    observation: RawObservation,
    limits: RawObservationLimits,
) -> (result: Result<ValidatedRawObservation, RawObservationRejection>)
    ensures
        match &result {
            Ok(validated) => validated@ == observation@
                && raw_observation_semantics_with_limits_spec(validated@, limits@),
            Err(rejection) => rejection@.observation == observation@,
        },
{
    let identity_limit = effective_limit(
        limits.max_identity_code_points,
        MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS,
    );
    let resource_limit = effective_limit(
        limits.max_resource_extensions,
        MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS,
    );
    let extension_limit = effective_limit(limits.max_extensions, MAX_RAW_OBSERVATION_EXTENSIONS);
    let namespace_limit = effective_limit(
        limits.max_extension_namespace_code_points,
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    );
    let media_limit = effective_limit(
        limits.max_extension_media_type_code_points,
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    );
    let payload_limit = effective_limit(
        limits.max_extension_payload_bytes_per_record,
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    );

    let mut identity_used = 0u64;
    if let Err(error) = validate_identity_part(
        observation.run_id.as_str(),
        RawObservationLocation::RunId,
        &mut identity_used,
        identity_limit,
        RawObservationErrorKind::EmptyIdentity,
    ) {
        return Err(RawObservationRejection { error, observation });
    }
    if let Err(error) = validate_identity_part(
        observation.attempt_id.as_str(),
        RawObservationLocation::AttemptId,
        &mut identity_used,
        identity_limit,
        RawObservationErrorKind::EmptyIdentity,
    ) {
        return Err(RawObservationRejection { error, observation });
    }
    if observation.resources.extensions.len() as u64 > resource_limit {
        let error = RawObservationError::new(
            RawObservationErrorKind::ResourceExtensionLimitExceeded,
            RawObservationLocation::ResourceExtension(resource_limit),
            None,
        );
        return Err(RawObservationRejection { error, observation });
    }
    if observation.extensions.len() as u64 > extension_limit {
        let error = RawObservationError::new(
            RawObservationErrorKind::ExtensionLimitExceeded,
            RawObservationLocation::Extension(extension_limit),
            None,
        );
        return Err(RawObservationRejection { error, observation });
    }
    if let Err(error) = validate_stream(
        &observation.stdout,
        RawObservationLocation::Stdout,
        payload_limit,
    ) {
        return Err(RawObservationRejection { error, observation });
    }
    if let Err(error) = validate_stream(
        &observation.stderr,
        RawObservationLocation::Stderr,
        payload_limit,
    ) {
        return Err(RawObservationRejection { error, observation });
    }
    if observation.wall_time.nanoseconds >= 1_000_000_000 {
        let error = RawObservationError::new(
            RawObservationErrorKind::InvalidDuration,
            RawObservationLocation::WallTime,
            None,
        );
        return Err(RawObservationRejection { error, observation });
    }
    if let Some(cpu_time) = &observation.cpu_time {
        if cpu_time.nanoseconds >= 1_000_000_000 {
            let error = RawObservationError::new(
                RawObservationErrorKind::InvalidDuration,
                RawObservationLocation::CpuTime,
                None,
            );
            return Err(RawObservationRejection { error, observation });
        }
    }
    let mut media_used = 0u64;
    if let Err(error) = account_artifact_media_type(
        &observation.stdout.artifact,
        RawObservationLocation::Stdout,
        &mut media_used,
        media_limit,
    ) {
        return Err(RawObservationRejection { error, observation });
    }
    if let Err(error) = account_artifact_media_type(
        &observation.stderr.artifact,
        RawObservationLocation::Stderr,
        &mut media_used,
        media_limit,
    ) {
        return Err(RawObservationRejection { error, observation });
    }
    let mut namespace_used = 0u64;
    let mut index = 0usize;
    proof {
        reveal(<RawObservation as View>::view);
        reveal(<ResourceSnapshot as View>::view);
        lemma_observation_extension_views_properties(observation.resources.extensions@);
        lemma_observation_extension_views_properties(observation.extensions@);
    }
    while index < observation.resources.extensions.len()
        invariant
            index <= observation.resources.extensions.len(),
            observation@.resources.extensions.len() == observation.resources.extensions.len(),
            namespace_used <= namespace_limit,
            media_used <= media_limit,
            namespace_used as nat == extension_namespace_total_spec(
                observation@.resources.extensions.take(index as int),
            ),
            media_used as nat == artifact_media_type_len_spec(observation@.stdout.artifact)
                + artifact_media_type_len_spec(observation@.stderr.artifact)
                + extension_media_total_spec(observation@.resources.extensions.take(index as int)),
            forall|prior: int|
                0 <= prior < index ==> extension_record_semantics_spec(
                    #[trigger] observation@.resources.extensions[prior],
                    payload_limit,
                ),
        decreases observation.resources.extensions.len() - index,
    {
        let location = RawObservationLocation::ResourceExtension(index as u64);
        let extension = &observation.resources.extensions[index];
        if let Err(error) = validate_extension_record(extension, location, payload_limit) {
            return Err(RawObservationRejection { error, observation });
        }
        assert(extension_record_semantics_spec(extension@, payload_limit));
        assert(observation@.resources.extensions == observation_extension_views_spec(
            observation.resources.extensions@,
        ));
        proof {
            lemma_observation_extension_view_at(observation.resources.extensions@, index as int);
        }
        assert(observation.resources.extensions@[index as int]@ == extension@);
        assert(observation@.resources.extensions[index as int] == extension@);
        let namespace_length = extension.namespace().unicode_len() as u64;
        if namespace_length > namespace_limit - namespace_used {
            let error = RawObservationError::new(
                RawObservationErrorKind::ExtensionNamespaceLimitExceeded,
                location,
                Some(namespace_limit - namespace_used),
            );
            return Err(RawObservationRejection { error, observation });
        }
        namespace_used += namespace_length;
        if let Some(media_type) = &extension.payload().media_type {
            let media_length = media_type.as_str().unicode_len() as u64;
            if media_length > media_limit - media_used {
                let error = RawObservationError::new(
                    RawObservationErrorKind::ExtensionMediaTypeLimitExceeded,
                    location,
                    Some(media_limit - media_used),
                );
                return Err(RawObservationRejection { error, observation });
            }
            media_used += media_length;
        }
        proof {
            let prefix = observation@.resources.extensions.take(index as int);
            observation@.resources.extensions.lemma_take_succ_push(index as int);
            assert(observation@.resources.extensions.take(index as int + 1) == prefix.push(
                observation@.resources.extensions[index as int],
            ));
            lemma_extension_namespace_total_push(prefix, extension@);
            lemma_extension_media_total_push(prefix, extension@);
        }
        index += 1;
    }

    if let Some(coverage) = &observation.coverage {
        macro_rules! identity {
            ($value:expr, $kind:expr) => {
                if let Err(error) = validate_identity_part($value, RawObservationLocation::Coverage, &mut identity_used, identity_limit, $kind) {
                    return Err(RawObservationRejection { error, observation });
                }
            };
        }
        identity!(coverage.provider.as_str(), RawObservationErrorKind::EmptyCoverageProvider);
        identity!(coverage.provider_version.as_str(), RawObservationErrorKind::EmptyCoverageProviderVersion);
        identity!(coverage.target_build.as_str(), RawObservationErrorKind::EmptyCoverageTargetBuild);
        identity!(coverage.feature_set_digest.as_str(), RawObservationErrorKind::EmptyFeatureSetDigest);
        if coverage.new_features > coverage.total_features {
            let error = RawObservationError::new(
                RawObservationErrorKind::CoverageCountMismatch,
                RawObservationLocation::Coverage,
                None,
            );
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = validate_artifact_reference(
            &coverage.artifact,
            RawObservationLocation::Coverage,
            payload_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = account_artifact_media_type(
            &coverage.artifact,
            RawObservationLocation::Coverage,
            &mut media_used,
            media_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
    }
    if let Some(state) = &observation.state_digest {
        if let Err(error) = validate_identity_part(
            state.namespace.as_str(),
            RawObservationLocation::StateDigest,
            &mut identity_used,
            identity_limit,
            RawObservationErrorKind::EmptyStateNamespace,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if state.schema_version == 0 {
            let error = RawObservationError::new(
                RawObservationErrorKind::ZeroStateSchemaVersion,
                RawObservationLocation::StateDigest,
                None,
            );
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = validate_artifact_reference(
            &state.artifact,
            RawObservationLocation::StateDigest,
            payload_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = account_artifact_media_type(
            &state.artifact,
            RawObservationLocation::StateDigest,
            &mut media_used,
            media_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
    }
    if let Some(trace) = &observation.schedule_trace {
        if let Err(error) = validate_identity_part(
            trace.namespace.as_str(),
            RawObservationLocation::ScheduleTrace,
            &mut identity_used,
            identity_limit,
            RawObservationErrorKind::EmptyScheduleNamespace,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if trace.schema_version == 0 {
            let error = RawObservationError::new(
                RawObservationErrorKind::ZeroScheduleSchemaVersion,
                RawObservationLocation::ScheduleTrace,
                None,
            );
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = validate_artifact_reference(
            &trace.artifact,
            RawObservationLocation::ScheduleTrace,
            payload_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = account_artifact_media_type(
            &trace.artifact,
            RawObservationLocation::ScheduleTrace,
            &mut media_used,
            media_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
    }
    if let Some(trace) = &observation.fault_trace {
        if let Err(error) = validate_identity_part(
            trace.namespace.as_str(),
            RawObservationLocation::FaultTrace,
            &mut identity_used,
            identity_limit,
            RawObservationErrorKind::EmptyFaultNamespace,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if trace.schema_version == 0 {
            let error = RawObservationError::new(
                RawObservationErrorKind::ZeroFaultSchemaVersion,
                RawObservationLocation::FaultTrace,
                None,
            );
            return Err(RawObservationRejection { error, observation });
        }
        let counts_match = trace.applied <= trace.reached && trace.skipped <= trace.reached
            - trace.applied && trace.shadowed <= trace.reached - trace.applied - trace.skipped
            && trace.rejected == trace.reached - trace.applied - trace.skipped - trace.shadowed;
        if !counts_match {
            let error = RawObservationError::new(
                RawObservationErrorKind::FaultTraceCountMismatch,
                RawObservationLocation::FaultTrace,
                None,
            );
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = validate_artifact_reference(
            &trace.artifact,
            RawObservationLocation::FaultTrace,
            payload_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
        if let Err(error) = account_artifact_media_type(
            &trace.artifact,
            RawObservationLocation::FaultTrace,
            &mut media_used,
            media_limit,
        ) {
            return Err(RawObservationRejection { error, observation });
        }
    }
    assert(observation@.resources.extensions.take(observation@.resources.extensions.len() as int)
        == observation@.resources.extensions);
    assert(extension_namespace_total_spec(observation@.resources.extensions)
        == namespace_used as nat);
    assert(media_used as nat == non_extension_media_total_spec(observation@)
        + extension_media_total_spec(observation@.resources.extensions));
    assert(forall|prior: int|
        0 <= prior < observation@.resources.extensions.len() ==> extension_record_semantics_spec(
            #[trigger] observation@.resources.extensions[prior],
            payload_limit,
        ));
    index = 0;
    while index < observation.extensions.len()
        invariant
            index <= observation.extensions.len(),
            observation@.extensions.len() == observation.extensions.len(),
            namespace_used <= namespace_limit,
            media_used <= media_limit,
            namespace_used as nat == extension_namespace_total_spec(
                observation@.resources.extensions,
            ) + extension_namespace_total_spec(observation@.extensions.take(index as int)),
            media_used as nat == non_extension_media_total_spec(observation@)
                + extension_media_total_spec(observation@.resources.extensions)
                + extension_media_total_spec(observation@.extensions.take(index as int)),
            forall|prior: int|
                0 <= prior < observation@.resources.extensions.len()
                    ==> extension_record_semantics_spec(
                    #[trigger] observation@.resources.extensions[prior],
                    payload_limit,
                ),
            forall|prior: int|
                0 <= prior < index ==> extension_record_semantics_spec(
                    #[trigger] observation@.extensions[prior],
                    payload_limit,
                ),
        decreases observation.extensions.len() - index,
    {
        let location = RawObservationLocation::Extension(index as u64);
        let extension = &observation.extensions[index];
        if let Err(error) = validate_extension_record(extension, location, payload_limit) {
            return Err(RawObservationRejection { error, observation });
        }
        assert(extension_record_semantics_spec(extension@, payload_limit));
        assert(observation@.extensions == observation_extension_views_spec(
            observation.extensions@,
        ));
        proof {
            lemma_observation_extension_view_at(observation.extensions@, index as int);
        }
        assert(observation.extensions@[index as int]@ == extension@);
        assert(observation@.extensions[index as int] == extension@);
        let namespace_length = extension.namespace().unicode_len() as u64;
        if namespace_length > namespace_limit - namespace_used {
            let error = RawObservationError::new(
                RawObservationErrorKind::ExtensionNamespaceLimitExceeded,
                location,
                Some(namespace_limit - namespace_used),
            );
            return Err(RawObservationRejection { error, observation });
        }
        namespace_used += namespace_length;
        if let Some(media_type) = &extension.payload().media_type {
            let media_length = media_type.as_str().unicode_len() as u64;
            if media_length > media_limit - media_used {
                let error = RawObservationError::new(
                    RawObservationErrorKind::ExtensionMediaTypeLimitExceeded,
                    location,
                    Some(media_limit - media_used),
                );
                return Err(RawObservationRejection { error, observation });
            }
            media_used += media_length;
        }
        proof {
            let prefix = observation@.extensions.take(index as int);
            observation@.extensions.lemma_take_succ_push(index as int);
            assert(observation@.extensions.take(index as int + 1) == prefix.push(
                observation@.extensions[index as int],
            ));
            lemma_extension_namespace_total_push(prefix, extension@);
            lemma_extension_media_total_push(prefix, extension@);
        }
        index += 1;
    }
    assert(observation@.extensions.take(observation@.extensions.len() as int)
        == observation@.extensions);
    assert(extension_namespace_total_spec(observation@.resources.extensions)
        + extension_namespace_total_spec(observation@.extensions) == namespace_used as nat);
    assert(non_extension_media_total_spec(observation@) + extension_media_total_spec(
        observation@.resources.extensions,
    ) + extension_media_total_spec(observation@.extensions) == media_used as nat);

    let RawObservation {
        run_id,
        attempt_id,
        outcome,
        stdout,
        stderr,
        wall_time,
        cpu_time,
        peak_rss_bytes,
        resources,
        coverage,
        state_digest,
        schedule_trace,
        fault_trace,
        extensions,
    } = observation;
    let ghost outcome_view = outcome@;
    let ghost outcome_limits_view = limits.outcome_limits@;
    let outcome = match validate_raw_execution_outcome(outcome, limits.outcome_limits) {
        Ok(validated) => {
            proof {
                crate::execution::lemma_successful_raw_execution_validation_has_semantics(
                    outcome_view,
                    outcome_limits_view,
                    validated@,
                );
            }
            validated.into_inner()
        },
        Err(rejection) => {
            let (nested, outcome) = rejection.into_parts();
            let error = RawObservationError::outcome(
                nested.kind(),
                nested.location(),
                nested.extension_code_point_index(),
            );
            let observation = RawObservation {
                run_id,
                attempt_id,
                outcome,
                stdout,
                stderr,
                wall_time,
                cpu_time,
                peak_rss_bytes,
                resources,
                coverage,
                state_digest,
                schedule_trace,
                fault_trace,
                extensions,
            };
            return Err(RawObservationRejection { error, observation });
        },
    };
    let observation = RawObservation {
        run_id,
        attempt_id,
        outcome,
        stdout,
        stderr,
        wall_time,
        cpu_time,
        peak_rss_bytes,
        resources,
        coverage,
        state_digest,
        schedule_trace,
        fault_trace,
        extensions,
    };
    proof {
        reveal(raw_observation_semantics_with_limits_spec);
        reveal(extensions_semantics_spec);
        assert(observation@.run_id.len() > 0);
        assert(observation@.attempt_id.len() > 0);
        assert(observation@.resources.extensions.len() <= resource_limit);
        assert(observation@.extensions.len() <= extension_limit);
        assert(captured_stream_semantics_spec(observation@.stdout, payload_limit));
        assert(captured_stream_semantics_spec(observation@.stderr, payload_limit));
        assert(observation@.wall_time.nanoseconds < 1_000_000_000);
        assert(observation@.cpu_time is None || observation@.cpu_time.unwrap().nanoseconds
            < 1_000_000_000);
        assert(extensions_semantics_spec(observation@.resources.extensions, payload_limit));
        assert(extensions_semantics_spec(observation@.extensions, payload_limit));
        assert(extension_namespace_total_spec(observation@.resources.extensions)
            + extension_namespace_total_spec(observation@.extensions) <= namespace_limit);
        assert(non_extension_media_total_spec(observation@) + extension_media_total_spec(
            observation@.resources.extensions,
        ) + extension_media_total_spec(observation@.extensions) <= media_limit);
        assert(observation@.coverage is None || {
            let coverage = observation@.coverage.unwrap();
            coverage.provider.len() > 0 && coverage.provider_version.len() > 0
                && coverage.target_build.len() > 0 && coverage.feature_set_digest.len() > 0
                && coverage.new_features <= coverage.total_features
                && artifact_reference_semantics_spec(coverage.artifact, payload_limit)
        });
        assert(observation@.state_digest is None || {
            let state = observation@.state_digest.unwrap();
            state.namespace.len() > 0 && state.schema_version > 0
                && artifact_reference_semantics_spec(state.artifact, payload_limit)
        });
        assert(observation@.schedule_trace is None || {
            let trace = observation@.schedule_trace.unwrap();
            trace.namespace.len() > 0 && trace.schema_version > 0
                && artifact_reference_semantics_spec(trace.artifact, payload_limit)
        });
        assert(observation@.fault_trace is None || {
            let trace = observation@.fault_trace.unwrap();
            trace.namespace.len() > 0 && trace.schema_version > 0 && fault_trace_counts_match_spec(
                trace,
            ) && artifact_reference_semantics_spec(trace.artifact, payload_limit)
        });
        assert(crate::execution::validate_raw_execution_outcome_spec(
            observation@.outcome,
            limits@.outcome_limits,
        ) == Ok(observation@.outcome));
    }
    Ok(ValidatedRawObservation { observation })
}

pub proof fn lemma_raw_observation_validation_preserves_exact_input(
    observation: RawObservationView,
    limits: RawObservationLimitsView,
)
    ensures
        forall|validated: RawObservationView|
            validate_raw_observation_spec(observation, limits) == Ok(validated) ==> validated
                == observation,
{
    reveal(validate_raw_observation_spec);
}

pub proof fn lemma_successful_raw_observation_validation_has_semantics(
    observation: RawObservationView,
    limits: RawObservationLimitsView,
)
    requires
        validate_raw_observation_spec(observation, limits) == Ok(observation),
    ensures
        raw_observation_semantics_with_limits_spec(observation, limits),
{
    reveal(validate_raw_observation_spec);
}

pub proof fn lemma_raw_observation_semantics_enforces_supplied_outcome_limits(
    observation: RawObservationView,
    limits: RawObservationLimitsView,
)
    ensures
        raw_observation_semantics_with_limits_spec(observation, limits)
            ==> crate::execution::validate_raw_execution_outcome_spec(
            observation.outcome,
            limits.outcome_limits,
        ) == Ok(observation.outcome),
{
    reveal(raw_observation_semantics_with_limits_spec);
}

} // verus!
