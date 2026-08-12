//! Bounded UTF-8 decoding for Crucible YAML profile 1.
//!
//! This module deliberately decodes bytes itself. It neither delegates to a host string decoder
//! nor constructs an unchecked string. Every accepted Unicode scalar retains a half-open span in
//! the original byte artifact. CRLF and CR are normalized to LF while their original byte spans
//! remain exact.
use vstd::prelude::*;

verus! {

pub const CRUCIBLE_YAML_PROFILE_VERSION: u16 = 1;

pub const UTF8_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_SOURCE_BYTES: u64 = 16 * 1024 * 1024;

pub const MAX_PROFILE1_DECODED_SCALARS: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BomPolicy {
    AllowAndStrip,
    Forbid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeLimits {
    max_source_bytes: u64,
    max_decoded_scalars: u64,
}

#[verifier::ext_equal]
pub struct DecodeLimitsView {
    pub max_source_bytes: u64,
    pub max_decoded_scalars: u64,
}

impl View for DecodeLimits {
    type V = DecodeLimitsView;

    closed spec fn view(&self) -> DecodeLimitsView {
        DecodeLimitsView {
            max_source_bytes: self.max_source_bytes,
            max_decoded_scalars: self.max_decoded_scalars,
        }
    }
}

impl DecodeLimits {
    pub fn new(max_source_bytes: u64, max_decoded_scalars: u64) -> (limits: Self)
        ensures
            limits@.max_source_bytes == max_source_bytes,
            limits@.max_decoded_scalars == max_decoded_scalars,
    {
        Self { max_source_bytes, max_decoded_scalars }
    }

    pub fn max_source_bytes(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_source_bytes,
    {
        self.max_source_bytes
    }

    pub fn max_decoded_scalars(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_decoded_scalars,
    {
        self.max_decoded_scalars
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeErrorKind {
    SourceByteLimitExceeded,
    DecodedScalarLimitExceeded,
    ForbiddenByteOrderMark,
    UnexpectedContinuationByte,
    InvalidLeadingByte,
    TruncatedSequence,
    InvalidContinuationByte,
    OverlongEncoding,
    SurrogateCodePoint,
    CodePointOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct DecodeErrorView {
    pub kind: DecodeErrorKind,
    pub byte_offset: u64,
}

impl View for DecodeError {
    type V = DecodeErrorView;

    closed spec fn view(&self) -> DecodeErrorView {
        DecodeErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl DecodeError {
    fn at(kind: DecodeErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error.kind == kind,
            error.byte_offset == byte_offset,
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: DecodeErrorKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn byte_offset(&self) -> (offset: u64)
        ensures
            offset == self@.byte_offset,
    {
        self.byte_offset
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePosition {
    byte_offset: u64,
    line: u64,
    column: u64,
}

#[verifier::ext_equal]
pub struct SourcePositionView {
    pub byte_offset: u64,
    pub line: u64,
    pub column: u64,
}

impl View for SourcePosition {
    type V = SourcePositionView;

    closed spec fn view(&self) -> SourcePositionView {
        SourcePositionView { byte_offset: self.byte_offset, line: self.line, column: self.column }
    }
}

impl SourcePosition {
    fn new(byte_offset: u64, line: u64, column: u64) -> (position: Self)
        ensures
            position.byte_offset == byte_offset,
            position.line == line,
            position.column == column,
    {
        Self { byte_offset, line, column }
    }

    pub fn byte_offset(&self) -> (offset: u64)
        ensures
            offset == self@.byte_offset,
    {
        self.byte_offset
    }

    pub fn line(&self) -> (line: u64)
        ensures
            line == self@.line,
    {
        self.line
    }

    pub fn column(&self) -> (column: u64)
        ensures
            column == self@.column,
    {
        self.column
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    start: SourcePosition,
    end: SourcePosition,
}

#[verifier::ext_equal]
pub struct SourceSpanView {
    pub start: SourcePositionView,
    pub end: SourcePositionView,
}

impl View for SourceSpan {
    type V = SourceSpanView;

    closed spec fn view(&self) -> SourceSpanView {
        SourceSpanView { start: self.start@, end: self.end@ }
    }
}

impl SourceSpan {
    fn new(start: SourcePosition, end: SourcePosition) -> (span: Self)
        ensures
            span.start == start,
            span.end == end,
    {
        Self { start, end }
    }

    pub fn start(&self) -> (start: SourcePosition)
        ensures
            start@ == self@.start,
    {
        self.start
    }

    pub fn end(&self) -> (end: SourcePosition)
        ensures
            end@ == self@.end,
    {
        self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// One decoded Unicode scalar and its exact source span.
///
/// ```compile_fail
/// use crucible_yaml::{DecodedScalar, SourcePosition, SourceSpan};
///
/// let invalid = DecodedScalar {
///     code_point: 0xd800,
///     span: SourceSpan {
///         start: SourcePosition { byte_offset: 2, line: 0, column: 2 },
///         end: SourcePosition { byte_offset: 1, line: 0, column: 1 },
///     },
/// };
/// ```
pub struct DecodedScalar {
    code_point: u32,
    span: SourceSpan,
}

#[verifier::ext_equal]
pub struct DecodedScalarView {
    pub code_point: u32,
    pub span: SourceSpanView,
}

impl View for DecodedScalar {
    type V = DecodedScalarView;

    closed spec fn view(&self) -> DecodedScalarView {
        DecodedScalarView { code_point: self.code_point, span: self.span@ }
    }
}

impl DeepView for DecodedScalar {
    type V = DecodedScalarView;

    closed spec fn deep_view(&self) -> DecodedScalarView {
        self@
    }
}

impl DecodedScalar {
    fn new(code_point: u32, span: SourceSpan) -> (scalar: Self)
        ensures
            scalar.code_point == code_point,
            scalar.span == span,
    {
        Self { code_point, span }
    }

    pub fn code_point(&self) -> (code_point: u32)
        ensures
            code_point == self@.code_point,
    {
        self.code_point
    }

    pub fn span(&self) -> (span: &SourceSpan)
        ensures
            span@ == self@.span,
    {
        &self.span
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    scalars: Vec<DecodedScalar>,
}

#[verifier::ext_equal]
pub struct DecodedSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub scalars: Seq<DecodedScalarView>,
}

pub open spec fn decoded_scalar_views_spec(scalars: Seq<DecodedScalar>) -> Seq<DecodedScalarView> {
    Seq::new(scalars.len(), |index: int| scalars[index]@)
}

impl View for DecodedSource {
    type V = DecodedSourceView;

    closed spec fn view(&self) -> DecodedSourceView {
        DecodedSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            scalars: decoded_scalar_views_spec(self.scalars@),
        }
    }
}

impl DecodedSource {
    pub fn profile_version(&self) -> (version: u16)
        ensures
            version == self@.profile_version,
    {
        self.profile_version
    }

    pub fn transformation_version(&self) -> (version: u16)
        ensures
            version == self@.transformation_version,
    {
        self.transformation_version
    }

    pub fn source_len_bytes(&self) -> (length: u64)
        ensures
            length == self@.source_len_bytes,
    {
        self.source_len_bytes
    }

    pub fn bom_bytes(&self) -> (length: u64)
        ensures
            length == self@.bom_bytes,
    {
        self.bom_bytes
    }

    pub fn scalars(&self) -> (scalars: &[DecodedScalar])
        ensures
            decoded_scalar_views_spec(scalars@) == self@.scalars,
    {
        self.scalars.as_slice()
    }

    pub open spec fn well_formed_spec(&self) -> bool {
        decoded_source_well_formed_spec(self@)
    }
}

pub open spec fn unicode_scalar_value_spec(code_point: u32) -> bool {
    code_point <= 0x10ffff && !(0xd800 <= code_point <= 0xdfff)
}

closed spec fn normalized_scalar_spec(scalar: DecodedScalar) -> bool {
    normalized_scalar_view_spec(scalar@)
}

pub open spec fn normalized_scalar_view_spec(scalar: DecodedScalarView) -> bool {
    unicode_scalar_value_spec(scalar.code_point) && scalar.code_point != 0x0d
        && scalar.span.start.byte_offset < scalar.span.end.byte_offset && if scalar.code_point
        == 0x0a {
        scalar.span.end.line == scalar.span.start.line + 1 && scalar.span.end.column == 0
    } else {
        scalar.span.end.line == scalar.span.start.line && scalar.span.end.column
            == scalar.span.start.column + 1
    }
}

closed spec fn decoded_prefix_well_formed_spec(
    scalars: Seq<DecodedScalar>,
    bom_bytes: u64,
    consumed_bytes: u64,
) -> bool {
    if scalars.len() == 0 {
        consumed_bytes == bom_bytes
    } else {
        scalars[0].span.start == SourcePosition { byte_offset: bom_bytes, line: 0, column: 0 }
            && scalars[scalars.len() - 1].span.end.byte_offset == consumed_bytes && forall|
            index: int,
        |
            0 <= index < scalars.len() ==> normalized_scalar_spec(#[trigger] scalars[index])
                && scalars[index].span.end.byte_offset <= consumed_bytes && (index > 0
                ==> scalars[index - 1].span.end == scalars[index].span.start)
    }
}

pub closed spec fn decoded_prefix_view_well_formed_spec(
    scalars: Seq<DecodedScalarView>,
    bom_bytes: u64,
    consumed_bytes: u64,
) -> bool {
    if scalars.len() == 0 {
        consumed_bytes == bom_bytes
    } else {
        scalars[0].span.start == SourcePositionView { byte_offset: bom_bytes, line: 0, column: 0 }
            && scalars[scalars.len() - 1].span.end.byte_offset == consumed_bytes && forall|
            index: int,
        |
            0 <= index < scalars.len() ==> normalized_scalar_view_spec(#[trigger] scalars[index])
                && scalars[index].span.end.byte_offset <= consumed_bytes && (index > 0
                ==> scalars[index - 1].span.end == scalars[index].span.start)
    }
}

pub closed spec fn decoded_source_well_formed_spec(source: DecodedSourceView) -> bool {
    source.profile_version == CRUCIBLE_YAML_PROFILE_VERSION && source.transformation_version
        == UTF8_TRANSFORMATION_VERSION && (source.bom_bytes == 0 || source.bom_bytes == 3)
        && source.bom_bytes <= source.source_len_bytes && decoded_prefix_view_well_formed_spec(
        source.scalars,
        source.bom_bytes,
        source.source_len_bytes,
    )
}

proof fn lemma_empty_prefix(bom_bytes: u64)
    ensures
        decoded_prefix_well_formed_spec(Seq::empty(), bom_bytes, bom_bytes),
{
    reveal(decoded_prefix_well_formed_spec);
    reveal(normalized_scalar_spec);
}

proof fn lemma_extend_prefix(
    scalars: Seq<DecodedScalar>,
    bom_bytes: u64,
    consumed_bytes: u64,
    scalar: DecodedScalar,
)
    requires
        decoded_prefix_well_formed_spec(scalars, bom_bytes, consumed_bytes),
        scalar.span.start.byte_offset == consumed_bytes,
        scalar.span.start == if scalars.len() == 0 {
            SourcePosition { byte_offset: bom_bytes, line: 0, column: 0 }
        } else {
            scalars[scalars.len() - 1].span.end
        },
        normalized_scalar_spec(scalar),
    ensures
        decoded_prefix_well_formed_spec(
            scalars.push(scalar),
            bom_bytes,
            scalar.span.end.byte_offset,
        ),
{
    reveal(decoded_prefix_well_formed_spec);
    assert forall|index: int|
        0 <= index < scalars.push(scalar).len() implies normalized_scalar_spec(
        #[trigger] scalars.push(scalar)[index],
    ) && scalars.push(scalar)[index].span.end.byte_offset <= scalar.span.end.byte_offset && (index
        > 0 ==> scalars.push(scalar)[index - 1].span.end == scalars.push(
        scalar,
    )[index].span.start) by {
        if index < scalars.len() {
            assert(scalars.push(scalar)[index] == scalars[index]);
            if index > 0 {
                assert(scalars.push(scalar)[index - 1] == scalars[index - 1]);
            }
            assert(scalars[index].span.end.byte_offset <= consumed_bytes);
            assert(consumed_bytes == scalar.span.start.byte_offset);
            assert(scalar.span.start.byte_offset < scalar.span.end.byte_offset);
        } else {
            assert(index == scalars.len());
            assert(scalars.push(scalar)[index] == scalar);
            if index > 0 {
                assert(scalars.push(scalar)[index - 1] == scalars[index - 1]);
            }
        }
    }
}

#[verifier::spinoff_prover]
proof fn lemma_prefix_views(scalars: Seq<DecodedScalar>, bom_bytes: u64, consumed_bytes: u64)
    requires
        decoded_prefix_well_formed_spec(scalars, bom_bytes, consumed_bytes),
    ensures
        decoded_prefix_view_well_formed_spec(
            decoded_scalar_views_spec(scalars),
            bom_bytes,
            consumed_bytes,
        ),
{
    reveal(decoded_prefix_well_formed_spec);
    reveal(decoded_prefix_view_well_formed_spec);
    reveal(decoded_scalar_views_spec);
    assert forall|index: int| 0 <= index < scalars.len() implies decoded_scalar_views_spec(
        scalars,
    )[index] == #[trigger] scalars[index]@ by {
        assert(decoded_scalar_views_spec(scalars)[index] == scalars[index]@);
    }
    assert forall|index: int|
        0 <= index < decoded_scalar_views_spec(scalars).len() implies normalized_scalar_view_spec(
        #[trigger] decoded_scalar_views_spec(scalars)[index],
    ) && decoded_scalar_views_spec(scalars)[index].span.end.byte_offset <= consumed_bytes && (index
        > 0 ==> decoded_scalar_views_spec(scalars)[index - 1].span.end == decoded_scalar_views_spec(
        scalars,
    )[index].span.start) by {
        assert(decoded_scalar_views_spec(scalars)[index] == scalars[index]@);
        assert(normalized_scalar_spec(scalars[index]));
        reveal(normalized_scalar_spec);
        if index > 0 {
            assert(decoded_scalar_views_spec(scalars)[index - 1] == scalars[index - 1]@);
        }
    }
}

#[allow(clippy::manual_range_contains)]
fn continuation(byte: u8) -> (is_continuation: bool)
    ensures
        is_continuation == continuation_byte_spec(byte),
{
    0x80 <= byte && byte <= 0xbf
}

pub open spec fn continuation_byte_spec(byte: u8) -> bool {
    0x80 <= byte && byte <= 0xbf
}

pub open spec fn decode_one_grammar_spec(input: Seq<u8>, index: int) -> Result<
    (u32, int),
    (DecodeErrorKind, int),
>
    recommends
        0 <= index < input.len(),
{
    let first = input[index];
    if first <= 0x7f {
        Ok((first as u32, 1))
    } else if first <= 0xbf {
        Err((DecodeErrorKind::UnexpectedContinuationByte, index))
    } else if first <= 0xc1 {
        Err((DecodeErrorKind::OverlongEncoding, index))
    } else if first <= 0xdf {
        if input.len() - index < 2 {
            Err((DecodeErrorKind::TruncatedSequence, input.len() as int))
        } else if !continuation_byte_spec(input[index + 1]) {
            Err((DecodeErrorKind::InvalidContinuationByte, index + 1))
        } else {
            Ok(
                (
                    (((first as u32 - 0xc0u32) * 64u32) + (input[index + 1] as u32
                        - 0x80u32)) as u32,
                    2,
                ),
            )
        }
    } else if first <= 0xef {
        if input.len() - index < 2 {
            Err((DecodeErrorKind::TruncatedSequence, input.len() as int))
        } else if !continuation_byte_spec(input[index + 1]) {
            Err((DecodeErrorKind::InvalidContinuationByte, index + 1))
        } else if first == 0xe0 && input[index + 1] < 0xa0 {
            Err((DecodeErrorKind::OverlongEncoding, index))
        } else if first == 0xed && input[index + 1] >= 0xa0 {
            Err((DecodeErrorKind::SurrogateCodePoint, index))
        } else if input.len() - index < 3 {
            Err((DecodeErrorKind::TruncatedSequence, input.len() as int))
        } else if !continuation_byte_spec(input[index + 2]) {
            Err((DecodeErrorKind::InvalidContinuationByte, index + 2))
        } else {
            Ok(
                (
                    (((first as u32 - 0xe0u32) * 4096u32) + ((input[index + 1] as u32 - 0x80u32)
                        * 64u32) + (input[index + 2] as u32 - 0x80u32)) as u32,
                    3,
                ),
            )
        }
    } else if first <= 0xf4 {
        if input.len() - index < 2 {
            Err((DecodeErrorKind::TruncatedSequence, input.len() as int))
        } else if !continuation_byte_spec(input[index + 1]) {
            Err((DecodeErrorKind::InvalidContinuationByte, index + 1))
        } else if first == 0xf0 && input[index + 1] < 0x90 {
            Err((DecodeErrorKind::OverlongEncoding, index))
        } else if first == 0xf4 && input[index + 1] > 0x8f {
            Err((DecodeErrorKind::CodePointOutOfRange, index))
        } else if input.len() - index < 3 {
            Err((DecodeErrorKind::TruncatedSequence, input.len() as int))
        } else if !continuation_byte_spec(input[index + 2]) {
            Err((DecodeErrorKind::InvalidContinuationByte, index + 2))
        } else if input.len() - index < 4 {
            Err((DecodeErrorKind::TruncatedSequence, input.len() as int))
        } else if !continuation_byte_spec(input[index + 3]) {
            Err((DecodeErrorKind::InvalidContinuationByte, index + 3))
        } else {
            Ok(
                (
                    (((first as u32 - 0xf0u32) * 262_144u32) + ((input[index + 1] as u32 - 0x80u32)
                        * 4096u32) + ((input[index + 2] as u32 - 0x80u32) * 64u32) + (input[index
                        + 3] as u32 - 0x80u32)) as u32,
                    4,
                ),
            )
        }
    } else if first <= 0xf7 {
        Err((DecodeErrorKind::CodePointOutOfRange, index))
    } else {
        Err((DecodeErrorKind::InvalidLeadingByte, index))
    }
}

/// Solver-isolated canonical single-scalar result used by executable correspondence proofs.
pub closed spec fn decode_one_spec(input: Seq<u8>, index: int) -> Result<
    (u32, int),
    (DecodeErrorKind, int),
>
    recommends
        0 <= index < input.len(),
{
    decode_one_grammar_spec(input, index)
}

closed spec fn normalized_step_spec(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    code_point: u32,
    width: int,
) -> (DecodedScalar, int, u64, u64)
    recommends
        0 <= index < input.len(),
        1 <= width,
        index + width <= input.len(),
        line < u64::MAX,
        column < u64::MAX,
{
    let consumed_width = normalized_width_spec(input, index, code_point, width);
    let normalized = if code_point == 0x0d {
        0x0a
    } else {
        code_point
    };
    let next_index = index + consumed_width;
    let next_line = if normalized == 0x0a {
        (line + 1) as u64
    } else {
        line
    };
    let next_column = if normalized == 0x0a {
        0
    } else {
        (column + 1) as u64
    };
    (
        DecodedScalar {
            code_point: normalized,
            span: SourceSpan {
                start: SourcePosition { byte_offset: index as u64, line, column },
                end: SourcePosition {
                    byte_offset: next_index as u64,
                    line: next_line,
                    column: next_column,
                },
            },
        },
        next_index,
        next_line,
        next_column,
    )
}

pub open spec fn normalized_width_spec(
    input: Seq<u8>,
    index: int,
    code_point: u32,
    width: int,
) -> int {
    if code_point == 0x0d && index + 1 < input.len() && input[index + 1] == 0x0a {
        2
    } else {
        width
    }
}

closed spec fn decode_next_spec(input: Seq<u8>, index: int, line: u64, column: u64) -> Result<
    (DecodedScalar, int, u64, u64),
    DecodeErrorView,
>
    recommends
        0 <= index < input.len(),
        line < u64::MAX,
        column < u64::MAX,
{
    match decode_one_spec(input, index) {
        Ok((code_point, width)) => {
            Ok(normalized_step_spec(input, index, line, column, code_point, width))
        },
        Err((kind, byte_offset)) => { Err(DecodeErrorView { kind, byte_offset: byte_offset as u64 })
        },
    }
}

closed spec fn decode_loop_spec(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    remaining_scalars: u64,
    scalars: Seq<DecodedScalar>,
    fuel: int,
) -> Result<Seq<DecodedScalar>, DecodeErrorView>
    decreases fuel,
{
    if index < 0 || input.len() < index || fuel < 0 || line == u64::MAX || column == u64::MAX {
        Err(DecodeErrorView { kind: DecodeErrorKind::InvalidLeadingByte, byte_offset: 0 })
    } else if index == input.len() {
        Ok(scalars)
    } else if remaining_scalars == 0 {
        Err(
            DecodeErrorView {
                kind: DecodeErrorKind::DecodedScalarLimitExceeded,
                byte_offset: index as u64,
            },
        )
    } else if fuel <= 0 {
        Err(DecodeErrorView { kind: DecodeErrorKind::TruncatedSequence, byte_offset: index as u64 })
    } else {
        match decode_next_spec(input, index, line, column) {
            Ok((scalar, next_index, next_line, next_column)) => {
                decode_loop_spec(
                    input,
                    next_index,
                    next_line,
                    next_column,
                    (remaining_scalars - 1) as u64,
                    scalars.push(scalar),
                    fuel - 1,
                )
            },
            Err(error) => Err(error),
        }
    }
}

/// Pure acceptance predicate for a bounded sequence of profile-1 Unicode scalars.
///
/// This deliberately excludes output construction so callers can establish acceptance without
/// gaining access to the private constructors of invariant-bearing decoded values.
pub open spec fn profile1_decodable_tail_spec(
    input: Seq<u8>,
    index: int,
    remaining_scalars: u64,
    fuel: int,
) -> bool
    decreases fuel,
{
    if index < 0 || input.len() < index || fuel < 0 {
        false
    } else if index == input.len() {
        true
    } else if remaining_scalars == 0 || fuel <= 0 {
        false
    } else {
        match decode_one_grammar_spec(input, index) {
            Ok((code_point, width)) => {
                profile1_decodable_tail_spec(
                    input,
                    index + normalized_width_spec(input, index, code_point, width),
                    (remaining_scalars - 1) as u64,
                    fuel - 1,
                )
            },
            Err(_) => false,
        }
    }
}

/// True exactly when the declared limits and BOM policy admit a fully decodable input.
pub open spec fn profile1_decodable_spec(
    input: Seq<u8>,
    limits: DecodeLimitsView,
    bom_policy: BomPolicy,
) -> bool {
    let effective_source_limit = if limits.max_source_bytes < MAX_PROFILE1_SOURCE_BYTES {
        limits.max_source_bytes
    } else {
        MAX_PROFILE1_SOURCE_BYTES
    };
    let effective_scalar_limit = if limits.max_decoded_scalars < MAX_PROFILE1_DECODED_SCALARS {
        limits.max_decoded_scalars
    } else {
        MAX_PROFILE1_DECODED_SCALARS
    };
    let has_bom = input.len() >= 3 && input[0] == 0xef && input[1] == 0xbb && input[2] == 0xbf;
    let start = if has_bom {
        3
    } else {
        0
    };
    input.len() <= effective_source_limit && !(has_bom && bom_policy == BomPolicy::Forbid)
        && profile1_decodable_tail_spec(input, start, effective_scalar_limit, input.len() - start)
}

closed spec fn finish_decode_spec(
    source_len_bytes: u64,
    bom_bytes: u64,
    decoded: Result<Seq<DecodedScalar>, DecodeErrorView>,
) -> Result<DecodedSourceView, DecodeErrorView> {
    match decoded {
        Ok(scalars) => Ok(
            DecodedSourceView {
                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                transformation_version: UTF8_TRANSFORMATION_VERSION,
                source_len_bytes,
                bom_bytes,
                scalars: decoded_scalar_views_spec(scalars),
            },
        ),
        Err(error) => Err(error),
    }
}

/// The total pure result of Crucible YAML profile-1 byte decoding.
///
/// Unlike a success-only invariant, this function fixes every accepted output and every typed
/// rejection, including diagnostic precedence and byte offsets.
pub closed spec fn decode_profile1_spec(
    input: Seq<u8>,
    limits: DecodeLimitsView,
    bom_policy: BomPolicy,
) -> Result<DecodedSourceView, DecodeErrorView> {
    let effective_source_limit = if limits.max_source_bytes < MAX_PROFILE1_SOURCE_BYTES {
        limits.max_source_bytes
    } else {
        MAX_PROFILE1_SOURCE_BYTES
    };
    let effective_scalar_limit = if limits.max_decoded_scalars < MAX_PROFILE1_DECODED_SCALARS {
        limits.max_decoded_scalars
    } else {
        MAX_PROFILE1_DECODED_SCALARS
    };
    if input.len() > effective_source_limit {
        Err(
            DecodeErrorView {
                kind: DecodeErrorKind::SourceByteLimitExceeded,
                byte_offset: effective_source_limit,
            },
        )
    } else {
        let has_bom = input.len() >= 3 && input[0] == 0xef && input[1] == 0xbb && input[2] == 0xbf;
        if has_bom && bom_policy == BomPolicy::Forbid {
            Err(DecodeErrorView { kind: DecodeErrorKind::ForbiddenByteOrderMark, byte_offset: 0 })
        } else {
            let start = if has_bom {
                3
            } else {
                0
            };
            finish_decode_spec(
                input.len() as u64,
                start as u64,
                decode_loop_spec(
                    input,
                    start,
                    0,
                    0,
                    effective_scalar_limit,
                    Seq::empty(),
                    input.len() - start,
                ),
            )
        }
    }
}

proof fn lemma_decode_loop_done(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    remaining_scalars: u64,
    scalars: Seq<DecodedScalar>,
    fuel: int,
)
    requires
        0 <= index == input.len(),
        line < u64::MAX,
        column < u64::MAX,
        0 <= fuel,
    ensures
        decode_loop_spec(input, index, line, column, remaining_scalars, scalars, fuel) == Ok(
            scalars,
        ),
{
    reveal_with_fuel(decode_loop_spec, 1);
}

proof fn lemma_decode_loop_limit(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    scalars: Seq<DecodedScalar>,
    fuel: int,
)
    requires
        0 <= index < input.len(),
        line < u64::MAX,
        column < u64::MAX,
        0 < fuel,
    ensures
        decode_loop_spec(input, index, line, column, 0, scalars, fuel) == Err(
            DecodeErrorView {
                kind: DecodeErrorKind::DecodedScalarLimitExceeded,
                byte_offset: index as u64,
            },
        ),
{
    reveal_with_fuel(decode_loop_spec, 1);
}

proof fn lemma_decode_loop_error(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    remaining_scalars: u64,
    scalars: Seq<DecodedScalar>,
    fuel: int,
    error: DecodeErrorView,
)
    requires
        0 <= index < input.len(),
        line < u64::MAX,
        column < u64::MAX,
        0 < fuel,
        0 < remaining_scalars,
        decode_next_spec(input, index, line, column) == Err(error),
    ensures
        decode_loop_spec(input, index, line, column, remaining_scalars, scalars, fuel) == Err(
            error,
        ),
{
    reveal_with_fuel(decode_loop_spec, 1);
}

proof fn lemma_decode_loop_step(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    remaining_scalars: u64,
    scalars: Seq<DecodedScalar>,
    fuel: int,
    scalar: DecodedScalar,
    next_index: int,
    next_line: u64,
    next_column: u64,
)
    requires
        0 <= index < input.len(),
        line < u64::MAX,
        column < u64::MAX,
        0 < fuel,
        0 < remaining_scalars,
        decode_next_spec(input, index, line, column) == Ok(
            (scalar, next_index, next_line, next_column),
        ),
    ensures
        decode_loop_spec(input, index, line, column, remaining_scalars, scalars, fuel)
            == decode_loop_spec(
            input,
            next_index,
            next_line,
            next_column,
            (remaining_scalars - 1) as u64,
            scalars.push(scalar),
            fuel - 1,
        ),
{
    reveal_with_fuel(decode_loop_spec, 1);
}

#[verifier::spinoff_prover]
proof fn lemma_decode_one_ok_bounds(input: Seq<u8>, index: int, code_point: u32, width: int)
    requires
        0 <= index < input.len(),
        decode_one_grammar_spec(input, index) == Ok((code_point, width)),
    ensures
        1 <= width <= 4,
        index + width <= input.len(),
        code_point == 0x0d ==> width == 1,
{
}

#[verifier::spinoff_prover]
proof fn lemma_normalized_step_bounds(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    code_point: u32,
    width: int,
)
    requires
        0 <= index < input.len(),
        input.len() <= MAX_PROFILE1_SOURCE_BYTES,
        line as int <= index,
        column as int <= index,
        1 <= width <= 4,
        index + width <= input.len(),
        code_point == 0x0d ==> width == 1,
    ensures
        ({
            let (_, next_index, next_line, next_column) = normalized_step_spec(
                input,
                index,
                line,
                column,
                code_point,
                width,
            );
            index < next_index <= input.len() && next_line as int <= next_index
                && next_column as int <= next_index
        }),
{
    reveal(normalized_step_spec);
}

#[verifier::spinoff_prover]
proof fn lemma_decodable_tail_yields_loop_ok(
    input: Seq<u8>,
    index: int,
    line: u64,
    column: u64,
    remaining_scalars: u64,
    scalars: Seq<DecodedScalar>,
    fuel: int,
)
    requires
        profile1_decodable_tail_spec(input, index, remaining_scalars, fuel),
        input.len() <= MAX_PROFILE1_SOURCE_BYTES,
        0 <= index <= input.len(),
        line as int <= index,
        column as int <= index,
        0 <= fuel,
    ensures
        exists|decoded: Seq<DecodedScalar>|
            decode_loop_spec(input, index, line, column, remaining_scalars, scalars, fuel) == Ok(
                decoded,
            ),
    decreases fuel,
{
    reveal_with_fuel(profile1_decodable_tail_spec, 1);
    if index == input.len() {
        reveal_with_fuel(decode_loop_spec, 1);
    } else {
        assert(0 < fuel);
        assert(0 < remaining_scalars);
        match decode_one_grammar_spec(input, index) {
            Ok((code_point, width)) => {
                lemma_decode_one_ok_bounds(input, index, code_point, width);
                let step = normalized_step_spec(input, index, line, column, code_point, width);
                let scalar = step.0;
                let next_index = step.1;
                let next_line = step.2;
                let next_column = step.3;
                lemma_normalized_step_bounds(input, index, line, column, code_point, width);
                assert(decode_next_spec(input, index, line, column) == Ok(step)) by {
                    reveal(decode_next_spec);
                }
                lemma_decodable_tail_yields_loop_ok(
                    input,
                    next_index,
                    next_line,
                    next_column,
                    (remaining_scalars - 1) as u64,
                    scalars.push(scalar),
                    fuel - 1,
                );
                let decoded = choose|decoded: Seq<DecodedScalar>|
                    decode_loop_spec(
                        input,
                        next_index,
                        next_line,
                        next_column,
                        (remaining_scalars - 1) as u64,
                        scalars.push(scalar),
                        fuel - 1,
                    ) == Ok(decoded);
                reveal_with_fuel(decode_loop_spec, 1);
                assert(decode_loop_spec(
                    input,
                    index,
                    line,
                    column,
                    remaining_scalars,
                    scalars,
                    fuel,
                ) == Ok(decoded));
            },
            Err(_) => {
                assert(false);
            },
        }
    }
}

/// A proved bridge from the public acceptance predicate to the total decoder result.
pub proof fn lemma_profile1_decodable_is_ok(
    input: Seq<u8>,
    limits: DecodeLimitsView,
    bom_policy: BomPolicy,
)
    requires
        profile1_decodable_spec(input, limits, bom_policy),
    ensures
        exists|source: DecodedSourceView|
            decode_profile1_spec(input, limits, bom_policy) == Ok(source),
{
    let effective_scalar_limit = if limits.max_decoded_scalars < MAX_PROFILE1_DECODED_SCALARS {
        limits.max_decoded_scalars
    } else {
        MAX_PROFILE1_DECODED_SCALARS
    };
    let has_bom = input.len() >= 3 && input[0] == 0xef && input[1] == 0xbb && input[2] == 0xbf;
    let start = if has_bom {
        3
    } else {
        0
    };
    lemma_decodable_tail_yields_loop_ok(
        input,
        start,
        0,
        0,
        effective_scalar_limit,
        Seq::empty(),
        input.len() - start,
    );
    reveal(decode_profile1_spec);
    reveal(finish_decode_spec);
}

/// Establish the exact total result when the first non-BOM scalar has a byte-decoding error.
pub proof fn lemma_profile1_non_bom_first_error(
    input: Seq<u8>,
    limits: DecodeLimitsView,
    bom_policy: BomPolicy,
    kind: DecodeErrorKind,
    byte_offset: int,
)
    requires
        0 < input.len(),
        input.len() <= limits.max_source_bytes,
        input.len() <= MAX_PROFILE1_SOURCE_BYTES,
        0 < limits.max_decoded_scalars,
        !(input.len() >= 3 && input[0] == 0xef && input[1] == 0xbb && input[2] == 0xbf),
        0 <= byte_offset <= input.len(),
        decode_one_grammar_spec(input, 0) == Err((kind, byte_offset)),
    ensures
        decode_profile1_spec(input, limits, bom_policy) == Err(
            DecodeErrorView { kind, byte_offset: byte_offset as u64 },
        ),
{
    reveal(decode_one_spec);
    assert(decode_one_spec(input, 0) == Err((kind, byte_offset)));
    reveal(decode_profile1_spec);
    reveal(finish_decode_spec);
    reveal_with_fuel(decode_loop_spec, 1);
    reveal(decode_next_spec);
}

/// Establish source-cap precedence and its exact first-rejected byte offset.
pub proof fn lemma_profile1_source_limit_error(
    input: Seq<u8>,
    limits: DecodeLimitsView,
    bom_policy: BomPolicy,
)
    requires
        input.len() > if limits.max_source_bytes < MAX_PROFILE1_SOURCE_BYTES {
            limits.max_source_bytes
        } else {
            MAX_PROFILE1_SOURCE_BYTES
        },
    ensures
        decode_profile1_spec(input, limits, bom_policy) == Err(
            DecodeErrorView {
                kind: DecodeErrorKind::SourceByteLimitExceeded,
                byte_offset: if limits.max_source_bytes < MAX_PROFILE1_SOURCE_BYTES {
                    limits.max_source_bytes
                } else {
                    MAX_PROFILE1_SOURCE_BYTES
                },
            },
        ),
{
    reveal(decode_profile1_spec);
}

/// Establish forbidden-leading-BOM precedence after the source cap has admitted the bytes.
pub proof fn lemma_profile1_forbidden_bom_error(input: Seq<u8>, limits: DecodeLimitsView)
    requires
        input.len() <= limits.max_source_bytes,
        input.len() <= MAX_PROFILE1_SOURCE_BYTES,
        input.len() >= 3,
        input[0] == 0xef,
        input[1] == 0xbb,
        input[2] == 0xbf,
    ensures
        decode_profile1_spec(input, limits, BomPolicy::Forbid) == Err(
            DecodeErrorView { kind: DecodeErrorKind::ForbiddenByteOrderMark, byte_offset: 0 },
        ),
{
    reveal(decode_profile1_spec);
}

closed spec fn scalar_decodes_input_spec(input: Seq<u8>, scalar: DecodedScalar) -> bool {
    scalar_view_decodes_input_spec(input, scalar@)
}

pub closed spec fn scalar_view_decodes_input_spec(
    input: Seq<u8>,
    scalar: DecodedScalarView,
) -> bool {
    let start = scalar.span.start.byte_offset as int;
    let end = scalar.span.end.byte_offset as int;
    if !(0 <= start < input.len()) {
        false
    } else {
        match decode_one_spec(input, start) {
            Ok((raw_code_point, width)) => {
                if raw_code_point == 0x0d {
                    scalar.code_point == 0x0a && end - start == if start + 1 < input.len()
                        && input[start + 1] == 0x0a {
                        2
                    } else {
                        width
                    }
                } else {
                    scalar.code_point == raw_code_point && end - start == width
                }
            },
            Err(_) => false,
        }
    }
}

#[verifier::spinoff_prover]
proof fn lemma_scalar_decode_correspondence(
    input: Seq<u8>,
    scalar: DecodedScalar,
    raw_code_point: u32,
    width: int,
)
    requires
        0 <= scalar.span.start.byte_offset as int,
        (scalar.span.start.byte_offset as int) < input.len(),
        decode_one_spec(input, scalar.span.start.byte_offset as int) == Ok((raw_code_point, width)),
        if raw_code_point == 0x0d {
            scalar.code_point == 0x0a && scalar.span.end.byte_offset as int
                - scalar.span.start.byte_offset as int == if scalar.span.start.byte_offset as int
                + 1 < input.len() && input[scalar.span.start.byte_offset as int + 1] == 0x0a {
                2
            } else {
                width
            }
        } else {
            scalar.code_point == raw_code_point && scalar.span.end.byte_offset as int
                - scalar.span.start.byte_offset as int == width
        },
    ensures
        scalar_decodes_input_spec(input, scalar),
{
    reveal(scalar_decodes_input_spec);
    reveal(scalar_view_decodes_input_spec);
}

closed spec fn decoded_scalars_match_input_spec(
    input: Seq<u8>,
    scalars: Seq<DecodedScalar>,
) -> bool {
    forall|index: int|
        0 <= index < scalars.len() ==> scalar_decodes_input_spec(input, #[trigger] scalars[index])
}

pub closed spec fn decoded_scalar_views_match_input_spec(
    input: Seq<u8>,
    scalars: Seq<DecodedScalarView>,
) -> bool {
    forall|index: int|
        0 <= index < scalars.len() ==> scalar_view_decodes_input_spec(
            input,
            #[trigger] scalars[index],
        )
}

pub closed spec fn decoded_source_matches_input_spec(
    input: Seq<u8>,
    source: DecodedSourceView,
) -> bool {
    source.source_len_bytes as int == input.len() && (source.bom_bytes == 3) == (input.len() >= 3
        && input[0] == 0xef && input[1] == 0xbb && input[2] == 0xbf)
        && decoded_scalar_views_match_input_spec(input, source.scalars)
}

proof fn lemma_matching_scalar_push(
    input: Seq<u8>,
    scalars: Seq<DecodedScalar>,
    scalar: DecodedScalar,
)
    requires
        decoded_scalars_match_input_spec(input, scalars),
        scalar_decodes_input_spec(input, scalar),
    ensures
        decoded_scalars_match_input_spec(input, scalars.push(scalar)),
{
    reveal(decoded_scalars_match_input_spec);
    assert forall|index: int|
        0 <= index < scalars.push(scalar).len() implies scalar_decodes_input_spec(
        input,
        #[trigger] scalars.push(scalar)[index],
    ) by {
        if index < scalars.len() {
            assert(scalars.push(scalar)[index] == scalars[index]);
        } else {
            assert(index == scalars.len());
            assert(scalars.push(scalar)[index] == scalar);
        }
    }
}

#[verifier::spinoff_prover]
proof fn lemma_matching_scalar_views(input: Seq<u8>, scalars: Seq<DecodedScalar>)
    requires
        decoded_scalars_match_input_spec(input, scalars),
    ensures
        decoded_scalar_views_match_input_spec(input, decoded_scalar_views_spec(scalars)),
{
    reveal(decoded_scalars_match_input_spec);
    reveal(decoded_scalar_views_match_input_spec);
    reveal(decoded_scalar_views_spec);
    assert forall|index: int|
        0 <= index < decoded_scalar_views_spec(
            scalars,
        ).len() implies scalar_view_decodes_input_spec(
        input,
        #[trigger] decoded_scalar_views_spec(scalars)[index],
    ) by {
        assert(decoded_scalar_views_spec(scalars)[index] == scalars[index]@);
        assert(scalar_decodes_input_spec(input, scalars[index]));
        reveal(scalar_decodes_input_spec);
    }
}

#[verifier::spinoff_prover]
proof fn lemma_complete_decoded_source(
    input: Seq<u8>,
    source: DecodedSourceView,
    scalars: Seq<DecodedScalar>,
)
    requires
        source.profile_version == CRUCIBLE_YAML_PROFILE_VERSION,
        source.transformation_version == UTF8_TRANSFORMATION_VERSION,
        source.bom_bytes == 0 || source.bom_bytes == 3,
        source.bom_bytes <= source.source_len_bytes,
        source.source_len_bytes as int == input.len(),
        source.scalars == decoded_scalar_views_spec(scalars),
        decoded_prefix_well_formed_spec(scalars, source.bom_bytes, source.source_len_bytes),
        decoded_scalars_match_input_spec(input, scalars),
        (source.bom_bytes == 3) == (input.len() >= 3 && input[0] == 0xef && input[1] == 0xbb
            && input[2] == 0xbf),
    ensures
        decoded_source_well_formed_spec(source),
        decoded_source_matches_input_spec(input, source),
{
    lemma_prefix_views(scalars, source.bom_bytes, source.source_len_bytes);
    lemma_matching_scalar_views(input, scalars);
    reveal(decoded_source_well_formed_spec);
    reveal(decoded_source_matches_input_spec);
}

fn error(kind: DecodeErrorKind, offset: usize) -> (error: DecodeError)
    ensures
        error.kind == kind,
        error.byte_offset == offset as u64,
{
    DecodeError::at(kind, offset as u64)
}

fn decode_one(input: &[u8], index: usize) -> (result: Result<(u32, usize), DecodeError>)
    requires
        index < input.len(),
    ensures
        match result {
            Ok((code_point, width)) => {
                decode_one_spec(input@, index as int) == Ok((code_point, width as int)) && 1
                    <= width && width <= 4 && index + width <= input.len()
                    && unicode_scalar_value_spec(code_point) && (code_point == 0x0d ==> width == 1)
            },
            Err(error) => {
                decode_one_spec(input@, index as int) == Err(
                    (error@.kind, error@.byte_offset as int),
                ) && index as u64 <= error@.byte_offset && error@.byte_offset <= input.len() as u64
            },
        },
{
    reveal(decode_one_spec);
    let first = input[index];
    if first <= 0x7f {
        return Ok((first as u32, 1));
    }
    if first <= 0xbf {
        return Err(error(DecodeErrorKind::UnexpectedContinuationByte, index));
    }
    if first <= 0xc1 {
        return Err(error(DecodeErrorKind::OverlongEncoding, index));
    }
    if first <= 0xdf {
        if input.len() - index < 2 {
            return Err(error(DecodeErrorKind::TruncatedSequence, input.len()));
        }
        let second = input[index + 1];
        if !continuation(second) {
            return Err(error(DecodeErrorKind::InvalidContinuationByte, index + 1));
        }
        assert(0xc2 <= first && first <= 0xdf);
        assert(0x80 <= second && second <= 0xbf);
        let code_point = ((first as u32 - 0xc0) * 64) + (second as u32 - 0x80);
        assert(0x80 <= code_point && code_point <= 0x7ff);
        return Ok((code_point, 2));
    }
    if first <= 0xef {
        if input.len() - index < 2 {
            return Err(error(DecodeErrorKind::TruncatedSequence, input.len()));
        }
        let second = input[index + 1];
        if !continuation(second) {
            return Err(error(DecodeErrorKind::InvalidContinuationByte, index + 1));
        }
        if first == 0xe0 && second < 0xa0 {
            return Err(error(DecodeErrorKind::OverlongEncoding, index));
        }
        if first == 0xed && second >= 0xa0 {
            return Err(error(DecodeErrorKind::SurrogateCodePoint, index));
        }
        if input.len() - index < 3 {
            return Err(error(DecodeErrorKind::TruncatedSequence, input.len()));
        }
        let third = input[index + 2];
        if !continuation(third) {
            return Err(error(DecodeErrorKind::InvalidContinuationByte, index + 2));
        }
        assert(0xe0 <= first && first <= 0xef);
        assert(0x80 <= second && second <= 0xbf);
        assert(0x80 <= third && third <= 0xbf);
        assert(first != 0xe0 || second >= 0xa0);
        assert(first != 0xed || second < 0xa0);
        let code_point = ((first as u32 - 0xe0) * 4096) + ((second as u32 - 0x80) * 64) + (
        third as u32 - 0x80);
        assert(0x800 <= code_point && code_point <= 0xffff);
        assert(!(0xd800 <= code_point && code_point <= 0xdfff));
        return Ok((code_point, 3));
    }
    if first <= 0xf4 {
        if input.len() - index < 2 {
            return Err(error(DecodeErrorKind::TruncatedSequence, input.len()));
        }
        let second = input[index + 1];
        if !continuation(second) {
            return Err(error(DecodeErrorKind::InvalidContinuationByte, index + 1));
        }
        if first == 0xf0 && second < 0x90 {
            return Err(error(DecodeErrorKind::OverlongEncoding, index));
        }
        if first == 0xf4 && second > 0x8f {
            return Err(error(DecodeErrorKind::CodePointOutOfRange, index));
        }
        if input.len() - index < 3 {
            return Err(error(DecodeErrorKind::TruncatedSequence, input.len()));
        }
        let third = input[index + 2];
        if !continuation(third) {
            return Err(error(DecodeErrorKind::InvalidContinuationByte, index + 2));
        }
        if input.len() - index < 4 {
            return Err(error(DecodeErrorKind::TruncatedSequence, input.len()));
        }
        let fourth = input[index + 3];
        if !continuation(fourth) {
            return Err(error(DecodeErrorKind::InvalidContinuationByte, index + 3));
        }
        assert(0xf0 <= first && first <= 0xf4);
        assert(0x80 <= second && second <= 0xbf);
        assert(0x80 <= third && third <= 0xbf);
        assert(0x80 <= fourth && fourth <= 0xbf);
        assert(first != 0xf0 || second >= 0x90);
        assert(first != 0xf4 || second <= 0x8f);
        let code_point = ((first as u32 - 0xf0) * 262_144) + ((second as u32 - 0x80) * 4096) + ((
        third as u32 - 0x80) * 64) + (fourth as u32 - 0x80);
        assert(0x10000 <= code_point && code_point <= 0x10ffff);
        return Ok((code_point, 4));
    }
    if first <= 0xf7 {
        return Err(error(DecodeErrorKind::CodePointOutOfRange, index));
    }
    Err(error(DecodeErrorKind::InvalidLeadingByte, index))
}

#[derive(Clone, Copy)]
struct DecodedStep {
    scalar: DecodedScalar,
    next_index: usize,
    next_line: u64,
    next_column: u64,
}

#[verifier::rlimit(100)]
#[verifier::spinoff_prover]
#[allow(clippy::question_mark)]
fn decode_next(input: &[u8], index: usize, line: u64, column: u64) -> (result: Result<
    DecodedStep,
    DecodeError,
>)
    requires
        index < input.len(),
        input.len() as u64 <= MAX_PROFILE1_SOURCE_BYTES,
        line <= index as u64,
        column <= index as u64,
    ensures
        match result {
            Ok(step) => {
                decode_next_spec(input@, index as int, line, column) == Ok(
                    (step.scalar, step.next_index as int, step.next_line, step.next_column),
                ) && index < step.next_index && step.next_index <= input.len() && step.next_line
                    <= step.next_index as u64 && step.next_column <= step.next_index as u64
                    && step.scalar.span.start == SourcePosition {
                    byte_offset: index as u64,
                    line,
                    column,
                } && step.scalar.span.end == SourcePosition {
                    byte_offset: step.next_index as u64,
                    line: step.next_line,
                    column: step.next_column,
                } && normalized_scalar_spec(step.scalar) && scalar_decodes_input_spec(
                    input@,
                    step.scalar,
                )
            },
            Err(error) => decode_next_spec(input@, index as int, line, column) == Err(error@),
        },
{
    let start = SourcePosition::new(index as u64, line, column);
    let (code_point, width) = match decode_one(input, index) {
        Ok(value) => value,
        Err(decode_error) => {
            proof {
                reveal(decode_next_spec);
            }
            return Err(decode_error);
        },
    };
    let mut consumed_width = width;
    let normalized = if code_point == 0x0d {
        if input.len() - index >= 2 && input[index + 1] == 0x0a {
            consumed_width = 2;
        }
        0x0a
    } else {
        code_point
    };
    assert(unicode_scalar_value_spec(normalized));
    assert(consumed_width >= 1);
    assert(index + consumed_width <= input.len());

    let next_index = index + consumed_width;
    let (next_line, next_column) = if normalized == 0x0a {
        (line + 1, 0)
    } else {
        (line, column + 1)
    };
    let end = SourcePosition::new(next_index as u64, next_line, next_column);
    let scalar = DecodedScalar::new(normalized, SourceSpan::new(start, end));
    assert(normalized_scalar_spec(scalar)) by {
        reveal(normalized_scalar_spec);
    }
    assert(if code_point == 0x0d {
        scalar.code_point == 0x0a && scalar.span.end.byte_offset as int
            - scalar.span.start.byte_offset as int == if scalar.span.start.byte_offset as int + 1
            < input@.len() && input@[scalar.span.start.byte_offset as int + 1] == 0x0a {
            2
        } else {
            width as int
        }
    } else {
        scalar.code_point == code_point && scalar.span.end.byte_offset as int
            - scalar.span.start.byte_offset as int == width as int
    });
    proof {
        lemma_scalar_decode_correspondence(input@, scalar, code_point, width as int);
        reveal(normalized_step_spec);
        reveal(decode_next_spec);
    }
    Ok(DecodedStep { scalar, next_index, next_line, next_column })
}

#[verifier::rlimit(100)]
#[verifier::spinoff_prover]
#[allow(clippy::question_mark)]
pub fn decode_profile1(input: &[u8], limits: DecodeLimits, bom_policy: BomPolicy) -> (result:
    Result<DecodedSource, DecodeError>)
    ensures
        decode_profile1_spec(input@, limits@, bom_policy) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(source) => {
                decoded_source_well_formed_spec(source@) && source@.source_len_bytes
                    == input.len() as u64 && source@.source_len_bytes <= limits@.max_source_bytes
                    && source@.source_len_bytes <= MAX_PROFILE1_SOURCE_BYTES
                    && source@.scalars.len() <= limits@.max_decoded_scalars && source@.scalars.len()
                    <= MAX_PROFILE1_DECODED_SCALARS && decoded_source_matches_input_spec(
                    input@,
                    source@,
                ) && match bom_policy {
                    BomPolicy::Forbid => source@.bom_bytes == 0,
                    BomPolicy::AllowAndStrip => true,
                }
            },
            Err(_) => true,
        },
{
    let source_len = input.len() as u64;
    let effective_source_limit = if limits.max_source_bytes < MAX_PROFILE1_SOURCE_BYTES {
        limits.max_source_bytes
    } else {
        MAX_PROFILE1_SOURCE_BYTES
    };
    if source_len > effective_source_limit {
        let source_error = DecodeError::at(
            DecodeErrorKind::SourceByteLimitExceeded,
            effective_source_limit,
        );
        proof {
            reveal(decode_profile1_spec);
        }
        return Err(source_error);
    }
    let effective_scalar_limit = if limits.max_decoded_scalars < MAX_PROFILE1_DECODED_SCALARS {
        limits.max_decoded_scalars
    } else {
        MAX_PROFILE1_DECODED_SCALARS
    };
    let has_bom = input.len() >= 3 && input[0] == 0xef && input[1] == 0xbb && input[2] == 0xbf;
    if has_bom {
        match bom_policy {
            BomPolicy::Forbid => {
                let bom_error = DecodeError::at(DecodeErrorKind::ForbiddenByteOrderMark, 0);
                proof {
                    reveal(decode_profile1_spec);
                }
                return Err(bom_error);
            },
            BomPolicy::AllowAndStrip => {},
        }
    }
    let mut index: usize = if has_bom {
        3
    } else {
        0
    };
    let bom_bytes: u64 = index as u64;
    assert((bom_bytes == 3) == has_bom);
    assert(has_bom == (input@.len() >= 3 && input@[0] == 0xef && input@[1] == 0xbb && input@[2]
        == 0xbf));
    let mut line: u64 = 0;
    let mut column: u64 = 0;
    let mut scalars: Vec<DecodedScalar> = Vec::new();
    proof {
        lemma_empty_prefix(bom_bytes);
        reveal(decode_profile1_spec);
    }
    assert(decoded_scalars_match_input_spec(input@, scalars@));
    assert(finish_decode_spec(
        source_len,
        bom_bytes,
        decode_loop_spec(
            input@,
            index as int,
            line,
            column,
            effective_scalar_limit,
            scalars@,
            input@.len() - bom_bytes as int,
        ),
    ) == decode_profile1_spec(input@, limits@, bom_policy));

    while index < input.len()
        invariant
            index <= input.len(),
            source_len == input.len() as u64,
            index as u64 <= source_len,
            source_len <= effective_source_limit,
            effective_source_limit <= MAX_PROFILE1_SOURCE_BYTES,
            effective_source_limit <= limits@.max_source_bytes,
            effective_scalar_limit <= MAX_PROFILE1_DECODED_SCALARS,
            effective_scalar_limit <= limits@.max_decoded_scalars,
            bom_bytes == 0 || bom_bytes == 3,
            bom_bytes <= index as u64,
            scalars@.len() <= index as int - bom_bytes as int,
            line <= index as u64,
            column <= index as u64,
            scalars@.len() <= effective_scalar_limit,
            scalars@.len() <= MAX_PROFILE1_DECODED_SCALARS,
            decoded_prefix_well_formed_spec(scalars@, bom_bytes, index as u64),
            decoded_scalars_match_input_spec(input@, scalars@),
            finish_decode_spec(
                source_len,
                bom_bytes,
                decode_loop_spec(
                    input@,
                    index as int,
                    line,
                    column,
                    (effective_scalar_limit - scalars@.len()) as u64,
                    scalars@,
                    input@.len() - bom_bytes as int - scalars@.len(),
                ),
            ) == decode_profile1_spec(input@, limits@, bom_policy),
            if scalars@.len() == 0 {
                line == 0 && column == 0
            } else {
                scalars@[scalars@.len() - 1].span.end == SourcePosition {
                    byte_offset: index as u64,
                    line,
                    column,
                }
            },
        decreases input.len() - index,
    {
        let ghost proof_fuel = input@.len() - bom_bytes as int - scalars@.len();
        let ghost remaining_scalars: u64 = (effective_scalar_limit - scalars@.len()) as u64;
        assert(0 < proof_fuel);
        if scalars.len() as u64 >= effective_scalar_limit {
            let scalar_limit_error = DecodeError::at(
                DecodeErrorKind::DecodedScalarLimitExceeded,
                index as u64,
            );
            proof {
                lemma_decode_loop_limit(input@, index as int, line, column, scalars@, proof_fuel);
                reveal(finish_decode_spec);
            }
            assert(decode_profile1_spec(input@, limits@, bom_policy) == Err(scalar_limit_error@));
            return Err(scalar_limit_error);
        }
        let step = match decode_next(input, index, line, column) {
            Ok(step) => step,
            Err(decode_error) => {
                proof {
                    lemma_decode_loop_error(
                        input@,
                        index as int,
                        line,
                        column,
                        remaining_scalars,
                        scalars@,
                        proof_fuel,
                        decode_error@,
                    );
                    reveal(finish_decode_spec);
                }
                assert(decode_profile1_spec(input@, limits@, bom_policy) == Err(decode_error@));
                return Err(decode_error);
            },
        };
        let next_index = step.next_index;
        let next_line = step.next_line;
        let next_column = step.next_column;
        let scalar = step.scalar;
        let _start = scalar.span.start;
        assert(_start == if scalars@.len() == 0 {
            SourcePosition { byte_offset: bom_bytes, line: 0, column: 0 }
        } else {
            scalars@[scalars@.len() - 1].span.end
        });
        proof {
            lemma_extend_prefix(scalars@, bom_bytes, index as u64, scalar);
            lemma_matching_scalar_push(input@, scalars@, scalar);
            lemma_decode_loop_step(
                input@,
                index as int,
                line,
                column,
                remaining_scalars,
                scalars@,
                proof_fuel,
                scalar,
                next_index as int,
                next_line,
                next_column,
            );
        }
        scalars.push(scalar);
        index = next_index;
        line = next_line;
        column = next_column;
    }

    let ghost concrete_scalars = scalars@;
    let source = DecodedSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        transformation_version: UTF8_TRANSFORMATION_VERSION,
        source_len_bytes: source_len,
        bom_bytes,
        scalars,
    };
    proof {
        lemma_complete_decoded_source(input@, source@, concrete_scalars);
        lemma_decode_loop_done(
            input@,
            index as int,
            line,
            column,
            (effective_scalar_limit - concrete_scalars.len()) as u64,
            concrete_scalars,
            input@.len() - bom_bytes as int - concrete_scalars.len(),
        );
        reveal(finish_decode_spec);
    }
    assert(decode_profile1_spec(input@, limits@, bom_policy) == Ok(source@));
    assert(match bom_policy {
        BomPolicy::Forbid => source@.bom_bytes == 0,
        BomPolicy::AllowAndStrip => true,
    });
    Ok(source)
}

} // verus!
