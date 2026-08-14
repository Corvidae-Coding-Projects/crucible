use crucible_core::{
    admit_persistence_item, admit_verified_bundle_signature, advance_publication,
    conservative_gc_plan, ArtifactId, ArtifactObjectStore, ArtifactRef, BundleSignatureAlgorithm,
    BundleSignatureError, BundleSignatureScope, GcCandidate, GcRootKind, GenerationLease,
    MetadataBackend, MetadataTransaction, ObjectBackend, PersistenceItem, PersistenceItemKind,
    PersistenceRetentionPolicy, PublicationState, PublicationTransition, StorageTopology,
    TransactionalMetadataStore, VerifiedBundleSignature, MAX_GC_CANDIDATES,
    MAX_PERSISTENCE_BATCH_BYTES, MAX_PERSISTENCE_BATCH_ITEMS,
};

fn id(value: &str) -> ArtifactId {
    ArtifactId::new(value.to_owned())
}

#[test]
fn publication_state_machine_never_commits_a_reference_before_verified_object_publication() {
    assert!(advance_publication(
        PublicationState::LeaseReserved,
        PublicationTransition::CommitReference
    )
    .is_err());
    let published = advance_publication(
        PublicationState::LeaseReserved,
        PublicationTransition::PublishVerifiedObject,
    )
    .unwrap();
    let committed = advance_publication(published, PublicationTransition::CommitReference).unwrap();
    let released = advance_publication(committed, PublicationTransition::ReleaseLease).unwrap();
    assert_eq!(released, PublicationState::Complete);
    assert!(advance_publication(released, PublicationTransition::CommitReference).is_err());
}

#[test]
fn distributed_and_embedded_topologies_preserve_the_same_object_identity_meaning() {
    let embedded = StorageTopology {
        metadata: MetadataBackend::Sqlite,
        objects: ObjectBackend::Filesystem,
        identity_schema_version: 1,
    };
    let distributed = StorageTopology {
        metadata: MetadataBackend::TransactionalServer,
        objects: ObjectBackend::Remote,
        identity_schema_version: 1,
    };
    assert!(embedded.is_supported());
    assert!(distributed.is_supported());
    assert!(embedded.preserves_identity_with(&distributed));
}

#[test]
fn conservative_gc_preserves_every_normative_root_and_generation_barrier() {
    let candidates = vec![
        GcCandidate::new(id("sha256:original"), 1, Some(GcRootKind::OriginalFinding)),
        GcCandidate::new(id("sha256:regression"), 1, Some(GcRootKind::Regression)),
        GcCandidate::new(id("sha256:campaign"), 1, Some(GcRootKind::ActiveCampaign)),
        GcCandidate::new(id("sha256:bundle"), 1, Some(GcRootKind::EvidenceBundle)),
        GcCandidate::new(id("sha256:leased"), 1, None),
        GcCandidate::new(id("sha256:barrier"), 7, None),
        GcCandidate::new(id("sha256:collect"), 2, None),
    ];
    let leases = vec![GenerationLease {
        artifact_id: Some(id("sha256:leased")),
        generation: 1,
        active: true,
    }];
    let plan = conservative_gc_plan(&candidates, &leases, 7).unwrap();
    assert_eq!(plan.collect.len(), 1);
    assert_eq!(plan.collect[0].as_str(), "sha256:collect");
    assert_eq!(plan.preserved, 6);
}

#[test]
fn gc_and_persistence_inputs_have_absolute_memory_and_work_caps() {
    let excessive = (0..=MAX_GC_CANDIDATES)
        .map(|index| GcCandidate::new(id(&format!("sha256:{index}")), 1, None))
        .collect::<Vec<_>>();
    assert!(conservative_gc_plan(&excessive, &[], 2).is_err());

    let too_many = PersistenceItem {
        kind: PersistenceItemKind::AggregateCounter,
        encoded_bytes: 1,
        current_batch_items: MAX_PERSISTENCE_BATCH_ITEMS,
        current_batch_bytes: 0,
    };
    assert!(admit_persistence_item(PersistenceRetentionPolicy::HighThroughput, too_many).is_err());
    let too_large = PersistenceItem {
        kind: PersistenceItemKind::Checkpoint,
        encoded_bytes: 1,
        current_batch_items: 0,
        current_batch_bytes: MAX_PERSISTENCE_BATCH_BYTES,
    };
    assert!(admit_persistence_item(PersistenceRetentionPolicy::HighThroughput, too_large).is_err());
}

