# Crucible

## Universal Software Bug-Finding, Triage, Repair, and Verification Harness

**Status:** Draft Implementation Specification<br>
**Primary implementation language:** Verus Rust, compiled as Rust<br>
**Audience:** coding agents, software engineers, verification engineers, fuzzing engineers<br>
**Purpose:** discover and fix software defects across as many bug classes as practical<br>
**Non-goal:** exploit generation, offensive security automation, intrusion, persistence, weaponization

---

## 1. Executive Summary

Crucible is a generic software correctness harness intended to find defects in software that the operator owns or is authorized to test.

Its purpose is to:

1. discover software defects,
2. reproduce them reliably,
3. classify them,
4. minimize them into small counterexamples,
5. identify the violated invariant or likely root cause,
6. generate or assist with candidate repairs,
7. aggressively verify those repairs,
8. convert fixed bugs into permanent regression knowledge.

Crucible is not an exploit framework.

A discovered defect is treated as:

> evidence that an expected property of the software is false.

The harness should combine many complementary techniques rather than relying on one fuzzer or one static analyzer.

The complete product must include:

- coverage-guided fuzzing,
- structure-aware fuzzing,
- property-based testing,
- metamorphic testing,
- differential testing,
- stateful testing,
- model-based testing,
- fault injection,
- sanitizer integration,
- static analysis integration,
- symbolic/concolic execution integration,
- deterministic concurrency schedule exploration,
- race and deadlock detection,
- mutation testing,
- environment perturbation,
- temporal testing,
- soak testing,
- regression generation,
- failure deduplication,
- test-case minimization,
- root-cause assistance,
- candidate patch generation,
- adversarial patch verification,
- formal-verification integration,
- persistent corpus management,
- reproducible evidence bundles.

The central design model is:

```text
hypothesis
    ↓
experiment
    ↓
observation
    ↓
oracle
    ↓
finding
    ↓
reproduction
    ↓
minimized counterexample
    ↓
root-cause hypothesis
    ↓
candidate repair
    ↓
verification gauntlet
    ↓
regression artifact
```

The system should progressively transform undocumented assumptions into executable knowledge:

```text
unknown assumption
    ↓
bug
    ↓
counterexample
    ↓
regression test
    ↓
property
    ↓
contract
    ↓
proof
```

Not every defect will reach the proof stage, but every confirmed defect should leave the software easier to reason about than before.

---

## 2. Mission

Crucible exists to maximize the probability of finding correctness defects before users encounter them.

It should optimize for:

- unique confirmed defects found,
- breadth of bug classes covered,
- reproducibility,
- counterexample quality,
- root-cause usefulness,
- regression prevention,
- patch verification strength,
- efficient use of compute.

It should not optimize for raw crash count.

One defect reproduced 100,000 times is still one defect.

---

## 3. Authorized Scope

Crucible may test software that the operator owns or is authorized to test, including:

- command-line applications,
- libraries,
- parsers,
- compilers,
- interpreters,
- databases,
- storage engines,
- state machines,
- serialization formats,
- local services,
- local HTTP APIs,
- embedded software,
- concurrent applications,
- distributed-system simulations,
- mathematical libraries,
- protocol implementations,
- language runtimes,
- developer tools,
- kernel components in controlled test environments.

The system should default toward local, isolated testing.

---

## 4. Explicit Non-Goals

Crucible must not contain features whose purpose is:

- exploit generation,
- shellcode generation,
- remote intrusion,
- persistence,
- credential theft,
- generic authentication bypass automation,
- privilege-escalation automation,
- exploit-chain construction,
- post-exploitation tooling,
- stealth or evasion,
- internet-wide scanning,
- destructive payload generation,
- vulnerability weaponization.

A defect with security implications is still treated as a correctness defect.

The harness should answer:

> What failed?<br>
> Why did it fail?<br>
> What invariant should have held?<br>
> What is the smallest reproducer?<br>
> Does the proposed fix restore correctness?

It should not attempt to answer:

> How can this defect be weaponized?

---

## 5. Design Principles

### 5.1 Evidence over speculation

Every reported defect should be backed by machine-generated evidence.

Preferred evidence includes:

- immutable target-build identity,
- source revision and dirty-source digest where available,
- compiler, toolchain, sanitizer, and build configuration,
- test input,
- environment,
- random seed,
- execution output,
- sanitizer events,
- stack trace,
- coverage,
- schedule trace,
- fault-injection plan,
- reproduction command,
- oracle that failed.

### 5.2 Reproduction evidence is mandatory

Every dynamic finding must be replayed under a defined policy. A finding that is never
observed again should remain a candidate or become non-reproducible rather than being called
confirmed. An intermittently reproduced race, temporal failure, or distributed-state failure
may still be confirmed when the failed oracle and attribution to the target are strong; its
report must state the observed rate and must not call the behavior deterministic.

Every confirmed defect should have a reproduction bundle.

### 5.3 Every bug is a violated property

Whenever possible, convert a discovered defect into:

1. a minimized reproducer,
2. a regression test,
3. an explicit property,
4. an explanatory invariant,
5. a formal contract or proof obligation when the target's configured proof systems can express
   the property, with a recorded attempted formalization otherwise.

### 5.4 Every patch is guilty until proven innocent

A candidate repair must pass a verification gauntlet.

Making the original reproducer stop failing is necessary but insufficient.

### 5.5 Portfolio testing beats monoculture

No single technique finds all classes of bugs.

Crucible should combine:

- dynamic testing,
- fuzzing,
- property testing,
- stateful testing,
- static analysis,
- symbolic exploration,
- concurrency exploration,
- mutation testing,
- fault injection,
- model checking,
- formal verification.

### 5.6 Shared evidence model

Every bug-finding engine should emit results through common normalized interfaces.

### 5.7 Determinism with explicit limits

Every stochastic engine should accept and record a seed.

Every failure should retain enough information to replay the original experiment.

### 5.8 Safe defaults

Default execution policy:

- network disabled,
- bounded resources,
- temporary working directory,
- explicit target configuration,
- controlled writable paths,
- local targets,
- no shell interpolation of target-controlled output.

### 5.9 Verified implementation saturation

Crucible is a Verus-Rust project. Verification is not reserved for algorithms that are
conventionally considered worth proving. Executable harness code must be written inside the
Verus-supported Rust subset whenever the current pinned Verus toolchain can express it, even
when the immediate proof obligation is simple or the assurance gain appears incremental.

The reason for this bias is that the implementation may be produced or modified substantially
by coding agents. Requiring specifications, proof obligations, and explicit trusted boundaries
provides a machine-checked constraint on generated code and a durable review surface.

Every executable function should therefore have, as applicable:

- explicit preconditions and postconditions,
- representation invariants,
- loop invariants and termination arguments,
- arithmetic range and overflow proofs,
- ownership, lifecycle, and state-transition properties,
- correspondence to a pure specification function,
- proofs that parsing, normalization, scheduling, and persistence transformations preserve
  their declared meaning.

Ordinary Rust outside the verified subset is permitted only when Verus cannot currently
express or compile the required feature, an external dependency or operating-system interface
forces the boundary, or generated bindings cannot be replaced. Convenience, proof effort, or
low perceived risk is not by itself an exemption.

### 5.10 Trusted-boundary accounting

Every use of an unverified or assumed boundary, including `assume`, `external_body`,
`external`, `assume_specification`, unsafe foreign code, solver axioms, generated bindings,
and trusted executable specifications, must create an entry in a versioned trusted-boundary
ledger.

Each entry must record:

- source location and owning component,
- the reason the boundary is currently required,
- the exact assumption made,
- the property that callers rely upon,
- tests or independent checks covering the boundary,
- reviewer and approval evidence,
- upstream Verus limitation or dependency issue where applicable,
- a review deadline or toolchain version that triggers reconsideration.

CI must reject unregistered assumptions and unapproved increases in trusted-computing-base
surface. Each Verus toolchain upgrade must run a boundary-reduction audit. Code that has become
expressible in Verus must be migrated into the verified subset rather than remaining external
by inertia.

Track verified executable lines, specified external lines, assumptions, admitted axioms, and
proof coverage as separate metrics. No aggregate percentage may hide a high-consequence
unverified boundary.

### 5.11 Requirement language

The terms **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are normative. **SHOULD** requires a
recorded reason when not followed. “Where supported” means that the capability manifest has
machine-detected a concrete unavailable capability; it is not a discretionary scope escape.

Release staging changes delivery order, not the committed product scope. Features assigned to
later phases remain requirements unless this specification is explicitly amended.

### 5.12 Append-only evidence and provenance

Crucible's primary data structure is an append-only evidence graph. The familiar defect
lifecycle is an important view of that graph, but evidence may branch, converge, contradict
other evidence, or exist without a confirmed target defect.

Required provenance relationships include:

```text
derived-from   generated-by   evaluates   supports   contradicts
reproduces     minimizes      verifies    invalidates supersedes
```

All derived facts must identify their input artifacts, implementation and schema versions,
configuration, actor, and transformation. Mutating a convenient current-state view must never
erase the historical graph from which it was derived.

Derived facts MUST be admitted atomically with their declared input provenance. A staging API MAY
exist later, but staged records are not admitted evidence and MUST NOT satisfy graph completeness,
reporting, or decision gates until their node and input edges commit as one append-only transition.

---

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

## 10. Target Interface

All targets should normalize behind a common abstraction, but a configured target name must
not stand in for an immutable build identity.

```rust
pub struct TargetBuild {
    pub id: TargetBuildId,
    pub target: TargetId,
    pub source_revision: Option<String>,
    pub dirty_source_digest: Option<String>,
    pub primary_executable: Option<ArtifactRef>,
    pub runtime_artifacts: Vec<ArtifactRef>,
    pub identity_digest: String,
    pub build_manifest: ArtifactRef,
    pub toolchain: ToolchainIdentity,
    pub platform: PlatformIdentity,
    pub source_snapshot: SourceSnapshotId,
    pub build_recipe: BuildRecipeId,
    pub build_execution: BuildExecutionId,
    pub instrumentation: Vec<InstrumentationIdentity>,
    pub symbols: Vec<ArtifactRef>,
    pub proof_artifacts: Vec<ProofArtifactId>,
}
```

The build manifest should record compiler and linker flags, enabled sanitizers, relevant
dependencies, runtime configuration, and other inputs needed to distinguish behaviorally
different builds. `identity_digest` should cover a canonical build manifest and all declared
runtime artifacts. Targets such as configured services may have no embedded executable but
must still provide an immutable version/configuration identity suitable for comparison.

Source, build, deployment, and runtime identity are separate records:

```text
SourceSnapshot
    ↓ built by
BuildRecipe + BuildEnvironment
    ↓ produces
BuildExecution + Deployable Artifact Set + Proof Artifacts
    ↓ installed by
Deployment Recipe
    ↓ creates
Target Deployment in a Runtime Environment
```

The build recipe records commands as typed argv arrays, declared inputs and outputs, dependency
locks, environment, network policy, and expected toolchains. The build execution records all
effective values, logs, exit outcomes, generated artifacts, and undeclared-input detections.
Hermetic and reproducible builds are preferred, but a non-hermetic build is still representable
and must disclose its unresolved inputs rather than claiming reproducibility.

Runtime identity includes the Crucible build, adapter and plugin versions, operating system and
kernel, CPU architecture and relevant feature flags, container or VM image, filesystem and
mount configuration, loaded dynamic libraries, locale, timezone, and other behaviorally
relevant state available to the backend.

Preparation creates an exclusively owned instance. A single prepared mutable target must not
be reset concurrently with another run.

```rust
#[async_trait]
pub trait TargetAdapter: Send + Sync {
    fn id(&self) -> TargetId;

    async fn prepare(
        &self,
        build: &TargetBuild,
        context: &PrepareContext,
    ) -> Result<Box<dyn TargetInstance>, HarnessError>;
}

#[async_trait]
pub trait TargetInstance: Send {
    async fn execute(
        &mut self,
        request: &RunRequest,
        context: &ExecutionContext,
    ) -> Result<RawObservation, HarnessError>;

    async fn reset(
        &mut self,
    ) -> Result<(), HarnessError>;

    async fn cleanup(
        &mut self,
    ) -> Result<(), HarnessError>;
}
```

The coordinator owns the instance lifecycle, guarantees cleanup, and discards an instance
whose state cannot be reset with confidence. Stateless CLI runs may use a fresh instance per
run. Stateful and persistent-mode adapters may pool instances explicitly.

Supported initial target adapters:

### CLI Target

Executes a binary.

Input delivery modes:

- stdin,
- generated relative file,
- argument,
- environment variable.

### Library Target

Calls a library function through:

- Rust API,
- C ABI,
- generated helper executable.

### Local HTTP Target

Tests explicitly configured local or authorized services.

Input dimensions may include:

- request path,
- method,
- headers,
- body,
- action sequence.

### Stateful Target

Represents systems such as:

- databases,
- caches,
- transactional services,
- protocol state machines.

Input is an action sequence.

### Scenario / Service-Topology Target

Coordinates multiple named processes, services, clients, network links, storage devices, and
observers through the scenario graph. It is the foundation for distributed-system and complex
integration testing.

### Virtual-Machine / Kernel Target

Boots a versioned VM image, restores a clean snapshot, injects guest stimuli, captures serial
output and crash dumps, and tears down the complete VM. Kernel targets must not execute with
host-kernel privilege merely because the process sandbox supports privileged operations.

### Emulator / Embedded Target

Runs firmware under an emulator or simulator, or through an explicitly configured
hardware-in-the-loop controller. It supports reset causes, watchdogs, serial/debug transports,
flash and nonvolatile state, virtual peripherals, power interruption, and deterministic clock
or interrupt control where the backend exposes them.

### Compiler / Toolchain Target

Models a compile pipeline followed by optional execution, interpretation, disassembly, IR
inspection, or proof. It preserves source-language validity assumptions and distinguishes
compiler crashes, invalid diagnostics, miscompilations, nondeterministic artifacts, and
disagreements that do not yet establish blame.

---

## 11. Execution Isolation

Buggy software must be treated as untrusted with respect to harness stability.

Every execution backend must publish a capability manifest and should support:

- wall-clock timeout,
- CPU limit,
- memory limit,
- process-count limit,
- file-size limit,
- bounded stdout/stderr capture,
- isolated temporary working directory,
- controlled environment variables,
- controlled writable paths,
- network disabled by default,
- deterministic seed where possible.

Isolation guarantees are platform-specific. The core must not encode one operating system or
CPU architecture as the universal execution model. The initial implementation should provide
platform backends for the environments where Crucible is built and tested, including Linux,
macOS, and Windows where CI capacity is available.

```text
Linux:
  process groups, rlimits, namespaces, cgroups, seccomp, containers where available

macOS:
  process groups, resource limits, application sandbox profiles, VMs where required

Windows:
  job objects, restricted tokens, resource controls, isolated working directories
```

Architecture variants should include x86_64 and AArch64 where toolchains and CI support them.
The same evidence contract applies across platform backends so cross-platform behavior can
participate in differential experiments.

Isolation tiers are explicit:

```text
Tier 0: direct process with resource accounting
Tier 1: operating-system sandbox
Tier 2: container or equivalent isolated userspace
Tier 3: disposable VM or microVM
Tier 4: emulator, simulator, or controlled hardware-in-the-loop lab
```

Projects declare the minimum tier and individual required capabilities. Kernel, privileged
runtime, hostile build, and untrusted candidate-patch workloads default to Tier 3 or stronger.
Sanitizer and fuzzer runtimes are detection tools, not security boundaries.

For every run, record which controls were requested, which were successfully enforced, and
which were unavailable. A run must not claim that networking or filesystem access was
isolated when the host could not enforce that policy. Projects may require particular
capabilities and fail closed when they are absent.

### 11.1 Capability manifest

```rust
pub struct CapabilityManifest {
    pub harness_build: HarnessBuildId,
    pub backend: BackendIdentity,
    pub platform: PlatformIdentity,
    pub capabilities: BTreeMap<CapabilityId, CapabilityStatus>,
    pub evidence: Vec<EvidenceId>,
}

pub enum CapabilityStatus {
    Enforced { mechanism: MechanismIdentity },
    AvailableButDisabled { reason: String },
    Degraded { limitations: Vec<String> },
    Unavailable { reason: String },
    NotApplicable { reason: String },
}
```

Capability detection is executable evidence and should be independently self-tested. Campaign
planning resolves every required capability before expensive preparation. A later runtime
failure to enforce an advertised capability invalidates affected attempts and creates a harness
finding.

The threat model must distinguish trusted project configuration from less-trusted target
binaries, build scripts, candidate patches, and fully untrusted target input and output.
Building or verifying a candidate patch executes project-controlled code and therefore
requires a separately declared build policy.

Secrets needed by an authorized target must be represented by opaque secret references. The
artifact store, logs, reports, agent packets, reproduction bundles, and external integrations
must apply field-aware redaction and must record that redaction occurred. Secret values must
not become corpus mutation material unless the project explicitly declares a synthetic test
secret safe for retention.

Target-controlled data must never be directly interpolated into shell commands.

Prefer direct process APIs over shell invocation.

When stdout or stderr reaches its capture limit, the runner must follow an explicit policy:
continue draining while discarding excess bytes, or terminate the target. Merely stopping the
read can deadlock a child on a full pipe. Record truncation, retained byte count, and discarded
byte count in the raw observation. Timeout and cancellation must act on the configured
process tree, not only the immediate child.

---

## 12. Configuration Format

Example `crucible.yaml`:

