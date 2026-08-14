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
| CLI and workspace persistence | The documented initial grammar is operational: workspace initialization; artifact import, verification, integrity checking, and conservative collection; build/run/fuzz; findings, inspection, replay, minimization refusal when no reducible input exists, patch verification evidence, and reports; configuration validation/canonicalization; and capability, proof, TCB, and plugin inspection. The Linux execution adapter persists immutable target, control, stream, outcome, observation, oracle, finding, replay, and campaign evidence. |
| Typed core identifiers | 17 distinct Verus types with versioned envelope round trips |
| Content-addressed artifact identity | Verified SHA-256 and canonical addressing drive an atomic, deduplicating filesystem object store with SQLite references, import provenance, bounded integrity scans, publication leases/generations, and conservative garbage collection; streaming/directory ingestion remains later breadth. |
| Evidence and provenance graph | Verified structural admission, append-only/idempotent transitions, borrowed retries, normative edge direction, and atomic multi-input derivations, with schema-version-4 SQLite persistence and bounded report projections. |
| Trusted-boundary enforcement | Verified scanner/reconciliation, approved baseline, and CI known-defect fixture |
| Crucible YAML and configuration | The verified profile-1 pipeline covers bounded decoding, lexical analysis, CST formation, semantic resolution, alias/tag handling, duplicate rejection, merge expansion, canonical DAG lowering, typed schema compilation, kind binding, and lossless recognized/unknown mapping partitioning. The Verus-written schema-version-1 bridge recursively validates the complete declared field tree, enforces execution-facing invariants, emits deterministic canonical YAML, authenticates source and canonical bytes with project-owned SHA-256, and powers `crucible config validate` and `config canonicalize`. Its current public proof authenticates versions, caller-lowered bounds, charged work, and both digests; the exact executable-to-pure proof of field preservation, canonical serialization, and all execution invariants required by §12.2 remains committed depth work after the runnable product spine, alongside broader YAML conformance, self-fuzzing, migrations, and compatibility-mode CLI exposure |
| Execution and evidence core | Portable raw execution outcomes now have independent completion/termination/events, every platform-specific variant, caller-lowered event/extension caps, exact stable tags, a bounded canonical version-1 byte codec, typed malformed-input diagnostics, preserved rejection bytes, and Verus semantic-validation contracts. Immutable raw observations retain typed identities, exact stream truncation accounting, portable resource evidence, versioned extensions, independent caps, and canonical full-field round trips. The first Linux CLI adapter executes direct argv through bubblewrap/prlimit without a target-visible host control mount, drains bounded streams, terminates the process group at timeout, and transactionally persists observations or disjoint harness failures. A pure proof binding accepted observation bytes to the exact decoded value and additional platform/input adapters remain required depth and breadth |
| Bug-finding engines and target adapters | The bounded seven-class scheduler, Linux local CLI adapter, process-exit oracle, finding persistence/deduplication, inspection, replay sampling, aggregate high-throughput fuzz retention, and known-defect rediscovery are operational. Native fuzzers, specialized engine execution, successful reducers, and the remaining target/platform adapters stay in their later owning phases. |

The complete specification is organized through the [normative specification index](crucible.md).
The delivery sequence and current milestone are summarized in [ROADMAP.md](ROADMAP.md). The accepted
§§71–80 units are listed in the
[scheduling/storage/CLI/reporting work-slice ledger](docs/work-slices/scheduling-storage-cli-reporting.md).
Release staging controls order, not the committed end-state scope.

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
