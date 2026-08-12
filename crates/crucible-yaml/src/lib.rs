#![forbid(unsafe_code)]
//! Project-owned, Verus-authored Crucible YAML profile implementation.

pub mod utf8;

pub use utf8::{
    decode_profile1, BomPolicy, DecodeError, DecodeErrorKind, DecodeLimits, DecodedScalar,
    DecodedSource, SourcePosition, SourceSpan, CRUCIBLE_YAML_PROFILE_VERSION,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_SOURCE_BYTES, UTF8_TRANSFORMATION_VERSION,
};
