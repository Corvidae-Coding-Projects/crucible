# Work slices for specification §§71–80

This acceptance ledger decomposes the normative
[scheduling, storage, CLI, and reporting specification](../spec/09-scheduling-storage-cli-and-reporting.md)
into independently reviewable vertical slices. It records evidence locations and gates; it does
not duplicate or alter normative requirements.

A slice is accepted only when its listed executable tests, Verus targets, documentation checks,
and strict trusted-boundary reconciliation all pass from the same source tree. Source comments are
not used for progress tracking.

| Slice | Normative scope | Product boundary | Acceptance evidence | State |
| --- | --- | --- | --- | --- |
| 09-A | §71 | Bounded seven-engine scheduler, baseline allocation, periodic adaptation, provenance credit, exploration floors, and capability disablement | `crates/crucible-core/tests/scheduler.rs` and workspace Verus verification | Accepted |
| 09-B | §72 | SQLite/filesystem and transactional-server/remote storage interfaces, crash-safe publication, generations and leases, conservative collection, bounded batching, retention modes, and signature scope admission | `crates/crucible-core/tests/storage_policy.rs`, `crates/crucible-cli/tests/storage_maintenance.rs`, and workspace Verus verification | Accepted |
| 09-C | §73 | Schema-version-4 domain/storage model, monotonic migration identity, independent evidence/configuration/engine/bundle/report versions, and scenario ownership constraints | `crates/crucible-cli/tests/storage_schema.rs`, `run_schema.rs`, and initialization migration tests | Accepted |
| 09-D | §74 | Complete documented command grammar with operational build, run, fuzz, inspection, storage maintenance, finding, replay, verification, report, configuration, capability, proof, TCB, and plugin surfaces | `crates/crucible-cli/tests/command_surface.rs` and the command-specific process suites | Accepted |
| 09-E | §§75–76 | Fact/hypothesis separation; bounded human, JSON, JSONL, SARIF, JUnit, evidence-graph, bundle, proof, TCB, capability, and plugin reports | `crates/crucible-cli/tests/reporting_cli.rs` and `inspect_cli.rs` | Accepted |
| 09-F | §77 | Fail-closed every-commit, nightly, and weekly tiers with explicit memory/process bounds and concrete regression, fuzz, property, parser, sanitizer, mutation, fault, exploration, conformance, proof, and TCB commands | `crates/crucible-xtask/tests/repository_policy.rs`, `actionlint`, and `.github/workflows/` | Accepted |
| 09-G | §78 | Opt-in JSON `tracing` events with correlation identities and strict separation from target output | `crates/crucible-cli/tests/logging_cli.rs` | Accepted |
| 09-H | §79 | Five derived seeds, external seed status, environment/schedule/fault/version/predicate evidence, finding replay sampling, and explicit non-determinism claims | `crates/crucible-core/tests/replay_seeds.rs`, `crates/crucible-cli/tests/finding_persistence.rs`, and `replay_cli.rs` | Accepted |
| 09-I | §80 | Eleven known-defect targets plus bounded untrusted-boundary corpora, sequential rediscovery, and tiered depth/soak execution | `crates/crucible-cli/tests/harness_selftest.rs`, `boundary_corpus.rs`, and `testdata/` | Accepted |

## Resource envelope

The canonical proof command uses one verifier thread. All three CI tiers set one Cargo build job,
one Rust test thread, disabled incremental compilation, explicit job timeouts, and at most one
platform-matrix job at a time. Runtime adapters cap configuration and report bytes, artifact size,
stream retention, scheduler inputs, collection candidates, SQLite values, persistence batches,
fixture size, process count, target memory, and target wall time. High-throughput fuzz success is
aggregated instead of retaining transient run graphs; failure evidence remains reachable.

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
