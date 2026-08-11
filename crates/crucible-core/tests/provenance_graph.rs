use crucible_core::{
    ActorIdentity, ActorKind, ArtifactRef, EvidenceEnvelope, EvidenceEnvelopeError, EvidenceField,
    EvidenceGraph, EvidenceGraphError, EvidenceId, EvidenceKind, EvidenceNode,
    EvidenceValidationError, GraphInsertOutcome, ProducerIdentity, ProvenanceEdge,
    ProvenanceRelation, SchemaIdentity, TimestampError, TransformationConfiguration,
    TransformationIdentity, UtcTimestamp,
};
use std::string::String;

fn timestamp(seconds: i64) -> UtcTimestamp {
    UtcTimestamp::new(seconds, 123_456_789).expect("nanoseconds are canonical")
}

fn actor(identifier: &str) -> ActorIdentity {
    ActorIdentity {
        kind: ActorKind::Engine,
        identifier: String::from(identifier),
    }
}

fn node(id: &str, payload: &[u8]) -> EvidenceNode {
    EvidenceNode {
        id: EvidenceId::new(String::from(id)),
        kind: EvidenceKind::OriginalObservation,
        payload: ArtifactRef::from_bytes(payload, Some(String::from("application/octet-stream")))
            .expect("test payload is hashable"),
        schema: SchemaIdentity {
            namespace: String::from("gay.dollspace.crucible"),
            name: String::from("raw-observation"),
            version: 1,
        },
        producer: ProducerIdentity {
            actor: actor("engine:test"),
            implementation: None,
            version: String::from("test-v1"),
        },
        created_at: timestamp(1_786_487_275),
    }
}

fn derived_node(id: &str, payload: &[u8]) -> EvidenceNode {
    let mut derived = node(id, payload);
    derived.kind = EvidenceKind::DerivedObservation;
    derived.producer.implementation = Some(
        ArtifactRef::from_bytes(b"engine implementation", None)
            .expect("test implementation is hashable"),
    );
    derived
}

fn transformation() -> TransformationIdentity {
    TransformationIdentity {
        name: String::from("normalize"),
        version: String::from("1"),
        implementation: ArtifactRef::from_bytes(b"transform implementation", None)
            .expect("test implementation is hashable"),
        configuration: TransformationConfiguration::NoneDeclared,
    }
}

fn edge(subject: &str, object: &str, relation: ProvenanceRelation) -> ProvenanceEdge {
    ProvenanceEdge {
        subject: EvidenceId::new(String::from(subject)),
        object: EvidenceId::new(String::from(object)),
        relation,
        transformation: None,
        actor: actor("coordinator:test"),
        recorded_at: timestamp(1_786_487_276),
    }
}

#[test]
fn timestamp_and_envelope_versions_fail_with_typed_errors() {
    assert_eq!(
        UtcTimestamp::new(0, 1_000_000_000),
        Err(TimestampError::NanosecondsOutOfRange)
    );

    let envelope = EvidenceEnvelope::new(node("evidence:enveloped", b"payload"));
    assert_eq!(envelope.schema_version, 1);
    assert!(envelope.into_node().is_ok());

    let future = EvidenceEnvelope {
        schema_version: 2,
        node: node("evidence:future", b"payload"),
    };
    assert_eq!(
        future.into_node(),
        Err(EvidenceEnvelopeError::UnsupportedSchemaVersion)
    );
}

#[test]
fn node_publication_is_idempotent_and_conflicts_never_overwrite() {
    let mut graph = EvidenceGraph::new();
    assert_eq!(
        graph.insert_node(&node("evidence:observation", b"original")),
        Ok(GraphInsertOutcome::Inserted)
    );
    assert_eq!(graph.nodes().len(), 1);

    assert_eq!(
        graph.insert_node(&node("evidence:observation", b"original")),
        Ok(GraphInsertOutcome::AlreadyPresent)
    );
    assert_eq!(graph.nodes().len(), 1);

    assert_eq!(
        graph.insert_node(&node("evidence:observation", b"conflicting")),
        Err(EvidenceGraphError::NodeConflict)
    );
    assert_eq!(graph.nodes().len(), 1);
    assert_eq!(graph.nodes()[0].payload.verify(b"original"), Ok(()));
    assert_ne!(graph.nodes()[0].payload.verify(b"conflicting"), Ok(()));
}

