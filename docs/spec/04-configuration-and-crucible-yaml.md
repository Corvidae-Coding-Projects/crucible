# Configuration and Crucible YAML

## 12. Configuration Format

Example `crucible.yaml`:

```yaml
version: 1

language:
  profile: crucible-yaml-1

project:
  name: example-parser

target:
  adapter: cli
  command: ./build/example-parser
  args:
    - "{input_file}"

execution:
  timeout_ms: 2000
  memory_mb: 1024
  max_processes: 32
  max_output_mb: 16
  network: false
  required_capabilities:
    - process_group_termination
    - resource_limits

oracles:
  process_exit:
    allowed_codes: [0]
    timeout_is_failure: true

inputs:
  corpus:
    - ./seeds/

engines:
  fuzz:
    enabled: true
    modes:
      - managed
      - native
    native_backends:
      - afl++
      - libfuzzer
      - honggfuzz

  property:
    enabled: true

  differential:
    enabled: false

  metamorphic:
    enabled: true

  fault:
    enabled: true

  concurrency:
    enabled: false

  symbolic:
    enabled: false

  mutation:
    enabled: false

sanitizers:
  address: true
  undefined: true
  thread: false
  memory: false
  leak: true

campaign:
  duration: 8h
  workers: 8
  seed: 123456789

storage:
  root: .crucible

verification:
  verus:
    required: true
    deny_unregistered_assumptions: true
    deny_unapproved_tcb_growth: true
```

### 12.1 Crucible YAML

Crucible owns its configuration language and implementation. `crucible-yaml` is a
YAML-compatible, versioned configuration language written in Verus Rust; it must not wrap or
delegate parsing to `serde_yaml` or another general YAML parser.

The implementation pipeline is:

```text
untrusted bytes
    ↓ verified UTF decoding or byte-level diagnostic
normalized decoded scalars with exact source spans
    ↓ verified lexical atomization
scalar-preserving lexical atoms
    ↓ verified line and indentation layout
exact line descriptors over atom ranges
    ↓ verified context-sensitive lexer
tokens with exact spans
    ↓ verified parser
concrete syntax tree
    ↓ alias, tag, and scalar resolution
semantic YAML graph
    ↓ duplicate rejection, merge application, and canonical graph lowering
alias-transparent canonical YAML DAG
    ↓ schema-directed typed-field lowering
typed Crucible configuration
    ↓ schema-version, unknown-field, and cross-field validation
verified effective configuration
```

Each arrow is a named, versioned transformation with an executable implementation, a pure
Verus specification, and correspondence proofs wherever the pinned Verus toolchain can express
them. The canonical lowered representation is the authoritative input to campaign identity,
not parser-internal objects or host paths.

#### 12.1.1 Effective configuration schema and canonicalization version 1

The initial production configuration bridge consumes exactly one profile-1 YAML document whose
root is a mapping. Configuration-schema version 1 is the complete field tree shown in the example
above: `version`; `language.profile`; `project.name`; `target.adapter`, `command`, and `args`;
every shown `execution`, `oracles`, `inputs`, `engines`, `sanitizers`, `campaign`, `storage`, and
`verification` field; and no undeclared field. Every shown field is required in schema version 1.
This strict initial schema avoids silent host-dependent defaults. A later optional field, default,
deprecation, or compatibility rule requires an independently versioned configuration-schema
decision; the lossless unknown-field compatibility mechanism remains available to migrations and
inspection, while the production execution path rejects unknown fields by default.

All mappings are lowered through the compiled typed-field schema and all sequences through their
declared item schema. No string-to-number, number-to-string, collection-to-scalar, tag-erasing, or
host-width coercion is permitted. Version 1 accepts only `version: 1`, language profile
`crucible-yaml-1`, and target adapter `cli`. Project name, target command, campaign duration, and
storage root are nonempty. Timeout, memory, process, output, and worker limits are positive
integers representable by `u64`; the campaign seed is a nonnegative `u64`; allowed process exit
codes are signed 32-bit integers. Duration uses a positive canonical magnitude followed by one of
`ms`, `s`, `m`, `h`, or `d`. Corpus paths, target arguments, required capabilities, engine modes,
and native-backend identities retain exact Unicode content and sequence order; duplicate
capabilities, modes, and backends are invalid. Version 1 engine modes are `managed` and `native`,
and native backends are `afl++`, `libfuzzer`, and `honggfuzz`. An enabled fuzz engine has at least
one mode; native mode has at least one native backend; disabled or non-native fuzzing cannot retain
effective native backends. Address, thread, and memory sanitizers are mutually exclusive. The
three Verus policy booleans under `verification.verus` must be true for an accepted production
configuration. These checks are the initial cross-field execution invariants, not permission to
omit later adapter-, engine-, oracle-, or platform-specific validation.

Canonicalization algorithm version 1 emits one JSON-compatible YAML 1.2 flow document followed by
one LF. Mapping fields appear in compiled schema order; sequences preserve semantic order; booleans
are lowercase; Core integers use minimal decimal spelling derived from their arbitrary-width
semantic magnitude; and every string and mapping key is double-quoted with deterministic YAML
escapes. Quote, reverse-solidus, backspace, tab, line feed, form feed, and carriage return use their
JSON escape spellings; other non-ASCII Basic Multilingual Plane code points use lowercase
`\uXXXX`; supplementary code points use their shortest UTF-8 spelling inside the quoted string.
Comments, directives, anchors, aliases, merge keys, scalar style, numeric base, mapping presentation
order, and harmless whitespace do not enter these bytes. Parsing
and validating the canonical document yields the same effective configuration and exact canonical
bytes. The effective-configuration digest is lowercase `sha256:` plus the project-owned SHA-256 of
those bytes. Original source bytes remain a separate artifact with their own digest and diagnostic
provenance.

The absolute production bridge limits are 16 MiB of source bytes, 1,048,576 admitted typed values,
16 MiB of canonical bytes, depth 4,096, and 4,194,304 iterative rendering tasks. Callers may lower
source, typed-value, canonical-byte, depth, and work limits but cannot raise them. Work charges each
render-task dispatch, each candidate examined during duplicate detection, and each code-point
comparison needed to resolve an equal-length candidate exactly. Admission is checked before the
next value, output byte, or charged unit of work; a source or canonical-output error identifies the
exact first excluded byte, a typed-value error identifies the first excluded YAML node, and a depth
or work error identifies the first task or comparison that cannot be admitted. Checked arithmetic,
the work bound, and the independent source/value/output caps prevent alias sharing, large sequences,
duplicate detection, or canonical escaping from becoming an unbounded allocation or traversal.

`crucible config validate <path>` reads one regular non-symlink file under the source limit,
validates it without starting a campaign, and writes its `sha256:<lowercase-hex>` effective digest
plus LF. `crucible config canonicalize <path>` performs the same validation and writes only the
canonical YAML bytes. Both reject non-UTF-8 paths, symlinks, non-regular files, oversized input,
malformed YAML, schema/type/version errors, and cross-field errors through stable typed diagnostics;
neither mutates a workspace or emits a partial canonical document on failure. Source adapters must
resolve every component through descriptor-relative no-follow operations and validate the opened
descriptor, or fail closed with an unsupported-platform diagnostic until an equivalent platform
primitive is implemented; a check-then-open path walk is not an admissible substitute.

Crucible YAML profile 1 must specify rather than inherit ambiguous implementation behavior.
At minimum it defines:

- UTF-8 handling, optional byte-order mark behavior, line endings, and source spans,
- indentation, plain/quoted/block scalars, comments, sequences, and mappings,
- numeric grammar and checked conversion without host-width dependence,
- explicit boolean and null spellings without YAML 1.1 implicit coercions,
- anchors, aliases, tags, and merge behavior, including expansion and recursion limits,
- duplicate-key rejection after canonical key resolution,
- deterministic map ordering for hashing while preserving source order for diagnostics,
- maximum depth, token count, alias expansion, scalar size, and total decoded size,
- unknown-field policy, deprecation handling, and schema-version negotiation,
- diagnostic stability and recovery behavior,
- canonical serialization used for configuration and campaign digests.

