# ADR-0005: Scheduler, storage, reporting, replay, and tiered-CI boundaries

- **Status:** Accepted
- **Date:** 2026-08-14
- **Decision owners:** dollspace.gay (approved 2026-08-14)
- **Related issue:** Specification §§71–80 completion slice
- **Supersedes:** None
- **Amends:** ADR-0001, ADR-0002, and ADR-0004

## Context and evidence

The first run path established immutable execution evidence but did not yet supply the surrounding
campaign scheduler, complete domain schema, conservative collection protocol, findings and replay
views, portable report formats, structured correlation logs, or the three required CI depths.
Those capabilities share identity and retention rules: scheduler decisions create campaign state,
storage must preserve every evidence root, reports may only project retained facts, and replay must
state which environmental dimensions were actually reconstructed.

The pinned Verus toolchain still cannot specify filesystem traversal and mutation, SQLite,
process execution, wall clocks, environment variables, terminal output, or the `tracing` and
`serde_json` implementations used at host boundaries.

## Decision

Adopt a verified, bounded seven-class scheduler with the documented 35/20/15/10/10/5/5 baseline,
periodic integer-only adaptation, shared-ancestry and confirmation credit, exploration floors, and
capability-based disablement. Define engine-neutral transactional metadata and object-store traits
so embedded SQLite/filesystem and distributed transactional-server/remote implementations preserve
the same IDs and evidence meaning.

Use schema version 4 for projects, targets, builds, campaigns, experiments, scenarios, findings,
evidence, coverage, corpus, repair, proof, plugin, scheduler, replay, collection, and persistence
coordination records. Scenario edges use composite foreign keys so both endpoint steps belong to
the named scenario. Immutable serialized formats keep independent versions.

Complete the documented CLI grammar. Successful high-throughput fuzz executions retain aggregate
engine statistics and discard transient run graphs; failures take the managed evidence path.
Finding reports distinguish facts from hypotheses and expose bounded human, JSON, JSONL, SARIF,
JUnit, evidence-graph, and bundle-manifest projections. Bundle signature policy admits only a
cryptographically verified signature over the exact manifest and provenance identities and rejects
hypothesis-truth claims; an unsigned projection says so explicitly.

Register five additional CLI trusted boundaries: persisted-run inspection, storage maintenance,
read-only domain reporting, finding replay/verification persistence, and structured logging.
Target output remains a separate bounded artifact stream and is never copied into harness events.

Adopt three fail-closed CI tiers. Every-commit CI reproduces all proofs and the strict TCB audit and
runs lint, unit, regression, bounded fuzz, core property, and YAML boundary tests. Nightly CI adds
AddressSanitizer, extended campaigns, metamorphic/differential checks, checked-in mutation
operators, storage fault injection, untrusted-boundary corpora, and proof reproduction. Weekly CI
adds bounded state-space and publication-interleaving exploration, sequential soak, scenario
topology, platform/architecture checks, a cold proof refresh, and a TCB reduction audit.

## Preserved invariants

- Allocation totals are exact, unavailable engines receive no work, and enabled engines retain the
  configured exploration floor.
- No committed reference names a partial object; collection never races an active publication or
  deletes an original finding, regression, active campaign, or bundle root.
- Persistence inputs and batches have absolute item and byte limits.
- Historical immutable evidence meaning is independent of SQLite migration version.
- Reports never promote hypotheses to observed facts or infer determinism from a seed.
- Finding replay records observed rates and exact environment/schedule/fault equivalence flags.
- CI depth increases runtime and state-space coverage without increasing command parallelism.

## Alternatives considered

- A reinforcement-learning scheduler would be harder to audit and easier to game before utility
  signals and provenance are mature.
- SQLite-specific domain APIs would make distributed workers change evidence meaning.
- Per-input persistence in high-throughput mode would amplify memory, disk, and database use without
  improving evidence for uninteresting passing executions.
- Free-form report construction would make schema drift and fact/hypothesis confusion difficult to
  detect.
- One large CI job would hide the distinction between fast merge protection, nightly depth, and
  weekly exploration and would make resource failures harder to diagnose.

## Verus and trusted-boundary impact

Scheduler arithmetic, replay seed derivation, publication transitions, collection planning,
retention admission, storage interface policy, bundle signature scope admission, CLI parsing, and
schema identities remain Verus Rust and are reproduced by the canonical proof command. Effectful
host behavior is registered in `tcb/ledger.tsv`, bound to exact source metadata in
`tcb/approved.tsv`, exercised by process tests, and rejected on unregistered or unapproved growth.

The trusted boundary does not prove SQLite, the filesystem, Linux isolation, `serde_json`,
`tracing`, or external cryptography. Exact byte authentication, read-only database transactions,
bounded output, foreign keys, quick checks, no-follow path handling, and process-level fixtures
reduce and expose those risks.

## Safety, privacy, resource, and authorization impact

All new command inputs and outputs are bounded. SQLite value length, object size, report rows,
report bytes, scheduler credits and slots, collection candidates, persistence items and bytes,
processes, target memory, target time, and captured streams have explicit caps. CI runs one build
and test worker at a time; the platform matrix also limits concurrent jobs to one.

The decision adds no network access to target execution. Workflow downloads remain digest-checked
where they enter the proof toolchain. Target output and hostile evidence are data, never
instructions or harness logs.

## Compatibility and migration

Schema versions 1–3 migrate monotonically to version 4. Migration name, bytes, checksum, installed
schema digest, application identity, and exact history are checked. Unknown, divergent, partially
applied, or cross-scenario records are rejected. Existing artifact IDs and serialized evidence do
not change meaning.

## Verification and acceptance

Acceptance is the exact set of slice tests and closure commands recorded in
`docs/work-slices/scheduling-storage-cli-reporting.md`. The combined gate includes formatting,
Clippy, every executable and documentation test, all Verus targets with one verifier thread, strict
TCB reconciliation, documentation ownership checks, and workflow linting.

## Consequences

Crucible now has a complete initial control plane around its Linux execution adapter: scheduling,
durable campaign/finding state, conservative object management, replay evidence, portable reports,
correlated logs, self-test corpora, and explicit CI depth. Later engine and platform implementations
can use these interfaces without changing durable IDs or report truth semantics.