#[test]
fn edge_publication_requires_endpoints_and_is_idempotent() {
    let mut graph = EvidenceGraph::new();
    let provenance = edge(
        "evidence:source",
        "evidence:derived",
        ProvenanceRelation::DerivedFrom,
    );
    assert_eq!(
        graph.insert_edge(&provenance),
        Err(EvidenceGraphError::MissingSubjectNode)
    );

    assert_eq!(
        graph.insert_node(&node("evidence:source", b"source")),
        Ok(GraphInsertOutcome::Inserted)
    );
    assert_eq!(
        graph.insert_edge(&provenance),
        Err(EvidenceGraphError::MissingObjectNode)
    );
    assert_eq!(
        graph.insert_node(&node("evidence:derived", b"derived")),
        Ok(GraphInsertOutcome::Inserted)
    );

    // The failed calls borrowed, rather than consumed, this exact timestamped publication.
    assert_eq!(
        graph.insert_edge(&provenance),
        Ok(GraphInsertOutcome::Inserted)
    );
    assert_eq!(
        graph.insert_edge(&provenance),
        Ok(GraphInsertOutcome::AlreadyPresent)
    );
    assert_eq!(
        graph.insert_edge(&edge(
            "evidence:source",
            "evidence:derived",
            ProvenanceRelation::Supports,
        )),
        Ok(GraphInsertOutcome::Inserted)
    );
    assert_eq!(graph.edges().len(), 2);
}

#[test]
fn invalid_records_are_rejected_without_weakening_graph_well_formedness() {
    let mut graph = EvidenceGraph::new();
    let mut invalid_schema = node("evidence:invalid-schema", b"payload");
    invalid_schema.schema.namespace = String::new();
    assert_eq!(
        graph.insert_node(&invalid_schema),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Empty(EvidenceField::SchemaNamespace)
        ))
    );

    let mut malformed_payload = node("evidence:invalid-payload", b"payload");
    malformed_payload.payload.id = crucible_core::ArtifactId::new(String::from("not-an-id"));
    assert_eq!(
        graph.insert_node(&malformed_payload),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::MalformedArtifact(EvidenceField::Payload)
        ))
    );
    assert!(graph.nodes().is_empty());
    assert!(graph.edges().is_empty());
}

#[test]
fn derived_evidence_is_published_atomically_with_all_declared_inputs() {
    let mut graph = EvidenceGraph::new();
    let source_a = node("evidence:source-a", b"source-a");
    let source_b = node("evidence:source-b", b"source-b");
    assert_eq!(
        graph.insert_node(&source_a),
        Ok(GraphInsertOutcome::Inserted)
    );
    assert_eq!(
        graph.insert_node(&source_b),
        Ok(GraphInsertOutcome::Inserted)
    );

    let derived = derived_node("evidence:derived", b"derived");
    assert_eq!(
        graph.insert_node(&derived),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Missing(EvidenceField::DerivationInputs)
        ))
    );
    assert_eq!(
        graph.publish_derivation(&derived, &[]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Missing(EvidenceField::DerivationInputs)
        ))
    );

    let mut input_a = edge(
        "evidence:derived",
        "evidence:source-a",
        ProvenanceRelation::DerivedFrom,
    );
    input_a.actor = actor("engine:test");
    input_a.transformation = Some(transformation());
    let mut input_b = edge(
        "evidence:derived",
        "evidence:source-b",
        ProvenanceRelation::DerivedFrom,
    );
    input_b.actor = actor("engine:test");
    input_b.transformation = Some(transformation());

    let inputs = vec![input_a, input_b];
    assert_eq!(
        graph.publish_derivation(&derived, &inputs),
        Ok(GraphInsertOutcome::Inserted)
    );
    assert_eq!(
        graph.publish_derivation(&derived, &inputs),
        Ok(GraphInsertOutcome::AlreadyPresent)
    );
    assert_eq!(graph.nodes().len(), 3);
    assert_eq!(graph.edges().len(), 2);
    assert_eq!(graph.edges()[0].subject.as_str(), "evidence:derived");
    assert_eq!(graph.edges()[0].object.as_str(), "evidence:source-a");
}

