//! Verified block-scalar token formation and presentation-to-content transformation.
//!
//! This final scalar-boundary slice authenticates every preceding lexer transformation, parses
//! literal and folded headers, detects content indentation, applies YAML 1.2.2 folding/chomping,
//! and retains exact source provenance for every normalized content code point.
use crate::atom::{AtomizedSource, LexicalAtom, LexicalAtomKind, MAX_PROFILE1_LEXICAL_ATOMS};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::layout::{analyze_profile1_layout, LayoutLine, LayoutSource};
#[allow(unused_imports)]
use crate::layout::{LayoutLineView, LayoutSourceView};
use crate::plain::{canonical_plain_scalar_limits, scan_profile1_plain_scalars, PlainScalarSource};
#[allow(unused_imports)]
use crate::plain::{PlainScalarSourceView, PlainScalarView};
use crate::quoted::{
    canonical_quoted_scalar_limits, scan_profile1_quoted_scalars, QuotedScalarSource,
};
#[allow(unused_imports)]
use crate::quoted::{QuotedScalarSourceView, QuotedScalarView};
use crate::structural::{
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    scan_profile1_structural_lexemes, StructuralCandidateRole, StructuralLexeme,
    StructuralLexemeSource,
};
#[allow(unused_imports)]
use crate::structural::{StructuralLexemeSourceView, StructuralLexemeView};
use crate::utf8::CRUCIBLE_YAML_PROFILE_VERSION;
use crate::YamlIndicator;
use vstd::prelude::*;