Profile 1's byte-decoding transformation is version 1. It accepts only shortest-form UTF-8 that
decodes to Unicode scalar values, with distinct diagnostics for unexpected or invalid continuation
bytes, invalid leaders, truncation, overlong forms, surrogate code points, and values above
`U+10FFFF`. A leading UTF-8 byte-order mark is either explicitly forbidden or stripped under an
explicit `AllowAndStrip` policy; `U+FEFF` anywhere else remains data. CRLF and standalone CR are
normalized to LF. Every decoded scalar retains a half-open span over the original source bytes,
including both bytes of CRLF, with zero-based lines and scalar columns. A stripped BOM consumes
bytes 0 through 2 without incrementing the first scalar's line or column.

The version-1 absolute decoding limits are 16 MiB of source bytes and 1,048,576 decoded scalars.
Callers MUST provide per-operation limits and MAY lower either bound but cannot raise the profile
maximum. A larger accepted representation requires a new language-profile decision rather than a
platform-dependent allocation accident. These decoder limits do not replace the lexer, parser,
alias-expansion, depth, individual-scalar, or total semantic-value limits required below.

Diagnostic precedence for byte-decoding transformation version 1 is normative. The decoder first
applies the effective source-byte cap, then rejects a forbidden leading BOM, then applies the
decoded-scalar cap before attempting the next scalar. Within a scalar it classifies an intrinsically
invalid leader first. For a potentially valid multi-byte leader, it examines each available
continuation byte from left to right. An available non-continuation byte is reported at that byte.
After a valid second byte is present, the `E0`, `ED`, `F0`, and `F4` second-byte restrictions are
reported as overlong, surrogate, or out-of-range errors at the leader before inspecting any later
byte. Only when every available byte required so far is valid and another required byte is absent is
the sequence truncated.

Offsets are also normative: source-limit errors identify the first byte excluded by the effective
cap; scalar-limit errors identify the start of the next scalar; forbidden BOM, intrinsically
overlong, surrogate, and out-of-range errors identify the leader; continuation errors identify the
offending byte; and truncation identifies the end-of-input offset. The pure Verus transformation is
total over bytes, limits, and BOM policy. Its result fixes both every successful decoded source and
every error kind and offset, and executable correspondence is required for both result variants.

Profile 1's lexical-atom transformation is version 1 and is the verified context-free first stage
of lexing. It consumes only a successfully decoded source and produces exactly one atom for every
decoded scalar, preserving the code point and source span without alteration. It distinguishes LF,
space, tab, the YAML indicator characters `-?:,[]{}#&*!|>'"%@` plus the grave accent, and all other
content. Indicators retain distinct kinds, including the reserved at-sign and grave-accent kinds,
so a later stage can reject or interpret them using position and context rather than losing source
evidence. A stripped leading BOM has no atom; a non-leading `U+FEFF` is content.

The version-1 absolute lexical-atom limit is 1,048,576, equal to the decoded-scalar cap. Callers MAY
lower it but cannot raise it. Exceeding the effective limit returns a typed error at the first scalar
that would be excluded and constructs no partial public result. The pure Verus transformation is
total and fixes both the complete atom sequence and the exact limit error. Its exact-correspondence
predicate remains usable over arbitrary ghost views, while semantic atom-source validity also
requires a well-formed decoded source; downstream proofs can recover that obligation instead of
silently laundering forged decoded views. Lexical atomization does not replace context-sensitive
token formation: indentation, comments, directives, document markers, flow delimiters, and plain,
quoted, and block scalar boundaries remain mandatory verified lexer behavior.

Profile 1's line-layout transformation is version 1. It consumes lexical atoms and emits one exact
descriptor for every non-phantom source line: an empty atom stream, including a stripped-BOM-only
source, has no line; a final line feed terminates its line without creating an extra trailing empty
line; and a nonempty unterminated tail is its own line. Each descriptor records its zero-based line
number, half-open atom range excluding the optional line feed, first non-space atom, leading-space
width, termination state, and byte offsets for the line start, content-candidate start, and consumed
end. CR and CRLF already normalized by the decoder therefore remain one line-feed atom while their
original byte widths remain visible.

Only spaces contribute leading indentation columns. The layout stage MUST preserve a tab as the
first non-space atom rather than reject it: tab legality depends on context that this pre-pass does
not yet possess. In particular, a tab can be scalar content after required block-scalar indentation
or in a multiline flow scalar even though it cannot serve as structural indentation. The verified
context-sensitive token scanner therefore decides whether each preserved tab is scalar content,
valid separation, or an indentation violation and reports the contextual diagnostic. This deferral
is lossless and is not permission for the completed lexer to accept a tab where YAML forbids one.

The absolute profile caps are 1,048,576 line descriptors and 4,096 leading-space columns per line.
Callers MAY lower either cap but cannot raise it. A line-limit error identifies the first atom of the
first excluded line, and an indentation-limit error identifies the first excluded leading space. A
defensive atom-count check rejects a forged over-cap atom source before indexing it. Error precedence
is atom-count cap, then admission of a new line under the line cap, then left-to-right leading-space
validation within that admitted line. The transformation is iterative in executable code, total in
its pure Verus model, constructs no partial public result on error, and preserves a semantic validity
witness from the atomized input. It is a proof-carrying substage of the full lexer, not a substitute
for comments, directives, flow state, or scalar token formation.

Profile 1's structural-candidate transformation is version 1 and is a second proof-carrying lexer
substage. It authenticates its input by recomputing the canonical line layout, then partitions every
atom exactly once into nonempty, monotonic ranges for indentation, separation, line feeds,
directives, document markers, comments, flow punctuation, contextually plausible indicators, and
still-ambiguous content. Each range retains its line number, half-open atom indices, and original
half-open byte span. These roles are lossless candidates rather than completed YAML tokens: quoted,
plain, and block scalar context may reinterpret punctuation or separation candidates, and the full
context-sensitive scanner remains responsible for final scalar boundaries, flow state, reserved
indicators, and contextual tab legality. No atom or byte evidence may be discarded by that later
reinterpretation.

The absolute structural-candidate cap is 1,048,576, equal to the lexical-atom cap; callers MAY lower it
but cannot raise it. An exhausted cap reports the first byte of the first excluded lexeme and exposes
no partial public result. An empty or stripped-BOM-only atom stream requires zero lexeme capacity.
A layout that is not the canonical result for the supplied atom stream is rejected before scanning.
The executable scan is iterative, its pure Verus model is total under explicit fuel equal to the
input atom count, and exact correspondence covers success, layout mismatch, and limit failure. Its
semantic validity predicate retains both intrinsic atom validity and the authenticated layout
witness, so forged ghost views cannot be laundered into a valid structural source.

Profile 1's quoted-scalar boundary transformation is version 1 and is the first completed slice of
the verified context-sensitive token scanner. It authenticates both the canonical line layout and
canonical structural-candidate partition before interpreting any candidate. The structural stage
therefore retains every single- and double-quote atom as an explicit provisional candidate,
including quotes adjacent to JSON-style flow punctuation; this stage alone decides whether a quote
can begin a scalar. Its verified state tracks provisional plain-scalar continuation and block-scalar
header/body regions before updating flow depth, so quotes and provisional punctuation inside those
regions remain raw content. Quotes may begin at the stream or after separation, a line break, a flow
collection opener, a flow entry, or a mapping-value indicator inside a flow collection. This start
decision is provisional token context, not a waiver of the parser's separation and
collection-grammar obligations.

Each accepted quoted scalar includes both delimiters and records its style, starting and ending
line, half-open atom range, and exact half-open original-byte range. Single-quoted content escapes a
single quote only by doubling it. Double-quoted content recognizes YAML 1.2's complete simple escape
set, an escaped line break, and `x`, `u`, and `U` escapes with exactly two, four, and eight hexadecimal
digits respectively. Escaped values above `U+10FFFF` and surrogate code points are rejected rather
than converted. Unescaped content admits exactly YAML's printable character set: tab, line feed,
ASCII `0x20` through `0x7E`, `U+0085`, `U+00A0` through `U+D7FF`, `U+E000` through `U+FFFD`, and
`U+10000` through `U+10FFFF`. This transformation validates boundaries and escape spelling without
prematurely discarding raw presentation bytes; semantic decoding and flow-line folding remain
mandatory parts of scalar resolution.

