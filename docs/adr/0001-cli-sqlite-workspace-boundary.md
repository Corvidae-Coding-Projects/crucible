# ADR-0001: CLI workspace initialization and SQLite host boundary

- **Status:** Accepted
- **Date:** 2026-08-13
- **Decision owners:** dollspace.gay (approved 2026-08-13)
- **Related issue:** Phase 0 `crucible init` acceptance slice
- **Supersedes:** None
- **Amended by:** ADR-0002 (schema version 2 and the separate artifact host boundary), ADR-0004
  (schema version 3 and immutable run evidence)

## Context and evidence

Phase 0 requires `crucible init` to create a valid workspace and versioned database. The pinned
Verus toolchain cannot specify operating-system argument delivery, filesystem mutation, process
completion, SQLite's C implementation, or `rusqlite`'s foreign calls. Continuing to elaborate
pure YAML machinery does not deliver this runnable acceptance outcome.

## Decision

Add a `crucible` binary with `crucible init [path]`. It creates the documented `.crucible`
directory layout. This decision originally initialized schema version 1 in an
application-identified SQLite database; ADR-0002 now makes new workspaces version 2 and migrates
exact version-1 workspaces monotonically.
The command is idempotent for a valid Crucible workspace and refuses an incompatible or occupied
state path.

Use `rusqlite` 0.40.2 with default features disabled and bundled SQLite enabled. Keep command
selection in Verus Rust and isolate initialization behavior in three registered external bodies:
argument acquisition, workspace/database initialization, and process completion. ADR-0002 adds a
separately approved fourth boundary for artifact filesystem and SQLite operations.

## Preserved invariants

- Initialization never intentionally removes or replaces an existing path.
- An existing database is accepted only when its application ID, schema version, migration row,
  workspace-format metadata, and SQLite integrity check all match.
- Migration history and the database schema version are explicit and independently queryable.
- Repeating initialization does not duplicate migration state.
- This slice does not reduce any Phase 0 through Phase 22 capability.

## Alternatives considered

- Invoking the host `sqlite3` executable adds process-discovery and command-execution boundaries
  while making deployment depend on a separately installed tool.
- Depending on the system SQLite library reduces build work but makes supported features and
  versions host-dependent. Bundled SQLite is more reproducible for an application-owned database.
- Deferring persistence would continue the depth-first YAML focus and would not advance the Phase
  0 executable acceptance path.

## Verus and trusted-boundary impact

The three initialization `external_body` boundaries are approved and registered. Their exact assumptions and
independent checks are recorded in `tcb/ledger.tsv`. `rusqlite`, `libsqlite3-sys`, and bundled
SQLite are unverified dependencies reachable through the initialization and later ADR-0002
artifact boundaries. A future
verified filesystem or SQLite model should narrow the boundary; the boundary is reconsidered on
every Verus, rusqlite, or SQLite upgrade.

## Security, privacy, and authorization impact

The command writes only below the selected workspace root. It performs no network access, starts
no target, reads no secrets, and does not broaden target authorization. SQLite foreign keys,
full synchronous mode, an application ID, an explicit schema version, and an integrity check are
enabled from the first migration.

## Compatibility and migration

Schema version 1 creates only migration history and workspace-format metadata. Later tables are
added by monotonic numbered migrations. A database with another application ID or unsupported
schema version is rejected instead of being adopted or rewritten.

## Verification and acceptance

Process tests must demonstrate exact layout creation, database identity and schema version,
migration history, metadata, integrity, idempotence, current-directory behavior, non-overwrite of
an occupied state path, and invalid-argument failure. The Verus all-target check and strict
trusted-boundary audit must pass before publication.

## Consequences and follow-up

This adds a real executable and embedded persistence dependency while growing the explicit TCB.
Next work should import and integrity-check one artifact through the initialized workspace, then
connect configuration validation and canonical digesting. Broader schema tables arrive with the
vertical feature that owns them rather than as speculative empty tables.
