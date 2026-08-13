# Roadmap

Crucible's complete scope and phase acceptance criteria live in the
[design specification](crucible.md). This document summarizes delivery groups and the current
repository milestone; it does not replace or narrow the specification.

No dates are promised until implementation velocity and Verus/toolchain constraints are measured.

## Current milestone: Phase 0 implementation foundation

Current work establishes and extends:

- public project governance and contribution processes;
- the complete architectural specification;
- explicit Verus-first and trusted-boundary policy;
- the project-owned Crucible YAML requirements;
- evidence, provenance, scenario, build, plugin, proof, and capability identities;
- implementation-ready acceptance criteria;
- the pinned Verus/Rust/Z3/verusfmt workspace and reproducible proof command;
- verified typed identifiers and versioned identity envelopes;
- project-owned verified SHA-256, canonical artifact-ID parsing, and typed algorithm dispatch;
- verified structurally valid append-only evidence/provenance transitions, atomic multi-input
  derivations, retry-safe borrowed publication, and versioned evidence envelopes;
- project-owned Crucible YAML profile-1 byte decoding with exact source spans, typed UTF-8
  diagnostics, explicit BOM behavior, line-ending normalization, resource caps, and Verus
  total-result correspondence proofs;
- bounded verified lexical atomization with exhaustive indicator classification and one-to-one
  decoded-scalar/span preservation as the context-free foundation of the remaining YAML lexer;
- verified line and indentation layout with exact atom ranges, original byte offsets, lossless
  leading-tab preservation, and caller-lowered absolute resource caps for the context-sensitive
  lexer that decides tab legality from scalar and separation context;
- verified lossless structural-candidate partitioning with canonical-layout authentication, exact
  atom/byte spans, bounded all-or-error output, and candidate roles retained for final
  context-sensitive scalar and flow interpretation;
- verified context-sensitive single- and double-quoted scalar boundaries with canonical upstream
  authentication, provisional plain/block-region tracking, YAML-printable-character admission,
  complete YAML 1.2 escape-spelling validation, publicly proved exact and non-overlapping raw
  ranges, bounded all-or-error output, and total executable-to-pure correspondence;
- verified context-sensitive plain-scalar boundaries with authenticated quoted-scalar exclusion,
  exact retained presentation ranges, node-property and block-region exclusion, flow-sensitive
  indicators and mapping colons, contextual leading-tab diagnostics, caller-lowered absolute caps,
  public semantic/range proofs, and total executable-to-pure correspondence;
- verified literal and folded block-scalar formation with authenticated exclusion from quoted,
  plain, property, and flow regions; both header-modifier orders; automatic, explicit, and
  all-empty indentation derived from compact block-collection grammar context; contextual tab
  diagnostics; strip/clip/keep chomping; exact folding and more-indented-line behavior; four
  independent resource caps; distinct upstream evidence diagnostics; raw ranges; normalized
  content provenance; total executable-to-pure success/error correspondence; and general nonempty
  exact-render/non-overlap proofs;
- verified completed YAML token formation with canonical authentication of every preceding lexer
  witness; retained indentation, separation, comments, line feeds, and document-prefix BOMs; exact
  directive/property/alias payload ranges; scalar shielding; context-sensitive document and
  collection indicators; bounded typed flow-stack validation; a complete lossless atom/byte
  partition; distinct exact-offset diagnostics; and total executable-to-pure result correspondence
  with public semantic, partition, and flow-balance proofs;
- verified nonrecursive YAML concrete-syntax-tree formation over the authenticated completed-token
  stream, covering multidocument bare/explicit/directive forms, block and flow collections,
  compact mappings, arbitrary and empty keys/values, aliases, both property orders, and every
  scalar style; exact prefix/directive/start/root/end/suffix document regions; lossless trivia;
  child-before-parent table ordering; exact syntax-token ownership; caller-lowered document, node,
  entry, directive, warning, and depth caps; typed first-impossible diagnostics; a total fueled pure
  parser; strict machine progress; deterministic results; and exact executable-to-pure
  correspondence with adversarial forgery proof fixtures;
- verified YAML 1.2.2 Core plain-scalar classification with exact null, boolean, decimal/octal/hex
  integer, finite decimal, infinity, and NaN spellings; explicit exclusion of YAML 1.1 boolean and
  octal coercions; exact numeric digit subranges; caller-lowered absolute scalar limits; and exact
  executable-to-pure correspondence as the first semantic-resolution submachine;
- verified host-width-independent Core integer conversion into canonical little-endian
  base-1,000,000,000 limbs, with decimal/octal/hex equivalence, positive-zero normalization,
  per-digit canonicalization, caller-lowered absolute limb limits, overflow proofs, and exact
  executable-to-pure correspondence;
