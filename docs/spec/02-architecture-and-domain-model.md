# Architecture and Domain Model

## 6. High-Level Architecture

```text
                         ┌──────────────────────┐
                         │ Project Definition   │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Campaign Coordinator │
                         └──────────┬───────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
     ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
     │ Input Producers │   │ Static Engines  │   │ Specs / Models  │
     └────────┬────────┘   └────────┬────────┘   └────────┬────────┘
              │                     │                     │
              └─────────────────────┼─────────────────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Execution Scheduler  │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Target Executor      │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Observation Capture  │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Oracle Engine        │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Finding Pipeline     │
                         └──────────┬───────────┘
                                    │
                  ┌─────────────────┴─────────────────┐
                  ▼                                   ▼
         ┌─────────────────┐                 ┌─────────────────┐
         │ Deduplication   │                 │ Minimization    │
         └────────┬────────┘                 └────────┬────────┘
                  └─────────────────┬─────────────────┘
                                    ▼
                         ┌──────────────────────┐
                         │ Defect Repository    │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Root-Cause Analysis  │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Candidate Repair     │
                         └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │ Verification         │
                         │ Gauntlet             │
                         └──────────┬───────────┘
                                    │
                                    ▼
                              PASS / REJECT
```

The diagram shows the default dynamic-testing path. Native fuzzers may own their execution
loop and publish significant observations into the finding pipeline. Static analyzers and
proof systems may emit versioned hypotheses or proof findings without a target run; when they
seed a dynamic confirmation experiment, that experiment follows the normal run and oracle
path.

---

## 7. Recommended Technology Stack

Core implementation:

- Verus and its pinned compatible Rust toolchain,
- `vstd` and project-owned verified support libraries,
- Rust for executable output and explicitly recorded external boundaries,
- Tokio,
- async-trait where dynamic async adapter interfaces require object safety,
- serde,
- serde_json,
- a Crucible-owned Verus-Rust YAML implementation; `serde_yaml` must not be used,
- SQLite,
- tracing,
- anyhow for application-level errors,
- thiserror for typed errors at unverified Rust library boundaries.

Verus, Rust, Z3, standard-library, and platform-toolchain identities are part of the build and
proof evidence. They must be pinned for CI and reproduction. The project should follow Verus
development rather than freezing permanently on an old subset: upgrades are expected to move
code out of the trusted-boundary ledger and into verified executable code.

Dependencies should be selected partly for how small and explicit a verification boundary they
permit. A dependency that cannot be verified is wrapped behind the narrowest possible
project-owned Verus interface with executable contract checks where boundary inputs are not
statically controlled.

Procedural macro expansion, including `async-trait`, must be retained or reproducibly generated
as review evidence. If Verus cannot verify the expanded async or dynamic-dispatch surface, that
surface is a registered external boundary around a verified state machine; the macro is not a
blanket exemption for adapter logic.

External integrations may include:

- AFL++,
- libFuzzer,
- cargo-fuzz,
- honggfuzz,
- ASan,
- UBSan,
- MSan,
- TSan,
- LeakSanitizer,
- Valgrind,
- Kani,
- CBMC,
- Verus,
- Lean,
- Coq,
- Dafny,
- TLA+,
- Alloy,
- external symbolic or concolic engines.

The core orchestration and evidence model should remain in Rust.

---

## 8. Repository Layout

Recommended repository structure:

```text
crucible/
├── Cargo.toml
├── DESIGN.md
├── README.md
├── LICENSE
├── crates/
│   ├── crucible-core/
│   ├── crucible-provenance/
│   ├── crucible-scenario/
│   ├── crucible-build/
│   ├── crucible-cli/
│   ├── crucible-config/
│   ├── crucible-yaml/
│   ├── crucible-executor/
│   ├── crucible-oracles/
│   ├── crucible-corpus/
│   ├── crucible-findings/
│   ├── crucible-minimize/
│   ├── crucible-scheduler/
│   ├── crucible-static/
│   ├── crucible-fuzz/
│   ├── crucible-property/
│   ├── crucible-model/
│   ├── crucible-differential/
│   ├── crucible-metamorphic/
│   ├── crucible-fault/
│   ├── crucible-concurrency/
│   ├── crucible-symbolic/
│   ├── crucible-mutation/
│   ├── crucible-repair/
│   ├── crucible-verify/
│   ├── crucible-agent/
│   ├── crucible-plugin/
│   ├── crucible-distributed/
│   ├── crucible-embedded/
│   ├── crucible-kernel/
│   ├── crucible-compiler/
│   ├── crucible-performance/
│   └── crucible-report/
├── adapters/
│   ├── cli/
│   ├── library/
│   ├── local-http/
│   └── state-machine/
├── schemas/
├── testdata/
│   └── targets/
├── examples/
├── tests/
└── docs/
```