verus! {

pub const BLOCK_SCALAR_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_BLOCK_SCALARS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockScalarScanLimits {
    max_scalars: u64,
    max_scalar_presentation_atoms: u64,
    max_scalar_content_code_points: u64,
    max_total_content_code_points: u64,
}

#[verifier::ext_equal]
pub struct BlockScalarScanLimitsView {
    pub max_scalars: u64,
    pub max_scalar_presentation_atoms: u64,
    pub max_scalar_content_code_points: u64,
    pub max_total_content_code_points: u64,
}

impl View for BlockScalarScanLimits {
    type V = BlockScalarScanLimitsView;

    closed spec fn view(&self) -> BlockScalarScanLimitsView {
        BlockScalarScanLimitsView {
            max_scalars: self.max_scalars,
            max_scalar_presentation_atoms: self.max_scalar_presentation_atoms,
            max_scalar_content_code_points: self.max_scalar_content_code_points,
            max_total_content_code_points: self.max_total_content_code_points,
        }
    }
}

impl BlockScalarScanLimits {
    pub fn new(
        max_scalars: u64,
        max_scalar_presentation_atoms: u64,
        max_scalar_content_code_points: u64,
        max_total_content_code_points: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (BlockScalarScanLimitsView {
                max_scalars,
                max_scalar_presentation_atoms,
                max_scalar_content_code_points,
                max_total_content_code_points,
            }),
    {
        Self {
            max_scalars,
            max_scalar_presentation_atoms,
            max_scalar_content_code_points,
            max_total_content_code_points,
        }
    }

    pub fn max_scalars(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_scalars,
    {
        self.max_scalars
    }

    pub fn max_scalar_presentation_atoms(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_scalar_presentation_atoms,
    {
        self.max_scalar_presentation_atoms
    }

    pub fn max_scalar_content_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_scalar_content_code_points,
    {
        self.max_scalar_content_code_points
    }

    pub fn max_total_content_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_total_content_code_points,
    {
        self.max_total_content_code_points
    }
}

pub open spec fn canonical_block_scalar_limits_spec() -> BlockScalarScanLimitsView {
    BlockScalarScanLimitsView {
        max_scalars: MAX_PROFILE1_BLOCK_SCALARS,
        max_scalar_presentation_atoms: MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
        max_scalar_content_code_points: MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS,
        max_total_content_code_points: MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
    }
}

pub fn canonical_block_scalar_limits() -> (limits: BlockScalarScanLimits)
    ensures
        limits@ == canonical_block_scalar_limits_spec(),
{
    BlockScalarScanLimits::new(
        MAX_PROFILE1_BLOCK_SCALARS,
        MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
        MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS,
        MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum BlockScalarStyle {
    Literal,
    Folded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum BlockChomping {
    Strip,
    Clip,
    Keep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum BlockScalarContentOrigin {
    Direct,
    FoldedLineBreak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum BlockScalarErrorKind {
    InputLayoutMismatch,
    InputStructuralMismatch,
    InputQuotedMismatch,
    InputPlainMismatch,
    ScalarLimitExceeded,
    PresentationAtomLimitExceeded,
    ScalarContentLimitExceeded,
    TotalContentLimitExceeded,
    InvalidBlockHeader,
    MissingBlockHeaderLineBreak,
    InvalidIndentationIndicator,
    InvalidLeadingEmptyIndentation,
    InvalidBlockIndentation,
    TabInIndentation,
    InvalidBlockCharacter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockScalarError {
    kind: BlockScalarErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct BlockScalarErrorView {
    pub kind: BlockScalarErrorKind,
    pub byte_offset: u64,
}

impl View for BlockScalarError {
    type V = BlockScalarErrorView;

    closed spec fn view(&self) -> BlockScalarErrorView {
        BlockScalarErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl BlockScalarError {
    fn at(kind: BlockScalarErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (BlockScalarErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: BlockScalarErrorKind)
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
/// One normalized block-scalar content code point with exact source provenance.
///
/// ```compile_fail
/// use crucible_yaml::{BlockScalarContentOrigin, BlockScalarContentScalar};
///
/// let forged = BlockScalarContentScalar {
///     code_point: 0x20,
///     source_atom_index: 99,
///     byte_start: 9,
///     byte_end: 1,
///     origin: BlockScalarContentOrigin::FoldedLineBreak,
/// };
/// ```
pub struct BlockScalarContentScalar {
    code_point: u32,
    source_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
    origin: BlockScalarContentOrigin,
}

#[verifier::ext_equal]
pub struct BlockScalarContentScalarView {
    pub code_point: u32,
    pub source_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub origin: BlockScalarContentOrigin,
}

impl View for BlockScalarContentScalar {
    type V = BlockScalarContentScalarView;

    closed spec fn view(&self) -> BlockScalarContentScalarView {
        BlockScalarContentScalarView {
            code_point: self.code_point,
            source_atom_index: self.source_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            origin: self.origin,
        }
    }
}

impl DeepView for BlockScalarContentScalar {
    type V = BlockScalarContentScalarView;

    closed spec fn deep_view(&self) -> BlockScalarContentScalarView {
        self@
    }
}

impl BlockScalarContentScalar {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        self.code_point == other.code_point && self.source_atom_index == other.source_atom_index
            && self.byte_start == other.byte_start && self.byte_end == other.byte_end && self.origin
            == other.origin
    }

    fn new(
        code_point: u32,
        source_atom_index: u64,
        byte_start: u64,
        byte_end: u64,
        origin: BlockScalarContentOrigin,
    ) -> (scalar: Self)
        ensures
            scalar@ == (BlockScalarContentScalarView {
                code_point,
                source_atom_index,
                byte_start,
                byte_end,
                origin,
            }),
    {
        Self { code_point, source_atom_index, byte_start, byte_end, origin }
    }

    pub fn code_point(&self) -> (code_point: u32)
        ensures
            code_point == self@.code_point,
    {
        self.code_point
    }

    pub fn source_atom_index(&self) -> (index: u64)
        ensures
            index == self@.source_atom_index,
    {
        self.source_atom_index
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

    pub fn origin(&self) -> (origin: BlockScalarContentOrigin)
        ensures
            origin == self@.origin,
    {
        self.origin
    }
}

pub open spec fn block_content_views_spec(content: Seq<BlockScalarContentScalar>) -> Seq<
    BlockScalarContentScalarView,
> {
    Seq::new(content.len(), |index: int| content[index]@)
}

proof fn lemma_block_content_views_push(
    content: Seq<BlockScalarContentScalar>,
    scalar: BlockScalarContentScalar,
)
    ensures
        block_content_views_spec(content.push(scalar)) == block_content_views_spec(content).push(
            scalar@,
        ),
{
    reveal(block_content_views_spec);
    assert(block_content_views_spec(content.push(scalar)) =~= block_content_views_spec(
        content,
    ).push(scalar@));
}

#[derive(Debug, PartialEq, Eq)]
/// One complete block scalar from its `|` or `>` indicator through its presentation boundary.
///
/// ```compile_fail
/// use crucible_yaml::{BlockChomping, BlockScalar, BlockScalarStyle};
///
/// let forged = BlockScalar {
///     style: BlockScalarStyle::Literal,
///     chomping: BlockChomping::Clip,
///     explicit_indentation: None,
///     parent_indentation: 0,
///     content_indentation: 1,
///     start_line_number: 2,
///     end_line_number: 1,
///     start_atom_index: 9,
///     header_end_atom_index: 3,
///     content_start_atom_index: 3,
///     end_atom_index: 2,
///     byte_start: 9,
///     byte_end: 2,
///     content: Vec::new(),
/// };
/// ```
pub struct BlockScalar {
    style: BlockScalarStyle,
    chomping: BlockChomping,
    explicit_indentation: Option<u8>,
    parent_indentation: u64,
    content_indentation: u64,
    start_line_number: u64,
    end_line_number: u64,
    start_atom_index: u64,
    header_end_atom_index: u64,
    content_start_atom_index: u64,
    end_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
    content: Vec<BlockScalarContentScalar>,
}

#[verifier::ext_equal]
pub struct BlockScalarView {
    pub style: BlockScalarStyle,
    pub chomping: BlockChomping,
    pub explicit_indentation: Option<u8>,
    pub parent_indentation: u64,
    pub content_indentation: u64,
    pub start_line_number: u64,
    pub end_line_number: u64,
    pub start_atom_index: u64,
    pub header_end_atom_index: u64,
    pub content_start_atom_index: u64,
    pub end_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub content: Seq<BlockScalarContentScalarView>,
}

impl View for BlockScalar {
    type V = BlockScalarView;

    closed spec fn view(&self) -> BlockScalarView {
        BlockScalarView {
            style: self.style,
            chomping: self.chomping,
            explicit_indentation: self.explicit_indentation,
            parent_indentation: self.parent_indentation,
            content_indentation: self.content_indentation,
            start_line_number: self.start_line_number,
            end_line_number: self.end_line_number,
            start_atom_index: self.start_atom_index,
            header_end_atom_index: self.header_end_atom_index,
            content_start_atom_index: self.content_start_atom_index,
            end_atom_index: self.end_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            content: block_content_views_spec(self.content@),
        }
    }
}

impl BlockScalar {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.style != other.style || self.chomping != other.chomping || self.explicit_indentation
            != other.explicit_indentation || self.parent_indentation != other.parent_indentation
            || self.content_indentation != other.content_indentation || self.start_line_number
            != other.start_line_number || self.end_line_number != other.end_line_number
            || self.start_atom_index != other.start_atom_index || self.header_end_atom_index
            != other.header_end_atom_index || self.content_start_atom_index
            != other.content_start_atom_index || self.end_atom_index != other.end_atom_index
            || self.byte_start != other.byte_start || self.byte_end != other.byte_end {
            assert(self@ != other@);
            return false;
        }
        if self.content.len() != other.content.len() {
            proof {
                reveal(block_content_views_spec);
                assert(self@.content.len() != other@.content.len());
                assert(self@ != other@);
            }
            return false;
        }
        let mut index = 0usize;
        while index < self.content.len()
            invariant
                self.content.len() == other.content.len(),
                index <= self.content.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < index ==> self.content[prior]@ == other.content[prior]@,
            decreases self.content.len() - index,
        {
            if !self.content[index].same_as(&other.content[index]) {
                proof {
                    reveal(block_content_views_spec);
                    assert(self.content[index as int]@ != other.content[index as int]@);
                    assert(self@.content[index as int] != other@.content[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(block_content_views_spec);
            assert(self@.content =~= other@.content);
        }
        true
    }

    pub fn style(&self) -> (style: BlockScalarStyle)
        ensures
            style == self@.style,
    {
        self.style
    }

    pub fn chomping(&self) -> (chomping: BlockChomping)
        ensures
            chomping == self@.chomping,
    {
        self.chomping
    }

    pub fn explicit_indentation(&self) -> (indentation: Option<u8>)
        ensures
            indentation == self@.explicit_indentation,
    {
        self.explicit_indentation
    }

    pub fn parent_indentation(&self) -> (indentation: u64)
        ensures
            indentation == self@.parent_indentation,
    {
        self.parent_indentation
    }

    pub fn content_indentation(&self) -> (indentation: u64)
        ensures
            indentation == self@.content_indentation,
    {
        self.content_indentation
    }

    pub fn start_line_number(&self) -> (line: u64)
        ensures
            line == self@.start_line_number,
    {
        self.start_line_number
    }

    pub fn end_line_number(&self) -> (line: u64)
        ensures
            line == self@.end_line_number,
    {
        self.end_line_number
    }

    pub fn start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.start_atom_index,
    {
        self.start_atom_index
    }

    pub fn header_end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.header_end_atom_index,
    {
        self.header_end_atom_index
    }

    pub fn content_start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.content_start_atom_index,
    {
        self.content_start_atom_index
    }

    pub fn end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.end_atom_index,
    {
        self.end_atom_index
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

    pub fn content(&self) -> (content: &[BlockScalarContentScalar])
        ensures
            block_content_views_spec(content@) == self@.content,
    {
        self.content.as_slice()
    }
}

pub open spec fn block_scalar_views_spec(scalars: Seq<BlockScalar>) -> Seq<BlockScalarView> {
    Seq::new(scalars.len(), |index: int| scalars[index]@)
}

proof fn lemma_block_scalar_views_push(scalars: Seq<BlockScalar>, scalar: BlockScalar)
    ensures
        block_scalar_views_spec(scalars.push(scalar)) == block_scalar_views_spec(scalars).push(
            scalar@,
        ),
{
    reveal(block_scalar_views_spec);
    assert(block_scalar_views_spec(scalars.push(scalar)) =~= block_scalar_views_spec(scalars).push(
        scalar@,
    ));
}

#[derive(Debug, PartialEq, Eq)]
pub struct BlockScalarSource {
    profile_version: u16,
    input_transformation_version: u16,
    layout_transformation_version: u16,
    structural_transformation_version: u16,
    quoted_transformation_version: u16,
    plain_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    input_atom_count: u64,
    input_line_count: u64,
    input_structural_lexeme_count: u64,
    input_quoted_scalar_count: u64,
    input_plain_scalar_count: u64,
    total_content_code_points: u64,
    scalars: Vec<BlockScalar>,
}

#[verifier::ext_equal]
pub struct BlockScalarSourceView {
    pub profile_version: u16,
    pub input_transformation_version: u16,
    pub layout_transformation_version: u16,
    pub structural_transformation_version: u16,
    pub quoted_transformation_version: u16,
    pub plain_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub input_atom_count: u64,
    pub input_line_count: u64,
    pub input_structural_lexeme_count: u64,
    pub input_quoted_scalar_count: u64,
    pub input_plain_scalar_count: u64,
    pub total_content_code_points: u64,
    pub scalars: Seq<BlockScalarView>,
}

impl View for BlockScalarSource {
    type V = BlockScalarSourceView;

    closed spec fn view(&self) -> BlockScalarSourceView {
        BlockScalarSourceView {
            profile_version: self.profile_version,
            input_transformation_version: self.input_transformation_version,
            layout_transformation_version: self.layout_transformation_version,
            structural_transformation_version: self.structural_transformation_version,
            quoted_transformation_version: self.quoted_transformation_version,
            plain_transformation_version: self.plain_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            input_atom_count: self.input_atom_count,
            input_line_count: self.input_line_count,
            input_structural_lexeme_count: self.input_structural_lexeme_count,
            input_quoted_scalar_count: self.input_quoted_scalar_count,
            input_plain_scalar_count: self.input_plain_scalar_count,
            total_content_code_points: self.total_content_code_points,
            scalars: block_scalar_views_spec(self.scalars@),
        }
    }
}

impl BlockScalarSource {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.profile_version != other.profile_version || self.input_transformation_version
            != other.input_transformation_version || self.layout_transformation_version
            != other.layout_transformation_version || self.structural_transformation_version
            != other.structural_transformation_version || self.quoted_transformation_version
            != other.quoted_transformation_version || self.plain_transformation_version
            != other.plain_transformation_version || self.transformation_version
            != other.transformation_version || self.source_len_bytes != other.source_len_bytes
            || self.bom_bytes != other.bom_bytes || self.input_atom_count != other.input_atom_count
            || self.input_line_count != other.input_line_count || self.input_structural_lexeme_count
            != other.input_structural_lexeme_count || self.input_quoted_scalar_count
            != other.input_quoted_scalar_count || self.input_plain_scalar_count
            != other.input_plain_scalar_count || self.total_content_code_points
            != other.total_content_code_points {
            assert(self@ != other@);
            return false;
        }
        if self.scalars.len() != other.scalars.len() {
            proof {
                reveal(block_scalar_views_spec);
                assert(self@.scalars.len() != other@.scalars.len());
                assert(self@ != other@);
            }
            return false;
        }
        let mut index = 0usize;
        while index < self.scalars.len()
            invariant
                self.scalars.len() == other.scalars.len(),
                index <= self.scalars.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < index ==> self.scalars[prior]@ == other.scalars[prior]@,
            decreases self.scalars.len() - index,
        {
            if !self.scalars[index].same_as(&other.scalars[index]) {
                proof {
                    reveal(block_scalar_views_spec);
                    assert(self.scalars[index as int]@ != other.scalars[index as int]@);
                    assert(self@.scalars[index as int] != other@.scalars[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(block_scalar_views_spec);
            assert(self@.scalars =~= other@.scalars);
        }
        true
    }

    pub fn profile_version(&self) -> (version: u16)
        ensures
            version == self@.profile_version,
    {
        self.profile_version
    }

    pub fn input_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.input_transformation_version,
    {
        self.input_transformation_version
    }

    pub fn layout_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.layout_transformation_version,
    {
        self.layout_transformation_version
    }

    pub fn structural_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.structural_transformation_version,
    {
        self.structural_transformation_version
    }

    pub fn quoted_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.quoted_transformation_version,
    {
        self.quoted_transformation_version
    }

    pub fn plain_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.plain_transformation_version,
    {
        self.plain_transformation_version
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

    pub fn input_atom_count(&self) -> (count: u64)
        ensures
            count == self@.input_atom_count,
    {
        self.input_atom_count
    }

    pub fn input_line_count(&self) -> (count: u64)
        ensures
            count == self@.input_line_count,
    {
        self.input_line_count
    }

    pub fn input_structural_lexeme_count(&self) -> (count: u64)
        ensures
            count == self@.input_structural_lexeme_count,
    {
        self.input_structural_lexeme_count
    }

    pub fn input_quoted_scalar_count(&self) -> (count: u64)
        ensures
            count == self@.input_quoted_scalar_count,
    {
        self.input_quoted_scalar_count
    }

    pub fn input_plain_scalar_count(&self) -> (count: u64)
        ensures
            count == self@.input_plain_scalar_count,
    {
        self.input_plain_scalar_count
    }

    pub fn total_content_code_points(&self) -> (count: u64)
        ensures
            count == self@.total_content_code_points,
    {
        self.total_content_code_points
    }

    pub fn scalars(&self) -> (scalars: &[BlockScalar])
        ensures
            block_scalar_views_spec(scalars@) == self@.scalars,
    {
        self.scalars.as_slice()
    }
}

pub open spec fn content_provenance_code_point_spec(
    source_code_point: u32,
    content: BlockScalarContentScalarView,
) -> bool {
    content.origin == BlockScalarContentOrigin::Direct && content.code_point == source_code_point
        || content.origin == BlockScalarContentOrigin::FoldedLineBreak && source_code_point == 0x0a
        && content.code_point == 0x20
}

pub open spec fn block_content_scalar_provenance_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    scalar: BlockScalarView,
    index: int,
) -> bool {
    0 <= index < scalar.content.len() && scalar.content[index].source_atom_index < atoms.len()
        && scalar.content_start_atom_index <= scalar.content[index].source_atom_index
        < scalar.end_atom_index && scalar.content[index].byte_start
        == atoms[scalar.content[index].source_atom_index as int].span.start.byte_offset
        && scalar.content[index].byte_end
        == atoms[scalar.content[index].source_atom_index as int].span.end.byte_offset
        && content_provenance_code_point_spec(
        atoms[scalar.content[index].source_atom_index as int].code_point,
        scalar.content[index],
    ) && (scalar.content[index].origin == BlockScalarContentOrigin::Direct || scalar.style
        == BlockScalarStyle::Folded) && (index == 0 || scalar.content[index - 1].source_atom_index
        <= scalar.content[index].source_atom_index) && exists|line_index: int|
        scalar.start_line_number + 1 <= line_index < lines.len() && (logical_content_start_spec(
            lines[line_index],
            scalar.content_indentation,
        ) <= scalar.content[index].source_atom_index < lines[line_index].end_atom_index
            || lines[line_index].terminated && scalar.content[index].source_atom_index
            == lines[line_index].end_atom_index)
}

pub open spec fn block_scalar_range_and_content_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    scalar: BlockScalarView,
) -> bool {
    scalar.start_atom_index < scalar.header_end_atom_index && scalar.header_end_atom_index
        == scalar.content_start_atom_index && scalar.content_start_atom_index
        <= scalar.end_atom_index && scalar.end_atom_index <= atoms.len()
        && scalar.content_indentation > scalar.parent_indentation && scalar.byte_start
        == atoms[scalar.start_atom_index as int].span.start.byte_offset && scalar.byte_end
        == atoms[(scalar.end_atom_index - 1) as int].span.end.byte_offset && scalar.end_line_number
        == atoms[(scalar.end_atom_index - 1) as int].span.start.line
        && atoms[scalar.start_atom_index as int].kind == if scalar.style
        == BlockScalarStyle::Literal {
        LexicalAtomKind::Indicator(YamlIndicator::LiteralBlockScalar)
    } else {
        LexicalAtomKind::Indicator(YamlIndicator::FoldedBlockScalar)
    } && exists|end_line: int, last_nonempty: Option<int>| #[trigger]
        render_block_content_spec(
            atoms,
            lines,
            scalar.start_line_number as int + 1,
            end_line,
            scalar.content_indentation,
            scalar.style,
            scalar.chomping,
            last_nonempty,
        ) == Ok(scalar.content) && scalar.start_line_number + 1 <= end_line <= lines.len()
            && scalar.end_atom_index == if end_line < lines.len() {
            lines[end_line].start_atom_index
        } else {
            atoms.len() as u64
        }
}

pub open spec fn block_scalar_sequence_ranges_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    scalars: Seq<BlockScalarView>,
) -> bool {
    forall|index: int|
        #![trigger scalars[index]]
        0 <= index < scalars.len() ==> block_scalar_range_and_content_spec(
            atoms,
            lines,
            scalars[index],
        ) && (index == 0 || scalars[index - 1].end_atom_index <= scalars[index].start_atom_index
            && scalars[index - 1].byte_end <= scalars[index].byte_start)
}

pub open spec fn block_scalar_ranges_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    blocks: BlockScalarSourceView,
) -> bool {
    blocks.input_atom_count == atomized.atoms.len() && block_scalar_sequence_ranges_spec(
        atomized.atoms,
        layout.lines,
        blocks.scalars,
    )
}

proof fn lemma_block_scalar_sequence_push(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    scalars: Seq<BlockScalarView>,
    scalar: BlockScalarView,
)
    requires
        block_scalar_sequence_ranges_spec(atoms, lines, scalars),
        block_scalar_range_and_content_spec(atoms, lines, scalar),
        scalars.len() > 0 ==> scalars.last().end_atom_index <= scalar.start_atom_index
            && scalars.last().byte_end <= scalar.byte_start,
    ensures
        block_scalar_sequence_ranges_spec(atoms, lines, scalars.push(scalar)),
{
    reveal(block_scalar_sequence_ranges_spec);
    assert forall|index: int|
        #![trigger scalars.push(scalar)[index]]
        0 <= index < scalars.push(scalar).len() implies block_scalar_range_and_content_spec(
        atoms,
        lines,
        scalars.push(scalar)[index],
    ) && (index == 0 || scalars.push(scalar)[index - 1].end_atom_index <= scalars.push(
        scalar,
    )[index].start_atom_index && scalars.push(scalar)[index - 1].byte_end <= scalars.push(
        scalar,
    )[index].byte_start) by {
        if index < scalars.len() {
            assert(scalars.push(scalar)[index] == scalars[index]);
        } else {
            assert(index == scalars.len());
            assert(scalars.push(scalar)[index] == scalar);
            if index > 0 {
                assert(scalars.push(scalar)[index - 1] == scalars.last());
            }
        }
    }
}

proof fn lemma_earlier_block_atom_ends_before_later_atom_starts(
    atomized: AtomizedSourceView,
    earlier: int,
    later: int,
)
    requires
        crate::atom::atomized_source_intrinsically_well_formed_spec(atomized),
        0 <= earlier < later < atomized.atoms.len(),
    ensures
        atomized.atoms[earlier].span.end.byte_offset
            <= atomized.atoms[later].span.start.byte_offset,
    decreases later - earlier,
{
    crate::atom::lemma_intrinsic_atomized_spans_partition_source(atomized);
    if earlier + 1 < later {
        lemma_earlier_block_atom_ends_before_later_atom_starts(atomized, earlier, later - 1);
        crate::atom::lemma_intrinsic_atomized_scalar_is_normalized(atomized, later - 1);
    }
}

proof fn lemma_advancing_candidate_preserves_block_order(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    index: int,
    prior_end: u64,
)
    requires
        crate::structural::structural_lexeme_source_well_formed_spec(atomized, layout, structural),
        0 <= index,
        index + 1 < structural.lexemes.len(),
        prior_end <= structural.lexemes[index].start_atom_index,
    ensures
        prior_end <= structural.lexemes[index + 1].start_atom_index,
{
    crate::structural::lemma_structural_well_formed_has_exact_partition(
        atomized,
        layout,
        structural,
    );
    reveal(crate::structural::structural_lexeme_partition_spec);
    reveal(crate::structural::structural_candidate_prefix_partition_spec);
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct HeaderInfoView {
    chomping: BlockChomping,
    explicit_indentation: Option<u8>,
    header_end_atom_index: u64,
}

closed spec fn block_error_spec(
    kind: BlockScalarErrorKind,
    byte_offset: u64,
) -> BlockScalarErrorView {
    BlockScalarErrorView { kind, byte_offset }
}

closed spec fn parse_block_header_tail_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    explicit_indentation: Option<u8>,
    chomping: BlockChomping,
    saw_chomping: bool,
    separated: bool,
    fuel: nat,
) -> Result<(BlockChomping, Option<u8>), BlockScalarErrorView>
    decreases fuel,
{
    if index >= end || index >= atoms.len() || index < 0 || fuel == 0 {
        Ok((chomping, explicit_indentation))
    } else {
        let atom = atoms[index];
        let code_point = atom.code_point;
        if separated {
            if code_point == 0x20 || code_point == 0x09 {
                parse_block_header_tail_spec(
                    atoms,
                    index + 1,
                    end,
                    explicit_indentation,
                    chomping,
                    saw_chomping,
                    true,
                    (fuel - 1) as nat,
                )
            } else if code_point == 0x23 {
                Ok((chomping, explicit_indentation))
            } else {
                Err(
                    block_error_spec(
                        BlockScalarErrorKind::InvalidBlockHeader,
                        atom.span.start.byte_offset,
                    ),
                )
            }
        } else if code_point == 0x20 || code_point == 0x09 {
            parse_block_header_tail_spec(
                atoms,
                index + 1,
                end,
                explicit_indentation,
                chomping,
                saw_chomping,
                true,
                (fuel - 1) as nat,
            )
        } else if code_point == 0x30 {
            Err(
                block_error_spec(
                    BlockScalarErrorKind::InvalidIndentationIndicator,
                    atom.span.start.byte_offset,
                ),
            )
        } else if 0x31 <= code_point <= 0x39 {
            if explicit_indentation.is_some() {
                Err(
                    block_error_spec(
                        BlockScalarErrorKind::InvalidBlockHeader,
                        atom.span.start.byte_offset,
                    ),
                )
            } else {
                parse_block_header_tail_spec(
                    atoms,
                    index + 1,
                    end,
                    Some((code_point - 0x30) as u8),
                    chomping,
                    saw_chomping,
                    false,
                    (fuel - 1) as nat,
                )
            }
        } else if code_point == 0x2b || code_point == 0x2d {
            if saw_chomping {
                Err(
                    block_error_spec(
                        BlockScalarErrorKind::InvalidBlockHeader,
                        atom.span.start.byte_offset,
                    ),
                )
            } else {
                parse_block_header_tail_spec(
                    atoms,
                    index + 1,
                    end,
                    explicit_indentation,
                    if code_point == 0x2b {
                        BlockChomping::Keep
                    } else {
                        BlockChomping::Strip
                    },
                    true,
                    false,
                    (fuel - 1) as nat,
                )
            }
        } else {
            Err(
                block_error_spec(
                    BlockScalarErrorKind::InvalidBlockHeader,
                    atom.span.start.byte_offset,
                ),
            )
        }
    }
}

closed spec fn parse_block_header_spec(
    atoms: Seq<LexicalAtomView>,
    line: LayoutLineView,
    indicator_atom_index: int,
) -> Result<HeaderInfoView, BlockScalarErrorView> {
    if indicator_atom_index < 0 || indicator_atom_index >= line.end_atom_index
        || line.end_atom_index > atoms.len() {
        Err(block_error_spec(BlockScalarErrorKind::InvalidBlockHeader, line.byte_start))
    } else {
        match parse_block_header_tail_spec(
            atoms,
            indicator_atom_index + 1,
            line.end_atom_index as int,
            None,
            BlockChomping::Clip,
            false,
            false,
            (line.end_atom_index as int - indicator_atom_index - 1) as nat,
        ) {
            Err(error) => Err(error),
            Ok((chomping, explicit_indentation)) => if !line.terminated {
                Err(
                    block_error_spec(
                        BlockScalarErrorKind::MissingBlockHeaderLineBreak,
                        line.byte_end,
                    ),
                )
            } else {
                Ok(
                    HeaderInfoView {
                        chomping,
                        explicit_indentation,
                        header_end_atom_index: (line.end_atom_index + 1) as u64,
                    },
                )
            },
        }
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct BlockProbeView {
    first_nonempty: Option<int>,
    first_nonempty_indentation: u64,
    longest_leading_empty: u64,
    provisional_boundary: int,
}

closed spec fn line_all_space_spec(line: LayoutLineView) -> bool {
    line.content_atom_index == line.end_atom_index
}

closed spec fn block_probe_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    index: int,
    parent_indentation: u64,
    explicit_content_indentation: Option<u64>,
    longest_leading_empty: u64,
    fuel: nat,
) -> Result<BlockProbeView, BlockScalarErrorView>
    decreases fuel,
{
    if index >= lines.len() || index < 0 || fuel == 0 {
        Ok(
            BlockProbeView {
                first_nonempty: None,
                first_nonempty_indentation: 0,
                longest_leading_empty,
                provisional_boundary: lines.len() as int,
            },
        )
    } else {
        let line = lines[index];
        if line_all_space_spec(line) {
            block_probe_spec(
                atoms,
                lines,
                index + 1,
                parent_indentation,
                explicit_content_indentation,
                if line.indentation_columns > longest_leading_empty {
                    line.indentation_columns
                } else {
                    longest_leading_empty
                },
                (fuel - 1) as nat,
            )
        } else if line.content_atom_index >= atoms.len() {
            Err(block_error_spec(BlockScalarErrorKind::InputPlainMismatch, line.byte_start))
        } else {
            let atom = atoms[line.content_atom_index as int];
            match explicit_content_indentation {
                Some(required) => if atom.kind == LexicalAtomKind::Tab && line.indentation_columns
                    < required {
                    Err(
                        block_error_spec(
                            BlockScalarErrorKind::TabInIndentation,
                            atom.span.start.byte_offset,
                        ),
                    )
                } else if line.indentation_columns <= parent_indentation {
                    Ok(
                        BlockProbeView {
                            first_nonempty: None,
                            first_nonempty_indentation: 0,
                            longest_leading_empty,
                            provisional_boundary: index,
                        },
                    )
                } else if line.indentation_columns < required {
                    if atom.code_point == 0x23 {
                        Ok(
                            BlockProbeView {
                                first_nonempty: None,
                                first_nonempty_indentation: 0,
                                longest_leading_empty,
                                provisional_boundary: index,
                            },
                        )
                    } else {
                        Err(
                            block_error_spec(
                                BlockScalarErrorKind::InvalidBlockIndentation,
                                atom.span.start.byte_offset,
                            ),
                        )
                    }
                } else {
                    Ok(
                        BlockProbeView {
                            first_nonempty: Some(index),
                            first_nonempty_indentation: line.indentation_columns,
                            longest_leading_empty,
                            provisional_boundary: lines.len() as int,
                        },
                    )
                },
                None => if atom.kind == LexicalAtomKind::Tab && line.indentation_columns
                    <= parent_indentation {
                    Err(
                        block_error_spec(
                            BlockScalarErrorKind::TabInIndentation,
                            atom.span.start.byte_offset,
                        ),
                    )
                } else if line.indentation_columns <= parent_indentation {
                    Ok(
                        BlockProbeView {
                            first_nonempty: None,
                            first_nonempty_indentation: 0,
                            longest_leading_empty,
                            provisional_boundary: index,
                        },
                    )
                } else {
                    Ok(
                        BlockProbeView {
                            first_nonempty: Some(index),
                            first_nonempty_indentation: line.indentation_columns,
                            longest_leading_empty,
                            provisional_boundary: lines.len() as int,
                        },
                    )
                },
            }
        }
    }
}

closed spec fn validate_leading_empty_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    index: int,
    end: int,
    content_indentation: u64,
    fuel: nat,
) -> Result<(), BlockScalarErrorView>
    decreases fuel,
{
    if index >= end || index < 0 || index >= lines.len() || fuel == 0 {
        Ok(())
    } else {
        let line = lines[index];
        if line.indentation_columns > content_indentation {
            let bad_atom = line.start_atom_index + content_indentation;
            if bad_atom >= atoms.len() {
                Err(block_error_spec(BlockScalarErrorKind::InputPlainMismatch, line.byte_start))
            } else {
                Err(
                    block_error_spec(
                        BlockScalarErrorKind::InvalidLeadingEmptyIndentation,
                        atoms[bad_atom as int].span.start.byte_offset,
                    ),
                )
            }
        } else {
            validate_leading_empty_spec(
                atoms,
                lines,
                index + 1,
                end,
                content_indentation,
                (fuel - 1) as nat,
            )
        }
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct BlockEndView {
    end_line: int,
    last_nonempty: Option<int>,
}

pub open spec fn logical_content_start_spec(line: LayoutLineView, content_indentation: u64) -> u64 {
    if content_indentation <= line.indentation_columns && line.start_atom_index
        + content_indentation < line.end_atom_index {
        (line.start_atom_index + content_indentation) as u64
    } else {
        line.end_atom_index
    }
}

closed spec fn scan_block_end_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    index: int,
    parent_indentation: u64,
    content_indentation: u64,
    last_nonempty: Option<int>,
    fuel: nat,
) -> Result<BlockEndView, BlockScalarErrorView>
    decreases fuel,
{
    if index >= lines.len() || index < 0 || fuel == 0 {
        Ok(BlockEndView { end_line: index, last_nonempty })
    } else {
        let line = lines[index];
        if !line_all_space_spec(line) {
            if line.content_atom_index >= atoms.len() {
                Err(block_error_spec(BlockScalarErrorKind::InputPlainMismatch, line.byte_start))
            } else {
                let atom = atoms[line.content_atom_index as int];
                if atom.kind == LexicalAtomKind::Tab && line.indentation_columns
                    < content_indentation {
                    Err(
                        block_error_spec(
                            BlockScalarErrorKind::TabInIndentation,
                            atom.span.start.byte_offset,
                        ),
                    )
                } else if line.indentation_columns < content_indentation {
                    if line.indentation_columns <= parent_indentation || atom.code_point == 0x23 {
                        Ok(BlockEndView { end_line: index, last_nonempty })
                    } else {
                        Err(
                            block_error_spec(
                                BlockScalarErrorKind::InvalidBlockIndentation,
                                atom.span.start.byte_offset,
                            ),
                        )
                    }
                } else {
                    scan_block_end_spec(
                        atoms,
                        lines,
                        index + 1,
                        parent_indentation,
                        content_indentation,
                        if logical_content_start_spec(line, content_indentation)
                            < line.end_atom_index {
                            Some(index)
                        } else {
                            last_nonempty
                        },
                        (fuel - 1) as nat,
                    )
                }
            }
        } else {
            scan_block_end_spec(
                atoms,
                lines,
                index + 1,
                parent_indentation,
                content_indentation,
                if logical_content_start_spec(line, content_indentation) < line.end_atom_index {
                    Some(index)
                } else {
                    last_nonempty
                },
                (fuel - 1) as nat,
            )
        }
    }
}

closed spec fn direct_content_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Result<Seq<BlockScalarContentScalarView>, BlockScalarErrorView>
    decreases fuel,
{
    if index >= end || index >= atoms.len() || index < 0 || fuel == 0 {
        Ok(Seq::empty())
    } else if !crate::quoted::yaml_printable_character_spec(atoms[index].code_point)
        || atoms[index].code_point == 0xfeff {
        Err(
            block_error_spec(
                BlockScalarErrorKind::InvalidBlockCharacter,
                atoms[index].span.start.byte_offset,
            ),
        )
    } else {
        match direct_content_spec(atoms, index + 1, end, (fuel - 1) as nat) {
            Err(error) => Err(error),
            Ok(tail) => Ok(
                Seq::empty().push(
                    BlockScalarContentScalarView {
                        code_point: atoms[index].code_point,
                        source_atom_index: index as u64,
                        byte_start: atoms[index].span.start.byte_offset,
                        byte_end: atoms[index].span.end.byte_offset,
                        origin: BlockScalarContentOrigin::Direct,
                    },
                ) + tail,
            ),
        }
    }
}

closed spec fn line_break_content_spec(
    atoms: Seq<LexicalAtomView>,
    line: LayoutLineView,
    folded: bool,
) -> Seq<BlockScalarContentScalarView> {
    if line.terminated && line.end_atom_index < atoms.len() {
        Seq::empty().push(
            BlockScalarContentScalarView {
                code_point: if folded {
                    0x20
                } else {
                    0x0a
                },
                source_atom_index: line.end_atom_index,
                byte_start: atoms[line.end_atom_index as int].span.start.byte_offset,
                byte_end: atoms[line.end_atom_index as int].span.end.byte_offset,
                origin: if folded {
                    BlockScalarContentOrigin::FoldedLineBreak
                } else {
                    BlockScalarContentOrigin::Direct
                },
            },
        )
    } else {
        Seq::empty()
    }
}

closed spec fn line_break_range_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    index: int,
    end: int,
    fuel: nat,
) -> Seq<BlockScalarContentScalarView>
    decreases fuel,
{
    if index >= end || index < 0 || end > lines.len() || fuel == 0 {
        Seq::empty()
    } else {
        line_break_range_spec(atoms, lines, index, end - 1, (fuel - 1) as nat)
            + line_break_content_spec(atoms, lines[end - 1], false)
    }
}

proof fn lemma_line_break_range_snoc(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    start: int,
    index: int,
)
    requires
        0 <= start <= index < lines.len(),
    ensures
        line_break_range_spec(atoms, lines, start, index + 1, (index + 1 - start) as nat)
            == line_break_range_spec(atoms, lines, start, index, (index - start) as nat)
            + line_break_content_spec(atoms, lines[index], false),
    decreases index - start,
{
    reveal(line_break_range_spec);
}

closed spec fn next_nonempty_line_spec(
    lines: Seq<LayoutLineView>,
    index: int,
    end: int,
    content_indentation: u64,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index >= end || index >= lines.len() || index < 0 || fuel == 0 {
        end
    } else if logical_content_start_spec(lines[index], content_indentation)
        < lines[index].end_atom_index {
        index
    } else {
        next_nonempty_line_spec(lines, index + 1, end, content_indentation, (fuel - 1) as nat)
    }
}

closed spec fn render_nonempty_tail_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    current: int,
    last: int,
    end_line: int,
    content_indentation: u64,
    style: BlockScalarStyle,
    chomping: BlockChomping,
    fuel: nat,
) -> Result<Seq<BlockScalarContentScalarView>, BlockScalarErrorView>
    decreases fuel,
{
    if current < 0 || current >= lines.len() || current > last || fuel == 0 {
        Ok(Seq::empty())
    } else {
        let line = lines[current];
        let start = logical_content_start_spec(line, content_indentation);
        match direct_content_spec(
            atoms,
            start as int,
            line.end_atom_index as int,
            (line.end_atom_index - start) as nat,
        ) {
            Err(error) => Err(error),
            Ok(direct) => if current == last {
                Ok(
                    direct + if chomping == BlockChomping::Strip {
                        Seq::empty()
                    } else if chomping == BlockChomping::Clip {
                        line_break_content_spec(atoms, line, false)
                    } else {
                        line_break_content_spec(atoms, line, false) + line_break_range_spec(
                            atoms,
                            lines,
                            current + 1,
                            end_line,
                            (end_line - current - 1) as nat,
                        )
                    },
                )
            } else {
                let next = next_nonempty_line_spec(
                    lines,
                    current + 1,
                    last + 1,
                    content_indentation,
                    (last - current) as nat,
                );
                let more = start < line.end_atom_index && atoms[start as int].kind
                    == LexicalAtomKind::Space || start < line.end_atom_index
                    && atoms[start as int].kind == LexicalAtomKind::Tab;
                let next_start = logical_content_start_spec(lines[next], content_indentation);
                let next_more = next_start < lines[next].end_atom_index && (
                atoms[next_start as int].kind == LexicalAtomKind::Space
                    || atoms[next_start as int].kind == LexicalAtomKind::Tab);
                let gap = if style == BlockScalarStyle::Folded && !more && !next_more {
                    if next == current + 1 {
                        line_break_content_spec(atoms, line, true)
                    } else {
                        line_break_range_spec(
                            atoms,
                            lines,
                            current + 1,
                            next,
                            (next - current - 1) as nat,
                        )
                    }
                } else {
                    line_break_content_spec(atoms, line, false) + line_break_range_spec(
                        atoms,
                        lines,
                        current + 1,
                        next,
                        (next - current - 1) as nat,
                    )
                };
                match render_nonempty_tail_spec(
                    atoms,
                    lines,
                    next,
                    last,
                    end_line,
                    content_indentation,
                    style,
                    chomping,
                    (fuel - 1) as nat,
                ) {
                    Err(error) => Err(error),
                    Ok(tail) => Ok(direct + gap + tail),
                }
            },
        }
    }
}

pub closed spec fn render_block_content_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    start_line: int,
    end_line: int,
    content_indentation: u64,
    style: BlockScalarStyle,
    chomping: BlockChomping,
    last_nonempty: Option<int>,
) -> Result<Seq<BlockScalarContentScalarView>, BlockScalarErrorView> {
    match last_nonempty {
        None => Ok(
            if chomping == BlockChomping::Keep {
                line_break_range_spec(
                    atoms,
                    lines,
                    start_line,
                    end_line,
                    (end_line - start_line) as nat,
                )
            } else {
                Seq::empty()
            },
        ),
        Some(last) => render_block_tail_spec(
            atoms,
            lines,
            start_line,
            start_line,
            last,
            end_line,
            content_indentation,
            style,
            chomping,
            None,
            Seq::empty(),
            (last + 1 - start_line) as nat,
        ),
    }
}

closed spec fn block_gap_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    start_line: int,
    line_index: int,
    content_indentation: u64,
    style: BlockScalarStyle,
    previous_nonempty: Option<int>,
) -> Seq<BlockScalarContentScalarView> {
    let content_start = logical_content_start_spec(lines[line_index], content_indentation);
    let more_indented = atoms[content_start as int].kind == LexicalAtomKind::Space
        || atoms[content_start as int].kind == LexicalAtomKind::Tab;
    match previous_nonempty {
        None => line_break_range_spec(
            atoms,
            lines,
            start_line,
            line_index,
            (line_index - start_line) as nat,
        ),
        Some(previous_index) => {
            let previous_line = lines[previous_index];
            let previous_start = logical_content_start_spec(previous_line, content_indentation);
            let previous_more_indented = previous_start < previous_line.end_atom_index && (
            atoms[previous_start as int].kind == LexicalAtomKind::Space
                || atoms[previous_start as int].kind == LexicalAtomKind::Tab);
            if style == BlockScalarStyle::Folded && !previous_more_indented && !more_indented {
                if line_index == previous_index + 1 {
                    line_break_content_spec(atoms, previous_line, true)
                } else {
                    line_break_range_spec(
                        atoms,
                        lines,
                        previous_index + 1,
                        line_index,
                        (line_index - previous_index - 1) as nat,
                    )
                }
            } else {
                line_break_content_spec(atoms, previous_line, false) + line_break_range_spec(
                    atoms,
                    lines,
                    previous_index + 1,
                    line_index,
                    (line_index - previous_index - 1) as nat,
                )
            }
        },
    }
}

closed spec fn render_block_tail_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    start_line: int,
    line_index: int,
    last: int,
    end_line: int,
    content_indentation: u64,
    style: BlockScalarStyle,
    chomping: BlockChomping,
    previous_nonempty: Option<int>,
    content: Seq<BlockScalarContentScalarView>,
    fuel: nat,
) -> Result<Seq<BlockScalarContentScalarView>, BlockScalarErrorView>
    decreases fuel,
{
    if line_index > last || line_index < 0 || line_index >= lines.len() || fuel == 0 {
        Ok(
            content + if chomping == BlockChomping::Strip {
                Seq::empty()
            } else if chomping == BlockChomping::Clip {
                line_break_content_spec(atoms, lines[last], false)
            } else {
                line_break_content_spec(atoms, lines[last], false) + line_break_range_spec(
                    atoms,
                    lines,
                    last + 1,
                    end_line,
                    (end_line - last - 1) as nat,
                )
            },
        )
    } else {
        let line = lines[line_index];
        let content_start = logical_content_start_spec(line, content_indentation);
        if content_start >= line.end_atom_index {
            render_block_tail_spec(
                atoms,
                lines,
                start_line,
                line_index + 1,
                last,
                end_line,
                content_indentation,
                style,
                chomping,
                previous_nonempty,
                content,
                (fuel - 1) as nat,
            )
        } else {
            let gap = block_gap_spec(
                atoms,
                lines,
                start_line,
                line_index,
                content_indentation,
                style,
                previous_nonempty,
            );
            match direct_content_spec(
                atoms,
                content_start as int,
                line.end_atom_index as int,
                (line.end_atom_index - content_start) as nat,
            ) {
                Err(error) => Err(error),
                Ok(direct) => render_block_tail_spec(
                    atoms,
                    lines,
                    start_line,
                    line_index + 1,
                    last,
                    end_line,
                    content_indentation,
                    style,
                    chomping,
                    Some(line_index),
                    content + gap + direct,
                    (fuel - 1) as nat,
                ),
            }
        }
    }
}

closed spec fn build_from_probe_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    candidate: StructuralLexemeView,
    style: BlockScalarStyle,
    header: HeaderInfoView,
    parent: u64,
    start_line: int,
    explicit: Option<u64>,
    probe: BlockProbeView,
) -> Result<(BlockScalarView, int), BlockScalarErrorView> {
    match probe.first_nonempty {
        Some(first) => match validate_leading_empty_spec(
            atoms,
            lines,
            start_line,
            first,
            probe.first_nonempty_indentation,
            (first - start_line) as nat,
        ) {
            Err(error) => Err(error),
            Ok(()) => finish_block_scalar_spec(
                atoms,
                lines,
                candidate,
                style,
                header,
                parent,
                start_line,
                match explicit {
                    Some(required) => required,
                    None => probe.first_nonempty_indentation,
                },
                lines.len() as int,
            ),
        },
        None => {
            let minimum = (parent + 1) as u64;
            let content_indentation = match explicit {
                Some(required) => required,
                None => if probe.longest_leading_empty > minimum {
                    probe.longest_leading_empty
                } else {
                    minimum
                },
            };
            finish_block_scalar_spec(
                atoms,
                lines,
                candidate,
                style,
                header,
                parent,
                start_line,
                content_indentation,
                probe.provisional_boundary,
            )
        },
    }
}

closed spec fn build_block_scalar_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    candidate: StructuralLexemeView,
    style: BlockScalarStyle,
    parent: u64,
) -> Result<(BlockScalarView, int), BlockScalarErrorView> {
    if candidate.line_number >= lines.len() || candidate.start_atom_index >= atoms.len() {
        Err(block_error_spec(BlockScalarErrorKind::InputPlainMismatch, candidate.byte_start))
    } else if parent > MAX_PROFILE1_LEXICAL_ATOMS - 9 {
        Err(block_error_spec(BlockScalarErrorKind::InputPlainMismatch, candidate.byte_start))
    } else {
        let header_line = lines[candidate.line_number as int];
        match parse_block_header_spec(atoms, header_line, candidate.start_atom_index as int) {
            Err(error) => Err(error),
            Ok(header) => {
                let start_line = candidate.line_number as int + 1;
                let explicit = match header.explicit_indentation {
                    Some(value) => Some((parent + value as u64) as u64),
                    None => None,
                };
                match block_probe_spec(
                    atoms,
                    lines,
                    start_line,
                    parent,
                    explicit,
                    0,
                    (lines.len() - start_line) as nat,
                ) {
                    Err(error) => Err(error),
                    Ok(probe) => build_from_probe_spec(
                        atoms,
                        lines,
                        candidate,
                        style,
                        header,
                        parent,
                        start_line,
                        explicit,
                        probe,
                    ),
                }
            },
        }
    }
}

closed spec fn finish_from_end_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    candidate: StructuralLexemeView,
    style: BlockScalarStyle,
    header: HeaderInfoView,
    parent: u64,
    start_line: int,
    content_indentation: u64,
    maximum_end_line: int,
    end: BlockEndView,
) -> Result<(BlockScalarView, int), BlockScalarErrorView> {
    let end_line = if end.end_line < maximum_end_line {
        end.end_line
    } else {
        maximum_end_line
    };
    let end_atom = if end_line < lines.len() {
        lines[end_line].start_atom_index
    } else {
        atoms.len() as u64
    };
    if end_atom < header.header_end_atom_index || end_atom > atoms.len() || end_atom == 0 {
        Err(block_error_spec(BlockScalarErrorKind::InputPlainMismatch, candidate.byte_start))
    } else {
        match render_block_content_spec(
            atoms,
            lines,
            start_line,
            end_line,
            content_indentation,
            style,
            header.chomping,
            end.last_nonempty,
        ) {
            Err(error) => Err(error),
            Ok(content) => Ok(
                (
                    BlockScalarView {
                        style,
                        chomping: header.chomping,
                        explicit_indentation: header.explicit_indentation,
                        parent_indentation: parent,
                        content_indentation,
                        start_line_number: candidate.line_number,
                        end_line_number: atoms[(end_atom - 1) as int].span.start.line,
                        start_atom_index: candidate.start_atom_index,
                        header_end_atom_index: header.header_end_atom_index,
                        content_start_atom_index: header.header_end_atom_index,
                        end_atom_index: end_atom,
                        byte_start: candidate.byte_start,
                        byte_end: atoms[(end_atom - 1) as int].span.end.byte_offset,
                        content,
                    },
                    end_line,
                ),
            ),
        }
    }
}

closed spec fn finish_block_scalar_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    candidate: StructuralLexemeView,
    style: BlockScalarStyle,
    header: HeaderInfoView,
    parent: u64,
    start_line: int,
    content_indentation: u64,
    maximum_end_line: int,
) -> Result<(BlockScalarView, int), BlockScalarErrorView> {
    match scan_block_end_spec(
        atoms,
        lines,
        start_line,
        parent,
        content_indentation,
        None,
        (maximum_end_line - start_line) as nat,
    ) {
        Err(error) => Err(error),
        Ok(end) => finish_from_end_spec(
            atoms,
            lines,
            candidate,
            style,
            header,
            parent,
            start_line,
            content_indentation,
            maximum_end_line,
            end,
        ),
    }
}

closed spec fn atom_inside_quoted_spec(quotes: Seq<QuotedScalarView>, atom_index: u64) -> bool {
    exists|index: int|
        0 <= index < quotes.len() && #[trigger] quotes[index].start_atom_index <= atom_index
            < quotes[index].end_atom_index
}

closed spec fn atom_inside_plain_spec(plains: Seq<PlainScalarView>, atom_index: u64) -> bool {
    exists|index: int|
        0 <= index < plains.len() && #[trigger] plains[index].start_atom_index <= atom_index
            < plains[index].end_atom_index
}

closed spec fn candidate_index_after_block_spec(
    candidates: Seq<StructuralLexemeView>,
    index: int,
    end_atom_index: u64,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < candidates.len() && index >= 0 && fuel > 0 && candidates[index].start_atom_index
        < end_atom_index {
        candidate_index_after_block_spec(candidates, index + 1, end_atom_index, (fuel - 1) as nat)
    } else {
        index
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct BlockScanView {
    scalars: Seq<BlockScalarView>,
    total_content_code_points: u64,
}

closed spec fn quoted_index_after_atom_spec(
    quotes: Seq<QuotedScalarView>,
    index: int,
    atom_index: u64,
    fuel: nat,
) -> int
    decreases fuel,
{
    if 0 <= index < quotes.len() && fuel > 0 && quotes[index].end_atom_index <= atom_index {
        quoted_index_after_atom_spec(quotes, index + 1, atom_index, (fuel - 1) as nat)
    } else {
        index
    }
}

closed spec fn plain_index_after_atom_spec(
    plains: Seq<PlainScalarView>,
    index: int,
    atom_index: u64,
    fuel: nat,
) -> int
    decreases fuel,
{
    if 0 <= index < plains.len() && fuel > 0 && plains[index].end_atom_index <= atom_index {
        plain_index_after_atom_spec(plains, index + 1, atom_index, (fuel - 1) as nat)
    } else {
        index
    }
}

#[verifier::ext_equal]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct BlockGrammarContextView {
    parent_indentation: u64,
    node_start_column: u64,
    expecting_node: bool,
    mapping_committed: bool,
}

#[derive(Clone, Copy)]
struct BlockGrammarContext {
    parent_indentation: u64,
    node_start_column: u64,
    expecting_node: bool,
    mapping_committed: bool,
}

impl View for BlockGrammarContext {
    type V = BlockGrammarContextView;

    closed spec fn view(&self) -> BlockGrammarContextView {
        BlockGrammarContextView {
            parent_indentation: self.parent_indentation,
            node_start_column: self.node_start_column,
            expecting_node: self.expecting_node,
            mapping_committed: self.mapping_committed,
        }
    }
}

closed spec fn initial_block_grammar_context_spec() -> BlockGrammarContextView {
    BlockGrammarContextView {
        parent_indentation: 0,
        node_start_column: 0,
        expecting_node: true,
        mapping_committed: false,
    }
}

fn initial_block_grammar_context() -> (context: BlockGrammarContext)
    ensures
        context@ == initial_block_grammar_context_spec(),
{
    BlockGrammarContext {
        parent_indentation: 0,
        node_start_column: 0,
        expecting_node: true,
        mapping_committed: false,
    }
}

closed spec fn block_grammar_context_after_candidate_spec(
    atoms: Seq<LexicalAtomView>,
    candidate: StructuralLexemeView,
    scalar_content: bool,
    context: BlockGrammarContextView,
) -> BlockGrammarContextView {
    if candidate.start_atom_index >= candidate.end_atom_index || candidate.end_atom_index
        > atoms.len() {
        context
    } else if candidate.kind == StructuralCandidateRole::LineFeed {
        initial_block_grammar_context_spec()
    } else if candidate.kind == StructuralCandidateRole::Indentation {
        let indentation = (candidate.end_atom_index - candidate.start_atom_index) as u64;
        BlockGrammarContextView {
            parent_indentation: indentation,
            node_start_column: indentation,
            expecting_node: true,
            mapping_committed: false,
        }
    } else if candidate.kind == StructuralCandidateRole::Separation {
        context
    } else if candidate.start_atom_index >= atoms.len() {
        context
    } else {
        let column = atoms[candidate.start_atom_index as int].span.start.column;
        let started = if context.expecting_node {
            BlockGrammarContextView { node_start_column: column, expecting_node: false, ..context }
        } else {
            context
        };
        if scalar_content {
            started
        } else if candidate.kind == StructuralCandidateRole::Indicator(
            YamlIndicator::BlockSequenceEntry,
        ) || candidate.kind == StructuralCandidateRole::Indicator(
            YamlIndicator::ExplicitMappingKey,
        ) {
            BlockGrammarContextView {
                parent_indentation: if started.node_start_column > started.parent_indentation {
                    started.node_start_column
                } else {
                    started.parent_indentation
                },
                expecting_node: true,
                mapping_committed: false,
                ..started
            }
        } else if candidate.kind == StructuralCandidateRole::Indicator(
            YamlIndicator::MappingValue,
        ) {
            BlockGrammarContextView {
                parent_indentation: if !started.mapping_committed && started.node_start_column
                    > started.parent_indentation {
                    started.node_start_column
                } else {
                    started.parent_indentation
                },
                expecting_node: true,
                mapping_committed: true,
                ..started
            }
        } else {
            started
        }
    }
}

fn block_grammar_context_after_candidate(
    atoms: &[LexicalAtom],
    candidate: &StructuralLexeme,
    scalar_content: bool,
    context: BlockGrammarContext,
) -> (next: BlockGrammarContext)
    ensures
        next@ == block_grammar_context_after_candidate_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            candidate@,
            scalar_content,
            context@,
        ),
{
    if candidate.start_atom_index() >= candidate.end_atom_index() || candidate.end_atom_index()
        > atoms.len() as u64 {
        return context;
    }
    let role = candidate.candidate_role();
    if role == StructuralCandidateRole::LineFeed {
        return initial_block_grammar_context();
    }
    if role == StructuralCandidateRole::Indentation {
        let indentation = candidate.end_atom_index() - candidate.start_atom_index();
        return BlockGrammarContext {
            parent_indentation: indentation,
            node_start_column: indentation,
            expecting_node: true,
            mapping_committed: false,
        };
    }
    if role == StructuralCandidateRole::Separation {
        return context;
    }
    let column = atoms[candidate.start_atom_index() as usize].span().start().column();
    let mut next = context;
    if next.expecting_node {
        next.node_start_column = column;
        next.expecting_node = false;
    }
    if scalar_content {
        return next;
    }
    if role == StructuralCandidateRole::Indicator(YamlIndicator::BlockSequenceEntry) || role
        == StructuralCandidateRole::Indicator(YamlIndicator::ExplicitMappingKey) {
        if next.node_start_column > next.parent_indentation {
            next.parent_indentation = next.node_start_column;
        }
        next.expecting_node = true;
        next.mapping_committed = false;
    } else if role == StructuralCandidateRole::Indicator(YamlIndicator::MappingValue) {
        if !next.mapping_committed && next.node_start_column > next.parent_indentation {
            next.parent_indentation = next.node_start_column;
        }
        next.expecting_node = true;
        next.mapping_committed = true;
    }
    next
}

closed spec fn scan_block_tail_spec(
    atoms: Seq<LexicalAtomView>,
    lines: Seq<LayoutLineView>,
    candidates: Seq<StructuralLexemeView>,
    quotes: Seq<QuotedScalarView>,
    plains: Seq<PlainScalarView>,
    candidate_index: int,
    quote_index: int,
    plain_index: int,
    flow_depth: u64,
    grammar: BlockGrammarContextView,
    built: Seq<BlockScalarView>,
    total_content_code_points: u64,
    limits: BlockScalarScanLimitsView,
    fuel: nat,
) -> Result<BlockScanView, BlockScalarErrorView>
    decreases fuel,
{
    if candidate_index >= candidates.len() || candidate_index < 0 || fuel == 0 {
        Ok(BlockScanView { scalars: built, total_content_code_points })
    } else {
        let candidate = candidates[candidate_index];
        let next_quote_index = quoted_index_after_atom_spec(
            quotes,
            quote_index,
            candidate.start_atom_index,
            (quotes.len() - quote_index) as nat,
        );
        if 0 <= next_quote_index < quotes.len() && quotes[next_quote_index].start_atom_index
            <= candidate.start_atom_index < quotes[next_quote_index].end_atom_index {
            scan_block_tail_spec(
                atoms,
                lines,
                candidates,
                quotes,
                plains,
                candidate_index + 1,
                next_quote_index,
                plain_index,
                flow_depth,
                block_grammar_context_after_candidate_spec(atoms, candidate, true, grammar),
                built,
                total_content_code_points,
                limits,
                (fuel - 1) as nat,
            )
        } else {
            let next_plain_index = plain_index_after_atom_spec(
                plains,
                plain_index,
                candidate.start_atom_index,
                (plains.len() - plain_index) as nat,
            );
            if 0 <= next_plain_index < plains.len() && plains[next_plain_index].start_atom_index
                <= candidate.start_atom_index < plains[next_plain_index].end_atom_index {
                scan_block_tail_spec(
                    atoms,
                    lines,
                    candidates,
                    quotes,
                    plains,
                    candidate_index + 1,
                    next_quote_index,
                    next_plain_index,
                    flow_depth,
                    block_grammar_context_after_candidate_spec(atoms, candidate, true, grammar),
                    built,
                    total_content_code_points,
                    limits,
                    (fuel - 1) as nat,
                )
            } else if candidate.kind == StructuralCandidateRole::FlowSequenceStart || candidate.kind
                == StructuralCandidateRole::FlowMappingStart {
                scan_block_tail_spec(
                    atoms,
                    lines,
                    candidates,
                    quotes,
                    plains,
                    candidate_index + 1,
                    next_quote_index,
                    next_plain_index,
                    (flow_depth + 1) as u64,
                    block_grammar_context_after_candidate_spec(atoms, candidate, false, grammar),
                    built,
                    total_content_code_points,
                    limits,
                    (fuel - 1) as nat,
                )
            } else if candidate.kind == StructuralCandidateRole::FlowSequenceEnd || candidate.kind
                == StructuralCandidateRole::FlowMappingEnd {
                scan_block_tail_spec(
                    atoms,
                    lines,
                    candidates,
                    quotes,
                    plains,
                    candidate_index + 1,
                    next_quote_index,
                    next_plain_index,
                    if flow_depth > 0 {
                        (flow_depth - 1) as u64
                    } else {
                        0u64
                    },
                    block_grammar_context_after_candidate_spec(atoms, candidate, false, grammar),
                    built,
                    total_content_code_points,
                    limits,
                    (fuel - 1) as nat,
                )
            } else {
                let style = block_candidate_style_spec(candidate.kind);
                if style.is_none() || flow_depth > 0 {
                    scan_block_tail_spec(
                        atoms,
                        lines,
                        candidates,
                        quotes,
                        plains,
                        candidate_index + 1,
                        next_quote_index,
                        next_plain_index,
                        flow_depth,
                        block_grammar_context_after_candidate_spec(
                            atoms,
                            candidate,
                            false,
                            grammar,
                        ),
                        built,
                        total_content_code_points,
                        limits,
                        (fuel - 1) as nat,
                    )
                } else if candidate.start_atom_index >= candidate.end_atom_index
                    || candidate.end_atom_index > atoms.len() || candidate.line_number
                    >= lines.len() {
                    Err(
                        block_error_spec(
                            BlockScalarErrorKind::InputPlainMismatch,
                            candidate.byte_start,
                        ),
                    )
                } else if candidate.start_atom_index
                    < lines[candidate.line_number as int].start_atom_index
                    || candidate.start_atom_index
                    >= lines[candidate.line_number as int].end_atom_index {
                    Err(
                        block_error_spec(
                            BlockScalarErrorKind::InputPlainMismatch,
                            candidate.byte_start,
                        ),
                    )
                } else {
                    match build_block_scalar_spec(
                        atoms,
                        lines,
                        candidate,
                        style.unwrap(),
                        grammar.parent_indentation,
                    ) {
                        Err(error) => Err(error),
                        Ok((scalar, _)) => if scalar.start_atom_index >= scalar.end_atom_index
                            || scalar.end_atom_index > atoms.len() {
                            Err(
                                block_error_spec(
                                    BlockScalarErrorKind::InputPlainMismatch,
                                    candidate.byte_start,
                                ),
                            )
                        } else if built.len() >= limits.max_scalars {
                            Err(
                                block_error_spec(
                                    BlockScalarErrorKind::ScalarLimitExceeded,
                                    scalar.byte_start,
                                ),
                            )
                        } else if scalar.end_atom_index - scalar.start_atom_index
                            > limits.max_scalar_presentation_atoms {
                            let excluded = scalar.start_atom_index
                                + limits.max_scalar_presentation_atoms;
                            Err(
                                block_error_spec(
                                    BlockScalarErrorKind::PresentationAtomLimitExceeded,
                                    atoms[excluded as int].span.start.byte_offset,
                                ),
                            )
                        } else if scalar.content.len() > limits.max_scalar_content_code_points {
                            Err(
                                block_error_spec(
                                    BlockScalarErrorKind::ScalarContentLimitExceeded,
                                    scalar.content[limits.max_scalar_content_code_points as int].byte_start,
                                ),
                            )
                        } else if scalar.content.len() > limits.max_total_content_code_points
                            - total_content_code_points {
                            Err(
                                block_error_spec(
                                    BlockScalarErrorKind::TotalContentLimitExceeded,
                                    scalar.content[(limits.max_total_content_code_points
                                        - total_content_code_points) as int].byte_start,
                                ),
                            )
                        } else {
                            let next = candidate_index_after_block_spec(
                                candidates,
                                candidate_index + 1,
                                scalar.end_atom_index,
                                (candidates.len() - candidate_index - 1) as nat,
                            );
                            scan_block_tail_spec(
                                atoms,
                                lines,
                                candidates,
                                quotes,
                                plains,
                                next,
                                next_quote_index,
                                next_plain_index,
                                flow_depth,
                                initial_block_grammar_context_spec(),
                                built.push(scalar),
                                (total_content_code_points + scalar.content.len()) as u64,
                                limits,
                                (fuel - 1) as nat,
                            )
                        },
                    }
                }
            }
        }
    }
}

closed spec fn effective_block_limits_spec(
    limits: BlockScalarScanLimitsView,
) -> BlockScalarScanLimitsView {
    BlockScalarScanLimitsView {
        max_scalars: if limits.max_scalars < MAX_PROFILE1_BLOCK_SCALARS {
            limits.max_scalars
        } else {
            MAX_PROFILE1_BLOCK_SCALARS
        },
        max_scalar_presentation_atoms: if limits.max_scalar_presentation_atoms
            < MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS {
            limits.max_scalar_presentation_atoms
        } else {
            MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS
        },
        max_scalar_content_code_points: if limits.max_scalar_content_code_points
            < MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS {
            limits.max_scalar_content_code_points
        } else {
            MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS
        },
        max_total_content_code_points: if limits.max_total_content_code_points
            < MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS {
            limits.max_total_content_code_points
        } else {
            MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS
        },
    }
}

pub closed spec fn scan_profile1_block_scalars_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
) -> Result<BlockScalarSourceView, BlockScalarErrorView> {
    match crate::layout::analyze_profile1_layout_spec(
        atomized,
        crate::structural::canonical_layout_limits_spec(),
    ) {
        Err(error) => Err(
            block_error_spec(BlockScalarErrorKind::InputLayoutMismatch, error.byte_offset),
        ),
        Ok(canonical_layout) => if canonical_layout != layout {
            Err(block_error_spec(BlockScalarErrorKind::InputLayoutMismatch, 0))
        } else {
            match crate::structural::scan_profile1_structural_lexemes_spec(
                atomized,
                layout,
                crate::structural::canonical_structural_scan_limits_spec(),
            ) {
                Err(error) => Err(
                    block_error_spec(
                        BlockScalarErrorKind::InputStructuralMismatch,
                        error.byte_offset,
                    ),
                ),
                Ok(canonical_structural) => if canonical_structural != structural {
                    Err(block_error_spec(BlockScalarErrorKind::InputStructuralMismatch, 0))
                } else {
                    match crate::quoted::scan_profile1_quoted_scalars_spec(
                        atomized,
                        layout,
                        structural,
                        crate::quoted::canonical_quoted_scalar_limits_spec(),
                    ) {
                        Err(error) => Err(
                            block_error_spec(
                                BlockScalarErrorKind::InputQuotedMismatch,
                                error.byte_offset,
                            ),
                        ),
                        Ok(canonical_quoted) => if canonical_quoted != quoted {
                            Err(block_error_spec(BlockScalarErrorKind::InputQuotedMismatch, 0))
                        } else {
                            match crate::plain::scan_profile1_plain_scalars_spec(
                                atomized,
                                layout,
                                structural,
                                quoted,
                                crate::plain::canonical_plain_scalar_limits_spec(),
                            ) {
                                Err(error) => Err(
                                    block_error_spec(
                                        BlockScalarErrorKind::InputPlainMismatch,
                                        error.byte_offset,
                                    ),
                                ),
                                Ok(canonical_plain) => if canonical_plain != plain {
                                    Err(
                                        block_error_spec(
                                            BlockScalarErrorKind::InputPlainMismatch,
                                            0,
                                        ),
                                    )
                                } else {
                                    match scan_block_tail_spec(
                                        atomized.atoms,
                                        layout.lines,
                                        structural.lexemes,
                                        quoted.scalars,
                                        plain.scalars,
                                        0,
                                        0,
                                        0,
                                        0,
                                        initial_block_grammar_context_spec(),
                                        Seq::empty(),
                                        0,
                                        effective_block_limits_spec(limits),
                                        (structural.lexemes.len() + 1) as nat,
                                    ) {
                                        Err(error) => Err(error),
                                        Ok(scan) => Ok(
                                            BlockScalarSourceView {
                                                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                                                input_transformation_version:
                                                    atomized.transformation_version,
                                                layout_transformation_version:
                                                    layout.transformation_version,
                                                structural_transformation_version:
                                                    structural.transformation_version,
                                                quoted_transformation_version:
                                                    quoted.transformation_version,
                                                plain_transformation_version:
                                                    plain.transformation_version,
                                                transformation_version:
                                                    BLOCK_SCALAR_TRANSFORMATION_VERSION,
                                                source_len_bytes: atomized.source_len_bytes,
                                                bom_bytes: atomized.bom_bytes,
                                                input_atom_count: atomized.atoms.len() as u64,
                                                input_line_count: layout.lines.len() as u64,
                                                input_structural_lexeme_count:
                                                    structural.lexemes.len() as u64,
                                                input_quoted_scalar_count:
                                                    quoted.scalars.len() as u64,
                                                input_plain_scalar_count:
                                                    plain.scalars.len() as u64,
                                                total_content_code_points:
                                                    scan.total_content_code_points,
                                                scalars: scan.scalars,
                                            },
                                        ),
                                    }
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}

proof fn lemma_block_scan_spec_from_tail_error(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
    error: BlockScalarErrorView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted),
        crate::plain::scan_profile1_plain_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain),
        scan_block_tail_spec(
            atomized.atoms,
            layout.lines,
            structural.lexemes,
            quoted.scalars,
            plain.scalars,
            0,
            0,
            0,
            0,
            initial_block_grammar_context_spec(),
            Seq::empty(),
            0,
            effective_block_limits_spec(limits),
            (structural.lexemes.len() + 1) as nat,
        ) == Err(error),
    ensures
        scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits)
            == Err(error),
{
    reveal(scan_profile1_block_scalars_spec);
}

proof fn lemma_block_scan_spec_from_tail_success(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
    scan: BlockScanView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted),
        crate::plain::scan_profile1_plain_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain),
        scan_block_tail_spec(
            atomized.atoms,
            layout.lines,
            structural.lexemes,
            quoted.scalars,
            plain.scalars,
            0,
            0,
            0,
            0,
            initial_block_grammar_context_spec(),
            Seq::empty(),
            0,
            effective_block_limits_spec(limits),
            (structural.lexemes.len() + 1) as nat,
        ) == Ok(scan),
    ensures
        exists|source: BlockScalarSourceView|
            scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits)
                == Ok(source) && source.scalars == scan.scalars && source.total_content_code_points
                == scan.total_content_code_points,
{
    reveal(scan_profile1_block_scalars_spec);
}

pub closed spec fn block_scalar_source_corresponds_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    blocks: BlockScalarSourceView,
) -> bool {
    exists|limits: BlockScalarScanLimitsView|
        scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits) == Ok(
            blocks,
        )
}

/// Every exact successful block scan retains its authenticated upstream counts and metadata.
pub proof fn lemma_block_scan_success_metadata(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
    blocks: BlockScalarSourceView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted),
        crate::plain::scan_profile1_plain_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain),
        scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits) == Ok(
            blocks,
        ),
    ensures
        blocks.input_atom_count == atomized.atoms.len() as u64,
        blocks.input_line_count == layout.lines.len() as u64,
        blocks.input_structural_lexeme_count == structural.lexemes.len() as u64,
        blocks.input_quoted_scalar_count == quoted.scalars.len() as u64,
        blocks.input_plain_scalar_count == plain.scalars.len() as u64,
{
    reveal(scan_profile1_block_scalars_spec);
    let tail = scan_block_tail_spec(
        atomized.atoms,
        layout.lines,
        structural.lexemes,
        quoted.scalars,
        plain.scalars,
        0,
        0,
        0,
        0,
        initial_block_grammar_context_spec(),
        Seq::empty(),
        0,
        effective_block_limits_spec(limits),
        (structural.lexemes.len() + 1) as nat,
    );
    assert(scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits)
        == match tail {
        Err(error) => Err(error),
        Ok(scan) => Ok(
            BlockScalarSourceView {
                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                input_transformation_version: atomized.transformation_version,
                layout_transformation_version: layout.transformation_version,
                structural_transformation_version: structural.transformation_version,
                quoted_transformation_version: quoted.transformation_version,
                plain_transformation_version: plain.transformation_version,
                transformation_version: BLOCK_SCALAR_TRANSFORMATION_VERSION,
                source_len_bytes: atomized.source_len_bytes,
                bom_bytes: atomized.bom_bytes,
                input_atom_count: atomized.atoms.len() as u64,
                input_line_count: layout.lines.len() as u64,
                input_structural_lexeme_count: structural.lexemes.len() as u64,
                input_quoted_scalar_count: quoted.scalars.len() as u64,
                input_plain_scalar_count: plain.scalars.len() as u64,
                total_content_code_points: scan.total_content_code_points,
                scalars: scan.scalars,
            },
        ),
    });
    match tail {
        Err(_) => assert(false),
        Ok(scan) => {
            let expected = BlockScalarSourceView {
                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                input_transformation_version: atomized.transformation_version,
                layout_transformation_version: layout.transformation_version,
                structural_transformation_version: structural.transformation_version,
                quoted_transformation_version: quoted.transformation_version,
                plain_transformation_version: plain.transformation_version,
                transformation_version: BLOCK_SCALAR_TRANSFORMATION_VERSION,
                source_len_bytes: atomized.source_len_bytes,
                bom_bytes: atomized.bom_bytes,
                input_atom_count: atomized.atoms.len() as u64,
                input_line_count: layout.lines.len() as u64,
                input_structural_lexeme_count: structural.lexemes.len() as u64,
                input_quoted_scalar_count: quoted.scalars.len() as u64,
                input_plain_scalar_count: plain.scalars.len() as u64,
                total_content_code_points: scan.total_content_code_points,
                scalars: scan.scalars,
            };
            assert(Result::<BlockScalarSourceView, BlockScalarErrorView>::Ok(expected) == Result::<
                BlockScalarSourceView,
                BlockScalarErrorView,
            >::Ok(blocks));
            assert(expected == blocks);
        },
    }
}

pub closed spec fn block_scalar_source_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    blocks: BlockScalarSourceView,
) -> bool {
    crate::atom::atomized_source_intrinsically_well_formed_spec(atomized)
        && crate::layout::layout_source_well_formed_spec(atomized, layout)
        && crate::structural::structural_lexeme_source_well_formed_spec(
        atomized,
        layout,
        structural,
    ) && crate::quoted::quoted_scalar_source_well_formed_spec(atomized, layout, structural, quoted)
        && crate::plain::plain_scalar_source_well_formed_spec(
        atomized,
        layout,
        structural,
        quoted,
        plain,
    ) && block_scalar_source_corresponds_spec(atomized, layout, structural, quoted, plain, blocks)
        && block_scalar_ranges_well_formed_spec(atomized, layout, blocks)
}

/// Semantic block-scalar validity exposes exact raw ranges and normalized-content provenance.
pub proof fn lemma_block_well_formed_has_exact_ranges_and_content(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    blocks: BlockScalarSourceView,
)
    requires
        block_scalar_source_well_formed_spec(atomized, layout, structural, quoted, plain, blocks),
    ensures
        block_scalar_ranges_well_formed_spec(atomized, layout, blocks),
{
    reveal(block_scalar_source_well_formed_spec);
}

/// A canonical empty upstream pipeline always admits one exact empty block-scalar source.
pub proof fn lemma_empty_input_fits_block_scalar_scan_limits(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted),
        crate::plain::scan_profile1_plain_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain),
        atomized.atoms.len() == 0,
        layout.lines.len() == 0,
        structural.lexemes.len() == 0,
        quoted.scalars.len() == 0,
        plain.scalars.len() == 0,
    ensures
        exists|source: BlockScalarSourceView|
            scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits)
                == Ok(source),
{
    let scan = BlockScanView { scalars: Seq::empty(), total_content_code_points: 0 };
    assert(scan_block_tail_spec(
        atomized.atoms,
        layout.lines,
        structural.lexemes,
        quoted.scalars,
        plain.scalars,
        0,
        0,
        0,
        0,
        initial_block_grammar_context_spec(),
        Seq::empty(),
        0,
        effective_block_limits_spec(limits),
        (structural.lexemes.len() + 1) as nat,
    ) == Ok(scan)) by {
        reveal(scan_block_tail_spec);
    }
    lemma_block_scan_spec_from_tail_success(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        limits,
        scan,
    );
}

/// Canonical success on an empty structural stream contains no block-scalar ranges or content.
pub proof fn lemma_empty_block_scan_has_no_scalars(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
    blocks: BlockScalarSourceView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted),
        crate::plain::scan_profile1_plain_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain),
        structural.lexemes.len() == 0,
        scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits) == Ok(
            blocks,
        ),
    ensures
        blocks.scalars.len() == 0,
        blocks.total_content_code_points == 0,
{
    reveal(scan_profile1_block_scalars_spec);
    reveal(scan_block_tail_spec);
}

/// Empty exact success over semantic upstream evidence is a semantically valid block source.
pub proof fn lemma_empty_block_success_is_well_formed(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    limits: BlockScalarScanLimitsView,
    blocks: BlockScalarSourceView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted),
        crate::plain::scan_profile1_plain_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain),
        crate::atom::atomized_source_intrinsically_well_formed_spec(atomized),
        crate::layout::layout_source_well_formed_spec(atomized, layout),
        crate::structural::structural_lexeme_source_well_formed_spec(atomized, layout, structural),
        crate::quoted::quoted_scalar_source_well_formed_spec(atomized, layout, structural, quoted),
        crate::plain::plain_scalar_source_well_formed_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
        ),
        scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits) == Ok(
            blocks,
        ),
        blocks.scalars.len() == 0,
    ensures
        block_scalar_source_well_formed_spec(atomized, layout, structural, quoted, plain, blocks),
{
    reveal(block_scalar_source_corresponds_spec);
    assert(exists|candidate_limits: BlockScalarScanLimitsView|
        scan_profile1_block_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            candidate_limits,
        ) == Ok(blocks)) by {
        assert(scan_profile1_block_scalars_spec(atomized, layout, structural, quoted, plain, limits)
            == Ok(blocks));
    }
    lemma_block_scan_success_metadata(atomized, layout, structural, quoted, plain, limits, blocks);
    crate::layout::lemma_layout_success_input_within_atom_cap(
        atomized,
        crate::structural::canonical_layout_limits_spec(),
        layout,
    );
    assert(atomized.atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
    assert(MAX_PROFILE1_LEXICAL_ATOMS <= u64::MAX);
    assert(atomized.atoms.len() as u64 == atomized.atoms.len());
    assert(blocks.input_atom_count == atomized.atoms.len());
    reveal(block_scalar_sequence_ranges_spec);
    reveal(block_scalar_ranges_well_formed_spec);
    assert(block_scalar_ranges_well_formed_spec(atomized, layout, blocks));
    reveal(block_scalar_source_well_formed_spec);
}

#[derive(Clone, Copy)]
struct HeaderInfo {
    chomping: BlockChomping,
    explicit_indentation: Option<u8>,
    header_end_atom_index: u64,
}

impl View for HeaderInfo {
    type V = HeaderInfoView;

    closed spec fn view(&self) -> HeaderInfoView {
        HeaderInfoView {
            chomping: self.chomping,
            explicit_indentation: self.explicit_indentation,
            header_end_atom_index: self.header_end_atom_index,
        }
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (effective: u64)
    ensures
        effective <= absolute,
        effective <= requested,
        effective == requested || effective == absolute,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[allow(clippy::manual_range_contains)]
fn yaml_printable_block_character(code_point: u32) -> (printable: bool)
    ensures
        printable == crate::quoted::yaml_printable_character_spec(code_point),
{
    code_point == 0x09 || code_point == 0x0a || (0x20 <= code_point && code_point <= 0x7e)
        || code_point == 0x85 || (0xa0 <= code_point && code_point <= 0xd7ff) || (0xe000
        <= code_point && code_point <= 0xfffd) || (0x10000 <= code_point && code_point <= 0x10ffff)
}

proof fn lemma_exec_layout_line_bounds(atoms: &[LexicalAtom], lines: &[LayoutLine], index: usize)
    requires
        index < lines@.len(),
        crate::layout::layout_line_sequence_bounds_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
        ),
    ensures
        crate::layout::layout_line_bounds_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            lines[index as int]@,
        ),
{
    reveal(crate::layout::layout_line_sequence_bounds_spec);
    reveal(crate::layout::layout_line_views_spec);
    assert(crate::layout::layout_line_views_spec(lines@)[index as int] == lines[index as int]@);
}

proof fn lemma_layout_line_view_at(lines: &[LayoutLine], index: usize)
    requires
        index < lines@.len(),
    ensures
        crate::layout::layout_line_views_spec(lines@)[index as int] == lines[index as int]@,
{
    reveal(crate::layout::layout_line_views_spec);
}

proof fn lemma_atom_view_at(atoms: &[LexicalAtom], index: usize)
    requires
        index < atoms@.len(),
    ensures
        crate::atom::lexical_atom_views_spec(atoms@)[index as int] == atoms[index as int]@,
{
    reveal(crate::atom::lexical_atom_views_spec);
}

#[allow(clippy::manual_range_contains)]
fn parse_block_header(
    atoms: &[LexicalAtom],
    line: &LayoutLine,
    indicator_atom_index: usize,
) -> (result: Result<HeaderInfo, BlockScalarError>)
    requires
        crate::layout::layout_line_bounds_spec(crate::atom::lexical_atom_views_spec(atoms@), line@),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        (indicator_atom_index as u64) < line@.end_atom_index,
    ensures
        parse_block_header_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            line@,
            indicator_atom_index as int,
        ) == match result {
            Ok(header) => Ok(header@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(header) => indicator_atom_index < header.header_end_atom_index
                && header.header_end_atom_index <= atoms@.len()
                && match header.explicit_indentation {
                Some(value) => 1 <= value <= 9,
                None => true,
            },
            Err(_) => true,
        },
{
    let line_end = line.end_atom_index() as usize;
    if indicator_atom_index >= line_end || line_end > atoms.len() {
        return Err(
            BlockScalarError::at(BlockScalarErrorKind::InvalidBlockHeader, line.byte_start()),
        );
    }
    let mut index = indicator_atom_index + 1;
    let mut explicit_indentation: Option<u8> = None;
    let mut chomping = BlockChomping::Clip;
    let mut saw_chomping = false;
    let mut separated = false;
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected_tail = parse_block_header_tail_spec(
        atom_views,
        indicator_atom_index as int + 1,
        line@.end_atom_index as int,
        None,
        BlockChomping::Clip,
        false,
        false,
        (line@.end_atom_index as int - indicator_atom_index as int - 1) as nat,
    );
    let ghost expected_result = parse_block_header_spec(
        atom_views,
        line@,
        indicator_atom_index as int,
    );
    proof {
        reveal(parse_block_header_spec);
        assert(expected_result == match expected_tail {
            Err(error) => Err(error),
            Ok((expected_chomping, expected_indentation)) => if !line@.terminated {
                Err(
                    block_error_spec(
                        BlockScalarErrorKind::MissingBlockHeaderLineBreak,
                        line@.byte_end,
                    ),
                )
            } else {
                Ok(
                    HeaderInfoView {
                        chomping: expected_chomping,
                        explicit_indentation: expected_indentation,
                        header_end_atom_index: (line@.end_atom_index + 1) as u64,
                    },
                )
            },
        });
    }
    while index < line_end
        invariant
            indicator_atom_index < index <= line_end <= atoms@.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            expected_result == parse_block_header_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                line@,
                indicator_atom_index as int,
            ),
            expected_tail == parse_block_header_tail_spec(
                atom_views,
                index as int,
                line_end as int,
                explicit_indentation,
                chomping,
                saw_chomping,
                separated,
                (line_end - index) as nat,
            ),
            match explicit_indentation {
                Some(value) => 1 <= value <= 9,
                None => true,
            },
            expected_result == match expected_tail {
                Err(error) => Err(error),
                Ok((expected_chomping, expected_indentation)) => if !line@.terminated {
                    Err(
                        block_error_spec(
                            BlockScalarErrorKind::MissingBlockHeaderLineBreak,
                            line@.byte_end,
                        ),
                    )
                } else {
                    Ok(
                        HeaderInfoView {
                            chomping: expected_chomping,
                            explicit_indentation: expected_indentation,
                            header_end_atom_index: (line@.end_atom_index + 1) as u64,
                        },
                    )
                },
            },
        decreases line_end - index,
    {
        let atom = &atoms[index];
        let code_point = atom.code_point();
        assert(atom_views[index as int] == atom@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        proof {
            reveal(parse_block_header_tail_spec);
        }
        if separated {
            if code_point == 0x20 || code_point == 0x09 {
                index += 1;
                continue;
            }
            if code_point == 0x23 {
                assert(expected_tail == Ok((chomping, explicit_indentation)));
                index = line_end;
                continue;
            }
            let error = BlockScalarError::at(
                BlockScalarErrorKind::InvalidBlockHeader,
                atom.span().start().byte_offset(),
            );
            assert(expected_tail == Err(error@));
            assert(expected_result == Err(error@));
            assert(parse_block_header_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                line@,
                indicator_atom_index as int,
            ) == Err(error@));
            return Err(error);
        }
        if code_point == 0x20 || code_point == 0x09 {
            separated = true;
            index += 1;
            continue;
        }
        if code_point == 0x30 {
            let error = BlockScalarError::at(
                BlockScalarErrorKind::InvalidIndentationIndicator,
                atom.span().start().byte_offset(),
            );
            assert(expected_tail == Err(error@));
            assert(expected_result == Err(error@));
            assert(parse_block_header_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                line@,
                indicator_atom_index as int,
            ) == Err(error@));
            return Err(error);
        }
        if 0x31 <= code_point && code_point <= 0x39 {
            if explicit_indentation.is_some() {
                let error = BlockScalarError::at(
                    BlockScalarErrorKind::InvalidBlockHeader,
                    atom.span().start().byte_offset(),
                );
                assert(expected_tail == Err(error@));
                assert(expected_result == Err(error@));
                assert(parse_block_header_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    line@,
                    indicator_atom_index as int,
                ) == Err(error@));
                return Err(error);
            }
            explicit_indentation = Some((code_point - 0x30) as u8);
            index += 1;
            continue;
        }
        if code_point == 0x2b || code_point == 0x2d {
            if saw_chomping {
                let error = BlockScalarError::at(
                    BlockScalarErrorKind::InvalidBlockHeader,
                    atom.span().start().byte_offset(),
                );
                assert(expected_tail == Err(error@));
                assert(expected_result == Err(error@));
                assert(parse_block_header_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    line@,
                    indicator_atom_index as int,
                ) == Err(error@));
                return Err(error);
            }
            saw_chomping = true;
            chomping =
            if code_point == 0x2b {
                BlockChomping::Keep
            } else {
                BlockChomping::Strip
            };
            index += 1;
            continue;
        }
        let error = BlockScalarError::at(
            BlockScalarErrorKind::InvalidBlockHeader,
            atom.span().start().byte_offset(),
        );
        assert(expected_tail == Err(error@));
        assert(expected_result == Err(error@));
        assert(parse_block_header_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            line@,
            indicator_atom_index as int,
        ) == Err(error@));
        return Err(error);
    }
    assert(expected_tail == Ok((chomping, explicit_indentation))) by {
        reveal(parse_block_header_tail_spec);
    }
    if !line.is_terminated() {
        let error = BlockScalarError::at(
            BlockScalarErrorKind::MissingBlockHeaderLineBreak,
            line.byte_end(),
        );
        assert(expected_result == Err(error@));
        assert(parse_block_header_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            line@,
            indicator_atom_index as int,
        ) == Err(error@));
        return Err(error);
    }
    let header = HeaderInfo {
        chomping,
        explicit_indentation,
        header_end_atom_index: line.end_atom_index() + 1,
    };
    assert(expected_result == Ok(header@));
    assert(parse_block_header_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        line@,
        indicator_atom_index as int,
    ) == Ok(header@));
    Ok(header)
}

