use crate::artifact::ArtifactRef;
use crate::execution::{
    canonical_raw_execution_outcome_limits, validate_raw_execution_outcome, CompletionDisposition,
    HarnessTerminationReason, LogicalProcessId, RawExecutionEvent, RawExecutionOutcome,
    RawExecutionOutcomeErrorKind, RawExecutionOutcomeLimits, RawExecutionOutcomeLimitsView,
    RawExecutionOutcomeLocation, ResetCause, ResourceKind, TerminationRecord,
    ValidatedRawExecutionOutcome, VersionedExtensionRef, MAX_RAW_EXECUTION_EVENTS,
    MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD, RAW_EXECUTION_OUTCOME_SCHEMA_VERSION,
};
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

use crate::ArtifactId;

verus! {

pub const MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES: u64 = 134_217_728;

const RAW_EXECUTION_OUTCOME_MAGIC_0: u8 = b'C';

const RAW_EXECUTION_OUTCOME_MAGIC_1: u8 = b'R';

const RAW_EXECUTION_OUTCOME_MAGIC_2: u8 = b'X';

const RAW_EXECUTION_OUTCOME_MAGIC_3: u8 = b'O';

const MAX_ENCODED_ARTIFACT_ID_BYTES: u64 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawExecutionOutcomeCodecLimits {
    max_encoded_bytes: u64,
    outcome_limits: RawExecutionOutcomeLimits,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeCodecLimitsView {
    pub max_encoded_bytes: u64,
    pub outcome_limits: RawExecutionOutcomeLimitsView,
}

impl View for RawExecutionOutcomeCodecLimits {
    type V = RawExecutionOutcomeCodecLimitsView;

    closed spec fn view(&self) -> RawExecutionOutcomeCodecLimitsView {
        RawExecutionOutcomeCodecLimitsView {
            max_encoded_bytes: self.max_encoded_bytes,
            outcome_limits: self.outcome_limits@,
        }
    }
}

impl RawExecutionOutcomeCodecLimits {
    pub fn new(max_encoded_bytes: u64, outcome_limits: RawExecutionOutcomeLimits) -> (limits: Self)
        ensures
            limits@ == (RawExecutionOutcomeCodecLimitsView {
                max_encoded_bytes,
                outcome_limits: outcome_limits@,
            }),
    {
        Self { max_encoded_bytes, outcome_limits }
    }

    pub fn max_encoded_bytes(&self) -> (value: u64)
        ensures
            value == self@.max_encoded_bytes,
    {
        self.max_encoded_bytes
    }

    pub fn outcome_limits(&self) -> (value: RawExecutionOutcomeLimits)
        ensures
            value@ == self@.outcome_limits,
    {
        self.outcome_limits
    }
}

pub fn canonical_raw_execution_outcome_codec_limits() -> (limits: RawExecutionOutcomeCodecLimits)
    ensures
        limits@.max_encoded_bytes == MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
        limits@.outcome_limits == (RawExecutionOutcomeLimitsView {
            max_events: MAX_RAW_EXECUTION_EVENTS,
            max_extension_namespace_code_points: MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            max_extension_media_type_code_points:
                MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            max_extension_payload_bytes_per_record:
                MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
        }),
{
    RawExecutionOutcomeCodecLimits::new(
        MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
        canonical_raw_execution_outcome_limits(),
    )
}

pub open spec fn effective_raw_execution_encoded_limit_spec(requested: u64) -> u64 {
    if requested < MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES {
        requested
    } else {
        MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES
    }
}

fn effective_raw_execution_encoded_limit(requested: u64) -> (limit: u64)
    ensures
        limit == effective_raw_execution_encoded_limit_spec(requested),
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
        limit <= requested,
{
    if requested < MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES {
        requested
    } else {
        MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawExecutionOutcomeCodecErrorKind {
    EncodedByteLimitExceeded,
    Truncated,
    InvalidMagic,
    UnsupportedSchemaVersion,
    UnknownCompletionTag,
    InvalidOptionTag,
    UnknownTerminationTag,
    UnknownResetCauseTag,
    UnknownHarnessTerminationReasonTag,
    UnknownEventTag,
    UnknownResourceKindTag,
    InvalidBoolean,
    InvalidUtf8,
    StringLengthLimitExceeded,
    DeclaredEventLimitExceeded,
    DeclaredNamespaceLimitExceeded,
    DeclaredMediaTypeLimitExceeded,
    DeclaredPayloadLimitExceeded,
    InvalidLogicalProcessId,
    TrailingBytes,
    SemanticValidationFailed,
}

pub open spec fn raw_execution_outcome_codec_error_kind_stable_tag_spec(
    value: RawExecutionOutcomeCodecErrorKind,
) -> u16 {
    match value {
        RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded => 1,
        RawExecutionOutcomeCodecErrorKind::Truncated => 2,
        RawExecutionOutcomeCodecErrorKind::InvalidMagic => 3,
        RawExecutionOutcomeCodecErrorKind::UnsupportedSchemaVersion => 4,
        RawExecutionOutcomeCodecErrorKind::UnknownCompletionTag => 5,
        RawExecutionOutcomeCodecErrorKind::InvalidOptionTag => 6,
        RawExecutionOutcomeCodecErrorKind::UnknownTerminationTag => 7,
        RawExecutionOutcomeCodecErrorKind::UnknownResetCauseTag => 8,
        RawExecutionOutcomeCodecErrorKind::UnknownHarnessTerminationReasonTag => 9,
        RawExecutionOutcomeCodecErrorKind::UnknownEventTag => 10,
        RawExecutionOutcomeCodecErrorKind::UnknownResourceKindTag => 11,
        RawExecutionOutcomeCodecErrorKind::InvalidBoolean => 12,
        RawExecutionOutcomeCodecErrorKind::InvalidUtf8 => 13,
        RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded => 14,
        RawExecutionOutcomeCodecErrorKind::DeclaredEventLimitExceeded => 15,
        RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded => 16,
        RawExecutionOutcomeCodecErrorKind::DeclaredMediaTypeLimitExceeded => 17,
        RawExecutionOutcomeCodecErrorKind::DeclaredPayloadLimitExceeded => 18,
        RawExecutionOutcomeCodecErrorKind::InvalidLogicalProcessId => 19,
        RawExecutionOutcomeCodecErrorKind::TrailingBytes => 20,
        RawExecutionOutcomeCodecErrorKind::SemanticValidationFailed => 21,
    }
}

impl RawExecutionOutcomeCodecErrorKind {
    pub fn stable_tag(self) -> (tag: u16)
        ensures
            tag == raw_execution_outcome_codec_error_kind_stable_tag_spec(self),
    {
        match self {
            RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded => 1,
            RawExecutionOutcomeCodecErrorKind::Truncated => 2,
            RawExecutionOutcomeCodecErrorKind::InvalidMagic => 3,
            RawExecutionOutcomeCodecErrorKind::UnsupportedSchemaVersion => 4,
            RawExecutionOutcomeCodecErrorKind::UnknownCompletionTag => 5,
            RawExecutionOutcomeCodecErrorKind::InvalidOptionTag => 6,
            RawExecutionOutcomeCodecErrorKind::UnknownTerminationTag => 7,
            RawExecutionOutcomeCodecErrorKind::UnknownResetCauseTag => 8,
            RawExecutionOutcomeCodecErrorKind::UnknownHarnessTerminationReasonTag => 9,
            RawExecutionOutcomeCodecErrorKind::UnknownEventTag => 10,
            RawExecutionOutcomeCodecErrorKind::UnknownResourceKindTag => 11,
            RawExecutionOutcomeCodecErrorKind::InvalidBoolean => 12,
            RawExecutionOutcomeCodecErrorKind::InvalidUtf8 => 13,
            RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded => 14,
            RawExecutionOutcomeCodecErrorKind::DeclaredEventLimitExceeded => 15,
            RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded => 16,
            RawExecutionOutcomeCodecErrorKind::DeclaredMediaTypeLimitExceeded => 17,
            RawExecutionOutcomeCodecErrorKind::DeclaredPayloadLimitExceeded => 18,
            RawExecutionOutcomeCodecErrorKind::InvalidLogicalProcessId => 19,
            RawExecutionOutcomeCodecErrorKind::TrailingBytes => 20,
            RawExecutionOutcomeCodecErrorKind::SemanticValidationFailed => 21,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawExecutionOutcomeCodecError {
    kind: RawExecutionOutcomeCodecErrorKind,
    byte_offset: u64,
    event_index: Option<u64>,
    code_point_index: Option<u64>,
    semantic_kind: Option<RawExecutionOutcomeErrorKind>,
    semantic_location: Option<RawExecutionOutcomeLocation>,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeCodecErrorView {
    pub kind: RawExecutionOutcomeCodecErrorKind,
    pub byte_offset: u64,
    pub event_index: Option<u64>,
    pub code_point_index: Option<u64>,
    pub semantic_kind: Option<RawExecutionOutcomeErrorKind>,
    pub semantic_location: Option<RawExecutionOutcomeLocation>,
}

impl View for RawExecutionOutcomeCodecError {
    type V = RawExecutionOutcomeCodecErrorView;

    closed spec fn view(&self) -> RawExecutionOutcomeCodecErrorView {
        RawExecutionOutcomeCodecErrorView {
            kind: self.kind,
            byte_offset: self.byte_offset,
            event_index: self.event_index,
            code_point_index: self.code_point_index,
            semantic_kind: self.semantic_kind,
            semantic_location: self.semantic_location,
        }
    }
}

impl RawExecutionOutcomeCodecError {
    fn new(
        kind: RawExecutionOutcomeCodecErrorKind,
        byte_offset: u64,
        event_index: Option<u64>,
    ) -> (error: Self)
        ensures
            error@ == (RawExecutionOutcomeCodecErrorView {
                kind,
                byte_offset,
                event_index,
                code_point_index: None,
                semantic_kind: None,
                semantic_location: None,
            }),
    {
        Self {
            kind,
            byte_offset,
            event_index,
            code_point_index: None,
            semantic_kind: None,
            semantic_location: None,
        }
    }

    fn semantic(
        kind: RawExecutionOutcomeErrorKind,
        location: RawExecutionOutcomeLocation,
        code_point_index: Option<u64>,
        byte_offset: u64,
    ) -> (error: Self)
        ensures
            error@.kind == RawExecutionOutcomeCodecErrorKind::SemanticValidationFailed,
            error@.byte_offset == byte_offset,
            error@.semantic_kind == Some(kind),
            error@.semantic_location == Some(location),
            error@.code_point_index == code_point_index,
    {
        Self {
            kind: RawExecutionOutcomeCodecErrorKind::SemanticValidationFailed,
            byte_offset,
            event_index: match location {
                RawExecutionOutcomeLocation::Event(index) => Some(index),
                RawExecutionOutcomeLocation::Termination => None,
            },
            code_point_index,
            semantic_kind: Some(kind),
            semantic_location: Some(location),
        }
    }

    fn metadata_limit(
        kind: RawExecutionOutcomeCodecErrorKind,
        byte_offset: u64,
        event_index: Option<u64>,
        code_point_index: u64,
    ) -> (error: Self)
        ensures
            error@.kind == kind,
            error@.byte_offset == byte_offset,
            error@.event_index == event_index,
            error@.code_point_index == Some(code_point_index),
    {
        Self {
            kind,
            byte_offset,
            event_index,
            code_point_index: Some(code_point_index),
            semantic_kind: None,
            semantic_location: None,
        }
    }

    pub fn kind(&self) -> (value: RawExecutionOutcomeCodecErrorKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn byte_offset(&self) -> (value: u64)
        ensures
            value == self@.byte_offset,
    {
        self.byte_offset
    }

    pub fn event_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.event_index,
    {
        self.event_index
    }

    pub fn code_point_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.code_point_index,
    {
        self.code_point_index
    }

    pub fn semantic_kind(&self) -> (value: Option<RawExecutionOutcomeErrorKind>)
        ensures
            value == self@.semantic_kind,
    {
        self.semantic_kind
    }

    pub fn semantic_location(&self) -> (value: Option<RawExecutionOutcomeLocation>)
        ensures
            value == self@.semantic_location,
    {
        self.semantic_location
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawExecutionOutcomeCodecRejection {
    error: RawExecutionOutcomeCodecError,
    encoded: Vec<u8>,
}

#[verifier::ext_equal]
pub struct RawExecutionOutcomeCodecRejectionView {
    pub error: RawExecutionOutcomeCodecErrorView,
    pub encoded: Seq<u8>,
}

impl View for RawExecutionOutcomeCodecRejection {
    type V = RawExecutionOutcomeCodecRejectionView;

    closed spec fn view(&self) -> RawExecutionOutcomeCodecRejectionView {
        RawExecutionOutcomeCodecRejectionView { error: self.error@, encoded: self.encoded@ }
    }
}

impl RawExecutionOutcomeCodecRejection {
    pub fn error(&self) -> (value: &RawExecutionOutcomeCodecError) {
        &self.error
    }

    pub fn encoded(&self) -> (value: &[u8]) {
        self.encoded.as_slice()
    }

    pub fn into_encoded(self) -> (value: Vec<u8>) {
        self.encoded
    }
}

pub(crate) fn push_encoded_byte(output: &mut Vec<u8>, byte: u8, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        accepted ==> final(output)@ == old(output)@.push(byte),
        !accepted ==> final(output)@ == old(output)@,
        final(output)@.len() <= limit,
{
    if output.len() as u64 >= limit {
        false
    } else {
        output.push(byte);
        true
    }
}

pub(crate) fn append_encoded_bytes(output: &mut Vec<u8>, bytes: &[u8], limit: u64) -> (accepted:
    bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
        accepted ==> final(output)@ == old(output)@ + bytes@,
{
    let ghost before = output@;
    let mut index = 0usize;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
            limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
            output@.len() <= limit,
            output@ == before + bytes@.subrange(0, index as int),
        decreases bytes.len() - index,
    {
        if !push_encoded_byte(output, bytes[index], limit) {
            return false;
        }
        index += 1;
    }
    true
}

pub(crate) fn append_u16(output: &mut Vec<u8>, value: u16, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    push_encoded_byte(output, (value >> 8) as u8, limit) && push_encoded_byte(
        output,
        value as u8,
        limit,
    )
}

pub(crate) fn append_u32(output: &mut Vec<u8>, value: u32, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    push_encoded_byte(output, (value >> 24) as u8, limit) && push_encoded_byte(
        output,
        (value >> 16) as u8,
        limit,
    ) && push_encoded_byte(output, (value >> 8) as u8, limit) && push_encoded_byte(
        output,
        value as u8,
        limit,
    )
}

pub(crate) fn append_u64(output: &mut Vec<u8>, value: u64, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    push_encoded_byte(output, (value >> 56) as u8, limit) && push_encoded_byte(
        output,
        (value >> 48) as u8,
        limit,
    ) && push_encoded_byte(output, (value >> 40) as u8, limit) && push_encoded_byte(
        output,
        (value >> 32) as u8,
        limit,
    ) && push_encoded_byte(output, (value >> 24) as u8, limit) && push_encoded_byte(
        output,
        (value >> 16) as u8,
        limit,
    ) && push_encoded_byte(output, (value >> 8) as u8, limit) && push_encoded_byte(
        output,
        value as u8,
        limit,
    )
}

pub(crate) fn append_string(output: &mut Vec<u8>, value: &str, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    let bytes = value.as_bytes();
    append_u64(output, bytes.len() as u64, limit) && append_encoded_bytes(output, bytes, limit)
}

pub(crate) fn encode_extension(
    output: &mut Vec<u8>,
    extension: &VersionedExtensionRef,
    limit: u64,
) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    if !append_string(output, extension.namespace(), limit) || !append_u32(
        output,
        extension.schema_version(),
        limit,
    ) || !append_u64(output, extension.payload().size_bytes, limit) || !append_string(
        output,
        extension.payload().id.as_str(),
        limit,
    ) {
        return false;
    }
    match &extension.payload().media_type {
        None => push_encoded_byte(output, 0, limit),
        Some(media_type) => {
            push_encoded_byte(output, 1, limit) && append_string(output, media_type.as_str(), limit)
        },
    }
}

fn encode_termination(
    output: &mut Vec<u8>,
    termination: &TerminationRecord,
    limit: u64,
) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    if !append_u16(output, termination.stable_tag(), limit) {
        return false;
    }
    match termination {
        TerminationRecord::ExitCode { code } => append_u64(output, *code as u64, limit),
        TerminationRecord::UnixSignal { signal, core_dumped } => {
            append_u32(output, *signal as u32, limit) && push_encoded_byte(
                output,
                if *core_dumped {
                    1
                } else {
                    0
                },
                limit,
            )
        },
        TerminationRecord::WindowsException { status } => append_u32(output, *status, limit),
        TerminationRecord::EmbeddedReset { cause } => append_u16(output, cause.stable_tag(), limit),
        TerminationRecord::HarnessTerminated { reason } => {
            append_u16(output, reason.stable_tag(), limit)
        },
        TerminationRecord::PlatformSpecific(extension) => {
            encode_extension(output, extension, limit)
        },
    }
}

fn encode_event(output: &mut Vec<u8>, event: &RawExecutionEvent, limit: u64) -> (accepted: bool)
    requires
        old(output)@.len() <= limit,
        limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        final(output)@.len() <= limit,
{
    if !append_u16(output, event.stable_tag(), limit) {
        return false;
    }
    match event {
        RawExecutionEvent::TimeoutThresholdReached
        | RawExecutionEvent::DeadlockSuspected
        | RawExecutionEvent::LivelockSuspected
        | RawExecutionEvent::WatchdogTriggered => true,
        RawExecutionEvent::ResourceThresholdReached { resource } => {
            append_u16(output, resource.stable_tag(), limit)
        },
        RawExecutionEvent::ProcessCreated { logical_process }
        | RawExecutionEvent::ProcessExited { logical_process } => {
            append_u64(output, logical_process.value(), limit)
        },
        RawExecutionEvent::PlatformSpecific(extension) => encode_extension(
            output,
            extension,
            limit,
        ),
    }
}

pub(crate) fn encode_raw_execution_outcome_value(
    value: &RawExecutionOutcome,
    requested_limit: u64,
) -> (result: Result<Vec<u8>, RawExecutionOutcomeCodecError>)
    ensures
        match &result {
            Ok(encoded) => encoded@.len() <= effective_raw_execution_encoded_limit_spec(
                requested_limit,
            ),
            Err(error) => error@.kind
                == RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
        },
{
    let limit = effective_raw_execution_encoded_limit(requested_limit);
    let mut output = Vec::new();
    let accepted = push_encoded_byte(&mut output, RAW_EXECUTION_OUTCOME_MAGIC_0, limit)
        && push_encoded_byte(&mut output, RAW_EXECUTION_OUTCOME_MAGIC_1, limit)
        && push_encoded_byte(&mut output, RAW_EXECUTION_OUTCOME_MAGIC_2, limit)
        && push_encoded_byte(&mut output, RAW_EXECUTION_OUTCOME_MAGIC_3, limit) && append_u16(
        &mut output,
        RAW_EXECUTION_OUTCOME_SCHEMA_VERSION,
        limit,
    ) && append_u16(&mut output, value.completion().stable_tag(), limit) && append_u64(
        &mut output,
        value.events().len() as u64,
        limit,
    ) && match value.termination() {
        None => push_encoded_byte(&mut output, 0, limit),
        Some(termination) => {
            push_encoded_byte(&mut output, 1, limit) && encode_termination(
                &mut output,
                termination,
                limit,
            )
        },
    };
    if !accepted {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
                output.len() as u64,
                None,
            ),
        );
    }
    let events = value.events();
    let mut index = 0usize;
    while index < events.len()
        invariant
            index <= events.len(),
            limit <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
            output@.len() <= limit,
        decreases events.len() - index,
    {
        if !encode_event(&mut output, &events[index], limit) {
            return Err(
                RawExecutionOutcomeCodecError::new(
                    RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
                    output.len() as u64,
                    Some(index as u64),
                ),
            );
        }
        index += 1;
    }
    Ok(output)
}

pub fn encode_raw_execution_outcome(
    outcome: &ValidatedRawExecutionOutcome,
    requested_limit: u64,
) -> (result: Result<Vec<u8>, RawExecutionOutcomeCodecError>)
    ensures
        match &result {
            Ok(encoded) => encoded@.len() <= effective_raw_execution_encoded_limit_spec(
                requested_limit,
            ),
            Err(error) => error@.kind
                == RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
        },
{
    encode_raw_execution_outcome_value(outcome.outcome(), requested_limit)
}

fn truncated(bytes: &[u8], event_index: Option<u64>) -> (error: RawExecutionOutcomeCodecError)
    ensures
        error@.kind == RawExecutionOutcomeCodecErrorKind::Truncated,
        error@.byte_offset == bytes@.len(),
        error@.event_index == event_index,
{
    RawExecutionOutcomeCodecError::new(
        RawExecutionOutcomeCodecErrorKind::Truncated,
        bytes.len() as u64,
        event_index,
    )
}

pub(crate) fn read_u8(bytes: &[u8], index: &mut usize, event_index: Option<u64>) -> (result: Result<
    u8,
    RawExecutionOutcomeCodecError,
>)
    requires
        *old(index) <= bytes@.len(),
    ensures
        *final(index) <= bytes@.len(),
        result is Err ==> *final(index) == *old(index),
{
    if *index >= bytes.len() {
        Err(truncated(bytes, event_index))
    } else {
        let value = bytes[*index];
        *index += 1;
        Ok(value)
    }
}

pub(crate) fn read_u16(bytes: &[u8], index: &mut usize, event_index: Option<u64>) -> (result:
    Result<u16, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
    ensures
        *final(index) <= bytes@.len(),
        result is Err ==> *final(index) == *old(index),
{
    let start = *index;
    if bytes.len() - start < 2 {
        return Err(truncated(bytes, event_index));
    }
    let value = ((bytes[start] as u16) << 8) | bytes[start + 1] as u16;
    *index = start + 2;
    Ok(value)
}

pub(crate) fn read_u32(bytes: &[u8], index: &mut usize, event_index: Option<u64>) -> (result:
    Result<u32, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
    ensures
        *final(index) <= bytes@.len(),
        result is Err ==> *final(index) == *old(index),
{
    let start = *index;
    if bytes.len() - start < 4 {
        return Err(truncated(bytes, event_index));
    }
    let value = ((bytes[start] as u32) << 24) | ((bytes[start + 1] as u32) << 16) | ((bytes[start
        + 2] as u32) << 8) | bytes[start + 3] as u32;
    *index = start + 4;
    Ok(value)
}

pub(crate) fn read_u64(bytes: &[u8], index: &mut usize, event_index: Option<u64>) -> (result:
    Result<u64, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
    ensures
        *final(index) <= bytes@.len(),
        result is Err ==> *final(index) == *old(index),
{
    let start = *index;
    if bytes.len() - start < 8 {
        return Err(truncated(bytes, event_index));
    }
    let value = ((bytes[start] as u64) << 56) | ((bytes[start + 1] as u64) << 48) | ((bytes[start
        + 2] as u64) << 40) | ((bytes[start + 3] as u64) << 32) | ((bytes[start + 4] as u64) << 24)
        | ((bytes[start + 5] as u64) << 16) | ((bytes[start + 6] as u64) << 8) | bytes[start
        + 7] as u64;
    *index = start + 8;
    Ok(value)
}

// CRUCIBLE-TCB: CORE-HOST-UTF8-001
#[verifier::external_body]
fn host_decode_utf8_range(bytes: &[u8], start: usize, end: usize) -> (result: Result<String, u64>)
    requires
        start <= end <= bytes@.len(),
    ensures
        match &result {
            Ok(text) => vstd::utf8::encode_utf8(text@) == bytes@.subrange(start as int, end as int),
            Err(offset) => *offset <= (end - start) as u64,
        },
{
    match std::str::from_utf8(&bytes[start..end]) {
        Ok(text) => Ok(String::from(text)),
        Err(error) => Err(error.valid_up_to() as u64),
    }
}

pub(crate) fn read_string(
    bytes: &[u8],
    index: &mut usize,
    max_bytes: u64,
    length_error: RawExecutionOutcomeCodecErrorKind,
    event_index: Option<u64>,
    code_point_limit: Option<u64>,
) -> (result: Result<String, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
        max_bytes <= MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
    ensures
        *final(index) <= bytes@.len(),
{
    let length_offset = *index;
    let length = read_u64(bytes, index, event_index)?;
    if length > max_bytes {
        return match code_point_limit {
            Some(first_excluded) => Err(
                RawExecutionOutcomeCodecError::metadata_limit(
                    length_error,
                    length_offset as u64,
                    event_index,
                    first_excluded,
                ),
            ),
            None => Err(
                RawExecutionOutcomeCodecError::new(length_error, length_offset as u64, event_index),
            ),
        };
    }
    let start = *index;
    if length > (bytes.len() - start) as u64 {
        return Err(truncated(bytes, event_index));
    }
    let end = start + length as usize;
    match host_decode_utf8_range(bytes, start, end) {
        Ok(text) => {
            *index = end;
            Ok(text)
        },
        Err(relative_offset) => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::InvalidUtf8,
                start as u64 + relative_offset,
                event_index,
            ),
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DecodedMetadataUsage {
    namespace_code_points: u64,
    media_type_code_points: u64,
}

fn decode_extension(
    bytes: &[u8],
    index: &mut usize,
    event_index: Option<u64>,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    usage: &mut DecodedMetadataUsage,
) -> (result: Result<VersionedExtensionRef, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
        old(usage).namespace_code_points <= namespace_limit,
        old(usage).media_type_code_points <= media_type_limit,
        namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ensures
        *final(index) <= bytes@.len(),
        final(usage).namespace_code_points <= namespace_limit,
        final(usage).media_type_code_points <= media_type_limit,
{
    let namespace_length_offset = *index;
    let namespace_remaining = namespace_limit - usage.namespace_code_points;
    let namespace = read_string(
        bytes,
        index,
        namespace_remaining * 4,
        RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded,
        event_index,
        Some(namespace_remaining),
    )?;
    let namespace_code_points = namespace.as_str().unicode_len() as u64;
    if namespace_code_points > namespace_remaining {
        return Err(
            RawExecutionOutcomeCodecError::metadata_limit(
                RawExecutionOutcomeCodecErrorKind::DeclaredNamespaceLimitExceeded,
                namespace_length_offset as u64,
                event_index,
                namespace_remaining,
            ),
        );
    }
    usage.namespace_code_points += namespace_code_points;
    let schema_version = read_u32(bytes, index, event_index)?;
    let payload_offset = *index;
    let size_bytes = read_u64(bytes, index, event_index)?;
    if size_bytes > payload_limit {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::DeclaredPayloadLimitExceeded,
                payload_offset as u64,
                event_index,
            ),
        );
    }
    let artifact_id = read_string(
        bytes,
        index,
        MAX_ENCODED_ARTIFACT_ID_BYTES,
        RawExecutionOutcomeCodecErrorKind::StringLengthLimitExceeded,
        event_index,
        None,
    )?;
    let option_offset = *index;
    let media_option = read_u8(bytes, index, event_index)?;
    let media_type = if media_option == 0 {
        None
    } else if media_option == 1 {
        let media_length_offset = *index;
        let media_remaining = media_type_limit - usage.media_type_code_points;
        let media = read_string(
            bytes,
            index,
            media_remaining * 4,
            RawExecutionOutcomeCodecErrorKind::DeclaredMediaTypeLimitExceeded,
            event_index,
            Some(media_remaining),
        )?;
        let media_code_points = media.as_str().unicode_len() as u64;
        if media_code_points > media_remaining {
            return Err(
                RawExecutionOutcomeCodecError::metadata_limit(
                    RawExecutionOutcomeCodecErrorKind::DeclaredMediaTypeLimitExceeded,
                    media_length_offset as u64,
                    event_index,
                    media_remaining,
                ),
            );
        }
        usage.media_type_code_points += media_code_points;
        Some(media)
    } else {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::InvalidOptionTag,
                option_offset as u64,
                event_index,
            ),
        );
    };
    Ok(
        VersionedExtensionRef::new(
            namespace,
            schema_version,
            ArtifactRef { id: ArtifactId::new(artifact_id), size_bytes, media_type },
        ),
    )
}

fn decode_completion(tag: u16, offset: u64) -> (result: Result<
    CompletionDisposition,
    RawExecutionOutcomeCodecError,
>) {
    match tag {
        1 => Ok(CompletionDisposition::Completed),
        2 => Ok(CompletionDisposition::Cancelled),
        3 => Ok(CompletionDisposition::Incomplete),
        _ => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnknownCompletionTag,
                offset,
                None,
            ),
        ),
    }
}

fn decode_reset_cause(tag: u16, offset: u64) -> (result: Result<
    ResetCause,
    RawExecutionOutcomeCodecError,
>) {
    match tag {
        1 => Ok(ResetCause::PowerOn),
        2 => Ok(ResetCause::Watchdog),
        3 => Ok(ResetCause::Software),
        4 => Ok(ResetCause::Brownout),
        5 => Ok(ResetCause::External),
        6 => Ok(ResetCause::Unknown),
        _ => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnknownResetCauseTag,
                offset,
                None,
            ),
        ),
    }
}

fn decode_harness_reason(tag: u16, offset: u64) -> (result: Result<
    HarnessTerminationReason,
    RawExecutionOutcomeCodecError,
>) {
    match tag {
        1 => Ok(HarnessTerminationReason::Timeout),
        2 => Ok(HarnessTerminationReason::Cancellation),
        3 => Ok(HarnessTerminationReason::CpuTimeLimit),
        4 => Ok(HarnessTerminationReason::MemoryLimit),
        5 => Ok(HarnessTerminationReason::ProcessCountLimit),
        6 => Ok(HarnessTerminationReason::FileSizeLimit),
        7 => Ok(HarnessTerminationReason::OutputLimit),
        8 => Ok(HarnessTerminationReason::CleanupFailure),
        _ => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnknownHarnessTerminationReasonTag,
                offset,
                None,
            ),
        ),
    }
}

fn decode_resource_kind(tag: u16, offset: u64, event_index: u64) -> (result: Result<
    ResourceKind,
    RawExecutionOutcomeCodecError,
>) {
    match tag {
        1 => Ok(ResourceKind::WallTime),
        2 => Ok(ResourceKind::CpuTime),
        3 => Ok(ResourceKind::Memory),
        4 => Ok(ResourceKind::ProcessCount),
        5 => Ok(ResourceKind::FileSize),
        6 => Ok(ResourceKind::StandardOutput),
        7 => Ok(ResourceKind::StandardError),
        _ => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnknownResourceKindTag,
                offset,
                Some(event_index),
            ),
        ),
    }
}

fn decode_termination(
    bytes: &[u8],
    index: &mut usize,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    usage: &mut DecodedMetadataUsage,
) -> (result: Result<TerminationRecord, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
        old(usage).namespace_code_points <= namespace_limit,
        old(usage).media_type_code_points <= media_type_limit,
        namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ensures
        *final(index) <= bytes@.len(),
        final(usage).namespace_code_points <= namespace_limit,
        final(usage).media_type_code_points <= media_type_limit,
{
    let tag_offset = *index;
    let tag = read_u16(bytes, index, None)?;
    match tag {
        1 => match read_u64(bytes, index, None) {
            Ok(value) => Ok(TerminationRecord::ExitCode { code: value as i64 }),
            Err(error) => Err(error),
        },
        2 => {
            let signal = read_u32(bytes, index, None)? as i32;
            let bool_offset = *index;
            let core_dumped = match read_u8(bytes, index, None) {
                Ok(0) => false,
                Ok(1) => true,
                Ok(_) => return Err(
                    RawExecutionOutcomeCodecError::new(
                        RawExecutionOutcomeCodecErrorKind::InvalidBoolean,
                        bool_offset as u64,
                        None,
                    ),
                ),
                Err(error) => return Err(error),
            };
            Ok(TerminationRecord::UnixSignal { signal, core_dumped })
        },
        3 => match read_u32(bytes, index, None) {
            Ok(status) => Ok(TerminationRecord::WindowsException { status }),
            Err(error) => Err(error),
        },
        4 => {
            let cause_offset = *index;
            let cause_tag = read_u16(bytes, index, None)?;
            match decode_reset_cause(cause_tag, cause_offset as u64) {
                Ok(cause) => Ok(TerminationRecord::EmbeddedReset { cause }),
                Err(error) => Err(error),
            }
        },
        5 => {
            let reason_offset = *index;
            let reason_tag = read_u16(bytes, index, None)?;
            match decode_harness_reason(reason_tag, reason_offset as u64) {
                Ok(reason) => Ok(TerminationRecord::HarnessTerminated { reason }),
                Err(error) => Err(error),
            }
        },
        6 => match decode_extension(
            bytes,
            index,
            None,
            namespace_limit,
            media_type_limit,
            payload_limit,
            usage,
        ) {
            Ok(extension) => Ok(TerminationRecord::PlatformSpecific(extension)),
            Err(error) => Err(error),
        },
        _ => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnknownTerminationTag,
                tag_offset as u64,
                None,
            ),
        ),
    }
}

fn decode_event(
    bytes: &[u8],
    index: &mut usize,
    event_index: u64,
    namespace_limit: u64,
    media_type_limit: u64,
    payload_limit: u64,
    usage: &mut DecodedMetadataUsage,
) -> (result: Result<RawExecutionEvent, RawExecutionOutcomeCodecError>)
    requires
        *old(index) <= bytes@.len(),
        old(usage).namespace_code_points <= namespace_limit,
        old(usage).media_type_code_points <= media_type_limit,
        namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
        media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
        payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
    ensures
        *final(index) <= bytes@.len(),
        final(usage).namespace_code_points <= namespace_limit,
        final(usage).media_type_code_points <= media_type_limit,
{
    let location = Some(event_index);
    let tag_offset = *index;
    let tag = read_u16(bytes, index, location)?;
    match tag {
        1 => Ok(RawExecutionEvent::TimeoutThresholdReached),
        2 => {
            let resource_offset = *index;
            let resource_tag = read_u16(bytes, index, location)?;
            match decode_resource_kind(resource_tag, resource_offset as u64, event_index) {
                Ok(resource) => Ok(RawExecutionEvent::ResourceThresholdReached { resource }),
                Err(error) => Err(error),
            }
        },
        3 => Ok(RawExecutionEvent::DeadlockSuspected),
        4 => Ok(RawExecutionEvent::LivelockSuspected),
        5 => Ok(RawExecutionEvent::WatchdogTriggered),
        6 | 7 => {
            let id_offset = *index;
            let value = read_u64(bytes, index, location)?;
            let logical_process = match LogicalProcessId::new(value) {
                Ok(id) => id,
                Err(_) => return Err(
                    RawExecutionOutcomeCodecError::new(
                        RawExecutionOutcomeCodecErrorKind::InvalidLogicalProcessId,
                        id_offset as u64,
                        location,
                    ),
                ),
            };
            if tag == 6 {
                Ok(RawExecutionEvent::ProcessCreated { logical_process })
            } else {
                Ok(RawExecutionEvent::ProcessExited { logical_process })
            }
        },
        8 => match decode_extension(
            bytes,
            index,
            location,
            namespace_limit,
            media_type_limit,
            payload_limit,
            usage,
        ) {
            Ok(extension) => Ok(RawExecutionEvent::PlatformSpecific(extension)),
            Err(error) => Err(error),
        },
        _ => Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnknownEventTag,
                tag_offset as u64,
                location,
            ),
        ),
    }
}