These names express mandatory subsystem ownership boundaries, not an irrevocable requirement
that every boundary begin as a separately compiled crate. Before implementation, the workspace
must publish an allowed dependency graph that prevents cycles and prevents `crucible-core`
from becoming a behavior-bearing god crate. A subsystem may begin as a module or may be split
across multiple crates when Verus or Rust compilation boundaries require it; the capability
and its contracts must not be collapsed or omitted for packaging convenience.

---

## 9. Core Domain Types

### 9.1 IDs

```rust
pub struct TargetId(pub String);
pub struct TargetBuildId(pub String);
pub struct CampaignId(pub String);
pub struct ExperimentId(pub String);
pub struct ScenarioId(pub String);
pub struct ParticipantId(pub String);
pub struct RunId(pub String);
pub struct RunAttemptId(pub String);
pub struct FindingId(pub String);
pub struct FindingInstanceId(pub String);
pub struct PatchId(pub String);
pub struct OracleId(pub String);
pub struct EngineId(pub String);
pub struct ArtifactId(pub String);
pub struct EvidenceId(pub String);
pub struct ProofArtifactId(pub String);
```

IDs should be stable, serializable, and distinct from display names. Random IDs identify
records; content-derived IDs identify immutable artifacts and builds whenever their complete
identity inputs are available.

### 9.2 ArtifactRef

Persisted inputs and outputs should refer to immutable content-addressed artifacts rather
than host filesystem paths.

```rust
pub struct ArtifactRef {
    pub id: ArtifactId,
    pub size_bytes: u64,
    pub media_type: Option<String>,
}
```

Host paths may be accepted while loading configuration or importing a corpus, but the
referenced contents should be ingested before a replayable run is created.

### 9.3 Stimulus

A stimulus describes data delivered to the target. It is separate from scheduling, faults,
resource limits, and environment controls.

```rust
pub struct Stimulus {
    pub entries: Vec<StimulusEntry>,
}

pub enum StimulusEntry {
    Stdin(ArtifactRef),
    Argument(PortableOsValue),
    EnvironmentVariable {
        name: PortableOsValue,
        value: PortableOsValue,
    },
    File {
        path: VirtualPath,
        contents: ArtifactRef,
    },
    Structured {
        schema: SchemaIdentity,
        codec: CodecIdentity,
        value: ArtifactRef,
    },
    Stream {
        channel: ChannelId,
        frames: Vec<StimulusFrame>,
    },
    PacketSequence(Vec<PacketStimulus>),
    ActionSequence(Vec<Action>),
}
```

`VirtualPath` must be relative, normalized, and incapable of escaping the run sandbox.
Stimulus environment variables are generated input channels; `ExecutionControls.environment`
defines the controlled base environment needed to run the target. The adapter must resolve
conflicts deterministically and record the effective environment.
Adapters may define additional typed stimulus entries through a versioned extension field.
Avoid an unlabelled `Composite(Vec<...>)`; every channel must have explicit delivery
semantics.

`PortableOsValue` must preserve the original byte or wide-character representation and its
platform encoding rather than forcing every argument, environment value, or path through
UTF-8. Adapters must validate platform-invalid values such as embedded NULs without losing the
original stimulus evidence. Structured stimuli are stored in their declared codec rather than
in `serde_json::Value`, which cannot losslessly represent every syntax, numeric spelling, map,
or duplicate-key behavior that a target may consume.

The schema must define multiplicity and ordering for each channel. Duplicate stdin entries,
environment names, file paths, and channel identifiers are invalid unless the selected adapter
explicitly defines their composition semantics.

### 9.4 Execution Controls

```rust
pub struct ExecutionControls {
    pub timeout: Duration,
    pub resource_limits: ResourceLimits,
    pub environment: BTreeMap<String, String>,
    pub network_policy: NetworkPolicy,
    pub schedule: Option<ThreadSchedule>,
    pub fault_plan: Option<FaultPlan>,
    pub perturbations: Vec<EnvironmentPerturbation>,
}
```

`NetworkPolicy` must support, at minimum:

- no network namespace or network access,
- loopback-only communication,
- an isolated virtual network shared only by named scenario participants,
- explicitly allowlisted endpoints,
- recorded proxy or fault-injection mediation.

A boolean network setting may remain configuration shorthand but is not the persisted policy.

The effective controls, including defaults supplied by the host, must be persisted with the
run. A seed is not a substitute for recording a generated schedule or applied fault trace.