fn scalar_contains_atom(start: u64, end: u64, atom_index: u64) -> (contains: bool)
    ensures
        contains == (start <= atom_index < end),
{
    start <= atom_index && atom_index < end
}

fn candidate_is_inside_quote(
    quotes: &[crate::QuotedScalar],
    quote_index: &mut usize,
    atom_index: u64,
) -> (inside: bool)
    requires
        *quote_index <= quotes@.len(),
    ensures
        *final(quote_index) <= quotes@.len(),
        *final(quote_index) as int == quoted_index_after_atom_spec(
            crate::quoted::quoted_scalar_views_spec(quotes@),
            *old(quote_index) as int,
            atom_index,
            (quotes@.len() - *old(quote_index)) as nat,
        ),
        inside == (*final(quote_index) < quotes@.len()
            && quotes[*final(quote_index) as int]@.start_atom_index <= atom_index
            < quotes[*final(quote_index) as int]@.end_atom_index),
{
    let ghost views = crate::quoted::quoted_scalar_views_spec(quotes@);
    let ghost start_index = *quote_index;
    let ghost expected = quoted_index_after_atom_spec(
        views,
        start_index as int,
        atom_index,
        (quotes@.len() - start_index) as nat,
    );
    while *quote_index < quotes.len() && quotes[*quote_index].end_atom_index() <= atom_index
        invariant
            *quote_index <= quotes@.len(),
            views == crate::quoted::quoted_scalar_views_spec(quotes@),
            start_index <= *quote_index,
            expected == quoted_index_after_atom_spec(
                views,
                *quote_index as int,
                atom_index,
                (quotes@.len() - *quote_index) as nat,
            ),
        decreases quotes.len() - *quote_index,
    {
        proof {
            reveal(quoted_index_after_atom_spec);
            reveal(crate::quoted::quoted_scalar_views_spec);
        }
        *quote_index += 1;
    }
    proof {
        reveal(quoted_index_after_atom_spec);
        reveal(crate::quoted::quoted_scalar_views_spec);
    }
    let inside = if *quote_index < quotes.len() {
        scalar_contains_atom(
            quotes[*quote_index].start_atom_index(),
            quotes[*quote_index].end_atom_index(),
            atom_index,
        )
    } else {
        false
    };
    proof {
        if *quote_index < quotes@.len() {
            reveal(crate::quoted::quoted_scalar_views_spec);
        }
    }
    inside
}

