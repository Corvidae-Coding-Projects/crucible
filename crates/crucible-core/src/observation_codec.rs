//! Canonical, bounded byte serialization for immutable raw observations.
use crate::artifact::{parse_artifact_id, ArtifactIdParseError, ContentDigest};
use crate::execution::{
    RawExecutionOutcomeErrorKind, RawExecutionOutcomeLocation, VersionedExtensionRef,
};
use crate::execution_codec::{
    append_string, append_u16, append_u32, append_u64, decode_raw_execution_outcome,
    encode_extension, encode_raw_execution_outcome_value, push_encoded_byte, read_string, read_u16,
    read_u32, read_u64, read_u8, RawExecutionOutcomeCodecErrorKind, RawExecutionOutcomeCodecLimits,
};
use crate::observation::{
    canonical_raw_observation_limits, validate_raw_observation, CapturedStreamRef, CoverageRef,
    FaultTrace, RawObservation, RawObservationErrorKind, RawObservationLimits,
    RawObservationLimitsView, RawObservationLocation, RecordedDuration, ResourceSnapshot,
    ScheduleTrace, StateDigest, ValidatedRawObservation, MAX_RAW_OBSERVATION_EXTENSIONS,
    MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS, RAW_OBSERVATION_SCHEMA_VERSION,
};
use crate::{ArtifactId, ArtifactRef, CoverageProviderId, RunAttemptId, RunId, TargetBuildId};
use vstd::prelude::*;

verus! {

pub const MAX_RAW_OBSERVATION_ENCODED_BYTES: u64 = 134_217_728;

const MAGIC_0: u8 = b'C';

const MAGIC_1: u8 = b'R';

const MAGIC_2: u8 = b'O';

const MAGIC_3: u8 = b'B';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawObservationCodecLimits {
    max_encoded_bytes: u64,
    observation_limits: RawObservationLimits,
}

#[verifier::ext_equal]
pub struct RawObservationCodecLimitsView {
    pub max_encoded_bytes: u64,
    pub observation_limits: RawObservationLimitsView,
}

impl View for RawObservationCodecLimits {
    type V = RawObservationCodecLimitsView;

    closed spec fn view(&self) -> RawObservationCodecLimitsView {
        RawObservationCodecLimitsView {
            max_encoded_bytes: self.max_encoded_bytes,
            observation_limits: self.observation_limits@,
        }
    }
}

impl RawObservationCodecLimits {
    pub fn new(max_encoded_bytes: u64, observation_limits: RawObservationLimits) -> (limits: Self)
        ensures
            limits@ == (RawObservationCodecLimitsView {
                max_encoded_bytes,
                observation_limits: observation_limits@,
            }),
    {
        Self { max_encoded_bytes, observation_limits }
    }

    pub fn max_encoded_bytes(&self) -> (value: u64) {
        self.max_encoded_bytes
    }

    pub fn observation_limits(&self) -> (value: RawObservationLimits) {
        self.observation_limits
    }
}

pub fn canonical_raw_observation_codec_limits() -> (limits: RawObservationCodecLimits) {
    RawObservationCodecLimits::new(
        MAX_RAW_OBSERVATION_ENCODED_BYTES,
        canonical_raw_observation_limits(),
    )
}

pub open spec fn effective_raw_observation_encoded_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_OBSERVATION_ENCODED_BYTES {
        requested
    } else {
        MAX_RAW_OBSERVATION_ENCODED_BYTES
    }
}