```yaml
version: 1

language:
  profile: crucible-yaml-1

project:
  name: example-parser

target:
  adapter: cli
  command: ./build/example-parser
  args:
    - "{input_file}"

execution:
  timeout_ms: 2000
  memory_mb: 1024
  max_processes: 32
  max_output_mb: 16
  network: false
  required_capabilities:
    - process_group_termination
    - resource_limits

oracles:
  process_exit:
    allowed_codes: [0]
    timeout_is_failure: true

inputs:
  corpus:
    - ./seeds/

engines:
  fuzz:
    enabled: true
    modes:
      - managed
      - native
    native_backends:
      - afl++
      - libfuzzer
      - honggfuzz

  property:
    enabled: true

  differential:
    enabled: false

  metamorphic:
    enabled: true

  fault:
    enabled: true

  concurrency:
    enabled: false

  symbolic:
    enabled: false

  mutation:
    enabled: false

sanitizers:
  address: true
  undefined: true
  thread: false
  memory: false
  leak: true

campaign:
  duration: 8h
  workers: 8
  seed: 123456789

storage:
  root: .crucible

verification:
  verus:
    required: true
    deny_unregistered_assumptions: true
    deny_unapproved_tcb_growth: true
```

### 12.1 Crucible YAML

Crucible owns its configuration language and implementation. `crucible-yaml` is a
YAML-compatible, versioned configuration language written in Verus Rust; it must not wrap or
delegate parsing to `serde_yaml` or another general YAML parser.

The implementation pipeline is:

```text
untrusted bytes
    ↓ verified UTF decoding or byte-level diagnostic
normalized decoded scalars with exact source spans
    ↓ verified lexical atomization
scalar-preserving lexical atoms
    ↓ verified line and indentation layout
exact line descriptors over atom ranges
    ↓ verified context-sensitive lexer
tokens with exact spans
    ↓ verified parser
concrete syntax tree
    ↓ alias, tag, and scalar resolution
semantic YAML graph
    ↓ duplicate rejection, merge application, and canonical graph lowering
alias-transparent canonical YAML DAG
    ↓ schema-directed typed-field lowering
typed Crucible configuration
    ↓ schema-version, unknown-field, and cross-field validation
verified effective configuration
```

Each arrow is a named, versioned transformation with an executable implementation, a pure
Verus specification, and correspondence proofs wherever the pinned Verus toolchain can express
them. The canonical lowered representation is the authoritative input to campaign identity,
not parser-internal objects or host paths.

Crucible YAML profile 1 must specify rather than inherit ambiguous implementation behavior.
At minimum it defines:

- UTF-8 handling, optional byte-order mark behavior, line endings, and source spans,
- indentation, plain/quoted/block scalars, comments, sequences, and mappings,
- numeric grammar and checked conversion without host-width dependence,
- explicit boolean and null spellings without YAML 1.1 implicit coercions,
- anchors, aliases, tags, and merge behavior, including expansion and recursion limits,
- duplicate-key rejection after canonical key resolution,
- deterministic map ordering for hashing while preserving source order for diagnostics,
- maximum depth, token count, alias expansion, scalar size, and total decoded size,
- unknown-field policy, deprecation handling, and schema-version negotiation,
- diagnostic stability and recovery behavior,
- canonical serialization used for configuration and campaign digests.

Profile 1's byte-decoding transformation is version 1. It accepts only shortest-form UTF-8 that
decodes to Unicode scalar values, with distinct diagnostics for unexpected or invalid continuation
bytes, invalid leaders, truncation, overlong forms, surrogate code points, and values above
`U+10FFFF`. A leading UTF-8 byte-order mark is either explicitly forbidden or stripped under an
explicit `AllowAndStrip` policy; `U+FEFF` anywhere else remains data. CRLF and standalone CR are
normalized to LF. Every decoded scalar retains a half-open span over the original source bytes,
including both bytes of CRLF, with zero-based lines and scalar columns. A stripped BOM consumes
bytes 0 through 2 without incrementing the first scalar's line or column.

The version-1 absolute decoding limits are 16 MiB of source bytes and 1,048,576 decoded scalars.
Callers MUST provide per-operation limits and MAY lower either bound but cannot raise the profile
maximum. A larger accepted representation requires a new language-profile decision rather than a
platform-dependent allocation accident. These decoder limits do not replace the lexer, parser,
alias-expansion, depth, individual-scalar, or total semantic-value limits required below.

Diagnostic precedence for byte-decoding transformation version 1 is normative. The decoder first
applies the effective source-byte cap, then rejects a forbidden leading BOM, then applies the
decoded-scalar cap before attempting the next scalar. Within a scalar it classifies an intrinsically
invalid leader first. For a potentially valid multi-byte leader, it examines each available
continuation byte from left to right. An available non-continuation byte is reported at that byte.
After a valid second byte is present, the `E0`, `ED`, `F0`, and `F4` second-byte restrictions are
reported as overlong, surrogate, or out-of-range errors at the leader before inspecting any later
byte. Only when every available byte required so far is valid and another required byte is absent is
the sequence truncated.

Offsets are also normative: source-limit errors identify the first byte excluded by the effective
cap; scalar-limit errors identify the start of the next scalar; forbidden BOM, intrinsically
overlong, surrogate, and out-of-range errors identify the leader; continuation errors identify the
offending byte; and truncation identifies the end-of-input offset. The pure Verus transformation is
total over bytes, limits, and BOM policy. Its result fixes both every successful decoded source and
every error kind and offset, and executable correspondence is required for both result variants.

Profile 1's lexical-atom transformation is version 1 and is the verified context-free first stage
of lexing. It consumes only a successfully decoded source and produces exactly one atom for every
decoded scalar, preserving the code point and source span without alteration. It distinguishes LF,
space, tab, the YAML indicator characters `-?:,[]{}#&*!|>'"%@` plus the grave accent, and all other
content. Indicators retain distinct kinds, including the reserved at-sign and grave-accent kinds,
so a later stage can reject or interpret them using position and context rather than losing source
evidence. A stripped leading BOM has no atom; a non-leading `U+FEFF` is content.

The version-1 absolute lexical-atom limit is 1,048,576, equal to the decoded-scalar cap. Callers MAY
lower it but cannot raise it. Exceeding the effective limit returns a typed error at the first scalar
that would be excluded and constructs no partial public result. The pure Verus transformation is
total and fixes both the complete atom sequence and the exact limit error. Its exact-correspondence
predicate remains usable over arbitrary ghost views, while semantic atom-source validity also
requires a well-formed decoded source; downstream proofs can recover that obligation instead of
silently laundering forged decoded views. Lexical atomization does not replace context-sensitive
token formation: indentation, comments, directives, document markers, flow delimiters, and plain,
quoted, and block scalar boundaries remain mandatory verified lexer behavior.

Profile 1's line-layout transformation is version 1. It consumes lexical atoms and emits one exact
descriptor for every non-phantom source line: an empty atom stream, including a stripped-BOM-only
source, has no line; a final line feed terminates its line without creating an extra trailing empty
line; and a nonempty unterminated tail is its own line. Each descriptor records its zero-based line
number, half-open atom range excluding the optional line feed, first non-space atom, leading-space
width, termination state, and byte offsets for the line start, content-candidate start, and consumed
end. CR and CRLF already normalized by the decoder therefore remain one line-feed atom while their
original byte widths remain visible.

Only spaces contribute leading indentation columns. The layout stage MUST preserve a tab as the
first non-space atom rather than reject it: tab legality depends on context that this pre-pass does
not yet possess. In particular, a tab can be scalar content after required block-scalar indentation
or in a multiline flow scalar even though it cannot serve as structural indentation. The verified
context-sensitive token scanner therefore decides whether each preserved tab is scalar content,
valid separation, or an indentation violation and reports the contextual diagnostic. This deferral
is lossless and is not permission for the completed lexer to accept a tab where YAML forbids one.

The absolute profile caps are 1,048,576 line descriptors and 4,096 leading-space columns per line.
Callers MAY lower either cap but cannot raise it. A line-limit error identifies the first atom of the
first excluded line, and an indentation-limit error identifies the first excluded leading space. A
defensive atom-count check rejects a forged over-cap atom source before indexing it. Error precedence
is atom-count cap, then admission of a new line under the line cap, then left-to-right leading-space
validation within that admitted line. The transformation is iterative in executable code, total in
its pure Verus model, constructs no partial public result on error, and preserves a semantic validity
witness from the atomized input. It is a proof-carrying substage of the full lexer, not a substitute
for comments, directives, flow state, or scalar token formation.

Profile 1's structural-candidate transformation is version 1 and is a second proof-carrying lexer
substage. It authenticates its input by recomputing the canonical line layout, then partitions every
atom exactly once into nonempty, monotonic ranges for indentation, separation, line feeds,
directives, document markers, comments, flow punctuation, contextually plausible indicators, and
still-ambiguous content. Each range retains its line number, half-open atom indices, and original
half-open byte span. These roles are lossless candidates rather than completed YAML tokens: quoted,
plain, and block scalar context may reinterpret punctuation or separation candidates, and the full
context-sensitive scanner remains responsible for final scalar boundaries, flow state, reserved
indicators, and contextual tab legality. No atom or byte evidence may be discarded by that later
reinterpretation.

The absolute structural-candidate cap is 1,048,576, equal to the lexical-atom cap; callers MAY lower it
but cannot raise it. An exhausted cap reports the first byte of the first excluded lexeme and exposes
no partial public result. An empty or stripped-BOM-only atom stream requires zero lexeme capacity.
A layout that is not the canonical result for the supplied atom stream is rejected before scanning.
The executable scan is iterative, its pure Verus model is total under explicit fuel equal to the
input atom count, and exact correspondence covers success, layout mismatch, and limit failure. Its
semantic validity predicate retains both intrinsic atom validity and the authenticated layout
witness, so forged ghost views cannot be laundered into a valid structural source.

Profile 1's quoted-scalar boundary transformation is version 1 and is the first completed slice of
the verified context-sensitive token scanner. It authenticates both the canonical line layout and
canonical structural-candidate partition before interpreting any candidate. The structural stage
therefore retains every single- and double-quote atom as an explicit provisional candidate,
including quotes adjacent to JSON-style flow punctuation; this stage alone decides whether a quote
can begin a scalar. Its verified state tracks provisional plain-scalar continuation and block-scalar
header/body regions before updating flow depth, so quotes and provisional punctuation inside those
regions remain raw content. Quotes may begin at the stream or after separation, a line break, a flow
collection opener, a flow entry, or a mapping-value indicator inside a flow collection. This start
decision is provisional token context, not a waiver of the parser's separation and
collection-grammar obligations.

Each accepted quoted scalar includes both delimiters and records its style, starting and ending
line, half-open atom range, and exact half-open original-byte range. Single-quoted content escapes a
single quote only by doubling it. Double-quoted content recognizes YAML 1.2's complete simple escape
set, an escaped line break, and `x`, `u`, and `U` escapes with exactly two, four, and eight hexadecimal
digits respectively. Escaped values above `U+10FFFF` and surrogate code points are rejected rather
than converted. Unescaped content admits exactly YAML's printable character set: tab, line feed,
ASCII `0x20` through `0x7E`, `U+0085`, `U+00A0` through `U+D7FF`, `U+E000` through `U+FFFD`, and
`U+10000` through `U+10FFFF`. This transformation validates boundaries and escape spelling without
prematurely discarding raw presentation bytes; semantic decoding and flow-line folding remain
mandatory parts of scalar resolution.

The absolute quoted-scalar count and per-scalar atom caps are each 1,048,576. Callers MAY lower
either cap but cannot raise it. Scalar-count exhaustion reports the opening quote of the first
excluded scalar. Per-scalar exhaustion reports the first atom excluded from that scalar, counting
both delimiters. Invalid escape characters and hexadecimal digits report the offending atom;
invalid escaped code points report the initiating backslash; an unfinished escape or hexadecimal
sequence reports end of input; an unescaped forbidden character reports that character's original
byte offset; and an otherwise unterminated quote also reports end of input. All failures expose no
partial public result.

The executable quote scan and scalar-body machines are iterative. Their pure Verus models are total
under explicit structural-candidate and atom fuel, and exact correspondence covers success, input
authentication failure, both resource caps, all escape failures, and unterminated scalars. The
semantic validity predicate retains the authenticated atom, layout, and structural witnesses so
forged upstream ghost views cannot be laundered. Its public range predicate and theorem expose, for
every semantically valid result, nonempty bounded atom ranges, exact source byte and line endpoints,
opening and closing delimiters matching the recorded style, and ordered pairwise atom/byte
non-overlap. This completed slice does not replace or defer the committed plain-scalar,
block-scalar, comment, directive, flow-balance, contextual-tab, final token, parser, resolution,
lowering, validation, canonical-serialization, or self-fuzzing work.

Profile 1's plain-scalar boundary transformation is version 1 and is the second completed slice of
the same verified context-sensitive scanner. It authenticates the canonical atom, layout,
structural-candidate, and quoted-scalar results before classifying any unquoted content. Its
accepted first and continuation characters implement YAML 1.2.2 productions 126 through 135: a
plain scalar is nonempty, has no leading or trailing white space, admits `?`, `:` or `-` initially
only when followed by a context-safe nonspace character, never contains a colon-space or space-hash
delimiter, and treats `[`, `]`, `{`, `}`, and `,` as terminating syntax only inside flow
collections. A `:` followed by a context-safe character and a `#` immediately preceded by nonspace
content remain scalar content, including URL and `foo#bar` spellings. Quotes and otherwise
structural-looking indicators encountered after a plain scalar begins remain plain content when
the YAML productions admit them.

Plain scalar ranges retain every internal presentation atom used for line folding, including line
feeds, required indentation spaces, and safe tabs, but exclude leading separation and trailing
white space. Block-context continuation requires indentation beyond the parent indentation; flow
context continuation remains governed by flow-line-prefix rules. Empty folded lines may be crossed
only when a later nonempty continuation remains in the same scalar. The range's ending line and
byte offset therefore belong to its final nonspace content atom, not to discarded presentation
white space.

The parent indentation is the grammar parent carried through compact block productions, not merely
the physical leading-space count of the line on which the scalar begins. Thus an inline value in
`- key: value`, a scalar after a compact nested `-`, and a block-scalar header after either form all
terminate against the nested mapping or sequence indentation. A sibling entry at that indentation
cannot be folded into the preceding scalar merely because the outer line began at column zero.

Node-property exclusion retains distinct anchor, alias, shorthand-tag, and verbatim-tag substates.
In particular, the comma and square brackets admitted by `ns-uri-char` inside a `!<...>` verbatim
tag remain property spelling until the closing `>` and are never exposed as plain-scalar content.
Flow-adjacent mapping colons are excluded even when the structural-candidate stage has coalesced a
colon and neighboring content into one provisional range.

Tab legality is decided from this scanner context rather than from a global character ban. A tab
before required structural indentation is a typed indentation error. A tab after the required
space indentation in plain, quoted, or block-scalar presentation is separation or scalar content
as required by the applicable YAML production. Literal and folded block scalar header/body regions
are tracked while finding plain boundaries so their punctuation, quotes, comments, and tabs cannot
be relabeled as plain scalars; completed block-scalar token formation and chomping/folding remain a
separate mandatory output of the same lexer.

The absolute plain-scalar count and per-scalar presentation-atom caps are each 1,048,576. Callers
MAY lower either cap but cannot raise it. Scalar-count exhaustion identifies the first content atom
of the first excluded scalar. Atom exhaustion identifies the first atom excluded from the final
accepted half-open range and is checked only when later nonspace content makes intervening
presentation white space part of that range. Raw characters outside YAML's printable set are
rejected at their original byte offset. A reserved `@` or grave-accent indicator in node-start
position is rejected with its own typed diagnostic. A `?`, `:`, or `-` without the context-safe
lookahead required by production 126 reports `InvalidPlainStart` at that indicator. For a given
candidate, syntax and character validity are checked before caller scalar-count and per-scalar atom
caps, so that malformed start cannot be hidden by a zero cap. Earlier stream failures retain their
normal source-order precedence. Every failure exposes no partial public result.

The executable outer and scalar-body scans are iterative; their pure Verus models are total under
explicit candidate fuel, prove strict progress, and fix the complete success or error result. The
public semantic and range contracts bind every scalar to exact opening/final-content atom, byte,
and line endpoints and prove ordered pairwise atom/byte non-overlap. This slice does not relax the
remaining block-scalar, completed-token, parser, resolution, lowering, schema, canonicalization, or
self-fuzzing requirements.

Profile 1's block-scalar transformation is version 1 and completes YAML 1.2.2 productions 162
through 182 rather than merely identifying a provisional `|` or `>` region. It authenticates the
canonical atom, layout, structural-candidate, quoted-scalar, and plain-scalar results before
recognizing a block scalar in block node-start context. An indicator inside a quoted scalar, plain
scalar, node property, comment, directive, document marker, or flow collection is never promoted to
a block scalar. Each accepted scalar records literal or folded style, strip/clip/keep chomping,
optional explicit indentation `1` through `9`, detected effective content indentation, the complete
header and presentation atom/byte/line ranges, and the exact normalized content code points.

The header admits an indentation indicator and chomping indicator in either order, at most once
each, followed only by separation, an optional comment, and the mandatory non-content line break.
An indentation digit of `0`, a duplicate or third modifier, nonseparated `#`, trailing noncomment
text, or end of input before that line break is rejected at the first atom that makes the header
invalid. Header comments end at that single line and cannot absorb following comment lines.

The parent indentation is the verified block-collection grammar context, not merely the header
line's leading-space count. Compact sequence and mapping forms therefore retain their nested
context: for example, `- key: |2` has parent indentation two, while a direct `- |2` remains in the
containing sequence's context. Explicit content indentation equals that parent indentation plus the
indicated value. Without an explicit indicator, the first nonempty content line determines the
indentation; if every candidate content line is empty, the longest such line determines it. The
detected level must be strictly greater than the parent indentation. A leading all-space line may
not be more indented than the first nonempty line. A nonempty line indented more than the parent
but less than the required content indentation is an error rather than an implicit scalar
terminator. A line at or below the parent indentation terminates the scalar. Tabs never count
toward indentation: a tab
encountered before all required indentation spaces reports `TabInIndentation`, while a tab after
those spaces is preserved as content. Empty and more-indented lines retain the behavior required by
the block productions, including content that begins with `#`, YAML indicators, or tabs.

