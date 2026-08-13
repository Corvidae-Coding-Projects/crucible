# ADR-0002: Local content-addressed artifact publication boundary

- **Status:** Accepted
- **Date:** 2026-08-13
- **Decision owners:** dollspace.gay (approved 2026-08-13)
- **Related issue:** Phase 0 imported-artifact acceptance slice
- **Supersedes:** None

## Context and evidence

Phase 0 requires an imported artifact to survive an integrity check. The project already owns a
Verus-verified SHA-256 implementation, canonical artifact identity, and exact reference checking,
but had no durable object store or metadata reference. Verus does not specify host path traversal,
file publication and syncing, SQLite transactions, or foreign SQLite behavior.

## Decision

Add `crucible artifact import <file> [workspace]` and `crucible artifact verify <artifact-id>
[workspace]`. Keep hashing, canonical object-address derivation, publication planning, stored-record
comparison, integrity checking, command parsing, and error routing in Verus Rust. Isolate the
remaining filesystem and SQLite behavior in one registered artifact host boundary.

Schema migration 2 adds exact `artifacts` and `artifact_imports` tables. Objects use
`objects/sha256/ab/cd/<full_hash>`. Publication writes and syncs a private temporary file, verifies
its completed bytes, atomically links it into place without replacement, syncs its directory chain,
then commits the artifact and provenance rows in an immediate SQLite transaction. A database
failure may leave an unreachable complete object, but never a row naming a partial object.

The initial local command has an explicit 64 MiB input cap so it cannot allocate from an unbounded
host file. Streaming hashing/import, directory corpus ingestion, reachability checking, generation
barriers, and conservative garbage collection remain required later capabilities.

## Preserved invariants

- Every selected object path is derived from a canonical supported digest through verified
  lowercase-hex components; source paths and user-provided IDs cannot inject path components.
- Existing objects are never replaced. Duplicate content reuses one object and one artifact row.
- Distinct import paths retain distinct provenance rows, while repeating the same import is
  idempotent.
- A committed database row is created only after complete object publication.
- Import success is returned only after a fresh load agrees on identity, size, bytes, and source
  provenance under the verified predicate.
- Existing exact schema-v1 workspaces migrate monotonically; incompatible databases remain
  untouched.
- No later Phase 0 through Phase 22 artifact, corpus, remote-store, or garbage-collection capability
  is removed.

## Alternatives considered

- Storing large bytes in SQLite would couple the metadata backend to object transport and make the
  remote-store interface harder to preserve.
- A check-then-rename sequence can overwrite a concurrently published path on Unix. Same-filesystem
  hard-link publication gives atomic no-clobber visibility with standard safe Rust.
- Trusting a host hash utility or SQLite extension would duplicate the project-owned digest and add
  another command or foreign-code boundary.
- Unbounded whole-file reads are simpler but contradict the harness resource-safety requirement.

## Verus and trusted-boundary impact

`CLI-HOST-ARTIFACT-001` covers bounded host reads, filesystem identity and symlink observations,
private temporary-file publication and syncing, and SQLite reads/transactions. The verified caller
does not trust a claimed publication: it reloads the row and bytes and checks them against the
project-owned digest and expected provenance. The boundary must be reconsidered on relevant Verus,
Rust filesystem, rusqlite, libsqlite3-sys, or bundled SQLite changes.

## Security, privacy, and authorization impact

Source and workspace ancestors, managed directories, database files, object parents, and object
files are rejected when symlinked or of the wrong kind. Created object directories are mode 0700
and temporary/object files mode 0600 on Unix. The implementation performs no network access and
does not execute imported content. Check/use races remain part of the explicit host-boundary
assumption until a verified descriptor-relative filesystem API is available.

## Compatibility and migration

Fresh workspaces start at SQLite schema version 2 with both migration rows. An exact version-1
workspace is upgraded in one transaction and re-inspected before success. Extra, altered, forged,
or otherwise incompatible schema/history remains a typed failure rather than being adopted.

## Verification and acceptance

Runtime fixtures cover exact addressing and rows, canonical source provenance, source removal,
object and database-record corruption, duplicate contents, repeated and concurrent publication,
malformed IDs, symlink sources and fanout directories, oversized sparse sources, restart states on
both sides of object publication, argument shapes, and concurrent v1 migration. Proof-contract
fixtures require executable/pure correspondence, reject forged database digests, and prove
canonical object names contain no path separators. Strict TCB, formatting, Clippy, runtime, and
all-target Verus gates must pass before publication.

## Consequences and follow-up

The shortest Phase 0 operational spine now reaches durable artifact storage. The next breadth slice
should validate, canonicalize, lower, and digest a real configuration through the production CLI.
Depth work then adds streaming and directory import, richer media typing, crash fault injection,
reachability/garbage collection with generation barriers, and remote object-store parity.
