# Campaigns, Oracles, and Bug Model

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
