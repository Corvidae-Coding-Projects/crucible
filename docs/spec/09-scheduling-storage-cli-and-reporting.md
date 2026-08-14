# Scheduling, Storage, CLI, and Reporting

## 71. Scheduler

The campaign scheduler allocates compute across engines.

Track per-engine metrics:

```rust
pub struct EngineStats {
    pub executions: u64,
    pub cpu_time: Duration,
    pub new_coverage: u64,
    pub new_findings: u64,
    pub unique_states: u64,
    pub minimized_findings: u64,
}
```

Useful utility signals:

- unique findings per CPU hour,
- coverage increase per CPU hour,
- unique states per CPU hour,
- mutation score improvement,
- new oracle failures,
- corpus quality improvement.

Initial scheduling should remain simple and understandable.

Example starting allocation:

```text
coverage fuzzing       35%
property testing       20%
stateful testing       15%
metamorphic testing    10%
fault injection        10%
symbolic testing        5%
miscellaneous           5%
```

Reallocate periodically based on observed usefulness.

Credit assignment must account for shared corpus ancestry and delayed confirmation. An engine
that discovers a precursor input and an engine that mutates it into a failure may both receive
provenance-based credit. Raw candidate volume, duplicate volume, or easy-to-game coverage
counters must not dominate allocation. Every enabled engine receives a configurable exploration
floor unless a capability failure disables it.

Do not implement complicated reinforcement learning for the initial version.

---

## 72. Storage Architecture

Use SQLite for structured metadata.

Use filesystem content-addressed storage for large artifacts.

Example:

```text
objects/sha256/ab/cd/<full_hash>
```

Artifacts may include:

- inputs,
- logs,
- traces,
- binaries,
- coverage data,
- patches,
- reports,
- environment captures.

Database rows should reference artifact hashes.

SQLite is the embedded metadata backend, not a domain-model dependency. Storage interfaces must
also support a transactional server database and remote object store for distributed workers
without changing evidence meaning or IDs. The embedded implementation remains a complete
single-host mode rather than a reduced client.

Artifact publication and database references must be crash-safe:

1. write to a temporary file in the object-store filesystem,
2. hash and verify the completed contents,
3. atomically publish the object without replacement into its content-addressed location, using a
   same-filesystem rename or an equivalent no-clobber primitive,
4. commit database references in a transaction.

An interrupted write may leave an unreferenced object, but must never leave a committed row
that names a partial object. Provide integrity checking and conservative garbage collection
for unreachable objects. Never garbage-collect artifacts reachable from original findings,
regressions, active campaigns, or evidence bundles.

Concurrent publication uses leases or generation barriers so garbage collection cannot race a
worker that has published an object but not yet committed its reference. Hash algorithms are
versioned and algorithm-agile. Evidence bundle manifests may be cryptographically signed; a
signature attests to the exact manifest and provenance, not to the truth of every contained
hypothesis.

SQLite writes should be batched through a bounded persistence service. Managed replay mode
may retain every run; high-throughput fuzzing should normally retain aggregate counters,
checkpoints, interesting corpus entries, and candidate failures rather than one row per
transient input. The selected retention policy is part of campaign metadata.

---

## 73. Database Schema

Minimum conceptual tables:

```text
projects
targets
target_builds
source_snapshots
build_recipes
build_executions
deployments
campaigns
experiments
scenarios
scenario_participants
scenario_steps
scenario_edges
runs
run_attempts
observations
harness_failures
oracle_verdicts
findings
finding_instances
finding_campaigns
finding_transitions
artifacts
evidence_nodes
provenance_edges
coverage_records
corpus_entries
patches
verification_runs
proof_artifacts
trusted_boundaries
plugin_identities
capability_manifests
engine_stats
```

Schema migrations must be versioned.

Serialized evidence, configuration, engine event, and bundle schemas must also carry explicit
versions independent of the SQLite schema version. Database migrations must not rewrite or
invalidate the meaning of immutable historical evidence.

---

## 74. CLI

Initial commands:

```text
crucible init
crucible artifact import
crucible artifact verify
crucible build
crucible run
crucible fuzz
crucible replay
crucible minimize
crucible findings
crucible inspect
crucible verify
crucible report
crucible config validate
crucible config canonicalize
crucible capabilities
crucible proof
crucible tcb
crucible plugins
```

Examples:

```bash
crucible init
```

`crucible init [path]` creates the documented `.crucible` directory layout beneath the selected
path (the current directory by default) and initializes the embedded database through monotonic,
versioned migrations. The database carries a Crucible application ID, an independent SQLite schema
version, exact migration history, and workspace-format metadata. Repeating the command against the
same valid version is idempotent. Initialization rejects an occupied or symlinked managed path and
an existing database whose identity, version, migration history, metadata, or integrity check does
not match; it does not adopt or overwrite unrelated state.

`crucible artifact import <file> [workspace]` ingests one regular, non-symlink source into the
workspace object store, emits its canonical algorithm-qualified artifact ID, retains source-path
provenance, and transactionally records the immutable object after atomic no-clobber publication.
Duplicate bytes share one object and one artifact row while distinct source paths retain distinct
provenance rows. The initial local command enforces an explicit 64 MiB per-file in-memory import
limit; streaming and directory corpus ingestion remain required follow-on paths for larger inputs
rather than being removed from scope.

