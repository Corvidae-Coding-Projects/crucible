#![forbid(unsafe_code)]
//! Project-owned, Verus-authored Crucible YAML profile implementation.

pub mod atom;
pub mod utf8;

pub use atom::{
    atomize_profile1, classify_lexical_atom, AtomizeError, AtomizeErrorKind, AtomizeLimits,
    AtomizedSource, LexicalAtom, LexicalAtomKind, YamlIndicator,
    LEXICAL_ATOM_TRANSFORMATION_VERSION, MAX_PROFILE1_LEXICAL_ATOMS,
};

pub use utf8::{
    decode_profile1, BomPolicy, DecodeError, DecodeErrorKind, DecodeLimits, DecodedScalar,
    DecodedSource, SourcePosition, SourceSpan, CRUCIBLE_YAML_PROFILE_VERSION,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_SOURCE_BYTES, UTF8_TRANSFORMATION_VERSION,
};