#[test]
fn every_node_identity_field_participates_in_conflict_detection() {
    macro_rules! assert_conflict {
        ($mutation:expr) => {{
            let mut graph = EvidenceGraph::new();
            let original = node("evidence:identity", b"payload");
            let mut changed = node("evidence:identity", b"payload");
            ($mutation)(&mut changed);
            assert_eq!(
                graph.insert_node(&original),
                Ok(GraphInsertOutcome::Inserted)
            );
            assert_eq!(
                graph.insert_node(&changed),
                Err(EvidenceGraphError::NodeConflict)
            );
            assert_eq!(graph.nodes().len(), 1);
        }};
    }

    assert_conflict!(|changed: &mut EvidenceNode| changed.kind = EvidenceKind::SourceSnapshot);
    assert_conflict!(|changed: &mut EvidenceNode| changed.payload =
        ArtifactRef::from_bytes(b"other", Some(String::from("application/octet-stream")))
            .expect("test payload is hashable"));
    assert_conflict!(|changed: &mut EvidenceNode| changed.payload.size_bytes += 1);
    assert_conflict!(|changed: &mut EvidenceNode| changed.payload.media_type =
        Some(String::from("application/cbor")));
    assert_conflict!(|changed: &mut EvidenceNode| changed.schema.namespace.push('x'));
    assert_conflict!(|changed: &mut EvidenceNode| changed.schema.name.push('x'));
    assert_conflict!(|changed: &mut EvidenceNode| changed.schema.version += 1);
    assert_conflict!(|changed: &mut EvidenceNode| changed.producer.actor.kind = ActorKind::Worker);
    assert_conflict!(|changed: &mut EvidenceNode| changed.producer.actor.identifier.push('x'));
    assert_conflict!(
        |changed: &mut EvidenceNode| changed.producer.implementation = Some(
            ArtifactRef::from_bytes(b"implementation", None)
                .expect("test implementation is hashable")
        )
    );
    assert_conflict!(|changed: &mut EvidenceNode| changed.producer.version.push('x'));
    assert_conflict!(|changed: &mut EvidenceNode| changed.created_at = timestamp(1_786_487_277));
}

#[test]
fn every_edge_identity_field_participates_in_exact_retry_detection() {
    macro_rules! assert_distinct {
        ($mutation:expr) => {{
            let mut graph = EvidenceGraph::new();
            assert_eq!(
                graph.insert_node(&node("evidence:subject", b"subject")),
                Ok(GraphInsertOutcome::Inserted)
            );
            assert_eq!(
                graph.insert_node(&node("evidence:object", b"object")),
                Ok(GraphInsertOutcome::Inserted)
            );
            let mut original = edge(
                "evidence:subject",
                "evidence:object",
                ProvenanceRelation::Supports,
            );
            original.transformation = Some(transformation());
            let mut changed = edge(
                "evidence:subject",
                "evidence:object",
                ProvenanceRelation::Supports,
            );
            changed.transformation = Some(transformation());
            ($mutation)(&mut changed);
            assert_eq!(
                graph.insert_edge(&original),
                Ok(GraphInsertOutcome::Inserted)
            );
            assert_eq!(
                graph.insert_edge(&changed),
                Ok(GraphInsertOutcome::Inserted)
            );
            assert_eq!(graph.edges().len(), 2);
        }};
    }

    assert_distinct!(
        |changed: &mut ProvenanceEdge| changed.relation = ProvenanceRelation::Contradicts
    );
    assert_distinct!(|changed: &mut ProvenanceEdge| changed.actor.kind = ActorKind::Worker);
    assert_distinct!(|changed: &mut ProvenanceEdge| changed.actor.identifier.push('x'));
    assert_distinct!(|changed: &mut ProvenanceEdge| changed.recorded_at = timestamp(1_786_487_278));
    assert_distinct!(|changed: &mut ProvenanceEdge| changed
        .transformation
        .as_mut()
        .expect("present")
        .name
        .push('x'));
    assert_distinct!(|changed: &mut ProvenanceEdge| changed
        .transformation
        .as_mut()
        .expect("present")
        .version
        .push('x'));
    assert_distinct!(|changed: &mut ProvenanceEdge| changed
        .transformation
        .as_mut()
        .expect("present")
        .implementation =
        ArtifactRef::from_bytes(b"other implementation", None).expect("hashable"));
    assert_distinct!(|changed: &mut ProvenanceEdge| changed
        .transformation
        .as_mut()
        .expect("present")
        .configuration =
        TransformationConfiguration::Artifact(
            ArtifactRef::from_bytes(b"config", None).expect("hashable")
        ));
}