### 9.5 Raw Execution Outcome

The executor records facts. It does not decide whether those facts constitute a target bug.

Termination and detected conditions are separate because one execution may time out, exceed a
resource threshold, emit sanitizer events, and then be killed by the harness. Platform-native
termination evidence must not be forced into Unix signal semantics.

```rust
pub struct RawExecutionOutcome {
    pub completion: CompletionDisposition,
    pub termination: Option<TerminationRecord>,
    pub events: Vec<RawExecutionEvent>,
}

pub enum CompletionDisposition {
    Completed,
    Cancelled,
    Incomplete,
}

pub enum TerminationRecord {
    ExitCode { code: i64 },
    UnixSignal { signal: i32, core_dumped: bool },
    WindowsException { status: u32 },
    EmbeddedReset { cause: ResetCause },
    HarnessTerminated { reason: HarnessTerminationReason },
    PlatformSpecific(VersionedExtensionRef),
}

pub enum RawExecutionEvent {
    TimeoutThresholdReached,
    ResourceThresholdReached { resource: ResourceKind },
    DeadlockSuspected,
    LivelockSuspected,
    WatchdogTriggered,
    ProcessCreated { logical_process: LogicalProcessId },
    ProcessExited { logical_process: LogicalProcessId },
    PlatformSpecific(VersionedExtensionRef),
}
```

Raw execution outcomes use canonical binary serialization version 1. The byte order is big-endian
and the wire shape is exact: four-byte ASCII magic `CRXO`; a `u16` schema version; a stable `u16`
completion tag; a `u64` event count; a one-byte termination option; the optional termination; then
the declared events in recorded order. Every enum uses its documented stable numeric tag. Signed
integers use their fixed-width two's-complement bit pattern, booleans and options accept only zero
or one, and strings are length-prefixed UTF-8. Extension records serialize the complete namespace,
schema version, artifact size, algorithm-qualified artifact ID, and optional media type.

Version-1 decoding is total over bytes and configured limits. It checks the absolute/caller-lowered
encoded-byte and event-count caps before constructing nested records; checks every declared length
against both remaining input and its applicable cap before allocation; rejects truncation, trailing
bytes, unknown tags, invalid boolean/option spellings, invalid UTF-8, oversized strings, unsupported
schema versions, and semantic-invalid decoded outcomes with distinct typed errors; and retains the
exact original bytes on every rejection. Future schema versions are therefore preserved as opaque
bounded bytes rather than relabeled as a current Rust value. Version-1 encoding is capped at 128
MiB. Decoding an encoding produced for a validated representable outcome must return the exact same
outcome.

Outcome validation has independent, caller-lowerable absolute limits for 1,048,576 events,
1,048,576 aggregate extension-namespace code points, 1,048,576 aggregate inline media-type code
points, and a 1 TiB out-of-line extension payload per record. Event count has cheap preflight
precedence. Payload size is not summed across records because identical content-addressed artifacts
may be referenced repeatedly; execution controls may lower the per-record policy. Namespace and
media-type budgets are aggregate because their bytes are inline. Exact limit, first-excluded, and
multi-defect precedence are part of the public diagnostic contract.

Failure to prepare the sandbox, spawn the target, capture required evidence, or persist the
observation is a typed harness error, not a target outcome.

### 9.6 RawObservation

```rust
pub struct CapturedStreamRef {
    pub artifact: ArtifactRef,
    pub truncated: bool,
    pub retained_bytes: u64,
    pub discarded_bytes: u64,
}

pub struct RawObservation {
    pub run_id: RunId,
    pub attempt_id: RunAttemptId,
    pub outcome: RawExecutionOutcome,
    pub stdout: CapturedStreamRef,
    pub stderr: CapturedStreamRef,

    pub wall_time: Duration,
    pub cpu_time: Option<Duration>,
    pub peak_rss_bytes: Option<u64>,

    pub resources: ResourceSnapshot,
    pub coverage: Option<CoverageRef>,

    pub state_digest: Option<StateDigest>,
    pub schedule_trace: Option<ScheduleTrace>,
    pub fault_trace: Option<FaultTrace>,

    pub extensions: Vec<VersionedExtensionRef>,
}
```

Sanitizer events, assertion events, normalized stack traces, and oracle verdicts are derived
records that reference this immutable observation. Preserve the raw streams even when a
parser successfully extracts structured events. Extension records must declare a namespace
and schema version and remain subject to configured size limits; an unversioned arbitrary
metadata map must not become a compatibility escape hatch.