fn candidate_is_inside_plain(
    plains: &[crate::PlainScalar],
    plain_index: &mut usize,
    atom_index: u64,
) -> (inside: bool)
    requires
        *plain_index <= plains@.len(),
    ensures
        *final(plain_index) <= plains@.len(),
        *final(plain_index) as int == plain_index_after_atom_spec(
            crate::plain::plain_scalar_views_spec(plains@),
            *old(plain_index) as int,
            atom_index,
            (plains@.len() - *old(plain_index)) as nat,
        ),
        inside == (*final(plain_index) < plains@.len()
            && plains[*final(plain_index) as int]@.start_atom_index <= atom_index
            < plains[*final(plain_index) as int]@.end_atom_index),
{
    let ghost views = crate::plain::plain_scalar_views_spec(plains@);
    let ghost start_index = *plain_index;
    let ghost expected = plain_index_after_atom_spec(
        views,
        start_index as int,
        atom_index,
        (plains@.len() - start_index) as nat,
    );
    while *plain_index < plains.len() && plains[*plain_index].end_atom_index() <= atom_index
        invariant
            *plain_index <= plains@.len(),
            views == crate::plain::plain_scalar_views_spec(plains@),
            start_index <= *plain_index,
            expected == plain_index_after_atom_spec(
                views,
                *plain_index as int,
                atom_index,
                (plains@.len() - *plain_index) as nat,
            ),
        decreases plains.len() - *plain_index,
    {
        proof {
            reveal(plain_index_after_atom_spec);
            reveal(crate::plain::plain_scalar_views_spec);
        }
        *plain_index += 1;
    }
    proof {
        reveal(plain_index_after_atom_spec);
        reveal(crate::plain::plain_scalar_views_spec);
    }
    let inside = if *plain_index < plains.len() {
        scalar_contains_atom(
            plains[*plain_index].start_atom_index(),
            plains[*plain_index].end_atom_index(),
            atom_index,
        )
    } else {
        false
    };
    proof {
        if *plain_index < plains@.len() {
            reveal(crate::plain::plain_scalar_views_spec);
        }
    }
    inside
}