fn effective_encoded_limit(requested: u64) -> (limit: u64)
    ensures
        limit == effective_raw_observation_encoded_limit_spec(requested),
        limit <= requested,
        limit <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
{
    if requested < MAX_RAW_OBSERVATION_ENCODED_BYTES {
        requested
    } else {
        MAX_RAW_OBSERVATION_ENCODED_BYTES
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawObservationCodecErrorKind {
    EncodedByteLimitExceeded,
    Truncated,
    InvalidMagic,
    UnsupportedSchemaVersion,
    InvalidUtf8,
    StringLengthLimitExceeded,
    NestedOutcomeRejected,
    UnknownBooleanTag,
    InvalidDuration,
    DeclaredResourceExtensionLimitExceeded,
    DeclaredExtensionLimitExceeded,
    SemanticValidationFailed,
    TrailingBytes,
    InvalidOptionTag,
}

pub open spec fn raw_observation_codec_error_kind_stable_tag_spec(
    kind: RawObservationCodecErrorKind,
) -> u16 {
    match kind {
        RawObservationCodecErrorKind::EncodedByteLimitExceeded => 1,
        RawObservationCodecErrorKind::Truncated => 2,
        RawObservationCodecErrorKind::InvalidMagic => 3,
        RawObservationCodecErrorKind::UnsupportedSchemaVersion => 4,
        RawObservationCodecErrorKind::InvalidUtf8 => 5,
        RawObservationCodecErrorKind::StringLengthLimitExceeded => 6,
        RawObservationCodecErrorKind::NestedOutcomeRejected => 7,
        RawObservationCodecErrorKind::UnknownBooleanTag => 8,
        RawObservationCodecErrorKind::InvalidDuration => 9,
        RawObservationCodecErrorKind::DeclaredResourceExtensionLimitExceeded => 10,
        RawObservationCodecErrorKind::DeclaredExtensionLimitExceeded => 11,
        RawObservationCodecErrorKind::SemanticValidationFailed => 12,
        RawObservationCodecErrorKind::TrailingBytes => 13,
        RawObservationCodecErrorKind::InvalidOptionTag => 14,
    }
}

impl RawObservationCodecErrorKind {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == raw_observation_codec_error_kind_stable_tag_spec(self),
    {
        match self {
            Self::EncodedByteLimitExceeded => 1,
            Self::Truncated => 2,
            Self::InvalidMagic => 3,
            Self::UnsupportedSchemaVersion => 4,
            Self::InvalidUtf8 => 5,
            Self::StringLengthLimitExceeded => 6,
            Self::NestedOutcomeRejected => 7,
            Self::UnknownBooleanTag => 8,
            Self::InvalidDuration => 9,
            Self::DeclaredResourceExtensionLimitExceeded => 10,
            Self::DeclaredExtensionLimitExceeded => 11,
            Self::SemanticValidationFailed => 12,
            Self::TrailingBytes => 13,
            Self::InvalidOptionTag => 14,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawObservationCodecError {
    kind: RawObservationCodecErrorKind,
    byte_offset: u64,
    record_index: Option<u64>,
    nested_error_tag: Option<u16>,
    semantic_error_kind: Option<RawObservationErrorKind>,
    semantic_error_location: Option<RawObservationLocation>,
    code_point_index: Option<u64>,
    outcome_error_kind: Option<RawExecutionOutcomeErrorKind>,
    outcome_error_location: Option<RawExecutionOutcomeLocation>,
}

#[verifier::ext_equal]
pub struct RawObservationCodecErrorView {
    pub kind: RawObservationCodecErrorKind,
    pub byte_offset: u64,
    pub record_index: Option<u64>,
    pub nested_error_tag: Option<u16>,
    pub semantic_error_kind: Option<RawObservationErrorKind>,
    pub semantic_error_location: Option<RawObservationLocation>,
    pub code_point_index: Option<u64>,
    pub outcome_error_kind: Option<RawExecutionOutcomeErrorKind>,
    pub outcome_error_location: Option<RawExecutionOutcomeLocation>,
}

impl View for RawObservationCodecError {
    type V = RawObservationCodecErrorView;

    closed spec fn view(&self) -> RawObservationCodecErrorView {
        RawObservationCodecErrorView {
            kind: self.kind,
            byte_offset: self.byte_offset,
            record_index: self.record_index,
            nested_error_tag: self.nested_error_tag,
            semantic_error_kind: self.semantic_error_kind,
            semantic_error_location: self.semantic_error_location,
            code_point_index: self.code_point_index,
            outcome_error_kind: self.outcome_error_kind,
            outcome_error_location: self.outcome_error_location,
        }
    }
}

impl RawObservationCodecError {
    fn new(kind: RawObservationCodecErrorKind, byte_offset: u64) -> Self {
        Self {
            kind,
            byte_offset,
            record_index: None,
            nested_error_tag: None,
            semantic_error_kind: None,
            semantic_error_location: None,
            code_point_index: None,
            outcome_error_kind: None,
            outcome_error_location: None,
        }
    }

    fn indexed(kind: RawObservationCodecErrorKind, byte_offset: u64, record_index: u64) -> Self {
        Self {
            kind,
            byte_offset,
            record_index: Some(record_index),
            nested_error_tag: None,
            semantic_error_kind: None,
            semantic_error_location: None,
            code_point_index: None,
            outcome_error_kind: None,
            outcome_error_location: None,
        }
    }

    fn nested(
        byte_offset: u64,
        nested_error_tag: u16,
        code_point_index: Option<u64>,
        outcome_error_kind: Option<RawExecutionOutcomeErrorKind>,
        outcome_error_location: Option<RawExecutionOutcomeLocation>,
    ) -> Self {
        Self {
            kind: RawObservationCodecErrorKind::NestedOutcomeRejected,
            byte_offset,
            record_index: None,
            nested_error_tag: Some(nested_error_tag),
            semantic_error_kind: None,
            semantic_error_location: None,
            code_point_index,
            outcome_error_kind,
            outcome_error_location,
        }
    }

    fn semantic(
        byte_offset: u64,
        semantic_error_kind: RawObservationErrorKind,
        semantic_error_location: RawObservationLocation,
        code_point_index: Option<u64>,
        outcome_error_kind: Option<RawExecutionOutcomeErrorKind>,
        outcome_error_location: Option<RawExecutionOutcomeLocation>,
    ) -> Self {
        Self {
            kind: RawObservationCodecErrorKind::SemanticValidationFailed,
            byte_offset,
            record_index: None,
            nested_error_tag: None,
            semantic_error_kind: Some(semantic_error_kind),
            semantic_error_location: Some(semantic_error_location),
            code_point_index,
            outcome_error_kind,
            outcome_error_location,
        }
    }

    pub fn kind(&self) -> (value: RawObservationCodecErrorKind) {
        self.kind
    }

    pub fn byte_offset(&self) -> (value: u64) {
        self.byte_offset
    }

    pub fn record_index(&self) -> (value: Option<u64>) {
        self.record_index
    }

    pub fn nested_error_tag(&self) -> (value: Option<u16>) {
        self.nested_error_tag
    }

    pub fn semantic_error_kind(&self) -> (value: Option<RawObservationErrorKind>) {
        self.semantic_error_kind
    }

    pub fn semantic_error_location(&self) -> (value: Option<RawObservationLocation>) {
        self.semantic_error_location
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

#[derive(Debug, PartialEq, Eq)]
pub struct RawObservationCodecRejection {
    error: RawObservationCodecError,
    encoded: Vec<u8>,
}

#[verifier::ext_equal]
pub struct RawObservationCodecRejectionView {
    pub error: RawObservationCodecErrorView,
    pub encoded: Seq<u8>,
}

impl View for RawObservationCodecRejection {
    type V = RawObservationCodecRejectionView;

    closed spec fn view(&self) -> RawObservationCodecRejectionView {
        RawObservationCodecRejectionView { error: self.error@, encoded: self.encoded@ }
    }
}

impl RawObservationCodecRejection {
    pub fn error(&self) -> (value: &RawObservationCodecError) {
        &self.error
    }

    pub fn encoded(&self) -> (value: &[u8]) {
        self.encoded.as_slice()
    }

    pub fn into_encoded(self) -> (value: Vec<u8>) {
        self.encoded
    }
}

pub open spec fn raw_observation_decode_contract_spec(
    encoded: Seq<u8>,
    limits: RawObservationCodecLimitsView,
    result: Result<crate::observation::RawObservationView, RawObservationCodecRejectionView>,
) -> bool {
    match result {
        Ok(observation) => encoded.len() <= effective_raw_observation_encoded_limit_spec(
            limits.max_encoded_bytes,
        ) && crate::observation::raw_observation_semantics_with_limits_spec(
            observation,
            limits.observation_limits,
        ),
        Err(rejection) => rejection.encoded == encoded,
    }
}

pub proof fn lemma_raw_observation_decode_contract_rejection_preserves_bytes(
    encoded: Seq<u8>,
    limits: RawObservationCodecLimitsView,
    rejection: RawObservationCodecRejectionView,
)
    requires
        raw_observation_decode_contract_spec(encoded, limits, Err(rejection)),
    ensures
        rejection.encoded == encoded,
{
    reveal(raw_observation_decode_contract_spec);
}

pub proof fn lemma_raw_observation_decode_contract_success_has_semantics(
    encoded: Seq<u8>,
    limits: RawObservationCodecLimitsView,
    observation: crate::observation::RawObservationView,
)
    requires
        raw_observation_decode_contract_spec(encoded, limits, Ok(observation)),
    ensures
        encoded.len() <= effective_raw_observation_encoded_limit_spec(limits.max_encoded_bytes),
        crate::observation::raw_observation_semantics_with_limits_spec(
            observation,
            limits.observation_limits,
        ),
{
    reveal(raw_observation_decode_contract_spec);
}

fn append_bool(output: &mut Vec<u8>, value: bool, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    push_encoded_byte(
        output,
        if value {
            1
        } else {
            0
        },
        limit,
    )
}

fn append_optional_u64(output: &mut Vec<u8>, value: Option<u64>, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    match value {
        None => push_encoded_byte(output, 0, limit),
        Some(number) => push_encoded_byte(output, 1, limit) && append_u64(output, number, limit),
    }
}

fn append_duration(output: &mut Vec<u8>, value: &RecordedDuration, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    append_u64(output, value.seconds(), limit) && append_u32(output, value.nanoseconds(), limit)
}

fn append_optional_duration(
    output: &mut Vec<u8>,
    value: &Option<RecordedDuration>,
    limit: u64,
) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    match value {
        None => push_encoded_byte(output, 0, limit),
        Some(duration) => push_encoded_byte(output, 1, limit) && append_duration(
            output,
            duration,
            limit,
        ),
    }
}

fn append_artifact(output: &mut Vec<u8>, artifact: &ArtifactRef, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    append_u64(output, artifact.size_bytes, limit) && append_string(
        output,
        artifact.id.as_str(),
        limit,
    ) && match &artifact.media_type {
        None => push_encoded_byte(output, 0, limit),
        Some(media_type) => push_encoded_byte(output, 1, limit) && append_string(
            output,
            media_type.as_str(),
            limit,
        ),
    }
}

fn append_stream(output: &mut Vec<u8>, stream: &CapturedStreamRef, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    append_artifact(output, stream.artifact(), limit) && append_bool(
        output,
        stream.truncated(),
        limit,
    ) && append_u64(output, stream.retained_bytes(), limit) && append_u64(
        output,
        stream.discarded_bytes(),
        limit,
    )
}

fn append_extensions(
    output: &mut Vec<u8>,
    extensions: &[VersionedExtensionRef],
    limit: u64,
) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    if !append_u64(output, extensions.len() as u64, limit) {
        return false;
    }
    let mut index = 0usize;
    while index < extensions.len()
        invariant
            index <= extensions.len(),
            output@.len() <= limit,
            limit <= crate::execution_codec::MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
        decreases extensions.len() - index,
    {
        if !encode_extension(output, &extensions[index], limit) {
            return false;
        }
        index += 1;
    }
    true
}

pub fn encode_raw_observation(
    observation: &ValidatedRawObservation,
    requested_limit: u64,
) -> (result: Result<Vec<u8>, RawObservationCodecError>)
    ensures
        match &result {
            Ok(encoded) => encoded@.len() <= effective_raw_observation_encoded_limit_spec(
                requested_limit,
            ),
            Err(_) => true,
        },
{
    let limit = effective_encoded_limit(requested_limit);
    let value = observation.observation();
    let nested = match encode_raw_execution_outcome_value(value.outcome(), limit) {
        Ok(encoded) => encoded,
        Err(error) => return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::EncodedByteLimitExceeded,
                error.byte_offset(),
            ),
        ),
    };
    let mut output = Vec::new();
    let mut accepted = push_encoded_byte(&mut output, MAGIC_0, limit) && push_encoded_byte(
        &mut output,
        MAGIC_1,
        limit,
    ) && push_encoded_byte(&mut output, MAGIC_2, limit) && push_encoded_byte(
        &mut output,
        MAGIC_3,
        limit,
    ) && append_u16(&mut output, RAW_OBSERVATION_SCHEMA_VERSION, limit) && append_string(
        &mut output,
        value.run_id().as_str(),
        limit,
    ) && append_string(&mut output, value.attempt_id().as_str(), limit) && append_u64(
        &mut output,
        nested.len() as u64,
        limit,
    ) && crate::execution_codec::append_encoded_bytes(&mut output, nested.as_slice(), limit)
        && append_stream(&mut output, value.stdout(), limit) && append_stream(
        &mut output,
        value.stderr(),
        limit,
    ) && append_duration(&mut output, value.wall_time(), limit) && append_optional_duration(
        &mut output,
        value.cpu_time(),
        limit,
    ) && append_optional_u64(&mut output, value.peak_rss_bytes(), limit);
    if !accepted {
        return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::EncodedByteLimitExceeded,
                output.len() as u64,
            ),
        );
    }
    let resources = value.resources();
    accepted =
    append_optional_u64(&mut output, resources.process_count(), limit) && append_optional_u64(
        &mut output,
        resources.thread_count(),
        limit,
    ) && append_optional_u64(&mut output, resources.open_file_count(), limit)
        && append_optional_u64(&mut output, resources.handle_count(), limit) && append_optional_u64(
        &mut output,
        resources.read_bytes(),
        limit,
    ) && append_optional_u64(&mut output, resources.written_bytes(), limit) && append_extensions(
        &mut output,
        resources.extensions(),
        limit,
    );
    if !accepted {
        return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::EncodedByteLimitExceeded,
                output.len() as u64,
            ),
        );
    }
    accepted =
    match value.coverage() {
        None => push_encoded_byte(&mut output, 0, limit),
        Some(coverage) => push_encoded_byte(&mut output, 1, limit) && append_string(
            &mut output,
            coverage.provider().as_str(),
            limit,
        ) && append_string(&mut output, coverage.provider_version(), limit) && append_string(
            &mut output,
            coverage.target_build().as_str(),
            limit,
        ) && append_string(&mut output, coverage.feature_set_digest(), limit) && append_artifact(
            &mut output,
            coverage.artifact(),
            limit,
        ) && append_u64(&mut output, coverage.new_features(), limit) && append_u64(
            &mut output,
            coverage.total_features(),
            limit,
        ),
    };
    accepted =
    accepted && match value.state_digest() {
        None => push_encoded_byte(&mut output, 0, limit),
        Some(state) => push_encoded_byte(&mut output, 1, limit) && append_string(
            &mut output,
            state.namespace(),
            limit,
        ) && append_u32(&mut output, state.schema_version(), limit) && append_artifact(
            &mut output,
            state.artifact(),
            limit,
        ),
    };
    accepted =
    accepted && match value.schedule_trace() {
        None => push_encoded_byte(&mut output, 0, limit),
        Some(trace) => push_encoded_byte(&mut output, 1, limit) && append_string(
            &mut output,
            trace.namespace(),
            limit,
        ) && append_u32(&mut output, trace.schema_version(), limit) && append_artifact(
            &mut output,
            trace.artifact(),
            limit,
        ) && append_u64(&mut output, trace.decisions(), limit) && append_bool(
            &mut output,
            trace.complete(),
            limit,
        ),
    };
    accepted =
    accepted && match value.fault_trace() {
        None => push_encoded_byte(&mut output, 0, limit),
        Some(trace) => push_encoded_byte(&mut output, 1, limit) && append_string(
            &mut output,
            trace.namespace(),
            limit,
        ) && append_u32(&mut output, trace.schema_version(), limit) && append_artifact(
            &mut output,
            trace.artifact(),
            limit,
        ) && append_u64(&mut output, trace.reached(), limit) && append_u64(
            &mut output,
            trace.applied(),
            limit,
        ) && append_u64(&mut output, trace.skipped(), limit) && append_u64(
            &mut output,
            trace.shadowed(),
            limit,
        ) && append_u64(&mut output, trace.rejected(), limit) && append_bool(
            &mut output,
            trace.complete(),
            limit,
        ),
    };
    accepted = accepted && append_extensions(&mut output, value.extensions(), limit);
    if !accepted {
        Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::EncodedByteLimitExceeded,
                output.len() as u64,
            ),
        )
    } else {
        Ok(output)
    }
}