Literal style preserves each normalized content line break. Folded style maps a single break
between adjacent nonempty, non-more-indented text lines to one space; breaks adjacent to an empty or
more-indented line remain line feeds. The final content break and trailing empty lines are never
folded. Strip removes the final break and trailing empty lines, clip retains exactly the final break
when the scalar has nonempty content, and keep retains the final break and every trailing empty line.
For an all-empty scalar, strip and clip produce empty content while keep preserves its presented
line feeds. Less-indented trailing comment lines are outside the scalar presentation range and are
left for completed comment-token formation.

Every emitted normalized content code point records the source atom and original half-open byte
range that caused it. Literal characters and preserved line feeds keep their code point; a folded
space names the normalized source line-feed atom it replaces. Indentation prefixes, the header, and
chomped presentation do not acquire fabricated content provenance. Public predicates prove that
these mappings are ordered, remain within the scalar's presentation range, and name either the
same source code point or the permitted folded line-feed-to-space transformation.

The absolute block-scalar count, per-scalar presentation-atom count, per-scalar normalized-content
count, and aggregate normalized-content count are each 1,048,576. Callers MAY lower any bound but
cannot raise it. Scalar exhaustion reports the first excluded `|` or `>`. Presentation exhaustion
reports the first atom outside the allowed complete scalar presentation. Per-scalar content
exhaustion reports the source atom for the first excluded normalized content code point; aggregate
content exhaustion does the same across scalars. Intrinsic header, indentation, and character
errors take precedence over limits for the same scalar, so a malformed scalar cannot be hidden by a
zero cap. All errors are typed, retain exact original byte offsets, and expose no partial result.

The executable header, contextual block-grammar indentation, line-classification, folding,
chomping, provenance, and outer scan machines are iterative. Their pure Verus models are total
under explicit atom and line fuel, prove strict progress and bounded arithmetic, and fix the exact
success or error result. Every semantically authenticated executable success, including nonempty
success, proves the full range/content predicate and ordered atom/byte non-overlap. That predicate
equates each scalar's content to the exact verified renderer, while the per-content provenance
predicate excludes header and body-indentation atoms and permits folded line-feed-to-space origins
only for folded style. Public semantic, range, content, provenance, and ordered-non-overlap
contracts remain usable downstream; forged upstream or output ghost views cannot be laundered into
valid block-scalar evidence. This completed transformation remains an input to final token
formation and scalar/tag resolution and does not defer the parser, lowering, schema,
canonicalization, or self-fuzzing requirements.

Profile 1's completed-token transformation is version 1 and closes the context-sensitive lexer;
the earlier structural and scalar transformations are authenticated evidence inputs rather than
alternative token streams. The transformation recomputes and authenticates the canonical atom,
layout, structural-candidate, quoted-scalar, plain-scalar, and block-scalar results before emitting
anything. A failure in one input witness has a distinct typed diagnostic naming that transformation,
so an unrelated or forged view cannot be accepted or mislabeled as a later lexical error.

The successful token sequence is lossless over the post-BOM atom stream. Every token has a nonempty
half-open atom range, exact original half-open byte range, and starting and ending source line;
tokens are strictly ordered and adjacent in both atom and byte space, the first begins at atom zero
and byte `bom_bytes`, and the last ends at the atom count and original source length. Empty and
stripped-BOM-only input emits no tokens. This partition includes presentation detail instead of
discarding it: leading-space indentation, in-line separation, normalized line feeds, comments,
and a document-prefix `U+FEFF` each have explicit token kinds. A later concrete syntax tree can
therefore reproduce all non-BOM source evidence and attach comments and formatting without guessing.
The optional stripped leading BOM remains represented by source metadata rather than a fabricated
zero-width token.

The non-trivia token kinds are YAML and TAG directives, reserved directives, directives-end (`---`)
and document-end (`...`) markers, flow-sequence and flow-mapping delimiters, flow entry, block
sequence entry, explicit mapping key, mapping value, anchor property, tag property, alias, and the
five scalar styles: plain, single quoted, double quoted, literal block, and folded block. Scalar
tokens name the exact authenticated scalar record and cover its complete presentation range; their
punctuation, comments, internal separation, and line feeds are not emitted again as nested tokens.
Empty scalars consume no input and are consequently parser/CST nodes anchored between real tokens,
not zero-width lexer tokens that would violate the partition invariant.

Comments begin only where separation context admits `#` and extend through the final non-break atom;
their terminating line feed remains a separate token. Spaces and tabs outside a scalar are grouped
only across a single maximal separation or indentation run. Only spaces form structural indentation.
A tab before the required structural indentation, including a tab-only prefix before a flow
collection, property, alias, or quoted scalar, reports `TabInIndentation` at that tab; tabs admitted
by flow-line prefixes, separation, or authenticated scalar presentation remain lossless input.
Reserved `@` and grave-accent indicators that survive outside scalar content are rejected at their
own offsets rather than emitted as generic content.

Directive tokens implement YAML 1.2.2 productions 82 through 95. A directive is recognized only as
a non-indented line in document-prefix directive mode. Its token ends at the last nonspace parameter,
leaving trailing separation, comment, and line feed as trivia tokens. `%YAML` requires exactly one
decimal `major.minor` parameter and records both checked base-10 components without host-width
dependence. `%TAG` requires exactly one valid primary, secondary, or named handle and one nonempty
local or global tag prefix. A reserved directive retains its nonempty name and every nonempty
parameter range losslessly; the parser later emits the required unknown-directive warning instead
of erasing it. Duplicate YAML directives, duplicate TAG handles, unsupported YAML major versions,
and directive/document ordering depend on document grammar and are mandatory parser diagnostics,
not reasons to weaken lexical spelling checks.

Document markers implement productions 203 and 204: they begin at column zero, contain exactly
three marker atoms, and are followed only by white space, a line break, a comment, or end of input.
The lexer tracks whether the stream is in document-prefix directive mode. A directives-end marker
leaves that mode and a document-end marker re-enters it. A non-leading `U+FEFF` is a
`DocumentByteOrderMark` token only at the true non-indented start of a document prefix. It is also
permitted as content inside an authenticated single- or double-quoted scalar, as required by YAML
1.2.2; every occurrence in plain or block content, directives, properties, aliases, anchors,
comments, or after same-line indentation is rejected at its original byte offset. Marker-looking
text inside any authenticated scalar or comment is never promoted to stream structure.

Node properties and aliases implement productions 97 through 104. Anchor and alias names are
nonempty and exclude white space and flow indicators. A tag token records whether it is
non-specific, verbatim, primary shorthand, secondary shorthand, or named-handle shorthand, plus
exact handle and suffix ranges when present. Verbatim tags require a nonempty `ns-uri-char` payload
and closing `>`; shorthand suffixes are nonempty except for the standalone non-specific `!`, exclude
raw `!` and flow indicators, and validate each percent escape as exactly two hexadecimal digits.
The comma and brackets permitted in a verbatim URI remain inside that tag token. URI well-formedness,
TAG-handle declaration lookup, tag expansion, alias-to-prior-anchor resolution, duplicate node
properties, and the prohibition on alias content are mandatory parser/resolution checks over these
lossless tokens.

Indicator classification follows the context productions rather than character identity alone.
`-`, `?`, and `:` become block collection indicators only with the required separation or line/end
lookahead, while flow mapping value indicators retain YAML's JSON-compatible no-space form. The
lexer maintains a bounded typed stack for `[` and `{`; a closing delimiter must match its opener,
flow entries and flow-only indicator roles require nonzero flow depth, and end of input with a
nonempty stack reports an unclosed-flow diagnostic. Delimiters inside authenticated scalar,
property, directive, or comment ranges never affect the stack. Collection-entry grammar,
key/value placement, indentation nesting, and complete document structure remain parser work, but
the parser never receives an internally impossible or unbalanced final token stream.

The absolute completed-token count is 1,048,576 and the absolute flow-delimiter depth is 4,096.
Callers MAY lower either cap but cannot raise it. Intrinsic spelling, contextual-tab, reserved
indicator, and mismatched-closing-delimiter errors take precedence over admitting the malformed
token under a caller cap. A token-count error identifies the first atom of the first otherwise valid
excluded token. A flow-depth error identifies the first opener beyond the effective depth. An
unclosed flow reports the original end-of-input byte offset; malformed directive, property, alias,
tag, and percent-escape errors identify the first atom that makes the spelling invalid, or end of
input when required spelling is absent. Every error is all-or-nothing and exposes no partial public
token source.

The executable completed-token scanner, directive/property submachines, scalar-evidence merge, and
flow stack are iterative Verus Rust. Their pure models are total under explicit atom and token fuel,
prove strict progress and bounded stack arithmetic, and determine the exact successful sequence or
typed error. Public semantic contracts prove upstream authentication, the complete adjacent
atom/byte partition, exact token-kind spelling, scalar-record identity, trivia maximality, balanced
and properly nested flow delimiters, deterministic result, and the count/depth limits. Proof tests
must include nonempty mixed streams and forged token views that attempt gaps, overlap, incorrect byte
endpoints, scalar-range substitution, header/property leakage, or flow-stack laundering.

Profile 1's concrete-syntax transformation is version 1 and implements YAML 1.2.2 productions 105
through 211 over the authenticated completed-token stream. It supports the complete presentation
grammar rather than a configuration-shaped subset: zero or more independent documents; bare,
explicit, and directive documents; block and flow sequences and mappings; compact collection
forms; explicit and implicit keys; arbitrary node kinds as mapping keys; empty sequence entries,
keys, values, and documents where the productions admit them; aliases; node properties in either
order; and every scalar style. Block collections cannot appear in flow context, while flow nodes
remain valid in block context. The parser retains duplicate mapping entries in source order because
duplicate-key rejection is defined after canonical key resolution, not by raw presentation text.

The concrete syntax tree is a nonrecursive, versioned table representation. A stream record names
ordered document records and the complete token interval. Each document records its prefix,
directive, explicit-start, root-node, explicit-end, and suffix token intervals, plus every reserved
directive warning and YAML-version warning. Each node records its kind and presentation style,
complete token and byte range, optional anchor-property token, optional tag-property token, and an
exact reference to its authenticated scalar or alias token when applicable. Sequence entries and
mapping key/value pairs are separate ordered tables. Collection nodes name contiguous entry-table
ranges; child-node indices are strictly smaller than their completed parent index, so the CST is a
finite acyclic tree without host recursion. A document root names the final node for that document.
Zero-width empty nodes record an exact between-token anchor and source byte offset instead of
fabricating a lexer token or overlapping the lossless token partition.

Trivia remains lossless presentation evidence. Indentation, separation, comments, line feeds, and
document-prefix BOM tokens are not discarded or reinterpreted as semantic nodes. CST token
intervals include their surrounding presentation where the YAML production consumes it, while core
node and entry intervals identify the exact syntax-bearing subrange. Every token belongs to one
ordered stream-prefix, document, or stream-suffix interval, and every syntax-bearing token is owned
by exactly one directive, property, marker, node, or collection-entry record. Comments remain
presentation detail rather than acquiring semantic association with a node; tools may derive a
display attachment without changing the CST or graph meaning.

Directive state is reset for every document. A directive requires a following directives-end
marker. At most one YAML directive is permitted in a document, and a TAG handle may be declared at
most once in that document even when the repeated prefix is identical. `%YAML 1.1`, `%YAML 1.2`, and
the absence of a version directive are accepted and parsed using profile-1 rules; 1.1 records a
compatibility warning. A higher `1.x` minor version is attempted using profile-1 rules with a stable
future-minor warning. A major version other than one is rejected at the version token. Reserved
directives are retained and produce stable warnings rather than errors. Primary `!` and secondary
`!!` handles have their YAML defaults, named handles require a declaration, and explicit TAG
directives may override the two defaults for only that document. Expanded tag URI validity and
exact percent-escape preservation are resolution work, but the parser retains the exact declaration
and property tokens needed to perform them.

A non-alias node admits at most one anchor property and at most one tag property, in either order,
with the separation and indentation required by its context. A repeated property is a typed parse
error at the second property. Properties without explicit scalar or collection content form the
empty scalar node admitted by the applicable production. An alias is a complete node by itself: it
cannot carry a tag, anchor, scalar, or collection child. Whether an alias names the most recent
preceding anchor, whether tag handles resolve, and whether an expanded tag is compatible with the
node kind are mandatory composition checks over the completed CST rather than parser shortcuts.

Flow collection frames enforce comma and key/value placement in addition to the lexer's delimiter
balance. Flow sequences admit ordinary nodes and compact single-pair mappings; flow mappings admit
explicit and implicit entries, JSON-style no-space mapping values, empty keys or values where the
productions allow them, and one optional trailing comma. A leading comma, repeated comma, missing
comma, second mapping-value indicator, or collection closer while an entry is incomplete reports
the first token that makes the production impossible. Flow collection depth is inherited from the
authenticated lexer evidence and is checked again against the parser's caller-lowered stack bound.

Block collection frames are driven by exact token line, indentation width, and parent grammar
context rather than raw line-prefix heuristics. Entries at one indentation form one collection;
greater indentation begins only a child permitted by the pending sequence entry, mapping key, or
mapping value; and lesser indentation closes frames until the owning context is reached. Compact
forms such as `- key: value`, `? key : value`, a block sequence nested directly under a mapping
value, and a mapping nested directly under a sequence entry preserve the parent context already
used by block-scalar formation. Explicit keys may be arbitrary nodes and span the forms admitted by
the YAML grammar. Implicit keys obey the single-line restriction but do not inherit YAML 1.1's
removed 1,024-character limit. Indentation or an indicator that cannot belong to any open frame is
reported at the first offending token.

The absolute document, CST-node, sequence-entry, mapping-entry, directive, and warning caps are each
1,048,576, matching the token ceiling; the absolute parser frame depth is 4,096. Callers MAY lower
any cap but cannot raise it. Empty nodes and empty documents count toward the node and document
caps. Intrinsic document, directive, property, indentation, and collection-grammar errors take
precedence over the caller cap that would otherwise exclude the same completed record. A count
error identifies the first token or between-token anchor of the first excluded record; a depth
error identifies the opener or block entry that would create the first excluded frame. End-of-input
diagnostics use the original source length. Parsing is all-or-error and exposes no partial public
CST. A future diagnostic-recovery transformation may retain multiple errors for editors, but its
output cannot be accepted as executable configuration evidence.

The executable parser uses explicit bounded vectors of document, node, entry, directive, warning,
and frame records; production parsing and frame completion are iterative Verus Rust even where a
recursive implementation would be shorter. The pure model is total under explicit token and frame
fuel and fixes the complete CST or the first typed diagnostic. Public contracts prove canonical
token authentication, strict token progress or frame reduction on every step, bounded indexing and
arithmetic, exact node/property/scalar identity, document and entry ordering, child-before-parent
acyclicity, token ownership without syntax leakage, deterministic results, and every absolute and
caller-lowered limit. Proof tests include nonempty multidocument block/flow mixtures and forged CSTs
that attempt cycles, forward child references, duplicated token ownership, property substitution,
cross-document directive leakage, invalid empty-node anchors, or incomplete frame laundering.

Profile 1's semantic-resolution transformation is version 1. It authenticates the completed-token
source and CST before composing one semantic graph per document. The graph is a nonrecursive table
of scalar, sequence, and mapping nodes plus explicit edges; aliases resolve to existing node
identities rather than copying host objects. CST indices, source tokens, and original byte offsets
remain attached to every semantic node and mapping entry so later lowering and diagnostics never
need to reconstruct presentation provenance. Anchor names and alias spellings remain diagnostic
evidence even though anchor names are not part of semantic node equality.

The default implicit scalar policy is YAML 1.2.2 Core Schema, not YAML 1.1 compatibility
coercion. An untagged plain scalar resolves only as Core null, boolean, integer, finite decimal,
infinity, not-a-number, or string; `yes`, `no`, `on`, `off`, sexagesimal numbers, and legacy
single-letter booleans such as `y` and `n` remain strings. A leading-zero decimal such as `0123`
is a Core decimal integer and is never interpreted as YAML 1.1 octal; octal requires the Core `0o`
prefix. Non-plain scalars without a specific tag resolve as strings.
Empty scalar nodes resolve as null only when they have the `?` non-specific tag; an explicitly
quoted empty scalar is a string. Explicit standard tags `!!null`, `!!bool`, `!!int`, `!!float`,
`!!str`, `!!seq`, and `!!map` require a compatible node kind and a spelling valid for that tag.
Unknown well-formed global and local tags are retained losslessly on compatible semantic nodes for
application-specific validation; they are not silently rewritten to Core tags.

Integer resolution is independent of host width. A resolved integer is a sign plus a normalized
arbitrary-length magnitude stored as little-endian base-1,000,000,000 limbs, and Core decimal,
`0o` octal, and `0x` hexadecimal spellings convert by a bespoke verified multiply-add machine.
Leading magnitude zeroes are removed, every zero has one positive canonical form, and a lowered
limb cap fails at the exact first source digit whose canonical per-digit result needs the excluded
limb. Finite floats are represented exactly as a sign, a canonical arbitrary-length coefficient
stored as little-endian decimal digits, and a canonical arbitrary-length signed exponent stored as
little-endian decimal digits. Leading coefficient zeroes and value-preserving trailing coefficient
zeroes are removed in linear time, with the removed scale applied by a bespoke verified signed
decimal add/subtract machine. Exponent zero has one positive canonical form; coefficient zero
retains the source sign so negative zero remains distinct. Equivalent spellings such as `1.0`,
`1e0`, and `10e-1` normalize to one value without an intermediate IEEE-754 rounding step. A
caller-lowered coefficient- or exponent-digit limit reports the exact first source digit requiring
the excluded canonical digit; when normalization creates the excluded exponent digit, the error
uses the exact coefficient, fraction, or exponent source anchor responsible for that scale.
Positive and negative infinity are distinct values, every accepted NaN spelling has one canonical
semantic value, and negative zero remains distinguishable until schema lowering explicitly chooses
otherwise. Later lowering performs checked conversion to any required fixed-width integer or
floating format and reports range or precision policy errors instead of inheriting host casts.

