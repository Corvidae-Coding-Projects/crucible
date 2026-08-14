# Phases, MVP, and Acceptance

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