The absolute quoted-scalar count and per-scalar atom caps are each 1,048,576. Callers MAY lower
either cap but cannot raise it. Scalar-count exhaustion reports the opening quote of the first
excluded scalar. Per-scalar exhaustion reports the first atom excluded from that scalar, counting
both delimiters. Invalid escape characters and hexadecimal digits report the offending atom;
invalid escaped code points report the initiating backslash; an unfinished escape or hexadecimal
sequence reports end of input; an unescaped forbidden character reports that character's original
byte offset; and an otherwise unterminated quote also reports end of input. All failures expose no
partial public result.

The executable quote scan and scalar-body machines are iterative. Their pure Verus models are total
under explicit structural-candidate and atom fuel, and exact correspondence covers success, input
authentication failure, both resource caps, all escape failures, and unterminated scalars. The
semantic validity predicate retains the authenticated atom, layout, and structural witnesses so
forged upstream ghost views cannot be laundered. Its public range predicate and theorem expose, for
every semantically valid result, nonempty bounded atom ranges, exact source byte and line endpoints,
opening and closing delimiters matching the recorded style, and ordered pairwise atom/byte
non-overlap. This completed slice does not replace or defer the committed plain-scalar,
block-scalar, comment, directive, flow-balance, contextual-tab, final token, parser, resolution,
lowering, validation, canonical-serialization, or self-fuzzing work.

Profile 1's plain-scalar boundary transformation is version 1 and is the second completed slice of
the same verified context-sensitive scanner. It authenticates the canonical atom, layout,
structural-candidate, and quoted-scalar results before classifying any unquoted content. Its
accepted first and continuation characters implement YAML 1.2.2 productions 126 through 135: a
plain scalar is nonempty, has no leading or trailing white space, admits `?`, `:` or `-` initially
only when followed by a context-safe nonspace character, never contains a colon-space or space-hash
delimiter, and treats `[`, `]`, `{`, `}`, and `,` as terminating syntax only inside flow
collections. A `:` followed by a context-safe character and a `#` immediately preceded by nonspace
content remain scalar content, including URL and `foo#bar` spellings. Quotes and otherwise
structural-looking indicators encountered after a plain scalar begins remain plain content when
the YAML productions admit them.

Plain scalar ranges retain every internal presentation atom used for line folding, including line
feeds, required indentation spaces, and safe tabs, but exclude leading separation and trailing
white space. Block-context continuation requires indentation beyond the parent indentation; flow
context continuation remains governed by flow-line-prefix rules. Empty folded lines may be crossed
only when a later nonempty continuation remains in the same scalar. The range's ending line and
byte offset therefore belong to its final nonspace content atom, not to discarded presentation
white space.

The parent indentation is the grammar parent carried through compact block productions, not merely
the physical leading-space count of the line on which the scalar begins. Thus an inline value in
`- key: value`, a scalar after a compact nested `-`, and a block-scalar header after either form all
terminate against the nested mapping or sequence indentation. A sibling entry at that indentation
cannot be folded into the preceding scalar merely because the outer line began at column zero.

Node-property exclusion retains distinct anchor, alias, shorthand-tag, and verbatim-tag substates.
In particular, the comma and square brackets admitted by `ns-uri-char` inside a `!<...>` verbatim
tag remain property spelling until the closing `>` and are never exposed as plain-scalar content.
Flow-adjacent mapping colons are excluded even when the structural-candidate stage has coalesced a
colon and neighboring content into one provisional range.

Tab legality is decided from this scanner context rather than from a global character ban. A tab
before required structural indentation is a typed indentation error. A tab after the required
space indentation in plain, quoted, or block-scalar presentation is separation or scalar content
as required by the applicable YAML production. Literal and folded block scalar header/body regions
are tracked while finding plain boundaries so their punctuation, quotes, comments, and tabs cannot
be relabeled as plain scalars; completed block-scalar token formation and chomping/folding remain a
separate mandatory output of the same lexer.

The absolute plain-scalar count and per-scalar presentation-atom caps are each 1,048,576. Callers
MAY lower either cap but cannot raise it. Scalar-count exhaustion identifies the first content atom
of the first excluded scalar. Atom exhaustion identifies the first atom excluded from the final
accepted half-open range and is checked only when later nonspace content makes intervening
presentation white space part of that range. Raw characters outside YAML's printable set are
rejected at their original byte offset. A reserved `@` or grave-accent indicator in node-start
position is rejected with its own typed diagnostic. A `?`, `:`, or `-` without the context-safe
lookahead required by production 126 reports `InvalidPlainStart` at that indicator. For a given
candidate, syntax and character validity are checked before caller scalar-count and per-scalar atom
caps, so that malformed start cannot be hidden by a zero cap. Earlier stream failures retain their
normal source-order precedence. Every failure exposes no partial public result.

The executable outer and scalar-body scans are iterative; their pure Verus models are total under
explicit candidate fuel, prove strict progress, and fix the complete success or error result. The
public semantic and range contracts bind every scalar to exact opening/final-content atom, byte,
and line endpoints and prove ordered pairwise atom/byte non-overlap. This slice does not relax the
remaining block-scalar, completed-token, parser, resolution, lowering, schema, canonicalization, or
self-fuzzing requirements.

Profile 1's block-scalar transformation is version 1 and completes YAML 1.2.2 productions 162
through 182 rather than merely identifying a provisional `|` or `>` region. It authenticates the
canonical atom, layout, structural-candidate, quoted-scalar, and plain-scalar results before
recognizing a block scalar in block node-start context. An indicator inside a quoted scalar, plain
scalar, node property, comment, directive, document marker, or flow collection is never promoted to
a block scalar. Each accepted scalar records literal or folded style, strip/clip/keep chomping,
optional explicit indentation `1` through `9`, detected effective content indentation, the complete
header and presentation atom/byte/line ranges, and the exact normalized content code points.

The header admits an indentation indicator and chomping indicator in either order, at most once
each, followed only by separation, an optional comment, and the mandatory non-content line break.
An indentation digit of `0`, a duplicate or third modifier, nonseparated `#`, trailing noncomment
text, or end of input before that line break is rejected at the first atom that makes the header
invalid. Header comments end at that single line and cannot absorb following comment lines.

The parent indentation is the verified block-collection grammar context, not merely the header
line's leading-space count. Compact sequence and mapping forms therefore retain their nested
context: for example, `- key: |2` has parent indentation two, while a direct `- |2` remains in the
containing sequence's context. Explicit content indentation equals that parent indentation plus the
indicated value. Without an explicit indicator, the first nonempty content line determines the
indentation; if every candidate content line is empty, the longest such line determines it. The
detected level must be strictly greater than the parent indentation. A leading all-space line may
not be more indented than the first nonempty line. A nonempty line indented more than the parent
but less than the required content indentation is an error rather than an implicit scalar
terminator. A line at or below the parent indentation terminates the scalar. Tabs never count
toward indentation: a tab
encountered before all required indentation spaces reports `TabInIndentation`, while a tab after
those spaces is preserved as content. Empty and more-indented lines retain the behavior required by
the block productions, including content that begins with `#`, YAML indicators, or tabs.

Literal style preserves each normalized content line break. Folded style maps a single break
between adjacent nonempty, non-more-indented text lines to one space; breaks adjacent to an empty or
more-indented line remain line feeds. The final content break and trailing empty lines are never
folded. Strip removes the final break and trailing empty lines, clip retains exactly the final break
when the scalar has nonempty content, and keep retains the final break and every trailing empty line.
For an all-empty scalar, strip and clip produce empty content while keep preserves its presented
line feeds. Less-indented trailing comment lines are outside the scalar presentation range and are
left for completed comment-token formation.

Every emitted normalized content code point records the source atom and original half-open byte
range that caused it. Literal characters and preserved line feeds keep their code point; a folded
space names the normalized source line-feed atom it replaces. Indentation prefixes, the header, and
chomped presentation do not acquire fabricated content provenance. Public predicates prove that
these mappings are ordered, remain within the scalar's presentation range, and name either the
same source code point or the permitted folded line-feed-to-space transformation.

