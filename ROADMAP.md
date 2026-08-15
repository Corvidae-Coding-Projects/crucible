# Roadmap

Crucible's complete scope and phase acceptance criteria live in the slices listed by the
[design specification index](crucible.md). This document summarizes delivery groups and the
current repository milestone; it does not replace or narrow the specification.

No dates are promised until implementation velocity and Verus/toolchain constraints are measured.

## Current milestone: Phase 0 implementation foundation

### Immediate breadth-first acceptance path

Implementation prioritizes the shortest runnable Phase 0-through-MVP path. The §§71–80 control-plane
slice is accepted: scheduler policy, schema-version-4 domain persistence, bounded publication and
collection, the documented CLI grammar, hard process-exit findings, inspection, replay sampling,
portable reports, structured logs, self-test corpora, and three CI depths now surround the first
Linux execution adapter. Its coherent acceptance units and exact evidence locations are recorded in
[the §§71–80 work-slice ledger](docs/work-slices/scheduling-storage-cli-reporting.md).

The next milestone proceeds from that accepted control plane into the remaining engine and adapter
phases in normative dependency order. Further theorem strengthening, exhaustive YAML conformance,
streaming corpus ingestion, native fuzzers, minimization reducers, additional platforms, and richer
scenario execution remain governed by their owning specification sections. Delivery order changes
do not remove an acceptance criterion or later capability.

Phase 1 execution-core breadth and exit acceptance are divided into the all-or-nothing units in
[the Phase 1 work-slice ledger](docs/work-slices/phase-1-execution-core.md). The existing Linux
direct-argument boundary and common exclusive adapter lifecycle are accepted; all input-delivery
modes, cancellation and terminal-path cleanup, executor identity completion, macOS and Windows
backends, architecture conformance, and corpus-level phase acceptance remain planned until their
complete evidence gates pass.

Current work establishes and extends:

- public project governance and contribution processes;
- the complete architectural specification;
- explicit Verus-first and trusted-boundary policy;
- the project-owned Crucible YAML requirements;
- evidence, provenance, scenario, build, plugin, proof, and capability identities;
- implementation-ready acceptance criteria;
- the pinned Verus/Rust/Z3/verusfmt workspace and reproducible proof command;
- a production `crucible init [path]` command that creates the documented workspace layout and an
  application-identified, explicitly migrated SQLite database; refuses incompatible, occupied, or
  symlinked managed state; verifies database integrity; and is idempotent for a valid workspace;
- production `crucible artifact import` and `crucible artifact verify` commands backed by verified
  SHA-256 object addressing, atomic no-clobber publication, deduplication, retained import
  provenance, transactional SQLite references, and post-publication integrity checking;
- production `crucible artifact check` and `crucible artifact gc` commands backed by bounded
  integrity scans, generation/lease barriers, and conservative preservation of normative roots;
- production `crucible config validate` and `crucible config canonicalize` commands backed by the
  complete version-1 field schema, recursive typed validation without coercion, cross-field
  execution invariants, deterministic canonical YAML, project-owned SHA-256 identity, exact typed
  diagnostics, explicitly charged bounded work, Unix descriptor-relative no-follow source
  admission, and fail-closed behavior on platforms awaiting an equivalent adapter. The current
  public Verus contract authenticates versions, bounds, charged work, and both digests; exact
  executable-to-pure field/canonicalization/invariant correspondence remains required by §12.2;
- a first production `crucible run <configuration>` Linux adapter that projects the validated
  execution configuration into an explicit capability plan; invokes one local CLI target with
  direct arguments through bubblewrap and prlimit; provides a private working directory, cleared
  environment, no target-visible host control mount or procfs, network isolation by default,
  wall-time/address-space/process/file-size controls, bounded concurrent stdout/stderr
  drain-and-discard capture, and process-group cleanup; and
  persists the source and canonical configurations, effective controls, capability and target-build
  manifests, exact stream accounting, portable raw observation, and disjoint typed harness failure
  through immutable content-addressed artifacts and schema-version-4 SQLite transitions. Other
  platforms, input-delivery modes, cancellation paths, and target adapters remain required breadth;
