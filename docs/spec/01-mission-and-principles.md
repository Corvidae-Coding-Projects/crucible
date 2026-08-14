# Mission, Scope, and Principles

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