The absolute block-scalar count, per-scalar presentation-atom count, per-scalar normalized-content
count, and aggregate normalized-content count are each 1,048,576. Callers MAY lower any bound but
cannot raise it. Scalar exhaustion reports the first excluded `|` or `>`. Presentation exhaustion
reports the first atom outside the allowed complete scalar presentation. Per-scalar content
exhaustion reports the source atom for the first excluded normalized content code point; aggregate
content exhaustion does the same across scalars. Intrinsic header, indentation, and character
errors take precedence over limits for the same scalar, so a malformed scalar cannot be hidden by a
zero cap. All errors are typed, retain exact original byte offsets, and expose no partial result.

The executable header, contextual block-grammar indentation, line-classification, folding,
chomping, provenance, and outer scan machines are iterative. Their pure Verus models are total
under explicit atom and line fuel, prove strict progress and bounded arithmetic, and fix the exact
success or error result. Every semantically authenticated executable success, including nonempty
success, proves the full range/content predicate and ordered atom/byte non-overlap. That predicate
equates each scalar's content to the exact verified renderer, while the per-content provenance
predicate excludes header and body-indentation atoms and permits folded line-feed-to-space origins
only for folded style. Public semantic, range, content, provenance, and ordered-non-overlap
contracts remain usable downstream; forged upstream or output ghost views cannot be laundered into
valid block-scalar evidence. This completed transformation remains an input to final token
formation and scalar/tag resolution and does not defer the parser, lowering, schema,
canonicalization, or self-fuzzing requirements.

Profile 1's completed-token transformation is version 1 and closes the context-sensitive lexer;
the earlier structural and scalar transformations are authenticated evidence inputs rather than
alternative token streams. The transformation recomputes and authenticates the canonical atom,
layout, structural-candidate, quoted-scalar, plain-scalar, and block-scalar results before emitting
anything. A failure in one input witness has a distinct typed diagnostic naming that transformation,
so an unrelated or forged view cannot be accepted or mislabeled as a later lexical error.

The successful token sequence is lossless over the post-BOM atom stream. Every token has a nonempty
half-open atom range, exact original half-open byte range, and starting and ending source line;
tokens are strictly ordered and adjacent in both atom and byte space, the first begins at atom zero
and byte `bom_bytes`, and the last ends at the atom count and original source length. Empty and
stripped-BOM-only input emits no tokens. This partition includes presentation detail instead of
discarding it: leading-space indentation, in-line separation, normalized line feeds, comments,
and a document-prefix `U+FEFF` each have explicit token kinds. A later concrete syntax tree can
therefore reproduce all non-BOM source evidence and attach comments and formatting without guessing.
The optional stripped leading BOM remains represented by source metadata rather than a fabricated
zero-width token.

The non-trivia token kinds are YAML and TAG directives, reserved directives, directives-end (`---`)
and document-end (`...`) markers, flow-sequence and flow-mapping delimiters, flow entry, block
sequence entry, explicit mapping key, mapping value, anchor property, tag property, alias, and the
five scalar styles: plain, single quoted, double quoted, literal block, and folded block. Scalar
tokens name the exact authenticated scalar record and cover its complete presentation range; their
punctuation, comments, internal separation, and line feeds are not emitted again as nested tokens.
Empty scalars consume no input and are consequently parser/CST nodes anchored between real tokens,
not zero-width lexer tokens that would violate the partition invariant.

Comments begin only where separation context admits `#` and extend through the final non-break atom;
their terminating line feed remains a separate token. Spaces and tabs outside a scalar are grouped
only across a single maximal separation or indentation run. Only spaces form structural indentation.
A tab before the required structural indentation, including a tab-only prefix before a flow
collection, property, alias, or quoted scalar, reports `TabInIndentation` at that tab; tabs admitted
by flow-line prefixes, separation, or authenticated scalar presentation remain lossless input.
Reserved `@` and grave-accent indicators that survive outside scalar content are rejected at their
own offsets rather than emitted as generic content.

Directive tokens implement YAML 1.2.2 productions 82 through 95. A directive is recognized only as
a non-indented line in document-prefix directive mode. Its token ends at the last nonspace parameter,
leaving trailing separation, comment, and line feed as trivia tokens. `%YAML` requires exactly one
decimal `major.minor` parameter and records both checked base-10 components without host-width
dependence. `%TAG` requires exactly one valid primary, secondary, or named handle and one nonempty
local or global tag prefix. A reserved directive retains its nonempty name and every nonempty
parameter range losslessly; the parser later emits the required unknown-directive warning instead
of erasing it. Duplicate YAML directives, duplicate TAG handles, unsupported YAML major versions,
and directive/document ordering depend on document grammar and are mandatory parser diagnostics,
not reasons to weaken lexical spelling checks.

Document markers implement productions 203 and 204: they begin at column zero, contain exactly
three marker atoms, and are followed only by white space, a line break, a comment, or end of input.
The lexer tracks whether the stream is in document-prefix directive mode. A directives-end marker
leaves that mode and a document-end marker re-enters it. A non-leading `U+FEFF` is a
`DocumentByteOrderMark` token only at the true non-indented start of a document prefix. It is also
permitted as content inside an authenticated single- or double-quoted scalar, as required by YAML
1.2.2; every occurrence in plain or block content, directives, properties, aliases, anchors,
comments, or after same-line indentation is rejected at its original byte offset. Marker-looking
text inside any authenticated scalar or comment is never promoted to stream structure.

Node properties and aliases implement productions 97 through 104. Anchor and alias names are
nonempty and exclude white space and flow indicators. A tag token records whether it is
non-specific, verbatim, primary shorthand, secondary shorthand, or named-handle shorthand, plus
exact handle and suffix ranges when present. Verbatim tags require a nonempty `ns-uri-char` payload
and closing `>`; shorthand suffixes are nonempty except for the standalone non-specific `!`, exclude
raw `!` and flow indicators, and validate each percent escape as exactly two hexadecimal digits.
The comma and brackets permitted in a verbatim URI remain inside that tag token. URI well-formedness,
TAG-handle declaration lookup, tag expansion, alias-to-prior-anchor resolution, duplicate node
properties, and the prohibition on alias content are mandatory parser/resolution checks over these
lossless tokens.

Indicator classification follows the context productions rather than character identity alone.
`-`, `?`, and `:` become block collection indicators only with the required separation or line/end
lookahead, while flow mapping value indicators retain YAML's JSON-compatible no-space form. The
lexer maintains a bounded typed stack for `[` and `{`; a closing delimiter must match its opener,
flow entries and flow-only indicator roles require nonzero flow depth, and end of input with a
nonempty stack reports an unclosed-flow diagnostic. Delimiters inside authenticated scalar,
property, directive, or comment ranges never affect the stack. Collection-entry grammar,
key/value placement, indentation nesting, and complete document structure remain parser work, but
the parser never receives an internally impossible or unbalanced final token stream.

The absolute completed-token count is 1,048,576 and the absolute flow-delimiter depth is 4,096.
Callers MAY lower either cap but cannot raise it. Intrinsic spelling, contextual-tab, reserved
indicator, and mismatched-closing-delimiter errors take precedence over admitting the malformed
token under a caller cap. A token-count error identifies the first atom of the first otherwise valid
excluded token. A flow-depth error identifies the first opener beyond the effective depth. An
unclosed flow reports the original end-of-input byte offset; malformed directive, property, alias,
tag, and percent-escape errors identify the first atom that makes the spelling invalid, or end of
input when required spelling is absent. Every error is all-or-nothing and exposes no partial public
token source.

The executable completed-token scanner, directive/property submachines, scalar-evidence merge, and
flow stack are iterative Verus Rust. Their pure models are total under explicit atom and token fuel,
prove strict progress and bounded stack arithmetic, and determine the exact successful sequence or
typed error. Public semantic contracts prove upstream authentication, the complete adjacent
atom/byte partition, exact token-kind spelling, scalar-record identity, trivia maximality, balanced
and properly nested flow delimiters, deterministic result, and the count/depth limits. Proof tests
must include nonempty mixed streams and forged token views that attempt gaps, overlap, incorrect byte
endpoints, scalar-range substitution, header/property leakage, or flow-stack laundering.