`Duration` in this persisted record is a portable `(u64 seconds, u32 nanoseconds)` value with
nanoseconds strictly below one billion. A captured stream's retained byte count must equal its
artifact size, and `truncated` is true exactly when one or more bytes were discarded. The portable
resource snapshot contains optional process, thread, open-file, handle, read-byte, and written-byte
counters plus versioned resource extensions for platform-native measurements; a missing counter is
not silently replaced with zero. Coverage retains a typed `CoverageProviderId`, provider-version identity,
target-build identity, feature-space digest, out-of-line artifact, and new/total feature counts.
State digests, schedule traces, and fault traces are namespace-identified, versioned
artifact-backed records; schedule traces record decision count/completeness, while fault traces
separately count reached, applied, skipped, shadowed, and rejected events. A bare numeric trace
version without a namespace is not a portable schema identity.

Raw observations use canonical binary serialization version 1 with four-byte magic `CROB`, a
big-endian `u16` schema version, length-prefixed UTF-8 identities, a length-delimited canonical
`RawExecutionOutcome`, complete stream/resource/coverage/state/schedule/fault fields, and ordered
resource and observation extensions. Integers are fixed-width big-endian and every option/boolean
accepts only zero or one. Decoding enforces a 128 MiB absolute/caller-lowered byte cap before nested
work, preflights resource-extension and observation-extension counts before allocating records,
passes the nested outcome through its own bounded decoder, semantically validates the complete
observation, rejects truncation/trailing bytes/invalid UTF-8/options/durations with typed exact
offsets, and retains the exact encoded bytes on every rejection including future schemas.

The exact version-1 inner grammar is below. `string` is `u64 byte_length || byte_length bytes of
canonical UTF-8`; `option<T>` is `u8 0` or `u8 1 || T`; `sequence<T>` is `u64 count || T[count]`;
all integers are unsigned big-endian unless the referenced nested format says otherwise.

```text
observation = "CROB" || u16(1)
              || string(run_id) || string(attempt_id)
              || u64(outcome_bytes.len) || outcome_bytes
              || stream(stdout) || stream(stderr)
              || duration(wall_time) || option<duration>(cpu_time)
              || option<u64>(peak_rss_bytes)
              || resources
              || option<coverage> || option<state>
              || option<schedule> || option<fault>
              || sequence<extension>(extensions)

artifact  = u64(size_bytes) || string(algorithm_qualified_id)
            || option<string>(media_type)
stream    = artifact || bool(truncated) || u64(retained_bytes) || u64(discarded_bytes)
duration  = u64(seconds) || u32(nanoseconds)
extension = string(namespace) || u32(schema_version) || artifact
resources = option<u64>(process_count) || option<u64>(thread_count)
            || option<u64>(open_file_count) || option<u64>(handle_count)
            || option<u64>(read_bytes) || option<u64>(written_bytes)
            || sequence<extension>(resource_extensions)
coverage  = string(provider_id) || string(provider_version) || string(target_build_id)
            || string(feature_set_digest) || artifact
            || u64(new_features) || u64(total_features)
state     = string(namespace) || u32(schema_version) || artifact
schedule  = string(namespace) || u32(schema_version) || artifact
            || u64(decisions) || bool(complete)
fault     = string(namespace) || u32(schema_version) || artifact
            || u64(reached) || u64(applied) || u64(skipped)
            || u64(shadowed) || u64(rejected) || bool(complete)
```

Observation admission has independent caller-lowerable absolute caps for 65,536 aggregate identity
code points, 65,536 resource extensions, 1,048,576 top-level extensions, 1,048,576 aggregate
extension-namespace code points, 1,048,576 aggregate inline media-type code points across every
observation-owned artifact reference, and 1 TiB per out-of-line artifact record. Caller policy
cannot raise those absolutes. Every declared string length is checked against the applicable
remaining identity, namespace, or media budget before UTF-8 materialization. Success proves exact
input identity and the public semantic predicate; codec success proves the same predicate, and
codec rejection proves byte-for-byte input retention. The executable version-1 encoder/decoder is
also frozen by a complete golden-byte fixture; a pure exact byte-to-value correspondence model
remains required before the codec may be described as input-authenticating in the formal layer.

### 9.7 Evidence and provenance graph

```rust
pub struct EvidenceNode {
    pub id: EvidenceId,
    pub kind: EvidenceKind,
    pub payload: ArtifactRef,
    pub schema: SchemaIdentity,
    pub producer: ProducerIdentity,
    pub created_at: UtcTimestamp,
}

pub struct ProvenanceEdge {
    pub subject: EvidenceId,
    pub object: EvidenceId,
    pub relation: ProvenanceRelation,
    pub transformation: Option<TransformationIdentity>,
    pub actor: ActorIdentity,
    pub recorded_at: UtcTimestamp,
}

pub enum TransformationConfiguration {
    NoneDeclared,
    Artifact(ArtifactRef),
}
```

