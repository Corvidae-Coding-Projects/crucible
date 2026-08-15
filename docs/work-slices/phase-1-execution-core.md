# Work slices for Phase 1: Execution Core

This acceptance ledger decomposes Phase 1 from the normative
[implementation phases](../spec/10-phases-mvp-and-acceptance.md) together with the applicable
[target and isolation requirements](../spec/03-targets-execution-and-isolation.md) and
[runtime operational contracts](../spec/11-runtime-operational-contracts.md). It records delivery
boundaries, dependencies, and executable acceptance evidence without changing or narrowing the
normative specification.

A slice is accepted only when its complete product boundary is implemented in Verus Rust or
covered by the smallest approved external boundary, its red-before-green tests pass, its resource
limits are executable, and the canonical closure commands pass from the same source tree. Planned
slices receive no completion credit. Existing code that contributes to a planned slice may be
reused, but it does not create a partial or implied acceptance state. Source comments are not used
for progress tracking.

## Slice ledger

| Slice | Normative scope | Product boundary | Acceptance evidence | State |
| --- | --- | --- | --- | --- |
| P1-A | Phase 1; §§10–11 | Linux x86_64 local CLI execution by exact direct arguments through the authenticated bubblewrap/prlimit supervisor; bounded output draining, timeout and error process-group cleanup, capability and control capture, immutable raw observations, and disjoint harness failures | `crates/crucible-cli/tests/run_cli.rs`, `local_run_plan.rs`, `local_observation.rs`, `local_execution_boundary_proof_contract.rs`, workspace Verus verification, strict TCB reconciliation, and `docs/adr/0004-linux-local-run-and-immutable-evidence-boundaries.md` | Accepted |
| P1-B | Phase 1; §10 | Common CLI adapter lifecycle with immutable adapter/build identity, exclusive prepared-instance ownership, typed `prepare`, `execute`, `reset`, and `cleanup` transitions, and discard-on-uncertain-reset behavior | `crates/crucible-core/tests/target_adapter.rs`, `target_adapter_proof_contract.rs`, `crates/crucible-cli/tests/target_adapter_lifecycle.rs`, `logging_cli.rs`, workspace Verus verification, ADR-0006, and strict TCB reconciliation | Accepted |
| P1-C | Phase 1; §§10–11 | Complete input-delivery matrix: bounded stdin, generated relative files, exact arguments, and bounded environment variables; private-path confinement, collision policy, exact delivered-input evidence, and no shell interpolation | `crates/crucible-cli/tests/input_delivery.rs`, delivery-plan proofs, untrusted path/key/value fixtures, and per-platform adapter tests | Planned |
| P1-D | Phase 1; §§11, 84, 85, and 87 | Cancellation and terminal-path cleanup state machine covering normal exit, timeout, explicit cancellation, capture failure, preparation failure, and execution failure; graceful deadline followed by forced process-tree termination, drain completion, persistence acknowledgement, and no surviving descendant | `crates/crucible-cli/tests/execution_cleanup.rs`, cancellation/shutdown model proofs, tagged-descendant fixtures, forced capture-failure fixtures, and strict TCB reconciliation | Planned |
| P1-E | Phase 1; §§10–11 | Complete executor identity and evidence binding: Crucible executable identity, adapter/supervisor and isolation-tool identities, proof artifacts, trusted-boundary ledger and approval identities, runtime/platform identity, requested/effective controls, and exact target-outcome versus harness-failure attribution | `crates/crucible-cli/tests/executor_identity.rs`, canonical identity codec tests and proofs, persistence migration tests, tamper fixtures, and strict TCB reconciliation | Planned |
| P1-F | Phase 1; §§10–11 | macOS CLI backend with direct process execution, process-group cleanup, enforceable resource controls, private working directory, controlled environment and writable paths, network/capability truthfulness, bounded stream capture, and the same portable evidence meanings as Linux | `crates/crucible-cli/tests/macos_run_cli.rs`, macOS capability-contract tests, platform evidence fixtures, and a resource-bounded macOS CI execution job | Planned |
| P1-G | Phase 1; §§10–11 | Windows CLI backend with direct process execution, job-object process-tree ownership, restricted execution controls where enforceable, isolated working directory, bounded stream capture, truthful degradation evidence, and the same portable evidence meanings as Linux | `crates/crucible-cli/tests/windows_run_cli.rs`, Windows capability-contract tests, platform evidence fixtures, and a resource-bounded Windows CI execution job | Planned |
| P1-H | Phase 1; §11 | Backend and architecture conformance matrix that explicitly classifies every declared Linux, macOS, and Windows x86_64/AArch64 cell; executes rather than merely compiles supported cells; proves common observation meanings; and records typed degraded, unavailable, or not-applicable evidence without silent skips | `crates/crucible-cli/tests/backend_conformance.rs`, capability-manifest golden fixtures, architecture runner evidence, and repository-policy tests for every declared matrix cell | Planned |
| P1-I | Phase 1 acceptance; §§10–11 and 84–89 | Bounded local-corpus execution on every supported backend through every delivery mode, with one immutable attempt and raw observation or typed harness failure per case, complete cleanup across all terminal paths, resource-adversarial fixtures, and a machine-readable Phase 1 acceptance report | `crates/crucible-cli/tests/phase_1_acceptance.rs`, backend corpus fixtures, Phase 1 report-schema tests, workspace Verus verification, and all canonical closure commands | Planned |

