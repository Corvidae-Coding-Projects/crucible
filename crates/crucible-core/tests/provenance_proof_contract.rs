#![allow(unused_imports)]
// Explicit result matches and incremental vectors expose branch/sequence state to Verus.
#![allow(clippy::single_match, clippy::vec_init_then_push)]

use crucible_core::{
    ActorIdentity, ActorKind, ArtifactIdentityError, ArtifactRef, EvidenceGraph,
    EvidenceGraphError, EvidenceId, EvidenceKind, EvidenceNode, GraphInsertOutcome,
    ProducerIdentity, ProvenanceEdge, ProvenanceRelation, SchemaIdentity,
    TransformationConfiguration, TransformationIdentity, UtcTimestamp,
};
use vstd::prelude::*;

verus! {

fn nonempty_text(character: char) -> (text: String)
    ensures
        text@ == seq![character],
{
    let mut text = String::new();
    text.push(character);
    text
}

fn proof_node(id: String, payload: &[u8]) -> (created: EvidenceNode)
    requires
        crucible_core::artifact::sha256_input_supported(payload@.len() as nat),
        id@.len() > 0,
    ensures
        created.id@ == id@,
        created.kind == EvidenceKind::OriginalObservation,
        created.schema.version == 1,
        created.producer.actor.kind == ActorKind::Engine,
        created.producer.actor@.identifier == seq!['a'],
        created.producer.implementation is None,
        crucible_core::provenance::evidence_node_structurally_valid_spec(created@),
{
    let artifact = match ArtifactRef::from_bytes(payload, None) {
        Ok(artifact) => artifact,
        Err(_) => vstd::pervasive::unreached(),
    };
    proof {
        crucible_core::artifact::lemma_artifact_id_spec_is_canonical(
            crucible_core::artifact::sha256_spec(payload@),
        );
    }
    let created_at = match UtcTimestamp::new(0, 0) {
        Ok(timestamp) => timestamp,
        Err(_) => vstd::pervasive::unreached(),
    };
    EvidenceNode {
        id: EvidenceId::new(id),
        kind: EvidenceKind::OriginalObservation,
        payload: artifact,
        schema: SchemaIdentity {
            namespace: nonempty_text('n'),
            name: nonempty_text('s'),
            version: 1,
        },
        producer: ProducerIdentity {
            actor: ActorIdentity { kind: ActorKind::Engine, identifier: nonempty_text('a') },
            implementation: None,
            version: nonempty_text('v'),
        },
        created_at,
    }
}

fn proof_timestamp(seconds: i64) -> (timestamp: UtcTimestamp)
    ensures
        crucible_core::provenance::timestamp_valid_spec(timestamp@),
{
    match UtcTimestamp::new(seconds, 0) {
        Ok(timestamp) => timestamp,
        Err(_) => vstd::pervasive::unreached(),
    }
}

fn proof_edge(subject: char, object: char) -> (edge: ProvenanceEdge)
    ensures
        edge.subject@ == seq![subject],
        edge.object@ == seq![object],
        crucible_core::provenance::provenance_edge_structurally_valid_spec(edge@),
{
    ProvenanceEdge {
        subject: EvidenceId::new(nonempty_text(subject)),
        object: EvidenceId::new(nonempty_text(object)),
        relation: ProvenanceRelation::Supports,
        transformation: None,
        actor: ActorIdentity { kind: ActorKind::Engine, identifier: nonempty_text('a') },
        recorded_at: proof_timestamp(1),
    }
}

fn proof_derivation_edge(subject: char, object: char) -> (edge: ProvenanceEdge)
    ensures
        edge.subject@ == seq![subject],
        edge.object@ == seq![object],
        edge.actor.kind == ActorKind::Engine,
        edge.actor@.identifier == seq!['a'],
        crucible_core::provenance::provenance_edge_structurally_valid_spec(edge@),
        edge.relation == ProvenanceRelation::DerivedFrom,
        edge.transformation is Some,
{
    let implementation = match ArtifactRef::from_bytes(b"transform", None) {
        Ok(implementation) => implementation,
        Err(_) => vstd::pervasive::unreached(),
    };
    proof {
        crucible_core::artifact::lemma_artifact_id_spec_is_canonical(
            crucible_core::artifact::sha256_spec(b"transform"@),
        );
    }
    ProvenanceEdge {
        subject: EvidenceId::new(nonempty_text(subject)),
        object: EvidenceId::new(nonempty_text(object)),
        relation: ProvenanceRelation::DerivedFrom,
        transformation: Some(
            TransformationIdentity {
                name: nonempty_text('t'),
                version: nonempty_text('v'),
                implementation,
                configuration: TransformationConfiguration::NoneDeclared,
            },
        ),
        actor: ActorIdentity { kind: ActorKind::Engine, identifier: nonempty_text('a') },
        recorded_at: proof_timestamp(2),
    }
}

fn proof_derived_node(id: char, payload: &[u8]) -> (created: EvidenceNode)
    requires
        crucible_core::artifact::sha256_input_supported(payload@.len() as nat),
    ensures
        created.id@ == seq![id],
        created.producer.actor.kind == ActorKind::Engine,
        created.producer.actor@.identifier == seq!['a'],
        created.kind == EvidenceKind::DerivedObservation,
        crucible_core::provenance::evidence_node_structurally_valid_spec(created@),
{
    let mut created = proof_node(nonempty_text(id), payload);
    let implementation = match ArtifactRef::from_bytes(b"producer", None) {
        Ok(implementation) => implementation,
        Err(_) => vstd::pervasive::unreached(),
    };
    proof {
        crucible_core::artifact::lemma_artifact_id_spec_is_canonical(
            crucible_core::artifact::sha256_spec(b"producer"@),
        );
    }
    created.kind = EvidenceKind::DerivedObservation;
    created.producer.implementation = Some(implementation);
    created
}

#[test]
fn successful_node_insert_is_an_append_only_transition() {
    let mut graph = EvidenceGraph::new();
    let inserted = proof_node(nonempty_text('e'), b"proof-payload");
    let ghost inserted_view = inserted@;
    assert(crucible_core::provenance::evidence_node_structurally_valid_spec(inserted@));
    assert(!crucible_core::provenance::evidence_kind_requires_derivation_spec(inserted.kind));
    match graph.insert_node(&inserted) {
        Ok(GraphInsertOutcome::Inserted) => {
            assert(graph@.nodes == seq![inserted_view]);
            assert(graph@.edges.len() == 0);
            assert(crucible_core::provenance::evidence_graph_well_formed_spec(graph@));
        },
        Ok(GraphInsertOutcome::AlreadyPresent) => assert(false),
        Err(EvidenceGraphError::NodeConflict)
        | Err(EvidenceGraphError::MissingSubjectNode)
        | Err(EvidenceGraphError::MissingObjectNode)
        | Err(EvidenceGraphError::Validation(_)) => assert(false),
        Err(_) => {},
        Ok(_) => {},
    }
}

#[test]
fn retries_conflicts_endpoints_edges_and_derivations_preserve_contracts() {
    let mut graph = EvidenceGraph::new();
    let source = proof_node(nonempty_text('s'), b"source");
    match graph.insert_node(&source) {
        Ok(GraphInsertOutcome::Inserted) => {},
        _ => assert(false),
    }
    let ghost after_source = graph@;
    assert(after_source.nodes == seq![source@]);
    assert(crucible_core::provenance::artifact_ref_equal_spec(source@.payload, source@.payload));
    assert(crucible_core::provenance::schema_identity_equal_spec(source@.schema, source@.schema));
    assert(crucible_core::provenance::actor_identity_equal_spec(
        source@.producer.actor,
        source@.producer.actor,
    ));
    assert(crucible_core::provenance::producer_identity_equal_spec(
        source@.producer,
        source@.producer,
    ));
    assert(crucible_core::provenance::evidence_node_equal_spec(source@, source@));
    assert(crucible_core::provenance::contains_evidence_node_spec(after_source.nodes, source@)) by {
        assert(after_source.nodes[0] == source@);
        assert(crucible_core::provenance::evidence_node_equal_spec(after_source.nodes[0], source@));
    };
    match graph.insert_node(&source) {
        Ok(GraphInsertOutcome::AlreadyPresent) => assert(graph@ == after_source),
        _ => assert(false),
    }

    let mut conflict = proof_node(nonempty_text('s'), b"source");
    conflict.schema.version = 2;
    assert(source@.schema.version == 1);
    assert(conflict@.schema.version == 2);
    assert(!crucible_core::provenance::evidence_node_equal_spec(source@, conflict@));
    assert(crucible_core::provenance::contains_conflicting_evidence_node_spec(
        after_source.nodes,
        conflict@,
    )) by {
        assert(after_source.nodes[0] == source@);
        assert(after_source.nodes[0].id == conflict@.id);
        assert(!crucible_core::provenance::evidence_node_equal_spec(
            after_source.nodes[0],
            conflict@,
        ));
    };
    match graph.insert_node(&conflict) {
        Err(EvidenceGraphError::NodeConflict) => assert(graph@ == after_source),
        _ => assert(false),
    }

    let relation = proof_edge('s', 'o');
    assert(crucible_core::provenance::contains_evidence_id_spec(graph@.nodes, relation@.subject))
        by {
        assert(graph@.nodes[0].id == relation@.subject);
    };
    assert(!crucible_core::provenance::contains_evidence_id_spec(graph@.nodes, relation@.object))
        by {
        assert(graph@.nodes.len() == 1);
        assert(graph@.nodes[0].id == seq!['s']);
        assert(relation@.object == seq!['o']);
        assert(seq!['s'] != seq!['o']) by {
            assert(seq!['s'][0] == 's');
            assert(seq!['o'][0] == 'o');
        };
        assert(graph@.nodes[0].id != relation@.object);
    };
    match graph.insert_edge(&relation) {
        Err(EvidenceGraphError::MissingObjectNode) => assert(graph@ == after_source),
        _ => assert(false),
    }
    let object = proof_node(nonempty_text('o'), b"object");
    match graph.insert_node(&object) {
        Ok(GraphInsertOutcome::Inserted) => {},
        _ => assert(false),
    }
    assert(graph@.nodes.len() == 2);
    assert(graph@.edges.len() == 0);
    assert(crucible_core::provenance::contains_evidence_id_spec(graph@.nodes, relation@.subject))
        by {
        assert(graph@.nodes[0].id == relation@.subject);
    };
    assert(crucible_core::provenance::contains_evidence_id_spec(graph@.nodes, relation@.object))
        by {
        assert(graph@.nodes[1].id == relation@.object);
    };
    assert(!crucible_core::provenance::contains_provenance_edge_spec(graph@.edges, relation@));
    match graph.insert_edge(&relation) {
        Ok(GraphInsertOutcome::Inserted) => {
            assert(graph@.edges.len() == 1);
            assert(crucible_core::provenance::evidence_graph_well_formed_spec(graph@));
        },
        _ => assert(false),
    }
    let ghost after_edge = graph@;
    assert(after_edge.edges == seq![relation@]);
    assert(crucible_core::provenance::actor_identity_equal_spec(relation@.actor, relation@.actor));
    assert(crucible_core::provenance::provenance_edge_equal_spec(relation@, relation@));
    assert(crucible_core::provenance::contains_provenance_edge_spec(after_edge.edges, relation@))
        by {
        assert(after_edge.edges[0] == relation@);
        assert(crucible_core::provenance::provenance_edge_equal_spec(
            after_edge.edges[0],
            relation@,
        ));
    };
    match graph.insert_edge(&relation) {
        Ok(GraphInsertOutcome::AlreadyPresent) => assert(graph@ == after_edge),
        _ => assert(false),
    }

    let mut derived_graph = EvidenceGraph::new();
    let derivation_source = proof_node(nonempty_text('s'), b"source");
    match derived_graph.insert_node(&derivation_source) {
        Ok(GraphInsertOutcome::Inserted) => {},
        _ => assert(false),
    }
    let derived = proof_derived_node('d', b"derived");
    let input = proof_derivation_edge('d', 's');
    let mut inputs = Vec::new();
    inputs.push(input);
    let ghost input_views = crucible_core::provenance::provenance_edge_views_spec(inputs@);
    assert(input_views.len() == 1);
    assert(input_views[0] == inputs@[0]@);
    assert(input_views[0].subject == derived@.id);
    assert(input_views[0].relation == ProvenanceRelation::DerivedFrom);
    assert(input_views[0].transformation is Some);
    assert(crucible_core::provenance::actor_identity_equal_spec(
        input_views[0].actor,
        derived@.producer.actor,
    ));
    assert(crucible_core::provenance::contains_evidence_id_spec(
        derived_graph@.nodes,
        input_views[0].object,
    )) by {
        assert(derived_graph@.nodes[0].id == input_views[0].object);
    };
    assert(crucible_core::provenance::derivation_inputs_valid_spec(
        derived_graph@,
        derived@,
        input_views,
    )) by {
        assert forall|index: int| 0 <= index < input_views.len() implies {
            &&& crucible_core::provenance::provenance_edge_structurally_valid_spec(
                #[trigger] input_views[index],
            )
            &&& input_views[index].subject == derived@.id
            &&& input_views[index].relation == ProvenanceRelation::DerivedFrom
            &&& input_views[index].transformation is Some
            &&& crucible_core::provenance::actor_identity_equal_spec(
                input_views[index].actor,
                derived@.producer.actor,
            )
            &&& crucible_core::provenance::contains_evidence_id_spec(
                derived_graph@.nodes,
                input_views[index].object,
            )
        } by {
            assert(index == 0);
        };
    };
    assert(!crucible_core::provenance::contains_evidence_id_spec(derived_graph@.nodes, derived@.id))
        by {
        assert(derived_graph@.nodes.len() == 1);
        assert(derived_graph@.nodes[0].id == seq!['s']);
        assert(derived@.id == seq!['d']);
        assert(seq!['s'] != seq!['d']) by {
            assert(seq!['s'][0] != seq!['d'][0]);
        };
    };
    match derived_graph.publish_derivation(&derived, inputs.as_slice()) {
        Ok(GraphInsertOutcome::Inserted) => {
            assert(derived_graph@.nodes.len() == 2);
            assert(derived_graph@.edges.len() == 1);
            assert(crucible_core::provenance::evidence_graph_well_formed_spec(derived_graph@));
            let ghost after_derivation = derived_graph@;
            proof {
                crucible_core::provenance::lemma_evidence_node_equal_reflexive(derived@);
            }
            assert(crucible_core::provenance::contains_evidence_node_spec(
                after_derivation.nodes,
                derived@,
            )) by {
                assert(after_derivation.nodes[1] == derived@);
            };
            proof {
                crucible_core::provenance::lemma_provenance_edge_equal_reflexive(input_views[0]);
            }
            assert(crucible_core::provenance::contains_provenance_edge_spec(
                after_derivation.edges,
                input_views[0],
            )) by {
                assert(after_derivation.edges[0] == input_views[0]);
            };
            assert(crucible_core::provenance::derivation_inputs_valid_spec(
                after_derivation,
                derived@,
                input_views,
            ));
            match derived_graph.publish_derivation(&derived, inputs.as_slice()) {
                Ok(GraphInsertOutcome::AlreadyPresent) => {
                    assert(derived_graph@ == after_derivation);
                },
                _ => {},
            }
        },
        _ => {},
    }
}

} // verus!
