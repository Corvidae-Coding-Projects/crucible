# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
intends to use [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once executable releases
begin.

## [Unreleased]

### Added

- Complete draft implementation specification for the Crucible correctness harness.
- Verus-first implementation saturation and trusted-boundary accounting requirements.
- Project-owned, Verus-authored Crucible YAML design and proof obligations.
- Evidence/provenance and multi-participant scenario architecture.
- Public governance, security, support, contribution, roadmap, and community-health policies.
- Documentation, dependency-review, and OpenSSF Scorecard automation.
- Pinned Verus workspace, verified typed identifiers, strict trusted-boundary policy, and code CI.
- Project-owned verified SHA-256, complete canonical digest decoding, algorithm-labeled artifact-ID
  parsing and dispatch, checksum-pinned NIST CAVP vectors, and content/size integrity checks.
- Verified structurally validated append-only evidence/provenance publication, typed identity
  conflicts and missing endpoints, constructor-only portable UTC timestamps, exhaustive stable-tag
  equality, retry-safe borrowed payloads, normative relation direction, atomic multi-input derived
  evidence, explicit no-configuration identity, and versioned envelopes.
- Project-owned Verus Crucible YAML profile-1 UTF-8 decoding with exact original-byte spans,
  explicit BOM policy, CR/CRLF normalization, typed malformed-input diagnostics, absolute and
  caller-lowered resource caps, private invariant-bearing constructors, a total pure success-or-error
  specification, and exact executable correspondence proofs.
- Verified profile-1 lexical atomization with exhaustive YAML-indicator classification, one-to-one
  decoded-scalar and source-span preservation, private constructors, bounded all-or-error output,
  exhaustive Unicode tests, and total executable-to-pure correspondence.
- Verified profile-1 line-layout analysis with exact atom and byte ranges, space-only indentation
  measurement, lossless leading-tab preservation for contextual scalar/separation decisions,
  deterministic resource-limit diagnostics, iterative progress, maximum-boundary fixtures, and
  total executable-to-pure correspondence.
- Verified profile-1 structural-candidate partitioning with canonical-layout authentication, exact
  lossless atom/byte coverage, directives and document markers, separation/comment/flow candidates,
  caller-lowered all-or-error bounds, typed mismatch and first-excluded diagnostics, iterative
  progress, a total pure model, and exact executable correspondence for success and failure.

[Unreleased]: https://github.com/Corvidae-Coding-Projects/crucible/commits/main