- verified exact Core finite-decimal conversion into canonical little-endian decimal coefficient
  and signed-exponent digits, with equivalent-spelling normalization, distinct negative zero,
  linear-time arbitrary-width handling, caller-lowered coefficient/exponent limits at exact source
  anchors, no IEEE-754 intermediate, and exact executable-to-pure correspondence;
- verified canonical Core infinity and NaN conversion, with all YAML 1.2.2 case variants, distinct
  positive and negative infinity, one NaN value, exact caller-lowered input diagnostics, and exact
  executable-to-pure correspondence;
- verified semantic decoding of authenticated literal and folded block content into the shared
  scalar-content representation, retaining exact direct/folded atom and byte provenance, empty
  content, caller-lowered first-excluded output diagnostics, and executable-to-pure correspondence;
- verified semantic decoding of authenticated single-quoted content into the shared scalar-content
  representation, including direct Unicode, quote doubling, multiline flow folding, exact atom and
  byte provenance, empty content, caller-lowered first-excluded output diagnostics, and total
  executable-to-pure correspondence;
- verified semantic decoding of authenticated double-quoted content into the shared scalar-content
  representation, including every YAML 1.2.2 simple and hexadecimal escape, ordinary flow folding,
  escaped line breaks and following empty-line content, exact atom/byte provenance, empty content,
  caller-lowered first-excluded output diagnostics, and total executable-to-pure correspondence;
- verified semantic decoding of authenticated plain-scalar presentation into the shared
  scalar-content representation, including direct Unicode, preserved internal space/tab content,
  multiline flow folding, exact atom/byte provenance, caller-lowered first-excluded output
  diagnostics, input authentication, and total executable-to-pure correspondence;
- verified CST-node scalar dispatch across empty, plain, single-quoted, double-quoted, literal, and
  folded styles, binding each scalar record to its exact CST node and completed token, preserving
  the style-specific decoded provenance without fabricating content for zero-width empty nodes,
  returning collections separately, authenticating every selected producer, and proving total
  executable-to-pure correspondence for graph-composer consumption;
- verified scalar-value composition from those authenticated records, including implicit YAML
  1.2.2 Core null/boolean/arbitrary-width integer/exact finite-float/infinity/NaN/string values;
  quoted/block string behavior; explicit `!!null`, `!!bool`, `!!int`, `!!float`, and `!!str`
  spelling checks; scalar rejection of `!!seq`/`!!map`; non-specific `!`; losslessly retained local
  and global custom tags; exact nested-limit diagnostics; and anti-forgery node-index binding;
- verified collection-tag composition for sequence and mapping nodes, including implicit and
  non-specific Core kind tags, exact `!!seq`/`!!map` compatibility, rejection of scalar standard
  tags on collections, losslessly retained custom local/global tags, authenticated scalar/alias
  bypass, exact tag-limit and node-index diagnostics, and total executable-to-pure correspondence;
- verified bounded semantic-topology projection with one exact document-root record per CST
  document, one identity/range-preserving record per CST node, and one source-ordered edge record
  per sequence or mapping entry; independently caller-lowered caps with exact first-excluded byte
  diagnostics; authenticated CST/token inputs; public anti-laundering semantics; and total
  executable-to-pure correspondence;
- verified aggregate semantic scalar-table population in CST order, covering every scalar and
  zero-width empty node exactly once; preserving complete resolved values, tags, presentation, and
  provenance; enforcing independent scalar-record, aggregate-content, and nested scalar limits at
  the exact first excluded code point; and proving total executable correspondence plus public
  exact-coverage and exact-accounting extraction;
- verified document-scoped explicit tag-property resolution with default and `%TAG`-overridden
  primary, secondary, and named handles; verbatim and local identities; exact YAML 1.2.2
  percent-escape preservation; absolute global-tag URI admission; per-code-point provenance;
  document-boundary reset; caller-lowered first-excluded diagnostics; authenticated CST/token
  inputs; and total executable-to-pure correspondence;
- verified document-scoped anchor/alias binding in presentation-token order, including exact
  Unicode-name comparison, duplicate-anchor shadowing, collection anchors visible to descendants
  before parent-node completion, document-boundary reset, forward/cross-document rejection,
  independent caller-lowered declaration and binding limits, exact source records, authenticated
  CST ownership, and total executable-to-pure correspondence;