Profile 1's concrete-syntax transformation is version 1 and implements YAML 1.2.2 productions 105
through 211 over the authenticated completed-token stream. It supports the complete presentation
grammar rather than a configuration-shaped subset: zero or more independent documents; bare,
explicit, and directive documents; block and flow sequences and mappings; compact collection
forms; explicit and implicit keys; arbitrary node kinds as mapping keys; empty sequence entries,
keys, values, and documents where the productions admit them; aliases; node properties in either
order; and every scalar style. Block collections cannot appear in flow context, while flow nodes
remain valid in block context. The parser retains duplicate mapping entries in source order because
duplicate-key rejection is defined after canonical key resolution, not by raw presentation text.

The concrete syntax tree is a nonrecursive, versioned table representation. A stream record names
ordered document records and the complete token interval. Each document records its prefix,
directive, explicit-start, root-node, explicit-end, and suffix token intervals, plus every reserved
directive warning and YAML-version warning. Each node records its kind and presentation style,
complete token and byte range, optional anchor-property token, optional tag-property token, and an
exact reference to its authenticated scalar or alias token when applicable. Sequence entries and
mapping key/value pairs are separate ordered tables. Collection nodes name contiguous entry-table
ranges; child-node indices are strictly smaller than their completed parent index, so the CST is a
finite acyclic tree without host recursion. A document root names the final node for that document.
Zero-width empty nodes record an exact between-token anchor and source byte offset instead of
fabricating a lexer token or overlapping the lossless token partition.

Trivia remains lossless presentation evidence. Indentation, separation, comments, line feeds, and
document-prefix BOM tokens are not discarded or reinterpreted as semantic nodes. CST token
intervals include their surrounding presentation where the YAML production consumes it, while core
node and entry intervals identify the exact syntax-bearing subrange. Every token belongs to one
ordered stream-prefix, document, or stream-suffix interval, and every syntax-bearing token is owned
by exactly one directive, property, marker, node, or collection-entry record. Comments remain
presentation detail rather than acquiring semantic association with a node; tools may derive a
display attachment without changing the CST or graph meaning.

Directive state is reset for every document. A directive requires a following directives-end
marker. At most one YAML directive is permitted in a document, and a TAG handle may be declared at
most once in that document even when the repeated prefix is identical. `%YAML 1.1`, `%YAML 1.2`, and
the absence of a version directive are accepted and parsed using profile-1 rules; 1.1 records a
compatibility warning. A higher `1.x` minor version is attempted using profile-1 rules with a stable
future-minor warning. A major version other than one is rejected at the version token. Reserved
directives are retained and produce stable warnings rather than errors. Primary `!` and secondary
`!!` handles have their YAML defaults, named handles require a declaration, and explicit TAG
directives may override the two defaults for only that document. Expanded tag URI validity and
exact percent-escape preservation are resolution work, but the parser retains the exact declaration
and property tokens needed to perform them.

A non-alias node admits at most one anchor property and at most one tag property, in either order,
with the separation and indentation required by its context. A repeated property is a typed parse
error at the second property. Properties without explicit scalar or collection content form the
empty scalar node admitted by the applicable production. An alias is a complete node by itself: it
cannot carry a tag, anchor, scalar, or collection child. Whether an alias names the most recent
preceding anchor, whether tag handles resolve, and whether an expanded tag is compatible with the
node kind are mandatory composition checks over the completed CST rather than parser shortcuts.

Flow collection frames enforce comma and key/value placement in addition to the lexer's delimiter
balance. Flow sequences admit ordinary nodes and compact single-pair mappings; flow mappings admit
explicit and implicit entries, JSON-style no-space mapping values, empty keys or values where the
productions allow them, and one optional trailing comma. A leading comma, repeated comma, missing
comma, second mapping-value indicator, or collection closer while an entry is incomplete reports
the first token that makes the production impossible. Flow collection depth is inherited from the
authenticated lexer evidence and is checked again against the parser's caller-lowered stack bound.

Block collection frames are driven by exact token line, indentation width, and parent grammar
context rather than raw line-prefix heuristics. Entries at one indentation form one collection;
greater indentation begins only a child permitted by the pending sequence entry, mapping key, or
mapping value; and lesser indentation closes frames until the owning context is reached. Compact
forms such as `- key: value`, `? key : value`, a block sequence nested directly under a mapping
value, and a mapping nested directly under a sequence entry preserve the parent context already
used by block-scalar formation. Explicit keys may be arbitrary nodes and span the forms admitted by
the YAML grammar. Implicit keys obey the single-line restriction but do not inherit YAML 1.1's
removed 1,024-character limit. Indentation or an indicator that cannot belong to any open frame is
reported at the first offending token.

The absolute document, CST-node, sequence-entry, mapping-entry, directive, and warning caps are each
1,048,576, matching the token ceiling; the absolute parser frame depth is 4,096. Callers MAY lower
any cap but cannot raise it. Empty nodes and empty documents count toward the node and document
caps. Intrinsic document, directive, property, indentation, and collection-grammar errors take
precedence over the caller cap that would otherwise exclude the same completed record. A count
error identifies the first token or between-token anchor of the first excluded record; a depth
error identifies the opener or block entry that would create the first excluded frame. End-of-input
diagnostics use the original source length. Parsing is all-or-error and exposes no partial public
CST. A future diagnostic-recovery transformation may retain multiple errors for editors, but its
output cannot be accepted as executable configuration evidence.

The executable parser uses explicit bounded vectors of document, node, entry, directive, warning,
and frame records; production parsing and frame completion are iterative Verus Rust even where a
recursive implementation would be shorter. The pure model is total under explicit token and frame
fuel and fixes the complete CST or the first typed diagnostic. Public contracts prove canonical
token authentication, strict token progress or frame reduction on every step, bounded indexing and
arithmetic, exact node/property/scalar identity, document and entry ordering, child-before-parent
acyclicity, token ownership without syntax leakage, deterministic results, and every absolute and
caller-lowered limit. Proof tests include nonempty multidocument block/flow mixtures and forged CSTs
that attempt cycles, forward child references, duplicated token ownership, property substitution,
cross-document directive leakage, invalid empty-node anchors, or incomplete frame laundering.

Profile 1's semantic-resolution transformation is version 1. It authenticates the completed-token
source and CST before composing one semantic graph per document. The graph is a nonrecursive table
of scalar, sequence, and mapping nodes plus explicit edges; aliases resolve to existing node
identities rather than copying host objects. CST indices, source tokens, and original byte offsets
remain attached to every semantic node and mapping entry so later lowering and diagnostics never
need to reconstruct presentation provenance. Anchor names and alias spellings remain diagnostic
evidence even though anchor names are not part of semantic node equality.

The default implicit scalar policy is YAML 1.2.2 Core Schema, not YAML 1.1 compatibility
coercion. An untagged plain scalar resolves only as Core null, boolean, integer, finite decimal,
infinity, not-a-number, or string; `yes`, `no`, `on`, `off`, sexagesimal numbers, and legacy
single-letter booleans such as `y` and `n` remain strings. A leading-zero decimal such as `0123`
is a Core decimal integer and is never interpreted as YAML 1.1 octal; octal requires the Core `0o`
prefix. Non-plain scalars without a specific tag resolve as strings.
Empty scalar nodes resolve as null only when they have the `?` non-specific tag; an explicitly
quoted empty scalar is a string. Explicit standard tags `!!null`, `!!bool`, `!!int`, `!!float`,
`!!str`, `!!seq`, and `!!map` require a compatible node kind and a spelling valid for that tag.
Unknown well-formed global and local tags are retained losslessly on compatible semantic nodes for
application-specific validation; they are not silently rewritten to Core tags.