Single-quoted, double-quoted, plain, literal, and folded scalar presentation is decoded by verified
style-specific machines. Single-quote doubling, every accepted double-quote escape, escaped line
breaks, and YAML flow-line folding produce exact Unicode scalar content with per-output provenance.
Plain-scalar line folding uses the authenticated presentation range and indentation context rather
than trimming arbitrary host strings. Block scalars reuse the already authenticated normalized
content and provenance. Every shared decoded-content record retains its Unicode code point, exact
half-open source-atom range, original byte range, and whether it was direct, folded, quote-doubled,
escape-produced, or derived from an escaped line break. A caller-lowered output cap reports the
exact first excluded provenance record before copying it. Scalar decoding does not apply Unicode
normalization; code-point identity is preserved unless an explicit future language-profile version
says otherwise.

The provisional quoted-scalar context also carries tag- and anchor-property payload state. Property
punctuation and payload candidates therefore cannot become plain content before a following quoted
node, while the required separation clears that state and permits the quote delimiter to start the
node. The same state remains subordinate to authenticated plain- and block-scalar regions, so a
tag- or anchor-like spelling inside scalar content never escapes into presentation structure.

The CST-scalar dispatch submachine authenticates the atom, completed-token, and CST identities,
then binds a scalar result to the exact CST node and completed scalar token before invoking the
verified style-specific decoder. Zero-width empty nodes produce an explicit empty-style scalar
record with neither a fabricated token nor fabricated decoded provenance. Alias and collection
nodes remain outside this scalar producer and return no scalar record; the graph composer handles
them through the independently authenticated alias bindings and collection-entry tables. The
dispatch result is therefore a lossless graph-composer input, not a second parser or a reduced
semantic graph.

Scalar-value composition combines that authenticated presentation with the independently resolved
tag property. Untagged plain and empty nodes follow the YAML 1.2.2 Core rules; untagged quoted and
block nodes and the explicit non-specific `!` tag resolve as strings. Explicit standard scalar
tags require both scalar-kind compatibility and a spelling accepted for that exact tag, while
`!!seq` and `!!map` are rejected on scalar nodes. Well-formed unknown local and global tags retain
their exact resolved tag provenance alongside the decoded scalar content. Integer and float values
are produced only by the existing verified arbitrary-precision converters, and every nested
decoded-content, tag, magnitude, coefficient, or exponent limit is preserved as an exact typed
source diagnostic. The public result retains the CST node index, explicit tag record, full decoded
presentation provenance, canonical semantic tag, and canonical semantic value.

Collection-tag composition authenticates the completed-token source and CST before inspecting node
kind. Untagged and non-specific sequence or mapping nodes receive the corresponding Core tag;
`!!seq` and `!!map` require the exact compatible collection kind; scalar standard tags are rejected
on collections; and unknown local or global tags remain lossless application-defined identities.
The result retains the CST node index, collection kind, canonical semantic tag, and complete
explicit tag provenance. Non-collection nodes return no collection record only after source and
CST identity and node-index bounds are authenticated, leaving them to the scalar or alias producer.

Semantic graph construction begins with a separate verified topology projection rather than a
recursive host graph. It emits one root record for every CST document, one node record for every
CST node, and source-ordered sequence and mapping edge tables that retain the original CST entry
indices, child indices, token intervals, and node byte ranges exactly. Collection node edge
intervals remain indices into the corresponding edge table, so child-before-parent identities and
sharing are preserved without renumbering. Document-root, node, sequence-edge, and mapping-edge
caps are independently caller-lowerable; each failure names the first excluded source record at
its exact byte anchor. Its public semantic predicate includes the completed CST semantic contract,
preventing a forged CST view from laundering an otherwise plausible topology table. Scalar and
collection values, alias redirection, cycle rejection, duplicate-key checks, and merge expansion
compose over these stable identities in the subsequent resolution machines.

Scalar semantic-node population is an independently verified aggregate transformation over those
stable CST identities. It emits exactly one complete resolved-scalar record for every scalar or
zero-width empty CST node, in increasing CST-node order, while collection and alias nodes remain
for their dedicated producers. Besides preserving the per-scalar tag, canonical value, decoded
presentation, and provenance, it counts decoded content across the entire source. A caller-lowered
scalar-record cap fails at the first excluded scalar node; a caller-lowered aggregate-content cap
fails at the exact first excluded decoded content code point, not merely at the containing node.
Per-scalar content, tag, integer-limb, and finite-float digit caps retain their existing nested
typed diagnostics and precedence. The public success theorem extracts both exact CST coverage and
exact aggregate accounting, while the executable loop remains iterative and bounded.

Tag resolution is document-scoped. The primary `!` and secondary `!!` handles begin with their YAML
defaults and may be overridden by that document's `%TAG` directives; named handles must have an
exact declaration in the same document. Verbatim tags bypass handle expansion. In accordance with
YAML 1.2.2 production 39, percent escapes in tag prefixes, suffixes, and verbatim payloads are
retained and compared exactly as presented rather than decoded; malformed escape triples retain
their lexer diagnostic, and escaped and unescaped spellings never collapse into one identity. URI
spelling is validated before a global tag identity is admitted. A local tag remains a distinct
local identity rather than being confused with a global URI. Directive state, local tag identity,
and anchor lookup reset at every document boundary.

An alias names the most recent preceding anchor property with the same exact Unicode name in its
document. Duplicate anchor names are therefore permitted and replace only subsequent lookup;
forward and cross-document aliases are typed errors at the alias token. An anchor on a collection
is visible to its descendants as soon as its property occurs, so a later descendant alias can form
a graph cycle even though CST parents complete after their children. Profile 1 rejects every direct
or indirect alias cycle at the alias edge that first closes the active resolution path. Shared
acyclic subgraphs remain shared semantic identities. Resolution and all later traversals use
explicit stacks and color/state tables rather than host recursion.

The anchor/alias binding submachine scans each authenticated document's root token interval in
presentation order and emits separate immutable declaration and alias-binding tables. Every record
retains its document, CST node, completed token, exact name atom range, and byte range; an alias
also retains the exact declaration and target node selected by most-recent-preceding shadowing.
This representation deliberately permits a collection target's CST node index to follow a
descendant alias node index, because visibility is fixed by property-token position rather than
child-before-parent completion order. Missing-name detection precedes a caller-lowered alias-record
limit at the same alias token, while an earlier source event still retains ordinary first-error
precedence. Cycle detection remains a distinct graph-composition check over these exact bindings.

Semantic node-table composition is a separate verified aggregate transformation. It re-runs and
owns the exact topology, scalar-table, and anchor/alias results produced from the authenticated raw
inputs instead of accepting detached same-length aggregates from a caller. It emits exactly one
slot for every CST node in increasing CST-node order: scalar and zero-width empty slots reference
their exact resolved scalar record, sequence and mapping slots reference their exact resolved
collection record, and alias slots reference the selected target node identity plus a complete
immutable alias-redirect record. Every slot retains the CST token and byte ranges, property-token
indices, and topology edge interval, while every redirect retains the document, alias token,
selected anchor declaration and node, and exact Unicode-name atom and byte ranges. Alias targets
remain shared node identities and are never materialized as copied host values. Node, collection,
and alias-redirect caps are independently caller-lowerable, nested collection-tag limits retain
their own diagnostics, and each failure names the exact first excluded source anchor. The public
semantic predicate exposes equality with the total pure composition result as well as the CST
semantic contract, so forged or stale topology, scalar, anchor, collection, or redirect evidence
cannot be laundered into a successful table.

Alias-cycle rejection consumes and owns that exact node table without materializing alias targets.
It examines alias redirects in presentation order before caller traversal caps, so the first direct
or indirect ancestor edge reports `AliasCycle` at the exact alias-name byte even when a lowered cap
would also reject the graph. Every accepted sequence, mapping, and alias edge is publicly proved to
target a strictly smaller stable CST node identity; this natural-number descent is the graph's
nonrecursive acyclicity witness. A second iterative pass records one exact semantic depth and
completed visit state for every node in deterministic CST order, the maximum observed depth, and
the complete node-index path attaining the first such maximum. The path is retained rather than
discarded so lowering and diagnostics can reuse checked traversal evidence. Semantic-depth and
explicit work-stack limits are independently caller-lowerable and report the byte start of the
first excluded node. The public success predicate binds all of this evidence to the total pure
cycle-resolution result and rejects forged forward redirects.

Canonical scalar-key composition consumes and owns that exact acyclic semantic graph and emits one
key record for every resolved scalar node. Each identity begins with a transformation/version
prefix, a resolved-tag discriminator, and an exact semantic-value discriminator. Core null,
boolean, arbitrary-width integer, exact finite-float, infinity, NaN, and string values use their
already normalized semantic records, so presentation variants such as `1`/`01`, `TRUE`/`true`,
`null`/`~`, `1.0`/`10e-1`, and plain/quoted strings compare by semantic identity rather than source
spelling. Local and global custom tags additionally encode their complete resolved tag content;
two different custom tag identities therefore cannot collapse merely because both carry the same
scalar value. Every variable-length component has a fixed-width length delimiter and every variant
has a distinct marker. Equality is exact byte equality, never host hash equality.

Every emitted byte retains a source-byte diagnostic anchor. Tag-content and decoded string bytes
use their retained per-code-point provenance; normalized numeric metadata uses the owning scalar's
exact node anchor. The executable encoder streams directly over retained tag, decoded-content,
integer-limb, and float-digit slices without cloning them into temporary aggregate buffers. Scalar
record count, per-key bytes, and total key bytes are independently caller-lowerable beneath their
absolute caps, and each failure reports the first excluded emitted byte before allocation. The
public semantic predicate is equality with the total pure composition result and therefore
authenticates the entire owned acyclic graph, exact record order, bytes, provenance, accounting,
limit precedence, and diagnostics. This scalar machine is retained as the authenticated scalar
input of the structural-key composition machine; it does not reduce the committed duplicate-key or
merge-expansion requirements.

Canonical structural-key composition consumes and owns that exact scalar-key source and emits one
identity record for every semantic node in stable CST-node order. Scalar nodes retain the complete
presentation-independent scalar identity. An alias record is byte-for-byte identical to its target
node record, including retained diagnostic provenance, so anchor names and alias presentation never
enter equality. A sequence record contains a structural/version prefix, complete resolved
collection-tag identity, entry count, and the length-delimited child identities in semantic order.
Sequence order therefore remains significant at every nesting depth.

A mapping record contains the corresponding prefix, complete resolved mapping-tag identity, entry
count, and every length-delimited key/value identity pair ordered lexicographically first by the
canonical key bytes and then by the canonical value bytes. The executable uses a verified
nonrecursive bottom-up merge sort over edge indices; it neither hashes identities nor clones full
key/value buffers into sort records. Mapping presentation order is therefore irrelevant, including
when two retained pre-diagnostic entries have equal keys and different values. The complete entry
multiset remains present at this layer: duplicate-key rejection is the next machine and is not
silently performed or hidden by canonicalization.

Core sequence/mapping tags use distinct discriminators. Custom local and global collection tags
add both a locality discriminator and their complete resolved Unicode tag identity, so distinct
application tags cannot collapse. Structural metadata bytes use the owning node's exact byte
anchor; each encoded custom-tag code point uses that code point's retained source-byte anchor; and
reused scalar, child, and alias bytes retain their original per-byte provenance. Every variable-size
component is length-delimited and every node kind is disjoint, making equality exact byte equality
rather than hash equality.

Structural record count, per-key bytes, aggregate stored key bytes, and mapping-sort entries are
independently caller-lowerable beneath their absolute caps. The mapping-sort cap is checked before
allocating its edge-index work vector, and all byte caps are checked before emitting the excluded
byte. Each failure names the owning node's exact source anchor. The public semantic predicate is
equality with the total pure composition result, authenticating the owned scalar/acyclic graph,
one-record-per-node ordering, recursive bytes, provenance, accounting, limit precedence, and typed
diagnostics. Proof and runtime fixtures cover alias transparency, nested collection keys, all
mapping permutations, equal-key/different-value pair permutations, custom collection tags,
sequence-order distinction, and exact accepted/rejected cap boundaries. This machine completes
structural key equality without reducing the subsequent duplicate-key or merge-expansion scope.

Duplicate-key rejection consumes and owns the exact canonical structural-key source. It checks
each mapping independently in retained mapping-entry source order and compares key identity bytes,
not host hashes or presentation spellings. Scalars therefore compare by resolved tag and canonical
semantic value; aliases are transparent; sequence order remains significant; mapping order does
not; empty keys participate as canonical null nodes; and complete custom scalar and collection
tags remain part of equality. Equal keys in separate mappings are never conflated.

Within one mapping, every later key is compared with all earlier keys and the first later equal key
is a duplicate candidate at that key node's exact byte start. A discovery pass selects the minimum
candidate byte across every mapping, so child-before-parent internal node numbering cannot displace
an earlier source diagnostic. The implementation is iterative and allocation-free. Its work is
bounded by the already enforced structural-key byte and mapping-entry caps. Intrinsic duplicate
discovery completes before caller-lowered duplicate-check accounting is applied, so a limit cannot
disguise a duplicate on the mapping or key it excludes. Mapping count and aggregate checked
mapping-entry count are independently caller-lowerable beneath their profile caps; accepted exact
boundaries are inclusive.

Success emits one duplicate-free source that retains the entire authenticated structural-key input
and exact checked counts. The total pure result fixes error precedence, offsets, accounting, and
the owned output. In addition to executable correspondence, the public semantic predicate contains
an independently extractable theorem that every earlier/later key pair in every retained mapping
has unequal canonical bytes. A forged view cannot satisfy that predicate merely by changing byte
provenance while retaining equal canonical values. Runtime fixtures cover normalized scalar
spellings, styles, explicit standard tags, empty keys, aliases, recursively equal sequence and
mapping keys, distinct custom tags, independent mapping scope, multiple-duplicate ordering, and
exact caller-limit boundaries.

Profile 1 supports the YAML merge-key draft deliberately as a named compatibility extension because
merge behavior is part of Crucible's configuration-language contract even though YAML 1.2.2 Core
does not define it. Only an untagged plain mapping key spelled exactly `<<`, or the same scalar with
the explicit `tag:yaml.org,2002:merge` tag, is a merge key; quoted `"<<"` remains an ordinary
string key. A merge value must resolve to one mapping or to a sequence containing only mappings.
For a sequence, earlier mappings override later mappings. Explicit entries in the receiving mapping
override every inherited entry regardless of source order. The resolved diagnostic order is all
explicit non-merge entries in source order followed by still-unshadowed inherited entries in merge
precedence and source order. Merge keys themselves do not appear in the semantic mapping.

The merge result remains a shared semantic graph rather than a materialized alias tree. It records
every mapping, including mappings with no merge key, and each effective entry retains its key and
value node identities, original mapping and mapping-edge identities, and whether it was inherited.
Inherited suppression compares the complete canonical structural bytes, never a host hash. Mapping
records, effective entries, fully expanded tree references, and individual merge sources have
independent caller-lowered caps. Invalid merge shapes are diagnosed before those caller limits can
hide them. The executable result is equal to a total pure model, owns the exact duplicate-free
input, and has public identity and uniqueness theorems so a substituted input or output cannot be
authenticated as the expansion result.

Canonical graph lowering consumes and owns that exact merge-expanded source. It emits one stable
record for every source node but has no alias node kind: each alias record names its final resolved
node and reuses that target's scalar or collection identity and canonical edge interval. Every
sequence child, effective mapping key and value, and document root is likewise rewritten to its
final non-alias node. Non-alias sequences retain exact source-edge identity. Effective mappings
retain receiver identity plus original mapping, original edge, and inherited/explicit provenance.
Scalar indices preserve the complete resolved value, tag, decoded presentation, and code-point
provenance; collection indices preserve the complete resolved collection tag. The owned input
retains all presentation evidence and source spans without copying it into the canonical DAG.

The lowering transform is iterative and graph-preserving. It does not materialize alias subtrees,
duplicate collection edges for alias records, discard unaffected mappings, or erase merge
provenance. Node records, canonical sequence entries, canonical mapping entries, and document roots
have independent caller-lowered caps of at most 1,048,576, with exact first-excluded source-byte
diagnostics and inclusive accepted boundaries. Its total pure model fixes every output record and
error; executable correspondence covers success and failure. Public contracts authenticate exact
input ownership, version/accounting identity, and deterministic output uniqueness. This canonical
YAML DAG is the input to schema-directed typed-field lowering; completing this graph transform does
not narrow or replace the separate typed Crucible configuration, schema validation, canonical
serialization, or digest-stability requirements.

The typed-field schema consumed by that lowering stage is itself an authenticated, independently
versioned graph rather than an untyped host-language map. Schema nodes distinguish null, boolean,
arbitrary-width integer, exact finite float, both infinities, NaN, string, custom scalar, sequence,
custom sequence, mapping, and custom mapping values. Sequence nodes name their exact item schema;
mapping nodes own contiguous field intervals. Each field retains a globally stable nonzero field
identity, a nonempty Unicode-scalar name, an exact value-schema reference, and required/optional
metadata. Compilation rejects malformed node shapes and references, gaps or overlaps in field
ownership, invalid Unicode scalar values, duplicate stable identities globally, and duplicate names
within a mapping. Empty mappings and recursive schema references remain representable; runtime
lowering termination is governed by the finite acyclic YAML graph and its separate work limits.

Schema compilation is iterative Verus Rust and owns its exact input on success. Schema nodes,
fields, and aggregate field-name code points have independent caller-lowered caps of at most
1,048,576 with inclusive accepted boundaries and exact first-excluded schema indices. A total pure
result model fixes the successful compiled schema or first typed schema diagnostic, and public
contracts prove exact input identity and deterministic output uniqueness. This prerequisite does
not by itself claim that YAML fields have been lowered, required or unknown-field policy has been
validated, cross-field invariants have been checked, or a canonical serialization has been emitted.