`crucible artifact verify <artifact-id> [workspace]` reopens the database row and stored bytes,
recomputes the project-owned digest, and fails if the canonical identity, size, database record, or
object contents disagree. Artifact IDs determine only verified lowercase-hex path components under
`objects/sha256/ab/cd/<full_hash>`; malformed or unsupported IDs never reach path selection.

```bash
crucible run crucible.yaml
```

```bash
crucible replay BUG-000143
```

```bash
crucible minimize BUG-000143
```

```bash
crucible inspect BUG-000143
```

```bash
crucible verify BUG-000143 --patch candidate.diff
```

---

## 75. Human-Readable Reporting

Example:

```text
BUG-000143

Target:
  example-parser

Target build:
  sha256:9d9d... (revision 4e71bc9, clang 20.1.8, UBSan)

Class:
  arithmetic / signed-overflow

Confidence:
  confirmed

Reproduction:
  5/5 under recorded controls (sample is stable; determinism not proven)

Isolation:
  process group, rlimits, private working directory

Minimized input:
  19 bytes

Oracle:
  UBSan parser v1 / signed-overflow predicate v1

First relevant frame:
  parser.rs:411

Patch:
  PATCH-00017

Verification:
  all required stages passed (policy sha256:..., no unapproved TCB growth)
```

Reports should clearly distinguish observed facts from hypotheses.

---

## 76. Machine-Readable Reports

Support:

- JSON,
- JSONL,
- SARIF where appropriate,
- JUnit XML for CI.

Machine-readable output also includes evidence-graph export, proof and trusted-boundary reports,
capability manifests, scenario traces, statistical samples, and signed bundle manifests.

---

## 77. CI Integration

### Tier 1: Every Commit

Run:

- Verus verification for all eligible crates,
- trusted-boundary ledger validation and no-unapproved-growth check,
- unit tests,
- static analysis,
- regression corpus,
- short fuzz campaign,
- core property tests,
- Crucible YAML parser proofs and boundary fixtures.

Target runtime:

```text
minutes
```

### Tier 2: Nightly

Run:

- sanitizers,
- longer fuzzing,
- metamorphic tests,
- differential tests,
- mutation testing,
- fault injection.

Nightly CI also fuzzes configuration, evidence, bundle, plugin, report, and agent-packet parsers
as first-class untrusted boundaries and verifies proof reproducibility for a rotating shard.

Target runtime:

```text
hours
```

### Tier 3: Weekly

Run:

- symbolic exploration,
- concurrency schedule exploration,
- extended fuzzing,
- soak testing,
- larger build matrix,
- formal proof refresh.

Weekly CI performs a cold-cache proof rebuild, trusted-boundary reduction audit, supported
platform and architecture capability conformance, and at least one scenario-topology campaign.

---

## 78. Structured Logging

Use `tracing`.

Required fields should include where applicable:

- timestamp,
- campaign ID,
- engine ID,
- experiment ID,
- run ID,
- run-attempt ID,
- scenario and scenario-step ID,
- participant ID,
- target ID,
- target-build ID,
- worker ID,
- finding ID,
- proof artifact and trusted-boundary ID,
- severity.

Target output should remain separate from harness logs.

---

## 79. Determinism and Replay

Every stochastic subsystem implemented by Crucible must accept a seed. External engines must
receive and report a seed when supported; otherwise the adapter must explicitly record that
seeded replay is unavailable and preserve engine-native checkpoints where possible.

Record:

- campaign seed,
- engine seed,
- experiment seed,
- scheduling seed,
- fault seed.

A reproduction bundle must retain all relevant seeds.

A seed does not prove replayability. Also retain generated schedules, applied fault traces,
target-build identity, effective controls, relevant environment, engine version, and the
versioned failure predicate.

Distinguish three promises:

- finding replay: attempt to reproduce one failed experiment,
- experiment replay: reconstruct all related runs and their ordering/relations,
- campaign replay: reconstruct generation decisions and scheduling as far as the selected
  retention mode permits.

If a target itself is nondeterministic, record as much environmental state as practical and
report observed reproduction rates without claiming exact determinism.

---

## 80. Harness Self-Testing

Crucible itself must be tested against intentionally buggy fixtures.

Create known-defect targets for:

- crash,
- timeout,
- integer overflow,
- memory bug,
- race,
- deadlock,
- state-machine invariant failure,
- serialization mismatch,
- resource leak,
- incorrect result,
- recovery failure.

Also create harness-boundary fixtures for:

- malformed and resource-adversarial Crucible YAML,
- duplicate keys, aliases, alias cycles, and canonicalization,
- corrupt artifact and evidence-graph records,
- partial database and object publication,
- plugin protocol violations and stalled plugins,
- scenario cancellation and partial cleanup,
- VM guest escape-attempt containment fixtures that remain non-offensive,
- prompt injection in source, logs, reports, and target output,
- unregistered Verus assumptions and deliberately false external specifications,
- proof timeout, solver failure, and stale proof-cache identity.

Each subsystem should demonstrate that it can rediscover the expected defect.

Recommended fixture layout:

```text
testdata/targets/
├── crash/
├── hang/
├── arithmetic/
├── memory/
├── concurrency/
├── state/
├── serializer/
├── leak/
├── fault-recovery/
└── logic/
```

---