#[test]
fn retention_policy_keeps_replay_and_aggregates_transient_fuzz_work() {
    let transient = PersistenceItem {
        kind: PersistenceItemKind::TransientSuccessfulRun,
        encoded_bytes: 128,
        current_batch_items: 0,
        current_batch_bytes: 0,
    };
    assert!(
        admit_persistence_item(PersistenceRetentionPolicy::ManagedReplay, transient)
            .unwrap()
            .retained
    );
    let high_throughput =
        admit_persistence_item(PersistenceRetentionPolicy::HighThroughput, transient).unwrap();
    assert!(!high_throughput.retained);
    assert!(high_throughput.aggregate_only);

    for kind in [
        PersistenceItemKind::AggregateCounter,
        PersistenceItemKind::Checkpoint,
        PersistenceItemKind::InterestingCorpus,
        PersistenceItemKind::CandidateFailure,
        PersistenceItemKind::OriginalFinding,
        PersistenceItemKind::Regression,
    ] {
        let decision = admit_persistence_item(
            PersistenceRetentionPolicy::HighThroughput,
            PersistenceItem {
                kind,
                encoded_bytes: 128,
                current_batch_items: 0,
                current_batch_bytes: 0,
            },
        )
        .unwrap();
        assert!(decision.retained);
        assert!(!decision.aggregate_only);
    }
}

#[derive(Default)]
struct TestTransaction {
    recorded: Vec<String>,
}

impl MetadataTransaction for TestTransaction {
    type Error = ();

    fn record_artifact(&mut self, artifact: &ArtifactRef) -> Result<(), Self::Error> {
        self.recorded.push(artifact.id.as_str().to_owned());
        Ok(())
    }

    fn commit(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct TestMetadataStore {
    backend: MetadataBackend,
}

impl TransactionalMetadataStore for TestMetadataStore {
    type Error = ();
    type Transaction<'a> = TestTransaction;

    fn backend(&self) -> MetadataBackend {
        self.backend
    }

    fn identity_schema_version(&self) -> u16 {
        1
    }

    fn max_batch_items(&self) -> u64 {
        MAX_PERSISTENCE_BATCH_ITEMS
    }

    fn max_batch_bytes(&self) -> u64 {
        MAX_PERSISTENCE_BATCH_BYTES
    }

    fn begin_transaction(&mut self) -> Result<Self::Transaction<'_>, Self::Error> {
        Ok(TestTransaction::default())
    }
}

struct TestObjectStore {
    backend: ObjectBackend,
    bytes: Vec<u8>,
}

impl ArtifactObjectStore for TestObjectStore {
    type Error = ();

    fn backend(&self) -> ObjectBackend {
        self.backend
    }

    fn identity_schema_version(&self) -> u16 {
        1
    }

    fn publish_verified_no_clobber(
        &mut self,
        artifact: &ArtifactRef,
        contents: &[u8],
        max_bytes: u64,
    ) -> Result<(), Self::Error> {
        if contents.len() as u64 > max_bytes {
            return Err(());
        }
        artifact.verify(contents).map_err(|_| ())?;
        if self.bytes.is_empty() {
            self.bytes.extend_from_slice(contents);
        } else if self.bytes != contents {
            return Err(());
        }
        Ok(())
    }