Typed-field lowering begins with an exact value-binding submachine. Given one canonical YAML node
and one compiled schema node, it binds only matching pairs: Core null, boolean, arbitrary-width
integer, exact finite float, positive infinity, negative infinity, NaN, and string values retain
their distinct resolved tags and value variants; custom scalar, sequence, and mapping schemas
require a custom local or global tag; and Core collection schemas reject custom-tagged collections.
No numeric conversion, string coercion, or tag erasure occurs. Alias records bind through their
canonical resolved-node and scalar/collection identity while diagnostics remain anchored to the
alias or value occurrence being lowered. The submachine rejects inconsistent record indices,
ranges, edge intervals, and version/accounting metadata before returning a typed binding. Its total
pure result fixes both successes and source-anchored typed errors, and its public exact-result
contract prevents a different binding from authenticating for the same graph, schema, and indices.
Graph-wide field traversal, recognized/unknown field partitioning, and required-field validation
remain subsequent machines rather than being silently folded into this local compatibility check.

Duplicate-key checking occurs after tag and scalar resolution and before merge application.
Two scalar keys are equal only when their resolved tag and canonical semantic value are equal;
sequence and mapping keys use structural equality over the acyclic resolved graph, with mapping
entry order ignored and anchors, aliases, styles, and comments ignored. Canonical key comparison is
implemented with a bounded, length-delimited representation rather than host hashing alone, so a
hash collision cannot make distinct keys equal. Two explicit entries in one mapping with equal
canonical keys are rejected at the later key. Merge precedence suppresses an inherited key only
after that same exact equality check and never hides a duplicate among the receiving mapping's
explicit entries.

The absolute semantic-node, sequence-edge, mapping-entry, anchor, alias, tag-byte, per-scalar
decoded-code-point, aggregate decoded-code-point, integer-magnitude-limb, finite-float
coefficient-digit, finite-float exponent-digit, expanded-reference, canonical-key-byte, and
work-stack caps are each 1,048,576; the absolute semantic depth is 4,096. Callers MAY lower every
cap but cannot raise it. Expanded-reference cost counts each node occurrence that a fully
materialized tree would visit, including repeated visits through aliases and merges, even though
the public graph retains sharing; checked addition rejects exponential alias or merge amplification
before allocation. Intrinsic invalid spelling, tag compatibility, missing alias, cycle, invalid
merge shape, and duplicate explicit key errors take precedence over the caller cap that would
otherwise exclude the same record. Limit errors identify the first source token whose admission or
expansion would exceed the effective bound, and all arithmetic is checked against both the profile
cap and `u64` representation.

The executable scalar decoders, handle/anchor tables, graph composer, cycle detector, merge
expander, and canonical-key machine are iterative Verus Rust. The pure transformation is total
under explicit token, node, edge, scalar, and traversal fuel and fixes the complete semantic graph
or first typed diagnostic. Public contracts prove authenticated input identity, exact scalar and
tag decoding, most-recent-preceding anchor selection, absence of unresolved aliases and cycles,
merge precedence, duplicate-key rejection, graph reachability from each document root, deterministic
output, source-order diagnostic retention, canonical key equality, strict progress, and every
absolute and caller-lowered limit. Proof tests include nonempty multidocument graphs and forged
views attempting cross-document handles or aliases, forward aliases, ancestor cycles, anchor
shadowing errors, tag-kind substitution, scalar-provenance laundering, merge amplification,
collision-based key equality, and duplicate-key laundering.

The YAML pipeline must never construct an unbounded alias or merge expansion, recurse without a
verified bound, silently accept duplicate effective keys, or permit a scalar coercion to change
across versions without a language-profile change. Resolution rejection is a typed configuration
result, not a harness panic, and exposes no partial public semantic graph.

### 12.2 Required YAML proofs and tests

The initial proof set must include:

- lexer and parser progress,
- termination under declared resource bounds,
- span validity and monotonic source consumption,
- absence of arithmetic overflow and out-of-bounds indexing,
- deterministic parsing and canonical serialization,
- `parse(canonical_serialize(value)) == value` for representable semantic values,
- canonical digest stability for semantically identical accepted configuration,
- duplicate-key rejection and alias-cycle rejection,
- lowering preserves every recognized field and rejects ill-typed values,
- successful validation implies all configuration invariants required by execution setup.

Fuzzing, property testing, mutation testing, differential testing against independently written
test oracles, and adversarial resource-exhaustion fixtures are required in addition to proofs.
The parser is itself a primary Crucible target and must continuously test its own untrusted
input boundary.

### 12.3 Configuration evolution

Language profile, syntax-tree schema, semantic-value schema, Crucible configuration schema,
and canonicalization algorithm have independent versions. Migrations produce new immutable
artifacts linked to their source configuration; they do not rewrite historical campaign
configuration. Unknown fields are rejected by default, with an explicit compatibility mode
available only when the unknown field can be preserved losslessly and cannot alter execution.

---

## 13. Campaign Model

A campaign is a bounded bug-finding effort.

```rust
pub struct Campaign {
    pub id: CampaignId,
    pub project: ProjectId,
    pub target_builds: Vec<TargetBuildId>,
    pub engines: Vec<EngineConfig>,
    pub configuration: ArtifactRef,
    pub capability_manifest: ArtifactRef,
    pub oracle_set: Vec<OracleIdentity>,
    pub harness_build: HarnessBuildId,
    pub budget: CampaignBudget,
    pub seed: u64,
}
```

Campaign budgets may constrain:

- CPU time,
- wall time,
- memory,
- total executions,
- finding count,
- worker count.

---

## 14. Run and Experiment Model

A run is one invocation of one immutable target build. An experiment groups one or more
related runs that an oracle evaluates together.

```rust
pub struct RunRequest {
    pub id: RunId,
    pub participant: ParticipantId,
    pub target_build: TargetBuildId,
    pub stimulus: Stimulus,
    pub controls: ExecutionControls,
    pub seed: u64,
}
```

```rust
pub enum ExperimentKind {
    SingleRun,
    Differential,
    Metamorphic,
    Stateful,
    Statistical,
    Custom(String),
}
```

```rust
pub struct Experiment {
    pub id: ExperimentId,
    pub producer: EngineId,
    pub kind: ExperimentKind,
    pub seed: u64,
    pub scenario: Scenario,
    pub expected_oracles: Vec<OracleId>,
}
```

A crash experiment normally has one run. A differential experiment has runs against two or
more builds. A metamorphic experiment has the source run and one or more transformed runs.
This grouping must remain intact during replay and minimization. A single-run experiment uses a
one-participant scenario and one execute step rather than bypassing the scenario model.

### 14.1 Scenario graph

```rust
pub struct Scenario {
    pub id: ScenarioId,
    pub participants: Vec<ScenarioParticipant>,
    pub steps: Vec<ScenarioStep>,
    pub edges: Vec<ScenarioEdge>,
    pub controls: ScenarioControls,
}

pub struct ScenarioParticipant {
    pub id: ParticipantId,
    pub role: String,
    pub target_build: TargetBuildId,
    pub deployment: DeploymentRecipeId,
}

pub enum ScenarioOperation {
    Prepare,
    Start,
    Execute(RunRequest),
    Send(StimulusFrame),
    Barrier(BarrierId),
    InjectFault(FaultEvent),
    ChangeNetwork(NetworkEvent),
    AdvanceVirtualTime(Duration),
    Snapshot(StateSnapshotRequest),
    Restore(StateSnapshotRef),
    Observe(ObservationRequest),
    AssertCheckpoint(Vec<OracleId>),
    Stop,
    Reset,
    Cleanup,
}
```

Edges express happens-before dependencies, permitted concurrency, barriers, and data flow.
Every executed step receives an attempt ID. Retries create new attempt records rather than
overwriting the failed or incomplete attempt. Logical participant, process, thread, device,
and connection identities must remain stable across capture, replay, and minimization even
when platform-native IDs change.

Scenario minimization may remove participants, steps, edges, messages, faults, topology, and
environment while preserving the original versioned failure predicate. It must not serialize
concurrent actions accidentally or discard a required negative event such as the absence of a
response.

```rust
pub struct ExperimentObservation {
    pub experiment_id: ExperimentId,
    pub scenario_id: ScenarioId,
    pub step_observations: Vec<ScenarioStepObservation>,
    pub runs: Vec<RunRecord>,
}

pub struct RunRecord {
    pub request: RunRequest,
    pub observation: Option<NormalizedObservation>,
    pub harness_failure: Option<HarnessFailureRef>,
}

pub struct NormalizedObservation {
    pub raw: RawObservation,
    pub sanitizer_events: Vec<SanitizerEvent>,
    pub assertion_events: Vec<AssertionFailure>,
    pub normalized_logs: Vec<LogEvent>,
}
```

At most one of `observation` and `harness_failure` may be present for a completed attempt. An
attempt may instead have an explicit cancelled or incomplete record. An incomplete experiment
usually makes an oracle inconclusive and separately records a harness or infrastructure
finding with its available evidence.

Managed engines may persist experiment specifications before execution when exact campaign
replay is enabled. High-throughput native engines are not required to create a database row
for every transient test case; they must persist interesting inputs, failures, checkpoints,
and enough engine-native state to reproduce imported evidence.

---

## 15. Oracle System

An oracle evaluates whether observed behavior violates expectations.

```rust
pub trait Oracle: Send + Sync {
    fn identity(&self) -> OracleIdentity;

    fn evaluate(
        &self,
        experiment: &Experiment,
        observations: &ExperimentObservation,
        context: &OracleContext,
    ) -> OracleVerdict;
}
```

```rust
pub enum OracleVerdict {
    Pass(OraclePass),
    Fail(OracleFailure),
    Inconclusive(InconclusiveReason),
    NotApplicable(NotApplicableReason),
    EvaluatorError(OracleEvaluatorError),
}
```

Every verdict records the evaluated subjects, property or specification identity, oracle and
normalizer implementation versions, assumptions, effective tolerance or statistical policy,
input evidence, and produced evidence. `EvaluatorError` is not a target failure.

Oracle aggregation must preserve conflicting results. A voting or precedence policy may
produce a derived campaign decision, but it must not erase the underlying verdicts or imply
that majority behavior is correct.

Oracle categories should include:

- hard-failure oracles,
- property oracles,
- differential oracles,
- metamorphic oracles,
- invariant oracles,
- statistical oracles,
- model-comparison oracles.

User-defined executable oracles run through a versioned plugin boundary. Project-owned Verus
oracles may execute in-process only after their panic-freedom, termination, resource bounds,
and interface invariants are verified. Unverified Rust, native, agent-generated, or third-party
oracles must run in a sandboxed helper process, WASM component, or equivalent isolated runtime
with bounded input and output.

### 15.1 Statistical oracle policy

Statistical verdicts must report sample design, number of observations, effect size, uncertainty
interval, estimator, null and alternative hypotheses where applicable, stopping rule, outlier
policy, multiple-comparison correction, environmental blocking, and raw measurement evidence.
A threshold crossing without an uncertainty model is an observation, not a confirmed
performance defect.

Adaptive sampling and sequential tests are permitted, but their policy and all interim samples
must be retained so replay can distinguish optional stopping from a predeclared procedure.

---

## 16. Hard Failure Oracles

Built-in hard failures:

- process crash,
- unexpected nonzero exit,
- sanitizer event,
- assertion failure,
- timeout,
- deadlock,
- panic,
- uncaught exception,
- resource limit exceeded,
- abnormal termination.

These should be among the earliest implemented oracles.

Hard-failure policies are target-specific. For example, a nonzero exit code may be the
documented response to malformed input, a timeout may be an intentionally configured budget
rather than proof of an infinite loop, and a resource limit may indicate an undersized test
environment. Configuration must define which observed outcomes are unexpected. The report
must retain both the raw outcome and the policy that classified it as a failure.

---

## 17. Property Oracles

Properties express expected behavior.

Examples:

```text
output_length <= configured_maximum
```

```text
sort(x) == sort(sort(x))
```

```text
decode(encode(x)) == x
```

```text
parse(print(parse(x))) preserves semantic value
```

```text
total_balance == sum(account_balances)
```

```text
tree is acyclic
```

Property evaluation should support user-defined Rust functions and adapter-defined semantics.

---

## 18. Differential Testing

Differential testing compares two or more implementations or configurations.

Examples:

```text
candidate(input) == reference(input)
```

Useful comparisons:

- optimized vs unoptimized build,
- debug vs release,
- compiler vs interpreter,
- previous version vs rewrite,
- implementation A vs independent implementation B,
- compiler A vs compiler B,
- CPU architecture A vs B.

Outputs should be normalized before comparison when nondeterministic metadata is irrelevant.
Each compared build is represented by a distinct `RunRequest` in the same differential
experiment. The oracle evaluates the complete `ExperimentObservation`; it must never recover
comparison partners through mutable global state.

A differential oracle establishes disagreement. It establishes which participant is defective
only when an independent specification, trusted reference, validity rule, proof, or explicitly
declared project policy supplies direction. N-version majority behavior may raise confidence
but is not proof of correctness. Reports must distinguish `discrepancy` from `attributed
defect`.

---

## 19. Metamorphic Testing

Metamorphic testing is useful when exact output is unknown but relationships between outputs are known.

Examples:

```text
sort(x) == sort(reverse(x))
```

```text
parse(pretty_print(parse(x))) == parse(x)
```

```text
normalize(normalize(x)) == normalize(x)
```

```text
f(identity_transform(x)) == f(x)
```

A metamorphic test consists of:

1. a source input,
2. a transformation,
3. execution of original and transformed inputs,
4. a relation evaluated across outputs.

The source and transformed runs belong to one metamorphic experiment. Store the
transformation identity and parameters so minimization can preserve or deliberately simplify
the relation.

---

## 20. Bug Taxonomy

The schema must support classification across the complete taxonomy. Classification work must
not delay evidence capture; `unknown` is a valid initial class.

Top-level classes:

- memory,
- arithmetic,
- concurrency,
- resource,
- state-machine,
- persistence,
- serialization,
- semantic-logic,
- API-contract,
- numerical,
- compiler,
- distributed-system,
- configuration,
- environmental,
- temporal,
- performance,
- undefined-behavior,
- error-handling,
- compatibility,
- unknown.

Classification should not block finding creation.

Unknown is acceptable.

---

## 21. Memory Bugs

Examples:

- out-of-bounds access,
- use after free,
- double free,
- invalid lifetime,
- invalid alignment,
- uninitialized memory,
- memory leak.

Detection techniques:

- ASan,
- MSan,
- LeakSanitizer,
- Valgrind,
- language runtime checks,
- fuzzing,
- symbolic execution,
- static analysis,
- ownership verification.

---

## 22. Arithmetic Bugs

Examples:

- overflow,
- underflow,
- truncation,
- sign conversion,
- invalid shift,
- division by zero,
- precision loss,
- incorrect rounding.

Detection techniques:

- UBSan,
- checked arithmetic,
- symbolic execution,
- boundary generation,
- property testing,
- differential numeric references.

---

## 23. Concurrency Bugs

Examples:

- race condition,
- deadlock,
- livelock,
- starvation,
- missed wakeup,
- atomicity violation,
- order dependence,
- incorrect lock ordering.

Detection techniques:

- TSan,
- deterministic scheduling,
- schedule fuzzing,
- state exploration,
- lock-order analysis.

---

## 24. Resource Bugs

Examples:

- memory leak,
- file descriptor leak,
- thread leak,
- handle leak,
- runaway CPU,
- runaway memory,
- unbounded queue,
- infinite loop.

Detection techniques:

- resource snapshots,
- repeated-run deltas,
- timeout monitoring,
- soak testing,
- resource budget enforcement.

---

## 25. State-Machine Bugs

Examples:

- invalid transition,
- impossible state,
- missing rollback,
- stale state,
- partial state mutation,
- illegal action accepted,
- legal action rejected.

Detection techniques:

- model-based testing,
- action-sequence generation,
- invariant checking,
- state hashing,
- sequence minimization.

---

## 26. Persistence Bugs

Examples:

- corrupted state after restart,
- partial write recovery failure,
- broken crash recovery,
- incompatible schema migration,
- journal replay error,
- state divergence after interruption.

Detection techniques:

- crash injection,
- restart testing,
- short-write simulation,
- state snapshot comparison,
- replay testing,
- fault injection.

Process termination is not equivalent to power loss. Crash-consistency experiments must model,
where supported by the selected backend, torn writes, dropped or reordered writes, volatile
device caches, flush and barrier semantics, filesystem type and mount options, block size,
copy-on-write behavior, and storage-device fault policy. Findings state the durability contract
under test and the storage assumptions required to reproduce it.

---

## 27. Serialization Bugs

Examples:

- failed round trip,
- noncanonical encoding,
- parser disagreement,
- version incompatibility,
- malformed-input handling,
- semantic information loss.

Useful properties:

```text
decode(encode(x)) == x
```

```text
normalize(parse(serialize(x))) == normalize(x)
```

---

## 28. Numerical Bugs

Examples:

- NaN propagation,
- catastrophic cancellation,
- unstable iteration,
- architecture-dependent divergence,
- precision-sensitive branch behavior,
- invalid rounding.

Detection techniques:

- high-precision reference implementation,
- interval arithmetic,
- metamorphic testing,
- cross-backend differential testing.

---

## 29. Semantic Logic Bugs

Examples:

- incorrect result,
- missing result,
- duplicate result,
- order-sensitive result,
- incorrect business rule,
- broken invariant without crash.

Detection techniques:

- property testing,
- model-based testing,
- differential testing,
- metamorphic testing,
- reference implementation.

### 29.1 Distributed-System Bugs

Examples:

- linearizability or serializability violation,
- lost or duplicated committed operation,
- split-brain acceptance,
- safety violation during leader change,
- failure to converge after healing,
- stale read outside the declared consistency model,
- broken quorum or fencing invariant,
- incorrect retry or idempotency behavior,
- protocol state divergence,
- liveness failure under an admissible schedule.

The distributed subsystem must support named-node topology, deterministic message delivery,
delay, loss, duplication, reordering, partition, bandwidth restriction, node pause, crash,
restart, disk fault, and controlled clock skew. It records client operation histories,
invocation and response intervals, logical and physical clock evidence, node logs, network
events, and durable-state snapshots.

