# Crucible

[![Documentation](https://github.com/Corvidae-Coding-Projects/crucible/actions/workflows/docs.yml/badge.svg)](https://github.com/Corvidae-Coding-Projects/crucible/actions/workflows/docs.yml)
[![Code verification](https://github.com/Corvidae-Coding-Projects/crucible/actions/workflows/code.yml/badge.svg)](https://github.com/Corvidae-Coding-Projects/crucible/actions/workflows/code.yml)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/Corvidae-Coding-Projects/crucible/badge)](https://scorecard.dev/viewer/?uri=github.com/Corvidae-Coding-Projects/crucible)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Crucible is a universal software bug-finding, triage, repair, and verification harness. It is
designed to turn failures into reproducible evidence, minimized counterexamples, explicit
invariants, verified repairs, and permanent regression knowledge.

> [!IMPORTANT]
> Crucible is under active Phase 0 implementation and is not yet a production-ready executable.
> The pinned Verus workspace, verified identity types, proof-reproduction interface, and strict
> trusted-boundary policy are present; later Phase 0–22 capabilities remain committed work.

Crucible is not an exploit framework. It is intended for software that the operator owns or is
authorized to test, with local and isolated execution as the default.

## Why Crucible

Most correctness tools specialize in one technique. Crucible is designed as a shared,
evidence-backed lifecycle across a portfolio that includes:

- coverage-guided and structure-aware fuzzing;
- property, stateful, model-based, metamorphic, and differential testing;
- fault injection, temporal testing, concurrency exploration, and soak testing;
- static analysis, sanitizers, symbolic execution, mutation testing, and formal methods;
- reproducible triage, deduplication, minimization, root-cause assistance, repair, and
  adversarial verification;
- CLI, library, service-topology, distributed-system, compiler, VM/kernel, embedded, and
  hardware-in-the-loop targets.

The central lifecycle is:

```text
hypothesis → experiment → observation → oracle → finding
           → reproduction → minimization → repair → verification → regression knowledge
```

The durable architectural artifact is an append-only evidence and provenance graph. Findings,
reports, proofs, and lifecycle state are projections over that evidence rather than lossy
replacements for it.

## Verus-first implementation

Crucible is written in Verus Rust wherever the pinned Verus toolchain can express the
required code. This applies to ordinary glue and transformations as well as traditionally
high-assurance algorithms.

Unavoidable unverified boundaries are narrow, specified, tested, and registered in a
versioned trusted-boundary ledger. CI rejects unregistered assumptions and unapproved
growth of the trusted computing base.

Crucible also owns its YAML-compatible configuration implementation. The lexer, parser,
canonical lowering, schema validation, resource bounds, and core semantic properties are
explicit Verus verification targets rather than delegated to an external YAML parser.

## Project status

| Area | Status |
| --- | --- |
| Architecture and complete product scope | Draft specification available |
| Repository governance and contribution process | Established |
| Verus toolchain and workspace | Phase 0.1 implemented: pinned, digest-bound, and proof-checked |
| Typed core identifiers | 16 distinct Verus types with versioned envelope round trips |
| Content-addressed artifact identity | Verified SHA-256, complete canonical codec, typed algorithm dispatch, checksum-pinned NIST vectors, and integrity checks; object store pending |
| Evidence and provenance graph | Verified structural admission, append-only/idempotent transitions, borrowed retries, normative edge direction, and atomic multi-input derivations; persistence pending |
| Trusted-boundary enforcement | Verified scanner/reconciliation, approved baseline, and CI known-defect fixture |
| Crucible YAML implementation | Verified profile-1 UTF-8/BOM decoding, bounded lexical atoms, exact line/indentation layout, lossless structural candidates, authenticated context-sensitive quoted/plain/block scalars, completed YAML token formation, and nonrecursive CST parsing for the complete presentation grammar. The CST retains six exact document regions and all trivia, binds properties/scalars/markers/entries to exact syntax ownership, enforces block/flow grammar and caller-lowered limits with typed first-impossible diagnostics, and has a total fueled pure model with strict progress, determinism, child-before-parent acyclicity, adversarial anti-forgery proofs, and exact executable correspondence. Semantic resolution now has verified YAML 1.2.2 Core plain-scalar classification, host-width-independent decimal/octal/hex integer conversion into one canonical arbitrary-width magnitude, exact finite-decimal conversion into canonical arbitrary-width coefficient and signed-exponent digits, canonical signed-infinity/NaN conversion, semantic decoding for all five scalar styles, authenticated CST-node dispatch, complete scalar-value composition, complete collection-tag composition for implicit Core, explicit standard, non-specific, and losslessly retained custom tags, document-scoped explicit tag-property resolution, presentation-ordered anchor/alias binding with exact Unicode shadowing and document reset, a bounded exact semantic-topology projection retaining every document root, CST node identity/range, sequence edge, and mapping edge, exact scalar-table population covering every scalar/empty CST node with aggregate decoded-content accounting, and an owned semantic node table assigning every CST node exactly one scalar, collection, or alias slot with exact collection values and alias redirects. These machines retain exact source ranges, exclude YAML 1.1 coercions and host numeric rounding, enforce caller-lowered limits at exact source anchors, and prove executable-to-pure correspondence. Alias-cycle detection, duplicate/merge-key handling, lowering, schema validation, canonical serialization, and self-fuzzing remain committed work |
| Execution and evidence core | Not yet implemented |
| Bug-finding engines and target adapters | Planned in staged phases |

The complete specification is [crucible.md](crucible.md). The delivery sequence and current
milestone are summarized in [ROADMAP.md](ROADMAP.md). Release staging controls order, not the
committed end-state scope.

## Design principles

- Evidence over speculation.
- Reproduction evidence is mandatory for dynamic findings.
- Every bug is a violated property.
- Every patch is guilty until independently verified.
- Portfolio testing beats monoculture.
- Raw observations remain immutable and reachable.
- Platform and isolation limitations are reported, never silently implied away.
- AI-authored implementation is constrained by machine-checked specifications, proofs, tests,
  and explicit trusted boundaries.

## Contributing

The project welcomes design review, corrections, proof work, fixtures, adapters, implementation,
and adversarial verification. Start with [CONTRIBUTING.md](CONTRIBUTING.md), the
[governance model](GOVERNANCE.md), and the relevant sections of the specification.

AI-assisted contributions are welcome, but a human contributor remains responsible for every
claim, assumption, test, proof, dependency, and line submitted. Material use of generated code
or analysis must be disclosed in the pull request.

## Security and support

- Report vulnerabilities privately according to [SECURITY.md](SECURITY.md).
- Ask usage and design questions through [GitHub Discussions](https://github.com/Corvidae-Coding-Projects/crucible/discussions).
- Use GitHub Issues for reproducible defects and scoped proposals.
- Review [SUPPORT.md](SUPPORT.md) before opening a support request.

Do not publish exploit details, secrets, private target data, or third-party vulnerabilities in
public issues.

## Governance

Crucible is maintained by Corvidae Coding Projects under the process in
[GOVERNANCE.md](GOVERNANCE.md). Architectural decisions are evidence-based, reviewable, and
recorded. Changes that reduce declared scope, weaken verification policy, or expand trusted
boundaries require explicit maintainer approval and rationale.

## License

Crucible is licensed under the [MIT License](LICENSE). Contributions are accepted under the same
license unless explicitly agreed otherwise in writing.
