//! Verified, bounded storage publication, retention, and collection policy.
use crate::{ArtifactId, ArtifactRef};
use vstd::prelude::*;

verus! {

/// Absolute cap on a single garbage-collection planning pass.
pub const MAX_GC_CANDIDATES: usize = 4_096;

/// Absolute cap on active leases considered by one collection pass.
pub const MAX_GC_LEASES: usize = 1_024;

/// Absolute item cap on an atomic persistence batch.
pub const MAX_PERSISTENCE_BATCH_ITEMS: u64 = 4_096;

/// Absolute encoded-byte cap on an atomic persistence batch.
pub const MAX_PERSISTENCE_BATCH_BYTES: u64 = 64 * 1_024 * 1_024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoragePolicyError {
    InvalidPublicationTransition,
    UnsupportedTopology,
    CandidateLimit,
    LeaseLimit,
    InvalidCollectionGeneration,
    BatchItemLimit,
    BatchByteLimit,
    EmptyPersistenceItem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationState {
    LeaseReserved,
    VerifiedObjectPublished,
    ReferenceCommitted,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationTransition {
    PublishVerifiedObject,
    CommitReference,
    ReleaseLease,
}

pub open spec fn publication_transition_spec(
    state: PublicationState,
    transition: PublicationTransition,
) -> Result<PublicationState, StoragePolicyError> {
    match (state, transition) {
        (PublicationState::LeaseReserved, PublicationTransition::PublishVerifiedObject) => {
            Ok(PublicationState::VerifiedObjectPublished)
        },
        (PublicationState::VerifiedObjectPublished, PublicationTransition::CommitReference) => {
            Ok(PublicationState::ReferenceCommitted)
        },
        (PublicationState::ReferenceCommitted, PublicationTransition::ReleaseLease) => {
            Ok(PublicationState::Complete)
        },
        _ => Err(StoragePolicyError::InvalidPublicationTransition),
    }
}

/// Advances publication only in the order lease -> verified object -> reference -> release.
pub fn advance_publication(state: PublicationState, transition: PublicationTransition) -> (result:
    Result<PublicationState, StoragePolicyError>)
    ensures
        result == publication_transition_spec(state, transition),
{
    match (state, transition) {
        (PublicationState::LeaseReserved, PublicationTransition::PublishVerifiedObject) => {
            Ok(PublicationState::VerifiedObjectPublished)
        },
        (PublicationState::VerifiedObjectPublished, PublicationTransition::CommitReference) => {
            Ok(PublicationState::ReferenceCommitted)
        },
        (PublicationState::ReferenceCommitted, PublicationTransition::ReleaseLease) => {
            Ok(PublicationState::Complete)
        },
        _ => Err(StoragePolicyError::InvalidPublicationTransition),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataBackend {
    Sqlite,
    TransactionalServer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectBackend {
    Filesystem,
    Remote,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageTopology {
    pub metadata: MetadataBackend,
    pub objects: ObjectBackend,
    pub identity_schema_version: u16,
}

impl StorageTopology {
    pub open spec fn supported_spec(&self) -> bool {
        self.identity_schema_version == 1 && match (self.metadata, self.objects) {
            (MetadataBackend::Sqlite, ObjectBackend::Filesystem) => true,
            (MetadataBackend::TransactionalServer, ObjectBackend::Remote) => true,
            _ => false,
        }
    }

    pub fn is_supported(&self) -> (supported: bool)
        ensures
            supported == self.supported_spec(),
    {
        if self.identity_schema_version != 1 {
            false
        } else {
            matches!(
                (self.metadata, self.objects),
                (MetadataBackend::Sqlite, ObjectBackend::Filesystem)
                    | (MetadataBackend::TransactionalServer, ObjectBackend::Remote)
            )
        }
    }

    pub fn preserves_identity_with(&self, other: &Self) -> (preserves: bool)
        ensures
            preserves == (self.identity_schema_version == 1 && other.identity_schema_version
                == self.identity_schema_version),
    {
        self.identity_schema_version == 1 && other.identity_schema_version
            == self.identity_schema_version
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcRootKind {
    OriginalFinding,
    Regression,
    ActiveCampaign,
    EvidenceBundle,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GcCandidate {
    pub artifact_id: ArtifactId,
    pub published_generation: u64,
    pub root: Option<GcRootKind>,
}

impl GcCandidate {
    pub fn new(
        artifact_id: ArtifactId,
        published_generation: u64,
        root: Option<GcRootKind>,
    ) -> Self {
        Self { artifact_id, published_generation, root }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GenerationLease {
    pub artifact_id: Option<ArtifactId>,
    pub generation: u64,
    pub active: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GcPlan {
    pub collect: Vec<ArtifactId>,
    pub preserved: u64,
}

/// Selects only old, unreachable, unleased objects. The two input caps bound both memory and work.
pub fn conservative_gc_plan(
    candidates: &[GcCandidate],
    leases: &[GenerationLease],
    collection_generation: u64,
) -> (result: Result<GcPlan, StoragePolicyError>) {
    if candidates.len() > MAX_GC_CANDIDATES {
        return Err(StoragePolicyError::CandidateLimit);
    }
    if leases.len() > MAX_GC_LEASES {
        return Err(StoragePolicyError::LeaseLimit);
    }
    if collection_generation == 0 {
        return Err(StoragePolicyError::InvalidCollectionGeneration);
    }
    let mut collect = Vec::new();
    let mut preserved = 0u64;
    let mut candidate_index = 0usize;
    while candidate_index < candidates.len()
        invariant
            candidate_index <= candidates.len(),
            candidates.len() <= MAX_GC_CANDIDATES,
            leases.len() <= MAX_GC_LEASES,
            collect.len() <= candidate_index,
            preserved <= candidate_index,
        decreases candidates.len() - candidate_index,
    {
        let candidate = &candidates[candidate_index];
        let mut protected = candidate.root.is_some() || candidate.published_generation
            >= collection_generation;
        let mut lease_index = 0usize;
        while !protected && lease_index < leases.len()
            invariant
                lease_index <= leases.len(),
                leases.len() <= MAX_GC_LEASES,
            decreases leases.len() - lease_index,
        {
            let lease = &leases[lease_index];
            if lease.active {
                protected =
                match &lease.artifact_id {
                    Some(artifact_id) => artifact_id == &candidate.artifact_id,
                    None => candidate.published_generation >= lease.generation,
                };
            }
            lease_index += 1;
        }
        if protected {
            preserved += 1;
        } else {
            collect.push(candidate.artifact_id.clone());
        }
        candidate_index += 1;
    }
    Ok(GcPlan { collect, preserved })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistenceRetentionPolicy {
    ManagedReplay,
    HighThroughput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistenceItemKind {
    TransientSuccessfulRun,
    AggregateCounter,
    Checkpoint,
    InterestingCorpus,
    CandidateFailure,
    OriginalFinding,
    Regression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PersistenceItem {
    pub kind: PersistenceItemKind,
    pub encoded_bytes: u64,
    pub current_batch_items: u64,
    pub current_batch_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PersistenceDecision {
    pub retained: bool,
    pub aggregate_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleSignatureAlgorithm {
    Ed25519,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleSignatureScope {
    ExactManifestAndProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleSignatureError {
    ScopeMismatch,
    UnverifiedSignature,
    HypothesisTruthClaim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedBundleSignature {
    pub algorithm: BundleSignatureAlgorithm,
    pub scope: BundleSignatureScope,
    pub manifest_artifact_id: ArtifactId,
    pub provenance_artifact_id: ArtifactId,
    pub signature_artifact_id: ArtifactId,
    pub cryptographically_verified: bool,
    pub hypothesis_truth_attested: bool,
}

/// Admits only a cryptographically checked signature over the exact manifest and provenance.
/// Hypotheses remain explicitly outside the truth claim of the signature.
pub fn admit_verified_bundle_signature(
    expected_manifest: &ArtifactId,
    expected_provenance: &ArtifactId,
    signature: &VerifiedBundleSignature,
) -> (result: Result<(), BundleSignatureError>) {
    if signature.scope != BundleSignatureScope::ExactManifestAndProvenance
        || &signature.manifest_artifact_id != expected_manifest || &signature.provenance_artifact_id
        != expected_provenance {
        return Err(BundleSignatureError::ScopeMismatch);
    }
    if !signature.cryptographically_verified {
        return Err(BundleSignatureError::UnverifiedSignature);
    }
    if signature.hypothesis_truth_attested {
        return Err(BundleSignatureError::HypothesisTruthClaim);
    }
    Ok(())
}

/// Applies retention semantics while rejecting any batch that would exceed absolute limits.
pub fn admit_persistence_item(policy: PersistenceRetentionPolicy, item: PersistenceItem) -> (result:
    Result<PersistenceDecision, StoragePolicyError>) {
    if item.encoded_bytes == 0 {
        return Err(StoragePolicyError::EmptyPersistenceItem);
    }
    if item.current_batch_items >= MAX_PERSISTENCE_BATCH_ITEMS {
        return Err(StoragePolicyError::BatchItemLimit);
    }
    if item.encoded_bytes > MAX_PERSISTENCE_BATCH_BYTES || item.current_batch_bytes
        > MAX_PERSISTENCE_BATCH_BYTES - item.encoded_bytes {
        return Err(StoragePolicyError::BatchByteLimit);
    }
    if policy == PersistenceRetentionPolicy::HighThroughput && item.kind
        == PersistenceItemKind::TransientSuccessfulRun {
        Ok(PersistenceDecision { retained: false, aggregate_only: true })
    } else {
        Ok(PersistenceDecision { retained: true, aggregate_only: false })
    }
}

} // verus!
/// One atomic metadata transaction. Implementations may be embedded or server-backed.
pub trait MetadataTransaction {
    type Error;

    fn record_artifact(&mut self, artifact: &ArtifactRef) -> Result<(), Self::Error>;

    fn commit(self) -> Result<(), Self::Error>;
}

/// Transactional metadata interface shared by SQLite and distributed server implementations.
pub trait TransactionalMetadataStore {
    type Error;
    type Transaction<'a>: MetadataTransaction<Error = Self::Error>
    where
        Self: 'a;

    fn backend(&self) -> MetadataBackend;

    fn identity_schema_version(&self) -> u16;

    fn max_batch_items(&self) -> u64;

    fn max_batch_bytes(&self) -> u64;

    fn begin_transaction(&mut self) -> Result<Self::Transaction<'_>, Self::Error>;
}

/// Content-addressed object interface shared by filesystem and remote object implementations.
pub trait ArtifactObjectStore {
    type Error;

    fn backend(&self) -> ObjectBackend;

    fn identity_schema_version(&self) -> u16;

    fn publish_verified_no_clobber(
        &mut self,
        artifact: &ArtifactRef,
        contents: &[u8],
        max_bytes: u64,
    ) -> Result<(), Self::Error>;

    fn load_verified(&self, artifact: &ArtifactRef, max_bytes: u64)
        -> Result<Vec<u8>, Self::Error>;
}