Built-in oracle families should include linearizability, serializability, invariant checking,
monotonic-read and read-your-writes policies, convergence, exactly-once or at-least-once
contracts, and bounded-liveness policies. A bounded liveness failure must state its fairness
and time assumptions.

### 29.2 Compiler and Toolchain Bugs

Examples:

- compiler crash or hang on valid input,
- incorrect rejection or acceptance,
- miscompilation,
- invalid code or metadata emission,
- nondeterministic artifact without declared entropy,
- optimizer disagreement,
- linker, assembler, formatter, or analyzer semantic corruption,
- diagnostic location or fix-it error.

Compiler experiments must carry source-language version, target triple, flags, validity model,
and any undefined or implementation-defined behavior assumptions. A miscompilation finding
requires evidence that the source input is within the compared semantic domain. Generated
programs must support AST, token, IR, flag, and multi-file reduction while preserving validity
and the semantic failure predicate.

### 29.3 Performance and Complexity Bugs

Examples:

- latency or throughput regression,
- tail-latency amplification,
- unexpected allocation or I/O growth,
- algorithmic complexity regression,
- startup or shutdown regression,
- unbounded state growth,
- performance cliff at a boundary,
- unfairness or starvation visible as a distributional regression.

Performance experiments use paired or otherwise statistically controlled baseline and
candidate measurements. They record warm-up, affinity, frequency policy, background load,
hardware counters where available, sample order, raw measurements, and environmental drift.
Complexity tests vary input size and fit or falsify a declared growth envelope rather than
merely comparing one benchmark point.

### 29.4 API, ABI, Schema, and Protocol Compatibility Bugs

Required comparison modes include:

- source and binary API/ABI compatibility,
- request/response schema compatibility,
- wire-protocol negotiation,
- persisted-data forward and backward compatibility,
- rolling-upgrade compatibility,
- downgrade and rollback behavior,
- migration idempotence and recovery,
- behavioral contract compatibility beyond shape compatibility.

Compatibility claims identify direction, supported version range, capability negotiation, and
whether unknown fields or messages must be preserved. Corpus entries from every retained
supported version should participate in compatibility regression campaigns.

### 29.5 Configuration and Environmental Bugs

Crucible must test missing, duplicated, reordered, conflicting, deprecated, partially applied,
and version-skewed configuration. It should explore environment discovery, default selection,
override precedence, filesystem and service discovery, and invalid-but-plausible operator
states. Configuration minimization preserves the effective configuration and provenance from
source field to runtime behavior.

### 29.6 Undefined-Behavior and Contract-Precondition Bugs

Undefined behavior may be a target defect or may invalidate a differential experiment's source
input. The finding must identify the applicable language or API semantics and which
precondition was violated. Sanitizers, interpreters, formal models, compiler flags, and static
analysis may contribute evidence, but no tool's silence proves absence of undefined behavior.

### 29.7 Error-Handling Bugs

Examples include swallowed errors, wrong error identity, loss of causal context, partial success
reported as complete, retry of non-idempotent operations, failure to roll back, secret leakage,
panic across an error boundary, and inconsistent errors across equivalent interfaces. Error
oracles should evaluate both state and the externally visible error contract.

---

## 30. Engine Interfaces

Crucible must support two execution modes. Managed engines ask Crucible to schedule each
experiment. Native engines such as AFL++ and libFuzzer own a high-throughput execution loop
and publish only significant events into the shared evidence pipeline.

### Managed Engine

```rust
pub trait ManagedEngine: Send + Sync {
    fn id(&self) -> EngineId;

    async fn next(
        &mut self,
        context: &GenerationContext,
    ) -> Result<Option<Experiment>, EngineError>;

    async fn feedback(
        &mut self,
        feedback: ExecutionFeedback,
    ) -> Result<(), EngineError>;
}
```

Managed engines may include:

- random generator,
- corpus mutator,
- grammar generator,
- property generator,
- state-sequence generator,
- metamorphic generator,
- symbolic engine adapter,
- schedule generator,
- fault-plan generator,
- agent-assisted generator.

### Native Campaign Engine

```rust
pub trait NativeEngineAdapter: Send + Sync {
    fn id(&self) -> EngineId;

    async fn run_campaign(
        &self,
        context: &NativeCampaignContext,
        events: &dyn NativeEventSink,
    ) -> Result<NativeCampaignSummary, EngineError>;
}
```

Native events may include:

- newly interesting corpus artifact,
- crash or timeout candidate,
- sanitizer candidate,
- coverage summary or checkpoint,
- engine log artifact,
- heartbeat and resource usage,
- final engine-native replay metadata.

A native engine must not be forced through a Tokio queue, generic subprocess launch, or
SQLite transaction for every generated case. Candidate failures and retained corpus entries
must still enter the common reproduction, normalization, oracle, deduplication,
minimization, and evidence-bundle pipeline. Native engine adapters must record their version,
configuration, target build, command without shell interpolation, and sufficient replay
metadata.

Engine identity distinguishes backend from integration frontend. For example, libFuzzer is a
backend and `cargo-fuzz` is a Rust-oriented frontend that invokes it; they are not counted as
two independent fuzzing engines. Corpus and finding provenance records both identities.

Native engines may provide different isolation guarantees from the managed runner. Record
their effective sandbox and resource policy independently. Candidate failures should be
replayed through the managed runner when the target form permits, but that replay must not
erase the conditions of the original native observation.

Native replay through the engine's own replay mode remains authoritative when an in-process,
persistent, forkserver, shared-memory, custom-mutator, or initialization behavior cannot be
reconstructed by the managed runner. Managed replay is then complementary evidence, not a
replacement observation.

### 30.1 Plugin protocol

External engines, analyzers, proof tools, oracles, and agent providers communicate through a
versioned capability-negotiated protocol. The protocol uses content-addressed artifact
references, typed event envelopes, idempotency keys, deadlines, cancellation, heartbeats, and
bounded messages. Plugins declare trust requirements, network and filesystem needs, target
forms, supported platforms, determinism promises, and schema versions before campaign start.

Unverified plugins run out of process at an isolation tier selected independently from the
target. Plugin output is untrusted evidence until schema validation and artifact verification
complete.

---

## 31. Byte Mutation Operators

Initial generic mutation operators:

- bit flip,
- byte replacement,
- byte insertion,
- byte deletion,
- range deletion,
- range duplication,
- range swap,
- input splice,
- repeated byte insertion,
- integer increment,
- integer decrement,
- endian-aware integer replacement,
- dictionary token insertion.

Mutation history must be recorded for retained interesting inputs when the producing backend
exposes it, and backend unavailability must be explicit.

---

## 32. Boundary Generation

Built-in boundary values should include:

```text
0
1
-1
2
-2
MAX
MAX - 1
MIN
MIN + 1
powers of two
powers of two - 1
powers of two + 1
empty
single item
two items
duplicate items
all equal
sorted
reverse sorted
very large
very small
```

Boundary generation should be type-aware where possible.

---

## 33. Structure-Aware Generation

Structured formats should support parsing into AST or schema-aware forms.

Pipeline:

```text
bytes
    ↓
parser
    ↓
structured representation
    ↓
semantic mutation
    ↓
serializer
    ↓
target input
```

Structure-aware mutation should be preferred over blind byte mutation when a grammar or schema exists.

---

## 34. Property-Based Testing

Provide a generator and shrinker abstraction.

```rust
pub trait ArbitraryValue: Sized {
    fn generate(rng: &mut impl Rng) -> Self;
    fn shrink(&self) -> Vec<Self>;
}
```

Property tests should automatically integrate with the common finding and minimization pipeline.

---

## 35. Stateful Testing

Stateful systems should model operations explicitly.

Example:

```rust
pub enum Action {
    Create(Item),
    Update(Id, Patch),
    Delete(Id),
    Read(Id),
    Restart,
}
```

Generated input:

```text
Create
Update
Read
Restart
Read
Delete
```

The harness should evaluate invariants after each action when possible.

---

## 36. Model-Based Testing

A reference model may implement:

```rust
pub trait Model {
    type Action;
    type Observation;

    fn apply(
        &mut self,
        action: &Self::Action,
    ) -> ModelResult<Self::Observation>;
}
```

The same action sequence is executed against:

```text
reference model
real system
```

Observable behavior is normalized and compared.

This is a preferred technique for deep state logic bugs.

Model disagreement is initially a behavioral discrepancy. The model is treated as authoritative
only when the project declares its validity domain and supplies model tests, proof, independent
cross-checks, or another justification. Counterexamples against the model itself remain useful
specification findings and must not be discarded.

---

## 37. Fault Injection

Crucible should support reproducible dependency faults.

Fault types may include:

- allocation failure,
- short read,
- short write,
- interrupted operation,
- disk full,
- read-only filesystem,
- missing file,
- corrupted file,
- dependency timeout,
- dependency unavailable,
- connection reset,
- clock jump,
- process restart,
- delayed response.

Fault injection should be implemented through explicit hooks, adapters, wrappers, or controlled test environments.

Faults must be reproducible.

---

## 38. Fault Plan

```rust
pub struct FaultPlan {
    pub seed: u64,
    pub events: Vec<FaultEvent>,
}
```

Each fault event must identify a stable fault point, participant and dependency scope,
occurrence selector, causal or state predicate, action, and behavior when the selected point is
never reached. A global operation count alone is insufficient under concurrency. The applied
trace records reached, applied, skipped, shadowed, and rejected events separately.

Example:

```yaml
faults:
  - after_operation: 14
    type: short_write
    bytes: 3

  - after_operation: 27
    type: restart
```

The exact applied fault trace must be stored with a finding.

---

## 39. Static Analysis Integration

Static analyzers should be treated as hypothesis producers.

```rust
pub struct StaticFinding {
    pub target_build: TargetBuildId,
    pub analyzer: ToolIdentity,
    pub category: String,
    pub location: SourceLocation,
    pub message: String,
    pub severity: Severity,
    pub raw_evidence: ArtifactRef,
}
```

Source locations should use normalized repository-relative paths plus source/build identity,
not machine-specific absolute paths. Static evidence must retain analyzer version,
configuration, and original output.

Static findings may seed dynamic experiments.

Example:

```text
static analyzer identifies suspicious integer conversion
    ↓
identify affected input dimension
    ↓
generate boundary inputs
    ↓
execute target
    ↓
confirm or reject hypothesis
```

---

## 40. Sanitizer Integration

Initial sanitizer support:

- AddressSanitizer,
- UndefinedBehaviorSanitizer,
- MemorySanitizer,
- ThreadSanitizer,
- LeakSanitizer.

Sanitizer planning is capability- and build-matrix-aware. ASan, TSan, and MSan require separate
build variants and must not be enabled together in one program. UBSan and LeakSanitizer may be
combined only in toolchain/platform configurations that declare compatibility. Unsupported
architectures, incomplete dependency instrumentation, static-linking constraints, symbolizer
requirements, and sanitizer runtime security limitations are recorded in the capability
manifest and finding evidence.

Sanitizer output should be parsed into structured events.

```rust
pub struct SanitizerEvent {
    pub kind: SanitizerKind,
    pub summary: String,
    pub frames: Vec<StackFrame>,
    pub parser: ParserIdentity,
    pub raw_evidence: ArtifactRef,
    pub symbolization: SymbolizationContext,
}
```

Do not rely solely on textual stderr comparison, but always preserve the original bytes.
Sanitizer format varies by sanitizer, compiler/runtime version, platform, options, and
symbolizer availability. Parsers should be fixture-tested against supported versions and
produce a partial structured event rather than discard evidence they cannot fully parse.

Adapters should use sanitizer-provided structured or dedicated log channels when available,
while retaining raw output and offline symbolization inputs. Symbolization is a versioned
derived transformation; unsymbolized program counters and module/build identities remain
authoritative raw evidence.

---

## 41. Coverage Model

Coverage providers differ in feature identity, stability, and storage format. Do not attach
large hash sets of all edges, blocks, and functions to every run record.

```rust
pub struct CoverageRef {
    pub provider: CoverageProviderId,
    pub provider_version: String,
    pub target_build: TargetBuildId,
    pub feature_set_digest: String,
    pub artifact: ArtifactRef,
    pub new_features: u64,
    pub total_features: u64,
}
```

Supported coverage types may include:

- edge coverage,
- block coverage,
- function coverage.

Additional providers should include branch-condition and value profiles, comparison operands,
data-flow and taint features, protocol and model states, semantic events, schedule features,
fault points, and specification or proof-obligation coverage. These feature spaces remain
typed and must not be collapsed into a misleading universal edge count.

Coverage drives:

- input retention,
- scheduling,
- novelty detection,
- campaign metrics.

Providers should use compressed bitmaps, sorted feature vectors, or their native artifact
format. Cross-run comparison is valid only when the provider and instrumented build establish
compatible feature identities. Cross-build source-level coverage is a separate normalization
capability with its own source-map and confidence evidence and must not be implied by this
generic reference type.

---

## 42. Corpus Management

Recommended workspace:

```text
.crucible/
├── corpus/
│   ├── seeds/
│   ├── interesting/
│   ├── coverage/
│   ├── regression/
│   └── minimized/
├── findings/
├── objects/
├── runs/
├── reports/
└── database.sqlite
```

Retain an input if it:

- increases coverage,
- reaches a new model/state digest,
- triggers new oracle behavior,
- exposes a unique finding,
- provides equivalent coverage with lower execution cost,
- contributes to a minimized reproducer,
- increases semantic diversity.

---

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

## 54. Root-Cause Analysis

Root-cause analysis is distinct from finding discovery.

Inputs may include:

- minimized reproducer,
- stack trace,
- sanitizer output,
- execution trace,
- source context,
- coverage,
- oracle failure,
- model state,
- first bad state,
- last good state.

Output:

```rust
pub struct RootCauseHypothesis {
    pub summary: String,
    pub likely_locations: Vec<SourceLocation>,
    pub violated_invariant: Option<String>,
    pub confidence: Confidence,
    pub evidence: Vec<EvidenceRef>,
}
```

Root-cause output is a hypothesis until confirmed by repair or stronger evidence.

---

## 55. First-Bad-State Analysis

Where replay and state snapshots are available:

```text
state 0 good
state 1 good
state 2 good
state 3 bad
state 4 bad
```

Locate the first transition that causes invariant failure.

Use binary search over execution checkpoints only when the failed predicate is proved or tested
to be monotonic over the selected trace. If state may recover, oscillate, or depend on replay
history, use sequential search, causal tracing, checkpoint-assisted delta debugging, or
change-point analysis without discarding intermediate states.

---

## 56. Mutation Testing

Mutation testing evaluates whether the existing test system can detect plausible wrong implementations.

Supported initial mutations:

- `<` → `<=`,
- `<=` → `<`,
- `==` → `!=`,
- `+` → `-`,
- `-` → `+`,
- `&&` → `||`,
- `true` → `false`,
- negate condition,
- remove guard,
- remove error propagation,
- replace constant,
- delete statement where syntactically valid.

A surviving mutant means:

> the test suite does not currently distinguish the original implementation from a plausible incorrect variation.

Surviving mutants should become test-generation tasks.

Surviving mutants create `TestAdequacyGap` findings, not target-defect findings. The mutation
engine records stillborn, killed, survived, timed-out, build-failed, unsupported, and suspected
equivalent outcomes separately. Equivalent-mutant hypotheses require evidence and never improve
the mutation score merely because classification is inconvenient.

---

## 57. Symbolic and Concolic Integration

Symbolic or concolic execution should be integrated through adapters rather than tightly coupled into the core.

Conceptual loop:

```text
execute concrete input
    ↓
record branch constraints
    ↓
select unexplored branch
    ↓
negate selected constraint
    ↓
solve
    ↓
produce concrete input
    ↓
execute normally
    ↓
feed result back into shared corpus
```

All generated inputs should pass through the same executor, oracle, finding, and corpus pipeline.

---

## 58. Concurrency Testing

Concurrency testing deserves its own subsystem.

Potential instrumentation points:

- mutex lock,
- mutex unlock,
- wait,
- notify,
- atomic operation,
- thread start,
- thread exit,
- explicit yield,
- timer,
- queue operation.

Schedules should be recordable and replayable.

Systematic exploration must support partial-order reduction, preemption bounding, sleep sets or
equivalent redundant-schedule reduction, logical thread and task identity, and declared
fairness. Weak-memory and relaxed-atomic behavior requires a memory-model-aware backend;
controlling only the next runnable thread is not sufficient. Instrumentation identity and
uncontrolled nondeterminism must be included in replay evidence.

Race-detector, deterministic-scheduler, model-checker, and production-build observations are
complementary because each instrumentation mode may change timing and semantics. Findings link
these observations rather than treating one as a drop-in replay of another.

---

## 59. Deterministic Schedule Controller

```rust
pub trait ScheduleController {
    fn choose_next(
        &mut self,
        runnable: &[ThreadId],
        state: &ScheduleState,
    ) -> ThreadId;
}
```

Scheduling modes:

- random,
- seeded random,
- recorded replay,
- systematic exploration,
- coverage-guided scheduling.

Additional systematic modes include dynamic partial-order reduction, context or preemption
bounding, state-hash exploration, and memory-model litmus exploration. Schedule decisions use
stable logical operation IDs and include asynchronous I/O completion, timer firing, signal or
interrupt delivery, and wake-up selection where the backend controls them.

---

## 60. Environment Perturbation

Crucible should support controlled variation of:

- locale,
- timezone,
- Unicode normalization,
- path length,
- current working directory,
- memory availability,
- CPU count,
- filesystem permissions,
- directory iteration ordering,
- environment variable presence,
- temporary directory layout,
- architecture where CI supports it.

Perturbation should be deterministic and recorded.

---

## 61. Temporal Testing

Built-in temporal edge cases should include:

- exactly at timeout,
- just before timeout,
- just after timeout,
- clock jump forward,
- clock jump backward,
- DST transition,
- month boundary,
- year boundary,
- leap day.

All time manipulation should occur only within controlled test environments.

---

## 62. Soak Testing

Long-running tests should monitor:

