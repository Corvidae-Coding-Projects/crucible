# ADR-0003: Raw execution outcome codec and UTF-8 materialization boundary

- **Status:** Accepted
- **Date:** 2026-08-13
- **Decision owners:** dollspace.gay (approved 2026-08-13)
- **Related issue:** Phase 0 initial domain-record serialization and Phase 1 execution evidence
- **Supersedes:** None

## Context and evidence

The portable raw execution outcome is the first breadth-first domain record needed between target
adapters, immutable observations, persistence, replay, and hard-failure oracles. An in-memory Rust
wrapper with a version integer did not serialize anything and could not retain an actual future
payload. Extension records also lacked the configured payload and inline-metadata limits required
by the design.

The pinned Verus toolchain specifies UTF-8 at the mathematical sequence level and specifies String
mutation, but it does not specify safe `char::from_u32` or `std::str::from_utf8`. A direct probe was
rejected by Verus as unsupported. A decoder cannot materialize arbitrary valid Unicode namespaces
and media types as Rust `String` values without one of those host operations.

## Decision

Adopt the exact canonical binary version-1 format in specification section 9.5 and implement its
encoder, byte parser, tag dispatch, bounds, semantic validation, error routing, and rejection-byte
preservation in Verus Rust.

Register one narrow boundary, `CORE-HOST-UTF8-001`, solely for safe Rust UTF-8 validation and
materialization of an already range-checked byte slice. It calls `std::str::from_utf8`, returns the
exact valid text or the standard library's first invalid-byte offset, and performs no I/O. The
verified caller checks the global encoded-byte cap and the declared string length against remaining
input and applicable aggregate metadata budget before entering the boundary.

## Preserved invariants

- Completion, termination, and detected events remain independent and retain exact event order.
- Unix, Windows, embedded, harness, and versioned platform-specific evidence are never coerced into
  one operating system's model.
- Every stable tag and fixed-width field has one canonical big-endian spelling.
- Every decoder failure retains the exact original bounded byte vector, including unknown future
  schema versions.
- Decoded current-version values pass the same Verus semantic validator as directly constructed
  outcomes before becoming validated values.
- Extension namespace and media-type accounting is aggregate; payload size is independently capped
  per out-of-line record; caller limits cannot raise absolute policy.
- This slice does not remove `RawObservation`, other initial domain serializers, persistence,
  bounded stream capture, execution controls, or any Phase 1 through Phase 22 capability.

## Alternatives considered

- Keeping the in-memory envelope would preserve a Rust move, not serialized evidence, and could
  falsely relabel current data as a future schema.
- Restricting extension text to ASCII would reduce the design's Unicode namespace capability.
- Replacing every existing artifact and extension text field with a private code-point container
  would broaden this slice across persistence, CLI, and YAML APIs without eliminating the eventual
  host conversion needed by ordinary Rust consumers.
- An unchecked UTF-8 conversion would add unsafe code and a stronger assumption than the safe
  standard-library operation.

## Verus and trusted-boundary impact

`CORE-HOST-UTF8-001` is one `external_body`. Its consequence is limited to the truth of the returned
String/invalid-offset pair; it cannot bypass encoded-byte, event, metadata, payload, stable-tag,
trailing-byte, or post-decode semantic validation. Runtime fixtures cover ASCII and multibyte
round trips plus invalid UTF-8 offsets. The boundary must be reconsidered whenever the pinned Verus
or Rust string specifications add a supported safe UTF-8 materialization path.

The owner approved this single TCB addition on 2026-08-13. Strict TCB gates and publication require
that approval to remain bound to the exact registered source occurrence, byte count, and line count.

## Security, privacy, and authorization impact

The codec performs no filesystem, process, network, target, or secret operation. The input byte cap
is checked before parser allocation. Declared counts and lengths are checked before vector or String
growth, and future or malformed data remains inert retained evidence.

## Compatibility and migration

Version 1 is identified by `CRXO` plus schema version 1. Stable tags, field order, integer widths,
endianness, and canonical string encoding are persisted compatibility commitments. A future schema
gets a new version and decoder; current decoders preserve but do not interpret its bytes. The former
in-memory envelope was unpublished and is removed rather than treated as a wire format.

## Verification and acceptance

Acceptance requires exact golden bytes, every termination/event family, multibyte text, semantic
round trips, cap-first precedence, exact/caller-lowered payload and aggregate metadata limits,
truncation, trailing bytes, invalid magic/tags/options/booleans/UTF-8, future-version preservation,
stable-tag mappings, executable Verus checks, public semantic-success and rejection-preservation
contracts, and the strict TCB audit.

## Consequences and follow-up

Raw outcomes now have a real persistence/transport contract usable by observations and adapters.
The one host text boundary is explicit rather than hidden. Next breadth work adds `RawObservation`,
captured-stream/resource records, and its versioned serialization, then persists those records and
connects the first bounded CLI target adapter. Stronger full-codec algebraic proofs and structured
codec fuzzing remain assurance-depth work, not removed scope.