- verified typed identifiers and versioned identity envelopes;
- portable raw execution outcomes that retain completion, platform-native termination, and detected
  events as independent facts; enforce caller-lowered event, aggregate extension-metadata, and
  per-record payload limits; expose exact stable tags and semantic-validation proofs; and round-trip
  through a bounded canonical version-1 byte codec with typed malformed-input diagnostics and exact
  rejection-byte preservation;
- immutable raw observations retaining complete stream truncation accounting, portable resource
  counters, typed coverage-provider identity, namespace-versioned state/schedule/fault artifacts,
  and versioned extensions under independent caller-lowered limits, with full-field canonical
  round trips and retained rejection bytes. The pure exact byte-to-value correspondence proof
  remains depth work without reducing the required codec contract; the first Linux CLI adapter now
  supplies live bounded capture, effective controls, and SQLite observation persistence, while the
  remaining input modes and platform/target adapters stay on the breadth path;
- project-owned verified SHA-256, canonical artifact-ID parsing, and typed algorithm dispatch;
- object reachability checking, generation barriers, and conservative garbage collection on top of
  the bounded local artifact path; streaming and directory corpus ingestion remain later breadth;
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
- verified YAML merge-key expansion over the owned duplicate-free structural graph, recognizing
  only untagged plain `<<` and the exact explicit merge tag; accepting direct mappings or sequences
  of mapping aliases; enforcing earlier-source and explicit-receiver precedence through exact
  canonical structural bytes; retaining graph sharing and complete source-edge provenance rather
  than materializing alias trees; recording every mapping, including empty and unaffected mappings;
  preflighting intrinsic shape errors before independently caller-lowered mapping, expanded-entry,
  full-tree-reference, and merge-source caps; total executable-to-pure correspondence; exact input
  identity and deterministic-output anti-forgery proofs; and deep traversal, precedence, shape,
  tag/style distinction, provenance, and exact-boundary fixtures;
- verified canonical YAML graph lowering over the owned merge-expanded source, retaining one stable
  record for every source node while eliminating alias kinds from the lowered type; normalizing
  sequence children, effective mapping keys and values, and document roots to final targets;
  preserving scalar and collection tag identities plus exact source-edge, receiver, merge-source,
  and inherited/explicit provenance; reusing shared target edge intervals instead of materializing
  alias trees; independently caller-lowered node, sequence-entry, mapping-entry, and document-root
  caps with exact first-excluded diagnostics; total executable correspondence; and public exact
  input-identity and deterministic-output anti-forgery proofs;
- verified typed-field schema compilation for the next lowering stage, with independent schema and
  compiler versions; exact scalar, sequence, mapping, and custom-tagged value kinds; nested
  sequence-item and mapping-field references; losslessly retained required-field metadata and
  globally stable field IDs; contiguous ownership of mapping field ranges; nonempty Unicode scalar
  names; global ID and per-mapping name uniqueness; independently caller-lowered node, field, and
  aggregate-name caps; total executable correspondence; and public exact-input identity and
  deterministic-output anti-forgery proofs;
- verified canonical-value/schema-node binding as the kind-authentication submachine for typed-field
  lowering, distinguishing all Core scalar variants, positive and negative infinity, NaN, strings,
  custom local/global scalars, Core sequences/mappings, and custom local/global collections without
  coercion; retaining alias-transparent resolved-node, scalar/collection, schema-node, and exact
  source-range identities; rejecting forged record identities and bounds; and proving total
  executable correspondence plus deterministic-output anti-forgery semantics;
- verified mapping-field partitioning over the canonical YAML DAG, matching only exact decoded
  string keys, validating every recognized value against its owned schema node without coercion,
  emitting recognized records in schema order, retaining direct/inherited mapping provenance, and
  rejecting duplicate, unknown, missing-required, non-string-key, and wrong-kind inputs with typed
  exact diagnostics; the default policy rejects unknown fields while the explicit compatibility
  policy preserves each unknown key/value node and source identity losslessly; independent field
  and aggregate-key-code-point caps are caller-lowerable but absolutely bounded; executable output
  equals the total pure model, whose public semantics prove strict schema order, required-field
  coverage, default-reject exclusion of unknown records, and exact inspectable scan/emission;
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