fn logical_content_start(line: &LayoutLine, content_indentation: u64) -> (start: usize)
    requires
        line@.start_atom_index <= line@.content_atom_index,
        line@.content_atom_index <= line@.end_atom_index,
        line@.end_atom_index <= usize::MAX,
        line@.indentation_columns == line@.content_atom_index - line@.start_atom_index,
    ensures
        line@.start_atom_index <= start <= line@.end_atom_index,
        start as u64 == logical_content_start_spec(line@, content_indentation),
{
    let line_end = line.end_atom_index() as usize;
    if content_indentation <= line.indentation_columns() {
        let candidate_u64 = line.start_atom_index() + content_indentation;
        let candidate = candidate_u64 as usize;
        if candidate < line_end {
            candidate
        } else {
            line_end
        }
    } else {
        line_end
    }
}

fn push_source_content(
    content: &mut Vec<BlockScalarContentScalar>,
    atoms: &[LexicalAtom],
    atom_index: usize,
    code_point: u32,
    origin: BlockScalarContentOrigin,
)
    requires
        atom_index < atoms@.len(),
    ensures
        block_content_views_spec(final(content)@) == block_content_views_spec(old(content)@).push(
            BlockScalarContentScalarView {
                code_point,
                source_atom_index: atom_index as u64,
                byte_start: atoms[atom_index as int]@.span.start.byte_offset,
                byte_end: atoms[atom_index as int]@.span.end.byte_offset,
                origin,
            },
        ),
{
    let atom = &atoms[atom_index];
    let scalar = BlockScalarContentScalar::new(
        code_point,
        atom_index as u64,
        atom.span().start().byte_offset(),
        atom.span().end().byte_offset(),
        origin,
    );
    proof {
        lemma_atom_view_at(atoms, atom_index);
        lemma_block_content_views_push(content@, scalar);
    }
    content.push(scalar);
}

fn push_line_break(
    content: &mut Vec<BlockScalarContentScalar>,
    atoms: &[LexicalAtom],
    line: &LayoutLine,
    folded: bool,
)
    requires
        crate::layout::layout_line_bounds_spec(crate::atom::lexical_atom_views_spec(atoms@), line@),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        block_content_views_spec(final(content)@) == block_content_views_spec(old(content)@)
            + line_break_content_spec(crate::atom::lexical_atom_views_spec(atoms@), line@, folded),
{
    let ghost before = block_content_views_spec(content@);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    if line.is_terminated() {
        assert(line@.end_atom_index < atoms@.len());
        assert(atoms@.len() <= usize::MAX);
        let atom_index = line.end_atom_index() as usize;
        let ghost expected_scalar = BlockScalarContentScalarView {
            code_point: if folded {
                0x20
            } else {
                0x0a
            },
            source_atom_index: atom_index as u64,
            byte_start: atoms[atom_index as int]@.span.start.byte_offset,
            byte_end: atoms[atom_index as int]@.span.end.byte_offset,
            origin: if folded {
                BlockScalarContentOrigin::FoldedLineBreak
            } else {
                BlockScalarContentOrigin::Direct
            },
        };
        push_source_content(
            content,
            atoms,
            atom_index,
            if folded {
                0x20
            } else {
                0x0a
            },
            if folded {
                BlockScalarContentOrigin::FoldedLineBreak
            } else {
                BlockScalarContentOrigin::Direct
            },
        );
        proof {
            reveal(line_break_content_spec);
            lemma_atom_view_at(atoms, atom_index);
            assert(atom_views[atom_index as int] == atoms[atom_index as int]@);
            assert(line@.end_atom_index == atom_index as u64);
            assert(block_content_views_spec(content@) == before.push(expected_scalar));
            assert(line_break_content_spec(atom_views, line@, folded) == Seq::empty().push(
                expected_scalar,
            ));
            assert(before.push(expected_scalar) =~= before + Seq::empty().push(expected_scalar));
            assert(block_content_views_spec(content@) =~= before + line_break_content_spec(
                atom_views,
                line@,
                folded,
            ));
        }
    } else {
        assert(block_content_views_spec(content@) == before);
        reveal(line_break_content_spec);
    }
    assert(block_content_views_spec(content@) =~= before + line_break_content_spec(
        atom_views,
        line@,
        folded,
    ));
}

fn append_line_break_range(
    content: &mut Vec<BlockScalarContentScalar>,
    atoms: &[LexicalAtom],
    lines: &[LayoutLine],
    start: usize,
    end: usize,
)
    requires
        start <= end <= lines@.len(),
        crate::layout::layout_line_sequence_bounds_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
        ),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        block_content_views_spec(final(content)@) == block_content_views_spec(old(content)@)
            + line_break_range_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
{
    let ghost old_views = block_content_views_spec(content@);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost line_views = crate::layout::layout_line_views_spec(lines@);
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= lines@.len(),
            crate::layout::layout_line_sequence_bounds_spec(atom_views, line_views),
            atom_views.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            block_content_views_spec(content@) == old_views + line_break_range_spec(
                atom_views,
                line_views,
                start as int,
                index as int,
                (index - start) as nat,
            ),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            line_views == crate::layout::layout_line_views_spec(lines@),
        decreases end - index,
    {
        proof {
            lemma_exec_layout_line_bounds(atoms, lines, index);
            lemma_layout_line_view_at(lines, index);
            lemma_line_break_range_snoc(atom_views, line_views, start as int, index as int);
        }
        push_line_break(content, atoms, &lines[index], false);
        index += 1;
    }
    assert(line_break_range_spec(
        atom_views,
        line_views,
        start as int,
        end as int,
        (end - start) as nat,
    ) == line_break_range_spec(
        atom_views,
        line_views,
        start as int,
        index as int,
        (index - start) as nat,
    ));
}

