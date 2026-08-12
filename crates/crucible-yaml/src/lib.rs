#![forbid(unsafe_code)]
//! Project-owned, Verus-authored Crucible YAML profile implementation.

pub mod atom;
pub mod block;
pub mod cst;
pub mod layout;
pub mod plain;
pub mod quoted;
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
