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
| CLI and workspace persistence | `crucible init [path]` creates or monotonically migrates the documented workspace; `crucible artifact import` and `artifact verify` provide integrity-checked persistent artifact ingestion |
| Typed core identifiers | 17 distinct Verus types with versioned envelope round trips |
| Content-addressed artifact identity | Verified SHA-256 and canonical addressing now drive an atomic, deduplicating filesystem object store with SQLite references and import provenance; streaming/directory import and garbage collection remain |
| Evidence and provenance graph | Verified structural admission, append-only/idempotent transitions, borrowed retries, normative edge direction, and atomic multi-input derivations; persistence pending |
| Trusted-boundary enforcement | Verified scanner/reconciliation, approved baseline, and CI known-defect fixture |
| Crucible YAML implementation | Verified profile-1 UTF-8/BOM decoding, bounded lexical atoms, exact line/indentation layout, lossless structural candidates, authenticated context-sensitive quoted/plain/block scalars, completed YAML token formation, and nonrecursive CST parsing for the complete presentation grammar. The CST retains six exact document regions and all trivia, binds properties/scalars/markers/entries to exact syntax ownership, enforces block/flow grammar and caller-lowered limits with typed first-impossible diagnostics, and has a total fueled pure model with strict progress, determinism, child-before-parent acyclicity, adversarial anti-forgery proofs, and exact executable correspondence. Semantic resolution now has verified YAML 1.2.2 Core plain-scalar classification, host-width-independent decimal/octal/hex integer conversion into one canonical arbitrary-width magnitude, exact finite-decimal conversion into canonical arbitrary-width coefficient and signed-exponent digits, canonical signed-infinity/NaN conversion, semantic decoding for all five scalar styles, authenticated CST-node dispatch, complete scalar-value composition, complete collection-tag composition for implicit Core, explicit standard, non-specific, and losslessly retained custom tags, document-scoped explicit tag-property resolution, presentation-ordered anchor/alias binding with exact Unicode shadowing and document reset, a bounded exact semantic-topology projection retaining every document root, CST node identity/range, sequence edge, and mapping edge, exact scalar-table population covering every scalar/empty CST node with aggregate decoded-content accounting, an owned semantic node table assigning every CST node exactly one scalar, collection, or alias slot with exact collection values and alias redirects, exact alias-cycle rejection with proved strictly descending graph edges, per-node semantic depths, visit completion, and a retained deepest path, collision-free canonical byte identities for every resolved scalar, recursive canonical structural identities for every semantic node, exact duplicate explicit-key rejection, verified merge-key expansion, alias-transparent canonical YAML DAG lowering, compilation of the versioned typed-field schema graph, exact canonical-value/schema-kind binding, and verified mapping-field partitioning. Structural identity is alias-transparent, sequence-order-sensitive, mapping-order-insensitive, custom-tag-complete, length-delimited, provenance-retaining, and produced by a verified iterative bottom-up merge sort under independent record, key-byte, aggregate-byte, and mapping-scratch caps. Duplicate rejection compares exact key bytes rather than hashes, is scoped independently to each mapping, reports the globally earliest later equal key by source byte rather than child-before-parent node order, owns the authenticated structural source on success, and proves every earlier/later key pair distinct. Merge expansion recognizes exact plain/explicit merge keys, supports mapping and sequence sources with YAML precedence, preserves graph sharing and full inherited-edge provenance, suppresses inherited keys by exact canonical bytes, preflights malformed shapes, and enforces independent mapping, entry, full-tree-reference, and source caps. Canonical graph lowering owns that result, retains every source node while eliminating alias kinds from the lowered type, normalizes roots and all collection edges, preserves scalar/collection tags and merge/source-edge provenance, and reuses shared target intervals under four independent caps. Typed-field schema compilation authenticates exact scalar/custom/collection kinds, nested references, stable field IDs, requiredness, Unicode names, field ownership, uniqueness, and independent node/field/name caps before configuration data can be lowered. Local typed binding requires exact Core/custom tags and resolved scalar variants, retaining canonical resolved-node and scalar/collection identity without coercion. Mapping-field partitioning then preserves exact recognized and compatibility-mode unknown key/value provenance, validates recognized values, emits stable schema order, rejects unknowns by default, enforces missing/duplicate/kind diagnostics and independent caller-lowered caps, and exposes a total public scan/emission model with strict-order and required-coverage theorems. These machines retain exact source ranges or owned schema identity, exclude YAML 1.1 coercions and host numeric rounding, enforce caller-lowered limits, and prove executable-to-pure correspondence plus public anti-forgery semantics. Recursive graph-wide configuration validation, canonical serialization, digesting, and self-fuzzing remain committed work |
| Execution and evidence core | Portable raw execution outcomes now have independent completion/termination/events, every platform-specific variant, caller-lowered event/extension caps, exact stable tags, a bounded canonical version-1 byte codec, typed malformed-input diagnostics, preserved rejection bytes, and Verus semantic-validation contracts. Immutable raw observations now retain typed run/attempt and coverage-provider identity, raw outcome, exact stream truncation accounting, portable optional resource counters, namespace-versioned state/schedule/fault evidence, and versioned extensions; validate under independent caller-lowered caps; and round-trip every field through a bounded canonical codec with exact rejection-byte preservation. A pure proof binding accepted observation bytes to the exact decoded value remains depth work; SQLite observation persistence, live bounded capture, execution controls, and target adapters remain committed work |
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
