# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
intends to use [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once executable releases
begin.

## [Unreleased]

### Added

- Added the first production `crucible` binary and `crucible init [path]`, creating the documented
  workspace layout plus an application-identified, versioned, integrity-checked SQLite database;
  initialization is idempotent, refuses incompatible or symlinked state, and is covered through
  compiled-binary process fixtures.
- Added monotonic workspace schema migration from v1 to v2 plus `crucible artifact import` and
  `crucible artifact verify`: verified canonical SHA-256 addressing feeds atomic no-clobber object
  publication, deduplication, source provenance, transactional database references, bounded reads,
  and post-publication corruption detection.
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
- Verified profile-1 context-sensitive single- and double-quoted scalar boundaries with canonical
  upstream authentication, provisional plain/block-region tracking, complete YAML 1.2 escape
  validation, exact source ranges, caller-lowered all-or-error caps, public semantic/range proofs,
  and total executable correspondence for success and every typed failure.
- Verified profile-1 context-sensitive plain-scalar boundaries with retained multiline presentation,
  flow-sensitive indicator and mapping-colon handling, node-property and block-region exclusion,
  contextual leading-tab diagnostics, exact source ranges, caller-lowered all-or-error caps, public
  semantic/range proofs, and total executable correspondence. Adversarial fixtures cover verbatim
  tag punctuation, coalesced flow colons, tab-only prefixes, block-header comments, and malformed
  `?`, `:`, and `-` scalar starts.
- Verified profile-1 literal and folded block-scalar formation with complete YAML 1.2 header,
  contextual compact-collection indentation, folding, and strip/clip/keep chomping behavior;
  contextual tabs; exact raw ranges; per-code-point direct/folded source provenance; independent
  scalar, presentation, scalar-content, and total-content caps; distinct typed upstream evidence
  diagnostics; a total pure model; exact executable correspondence for every success and failure;
  and a general proof that every authenticated nonempty success has exact rendered content and
  ordered non-overlapping atom/byte ranges.
- Verified profile-1 completed-token formation with canonical authentication of every preceding
  lexer witness; a lossless adjacent atom/byte partition; retained trivia and exact directive,
  property, alias, tag, and scalar identities; typed contextual indicators and bounded flow-stack
  validation; exact tag-character and BOM-context diagnostics; caller-lowered absolute limits; a
  total pure result model; exact executable correspondence; and public partition, formation,
  balance, anti-laundering, scalar-identity, parts, trivia-maximality, and limit proof surfaces.
- Verified alias-transparent canonical YAML DAG lowering after duplicate rejection and merge
  expansion, with stable source-node identity, normalized roots and collection edges, retained
  scalar/collection tags and merge provenance, graph sharing, four independent resource caps, a
  total pure result model, exact executable correspondence, and public anti-forgery contracts.
- Added verified compilation for the versioned typed-field schema graph consumed by configuration
  lowering, including every scalar and custom-tagged collection kind, nested sequence/mapping
  references, required-field metadata, stable field IDs, Unicode names, ambiguity rejection,
  caller-lowered resource caps, exact pure semantics, and public identity/uniqueness contracts.
- Added verified canonical-YAML value binding against every typed schema kind, with exact Core and
  custom tag separation, distinct finite/nonfinite scalar variants, alias-transparent resolved-node
  identity, scalar/collection record authentication, source-anchored typed diagnostics, a total
  pure result model, and public deterministic-output anti-forgery contracts.

[Unreleased]: https://github.com/Corvidae-Coding-Projects/crucible/commits/main
