# Findings, Replay, and Minimization

## 43. Finding Model

```rust
pub struct Finding {
    pub id: FindingId,
    pub project: ProjectId,
    pub kind: FindingKind,
    pub campaigns: Vec<CampaignId>,
    pub target_builds: Vec<TargetBuildId>,

    pub class: BugClass,
    pub confidence: Confidence,
    pub signature: FindingSignature,
    pub created_at: DateTime<Utc>,
    pub first_instance: FindingInstanceId,
    pub representative_instance: FindingInstanceId,
    pub minimized_experiment: Option<ExperimentId>,
    pub reproduction: Option<ReproductionSummary>,
}

pub enum FindingKind {
    TargetDefect,
    BehavioralDiscrepancy,
    TestAdequacyGap,
    SpecificationOrModelConflict,
    ProofFailure,
    StaticHypothesis,
    HarnessDefect,
    InfrastructureFailure,
    Unknown,
}

pub struct FindingInstance {
    pub id: FindingInstanceId,
    pub finding: FindingId,
    pub experiment: ExperimentId,
    pub observations: ExperimentObservationRef,
    pub oracle_failure: OracleFailureRef,
    pub observed_at: DateTime<Utc>,
}
```

Finding rows should reference immutable evidence instead of embedding mutable copies of the
input and execution result. Classification, confidence, root-cause analysis, and duplicate
relationships are derived records with provenance. The original instance must remain
reachable after a representative or minimized instance is selected.

A canonical finding may accumulate instances from multiple campaigns, engines, configurations,
and target builds. Campaign membership is an append-only relationship, not ownership by the
first campaign. Split, merge, exact-duplicate, probable-duplicate, related, regression-of, and
same-root-cause decisions are reversible derived relationships with decision provenance.

---

## 44. Finding Lifecycle

```text
candidate
    ↓
reproducing
    ↓
confirmed
    ↓
minimized
    ↓
triaged
    ↓
fix-proposed
    ↓
verified
    ↓
closed
```

Terminal or alternate states:

- duplicate,
- false-positive,
- non-reproducible,
- wont-fix,
- superseded.

Flakiness and reproduction rate are properties of evidence, not terminal lifecycle states. A
flaky concurrency bug may still be confirmed and fixed. State transitions must be validated
and appended to a history table with timestamp, actor, reason, and supporting evidence; do
not store only a mutable current-state string. `verified` means the configured gauntlet
passed, while `closed` records the project's disposition.

---

## 45. Deduplication

Naive stderr hashing is insufficient.

A finding signature should combine multiple signals:

- bug class,
- sanitizer kind,
- signal,
- normalized top stack frames,
- first relevant source location,
- normalized error text,
- coverage neighborhood,
- state digest where useful.

Example conceptual signature:

```text
hash(
    signature_algorithm_version,
    bug_class,
    sanitizer_kind,
    top_frames,
    source_location,
    normalized_error
)
```

Every signature must record its algorithm and normalization versions. Recomputing signatures
must not destroy the historical value used when instances were originally grouped. The
system must support clustering similar findings even when signatures are not
identical, but similarity clustering must remain distinguishable from exact-signature
deduplication.

Deduplication must prefer false splits over destructive false merges when evidence is
ambiguous. Same crash site does not prove same root cause, and different crash sites do not
prove distinct root causes. Cluster changes never delete original instances or historical
signatures.

---

## 46. Reproduction Engine

Every candidate finding should be replayed.

Default initial replay policy:

```text
5 attempts
```

Suggested classification:

```text
5/5 => stable under the recorded replay conditions
1-4/5 => intermittent under the recorded replay conditions
0/5 => not observed during this replay sample
```

Do not infer determinism solely from five successful attempts. A reproduction summary must
identify the exact failure predicate, attempt count, successes, environmental equivalence,
and whether schedule and fault traces were replayed exactly. Projects may configure larger
samples or statistical confidence rules. Expensive, temporal, soak, and concurrency findings
may require category-specific policies.

A zero-success sample does not erase the original evidence. It lowers confidence and may
move a finding to `non-reproducible` after policy-defined investigation.

Concurrency findings may require captured schedule replay.

