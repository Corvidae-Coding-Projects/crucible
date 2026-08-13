#![forbid(unsafe_code)]
//! Project-owned, Verus-authored Crucible YAML profile implementation.

pub mod atom;
pub mod block;
pub mod cst;
pub mod layout;
pub mod plain;
pub mod quoted;
pub mod resolve;
pub mod resolve_alias_cycle;
pub mod resolve_anchor;
pub mod resolve_canonical_scalar_key;
pub mod resolve_canonical_structural_key;
pub mod resolve_collection_tag;
pub mod resolve_float;
pub mod resolve_integer;
pub mod resolve_node_table;
pub mod resolve_scalar_node;
pub mod resolve_scalar_table;
pub mod resolve_scalar_value;
pub mod resolve_special_float;
pub mod resolve_tag;
pub mod resolve_topology;
pub mod scalar_decode;
pub mod structural;
pub mod token;
pub mod utf8;

pub use atom::{
    atomize_profile1, classify_lexical_atom, AtomizeError, AtomizeErrorKind, AtomizeLimits,
    AtomizedSource, LexicalAtom, LexicalAtomKind, YamlIndicator,
    LEXICAL_ATOM_TRANSFORMATION_VERSION, MAX_PROFILE1_LEXICAL_ATOMS,
};

pub use block::{
    canonical_block_scalar_limits, scan_profile1_block_scalars, BlockChomping, BlockScalar,
    BlockScalarContentOrigin, BlockScalarContentScalar, BlockScalarError, BlockScalarErrorKind,
    BlockScalarScanLimits, BlockScalarSource, BlockScalarStyle,
    BLOCK_SCALAR_TRANSFORMATION_VERSION, MAX_PROFILE1_BLOCK_SCALARS,
    MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
    MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
};

pub use cst::{
    canonical_cst_limits, parse_profile1_cst, CstDocument, CstDocumentView, CstError, CstErrorKind,
    CstErrorView, CstLimits, CstLimitsView, CstMappingEntry, CstMappingEntryView, CstNode,
    CstNodeKind, CstNodeStyle, CstNodeView, CstSequenceEntry, CstSequenceEntryView, CstSource,
    CstSourceView, CstSyntaxOwner, CstSyntaxOwnerKind, CstSyntaxOwnerView, CstWarning,
    CstWarningKind, CstWarningView, CST_TRANSFORMATION_VERSION, MAX_PROFILE1_CST_DEPTH,
    MAX_PROFILE1_CST_DIRECTIVES, MAX_PROFILE1_CST_DOCUMENTS, MAX_PROFILE1_CST_MAPPING_ENTRIES,
    MAX_PROFILE1_CST_NODES, MAX_PROFILE1_CST_SEQUENCE_ENTRIES, MAX_PROFILE1_CST_WARNINGS,
};

pub use layout::{
    analyze_profile1_layout, LayoutError, LayoutErrorKind, LayoutLimits, LayoutLine, LayoutSource,
    LINE_LAYOUT_TRANSFORMATION_VERSION, MAX_PROFILE1_INDENTATION_COLUMNS,
    MAX_PROFILE1_LAYOUT_LINES,
};

pub use quoted::{
    canonical_quoted_scalar_limits, scan_profile1_quoted_scalars, QuotedScalar, QuotedScalarError,
    QuotedScalarErrorKind, QuotedScalarScanLimits, QuotedScalarSource, QuotedScalarStyle,
    MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_QUOTED_SCALAR_ATOMS,
    QUOTED_SCALAR_TRANSFORMATION_VERSION,
};

pub use resolve::{
    classify_core_plain_scalar, CoreIntegerBase, CorePlainScalarClass, CoreScalarError,
    CoreScalarErrorKind, CoreScalarLimits, CoreScalarRange, CORE_SCALAR_CLASSIFIER_VERSION,
    MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
};

pub use resolve_alias_cycle::{
    canonical_alias_cycle_limits, resolve_profile1_alias_cycles, AcyclicSemanticGraphSource,
    AliasCycleError, AliasCycleErrorKind, AliasCycleLimits, SemanticVisitState,
    ALIAS_CYCLE_RESOLUTION_VERSION, MAX_PROFILE1_SEMANTIC_DEPTH, MAX_PROFILE1_SEMANTIC_WORK_STACK,
};

pub use resolve_canonical_scalar_key::{
    canonical_scalar_key_limits, compose_profile1_canonical_scalar_keys, CanonicalKeyByte,
    CanonicalScalarKeyError, CanonicalScalarKeyErrorKind, CanonicalScalarKeyLimits,
    CanonicalScalarKeyRecord, CanonicalScalarKeySource,
    CANONICAL_SCALAR_KEY_TRANSFORMATION_VERSION, MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
    MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS, MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
};

pub use resolve_canonical_structural_key::{
    canonical_structural_key_limits, compose_profile1_canonical_structural_keys,
    CanonicalStructuralKeyError, CanonicalStructuralKeyErrorKind, CanonicalStructuralKeyLimits,
    CanonicalStructuralKeyRecord, CanonicalStructuralKeySource,
    CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION, MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
    MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS, MAX_PROFILE1_MAPPING_SORT_ENTRIES,
    MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
};

pub use resolve_anchor::{
    resolve_profile1_anchor_aliases, AliasBinding, AnchorAliasError, AnchorAliasErrorKind,
    AnchorAliasLimits, AnchorAliasSource, AnchorDeclaration, ANCHOR_ALIAS_RESOLUTION_VERSION,
    MAX_PROFILE1_ALIAS_BINDINGS, MAX_PROFILE1_ANCHOR_DECLARATIONS,
};

