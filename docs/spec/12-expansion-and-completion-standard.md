# Expansion and Completion Standard

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
- no committed capability is represented only by a stub, placeholder, or unowned tracking note,
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
