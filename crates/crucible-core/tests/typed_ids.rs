use crucible_core::{
    ArtifactId, CampaignId, CanonicalIdEnvelope, EngineId, EvidenceId, ExperimentId, FindingId,
    FindingInstanceId, IdDecodeError, IdKind, OracleId, ParticipantId, PatchId, ProofArtifactId,
    RunAttemptId, RunId, ScenarioId, TargetBuildId, TargetId,
};
use vstd::prelude::*;

verus! {

#[test]
fn every_required_id_is_a_concrete_public_type() {
    let value = String::from_str("stable-record-id");

    let _target_id: TargetId = TargetId::new(value.clone());
    let _target_build_id: TargetBuildId = TargetBuildId::new(value.clone());
    let _campaign_id: CampaignId = CampaignId::new(value.clone());
    let _experiment_id: ExperimentId = ExperimentId::new(value.clone());
    let _scenario_id: ScenarioId = ScenarioId::new(value.clone());
    let _participant_id: ParticipantId = ParticipantId::new(value.clone());
    let _run_id: RunId = RunId::new(value.clone());
    let _run_attempt_id: RunAttemptId = RunAttemptId::new(value.clone());
    let _finding_id: FindingId = FindingId::new(value.clone());
    let _finding_instance_id: FindingInstanceId = FindingInstanceId::new(value.clone());
    let _patch_id: PatchId = PatchId::new(value.clone());
    let _oracle_id: OracleId = OracleId::new(value.clone());
    let _engine_id: EngineId = EngineId::new(value.clone());
    let _artifact_id: ArtifactId = ArtifactId::new(value.clone());
    let _evidence_id: EvidenceId = EvidenceId::new(value.clone());
    let _proof_artifact_id: ProofArtifactId = ProofArtifactId::new(value);
}

#[test]
fn textual_serialization_is_lossless_and_stable() {
    let source = String::from_str("artifact:sha256:cephalopod-☃");
    let id = ArtifactId::new(source.clone());

    let _id_text = id.as_str();
    let _source_text = source.as_str();
    assert(_id_text@ == _source_text@);
    vstd::pervasive::runtime_assert(id.clone().into_inner() == source.clone());

    let serialized = id.into_inner();
    vstd::pervasive::runtime_assert(serialized == source);
}

#[test]
fn empty_text_is_preserved_without_adding_an_unspecified_policy() {
    let source = String::new();
    let id = RunId::new(source.clone());

    let _id_text = id.as_str();
    let _source_text = source.as_str();
    assert(_id_text@ == _source_text@);
    vstd::pervasive::runtime_assert(id.clone().into_inner() == source.clone());
    vstd::pervasive::runtime_assert(id.into_inner() == source);
}

#[test]
fn versioned_envelope_round_trips_the_id_kind_and_value() {
    let source = String::from_str("run:stable-record-id");
    let encoded: CanonicalIdEnvelope = RunId::new(source.clone()).to_envelope();

    assert(encoded.schema_version == 1);
    assert(encoded.kind == IdKind::Run);
    assert(encoded.value@ == source@);

    let decoded = RunId::from_envelope(encoded);
    match decoded {
        Ok(id) => vstd::pervasive::runtime_assert(id.into_inner() == source),
        Err(_) => vstd::pervasive::unreached(),
    }
}

#[test]
fn versioned_envelope_rejects_cross_id_type_confusion() {
    let envelope = CanonicalIdEnvelope {
        schema_version: 1,
        kind: IdKind::Target,
        value: String::from_str("same-text-different-type"),
    };

    let decoded = RunId::from_envelope(envelope);
    match decoded {
        Err(IdDecodeError::WrongKind) => {},
        _ => vstd::pervasive::unreached(),
    }
}

#[test]
fn versioned_envelope_rejects_unknown_schema_versions() {
    let envelope = CanonicalIdEnvelope {
        schema_version: 2,
        kind: IdKind::Run,
        value: String::from_str("future-version"),
    };

    let decoded = RunId::from_envelope(envelope);
    match decoded {
        Err(IdDecodeError::UnsupportedSchemaVersion) => {},
        _ => vstd::pervasive::unreached(),
    }
}

} // verus!