Integer resolution is independent of host width. A resolved integer is a sign plus a normalized
arbitrary-length magnitude stored as little-endian base-1,000,000,000 limbs, and Core decimal,
`0o` octal, and `0x` hexadecimal spellings convert by a bespoke verified multiply-add machine.
Leading magnitude zeroes are removed, every zero has one positive canonical form, and a lowered
limb cap fails at the exact first source digit whose canonical per-digit result needs the excluded
limb. Finite floats are represented exactly as a sign, a canonical arbitrary-length coefficient
stored as little-endian decimal digits, and a canonical arbitrary-length signed exponent stored as
little-endian decimal digits. Leading coefficient zeroes and value-preserving trailing coefficient
zeroes are removed in linear time, with the removed scale applied by a bespoke verified signed
decimal add/subtract machine. Exponent zero has one positive canonical form; coefficient zero
retains the source sign so negative zero remains distinct. Equivalent spellings such as `1.0`,
`1e0`, and `10e-1` normalize to one value without an intermediate IEEE-754 rounding step. A
caller-lowered coefficient- or exponent-digit limit reports the exact first source digit requiring
the excluded canonical digit; when normalization creates the excluded exponent digit, the error
uses the exact coefficient, fraction, or exponent source anchor responsible for that scale.
Positive and negative infinity are distinct values, every accepted NaN spelling has one canonical
semantic value, and negative zero remains distinguishable until schema lowering explicitly chooses
otherwise. Later lowering performs checked conversion to any required fixed-width integer or
floating format and reports range or precision policy errors instead of inheriting host casts.

Single-quoted, double-quoted, plain, literal, and folded scalar presentation is decoded by verified
style-specific machines. Single-quote doubling, every accepted double-quote escape, escaped line
breaks, and YAML flow-line folding produce exact Unicode scalar content with per-output provenance.
Plain-scalar line folding uses the authenticated presentation range and indentation context rather
than trimming arbitrary host strings. Block scalars reuse the already authenticated normalized
content and provenance. Every shared decoded-content record retains its Unicode code point, exact
half-open source-atom range, original byte range, and whether it was direct, folded, quote-doubled,
escape-produced, or derived from an escaped line break. A caller-lowered output cap reports the
exact first excluded provenance record before copying it. Scalar decoding does not apply Unicode
normalization; code-point identity is preserved unless an explicit future language-profile version
says otherwise.

The provisional quoted-scalar context also carries tag- and anchor-property payload state. Property
punctuation and payload candidates therefore cannot become plain content before a following quoted
node, while the required separation clears that state and permits the quote delimiter to start the
node. The same state remains subordinate to authenticated plain- and block-scalar regions, so a
tag- or anchor-like spelling inside scalar content never escapes into presentation structure.

The CST-scalar dispatch submachine authenticates the atom, completed-token, and CST identities,
then binds a scalar result to the exact CST node and completed scalar token before invoking the
verified style-specific decoder. Zero-width empty nodes produce an explicit empty-style scalar
record with neither a fabricated token nor fabricated decoded provenance. Alias and collection
nodes remain outside this scalar producer and return no scalar record; the graph composer handles
them through the independently authenticated alias bindings and collection-entry tables. The
dispatch result is therefore a lossless graph-composer input, not a second parser or a reduced
semantic graph.

Scalar-value composition combines that authenticated presentation with the independently resolved
tag property. Untagged plain and empty nodes follow the YAML 1.2.2 Core rules; untagged quoted and
block nodes and the explicit non-specific `!` tag resolve as strings. Explicit standard scalar
tags require both scalar-kind compatibility and a spelling accepted for that exact tag, while
`!!seq` and `!!map` are rejected on scalar nodes. Well-formed unknown local and global tags retain
their exact resolved tag provenance alongside the decoded scalar content. Integer and float values
are produced only by the existing verified arbitrary-precision converters, and every nested
decoded-content, tag, magnitude, coefficient, or exponent limit is preserved as an exact typed
source diagnostic. The public result retains the CST node index, explicit tag record, full decoded
presentation provenance, canonical semantic tag, and canonical semantic value.

Collection-tag composition authenticates the completed-token source and CST before inspecting node
kind. Untagged and non-specific sequence or mapping nodes receive the corresponding Core tag;
`!!seq` and `!!map` require the exact compatible collection kind; scalar standard tags are rejected
on collections; and unknown local or global tags remain lossless application-defined identities.
The result retains the CST node index, collection kind, canonical semantic tag, and complete
explicit tag provenance. Non-collection nodes return no collection record only after source and
CST identity and node-index bounds are authenticated, leaving them to the scalar or alias producer.

Semantic graph construction begins with a separate verified topology projection rather than a
recursive host graph. It emits one root record for every CST document, one node record for every
CST node, and source-ordered sequence and mapping edge tables that retain the original CST entry
indices, child indices, token intervals, and node byte ranges exactly. Collection node edge
intervals remain indices into the corresponding edge table, so child-before-parent identities and
sharing are preserved without renumbering. Document-root, node, sequence-edge, and mapping-edge
caps are independently caller-lowerable; each failure names the first excluded source record at
its exact byte anchor. Its public semantic predicate includes the completed CST semantic contract,
preventing a forged CST view from laundering an otherwise plausible topology table. Scalar and
collection values, alias redirection, cycle rejection, duplicate-key checks, and merge expansion
compose over these stable identities in the subsequent resolution machines.

Scalar semantic-node population is an independently verified aggregate transformation over those
stable CST identities. It emits exactly one complete resolved-scalar record for every scalar or
zero-width empty CST node, in increasing CST-node order, while collection and alias nodes remain
for their dedicated producers. Besides preserving the per-scalar tag, canonical value, decoded
presentation, and provenance, it counts decoded content across the entire source. A caller-lowered
scalar-record cap fails at the first excluded scalar node; a caller-lowered aggregate-content cap
fails at the exact first excluded decoded content code point, not merely at the containing node.
Per-scalar content, tag, integer-limb, and finite-float digit caps retain their existing nested
typed diagnostics and precedence. The public success theorem extracts both exact CST coverage and
exact aggregate accounting, while the executable loop remains iterative and bounded.

Tag resolution is document-scoped. The primary `!` and secondary `!!` handles begin with their YAML
defaults and may be overridden by that document's `%TAG` directives; named handles must have an
exact declaration in the same document. Verbatim tags bypass handle expansion. In accordance with
YAML 1.2.2 production 39, percent escapes in tag prefixes, suffixes, and verbatim payloads are
retained and compared exactly as presented rather than decoded; malformed escape triples retain
their lexer diagnostic, and escaped and unescaped spellings never collapse into one identity. URI
spelling is validated before a global tag identity is admitted. A local tag remains a distinct
local identity rather than being confused with a global URI. Directive state, local tag identity,
and anchor lookup reset at every document boundary.

An alias names the most recent preceding anchor property with the same exact Unicode name in its
document. Duplicate anchor names are therefore permitted and replace only subsequent lookup;
forward and cross-document aliases are typed errors at the alias token. An anchor on a collection
is visible to its descendants as soon as its property occurs, so a later descendant alias can form
a graph cycle even though CST parents complete after their children. Profile 1 rejects every direct
or indirect alias cycle at the alias edge that first closes the active resolution path. Shared
acyclic subgraphs remain shared semantic identities. Resolution and all later traversals use
explicit stacks and color/state tables rather than host recursion.

The anchor/alias binding submachine scans each authenticated document's root token interval in
presentation order and emits separate immutable declaration and alias-binding tables. Every record
retains its document, CST node, completed token, exact name atom range, and byte range; an alias
also retains the exact declaration and target node selected by most-recent-preceding shadowing.
This representation deliberately permits a collection target's CST node index to follow a
descendant alias node index, because visibility is fixed by property-token position rather than
child-before-parent completion order. Missing-name detection precedes a caller-lowered alias-record
limit at the same alias token, while an earlier source event still retains ordinary first-error
precedence. Cycle detection remains a distinct graph-composition check over these exact bindings.