#[test]
fn derivation_direction_and_identity_are_validated() {
    let mut graph = EvidenceGraph::new();
    let source = node("evidence:source", b"source");
    let derived = derived_node("evidence:derived", b"derived");
    assert_eq!(graph.insert_node(&source), Ok(GraphInsertOutcome::Inserted));

    let mut missing_implementation = node("evidence:missing-implementation", b"derived");
    missing_implementation.kind = EvidenceKind::DerivedObservation;
    assert_eq!(
        graph.publish_derivation(&missing_implementation, &[]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Missing(EvidenceField::ProducerImplementation)
        ))
    );

    let mut backwards = edge(
        "evidence:source",
        "evidence:derived",
        ProvenanceRelation::DerivedFrom,
    );
    backwards.actor = actor("engine:test");
    backwards.transformation = Some(transformation());
    assert_eq!(
        graph.publish_derivation(&derived, &[backwards]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Mismatch(EvidenceField::DerivationSubject)
        ))
    );

    let mut missing_transformation = edge(
        "evidence:derived",
        "evidence:source",
        ProvenanceRelation::DerivedFrom,
    );
    missing_transformation.actor = actor("engine:test");
    assert_eq!(
        graph.publish_derivation(&derived, &[missing_transformation]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Missing(EvidenceField::DerivationTransformation)
        ))
    );

    let mut wrong_relation = edge(
        "evidence:derived",
        "evidence:source",
        ProvenanceRelation::Supports,
    );
    wrong_relation.actor = actor("engine:test");
    wrong_relation.transformation = Some(transformation());
    assert_eq!(
        graph.publish_derivation(&derived, &[wrong_relation]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Mismatch(EvidenceField::DerivationRelation)
        ))
    );

    let mut wrong_actor = edge(
        "evidence:derived",
        "evidence:source",
        ProvenanceRelation::DerivedFrom,
    );
    wrong_actor.transformation = Some(transformation());
    assert_eq!(
        graph.publish_derivation(&derived, &[wrong_actor]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Mismatch(EvidenceField::DerivationActor)
        ))
    );

    let mut duplicate_a = edge(
        "evidence:derived",
        "evidence:source",
        ProvenanceRelation::DerivedFrom,
    );
    duplicate_a.actor = actor("engine:test");
    duplicate_a.transformation = Some(transformation());
    let mut duplicate_b = edge(
        "evidence:derived",
        "evidence:source",
        ProvenanceRelation::DerivedFrom,
    );
    duplicate_b.actor = actor("engine:test");
    duplicate_b.transformation = Some(transformation());
    assert_eq!(
        graph.publish_derivation(&derived, &[duplicate_a, duplicate_b]),
        Err(EvidenceGraphError::Validation(
            EvidenceValidationError::Duplicate(EvidenceField::DerivationInputs)
        ))
    );
    assert_eq!(graph.nodes().len(), 1);
    assert!(graph.edges().is_empty());
}

#[test]
fn every_required_provenance_relation_is_representable() {
    let relations = [
        ProvenanceRelation::DerivedFrom,
        ProvenanceRelation::GeneratedBy,
        ProvenanceRelation::Evaluates,
        ProvenanceRelation::Supports,
        ProvenanceRelation::Contradicts,
        ProvenanceRelation::Reproduces,
        ProvenanceRelation::Minimizes,
        ProvenanceRelation::Verifies,
        ProvenanceRelation::Invalidates,
        ProvenanceRelation::Supersedes,
    ];
    assert_eq!(relations.len(), 10);
}