## Dependency order

```text
P1-A accepted Linux baseline
  ↓
P1-B accepted common adapter lifecycle
  ├──→ P1-C input-delivery matrix
  ├──→ P1-D cancellation and cleanup
  └──→ P1-E executor identity and evidence binding
          ├──→ P1-F macOS backend
          └──→ P1-G Windows backend
                    ↓
          P1-H backend/architecture conformance
                    ↓
          P1-I corpus and Phase 1 exit acceptance
```

P1-C and P1-D may proceed after P1-B in either order. P1-E may proceed concurrently with them, but
P1-F and P1-G must consume its common identity contract rather than invent platform-specific
evidence meanings. P1-I is the only Phase 1 exit slice and cannot be accepted by substituting unit
coverage for actual supported-backend process execution.

## Phase exit invariants

Phase 1 is complete only when all nine slices are accepted and the following statements hold for
the same revision:

- every declared supported host and architecture cell has executable capability evidence;
- every corpus case reaches exactly one immutable raw observation or one typed harness failure;
- target outcomes cannot be relabeled harness failures, and harness failures cannot become target
  findings;
- stdin, generated-file, argument, and environment delivery retain exact delivered-input identity;
- timeout, cancellation, capture failure, and adapter failure leave no target process alive;
- retained output and discarded-byte accounting stay exact at their configured bounds;
- requested isolation is never reported as enforced when the backend cannot enforce it;
- target-build, executor, runtime, proof, trusted-boundary, and effective-control identities remain
  independently inspectable; and
- all target-controlled bytes, paths, environment entries, output, and status records are admitted
  through bounded untrusted-input boundaries.

## Resource envelope

Each adapter and acceptance fixture must lower caller-selected limits beneath absolute caps for
input bytes, generated files, generated path length, argument count and bytes, environment count
and bytes, retained stdout and stderr, aggregate discarded-byte accounting, process count, address
space, file size, wall time, cancellation grace time, corpus entries, concurrent executions,
persistence batch size, and report bytes. Queues are bounded, native pipe readers continue draining
after retention fills, and platform jobs run with one Cargo build job and one Rust test thread.

## State transitions

The only ledger states are `Planned` and `Accepted`. A state changes to `Accepted` in the same
logical change that supplies every named acceptance artifact, updates any affected ADR and trusted
boundary records, and passes all closure commands. A failing test is retained only during the local
red step immediately preceding its implementation; accepted revisions remain green. Missing
capabilities fail with typed unsupported or unavailable evidence and never with placeholder
behavior.

## Canonical closure commands

```bash
cargo xtask format --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked -- --test-threads=1
cargo xtask verify --all
cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth
bash scripts/check-docs.sh
actionlint .github/workflows/*.yml
```
