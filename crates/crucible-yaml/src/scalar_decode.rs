//! Verified style-specific scalar content decoding for Crucible YAML profile 1.
//!
//! Block scalars are already normalized by the authenticated block-scalar machine.  This first
//! semantic decoding submachine copies that content into the shared decoded representation while
//! retaining exact atom and byte provenance.  Quoted and plain styles extend the same model.
use crate::block::{
    BlockScalarContentOrigin, BlockScalarContentScalar, BlockScalarSource, BlockScalarStyle,
};
#[allow(unused_imports)]
use crate::block::{BlockScalarContentScalarView, BlockScalarSourceView};
use vstd::prelude::*;

verus! {

pub const SCALAR_DECODE_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarDecodeLimits {
    max_content_code_points: u64,
}

#[verifier::ext_equal]
pub struct ScalarDecodeLimitsView {
    pub max_content_code_points: u64,
}

impl View for ScalarDecodeLimits {
    type V = ScalarDecodeLimitsView;

    closed spec fn view(&self) -> ScalarDecodeLimitsView {
        ScalarDecodeLimitsView { max_content_code_points: self.max_content_code_points }
    }
}

impl ScalarDecodeLimits {
    pub fn new(max_content_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (ScalarDecodeLimitsView { max_content_code_points }),
    {
        Self { max_content_code_points }
    }

    pub fn max_content_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_content_code_points,
    {
        self.max_content_code_points
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum DecodedScalarStyle {
    LiteralBlock,
    FoldedBlock,
    SingleQuoted,
    DoubleQuoted,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum DecodedContentOrigin {
    Direct,
    FoldedLineBreak,
    SingleQuoteDoubled,
    DoubleQuotedEscape,
    EscapedLineBreak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedContentScalar {
    code_point: u32,
    source_atom_start: u64,
    source_atom_end: u64,
    byte_start: u64,
    byte_end: u64,
    origin: DecodedContentOrigin,
}

#[verifier::ext_equal]
pub struct DecodedContentScalarView {
    pub code_point: u32,
    pub source_atom_start: u64,
    pub source_atom_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub origin: DecodedContentOrigin,
}

impl View for DecodedContentScalar {
    type V = DecodedContentScalarView;

    closed spec fn view(&self) -> DecodedContentScalarView {
        DecodedContentScalarView {
            code_point: self.code_point,
            source_atom_start: self.source_atom_start,
            source_atom_end: self.source_atom_end,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            origin: self.origin,
        }
    }
}

impl DecodedContentScalar {
    fn new(
        code_point: u32,
        source_atom_start: u64,
        source_atom_end: u64,
        byte_start: u64,
        byte_end: u64,
        origin: DecodedContentOrigin,
    ) -> (scalar: Self)
        ensures
            scalar@ == (DecodedContentScalarView {
                code_point,
                source_atom_start,
                source_atom_end,
                byte_start,
                byte_end,
                origin,
            }),
    {
        Self { code_point, source_atom_start, source_atom_end, byte_start, byte_end, origin }
    }

    pub fn code_point(&self) -> (code_point: u32)
        ensures
            code_point == self@.code_point,
    {
        self.code_point
    }

    pub fn source_atom_start(&self) -> (index: u64)
        ensures
            index == self@.source_atom_start,
    {
        self.source_atom_start
    }

    pub fn source_atom_end(&self) -> (index: u64)
        ensures
            index == self@.source_atom_end,
    {
        self.source_atom_end
    }

    pub fn byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.byte_start,
    {
        self.byte_start
    }

    pub fn byte_end(&self) -> (offset: u64)
        ensures
            offset == self@.byte_end,
    {
        self.byte_end
    }

    pub fn origin(&self) -> (origin: DecodedContentOrigin)
        ensures
            origin == self@.origin,
    {
        self.origin
    }
}

pub open spec fn decoded_content_scalar_views_spec(content: Seq<DecodedContentScalar>) -> Seq<
    DecodedContentScalarView,
> {
    Seq::new(content.len(), |index: int| content[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedScalarContent {
    style: DecodedScalarStyle,
    content: Vec<DecodedContentScalar>,
}

#[verifier::ext_equal]
pub struct DecodedScalarContentView {
    pub style: DecodedScalarStyle,
    pub content: Seq<DecodedContentScalarView>,
}

impl View for DecodedScalarContent {
    type V = DecodedScalarContentView;

    closed spec fn view(&self) -> DecodedScalarContentView {
        DecodedScalarContentView {
            style: self.style,
            content: decoded_content_scalar_views_spec(self.content@),
        }
    }
}

impl DecodedScalarContent {
    fn new(style: DecodedScalarStyle, content: Vec<DecodedContentScalar>) -> (decoded: Self)
        ensures
            decoded@ == (DecodedScalarContentView {
                style,
                content: decoded_content_scalar_views_spec(content@),
            }),
    {
        Self { style, content }
    }

    pub fn style(&self) -> (style: DecodedScalarStyle)
        ensures
            style == self@.style,
    {
        self.style
    }

    pub fn content(&self) -> (content: &[DecodedContentScalar])
        ensures
            decoded_content_scalar_views_spec(content@) == self@.content,
    {
        self.content.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum ScalarDecodeErrorKind {
    ScalarIndexOutOfRange,
    ContentLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarDecodeError {
    kind: ScalarDecodeErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct ScalarDecodeErrorView {
    pub kind: ScalarDecodeErrorKind,
    pub byte_offset: u64,
}

impl View for ScalarDecodeError {
    type V = ScalarDecodeErrorView;

    closed spec fn view(&self) -> ScalarDecodeErrorView {
        ScalarDecodeErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl ScalarDecodeError {
    fn at(kind: ScalarDecodeErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (ScalarDecodeErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: ScalarDecodeErrorKind)
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

pub open spec fn effective_scalar_content_limit_spec(limits: ScalarDecodeLimitsView) -> u64 {
    if limits.max_content_code_points < MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS {
        limits.max_content_code_points
    } else {
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS
    }
}

pub open spec fn decoded_block_style_spec(style: BlockScalarStyle) -> DecodedScalarStyle {
    match style {
        BlockScalarStyle::Literal => DecodedScalarStyle::LiteralBlock,
        BlockScalarStyle::Folded => DecodedScalarStyle::FoldedBlock,
    }
}

pub open spec fn decoded_block_origin_spec(
    origin: BlockScalarContentOrigin,
) -> DecodedContentOrigin {
    match origin {
        BlockScalarContentOrigin::Direct => DecodedContentOrigin::Direct,
        BlockScalarContentOrigin::FoldedLineBreak => DecodedContentOrigin::FoldedLineBreak,
    }
}

pub open spec fn decoded_block_content_item_spec(
    source: BlockScalarContentScalarView,
) -> DecodedContentScalarView {
    DecodedContentScalarView {
        code_point: source.code_point,
        source_atom_start: source.source_atom_index,
        source_atom_end: if source.source_atom_index == u64::MAX {
            u64::MAX
        } else {
            (source.source_atom_index + 1) as u64
        },
        byte_start: source.byte_start,
        byte_end: source.byte_end,
        origin: decoded_block_origin_spec(source.origin),
    }
}

pub open spec fn decoded_block_content_prefix_spec(
    source: Seq<BlockScalarContentScalarView>,
    end: nat,
) -> Seq<DecodedContentScalarView> {
    Seq::new(end, |index: int| decoded_block_content_item_spec(source[index]))
}

pub open spec fn decoded_block_content_spec(source: Seq<BlockScalarContentScalarView>) -> Seq<
    DecodedContentScalarView,
> {
    decoded_block_content_prefix_spec(source, source.len())
}

pub open spec fn decode_block_content_spec(
    style: BlockScalarStyle,
    source: Seq<BlockScalarContentScalarView>,
    limits: ScalarDecodeLimitsView,
) -> Result<DecodedScalarContentView, ScalarDecodeErrorView> {
    let effective_limit = effective_scalar_content_limit_spec(limits);
    if source.len() > effective_limit {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::ContentLimitExceeded,
                byte_offset: source[effective_limit as int].byte_start,
            },
        )
    } else {
        Ok(
            DecodedScalarContentView {
                style: decoded_block_style_spec(style),
                content: decoded_block_content_spec(source),
            },
        )
    }
}

pub open spec fn decode_profile1_block_scalar_content_spec(
    source: BlockScalarSourceView,
    scalar_index: u64,
    limits: ScalarDecodeLimitsView,
) -> Result<DecodedScalarContentView, ScalarDecodeErrorView> {
    if scalar_index >= source.scalars.len() {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::ScalarIndexOutOfRange,
                byte_offset: source.source_len_bytes,
            },
        )
    } else {
        let scalar = source.scalars[scalar_index as int];
        decode_block_content_spec(scalar.style, scalar.content, limits)
    }
}

proof fn lemma_decoded_content_views_push(
    content: Seq<DecodedContentScalar>,
    scalar: DecodedContentScalar,
)
    ensures
        decoded_content_scalar_views_spec(content.push(scalar))
            == decoded_content_scalar_views_spec(content).push(scalar@),
{
    reveal(decoded_content_scalar_views_spec);
    assert(decoded_content_scalar_views_spec(content.push(scalar))
        =~= decoded_content_scalar_views_spec(content).push(scalar@));
}

fn copy_block_content(source: &[BlockScalarContentScalar]) -> (decoded: Vec<DecodedContentScalar>)
    ensures
        decoded_content_scalar_views_spec(decoded@) == decoded_block_content_spec(
            crate::block::block_content_views_spec(source@),
        ),
{
    let ghost source_views = crate::block::block_content_views_spec(source@);
    let mut decoded = Vec::new();
    let mut index = 0usize;
    while index < source.len()
        invariant
            index <= source@.len(),
            source_views == crate::block::block_content_views_spec(source@),
            decoded_content_scalar_views_spec(decoded@) == decoded_block_content_prefix_spec(
                source_views,
                index as nat,
            ),
        decreases source.len() - index,
    {
        assert(source_views[index as int] == source[index as int]@) by {
            reveal(crate::block::block_content_views_spec);
        }
        let item = &source[index];
        let origin = match item.origin() {
            BlockScalarContentOrigin::Direct => DecodedContentOrigin::Direct,
            BlockScalarContentOrigin::FoldedLineBreak => DecodedContentOrigin::FoldedLineBreak,
        };
        let source_atom_start = item.source_atom_index();
        let source_atom_end = if source_atom_start == u64::MAX {
            u64::MAX
        } else {
            source_atom_start + 1
        };
        let copied = DecodedContentScalar::new(
            item.code_point(),
            source_atom_start,
            source_atom_end,
            item.byte_start(),
            item.byte_end(),
            origin,
        );
        proof {
            reveal(decoded_block_content_prefix_spec);
            reveal(decoded_block_content_item_spec);
            reveal(decoded_block_origin_spec);
            lemma_decoded_content_views_push(decoded@, copied);
        }
        decoded.push(copied);
        index += 1;
    }
    proof {
        reveal(decoded_block_content_spec);
    }
    decoded
}

/// Copy one authenticated block scalar's normalized content into the shared semantic form.
pub fn decode_profile1_block_scalar_content(
    source: &BlockScalarSource,
    scalar_index: u64,
    limits: ScalarDecodeLimits,
) -> (result: Result<DecodedScalarContent, ScalarDecodeError>)
    ensures
        decode_profile1_block_scalar_content_spec(source@, scalar_index, limits@) == match result {
            Ok(content) => Ok(content@),
            Err(error) => Err(error@),
        },
{
    let scalars = source.scalars();
    if scalar_index >= scalars.len() as u64 {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ScalarIndexOutOfRange,
            source.source_len_bytes(),
        );
        proof {
            reveal(decode_profile1_block_scalar_content_spec);
            reveal(crate::block::block_scalar_views_spec);
        }
        return Err(error);
    }
    let index = scalar_index as usize;
    assert(index < scalars.len());
    assert(source@.scalars[index as int] == scalars[index as int]@) by {
        reveal(crate::block::block_scalar_views_spec);
    }
    let scalar = &scalars[index];
    let source_content = scalar.content();
    let effective_limit = if limits.max_content_code_points
        < MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS {
        limits.max_content_code_points
    } else {
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS
    };
    if source_content.len() as u64 > effective_limit {
        let excluded = &source_content[effective_limit as usize];
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ContentLimitExceeded,
            excluded.byte_start(),
        );
        proof {
            reveal(decode_profile1_block_scalar_content_spec);
            reveal(decode_block_content_spec);
            reveal(effective_scalar_content_limit_spec);
            reveal(crate::block::block_content_views_spec);
        }
        return Err(error);
    }
    let decoded = copy_block_content(source_content);
    let style = match scalar.style() {
        BlockScalarStyle::Literal => DecodedScalarStyle::LiteralBlock,
        BlockScalarStyle::Folded => DecodedScalarStyle::FoldedBlock,
    };
    let result = DecodedScalarContent::new(style, decoded);
    proof {
        reveal(decode_profile1_block_scalar_content_spec);
        reveal(decode_block_content_spec);
        reveal(effective_scalar_content_limit_spec);
        reveal(decoded_block_style_spec);
    }
    Ok(result)
}

} // verus!