- RSS,
- allocator usage,
- file descriptor count,
- thread count,
- queue depth,
- latency,
- CPU use,
- state size,
- error count.

Interesting conditions include:

```text
monotonic unexplained resource growth
```

and:

```text
latency drift correlated with execution count
```

---

## 63. Build Matrix

Crucible should support multiple builds and compare behavior across them.

Useful build variants:

- debug,
- release,
- O0,
- O1,
- O2,
- O3,
- ASan,
- UBSan,
- TSan,
- MSan,
- LTO,
- no-LTO,
- compiler A,
- compiler B,
- 32-bit where supported,
- 64-bit,
- different standard library versions as declared by the target's supported compatibility
  matrix.

Build-matrix planning must encode compatibility constraints. ASan, TSan, and MSan are separate
instrumented builds and must not be combined into one program; compatible checks such as UBSan
or LeakSanitizer are combined only where the selected compiler/runtime/platform declares
support. Each unavailable or incompatible cell remains visible in the capability matrix rather
than being silently skipped.

Verus verification is a matrix dimension independent of optimization, sanitizer, target
architecture, and compiler. Proof results are tied to the exact verified source and assumptions;
compiled variants still require evidence connecting them to that source.

Cross-build behavior differences may become differential findings.

---

## 64. Candidate Repair Model

Patch generation is a required Crucible capability. Individual projects may disable automated
generation by policy, but the product scope includes evidence-grounded repair production and
verification.

A repair engine receives:

- source context,
- minimized reproducer,
- finding,
- root-cause hypothesis,
- relevant tests,
- properties,
- invariants.

It outputs:

```rust
pub struct CandidatePatch {
    pub id: PatchId,
    pub base_source: SourceSnapshotId,
    pub diff: ArtifactRef,
    pub resulting_source_digest: String,
    pub rationale: String,
    pub expected_invariant: String,
    pub producer: ProducerIdentity,
    pub evidence_inputs: Vec<EvidenceId>,
    pub declared_scope: Vec<VirtualPath>,
}
```

Candidate repairs must not be merged automatically.

Dependency, build-script, schema, migration, generated-code, proof, and configuration changes
are valid repairs when required by the invariant; “minimal” means no unrelated change, not
artificial restriction to one source file. Patch application occurs against the exact base
snapshot and produces a new immutable source snapshot.

---

## 65. Verification Gauntlet

Every candidate repair should run through:

1. original reproducer,
2. minimized reproducer,
3. existing test suite,
4. regression corpus,
5. nearby fuzz corpus,
6. property tests,
7. static analysis,
8. enabled sanitizers,
9. differential tests,
10. metamorphic tests,
11. mutation tests,
12. performance checks where relevant,
13. concurrency checks where relevant,
14. formal verification where configured.

The gauntlet runs matched baseline and patched builds where the stage permits comparison. It
distinguishes pre-existing failures from patch-introduced failures without allowing baseline
failures to excuse a regression in severity, rate, affected configuration, or evidence quality.

Patch-integrity checks must detect attempts to pass verification by weakening, deleting, or
skipping tests, properties, oracles, sanitizers, proof obligations, instrumentation, logging,
resource accounting, supported configurations, or compatibility claims. Such changes require
explicit review and independent justification. Generated patches may not edit Crucible's
verification policy or their own acceptance thresholds.

Targeted post-patch exploration includes changed-code and dependency-neighborhood fuzzing,
multiple independent seeds, boundary generation around the repair, adversarial metamorphic
relations, state migration and rollback, API/ABI/data compatibility, and statistical
performance comparison. The stored verification result identifies the complete policy digest,
budgets, skipped stages and reasons, baseline evidence, and expiration or revalidation triggers.

---

## 66. Patch Acceptance Policy

A repair may be accepted only if:

```text
original bug no longer reproduces
AND
minimized reproducer passes
AND
existing tests pass
AND
regression suite passes
AND
no new sanitizer failure appears
AND
required properties pass
AND
required static checks pass
AND
the verification policy, tests, oracles, instrumentation, and proof obligations were not
weakened without an independently approved specification change
AND
the patched source and binaries are linked to their build, proof, and trusted-boundary evidence
```

For high-assurance projects:

```text
AND
formal proof obligations pass
```

The system should distinguish:

- verified fix,
- likely fix,
- incomplete fix,
- regression-producing fix,
- unverifiable fix.

---

## 67. Bug-to-Invariant Promotion

Every fixed defect should attempt to leave behind:

- one regression test,
- one property,
- one explanatory invariant.

Example:

```text
Bug:
empty input caused parser panic

Regression:
test empty input

Property:
parser never panics for arbitrary byte input

Invariant:
malformed input is represented as an error value, never process termination
```

This is a core architectural goal.

---

## 68. Formal Verification Integration

Formal systems should be pluggable.

Potential integrations:

- Verus,
- Lean,
- Coq,
- Dafny,
- Prusti,
- Kani,
- CBMC,
- TLA+,
- Alloy.

Preferred lifecycle:

```text
dynamic testing finds bug
    ↓
root cause identifies invariant
    ↓
regression test added
    ↓
property encoded
    ↓
formal contract added or a retained failed-formalization record
```

A proof failure should be able to participate in the same finding and verification pipeline.

### 68.1 Crucible implementation verification

Verus is not merely one optional target-side integration. It is the primary implementation and
verification environment for Crucible itself. Every project crate must classify its contents
as:

```text
verified exec
verified spec/proof
specified external boundary
unverified isolated component
generated binding with audited contract
```

New implementation work begins in Verus Rust. Authors must not begin in unrestricted Rust and
defer verification as unspecified cleanup. If a required language or library feature is not
supported, the change includes the narrow external wrapper, its specification, trusted-boundary
entry, executable boundary tests, and a migration issue.

Proof artifacts are content-addressed evidence. CI reproduces proofs from a pinned Verus and
solver toolchain, applies time and memory budgets, and distinguishes proof failure from timeout,
solver unknown, tool crash, unsupported syntax, and trusted assumption. Proof cache keys include
the specification, imported lemmas, solver options, and complete reachable assumption set.

### 68.2 Proof meaning and trusted computing base

Every proof result states exactly what was proved, over which mathematical model, under which
preconditions and axioms, and how the proved executable source maps to a target binary. Model
checking bounds, theorem-prover axioms, opaque functions, admitted lemmas, foreign-function
contracts, compiler/runtime assumptions, and hardware assumptions remain visible.

Lean, Coq, Dafny, TLA+, Alloy, Kani, CBMC, and Verus results are not flattened into a generic
“formal verification passed” flag. The evidence schema identifies theorem proving, deductive
verification, bounded model checking, explicit-state model checking, specification analysis,
and translation validation separately.

---

## 69. Agent Architecture

LLM or coding-agent support is a required product capability and is opt-in for an individual
project or campaign. Disabling external agents must not disable non-agent discovery,
reproduction, minimization, repair registration, or verification.

Agents should consume evidence rather than operate from vague prompts.

Suggested roles:

### Generator Agent

Responsibilities:

- identify edge cases,
- propose input grammars,
- derive properties,
- derive metamorphic relations,
- identify suspicious untested states,
- suggest targeted experiments.

### Triage Agent

Responsibilities:

- classify findings,
- summarize evidence,
- identify likely duplicates,
- propose root-cause hypotheses,
- identify relevant source regions.

### Minimization Agent

Responsibilities:

- propose semantic simplifications,
- assist AST shrinking,
- assist state-sequence shrinking,
- identify irrelevant setup.

### Repair Agent

Responsibilities:

- propose minimal candidate patches,
- explain intended invariant restoration,
- avoid unrelated refactoring unless required.

### Adversarial Review Agent

Assume the patch is wrong.

Responsibilities:

- search for counterexamples,
- identify incomplete fixes,
- test neighboring edge cases,
- propose additional regression tests,
- inspect whether the patch merely hides the symptom.

### Formalization Agent

Responsibilities:

- translate stable invariants into:
  - assertions,
  - property tests,
  - contracts,
  - proof obligations.

---

## 70. Agent Evidence Contract

Agents should receive structured evidence.

Example:

```json
{
  "target_build": {
    "id": "sha256:...",
    "source_revision": "abc123",
    "build_manifest": {}
  },
  "finding": {},
  "experiment": {},
  "observations": [],
  "effective_execution_controls": {},
  "reproduction_summary": {},
  "source_context": [],
  "stacktrace": [],
  "sanitizer_events": [],
  "oracle": {},
  "properties": [],
  "tests": [],
  "coverage_context": {}
}
```

Avoid vague tasks such as:

```text
Find some bugs.
```

Prefer evidence-grounded tasks such as:

```text
Given this minimized reproducer, stack trace, source context, and failed property,
identify the most likely violated invariant and propose tests that could falsify
your root-cause hypothesis.
```

Agent output should be advisory until verified by execution or proof.

### 70.1 Agent identity, containment, and hostile context

Every agent action records provider, model and version, system and task prompts, tool schema and
permissions, decoding parameters, input evidence digests, output, token and cost accounting,
network policy, and human approvals. If a provider cannot supply an immutable model version,
that limitation is explicit.

Source code, build logs, target output, corpus data, issue text, documentation, patches, and
external analyzer results are potentially prompt-injecting untrusted content. They must be
delimited and labeled with provenance. An agent must not gain authority because untrusted text
asks it to run a command, reveal a secret, change verification policy, fetch a URL, or ignore a
contract.

Agents receive least-privilege, task-scoped capabilities and opaque artifact references. Secret
redaction and external-provider data-egress policy apply before packet construction. Agent
workers run separately from target sandboxes, proof workers, signing keys, and production
credentials.

Agent-generated tests, properties, invariants, specifications, assumptions, patches, and
classification decisions are untrusted proposals until independently checked. In particular,
an agent may not introduce or approve its own trusted-boundary exception.

---

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

## 81. Implementation Phases

### Phase 0: Foundation

Implement:

- Verus-Rust workspace,
- pinned Verus, Rust, solver, formatter, and proof-reproduction toolchain,
- Crucible YAML lexer, parser, canonical lowering, schema validation, proofs, and fixtures,
- logging,
- typed IDs and errors,
- evidence and provenance graph,
- scenario, participant, run-attempt, experiment, and observation domain types,
- source, build-recipe, build-execution, deployment, target-build, harness, plugin, and proof
  identity,
- versioned evidence envelopes,
- trusted-boundary ledger and CI policy,
- capability manifests,
- SQLite migrations,
- crash-safe content-addressed artifact store.

Acceptance:

```text
crucible init
```

creates a valid workspace and database. An imported artifact survives an integrity check, and
the initial domain records round-trip through their versioned serialization. The example
configuration parses, canonicalizes, lowers, validates, and receives a stable digest. Parser
proofs pass, malformed and resource-adversarial inputs fail safely, the trusted-boundary ledger
contains every assumption, and CI rejects an unregistered assumption.

### Phase 1: Execution Core

Implement:

- CLI target adapter on supported host platforms,
- input delivery through stdin, generated files, arguments, and environment variables,
- platform-specific process execution and process-tree termination,
- timeout and bounded stdout/stderr capture,
- memory/process/file-size limits where enforceable,
- private temporary working directory,
- capability detection and effective-isolation recording,
- target-build and execution-control capture.
- proof and trusted-boundary identity capture for the Crucible executor itself.

Acceptance:

Crucible can execute a corpus of local test inputs on every supported platform backend,
persist immutable raw observations, and correctly distinguish target outcomes from harness
failures. Cleanup leaves no target process running after normal exit, timeout, cancellation,
or capture failure.

### Phase 2: Observation Normalization and Hard Oracles

Implement:

- abnormal exit detection,
- crash detection,
- timeout detection,
- assertion detection,
- versioned ASan and UBSan parsers,
- target-specific exit and timeout policies,
- versioned oracle verdicts.

Acceptance:

Known crash, timeout, assertion, ASan, and UBSan fixtures produce the expected verdicts while
documented nonzero exits do not. Raw output remains available when parsing is partial or
fails.

### Phase 3: Finding Pipeline and Replay

Implement:

- finding and finding-instance schemas,
- signatures and exact duplicate detection,
- lifecycle transition history,
- replay under recorded controls,
- reproduction summaries,
- harness-defect recording.

Acceptance:

Repeated executions of one defect collapse into one finding without losing the original
instances. Stable, intermittent, and not-observed replay samples are reported without
overstating determinism.

### Phase 4: Minimization and Evidence Bundles

Implement:

- byte delta debugging,
- exact failure-predicate preservation,
- original and minimized experiment retention,
- versioned evidence bundles,
- safe replay wrapper,
- CLI inspection and reporting.

Acceptance:

A large crashing input is reduced while preserving the exact oracle failure. Both inputs
remain reachable, and a fresh Crucible installation can inspect the bundle and replay it when
the declared external target artifact is available.

### Phase 5: Corpus and Coverage-Guided Fuzzing

Implement:

- persistent seed, interesting, coverage, regression, and minimized corpora,
- managed byte mutation and input splicing,
- boundary-value generation,
- coverage-provider abstraction,
- interesting-corpus retention and indexing,
- provider-specific compressed coverage artifacts,
- native engine adapters for AFL++, libFuzzer/cargo-fuzz, and honggfuzz,
- interesting-input and candidate-failure event ingestion,
- engine checkpoint and replay metadata,
- bounded batched persistence,
- basic coverage- and cost-guided input scheduling.

Acceptance:

A hidden-branch fixture is discovered both by the managed mutation path and by each enabled
native backend for which the target can be built. The responsible input enters the shared
corpus, and any failure passes through Crucible's normal replay, oracle, deduplication,
minimization, and bundle pipeline. Native hot loops do not perform a SQLite transaction per
generated case, and corpus artifacts can move between compatible engines through the shared
evidence model.

### Phase 6: Property and Metamorphic Testing

Implement:

- property oracle API,
- property generators,
- shrinking,
- metamorphic transformations.

Acceptance:

Known semantic fixtures generate minimized counterexamples without relying on crashes.

### Phase 7: Stateful and Model-Based Testing

Implement:

- action-sequence input type,
- model interface,
- state comparison,
- sequence shrinking,
- state digests.

Acceptance:

Known state-machine fixture violations are discovered and minimized.

### Phase 8: Fault Injection

Implement:

- fault-plan abstraction,
- reproducible fault traces,
- dependency hooks,
- restart testing.

Acceptance:

Known recovery fixture fails reproducibly under an injected fault.

### Phase 9: Concurrency

Implement:

- schedule abstraction,
- deterministic replay,
- schedule minimization,
- TSan parsing.

Acceptance:

Known race or deadlock fixture can be reproduced sufficiently to produce a stable finding where platform support permits.

### Phase 10: Mutation Testing

Implement:

- source mutation engine,
- mutant build/run,
- mutation score,
- surviving-mutant report.

Acceptance:

Weak tests in fixtures are identified because selected mutants survive.

### Phase 11: Symbolic Integration

Implement adapters for external symbolic/concolic engines.

Acceptance:

Generated symbolic inputs enter the shared corpus and execution pipeline.

### Phase 12: Repair Verification

Implement:

- patch registration,
- temporary worktree,
- patch application,
- build,
- verification gauntlet,
- pass/fail report.

Acceptance:

A complete fix passes and an intentionally incomplete fix fails.

### Phase 13: Agent Integration

Implement:

- evidence packet generation,
- agent API,
- triage role,
- test-generation role,
- repair-review role.
- model, prompt, tool, permission, cost, and egress provenance,
- hostile-context labeling and prompt-injection fixtures,
- least-privilege isolated agent workers.

Acceptance:

Agent suggestions are grounded in stored evidence and are independently verified by the harness.

### Phase 14: External Formal Methods Integration

Implement plugin interfaces for proof systems.

Acceptance:

Proof results can participate in findings and patch verification.

### Phase 15: Scenario and Service-Topology Execution

Implement multi-participant scenario graphs, causal and concurrent step scheduling, barriers,
network namespaces, virtual links, service lifecycle, snapshots, scenario replay, and
topology-aware minimization.

Acceptance:

A multi-service fixture is started, partitioned, observed, healed, replayed, minimized, and
fully cleaned up without changing logical participant identity.

### Phase 16: Distributed-System Correctness

Implement deterministic network scheduling, client-history capture, node crash/restart and
clock faults, linearizability and serializability checking, convergence and consistency-model
oracles, and distributed liveness policies.

Acceptance:

Known safety, convergence, and bounded-liveness defects are found under recorded schedules and
reduced to smaller histories and topologies.

### Phase 17: Performance and Complexity

Implement controlled benchmark scenarios, raw sample retention, statistical oracle policies,
paired baseline/candidate comparison, tail metrics, hardware/environment evidence, and
complexity-growth experiments.

Acceptance:

Known constant-factor, tail-latency, leak-like growth, and algorithmic-complexity regressions
are distinguished from injected measurement noise at the configured confidence policy.

### Phase 18: Compatibility and Migration

Implement API/ABI/schema/protocol comparison, cross-version corpus replay, rolling upgrades,
persisted-state migration, downgrade and rollback, and compatibility minimization.

Acceptance:

Known forward, backward, rolling-upgrade, and failed-migration recovery defects produce
directional compatibility findings and portable evidence.

### Phase 19: Compiler and Toolchain Testing

Implement source and IR generation, language-validity adapters, compile-execute scenarios,
cross-toolchain and cross-optimization comparison, diagnostic oracles, and AST/token/IR/flag
reduction.

Acceptance:

Crash, hang, invalid diagnostic, nondeterministic output, and seeded miscompilation fixtures are
found and minimized without treating undefined source behavior as a compiler defect.

### Phase 20: Kernel and Virtual-Machine Targets

Implement disposable VM snapshots, guest-agent and serial protocols, guest artifact transfer,
kernel sanitizer and crash-dump normalization, virtual-device faults, and complete VM cleanup.

Acceptance:

Known guest kernel crash, deadlock, resource, and recovery defects are replayed from clean
snapshots without exposing the host kernel to target privilege.

### Phase 21: Embedded and Hardware-in-the-Loop Targets