fn map_read_error(
    error: crate::execution_codec::RawExecutionOutcomeCodecError,
) -> RawObservationCodecError {
    let kind = match error.kind() {
        RawExecutionOutcomeCodecErrorKind::Truncated => RawObservationCodecErrorKind::Truncated,
        RawExecutionOutcomeCodecErrorKind::InvalidUtf8 => RawObservationCodecErrorKind::InvalidUtf8,
        _ => RawObservationCodecErrorKind::StringLengthLimitExceeded,
    };
    RawObservationCodecError::new(kind, error.byte_offset())
}

const MAX_OBSERVATION_ARTIFACT_ID_BYTES: u64 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DecodedMetadataUsage {
    identity_code_points: u64,
    namespace_code_points: u64,
    media_type_code_points: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DecodedExtensionLimits {
    namespace_code_points: u64,
    media_type_code_points: u64,
    payload_bytes_per_record: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExtensionSequencePolicy {
    count: u64,
    count_error: RawObservationCodecErrorKind,
    resource_extensions: bool,
}

fn read_bounded_text(bytes: &[u8], index: &mut usize, max_bytes: u64) -> (result: Result<
    String,
    RawObservationCodecError,
>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
        max_bytes <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
{
    match read_string(
        bytes,
        index,
        max_bytes,
        RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded,
        None,
        None,
    ) {
        Ok(value) => Ok(value),
        Err(error) => Err(map_read_error(error)),
    }
}

fn read_limited_text(
    bytes: &[u8],
    index: &mut usize,
    used: &mut u64,
    limit: u64,
    location: RawObservationLocation,
    empty_kind: Option<RawObservationErrorKind>,
    limit_kind: RawObservationErrorKind,
) -> (result: Result<String, RawObservationCodecError>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
        *old(used) <= limit,
        limit <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    ensures
        *final(index) <= bytes@.len(),
        *final(used) <= limit,
{
    let length_offset = *index;
    let remaining = limit - *used;
    let value = match read_string(
        bytes,
        index,
        remaining * 4,
        RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded,
        None,
        Some(remaining),
    ) {
        Ok(value) => value,
        Err(error) => {
            if error.kind() == RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded {
                return Err(
                    RawObservationCodecError::semantic(
                        length_offset as u64,
                        limit_kind,
                        location,
                        Some(remaining),
                        None,
                        None,
                    ),
                );
            }
            return Err(map_read_error(error));
        },
    };
    let code_points = value.as_str().unicode_len() as u64;
    if code_points == 0 {
        if let Some(kind) = empty_kind {
            return Err(
                RawObservationCodecError::semantic(
                    length_offset as u64,
                    kind,
                    location,
                    None,
                    None,
                    None,
                ),
            );
        }
    }
    if code_points > remaining {
        return Err(
            RawObservationCodecError::semantic(
                length_offset as u64,
                limit_kind,
                location,
                Some(remaining),
                None,
                None,
            ),
        );
    }
    *used += code_points;
    Ok(value)
}

fn read_boolean(bytes: &[u8], index: &mut usize) -> (result: Result<bool, RawObservationCodecError>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
{
    let offset = *index;
    match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::UnknownBooleanTag,
                offset as u64,
            ),
        ),
    }
}

fn read_optional_u64(bytes: &[u8], index: &mut usize) -> (result: Result<
    Option<u64>,
    RawObservationCodecError,
>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
{
    let offset = *index;
    match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => Ok(None),
        1 => Ok(Some(read_u64(bytes, index, None).map_err(map_read_error)?)),
        _ => Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                offset as u64,
            ),
        ),
    }
}