fn append_direct_content(
    content: &mut Vec<BlockScalarContentScalar>,
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
) -> (result: Result<(), BlockScalarError>)
    requires
        start <= end <= atoms@.len(),
    ensures
        match result {
            Err(error) => direct_content_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ) == Err(error@),
            Ok(()) => exists|expected: Seq<BlockScalarContentScalarView>|
                direct_content_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    start as int,
                    end as int,
                    (end - start) as nat,
                ) == Ok(expected) && block_content_views_spec(final(content)@)
                    == block_content_views_spec(old(content)@) + expected,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost old_content = block_content_views_spec(content@);
    let ghost expected = direct_content_spec(
        atom_views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let ghost mut built: Seq<BlockScalarContentScalarView> = Seq::empty();
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == direct_content_spec(
                atom_views,
                start as int,
                end as int,
                (end - start) as nat,
            ),
            block_content_views_spec(content@) == old_content + built,
            expected == match direct_content_spec(
                atom_views,
                index as int,
                end as int,
                (end - index) as nat,
            ) {
                Err(error) => Err(error),
                Ok(tail) => Ok(built + tail),
            },
        decreases end - index,
    {
        let code_point = atoms[index].code_point();
        if !yaml_printable_block_character(code_point) || code_point == 0xfeff {
            let error = BlockScalarError::at(
                BlockScalarErrorKind::InvalidBlockCharacter,
                atoms[index].span().start().byte_offset(),
            );
            proof {
                lemma_atom_view_at(atoms, index);
                reveal(direct_content_spec);
                assert(expected == Err(error@));
                assert(direct_content_spec(
                    atom_views,
                    start as int,
                    end as int,
                    (end - start) as nat,
                ) == expected);
                assert(atom_views == crate::atom::lexical_atom_views_spec(atoms@));
                assert(direct_content_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    start as int,
                    end as int,
                    (end - start) as nat,
                ) == Err(error@));
            }
            return Err(error);
        }
        let ghost scalar_view = BlockScalarContentScalarView {
            code_point,
            source_atom_index: index as u64,
            byte_start: atoms[index as int]@.span.start.byte_offset,
            byte_end: atoms[index as int]@.span.end.byte_offset,
            origin: BlockScalarContentOrigin::Direct,
        };
        let ghost old_built = built;
        push_source_content(content, atoms, index, code_point, BlockScalarContentOrigin::Direct);
        proof {
            lemma_atom_view_at(atoms, index);
            reveal(direct_content_spec);
            built = old_built.push(scalar_view);
            assert(old_built + Seq::empty().push(scalar_view) =~= built);
            assert forall|tail: Seq<BlockScalarContentScalarView>|
                old_built + (Seq::empty().push(scalar_view) + tail) =~= built + tail by {};
            assert(expected == match direct_content_spec(
                atom_views,
                index as int + 1,
                end as int,
                (end - index - 1) as nat,
            ) {
                Err(error) => Err(error),
                Ok(tail) => Ok(built + tail),
            });
            assert(block_content_views_spec(content@) =~= old_content + built);
        }
        index += 1;
    }
    proof {
        reveal(direct_content_spec);
        assert(expected == Ok(built + Seq::empty()));
        assert(block_content_views_spec(content@) =~= old_content + built);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn append_block_gap(
    content: &mut Vec<BlockScalarContentScalar>,
    atoms: &[LexicalAtom],
    lines: &[LayoutLine],
    start_line: usize,
    line_index: usize,
    content_indentation: u64,
    style: BlockScalarStyle,
    previous_nonempty: Option<usize>,
)
    requires
        start_line <= line_index < lines@.len(),
        crate::layout::layout_line_sequence_bounds_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
        ),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        logical_content_start_spec(lines[line_index as int]@, content_indentation)
            < lines[line_index as int]@.end_atom_index,
        match previous_nonempty {
            Some(index) => start_line <= index < line_index && logical_content_start_spec(
                lines[index as int]@,
                content_indentation,
            ) < lines[index as int]@.end_atom_index,
            None => true,
        },
    ensures
        block_content_views_spec(final(content)@) == block_content_views_spec(old(content)@)
            + block_gap_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
            start_line as int,
            line_index as int,
            content_indentation,
            style,
            match previous_nonempty {
                Some(index) => Some(index as int),
                None => None,
            },
        ),
{
    let ghost before = block_content_views_spec(content@);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost line_views = crate::layout::layout_line_views_spec(lines@);
    proof {
        lemma_exec_layout_line_bounds(atoms, lines, line_index);
        lemma_layout_line_view_at(lines, line_index);
    }
    let line = &lines[line_index];
    let content_start = logical_content_start(line, content_indentation);
    proof {
        lemma_atom_view_at(atoms, content_start);
    }
    let more_indented = atoms[content_start].kind() == LexicalAtomKind::Space
        || atoms[content_start].kind() == LexicalAtomKind::Tab;
    match previous_nonempty {
        None => {
            append_line_break_range(content, atoms, lines, start_line, line_index);
        },
        Some(previous_index) => {
            proof {
                lemma_exec_layout_line_bounds(atoms, lines, previous_index);
                lemma_layout_line_view_at(lines, previous_index);
            }
            let previous_line = &lines[previous_index];
            let previous_start = logical_content_start(previous_line, content_indentation);
            proof {
                lemma_atom_view_at(atoms, previous_start);
            }
            let previous_more_indented = atoms[previous_start].kind() == LexicalAtomKind::Space
                || atoms[previous_start].kind() == LexicalAtomKind::Tab;
            if style == BlockScalarStyle::Folded && !previous_more_indented && !more_indented {
                if line_index == previous_index + 1 {
                    push_line_break(content, atoms, previous_line, true);
                } else {
                    append_line_break_range(content, atoms, lines, previous_index + 1, line_index);
                }
            } else {
                push_line_break(content, atoms, previous_line, false);
                append_line_break_range(content, atoms, lines, previous_index + 1, line_index);
            }
        },
    }
    proof {
        reveal(block_gap_spec);
        assert(block_content_views_spec(content@) =~= before + block_gap_spec(
            atom_views,
            line_views,
            start_line as int,
            line_index as int,
            content_indentation,
            style,
            match previous_nonempty {
                Some(index) => Some(index as int),
                None => None,
            },
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn render_block_content(
    atoms: &[LexicalAtom],
    lines: &[LayoutLine],
    start_line: usize,
    end_line: usize,
    content_indentation: u64,
    style: BlockScalarStyle,
    chomping: BlockChomping,
    last_nonempty: Option<usize>,
) -> (result: Result<Vec<BlockScalarContentScalar>, BlockScalarError>)
    requires
        crate::layout::layout_line_sequence_bounds_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
        ),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        start_line <= end_line <= lines@.len(),
        match last_nonempty {
            Some(index) => start_line <= index < end_line,
            None => true,
        },
    ensures
        render_block_content_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
            start_line as int,
            end_line as int,
            content_indentation,
            style,
            chomping,
            match last_nonempty {
                Some(index) => Some(index as int),
                None => None,
            },
        ) == match result {
            Ok(content) => Ok(block_content_views_spec(content@)),
            Err(error) => Err(error@),
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost line_views = crate::layout::layout_line_views_spec(lines@);
    let ghost expected = render_block_content_spec(
        atom_views,
        line_views,
        start_line as int,
        end_line as int,
        content_indentation,
        style,
        chomping,
        match last_nonempty {
            Some(index) => Some(index as int),
            None => None,
        },
    );
    let mut content: Vec<BlockScalarContentScalar> = Vec::new();
    let last = match last_nonempty {
        None => {
            proof {
                reveal(render_block_content_spec);
            }
            if chomping == BlockChomping::Keep {
                append_line_break_range(&mut content, atoms, lines, start_line, end_line);
                assert(block_content_views_spec(content@) == Seq::empty() + line_break_range_spec(
                    atom_views,
                    line_views,
                    start_line as int,
                    end_line as int,
                    (end_line - start_line) as nat,
                ));
            }
            assert(expected == Ok(block_content_views_spec(content@)));
            return Ok(content);
        },
        Some(index) => index,
    };
    assert(last_nonempty == Some(last));

    proof {
        reveal(render_block_content_spec);
        assert(expected == render_block_tail_spec(
            atom_views,
            line_views,
            start_line as int,
            start_line as int,
            last as int,
            end_line as int,
            content_indentation,
            style,
            chomping,
            None,
            Seq::empty(),
            (last + 1 - start_line) as nat,
        ));
    }

    let mut previous_nonempty: Option<usize> = None;
    let mut line_index = start_line;
    while line_index <= last
        invariant
            start_line <= line_index,
            last_nonempty == Some(last),
            last < end_line <= lines@.len(),
            line_index <= last + 1,
            match previous_nonempty {
                Some(index) => start_line <= index < line_index && logical_content_start_spec(
                    crate::layout::layout_line_views_spec(lines@)[index as int],
                    content_indentation,
                ) < crate::layout::layout_line_views_spec(lines@)[index as int].end_atom_index,
                None => true,
            },
            crate::layout::layout_line_sequence_bounds_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::layout::layout_line_views_spec(lines@),
            ),
            atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            line_views == crate::layout::layout_line_views_spec(lines@),
            expected == render_block_content_spec(
                atom_views,
                line_views,
                start_line as int,
                end_line as int,
                content_indentation,
                style,
                chomping,
                Some(last as int),
            ),
            expected == render_block_tail_spec(
                atom_views,
                line_views,
                start_line as int,
                line_index as int,
                last as int,
                end_line as int,
                content_indentation,
                style,
                chomping,
                match previous_nonempty {
                    Some(index) => Some(index as int),
                    None => None,
                },
                block_content_views_spec(content@),
                (last + 1 - line_index) as nat,
            ),
        decreases last + 1 - line_index,
    {
        proof {
            lemma_exec_layout_line_bounds(atoms, lines, line_index);
            lemma_layout_line_view_at(lines, line_index);
            reveal(render_block_tail_spec);
        }
        let line = &lines[line_index];
        let content_start = logical_content_start(line, content_indentation);
        let line_end = line.end_atom_index() as usize;
        let ghost previous_view = match previous_nonempty {
            Some(index) => Some(index as int),
            None => None,
        };
        if content_start < line_end {
            let ghost content_before_gap = block_content_views_spec(content@);
            append_block_gap(
                &mut content,
                atoms,
                lines,
                start_line,
                line_index,
                content_indentation,
                style,
                previous_nonempty,
            );
            let ghost gap = block_gap_spec(
                atom_views,
                line_views,
                start_line as int,
                line_index as int,
                content_indentation,
                style,
                previous_view,
            );
            assert(block_content_views_spec(content@) == content_before_gap + gap);
            match append_direct_content(&mut content, atoms, content_start, line_end) {
                Err(error) => {
                    assert(direct_content_spec(
                        atom_views,
                        content_start as int,
                        line_end as int,
                        (line_end - content_start) as nat,
                    ) == Err(error@));
                    assert(expected == Err(error@));
                    assert(render_block_content_spec(
                        atom_views,
                        line_views,
                        start_line as int,
                        end_line as int,
                        content_indentation,
                        style,
                        chomping,
                        Some(last as int),
                    ) == Err(error@));
                    assert(last_nonempty == Some(last));
                    assert(render_block_content_spec(
                        crate::atom::lexical_atom_views_spec(atoms@),
                        crate::layout::layout_line_views_spec(lines@),
                        start_line as int,
                        end_line as int,
                        content_indentation,
                        style,
                        chomping,
                        match last_nonempty {
                            Some(index) => Some(index as int),
                            None => None,
                        },
                    ) == Err(error@));
                    return Err(error);
                },
                Ok(()) => {
                    let ghost direct = choose|direct: Seq<BlockScalarContentScalarView>|
                        direct_content_spec(
                            atom_views,
                            content_start as int,
                            line_end as int,
                            (line_end - content_start) as nat,
                        ) == Ok(direct) && block_content_views_spec(content@) == (content_before_gap
                            + gap) + direct;
                    assert(direct_content_spec(
                        atom_views,
                        content_start as int,
                        line_end as int,
                        (line_end - content_start) as nat,
                    ) == Ok(direct));
                    assert(block_content_views_spec(content@) =~= content_before_gap + gap
                        + direct);
                },
            }
            previous_nonempty = Some(line_index);
            assert(expected == render_block_tail_spec(
                atom_views,
                line_views,
                start_line as int,
                line_index as int + 1,
                last as int,
                end_line as int,
                content_indentation,
                style,
                chomping,
                Some(line_index as int),
                block_content_views_spec(content@),
                (last - line_index) as nat,
            ));
        } else {
            assert(expected == render_block_tail_spec(
                atom_views,
                line_views,
                start_line as int,
                line_index as int + 1,
                last as int,
                end_line as int,
                content_indentation,
                style,
                chomping,
                previous_view,
                block_content_views_spec(content@),
                (last - line_index) as nat,
            ));
        }
        line_index += 1;
    }

    let ghost content_before_chomping = block_content_views_spec(content@);
    proof {
        reveal(render_block_tail_spec);
        assert(expected == Ok(
            content_before_chomping + if chomping == BlockChomping::Strip {
                Seq::empty()
            } else if chomping == BlockChomping::Clip {
                line_break_content_spec(atom_views, line_views[last as int], false)
            } else {
                line_break_content_spec(atom_views, line_views[last as int], false)
                    + line_break_range_spec(
                    atom_views,
                    line_views,
                    last as int + 1,
                    end_line as int,
                    (end_line - last - 1) as nat,
                )
            },
        ));
    }
    if chomping == BlockChomping::Clip || chomping == BlockChomping::Keep {
        proof {
            lemma_exec_layout_line_bounds(atoms, lines, last);
        }
        push_line_break(&mut content, atoms, &lines[last], false);
    }
    if chomping == BlockChomping::Keep {
        append_line_break_range(&mut content, atoms, lines, last + 1, end_line);
    }
    proof {
        assert(block_content_views_spec(content@) =~= content_before_chomping + if chomping
            == BlockChomping::Strip {
            Seq::empty()
        } else if chomping == BlockChomping::Clip {
            line_break_content_spec(atom_views, line_views[last as int], false)
        } else {
            line_break_content_spec(atom_views, line_views[last as int], false)
                + line_break_range_spec(
                atom_views,
                line_views,
                last as int + 1,
                end_line as int,
                (end_line - last - 1) as nat,
            )
        });
        assert(expected == Ok(block_content_views_spec(content@)));
    }
    Ok(content)
}

#[verifier::rlimit(180)]
#[verifier::spinoff_prover]
#[allow(clippy::manual_map)]
fn build_block_scalar(
    atoms: &[LexicalAtom],
    lines: &[LayoutLine],
    candidate: &StructuralLexeme,
    style: BlockScalarStyle,
    parent_indentation: u64,
) -> (result: Result<(BlockScalar, usize), BlockScalarError>)
    requires
        crate::layout::layout_line_sequence_bounds_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
        ),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        candidate@.line_number < lines@.len(),
        candidate@.start_atom_index < candidate@.end_atom_index,
        candidate@.end_atom_index <= atoms@.len(),
        lines[candidate@.line_number as int]@.start_atom_index <= candidate@.start_atom_index,
        candidate@.start_atom_index < lines[candidate@.line_number as int]@.end_atom_index,
    ensures
        build_block_scalar_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::layout::layout_line_views_spec(lines@),
            candidate@,
            style,
            parent_indentation,
        ) == match result {
            Ok((scalar, end_line)) => Ok((scalar@, end_line as int)),
            Err(error) => Err(error@),
        },
        match result {
            Ok((scalar, _)) => (crate::structural::structural_candidate_range_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                candidate@,
            ) && candidate@.kind == if style == BlockScalarStyle::Literal {
                StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar)
            } else {
                StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar)
            }) ==> block_scalar_range_and_content_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::layout::layout_line_views_spec(lines@),
                scalar@,
            ),
            Err(_) => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost line_views = crate::layout::layout_line_views_spec(lines@);
    let ghost expected_build = build_block_scalar_spec(
        atom_views,
        line_views,
        candidate@,
        style,
        parent_indentation,
    );
    let line_index = candidate.line_number() as usize;
    if line_index >= lines.len() {
        assert(false);
        return Err(
            BlockScalarError::at(BlockScalarErrorKind::InputPlainMismatch, candidate.byte_start()),
        );
    }
    if parent_indentation > MAX_PROFILE1_LEXICAL_ATOMS - 9 {
        assert(expected_build == Err(
            BlockScalarErrorView {
                kind: BlockScalarErrorKind::InputPlainMismatch,
                byte_offset: candidate@.byte_start,
            },
        )) by {
            reveal(build_block_scalar_spec);
        }
        return Err(
            BlockScalarError::at(BlockScalarErrorKind::InputPlainMismatch, candidate.byte_start()),
        );
    }
    let header_line = &lines[line_index];
    proof {
        lemma_exec_layout_line_bounds(atoms, lines, line_index);
    }
    let indicator_atom_index = candidate.start_atom_index() as usize;
    let header = match parse_block_header(atoms, header_line, indicator_atom_index) {
        Ok(header) => header,
        Err(error) => {
            proof {
                reveal(build_block_scalar_spec);
                assert(expected_build == Err(error@));
            }
            return Err(error);
        },
    };
    assert(parent_indentation <= MAX_PROFILE1_LEXICAL_ATOMS - 9);
    let start_line = line_index + 1;
    let mut probe = start_line;
    let mut longest_leading_empty = 0u64;
    let mut first_nonempty: Option<usize> = None;
    let mut first_nonempty_indentation = 0u64;
    let mut provisional_boundary = lines.len();
    let explicit_content_indentation = match header.explicit_indentation {
        Some(value) => Some(parent_indentation + value as u64),
        None => None,
    };
    proof {
        reveal(crate::layout::layout_line_views_spec);
        reveal(crate::atom::lexical_atom_views_spec);
        assert(line_views.len() == lines@.len());
        assert(atom_views.len() == atoms@.len());
    }
    let ghost expected_probe = block_probe_spec(
        atom_views,
        line_views,
        start_line as int,
        parent_indentation,
        explicit_content_indentation,
        0,
        (lines@.len() - start_line) as nat,
    );
    proof {
        reveal(build_block_scalar_spec);
        assert(expected_build == match expected_probe {
            Err(error) => Err(error),
            Ok(probe) => build_from_probe_spec(
                atom_views,
                line_views,
                candidate@,
                style,
                header@,
                parent_indentation,
                start_line as int,
                explicit_content_indentation,
                probe,
            ),
        });
    }

    loop
        invariant_except_break
            first_nonempty.is_none(),
            first_nonempty_indentation == 0,
            provisional_boundary == lines.len(),
        invariant
            start_line <= probe <= lines@.len(),
            crate::layout::layout_line_sequence_bounds_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::layout::layout_line_views_spec(lines@),
            ),
            longest_leading_empty <= atoms@.len(),
            first_nonempty.is_some() ==> start_line <= first_nonempty.unwrap() < lines@.len(),
            first_nonempty.is_some() ==> first_nonempty_indentation > parent_indentation,
            start_line <= provisional_boundary <= lines@.len(),
            expected_probe == block_probe_spec(
                atom_views,
                line_views,
                probe as int,
                parent_indentation,
                explicit_content_indentation,
                longest_leading_empty,
                (lines@.len() - probe) as nat,
            ),
            line_views.len() == lines@.len(),
            atom_views.len() == atoms@.len(),
            line_views == crate::layout::layout_line_views_spec(lines@),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            expected_build == build_block_scalar_spec(
                atom_views,
                line_views,
                candidate@,
                style,
                parent_indentation,
            ),
            expected_build == match expected_probe {
                Err(error) => Err(error),
                Ok(probe) => build_from_probe_spec(
                    atom_views,
                    line_views,
                    candidate@,
                    style,
                    header@,
                    parent_indentation,
                    start_line as int,
                    explicit_content_indentation,
                    probe,
                ),
            },
        ensures
            expected_probe == Ok(
                BlockProbeView {
                    first_nonempty: match first_nonempty {
                        Some(index) => Some(index as int),
                        None => None,
                    },
                    first_nonempty_indentation,
                    longest_leading_empty,
                    provisional_boundary: provisional_boundary as int,
                },
            ),
        decreases lines.len() - probe,
    {
        if probe >= lines.len() {
            let ghost finished = BlockProbeView {
                first_nonempty: None,
                first_nonempty_indentation: 0,
                longest_leading_empty,
                provisional_boundary: lines@.len() as int,
            };
            assert(expected_probe == Ok(finished)) by {
                reveal(block_probe_spec);
            }
            assert(expected_probe == Ok(
                BlockProbeView {
                    first_nonempty: match first_nonempty {
                        Some(index) => Some(index as int),
                        None => None,
                    },
                    first_nonempty_indentation,
                    longest_leading_empty,
                    provisional_boundary: provisional_boundary as int,
                },
            ));
            break;
        }
        proof {
            lemma_exec_layout_line_bounds(atoms, lines, probe);
        }
        let line = &lines[probe];
        assert(line@ == lines[probe as int]@);
        proof {
            lemma_layout_line_view_at(lines, probe);
        }
        assert(line_views[probe as int] == line@);
        proof {
            reveal(block_probe_spec);
            reveal(line_all_space_spec);
        }
        let all_space = line.content_atom_index() == line.end_atom_index();
        if all_space {
            if line.indentation_columns() > longest_leading_empty {
                longest_leading_empty = line.indentation_columns();
            }
            probe += 1;
            continue;
        }
        let content_atom_index = line.content_atom_index() as usize;
        proof {
            lemma_atom_view_at(atoms, content_atom_index);
        }
        assert(atom_views[content_atom_index as int] == atoms[content_atom_index as int]@);
        let first_kind = atoms[content_atom_index].kind();
        let indentation = line.indentation_columns();
        match explicit_content_indentation {
            Some(required) => {
                if first_kind == LexicalAtomKind::Tab && indentation < required {
                    let error = BlockScalarError::at(
                        BlockScalarErrorKind::TabInIndentation,
                        atoms[content_atom_index].span().start().byte_offset(),
                    );
                    assert(expected_probe == Err(error@));
                    assert(expected_build == Err(error@));
                    return Err(error);
                }
                if indentation <= parent_indentation {
                    provisional_boundary = probe;
                    let ghost finished = BlockProbeView {
                        first_nonempty: None,
                        first_nonempty_indentation: 0,
                        longest_leading_empty,
                        provisional_boundary: probe as int,
                    };
                    assert(expected_probe == Ok(finished));
                    assert(expected_probe == Ok(
                        BlockProbeView {
                            first_nonempty: None,
                            first_nonempty_indentation,
                            longest_leading_empty,
                            provisional_boundary: provisional_boundary as int,
                        },
                    ));
                    break;
                }
                if indentation < required {
                    if atoms[content_atom_index].code_point() == 0x23 {
                        provisional_boundary = probe;
                        let ghost finished = BlockProbeView {
                            first_nonempty: None,
                            first_nonempty_indentation: 0,
                            longest_leading_empty,
                            provisional_boundary: probe as int,
                        };
                        assert(expected_probe == Ok(finished));
                        assert(expected_probe == Ok(
                            BlockProbeView {
                                first_nonempty: None,
                                first_nonempty_indentation,
                                longest_leading_empty,
                                provisional_boundary: provisional_boundary as int,
                            },
                        ));
                        break;
                    }
                    let error = BlockScalarError::at(
                        BlockScalarErrorKind::InvalidBlockIndentation,
                        atoms[content_atom_index].span().start().byte_offset(),
                    );
                    assert(expected_probe == Err(error@));
                    assert(expected_build == Err(error@));
                    return Err(error);
                }
            },
            None => {
                if first_kind == LexicalAtomKind::Tab && indentation <= parent_indentation {
                    let error = BlockScalarError::at(
                        BlockScalarErrorKind::TabInIndentation,
                        atoms[content_atom_index].span().start().byte_offset(),
                    );
                    assert(expected_probe == Err(error@));
                    assert(expected_build == Err(error@));
                    return Err(error);
                }
                if indentation <= parent_indentation {
                    provisional_boundary = probe;
                    let ghost finished = BlockProbeView {
                        first_nonempty: None,
                        first_nonempty_indentation: 0,
                        longest_leading_empty,
                        provisional_boundary: probe as int,
                    };
                    assert(expected_probe == Ok(finished));
                    assert(expected_probe == Ok(
                        BlockProbeView {
                            first_nonempty: None,
                            first_nonempty_indentation,
                            longest_leading_empty,
                            provisional_boundary: provisional_boundary as int,
                        },
                    ));
                    break;
                }
            },
        }
        first_nonempty = Some(probe);
        first_nonempty_indentation = indentation;
        assert(first_nonempty_indentation <= atoms.len() as u64);
        let ghost finished = BlockProbeView {
            first_nonempty: Some(probe as int),
            first_nonempty_indentation: indentation,
            longest_leading_empty,
            provisional_boundary: lines@.len() as int,
        };
        assert(expected_probe == Ok(finished));
        assert(expected_probe == Ok(
            BlockProbeView {
                first_nonempty: Some(probe as int),
                first_nonempty_indentation,
                longest_leading_empty,
                provisional_boundary: provisional_boundary as int,
            },
        ));
        break;
    }
    let ghost completed_probe = BlockProbeView {
        first_nonempty: match first_nonempty {
            Some(index) => Some(index as int),
            None => None,
        },
        first_nonempty_indentation,
        longest_leading_empty,
        provisional_boundary: provisional_boundary as int,
    };
    assert(expected_probe == Ok(completed_probe));
    assert(expected_build == build_from_probe_spec(
        atom_views,
        line_views,
        candidate@,
        style,
        header@,
        parent_indentation,
        start_line as int,
        explicit_content_indentation,
        completed_probe,
    ));

    if let Some(first_nonempty_index) = first_nonempty {
        let mut leading = start_line;
        let ghost expected_leading = validate_leading_empty_spec(
            atom_views,
            line_views,
            start_line as int,
            first_nonempty_index as int,
            first_nonempty_indentation,
            (first_nonempty_index - start_line) as nat,
        );
        while leading < first_nonempty_index
            invariant
                start_line <= leading <= first_nonempty_index,
                first_nonempty_index < lines@.len(),
                crate::layout::layout_line_sequence_bounds_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    crate::layout::layout_line_views_spec(lines@),
                ),
                expected_leading == validate_leading_empty_spec(
                    atom_views,
                    line_views,
                    leading as int,
                    first_nonempty_index as int,
                    first_nonempty_indentation,
                    (first_nonempty_index - leading) as nat,
                ),
                line_views == crate::layout::layout_line_views_spec(lines@),
                atom_views == crate::atom::lexical_atom_views_spec(atoms@),
                first_nonempty == Some(first_nonempty_index),
                completed_probe == (BlockProbeView {
                    first_nonempty: Some(first_nonempty_index as int),
                    first_nonempty_indentation,
                    longest_leading_empty,
                    provisional_boundary: provisional_boundary as int,
                }),
                expected_leading == validate_leading_empty_spec(
                    atom_views,
                    line_views,
                    start_line as int,
                    first_nonempty_index as int,
                    first_nonempty_indentation,
                    (first_nonempty_index - start_line) as nat,
                ),
                expected_build == build_from_probe_spec(
                    atom_views,
                    line_views,
                    candidate@,
                    style,
                    header@,
                    parent_indentation,
                    start_line as int,
                    explicit_content_indentation,
                    completed_probe,
                ),
                expected_build == build_block_scalar_spec(
                    atom_views,
                    line_views,
                    candidate@,
                    style,
                    parent_indentation,
                ),
            decreases first_nonempty_index - leading,
        {
            proof {
                lemma_exec_layout_line_bounds(atoms, lines, leading);
            }
            let line = &lines[leading];
            proof {
                lemma_layout_line_view_at(lines, leading);
                reveal(validate_leading_empty_spec);
            }
            assert(line_views[leading as int] == line@);
            if line.indentation_columns() > first_nonempty_indentation {
                let bad_atom = line.start_atom_index() + first_nonempty_indentation;
                if bad_atom >= atoms.len() as u64 {
                    let error = BlockScalarError::at(
                        BlockScalarErrorKind::InputPlainMismatch,
                        line.byte_start(),
                    );
                    assert(expected_leading == Err(error@));
                    proof {
                        reveal(build_from_probe_spec);
                        assert(expected_build == Err(error@));
                    }
                    return Err(error);
                }
                proof {
                    lemma_atom_view_at(atoms, bad_atom as usize);
                }
                assert(atom_views[bad_atom as int] == atoms[bad_atom as int]@);
                let error = BlockScalarError::at(
                    BlockScalarErrorKind::InvalidLeadingEmptyIndentation,
                    atoms[bad_atom as usize].span().start().byte_offset(),
                );
                assert(expected_leading == Err(error@));
                proof {
                    reveal(build_from_probe_spec);
                    assert(expected_build == Err(error@));
                }
                return Err(error);
            }
            leading += 1;
        }
        assert(expected_leading == Ok(())) by {
            reveal(validate_leading_empty_spec);
        }
        proof {
            reveal(build_from_probe_spec);
            assert(expected_build == finish_block_scalar_spec(
                atom_views,
                line_views,
                candidate@,
                style,
                header@,
                parent_indentation,
                start_line as int,
                match explicit_content_indentation {
                    Some(required) => required,
                    None => first_nonempty_indentation,
                },
                line_views.len() as int,
            ));
        }
    }
    let content_indentation = match explicit_content_indentation {
        Some(required) => required,
        None => match first_nonempty {
            Some(_) => first_nonempty_indentation,
            None => {
                let minimum = parent_indentation + 1;
                if longest_leading_empty > minimum {
                    longest_leading_empty
                } else {
                    minimum
                }
            },
        },
    };
    assert(content_indentation > parent_indentation) by {
        match explicit_content_indentation {
            Some(required) => {
                assert(required > parent_indentation);
            },
            None => match first_nonempty {
                Some(_) => {
                    assert(first_nonempty_indentation > parent_indentation);
                },
                None => {},
            },
        }
    }

    let mut end_line = if first_nonempty.is_none() {
        provisional_boundary
    } else {
        lines.len()
    };
    assert(start_line <= end_line);
    let ghost initial_end_line = end_line;
    let ghost expected_finish = finish_block_scalar_spec(
        atom_views,
        line_views,
        candidate@,
        style,
        header@,
        parent_indentation,
        start_line as int,
        content_indentation,
        initial_end_line as int,
    );
    proof {
        match first_nonempty {
            Some(_) => {
                assert(initial_end_line == lines@.len());
                assert(content_indentation == match explicit_content_indentation {
                    Some(required) => required,
                    None => first_nonempty_indentation,
                });
                assert(line_views.len() == lines@.len());
                assert(expected_build == expected_finish);
            },
            None => {
                reveal(build_from_probe_spec);
                assert(expected_build == expected_finish);
            },
        }
    }
    let mut last_nonempty: Option<usize> = None;
    let mut scan_line = start_line;
    let ghost expected_end = scan_block_end_spec(
        atom_views,
        line_views,
        start_line as int,
        parent_indentation,
        content_indentation,
        None,
        (initial_end_line - start_line) as nat,
    );
    proof {
        reveal(finish_block_scalar_spec);
        assert(expected_finish == match expected_end {
            Err(error) => Err(error),
            Ok(end) => finish_from_end_spec(
                atom_views,
                line_views,
                candidate@,
                style,
                header@,
                parent_indentation,
                start_line as int,
                content_indentation,
                initial_end_line as int,
                end,
            ),
        });
    }
    loop
        invariant_except_break
            end_line == initial_end_line,
        invariant
            start_line <= scan_line <= end_line <= initial_end_line <= lines@.len(),
            crate::layout::layout_line_sequence_bounds_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::layout::layout_line_views_spec(lines@),
            ),
            atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            last_nonempty.is_some() ==> start_line <= last_nonempty.unwrap() < scan_line,
            expected_end == scan_block_end_spec(
                atom_views,
                line_views,
                scan_line as int,
                parent_indentation,
                content_indentation,
                match last_nonempty {
                    Some(index) => Some(index as int),
                    None => None,
                },
                (initial_end_line - scan_line) as nat,
            ),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            line_views == crate::layout::layout_line_views_spec(lines@),
            expected_build == build_block_scalar_spec(
                atom_views,
                line_views,
                candidate@,
                style,
                parent_indentation,
            ),
            expected_build == expected_finish,
            expected_finish == match expected_end {
                Err(error) => Err(error),
                Ok(end) => finish_from_end_spec(
                    atom_views,
                    line_views,
                    candidate@,
                    style,
                    header@,
                    parent_indentation,
                    start_line as int,
                    content_indentation,
                    initial_end_line as int,
                    end,
                ),
            },
        ensures
            expected_end == Ok(
                BlockEndView {
                    end_line: end_line as int,
                    last_nonempty: match last_nonempty {
                        Some(index) => Some(index as int),
                        None => None,
                    },
                },
            ),
        decreases initial_end_line - scan_line,
    {
        if scan_line >= end_line {
            assert(expected_end == Ok(
                BlockEndView {
                    end_line: end_line as int,
                    last_nonempty: match last_nonempty {
                        Some(index) => Some(index as int),
                        None => None,
                    },
                },
            )) by {
                reveal(scan_block_end_spec);
            }
            break;
        }
        proof {
            lemma_exec_layout_line_bounds(atoms, lines, scan_line);
            lemma_layout_line_view_at(lines, scan_line);
            reveal(scan_block_end_spec);
            reveal(line_all_space_spec);
        }
        let line = &lines[scan_line];
        assert(line_views[scan_line as int] == line@);
        let all_space = line.content_atom_index() == line.end_atom_index();
        assert(all_space == line_all_space_spec(line@)) by {
            reveal(line_all_space_spec);
        }
        if !all_space {
            let indentation = line.indentation_columns();
            let content_atom_index = line.content_atom_index() as usize;
            proof {
                lemma_atom_view_at(atoms, content_atom_index);
            }
            assert(atom_views[content_atom_index as int] == atoms[content_atom_index as int]@);
            let first_kind = atoms[content_atom_index].kind();
            if first_kind == LexicalAtomKind::Tab && indentation < content_indentation {
                let error = BlockScalarError::at(
                    BlockScalarErrorKind::TabInIndentation,
                    atoms[content_atom_index].span().start().byte_offset(),
                );
                assert(expected_end == Err(error@));
                assert(expected_finish == Err(error@));
                assert(expected_build == Err(error@));
                return Err(error);
            }
            if indentation < content_indentation {
                if indentation <= parent_indentation {
                    end_line = scan_line;
                    assert(expected_end == Ok(
                        BlockEndView {
                            end_line: end_line as int,
                            last_nonempty: match last_nonempty {
                                Some(index) => Some(index as int),
                                None => None,
                            },
                        },
                    ));
                    break;
                }
                if atoms[content_atom_index].code_point() == 0x23 {
                    end_line = scan_line;
                    assert(expected_end == Ok(
                        BlockEndView {
                            end_line: end_line as int,
                            last_nonempty: match last_nonempty {
                                Some(index) => Some(index as int),
                                None => None,
                            },
                        },
                    ));
                    break;
                }
                let error = BlockScalarError::at(
                    BlockScalarErrorKind::InvalidBlockIndentation,
                    atoms[content_atom_index].span().start().byte_offset(),
                );
                assert(expected_end == Err(error@));
                assert(expected_finish == Err(error@));
                assert(expected_build == Err(error@));
                return Err(error);
            }
        }
        let logical_start = logical_content_start(line, content_indentation);
        assert(logical_start as u64 == logical_content_start_spec(line@, content_indentation));
        if logical_start < line.end_atom_index() as usize {
            last_nonempty = Some(scan_line);
        }
        if all_space {
            assert(expected_end == scan_block_end_spec(
                atom_views,
                line_views,
                scan_line as int + 1,
                parent_indentation,
                content_indentation,
                match last_nonempty {
                    Some(index) => Some(index as int),
                    None => None,
                },
                (initial_end_line - scan_line - 1) as nat,
            ));
        } else {
            assert(line@.content_atom_index < atom_views.len());
            assert(line@.indentation_columns >= content_indentation);
            assert(expected_end == scan_block_end_spec(
                atom_views,
                line_views,
                scan_line as int + 1,
                parent_indentation,
                content_indentation,
                match last_nonempty {
                    Some(index) => Some(index as int),
                    None => None,
                },
                (initial_end_line - scan_line - 1) as nat,
            ));
        }
        assert(expected_end == scan_block_end_spec(
            atom_views,
            line_views,
            scan_line as int + 1,
            parent_indentation,
            content_indentation,
            match last_nonempty {
                Some(index) => Some(index as int),
                None => None,
            },
            (initial_end_line - scan_line - 1) as nat,
        ));
        scan_line += 1;
    }

    let ghost completed_end = BlockEndView {
        end_line: end_line as int,
        last_nonempty: match last_nonempty {
            Some(index) => Some(index as int),
            None => None,
        },
    };
    assert(expected_end == Ok(completed_end));
    assert(expected_finish == finish_from_end_spec(
        atom_views,
        line_views,
        candidate@,
        style,
        header@,
        parent_indentation,
        start_line as int,
        content_indentation,
        initial_end_line as int,
        completed_end,
    ));
    assert(expected_build == expected_finish);

    let end_atom_index = if end_line < lines.len() {
        lines[end_line].start_atom_index()
    } else {
        atoms.len() as u64
    };
    if end_atom_index < header.header_end_atom_index || end_atom_index > atoms.len() as u64 {
        let error = BlockScalarError::at(
            BlockScalarErrorKind::InputPlainMismatch,
            candidate.byte_start(),
        );
        proof {
            reveal(finish_from_end_spec);
            assert(expected_finish == Err(error@));
            assert(expected_build == Err(error@));
        }
        return Err(error);
    }
    assert(0 < end_atom_index);
    let content = match render_block_content(
        atoms,
        lines,
        start_line,
        end_line,
        content_indentation,
        style,
        header.chomping,
        last_nonempty,
    ) {
        Ok(content) => content,
        Err(error) => {
            proof {
                reveal(finish_from_end_spec);
                assert(expected_finish == Err(error@));
                assert(expected_build == Err(error@));
            }
            return Err(error);
        },
    };
    let byte_end = atoms[(end_atom_index - 1) as usize].span().end().byte_offset();
    let end_line_number = atoms[(end_atom_index - 1) as usize].span().start().line();
    let scalar = BlockScalar {
        style,
        chomping: header.chomping,
        explicit_indentation: header.explicit_indentation,
        parent_indentation,
        content_indentation,
        start_line_number: candidate.line_number(),
        end_line_number,
        start_atom_index: candidate.start_atom_index(),
        header_end_atom_index: header.header_end_atom_index,
        content_start_atom_index: header.header_end_atom_index,
        end_atom_index,
        byte_start: candidate.byte_start(),
        byte_end,
        content,
    };
    proof {
        lemma_atom_view_at(atoms, (end_atom_index - 1) as usize);
        reveal(finish_from_end_spec);
        assert(expected_finish == Ok((scalar@, end_line as int)));
        assert(expected_build == Ok((scalar@, end_line as int)));
        if crate::structural::structural_candidate_range_spec(atom_views, candidate@)
            && candidate@.kind == if style == BlockScalarStyle::Literal {
            StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar)
        } else {
            StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar)
        } {
            reveal(block_scalar_range_and_content_spec);
            assert(render_block_content_spec(
                atom_views,
                line_views,
                start_line as int,
                end_line as int,
                content_indentation,
                style,
                header.chomping,
                match last_nonempty {
                    Some(index) => Some(index as int),
                    None => None,
                },
            ) == Ok(scalar@.content));
            assert(exists|render_end: int, render_last: Option<int>| #[trigger]
                render_block_content_spec(
                    atom_views,
                    line_views,
                    scalar@.start_line_number as int + 1,
                    render_end,
                    scalar@.content_indentation,
                    scalar@.style,
                    scalar@.chomping,
                    render_last,
                ) == Ok(scalar@.content) && scalar@.start_line_number + 1 <= render_end
                    <= line_views.len() && scalar@.end_atom_index == if render_end
                    < line_views.len() {
                    line_views[render_end].start_atom_index
                } else {
                    atom_views.len() as u64
                }) by {
                assert(start_line == scalar@.start_line_number + 1);
            }
            assert(scalar@.start_atom_index < scalar@.header_end_atom_index);
            assert(scalar@.header_end_atom_index == scalar@.content_start_atom_index);
            assert(scalar@.content_start_atom_index <= scalar@.end_atom_index);
            assert(scalar@.end_atom_index <= atom_views.len());
            assert(scalar@.content_indentation > scalar@.parent_indentation);
            assert(scalar@.byte_start
                == atom_views[scalar@.start_atom_index as int].span.start.byte_offset);
            assert(scalar@.byte_end == atom_views[(scalar@.end_atom_index
                - 1) as int].span.end.byte_offset);
            assert(scalar@.end_line_number == atom_views[(scalar@.end_atom_index
                - 1) as int].span.start.line);
            assert(atom_views[scalar@.start_atom_index as int].kind == if scalar@.style
                == BlockScalarStyle::Literal {
                LexicalAtomKind::Indicator(YamlIndicator::LiteralBlockScalar)
            } else {
                LexicalAtomKind::Indicator(YamlIndicator::FoldedBlockScalar)
            });
            assert(block_scalar_range_and_content_spec(atom_views, line_views, scalar@));
        }
    }
    Ok((scalar, end_line))
}