pub use resolve_collection_tag::{
    resolve_profile1_cst_node_collection_tag, CollectionTagError, CollectionTagErrorKind,
    CollectionTagLimits, ResolvedCollection, ResolvedCollectionTag,
    COLLECTION_TAG_RESOLUTION_VERSION,
};

pub use resolve_integer::{
    convert_core_integer, CoreInteger, CoreIntegerError, CoreIntegerErrorKind, CoreIntegerLimits,
    CORE_INTEGER_CONVERSION_VERSION, CORE_INTEGER_MAGNITUDE_RADIX, MAX_PROFILE1_CORE_INTEGER_LIMBS,
};

pub use resolve_node_table::{
    canonical_semantic_node_table_limits, compose_profile1_semantic_node_table,
    SemanticAliasRedirect, SemanticNodeKind, SemanticNodeSlot, SemanticNodeTableError,
    SemanticNodeTableErrorKind, SemanticNodeTableLimits, SemanticNodeTableSource,
    MAX_PROFILE1_SEMANTIC_ALIAS_REDIRECTS, MAX_PROFILE1_SEMANTIC_COLLECTIONS,
    MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES, SEMANTIC_NODE_TABLE_TRANSFORMATION_VERSION,
};

pub use resolve_float::{
    convert_core_finite_float, CoreFiniteFloat, CoreFiniteFloatError, CoreFiniteFloatErrorKind,
    CoreFiniteFloatLimits, CORE_FINITE_FLOAT_CONVERSION_VERSION,
    MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS, MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
};

pub use resolve_special_float::{
    convert_core_special_float, CoreSpecialFloat, CoreSpecialFloatError, CoreSpecialFloatErrorKind,
    CoreSpecialFloatLimits, CORE_SPECIAL_FLOAT_CONVERSION_VERSION,
};

pub use resolve_tag::{
    resolve_profile1_node_tag_property, ResolvedTagCodePoint, ResolvedTagKind, ResolvedTagOrigin,
    ResolvedTagProperty, TagResolutionError, TagResolutionErrorKind, TagResolutionLimits,
    MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS, TAG_RESOLUTION_VERSION,
};

pub use resolve_topology::{
    canonical_semantic_topology_limits, compose_profile1_semantic_topology, SemanticDocumentRoot,
    SemanticMappingEdge, SemanticSequenceEdge, SemanticTopologyError, SemanticTopologyErrorKind,
    SemanticTopologyLimits, SemanticTopologyNode, SemanticTopologySource,
    MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS, MAX_PROFILE1_SEMANTIC_MAPPING_EDGES,
    MAX_PROFILE1_SEMANTIC_NODES, MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES,
    SEMANTIC_TOPOLOGY_TRANSFORMATION_VERSION,
};

pub use resolve_scalar_node::{
    decode_profile1_cst_node_scalar, CstScalarDecodeError, CstScalarDecodeErrorKind,
    CstScalarDecodeLimits, DecodedCstScalar, CST_SCALAR_DECODE_VERSION,
};

pub use resolve_scalar_table::{
    canonical_semantic_scalar_table_limits, compose_profile1_semantic_scalar_table,
    SemanticScalarTableError, SemanticScalarTableErrorKind, SemanticScalarTableLimits,
    SemanticScalarTableSource, MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
    SEMANTIC_SCALAR_TABLE_TRANSFORMATION_VERSION,
};

pub use resolve_scalar_value::{
    resolve_profile1_cst_node_scalar_value, ResolvedScalar, ResolvedScalarTag, ResolvedScalarValue,
    ScalarValueError, ScalarValueErrorKind, ScalarValueLimits, SCALAR_VALUE_RESOLUTION_VERSION,
};

pub use scalar_decode::{
    decode_profile1_block_scalar_content, decode_profile1_double_quoted_scalar_content,
    decode_profile1_plain_scalar_content, decode_profile1_single_quoted_scalar_content,
    DecodedContentOrigin, DecodedContentScalar, DecodedScalarContent, DecodedScalarStyle,
    ScalarDecodeError, ScalarDecodeErrorKind, ScalarDecodeLimits,
    MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS, SCALAR_DECODE_TRANSFORMATION_VERSION,
};

pub use plain::{
    canonical_plain_scalar_limits, scan_profile1_plain_scalars, PlainScalar, PlainScalarError,
    PlainScalarErrorKind, PlainScalarScanLimits, PlainScalarSource, PlainScalarSourceView,
    MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS,
    PLAIN_SCALAR_TRANSFORMATION_VERSION,
};

pub use structural::{
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    scan_profile1_structural_lexemes, StructuralCandidateRole, StructuralLexeme,
    StructuralLexemeSource, StructuralScanError, StructuralScanErrorKind, StructuralScanLimits,
    MAX_PROFILE1_STRUCTURAL_LEXEMES, STRUCTURAL_LEXEME_TRANSFORMATION_VERSION,
};

pub use token::{
    canonical_completed_token_limits, scan_profile1_completed_tokens, CompletedToken,
    CompletedTokenError, CompletedTokenErrorKind, CompletedTokenKind, CompletedTokenLimits,
    CompletedTokenPart, CompletedTokenPartKind, CompletedTokenSource,
    COMPLETED_TOKEN_TRANSFORMATION_VERSION, MAX_PROFILE1_COMPLETED_TOKENS, MAX_PROFILE1_FLOW_DEPTH,
};

pub use utf8::{
    decode_profile1, BomPolicy, DecodeError, DecodeErrorKind, DecodeLimits, DecodedScalar,
    DecodedSource, SourcePosition, SourceSpan, CRUCIBLE_YAML_PROFILE_VERSION,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_SOURCE_BYTES, UTF8_TRANSFORMATION_VERSION,
};
