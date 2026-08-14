# Harness-boundary fixtures

This corpus maps every §80 harness boundary to an executable rejection or containment test. The
fixtures are deliberately inert data; test code chooses the parser, storage adapter, plugin
transport, scenario controller, proof runner, or target-output channel that receives them.

| Boundary | Fixture | Exercised by |
| --- | --- | --- |
| malformed/resource-adversarial YAML | `yaml/resource-limit.yaml` | configuration source/limit tests |
| duplicate key, alias, cycle, canonicalization | `yaml/duplicate-and-cycle.yaml` | Crucible YAML resolution tests |
| corrupt artifact/evidence graph | `storage/corrupt-record.txt` | artifact and inspection integrity tests |
| partial database/object publication | `storage/interrupted-publication.txt` | artifact recovery and GC tests |
| plugin violation/stall | `plugin/protocol-violation.jsonl` | bounded boundary-corpus parser and nightly corpus job |
| scenario cancellation/cleanup | `scenario/cancel.trace` | bounded boundary-corpus parser and weekly scenario-topology job |
| non-offensive guest escape attempt | `vm/escape-attempt.txt` | bounded boundary-corpus parser and local isolation fixtures |
| prompt injection in hostile evidence | `hostile/prompt-injection.txt` | bounded boundary-corpus parser and structured-log separation test |
| unregistered/false assumption | `proof/unregistered-boundary.rs.txt` | bounded boundary-corpus parser and strict TCB known-defect job |
| proof timeout/solver/cache failure | `proof/failure-modes.tsv` | bounded boundary-corpus parser and proof report contract |

`crates/crucible-cli/tests/boundary_corpus.rs` names and validates each corpus explicitly, and the
nightly workflow invokes that exact test target. Absence cannot silently turn into a zero-test pass.
