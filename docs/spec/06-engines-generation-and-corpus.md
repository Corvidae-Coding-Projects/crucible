# Engines, Generation, and Corpus

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