    fn load_verified(
        &self,
        artifact: &ArtifactRef,
        max_bytes: u64,
    ) -> Result<Vec<u8>, Self::Error> {
        if self.bytes.len() as u64 > max_bytes {
            return Err(());
        }
        artifact.verify(&self.bytes).map_err(|_| ())?;
        Ok(self.bytes.clone())
    }
}

#[test]
fn embedded_and_distributed_implementations_share_transaction_and_object_interfaces() {
    let artifact = ArtifactRef::from_bytes(b"portable evidence", None).unwrap();
    for (metadata_backend, object_backend) in [
        (MetadataBackend::Sqlite, ObjectBackend::Filesystem),
        (MetadataBackend::TransactionalServer, ObjectBackend::Remote),
    ] {
        let mut metadata = TestMetadataStore {
            backend: metadata_backend,
        };
        let mut objects = TestObjectStore {
            backend: object_backend,
            bytes: Vec::new(),
        };
        assert_eq!(
            metadata.identity_schema_version(),
            objects.identity_schema_version()
        );
        objects
            .publish_verified_no_clobber(&artifact, b"portable evidence", 1_024)
            .unwrap();
        let mut transaction = metadata.begin_transaction().unwrap();
        transaction.record_artifact(&artifact).unwrap();
        transaction.commit().unwrap();
        assert_eq!(
            objects.load_verified(&artifact, 1_024).unwrap(),
            b"portable evidence"
        );
    }
}

#[test]
fn bundle_signature_attests_exact_manifest_and_provenance_but_never_hypothesis_truth() {
    let manifest = id("sha256:manifest");
    let provenance = id("sha256:provenance");
    let signature = VerifiedBundleSignature {
        algorithm: BundleSignatureAlgorithm::Ed25519,
        scope: BundleSignatureScope::ExactManifestAndProvenance,
        manifest_artifact_id: manifest.clone(),
        provenance_artifact_id: provenance.clone(),
        signature_artifact_id: id("sha256:signature"),
        cryptographically_verified: true,
        hypothesis_truth_attested: false,
    };
    assert!(admit_verified_bundle_signature(&manifest, &provenance, &signature).is_ok());

    let mut wrong_provenance = signature.clone();
    wrong_provenance.provenance_artifact_id = id("sha256:different");
    assert_eq!(
        admit_verified_bundle_signature(&manifest, &provenance, &wrong_provenance),
        Err(BundleSignatureError::ScopeMismatch)
    );
    let mut false_claim = signature.clone();
    false_claim.hypothesis_truth_attested = true;
    assert_eq!(
        admit_verified_bundle_signature(&manifest, &provenance, &false_claim),
        Err(BundleSignatureError::HypothesisTruthClaim)
    );
    let mut unverified = signature;
    unverified.cryptographically_verified = false;
    assert_eq!(
        admit_verified_bundle_signature(&manifest, &provenance, &unverified),
        Err(BundleSignatureError::UnverifiedSignature)
    );
}

#[test]
fn weekly_concurrency_exploration_rejects_every_invalid_publication_interleaving() {
    use PublicationTransition::{CommitReference, PublishVerifiedObject, ReleaseLease};
    let schedules = [
        [PublishVerifiedObject, CommitReference, ReleaseLease],
        [PublishVerifiedObject, ReleaseLease, CommitReference],
        [CommitReference, PublishVerifiedObject, ReleaseLease],
        [CommitReference, ReleaseLease, PublishVerifiedObject],
        [ReleaseLease, PublishVerifiedObject, CommitReference],
        [ReleaseLease, CommitReference, PublishVerifiedObject],
    ];
    for (index, schedule) in schedules.into_iter().enumerate() {
        let mut state = PublicationState::LeaseReserved;
        let mut accepted = true;
        for transition in schedule {
            match advance_publication(state, transition) {
                Ok(next) => state = next,
                Err(_) => {
                    accepted = false;
                    break;
                }
            }
        }
        assert_eq!(
            accepted,
            index == 0,
            "unexpected schedule acceptance: {index}"
        );
        if accepted {
            assert_eq!(state, PublicationState::Complete);
        }
    }

    let candidate = GcCandidate::new(id("sha256:in-flight"), 3, None);
    let active = GenerationLease {
        artifact_id: Some(id("sha256:in-flight")),
        generation: 3,
        active: true,
    };
    let protected = conservative_gc_plan(
        std::slice::from_ref(&candidate),
        std::slice::from_ref(&active),
        4,
    )
    .unwrap();
    assert!(protected.collect.is_empty());
    assert_eq!(protected.preserved, 1);
    let released = conservative_gc_plan(&[candidate], &[], 4).unwrap();
    assert_eq!(released.collect.len(), 1);
}