Semantic node-table composition is a separate verified aggregate transformation. It re-runs and
owns the exact topology, scalar-table, and anchor/alias results produced from the authenticated raw
inputs instead of accepting detached same-length aggregates from a caller. It emits exactly one
slot for every CST node in increasing CST-node order: scalar and zero-width empty slots reference
their exact resolved scalar record, sequence and mapping slots reference their exact resolved
collection record, and alias slots reference the selected target node identity plus a complete
immutable alias-redirect record. Every slot retains the CST token and byte ranges, property-token
indices, and topology edge interval, while every redirect retains the document, alias token,
selected anchor declaration and node, and exact Unicode-name atom and byte ranges. Alias targets
remain shared node identities and are never materialized as copied host values. Node, collection,
and alias-redirect caps are independently caller-lowerable, nested collection-tag limits retain
their own diagnostics, and each failure names the exact first excluded source anchor. The public
semantic predicate exposes equality with the total pure composition result as well as the CST
semantic contract, so forged or stale topology, scalar, anchor, collection, or redirect evidence
cannot be laundered into a successful table.

Alias-cycle rejection consumes and owns that exact node table without materializing alias targets.
It examines alias redirects in presentation order before caller traversal caps, so the first direct
or indirect ancestor edge reports `AliasCycle` at the exact alias-name byte even when a lowered cap
would also reject the graph. Every accepted sequence, mapping, and alias edge is publicly proved to
target a strictly smaller stable CST node identity; this natural-number descent is the graph's
nonrecursive acyclicity witness. A second iterative pass records one exact semantic depth and
completed visit state for every node in deterministic CST order, the maximum observed depth, and
the complete node-index path attaining the first such maximum. The path is retained rather than
discarded so lowering and diagnostics can reuse checked traversal evidence. Semantic-depth and
explicit work-stack limits are independently caller-lowerable and report the byte start of the
first excluded node. The public success predicate binds all of this evidence to the total pure
cycle-resolution result and rejects forged forward redirects.

Canonical scalar-key composition consumes and owns that exact acyclic semantic graph and emits one
key record for every resolved scalar node. Each identity begins with a transformation/version
prefix, a resolved-tag discriminator, and an exact semantic-value discriminator. Core null,
boolean, arbitrary-width integer, exact finite-float, infinity, NaN, and string values use their
already normalized semantic records, so presentation variants such as `1`/`01`, `TRUE`/`true`,
`null`/`~`, `1.0`/`10e-1`, and plain/quoted strings compare by semantic identity rather than source
spelling. Local and global custom tags additionally encode their complete resolved tag content;
two different custom tag identities therefore cannot collapse merely because both carry the same
scalar value. Every variable-length component has a fixed-width length delimiter and every variant
has a distinct marker. Equality is exact byte equality, never host hash equality.

Every emitted byte retains a source-byte diagnostic anchor. Tag-content and decoded string bytes
use their retained per-code-point provenance; normalized numeric metadata uses the owning scalar's
exact node anchor. The executable encoder streams directly over retained tag, decoded-content,
integer-limb, and float-digit slices without cloning them into temporary aggregate buffers. Scalar
record count, per-key bytes, and total key bytes are independently caller-lowerable beneath their
absolute caps, and each failure reports the first excluded emitted byte before allocation. The
public semantic predicate is equality with the total pure composition result and therefore
authenticates the entire owned acyclic graph, exact record order, bytes, provenance, accounting,
limit precedence, and diagnostics. This scalar machine is retained as the authenticated scalar
input of the structural-key composition machine; it does not reduce the committed duplicate-key or
merge-expansion requirements.

Canonical structural-key composition consumes and owns that exact scalar-key source and emits one
identity record for every semantic node in stable CST-node order. Scalar nodes retain the complete
presentation-independent scalar identity. An alias record is byte-for-byte identical to its target
node record, including retained diagnostic provenance, so anchor names and alias presentation never
enter equality. A sequence record contains a structural/version prefix, complete resolved
collection-tag identity, entry count, and the length-delimited child identities in semantic order.
Sequence order therefore remains significant at every nesting depth.

A mapping record contains the corresponding prefix, complete resolved mapping-tag identity, entry
count, and every length-delimited key/value identity pair ordered lexicographically first by the
canonical key bytes and then by the canonical value bytes. The executable uses a verified
nonrecursive bottom-up merge sort over edge indices; it neither hashes identities nor clones full
key/value buffers into sort records. Mapping presentation order is therefore irrelevant, including
when two retained pre-diagnostic entries have equal keys and different values. The complete entry
multiset remains present at this layer: duplicate-key rejection is the next machine and is not
silently performed or hidden by canonicalization.

Core sequence/mapping tags use distinct discriminators. Custom local and global collection tags
add both a locality discriminator and their complete resolved Unicode tag identity, so distinct
application tags cannot collapse. Structural metadata bytes use the owning node's exact byte
anchor; each encoded custom-tag code point uses that code point's retained source-byte anchor; and
reused scalar, child, and alias bytes retain their original per-byte provenance. Every variable-size
component is length-delimited and every node kind is disjoint, making equality exact byte equality
rather than hash equality.

Structural record count, per-key bytes, aggregate stored key bytes, and mapping-sort entries are
independently caller-lowerable beneath their absolute caps. The mapping-sort cap is checked before
allocating its edge-index work vector, and all byte caps are checked before emitting the excluded
byte. Each failure names the owning node's exact source anchor. The public semantic predicate is
equality with the total pure composition result, authenticating the owned scalar/acyclic graph,
one-record-per-node ordering, recursive bytes, provenance, accounting, limit precedence, and typed
diagnostics. Proof and runtime fixtures cover alias transparency, nested collection keys, all
mapping permutations, equal-key/different-value pair permutations, custom collection tags,
sequence-order distinction, and exact accepted/rejected cap boundaries. This machine completes
structural key equality without reducing the subsequent duplicate-key or merge-expansion scope.

Duplicate-key rejection consumes and owns the exact canonical structural-key source. It checks
each mapping independently in retained mapping-entry source order and compares key identity bytes,
not host hashes or presentation spellings. Scalars therefore compare by resolved tag and canonical
semantic value; aliases are transparent; sequence order remains significant; mapping order does
not; empty keys participate as canonical null nodes; and complete custom scalar and collection
tags remain part of equality. Equal keys in separate mappings are never conflated.

Within one mapping, every later key is compared with all earlier keys and the first later equal key
is a duplicate candidate at that key node's exact byte start. A discovery pass selects the minimum
candidate byte across every mapping, so child-before-parent internal node numbering cannot displace
an earlier source diagnostic. The implementation is iterative and allocation-free. Its work is
bounded by the already enforced structural-key byte and mapping-entry caps. Intrinsic duplicate
discovery completes before caller-lowered duplicate-check accounting is applied, so a limit cannot
disguise a duplicate on the mapping or key it excludes. Mapping count and aggregate checked
mapping-entry count are independently caller-lowerable beneath their profile caps; accepted exact
boundaries are inclusive.

Success emits one duplicate-free source that retains the entire authenticated structural-key input
and exact checked counts. The total pure result fixes error precedence, offsets, accounting, and
the owned output. In addition to executable correspondence, the public semantic predicate contains
an independently extractable theorem that every earlier/later key pair in every retained mapping
has unequal canonical bytes. A forged view cannot satisfy that predicate merely by changing byte
provenance while retaining equal canonical values. Runtime fixtures cover normalized scalar
spellings, styles, explicit standard tags, empty keys, aliases, recursively equal sequence and
mapping keys, distinct custom tags, independent mapping scope, multiple-duplicate ordering, and
exact caller-limit boundaries.

Profile 1 supports the YAML merge-key draft deliberately as a named compatibility extension because
merge behavior is part of Crucible's configuration-language contract even though YAML 1.2.2 Core
does not define it. Only an untagged plain mapping key spelled exactly `<<`, or the same scalar with
the explicit `tag:yaml.org,2002:merge` tag, is a merge key; quoted `"<<"` remains an ordinary
string key. A merge value must resolve to one mapping or to a sequence containing only mappings.
For a sequence, earlier mappings override later mappings. Explicit entries in the receiving mapping
override every inherited entry regardless of source order. The resolved diagnostic order is all
explicit non-merge entries in source order followed by still-unshadowed inherited entries in merge
precedence and source order. Merge keys themselves do not appear in the semantic mapping.