`UtcTimestamp` is represented portably as signed Unix-epoch seconds plus a nanosecond field that
must be less than 1,000,000,000. `SchemaIdentity` contains a namespace, schema name, and version.
`ProducerIdentity` binds an actor, producer version, and optional immutable implementation artifact.
`TransformationIdentity` binds its name and version to an immutable implementation and explicit
configuration state. The absence of a configuration artifact is a `NoneDeclared` value, not an
ambiguous missing field. Canonical configuration lowering validates namespace, name, and version
policies before these records are admitted.

Timestamp fields are construction-only: callers cannot bypass the nanosecond range check with a
struct literal. Graph admission independently validates every record, including nonempty evidence,
schema, actor, producer, and transformation identity fields; nonzero schema versions; canonical
algorithm-labeled artifact identities; nonempty declared media types; and valid timestamps. A
record assembled through public mutable domain fields is not trusted merely because it has the
right Rust type. Validation failures are typed, name the rejected field, preserve the graph, and do
not consume the caller's publication payload.

`EvidenceKind` is a non-exhaustive versioned enum covering source/build evidence, original and
derived observations, oracle verdicts, findings, reproducers, minimizations, hypotheses, candidate
patches, verification/proof results, trusted-boundary audits, decisions, reports, and explicitly
versioned extensions. `ProvenanceRelation` contains every relationship required by Section 5.12.
Every public kind, relation, outcome, and validation/error enum is non-exhaustive. Internally, kind
and relation equality uses exhaustive stable-tag mappings: adding a variant fails compilation or
verification until equality, classification, and tests are deliberately updated.

Edges use the normative assertion direction `subject --relation--> object`:

- `DerivedFrom`: the derived subject was computed from the input object;
- `GeneratedBy`: the output subject was generated by the invocation/build object;
- `Evaluates`: the evaluation or verdict subject evaluates the evidence object;
- `Supports` and `Contradicts`: the subject supports or contradicts the object;
- `Reproduces`: the reproduction subject reproduces the finding or observation object;
- `Minimizes`: the minimized subject minimizes the larger source object;
- `Verifies` and `Invalidates`: the subject verifies or invalidates the object;
- `Supersedes`: the newer subject supersedes the older object.

Direction is part of canonical edge identity. Producers must not reverse endpoints based on storage
or traversal convenience.

Evidence nodes and provenance edges are immutable. Current finding state, representative
instances, confidence, clusters, and reports are projections that may be rebuilt from the
graph plus append-only decisions. Graph insertion must be idempotent so native engines,
resumed workers, and distributed workers can safely retry publication.

An idempotent node retry is field-identical to the stored node. Reusing an `EvidenceId` with any
different immutable field is a typed conflict and must never overwrite the original. An edge is
accepted only after both endpoint nodes exist. Retrying the exact edge is a no-op; edges that share
endpoints but differ in relation, transformation, actor, or recorded time remain distinct evidence.
Node and edge insertion borrow their payload and clone only after successful admission, so a typed
failure leaves the exact timestamped value available for a distributed or resumed-worker retry.

`SourceSnapshot` and `OriginalObservation` are primary evidence kinds. Every other current kind,
including versioned extensions, is conservatively classified as derived evidence. Ordinary node
insertion rejects derived evidence. `publish_derivation` atomically admits a structurally valid
derived node together with a nonempty, duplicate-free batch of declared input edges. Every input
edge must be directed from the new node to an existing input node, use `DerivedFrom`, carry a valid
transformation identity, and identify the same actor as the node's producer; the derived producer
must identify an immutable implementation artifact. The verified graph invariant requires every
admitted derived node to retain at least one such edge, while the atomic batch records all inputs
declared for that publication. Exact retries are no-ops; incomplete or divergent retries are typed
failures and never create a partially admitted derivation.

### 9.8 Verification and trusted-boundary evidence

Every crate and produced binary must publish:

- Verus source and proof artifact identity,
- Verus, solver, Rust, linker, and standard-library identities,
- verification result and resource limits,
- the set of specifications claimed,
- trusted-boundary ledger entries reachable from the artifact,
- unresolved proof gaps and approved temporary assumptions,
- a digest connecting verified source to the executable build.

A green proof result without the proved specification and assumption set is not sufficient
evidence. Proof success, timeout, resource exhaustion, unsupported feature, and counterexample
are distinct outcomes.

---