fn read_duration(bytes: &[u8], index: &mut usize) -> (result: Result<
    RecordedDuration,
    RawObservationCodecError,
>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
{
    let seconds = read_u64(bytes, index, None).map_err(map_read_error)?;
    let nanos_offset = *index;
    let nanoseconds = read_u32(bytes, index, None).map_err(map_read_error)?;
    RecordedDuration::new(seconds, nanoseconds).map_err(
        |_error|
            {
                RawObservationCodecError::new(
                    RawObservationCodecErrorKind::InvalidDuration,
                    nanos_offset as u64,
                )
            },
    )
}

fn read_optional_duration(bytes: &[u8], index: &mut usize) -> (result: Result<
    Option<RecordedDuration>,
    RawObservationCodecError,
>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
{
    let offset = *index;
    match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => Ok(None),
        1 => Ok(Some(read_duration(bytes, index)?)),
        _ => Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                offset as u64,
            ),
        ),
    }
}

fn read_artifact(
    bytes: &[u8],
    index: &mut usize,
    location: RawObservationLocation,
    media_used: &mut u64,
    media_limit: u64,
    payload_limit: u64,
) -> (result: Result<ArtifactRef, RawObservationCodecError>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
        *old(media_used) <= media_limit,
        media_limit <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    ensures
        *final(index) <= bytes@.len(),
        *final(media_used) <= media_limit,
{
    let size_offset = *index;
    let size_bytes = read_u64(bytes, index, None).map_err(map_read_error)?;
    if size_bytes > payload_limit {
        return Err(
            RawObservationCodecError::semantic(
                size_offset as u64,
                RawObservationErrorKind::ArtifactPayloadLimitExceeded,
                location,
                None,
                None,
                None,
            ),
        );
    }
    let id_offset = *index;
    let id = ArtifactId::new(read_bounded_text(bytes, index, MAX_OBSERVATION_ARTIFACT_ID_BYTES)?);
    match parse_artifact_id(&id) {
        Ok(ContentDigest::Sha256(_)) => {},
        Err(ArtifactIdParseError::MalformedArtifactId) => {
            return Err(
                RawObservationCodecError::semantic(
                    id_offset as u64,
                    RawObservationErrorKind::MalformedArtifactId,
                    location,
                    None,
                    None,
                    None,
                ),
            );
        },
        Err(ArtifactIdParseError::UnsupportedAlgorithm) => {
            return Err(
                RawObservationCodecError::semantic(
                    id_offset as u64,
                    RawObservationErrorKind::UnsupportedArtifactAlgorithm,
                    location,
                    None,
                    None,
                    None,
                ),
            );
        },
    }
    let option_offset = *index;
    let media_type = match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => None,
        1 => Some(
            read_limited_text(
                bytes,
                index,
                media_used,
                media_limit,
                location,
                Some(RawObservationErrorKind::EmptyMediaType),
                RawObservationErrorKind::ExtensionMediaTypeLimitExceeded,
            )?,
        ),
        _ => return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                option_offset as u64,
            ),
        ),
    };
    Ok(ArtifactRef { id, size_bytes, media_type })
}

