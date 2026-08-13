#![forbid(unsafe_code)]
#![doc = r#"
Crucible's domain identifiers are intentionally different Rust types.

```compile_fail
use crucible_core::{RunId, TargetId};

let target = TargetId::new(String::from("same-text"));
let run: RunId = target;
```
"#]

use vstd::prelude::*;

pub mod artifact;
pub mod execution;
pub mod execution_codec;
pub mod provenance;

pub use artifact::{
    parse_artifact_id, sha256, ArtifactIdParseError, ArtifactIdentityError, ArtifactRef,
    ContentDigest, DigestAlgorithm, DigestDecodeError, HashError, Sha256Digest,
};
pub use execution::{
    canonical_raw_execution_outcome_limits, validate_raw_execution_outcome, CompletionDisposition,
    HarnessTerminationReason, LogicalProcessId, LogicalProcessIdError, RawExecutionEvent,
    RawExecutionOutcome, RawExecutionOutcomeError, RawExecutionOutcomeErrorKind,
    RawExecutionOutcomeLimits, RawExecutionOutcomeLimitsView, RawExecutionOutcomeLocation,
    RawExecutionOutcomeRejection, ResetCause, ResourceKind, TerminationRecord,
    ValidatedRawExecutionOutcome, VersionedExtensionRef, MAX_RAW_EXECUTION_EVENTS,
    MAX_RAW_EXECUTION_EXTENSION_MEDIA_TYPE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_NAMESPACE_CODE_POINTS,
    MAX_RAW_EXECUTION_EXTENSION_PAYLOAD_BYTES_PER_RECORD, RAW_EXECUTION_OUTCOME_SCHEMA_VERSION,
};
pub use execution_codec::{
    canonical_raw_execution_outcome_codec_limits, decode_raw_execution_outcome,
    encode_raw_execution_outcome, RawExecutionOutcomeCodecError, RawExecutionOutcomeCodecErrorKind,
    RawExecutionOutcomeCodecLimits, RawExecutionOutcomeCodecRejection,
    MAX_RAW_EXECUTION_OUTCOME_ENCODED_BYTES,
};
pub use provenance::{
    ActorIdentity, ActorKind, EvidenceEnvelope, EvidenceEnvelopeError, EvidenceField,
    EvidenceGraph, EvidenceGraphError, EvidenceKind, EvidenceNode, EvidenceValidationError,
    GraphInsertOutcome, ProducerIdentity, ProvenanceEdge, ProvenanceRelation, SchemaIdentity,
    TimestampError, TransformationConfiguration, TransformationIdentity, UtcTimestamp,
};

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdKind {
    Target,
    TargetBuild,
    Campaign,
    Experiment,
    Scenario,
    Participant,
    Run,
    RunAttempt,
    Finding,
    FindingInstance,
    Patch,
    Oracle,
    Engine,
    Artifact,
    Evidence,
    ProofArtifact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdDecodeError {
    UnsupportedSchemaVersion,
    WrongKind,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalIdEnvelope {
    pub schema_version: u8,
    pub kind: IdKind,
    pub value: String,
}

pub open spec fn same_id_kind_spec(left: IdKind, right: IdKind) -> bool {
    left == right
}

#[allow(clippy::match_like_matches_macro)]
pub fn same_id_kind(left: IdKind, right: IdKind) -> (same: bool)
    ensures
        same == same_id_kind_spec(left, right),
{
    match (left, right) {
        (IdKind::Target, IdKind::Target)
        | (IdKind::TargetBuild, IdKind::TargetBuild)
        | (IdKind::Campaign, IdKind::Campaign)
        | (IdKind::Experiment, IdKind::Experiment)
        | (IdKind::Scenario, IdKind::Scenario)
        | (IdKind::Participant, IdKind::Participant)
        | (IdKind::Run, IdKind::Run)
        | (IdKind::RunAttempt, IdKind::RunAttempt)
        | (IdKind::Finding, IdKind::Finding)
        | (IdKind::FindingInstance, IdKind::FindingInstance)
        | (IdKind::Patch, IdKind::Patch)
        | (IdKind::Oracle, IdKind::Oracle)
        | (IdKind::Engine, IdKind::Engine)
        | (IdKind::Artifact, IdKind::Artifact)
        | (IdKind::Evidence, IdKind::Evidence)
        | (IdKind::ProofArtifact, IdKind::ProofArtifact) => true,
        _ => false,
    }
}

} // verus!
macro_rules! define_text_id {
    ($name:ident, $kind:expr) => {
        verus! {

        #[derive(Debug, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $name(pub String);

        impl View for $name {
            type V = Seq<char>;

            open spec fn view(&self) -> Seq<char> {
                self.0@
            }
        }

        impl $name {
            pub fn new(value: String) -> (id: Self)
                ensures
                    id@ == value@,
            {
                Self(value)
            }

            pub fn as_str(&self) -> (value: &str)
                ensures
                    value@ == self@,
            {
                self.0.as_str()
            }

            pub fn into_inner(self) -> (value: String)
                ensures
                    value@ == self@,
            {
                self.0
            }

            pub fn to_envelope(&self) -> (envelope: CanonicalIdEnvelope)
                ensures
                    envelope.schema_version == 1,
                    envelope.kind == $kind,
                    envelope.value@ == self@,
            {
                CanonicalIdEnvelope {
                    schema_version: 1,
                    kind: $kind,
                    value: self.0.clone(),
                }
            }

            pub fn from_envelope(
                envelope: CanonicalIdEnvelope,
            ) -> (result: Result<Self, IdDecodeError>)
                ensures
                    envelope.schema_version != 1 ==> match result {
                        Err(IdDecodeError::UnsupportedSchemaVersion) => true,
                        _ => false,
                    },
                    envelope.schema_version == 1 && !same_id_kind_spec(envelope.kind, $kind)
                        ==> match result {
                            Err(IdDecodeError::WrongKind) => true,
                            _ => false,
                        },
                    envelope.schema_version == 1 && same_id_kind_spec(envelope.kind, $kind)
                        ==> match result {
                        Ok(id) => id@ == envelope.value@,
                        Err(_) => false,
                    },
            {
                let same_kind = same_id_kind(envelope.kind, $kind);
                if envelope.schema_version != 1 {
                    Err(IdDecodeError::UnsupportedSchemaVersion)
                } else if !same_kind {
                    Err(IdDecodeError::WrongKind)
                } else {
                    Ok(Self(envelope.value))
                }
            }
        }

        impl Clone for $name {
            fn clone(&self) -> (id: Self)
                ensures
                    id@ == self@,
            {
                Self(self.0.clone())
            }
        }

        } // verus!
    };
}

define_text_id!(TargetId, IdKind::Target);
define_text_id!(TargetBuildId, IdKind::TargetBuild);
define_text_id!(CampaignId, IdKind::Campaign);
define_text_id!(ExperimentId, IdKind::Experiment);
define_text_id!(ScenarioId, IdKind::Scenario);
define_text_id!(ParticipantId, IdKind::Participant);
define_text_id!(RunId, IdKind::Run);
define_text_id!(RunAttemptId, IdKind::RunAttempt);
define_text_id!(FindingId, IdKind::Finding);
define_text_id!(FindingInstanceId, IdKind::FindingInstance);
define_text_id!(PatchId, IdKind::Patch);
define_text_id!(OracleId, IdKind::Oracle);
define_text_id!(EngineId, IdKind::Engine);
define_text_id!(ArtifactId, IdKind::Artifact);
define_text_id!(EvidenceId, IdKind::Evidence);
define_text_id!(ProofArtifactId, IdKind::ProofArtifact);

#[cfg(test)]
mod artifact_runtime_tests;

#[cfg(test)]
mod provenance_runtime_tests;

#[cfg(test)]
mod unit_tests;
