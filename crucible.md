# Crucible

## Normative Design Specification

Crucible's draft implementation specification is divided into the authoritative slices below.
This index and the listed slices jointly form the complete specification. Each numbered section
has exactly one owning slice; the organizational slice titles are not additional specification
sections.

Section numbers are stable cross-reference identities. A reference such as `§12.2` continues to
name the same requirement after this split. Delivery order and current implementation status remain
in [ROADMAP.md](ROADMAP.md), which summarizes but does not replace or narrow these requirements.

| Sections | Authoritative slice |
| --- | --- |
| 1–5 | [Mission, Scope, and Principles](docs/spec/01-mission-and-principles.md) |
| 6–9 | [Architecture and Domain Model](docs/spec/02-architecture-and-domain-model.md) |
| 10–11 | [Targets, Execution, and Isolation](docs/spec/03-targets-execution-and-isolation.md) |
| 12 | [Configuration and Crucible YAML](docs/spec/04-configuration-and-crucible-yaml.md) |
| 13–29 | [Campaigns, Oracles, and Bug Model](docs/spec/05-campaigns-oracles-and-bug-model.md) |
| 30–42 | [Engines, Generation, and Corpus](docs/spec/06-engines-generation-and-corpus.md) |
| 43–53 | [Findings, Replay, and Minimization](docs/spec/07-findings-replay-and-minimization.md) |
| 54–70 | [Repair, Verification, and Agents](docs/spec/08-repair-verification-and-agents.md) |
| 71–80 | [Scheduling, Storage, CLI, and Reporting](docs/spec/09-scheduling-storage-cli-and-reporting.md) |
| 81–83 | [Phases, MVP, and Acceptance](docs/spec/10-phases-mvp-and-acceptance.md) |
| 84–93 | [Runtime Operational Contracts](docs/spec/11-runtime-operational-contracts.md) |
| 94–97 | [Expansion and Completion Standard](docs/spec/12-expansion-and-completion-standard.md) |

## Editing rules

- Change normative text only in its owning slice.
- Preserve numbered section identities unless an approved specification change explicitly migrates
  every affected reference.
- Do not duplicate requirements between slices; use section references or links instead.
- Keep roadmap status, delivery sequencing, and incomplete-work tracking out of normative sections
  unless they are themselves product requirements.
- Run `bash scripts/check-docs.sh` after structural changes. The check rejects missing, duplicated,
  reordered, or incorrectly owned numbered sections.