fn read_stream(
    bytes: &[u8],
    index: &mut usize,
    location: RawObservationLocation,
    media_used: &mut u64,
    media_limit: u64,
    payload_limit: u64,
) -> (result: Result<CapturedStreamRef, RawObservationCodecError>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
        *old(media_used) <= media_limit,
        media_limit <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    ensures
        *final(index) <= bytes@.len(),
        *final(media_used) <= media_limit,
{
    let artifact = read_artifact(bytes, index, location, media_used, media_limit, payload_limit)?;
    let truncated = read_boolean(bytes, index)?;
    let retained_bytes = read_u64(bytes, index, None).map_err(map_read_error)?;
    let discarded_bytes = read_u64(bytes, index, None).map_err(map_read_error)?;
    Ok(CapturedStreamRef::new(artifact, truncated, retained_bytes, discarded_bytes))
}

fn read_extension(
    bytes: &[u8],
    index: &mut usize,
    location: RawObservationLocation,
    usage: &mut DecodedMetadataUsage,
    limits: DecodedExtensionLimits,
) -> (result: Result<VersionedExtensionRef, RawObservationCodecError>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
        old(usage).namespace_code_points <= limits.namespace_code_points,
        old(usage).media_type_code_points <= limits.media_type_code_points,
        limits.namespace_code_points
            <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        limits.media_type_code_points
            <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    ensures
        *final(index) <= bytes@.len(),
        final(usage).identity_code_points == old(usage).identity_code_points,
        final(usage).namespace_code_points <= limits.namespace_code_points,
        final(usage).media_type_code_points <= limits.media_type_code_points,
{
    let namespace = read_limited_text(
        bytes,
        index,
        &mut usage.namespace_code_points,
        limits.namespace_code_points,
        location,
        Some(RawObservationErrorKind::EmptyExtensionNamespace),
        RawObservationErrorKind::ExtensionNamespaceLimitExceeded,
    )?;
    let schema_offset = *index;
    let schema_version = read_u32(bytes, index, None).map_err(map_read_error)?;
    if schema_version == 0 {
        return Err(
            RawObservationCodecError::semantic(
                schema_offset as u64,
                RawObservationErrorKind::ZeroExtensionSchemaVersion,
                location,
                None,
                None,
                None,
            ),
        );
    }
    let payload = read_artifact(
        bytes,
        index,
        location,
        &mut usage.media_type_code_points,
        limits.media_type_code_points,
        limits.payload_bytes_per_record,
    )?;
    Ok(VersionedExtensionRef::new(namespace, schema_version, payload))
}

fn read_extensions(
    bytes: &[u8],
    index: &mut usize,
    policy: ExtensionSequencePolicy,
    usage: &mut DecodedMetadataUsage,
    limits: DecodedExtensionLimits,
) -> (result: Result<Vec<VersionedExtensionRef>, RawObservationCodecError>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
        old(usage).namespace_code_points <= limits.namespace_code_points,
        old(usage).media_type_code_points <= limits.media_type_code_points,
        limits.namespace_code_points
            <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        limits.media_type_code_points
            <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    ensures
        *final(index) <= bytes@.len(),
        final(usage).identity_code_points == old(usage).identity_code_points,
        final(usage).namespace_code_points <= limits.namespace_code_points,
        final(usage).media_type_code_points <= limits.media_type_code_points,
{
    let count_offset = *index;
    let count = read_u64(bytes, index, None).map_err(map_read_error)?;
    if count > policy.count {
        return Err(
            RawObservationCodecError::indexed(
                policy.count_error,
                count_offset as u64,
                policy.count,
            ),
        );
    }
    let mut values = Vec::new();
    let mut position = 0u64;
    while position < count
        invariant
            position <= count,
            *index <= bytes@.len(),
            bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
            usage.namespace_code_points <= limits.namespace_code_points,
            usage.media_type_code_points <= limits.media_type_code_points,
            usage.identity_code_points == old(usage).identity_code_points,
            limits.namespace_code_points
                <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            limits.media_type_code_points
                <= crate::execution::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        decreases count - position,
    {
        let location = if policy.resource_extensions {
            RawObservationLocation::ResourceExtension(position)
        } else {
            RawObservationLocation::Extension(position)
        };
        values.push(read_extension(bytes, index, location, usage, limits)?);
        position += 1;
    }
    Ok(values)
}

fn effective_count_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit <= requested,
        limit <= absolute,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[expect(
    clippy::manual_map,
    reason = "Verus rejects enum constructors used as function values, so the explicit match is required"
)]
fn event_index_location(event_index: Option<u64>) -> (location: Option<
    RawExecutionOutcomeLocation,
>) {
    match event_index {
        Some(event_index) => Some(RawExecutionOutcomeLocation::Event(event_index)),
        None => None,
    }
}

fn decode_current(bytes: &[u8], index: &mut usize, limits: RawObservationLimits) -> (result: Result<
    ValidatedRawObservation,
    RawObservationCodecError,
>)
    requires
        *old(index) <= bytes@.len(),
        bytes@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
        result is Ok ==> crate::observation::raw_observation_semantics_with_limits_spec(
            result.unwrap()@,
            limits@,
        ),
{
    let identity_limit = effective_count_limit(
        limits.max_identity_code_points(),
        crate::observation::MAX_RAW_OBSERVATION_IDENTITY_CODE_POINTS,
    );
    let namespace_limit = effective_count_limit(
        limits.max_extension_namespace_code_points(),
        crate::execution::MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    );
    let media_limit = effective_count_limit(
        limits.max_extension_media_type_code_points(),
        crate::execution::MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    );
    let payload_limit = effective_count_limit(
        limits.max_extension_payload_bytes_per_record(),
        crate::execution::MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    );
    let mut usage = DecodedMetadataUsage {
        identity_code_points: 0,
        namespace_code_points: 0,
        media_type_code_points: 0,
    };
    let extension_limits = DecodedExtensionLimits {
        namespace_code_points: namespace_limit,
        media_type_code_points: media_limit,
        payload_bytes_per_record: payload_limit,
    };
    let run_id_offset = *index;
    let run_id = RunId::new(
        read_limited_text(
            bytes,
            index,
            &mut usage.identity_code_points,
            identity_limit,
            RawObservationLocation::RunId,
            Some(RawObservationErrorKind::EmptyIdentity),
            RawObservationErrorKind::IdentityLimitExceeded,
        )?,
    );
    let attempt_id_offset = *index;
    let attempt_id = RunAttemptId::new(
        read_limited_text(
            bytes,
            index,
            &mut usage.identity_code_points,
            identity_limit,
            RawObservationLocation::AttemptId,
            Some(RawObservationErrorKind::EmptyIdentity),
            RawObservationErrorKind::IdentityLimitExceeded,
        )?,
    );
    let nested_length = read_u64(bytes, index, None).map_err(map_read_error)?;
    if nested_length > (bytes.len() - *index) as u64 {
        return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::Truncated,
                bytes.len() as u64,
            ),
        );
    }
    let nested_start = *index;
    let nested_end = nested_start + nested_length as usize;
    let mut nested = Vec::new();
    let mut nested_index = nested_start;
    while nested_index < nested_end
        invariant
            nested_start <= nested_index <= nested_end,
            nested_end <= bytes.len(),
        decreases nested_end - nested_index,
    {
        nested.push(bytes[nested_index]);
        nested_index += 1;
    }
    *index = nested_end;
    let outcome_limits = limits.outcome_limits();
    let outcome = match decode_raw_execution_outcome(
        nested,
        RawExecutionOutcomeCodecLimits::new(nested_length, outcome_limits),
    ) {
        Ok(value) => value.into_inner(),
        Err(rejection) => {
            let error = rejection.error();
            let relative = error.byte_offset();
            let offset = if relative <= nested_length {
                nested_start as u64 + relative
            } else {
                nested_start as u64
            };
            let inferred_outcome_kind = match error.kind() {
                RawExecutionOutcomeCodecErrorKind::DeclaredEventLimitExceeded => {
                    Some(RawExecutionOutcomeErrorKind::EventLimitExceeded)
                },
                RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded => {
                    Some(RawExecutionOutcomeErrorKind::ExtensionNamespaceLimitExceeded)
                },
                RawExecutionOutcomeCodecErrorKind::DeclaredMediaTypeLimitExceeded => {
                    Some(RawExecutionOutcomeErrorKind::ExtensionMediaTypeLimitExceeded)
                },
                RawExecutionOutcomeCodecErrorKind::DeclaredPayloadLimitExceeded => {
                    Some(RawExecutionOutcomeErrorKind::ExtensionPayloadLimitExceeded)
                },
                _ => error.semantic_kind(),
            };
            let inferred_outcome_location = match error.semantic_location() {
                Some(location) => Some(location),
                None => event_index_location(error.event_index()),
            };
            return Err(
                RawObservationCodecError::nested(
                    offset,
                    error.kind().stable_tag(),
                    error.code_point_index(),
                    inferred_outcome_kind,
                    inferred_outcome_location,
                ),
            );
        },
    };
    let stdout_offset = *index;
    let stdout = read_stream(
        bytes,
        index,
        RawObservationLocation::Stdout,
        &mut usage.media_type_code_points,
        media_limit,
        payload_limit,
    )?;
    let stderr_offset = *index;
    let stderr = read_stream(
        bytes,
        index,
        RawObservationLocation::Stderr,
        &mut usage.media_type_code_points,
        media_limit,
        payload_limit,
    )?;
    let wall_time_offset = *index;
    let wall_time = read_duration(bytes, index)?;
    let cpu_time_offset = *index;
    let cpu_time = read_optional_duration(bytes, index)?;
    let peak_rss_bytes = read_optional_u64(bytes, index)?;
    let resources_offset = *index;
    let process_count = read_optional_u64(bytes, index)?;
    let thread_count = read_optional_u64(bytes, index)?;
    let open_file_count = read_optional_u64(bytes, index)?;
    let handle_count = read_optional_u64(bytes, index)?;
    let read_bytes = read_optional_u64(bytes, index)?;
    let written_bytes = read_optional_u64(bytes, index)?;
    let resource_limit = effective_count_limit(
        limits.max_resource_extensions(),
        MAX_RAW_OBSERVATION_RESOURCE_EXTENSIONS,
    );
    let resource_extensions = read_extensions(
        bytes,
        index,
        ExtensionSequencePolicy {
            count: resource_limit,
            count_error: RawObservationCodecErrorKind::DeclaredResourceExtensionLimitExceeded,
            resource_extensions: true,
        },
        &mut usage,
        extension_limits,
    )?;
    let resources = ResourceSnapshot::new(
        process_count,
        thread_count,
        open_file_count,
        handle_count,
        read_bytes,
        written_bytes,
        resource_extensions,
    );
    let coverage_offset = *index;
    let coverage = match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => None,
        1 => Some(
            CoverageRef::new(
                CoverageProviderId::new(
                    read_limited_text(
                        bytes,
                        index,
                        &mut usage.identity_code_points,
                        identity_limit,
                        RawObservationLocation::Coverage,
                        Some(RawObservationErrorKind::EmptyCoverageProvider),
                        RawObservationErrorKind::IdentityLimitExceeded,
                    )?,
                ),
                read_limited_text(
                    bytes,
                    index,
                    &mut usage.identity_code_points,
                    identity_limit,
                    RawObservationLocation::Coverage,
                    Some(RawObservationErrorKind::EmptyCoverageProviderVersion),
                    RawObservationErrorKind::IdentityLimitExceeded,
                )?,
                TargetBuildId::new(
                    read_limited_text(
                        bytes,
                        index,
                        &mut usage.identity_code_points,
                        identity_limit,
                        RawObservationLocation::Coverage,
                        Some(RawObservationErrorKind::EmptyCoverageTargetBuild),
                        RawObservationErrorKind::IdentityLimitExceeded,
                    )?,
                ),
                read_limited_text(
                    bytes,
                    index,
                    &mut usage.identity_code_points,
                    identity_limit,
                    RawObservationLocation::Coverage,
                    Some(RawObservationErrorKind::EmptyFeatureSetDigest),
                    RawObservationErrorKind::IdentityLimitExceeded,
                )?,
                read_artifact(
                    bytes,
                    index,
                    RawObservationLocation::Coverage,
                    &mut usage.media_type_code_points,
                    media_limit,
                    payload_limit,
                )?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
            ),
        ),
        _ => return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                coverage_offset as u64,
            ),
        ),
    };
    let state_offset = *index;
    let state_digest = match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => None,
        1 => Some(
            StateDigest::new(
                read_limited_text(
                    bytes,
                    index,
                    &mut usage.identity_code_points,
                    identity_limit,
                    RawObservationLocation::StateDigest,
                    Some(RawObservationErrorKind::EmptyStateNamespace),
                    RawObservationErrorKind::IdentityLimitExceeded,
                )?,
                read_u32(bytes, index, None).map_err(map_read_error)?,
                read_artifact(
                    bytes,
                    index,
                    RawObservationLocation::StateDigest,
                    &mut usage.media_type_code_points,
                    media_limit,
                    payload_limit,
                )?,
            ),
        ),
        _ => return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                state_offset as u64,
            ),
        ),
    };
    let schedule_offset = *index;
    let schedule_trace = match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => None,
        1 => Some(
            ScheduleTrace::new(
                read_limited_text(
                    bytes,
                    index,
                    &mut usage.identity_code_points,
                    identity_limit,
                    RawObservationLocation::ScheduleTrace,
                    Some(RawObservationErrorKind::EmptyScheduleNamespace),
                    RawObservationErrorKind::IdentityLimitExceeded,
                )?,
                read_u32(bytes, index, None).map_err(map_read_error)?,
                read_artifact(
                    bytes,
                    index,
                    RawObservationLocation::ScheduleTrace,
                    &mut usage.media_type_code_points,
                    media_limit,
                    payload_limit,
                )?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_boolean(bytes, index)?,
            ),
        ),
        _ => return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                schedule_offset as u64,
            ),
        ),
    };
    let fault_offset = *index;
    let fault_trace = match read_u8(bytes, index, None).map_err(map_read_error)? {
        0 => None,
        1 => Some(
            FaultTrace::new(
                read_limited_text(
                    bytes,
                    index,
                    &mut usage.identity_code_points,
                    identity_limit,
                    RawObservationLocation::FaultTrace,
                    Some(RawObservationErrorKind::EmptyFaultNamespace),
                    RawObservationErrorKind::IdentityLimitExceeded,
                )?,
                read_u32(bytes, index, None).map_err(map_read_error)?,
                read_artifact(
                    bytes,
                    index,
                    RawObservationLocation::FaultTrace,
                    &mut usage.media_type_code_points,
                    media_limit,
                    payload_limit,
                )?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_u64(bytes, index, None).map_err(map_read_error)?,
                read_boolean(bytes, index)?,
            ),
        ),
        _ => return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::InvalidOptionTag,
                fault_offset as u64,
            ),
        ),
    };
    let extension_limit = effective_count_limit(
        limits.max_extensions(),
        MAX_RAW_OBSERVATION_EXTENSIONS,
    );
    let extensions_offset = *index;
    let extensions = read_extensions(
        bytes,
        index,
        ExtensionSequencePolicy {
            count: extension_limit,
            count_error: RawObservationCodecErrorKind::DeclaredExtensionLimitExceeded,
            resource_extensions: false,
        },
        &mut usage,
        extension_limits,
    )?;
    let observation = RawObservation::new(
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
    );
    match validate_raw_observation(observation, limits) {
        Ok(value) => Ok(value),
        Err(rejection) => {
            let error = rejection.error();
            let location = error.location();
            let semantic_offset = match location {
                RawObservationLocation::RunId => run_id_offset,
                RawObservationLocation::AttemptId => attempt_id_offset,
                RawObservationLocation::Outcome => nested_start,
                RawObservationLocation::Stdout => stdout_offset,
                RawObservationLocation::Stderr => stderr_offset,
                RawObservationLocation::WallTime => wall_time_offset,
                RawObservationLocation::CpuTime => cpu_time_offset,
                RawObservationLocation::Resources
                | RawObservationLocation::ResourceExtension(_) => resources_offset,
                RawObservationLocation::Coverage => coverage_offset,
                RawObservationLocation::StateDigest => state_offset,
                RawObservationLocation::ScheduleTrace => schedule_offset,
                RawObservationLocation::FaultTrace => fault_offset,
                RawObservationLocation::Extension(_) => extensions_offset,
            };
            Err(
                RawObservationCodecError::semantic(
                    semantic_offset as u64,
                    error.kind(),
                    location,
                    error.code_point_index(),
                    error.outcome_error_kind(),
                    error.outcome_error_location(),
                ),
            )
        },
    }
}