fn candidate_index_after_block(
    candidates: &[StructuralLexeme],
    start_index: usize,
    end_atom_index: u64,
) -> (index: usize)
    requires
        start_index <= candidates@.len(),
    ensures
        index as int == candidate_index_after_block_spec(
            crate::structural::structural_lexeme_views_spec(candidates@),
            start_index as int,
            end_atom_index,
            (candidates@.len() - start_index) as nat,
        ),
        start_index <= index <= candidates@.len(),
        index < candidates@.len() ==> candidates[index as int]@.start_atom_index >= end_atom_index,
{
    let ghost views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost expected = candidate_index_after_block_spec(
        views,
        start_index as int,
        end_atom_index,
        (candidates@.len() - start_index) as nat,
    );
    let mut index = start_index;
    while index < candidates.len() && candidates[index].start_atom_index() < end_atom_index
        invariant
            start_index <= index <= candidates@.len(),
            views == crate::structural::structural_lexeme_views_spec(candidates@),
            expected == candidate_index_after_block_spec(
                views,
                index as int,
                end_atom_index,
                (candidates@.len() - index) as nat,
            ),
        decreases candidates.len() - index,
    {
        proof {
            reveal(candidate_index_after_block_spec);
            reveal(crate::structural::structural_lexeme_views_spec);
        }
        index += 1;
    }
    proof {
        reveal(candidate_index_after_block_spec);
    }
    index
}

closed spec fn block_candidate_style_spec(role: StructuralCandidateRole) -> Option<
    BlockScalarStyle,
> {
    if role == StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar) {
        Some(BlockScalarStyle::Literal)
    } else if role == StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar) {
        Some(BlockScalarStyle::Folded)
    } else {
        None
    }
}

fn block_candidate_style(role: StructuralCandidateRole) -> (style: Option<BlockScalarStyle>)
    ensures
        style == block_candidate_style_spec(role),
{
    if role == StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar) {
        Some(BlockScalarStyle::Literal)
    } else if role == StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar) {
        Some(BlockScalarStyle::Folded)
    } else {
        None
    }
}