fn decode_raw_execution_outcome_body(
    bytes: &[u8],
    limits: RawExecutionOutcomeCodecLimits,
) -> (result: Result<ValidatedRawExecutionOutcome, RawExecutionOutcomeCodecError>)
    ensures
        result is Ok ==> crate::execution::raw_execution_outcome_semantics_spec(result.unwrap()@),
{
    let event_limit = if limits.outcome_limits.max_events() < MAX_RAW_EXECUTION_EVENTS {
        limits.outcome_limits.max_events()
    } else {
        MAX_RAW_EXECUTION_EVENTS
    };
    let namespace_limit = if limits.outcome_limits.max_extension_namespace_code_points()
        < MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS {
        limits.outcome_limits.max_extension_namespace_code_points()
    } else {
        MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS
    };
    let media_type_limit = if limits.outcome_limits.max_extension_media_type_code_points()
        < MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS {
        limits.outcome_limits.max_extension_media_type_code_points()
    } else {
        MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS
    };
    let payload_limit = if limits.outcome_limits.max_extension_payload_bytes_per_record()
        < MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD {
        limits.outcome_limits.max_extension_payload_bytes_per_record()
    } else {
        MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD
    };
    let mut index = 0usize;
    let magic_offset = index;
    let magic_0 = read_u8(bytes, &mut index, None)?;
    let magic_1 = read_u8(bytes, &mut index, None)?;
    let magic_2 = read_u8(bytes, &mut index, None)?;
    let magic_3 = read_u8(bytes, &mut index, None)?;
    if magic_0 != RAW_EXECUTION_OUTCOME_MAGIC_0 || magic_1 != RAW_EXECUTION_OUTCOME_MAGIC_1
        || magic_2 != RAW_EXECUTION_OUTCOME_MAGIC_2 || magic_3 != RAW_EXECUTION_OUTCOME_MAGIC_3 {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::InvalidMagic,
                magic_offset as u64,
                None,
            ),
        );
    }
    let schema_offset = index;
    let schema_version = read_u16(bytes, &mut index, None)?;
    if schema_version != RAW_EXECUTION_OUTCOME_SCHEMA_VERSION {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::UnsupportedSchemaVersion,
                schema_offset as u64,
                None,
            ),
        );
    }
    let completion_offset = index;
    let completion_tag = read_u16(bytes, &mut index, None)?;
    let completion = decode_completion(completion_tag, completion_offset as u64)?;
    let count_offset = index;
    let event_count = read_u64(bytes, &mut index, None)?;
    if event_count > event_limit {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::DeclaredEventLimitExceeded,
                count_offset as u64,
                Some(event_limit),
            ),
        );
    }
    let mut usage = DecodedMetadataUsage { namespace_code_points: 0, media_type_code_points: 0 };
    let option_offset = index;
    let termination_option = read_u8(bytes, &mut index, None)?;
    let termination = if termination_option == 0 {
        None
    } else if termination_option == 1 {
        Some(
            decode_termination(
                bytes,
                &mut index,
                namespace_limit,
                media_type_limit,
                payload_limit,
                &mut usage,
            )?,
        )
    } else {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::InvalidOptionTag,
                option_offset as u64,
                None,
            ),
        );
    };
    let mut events = Vec::new();
    let mut event_index = 0u64;
    while event_index < event_count
        invariant
            event_index <= event_count,
            event_count <= event_limit,
            event_limit <= MAX_RAW_EXECUTION_EVENTS,
            namespace_limit <= MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
            media_type_limit <= MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
            payload_limit <= MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD,
            index <= bytes@.len(),
            usage.namespace_code_points <= namespace_limit,
            usage.media_type_code_points <= media_type_limit,
        decreases event_count - event_index,
    {
        let event = decode_event(
            bytes,
            &mut index,
            event_index,
            namespace_limit,
            media_type_limit,
            payload_limit,
            &mut usage,
        )?;
        events.push(event);
        event_index += 1;
    }
    if index != bytes.len() {
        return Err(
            RawExecutionOutcomeCodecError::new(
                RawExecutionOutcomeCodecErrorKind::TrailingBytes,
                index as u64,
                None,
            ),
        );
    }
    let outcome = RawExecutionOutcome::new(completion, termination, events);
    let ghost outcome_view = outcome@;
    let ghost outcome_limits_view = limits.outcome_limits@;
    match validate_raw_execution_outcome(outcome, limits.outcome_limits) {
        Ok(validated) => {
            proof {
                crate::execution::lemma_successful_raw_execution_validation_has_semantics(
                    outcome_view,
                    outcome_limits_view,
                    validated@,
                );
            }
            Ok(validated)
        },
        Err(rejection) => {
            let kind = rejection.error().kind();
            let location = rejection.error().location();
            let code_point_index = rejection.error().extension_code_point_index();
            Err(
                RawExecutionOutcomeCodecError::semantic(
                    kind,
                    location,
                    code_point_index,
                    index as u64,
                ),
            )
        },
    }
}

pub fn decode_raw_execution_outcome(
    encoded: Vec<u8>,
    limits: RawExecutionOutcomeCodecLimits,
) -> (result: Result<ValidatedRawExecutionOutcome, RawExecutionOutcomeCodecRejection>)
    ensures
        match &result {
            Ok(outcome) => encoded@.len() <= effective_raw_execution_encoded_limit_spec(
                limits@.max_encoded_bytes,
            ) && crate::execution::raw_execution_outcome_semantics_spec(outcome@),
            Err(rejection) => rejection@.encoded == encoded@,
        },
{
    let encoded_limit = effective_raw_execution_encoded_limit(limits.max_encoded_bytes);
    if encoded.len() as u64 > encoded_limit {
        return Err(
            RawExecutionOutcomeCodecRejection {
                error: RawExecutionOutcomeCodecError::new(
                    RawExecutionOutcomeCodecErrorKind::EncodedByteLimitExceeded,
                    encoded_limit,
                    None,
                ),
                encoded,
            },
        );
    }
    match decode_raw_execution_outcome_body(encoded.as_slice(), limits) {
        Ok(outcome) => Ok(outcome),
        Err(error) => Err(RawExecutionOutcomeCodecRejection { error, encoded }),
    }
}

} // verus!
