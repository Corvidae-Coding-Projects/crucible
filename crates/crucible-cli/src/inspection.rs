//! Verified admission and bounded human rendering for persisted run inspection.
use crate::{stored_artifact_is_exact, StoredArtifactSnapshot, MAX_LOCAL_ARTIFACT_BYTES};
use crucible_core::{
    ArtifactRef, RawExecutionOutcomeLimits, RawObservationCodecLimits, RawObservationLimits,
    ValidatedRawObservation,
};
use vstd::prelude::*;
use vstd::string::StrSliceExecFns;

verus! {

pub const MAX_INSPECTION_TEXT_BYTES: u64 = 4_096;

pub const MAX_INSPECTION_OBSERVATION_BYTES: u64 = 1_048_576;

pub const MAX_INSPECTION_PREVIEW_BYTES: u64 = 4_096;

pub const MAX_INSPECTION_REPORT_BYTES: u64 = 131_072;

pub const MAX_INSPECTION_COLLECTION_ENTRIES: u64 = 1_024;

pub fn inspection_observation_codec_limits() -> (limits: RawObservationCodecLimits) {
    RawObservationCodecLimits::new(
        MAX_INSPECTION_OBSERVATION_BYTES,
        RawObservationLimits::new(
            RawExecutionOutcomeLimits::new(
                MAX_INSPECTION_COLLECTION_ENTRIES,
                MAX_INSPECTION_TEXT_BYTES,
                MAX_INSPECTION_TEXT_BYTES,
                MAX_LOCAL_ARTIFACT_BYTES,
            ),
            MAX_INSPECTION_TEXT_BYTES,
            MAX_INSPECTION_COLLECTION_ENTRIES,
            MAX_INSPECTION_COLLECTION_ENTRIES,
            MAX_INSPECTION_TEXT_BYTES,
            MAX_INSPECTION_TEXT_BYTES,
            MAX_LOCAL_ARTIFACT_BYTES,
        ),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionStatus {
    Reserved,
    TargetPrepared,
    Observed,
    HarnessFailure,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectionControls {
    pub timeout_ms: String,
    pub memory_bytes: String,
    pub max_processes: String,
    pub max_stream_bytes: String,
    pub network_policy: String,
    pub isolation_backend: String,
    pub output_capture_status: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectionTarget {
    pub build_id: String,
    pub target_artifact: ArtifactRef,
    pub manifest_artifact: ArtifactRef,
    pub identity_digest: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectionObservation {
    pub artifact: ArtifactRef,
    pub stdout_artifact: ArtifactRef,
    pub stderr_artifact: ArtifactRef,
    pub completion_tag: u16,
    pub termination_tag: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectionHarnessFailure {
    pub kind: String,
    pub detail_artifact: Option<ArtifactRef>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunInspectionSnapshot {
    pub run_id: String,
    pub attempt_id: String,
    pub status: InspectionStatus,
    pub configuration_source: ArtifactRef,
    pub effective_configuration: ArtifactRef,
    pub configuration_digest: String,
    pub target: Option<InspectionTarget>,
    pub capability_manifest: ArtifactRef,
    pub seed: String,
    pub controls: InspectionControls,
    pub observation: Option<InspectionObservation>,
    pub harness_failure: Option<InspectionHarnessFailure>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedRunInspection {
    snapshot: RunInspectionSnapshot,
}

impl ValidatedRunInspection {
    pub fn snapshot(&self) -> (snapshot: &RunInspectionSnapshot) {
        &self.snapshot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionValidationError {
    IdentityMismatch,
    InvalidMetadata,
    InvalidState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionArtifactError {
    TooLarge,
    Integrity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionReportError {
    EvidenceMismatch,
    ReportTooLarge,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AuthenticatedArtifactPreview {
    artifact_id: String,
    bytes: Vec<u8>,
    omitted_bytes: u64,
}

impl AuthenticatedArtifactPreview {
    pub fn artifact_id(&self) -> (value: &str) {
        self.artifact_id.as_str()
    }

    pub fn bytes(&self) -> (value: &[u8]) {
        self.bytes.as_slice()
    }

    pub fn omitted_bytes(&self) -> (value: u64) {
        self.omitted_bytes
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectionPreviews {
    pub configuration_source: AuthenticatedArtifactPreview,
    pub effective_configuration: AuthenticatedArtifactPreview,
    pub stdout: Option<AuthenticatedArtifactPreview>,
    pub stderr: Option<AuthenticatedArtifactPreview>,
}

fn bounded_text(value: &str) -> (valid: bool) {
    !value.is_empty() && value.unicode_len() as u64 <= MAX_INSPECTION_TEXT_BYTES
}

fn bounded_optional_text(value: &Option<String>) -> (valid: bool) {
    match value {
        Some(text) => bounded_text(text),
        None => true,
    }
}

fn bounded_artifact(artifact: &ArtifactRef) -> (valid: bool) {
    artifact.id.as_str().unicode_len() == 71 && artifact.size_bytes <= MAX_LOCAL_ARTIFACT_BYTES
        && bounded_optional_text(&artifact.media_type)
}

fn same_artifact(left: &ArtifactRef, right: &ArtifactRef) -> (same: bool) {
    left.id.as_str() == right.id.as_str() && left.size_bytes == right.size_bytes && left.media_type
        == right.media_type
}

pub fn validate_run_inspection(requested_run_id: &str, snapshot: RunInspectionSnapshot) -> (result:
    Result<ValidatedRunInspection, InspectionValidationError>) {
    if requested_run_id != snapshot.run_id.as_str() {
        return Err(InspectionValidationError::IdentityMismatch);
    }
    if !bounded_text(&snapshot.run_id) || !bounded_text(&snapshot.attempt_id) || !bounded_artifact(
        &snapshot.configuration_source,
    ) || !bounded_artifact(&snapshot.effective_configuration) || !bounded_text(
        &snapshot.configuration_digest,
    ) || snapshot.configuration_digest.as_str().unicode_len() != 71 || !bounded_artifact(
        &snapshot.capability_manifest,
    ) || !bounded_text(&snapshot.seed) || !bounded_text(&snapshot.controls.timeout_ms)
        || !bounded_text(&snapshot.controls.memory_bytes) || !bounded_text(
        &snapshot.controls.max_processes,
    ) || !bounded_text(&snapshot.controls.max_stream_bytes) || !bounded_text(
        &snapshot.controls.network_policy,
    ) || !bounded_text(&snapshot.controls.isolation_backend) || !bounded_text(
        &snapshot.controls.output_capture_status,
    ) {
        return Err(InspectionValidationError::InvalidMetadata);
    }
    if let Some(target) = &snapshot.target {
        if !bounded_text(&target.build_id) || !bounded_artifact(&target.target_artifact)
            || !bounded_artifact(&target.manifest_artifact) || !bounded_text(
            &target.identity_digest,
        ) {
            return Err(InspectionValidationError::InvalidMetadata);
        }
    }
    if let Some(observation) = &snapshot.observation {
        if !bounded_artifact(&observation.artifact) || observation.artifact.size_bytes
            > MAX_INSPECTION_OBSERVATION_BYTES || !bounded_artifact(&observation.stdout_artifact)
            || !bounded_artifact(&observation.stderr_artifact) || observation.completion_tag == 0
            || observation.termination_tag == 0 {
            return Err(InspectionValidationError::InvalidMetadata);
        }
    }
    if let Some(failure) = &snapshot.harness_failure {
        if !bounded_text(&failure.kind) {
            return Err(InspectionValidationError::InvalidMetadata);
        }
        if let Some(artifact) = &failure.detail_artifact {
            if !bounded_artifact(artifact) {
                return Err(InspectionValidationError::InvalidMetadata);
            }
        }
    }
    let valid_state = match snapshot.status {
        InspectionStatus::Reserved => snapshot.target.is_none() && snapshot.observation.is_none()
            && snapshot.harness_failure.is_none(),
        InspectionStatus::TargetPrepared => snapshot.target.is_some()
            && snapshot.observation.is_none() && snapshot.harness_failure.is_none(),
        InspectionStatus::Observed => snapshot.target.is_some() && snapshot.observation.is_some()
            && snapshot.harness_failure.is_none(),
        InspectionStatus::HarnessFailure => snapshot.observation.is_none()
            && snapshot.harness_failure.is_some(),
    };
    if !valid_state {
        return Err(InspectionValidationError::InvalidState);
    }
    Ok(ValidatedRunInspection { snapshot })
}

pub fn authenticate_artifact_contents(
    expected: &ArtifactRef,
    snapshot: StoredArtifactSnapshot,
    limit: u64,
) -> (result: Result<Vec<u8>, InspectionArtifactError>) {
    if expected.size_bytes > limit || limit > MAX_LOCAL_ARTIFACT_BYTES {
        return Err(InspectionArtifactError::TooLarge);
    }
    if !stored_artifact_is_exact(expected, false, &snapshot) {
        return Err(InspectionArtifactError::Integrity);
    }
    Ok(snapshot.contents)
}

pub fn authenticate_artifact_preview(
    expected: &ArtifactRef,
    snapshot: StoredArtifactSnapshot,
) -> (result: Result<AuthenticatedArtifactPreview, InspectionArtifactError>) {
    let contents = authenticate_artifact_contents(expected, snapshot, MAX_LOCAL_ARTIFACT_BYTES)?;
    let retained = if contents.len() as u64 <= MAX_INSPECTION_PREVIEW_BYTES {
        contents.len()
    } else {
        MAX_INSPECTION_PREVIEW_BYTES as usize
    };
    let mut bytes = Vec::new();
    let mut index = 0;
    while index < retained
        invariant
            index <= retained,
            retained <= contents.len(),
            bytes@ == contents@.subrange(0, index as int),
        decreases retained - index,
    {
        bytes.push(contents[index]);
        index += 1;
    }
    let omitted_bytes = contents.len() as u64 - retained as u64;
    Ok(
        AuthenticatedArtifactPreview {
            artifact_id: expected.id.as_str().to_owned(),
            bytes,
            omitted_bytes,
        },
    )
}

fn append_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> (accepted: bool) {
    if output.len() as u64 > MAX_INSPECTION_REPORT_BYTES || bytes.len() as u64
        > MAX_INSPECTION_REPORT_BYTES - output.len() as u64 {
        return false;
    }
    let mut index = 0;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
        decreases bytes.len() - index,
    {
        output.push(bytes[index]);
        index += 1;
    }
    true
}

fn append_text(output: &mut Vec<u8>, text: &str) -> (accepted: bool) {
    let bytes = text.as_bytes_vec();
    append_bytes(output, bytes.as_slice())
}

fn append_decimal(output: &mut Vec<u8>, mut value: u64) -> (accepted: bool) {
    let mut reversed = Vec::new();
    if value == 0 {
        reversed.push(b'0');
    } else {
        while value > 0
            decreases value,
        {
            reversed.push(b'0' + (value % 10) as u8);
            value /= 10;
        }
    }
    let mut index = reversed.len();
    while index > 0
        invariant
            index <= reversed.len(),
        decreases index,
    {
        if !append_bytes(output, &reversed[index - 1..index]) {
            return false;
        }
        index -= 1;
    }
    true
}

fn hexadecimal_digit(value: u8) -> (digit: u8)
    requires
        value < 16,
{
    if value < 10 {
        b'0' + value
    } else {
        b'a' - 10 + value
    }
}

fn append_hexadecimal(output: &mut Vec<u8>, bytes: &[u8]) -> (accepted: bool) {
    let mut index = 0;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
        decreases bytes.len() - index,
    {
        let byte = bytes[index];
        let encoded = [hexadecimal_digit(byte / 16), hexadecimal_digit(byte % 16)];
        if !append_bytes(output, &encoded) {
            return false;
        }
        index += 1;
    }
    true
}

fn append_field(output: &mut Vec<u8>, label: &[u8], value: &str) -> (accepted: bool) {
    append_bytes(output, label) && append_text(output, value) && append_bytes(output, b"\n")
}

fn append_number_field(output: &mut Vec<u8>, label: &[u8], value: u64) -> (accepted: bool) {
    append_bytes(output, label) && append_decimal(output, value) && append_bytes(output, b"\n")
}

fn append_preview(
    output: &mut Vec<u8>,
    hexadecimal_label: &[u8],
    omitted_label: &[u8],
    preview: &AuthenticatedArtifactPreview,
) -> (accepted: bool) {
    append_bytes(output, hexadecimal_label) && append_hexadecimal(output, preview.bytes())
        && append_bytes(output, b"\n") && append_bytes(output, omitted_label) && append_decimal(
        output,
        preview.omitted_bytes(),
    ) && append_bytes(output, b"\n")
}

fn artifact_matches_preview(
    artifact: &ArtifactRef,
    preview: &AuthenticatedArtifactPreview,
) -> (matches: bool) {
    artifact.id.as_str() == preview.artifact_id()
}

fn observation_matches_snapshot(
    snapshot: &RunInspectionSnapshot,
    decoded: &ValidatedRawObservation,
) -> (matches: bool) {
    let record = match &snapshot.observation {
        Some(record) => record,
        None => return false,
    };
    let observation = decoded.observation();
    if observation.run_id().as_str() != snapshot.run_id.as_str()
        || observation.attempt_id().as_str() != snapshot.attempt_id.as_str() || !same_artifact(
        observation.stdout().artifact(),
        &record.stdout_artifact,
    ) || !same_artifact(observation.stderr().artifact(), &record.stderr_artifact)
        || observation.outcome().completion().stable_tag() != record.completion_tag {
        return false;
    }
    match observation.outcome().termination() {
        Some(termination) => termination.stable_tag() == record.termination_tag,
        None => false,
    }
}

pub fn render_run_inspection_report(
    inspection: &ValidatedRunInspection,
    decoded_observation: Option<&ValidatedRawObservation>,
    previews: &InspectionPreviews,
) -> (result: Result<Vec<u8>, InspectionReportError>) {
    let snapshot = inspection.snapshot();
    if !artifact_matches_preview(&snapshot.configuration_source, &previews.configuration_source)
        || !artifact_matches_preview(
        &snapshot.effective_configuration,
        &previews.effective_configuration,
    ) {
        return Err(InspectionReportError::EvidenceMismatch);
    }
    match (&snapshot.observation, decoded_observation, &previews.stdout, &previews.stderr) {
        (Some(record), Some(decoded), Some(stdout), Some(stderr)) => {
            if !observation_matches_snapshot(snapshot, decoded) || !artifact_matches_preview(
                &record.stdout_artifact,
                stdout,
            ) || !artifact_matches_preview(&record.stderr_artifact, stderr) {
                return Err(InspectionReportError::EvidenceMismatch);
            }
        },
        (None, None, None, None) => {},
        _ => return Err(InspectionReportError::EvidenceMismatch),
    }

    let mut output = Vec::new();
    if !append_bytes(&mut output, b"facts:\n") || !append_field(
        &mut output,
        b"run: ",
        snapshot.run_id.as_str(),
    ) || !append_field(&mut output, b"attempt: ", snapshot.attempt_id.as_str()) {
        return Err(InspectionReportError::ReportTooLarge);
    }
    let status = match snapshot.status {
        InspectionStatus::Reserved => "reserved",
        InspectionStatus::TargetPrepared => "target_prepared",
        InspectionStatus::Observed => "observed",
        InspectionStatus::HarnessFailure => "harness_failure",
    };
    if !append_field(&mut output, b"status: ", status) || !append_field(
        &mut output,
        b"configuration-source: ",
        snapshot.configuration_source.id.as_str(),
    ) || !append_preview(
        &mut output,
        b"configuration-source.preview-hex: ",
        b"configuration-source.preview-omitted-bytes: ",
        &previews.configuration_source,
    ) || !append_field(
        &mut output,
        b"effective-configuration: ",
        snapshot.effective_configuration.id.as_str(),
    ) || !append_preview(
        &mut output,
        b"effective-configuration.preview-hex: ",
        b"effective-configuration.preview-omitted-bytes: ",
        &previews.effective_configuration,
    ) || !append_field(
        &mut output,
        b"configuration-digest: ",
        snapshot.configuration_digest.as_str(),
    ) || !append_field(
        &mut output,
        b"capability-manifest: ",
        snapshot.capability_manifest.id.as_str(),
    ) || !append_field(&mut output, b"seed: ", snapshot.seed.as_str()) || !append_field(
        &mut output,
        b"controls.timeout-ms: ",
        snapshot.controls.timeout_ms.as_str(),
    ) || !append_field(
        &mut output,
        b"controls.memory-bytes: ",
        snapshot.controls.memory_bytes.as_str(),
    ) || !append_field(
        &mut output,
        b"controls.max-processes: ",
        snapshot.controls.max_processes.as_str(),
    ) || !append_field(
        &mut output,
        b"controls.max-stream-bytes: ",
        snapshot.controls.max_stream_bytes.as_str(),
    ) || !append_field(
        &mut output,
        b"controls.network-policy: ",
        snapshot.controls.network_policy.as_str(),
    ) || !append_field(
        &mut output,
        b"controls.isolation-backend: ",
        snapshot.controls.isolation_backend.as_str(),
    ) || !append_field(
        &mut output,
        b"controls.output-capture-status: ",
        snapshot.controls.output_capture_status.as_str(),
    ) {
        return Err(InspectionReportError::ReportTooLarge);
    }
    match &snapshot.target {
        Some(target) => {
            if !append_field(&mut output, b"target-build: ", target.build_id.as_str())
                || !append_field(
                &mut output,
                b"target-artifact: ",
                target.target_artifact.id.as_str(),
            ) || !append_field(
                &mut output,
                b"target-manifest: ",
                target.manifest_artifact.id.as_str(),
            ) || !append_field(
                &mut output,
                b"target-identity-digest: ",
                target.identity_digest.as_str(),
            ) {
                return Err(InspectionReportError::ReportTooLarge);
            }
        },
        None => {
            if !append_bytes(&mut output, b"target-build: none\n") {
                return Err(InspectionReportError::ReportTooLarge);
            }
        },
    }
    match (&snapshot.observation, decoded_observation, &previews.stdout, &previews.stderr) {
        (Some(record), Some(decoded), Some(stdout), Some(stderr)) => {
            let raw = decoded.observation();
            if !append_field(&mut output, b"observation: ", record.artifact.id.as_str())
                || !append_number_field(
                &mut output,
                b"observation.completion-tag: ",
                record.completion_tag as u64,
            ) || !append_number_field(
                &mut output,
                b"observation.termination-tag: ",
                record.termination_tag as u64,
            ) || !append_field(
                &mut output,
                b"stdout.artifact: ",
                record.stdout_artifact.id.as_str(),
            ) || !append_number_field(
                &mut output,
                b"stdout.retained-bytes: ",
                raw.stdout().retained_bytes(),
            ) || !append_number_field(
                &mut output,
                b"stdout.discarded-bytes: ",
                raw.stdout().discarded_bytes(),
            ) || !append_field(
                &mut output,
                b"stdout.truncated: ",
                if raw.stdout().truncated() {
                    "true"
                } else {
                    "false"
                },
            ) || !append_preview(
                &mut output,
                b"stdout.preview-hex: ",
                b"stdout.preview-omitted-bytes: ",
                stdout,
            ) || !append_field(
                &mut output,
                b"stderr.artifact: ",
                record.stderr_artifact.id.as_str(),
            ) || !append_number_field(
                &mut output,
                b"stderr.retained-bytes: ",
                raw.stderr().retained_bytes(),
            ) || !append_number_field(
                &mut output,
                b"stderr.discarded-bytes: ",
                raw.stderr().discarded_bytes(),
            ) || !append_field(
                &mut output,
                b"stderr.truncated: ",
                if raw.stderr().truncated() {
                    "true"
                } else {
                    "false"
                },
            ) || !append_preview(
                &mut output,
                b"stderr.preview-hex: ",
                b"stderr.preview-omitted-bytes: ",
                stderr,
            ) || !append_bytes(&mut output, b"harness-failure: none\n") {
                return Err(InspectionReportError::ReportTooLarge);
            }
        },
        (None, None, None, None) => {
            if !append_bytes(&mut output, b"observation: none\n") {
                return Err(InspectionReportError::ReportTooLarge);
            }
            match &snapshot.harness_failure {
                Some(failure) => {
                    if !append_field(&mut output, b"harness-failure: ", failure.kind.as_str()) {
                        return Err(InspectionReportError::ReportTooLarge);
                    }
                },
                None => {
                    if !append_bytes(&mut output, b"harness-failure: none\n") {
                        return Err(InspectionReportError::ReportTooLarge);
                    }
                },
            }
        },
        _ => return Err(InspectionReportError::EvidenceMismatch),
    }
    if !append_bytes(&mut output, b"hypotheses: none\n") {
        return Err(InspectionReportError::ReportTooLarge);
    }
    Ok(output)
}

} // verus!
