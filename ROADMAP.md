# Roadmap

Crucible's complete scope and phase acceptance criteria live in the
[design specification](crucible.md). This document summarizes delivery groups and the current
repository milestone; it does not replace or narrow the specification.

No dates are promised until implementation velocity and Verus/toolchain constraints are measured.

## Current milestone: Phase 0 implementation foundation

Current work establishes and extends:

- public project governance and contribution processes;
- the complete architectural specification;
- explicit Verus-first and trusted-boundary policy;
- the project-owned Crucible YAML requirements;
- evidence, provenance, scenario, build, plugin, proof, and capability identities;
- implementation-ready acceptance criteria;
- the pinned Verus/Rust/Z3/verusfmt workspace and reproducible proof command;
- verified typed identifiers and versioned identity envelopes;
- verified trusted-boundary scanning, ledger/approval reconciliation, and code CI.

Exit requires every Phase 0 implementation and acceptance criterion in the design specification;
the completed Phase 0.1 slice is a dependency checkpoint, not a reduction of that exit standard.

## Delivery groups

### Foundation and first operational release

Phases 0–5 deliver the Verus-Rust workspace, Crucible YAML, proof and trusted-boundary
infrastructure, evidence storage, CLI execution, hard oracles, replay, minimization, bundles,
corpus management, and managed/native coverage-guided fuzzing.

This is the first operational milestone, not the completed product.

### Semantic and stateful correctness

Phases 6–8 add property and metamorphic testing, stateful and model-based testing, and
reproducible fault injection.

### Concurrency, adequacy, and symbolic exploration

Phases 9–11 add deterministic concurrency exploration, mutation testing, and symbolic/concolic
integration.

### Repair, agents, and external formal systems

Phases 12–14 add patch registration and adversarial verification, contained evidence-grounded
agents, and external proof-system integration. Verus verification of Crucible itself begins in
Phase 0 rather than Phase 14.

### Multi-participant and specialized targets

Phases 15–21 add scenario/service topology, distributed-system correctness, statistical
performance and complexity testing, compatibility and migration, compilers/toolchains,
VM/kernel targets, and embedded/hardware-in-the-loop targets.

### Distributed execution

Phase 22 adds distributed workers and remote build/proof farms while preserving identical
evidence meaning, idempotency, cancellation, and artifact integrity.

## Expansion track

Committed expansion targets include grammar and invariant learning, semantic and taint coverage,
regression localization, richer proof pipelines, proof-carrying patches, specialized WebAssembly,
GPU, mobile, browser, GUI, filesystem and database adapters, deterministic device simulation,
privacy-preserving corpus exchange, and translation validation.

## How priorities are chosen

Maintainers consider dependency order, assurance risk, breadth of bug classes, user evidence,
Verus support, platform capability, contributor availability, and the ability to leave a phase
in a genuinely accepted state. Priority changes reorder delivery; they do not silently delete
scope.
