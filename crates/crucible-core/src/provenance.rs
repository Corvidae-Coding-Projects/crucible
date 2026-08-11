//! Verified append-only evidence and provenance graph primitives.
//!
//! The graph owns immutable nodes and edges behind private vectors. Publication is idempotent,
//! conflicting node identities never overwrite prior evidence, and every accepted edge names two
//! existing nodes. Pure specifications describe the representation invariant and append-only
//! transitions.
use crate::artifact::{parse_artifact_id, ArtifactIdParseError, ArtifactRefView, ContentDigest};
use crate::{ArtifactRef, EvidenceId};
#[allow(unused_imports)]
use vstd::assert_seqs_equal;
use vstd::prelude::*;

verus! {

pub const EVIDENCE_ENVELOPE_SCHEMA_VERSION: u32 = 1;

pub const NANOSECONDS_PER_SECOND: u32 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimestampError {
    NanosecondsOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A normalized UTC instant constructible only through [`UtcTimestamp::new`].
///
/// ```compile_fail
/// use crucible_core::UtcTimestamp;
///
/// let invalid = UtcTimestamp { seconds: 0, nanoseconds: 1_000_000_000 };
/// ```
pub struct UtcTimestamp {
    seconds: i64,
    nanoseconds: u32,
}

#[verifier::ext_equal]
pub struct UtcTimestampView {
    pub seconds: i64,
    pub nanoseconds: u32,
}

impl View for UtcTimestamp {
    type V = UtcTimestampView;

    closed spec fn view(&self) -> UtcTimestampView {
        UtcTimestampView { seconds: self.seconds, nanoseconds: self.nanoseconds }
    }
}

impl UtcTimestamp {
    pub fn new(seconds: i64, nanoseconds: u32) -> (result: Result<Self, TimestampError>)
        ensures
            match result {
                Ok(timestamp) => nanoseconds < NANOSECONDS_PER_SECOND && timestamp@.seconds
                    == seconds && timestamp@.nanoseconds == nanoseconds,
                Err(TimestampError::NanosecondsOutOfRange) => nanoseconds >= NANOSECONDS_PER_SECOND,
            },
    {
        if nanoseconds >= NANOSECONDS_PER_SECOND {
            Err(TimestampError::NanosecondsOutOfRange)
        } else {
            Ok(Self { seconds, nanoseconds })
        }
    }

    pub fn seconds(&self) -> (seconds: i64)
        ensures
            seconds == self@.seconds,
    {
        self.seconds
    }

    pub fn nanoseconds(&self) -> (nanoseconds: u32)
        ensures
            nanoseconds == self@.nanoseconds,
    {
        self.nanoseconds
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActorKind {
    Human,
    Coordinator,
    Engine,
    Agent,
    Worker,
    ExternalTool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ActorIdentity {
    pub kind: ActorKind,
    pub identifier: String,
}

#[verifier::ext_equal]
pub struct ActorIdentityView {
    pub kind: ActorKind,
    pub identifier: Seq<char>,
}

impl View for ActorIdentity {
    type V = ActorIdentityView;

    open spec fn view(&self) -> ActorIdentityView {
        ActorIdentityView { kind: self.kind, identifier: self.identifier@ }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SchemaIdentity {
    pub namespace: String,
    pub name: String,
    pub version: u32,
}

#[verifier::ext_equal]
pub struct SchemaIdentityView {
    pub namespace: Seq<char>,
    pub name: Seq<char>,
    pub version: u32,
}

impl View for SchemaIdentity {
    type V = SchemaIdentityView;

    open spec fn view(&self) -> SchemaIdentityView {
        SchemaIdentityView { namespace: self.namespace@, name: self.name@, version: self.version }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProducerIdentity {
    pub actor: ActorIdentity,
    pub implementation: Option<ArtifactRef>,
    pub version: String,
}

#[verifier::ext_equal]
pub struct ProducerIdentityView {
    pub actor: ActorIdentityView,
    pub implementation: Option<ArtifactRefView>,
    pub version: Seq<char>,
}

impl View for ProducerIdentity {
    type V = ProducerIdentityView;

    open spec fn view(&self) -> ProducerIdentityView {
        ProducerIdentityView {
            actor: self.actor@,
            implementation: match &self.implementation {
                Some(artifact) => Some(artifact@),
                None => None,
            },
            version: self.version@,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TransformationIdentity {
    pub name: String,
    pub version: String,
    pub implementation: ArtifactRef,
    pub configuration: TransformationConfiguration,
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransformationConfiguration {
    NoneDeclared,
    Artifact(ArtifactRef),
}

#[verifier::ext_equal]
pub enum TransformationConfigurationView {
    NoneDeclared,
    Artifact(ArtifactRefView),
}

#[verifier::ext_equal]
pub struct TransformationIdentityView {
    pub name: Seq<char>,
    pub version: Seq<char>,
    pub implementation: ArtifactRefView,
    pub configuration: TransformationConfigurationView,
}

impl View for TransformationIdentity {
    type V = TransformationIdentityView;

    open spec fn view(&self) -> TransformationIdentityView {
        TransformationIdentityView {
            name: self.name@,
            version: self.version@,
            implementation: self.implementation@,
            configuration: match &self.configuration {
                TransformationConfiguration::NoneDeclared => {
                    TransformationConfigurationView::NoneDeclared
                },
                TransformationConfiguration::Artifact(configuration) => {
                    TransformationConfigurationView::Artifact(configuration@)
                },
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceKind {
    SourceSnapshot,
    Build,
    OriginalObservation,
    DerivedObservation,
    OracleVerdict,
    Finding,
    Reproducer,
    Minimization,
    RootCauseHypothesis,
    CandidatePatch,
    VerificationResult,
    ProofResult,
    TrustedBoundaryAudit,
    Decision,
    Report,
    VersionedExtension,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EvidenceNode {
    pub id: EvidenceId,
    pub kind: EvidenceKind,
    pub payload: ArtifactRef,
    pub schema: SchemaIdentity,
    pub producer: ProducerIdentity,
    pub created_at: UtcTimestamp,
}

#[verifier::ext_equal]
pub struct EvidenceNodeView {
    pub id: Seq<char>,
    pub kind: EvidenceKind,
    pub payload: ArtifactRefView,
    pub schema: SchemaIdentityView,
    pub producer: ProducerIdentityView,
    pub created_at: UtcTimestampView,
}

impl View for EvidenceNode {
    type V = EvidenceNodeView;

    open spec fn view(&self) -> EvidenceNodeView {
        EvidenceNodeView {
            id: self.id@,
            kind: self.kind,
            payload: self.payload@,
            schema: self.schema@,
            producer: self.producer@,
            created_at: self.created_at@,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProvenanceRelation {
    DerivedFrom,
    GeneratedBy,
    Evaluates,
    Supports,
    Contradicts,
    Reproduces,
    Minimizes,
    Verifies,
    Invalidates,
    Supersedes,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProvenanceEdge {
    /// The evidence making the relation's assertion.
    pub subject: EvidenceId,
    /// The evidence the assertion is about.
    pub object: EvidenceId,
    pub relation: ProvenanceRelation,
    pub transformation: Option<TransformationIdentity>,
    pub actor: ActorIdentity,
    pub recorded_at: UtcTimestamp,
}

#[verifier::ext_equal]
pub struct ProvenanceEdgeView {
    pub subject: Seq<char>,
    pub object: Seq<char>,
    pub relation: ProvenanceRelation,
    pub transformation: Option<TransformationIdentityView>,
    pub actor: ActorIdentityView,
    pub recorded_at: UtcTimestampView,
}

impl View for ProvenanceEdge {
    type V = ProvenanceEdgeView;

    open spec fn view(&self) -> ProvenanceEdgeView {
        ProvenanceEdgeView {
            subject: self.subject@,
            object: self.object@,
            relation: self.relation,
            transformation: match &self.transformation {
                Some(transformation) => Some(transformation@),
                None => None,
            },
            actor: self.actor@,
            recorded_at: self.recorded_at@,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GraphInsertOutcome {
    Inserted,
    AlreadyPresent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceField {
    EvidenceId,
    Payload,
    PayloadMediaType,
    SchemaNamespace,
    SchemaName,
    SchemaVersion,
    ProducerActorIdentifier,
    ProducerVersion,
    ProducerImplementation,
    TransformationName,
    TransformationVersion,
    TransformationImplementation,
    TransformationConfiguration,
    EdgeActorIdentifier,
    DerivationInputs,
    DerivationSubject,
    DerivationRelation,
    DerivationTransformation,
    DerivationActor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceValidationError {
    Empty(EvidenceField),
    Zero(EvidenceField),
    MalformedArtifact(EvidenceField),
    UnsupportedArtifactAlgorithm(EvidenceField),
    Missing(EvidenceField),
    Mismatch(EvidenceField),
    Duplicate(EvidenceField),
    TimestampOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceGraphError {
    NodeConflict,
    MissingSubjectNode,
    MissingObjectNode,
    Validation(EvidenceValidationError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceEnvelopeError {
    UnsupportedSchemaVersion,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EvidenceEnvelope {
    pub schema_version: u32,
    pub node: EvidenceNode,
}

impl EvidenceEnvelope {
    pub fn new(node: EvidenceNode) -> (envelope: Self)
        ensures
            envelope.schema_version == EVIDENCE_ENVELOPE_SCHEMA_VERSION,
            envelope.node@ == node@,
    {
        Self { schema_version: EVIDENCE_ENVELOPE_SCHEMA_VERSION, node }
    }

    pub fn into_node(self) -> (result: Result<EvidenceNode, EvidenceEnvelopeError>)
        ensures
            match result {
                Ok(node) => self.schema_version == EVIDENCE_ENVELOPE_SCHEMA_VERSION && node@
                    == self.node@,
                Err(EvidenceEnvelopeError::UnsupportedSchemaVersion) => self.schema_version
                    != EVIDENCE_ENVELOPE_SCHEMA_VERSION,
            },
    {
        if self.schema_version != EVIDENCE_ENVELOPE_SCHEMA_VERSION {
            Err(EvidenceEnvelopeError::UnsupportedSchemaVersion)
        } else {
            Ok(self.node)
        }
    }
}

pub open spec fn evidence_node_views_spec(nodes: Seq<EvidenceNode>) -> Seq<EvidenceNodeView> {
    Seq::new(nodes.len(), |index: int| nodes[index]@)
}

pub open spec fn provenance_edge_views_spec(edges: Seq<ProvenanceEdge>) -> Seq<ProvenanceEdgeView> {
    Seq::new(edges.len(), |index: int| edges[index]@)
}

#[verifier::ext_equal]
pub struct EvidenceGraphView {
    pub nodes: Seq<EvidenceNodeView>,
    pub edges: Seq<ProvenanceEdgeView>,
}

#[derive(Debug)]
pub struct EvidenceGraph {
    nodes: Vec<EvidenceNode>,
    edges: Vec<ProvenanceEdge>,
}

impl View for EvidenceGraph {
    type V = EvidenceGraphView;

    closed spec fn view(&self) -> EvidenceGraphView {
        EvidenceGraphView {
            nodes: evidence_node_views_spec(self.nodes@),
            edges: provenance_edge_views_spec(self.edges@),
        }
    }
}

pub open spec fn contains_evidence_id_spec(nodes: Seq<EvidenceNodeView>, id: Seq<char>) -> bool {
    exists|index: int| 0 <= index < nodes.len() && #[trigger] nodes[index].id == id
}

pub open spec fn contains_evidence_node_spec(
    nodes: Seq<EvidenceNodeView>,
    node: EvidenceNodeView,
) -> bool {
    exists|index: int|
        0 <= index < nodes.len() && evidence_node_equal_spec(#[trigger] nodes[index], node)
}

pub open spec fn contains_conflicting_evidence_node_spec(
    nodes: Seq<EvidenceNodeView>,
    node: EvidenceNodeView,
) -> bool {
    exists|index: int|
        0 <= index < nodes.len() && #[trigger] nodes[index].id == node.id
            && !evidence_node_equal_spec(nodes[index], node)
}

pub open spec fn contains_provenance_edge_spec(
    edges: Seq<ProvenanceEdgeView>,
    edge: ProvenanceEdgeView,
) -> bool {
    exists|index: int|
        0 <= index < edges.len() && provenance_edge_equal_spec(#[trigger] edges[index], edge)
}

pub open spec fn optional_artifact_ref_equal_spec(
    left: Option<ArtifactRefView>,
    right: Option<ArtifactRefView>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => artifact_ref_equal_spec(left, right),
        (None, None) => true,
        _ => false,
    }
}

pub open spec fn transformation_configuration_equal_spec(
    left: TransformationConfigurationView,
    right: TransformationConfigurationView,
) -> bool {
    match (left, right) {
        (
            TransformationConfigurationView::NoneDeclared,
            TransformationConfigurationView::NoneDeclared,
        ) => true,
        (
            TransformationConfigurationView::Artifact(left),
            TransformationConfigurationView::Artifact(right),
        ) => artifact_ref_equal_spec(left, right),
        _ => false,
    }
}

pub open spec fn artifact_ref_equal_spec(left: ArtifactRefView, right: ArtifactRefView) -> bool {
    left.id == right.id && left.size_bytes == right.size_bytes && left.media_type
        == right.media_type
}

pub open spec fn actor_identity_equal_spec(
    left: ActorIdentityView,
    right: ActorIdentityView,
) -> bool {
    left.kind == right.kind && left.identifier == right.identifier
}

pub open spec fn schema_identity_equal_spec(
    left: SchemaIdentityView,
    right: SchemaIdentityView,
) -> bool {
    left.namespace == right.namespace && left.name == right.name && left.version == right.version
}

pub open spec fn producer_identity_equal_spec(
    left: ProducerIdentityView,
    right: ProducerIdentityView,
) -> bool {
    actor_identity_equal_spec(left.actor, right.actor) && optional_artifact_ref_equal_spec(
        left.implementation,
        right.implementation,
    ) && left.version == right.version
}

pub open spec fn transformation_identity_equal_spec(
    left: TransformationIdentityView,
    right: TransformationIdentityView,
) -> bool {
    left.name == right.name && left.version == right.version && artifact_ref_equal_spec(
        left.implementation,
        right.implementation,
    ) && transformation_configuration_equal_spec(left.configuration, right.configuration)
}

pub open spec fn optional_transformation_identity_equal_spec(
    left: Option<TransformationIdentityView>,
    right: Option<TransformationIdentityView>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => transformation_identity_equal_spec(left, right),
        (None, None) => true,
        _ => false,
    }
}

pub open spec fn evidence_node_equal_spec(left: EvidenceNodeView, right: EvidenceNodeView) -> bool {
    left.id == right.id && left.kind == right.kind && artifact_ref_equal_spec(
        left.payload,
        right.payload,
    ) && schema_identity_equal_spec(left.schema, right.schema) && producer_identity_equal_spec(
        left.producer,
        right.producer,
    ) && left.created_at.seconds == right.created_at.seconds && left.created_at.nanoseconds
        == right.created_at.nanoseconds
}

pub open spec fn provenance_edge_equal_spec(
    left: ProvenanceEdgeView,
    right: ProvenanceEdgeView,
) -> bool {
    left.subject == right.subject && left.object == right.object && left.relation == right.relation
        && optional_transformation_identity_equal_spec(left.transformation, right.transformation)
        && actor_identity_equal_spec(left.actor, right.actor) && left.recorded_at.seconds
        == right.recorded_at.seconds && left.recorded_at.nanoseconds
        == right.recorded_at.nanoseconds
}

pub proof fn lemma_artifact_ref_equal_reflexive(artifact: ArtifactRefView)
    ensures
        artifact_ref_equal_spec(artifact, artifact),
{
}

pub proof fn lemma_actor_identity_equal_reflexive(actor: ActorIdentityView)
    ensures
        actor_identity_equal_spec(actor, actor),
{
}

pub proof fn lemma_schema_identity_equal_reflexive(schema: SchemaIdentityView)
    ensures
        schema_identity_equal_spec(schema, schema),
{
}

pub proof fn lemma_optional_artifact_ref_equal_reflexive(artifact: Option<ArtifactRefView>)
    ensures
        optional_artifact_ref_equal_spec(artifact, artifact),
{
    match artifact {
        Some(artifact) => lemma_artifact_ref_equal_reflexive(artifact),
        None => {},
    }
}

pub proof fn lemma_producer_identity_equal_reflexive(producer: ProducerIdentityView)
    ensures
        producer_identity_equal_spec(producer, producer),
{
    lemma_actor_identity_equal_reflexive(producer.actor);
    lemma_optional_artifact_ref_equal_reflexive(producer.implementation);
}

pub proof fn lemma_transformation_configuration_equal_reflexive(
    configuration: TransformationConfigurationView,
)
    ensures
        transformation_configuration_equal_spec(configuration, configuration),
{
    match configuration {
        TransformationConfigurationView::NoneDeclared => {},
        TransformationConfigurationView::Artifact(artifact) => {
            lemma_artifact_ref_equal_reflexive(artifact);
        },
    }
}

pub proof fn lemma_transformation_identity_equal_reflexive(
    transformation: TransformationIdentityView,
)
    ensures
        transformation_identity_equal_spec(transformation, transformation),
{
    lemma_artifact_ref_equal_reflexive(transformation.implementation);
    lemma_transformation_configuration_equal_reflexive(transformation.configuration);
}

pub proof fn lemma_optional_transformation_identity_equal_reflexive(
    transformation: Option<TransformationIdentityView>,
)
    ensures
        optional_transformation_identity_equal_spec(transformation, transformation),
{
    match transformation {
        Some(transformation) => lemma_transformation_identity_equal_reflexive(transformation),
        None => {},
    }
}

pub proof fn lemma_evidence_node_equal_reflexive(node: EvidenceNodeView)
    ensures
        evidence_node_equal_spec(node, node),
{
    lemma_artifact_ref_equal_reflexive(node.payload);
    lemma_schema_identity_equal_reflexive(node.schema);
    lemma_producer_identity_equal_reflexive(node.producer);
}

pub proof fn lemma_provenance_edge_equal_reflexive(edge: ProvenanceEdgeView)
    ensures
        provenance_edge_equal_spec(edge, edge),
{
    lemma_optional_transformation_identity_equal_reflexive(edge.transformation);
    lemma_actor_identity_equal_reflexive(edge.actor);
}

pub open spec fn unique_evidence_node_ids_spec(nodes: Seq<EvidenceNodeView>) -> bool {
    forall|left: int, right: int|
        0 <= left < right < nodes.len() ==> #[trigger] nodes[left].id != #[trigger] nodes[right].id
}

pub open spec fn unique_provenance_edges_spec(edges: Seq<ProvenanceEdgeView>) -> bool {
    forall|left: int, right: int|
        0 <= left < right < edges.len() ==> !provenance_edge_equal_spec(
            #[trigger] edges[left],
            #[trigger] edges[right],
        )
}

pub open spec fn provenance_endpoints_present_spec(graph: EvidenceGraphView) -> bool {
    forall|index: int|
        0 <= index < graph.edges.len() ==> contains_evidence_id_spec(
            graph.nodes,
            #[trigger] graph.edges[index].subject,
        ) && contains_evidence_id_spec(graph.nodes, graph.edges[index].object)
}

pub open spec fn timestamp_valid_spec(timestamp: UtcTimestampView) -> bool {
    timestamp.nanoseconds < NANOSECONDS_PER_SECOND
}

pub open spec fn artifact_ref_structurally_valid_spec(artifact: ArtifactRefView) -> bool {
    crate::artifact::canonical_sha256_artifact_id_spec(artifact.id) && match artifact.media_type {
        Some(media_type) => media_type.len() > 0,
        None => true,
    }
}

pub open spec fn optional_artifact_ref_structurally_valid_spec(
    artifact: Option<ArtifactRefView>,
) -> bool {
    match artifact {
        Some(artifact) => artifact_ref_structurally_valid_spec(artifact),
        None => true,
    }
}

pub open spec fn actor_identity_valid_spec(actor: ActorIdentityView) -> bool {
    actor.identifier.len() > 0
}

pub open spec fn schema_identity_valid_spec(schema: SchemaIdentityView) -> bool {
    schema.namespace.len() > 0 && schema.name.len() > 0 && schema.version > 0
}

pub open spec fn producer_identity_valid_spec(producer: ProducerIdentityView) -> bool {
    actor_identity_valid_spec(producer.actor) && producer.version.len() > 0
        && optional_artifact_ref_structurally_valid_spec(producer.implementation)
}

pub open spec fn transformation_configuration_valid_spec(
    configuration: TransformationConfigurationView,
) -> bool {
    match configuration {
        TransformationConfigurationView::NoneDeclared => true,
        TransformationConfigurationView::Artifact(artifact) => {
            artifact_ref_structurally_valid_spec(artifact)
        },
    }
}

pub open spec fn transformation_identity_valid_spec(
    transformation: TransformationIdentityView,
) -> bool {
    transformation.name.len() > 0 && transformation.version.len() > 0
        && artifact_ref_structurally_valid_spec(transformation.implementation)
        && transformation_configuration_valid_spec(transformation.configuration)
}

pub open spec fn optional_transformation_identity_valid_spec(
    transformation: Option<TransformationIdentityView>,
) -> bool {
    match transformation {
        Some(transformation) => transformation_identity_valid_spec(transformation),
        None => true,
    }
}

pub open spec fn evidence_kind_requires_derivation_spec(kind: EvidenceKind) -> bool {
    match kind {
        EvidenceKind::SourceSnapshot | EvidenceKind::OriginalObservation => false,
        EvidenceKind::Build
        | EvidenceKind::DerivedObservation
        | EvidenceKind::OracleVerdict
        | EvidenceKind::Finding
        | EvidenceKind::Reproducer
        | EvidenceKind::Minimization
        | EvidenceKind::RootCauseHypothesis
        | EvidenceKind::CandidatePatch
        | EvidenceKind::VerificationResult
        | EvidenceKind::ProofResult
        | EvidenceKind::TrustedBoundaryAudit
        | EvidenceKind::Decision
        | EvidenceKind::Report
        | EvidenceKind::VersionedExtension => true,
    }
}

pub open spec fn evidence_node_structurally_valid_spec(node: EvidenceNodeView) -> bool {
    node.id.len() > 0 && artifact_ref_structurally_valid_spec(node.payload)
        && schema_identity_valid_spec(node.schema) && producer_identity_valid_spec(node.producer)
        && timestamp_valid_spec(node.created_at) && (evidence_kind_requires_derivation_spec(
        node.kind,
    ) ==> node.producer.implementation is Some)
}

pub open spec fn provenance_edge_structurally_valid_spec(edge: ProvenanceEdgeView) -> bool {
    edge.subject.len() > 0 && edge.object.len() > 0 && actor_identity_valid_spec(edge.actor)
        && timestamp_valid_spec(edge.recorded_at) && optional_transformation_identity_valid_spec(
        edge.transformation,
    )
}

pub open spec fn all_evidence_nodes_structurally_valid_spec(nodes: Seq<EvidenceNodeView>) -> bool {
    forall|index: int|
        0 <= index < nodes.len() ==> evidence_node_structurally_valid_spec(#[trigger] nodes[index])
}

pub open spec fn all_provenance_edges_structurally_valid_spec(
    edges: Seq<ProvenanceEdgeView>,
) -> bool {
    forall|index: int|
        0 <= index < edges.len() ==> provenance_edge_structurally_valid_spec(
            #[trigger] edges[index],
        )
}

pub open spec fn node_has_complete_derivation_spec(
    graph: EvidenceGraphView,
    node: EvidenceNodeView,
) -> bool {
    !evidence_kind_requires_derivation_spec(node.kind) || exists|index: int|
        0 <= index < graph.edges.len() && #[trigger] graph.edges[index].subject == node.id
            && graph.edges[index].relation == ProvenanceRelation::DerivedFrom
            && graph.edges[index].transformation is Some && actor_identity_equal_spec(
            graph.edges[index].actor,
            node.producer.actor,
        )
}

pub open spec fn all_derivations_complete_spec(graph: EvidenceGraphView) -> bool {
    forall|index: int|
        0 <= index < graph.nodes.len() ==> node_has_complete_derivation_spec(
            graph,
            #[trigger] graph.nodes[index],
        )
}

pub open spec fn derivation_inputs_valid_spec(
    graph: EvidenceGraphView,
    node: EvidenceNodeView,
    inputs: Seq<ProvenanceEdgeView>,
) -> bool {
    evidence_node_structurally_valid_spec(node) && evidence_kind_requires_derivation_spec(node.kind)
        && inputs.len() > 0 && forall|index: int|
        0 <= index < inputs.len() ==> {
            &&& provenance_edge_structurally_valid_spec(#[trigger] inputs[index])
            &&& inputs[index].subject == node.id
            &&& inputs[index].relation == ProvenanceRelation::DerivedFrom
            &&& inputs[index].transformation is Some
            &&& actor_identity_equal_spec(inputs[index].actor, node.producer.actor)
            &&& contains_evidence_id_spec(graph.nodes, inputs[index].object)
        } && forall|left: int, right: int|
            0 <= left < right < inputs.len() ==> !provenance_edge_equal_spec(
                #[trigger] inputs[left],
                #[trigger] inputs[right],
            )
}

pub open spec fn evidence_graph_well_formed_spec(graph: EvidenceGraphView) -> bool {
    unique_evidence_node_ids_spec(graph.nodes) && unique_provenance_edges_spec(graph.edges)
        && provenance_endpoints_present_spec(graph) && all_evidence_nodes_structurally_valid_spec(
        graph.nodes,
    ) && all_provenance_edges_structurally_valid_spec(graph.edges) && all_derivations_complete_spec(
        graph,
    )
}

// Owned-string references preserve vstd's exact clone/equality specifications;
// converting to `&str` would weaken the proof surface for this comparison.
#[allow(clippy::ptr_arg)]
fn same_string(left: &String, right: &String) -> (same: bool)
    ensures
        same == (left@ == right@),
{
    let left_owned = left.clone();
    let right_owned = right.clone();
    left_owned == right_owned
}

fn same_evidence_id(left: &EvidenceId, right: &EvidenceId) -> (same: bool)
    ensures
        same == (left@ == right@),
{
    same_string(&left.0, &right.0)
}

pub open spec fn actor_kind_tag_spec(kind: ActorKind) -> u8 {
    match kind {
        ActorKind::Human => 0,
        ActorKind::Coordinator => 1,
        ActorKind::Engine => 2,
        ActorKind::Agent => 3,
        ActorKind::Worker => 4,
        ActorKind::ExternalTool => 5,
    }
}

fn actor_kind_tag(kind: ActorKind) -> (tag: u8)
    ensures
        tag == actor_kind_tag_spec(kind),
{
    match kind {
        ActorKind::Human => 0,
        ActorKind::Coordinator => 1,
        ActorKind::Engine => 2,
        ActorKind::Agent => 3,
        ActorKind::Worker => 4,
        ActorKind::ExternalTool => 5,
    }
}

fn same_actor_kind(left: ActorKind, right: ActorKind) -> (same: bool)
    ensures
        same == (left == right),
{
    actor_kind_tag(left) == actor_kind_tag(right)
}

pub open spec fn evidence_kind_tag_spec(kind: EvidenceKind) -> u8 {
    match kind {
        EvidenceKind::SourceSnapshot => 0,
        EvidenceKind::Build => 1,
        EvidenceKind::OriginalObservation => 2,
        EvidenceKind::DerivedObservation => 3,
        EvidenceKind::OracleVerdict => 4,
        EvidenceKind::Finding => 5,
        EvidenceKind::Reproducer => 6,
        EvidenceKind::Minimization => 7,
        EvidenceKind::RootCauseHypothesis => 8,
        EvidenceKind::CandidatePatch => 9,
        EvidenceKind::VerificationResult => 10,
        EvidenceKind::ProofResult => 11,
        EvidenceKind::TrustedBoundaryAudit => 12,
        EvidenceKind::Decision => 13,
        EvidenceKind::Report => 14,
        EvidenceKind::VersionedExtension => 15,
    }
}

fn evidence_kind_tag(kind: EvidenceKind) -> (tag: u8)
    ensures
        tag == evidence_kind_tag_spec(kind),
{
    match kind {
        EvidenceKind::SourceSnapshot => 0,
        EvidenceKind::Build => 1,
        EvidenceKind::OriginalObservation => 2,
        EvidenceKind::DerivedObservation => 3,
        EvidenceKind::OracleVerdict => 4,
        EvidenceKind::Finding => 5,
        EvidenceKind::Reproducer => 6,
        EvidenceKind::Minimization => 7,
        EvidenceKind::RootCauseHypothesis => 8,
        EvidenceKind::CandidatePatch => 9,
        EvidenceKind::VerificationResult => 10,
        EvidenceKind::ProofResult => 11,
        EvidenceKind::TrustedBoundaryAudit => 12,
        EvidenceKind::Decision => 13,
        EvidenceKind::Report => 14,
        EvidenceKind::VersionedExtension => 15,
    }
}

fn same_evidence_kind(left: EvidenceKind, right: EvidenceKind) -> (same: bool)
    ensures
        same == (left == right),
{
    evidence_kind_tag(left) == evidence_kind_tag(right)
}

pub open spec fn provenance_relation_tag_spec(relation: ProvenanceRelation) -> u8 {
    match relation {
        ProvenanceRelation::DerivedFrom => 0,
        ProvenanceRelation::GeneratedBy => 1,
        ProvenanceRelation::Evaluates => 2,
        ProvenanceRelation::Supports => 3,
        ProvenanceRelation::Contradicts => 4,
        ProvenanceRelation::Reproduces => 5,
        ProvenanceRelation::Minimizes => 6,
        ProvenanceRelation::Verifies => 7,
        ProvenanceRelation::Invalidates => 8,
        ProvenanceRelation::Supersedes => 9,
    }
}

fn provenance_relation_tag(relation: ProvenanceRelation) -> (tag: u8)
    ensures
        tag == provenance_relation_tag_spec(relation),
{
    match relation {
        ProvenanceRelation::DerivedFrom => 0,
        ProvenanceRelation::GeneratedBy => 1,
        ProvenanceRelation::Evaluates => 2,
        ProvenanceRelation::Supports => 3,
        ProvenanceRelation::Contradicts => 4,
        ProvenanceRelation::Reproduces => 5,
        ProvenanceRelation::Minimizes => 6,
        ProvenanceRelation::Verifies => 7,
        ProvenanceRelation::Invalidates => 8,
        ProvenanceRelation::Supersedes => 9,
    }
}

fn same_provenance_relation(left: ProvenanceRelation, right: ProvenanceRelation) -> (same: bool)
    ensures
        same == (left == right),
{
    provenance_relation_tag(left) == provenance_relation_tag(right)
}

fn same_optional_string(left: &Option<String>, right: &Option<String>) -> (same: bool)
    ensures
        same == match (left, right) {
            (Some(left), Some(right)) => left@ == right@,
            (None, None) => true,
            _ => false,
        },
{
    match (left, right) {
        (Some(left), Some(right)) => same_string(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_artifact_ref(left: &ArtifactRef, right: &ArtifactRef) -> (same: bool)
    ensures
        same == artifact_ref_equal_spec(left@, right@),
{
    same_string(&left.id.0, &right.id.0) && left.size_bytes == right.size_bytes
        && same_optional_string(&left.media_type, &right.media_type)
}

fn same_optional_artifact_ref(left: &Option<ArtifactRef>, right: &Option<ArtifactRef>) -> (same:
    bool)
    ensures
        same == optional_artifact_ref_equal_spec(
            match left {
                Some(left) => Some(left@),
                None => None,
            },
            match right {
                Some(right) => Some(right@),
                None => None,
            },
        ),
{
    match (left, right) {
        (Some(left), Some(right)) => same_artifact_ref(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_transformation_configuration(
    left: &TransformationConfiguration,
    right: &TransformationConfiguration,
) -> (same: bool)
    ensures
        same == transformation_configuration_equal_spec(
            match left {
                TransformationConfiguration::NoneDeclared => {
                    TransformationConfigurationView::NoneDeclared
                },
                TransformationConfiguration::Artifact(artifact) => {
                    TransformationConfigurationView::Artifact(artifact@)
                },
            },
            match right {
                TransformationConfiguration::NoneDeclared => {
                    TransformationConfigurationView::NoneDeclared
                },
                TransformationConfiguration::Artifact(artifact) => {
                    TransformationConfigurationView::Artifact(artifact@)
                },
            },
        ),
{
    match (left, right) {
        (
            TransformationConfiguration::NoneDeclared,
            TransformationConfiguration::NoneDeclared,
        ) => true,
        (
            TransformationConfiguration::Artifact(left),
            TransformationConfiguration::Artifact(right),
        ) => same_artifact_ref(left, right),
        _ => false,
    }
}

fn same_actor_identity(left: &ActorIdentity, right: &ActorIdentity) -> (same: bool)
    ensures
        same == actor_identity_equal_spec(left@, right@),
{
    same_actor_kind(left.kind, right.kind) && same_string(&left.identifier, &right.identifier)
}

fn same_schema_identity(left: &SchemaIdentity, right: &SchemaIdentity) -> (same: bool)
    ensures
        same == schema_identity_equal_spec(left@, right@),
{
    same_string(&left.namespace, &right.namespace) && same_string(&left.name, &right.name)
        && left.version == right.version
}

fn same_producer_identity(left: &ProducerIdentity, right: &ProducerIdentity) -> (same: bool)
    ensures
        same == producer_identity_equal_spec(left@, right@),
{
    same_actor_identity(&left.actor, &right.actor) && same_optional_artifact_ref(
        &left.implementation,
        &right.implementation,
    ) && same_string(&left.version, &right.version)
}

fn same_transformation_identity(
    left: &TransformationIdentity,
    right: &TransformationIdentity,
) -> (same: bool)
    ensures
        same == transformation_identity_equal_spec(left@, right@),
{
    same_string(&left.name, &right.name) && same_string(&left.version, &right.version)
        && same_artifact_ref(&left.implementation, &right.implementation)
        && same_transformation_configuration(&left.configuration, &right.configuration)
}

fn same_optional_transformation_identity(
    left: &Option<TransformationIdentity>,
    right: &Option<TransformationIdentity>,
) -> (same: bool)
    ensures
        same == optional_transformation_identity_equal_spec(
            match left {
                Some(left) => Some(left@),
                None => None,
            },
            match right {
                Some(right) => Some(right@),
                None => None,
            },
        ),
{
    match (left, right) {
        (Some(left), Some(right)) => same_transformation_identity(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_evidence_node(left: &EvidenceNode, right: &EvidenceNode) -> (same: bool)
    ensures
        same == evidence_node_equal_spec(left@, right@),
{
    same_evidence_id(&left.id, &right.id) && same_evidence_kind(left.kind, right.kind)
        && same_artifact_ref(&left.payload, &right.payload) && same_schema_identity(
        &left.schema,
        &right.schema,
    ) && same_producer_identity(&left.producer, &right.producer) && left.created_at.seconds
        == right.created_at.seconds && left.created_at.nanoseconds == right.created_at.nanoseconds
}

fn same_provenance_edge(left: &ProvenanceEdge, right: &ProvenanceEdge) -> (same: bool)
    ensures
        same == provenance_edge_equal_spec(left@, right@),
{
    same_evidence_id(&left.subject, &right.subject) && same_evidence_id(&left.object, &right.object)
        && same_provenance_relation(left.relation, right.relation)
        && same_optional_transformation_identity(&left.transformation, &right.transformation)
        && same_actor_identity(&left.actor, &right.actor) && left.recorded_at.seconds
        == right.recorded_at.seconds && left.recorded_at.nanoseconds
        == right.recorded_at.nanoseconds
}

fn string_is_empty(value: &str) -> (empty: bool)
    ensures
        empty == (value@.len() == 0),
{
    value.unicode_len() == 0
}

fn validate_artifact_ref(
    artifact: &ArtifactRef,
    artifact_field: EvidenceField,
    media_type_field: EvidenceField,
) -> (result: Result<(), EvidenceValidationError>)
    ensures
        result is Ok == artifact_ref_structurally_valid_spec(artifact@),
{
    match parse_artifact_id(&artifact.id) {
        Ok(ContentDigest::Sha256(_digest)) => proof {
            crate::artifact::lemma_artifact_id_spec_is_canonical(_digest@);
        },
        Err(ArtifactIdParseError::MalformedArtifactId) => {
            return Err(EvidenceValidationError::MalformedArtifact(artifact_field));
        },
        Err(ArtifactIdParseError::UnsupportedAlgorithm) => {
            return Err(EvidenceValidationError::UnsupportedArtifactAlgorithm(artifact_field));
        },
    }
    if let Some(media_type) = &artifact.media_type {
        if string_is_empty(media_type) {
            return Err(EvidenceValidationError::Empty(media_type_field));
        }
    }
    Ok(())
}

fn validate_actor_identity(actor: &ActorIdentity, identifier_field: EvidenceField) -> (result:
    Result<(), EvidenceValidationError>)
    ensures
        result is Ok == actor_identity_valid_spec(actor@),
{
    if string_is_empty(&actor.identifier) {
        Err(EvidenceValidationError::Empty(identifier_field))
    } else {
        Ok(())
    }
}

// Explicit typed-error returns keep each rejected field visible to Verus's branch contracts.
#[allow(clippy::question_mark)]
fn validate_transformation_identity(transformation: &TransformationIdentity) -> (result: Result<
    (),
    EvidenceValidationError,
>)
    ensures
        result is Ok == transformation_identity_valid_spec(transformation@),
{
    if string_is_empty(&transformation.name) {
        return Err(EvidenceValidationError::Empty(EvidenceField::TransformationName));
    }
    if string_is_empty(&transformation.version) {
        return Err(EvidenceValidationError::Empty(EvidenceField::TransformationVersion));
    }
    if let Err(error) = validate_artifact_ref(
        &transformation.implementation,
        EvidenceField::TransformationImplementation,
        EvidenceField::PayloadMediaType,
    ) {
        return Err(error);
    }
    match &transformation.configuration {
        TransformationConfiguration::NoneDeclared => {},
        TransformationConfiguration::Artifact(configuration) => {
            if let Err(error) = validate_artifact_ref(
                configuration,
                EvidenceField::TransformationConfiguration,
                EvidenceField::PayloadMediaType,
            ) {
                return Err(error);
            }
        },
    }
    Ok(())
}

fn evidence_kind_requires_derivation(kind: EvidenceKind) -> (requires_derivation: bool)
    ensures
        requires_derivation == evidence_kind_requires_derivation_spec(kind),
{
    match kind {
        EvidenceKind::SourceSnapshot | EvidenceKind::OriginalObservation => false,
        EvidenceKind::Build
        | EvidenceKind::DerivedObservation
        | EvidenceKind::OracleVerdict
        | EvidenceKind::Finding
        | EvidenceKind::Reproducer
        | EvidenceKind::Minimization
        | EvidenceKind::RootCauseHypothesis
        | EvidenceKind::CandidatePatch
        | EvidenceKind::VerificationResult
        | EvidenceKind::ProofResult
        | EvidenceKind::TrustedBoundaryAudit
        | EvidenceKind::Decision
        | EvidenceKind::Report
        | EvidenceKind::VersionedExtension => true,
    }
}

#[allow(clippy::question_mark)]
fn validate_evidence_node(node: &EvidenceNode) -> (result: Result<(), EvidenceValidationError>)
    ensures
        result is Ok == evidence_node_structurally_valid_spec(node@),
{
    if string_is_empty(&node.id.0) {
        return Err(EvidenceValidationError::Empty(EvidenceField::EvidenceId));
    }
    if let Err(error) = validate_artifact_ref(
        &node.payload,
        EvidenceField::Payload,
        EvidenceField::PayloadMediaType,
    ) {
        return Err(error);
    }
    if string_is_empty(&node.schema.namespace) {
        return Err(EvidenceValidationError::Empty(EvidenceField::SchemaNamespace));
    }
    if string_is_empty(&node.schema.name) {
        return Err(EvidenceValidationError::Empty(EvidenceField::SchemaName));
    }
    if node.schema.version == 0 {
        return Err(EvidenceValidationError::Zero(EvidenceField::SchemaVersion));
    }
    if let Err(error) = validate_actor_identity(
        &node.producer.actor,
        EvidenceField::ProducerActorIdentifier,
    ) {
        return Err(error);
    }
    if string_is_empty(&node.producer.version) {
        return Err(EvidenceValidationError::Empty(EvidenceField::ProducerVersion));
    }
    match &node.producer.implementation {
        Some(implementation) => {
            if let Err(error) = validate_artifact_ref(
                implementation,
                EvidenceField::ProducerImplementation,
                EvidenceField::PayloadMediaType,
            ) {
                return Err(error);
            }
        },
        None => {
            if evidence_kind_requires_derivation(node.kind) {
                return Err(EvidenceValidationError::Missing(EvidenceField::ProducerImplementation));
            }
        },
    }
    if node.created_at.nanoseconds >= NANOSECONDS_PER_SECOND {
        return Err(EvidenceValidationError::TimestampOutOfRange);
    }
    Ok(())
}

#[allow(clippy::question_mark)]
fn validate_provenance_edge(edge: &ProvenanceEdge) -> (result: Result<(), EvidenceValidationError>)
    ensures
        result is Ok == provenance_edge_structurally_valid_spec(edge@),
{
    if string_is_empty(&edge.subject.0) {
        return Err(EvidenceValidationError::Empty(EvidenceField::DerivationSubject));
    }
    if string_is_empty(&edge.object.0) {
        return Err(EvidenceValidationError::Empty(EvidenceField::DerivationInputs));
    }
    if let Err(error) = validate_actor_identity(&edge.actor, EvidenceField::EdgeActorIdentifier) {
        return Err(error);
    }
    if edge.recorded_at.nanoseconds >= NANOSECONDS_PER_SECOND {
        return Err(EvidenceValidationError::TimestampOutOfRange);
    }
    match &edge.transformation {
        Some(transformation) => validate_transformation_identity(transformation),
        None => Ok(()),
    }
}

// Explicit matches retain exact per-variant view postconditions without closure proof state.
#[allow(clippy::manual_map)]
fn clone_artifact_ref(artifact: &ArtifactRef) -> (cloned: ArtifactRef)
    ensures
        cloned@ == artifact@,
{
    ArtifactRef {
        id: artifact.id.clone(),
        size_bytes: artifact.size_bytes,
        media_type: match &artifact.media_type {
            Some(media_type) => Some(media_type.clone()),
            None => None,
        },
    }
}

fn clone_actor_identity(actor: &ActorIdentity) -> (cloned: ActorIdentity)
    ensures
        cloned@ == actor@,
{
    ActorIdentity { kind: actor.kind, identifier: actor.identifier.clone() }
}

fn clone_schema_identity(schema: &SchemaIdentity) -> (cloned: SchemaIdentity)
    ensures
        cloned@ == schema@,
{
    SchemaIdentity {
        namespace: schema.namespace.clone(),
        name: schema.name.clone(),
        version: schema.version,
    }
}

#[allow(clippy::manual_map)]
fn clone_optional_artifact_ref(artifact: &Option<ArtifactRef>) -> (cloned: Option<ArtifactRef>)
    ensures
        match (&cloned, artifact) {
            (Some(cloned), Some(artifact)) => cloned@ == artifact@,
            (None, None) => true,
            _ => false,
        },
{
    match artifact {
        Some(artifact) => Some(clone_artifact_ref(artifact)),
        None => None,
    }
}

fn clone_producer_identity(producer: &ProducerIdentity) -> (cloned: ProducerIdentity)
    ensures
        cloned@ == producer@,
{
    ProducerIdentity {
        actor: clone_actor_identity(&producer.actor),
        implementation: clone_optional_artifact_ref(&producer.implementation),
        version: producer.version.clone(),
    }
}

fn clone_transformation_configuration(configuration: &TransformationConfiguration) -> (cloned:
    TransformationConfiguration)
    ensures
        match (&cloned, configuration) {
            (
                TransformationConfiguration::NoneDeclared,
                TransformationConfiguration::NoneDeclared,
            ) => true,
            (
                TransformationConfiguration::Artifact(cloned),
                TransformationConfiguration::Artifact(configuration),
            ) => cloned@ == configuration@,
            _ => false,
        },
{
    match configuration {
        TransformationConfiguration::NoneDeclared => { TransformationConfiguration::NoneDeclared },
        TransformationConfiguration::Artifact(artifact) => {
            TransformationConfiguration::Artifact(clone_artifact_ref(artifact))
        },
    }
}

fn clone_transformation_identity(transformation: &TransformationIdentity) -> (cloned:
    TransformationIdentity)
    ensures
        cloned@ == transformation@,
{
    TransformationIdentity {
        name: transformation.name.clone(),
        version: transformation.version.clone(),
        implementation: clone_artifact_ref(&transformation.implementation),
        configuration: clone_transformation_configuration(&transformation.configuration),
    }
}

#[allow(clippy::manual_map)]
fn clone_optional_transformation_identity(
    transformation: &Option<TransformationIdentity>,
) -> (cloned: Option<TransformationIdentity>)
    ensures
        match (&cloned, transformation) {
            (Some(cloned), Some(transformation)) => cloned@ == transformation@,
            (None, None) => true,
            _ => false,
        },
{
    match transformation {
        Some(transformation) => Some(clone_transformation_identity(transformation)),
        None => None,
    }
}

fn clone_evidence_node(node: &EvidenceNode) -> (cloned: EvidenceNode)
    ensures
        cloned@ == node@,
{
    EvidenceNode {
        id: node.id.clone(),
        kind: node.kind,
        payload: clone_artifact_ref(&node.payload),
        schema: clone_schema_identity(&node.schema),
        producer: clone_producer_identity(&node.producer),
        created_at: node.created_at,
    }
}

fn clone_provenance_edge(edge: &ProvenanceEdge) -> (cloned: ProvenanceEdge)
    ensures
        cloned@ == edge@,
{
    ProvenanceEdge {
        subject: edge.subject.clone(),
        object: edge.object.clone(),
        relation: edge.relation,
        transformation: clone_optional_transformation_identity(&edge.transformation),
        actor: clone_actor_identity(&edge.actor),
        recorded_at: edge.recorded_at,
    }
}

fn contains_evidence_id(nodes: &[EvidenceNode], id: &EvidenceId) -> (contains: bool)
    ensures
        contains == contains_evidence_id_spec(evidence_node_views_spec(nodes@), id@),
{
    let mut index = 0;
    while index < nodes.len()
        invariant
            index <= nodes@.len(),
            forall|prior: int| 0 <= prior < index ==> #[trigger] nodes@[prior]@.id != id@,
        decreases nodes.len() - index,
    {
        if same_evidence_id(&nodes[index].id, id) {
            assert(evidence_node_views_spec(nodes@)[index as int] == nodes@[index as int]@);
            assert(0 <= index < evidence_node_views_spec(nodes@).len());
            assert(evidence_node_views_spec(nodes@)[index as int].id == id@);
            assert(contains_evidence_id_spec(evidence_node_views_spec(nodes@), id@));
            return true;
        }
        index += 1;
    }
    assert(!contains_evidence_id_spec(evidence_node_views_spec(nodes@), id@)) by {
        if contains_evidence_id_spec(evidence_node_views_spec(nodes@), id@) {
            let witness = choose|witness: int|
                #![trigger evidence_node_views_spec(nodes@)[witness]]
                0 <= witness < evidence_node_views_spec(nodes@).len() && evidence_node_views_spec(
                    nodes@,
                )[witness].id == id@;
            assert(evidence_node_views_spec(nodes@)[witness] == nodes@[witness]@);
        }
    };
    false
}

fn contains_provenance_edge(edges: &[ProvenanceEdge], edge: &ProvenanceEdge) -> (contains: bool)
    ensures
        contains == contains_provenance_edge_spec(provenance_edge_views_spec(edges@), edge@),
{
    let mut index = 0;
    while index < edges.len()
        invariant
            index <= edges@.len(),
            forall|prior: int|
                0 <= prior < index ==> !provenance_edge_equal_spec(
                    #[trigger] edges@[prior]@,
                    edge@,
                ),
        decreases edges.len() - index,
    {
        if same_provenance_edge(&edges[index], edge) {
            assert(provenance_edge_views_spec(edges@)[index as int] == edges@[index as int]@);
            assert(contains_provenance_edge_spec(provenance_edge_views_spec(edges@), edge@));
            return true;
        }
        index += 1;
    }
    assert(!contains_provenance_edge_spec(provenance_edge_views_spec(edges@), edge@)) by {
        if contains_provenance_edge_spec(provenance_edge_views_spec(edges@), edge@) {
            let witness = choose|witness: int|
                #![trigger provenance_edge_views_spec(edges@)[witness]]
                0 <= witness < provenance_edge_views_spec(edges@).len()
                    && provenance_edge_equal_spec(
                    provenance_edge_views_spec(edges@)[witness],
                    edge@,
                );
            assert(provenance_edge_views_spec(edges@)[witness] == edges@[witness]@);
        }
    };
    false
}

proof fn lemma_append_node_preserves_well_formed(graph: EvidenceGraphView, node: EvidenceNodeView)
    requires
        evidence_graph_well_formed_spec(graph),
        !contains_evidence_id_spec(graph.nodes, node.id),
        evidence_node_structurally_valid_spec(node),
        !evidence_kind_requires_derivation_spec(node.kind),
    ensures
        evidence_graph_well_formed_spec(
            EvidenceGraphView { nodes: graph.nodes.push(node), edges: graph.edges },
        ),
{
    assert(unique_evidence_node_ids_spec(graph.nodes.push(node))) by {
        assert forall|left: int, right: int|
            0 <= left < right < graph.nodes.push(node).len() implies #[trigger] graph.nodes.push(
            node,
        )[left].id != #[trigger] graph.nodes.push(node)[right].id by {
            if right < graph.nodes.len() {
                assert(graph.nodes.push(node)[left] == graph.nodes[left]);
                assert(graph.nodes.push(node)[right] == graph.nodes[right]);
            } else {
                assert(right == graph.nodes.len());
                assert(graph.nodes.push(node)[left] == graph.nodes[left]);
                assert(graph.nodes.push(node)[right] == node);
            }
        };
    };
    assert(provenance_endpoints_present_spec(
        EvidenceGraphView { nodes: graph.nodes.push(node), edges: graph.edges },
    )) by {
        assert forall|index: int| 0 <= index < graph.edges.len() implies contains_evidence_id_spec(
            graph.nodes.push(node),
            #[trigger] graph.edges[index].subject,
        ) && contains_evidence_id_spec(graph.nodes.push(node), graph.edges[index].object) by {
            let from_index = choose|candidate: int|
                #![trigger graph.nodes[candidate]]
                0 <= candidate < graph.nodes.len() && graph.nodes[candidate].id
                    == graph.edges[index].subject;
            let to_index = choose|candidate: int|
                #![trigger graph.nodes[candidate]]
                0 <= candidate < graph.nodes.len() && graph.nodes[candidate].id
                    == graph.edges[index].object;
            assert(graph.nodes.push(node)[from_index] == graph.nodes[from_index]);
            assert(graph.nodes.push(node)[to_index] == graph.nodes[to_index]);
        };
    };
    assert(all_evidence_nodes_structurally_valid_spec(graph.nodes.push(node))) by {
        assert forall|index: int|
            0 <= index < graph.nodes.push(node).len() implies evidence_node_structurally_valid_spec(
            #[trigger] graph.nodes.push(node)[index],
        ) by {
            if index < graph.nodes.len() {
                assert(graph.nodes.push(node)[index] == graph.nodes[index]);
            } else {
                assert(index == graph.nodes.len());
                assert(graph.nodes.push(node)[index] == node);
            }
        };
    };
    assert(all_derivations_complete_spec(
        EvidenceGraphView { nodes: graph.nodes.push(node), edges: graph.edges },
    )) by {
        assert forall|index: int|
            0 <= index < graph.nodes.push(node).len() implies node_has_complete_derivation_spec(
            EvidenceGraphView { nodes: graph.nodes.push(node), edges: graph.edges },
            #[trigger] graph.nodes.push(node)[index],
        ) by {
            if index < graph.nodes.len() {
                assert(graph.nodes.push(node)[index] == graph.nodes[index]);
            } else {
                assert(index == graph.nodes.len());
                assert(graph.nodes.push(node)[index] == node);
            }
        };
    };
}

proof fn lemma_append_edge_preserves_well_formed(graph: EvidenceGraphView, edge: ProvenanceEdgeView)
    requires
        evidence_graph_well_formed_spec(graph),
        contains_evidence_id_spec(graph.nodes, edge.subject),
        contains_evidence_id_spec(graph.nodes, edge.object),
        !contains_provenance_edge_spec(graph.edges, edge),
        provenance_edge_structurally_valid_spec(edge),
    ensures
        evidence_graph_well_formed_spec(
            EvidenceGraphView { nodes: graph.nodes, edges: graph.edges.push(edge) },
        ),
{
    assert(unique_provenance_edges_spec(graph.edges.push(edge))) by {
        assert forall|left: int, right: int|
            0 <= left < right < graph.edges.push(edge).len() implies !provenance_edge_equal_spec(
            #[trigger] graph.edges.push(edge)[left],
            #[trigger] graph.edges.push(edge)[right],
        ) by {
            if right < graph.edges.len() {
                assert(graph.edges.push(edge)[left] == graph.edges[left]);
                assert(graph.edges.push(edge)[right] == graph.edges[right]);
            } else {
                assert(right == graph.edges.len());
                assert(graph.edges.push(edge)[left] == graph.edges[left]);
                assert(graph.edges.push(edge)[right] == edge);
            }
        };
    };
    assert(provenance_endpoints_present_spec(
        EvidenceGraphView { nodes: graph.nodes, edges: graph.edges.push(edge) },
    )) by {
        assert forall|index: int|
            0 <= index < graph.edges.push(edge).len() implies contains_evidence_id_spec(
            graph.nodes,
            #[trigger] graph.edges.push(edge)[index].subject,
        ) && contains_evidence_id_spec(graph.nodes, graph.edges.push(edge)[index].object) by {
            if index < graph.edges.len() {
                assert(graph.edges.push(edge)[index] == graph.edges[index]);
            } else {
                assert(index == graph.edges.len());
                assert(graph.edges.push(edge)[index] == edge);
            }
        };
    };
    assert(all_provenance_edges_structurally_valid_spec(graph.edges.push(edge))) by {
        assert forall|index: int|
            0 <= index < graph.edges.push(
                edge,
            ).len() implies provenance_edge_structurally_valid_spec(
            #[trigger] graph.edges.push(edge)[index],
        ) by {
            if index < graph.edges.len() {
                assert(graph.edges.push(edge)[index] == graph.edges[index]);
            } else {
                assert(index == graph.edges.len());
                assert(graph.edges.push(edge)[index] == edge);
            }
        };
    };
    assert(all_derivations_complete_spec(
        EvidenceGraphView { nodes: graph.nodes, edges: graph.edges.push(edge) },
    )) by {
        assert forall|index: int|
            0 <= index < graph.nodes.len() implies node_has_complete_derivation_spec(
            EvidenceGraphView { nodes: graph.nodes, edges: graph.edges.push(edge) },
            #[trigger] graph.nodes[index],
        ) by {
            if evidence_kind_requires_derivation_spec(graph.nodes[index].kind) {
                let witness = choose|witness: int|
                    #![trigger graph.edges[witness]]
                    0 <= witness < graph.edges.len() && graph.edges[witness].subject
                        == graph.nodes[index].id && graph.edges[witness].relation
                        == ProvenanceRelation::DerivedFrom
                        && graph.edges[witness].transformation is Some && actor_identity_equal_spec(
                        graph.edges[witness].actor,
                        graph.nodes[index].producer.actor,
                    );
                assert(graph.edges.push(edge)[witness] == graph.edges[witness]);
            }
        };
    };
}

#[verifier::rlimit(30)]
proof fn lemma_publish_derivation_preserves_well_formed(
    graph: EvidenceGraphView,
    node: EvidenceNodeView,
    inputs: Seq<ProvenanceEdgeView>,
)
    requires
        evidence_graph_well_formed_spec(graph),
        evidence_node_structurally_valid_spec(node),
        evidence_kind_requires_derivation_spec(node.kind),
        !contains_evidence_id_spec(graph.nodes, node.id),
        inputs.len() > 0,
        forall|index: int|
            0 <= index < inputs.len() ==> {
                &&& provenance_edge_structurally_valid_spec(#[trigger] inputs[index])
                &&& inputs[index].subject == node.id
                &&& inputs[index].relation == ProvenanceRelation::DerivedFrom
                &&& inputs[index].transformation is Some
                &&& actor_identity_equal_spec(inputs[index].actor, node.producer.actor)
                &&& contains_evidence_id_spec(graph.nodes, inputs[index].object)
            },
        forall|left: int, right: int|
            0 <= left < right < inputs.len() ==> !provenance_edge_equal_spec(
                #[trigger] inputs[left],
                #[trigger] inputs[right],
            ),
    ensures
        evidence_graph_well_formed_spec(
            EvidenceGraphView { nodes: graph.nodes.push(node), edges: graph.edges + inputs },
        ),
{
    let final_graph = EvidenceGraphView {
        nodes: graph.nodes.push(node),
        edges: graph.edges + inputs,
    };
    assert(unique_evidence_node_ids_spec(final_graph.nodes)) by {
        assert forall|left: int, right: int|
            0 <= left < right
                < final_graph.nodes.len() implies #[trigger] final_graph.nodes[left].id
            != #[trigger] final_graph.nodes[right].id by {
            if right < graph.nodes.len() {
                assert(final_graph.nodes[left] == graph.nodes[left]);
                assert(final_graph.nodes[right] == graph.nodes[right]);
            } else {
                assert(right == graph.nodes.len());
                assert(final_graph.nodes[left] == graph.nodes[left]);
                assert(final_graph.nodes[right] == node);
            }
        };
    };
    assert(unique_provenance_edges_spec(final_graph.edges)) by {
        assert forall|left: int, right: int|
            0 <= left < right < final_graph.edges.len() implies !provenance_edge_equal_spec(
            #[trigger] final_graph.edges[left],
            #[trigger] final_graph.edges[right],
        ) by {
            if right < graph.edges.len() {
                assert(final_graph.edges[left] == graph.edges[left]);
                assert(final_graph.edges[right] == graph.edges[right]);
            } else if left < graph.edges.len() {
                let input_index = right - graph.edges.len();
                assert(0 <= input_index < inputs.len());
                assert(final_graph.edges[left] == graph.edges[left]);
                assert(final_graph.edges[right] == inputs[input_index]);
                let endpoint = choose|endpoint: int|
                    #![trigger graph.nodes[endpoint]]
                    0 <= endpoint < graph.nodes.len() && graph.nodes[endpoint].id
                        == graph.edges[left].subject;
                assert(graph.nodes[endpoint].id != node.id);
                assert(graph.edges[left].subject != inputs[input_index].subject);
            } else {
                let left_input = left - graph.edges.len();
                let right_input = right - graph.edges.len();
                assert(0 <= left_input < right_input < inputs.len());
                assert(final_graph.edges[left] == inputs[left_input]);
                assert(final_graph.edges[right] == inputs[right_input]);
            }
        };
    };
    assert(provenance_endpoints_present_spec(final_graph)) by {
        assert forall|index: int|
            0 <= index < final_graph.edges.len() implies contains_evidence_id_spec(
            final_graph.nodes,
            #[trigger] final_graph.edges[index].subject,
        ) && contains_evidence_id_spec(final_graph.nodes, final_graph.edges[index].object) by {
            if index < graph.edges.len() {
                assert(final_graph.edges[index] == graph.edges[index]);
                let subject = choose|endpoint: int|
                    #![trigger graph.nodes[endpoint]]
                    0 <= endpoint < graph.nodes.len() && graph.nodes[endpoint].id
                        == graph.edges[index].subject;
                let object = choose|endpoint: int|
                    #![trigger graph.nodes[endpoint]]
                    0 <= endpoint < graph.nodes.len() && graph.nodes[endpoint].id
                        == graph.edges[index].object;
                assert(final_graph.nodes[subject] == graph.nodes[subject]);
                assert(final_graph.nodes[object] == graph.nodes[object]);
            } else {
                let input_index = index - graph.edges.len();
                assert(0 <= input_index < inputs.len());
                assert(final_graph.edges[index] == inputs[input_index]);
                assert(final_graph.nodes[graph.nodes.len() as int] == node);
                let object = choose|endpoint: int|
                    #![trigger graph.nodes[endpoint]]
                    0 <= endpoint < graph.nodes.len() && graph.nodes[endpoint].id
                        == inputs[input_index].object;
                assert(final_graph.nodes[object] == graph.nodes[object]);
            }
        };
    };
    assert(all_evidence_nodes_structurally_valid_spec(final_graph.nodes)) by {
        assert forall|index: int|
            0 <= index < final_graph.nodes.len() implies evidence_node_structurally_valid_spec(
            #[trigger] final_graph.nodes[index],
        ) by {
            if index < graph.nodes.len() {
                assert(final_graph.nodes[index] == graph.nodes[index]);
            } else {
                assert(index == graph.nodes.len());
                assert(final_graph.nodes[index] == node);
            }
        };
    };
    assert(all_provenance_edges_structurally_valid_spec(final_graph.edges)) by {
        assert forall|index: int|
            0 <= index < final_graph.edges.len() implies provenance_edge_structurally_valid_spec(
            #[trigger] final_graph.edges[index],
        ) by {
            if index < graph.edges.len() {
                assert(final_graph.edges[index] == graph.edges[index]);
            } else {
                let input_index = index - graph.edges.len();
                assert(0 <= input_index < inputs.len());
                assert(final_graph.edges[index] == inputs[input_index]);
            }
        };
    };
    assert(all_derivations_complete_spec(final_graph)) by {
        assert forall|index: int|
            0 <= index < final_graph.nodes.len() implies node_has_complete_derivation_spec(
            final_graph,
            #[trigger] final_graph.nodes[index],
        ) by {
            if index < graph.nodes.len() {
                assert(final_graph.nodes[index] == graph.nodes[index]);
                if evidence_kind_requires_derivation_spec(graph.nodes[index].kind) {
                    let witness = choose|witness: int|
                        #![trigger graph.edges[witness]]
                        0 <= witness < graph.edges.len() && graph.edges[witness].subject
                            == graph.nodes[index].id && graph.edges[witness].relation
                            == ProvenanceRelation::DerivedFrom
                            && graph.edges[witness].transformation is Some
                            && actor_identity_equal_spec(
                            graph.edges[witness].actor,
                            graph.nodes[index].producer.actor,
                        );
                    assert(final_graph.edges[witness] == graph.edges[witness]);
                }
            } else {
                assert(index == graph.nodes.len());
                assert(final_graph.nodes[index] == node);
                assert(final_graph.edges[graph.edges.len() as int] == inputs[0]);
            }
        };
    };
}

impl EvidenceGraph {
    pub fn new() -> (graph: Self)
        ensures
            graph@.nodes.len() == 0,
            graph@.edges.len() == 0,
            evidence_graph_well_formed_spec(graph@),
    {
        Self { nodes: Vec::new(), edges: Vec::new() }
    }

    pub fn nodes(&self) -> (nodes: &[EvidenceNode])
        ensures
            evidence_node_views_spec(nodes@) == self@.nodes,
    {
        self.nodes.as_slice()
    }

    pub fn edges(&self) -> (edges: &[ProvenanceEdge])
        ensures
            provenance_edge_views_spec(edges@) == self@.edges,
    {
        self.edges.as_slice()
    }

    pub fn insert_node(&mut self, node: &EvidenceNode) -> (result: Result<
        GraphInsertOutcome,
        EvidenceGraphError,
    >)
        requires
            evidence_graph_well_formed_spec(old(self)@),
        ensures
            evidence_graph_well_formed_spec(final(self)@),
            evidence_node_structurally_valid_spec(node@) && !evidence_kind_requires_derivation_spec(
                node.kind,
            ) && contains_evidence_node_spec(old(self)@.nodes, node@) ==> result == Ok(
                GraphInsertOutcome::AlreadyPresent,
            ),
            evidence_node_structurally_valid_spec(node@) && !evidence_kind_requires_derivation_spec(
                node.kind,
            ) && contains_conflicting_evidence_node_spec(old(self)@.nodes, node@) ==> result == Err(
                EvidenceGraphError::NodeConflict,
            ),
            evidence_node_structurally_valid_spec(node@) && !evidence_kind_requires_derivation_spec(
                node.kind,
            ) && !contains_evidence_id_spec(old(self)@.nodes, node@.id) ==> result == Ok(
                GraphInsertOutcome::Inserted,
            ),
            match result {
                Ok(GraphInsertOutcome::Inserted) => final(self)@.nodes == old(self)@.nodes.push(
                    node@,
                ) && final(self)@.edges == old(self)@.edges,
                Ok(GraphInsertOutcome::AlreadyPresent) => final(self)@ == old(self)@ && exists|
                    index: int,
                |
                    0 <= index < old(self)@.nodes.len() && evidence_node_equal_spec(
                        #[trigger] old(self)@.nodes[index],
                        node@,
                    ),
                Err(EvidenceGraphError::NodeConflict) => final(self)@ == old(self)@ && exists|
                    index: int,
                |
                    0 <= index < old(self)@.nodes.len() && #[trigger] old(self)@.nodes[index].id
                        == node@.id && !evidence_node_equal_spec(old(self)@.nodes[index], node@),
                Err(EvidenceGraphError::Validation(_)) => final(self)@ == old(self)@ && (
                !evidence_node_structurally_valid_spec(node@)
                    || evidence_kind_requires_derivation_spec(node.kind)),
                Err(EvidenceGraphError::MissingSubjectNode)
                | Err(EvidenceGraphError::MissingObjectNode) => false,
            },
    {
        let ghost before = self@;
        assert(before == old(self)@);
        assert(evidence_graph_well_formed_spec(before));
        if let Err(error) = validate_evidence_node(node) {
            return Err(EvidenceGraphError::Validation(error));
        }
        if evidence_kind_requires_derivation(node.kind) {
            return Err(
                EvidenceGraphError::Validation(
                    EvidenceValidationError::Missing(EvidenceField::DerivationInputs),
                ),
            );
        }
        let mut index = 0;
        while index < self.nodes.len()
            invariant
                self@ == before,
                evidence_graph_well_formed_spec(before),
                index <= self@.nodes.len(),
                forall|prior: int|
                    0 <= prior < index ==> #[trigger] self@.nodes[prior].id != node@.id,
            decreases self.nodes.len() - index,
        {
            if same_evidence_id(&self.nodes[index].id, &node.id) {
                if same_evidence_node(&self.nodes[index], node) {
                    assert(self@ == before);
                    assert(before == old(self)@);
                    assert(evidence_graph_well_formed_spec(self@));
                    assert(evidence_node_equal_spec(self@.nodes[index as int], node@));
                    assert(exists|witness: int|
                        0 <= witness < old(self)@.nodes.len() && evidence_node_equal_spec(
                            #[trigger] old(self)@.nodes[witness],
                            node@,
                        ));
                    return Ok(GraphInsertOutcome::AlreadyPresent);
                }
                assert(self@ == before);
                assert(before == old(self)@);
                assert(evidence_graph_well_formed_spec(self@));
                assert(self@.nodes[index as int].id == node@.id);
                assert(!evidence_node_equal_spec(self@.nodes[index as int], node@));
                assert(exists|witness: int|
                    0 <= witness < old(self)@.nodes.len() && #[trigger] old(self)@.nodes[witness].id
                        == node@.id && !evidence_node_equal_spec(old(self)@.nodes[witness], node@));
                return Err(EvidenceGraphError::NodeConflict);
            }
            index += 1;
        }
        assert(!contains_evidence_id_spec(self@.nodes, node@.id));
        let ghost node_view = node@;
        let owned_node = clone_evidence_node(node);
        self.nodes.push(owned_node);
        assert(self@.nodes == before.nodes.push(node_view));
        proof {
            lemma_append_node_preserves_well_formed(before, node_view);
        }
        Ok(GraphInsertOutcome::Inserted)
    }

    pub fn insert_edge(&mut self, edge: &ProvenanceEdge) -> (result: Result<
        GraphInsertOutcome,
        EvidenceGraphError,
    >)
        requires
            evidence_graph_well_formed_spec(old(self)@),
        ensures
            evidence_graph_well_formed_spec(final(self)@),
            provenance_edge_structurally_valid_spec(edge@) && !contains_evidence_id_spec(
                old(self)@.nodes,
                edge@.subject,
            ) ==> result == Err(EvidenceGraphError::MissingSubjectNode),
            provenance_edge_structurally_valid_spec(edge@) && contains_evidence_id_spec(
                old(self)@.nodes,
                edge@.subject,
            ) && !contains_evidence_id_spec(old(self)@.nodes, edge@.object) ==> result == Err(
                EvidenceGraphError::MissingObjectNode,
            ),
            provenance_edge_structurally_valid_spec(edge@) && contains_evidence_id_spec(
                old(self)@.nodes,
                edge@.subject,
            ) && contains_evidence_id_spec(old(self)@.nodes, edge@.object)
                && contains_provenance_edge_spec(old(self)@.edges, edge@) ==> result == Ok(
                GraphInsertOutcome::AlreadyPresent,
            ),
            provenance_edge_structurally_valid_spec(edge@) && contains_evidence_id_spec(
                old(self)@.nodes,
                edge@.subject,
            ) && contains_evidence_id_spec(old(self)@.nodes, edge@.object)
                && !contains_provenance_edge_spec(old(self)@.edges, edge@) ==> result == Ok(
                GraphInsertOutcome::Inserted,
            ),
            match result {
                Ok(GraphInsertOutcome::Inserted) => final(self)@.nodes == old(self)@.nodes
                    && final(self)@.edges == old(self)@.edges.push(edge@),
                Ok(GraphInsertOutcome::AlreadyPresent) => final(self)@ == old(self)@ && exists|
                    index: int,
                |
                    0 <= index < old(self)@.edges.len() && provenance_edge_equal_spec(
                        #[trigger] old(self)@.edges[index],
                        edge@,
                    ),
                Err(EvidenceGraphError::MissingSubjectNode) => final(self)@ == old(self)@
                    && !contains_evidence_id_spec(old(self)@.nodes, edge@.subject),
                Err(EvidenceGraphError::MissingObjectNode) => final(self)@ == old(self)@
                    && contains_evidence_id_spec(old(self)@.nodes, edge@.subject)
                    && !contains_evidence_id_spec(old(self)@.nodes, edge@.object),
                Err(EvidenceGraphError::Validation(_)) => final(self)@ == old(self)@
                    && !provenance_edge_structurally_valid_spec(edge@),
                Err(EvidenceGraphError::NodeConflict) => false,
            },
    {
        let ghost before = self@;
        assert(before == old(self)@);
        assert(evidence_graph_well_formed_spec(before));
        if let Err(error) = validate_provenance_edge(edge) {
            return Err(EvidenceGraphError::Validation(error));
        }
        if !contains_evidence_id(&self.nodes, &edge.subject) {
            return Err(EvidenceGraphError::MissingSubjectNode);
        }
        if !contains_evidence_id(&self.nodes, &edge.object) {
            return Err(EvidenceGraphError::MissingObjectNode);
        }
        let mut index = 0;
        while index < self.edges.len()
            invariant
                self@ == before,
                evidence_graph_well_formed_spec(before),
                index <= self@.edges.len(),
                forall|prior: int|
                    0 <= prior < index ==> !provenance_edge_equal_spec(
                        #[trigger] self@.edges[prior],
                        edge@,
                    ),
            decreases self.edges.len() - index,
        {
            if same_provenance_edge(&self.edges[index], edge) {
                assert(self@ == before);
                assert(before == old(self)@);
                assert(evidence_graph_well_formed_spec(self@));
                assert(provenance_edge_equal_spec(self@.edges[index as int], edge@));
                assert(exists|witness: int|
                    0 <= witness < old(self)@.edges.len() && provenance_edge_equal_spec(
                        #[trigger] old(self)@.edges[witness],
                        edge@,
                    ));
                return Ok(GraphInsertOutcome::AlreadyPresent);
            }
            index += 1;
        }
        assert(!contains_provenance_edge_spec(self@.edges, edge@));
        let ghost edge_view = edge@;
        let owned_edge = clone_provenance_edge(edge);
        self.edges.push(owned_edge);
        assert(self@.edges == before.edges.push(edge_view));
        proof {
            lemma_append_edge_preserves_well_formed(before, edge_view);
        }
        Ok(GraphInsertOutcome::Inserted)
    }

    /// Atomically admits a derived node and every declared input edge.
    ///
    /// Each input is directed `derived subject --DerivedFrom--> existing object`, carries a
    /// transformation, and is attributed to the node's producer actor. The method borrows the
    /// complete publication so every failure leaves both graph and retry payload unchanged.
    #[verifier::rlimit(30)]
    pub fn publish_derivation(&mut self, node: &EvidenceNode, inputs: &[ProvenanceEdge]) -> (result:
        Result<GraphInsertOutcome, EvidenceGraphError>)
        requires
            evidence_graph_well_formed_spec(old(self)@),
        ensures
            evidence_graph_well_formed_spec(final(self)@),
            result is Err ==> final(self)@ == old(self)@,
            result == Ok(GraphInsertOutcome::AlreadyPresent) ==> final(self)@ == old(self)@,
            result == Ok(GraphInsertOutcome::Inserted) ==> final(self)@.nodes == old(
                self,
            )@.nodes.push(node@) && final(self)@.edges == old(self)@.edges
                + provenance_edge_views_spec(inputs@),
    {
        let ghost before = self@;
        assert(before == old(self)@);
        assert(evidence_graph_well_formed_spec(before));
        if let Err(error) = validate_evidence_node(node) {
            return Err(EvidenceGraphError::Validation(error));
        }
        if !evidence_kind_requires_derivation(node.kind) {
            return Err(
                EvidenceGraphError::Validation(
                    EvidenceValidationError::Mismatch(EvidenceField::DerivationRelation),
                ),
            );
        }
        if inputs.is_empty() {
            return Err(
                EvidenceGraphError::Validation(
                    EvidenceValidationError::Missing(EvidenceField::DerivationInputs),
                ),
            );
        }
        let mut input_index = 0;
        while input_index < inputs.len()
            invariant
                self@ == before,
                evidence_graph_well_formed_spec(before),
                input_index <= inputs@.len(),
                forall|prior: int|
                    0 <= prior < input_index ==> {
                        &&& provenance_edge_structurally_valid_spec(#[trigger] inputs@[prior]@)
                        &&& inputs@[prior]@.subject == node@.id
                        &&& inputs@[prior]@.relation == ProvenanceRelation::DerivedFrom
                        &&& inputs@[prior]@.transformation is Some
                        &&& actor_identity_equal_spec(inputs@[prior]@.actor, node@.producer.actor)
                        &&& contains_evidence_id_spec(before.nodes, inputs@[prior]@.object)
                    },
                forall|left: int, right: int|
                    0 <= left < right < input_index ==> !provenance_edge_equal_spec(
                        #[trigger] inputs@[left]@,
                        #[trigger] inputs@[right]@,
                    ),
            decreases inputs.len() - input_index,
        {
            if let Err(error) = validate_provenance_edge(&inputs[input_index]) {
                return Err(EvidenceGraphError::Validation(error));
            }
            if !same_evidence_id(&inputs[input_index].subject, &node.id) {
                return Err(
                    EvidenceGraphError::Validation(
                        EvidenceValidationError::Mismatch(EvidenceField::DerivationSubject),
                    ),
                );
            }
            if !same_provenance_relation(
                inputs[input_index].relation,
                ProvenanceRelation::DerivedFrom,
            ) {
                return Err(
                    EvidenceGraphError::Validation(
                        EvidenceValidationError::Mismatch(EvidenceField::DerivationRelation),
                    ),
                );
            }
            if inputs[input_index].transformation.is_none() {
                return Err(
                    EvidenceGraphError::Validation(
                        EvidenceValidationError::Missing(EvidenceField::DerivationTransformation),
                    ),
                );
            }
            if !same_actor_identity(&inputs[input_index].actor, &node.producer.actor) {
                return Err(
                    EvidenceGraphError::Validation(
                        EvidenceValidationError::Mismatch(EvidenceField::DerivationActor),
                    ),
                );
            }
            if !contains_evidence_id(&self.nodes, &inputs[input_index].object) {
                return Err(EvidenceGraphError::MissingObjectNode);
            }
            let mut prior = 0;
            while prior < input_index
                invariant
                    self@ == before,
                    evidence_graph_well_formed_spec(before),
                    prior <= input_index < inputs@.len(),
                    forall|earlier: int|
                        0 <= earlier < prior ==> !provenance_edge_equal_spec(
                            #[trigger] inputs@[earlier]@,
                            inputs@[input_index as int]@,
                        ),
                decreases input_index - prior,
            {
                let duplicate = same_provenance_edge(&inputs[prior], &inputs[input_index]);
                if duplicate {
                    return Err(
                        EvidenceGraphError::Validation(
                            EvidenceValidationError::Duplicate(EvidenceField::DerivationInputs),
                        ),
                    );
                }
                assert(!provenance_edge_equal_spec(
                    inputs@[prior as int]@,
                    inputs@[input_index as int]@,
                ));
                assert(!provenance_edge_equal_spec(
                    inputs@[prior as int]@,
                    inputs@[input_index as int]@,
                ));
                prior += 1;
            }
            input_index += 1;
        }

        let mut node_index = 0;
        while node_index < self.nodes.len()
            invariant
                self@ == before,
                evidence_graph_well_formed_spec(before),
                node_index <= self@.nodes.len(),
                forall|prior: int|
                    0 <= prior < node_index ==> #[trigger] self@.nodes[prior].id != node@.id,
            decreases self.nodes.len() - node_index,
        {
            if same_evidence_id(&self.nodes[node_index].id, &node.id) {
                if !same_evidence_node(&self.nodes[node_index], node) {
                    return Err(EvidenceGraphError::NodeConflict);
                }
                let mut retry_index = 0;
                while retry_index < inputs.len()
                    invariant
                        self@ == before,
                        evidence_graph_well_formed_spec(self@),
                        retry_index <= inputs@.len(),
                        forall|prior: int|
                            0 <= prior < retry_index ==> contains_provenance_edge_spec(
                                self@.edges,
                                #[trigger] inputs@[prior]@,
                            ),
                    decreases inputs.len() - retry_index,
                {
                    if !contains_provenance_edge(&self.edges, &inputs[retry_index]) {
                        return Err(
                            EvidenceGraphError::Validation(
                                EvidenceValidationError::Mismatch(EvidenceField::DerivationInputs),
                            ),
                        );
                    }
                    retry_index += 1;
                }
                return Ok(GraphInsertOutcome::AlreadyPresent);
            }
            node_index += 1;
        }

        let owned_node = clone_evidence_node(node);
        self.nodes.push(owned_node);
        let ghost input_views = provenance_edge_views_spec(inputs@);
        let mut append_index = 0;
        while append_index < inputs.len()
            invariant
                self@.nodes == before.nodes.push(node@),
                self@.edges == before.edges + input_views.take(append_index as int),
                append_index <= inputs@.len(),
                input_views.len() == inputs@.len(),
                input_views == provenance_edge_views_spec(inputs@),
            decreases inputs.len() - append_index,
        {
            assert(provenance_edge_views_spec(inputs@)[append_index as int]
                == inputs@[append_index as int]@);
            assert(input_views == provenance_edge_views_spec(inputs@));
            let owned_edge = clone_provenance_edge(&inputs[append_index]);
            let ghost owned_edge_view = owned_edge@;
            assert(owned_edge_view == input_views[append_index as int]);
            let ghost prior_edges = self@.edges;
            self.edges.push(owned_edge);
            assert(self@.edges == prior_edges.push(owned_edge_view));
            proof {
                input_views.lemma_take_succ_push(append_index as int);
            }
            assert((before.edges + input_views.take(append_index as int)).push(owned_edge_view)
                == before.edges + input_views.take(append_index as int).push(owned_edge_view)) by {
                assert_seqs_equal!(
                    (before.edges + input_views.take(append_index as int)).push(owned_edge_view)
                        == before.edges
                            + input_views.take(append_index as int).push(owned_edge_view)
                );
            };
            append_index += 1;
        }
        assert(self@.edges == before.edges + input_views);
        proof {
            lemma_publish_derivation_preserves_well_formed(before, node@, input_views);
        }
        Ok(GraphInsertOutcome::Inserted)
    }
}

impl Default for EvidenceGraph {
    fn default() -> Self {
        Self::new()
    }
}

} // verus!