Fault-related findings must replay the exact fault plan.

Statistical and intermittent reproduction policies must support sequential confidence updates,
environmental blocking, and failure-rate comparison rather than forcing every result through a
fixed five-attempt threshold. The default five attempts remain an inexpensive first sample,
not a universal confirmation policy.

---

## 47. Minimization Interface

```rust
pub trait Minimizer {
    async fn minimize(
        &self,
        finding: &Finding,
        oracle: &dyn ReproductionOracle,
    ) -> Result<Experiment, MinimizeError>;
}
```

Minimization must preserve the original, versioned failure predicate. For a single-run byte
finding, it typically changes one stimulus artifact. Differential and metamorphic minimizers
may need to shrink several related runs while preserving their relationship. Schedules,
faults, and environment controls are minimized by their specialized minimizers rather than
being treated as byte input.

Minimization is hierarchical and may jointly reduce scenario topology, participants, input,
actions, timing, schedule, faults, environment, build flags, and observation scope. Each
accepted step records the candidate and verdict so the reduction can be audited. For
intermittent failures, the minimizer uses the configured statistical preservation rule and
must not select tiny inputs merely because noise produced one lucky failure.

---

## 48. Byte-Level Minimization

Implement delta debugging.

Conceptual algorithm:

```text
partition input
remove one partition
rerun
if failure persists:
    keep removal
else:
    restore
reduce partition size
repeat
```

Additional simplifications:

- replace byte range with zeros,
- replace values with boundary values,
- reduce repeated runs,
- simplify integer encodings,
- normalize whitespace if syntactically irrelevant.

---

## 49. Structure-Level Minimization

For ASTs or structured inputs, try removing or simplifying:

- statements,
- declarations,
- expressions,
- tokens,
- fields,
- list elements,
- optional sections,
- nested structures.

Preserve syntax unless malformed syntax itself is part of the defect.

---

## 50. Sequence Minimization

For stateful action sequences:

1. remove actions,
2. replay,
3. retain deletion if failure persists,
4. simplify remaining actions,
5. simplify values inside actions.

Goal:

> smallest action sequence that still demonstrates the failure.

---

## 51. Schedule Minimization

Concurrency schedules should be shrinkable.

Input:

```text
T1 step
T2 step
T3 step
T1 step
T2 step
T1 step
```

Output:

```text
minimal reproducing interleaving
```

Schedule minimization should attempt to remove unnecessary scheduling decisions while preserving failure.

---

## 52. Environment Minimization

Remove unnecessary:

- environment variables,
- files,
- directories,
- configuration options,
- dependency state,
- services,
- time settings.

The final reproduction bundle should contain the smallest environment known to preserve the defect.

---

## 53. Reproduction Bundle

Every confirmed defect should create a self-contained artifact directory.

Example:

```text
findings/BUG-000001/
├── metadata.json
├── experiment.json
├── input.bin
├── stdout.txt
├── stderr.txt
├── stacktrace.txt
├── sanitizer.json
├── environment.json
├── build.json
├── isolation.json
├── oracle.json
├── coverage.json
├── schedule.json
├── faults.json
├── reproduce.sh
└── README.md
```

Not every file is required for every bug.

Every bundle must declare an evidence schema version, artifact digests, target-build identity,
effective execution controls, oracle/failure-predicate version, and any known replay
limitations. If licensing or size prevents embedding a target binary, the bundle must record
its digest and unambiguous rebuild instructions.

Bundles may embed or reference signed OCI images, VM snapshots, emulator images, reproducible
build recipes, dependency locks, symbol packages, proof artifacts, and remote immutable objects.
References state access and licensing requirements without claiming the bundle is fully
self-contained. A manifest identifies every required external artifact and verifies its digest
before replay.

`reproduce.sh` should be a small generated wrapper around `crucible replay --bundle ...`, not
a reconstruction of a command from target-controlled strings. It may interpolate only
generated, validated bundle paths using fixed safe quoting. A machine-readable replay command
is authoritative; the shell wrapper is a convenience.

The bundle must contain enough information to recreate the observed failure or state clearly
which external immutable artifacts are required.

---