/// Authenticates upstream evidence and forms every complete profile-1 block scalar.
#[verifier::rlimit(1000)]
#[verifier::spinoff_prover]
#[allow(clippy::implicit_saturating_sub)]
pub fn scan_profile1_block_scalars(
    atomized: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    limits: BlockScalarScanLimits,
) -> (result: Result<BlockScalarSource, BlockScalarError>)
    ensures
        scan_profile1_block_scalars_spec(atomized@, layout@, structural@, quoted@, plain@, limits@)
            == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(source) => block_scalar_source_corresponds_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                source@,
            ) && ((crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
                && crate::structural::structural_lexeme_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
            ) && crate::quoted::quoted_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
            ) && crate::plain::plain_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
            )) ==> block_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                source@,
            )) && source@.scalars.len() <= limits@.max_scalars && source@.scalars.len()
                <= MAX_PROFILE1_BLOCK_SCALARS && source@.total_content_code_points
                <= limits@.max_total_content_code_points && source@.total_content_code_points
                <= MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
            Err(_) => true,
        },
{
    let canonical_layout = match analyze_profile1_layout(
        atomized,
        canonical_structural_layout_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = BlockScalarError::at(
                BlockScalarErrorKind::InputLayoutMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_block_scalars_spec);
            }
            return Err(mismatch);
        },
    };
    if !canonical_layout.same_as(layout) {
        let mismatch = BlockScalarError::at(BlockScalarErrorKind::InputLayoutMismatch, 0);
        proof {
            reveal(scan_profile1_block_scalars_spec);
        }
        return Err(mismatch);
    }
    proof {
        assert(canonical_layout@ == layout@);
        assert(crate::layout::layout_line_sequence_bounds_spec(atomized@.atoms, layout@.lines));
    }
    let canonical_structural = match scan_profile1_structural_lexemes(
        atomized,
        layout,
        canonical_structural_scan_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = BlockScalarError::at(
                BlockScalarErrorKind::InputStructuralMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_block_scalars_spec);
            }
            return Err(mismatch);
        },
    };
    if !canonical_structural.same_as(structural) {
        let mismatch = BlockScalarError::at(BlockScalarErrorKind::InputStructuralMismatch, 0);
        proof {
            reveal(scan_profile1_block_scalars_spec);
        }
        return Err(mismatch);
    }
    assert(canonical_structural@ == structural@);
    let canonical_quoted = match scan_profile1_quoted_scalars(
        atomized,
        layout,
        structural,
        canonical_quoted_scalar_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = BlockScalarError::at(
                BlockScalarErrorKind::InputQuotedMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_block_scalars_spec);
            }
            return Err(mismatch);
        },
    };
    if !canonical_quoted.same_as(quoted) {
        let mismatch = BlockScalarError::at(BlockScalarErrorKind::InputQuotedMismatch, 0);
        proof {
            reveal(scan_profile1_block_scalars_spec);
        }
        return Err(mismatch);
    }
    assert(canonical_quoted@ == quoted@);
    let canonical_plain = match scan_profile1_plain_scalars(
        atomized,
        layout,
        structural,
        quoted,
        canonical_plain_scalar_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = BlockScalarError::at(
                BlockScalarErrorKind::InputPlainMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_block_scalars_spec);
            }
            return Err(mismatch);
        },
    };
    if !canonical_plain.same_as(plain) {
        let mismatch = BlockScalarError::at(BlockScalarErrorKind::InputPlainMismatch, 0);
        proof {
            reveal(scan_profile1_block_scalars_spec);
        }
        return Err(mismatch);
    }
    assert(canonical_plain@ == plain@);
    proof {
        assert(crate::layout::analyze_profile1_layout_spec(
            atomized@,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(layout@));
        assert(crate::structural::scan_profile1_structural_lexemes_spec(
            atomized@,
            layout@,
            crate::structural::canonical_structural_scan_limits_spec(),
        ) == Ok(structural@));
        assert(crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized@,
            layout@,
            structural@,
            crate::quoted::canonical_quoted_scalar_limits_spec(),
        ) == Ok(quoted@));
        assert(crate::plain::scan_profile1_plain_scalars_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            crate::plain::canonical_plain_scalar_limits_spec(),
        ) == Ok(plain@));
    }

    let atoms = atomized.atoms();
    let lines = layout.lines();
    let candidates = structural.lexemes();
    let quotes = quoted.scalars();
    let plains = plain.scalars();
    proof {
        crate::layout::lemma_layout_success_input_within_atom_cap(
            atomized@,
            crate::structural::canonical_layout_limits_spec(),
            layout@,
        );
        assert(atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
        assert(candidates@.len() <= crate::structural::MAX_PROFILE1_STRUCTURAL_LEXEMES);
        assert(crate::structural::MAX_PROFILE1_STRUCTURAL_LEXEMES == MAX_PROFILE1_LEXICAL_ATOMS);
    }
    if atoms.len() as u64 > MAX_PROFILE1_LEXICAL_ATOMS {
        assert(false);
        return Err(BlockScalarError::at(BlockScalarErrorKind::InputPlainMismatch, 0));
    }
    if candidates.len() as u64 > MAX_PROFILE1_LEXICAL_ATOMS {
        assert(false);
        return Err(BlockScalarError::at(BlockScalarErrorKind::InputPlainMismatch, 0));
    }
    let scalar_limit = effective_limit(limits.max_scalars(), MAX_PROFILE1_BLOCK_SCALARS);
    let presentation_limit = effective_limit(
        limits.max_scalar_presentation_atoms(),
        MAX_PROFILE1_BLOCK_SCALAR_PRESENTATION_ATOMS,
    );
    let scalar_content_limit = effective_limit(
        limits.max_scalar_content_code_points(),
        MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS,
    );
    let total_content_limit = effective_limit(
        limits.max_total_content_code_points(),
        MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS,
    );
    proof {
        reveal(effective_block_limits_spec);
        assert(effective_block_limits_spec(limits@) == BlockScalarScanLimitsView {
            max_scalars: scalar_limit,
            max_scalar_presentation_atoms: presentation_limit,
            max_scalar_content_code_points: scalar_content_limit,
            max_total_content_code_points: total_content_limit,
        });
    }
    assert(scalar_content_limit <= MAX_PROFILE1_BLOCK_SCALAR_CONTENT_CODE_POINTS);

    let mut scalars: Vec<BlockScalar> = Vec::new();
    let mut total_content_code_points = 0u64;
    let mut candidate_index = 0usize;
    let mut quote_index = 0usize;
    let mut plain_index = 0usize;
    let mut flow_depth = 0u64;
    let mut grammar = initial_block_grammar_context();
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost line_views = crate::layout::layout_line_views_spec(lines@);
    let ghost candidate_views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost quote_views = crate::quoted::quoted_scalar_views_spec(quotes@);
    let ghost plain_views = crate::plain::plain_scalar_views_spec(plains@);
    let ghost effective_limits = effective_block_limits_spec(limits@);
    let ghost semantic_inputs = crate::atom::atomized_source_intrinsically_well_formed_spec(
        atomized@,
    ) && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
        && crate::structural::structural_lexeme_source_well_formed_spec(
        atomized@,
        layout@,
        structural@,
    ) && crate::quoted::quoted_scalar_source_well_formed_spec(
        atomized@,
        layout@,
        structural@,
        quoted@,
    ) && crate::plain::plain_scalar_source_well_formed_spec(
        atomized@,
        layout@,
        structural@,
        quoted@,
        plain@,
    );
    let ghost mut fuel: nat = (candidates@.len() + 1) as nat;
    let ghost expected = scan_block_tail_spec(
        atom_views,
        line_views,
        candidate_views,
        quote_views,
        plain_views,
        0,
        0,
        0,
        0,
        initial_block_grammar_context_spec(),
        Seq::empty(),
        0,
        effective_limits,
        fuel,
    );
    proof {
        reveal(block_scalar_views_spec);
        reveal(block_scalar_sequence_ranges_spec);
        assert(block_scalar_views_spec(scalars@) =~= Seq::<BlockScalarView>::empty());
        assert(block_scalar_sequence_ranges_spec(
            atom_views,
            line_views,
            block_scalar_views_spec(scalars@),
        ));
        assert(atom_views == atomized@.atoms);
        assert(line_views == layout@.lines);
        assert(candidate_views == structural@.lexemes);
        assert(quote_views == quoted@.scalars);
        assert(plain_views == plain@.scalars);
    }
    while candidate_index < candidates.len()
        invariant
            candidate_index <= candidates@.len(),
            quote_index <= quotes@.len(),
            plain_index <= plains@.len(),
            scalars@.len() <= scalar_limit,
            total_content_code_points <= total_content_limit,
            flow_depth <= candidate_index,
            atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            candidates@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            crate::layout::layout_line_sequence_bounds_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::layout::layout_line_views_spec(lines@),
            ),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            atom_views == atomized@.atoms,
            line_views == crate::layout::layout_line_views_spec(lines@),
            line_views == layout@.lines,
            candidate_views == crate::structural::structural_lexeme_views_spec(candidates@),
            candidate_views == structural@.lexemes,
            quote_views == crate::quoted::quoted_scalar_views_spec(quotes@),
            quote_views == quoted@.scalars,
            plain_views == crate::plain::plain_scalar_views_spec(plains@),
            plain_views == plain@.scalars,
            effective_limits == effective_block_limits_spec(limits@),
            semantic_inputs == (crate::atom::atomized_source_intrinsically_well_formed_spec(
                atomized@,
            ) && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
                && crate::structural::structural_lexeme_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
            ) && crate::quoted::quoted_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
            ) && crate::plain::plain_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
            )),
            effective_limits == (BlockScalarScanLimitsView {
                max_scalars: scalar_limit,
                max_scalar_presentation_atoms: presentation_limit,
                max_scalar_content_code_points: scalar_content_limit,
                max_total_content_code_points: total_content_limit,
            }),
            crate::layout::analyze_profile1_layout_spec(
                atomized@,
                crate::structural::canonical_layout_limits_spec(),
            ) == Ok(layout@),
            crate::structural::scan_profile1_structural_lexemes_spec(
                atomized@,
                layout@,
                crate::structural::canonical_structural_scan_limits_spec(),
            ) == Ok(structural@),
            crate::quoted::scan_profile1_quoted_scalars_spec(
                atomized@,
                layout@,
                structural@,
                crate::quoted::canonical_quoted_scalar_limits_spec(),
            ) == Ok(quoted@),
            crate::plain::scan_profile1_plain_scalars_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                crate::plain::canonical_plain_scalar_limits_spec(),
            ) == Ok(plain@),
            block_scalar_views_spec(scalars@).len() == scalars@.len(),
            semantic_inputs ==> block_scalar_sequence_ranges_spec(
                atom_views,
                line_views,
                block_scalar_views_spec(scalars@),
            ),
            semantic_inputs && scalars@.len() > 0 && candidate_index < candidates@.len()
                ==> block_scalar_views_spec(scalars@)[scalars@.len() - 1].end_atom_index
                <= candidate_views[candidate_index as int].start_atom_index,
            fuel >= candidates@.len() - candidate_index + 1,
            expected == scan_block_tail_spec(
                atomized@.atoms,
                layout@.lines,
                structural@.lexemes,
                quoted@.scalars,
                plain@.scalars,
                0,
                0,
                0,
                0,
                initial_block_grammar_context_spec(),
                Seq::empty(),
                0,
                effective_block_limits_spec(limits@),
                (structural@.lexemes.len() + 1) as nat,
            ),
            expected == scan_block_tail_spec(
                atom_views,
                line_views,
                candidate_views,
                quote_views,
                plain_views,
                candidate_index as int,
                quote_index as int,
                plain_index as int,
                flow_depth,
                grammar@,
                block_scalar_views_spec(scalars@),
                total_content_code_points,
                effective_limits,
                fuel,
            ),
        decreases candidates.len() - candidate_index,
    {
        let candidate = &candidates[candidate_index];
        assert(candidate_views[candidate_index as int] == candidate@) by {
            reveal(crate::structural::structural_lexeme_views_spec);
        }
        proof {
            if semantic_inputs && scalars@.len() > 0 && candidate_index + 1 < candidates@.len() {
                lemma_advancing_candidate_preserves_block_order(
                    atomized@,
                    layout@,
                    structural@,
                    candidate_index as int,
                    block_scalar_views_spec(scalars@)[scalars@.len() - 1].end_atom_index,
                );
            }
        }
        let atom_index = candidate.start_atom_index();
        if candidate_is_inside_quote(quotes, &mut quote_index, atom_index) {
            grammar = block_grammar_context_after_candidate(atoms, candidate, true, grammar);
            proof {
                reveal(scan_block_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if candidate_is_inside_plain(plains, &mut plain_index, atom_index) {
            grammar = block_grammar_context_after_candidate(atoms, candidate, true, grammar);
            proof {
                reveal(scan_block_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        let role = candidate.candidate_role();
        if role == StructuralCandidateRole::FlowSequenceStart || role
            == StructuralCandidateRole::FlowMappingStart {
            grammar = block_grammar_context_after_candidate(atoms, candidate, false, grammar);
            proof {
                reveal(scan_block_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            flow_depth += 1;
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::FlowSequenceEnd || role
            == StructuralCandidateRole::FlowMappingEnd {
            grammar = block_grammar_context_after_candidate(atoms, candidate, false, grammar);
            proof {
                reveal(scan_block_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            if flow_depth > 0 {
                flow_depth -= 1;
            }
            candidate_index += 1;
            continue;
        }
        let style = block_candidate_style(role);
        if style.is_none() || flow_depth > 0 {
            grammar = block_grammar_context_after_candidate(atoms, candidate, false, grammar);
            proof {
                reveal(scan_block_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if candidate.start_atom_index() >= candidate.end_atom_index() || candidate.end_atom_index()
            > atoms.len() as u64 || candidate.line_number() >= lines.len() as u64 {
            let error = BlockScalarError::at(
                BlockScalarErrorKind::InputPlainMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
                assert(scan_profile1_block_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                ) == Err(error@));
            }
            return Err(error);
        }
        let candidate_line_index = candidate.line_number() as usize;
        if candidate.start_atom_index() < lines[candidate_line_index].start_atom_index()
            || candidate.start_atom_index() >= lines[candidate_line_index].end_atom_index() {
            let error = BlockScalarError::at(
                BlockScalarErrorKind::InputPlainMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
            }
            return Err(error);
        }
        let (scalar, _) = match build_block_scalar(
            atoms,
            lines,
            candidate,
            style.unwrap(),
            grammar.parent_indentation,
        ) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    reveal(scan_block_tail_spec);
                    assert(expected == Err(error@));
                    lemma_block_scan_spec_from_tail_error(
                        atomized@,
                        layout@,
                        structural@,
                        quoted@,
                        plain@,
                        limits@,
                        error@,
                    );
                    assert(scan_profile1_block_scalars_spec(
                        atomized@,
                        layout@,
                        structural@,
                        quoted@,
                        plain@,
                        limits@,
                    ) == Err(error@));
                }
                return Err(error);
            },
        };

        if scalar.start_atom_index() >= scalar.end_atom_index() || scalar.end_atom_index()
            > atoms.len() as u64 {
            let error = BlockScalarError::at(
                BlockScalarErrorKind::InputPlainMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
                assert(scan_profile1_block_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                ) == Err(error@));
            }
            return Err(error);
        }
        if scalars.len() as u64 >= scalar_limit {
            let error = BlockScalarError::at(
                BlockScalarErrorKind::ScalarLimitExceeded,
                scalar.byte_start(),
            );
            proof {
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
            }
            return Err(error);
        }
        let presentation_atoms = scalar.end_atom_index() - scalar.start_atom_index();
        if presentation_atoms > presentation_limit {
            let excluded = scalar.start_atom_index() + presentation_limit;
            proof {
                lemma_atom_view_at(atoms, excluded as usize);
            }
            let error = BlockScalarError::at(
                BlockScalarErrorKind::PresentationAtomLimitExceeded,
                atoms[excluded as usize].span().start().byte_offset(),
            );
            proof {
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
            }
            return Err(error);
        }
        let scalar_content = scalar.content();
        let scalar_content_len = scalar_content.len();
        if scalar_content_len as u64 > scalar_content_limit {
            assert(scalar_content_limit < scalar_content_len as u64);
            assert(scalar_content_limit <= usize::MAX);
            let excluded_index = scalar_content_limit as usize;
            assert(excluded_index < scalar_content_len);
            let error = BlockScalarError::at(
                BlockScalarErrorKind::ScalarContentLimitExceeded,
                scalar_content[excluded_index].byte_start(),
            );
            proof {
                reveal(block_content_views_spec);
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
                assert(scan_profile1_block_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                ) == Err(error@));
            }
            return Err(error);
        }
        let content_count = scalar_content_len as u64;
        if content_count > total_content_limit - total_content_code_points {
            let excluded = total_content_limit - total_content_code_points;
            assert(excluded < content_count);
            assert(excluded < scalar_content_len as u64);
            assert(excluded <= usize::MAX);
            let excluded_index = excluded as usize;
            assert(excluded_index < scalar_content_len);
            let error = BlockScalarError::at(
                BlockScalarErrorKind::TotalContentLimitExceeded,
                scalar_content[excluded_index].byte_start(),
            );
            proof {
                reveal(block_content_views_spec);
                reveal(scan_block_tail_spec);
                assert(expected == Err(error@));
                lemma_block_scan_spec_from_tail_error(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                    error@,
                );
                assert(scan_profile1_block_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    plain@,
                    limits@,
                ) == Err(error@));
            }
            return Err(error);
        }
        total_content_code_points += content_count;
        let end_atom_index = scalar.end_atom_index();
        let ghost old_scalars = scalars@;
        proof {
            if semantic_inputs {
                crate::structural::lemma_structural_well_formed_has_exact_partition(
                    atomized@,
                    layout@,
                    structural@,
                );
                reveal(crate::structural::structural_lexeme_partition_spec);
                reveal(crate::structural::structural_candidate_prefix_partition_spec);
                assert(crate::structural::structural_candidate_range_spec(atom_views, candidate@));
                assert(candidate@.kind == if style.unwrap() == BlockScalarStyle::Literal {
                    StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar)
                } else {
                    StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar)
                });
                assert(block_scalar_range_and_content_spec(atom_views, line_views, scalar@));
                if old_scalars.len() > 0 {
                    let previous = block_scalar_views_spec(old_scalars)[old_scalars.len() - 1];
                    assert(previous.end_atom_index <= candidate@.start_atom_index);
                    assert(scalar@.start_atom_index == candidate@.start_atom_index);
                    assert(0 < previous.end_atom_index);
                    lemma_earlier_block_atom_ends_before_later_atom_starts(
                        atomized@,
                        previous.end_atom_index as int - 1,
                        scalar@.start_atom_index as int,
                    );
                    assert(previous.byte_end <= scalar@.byte_start);
                }
                lemma_block_scalar_sequence_push(
                    atom_views,
                    line_views,
                    block_scalar_views_spec(old_scalars),
                    scalar@,
                );
            }
            lemma_block_scalar_views_push(scalars@, scalar);
        }
        scalars.push(scalar);
        assert(candidate_index < candidates.len());
        candidate_index =
        candidate_index_after_block(candidates, candidate_index + 1, end_atom_index);
        grammar = initial_block_grammar_context();
        proof {
            reveal(scan_block_tail_spec);
            fuel = (fuel - 1) as nat;
        }
    }

    let source = BlockScalarSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version(),
        layout_transformation_version: layout.transformation_version(),
        structural_transformation_version: structural.transformation_version(),
        quoted_transformation_version: quoted.transformation_version(),
        plain_transformation_version: plain.transformation_version(),
        transformation_version: BLOCK_SCALAR_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes(),
        bom_bytes: atomized.bom_bytes(),
        input_atom_count: atoms.len() as u64,
        input_line_count: lines.len() as u64,
        input_structural_lexeme_count: candidates.len() as u64,
        input_quoted_scalar_count: quotes.len() as u64,
        input_plain_scalar_count: plains.len() as u64,
        total_content_code_points,
        scalars,
    };
    proof {
        reveal(scan_block_tail_spec);
        assert(expected == Ok(
            BlockScanView {
                scalars: source@.scalars,
                total_content_code_points: source@.total_content_code_points,
            },
        ));
        reveal(scan_profile1_block_scalars_spec);
        assert(scan_profile1_block_scalars_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            limits@,
        ) == Ok(source@));
        reveal(block_scalar_source_corresponds_spec);
        assert(exists|candidate_limits: BlockScalarScanLimitsView|
            scan_profile1_block_scalars_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                candidate_limits,
            ) == Ok(source@)) by {
            assert(scan_profile1_block_scalars_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                limits@,
            ) == Ok(source@));
        }
        assert(source@.scalars.len() <= scalar_limit);
        assert(scalar_limit <= limits@.max_scalars);
        assert(scalar_limit <= MAX_PROFILE1_BLOCK_SCALARS);
        assert(source@.total_content_code_points <= total_content_limit);
        assert(total_content_limit <= limits@.max_total_content_code_points);
        assert(total_content_limit <= MAX_PROFILE1_TOTAL_BLOCK_SCALAR_CONTENT_CODE_POINTS);
        if semantic_inputs {
            assert(block_scalar_sequence_ranges_spec(
                atomized@.atoms,
                layout@.lines,
                source@.scalars,
            ));
            reveal(block_scalar_ranges_well_formed_spec);
            assert(block_scalar_ranges_well_formed_spec(atomized@, layout@, source@));
            reveal(block_scalar_source_well_formed_spec);
            assert(block_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                source@,
            ));
        }
    }
    Ok(source)
}

} // verus!
