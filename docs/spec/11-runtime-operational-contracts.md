# Runtime Operational Contracts

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
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked -- --test-threads=1
```

All must pass for an accepted change. A red test may exist only as the immediately preceding TDD
state and is never committed as an accepted result.

The workspace `xtask` commands are mandatory project interfaces. They pin and invoke the
correct Verus and solver versions, reproduce proofs, validate proof-artifact identities, and
emit machine-readable proof and trusted-boundary reports.

The formatting interface performs deterministic workspace source discovery and invokes the
pinned `verusfmt` on each source separately. This avoids the formatter's invalid zero-argument
form and shell expansion or command-line-length dependence.

### 93.2.1 Start in Verus

New executable code must begin in Verus Rust with specifications and proof obligations. Coding
agents must not first implement unrestricted Rust and defer conversion to a later tracking note.
When blocked by an unsupported feature, the same change introduces the smallest external boundary,
its contract, tests, ledger entry, and migration issue.

Trivial-looking getters, conversions, ID validation, serializers, queue transitions, error
mapping, and glue code are not categorically exempt. Verifying simple code is intentionally part
of the project's defense against large-scale AI-authored implementation.

### 93.3 Do not silently stub behavior

Incomplete capability work belongs in a coherent external work slice or issue with explicit
acceptance evidence. Executable source must fail with a typed unsupported-capability result where
appropriate; it must not contain a placeholder implementation or an inline completion marker.

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
