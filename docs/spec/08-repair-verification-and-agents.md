# Repair, Verification, and Agents

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
