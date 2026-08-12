#![forbid(unsafe_code)]
//! Project-owned, Verus-authored Crucible YAML profile implementation.

pub mod atom;
pub mod layout;
pub mod plain;
pub mod quoted;
pub mod structural;
pub mod utf8;

pub use atom::{
    atomize_profile1, classify_lexical_atom, AtomizeError, AtomizeErrorKind, AtomizeLimits,
    AtomizedSource, LexicalAtom, LexicalAtomKind, YamlIndicator,
    LEXICAL_ATOM_TRANSFORMATION_VERSION, MAX_PROFILE1_LEXICAL_ATOMS,
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
    scan_profile1_plain_scalars, PlainScalar, PlainScalarError, PlainScalarErrorKind,
    PlainScalarScanLimits, PlainScalarSource, PlainScalarSourceView, MAX_PROFILE1_PLAIN_SCALARS,
    MAX_PROFILE1_PLAIN_SCALAR_ATOMS, PLAIN_SCALAR_TRANSFORMATION_VERSION,
};

pub use structural::{
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    scan_profile1_structural_lexemes, StructuralCandidateRole, StructuralLexeme,
    StructuralLexemeSource, StructuralScanError, StructuralScanErrorKind, StructuralScanLimits,
    MAX_PROFILE1_STRUCTURAL_LEXEMES, STRUCTURAL_LEXEME_TRANSFORMATION_VERSION,
};

pub use utf8::{
    decode_profile1, BomPolicy, DecodeError, DecodeErrorKind, DecodeLimits, DecodedScalar,
    DecodedSource, SourcePosition, SourceSpan, CRUCIBLE_YAML_PROFILE_VERSION,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_SOURCE_BYTES, UTF8_TRANSFORMATION_VERSION,
};