Implement emulator/simulator adapters, firmware loading, serial/debug transport, deterministic
interrupt and clock controls, watchdog and reset evidence, flash snapshots, virtual peripherals,
power interruption, and controlled HIL leases.

Acceptance:

Known firmware state, timing, reset, and persistence defects are reproduced under emulation and,
where configured, equivalent HIL evidence is correlated without conflating the two backends.

### Phase 22: Distributed Workers and Remote Build/Proof Farms

Implement leased idempotent jobs, capability-aware placement, remote content-addressed storage,
transactional metadata, artifact verification, worker attestation, cancellation, retry, and
cross-worker provenance.

Acceptance:

A campaign survives worker loss, duplicate delivery, coordinator restart, and delayed artifact
publication without duplicate evidence corruption, lost reachable artifacts, or changed finding
meaning.

---

## 82. Initial MVP

The first useful release comprises Phases 0 through 5. It includes actual bug discovery,
evidence processing, and coverage-guided corpus growth rather than requiring the operator to
supply an already failing input.

This is an operational milestone, not a reduced definition of Crucible and not completion of
the committed scope in later phases. Its domain and storage schemas must already accommodate
scenario, provenance, proof, plugin, and target extensions without semantic rewrites.

It should implement:

- Verus-Rust project infrastructure and proof CI,
- Crucible YAML parsing, canonical lowering, validation, proofs, and self-fuzzing,
- trusted-boundary ledger and no-unapproved-growth policy,
- immutable evidence and provenance graph,
- CLI target execution,
- immutable target-build identity,
- platform-specific execution backends,
- timeout enforcement,
- memory/process/output limits,
- stdout/stderr capture,
- ASan parsing,
- UBSan parsing,
- persistent corpus storage,
- basic coverage-provider support,
- managed byte mutation and boundary generation,
- AFL++, libFuzzer/cargo-fuzz, and honggfuzz adapters,
- interesting-input retention and scheduling,
- finding creation,
- deduplication,
- reproduction,
- byte minimization,
- artifact bundles,
- SQLite persistence,
- CLI reporting.

The MVP may contain approved external boundaries for Tokio, SQLite, operating-system APIs,
native fuzzers, sanitizers, and other unsupported dependencies, but those boundaries must be
narrow, specified, tested, recorded, and scheduled for reconsideration. No subsystem is exempt
from Verus merely because it belongs to the MVP.

This is the minimum viable Crucible: it can discover a defect, preserve the evidence, replay
and deduplicate it, minimize it, and produce a portable reproduction bundle through the same
framework.

---

## 83. MVP Acceptance Tests

### Verified Configuration Fixture

Given equivalent accepted Crucible YAML documents with comments and different harmless source
formatting:

```text
Crucible parses and lowers both without an external YAML parser
Crucible produces the same canonical effective-configuration digest
Crucible preserves distinct source artifacts and diagnostics provenance
the required Verus proofs pass
```

Given duplicate keys, an alias cycle, invalid UTF-8, excessive nesting, integer overflow, or an
unknown execution-affecting field:

```text
Crucible rejects the configuration with a bounded typed diagnostic
Crucible does not panic, loop indefinitely, or partially start a campaign
```

### Trusted-Boundary Fixture

Given an intentionally introduced `assume`, `external_body`, external specification, or
unverified dependency call without a ledger entry:

```text
verification CI fails
the failure identifies the source location and boundary category
```

Given an approved boundary whose relied-upon contract is violated by a test double:

```text
boundary contract tests detect the violation
no proof result is reported as unconditional
```

### Crash Fixture

Given an input-dependent crash reachable from the configured seed corpus:

```text
Crucible discovers the responsible input through an enabled fuzzing engine
Crucible records it
Crucible reproduces it
Crucible minimizes it
Crucible produces a versioned evidence bundle and safe replay wrapper
```

### Sanitizer Fixture

Given an input that triggers a sanitizer:

```text
Crucible parses the sanitizer event
Crucible classifies the finding
Crucible deduplicates repeated instances
```

### Timeout Fixture

Given an input-dependent infinite loop:

```text
Crucible terminates the target at timeout
Crucible records a timeout finding
Crucible reproduces the timeout
```

### Corpus Import Fixture

Given a seed directory:

```text
Crucible ingests file contents into content-addressed storage
Crucible does not depend on the original host paths for replay
```

Given duplicate files:

```text
Crucible stores their contents once while retaining required provenance
```

### Coverage-Guided Discovery Fixture

Given a target with a hidden branch and a compatible instrumented build:

```text
Crucible retains an input that reaches new coverage
Crucible shares compatible interesting inputs through the common corpus
Crucible may discard an input that adds no evidence or efficiency value
```

Each enabled native backend used in acceptance testing must publish its retained inputs and
candidate failures through the common evidence pipeline.

### Minimization Fixture

Given a 1 KiB crashing input where only a few bytes matter:

```text
Crucible reduces the input substantially
while preserving reproduction
Crucible retains the original input and exact failure predicate
```

### Harness Failure Fixture

Given an invalid executable or an unavailable required isolation capability:

```text
Crucible records a harness failure
Crucible does not report a target finding
```

### Cleanup Fixture

Given a target that forks a child and then hangs:

```text
Crucible terminates the configured process tree at timeout
Crucible records the enforcement capabilities actually used
```

---

## 84. Error Handling

Library crates must use typed errors for recoverable domain failures.

Verified core paths must use explicit typed result enums with Verus specifications relating
error variants to state changes. `anyhow` is confined to application reporting boundaries and
must not erase typed evidence, retryability, attribution, or cleanup obligations.

Application boundaries may use:

```rust
anyhow::Result
```

Expected target failures must not panic the harness.

Panics inside Crucible should be treated as harness defects.

---

## 85. Concurrency Model

Use Tokio for orchestration of managed engines, persistence, and native-engine event
ingestion.

Tokio and operating-system async primitives are initially specified external boundaries unless
the pinned Verus toolchain can verify their used surface. Crucible must verify its own
queue-state machines, capacity invariants, ownership transfer, shutdown protocol, persistence
acknowledgements, and exactly-once or at-least-once semantics around that boundary.

Recommended managed-engine pipeline:

```text
coordinator
    ↓
bounded experiment queue
    ↓
worker pool
    ↓
bounded result queue
    ↓
oracle evaluation
    ↓
finding pipeline
```

All internal queues should be bounded.

Native fuzzing hot loops remain outside this per-run pipeline. They publish significant
events to a separate bounded ingestion queue. Backpressure may pause event publication or
engine checkpointing, but must not turn every native execution into an async coordinator
round trip.

---

## 86. Backpressure

The harness must not generate work faster than it can execute or persist.

Track:

- pending experiments,
- active executions,
- pending results,
- pending findings,
- artifact write backlog,
- native-engine event backlog.

Generation should slow or pause when queues reach configured thresholds.

---

## 87. Cancellation

Campaign cancellation should be graceful.

On Ctrl-C:

1. stop generating new managed experiments,
2. request native engines to checkpoint and stop,
3. stop accepting new long-running jobs,
4. allow active work a brief configured grace period,
5. terminate remaining target process groups and engine processes,
6. flush artifact and metadata queues,
7. write campaign summary,
8. exit with a clear status.

---

## 88. Resumability

Persist enough state to resume interrupted campaigns.

Persist:

- campaign configuration,
- target-build identities,
- seeds,
- corpus,
- findings,
- engine statistics,
- artifact references,
- native-engine checkpoints,
- effective execution controls required by retained evidence.

Exact execution order does not need to be preserved unless deterministic campaign replay is explicitly requested.

---

## 89. Harness Security

Target inputs and outputs must be treated as untrusted.

Requirements:

- never execute target output,
- never interpret target text as shell syntax,
- avoid shell invocation when direct process APIs suffice,
- cap stdout/stderr size,
- prevent path traversal in artifact storage,
- sanitize HTML/report rendering,
- validate configuration,
- isolate writable paths,
- use generated artifact names or content hashes.

The harness itself must be continuously fuzzed, property-tested, mutation-tested, and verified.
Every parser and protocol that consumes configuration, target, plugin, agent, database, bundle,
or remote-worker data is an explicit untrusted-input target.

---

## 90. Definition of a Confirmed Bug

A finding is confirmed when:

```text
a defined oracle fails
AND
the failure reproduces sufficiently
AND
the behavior is attributable to the target rather than harness failure
```

Static analysis alone may produce a candidate or hypothesis but should not automatically create a fully confirmed dynamic defect unless the static method itself constitutes sufficient proof.

Confirmation policy is finding-kind-specific. A sound proof may confirm a target defect without
dynamic reproduction; a behavioral discrepancy remains a discrepancy until blame is
established; a surviving mutant confirms a test-adequacy gap rather than a product defect; and
a harness or infrastructure failure is never relabeled as a target bug merely because it
occurred during target execution.

---

## 91. Definition of Done for a Fixed Bug

A defect is fixed when:

```text
the minimized reproducer no longer fails
AND
the original reproducer no longer fails
AND
the existing test suite passes
AND
the regression suite passes
AND
the configured verification gauntlet passes
AND
a regression artifact is retained
AND
the verification policy was not weakened to hide the defect
AND
the patch's proof and trusted-boundary evidence passes project policy
```

Where appropriate:

```text
AND
the violated property is encoded explicitly
```

If a property or invariant cannot be encoded, closure records the attempted formalization,
blocking reason, retained natural-language invariant, and follow-up work. “Where appropriate”
must not silently mean that formalization was skipped.

---

## 92. Metrics

Campaign metrics:

- executions,
- executions per second,
- CPU hours,
- unique coverage edges,
- unique states,
- unique findings,
- confirmed findings,
- duplicate findings,
- intermittent findings,
- minimized findings,
- time to first finding,
- time to reproduce,
- time to minimize,
- time to verified fix.

Project metrics:

- regression corpus size,
- mutation score,
- property count,
- invariant count,
- formal-proof count,
- defect recurrence rate,
- verification failure rate,
- sanitizer-clean build count.

Verification and scope metrics:

- verified executable lines and functions,
- specification and proof lines,
- proof-obligation count and pass/fail/timeout/unknown distribution,
- trusted-boundary entries by category and consequence,
- newly added and retired assumptions,
- unverified isolated component surface,
- capability-matrix coverage by target class, bug class, platform, architecture, engine,
  oracle, replay, minimization, and repair stage,
- unsupported, degraded, and untested capability cells,
- proof-to-executable artifact linkage coverage.

Coverage metrics are namespaced by provider, provider version, feature kind, and compatible
instrumented build. They must not be summed into a cross-build “unique edge” total.

Do not use a single aggregate score as the only measure of quality.

---

## 93. Implementation Rules for Coding Agents

An implementation agent consuming this document should follow these rules.

### 93.1 Work incrementally

Implement phases in dependency order.

Do not attempt to build the entire system in one uncontrolled change.

### 93.2 Keep the project compiling

At the end of every logical change, run:

```bash
cargo xtask format --check
cargo xtask verify --all
cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

All should pass unless the current task explicitly documents a temporary expected failure.

The workspace `xtask` commands are mandatory project interfaces. They pin and invoke the
correct Verus and solver versions, reproduce proofs, validate proof-artifact identities, and
emit machine-readable proof and trusted-boundary reports.

The formatting interface performs deterministic workspace source discovery and invokes the
pinned `verusfmt` on each source separately. This avoids the formatter's invalid zero-argument
form and shell expansion or command-line-length dependence.

### 93.2.1 Start in Verus

New executable code must begin in Verus Rust with specifications and proof obligations. Coding
agents must not first implement unrestricted Rust and leave “convert to Verus” as a TODO. When
blocked by an unsupported feature, the same change introduces the smallest external boundary,
its contract, tests, ledger entry, and follow-up migration issue.

Trivial-looking getters, conversions, ID validation, serializers, queue transitions, error
mapping, and glue code are not categorically exempt. Verifying simple code is intentionally part
of the project's defense against large-scale AI-authored implementation.

### 93.3 Do not silently stub behavior

If a component is incomplete, mark it clearly with `TODO` and document what remains.

Do not report a phase complete if its acceptance criteria are not met.

### 93.4 Preserve module boundaries

Keep execution, findings, corpus, minimization, scheduling, reporting, and verification logically separated.

### 93.5 Avoid speculative abstraction

Create interfaces where multiple implementations are expected.

Do not introduce unnecessary abstraction layers without a concrete need.

### 93.6 Test each subsystem

Every subsystem should have:

- unit tests,
- at least one integration test or an explicit platform-capability test for the subsystem,
- one known-defect fixture for behavior it claims to detect.

### 93.7 Never bypass reproduction

Do not mark a dynamic finding confirmed from code inspection alone.

### 93.8 Never auto-merge generated patches

Generated repairs are candidates only.

They must pass verification.

### 93.9 Prefer minimal patches

When repairing a bug, avoid unrelated refactoring unless necessary.

### 93.10 Preserve evidence

Never discard the original reproducer when a minimized reproducer is created.

Both are useful.

---

## 94. Committed Expansion and Research Track

Phases 15 through 22, including distributed-system simulation, distributed workers, remote
build and proof farms, compiler adapters, VM/kernel targets, embedded targets, performance, and
compatibility, are committed product scope rather than optional future work.

The following are additional committed expansion targets. Their exact phase assignment may be
refined without removing them from the product direction:

- target-specific grammar learning,
- automatic invariant mining,
- trace-based anomaly detection,
- semantic coverage,
- taint-assisted test generation,
- automated `git bisect`,
- cross-version regression localization,
- richer proof pipelines,
- automatic regression pull requests,
- learned scheduling,
- corpus exchange format,
- semantic failure clustering,
- automated API model extraction,
- proof-producing minimizers,
- persistent project knowledge graph linking bugs, properties, source locations, tests, and
  patches,
- proof-carrying patches and proof-directed repair synthesis,
- automatic specification and metamorphic-relation mining with falsification,
- causal tracing and dynamic data-flow-guided minimization,
- eBPF and kernel-observability providers used through isolated adapters,
- WebAssembly, GPU/accelerator, mobile, browser, GUI, filesystem, and database-specialized
  target adapters,
- deterministic simulation of storage devices, networks, clocks, schedulers, and virtual
  peripherals,
- privacy-preserving corpus exchange, federated campaign statistics, and artifact policy
  enforcement,
- translation validation connecting Verus-verified source to optimized machine code,
- verified schedulers, storage state machines, parsers, protocol codecs, reducers, and evidence
  transformations beyond the initial proof set.

---

## 95. Core Architectural Rule

Every feature participates in the evidence graph and scenario model:

```text
Producer
    creates
Scenario, Experiment, Static/Proof Observation, or Other Evidence

Scenario
    contains
Participants + Causally Ordered or Concurrent Steps

Run
    is attempted by
One Participant at One Scenario Step

Executor / Native Engine
    creates
Raw Observations and Execution Events

Static Analyzer / Proof System
    creates
Versioned Analysis Observations

Normalizer
    creates
Structured Observations

Oracle / Evidence Evaluator / Human Decision
    creates
Versioned Evidence and Verdicts

Evidence Graph
    relates
Artifacts, Observations, Verdicts, Findings, Hypotheses, Proofs, and Decisions

Finding Projection
    groups
Instances and Evidence Without Erasing Disagreement

Reproducer
    creates
Confirmation

Minimizer
    creates
Counterexample

Analysis
    creates
Root-Cause Hypothesis

Repair
    creates
Candidate Patch

Verification
    creates
Evidence

Regression System
    creates
Permanent Knowledge
```

The model is extensible rather than exclusionary. A feature that does not fit an existing node
or relationship must define a versioned evidence kind and provenance semantics; it must not be
discarded or distorted merely to preserve a linear lifecycle diagram.

---

## 96. Final Purpose

Crucible is a scientific instrument for software correctness.

Its purpose is not merely to crash programs.

Its purpose is to continuously ask:

```text
What assumptions does this software make?

Which assumptions are false?

What is the smallest counterexample?

What invariant should have prevented the failure?

Where does the invariant first become false?

What repair most directly restores the invariant?

Does that repair introduce another failure?

Can the repaired property be encoded as a test, contract, or proof?

How do we ensure this class of failure does not return?
```

The desired end state is software that becomes progressively harder to break because every discovered defect leaves behind stronger executable knowledge.

---

## 97. Project Completion Standard

Crucible should be considered architecturally successful when the system can perform the complete loop:

```text
discover
    ↓
reproduce
    ↓
deduplicate
    ↓
minimize
    ↓
classify
    ↓
identify likely invariant
    ↓
propose repair
    ↓
adversarially verify repair
    ↓
create regression test
    ↓
retain permanent evidence
```

across every declared top-level bug class and target class through the common evidence graph.
Not every target/bug-class pair is meaningful, but every cell must be explicitly marked
supported, degraded, unsupported-by-platform, not-applicable with reason, or not-yet-complete.

Architectural success additionally requires:

- the Phase 0 through Phase 22 acceptance suites pass for every declared supported backend,
- no committed capability is represented only by a stub or unowned TODO,
- the capability matrix has no silently skipped cells,
- the Crucible YAML implementation is project-owned, Verus-authored, proved to its declared
  specification, fuzzed, and mutation-tested,
- all eligible harness code is in the Verus-supported subset,
- every remaining unverified boundary is minimal, specified, tested, and present in the
  trusted-boundary ledger,
- proof results are reproducible and tied to executable artifacts and assumptions,
- distributed execution preserves the same evidence meaning as embedded execution,
- original evidence remains reachable through minimization, deduplication, schema migration,
  repair, and closure,
- a project can determine exactly which guarantees were enforced and which were unavailable.

Target-correctness conclusions should not depend exclusively on any single fuzzing backend,
static analyzer, reference model, agent, compiler, or external target-proof system when an
independent check is available. The Crucible implementation intentionally standardizes on and
depends on the pinned Verus/Rust toolchain; that dependency and its solver/compiler trusted base
are explicit rather than disguised as proof-system independence.

The central artifact is the evidence and provenance graph, with the defect lifecycle as one of
its principal verified projections.

Everything else is a producer, consumer, or verifier of that evidence.