- verified aggregate semantic node-table composition that freshly authenticates and owns the exact
  topology, scalar, and anchor/alias sources; assigns every CST node one exact scalar, sequence,
  mapping, or alias slot in CST order; retains collection tags, topology/property/edge intervals,
  and complete alias redirects without copying targets; enforces independent caller-lowered node,
  collection, redirect, and nested tag limits at exact first-excluded source anchors; and exposes
  the total pure composition result through public anti-laundering proof contracts;
- verified alias-cycle rejection over the owned semantic node table, with intrinsic direct and
  indirect cycle diagnostics taking precedence over caller traversal caps at the exact closing
  alias name; a proof that every accepted sequence, mapping, and alias edge strictly decreases the
  stable CST node identity; exact per-node semantic depths and completion states; deterministic
  visit order and deepest-path evidence; independently caller-lowered depth and work-stack limits;
  total executable correspondence; and a forged-forward-redirect anti-laundering proof fixture;
- verified canonical scalar-key identities over every resolved scalar node, with presentation-
  independent Core null, boolean, arbitrary-width integer, exact finite-float, infinity, NaN, and
  Unicode string encodings; exact lossless local/global custom-tag identity; explicit variant and
  length delimiters rather than hash equality; per-byte source anchors; independently caller-
  lowered record, per-key byte, and aggregate-byte limits; allocation-free streaming over retained
  scalar/tag content; total executable-to-pure correspondence; and public exact-result proof
  contracts;
- verified canonical structural-key identities over every semantic node, reusing exact scalar
  identities, making aliases byte-for-byte transparent, preserving sequence order, ignoring
  mapping presentation order through a verified iterative bottom-up merge sort over canonical
  key/value pairs, and retaining complete custom collection-tag identity; one exact record per
  node, explicit markers and length delimiters, exact per-code-point custom-tag provenance and
  per-byte child provenance, independently caller-lowered record/per-key/aggregate/mapping-sort
  caps at exact node anchors, total executable-to-pure correspondence, public exact-result proof
  contracts, all mapping permutations, nested collection-key, alias, and exact-boundary fixtures;
- verified duplicate explicit-key rejection after canonical resolution, using exact canonical byte
  equality rather than host hashing; scalar, alias, sequence, mapping, empty-key, and custom-tag
  equality; one-mapping scope; the globally earliest later equal key by source byte as the exact
  diagnostic despite child-before-parent internal node order; intrinsic duplicate precedence over
  a caller limit excluding the same mapping/key; independently lowered mapping and aggregate
  mapping-entry accounting; an owned duplicate-free structural source; total executable
  correspondence; an explicit all-pairs-distinct success theorem; and provenance-laundering forgery
  rejection;
- verified trusted-boundary scanning, ledger/approval reconciliation, and code CI.

Exit requires every Phase 0 implementation and acceptance criterion in the design specification;
the completed Phase 0.1 slice is a dependency checkpoint, not a reduction of that exit standard.

## Delivery groups

### Foundation and first operational release

Phases 0–5 deliver the Verus-Rust workspace, Crucible YAML, proof and trusted-boundary
infrastructure, evidence storage, CLI execution, hard oracles, replay, minimization, bundles,
corpus management, and managed/native coverage-guided fuzzing.

This is the first operational milestone, not the completed product.

### Semantic and stateful correctness

Phases 6–8 add property and metamorphic testing, stateful and model-based testing, and
reproducible fault injection.

### Concurrency, adequacy, and symbolic exploration

Phases 9–11 add deterministic concurrency exploration, mutation testing, and symbolic/concolic
integration.

### Repair, agents, and external formal systems

Phases 12–14 add patch registration and adversarial verification, contained evidence-grounded
agents, and external proof-system integration. Verus verification of Crucible itself begins in
Phase 0 rather than Phase 14.

### Multi-participant and specialized targets

Phases 15–21 add scenario/service topology, distributed-system correctness, statistical
performance and complexity testing, compatibility and migration, compilers/toolchains,
VM/kernel targets, and embedded/hardware-in-the-loop targets.

### Distributed execution

Phase 22 adds distributed workers and remote build/proof farms while preserving identical
evidence meaning, idempotency, cancellation, and artifact integrity.

## Expansion track

Committed expansion targets include grammar and invariant learning, semantic and taint coverage,
regression localization, richer proof pipelines, proof-carrying patches, specialized WebAssembly,
GPU, mobile, browser, GUI, filesystem and database adapters, deterministic device simulation,
privacy-preserving corpus exchange, and translation validation.

## How priorities are chosen

Maintainers consider dependency order, assurance risk, breadth of bug classes, user evidence,
Verus support, platform capability, contributor availability, and the ability to leave a phase
in a genuinely accepted state. Priority changes reorder delivery; they do not silently delete
scope.
