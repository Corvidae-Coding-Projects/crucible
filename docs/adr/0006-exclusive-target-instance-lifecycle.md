# ADR-0006: Exclusive target-instance lifecycle

- **Status:** Accepted
- **Date:** 2026-08-14
- **Decision owners:** dollspace.gay (approved 2026-08-14)
- **Related issue:** Phase 1 work slice P1-B
- **Supersedes:** None
- **Amends:** ADR-0004

## Context and evidence

ADR-0004 established the first Linux CLI execution boundary and durable run-attempt transitions,
but persistence state is not target-instance ownership. The runner had no common executable model
for preparation, exclusive execution, reset confidence, cleanup, or permanent discard. A future
stateful adapter could therefore reuse an uncertain instance or invent lifecycle meanings that did
not match the local CLI adapter.

Phase 1 and §10 require the coordinator to own each prepared instance exclusively, provide typed
`prepare`, `execute`, `reset`, and `cleanup` operations, and discard an instance whose state cannot
be reset confidently. These guarantees must precede the additional input and platform adapters so
they consume one lifecycle contract.

## Decision

Add a Verus-Rust target-instance lifecycle to `crucible-core`. Every lifecycle token binds a
versioned adapter kind, typed target ID, typed target-build ID, unique owning run-attempt ID, bounded
instance ordinal, and current state. Its fields are private, it is not cloneable, and every state
transition consumes the preceding token.

The total admitted transition graph is:

```text
Allocated --prepare succeeded--> Prepared --begin execute--> Executing
    |                                           |
    |                                           +--finish execute--> ResetRequired
    |                                                                  |
    +--prepare failed--> Discarded       reset succeeded---------------+--> Prepared
                                                                       |
                                              reset uncertain----------+--> Discarded

Prepared or ResetRequired --cleanup succeeded--> Cleaned
Prepared or ResetRequired --cleanup uncertain--> Discarded
```

`Cleaned` and `Discarded` are absorbing. A second execution cannot begin from `Executing` or
`ResetRequired`; successful reset is the only route from `ResetRequired` to `Prepared`. Failed or
uncertain preparation, reset, or cleanup never returns a reusable token.

The Linux CLI adapter is stateless and uses a fresh instance for each run attempt. It binds the
token to the database-allocated run-attempt identity, advances through preparation and execution,
and cleans the instance rather than pooling it. The three successful lifecycle transitions are
observable as correlated structured events with the same target, target-build, run-attempt, and
worker identities. A host cleanup error advances to `Discarded` and remains a typed harness
failure.

## Preserved invariants

- Adapter, target, target-build, owner-attempt, and instance-ordinal identity never change across
  an admitted transition.
- Rust ownership and the deliberately non-cloneable token prevent concurrent owners of one
  prepared instance in verified orchestration.
- The database-allocated owner attempt plus bounded ordinal distinguishes instances across runs
  and permits multiple explicitly owned instances within a later attempt.
- Execution begins only from `Prepared` and always enters `ResetRequired` before reset or cleanup.
- Uncertain reset never produces `Prepared`.
- Terminal instances cannot be prepared, executed, reset, or cleaned again.
- Lifecycle state does not replace immutable raw observations or the separate persisted
  run-attempt state machine.

## Alternatives considered

- Reusing the database attempt status would conflate durable evidence publication with mutable
  target ownership and would not model reset.
- A cloneable handle with a runtime reference count would permit multiple apparent owners and move
  exclusivity into an unverified convention.
- Platform-specific lifecycle enums would let later adapters assign different meanings to reset or
  cleanup confidence.
- Automatically treating every failed reset as successful cleanup would allow uncertain target
  state to return to the pool.

## Verus and trusted-boundary impact

Identity admission, the complete transition relation, executable transition correspondence,
identity preservation, reset-before-reuse, discard-on-uncertainty, and absorbing terminal states
are Verus targets. The Linux adapter consumes the verified lifecycle around the existing
`CLI-HOST-LOCAL-RUN-001` boundary.

No external body or trusted assumption is added. The existing host boundary remains responsible
for actual process, wait, signal, pipe, and cleanup effects. Its contract and process fixtures are
expanded to state that successful return or non-cleanup failure has killed and waited for the
owned process tree; an explicit cleanup error is mapped to lifecycle discard rather than reuse.
Strict TCB reconciliation must remain at the approved zero-growth baseline.

## Security, privacy, and authorization impact

The lifecycle grants no additional target authority and performs no I/O itself. Identity size is
bounded by existing typed-ID admission, and the per-attempt instance ordinal is capped at one
million. The model reduces cross-run contamination risk by making reset uncertainty terminal.
Structured lifecycle events contain identifiers only and never target output, input, secrets, or
environment values.

## Compatibility and migration

No SQLite, artifact, observation, configuration, bundle, or report schema changes are required.
The lifecycle is an orchestration contract layered around the existing Linux adapter. Later target
adapters must use the same state meanings but may introduce their own versioned adapter kind.

## Verification and acceptance

Acceptance requires:

- `crates/crucible-core/tests/target_adapter.rs` for identity admission, valid and invalid
  transitions, reset-before-reuse, discard-on-uncertainty, and terminal absorption;
- `crates/crucible-core/tests/target_adapter_proof_contract.rs` for executable-to-spec transition
  correspondence and proof-level invariants;
- `crates/crucible-cli/tests/target_adapter_lifecycle.rs` for binding a Linux execution plan to one
  prepared, attempt-owned CLI instance;
- `crates/crucible-cli/tests/logging_cli.rs` for the real prepared/executing/cleaned adapter path and
  exact correlated identities;
- the existing Linux run and cleanup process fixtures; and
- every canonical closure command from the Phase 1 work-slice ledger.

## Consequences and follow-up

P1-B supplies one common ownership protocol for subsequent input-delivery and platform work.
P1-C can extend execution requests without altering instance ownership; P1-D can add cancellation
and grace-period states around the same execution token; P1-F and P1-G can implement host effects
without changing lifecycle meanings. Stateful pooling remains unavailable until an adapter proves
its reset operation and advances `ResetRequired` through `ResetSucceeded`.