The merge result remains a shared semantic graph rather than a materialized alias tree. It records
every mapping, including mappings with no merge key, and each effective entry retains its key and
value node identities, original mapping and mapping-edge identities, and whether it was inherited.
Inherited suppression compares the complete canonical structural bytes, never a host hash. Mapping
records, effective entries, fully expanded tree references, and individual merge sources have
independent caller-lowered caps. Invalid merge shapes are diagnosed before those caller limits can
hide them. The executable result is equal to a total pure model, owns the exact duplicate-free
input, and has public identity and uniqueness theorems so a substituted input or output cannot be
authenticated as the expansion result.

Canonical graph lowering consumes and owns that exact merge-expanded source. It emits one stable
record for every source node but has no alias node kind: each alias record names its final resolved
node and reuses that target's scalar or collection identity and canonical edge interval. Every
sequence child, effective mapping key and value, and document root is likewise rewritten to its
final non-alias node. Non-alias sequences retain exact source-edge identity. Effective mappings
retain receiver identity plus original mapping, original edge, and inherited/explicit provenance.
Scalar indices preserve the complete resolved value, tag, decoded presentation, and code-point
provenance; collection indices preserve the complete resolved collection tag. The owned input
retains all presentation evidence and source spans without copying it into the canonical DAG.

The lowering transform is iterative and graph-preserving. It does not materialize alias subtrees,
duplicate collection edges for alias records, discard unaffected mappings, or erase merge
provenance. Node records, canonical sequence entries, canonical mapping entries, and document roots
have independent caller-lowered caps of at most 1,048,576, with exact first-excluded source-byte
diagnostics and inclusive accepted boundaries. Its total pure model fixes every output record and
error; executable correspondence covers success and failure. Public contracts authenticate exact
input ownership, version/accounting identity, and deterministic output uniqueness. This canonical
YAML DAG is the input to schema-directed typed-field lowering; completing this graph transform does
not narrow or replace the separate typed Crucible configuration, schema validation, canonical
serialization, or digest-stability requirements.

The typed-field schema consumed by that lowering stage is itself an authenticated, independently
versioned graph rather than an untyped host-language map. Schema nodes distinguish null, boolean,
arbitrary-width integer, exact finite float, both infinities, NaN, string, custom scalar, sequence,
custom sequence, mapping, and custom mapping values. Sequence nodes name their exact item schema;
mapping nodes own contiguous field intervals. Each field retains a globally stable nonzero field
identity, a nonempty Unicode-scalar name, an exact value-schema reference, and required/optional
metadata. Compilation rejects malformed node shapes and references, gaps or overlaps in field
ownership, invalid Unicode scalar values, duplicate stable identities globally, and duplicate names
within a mapping. Empty mappings and recursive schema references remain representable; runtime
lowering termination is governed by the finite acyclic YAML graph and its separate work limits.

Schema compilation is iterative Verus Rust and owns its exact input on success. Schema nodes,
fields, and aggregate field-name code points have independent caller-lowered caps of at most
1,048,576 with inclusive accepted boundaries and exact first-excluded schema indices. A total pure
result model fixes the successful compiled schema or first typed schema diagnostic, and public
contracts prove exact input identity and deterministic output uniqueness. This prerequisite does
not by itself claim that YAML fields have been lowered, required or unknown-field policy has been
validated, cross-field invariants have been checked, or a canonical serialization has been emitted.

Typed-field lowering begins with an exact value-binding submachine. Given one canonical YAML node
and one compiled schema node, it binds only matching pairs: Core null, boolean, arbitrary-width
integer, exact finite float, positive infinity, negative infinity, NaN, and string values retain
their distinct resolved tags and value variants; custom scalar, sequence, and mapping schemas
require a custom local or global tag; and Core collection schemas reject custom-tagged collections.
No numeric conversion, string coercion, or tag erasure occurs. Alias records bind through their
canonical resolved-node and scalar/collection identity while diagnostics remain anchored to the
alias or value occurrence being lowered. The submachine rejects inconsistent record indices,
ranges, edge intervals, and version/accounting metadata before returning a typed binding. Its total
pure result fixes both successes and source-anchored typed errors, and its public exact-result
contract prevents a different binding from authenticating for the same graph, schema, and indices.
Graph-wide field traversal, recognized/unknown field partitioning, and required-field validation
remain subsequent machines rather than being silently folded into this local compatibility check.

Duplicate-key checking occurs after tag and scalar resolution and before merge application.
Two scalar keys are equal only when their resolved tag and canonical semantic value are equal;
sequence and mapping keys use structural equality over the acyclic resolved graph, with mapping
entry order ignored and anchors, aliases, styles, and comments ignored. Canonical key comparison is
implemented with a bounded, length-delimited representation rather than host hashing alone, so a
hash collision cannot make distinct keys equal. Two explicit entries in one mapping with equal
canonical keys are rejected at the later key. Merge precedence suppresses an inherited key only
after that same exact equality check and never hides a duplicate among the receiving mapping's
explicit entries.

The absolute semantic-node, sequence-edge, mapping-entry, anchor, alias, tag-byte, per-scalar
decoded-code-point, aggregate decoded-code-point, integer-magnitude-limb, finite-float
coefficient-digit, finite-float exponent-digit, expanded-reference, canonical-key-byte, and
work-stack caps are each 1,048,576; the absolute semantic depth is 4,096. Callers MAY lower every
cap but cannot raise it. Expanded-reference cost counts each node occurrence that a fully
materialized tree would visit, including repeated visits through aliases and merges, even though
the public graph retains sharing; checked addition rejects exponential alias or merge amplification
before allocation. Intrinsic invalid spelling, tag compatibility, missing alias, cycle, invalid
merge shape, and duplicate explicit key errors take precedence over the caller cap that would
otherwise exclude the same record. Limit errors identify the first source token whose admission or
expansion would exceed the effective bound, and all arithmetic is checked against both the profile
cap and `u64` representation.

The executable scalar decoders, handle/anchor tables, graph composer, cycle detector, merge
expander, and canonical-key machine are iterative Verus Rust. The pure transformation is total
under explicit token, node, edge, scalar, and traversal fuel and fixes the complete semantic graph
or first typed diagnostic. Public contracts prove authenticated input identity, exact scalar and
tag decoding, most-recent-preceding anchor selection, absence of unresolved aliases and cycles,
merge precedence, duplicate-key rejection, graph reachability from each document root, deterministic
output, source-order diagnostic retention, canonical key equality, strict progress, and every
absolute and caller-lowered limit. Proof tests include nonempty multidocument graphs and forged
views attempting cross-document handles or aliases, forward aliases, ancestor cycles, anchor
shadowing errors, tag-kind substitution, scalar-provenance laundering, merge amplification,
collision-based key equality, and duplicate-key laundering.

The YAML pipeline must never construct an unbounded alias or merge expansion, recurse without a
verified bound, silently accept duplicate effective keys, or permit a scalar coercion to change
across versions without a language-profile change. Resolution rejection is a typed configuration
result, not a harness panic, and exposes no partial public semantic graph.

### 12.2 Required YAML proofs and tests

The initial proof set must include:

- lexer and parser progress,
- termination under declared resource bounds,
- span validity and monotonic source consumption,
- absence of arithmetic overflow and out-of-bounds indexing,
- deterministic parsing and canonical serialization,
- `parse(canonical_serialize(value)) == value` for representable semantic values,
- canonical digest stability for semantically identical accepted configuration,
- duplicate-key rejection and alias-cycle rejection,
- lowering preserves every recognized field and rejects ill-typed values,
- successful validation implies all configuration invariants required by execution setup.

Fuzzing, property testing, mutation testing, differential testing against independently written
test oracles, and adversarial resource-exhaustion fixtures are required in addition to proofs.
The parser is itself a primary Crucible target and must continuously test its own untrusted
input boundary.

### 12.3 Configuration evolution

Language profile, syntax-tree schema, semantic-value schema, Crucible configuration schema,
and canonicalization algorithm have independent versions. Migrations produce new immutable
artifacts linked to their source configuration; they do not rewrite historical campaign
configuration. Unknown fields are rejected by default, with an explicit compatibility mode
available only when the unknown field can be preserved losslessly and cannot alter execution.

---