fn decode_bounded_observation(encoded: &[u8], limits: RawObservationLimits) -> (result: Result<
    ValidatedRawObservation,
    RawObservationCodecError,
>)
    requires
        encoded@.len() <= MAX_RAW_OBSERVATION_ENCODED_BYTES,
    ensures
        result is Ok ==> crate::observation::raw_observation_semantics_with_limits_spec(
            result.unwrap()@,
            limits@,
        ),
{
    let mut index = 0usize;
    let first = read_u8(encoded, &mut index, None).map_err(map_read_error)?;
    let second = read_u8(encoded, &mut index, None).map_err(map_read_error)?;
    let third = read_u8(encoded, &mut index, None).map_err(map_read_error)?;
    let fourth = read_u8(encoded, &mut index, None).map_err(map_read_error)?;
    if first != MAGIC_0 || second != MAGIC_1 || third != MAGIC_2 || fourth != MAGIC_3 {
        return Err(RawObservationCodecError::new(RawObservationCodecErrorKind::InvalidMagic, 0));
    }
    let version_offset = index;
    let schema_version = read_u16(encoded, &mut index, None).map_err(map_read_error)?;
    if schema_version != RAW_OBSERVATION_SCHEMA_VERSION {
        return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::UnsupportedSchemaVersion,
                version_offset as u64,
            ),
        );
    }
    let value = decode_current(encoded, &mut index, limits)?;
    if index != encoded.len() {
        return Err(
            RawObservationCodecError::new(
                RawObservationCodecErrorKind::TrailingBytes,
                index as u64,
            ),
        );
    }
    Ok(value)
}

pub fn decode_raw_observation(encoded: Vec<u8>, limits: RawObservationCodecLimits) -> (result:
    Result<ValidatedRawObservation, RawObservationCodecRejection>)
    ensures
        raw_observation_decode_contract_spec(
            encoded@,
            limits@,
            match &result {
                Ok(observation) => Ok(observation@),
                Err(rejection) => Err(rejection@),
            },
        ),
{
    let limit = effective_encoded_limit(limits.max_encoded_bytes);
    if encoded.len() as u64 > limit {
        return Err(
            RawObservationCodecRejection {
                error: RawObservationCodecError::new(
                    RawObservationCodecErrorKind::EncodedByteLimitExceeded,
                    limit,
                ),
                encoded,
            },
        );
    }
    let decoded = decode_bounded_observation(encoded.as_slice(), limits.observation_limits);
    match decoded {
        Ok(value) => Ok(value),
        Err(error) => Err(RawObservationCodecRejection { error, encoded }),
    }
}

} // verus!
