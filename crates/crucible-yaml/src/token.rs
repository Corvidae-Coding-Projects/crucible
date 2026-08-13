//! Verified completed YAML token formation.
//!
//! This transformation authenticates every preceding lexer slice, merges the scalar evidence into
//! one lossless token partition, validates directive/property spellings, and enforces typed flow
//! delimiter nesting without delegating any YAML work to an external parser.
use crate::atom::{AtomizedSource, LexicalAtom, LexicalAtomKind, MAX_PROFILE1_LEXICAL_ATOMS};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::block::{
    canonical_block_scalar_limits, scan_profile1_block_scalars, BlockScalarSource, BlockScalarStyle,
};
#[allow(unused_imports)]
use crate::block::{BlockScalarSourceView, BlockScalarView};
#[allow(unused_imports)]
use crate::layout::LayoutSourceView;
use crate::layout::{analyze_profile1_layout, LayoutSource};
use crate::plain::{canonical_plain_scalar_limits, scan_profile1_plain_scalars, PlainScalarSource};
#[allow(unused_imports)]
use crate::plain::{PlainScalarSourceView, PlainScalarView};
use crate::quoted::{
    canonical_quoted_scalar_limits, scan_profile1_quoted_scalars, QuotedScalarSource,
    QuotedScalarStyle,
};
#[allow(unused_imports)]
use crate::quoted::{QuotedScalarSourceView, QuotedScalarView};
#[allow(unused_imports)]
use crate::structural::StructuralLexemeSourceView;
use crate::structural::{
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    scan_profile1_structural_lexemes, StructuralLexemeSource,
};
use crate::utf8::CRUCIBLE_YAML_PROFILE_VERSION;
use crate::YamlIndicator;
use vstd::prelude::*;

verus! {

pub const COMPLETED_TOKEN_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_COMPLETED_TOKENS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_FLOW_DEPTH: u64 = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletedTokenLimits {
    max_tokens: u64,
    max_flow_depth: u64,
}

#[verifier::ext_equal]
pub struct CompletedTokenLimitsView {
    pub max_tokens: u64,
    pub max_flow_depth: u64,
}

impl View for CompletedTokenLimits {
    type V = CompletedTokenLimitsView;

    closed spec fn view(&self) -> CompletedTokenLimitsView {
        CompletedTokenLimitsView {
            max_tokens: self.max_tokens,
            max_flow_depth: self.max_flow_depth,
        }
    }
}

impl CompletedTokenLimits {
    pub fn new(max_tokens: u64, max_flow_depth: u64) -> (limits: Self)
        ensures
            limits@ == (CompletedTokenLimitsView { max_tokens, max_flow_depth }),
    {
        Self { max_tokens, max_flow_depth }
    }

    pub fn max_tokens(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_tokens,
    {
        self.max_tokens
    }

    pub fn max_flow_depth(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_flow_depth,
    {
        self.max_flow_depth
    }
}

pub open spec fn canonical_completed_token_limits_spec() -> CompletedTokenLimitsView {
    CompletedTokenLimitsView {
        max_tokens: MAX_PROFILE1_COMPLETED_TOKENS,
        max_flow_depth: MAX_PROFILE1_FLOW_DEPTH,
    }
}

pub fn canonical_completed_token_limits() -> (limits: CompletedTokenLimits)
    ensures
        limits@ == canonical_completed_token_limits_spec(),
{
    CompletedTokenLimits::new(MAX_PROFILE1_COMPLETED_TOKENS, MAX_PROFILE1_FLOW_DEPTH)
}

closed spec fn effective_token_limit_spec(limits: CompletedTokenLimitsView) -> u64 {
    if limits.max_tokens < MAX_PROFILE1_COMPLETED_TOKENS {
        limits.max_tokens
    } else {
        MAX_PROFILE1_COMPLETED_TOKENS
    }
}

closed spec fn effective_flow_depth_spec(limits: CompletedTokenLimitsView) -> u64 {
    if limits.max_flow_depth < MAX_PROFILE1_FLOW_DEPTH {
        limits.max_flow_depth
    } else {
        MAX_PROFILE1_FLOW_DEPTH
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CompletedTokenKind {
    Indentation,
    Separation,
    LineFeed,
    Comment,
    DocumentByteOrderMark,
    YamlDirective,
    TagDirective,
    ReservedDirective,
    DirectivesEnd,
    DocumentEnd,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    FlowEntry,
    BlockSequenceEntry,
    ExplicitMappingKey,
    MappingValue,
    AnchorProperty,
    TagProperty,
    VerbatimTagProperty,
    Alias,
    PlainScalar,
    SingleQuotedScalar,
    DoubleQuotedScalar,
    LiteralBlockScalar,
    FoldedBlockScalar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CompletedTokenPartKind {
    DirectiveName,
    DirectiveParameter,
    YamlMajor,
    YamlMinor,
    TagHandle,
    TagPrefix,
    TagSuffix,
    VerbatimTagPayload,
    AnchorName,
    AliasName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletedTokenPart {
    kind: CompletedTokenPartKind,
    start_atom_index: u64,
    end_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
}

#[verifier::ext_equal]
pub struct CompletedTokenPartView {
    pub kind: CompletedTokenPartKind,
    pub start_atom_index: u64,
    pub end_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
}

impl View for CompletedTokenPart {
    type V = CompletedTokenPartView;

    closed spec fn view(&self) -> CompletedTokenPartView {
        CompletedTokenPartView {
            kind: self.kind,
            start_atom_index: self.start_atom_index,
            end_atom_index: self.end_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
        }
    }
}

impl CompletedTokenPart {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        self.kind == other.kind && self.start_atom_index == other.start_atom_index
            && self.end_atom_index == other.end_atom_index && self.byte_start == other.byte_start
            && self.byte_end == other.byte_end
    }

    pub fn kind(&self) -> (kind: CompletedTokenPartKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.start_atom_index,
    {
        self.start_atom_index
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
}

pub closed spec fn completed_token_part_views_spec(parts: Seq<CompletedTokenPart>) -> Seq<
    CompletedTokenPartView,
> {
    parts.map_values(|part: CompletedTokenPart| part@)
}

pub proof fn lemma_completed_token_part_views_len(parts: Seq<CompletedTokenPart>)
    ensures
        completed_token_part_views_spec(parts).len() == parts.len(),
{
    reveal(completed_token_part_views_spec);
}

pub proof fn lemma_completed_token_part_view_at(parts: Seq<CompletedTokenPart>, index: int)
    requires
        0 <= index < parts.len(),
    ensures
        completed_token_part_views_spec(parts)[index] == parts[index]@,
{
    reveal(completed_token_part_views_spec);
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompletedToken {
    kind: CompletedTokenKind,
    start_line_number: u64,
    end_line_number: u64,
    start_atom_index: u64,
    end_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
    scalar_index: Option<u64>,
    yaml_major: Option<u64>,
    yaml_minor: Option<u64>,
    parts: Vec<CompletedTokenPart>,
}

#[verifier::ext_equal]
pub struct CompletedTokenView {
    pub kind: CompletedTokenKind,
    pub start_line_number: u64,
    pub end_line_number: u64,
    pub start_atom_index: u64,
    pub end_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub scalar_index: Option<u64>,
    pub yaml_major: Option<u64>,
    pub yaml_minor: Option<u64>,
    pub parts: Seq<CompletedTokenPartView>,
}

impl View for CompletedToken {
    type V = CompletedTokenView;

    closed spec fn view(&self) -> CompletedTokenView {
        CompletedTokenView {
            kind: self.kind,
            start_line_number: self.start_line_number,
            end_line_number: self.end_line_number,
            start_atom_index: self.start_atom_index,
            end_atom_index: self.end_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            scalar_index: self.scalar_index,
            yaml_major: self.yaml_major,
            yaml_minor: self.yaml_minor,
            parts: completed_token_part_views_spec(self.parts@),
        }
    }
}

impl CompletedToken {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.kind != other.kind || self.start_line_number != other.start_line_number
            || self.end_line_number != other.end_line_number || self.start_atom_index
            != other.start_atom_index || self.end_atom_index != other.end_atom_index
            || self.byte_start != other.byte_start || self.byte_end != other.byte_end
            || self.scalar_index != other.scalar_index || self.yaml_major != other.yaml_major
            || self.yaml_minor != other.yaml_minor {
            return false;
        }
        if self.parts.len() != other.parts.len() {
            proof {
                reveal(completed_token_part_views_spec);
                assert(self@.parts.len() != other@.parts.len());
            }
            return false;
        }
        let mut index = 0usize;
        while index < self.parts.len()
            invariant
                self.parts.len() == other.parts.len(),
                index <= self.parts.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < index ==> self.parts[prior]@ == other.parts[prior]@,
            decreases self.parts.len() - index,
        {
            if !self.parts[index].same_as(&other.parts[index]) {
                proof {
                    reveal(completed_token_part_views_spec);
                    assert(self@.parts[index as int] != other@.parts[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(completed_token_part_views_spec);
            assert(self@.parts =~= other@.parts);
        }
        true
    }

    pub fn kind(&self) -> (kind: CompletedTokenKind)
        ensures
            kind == self@.kind,
    {
        self.kind
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

    pub fn scalar_index(&self) -> (index: Option<u64>)
        ensures
            index == self@.scalar_index,
    {
        self.scalar_index
    }

    pub fn yaml_version(&self) -> (version: Option<(u64, u64)>)
        ensures
            version == match (self@.yaml_major, self@.yaml_minor) {
                (Some(major), Some(minor)) => Some((major, minor)),
                _ => None,
            },
    {
        match (self.yaml_major, self.yaml_minor) {
            (Some(major), Some(minor)) => Some((major, minor)),
            _ => None,
        }
    }

    pub fn parts(&self) -> (parts: &[CompletedTokenPart])
        ensures
            completed_token_part_views_spec(parts@) == self@.parts,
    {
        self.parts.as_slice()
    }
}

pub closed spec fn completed_token_views_spec(tokens: Seq<CompletedToken>) -> Seq<
    CompletedTokenView,
> {
    tokens.map_values(|token: CompletedToken| token@)
}

pub proof fn lemma_completed_token_view_at(tokens: Seq<CompletedToken>, index: int)
    requires
        0 <= index < tokens.len(),
    ensures
        completed_token_views_spec(tokens)[index] == tokens[index]@,
{
    reveal(completed_token_views_spec);
}

pub proof fn lemma_completed_token_views_len(tokens: Seq<CompletedToken>)
    ensures
        completed_token_views_spec(tokens).len() == tokens.len(),
{
    reveal(completed_token_views_spec);
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompletedTokenSource {
    profile_version: u16,
    input_transformation_version: u16,
    layout_transformation_version: u16,
    structural_transformation_version: u16,
    quoted_transformation_version: u16,
    plain_transformation_version: u16,
    block_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    input_atom_count: u64,
    maximum_flow_depth: u64,
    tokens: Vec<CompletedToken>,
}

#[verifier::ext_equal]
pub struct CompletedTokenSourceView {
    pub profile_version: u16,
    pub input_transformation_version: u16,
    pub layout_transformation_version: u16,
    pub structural_transformation_version: u16,
    pub quoted_transformation_version: u16,
    pub plain_transformation_version: u16,
    pub block_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub input_atom_count: u64,
    pub maximum_flow_depth: u64,
    pub tokens: Seq<CompletedTokenView>,
}

impl View for CompletedTokenSource {
    type V = CompletedTokenSourceView;

    closed spec fn view(&self) -> CompletedTokenSourceView {
        CompletedTokenSourceView {
            profile_version: self.profile_version,
            input_transformation_version: self.input_transformation_version,
            layout_transformation_version: self.layout_transformation_version,
            structural_transformation_version: self.structural_transformation_version,
            quoted_transformation_version: self.quoted_transformation_version,
            plain_transformation_version: self.plain_transformation_version,
            block_transformation_version: self.block_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            input_atom_count: self.input_atom_count,
            maximum_flow_depth: self.maximum_flow_depth,
            tokens: completed_token_views_spec(self.tokens@),
        }
    }
}

impl CompletedTokenSource {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.profile_version != other.profile_version || self.input_transformation_version
            != other.input_transformation_version || self.layout_transformation_version
            != other.layout_transformation_version || self.structural_transformation_version
            != other.structural_transformation_version || self.quoted_transformation_version
            != other.quoted_transformation_version || self.plain_transformation_version
            != other.plain_transformation_version || self.block_transformation_version
            != other.block_transformation_version || self.transformation_version
            != other.transformation_version || self.source_len_bytes != other.source_len_bytes
            || self.bom_bytes != other.bom_bytes || self.input_atom_count != other.input_atom_count
            || self.maximum_flow_depth != other.maximum_flow_depth {
            return false;
        }
        if self.tokens.len() != other.tokens.len() {
            proof {
                reveal(completed_token_views_spec);
                assert(self@.tokens.len() != other@.tokens.len());
            }
            return false;
        }
        let mut index = 0usize;
        while index < self.tokens.len()
            invariant
                self.tokens.len() == other.tokens.len(),
                index <= self.tokens.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < index ==> self.tokens[prior]@ == other.tokens[prior]@,
            decreases self.tokens.len() - index,
        {
            if !self.tokens[index].same_as(&other.tokens[index]) {
                proof {
                    reveal(completed_token_views_spec);
                    assert(self@.tokens[index as int] != other@.tokens[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(completed_token_views_spec);
            assert(self@.tokens =~= other@.tokens);
        }
        true
    }

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

    pub fn input_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.input_transformation_version,
    {
        self.input_transformation_version
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

    pub fn maximum_flow_depth(&self) -> (depth: u64)
        ensures
            depth == self@.maximum_flow_depth,
    {
        self.maximum_flow_depth
    }

    pub fn tokens(&self) -> (tokens: &[CompletedToken])
        ensures
            completed_token_views_spec(tokens@) == self@.tokens,
    {
        self.tokens.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CompletedTokenErrorKind {
    InputLayoutMismatch,
    InputStructuralMismatch,
    InputQuotedMismatch,
    InputPlainMismatch,
    InputBlockMismatch,
    TokenLimitExceeded,
    FlowDepthLimitExceeded,
    MismatchedFlowEnd,
    UnexpectedFlowEnd,
    UnclosedFlowCollection,
    InvalidYamlDirective,
    InvalidTagDirective,
    EmptyDirectiveName,
    EmptyAnchorName,
    EmptyAliasName,
    InvalidAnchorCharacter,
    InvalidAliasCharacter,
    InvalidDirectiveCharacter,
    EmptyVerbatimTag,
    UnterminatedVerbatimTag,
    InvalidVerbatimTag,
    EmptyTagSuffix,
    InvalidTagCharacter,
    InvalidTagPercentEscape,
    ReservedIndicator,
    TabInIndentation,
    UnexpectedIndicator,
    UnexpectedContent,
    InputScalarOverlap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletedTokenError {
    kind: CompletedTokenErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CompletedTokenErrorView {
    pub kind: CompletedTokenErrorKind,
    pub byte_offset: u64,
}

impl View for CompletedTokenError {
    type V = CompletedTokenErrorView;

    closed spec fn view(&self) -> CompletedTokenErrorView {
        CompletedTokenErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CompletedTokenError {
    fn at(kind: CompletedTokenErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == token_error_spec(kind, byte_offset),
            error@ == (CompletedTokenErrorView { kind, byte_offset }),
    {
        proof {
            reveal(token_error_spec);
        }
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: CompletedTokenErrorKind)
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

pub open spec fn completed_token_range_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
) -> bool {
    token.start_atom_index < token.end_atom_index && token.end_atom_index <= atoms.len()
        && token.byte_start == atoms[token.start_atom_index as int].span.start.byte_offset
        && token.byte_end == atoms[(token.end_atom_index - 1) as int].span.end.byte_offset
        && token.start_line_number == atoms[token.start_atom_index as int].span.start.line
        && token.end_line_number == atoms[(token.end_atom_index - 1) as int].span.start.line
}

pub open spec fn completed_token_sequence_partition_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
) -> bool {
    forall|index: int|
        0 <= index < tokens.len() ==> #[trigger] completed_token_range_spec(atoms, tokens[index])
            && (index > 0 ==> tokens[index - 1].end_atom_index == tokens[index].start_atom_index
            && tokens[index - 1].byte_end == tokens[index].byte_start)
}

pub open spec fn completed_token_prefix_partition_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
    consumed_atoms: int,
) -> bool {
    0 <= consumed_atoms <= atoms.len() && tokens.len() <= consumed_atoms
        && completed_token_sequence_partition_spec(atoms, tokens) && if tokens.len() == 0 {
        consumed_atoms == 0
    } else {
        tokens[0].start_atom_index == 0 && tokens[tokens.len() - 1].end_atom_index == consumed_atoms
    }
}

pub open spec fn completed_token_partition_spec(
    atomized: AtomizedSourceView,
    source: CompletedTokenSourceView,
) -> bool {
    source.source_len_bytes == atomized.source_len_bytes && source.bom_bytes == atomized.bom_bytes
        && source.input_atom_count == atomized.atoms.len()
        && completed_token_sequence_partition_spec(atomized.atoms, source.tokens)
        && if atomized.atoms.len() == 0 {
        source.tokens.len() == 0 && atomized.source_len_bytes == atomized.bom_bytes
    } else {
        source.tokens.len() > 0 && source.tokens[0].start_atom_index == 0
            && source.tokens[0].byte_start == atomized.bom_bytes
            && source.tokens[source.tokens.len() - 1].end_atom_index == atomized.atoms.len()
            && source.tokens[source.tokens.len() - 1].byte_end == atomized.source_len_bytes
    }
}

pub open spec fn completed_token_flow_tail_spec(
    tokens: Seq<CompletedTokenView>,
    index: int,
    stack: Seq<CompletedTokenKind>,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if index < 0 || index > tokens.len() || tokens.len() - index > fuel {
        false
    } else if index == tokens.len() {
        stack.len() == 0
    } else {
        let kind = tokens[index].kind;
        if kind == CompletedTokenKind::FlowSequenceStart || kind
            == CompletedTokenKind::FlowMappingStart {
            completed_token_flow_tail_spec(tokens, index + 1, stack.push(kind), (fuel - 1) as nat)
        } else if kind == CompletedTokenKind::FlowSequenceEnd {
            stack.len() > 0 && stack.last() == CompletedTokenKind::FlowSequenceStart
                && completed_token_flow_tail_spec(
                tokens,
                index + 1,
                stack.drop_last(),
                (fuel - 1) as nat,
            )
        } else if kind == CompletedTokenKind::FlowMappingEnd {
            stack.len() > 0 && stack.last() == CompletedTokenKind::FlowMappingStart
                && completed_token_flow_tail_spec(
                tokens,
                index + 1,
                stack.drop_last(),
                (fuel - 1) as nat,
            )
        } else {
            completed_token_flow_tail_spec(tokens, index + 1, stack, (fuel - 1) as nat)
        }
    }
}

pub open spec fn completed_token_flow_stack_after_kind_spec(
    stack: Seq<CompletedTokenKind>,
    kind: CompletedTokenKind,
) -> Option<Seq<CompletedTokenKind>> {
    if kind == CompletedTokenKind::FlowSequenceStart || kind
        == CompletedTokenKind::FlowMappingStart {
        Some(stack.push(kind))
    } else if kind == CompletedTokenKind::FlowSequenceEnd {
        if stack.len() > 0 && stack.last() == CompletedTokenKind::FlowSequenceStart {
            Some(stack.drop_last())
        } else {
            None
        }
    } else if kind == CompletedTokenKind::FlowMappingEnd {
        if stack.len() > 0 && stack.last() == CompletedTokenKind::FlowMappingStart {
            Some(stack.drop_last())
        } else {
            None
        }
    } else if kind == CompletedTokenKind::FlowEntry && stack.len() == 0 {
        None
    } else {
        Some(stack)
    }
}

pub open spec fn completed_token_flow_prefix_spec(
    tokens: Seq<CompletedTokenView>,
    stack: Seq<CompletedTokenKind>,
) -> bool {
    if tokens.len() == 0 {
        stack.len() == 0
    } else {
        exists|states: Seq<Seq<CompletedTokenKind>>|
            states.len() == tokens.len() + 1 && states[0].len() == 0 && states[tokens.len() as int]
                == stack && forall|index: int|
                0 <= index < tokens.len() ==> completed_token_flow_stack_after_kind_spec(
                    #[trigger] states[index],
                    tokens[index].kind,
                ) == Some(states[index + 1])
    }
}

pub open spec fn completed_token_flow_balanced_spec(tokens: Seq<CompletedTokenView>) -> bool {
    completed_token_flow_prefix_spec(tokens, Seq::empty())
}

pub open spec fn completed_token_part_range_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
    part: CompletedTokenPartView,
) -> bool {
    token.start_atom_index <= part.start_atom_index < part.end_atom_index <= token.end_atom_index
        && part.end_atom_index <= atoms.len() && part.byte_start
        == atoms[part.start_atom_index as int].span.start.byte_offset && part.byte_end == atoms[(
    part.end_atom_index - 1) as int].span.end.byte_offset
}

pub open spec fn completed_token_parts_schema_spec(token: CompletedTokenView) -> bool {
    if token.kind == CompletedTokenKind::YamlDirective {
        token.parts.len() == 3 && token.parts[0].kind == CompletedTokenPartKind::DirectiveName
            && token.parts[1].kind == CompletedTokenPartKind::YamlMajor && token.parts[2].kind
            == CompletedTokenPartKind::YamlMinor
    } else if token.kind == CompletedTokenKind::TagDirective {
        token.parts.len() == 3 && token.parts[0].kind == CompletedTokenPartKind::DirectiveName
            && token.parts[1].kind == CompletedTokenPartKind::TagHandle && token.parts[2].kind
            == CompletedTokenPartKind::TagPrefix
    } else if token.kind == CompletedTokenKind::ReservedDirective {
        token.parts.len() > 0 && token.parts[0].kind == CompletedTokenPartKind::DirectiveName
            && forall|index: int|
            1 <= index < token.parts.len() ==> token.parts[index].kind
                == CompletedTokenPartKind::DirectiveParameter
    } else if token.kind == CompletedTokenKind::AnchorProperty {
        token.parts.len() == 1 && token.parts[0].kind == CompletedTokenPartKind::AnchorName
    } else if token.kind == CompletedTokenKind::Alias {
        token.parts.len() == 1 && token.parts[0].kind == CompletedTokenPartKind::AliasName
    } else if token.kind == CompletedTokenKind::VerbatimTagProperty {
        token.parts.len() == 1 && token.parts[0].kind == CompletedTokenPartKind::VerbatimTagPayload
    } else if token.kind == CompletedTokenKind::TagProperty {
        token.parts.len() == 0 || token.parts.len() == 2 && token.parts[0].kind
            == CompletedTokenPartKind::TagHandle && token.parts[1].kind
            == CompletedTokenPartKind::TagSuffix
    } else {
        token.parts.len() == 0
    }
}

pub open spec fn completed_token_parts_well_formed_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
) -> bool {
    completed_token_parts_schema_spec(token) && forall|index: int|
        0 <= index < token.parts.len() ==> #[trigger] completed_token_part_range_spec(
            atoms,
            token,
            token.parts[index],
        ) && (index > 0 ==> token.parts[index - 1].end_atom_index
            <= token.parts[index].start_atom_index && token.parts[index - 1].byte_end
            <= token.parts[index].byte_start)
}

pub open spec fn completed_token_scalar_identity_spec(
    quoted: Seq<QuotedScalarView>,
    plain: Seq<PlainScalarView>,
    block: Seq<BlockScalarView>,
    token: CompletedTokenView,
) -> bool {
    if token.kind == CompletedTokenKind::PlainScalar {
        token.scalar_index.is_some() && token.scalar_index.unwrap() < plain.len() && {
            let scalar = plain[token.scalar_index.unwrap() as int];
            token.start_atom_index == scalar.start_atom_index && token.end_atom_index
                == scalar.end_atom_index && token.byte_start == scalar.byte_start && token.byte_end
                == scalar.byte_end
        }
    } else if token.kind == CompletedTokenKind::SingleQuotedScalar || token.kind
        == CompletedTokenKind::DoubleQuotedScalar {
        token.scalar_index.is_some() && token.scalar_index.unwrap() < quoted.len() && {
            let scalar = quoted[token.scalar_index.unwrap() as int];
            token.start_atom_index == scalar.start_atom_index && token.end_atom_index
                == scalar.end_atom_index && token.byte_start == scalar.byte_start && token.byte_end
                == scalar.byte_end && (token.kind == CompletedTokenKind::SingleQuotedScalar
                <==> scalar.style == QuotedScalarStyle::Single)
        }
    } else if token.kind == CompletedTokenKind::LiteralBlockScalar || token.kind
        == CompletedTokenKind::FoldedBlockScalar {
        token.scalar_index.is_some() && token.scalar_index.unwrap() < block.len() && {
            let scalar = block[token.scalar_index.unwrap() as int];
            token.start_atom_index == scalar.start_atom_index && token.end_atom_index
                == scalar.end_atom_index && token.byte_start == scalar.byte_start && token.byte_end
                == scalar.byte_end && (token.kind == CompletedTokenKind::LiteralBlockScalar
                <==> scalar.style == BlockScalarStyle::Literal)
        }
    } else {
        token.scalar_index.is_none()
    }
}

pub open spec fn completed_token_trivia_maximal_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
) -> bool {
    if token.kind == CompletedTokenKind::Indentation || token.kind
        == CompletedTokenKind::Separation {
        token.start_atom_index < token.end_atom_index <= atoms.len() && (forall|index: int|
            token.start_atom_index <= index < token.end_atom_index ==> token_is_space_or_tab_spec(
                #[trigger] atoms[index].kind,
            )) && (token.end_atom_index == atoms.len() || !token_is_space_or_tab_spec(
            atoms[token.end_atom_index as int].kind,
        ))
    } else if token.kind == CompletedTokenKind::Comment {
        token.start_atom_index < token.end_atom_index <= atoms.len() && atoms[{
            token.start_atom_index as int
        }].kind == LexicalAtomKind::Indicator(YamlIndicator::Comment) && (forall|index: int|
            token.start_atom_index <= index < token.end_atom_index ==> #[trigger] atoms[index].kind
                != LexicalAtomKind::LineFeed) && (token.end_atom_index == atoms.len()
            || atoms[token.end_atom_index as int].kind == LexicalAtomKind::LineFeed)
    } else if token.kind == CompletedTokenKind::LineFeed {
        token.start_atom_index < token.end_atom_index <= atoms.len() && token.end_atom_index
            == token.start_atom_index + 1 && atoms[token.start_atom_index as int].kind
            == LexicalAtomKind::LineFeed
    } else {
        true
    }
}

pub open spec fn completed_token_absolute_limits_spec(source: CompletedTokenSourceView) -> bool {
    source.tokens.len() <= MAX_PROFILE1_COMPLETED_TOKENS && source.maximum_flow_depth
        <= MAX_PROFILE1_FLOW_DEPTH
}

proof fn lemma_empty_completed_token_prefix(atoms: Seq<LexicalAtomView>)
    ensures
        completed_token_prefix_partition_spec(atoms, Seq::empty(), 0),
{
    reveal(completed_token_prefix_partition_spec);
    reveal(completed_token_sequence_partition_spec);
}

proof fn lemma_empty_completed_flow_prefix()
    ensures
        completed_token_flow_prefix_spec(Seq::empty(), Seq::empty()),
{
    reveal(completed_token_flow_prefix_spec);
}

proof fn lemma_extend_completed_flow_prefix(
    tokens: Seq<CompletedTokenView>,
    stack: Seq<CompletedTokenKind>,
    token: CompletedTokenView,
    next_stack: Seq<CompletedTokenKind>,
)
    requires
        completed_token_flow_prefix_spec(tokens, stack),
        completed_token_flow_stack_after_kind_spec(stack, token.kind) == Some(next_stack),
    ensures
        completed_token_flow_prefix_spec(tokens.push(token), next_stack),
{
    reveal(completed_token_flow_prefix_spec);
    let states = if tokens.len() == 0 {
        assert(stack.len() == 0);
        Seq::empty().push(Seq::<CompletedTokenKind>::empty())
    } else {
        choose|states: Seq<Seq<CompletedTokenKind>>|
            states.len() == tokens.len() + 1 && states[0].len() == 0 && states[tokens.len() as int]
                == stack && forall|index: int|
                0 <= index < tokens.len() ==> completed_token_flow_stack_after_kind_spec(
                    #[trigger] states[index],
                    tokens[index].kind,
                ) == Some(states[index + 1])
    };
    assert(states.len() == tokens.len() + 1);
    assert(states[0].len() == 0);
    assert(states[tokens.len() as int] == stack);
    let extended = states.push(next_stack);
    assert(extended.len() == tokens.push(token).len() + 1);
    assert(extended[0].len() == 0);
    assert(extended[tokens.push(token).len() as int] == next_stack);
    assert forall|index: int|
        0 <= index < tokens.push(token).len() implies completed_token_flow_stack_after_kind_spec(
        #[trigger] extended[index],
        tokens.push(token)[index].kind,
    ) == Some(extended[index + 1]) by {
        if index < tokens.len() {
            assert(extended[index] == states[index]);
            assert(extended[index + 1] == states[index + 1]);
            assert(tokens.push(token)[index] == tokens[index]);
        } else {
            assert(index == tokens.len());
            assert(extended[index] == stack);
            assert(extended[index + 1] == next_stack);
            assert(tokens.push(token)[index] == token);
        }
    }
    assert(exists|witness: Seq<Seq<CompletedTokenKind>>|
        witness.len() == tokens.push(token).len() + 1 && witness[0].len() == 0
            && witness[tokens.push(token).len() as int] == next_stack && forall|index: int|
            0 <= index < tokens.push(token).len() ==> completed_token_flow_stack_after_kind_spec(
                #[trigger] witness[index],
                tokens.push(token)[index].kind,
            ) == Some(witness[index + 1])) by {
        assert(extended.len() == tokens.push(token).len() + 1);
    }
}

#[verifier::spinoff_prover]
#[verifier::rlimit(80)]
proof fn lemma_extend_completed_token_prefix(
    atoms: Seq<LexicalAtomView>,
    built: Seq<CompletedTokenView>,
    token: CompletedTokenView,
    consumed_atoms: int,
)
    requires
        completed_token_prefix_partition_spec(atoms, built, consumed_atoms),
        completed_token_range_spec(atoms, token),
        token.start_atom_index == consumed_atoms,
        forall|atom_index: int|
            0 < atom_index < atoms.len() ==> atoms[atom_index - 1].span.end
                == atoms[atom_index].span.start,
    ensures
        completed_token_prefix_partition_spec(
            atoms,
            built.push(token),
            token.end_atom_index as int,
        ),
{
    reveal(completed_token_prefix_partition_spec);
    reveal(completed_token_sequence_partition_spec);
    reveal(completed_token_range_spec);
    if built.len() > 0 {
        assert(built[built.len() - 1].end_atom_index == consumed_atoms);
        assert(completed_token_range_spec(atoms, built[built.len() - 1]));
        assert(0 < consumed_atoms < atoms.len());
        assert(built[built.len() - 1].byte_end == atoms[consumed_atoms - 1].span.end.byte_offset);
        assert(token.byte_start == atoms[consumed_atoms].span.start.byte_offset);
        assert(atoms[consumed_atoms - 1].span.end == atoms[consumed_atoms].span.start);
    }
    assert forall|index: int|
        0 <= index < built.push(token).len() implies completed_token_range_spec(
        atoms,
        #[trigger] built.push(token)[index],
    ) && (index > 0 ==> built.push(token)[index - 1].end_atom_index == built.push(
        token,
    )[index].start_atom_index && built.push(token)[index - 1].byte_end == built.push(
        token,
    )[index].byte_start) by {
        if index < built.len() {
            assert(built.push(token)[index] == built[index]);
            if index > 0 {
                assert(built.push(token)[index - 1] == built[index - 1]);
            }
        } else {
            assert(index == built.len());
            assert(built.push(token)[index] == token);
            if index > 0 {
                assert(built.push(token)[index - 1] == built[built.len() - 1]);
            }
        }
    }
}

closed spec fn token_error_spec(
    kind: CompletedTokenErrorKind,
    byte_offset: u64,
) -> CompletedTokenErrorView {
    CompletedTokenErrorView { kind, byte_offset }
}

closed spec fn token_offset_at_or_end_spec(atoms: Seq<LexicalAtomView>, index: int) -> u64 {
    if 0 <= index < atoms.len() {
        atoms[index].span.start.byte_offset
    } else if atoms.len() > 0 {
        atoms.last().span.end.byte_offset
    } else {
        0
    }
}

pub open spec fn token_is_space_or_tab_spec(kind: LexicalAtomKind) -> bool {
    kind == LexicalAtomKind::Space || kind == LexicalAtomKind::Tab
}

closed spec fn token_is_flow_indicator_spec(kind: LexicalAtomKind) -> bool {
    kind == LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd)
}

closed spec fn token_is_hex_spec(code_point: u32) -> bool {
    (0x30 <= code_point && code_point <= 0x39) || (0x41 <= code_point && code_point <= 0x46) || (
    0x61 <= code_point && code_point <= 0x66)
}

pub open spec fn token_is_word_spec(code_point: u32) -> bool {
    is_decimal_spec(code_point) || (0x41 <= code_point && code_point <= 0x5a) || (0x61 <= code_point
        && code_point <= 0x7a) || code_point == 0x2d
}

pub open spec fn token_is_ns_uri_char_spec(code_point: u32) -> bool {
    token_is_word_spec(code_point) || code_point == 0x25 || code_point == 0x23 || code_point == 0x3b
        || code_point == 0x2f || code_point == 0x3f || code_point == 0x3a || code_point == 0x40
        || code_point == 0x26 || code_point == 0x3d || code_point == 0x2b || code_point == 0x24
        || code_point == 0x2c || code_point == 0x5f || code_point == 0x2e || code_point == 0x21
        || code_point == 0x7e || code_point == 0x2a || code_point == 0x27 || code_point == 0x28
        || code_point == 0x29 || code_point == 0x5b || code_point == 0x5d
}

pub open spec fn token_is_ns_tag_char_spec(code_point: u32) -> bool {
    token_is_ns_uri_char_spec(code_point) && code_point != 0x21 && code_point != 0x2c && code_point
        != 0x5b && code_point != 0x5d && code_point != 0x7b && code_point != 0x7d
}

closed spec fn token_first_invalid_tag_alphabet_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    tag_alphabet: bool,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        None
    } else if if tag_alphabet {
        !token_is_ns_tag_char_spec(atoms[index].code_point)
    } else {
        !token_is_ns_uri_char_spec(atoms[index].code_point)
    } {
        Some(index)
    } else {
        token_first_invalid_tag_alphabet_spec(
            atoms,
            index + 1,
            end,
            tag_alphabet,
            (fuel - 1) as nat,
        )
    }
}

closed spec fn token_first_invalid_tag_prefix_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
) -> Option<int> {
    if start < 0 || start >= end || end > atoms.len() {
        Some(start)
    } else if atoms[start].code_point == 0x21 {
        token_first_invalid_tag_alphabet_spec(
            atoms,
            start + 1,
            end,
            false,
            (end - start - 1) as nat,
        )
    } else if !token_is_ns_tag_char_spec(atoms[start].code_point) {
        Some(start)
    } else {
        token_first_invalid_tag_alphabet_spec(
            atoms,
            start + 1,
            end,
            false,
            (end - start - 1) as nat,
        )
    }
}

closed spec fn token_first_bom_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        None
    } else if atoms[index].code_point == 0xfeff {
        Some(index)
    } else {
        token_first_bom_spec(atoms, index + 1, end, (fuel - 1) as nat)
    }
}

closed spec fn token_forward_run_end_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    mode: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        index
    } else {
        let matches = if mode == 0 {
            token_is_space_or_tab_spec(atoms[index].kind)
        } else if mode == 1 {
            atoms[index].kind != LexicalAtomKind::LineFeed
        } else if mode == 2 {
            !token_is_space_or_tab_spec(atoms[index].kind) && atoms[index].kind
                != LexicalAtomKind::LineFeed && !token_is_flow_indicator_spec(atoms[index].kind)
        } else {
            !token_is_space_or_tab_spec(atoms[index].kind)
        };
        if matches {
            token_forward_run_end_spec(atoms, index + 1, end, mode, (fuel - 1) as nat)
        } else {
            index
        }
    }
}

closed spec fn token_first_code_point_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    code_point: u32,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        index
    } else if atoms[index].code_point == code_point {
        index
    } else {
        token_first_code_point_spec(atoms, index + 1, end, code_point, (fuel - 1) as nat)
    }
}

closed spec fn token_directive_comment_start_spec(
    atoms: Seq<LexicalAtomView>,
    directive_start: int,
    index: int,
    end: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        end
    } else if atoms[index].kind == LexicalAtomKind::Indicator(YamlIndicator::Comment) && index
        > directive_start && token_is_space_or_tab_spec(atoms[index - 1].kind) {
        index
    } else {
        token_directive_comment_start_spec(
            atoms,
            directive_start,
            index + 1,
            end,
            (fuel - 1) as nat,
        )
    }
}

closed spec fn token_trim_trailing_separation_spec(
    atoms: Seq<LexicalAtomView>,
    minimum: int,
    end: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if minimum < 0 || end <= minimum || end > atoms.len() || fuel == 0 {
        end
    } else if token_is_space_or_tab_spec(atoms[end - 1].kind) {
        token_trim_trailing_separation_spec(atoms, minimum, end - 1, (fuel - 1) as nat)
    } else {
        end
    }
}

closed spec fn token_directive_payload_end_spec(atoms: Seq<LexicalAtomView>, start: int) -> int {
    let line_end = token_forward_run_end_spec(
        atoms,
        start,
        atoms.len() as int,
        1,
        (atoms.len() - start) as nat,
    );
    let comment = token_directive_comment_start_spec(
        atoms,
        start,
        start + 1,
        line_end,
        (line_end - start) as nat,
    );
    token_trim_trailing_separation_spec(atoms, start + 1, comment, (comment - start) as nat)
}

closed spec fn token_range_is_ascii_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
    expected: Seq<u32>,
) -> bool {
    0 <= start <= end <= atoms.len() && end - start == expected.len() && forall|index: int|
        0 <= index < expected.len() ==> atoms[start + index].code_point
            == #[trigger] expected[index]
}

closed spec fn token_part_for_range_spec(
    atoms: Seq<LexicalAtomView>,
    kind: CompletedTokenPartKind,
    start: int,
    end: int,
) -> CompletedTokenPartView {
    CompletedTokenPartView {
        kind,
        start_atom_index: start as u64,
        end_atom_index: end as u64,
        byte_start: atoms[start].span.start.byte_offset,
        byte_end: atoms[end - 1].span.end.byte_offset,
    }
}

closed spec fn token_for_range_spec(
    atoms: Seq<LexicalAtomView>,
    kind: CompletedTokenKind,
    start: int,
    end: int,
    scalar_index: Option<u64>,
    yaml_major: Option<u64>,
    yaml_minor: Option<u64>,
    parts: Seq<CompletedTokenPartView>,
) -> CompletedTokenView {
    CompletedTokenView {
        kind,
        start_line_number: atoms[start].span.start.line,
        end_line_number: atoms[end - 1].span.start.line,
        start_atom_index: start as u64,
        end_atom_index: end as u64,
        byte_start: atoms[start].span.start.byte_offset,
        byte_end: atoms[end - 1].span.end.byte_offset,
        scalar_index,
        yaml_major,
        yaml_minor,
        parts,
    }
}

closed spec fn token_first_invalid_percent_escape_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        None
    } else if atoms[index].code_point == 0x25 {
        if index + 1 >= end {
            Some(end)
        } else if !token_is_hex_spec(atoms[index + 1].code_point) {
            Some(index + 1)
        } else if index + 2 >= end {
            Some(end)
        } else if !token_is_hex_spec(atoms[index + 2].code_point) {
            Some(index + 2)
        } else if fuel < 3 {
            None
        } else {
            token_first_invalid_percent_escape_spec(atoms, index + 3, end, (fuel - 3) as nat)
        }
    } else {
        token_first_invalid_percent_escape_spec(atoms, index + 1, end, (fuel - 1) as nat)
    }
}

closed spec fn token_first_invalid_word_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        None
    } else if !token_is_word_spec(atoms[index].code_point) {
        Some(index)
    } else {
        token_first_invalid_word_spec(atoms, index + 1, end, (fuel - 1) as nat)
    }
}

closed spec fn token_decimal_tail_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    value: u64,
    error_kind: CompletedTokenErrorKind,
    fuel: nat,
) -> Result<u64, CompletedTokenErrorView>
    decreases fuel,
{
    if index >= end {
        Ok(value)
    } else if index < 0 || end > atoms.len() || fuel == 0 {
        Err(token_error_spec(error_kind, token_offset_at_or_end_spec(atoms, index)))
    } else if !is_decimal_spec(atoms[index].code_point) {
        Err(token_error_spec(error_kind, atoms[index].span.start.byte_offset))
    } else {
        let digit = (atoms[index].code_point - 0x30) as u64;
        if value > (u64::MAX - digit) / 10 {
            Err(token_error_spec(error_kind, atoms[index].span.start.byte_offset))
        } else {
            token_decimal_tail_spec(
                atoms,
                index + 1,
                end,
                (value * 10 + digit) as u64,
                error_kind,
                (fuel - 1) as nat,
            )
        }
    }
}

closed spec fn token_parse_decimal_component_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
    error_kind: CompletedTokenErrorKind,
) -> Result<u64, CompletedTokenErrorView> {
    if start == end {
        Err(token_error_spec(error_kind, token_offset_at_or_end_spec(atoms, start)))
    } else {
        token_decimal_tail_spec(atoms, start, end, 0, error_kind, (end - start) as nat)
    }
}

closed spec fn token_valid_tag_handle_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
) -> bool {
    if end - start == 1 {
        atoms[start].code_point == 0x21
    } else if end - start == 2 {
        atoms[start].code_point == 0x21 && atoms[start + 1].code_point == 0x21
    } else {
        atoms[start].code_point == 0x21 && atoms[end - 1].code_point == 0x21
            && token_first_invalid_word_spec(
            atoms,
            start + 1,
            end - 1,
            (end - start - 1) as nat,
        ).is_none()
    }
}

closed spec fn token_reserved_directive_parts_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: int,
    end: int,
    built: Seq<CompletedTokenPartView>,
    fuel: nat,
) -> Seq<CompletedTokenPartView>
    decreases fuel,
{
    if cursor < 0 || cursor >= end || end > atoms.len() || fuel == 0 {
        built
    } else {
        let parameter_start = token_forward_run_end_spec(
            atoms,
            cursor,
            end,
            0,
            (end - cursor) as nat,
        );
        if parameter_start == end {
            built
        } else {
            let parameter_end = token_forward_run_end_spec(
                atoms,
                parameter_start,
                end,
                3,
                (end - parameter_start) as nat,
            );
            token_reserved_directive_parts_spec(
                atoms,
                parameter_end,
                end,
                built.push(
                    token_part_for_range_spec(
                        atoms,
                        CompletedTokenPartKind::DirectiveParameter,
                        parameter_start,
                        parameter_end,
                    ),
                ),
                (fuel - 1) as nat,
            )
        }
    }
}

closed spec fn token_parse_directive_spec(atoms: Seq<LexicalAtomView>, start: int) -> Result<
    CompletedTokenView,
    CompletedTokenErrorView,
> {
    let token_end = token_directive_payload_end_spec(atoms, start);
    let name_end = token_forward_run_end_spec(
        atoms,
        start + 1,
        token_end,
        3,
        (token_end - start - 1) as nat,
    );
    if name_end == start + 1 {
        Err(
            token_error_spec(
                CompletedTokenErrorKind::EmptyDirectiveName,
                token_offset_at_or_end_spec(atoms, start + 1),
            ),
        )
    } else {
        let name = token_part_for_range_spec(
            atoms,
            CompletedTokenPartKind::DirectiveName,
            start + 1,
            name_end,
        );
        let is_yaml = token_range_is_ascii_spec(
            atoms,
            start + 1,
            name_end,
            seq![0x59u32, 0x41u32, 0x4du32, 0x4cu32],
        );
        let is_tag = token_range_is_ascii_spec(
            atoms,
            start + 1,
            name_end,
            seq![0x54u32, 0x41u32, 0x47u32],
        );
        if is_yaml {
            let parameter_start = token_forward_run_end_spec(
                atoms,
                name_end,
                token_end,
                0,
                (token_end - name_end) as nat,
            );
            if parameter_start == token_end {
                Err(
                    token_error_spec(
                        CompletedTokenErrorKind::InvalidYamlDirective,
                        token_offset_at_or_end_spec(atoms, name_end),
                    ),
                )
            } else {
                let parameter_end = token_forward_run_end_spec(
                    atoms,
                    parameter_start,
                    token_end,
                    3,
                    (token_end - parameter_start) as nat,
                );
                let trailing = token_forward_run_end_spec(
                    atoms,
                    parameter_end,
                    token_end,
                    0,
                    (token_end - parameter_end) as nat,
                );
                let dot = token_first_code_point_spec(
                    atoms,
                    parameter_start,
                    parameter_end,
                    0x2e,
                    (parameter_end - parameter_start) as nat,
                );
                if trailing != token_end {
                    Err(
                        token_error_spec(
                            CompletedTokenErrorKind::InvalidYamlDirective,
                            atoms[trailing].span.start.byte_offset,
                        ),
                    )
                } else if dot == parameter_end {
                    Err(
                        token_error_spec(
                            CompletedTokenErrorKind::InvalidYamlDirective,
                            atoms[parameter_start].span.start.byte_offset,
                        ),
                    )
                } else {
                    let extra_dot = token_first_code_point_spec(
                        atoms,
                        dot + 1,
                        parameter_end,
                        0x2e,
                        (parameter_end - dot - 1) as nat,
                    );
                    if extra_dot < parameter_end {
                        Err(
                            token_error_spec(
                                CompletedTokenErrorKind::InvalidYamlDirective,
                                atoms[extra_dot].span.start.byte_offset,
                            ),
                        )
                    } else {
                        match token_parse_decimal_component_spec(
                            atoms,
                            parameter_start,
                            dot,
                            CompletedTokenErrorKind::InvalidYamlDirective,
                        ) {
                            Err(error) => Err(error),
                            Ok(major) => match token_parse_decimal_component_spec(
                                atoms,
                                dot + 1,
                                parameter_end,
                                CompletedTokenErrorKind::InvalidYamlDirective,
                            ) {
                                Err(error) => Err(error),
                                Ok(minor) => Ok(
                                    token_for_range_spec(
                                        atoms,
                                        CompletedTokenKind::YamlDirective,
                                        start,
                                        token_end,
                                        None,
                                        Some(major),
                                        Some(minor),
                                        seq![
                                            name,
                                            token_part_for_range_spec(
                                                atoms,
                                                CompletedTokenPartKind::YamlMajor,
                                                parameter_start,
                                                dot,
                                            ),
                                            token_part_for_range_spec(
                                                atoms,
                                                CompletedTokenPartKind::YamlMinor,
                                                dot + 1,
                                                parameter_end,
                                            ),
                                        ],
                                    ),
                                ),
                            },
                        }
                    }
                }
            }
        } else if is_tag {
            let handle_start = token_forward_run_end_spec(
                atoms,
                name_end,
                token_end,
                0,
                (token_end - name_end) as nat,
            );
            if handle_start == token_end {
                Err(
                    token_error_spec(
                        CompletedTokenErrorKind::InvalidTagDirective,
                        token_offset_at_or_end_spec(atoms, handle_start),
                    ),
                )
            } else {
                let handle_end = token_forward_run_end_spec(
                    atoms,
                    handle_start,
                    token_end,
                    3,
                    (token_end - handle_start) as nat,
                );
                let prefix_start = token_forward_run_end_spec(
                    atoms,
                    handle_end,
                    token_end,
                    0,
                    (token_end - handle_end) as nat,
                );
                if prefix_start == token_end {
                    Err(
                        token_error_spec(
                            CompletedTokenErrorKind::InvalidTagDirective,
                            token_offset_at_or_end_spec(atoms, prefix_start),
                        ),
                    )
                } else {
                    let prefix_end = token_forward_run_end_spec(
                        atoms,
                        prefix_start,
                        token_end,
                        3,
                        (token_end - prefix_start) as nat,
                    );
                    let trailing = token_forward_run_end_spec(
                        atoms,
                        prefix_end,
                        token_end,
                        0,
                        (token_end - prefix_end) as nat,
                    );
                    if trailing != token_end || !token_valid_tag_handle_spec(
                        atoms,
                        handle_start,
                        handle_end,
                    ) {
                        Err(
                            token_error_spec(
                                CompletedTokenErrorKind::InvalidTagDirective,
                                atoms[handle_start].span.start.byte_offset,
                            ),
                        )
                    } else {
                        match token_first_invalid_tag_prefix_spec(atoms, prefix_start, prefix_end) {
                            Some(invalid) => Err(
                                token_error_spec(
                                    CompletedTokenErrorKind::InvalidTagDirective,
                                    token_offset_at_or_end_spec(atoms, invalid),
                                ),
                            ),
                            None => match token_first_invalid_percent_escape_spec(
                                atoms,
                                prefix_start,
                                prefix_end,
                                (prefix_end - prefix_start) as nat,
                            ) {
                                Some(invalid) => Err(
                                    token_error_spec(
                                        CompletedTokenErrorKind::InvalidTagDirective,
                                        token_offset_at_or_end_spec(atoms, invalid),
                                    ),
                                ),
                                None => Ok(
                                    token_for_range_spec(
                                        atoms,
                                        CompletedTokenKind::TagDirective,
                                        start,
                                        token_end,
                                        None,
                                        None,
                                        None,
                                        seq![
                                            name,
                                            token_part_for_range_spec(
                                                atoms,
                                                CompletedTokenPartKind::TagHandle,
                                                handle_start,
                                                handle_end,
                                            ),
                                            token_part_for_range_spec(
                                                atoms,
                                                CompletedTokenPartKind::TagPrefix,
                                                prefix_start,
                                                prefix_end,
                                            ),
                                        ],
                                    ),
                                ),
                            },
                        }
                    }
                }
            }
        } else {
            match token_first_bom_spec(
                atoms,
                start + 1,
                token_end,
                (token_end - start - 1) as nat,
            ) {
                Some(invalid) => Err(
                    token_error_spec(
                        CompletedTokenErrorKind::InvalidDirectiveCharacter,
                        atoms[invalid].span.start.byte_offset,
                    ),
                ),
                None => Ok(
                    token_for_range_spec(
                        atoms,
                        CompletedTokenKind::ReservedDirective,
                        start,
                        token_end,
                        None,
                        None,
                        None,
                        token_reserved_directive_parts_spec(
                            atoms,
                            name_end,
                            token_end,
                            seq![name],
                            (token_end - name_end + 1) as nat,
                        ),
                    ),
                ),
            }
        }
    }
}

closed spec fn token_parse_anchor_or_alias_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    alias: bool,
) -> Result<CompletedTokenView, CompletedTokenErrorView> {
    let end = token_forward_run_end_spec(
        atoms,
        start + 1,
        atoms.len() as int,
        2,
        (atoms.len() - start - 1) as nat,
    );
    if end == start + 1 {
        Err(
            token_error_spec(
                if alias {
                    CompletedTokenErrorKind::EmptyAliasName
                } else {
                    CompletedTokenErrorKind::EmptyAnchorName
                },
                token_offset_at_or_end_spec(atoms, end),
            ),
        )
    } else {
        match token_first_bom_spec(atoms, start + 1, end, (end - start - 1) as nat) {
            Some(invalid) => Err(
                token_error_spec(
                    if alias {
                        CompletedTokenErrorKind::InvalidAliasCharacter
                    } else {
                        CompletedTokenErrorKind::InvalidAnchorCharacter
                    },
                    atoms[invalid].span.start.byte_offset,
                ),
            ),
            None => Ok(
                token_for_range_spec(
                    atoms,
                    if alias {
                        CompletedTokenKind::Alias
                    } else {
                        CompletedTokenKind::AnchorProperty
                    },
                    start,
                    end,
                    None,
                    None,
                    None,
                    seq![
                        token_part_for_range_spec(
                            atoms,
                            if alias {
                                CompletedTokenPartKind::AliasName
                            } else {
                                CompletedTokenPartKind::AnchorName
                            },
                            start + 1,
                            end,
                        ),
                    ],
                ),
            ),
        }
    }
}

closed spec fn token_verbatim_tag_end_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || index >= atoms.len() || fuel == 0 {
        index
    } else if atoms[index].code_point == 0x3e || token_is_space_or_tab_spec(atoms[index].kind)
        || atoms[index].kind == LexicalAtomKind::LineFeed {
        index
    } else {
        token_verbatim_tag_end_spec(atoms, index + 1, (fuel - 1) as nat)
    }
}

closed spec fn token_parse_tag_spec(atoms: Seq<LexicalAtomView>, start: int) -> Result<
    CompletedTokenView,
    CompletedTokenErrorView,
> {
    if start + 1 < atoms.len() && atoms[start + 1].code_point == 0x3c {
        let end = token_verbatim_tag_end_spec(atoms, start + 2, (atoms.len() - start - 2) as nat);
        if end >= atoms.len() || atoms[end].code_point != 0x3e {
            Err(
                token_error_spec(
                    CompletedTokenErrorKind::UnterminatedVerbatimTag,
                    token_offset_at_or_end_spec(atoms, end),
                ),
            )
        } else if end == start + 2 {
            Err(
                token_error_spec(
                    CompletedTokenErrorKind::EmptyVerbatimTag,
                    atoms[end].span.start.byte_offset,
                ),
            )
        } else {
            let invalid_uri = token_first_invalid_tag_alphabet_spec(
                atoms,
                start + 2,
                end,
                false,
                (end - start - 2) as nat,
            );
            match invalid_uri {
                Some(invalid) => Err(
                    token_error_spec(
                        CompletedTokenErrorKind::InvalidVerbatimTag,
                        atoms[invalid].span.start.byte_offset,
                    ),
                ),
                None => match token_first_invalid_percent_escape_spec(
                    atoms,
                    start + 2,
                    end,
                    (end - start - 2) as nat,
                ) {
                    Some(invalid) => Err(
                        token_error_spec(
                            CompletedTokenErrorKind::InvalidTagPercentEscape,
                            token_offset_at_or_end_spec(atoms, invalid),
                        ),
                    ),
                    None => Ok(
                        token_for_range_spec(
                            atoms,
                            CompletedTokenKind::VerbatimTagProperty,
                            start,
                            end + 1,
                            None,
                            None,
                            None,
                            seq![
                                token_part_for_range_spec(
                                    atoms,
                                    CompletedTokenPartKind::VerbatimTagPayload,
                                    start + 2,
                                    end,
                                ),
                            ],
                        ),
                    ),
                },
            }
        }
    } else {
        let end = token_forward_run_end_spec(
            atoms,
            start + 1,
            atoms.len() as int,
            2,
            (atoms.len() - start - 1) as nat,
        );
        if end == start + 1 {
            Ok(
                token_for_range_spec(
                    atoms,
                    CompletedTokenKind::TagProperty,
                    start,
                    start + 1,
                    None,
                    None,
                    None,
                    Seq::empty(),
                ),
            )
        } else {
            let bang = if atoms[start + 1].code_point == 0x21 {
                start + 1
            } else {
                token_first_code_point_spec(atoms, start + 1, end, 0x21, (end - start - 1) as nat)
            };
            let handle_end = if bang < end {
                bang + 1
            } else {
                start + 1
            };
            let suffix_start = handle_end;
            let invalid_handle = if atoms[start + 1].code_point == 0x21 || bang >= end {
                None
            } else {
                token_first_invalid_word_spec(atoms, start + 1, bang, (bang - start - 1) as nat)
            };
            match invalid_handle {
                Some(invalid) => Err(
                    token_error_spec(
                        CompletedTokenErrorKind::InvalidTagCharacter,
                        atoms[invalid].span.start.byte_offset,
                    ),
                ),
                None => if suffix_start == end {
                    Err(
                        token_error_spec(
                            CompletedTokenErrorKind::EmptyTagSuffix,
                            token_offset_at_or_end_spec(atoms, end),
                        ),
                    )
                } else {
                    let invalid_tag = token_first_invalid_tag_alphabet_spec(
                        atoms,
                        suffix_start,
                        end,
                        true,
                        (end - suffix_start) as nat,
                    );
                    match invalid_tag {
                        Some(invalid) => Err(
                            token_error_spec(
                                CompletedTokenErrorKind::InvalidTagCharacter,
                                atoms[invalid].span.start.byte_offset,
                            ),
                        ),
                        None => match token_first_invalid_percent_escape_spec(
                            atoms,
                            suffix_start,
                            end,
                            (end - suffix_start) as nat,
                        ) {
                            Some(invalid) => Err(
                                token_error_spec(
                                    CompletedTokenErrorKind::InvalidTagPercentEscape,
                                    token_offset_at_or_end_spec(atoms, invalid),
                                ),
                            ),
                            None => Ok(
                                token_for_range_spec(
                                    atoms,
                                    CompletedTokenKind::TagProperty,
                                    start,
                                    end,
                                    None,
                                    None,
                                    None,
                                    seq![
                                        token_part_for_range_spec(
                                            atoms,
                                            CompletedTokenPartKind::TagHandle,
                                            start,
                                            handle_end,
                                        ),
                                        token_part_for_range_spec(
                                            atoms,
                                            CompletedTokenPartKind::TagSuffix,
                                            suffix_start,
                                            end,
                                        ),
                                    ],
                                ),
                            ),
                        },
                    }
                },
            }
        }
    }
}

closed spec fn token_marker_at_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    code_point: u32,
) -> bool {
    0 <= start && start + 3 <= atoms.len() && atoms[start].code_point == code_point && atoms[start
        + 1].code_point == code_point && atoms[start + 2].code_point == code_point && (start + 3
        == atoms.len() || token_is_space_or_tab_spec(atoms[start + 3].kind) || atoms[start + 3].kind
        == LexicalAtomKind::LineFeed || atoms[start + 3].kind == LexicalAtomKind::Indicator(
        YamlIndicator::Comment,
    ))
}

closed spec fn token_advance_quoted_index_spec(
    scalars: Seq<QuotedScalarView>,
    scalar_index: int,
    atom_index: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if scalar_index < 0 || scalar_index >= scalars.len() || fuel == 0
        || scalars[scalar_index].end_atom_index > atom_index {
        scalar_index
    } else {
        token_advance_quoted_index_spec(scalars, scalar_index + 1, atom_index, (fuel - 1) as nat)
    }
}

closed spec fn token_advance_plain_index_spec(
    scalars: Seq<PlainScalarView>,
    scalar_index: int,
    atom_index: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if scalar_index < 0 || scalar_index >= scalars.len() || fuel == 0
        || scalars[scalar_index].end_atom_index > atom_index {
        scalar_index
    } else {
        token_advance_plain_index_spec(scalars, scalar_index + 1, atom_index, (fuel - 1) as nat)
    }
}

closed spec fn token_advance_block_index_spec(
    scalars: Seq<BlockScalarView>,
    scalar_index: int,
    atom_index: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if scalar_index < 0 || scalar_index >= scalars.len() || fuel == 0
        || scalars[scalar_index].end_atom_index > atom_index {
        scalar_index
    } else {
        token_advance_block_index_spec(scalars, scalar_index + 1, atom_index, (fuel - 1) as nat)
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
pub struct CompletedTokenStepView {
    pub token: CompletedTokenView,
    pub next_atom_index: int,
    pub next_quote_index: int,
    pub next_plain_index: int,
    pub next_block_index: int,
    pub next_at_line_prefix: bool,
    pub next_directive_mode: bool,
}

struct CompletedTokenStep {
    token: CompletedToken,
    next_atom_index: usize,
    next_quote_index: usize,
    next_plain_index: usize,
    next_block_index: usize,
    next_at_line_prefix: bool,
    next_directive_mode: bool,
}

impl View for CompletedTokenStep {
    type V = CompletedTokenStepView;

    closed spec fn view(&self) -> CompletedTokenStepView {
        CompletedTokenStepView {
            token: self.token@,
            next_atom_index: self.next_atom_index as int,
            next_quote_index: self.next_quote_index as int,
            next_plain_index: self.next_plain_index as int,
            next_block_index: self.next_block_index as int,
            next_at_line_prefix: self.next_at_line_prefix,
            next_directive_mode: self.next_directive_mode,
        }
    }
}

closed spec fn token_single_indicator_kind_spec(kind: LexicalAtomKind) -> Result<
    CompletedTokenKind,
    CompletedTokenErrorKind,
> {
    match kind {
        LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart) => Ok(
            CompletedTokenKind::FlowSequenceStart,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart) => Ok(
            CompletedTokenKind::FlowMappingStart,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd) => Ok(
            CompletedTokenKind::FlowSequenceEnd,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd) => Ok(
            CompletedTokenKind::FlowMappingEnd,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) => Ok(CompletedTokenKind::FlowEntry),
        LexicalAtomKind::Indicator(YamlIndicator::BlockSequenceEntry) => Ok(
            CompletedTokenKind::BlockSequenceEntry,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::ExplicitMappingKey) => Ok(
            CompletedTokenKind::ExplicitMappingKey,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::MappingValue) => Ok(
            CompletedTokenKind::MappingValue,
        ),
        LexicalAtomKind::Indicator(YamlIndicator::ReservedAt)
        | LexicalAtomKind::Indicator(YamlIndicator::ReservedGraveAccent) => Err(
            CompletedTokenErrorKind::ReservedIndicator,
        ),
        LexicalAtomKind::Indicator(_) => Err(CompletedTokenErrorKind::UnexpectedIndicator),
        _ => Err(CompletedTokenErrorKind::UnexpectedContent),
    }
}

closed spec fn token_step_for_token_spec(
    token: CompletedTokenView,
    next_atom_index: int,
    quote_index: int,
    plain_index: int,
    block_index: int,
    at_line_prefix: bool,
    directive_mode: bool,
) -> CompletedTokenStepView {
    CompletedTokenStepView {
        token,
        next_atom_index,
        next_quote_index: quote_index,
        next_plain_index: plain_index,
        next_block_index: block_index,
        next_at_line_prefix: at_line_prefix,
        next_directive_mode: directive_mode,
    }
}

closed spec fn token_first_kind_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    kind: LexicalAtomKind,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || index >= end || end > atoms.len() || fuel == 0 {
        index
    } else if atoms[index].kind == kind {
        index
    } else {
        token_first_kind_spec(atoms, index + 1, end, kind, (fuel - 1) as nat)
    }
}

pub closed spec fn token_next_candidate_spec(
    atoms: Seq<LexicalAtomView>,
    quotes: Seq<QuotedScalarView>,
    plains: Seq<PlainScalarView>,
    blocks: Seq<BlockScalarView>,
    index: int,
    quote_index: int,
    plain_index: int,
    block_index: int,
    at_line_prefix: bool,
    directive_mode: bool,
) -> Result<CompletedTokenStepView, CompletedTokenErrorView> {
    if index < 0 || index >= atoms.len() {
        Err(token_error_spec(CompletedTokenErrorKind::InputScalarOverlap, 0))
    } else if at_line_prefix && directive_mode && atoms[index].code_point == 0xfeff {
        Ok(
            token_step_for_token_spec(
                token_for_range_spec(
                    atoms,
                    CompletedTokenKind::DocumentByteOrderMark,
                    index,
                    index + 1,
                    None,
                    None,
                    None,
                    Seq::empty(),
                ),
                index + 1,
                quote_index,
                plain_index,
                block_index,
                at_line_prefix,
                directive_mode,
            ),
        )
    } else if 0 <= block_index < blocks.len() && blocks[block_index].start_atom_index == index {
        let end = blocks[block_index].end_atom_index as int;
        if end <= index || end > atoms.len() {
            Err(
                token_error_spec(
                    CompletedTokenErrorKind::InputScalarOverlap,
                    atoms[index].span.start.byte_offset,
                ),
            )
        } else {
            Ok(
                token_step_for_token_spec(
                    token_for_range_spec(
                        atoms,
                        if blocks[block_index].style == BlockScalarStyle::Literal {
                            CompletedTokenKind::LiteralBlockScalar
                        } else {
                            CompletedTokenKind::FoldedBlockScalar
                        },
                        index,
                        end,
                        Some(block_index as u64),
                        None,
                        None,
                        Seq::empty(),
                    ),
                    end,
                    quote_index,
                    plain_index,
                    block_index + 1,
                    atoms[end - 1].kind == LexicalAtomKind::LineFeed,
                    false,
                ),
            )
        }
    } else if 0 <= quote_index < quotes.len() && quotes[quote_index].start_atom_index == index {
        let end = quotes[quote_index].end_atom_index as int;
        if end <= index || end > atoms.len() {
            Err(
                token_error_spec(
                    CompletedTokenErrorKind::InputScalarOverlap,
                    atoms[index].span.start.byte_offset,
                ),
            )
        } else {
            Ok(
                token_step_for_token_spec(
                    token_for_range_spec(
                        atoms,
                        if quotes[quote_index].style == QuotedScalarStyle::Single {
                            CompletedTokenKind::SingleQuotedScalar
                        } else {
                            CompletedTokenKind::DoubleQuotedScalar
                        },
                        index,
                        end,
                        Some(quote_index as u64),
                        None,
                        None,
                        Seq::empty(),
                    ),
                    end,
                    quote_index + 1,
                    plain_index,
                    block_index,
                    atoms[end - 1].kind == LexicalAtomKind::LineFeed,
                    false,
                ),
            )
        }
    } else if 0 <= plain_index < plains.len() && plains[plain_index].start_atom_index == index {
        let end = plains[plain_index].end_atom_index as int;
        if end <= index || end > atoms.len() {
            Err(
                token_error_spec(
                    CompletedTokenErrorKind::InputScalarOverlap,
                    atoms[index].span.start.byte_offset,
                ),
            )
        } else {
            Ok(
                token_step_for_token_spec(
                    token_for_range_spec(
                        atoms,
                        CompletedTokenKind::PlainScalar,
                        index,
                        end,
                        Some(plain_index as u64),
                        None,
                        None,
                        Seq::empty(),
                    ),
                    end,
                    quote_index,
                    plain_index + 1,
                    block_index,
                    atoms[end - 1].kind == LexicalAtomKind::LineFeed,
                    false,
                ),
            )
        }
    } else if atoms[index].kind == LexicalAtomKind::LineFeed {
        Ok(
            token_step_for_token_spec(
                token_for_range_spec(
                    atoms,
                    CompletedTokenKind::LineFeed,
                    index,
                    index + 1,
                    None,
                    None,
                    None,
                    Seq::empty(),
                ),
                index + 1,
                quote_index,
                plain_index,
                block_index,
                true,
                directive_mode,
            ),
        )
    } else if token_is_space_or_tab_spec(atoms[index].kind) {
        let end = token_forward_run_end_spec(
            atoms,
            index,
            atoms.len() as int,
            0,
            (atoms.len() - index) as nat,
        );
        let tab = token_first_kind_spec(
            atoms,
            index,
            end,
            LexicalAtomKind::Tab,
            (end - index) as nat,
        );
        if at_line_prefix && tab < end {
            Err(
                token_error_spec(
                    CompletedTokenErrorKind::TabInIndentation,
                    atoms[tab].span.start.byte_offset,
                ),
            )
        } else {
            Ok(
                token_step_for_token_spec(
                    token_for_range_spec(
                        atoms,
                        if at_line_prefix {
                            CompletedTokenKind::Indentation
                        } else {
                            CompletedTokenKind::Separation
                        },
                        index,
                        end,
                        None,
                        None,
                        None,
                        Seq::empty(),
                    ),
                    end,
                    quote_index,
                    plain_index,
                    block_index,
                    false,
                    directive_mode,
                ),
            )
        }
    } else if atoms[index].kind == LexicalAtomKind::Indicator(YamlIndicator::Comment) {
        let end = token_forward_run_end_spec(
            atoms,
            index,
            atoms.len() as int,
            1,
            (atoms.len() - index) as nat,
        );
        Ok(
            token_step_for_token_spec(
                token_for_range_spec(
                    atoms,
                    CompletedTokenKind::Comment,
                    index,
                    end,
                    None,
                    None,
                    None,
                    Seq::empty(),
                ),
                end,
                quote_index,
                plain_index,
                block_index,
                false,
                directive_mode,
            ),
        )
    } else if at_line_prefix && token_marker_at_spec(atoms, index, 0x2d) {
        Ok(
            token_step_for_token_spec(
                token_for_range_spec(
                    atoms,
                    CompletedTokenKind::DirectivesEnd,
                    index,
                    index + 3,
                    None,
                    None,
                    None,
                    Seq::empty(),
                ),
                index + 3,
                quote_index,
                plain_index,
                block_index,
                false,
                false,
            ),
        )
    } else if at_line_prefix && token_marker_at_spec(atoms, index, 0x2e) {
        Ok(
            token_step_for_token_spec(
                token_for_range_spec(
                    atoms,
                    CompletedTokenKind::DocumentEnd,
                    index,
                    index + 3,
                    None,
                    None,
                    None,
                    Seq::empty(),
                ),
                index + 3,
                quote_index,
                plain_index,
                block_index,
                false,
                true,
            ),
        )
    } else if at_line_prefix && directive_mode && atoms[index].kind == LexicalAtomKind::Indicator(
        YamlIndicator::Directive,
    ) {
        match token_parse_directive_spec(atoms, index) {
            Err(error) => Err(error),
            Ok(token) => Ok(
                token_step_for_token_spec(
                    token,
                    token.end_atom_index as int,
                    quote_index,
                    plain_index,
                    block_index,
                    false,
                    directive_mode,
                ),
            ),
        }
    } else if atoms[index].kind == LexicalAtomKind::Indicator(YamlIndicator::Anchor) {
        match token_parse_anchor_or_alias_spec(atoms, index, false) {
            Err(error) => Err(error),
            Ok(token) => Ok(
                token_step_for_token_spec(
                    token,
                    token.end_atom_index as int,
                    quote_index,
                    plain_index,
                    block_index,
                    false,
                    false,
                ),
            ),
        }
    } else if atoms[index].kind == LexicalAtomKind::Indicator(YamlIndicator::Alias) {
        match token_parse_anchor_or_alias_spec(atoms, index, true) {
            Err(error) => Err(error),
            Ok(token) => Ok(
                token_step_for_token_spec(
                    token,
                    token.end_atom_index as int,
                    quote_index,
                    plain_index,
                    block_index,
                    false,
                    false,
                ),
            ),
        }
    } else if atoms[index].kind == LexicalAtomKind::Indicator(YamlIndicator::Tag) {
        match token_parse_tag_spec(atoms, index) {
            Err(error) => Err(error),
            Ok(token) => Ok(
                token_step_for_token_spec(
                    token,
                    token.end_atom_index as int,
                    quote_index,
                    plain_index,
                    block_index,
                    false,
                    false,
                ),
            ),
        }
    } else {
        match token_single_indicator_kind_spec(atoms[index].kind) {
            Err(kind) => Err(token_error_spec(kind, atoms[index].span.start.byte_offset)),
            Ok(kind) => Ok(
                token_step_for_token_spec(
                    token_for_range_spec(
                        atoms,
                        kind,
                        index,
                        index + 1,
                        None,
                        None,
                        None,
                        Seq::empty(),
                    ),
                    index + 1,
                    quote_index,
                    plain_index,
                    block_index,
                    false,
                    false,
                ),
            ),
        }
    }
}

pub open spec fn completed_token_exact_formation_spec(
    atoms: Seq<LexicalAtomView>,
    quoted: Seq<QuotedScalarView>,
    plain: Seq<PlainScalarView>,
    block: Seq<BlockScalarView>,
    token: CompletedTokenView,
) -> bool {
    exists|
        index: int,
        quote_index: int,
        plain_index: int,
        block_index: int,
        at_line_prefix: bool,
        directive_mode: bool,
        step: CompletedTokenStepView,
    |
        0 <= index < atoms.len() && 0 <= quote_index <= quoted.len() && 0 <= plain_index
            <= plain.len() && 0 <= block_index <= block.len() && token_next_candidate_spec(
            atoms,
            quoted,
            plain,
            block,
            index,
            quote_index,
            plain_index,
            block_index,
            at_line_prefix,
            directive_mode,
        ) == Ok(step) && step.token == token
}

pub open spec fn completed_token_exact_formation_sequence_spec(
    atoms: Seq<LexicalAtomView>,
    quoted: Seq<QuotedScalarView>,
    plain: Seq<PlainScalarView>,
    block: Seq<BlockScalarView>,
    tokens: Seq<CompletedTokenView>,
) -> bool {
    forall|index: int|
        0 <= index < tokens.len() ==> #[trigger] completed_token_exact_formation_spec(
            atoms,
            quoted,
            plain,
            block,
            tokens[index],
        )
}

proof fn lemma_completed_token_exact_formation_push(
    atoms: Seq<LexicalAtomView>,
    quoted: Seq<QuotedScalarView>,
    plain: Seq<PlainScalarView>,
    block: Seq<BlockScalarView>,
    tokens: Seq<CompletedTokenView>,
    token: CompletedTokenView,
)
    requires
        completed_token_exact_formation_sequence_spec(atoms, quoted, plain, block, tokens),
        completed_token_exact_formation_spec(atoms, quoted, plain, block, token),
    ensures
        completed_token_exact_formation_sequence_spec(
            atoms,
            quoted,
            plain,
            block,
            tokens.push(token),
        ),
{
    reveal(completed_token_exact_formation_sequence_spec);
    assert forall|index: int|
        0 <= index < tokens.push(token).len() implies completed_token_exact_formation_spec(
        atoms,
        quoted,
        plain,
        block,
        #[trigger] tokens.push(token)[index],
    ) by {
        if index < tokens.len() {
            assert(tokens.push(token)[index] == tokens[index]);
        } else {
            assert(index == tokens.len());
            assert(tokens.push(token)[index] == token);
        }
    }
}

closed spec fn token_apply_flow_kind_spec(
    stack: Seq<CompletedTokenKind>,
    kind: CompletedTokenKind,
    byte_offset: u64,
    flow_limit: u64,
) -> Result<Seq<CompletedTokenKind>, CompletedTokenErrorView> {
    if kind == CompletedTokenKind::FlowSequenceStart || kind
        == CompletedTokenKind::FlowMappingStart {
        if stack.len() >= flow_limit {
            Err(token_error_spec(CompletedTokenErrorKind::FlowDepthLimitExceeded, byte_offset))
        } else {
            Ok(stack.push(kind))
        }
    } else if kind == CompletedTokenKind::FlowSequenceEnd {
        if stack.len() == 0 {
            Err(token_error_spec(CompletedTokenErrorKind::UnexpectedFlowEnd, byte_offset))
        } else if stack.last() != CompletedTokenKind::FlowSequenceStart {
            Err(token_error_spec(CompletedTokenErrorKind::MismatchedFlowEnd, byte_offset))
        } else {
            Ok(stack.drop_last())
        }
    } else if kind == CompletedTokenKind::FlowMappingEnd {
        if stack.len() == 0 {
            Err(token_error_spec(CompletedTokenErrorKind::UnexpectedFlowEnd, byte_offset))
        } else if stack.last() != CompletedTokenKind::FlowMappingStart {
            Err(token_error_spec(CompletedTokenErrorKind::MismatchedFlowEnd, byte_offset))
        } else {
            Ok(stack.drop_last())
        }
    } else if kind == CompletedTokenKind::FlowEntry && stack.len() == 0 {
        Err(token_error_spec(CompletedTokenErrorKind::UnexpectedIndicator, byte_offset))
    } else {
        Ok(stack)
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct CompletedTokenTailSuccessView {
    maximum_flow_depth: u64,
    tokens: Seq<CompletedTokenView>,
}

closed spec fn completed_token_source_from_tail_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    success: CompletedTokenTailSuccessView,
) -> CompletedTokenSourceView {
    CompletedTokenSourceView {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version,
        layout_transformation_version: layout.transformation_version,
        structural_transformation_version: structural.transformation_version,
        quoted_transformation_version: quoted.transformation_version,
        plain_transformation_version: plain.transformation_version,
        block_transformation_version: block.transformation_version,
        transformation_version: COMPLETED_TOKEN_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes,
        bom_bytes: atomized.bom_bytes,
        input_atom_count: atomized.atoms.len() as u64,
        maximum_flow_depth: success.maximum_flow_depth,
        tokens: success.tokens,
    }
}

closed spec fn completed_token_scan_tail_spec(
    atoms: Seq<LexicalAtomView>,
    quotes: Seq<QuotedScalarView>,
    plains: Seq<PlainScalarView>,
    blocks: Seq<BlockScalarView>,
    source_len_bytes: u64,
    index: int,
    quote_index: int,
    plain_index: int,
    block_index: int,
    at_line_prefix: bool,
    directive_mode: bool,
    flow_stack: Seq<CompletedTokenKind>,
    maximum_flow_depth: u64,
    built: Seq<CompletedTokenView>,
    token_limit: u64,
    flow_limit: u64,
    fuel: nat,
) -> Result<CompletedTokenTailSuccessView, CompletedTokenErrorView>
    decreases fuel,
{
    if index == atoms.len() {
        if flow_stack.len() > 0 {
            Err(token_error_spec(CompletedTokenErrorKind::UnclosedFlowCollection, source_len_bytes))
        } else {
            Ok(CompletedTokenTailSuccessView { maximum_flow_depth, tokens: built })
        }
    } else if index < 0 || index > atoms.len() || quote_index < 0 || plain_index < 0 || block_index
        < 0 || fuel == 0 {
        Err(
            token_error_spec(
                CompletedTokenErrorKind::InputScalarOverlap,
                token_offset_at_or_end_spec(atoms, index),
            ),
        )
    } else {
        let next_quote = token_advance_quoted_index_spec(
            quotes,
            quote_index,
            index,
            (quotes.len() - quote_index + 1) as nat,
        );
        let next_plain = token_advance_plain_index_spec(
            plains,
            plain_index,
            index,
            (plains.len() - plain_index + 1) as nat,
        );
        let next_block = token_advance_block_index_spec(
            blocks,
            block_index,
            index,
            (blocks.len() - block_index + 1) as nat,
        );
        match token_next_candidate_spec(
            atoms,
            quotes,
            plains,
            blocks,
            index,
            next_quote,
            next_plain,
            next_block,
            at_line_prefix,
            directive_mode,
        ) {
            Err(error) => Err(error),
            Ok(step) => {
                if step.token.start_atom_index != index || step.token.end_atom_index
                    != step.next_atom_index || !completed_token_range_spec(atoms, step.token) {
                    Err(
                        token_error_spec(
                            CompletedTokenErrorKind::InputScalarOverlap,
                            step.token.byte_start,
                        ),
                    )
                } else {
                    match token_apply_flow_kind_spec(
                        flow_stack,
                        step.token.kind,
                        step.token.byte_start,
                        flow_limit,
                    ) {
                        Err(error) => Err(error),
                        Ok(next_stack) => {
                            let next_maximum = if next_stack.len() > maximum_flow_depth {
                                next_stack.len() as u64
                            } else {
                                maximum_flow_depth
                            };
                            if built.len() >= token_limit {
                                Err(
                                    token_error_spec(
                                        CompletedTokenErrorKind::TokenLimitExceeded,
                                        step.token.byte_start,
                                    ),
                                )
                            } else {
                                completed_token_scan_tail_spec(
                                    atoms,
                                    quotes,
                                    plains,
                                    blocks,
                                    source_len_bytes,
                                    step.next_atom_index,
                                    step.next_quote_index,
                                    step.next_plain_index,
                                    step.next_block_index,
                                    step.next_at_line_prefix,
                                    step.next_directive_mode,
                                    next_stack,
                                    next_maximum,
                                    built.push(step.token),
                                    token_limit,
                                    flow_limit,
                                    (fuel - 1) as nat,
                                )
                            }
                        },
                    }
                }
            },
        }
    }
}

pub closed spec fn scan_profile1_completed_tokens_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    limits: CompletedTokenLimitsView,
) -> Result<CompletedTokenSourceView, CompletedTokenErrorView> {
    match crate::layout::analyze_profile1_layout_spec(
        atomized,
        crate::structural::canonical_layout_limits_spec(),
    ) {
        Err(error) => Err(
            token_error_spec(CompletedTokenErrorKind::InputLayoutMismatch, error.byte_offset),
        ),
        Ok(canonical_layout) => if canonical_layout != layout {
            Err(token_error_spec(CompletedTokenErrorKind::InputLayoutMismatch, atomized.bom_bytes))
        } else {
            match crate::structural::scan_profile1_structural_lexemes_spec(
                atomized,
                layout,
                crate::structural::canonical_structural_scan_limits_spec(),
            ) {
                Err(error) => Err(
                    token_error_spec(
                        CompletedTokenErrorKind::InputStructuralMismatch,
                        error.byte_offset,
                    ),
                ),
                Ok(canonical_structural) => if canonical_structural != structural {
                    Err(
                        token_error_spec(
                            CompletedTokenErrorKind::InputStructuralMismatch,
                            atomized.bom_bytes,
                        ),
                    )
                } else {
                    match crate::quoted::scan_profile1_quoted_scalars_spec(
                        atomized,
                        layout,
                        structural,
                        crate::quoted::canonical_quoted_scalar_limits_spec(),
                    ) {
                        Err(error) => Err(
                            token_error_spec(
                                CompletedTokenErrorKind::InputQuotedMismatch,
                                error.byte_offset,
                            ),
                        ),
                        Ok(canonical_quoted) => if canonical_quoted != quoted {
                            Err(
                                token_error_spec(
                                    CompletedTokenErrorKind::InputQuotedMismatch,
                                    atomized.bom_bytes,
                                ),
                            )
                        } else {
                            match crate::plain::scan_profile1_plain_scalars_spec(
                                atomized,
                                layout,
                                structural,
                                quoted,
                                crate::plain::canonical_plain_scalar_limits_spec(),
                            ) {
                                Err(error) => Err(
                                    token_error_spec(
                                        CompletedTokenErrorKind::InputPlainMismatch,
                                        error.byte_offset,
                                    ),
                                ),
                                Ok(canonical_plain) => if canonical_plain != plain {
                                    Err(
                                        token_error_spec(
                                            CompletedTokenErrorKind::InputPlainMismatch,
                                            atomized.bom_bytes,
                                        ),
                                    )
                                } else {
                                    match crate::block::scan_profile1_block_scalars_spec(
                                        atomized,
                                        layout,
                                        structural,
                                        quoted,
                                        plain,
                                        crate::block::canonical_block_scalar_limits_spec(),
                                    ) {
                                        Err(error) => Err(
                                            token_error_spec(
                                                CompletedTokenErrorKind::InputBlockMismatch,
                                                error.byte_offset,
                                            ),
                                        ),
                                        Ok(canonical_block) => if canonical_block != block {
                                            Err(
                                                token_error_spec(
                                                    CompletedTokenErrorKind::InputBlockMismatch,
                                                    atomized.bom_bytes,
                                                ),
                                            )
                                        } else {
                                            match completed_token_scan_tail_spec(
                                                atomized.atoms,
                                                quoted.scalars,
                                                plain.scalars,
                                                block.scalars,
                                                atomized.source_len_bytes,
                                                0,
                                                0,
                                                0,
                                                0,
                                                true,
                                                true,
                                                Seq::empty(),
                                                0,
                                                Seq::empty(),
                                                effective_token_limit_spec(limits),
                                                effective_flow_depth_spec(limits),
                                                (atomized.atoms.len() + 1) as nat,
                                            ) {
                                                Err(error) => Err(error),
                                                Ok(success) => Ok(
                                                    completed_token_source_from_tail_spec(
                                                        atomized,
                                                        layout,
                                                        structural,
                                                        quoted,
                                                        plain,
                                                        block,
                                                        success,
                                                    ),
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
        },
    }
}

pub open spec fn completed_token_empty_canonical_inputs_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
) -> bool {
    atomized.atoms.len() == 0 && crate::layout::analyze_profile1_layout_spec(
        atomized,
        crate::structural::canonical_layout_limits_spec(),
    ) == Ok(layout) && crate::structural::scan_profile1_structural_lexemes_spec(
        atomized,
        layout,
        crate::structural::canonical_structural_scan_limits_spec(),
    ) == Ok(structural) && crate::quoted::scan_profile1_quoted_scalars_spec(
        atomized,
        layout,
        structural,
        crate::quoted::canonical_quoted_scalar_limits_spec(),
    ) == Ok(quoted) && crate::plain::scan_profile1_plain_scalars_spec(
        atomized,
        layout,
        structural,
        quoted,
        crate::plain::canonical_plain_scalar_limits_spec(),
    ) == Ok(plain) && crate::block::scan_profile1_block_scalars_spec(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        crate::block::canonical_block_scalar_limits_spec(),
    ) == Ok(block)
}

proof fn lemma_nonempty_source_is_not_empty_canonical_inputs(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
)
    requires
        atomized.atoms.len() > 0,
    ensures
        !completed_token_empty_canonical_inputs_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
        ),
{
    reveal(completed_token_empty_canonical_inputs_spec);
}

pub closed spec fn completed_token_source_corresponds_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
) -> bool {
    exists|limits: CompletedTokenLimitsView|
        scan_profile1_completed_tokens_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            limits,
        ) == Ok(source)
}

pub open spec fn completed_token_public_semantics_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
) -> bool {
    completed_token_source_corresponds_spec(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        block,
        source,
    ) && completed_token_absolute_limits_spec(source)
}

pub closed spec fn completed_token_source_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
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
    ) && crate::block::block_scalar_source_well_formed_spec(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        block,
    ) && completed_token_source_corresponds_spec(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        block,
        source,
    ) && completed_token_partition_spec(atomized, source) && completed_token_flow_balanced_spec(
        source.tokens,
    ) && completed_token_public_semantics_spec(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        block,
        source,
    )
}

pub proof fn lemma_completed_tokens_well_formed_has_exact_partition_and_balance(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
)
    requires
        completed_token_source_well_formed_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            source,
        ),
    ensures
        completed_token_partition_spec(atomized, source),
        completed_token_flow_balanced_spec(source.tokens),
        completed_token_public_semantics_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            source,
        ),
{
    reveal(completed_token_source_well_formed_spec);
}

pub proof fn lemma_completed_tokens_well_formed_has_exact_formation_and_limits(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
)
    requires
        completed_token_source_well_formed_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            source,
        ),
    ensures
        completed_token_source_corresponds_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            source,
        ),
        completed_token_absolute_limits_spec(source),
{
    reveal(completed_token_source_well_formed_spec);
    reveal(completed_token_public_semantics_spec);
}

pub proof fn lemma_completed_tokens_reject_noncanonical_layout(
    atomized: AtomizedSourceView,
    canonical_layout: LayoutSourceView,
    supplied_layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    limits: CompletedTokenLimitsView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(canonical_layout),
        canonical_layout != supplied_layout,
    ensures
        scan_profile1_completed_tokens_spec(
            atomized,
            supplied_layout,
            structural,
            quoted,
            plain,
            block,
            limits,
        ) == Err(
            CompletedTokenErrorView {
                kind: CompletedTokenErrorKind::InputLayoutMismatch,
                byte_offset: atomized.bom_bytes,
            },
        ),
{
    reveal(scan_profile1_completed_tokens_spec);
    reveal(token_error_spec);
}

pub proof fn lemma_noncanonical_layout_cannot_correspond_to_completed_tokens(
    atomized: AtomizedSourceView,
    canonical_layout: LayoutSourceView,
    supplied_layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(
            atomized,
            crate::structural::canonical_layout_limits_spec(),
        ) == Ok(canonical_layout),
        canonical_layout != supplied_layout,
    ensures
        !completed_token_source_corresponds_spec(
            atomized,
            supplied_layout,
            structural,
            quoted,
            plain,
            block,
            source,
        ),
{
    reveal(completed_token_source_corresponds_spec);
    if completed_token_source_corresponds_spec(
        atomized,
        supplied_layout,
        structural,
        quoted,
        plain,
        block,
        source,
    ) {
        let limits = choose|limits: CompletedTokenLimitsView|
            scan_profile1_completed_tokens_spec(
                atomized,
                supplied_layout,
                structural,
                quoted,
                plain,
                block,
                limits,
            ) == Ok(source);
        lemma_completed_tokens_reject_noncanonical_layout(
            atomized,
            canonical_layout,
            supplied_layout,
            structural,
            quoted,
            plain,
            block,
            limits,
        );
        assert(false);
    }
}

pub proof fn lemma_noncanonical_structural_cannot_correspond_to_completed_tokens(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    canonical_structural: StructuralLexemeSourceView,
    supplied_structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
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
        ) == Ok(canonical_structural),
        canonical_structural != supplied_structural,
    ensures
        !completed_token_source_corresponds_spec(
            atomized,
            layout,
            supplied_structural,
            quoted,
            plain,
            block,
            source,
        ),
{
    reveal(completed_token_source_corresponds_spec);
    reveal(scan_profile1_completed_tokens_spec);
}

pub proof fn lemma_noncanonical_quoted_cannot_correspond_to_completed_tokens(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    canonical_quoted: QuotedScalarSourceView,
    supplied_quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
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
        ) == Ok(canonical_quoted),
        canonical_quoted != supplied_quoted,
    ensures
        !completed_token_source_corresponds_spec(
            atomized,
            layout,
            structural,
            supplied_quoted,
            plain,
            block,
            source,
        ),
{
    reveal(completed_token_source_corresponds_spec);
    reveal(scan_profile1_completed_tokens_spec);
}

pub proof fn lemma_noncanonical_plain_cannot_correspond_to_completed_tokens(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    canonical_plain: PlainScalarSourceView,
    supplied_plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
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
        ) == Ok(canonical_plain),
        canonical_plain != supplied_plain,
    ensures
        !completed_token_source_corresponds_spec(
            atomized,
            layout,
            structural,
            quoted,
            supplied_plain,
            block,
            source,
        ),
{
    reveal(completed_token_source_corresponds_spec);
    reveal(scan_profile1_completed_tokens_spec);
}

pub proof fn lemma_noncanonical_block_cannot_correspond_to_completed_tokens(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    canonical_block: BlockScalarSourceView,
    supplied_block: BlockScalarSourceView,
    source: CompletedTokenSourceView,
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
        crate::block::scan_profile1_block_scalars_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            crate::block::canonical_block_scalar_limits_spec(),
        ) == Ok(canonical_block),
        canonical_block != supplied_block,
    ensures
        !completed_token_source_corresponds_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            supplied_block,
            source,
        ),
{
    reveal(completed_token_source_corresponds_spec);
    reveal(scan_profile1_completed_tokens_spec);
}

pub proof fn lemma_empty_input_fits_completed_token_limits(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    limits: CompletedTokenLimitsView,
)
    requires
        completed_token_empty_canonical_inputs_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
        ),
    ensures
        scan_profile1_completed_tokens_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            limits,
        ).is_ok(),
{
    reveal(scan_profile1_completed_tokens_spec);
    reveal(completed_token_empty_canonical_inputs_spec);
    reveal(completed_token_scan_tail_spec);
    reveal(effective_token_limit_spec);
    reveal(effective_flow_depth_spec);
}

pub proof fn lemma_empty_completed_token_scan_has_no_tokens(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    limits: CompletedTokenLimitsView,
    source: CompletedTokenSourceView,
)
    requires
        atomized.atoms.len() == 0,
        scan_profile1_completed_tokens_spec(
            atomized,
            layout,
            structural,
            quoted,
            plain,
            block,
            limits,
        ) == Ok(source),
    ensures
        source.tokens.len() == 0,
{
    reveal(scan_profile1_completed_tokens_spec);
}

closed spec fn is_space_or_tab_spec(kind: LexicalAtomKind) -> bool {
    token_is_space_or_tab_spec(kind)
}

fn is_space_or_tab(kind: LexicalAtomKind) -> (result: bool)
    ensures
        result == token_is_space_or_tab_spec(kind),
{
    reveal(token_is_space_or_tab_spec);
    kind == LexicalAtomKind::Space || kind == LexicalAtomKind::Tab
}

fn is_flow_indicator(kind: LexicalAtomKind) -> (result: bool)
    ensures
        result == token_is_flow_indicator_spec(kind),
{
    kind == LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart) || kind
        == LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd)
}

#[allow(clippy::manual_range_contains)]
fn is_hex(code_point: u32) -> (result: bool)
    ensures
        result == token_is_hex_spec(code_point),
{
    (0x30 <= code_point && code_point <= 0x39) || (0x41 <= code_point && code_point <= 0x46) || (
    0x61 <= code_point && code_point <= 0x66)
}

pub open spec fn is_decimal_spec(code_point: u32) -> bool {
    0x30 <= code_point && code_point <= 0x39
}

#[allow(clippy::manual_range_contains)]
fn is_decimal(code_point: u32) -> (result: bool)
    ensures
        result == is_decimal_spec(code_point),
{
    0x30 <= code_point && code_point <= 0x39
}

#[allow(clippy::manual_range_contains)]
fn is_word(code_point: u32) -> (result: bool)
    ensures
        result == token_is_word_spec(code_point),
{
    is_decimal(code_point) || (0x41 <= code_point && code_point <= 0x5a) || (0x61 <= code_point
        && code_point <= 0x7a) || code_point == 0x2d
}

#[allow(clippy::manual_range_contains)]
fn is_ns_uri_char(code_point: u32) -> (result: bool)
    ensures
        result == token_is_ns_uri_char_spec(code_point),
{
    is_word(code_point) || code_point == 0x25 || code_point == 0x23 || code_point == 0x3b
        || code_point == 0x2f || code_point == 0x3f || code_point == 0x3a || code_point == 0x40
        || code_point == 0x26 || code_point == 0x3d || code_point == 0x2b || code_point == 0x24
        || code_point == 0x2c || code_point == 0x5f || code_point == 0x2e || code_point == 0x21
        || code_point == 0x7e || code_point == 0x2a || code_point == 0x27 || code_point == 0x28
        || code_point == 0x29 || code_point == 0x5b || code_point == 0x5d
}

fn is_ns_tag_char(code_point: u32) -> (result: bool)
    ensures
        result == token_is_ns_tag_char_spec(code_point),
{
    is_ns_uri_char(code_point) && code_point != 0x21 && code_point != 0x2c && code_point != 0x5b
        && code_point != 0x5d && code_point != 0x7b && code_point != 0x7d
}

fn first_invalid_tag_alphabet(
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
    tag_alphabet: bool,
) -> (invalid: Option<usize>)
    requires
        start <= end <= atoms@.len(),
    ensures
        match invalid {
            Some(index) => token_first_invalid_tag_alphabet_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                tag_alphabet,
                (end - start) as nat,
            ) == Some(index as int),
            None => token_first_invalid_tag_alphabet_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                tag_alphabet,
                (end - start) as nat,
            ).is_none(),
        },
        match invalid {
            Some(index) => start <= index < end,
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_first_invalid_tag_alphabet_spec(
        views,
        start as int,
        end as int,
        tag_alphabet,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == token_first_invalid_tag_alphabet_spec(
                views,
                index as int,
                end as int,
                tag_alphabet,
                (end - index) as nat,
            ),
            expected == token_first_invalid_tag_alphabet_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                tag_alphabet,
                (end - start) as nat,
            ),
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let valid = if tag_alphabet {
            is_ns_tag_char(atoms[index].code_point())
        } else {
            is_ns_uri_char(atoms[index].code_point())
        };
        if !valid {
            proof {
                reveal(token_first_invalid_tag_alphabet_spec);
                assert(expected == Some(index as int));
            }
            return Some(index);
        }
        proof {
            reveal(token_first_invalid_tag_alphabet_spec);
        }
        index += 1;
    }
    proof {
        reveal(token_first_invalid_tag_alphabet_spec);
    }
    None
}

fn first_invalid_tag_prefix(atoms: &[LexicalAtom], start: usize, end: usize) -> (invalid: Option<
    usize,
>)
    requires
        start < end <= atoms@.len(),
    ensures
        match invalid {
            Some(index) => token_first_invalid_tag_prefix_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
            ) == Some(index as int),
            None => token_first_invalid_tag_prefix_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
            ).is_none(),
        },
        match invalid {
            Some(index) => start <= index < end,
            None => true,
        },
{
    proof {
        reveal(token_first_invalid_tag_prefix_spec);
    }
    if atoms[start].code_point() == 0x21 {
        first_invalid_tag_alphabet(atoms, start + 1, end, false)
    } else if !is_ns_tag_char(atoms[start].code_point()) {
        Some(start)
    } else {
        first_invalid_tag_alphabet(atoms, start + 1, end, false)
    }
}

fn first_bom(atoms: &[LexicalAtom], start: usize, end: usize) -> (invalid: Option<usize>)
    requires
        start <= end <= atoms@.len(),
    ensures
        match invalid {
            Some(index) => token_first_bom_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ) == Some(index as int),
            None => token_first_bom_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ).is_none(),
        },
        match invalid {
            Some(index) => start <= index < end,
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_first_bom_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == token_first_bom_spec(views, index as int, end as int, (end - index) as nat),
            expected == token_first_bom_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ),
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if atoms[index].code_point() == 0xfeff {
            proof {
                reveal(token_first_bom_spec);
                assert(expected == Some(index as int));
            }
            return Some(index);
        }
        proof {
            reveal(token_first_bom_spec);
        }
        index += 1;
    }
    proof {
        reveal(token_first_bom_spec);
    }
    None
}

fn run_of_space_or_tab(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start < atoms@.len(),
        is_space_or_tab_spec(atoms@[start as int]@.kind),
    ensures
        start < end <= atoms@.len(),
        end == token_forward_run_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            atoms@.len() as int,
            0,
            (atoms@.len() - start) as nat,
        ),
{
    let ghost expected = token_forward_run_end_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        atoms@.len() as int,
        0,
        (atoms@.len() - start) as nat,
    );
    let mut end = start + 1;
    proof {
        reveal(token_forward_run_end_spec);
    }
    while end < atoms.len() && is_space_or_tab(atoms[end].kind())
        invariant
            start < end <= atoms@.len(),
            expected == token_forward_run_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                end as int,
                atoms@.len() as int,
                0,
                (atoms@.len() - end) as nat,
            ),
        decreases atoms.len() - end,
    {
        proof {
            reveal(token_forward_run_end_spec);
        }
        end += 1;
    }
    proof {
        reveal(token_forward_run_end_spec);
    }
    end
}

fn line_tail_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start <= atoms@.len(),
    ensures
        start <= end <= atoms@.len(),
        start < atoms@.len() && atoms@[start as int]@.kind != LexicalAtomKind::LineFeed ==> start
            < end,
        end == token_forward_run_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            atoms@.len() as int,
            1,
            (atoms@.len() - start) as nat,
        ),
{
    let ghost expected = token_forward_run_end_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        atoms@.len() as int,
        1,
        (atoms@.len() - start) as nat,
    );
    let mut end = start;
    while end < atoms.len() && atoms[end].kind() != LexicalAtomKind::LineFeed
        invariant
            start <= end <= atoms@.len(),
            expected == token_forward_run_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                end as int,
                atoms@.len() as int,
                1,
                (atoms@.len() - end) as nat,
            ),
        decreases atoms.len() - end,
    {
        proof {
            reveal(token_forward_run_end_spec);
        }
        end += 1;
    }
    proof {
        reveal(token_forward_run_end_spec);
    }
    end
}

fn property_name_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start <= atoms@.len(),
    ensures
        start <= end <= atoms@.len(),
        end == token_forward_run_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            atoms@.len() as int,
            2,
            (atoms@.len() - start) as nat,
        ),
{
    let ghost expected = token_forward_run_end_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        atoms@.len() as int,
        2,
        (atoms@.len() - start) as nat,
    );
    let mut end = start;
    while end < atoms.len() && !is_space_or_tab(atoms[end].kind()) && atoms[end].kind()
        != LexicalAtomKind::LineFeed && !is_flow_indicator(atoms[end].kind())
        invariant
            start <= end <= atoms@.len(),
            expected == token_forward_run_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                end as int,
                atoms@.len() as int,
                2,
                (atoms@.len() - end) as nat,
            ),
        decreases atoms.len() - end,
    {
        proof {
            reveal(token_forward_run_end_spec);
        }
        end += 1;
    }
    proof {
        reveal(token_forward_run_end_spec);
    }
    end
}

fn make_completed_part(
    atoms: &[LexicalAtom],
    kind: CompletedTokenPartKind,
    start: usize,
    end: usize,
) -> (part: CompletedTokenPart)
    requires
        start < end <= atoms@.len(),
    ensures
        part@ == token_part_for_range_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            kind,
            start as int,
            end as int,
        ),
        part@ == (CompletedTokenPartView {
            kind,
            start_atom_index: start as u64,
            end_atom_index: end as u64,
            byte_start: atoms@[start as int]@.span.start.byte_offset,
            byte_end: atoms@[(end - 1) as int]@.span.end.byte_offset,
        }),
{
    proof {
        reveal(token_part_for_range_spec);
    }
    CompletedTokenPart {
        kind,
        start_atom_index: start as u64,
        end_atom_index: end as u64,
        byte_start: atoms[start].span().start().byte_offset(),
        byte_end: atoms[end - 1].span().end().byte_offset(),
    }
}

#[allow(clippy::too_many_arguments)]
fn make_completed_token(
    atoms: &[LexicalAtom],
    kind: CompletedTokenKind,
    start: usize,
    end: usize,
    scalar_index: Option<u64>,
    yaml_major: Option<u64>,
    yaml_minor: Option<u64>,
    parts: Vec<CompletedTokenPart>,
) -> (token: CompletedToken)
    requires
        start < end <= atoms@.len(),
    ensures
        token@ == token_for_range_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            kind,
            start as int,
            end as int,
            scalar_index,
            yaml_major,
            yaml_minor,
            completed_token_part_views_spec(parts@),
        ),
        token@.kind == kind,
        token@.start_atom_index == start,
        token@.end_atom_index == end,
        token@.byte_start == atoms@[start as int]@.span.start.byte_offset,
        token@.byte_end == atoms@[(end - 1) as int]@.span.end.byte_offset,
        token@.start_line_number == atoms@[start as int]@.span.start.line,
        token@.end_line_number == atoms@[(end - 1) as int]@.span.start.line,
        token@.scalar_index == scalar_index,
        token@.yaml_major == yaml_major,
        token@.yaml_minor == yaml_minor,
        token@.parts == completed_token_part_views_spec(parts@),
{
    proof {
        reveal(token_for_range_spec);
    }
    CompletedToken {
        kind,
        start_line_number: atoms[start].span().start().line(),
        end_line_number: atoms[end - 1].span().start().line(),
        start_atom_index: start as u64,
        end_atom_index: end as u64,
        byte_start: atoms[start].span().start().byte_offset(),
        byte_end: atoms[end - 1].span().end().byte_offset(),
        scalar_index,
        yaml_major,
        yaml_minor,
        parts,
    }
}

fn completed_token_range_valid(atoms: &[LexicalAtom], token: &CompletedToken) -> (valid: bool)
    ensures
        valid == completed_token_range_spec(crate::atom::lexical_atom_views_spec(atoms@), token@),
{
    if token.start_atom_index >= token.end_atom_index || token.end_atom_index > atoms.len() as u64 {
        return false;
    }
    let start = token.start_atom_index as usize;
    let end = token.end_atom_index as usize;
    assert(start < end <= atoms.len());
    let valid = token.byte_start == atoms[start].span().start().byte_offset() && token.byte_end
        == atoms[end - 1].span().end().byte_offset() && token.start_line_number
        == atoms[start].span().start().line() && token.end_line_number == atoms[end
        - 1].span().start().line();
    proof {
        reveal(completed_token_range_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    valid
}

fn range_is_yaml(atoms: &[LexicalAtom], start: usize, end: usize) -> (yes: bool)
    requires
        start <= end <= atoms@.len(),
    ensures
        yes == token_range_is_ascii_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            seq![0x59u32, 0x41u32, 0x4du32, 0x4cu32],
        ),
{
    proof {
        reveal(token_range_is_ascii_spec);
    }
    end - start == 4 && atoms[start].code_point() == 0x59 && atoms[start + 1].code_point() == 0x41
        && atoms[start + 2].code_point() == 0x4d && atoms[start + 3].code_point() == 0x4c
}

fn range_is_tag(atoms: &[LexicalAtom], start: usize, end: usize) -> (yes: bool)
    requires
        start <= end <= atoms@.len(),
    ensures
        yes == token_range_is_ascii_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            seq![0x54u32, 0x41u32, 0x47u32],
        ),
{
    proof {
        reveal(token_range_is_ascii_spec);
    }
    end - start == 3 && atoms[start].code_point() == 0x54 && atoms[start + 1].code_point() == 0x41
        && atoms[start + 2].code_point() == 0x47
}

fn directive_payload_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start < atoms@.len(),
        atoms@[start as int]@.kind == LexicalAtomKind::Indicator(YamlIndicator::Directive),
    ensures
        start < end <= atoms@.len(),
        end == token_directive_payload_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
        ),
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(token_is_space_or_tab_spec);
    }
    let line_end = line_tail_end(atoms, start);
    assert(atoms@[start as int]@.kind != LexicalAtomKind::LineFeed);
    let mut cursor = start + 1;
    let mut comment = line_end;
    let ghost expected_comment = token_directive_comment_start_spec(
        atom_views,
        start as int,
        (start + 1) as int,
        line_end as int,
        (line_end - start) as nat,
    );
    while cursor < line_end
        invariant_except_break
            start < cursor <= line_end <= atoms@.len(),
            comment == line_end,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            expected_comment == token_directive_comment_start_spec(
                atom_views,
                start as int,
                (start + 1) as int,
                line_end as int,
                (line_end - start) as nat,
            ),
            expected_comment == token_directive_comment_start_spec(
                atom_views,
                start as int,
                cursor as int,
                line_end as int,
                (line_end - cursor + 1) as nat,
            ),
        ensures
            start < cursor <= line_end <= atoms@.len(),
            start < comment <= line_end,
            comment as int == expected_comment,
        decreases line_end - cursor,
    {
        assert(atom_views[cursor as int] == atoms[cursor as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        assert(atom_views[(cursor - 1) as int] == atoms[(cursor - 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if atoms[cursor].kind() == LexicalAtomKind::Indicator(YamlIndicator::Comment) && cursor
            > start && is_space_or_tab(atoms[cursor - 1].kind()) {
            comment = cursor;
            proof {
                reveal(token_directive_comment_start_spec);
                assert(expected_comment == cursor as int);
            }
            break;
        }
        proof {
            reveal(token_directive_comment_start_spec);
            reveal(token_is_space_or_tab_spec);
        }
        cursor += 1;
    }
    proof {
        if cursor == line_end {
            reveal(token_directive_comment_start_spec);
        }
    }
    let mut end = comment;
    let ghost expected_end = token_trim_trailing_separation_spec(
        atom_views,
        (start + 1) as int,
        comment as int,
        (comment - start) as nat,
    );
    while end > start + 1 && is_space_or_tab(atoms[end - 1].kind())
        invariant
            start < end <= comment <= line_end <= atoms@.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            expected_end == token_trim_trailing_separation_spec(
                atom_views,
                (start + 1) as int,
                comment as int,
                (comment - start) as nat,
            ),
            expected_end == token_trim_trailing_separation_spec(
                atom_views,
                (start + 1) as int,
                end as int,
                (end - start) as nat,
            ),
        decreases end - (start + 1),
    {
        assert(atom_views[(end - 1) as int] == atoms[(end - 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        proof {
            reveal(token_trim_trailing_separation_spec);
            reveal(token_is_space_or_tab_spec);
        }
        end -= 1;
    }
    proof {
        reveal(token_trim_trailing_separation_spec);
        reveal(token_directive_payload_end_spec);
    }
    end
}

fn skip_inline_separation(atoms: &[LexicalAtom], start: usize, end: usize) -> (cursor: usize)
    requires
        start <= end <= atoms@.len(),
    ensures
        start <= cursor <= end,
        cursor < end ==> !is_space_or_tab_spec(atoms@[cursor as int]@.kind),
        cursor == token_forward_run_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            0,
            (end - start) as nat,
        ),
{
    let ghost expected = token_forward_run_end_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        end as int,
        0,
        (end - start) as nat,
    );
    let mut cursor = start;
    while cursor < end && is_space_or_tab(atoms[cursor].kind())
        invariant
            start <= cursor <= end <= atoms@.len(),
            expected == token_forward_run_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                cursor as int,
                end as int,
                0,
                (end - cursor) as nat,
            ),
        decreases end - cursor,
    {
        proof {
            reveal(token_forward_run_end_spec);
        }
        cursor += 1;
    }
    proof {
        reveal(token_forward_run_end_spec);
    }
    cursor
}

fn directive_parameter_end(atoms: &[LexicalAtom], start: usize, end: usize) -> (cursor: usize)
    requires
        start <= end <= atoms@.len(),
    ensures
        start <= cursor <= end,
        start < end && !token_is_space_or_tab_spec(atoms@[start as int]@.kind) ==> start < cursor,
        cursor == token_forward_run_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            3,
            (end - start) as nat,
        ),
{
    let ghost expected = token_forward_run_end_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        end as int,
        3,
        (end - start) as nat,
    );
    let mut cursor = start;
    while cursor < end && !is_space_or_tab(atoms[cursor].kind())
        invariant
            start <= cursor <= end <= atoms@.len(),
            expected == token_forward_run_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                cursor as int,
                end as int,
                3,
                (end - cursor) as nat,
            ),
        decreases end - cursor,
    {
        proof {
            reveal(token_forward_run_end_spec);
        }
        cursor += 1;
    }
    proof {
        reveal(token_forward_run_end_spec);
    }
    cursor
}

fn first_code_point(atoms: &[LexicalAtom], start: usize, end: usize, code_point: u32) -> (cursor:
    usize)
    requires
        start <= end <= atoms@.len(),
    ensures
        start <= cursor <= end,
        cursor == token_first_code_point_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            code_point,
            (end - start) as nat,
        ),
{
    let ghost expected = token_first_code_point_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        end as int,
        code_point,
        (end - start) as nat,
    );
    let mut cursor = start;
    while cursor < end && atoms[cursor].code_point() != code_point
        invariant
            start <= cursor <= end <= atoms@.len(),
            expected == token_first_code_point_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                cursor as int,
                end as int,
                code_point,
                (end - cursor) as nat,
            ),
        decreases end - cursor,
    {
        proof {
            reveal(token_first_code_point_spec);
        }
        cursor += 1;
    }
    proof {
        reveal(token_first_code_point_spec);
    }
    cursor
}

fn first_invalid_word(atoms: &[LexicalAtom], start: usize, end: usize) -> (invalid: Option<usize>)
    requires
        start <= end <= atoms@.len(),
    ensures
        match invalid {
            Some(index) => start <= index < end,
            None => true,
        },
        match invalid {
            Some(index) => token_first_invalid_word_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ) == Some(index as int),
            None => token_first_invalid_word_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ).is_none(),
        },
{
    let ghost expected = token_first_invalid_word_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        end as int,
        (end - start) as nat,
    );
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(token_is_word_spec);
    }
    let mut cursor = start;
    while cursor < end
        invariant
            start <= cursor <= end <= atoms@.len(),
            expected == token_first_invalid_word_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ),
            expected == token_first_invalid_word_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                cursor as int,
                end as int,
                (end - cursor) as nat,
            ),
        decreases end - cursor,
    {
        assert(crate::atom::lexical_atom_views_spec(atoms@)[cursor as int] == atoms[cursor as int]@)
            by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if !is_word(atoms[cursor].code_point()) {
            proof {
                reveal(token_first_invalid_word_spec);
                assert(expected == Some(cursor as int));
            }
            return Some(cursor);
        }
        proof {
            reveal(token_first_invalid_word_spec);
        }
        cursor += 1;
    }
    proof {
        reveal(token_first_invalid_word_spec);
    }
    None
}

fn verbatim_tag_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start <= atoms@.len(),
    ensures
        start <= end <= atoms@.len(),
        end == token_verbatim_tag_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            (atoms@.len() - start) as nat,
        ),
{
    let ghost expected = token_verbatim_tag_end_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        start as int,
        (atoms@.len() - start) as nat,
    );
    let mut end = start;
    while end < atoms.len() && atoms[end].code_point() != 0x3e && !is_space_or_tab(
        atoms[end].kind(),
    ) && atoms[end].kind() != LexicalAtomKind::LineFeed
        invariant
            start <= end <= atoms@.len(),
            expected == token_verbatim_tag_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                end as int,
                (atoms@.len() - end) as nat,
            ),
        decreases atoms.len() - end,
    {
        proof {
            reveal(token_verbatim_tag_end_spec);
        }
        end += 1;
    }
    proof {
        reveal(token_verbatim_tag_end_spec);
    }
    end
}

fn valid_tag_handle(atoms: &[LexicalAtom], start: usize, end: usize) -> (valid: bool)
    requires
        start < end <= atoms@.len(),
    ensures
        valid == token_valid_tag_handle_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
        ),
{
    if end - start == 1 {
        atoms[start].code_point() == 0x21
    } else if end - start == 2 {
        atoms[start].code_point() == 0x21 && atoms[start + 1].code_point() == 0x21
    } else {
        let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
        let ghost expected = token_valid_tag_handle_spec(atom_views, start as int, end as int);
        let mut valid = atoms[start].code_point() == 0x21 && atoms[end - 1].code_point() == 0x21;
        let mut index = start + 1;
        proof {
            assert(atom_views[start as int] == atoms[start as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            assert(atom_views[(end - 1) as int] == atoms[(end - 1) as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            reveal(crate::atom::lexical_atom_views_spec);
            reveal(token_is_word_spec);
            reveal(token_valid_tag_handle_spec);
        }
        while valid && index + 1 < end
            invariant
                start < index < end <= atoms@.len(),
                atom_views == crate::atom::lexical_atom_views_spec(atoms@),
                expected == (valid && token_first_invalid_word_spec(
                    atom_views,
                    index as int,
                    (end - 1) as int,
                    (end - index) as nat,
                ).is_none()),
            decreases end - index,
        {
            assert(atom_views[index as int] == atoms[index as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            if !is_word(atoms[index].code_point()) {
                valid = false;
            }
            proof {
                reveal(token_first_invalid_word_spec);
            }
            index += 1;
        }
        proof {
            reveal(token_first_invalid_word_spec);
        }
        valid
    }
}

fn parse_decimal_component(
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
    error_kind: CompletedTokenErrorKind,
) -> (result: Result<u64, CompletedTokenError>)
    requires
        start <= end <= atoms@.len(),
    ensures
        token_parse_decimal_component_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            error_kind,
        ) == match result {
            Ok(value) => Ok(value),
            Err(error) => Err(error@),
        },
        match result {
            Ok(_) => start < end,
            Err(_) => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost whole_expected = token_parse_decimal_component_spec(
        atom_views,
        start as int,
        end as int,
        error_kind,
    );
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(token_error_spec);
    }
    if start == end {
        let offset = if start < atoms.len() {
            atoms[start].span().start().byte_offset()
        } else if !atoms.is_empty() {
            atoms[atoms.len() - 1].span().end().byte_offset()
        } else {
            0
        };
        let error = CompletedTokenError::at(error_kind, offset);
        proof {
            reveal(token_parse_decimal_component_spec);
            reveal(token_offset_at_or_end_spec);
            assert(whole_expected == Err(error@));
        }
        return Err(error);
    }
    let mut value = 0u64;
    let mut index = start;
    let ghost expected = token_decimal_tail_spec(
        atom_views,
        start as int,
        end as int,
        0,
        error_kind,
        (end - start) as nat,
    );
    proof {
        reveal(token_parse_decimal_component_spec);
        assert(whole_expected == expected);
    }
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            whole_expected == token_parse_decimal_component_spec(
                atom_views,
                start as int,
                end as int,
                error_kind,
            ),
            whole_expected == expected,
            expected == token_decimal_tail_spec(
                atom_views,
                index as int,
                end as int,
                value,
                error_kind,
                (end - index) as nat,
            ),
        decreases end - index,
    {
        assert(atom_views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let code_point = atoms[index].code_point();
        if !is_decimal(code_point) {
            let error = CompletedTokenError::at(
                error_kind,
                atoms[index].span().start().byte_offset(),
            );
            proof {
                reveal(token_decimal_tail_spec);
                reveal(token_parse_decimal_component_spec);
                assert(expected == Err(error@));
                assert(whole_expected == Err(error@));
            }
            return Err(error);
        }
        reveal(is_decimal_spec);
        assert(0x30 <= code_point);
        let digit = (code_point - 0x30) as u64;
        if value > (u64::MAX - digit) / 10 {
            let error = CompletedTokenError::at(
                error_kind,
                atoms[index].span().start().byte_offset(),
            );
            proof {
                reveal(token_decimal_tail_spec);
                reveal(token_parse_decimal_component_spec);
                assert(expected == Err(error@));
                assert(whole_expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            reveal(token_decimal_tail_spec);
        }
        value = value * 10 + digit;
        index += 1;
    }
    proof {
        reveal(token_decimal_tail_spec);
        reveal(token_parse_decimal_component_spec);
        assert(expected == Ok(value));
        assert(whole_expected == Ok(value));
    }
    Ok(value)
}

fn first_invalid_percent_escape(atoms: &[LexicalAtom], start: usize, end: usize) -> (invalid:
    Option<usize>)
    requires
        start <= end <= atoms@.len(),
    ensures
        match invalid {
            Some(index) => token_first_invalid_percent_escape_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ) == Some(index as int),
            None => token_first_invalid_percent_escape_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ).is_none(),
        },
        match invalid {
            Some(index) => start <= index <= end,
            None => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_first_invalid_percent_escape_spec(
        atom_views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(token_is_hex_spec);
    }
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == token_first_invalid_percent_escape_spec(
                atom_views,
                start as int,
                end as int,
                (end - start) as nat,
            ),
            expected == token_first_invalid_percent_escape_spec(
                atom_views,
                index as int,
                end as int,
                (end - index) as nat,
            ),
        decreases end - index,
    {
        assert(atom_views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if atoms[index].code_point() == 0x25 {
            if index + 1 >= end {
                proof {
                    reveal(token_first_invalid_percent_escape_spec);
                    assert(expected == Some(end as int));
                }
                return Some(end);
            }
            assert(atom_views[(index + 1) as int] == atoms[(index + 1) as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            if !is_hex(atoms[index + 1].code_point()) {
                proof {
                    reveal(token_first_invalid_percent_escape_spec);
                    assert(expected == Some((index + 1) as int));
                }
                return Some(index + 1);
            }
            if index + 2 >= end {
                proof {
                    reveal(token_first_invalid_percent_escape_spec);
                    assert(expected == Some(end as int));
                }
                return Some(end);
            }
            assert(atom_views[(index + 2) as int] == atoms[(index + 2) as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            if !is_hex(atoms[index + 2].code_point()) {
                proof {
                    reveal(token_first_invalid_percent_escape_spec);
                    assert(expected == Some((index + 2) as int));
                }
                return Some(index + 2);
            }
            proof {
                reveal(token_first_invalid_percent_escape_spec);
            }
            index += 3;
        } else {
            proof {
                reveal(token_first_invalid_percent_escape_spec);
            }
            index += 1;
        }
    }
    proof {
        reveal(token_first_invalid_percent_escape_spec);
    }
    None
}

fn offset_at_or_end(atoms: &[LexicalAtom], index: usize) -> (offset: u64)
    requires
        index <= atoms@.len(),
    ensures
        offset == token_offset_at_or_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            index as int,
        ),
{
    proof {
        reveal(token_offset_at_or_end_spec);
    }
    if index < atoms.len() {
        atoms[index].span().start().byte_offset()
    } else if !atoms.is_empty() {
        atoms[atoms.len() - 1].span().end().byte_offset()
    } else {
        0
    }
}

#[verifier::spinoff_prover]
#[verifier::rlimit(200)]
fn parse_directive_token(atoms: &[LexicalAtom], start: usize) -> (result: Result<
    CompletedToken,
    CompletedTokenError,
>)
    requires
        start < atoms@.len(),
        atoms@[start as int]@.kind == LexicalAtomKind::Indicator(YamlIndicator::Directive),
    ensures
        token_parse_directive_spec(crate::atom::lexical_atom_views_spec(atoms@), start as int)
            == match result {
            Ok(token) => Ok(token@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(token) => token@.start_atom_index == start && start < token@.end_atom_index
                <= atoms@.len(),
            Err(_) => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_parse_directive_spec(atom_views, start as int);
    proof {
        reveal(token_parse_directive_spec);
    }
    let token_end = directive_payload_end(atoms, start);
    let name_end = directive_parameter_end(atoms, start + 1, token_end);
    if name_end == start + 1 {
        let error = CompletedTokenError::at(
            CompletedTokenErrorKind::EmptyDirectiveName,
            offset_at_or_end(atoms, start + 1),
        );
        proof {
            reveal(token_parse_directive_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let mut parts = Vec::new();
    let name_part = make_completed_part(
        atoms,
        CompletedTokenPartKind::DirectiveName,
        start + 1,
        name_end,
    );
    proof {
        lemma_completed_token_part_views_push(parts@, name_part);
    }
    parts.push(name_part);
    let is_yaml = range_is_yaml(atoms, start + 1, name_end);
    let is_tag = range_is_tag(atoms, start + 1, name_end);
    if is_yaml {
        let parameter_start = skip_inline_separation(atoms, name_end, token_end);
        if parameter_start == token_end {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidYamlDirective,
                offset_at_or_end(atoms, name_end),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let parameter_end = directive_parameter_end(atoms, parameter_start, token_end);
        let trailing = skip_inline_separation(atoms, parameter_end, token_end);
        if trailing != token_end {
            assert(atom_views[trailing as int] == atoms[trailing as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidYamlDirective,
                atoms[trailing].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let dot = first_code_point(atoms, parameter_start, parameter_end, 0x2e);
        if dot == parameter_end {
            assert(atom_views[parameter_start as int] == atoms[parameter_start as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidYamlDirective,
                atoms[parameter_start].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let extra_dot = first_code_point(atoms, dot + 1, parameter_end, 0x2e);
        if extra_dot < parameter_end {
            assert(atom_views[extra_dot as int] == atoms[extra_dot as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidYamlDirective,
                atoms[extra_dot].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let major = match parse_decimal_component(
            atoms,
            parameter_start,
            dot,
            CompletedTokenErrorKind::InvalidYamlDirective,
        ) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    reveal(token_parse_directive_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        let minor = match parse_decimal_component(
            atoms,
            dot + 1,
            parameter_end,
            CompletedTokenErrorKind::InvalidYamlDirective,
        ) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    reveal(token_parse_directive_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        let major_part = make_completed_part(
            atoms,
            CompletedTokenPartKind::YamlMajor,
            parameter_start,
            dot,
        );
        proof {
            lemma_completed_token_part_views_push(parts@, major_part);
        }
        parts.push(major_part);
        let minor_part = make_completed_part(
            atoms,
            CompletedTokenPartKind::YamlMinor,
            dot + 1,
            parameter_end,
        );
        proof {
            lemma_completed_token_part_views_push(parts@, minor_part);
        }
        parts.push(minor_part);
        proof {
            assert(completed_token_part_views_spec(parts@) =~= seq![
                name_part@,
                major_part@,
                minor_part@,
            ]);
        }
        let token = make_completed_token(
            atoms,
            CompletedTokenKind::YamlDirective,
            start,
            token_end,
            None,
            Some(major),
            Some(minor),
            parts,
        );
        proof {
            reveal(completed_token_part_views_spec);
            reveal(token_parse_directive_spec);
            assert(expected == Ok(token@));
        }
        Ok(token)
    } else if is_tag {
        let handle_start = skip_inline_separation(atoms, name_end, token_end);
        if handle_start == token_end {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidTagDirective,
                offset_at_or_end(atoms, handle_start),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let handle_end = directive_parameter_end(atoms, handle_start, token_end);
        let prefix_start = skip_inline_separation(atoms, handle_end, token_end);
        if prefix_start == token_end {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidTagDirective,
                offset_at_or_end(atoms, prefix_start),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let prefix_end = directive_parameter_end(atoms, prefix_start, token_end);
        let trailing = skip_inline_separation(atoms, prefix_end, token_end);
        if trailing != token_end || !valid_tag_handle(atoms, handle_start, handle_end) {
            assert(atom_views[handle_start as int] == atoms[handle_start as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidTagDirective,
                atoms[handle_start].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if let Some(invalid) = first_invalid_tag_prefix(atoms, prefix_start, prefix_end) {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidTagDirective,
                offset_at_or_end(atoms, invalid),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if let Some(invalid) = first_invalid_percent_escape(atoms, prefix_start, prefix_end) {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidTagDirective,
                offset_at_or_end(atoms, invalid),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let handle_part = make_completed_part(
            atoms,
            CompletedTokenPartKind::TagHandle,
            handle_start,
            handle_end,
        );
        proof {
            lemma_completed_token_part_views_push(parts@, handle_part);
        }
        parts.push(handle_part);
        let prefix_part = make_completed_part(
            atoms,
            CompletedTokenPartKind::TagPrefix,
            prefix_start,
            prefix_end,
        );
        proof {
            lemma_completed_token_part_views_push(parts@, prefix_part);
        }
        parts.push(prefix_part);
        proof {
            assert(completed_token_part_views_spec(parts@) =~= seq![
                name_part@,
                handle_part@,
                prefix_part@,
            ]);
        }
        let token = make_completed_token(
            atoms,
            CompletedTokenKind::TagDirective,
            start,
            token_end,
            None,
            None,
            None,
            parts,
        );
        proof {
            reveal(completed_token_part_views_spec);
            reveal(token_parse_directive_spec);
            assert(expected == Ok(token@));
        }
        Ok(token)
    } else {
        if let Some(invalid) = first_bom(atoms, start + 1, token_end) {
            assert(atom_views[invalid as int] == atoms[invalid as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidDirectiveCharacter,
                atoms[invalid].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_directive_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let mut cursor = name_end;
        proof {
            assert(completed_token_part_views_spec(parts@) =~= seq![name_part@]);
        }
        let ghost expected_parts = token_reserved_directive_parts_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            name_end as int,
            token_end as int,
            completed_token_part_views_spec(parts@),
            (token_end - name_end + 1) as nat,
        );
        let mut _reserved_steps = 0usize;
        while cursor < token_end
            invariant
                name_end <= cursor <= token_end <= atoms@.len(),
                _reserved_steps <= cursor - name_end,
                _reserved_steps <= token_end - name_end,
                expected_parts == token_reserved_directive_parts_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    cursor as int,
                    token_end as int,
                    completed_token_part_views_spec(parts@),
                    (token_end - name_end + 1 - _reserved_steps) as nat,
                ),
            decreases token_end - cursor,
        {
            let parameter_start = skip_inline_separation(atoms, cursor, token_end);
            if parameter_start == token_end {
                proof {
                    reveal(token_reserved_directive_parts_spec);
                }
                cursor = token_end;
                _reserved_steps += 1;
            } else {
                let parameter_end = directive_parameter_end(atoms, parameter_start, token_end);
                let part = make_completed_part(
                    atoms,
                    CompletedTokenPartKind::DirectiveParameter,
                    parameter_start,
                    parameter_end,
                );
                proof {
                    lemma_completed_token_part_views_push(parts@, part);
                    reveal(token_reserved_directive_parts_spec);
                }
                parts.push(part);
                cursor = parameter_end;
                _reserved_steps += 1;
            }
        }
        proof {
            reveal(token_reserved_directive_parts_spec);
            assert(completed_token_part_views_spec(parts@) == expected_parts);
        }
        let token = make_completed_token(
            atoms,
            CompletedTokenKind::ReservedDirective,
            start,
            token_end,
            None,
            None,
            None,
            parts,
        );
        proof {
            reveal(token_parse_directive_spec);
            assert(expected == Ok(token@));
        }
        Ok(token)
    }
}

#[verifier::spinoff_prover]
#[verifier::rlimit(80)]
fn parse_anchor_or_alias_token(atoms: &[LexicalAtom], start: usize, alias: bool) -> (result: Result<
    CompletedToken,
    CompletedTokenError,
>)
    requires
        start < atoms@.len(),
    ensures
        token_parse_anchor_or_alias_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            alias,
        ) == match result {
            Ok(token) => Ok(token@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(token) => token@.start_atom_index == start && start < token@.end_atom_index
                <= atoms@.len(),
            Err(_) => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_parse_anchor_or_alias_spec(atom_views, start as int, alias);
    proof {
        reveal(token_parse_anchor_or_alias_spec);
    }
    assert(start + 1 <= atoms.len());
    let end = property_name_end(atoms, start + 1);
    if end == start + 1 {
        let error = CompletedTokenError::at(
            if alias {
                CompletedTokenErrorKind::EmptyAliasName
            } else {
                CompletedTokenErrorKind::EmptyAnchorName
            },
            offset_at_or_end(atoms, end),
        );
        proof {
            reveal(token_parse_anchor_or_alias_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    if let Some(invalid) = first_bom(atoms, start + 1, end) {
        assert(atom_views[invalid as int] == atoms[invalid as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let error = CompletedTokenError::at(
            if alias {
                CompletedTokenErrorKind::InvalidAliasCharacter
            } else {
                CompletedTokenErrorKind::InvalidAnchorCharacter
            },
            atoms[invalid].span().start().byte_offset(),
        );
        proof {
            reveal(token_parse_anchor_or_alias_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let mut parts = Vec::new();
    let part = make_completed_part(
        atoms,
        if alias {
            CompletedTokenPartKind::AliasName
        } else {
            CompletedTokenPartKind::AnchorName
        },
        start + 1,
        end,
    );
    proof {
        lemma_completed_token_part_views_push(parts@, part);
    }
    parts.push(part);
    proof {
        assert(completed_token_part_views_spec(parts@) =~= seq![part@]);
    }
    let token = make_completed_token(
        atoms,
        if alias {
            CompletedTokenKind::Alias
        } else {
            CompletedTokenKind::AnchorProperty
        },
        start,
        end,
        None,
        None,
        None,
        parts,
    );
    proof {
        reveal(completed_token_part_views_spec);
        reveal(token_parse_anchor_or_alias_spec);
        assert(expected == Ok(token@));
    }
    Ok(token)
}

#[verifier::spinoff_prover]
#[verifier::rlimit(160)]
fn parse_tag_token(atoms: &[LexicalAtom], start: usize) -> (result: Result<
    CompletedToken,
    CompletedTokenError,
>)
    requires
        start < atoms@.len(),
        atoms@[start as int]@.kind == LexicalAtomKind::Indicator(YamlIndicator::Tag),
    ensures
        token_parse_tag_spec(crate::atom::lexical_atom_views_spec(atoms@), start as int)
            == match result {
            Ok(token) => Ok(token@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(token) => token@.start_atom_index == start && start < token@.end_atom_index
                <= atoms@.len(),
            Err(_) => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_parse_tag_spec(atom_views, start as int);
    proof {
        reveal(token_parse_tag_spec);
    }
    assert(start + 1 <= atoms.len());
    if start + 1 < atoms.len() {
        assert(atom_views[(start + 1) as int] == atoms[(start + 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
    }
    if start + 1 < atoms.len() && atoms[start + 1].code_point() == 0x3c {
        let end = verbatim_tag_end(atoms, start + 2);
        if end >= atoms.len() || atoms[end].code_point() != 0x3e {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::UnterminatedVerbatimTag,
                offset_at_or_end(atoms, end),
            );
            proof {
                reveal(token_parse_tag_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        assert(atom_views[end as int] == atoms[end as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if end == start + 2 {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::EmptyVerbatimTag,
                atoms[end].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_tag_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if let Some(invalid) = first_invalid_tag_alphabet(atoms, start + 2, end, false) {
            assert(atom_views[invalid as int] == atoms[invalid as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidVerbatimTag,
                atoms[invalid].span().start().byte_offset(),
            );
            proof {
                reveal(token_parse_tag_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if let Some(invalid) = first_invalid_percent_escape(atoms, start + 2, end) {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InvalidTagPercentEscape,
                offset_at_or_end(atoms, invalid),
            );
            proof {
                reveal(token_parse_tag_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let mut parts = Vec::new();
        let part = make_completed_part(
            atoms,
            CompletedTokenPartKind::VerbatimTagPayload,
            start + 2,
            end,
        );
        proof {
            lemma_completed_token_part_views_push(parts@, part);
        }
        parts.push(part);
        proof {
            assert(completed_token_part_views_spec(parts@) =~= seq![part@]);
        }
        let token = make_completed_token(
            atoms,
            CompletedTokenKind::VerbatimTagProperty,
            start,
            end + 1,
            None,
            None,
            None,
            parts,
        );
        proof {
            reveal(completed_token_part_views_spec);
            reveal(token_parse_tag_spec);
            assert(expected == Ok(token@));
        }
        return Ok(token);
    }
    let end = property_name_end(atoms, start + 1);
    if end == start + 1 {
        let token = make_completed_token(
            atoms,
            CompletedTokenKind::TagProperty,
            start,
            start + 1,
            None,
            None,
            None,
            Vec::new(),
        );
        proof {
            reveal(completed_token_part_views_spec);
            assert(token@.parts =~= Seq::<CompletedTokenPartView>::empty());
            reveal(token_parse_tag_spec);
            assert(expected == Ok(token@));
        }
        return Ok(token);
    }
    let mut handle_end = start + 1;
    let mut suffix_start = start + 1;
    if atoms[start + 1].code_point() == 0x21 {
        handle_end = start + 2;
        suffix_start = handle_end;
    } else {
        let bang = first_code_point(atoms, start + 1, end, 0x21);
        if bang < end {
            if let Some(handle_index) = first_invalid_word(atoms, start + 1, bang) {
                assert(atom_views[handle_index as int] == atoms[handle_index as int]@) by {
                    reveal(crate::atom::lexical_atom_views_spec);
                }
                let error = CompletedTokenError::at(
                    CompletedTokenErrorKind::InvalidTagCharacter,
                    atoms[handle_index].span().start().byte_offset(),
                );
                proof {
                    reveal(token_parse_tag_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            handle_end = bang + 1;
            suffix_start = handle_end;
        }
    }
    if suffix_start == end {
        let error = CompletedTokenError::at(
            CompletedTokenErrorKind::EmptyTagSuffix,
            offset_at_or_end(atoms, end),
        );
        proof {
            reveal(token_parse_tag_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    if let Some(invalid) = first_invalid_tag_alphabet(atoms, suffix_start, end, true) {
        assert(atom_views[invalid as int] == atoms[invalid as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let error = CompletedTokenError::at(
            CompletedTokenErrorKind::InvalidTagCharacter,
            atoms[invalid].span().start().byte_offset(),
        );
        proof {
            reveal(token_parse_tag_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    if let Some(invalid) = first_invalid_percent_escape(atoms, suffix_start, end) {
        let error = CompletedTokenError::at(
            CompletedTokenErrorKind::InvalidTagPercentEscape,
            offset_at_or_end(atoms, invalid),
        );
        proof {
            reveal(token_parse_tag_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let mut parts = Vec::new();
    let handle_part = make_completed_part(
        atoms,
        CompletedTokenPartKind::TagHandle,
        start,
        handle_end,
    );
    proof {
        lemma_completed_token_part_views_push(parts@, handle_part);
    }
    parts.push(handle_part);
    let suffix_part = make_completed_part(
        atoms,
        CompletedTokenPartKind::TagSuffix,
        suffix_start,
        end,
    );
    proof {
        lemma_completed_token_part_views_push(parts@, suffix_part);
    }
    parts.push(suffix_part);
    proof {
        assert(completed_token_part_views_spec(parts@) =~= seq![handle_part@, suffix_part@]);
    }
    let token = make_completed_token(
        atoms,
        CompletedTokenKind::TagProperty,
        start,
        end,
        None,
        None,
        None,
        parts,
    );
    proof {
        reveal(completed_token_part_views_spec);
        reveal(token_parse_tag_spec);
        assert(expected == Ok(token@));
    }
    Ok(token)
}

fn marker_at(atoms: &[LexicalAtom], start: usize, code_point: u32) -> (present: bool)
    requires
        start < atoms@.len(),
    ensures
        present ==> start + 3 <= atoms@.len(),
        present == token_marker_at_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            code_point,
        ),
{
    proof {
        reveal(token_marker_at_spec);
    }
    atoms.len() - start >= 3 && atoms[start].code_point() == code_point && atoms[start
        + 1].code_point() == code_point && atoms[start + 2].code_point() == code_point && (start + 3
        == atoms.len() || is_space_or_tab(atoms[start + 3].kind()) || atoms[start + 3].kind()
        == LexicalAtomKind::LineFeed || atoms[start + 3].kind() == LexicalAtomKind::Indicator(
        YamlIndicator::Comment,
    ))
}

fn scalar_token(
    atoms: &[LexicalAtom],
    kind: CompletedTokenKind,
    start: usize,
    end: usize,
    scalar_index: usize,
) -> (token: CompletedToken)
    requires
        start < end <= atoms@.len(),
    ensures
        token@ == token_for_range_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            kind,
            start as int,
            end as int,
            Some(scalar_index as u64),
            None,
            None,
            Seq::empty(),
        ),
{
    make_completed_token(atoms, kind, start, end, Some(scalar_index as u64), None, None, Vec::new())
}

fn make_completed_step(
    token: CompletedToken,
    next_atom_index: usize,
    next_quote_index: usize,
    next_plain_index: usize,
    next_block_index: usize,
    next_at_line_prefix: bool,
    next_directive_mode: bool,
) -> (step: CompletedTokenStep)
    ensures
        step@ == token_step_for_token_spec(
            token@,
            next_atom_index as int,
            next_quote_index as int,
            next_plain_index as int,
            next_block_index as int,
            next_at_line_prefix,
            next_directive_mode,
        ),
{
    proof {
        reveal(token_step_for_token_spec);
    }
    CompletedTokenStep {
        token,
        next_atom_index,
        next_quote_index,
        next_plain_index,
        next_block_index,
        next_at_line_prefix,
        next_directive_mode,
    }
}

fn first_atom_kind(
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
    kind: LexicalAtomKind,
) -> (cursor: usize)
    requires
        start <= end <= atoms@.len(),
    ensures
        start <= cursor <= end,
        cursor == token_first_kind_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            kind,
            (end - start) as nat,
        ),
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = token_first_kind_spec(
        views,
        start as int,
        end as int,
        kind,
        (end - start) as nat,
    );
    let mut cursor = start;
    while cursor < end && atoms[cursor].kind() != kind
        invariant
            start <= cursor <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == token_first_kind_spec(
                views,
                cursor as int,
                end as int,
                kind,
                (end - cursor) as nat,
            ),
        decreases end - cursor,
    {
        assert(views[cursor as int] == atoms[cursor as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        proof {
            reveal(token_first_kind_spec);
        }
        cursor += 1;
    }
    proof {
        if cursor < end {
            assert(views[cursor as int] == atoms[cursor as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
        }
        reveal(token_first_kind_spec);
    }
    cursor
}

fn single_indicator_kind(kind: LexicalAtomKind) -> (result: Result<
    CompletedTokenKind,
    CompletedTokenErrorKind,
>)
    ensures
        result == token_single_indicator_kind_spec(kind),
{
    proof {
        reveal(token_single_indicator_kind_spec);
    }
    match kind {
        LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart) => {
            Ok(CompletedTokenKind::FlowSequenceStart)
        },
        LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart) => {
            Ok(CompletedTokenKind::FlowMappingStart)
        },
        LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd) => {
            Ok(CompletedTokenKind::FlowSequenceEnd)
        },
        LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd) => {
            Ok(CompletedTokenKind::FlowMappingEnd)
        },
        LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) => { Ok(CompletedTokenKind::FlowEntry)
        },
        LexicalAtomKind::Indicator(YamlIndicator::BlockSequenceEntry) => {
            Ok(CompletedTokenKind::BlockSequenceEntry)
        },
        LexicalAtomKind::Indicator(YamlIndicator::ExplicitMappingKey) => {
            Ok(CompletedTokenKind::ExplicitMappingKey)
        },
        LexicalAtomKind::Indicator(YamlIndicator::MappingValue) => {
            Ok(CompletedTokenKind::MappingValue)
        },
        LexicalAtomKind::Indicator(YamlIndicator::ReservedAt)
        | LexicalAtomKind::Indicator(YamlIndicator::ReservedGraveAccent) => {
            Err(CompletedTokenErrorKind::ReservedIndicator)
        },
        LexicalAtomKind::Indicator(_) => Err(CompletedTokenErrorKind::UnexpectedIndicator),
        _ => Err(CompletedTokenErrorKind::UnexpectedContent),
    }
}

#[verifier::spinoff_prover]
#[verifier::rlimit(500)]
#[allow(clippy::too_many_arguments)]
fn next_completed_token(
    atoms: &[LexicalAtom],
    quotes: &[crate::quoted::QuotedScalar],
    plains: &[crate::plain::PlainScalar],
    blocks: &[crate::block::BlockScalar],
    index: usize,
    quote_index: usize,
    plain_index: usize,
    block_index: usize,
    at_line_prefix: bool,
    directive_mode: bool,
) -> (result: Result<CompletedTokenStep, CompletedTokenError>)
    requires
        index < atoms@.len(),
        quote_index <= quotes@.len(),
        plain_index <= plains@.len(),
        block_index <= blocks@.len(),
    ensures
        token_next_candidate_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::quoted::quoted_scalar_views_spec(quotes@),
            crate::plain::plain_scalar_views_spec(plains@),
            crate::block::block_scalar_views_spec(blocks@),
            index as int,
            quote_index as int,
            plain_index as int,
            block_index as int,
            at_line_prefix,
            directive_mode,
        ) == match result {
            Ok(step) => Ok(step@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(step) => index < step@.next_atom_index <= atoms@.len()
                && completed_token_exact_formation_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::quoted::quoted_scalar_views_spec(quotes@),
                crate::plain::plain_scalar_views_spec(plains@),
                crate::block::block_scalar_views_spec(blocks@),
                step@.token,
            ),
            Err(_) => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost quote_views = crate::quoted::quoted_scalar_views_spec(quotes@);
    let ghost plain_views = crate::plain::plain_scalar_views_spec(plains@);
    let ghost block_views = crate::block::block_scalar_views_spec(blocks@);
    let ghost expected = token_next_candidate_spec(
        atom_views,
        quote_views,
        plain_views,
        block_views,
        index as int,
        quote_index as int,
        plain_index as int,
        block_index as int,
        at_line_prefix,
        directive_mode,
    );
    assert(atom_views[index as int] == atoms[index as int]@) by {
        reveal(crate::atom::lexical_atom_views_spec);
    }
    let next_atom_index: usize;
    let mut next_quote_index = quote_index;
    let mut next_plain_index = plain_index;
    let mut next_block_index = block_index;
    let mut next_at_line_prefix = at_line_prefix;
    let mut next_directive_mode = directive_mode;
    let token: CompletedToken;
    if at_line_prefix && directive_mode && atoms[index].code_point() == 0xfeff {
        token =
        make_completed_token(
            atoms,
            CompletedTokenKind::DocumentByteOrderMark,
            index,
            index + 1,
            None,
            None,
            None,
            Vec::new(),
        );
        next_atom_index = index + 1;
    } else if block_index < blocks.len() && blocks[block_index].start_atom_index() == index as u64 {
        assert(block_views[block_index as int] == blocks[block_index as int]@) by {
            reveal(crate::block::block_scalar_views_spec);
        }
        let scalar = &blocks[block_index];
        let end_u64 = scalar.end_atom_index();
        if end_u64 <= index as u64 || end_u64 > atoms.len() as u64 {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InputScalarOverlap,
                atoms[index].span().start().byte_offset(),
            );
            proof {
                reveal(token_next_candidate_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let end = end_u64 as usize;
        assert(index < end <= atoms.len());
        token =
        scalar_token(
            atoms,
            if scalar.style() == BlockScalarStyle::Literal {
                CompletedTokenKind::LiteralBlockScalar
            } else {
                CompletedTokenKind::FoldedBlockScalar
            },
            index,
            end,
            block_index,
        );
        next_atom_index = end;
        next_block_index += 1;
        assert(atom_views[(end - 1) as int] == atoms[(end - 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        next_at_line_prefix = atoms[end - 1].kind() == LexicalAtomKind::LineFeed;
        next_directive_mode = false;
    } else if quote_index < quotes.len() && quotes[quote_index].start_atom_index() == index as u64 {
        assert(quote_views[quote_index as int] == quotes[quote_index as int]@) by {
            reveal(crate::quoted::quoted_scalar_views_spec);
        }
        let scalar = &quotes[quote_index];
        let end_u64 = scalar.end_atom_index();
        if end_u64 <= index as u64 || end_u64 > atoms.len() as u64 {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InputScalarOverlap,
                atoms[index].span().start().byte_offset(),
            );
            proof {
                reveal(token_next_candidate_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let end = end_u64 as usize;
        assert(index < end <= atoms.len());
        token =
        scalar_token(
            atoms,
            if scalar.style() == QuotedScalarStyle::Single {
                CompletedTokenKind::SingleQuotedScalar
            } else {
                CompletedTokenKind::DoubleQuotedScalar
            },
            index,
            end,
            quote_index,
        );
        next_atom_index = end;
        next_quote_index += 1;
        assert(atom_views[(end - 1) as int] == atoms[(end - 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        next_at_line_prefix = atoms[end - 1].kind() == LexicalAtomKind::LineFeed;
        next_directive_mode = false;
    } else if plain_index < plains.len() && plains[plain_index].start_atom_index() == index as u64 {
        assert(plain_views[plain_index as int] == plains[plain_index as int]@) by {
            reveal(crate::plain::plain_scalar_views_spec);
        }
        let scalar = &plains[plain_index];
        let end_u64 = scalar.end_atom_index();
        if end_u64 <= index as u64 || end_u64 > atoms.len() as u64 {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InputScalarOverlap,
                atoms[index].span().start().byte_offset(),
            );
            proof {
                reveal(token_next_candidate_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let end = end_u64 as usize;
        assert(index < end <= atoms.len());
        token = scalar_token(atoms, CompletedTokenKind::PlainScalar, index, end, plain_index);
        next_atom_index = end;
        next_plain_index += 1;
        assert(atom_views[(end - 1) as int] == atoms[(end - 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        next_at_line_prefix = atoms[end - 1].kind() == LexicalAtomKind::LineFeed;
        next_directive_mode = false;
    } else {
        let atom_kind = atoms[index].kind();
        if atom_kind == LexicalAtomKind::LineFeed {
            token =
            make_completed_token(
                atoms,
                CompletedTokenKind::LineFeed,
                index,
                index + 1,
                None,
                None,
                None,
                Vec::new(),
            );
            next_atom_index = index + 1;
            next_at_line_prefix = true;
        } else if is_space_or_tab(atom_kind) {
            let end = run_of_space_or_tab(atoms, index);
            let tab = first_atom_kind(atoms, index, end, LexicalAtomKind::Tab);
            if at_line_prefix && tab < end {
                assert(atom_views[tab as int] == atoms[tab as int]@) by {
                    reveal(crate::atom::lexical_atom_views_spec);
                }
                let error = CompletedTokenError::at(
                    CompletedTokenErrorKind::TabInIndentation,
                    atoms[tab].span().start().byte_offset(),
                );
                proof {
                    reveal(token_next_candidate_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            token =
            make_completed_token(
                atoms,
                if at_line_prefix {
                    CompletedTokenKind::Indentation
                } else {
                    CompletedTokenKind::Separation
                },
                index,
                end,
                None,
                None,
                None,
                Vec::new(),
            );
            next_atom_index = end;
            next_at_line_prefix = false;
        } else if atom_kind == LexicalAtomKind::Indicator(YamlIndicator::Comment) {
            let end = line_tail_end(atoms, index);
            token =
            make_completed_token(
                atoms,
                CompletedTokenKind::Comment,
                index,
                end,
                None,
                None,
                None,
                Vec::new(),
            );
            next_atom_index = end;
            next_at_line_prefix = false;
        } else if at_line_prefix && marker_at(atoms, index, 0x2d) {
            token =
            make_completed_token(
                atoms,
                CompletedTokenKind::DirectivesEnd,
                index,
                index + 3,
                None,
                None,
                None,
                Vec::new(),
            );
            next_atom_index = index + 3;
            next_at_line_prefix = false;
            next_directive_mode = false;
        } else if at_line_prefix && marker_at(atoms, index, 0x2e) {
            token =
            make_completed_token(
                atoms,
                CompletedTokenKind::DocumentEnd,
                index,
                index + 3,
                None,
                None,
                None,
                Vec::new(),
            );
            next_atom_index = index + 3;
            next_at_line_prefix = false;
            next_directive_mode = true;
        } else if at_line_prefix && directive_mode && atom_kind == LexicalAtomKind::Indicator(
            YamlIndicator::Directive,
        ) {
            token =
            match parse_directive_token(atoms, index) {
                Ok(token) => token,
                Err(error) => {
                    proof {
                        reveal(token_next_candidate_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            next_atom_index = token.end_atom_index as usize;
            next_at_line_prefix = false;
        } else if atom_kind == LexicalAtomKind::Indicator(YamlIndicator::Anchor) {
            token =
            match parse_anchor_or_alias_token(atoms, index, false) {
                Ok(token) => token,
                Err(error) => {
                    proof {
                        reveal(token_next_candidate_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            next_atom_index = token.end_atom_index as usize;
            next_at_line_prefix = false;
            next_directive_mode = false;
        } else if atom_kind == LexicalAtomKind::Indicator(YamlIndicator::Alias) {
            token =
            match parse_anchor_or_alias_token(atoms, index, true) {
                Ok(token) => token,
                Err(error) => {
                    proof {
                        reveal(token_next_candidate_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            next_atom_index = token.end_atom_index as usize;
            next_at_line_prefix = false;
            next_directive_mode = false;
        } else if atom_kind == LexicalAtomKind::Indicator(YamlIndicator::Tag) {
            token =
            match parse_tag_token(atoms, index) {
                Ok(token) => token,
                Err(error) => {
                    proof {
                        reveal(token_next_candidate_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            next_atom_index = token.end_atom_index as usize;
            next_at_line_prefix = false;
            next_directive_mode = false;
        } else {
            let kind = match single_indicator_kind(atom_kind) {
                Ok(kind) => kind,
                Err(kind) => {
                    let error = CompletedTokenError::at(
                        kind,
                        atoms[index].span().start().byte_offset(),
                    );
                    proof {
                        reveal(token_next_candidate_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            token =
            make_completed_token(atoms, kind, index, index + 1, None, None, None, Vec::new());
            next_atom_index = index + 1;
            next_at_line_prefix = false;
            next_directive_mode = false;
        }
    }
    let step = make_completed_step(
        token,
        next_atom_index,
        next_quote_index,
        next_plain_index,
        next_block_index,
        next_at_line_prefix,
        next_directive_mode,
    );
    proof {
        reveal(completed_token_part_views_spec);
        reveal(token_for_range_spec);
        reveal(token_step_for_token_spec);
        reveal(token_next_candidate_spec);
        assert(exists|model_step: CompletedTokenStepView| expected == Ok(model_step));
        let model_step = choose|model_step: CompletedTokenStepView| expected == Ok(model_step);
        assert(model_step.token == step@.token);
        assert(model_step.next_atom_index == step@.next_atom_index);
        assert(model_step.next_quote_index == step@.next_quote_index);
        assert(model_step.next_plain_index == step@.next_plain_index);
        assert(model_step.next_block_index == step@.next_block_index);
        assert(model_step.next_at_line_prefix == step@.next_at_line_prefix);
        assert(model_step.next_directive_mode == step@.next_directive_mode);
        assert(expected == Ok(step@));
        reveal(completed_token_exact_formation_spec);
        assert(completed_token_exact_formation_spec(
            atom_views,
            quote_views,
            plain_views,
            block_views,
            step@.token,
        )) by {
            assert(token_next_candidate_spec(
                atom_views,
                quote_views,
                plain_views,
                block_views,
                index as int,
                quote_index as int,
                plain_index as int,
                block_index as int,
                at_line_prefix,
                directive_mode,
            ) == Ok(step@));
        }
    }
    Ok(step)
}

fn apply_completed_flow_kind(
    stack: &mut Vec<CompletedTokenKind>,
    kind: CompletedTokenKind,
    byte_offset: u64,
    flow_limit: u64,
) -> (result: Result<(), CompletedTokenError>)
    requires
        old(stack)@.len() <= flow_limit,
        flow_limit <= MAX_PROFILE1_FLOW_DEPTH,
    ensures
        token_apply_flow_kind_spec(old(stack)@, kind, byte_offset, flow_limit) == match result {
            Ok(()) => Ok(final(stack)@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(()) => completed_token_flow_stack_after_kind_spec(old(stack)@, kind) == Some(
                final(stack)@,
            ) && final(stack)@.len() <= flow_limit,
            Err(_) => final(stack)@ == old(stack)@,
        },
{
    proof {
        reveal(token_apply_flow_kind_spec);
    }
    if kind == CompletedTokenKind::FlowSequenceStart || kind
        == CompletedTokenKind::FlowMappingStart {
        if stack.len() as u64 >= flow_limit {
            return Err(
                CompletedTokenError::at(
                    CompletedTokenErrorKind::FlowDepthLimitExceeded,
                    byte_offset,
                ),
            );
        }
        stack.push(kind);
        proof {
            reveal(completed_token_flow_stack_after_kind_spec);
        }
        Ok(())
    } else if kind == CompletedTokenKind::FlowSequenceEnd {
        if stack.is_empty() {
            return Err(
                CompletedTokenError::at(CompletedTokenErrorKind::UnexpectedFlowEnd, byte_offset),
            );
        }
        if stack[stack.len() - 1] != CompletedTokenKind::FlowSequenceStart {
            return Err(
                CompletedTokenError::at(CompletedTokenErrorKind::MismatchedFlowEnd, byte_offset),
            );
        }
        stack.pop();
        proof {
            reveal(completed_token_flow_stack_after_kind_spec);
        }
        Ok(())
    } else if kind == CompletedTokenKind::FlowMappingEnd {
        if stack.is_empty() {
            return Err(
                CompletedTokenError::at(CompletedTokenErrorKind::UnexpectedFlowEnd, byte_offset),
            );
        }
        if stack[stack.len() - 1] != CompletedTokenKind::FlowMappingStart {
            return Err(
                CompletedTokenError::at(CompletedTokenErrorKind::MismatchedFlowEnd, byte_offset),
            );
        }
        stack.pop();
        proof {
            reveal(completed_token_flow_stack_after_kind_spec);
        }
        Ok(())
    } else if kind == CompletedTokenKind::FlowEntry && stack.is_empty() {
        Err(CompletedTokenError::at(CompletedTokenErrorKind::UnexpectedIndicator, byte_offset))
    } else {
        proof {
            reveal(completed_token_flow_stack_after_kind_spec);
        }
        Ok(())
    }
}

proof fn lemma_completed_token_views_push(tokens: Seq<CompletedToken>, token: CompletedToken)
    ensures
        completed_token_views_spec(tokens.push(token)) == completed_token_views_spec(tokens).push(
            token@,
        ),
{
    reveal(completed_token_views_spec);
    assert(completed_token_views_spec(tokens.push(token)) =~= completed_token_views_spec(
        tokens,
    ).push(token@));
}

proof fn lemma_completed_token_part_views_push(
    parts: Seq<CompletedTokenPart>,
    part: CompletedTokenPart,
)
    ensures
        completed_token_part_views_spec(parts.push(part)) == completed_token_part_views_spec(
            parts,
        ).push(part@),
{
    reveal(completed_token_part_views_spec);
    assert(completed_token_part_views_spec(parts.push(part)) =~= completed_token_part_views_spec(
        parts,
    ).push(part@));
}

fn advance_quoted_scalar_index(
    scalars: &[crate::quoted::QuotedScalar],
    start: usize,
    atom_index: usize,
) -> (index: usize)
    requires
        start <= scalars@.len(),
    ensures
        start <= index <= scalars@.len(),
        index == token_advance_quoted_index_spec(
            crate::quoted::quoted_scalar_views_spec(scalars@),
            start as int,
            atom_index as int,
            (scalars@.len() - start + 1) as nat,
        ),
{
    let ghost views = crate::quoted::quoted_scalar_views_spec(scalars@);
    let ghost expected = token_advance_quoted_index_spec(
        views,
        start as int,
        atom_index as int,
        (scalars@.len() - start + 1) as nat,
    );
    let mut index = start;
    while index < scalars.len() && scalars[index].end_atom_index() <= atom_index as u64
        invariant
            start <= index <= scalars@.len(),
            views == crate::quoted::quoted_scalar_views_spec(scalars@),
            expected == token_advance_quoted_index_spec(
                views,
                index as int,
                atom_index as int,
                (scalars@.len() - index + 1) as nat,
            ),
        decreases scalars.len() - index,
    {
        assert(views[index as int] == scalars[index as int]@) by {
            reveal(crate::quoted::quoted_scalar_views_spec);
        }
        proof {
            reveal(token_advance_quoted_index_spec);
        }
        index += 1;
    }
    proof {
        if index < scalars.len() {
            assert(views[index as int] == scalars[index as int]@) by {
                reveal(crate::quoted::quoted_scalar_views_spec);
            }
        }
        reveal(token_advance_quoted_index_spec);
    }
    index
}

fn advance_plain_scalar_index(
    scalars: &[crate::plain::PlainScalar],
    start: usize,
    atom_index: usize,
) -> (index: usize)
    requires
        start <= scalars@.len(),
    ensures
        start <= index <= scalars@.len(),
        index == token_advance_plain_index_spec(
            crate::plain::plain_scalar_views_spec(scalars@),
            start as int,
            atom_index as int,
            (scalars@.len() - start + 1) as nat,
        ),
{
    let ghost views = crate::plain::plain_scalar_views_spec(scalars@);
    let ghost expected = token_advance_plain_index_spec(
        views,
        start as int,
        atom_index as int,
        (scalars@.len() - start + 1) as nat,
    );
    let mut index = start;
    while index < scalars.len() && scalars[index].end_atom_index() <= atom_index as u64
        invariant
            start <= index <= scalars@.len(),
            views == crate::plain::plain_scalar_views_spec(scalars@),
            expected == token_advance_plain_index_spec(
                views,
                index as int,
                atom_index as int,
                (scalars@.len() - index + 1) as nat,
            ),
        decreases scalars.len() - index,
    {
        assert(views[index as int] == scalars[index as int]@) by {
            reveal(crate::plain::plain_scalar_views_spec);
        }
        proof {
            reveal(token_advance_plain_index_spec);
        }
        index += 1;
    }
    proof {
        if index < scalars.len() {
            assert(views[index as int] == scalars[index as int]@) by {
                reveal(crate::plain::plain_scalar_views_spec);
            }
        }
        reveal(token_advance_plain_index_spec);
    }
    index
}

fn advance_block_scalar_index(
    scalars: &[crate::block::BlockScalar],
    start: usize,
    atom_index: usize,
) -> (index: usize)
    requires
        start <= scalars@.len(),
    ensures
        start <= index <= scalars@.len(),
        index == token_advance_block_index_spec(
            crate::block::block_scalar_views_spec(scalars@),
            start as int,
            atom_index as int,
            (scalars@.len() - start + 1) as nat,
        ),
{
    let ghost views = crate::block::block_scalar_views_spec(scalars@);
    let ghost expected = token_advance_block_index_spec(
        views,
        start as int,
        atom_index as int,
        (scalars@.len() - start + 1) as nat,
    );
    let mut index = start;
    while index < scalars.len() && scalars[index].end_atom_index() <= atom_index as u64
        invariant
            start <= index <= scalars@.len(),
            views == crate::block::block_scalar_views_spec(scalars@),
            expected == token_advance_block_index_spec(
                views,
                index as int,
                atom_index as int,
                (scalars@.len() - index + 1) as nat,
            ),
        decreases scalars.len() - index,
    {
        assert(views[index as int] == scalars[index as int]@) by {
            reveal(crate::block::block_scalar_views_spec);
        }
        proof {
            reveal(token_advance_block_index_spec);
        }
        index += 1;
    }
    proof {
        if index < scalars.len() {
            assert(views[index as int] == scalars[index as int]@) by {
                reveal(crate::block::block_scalar_views_spec);
            }
        }
        reveal(token_advance_block_index_spec);
    }
    index
}

#[verifier::spinoff_prover]
#[verifier::rlimit(700)]
pub fn scan_profile1_completed_tokens(
    atomized: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    limits: CompletedTokenLimits,
) -> (result: Result<CompletedTokenSource, CompletedTokenError>)
    ensures
        scan_profile1_completed_tokens_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
            limits@,
        ) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
        completed_token_empty_canonical_inputs_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
        ) ==> result.is_ok(),
        match result {
            Ok(source) => completed_token_source_corresponds_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
                source@,
            ) && (crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                ==> completed_token_partition_spec(atomized@, source@)
                && completed_token_flow_balanced_spec(source@.tokens)) && (
            crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
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
            ) && crate::block::block_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
            ) ==> completed_token_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
                source@,
            )),
            Err(_) => true,
        },
{
    let ghost full_expected = scan_profile1_completed_tokens_spec(
        atomized@,
        layout@,
        structural@,
        quoted@,
        plain@,
        block@,
        limits@,
    );
    let ghost empty_canonical_inputs = completed_token_empty_canonical_inputs_spec(
        atomized@,
        layout@,
        structural@,
        quoted@,
        plain@,
        block@,
    );
    proof {
        if empty_canonical_inputs {
            reveal(completed_token_empty_canonical_inputs_spec);
        }
    }
    let canonical_layout = match analyze_profile1_layout(
        atomized,
        canonical_structural_layout_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = CompletedTokenError::at(
                CompletedTokenErrorKind::InputLayoutMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_completed_tokens_spec);
                assert(full_expected == Err(mismatch@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_layout.same_as(layout) {
        let mismatch = CompletedTokenError::at(
            CompletedTokenErrorKind::InputLayoutMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_completed_tokens_spec);
            assert(full_expected == Err(mismatch@));
        }
        return Err(mismatch);
    }
    assert(canonical_layout@ == layout@);
    let canonical_structural = match scan_profile1_structural_lexemes(
        atomized,
        layout,
        canonical_structural_scan_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = CompletedTokenError::at(
                CompletedTokenErrorKind::InputStructuralMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_completed_tokens_spec);
                assert(full_expected == Err(mismatch@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_structural.same_as(structural) {
        let mismatch = CompletedTokenError::at(
            CompletedTokenErrorKind::InputStructuralMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_completed_tokens_spec);
            assert(full_expected == Err(mismatch@));
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
            let mismatch = CompletedTokenError::at(
                CompletedTokenErrorKind::InputQuotedMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_completed_tokens_spec);
                assert(full_expected == Err(mismatch@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_quoted.same_as(quoted) {
        let mismatch = CompletedTokenError::at(
            CompletedTokenErrorKind::InputQuotedMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_completed_tokens_spec);
            assert(full_expected == Err(mismatch@));
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
            let mismatch = CompletedTokenError::at(
                CompletedTokenErrorKind::InputPlainMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_completed_tokens_spec);
                assert(full_expected == Err(mismatch@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_plain.same_as(plain) {
        let mismatch = CompletedTokenError::at(
            CompletedTokenErrorKind::InputPlainMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_completed_tokens_spec);
            assert(full_expected == Err(mismatch@));
        }
        return Err(mismatch);
    }
    assert(canonical_plain@ == plain@);
    let canonical_block = match scan_profile1_block_scalars(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        canonical_block_scalar_limits(),
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = CompletedTokenError::at(
                CompletedTokenErrorKind::InputBlockMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_completed_tokens_spec);
                assert(full_expected == Err(mismatch@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_block.same_as(block) {
        let mismatch = CompletedTokenError::at(
            CompletedTokenErrorKind::InputBlockMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_completed_tokens_spec);
            assert(full_expected == Err(mismatch@));
        }
        return Err(mismatch);
    }
    assert(canonical_block@ == block@);

    let token_limit = if limits.max_tokens < MAX_PROFILE1_COMPLETED_TOKENS {
        limits.max_tokens
    } else {
        MAX_PROFILE1_COMPLETED_TOKENS
    };
    let flow_limit = if limits.max_flow_depth < MAX_PROFILE1_FLOW_DEPTH {
        limits.max_flow_depth
    } else {
        MAX_PROFILE1_FLOW_DEPTH
    };
    let atoms = atomized.atoms();
    let quotes = quoted.scalars();
    let plains = plain.scalars();
    let blocks = block.scalars();
    let mut tokens: Vec<CompletedToken> = Vec::new();
    let mut flow_stack: Vec<CompletedTokenKind> = Vec::new();
    let mut maximum_flow_depth = 0u64;
    let mut quote_index = 0usize;
    let mut plain_index = 0usize;
    let mut block_index = 0usize;
    let mut index = 0usize;
    let mut at_line_prefix = true;
    let mut directive_mode = true;
    let ghost tail_expected = completed_token_scan_tail_spec(
        atomized@.atoms,
        quoted@.scalars,
        plain@.scalars,
        block@.scalars,
        atomized@.source_len_bytes,
        0,
        0,
        0,
        0,
        true,
        true,
        Seq::empty(),
        0,
        Seq::empty(),
        token_limit,
        flow_limit,
        (atomized@.atoms.len() + 1) as nat,
    );
    let ghost tail_relation = match tail_expected {
        Err(error) => Err(error),
        Ok(success) => Ok(
            completed_token_source_from_tail_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
                success,
            ),
        ),
    };

    proof {
        reveal(scan_profile1_completed_tokens_spec);
        reveal(effective_token_limit_spec);
        reveal(effective_flow_depth_spec);
        assert(full_expected == tail_relation);
        reveal(completed_token_views_spec);
        assert(completed_token_views_spec(tokens@) =~= Seq::<CompletedTokenView>::empty());
        assert(tail_expected == completed_token_scan_tail_spec(
            atomized@.atoms,
            quoted@.scalars,
            plain@.scalars,
            block@.scalars,
            atomized@.source_len_bytes,
            index as int,
            quote_index as int,
            plain_index as int,
            block_index as int,
            at_line_prefix,
            directive_mode,
            flow_stack@,
            maximum_flow_depth,
            completed_token_views_spec(tokens@),
            token_limit,
            flow_limit,
            (atoms@.len() + 1 - tokens@.len()) as nat,
        ));
        lemma_empty_completed_token_prefix(atomized@.atoms);
        lemma_empty_completed_flow_prefix();
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) {
            crate::atom::lemma_intrinsic_atomized_spans_partition_source(atomized@);
        }
    }

    while index < atoms.len()
        invariant
            crate::atom::lexical_atom_views_spec(atoms@) == atomized@.atoms,
            crate::quoted::quoted_scalar_views_spec(quotes@) == quoted@.scalars,
            crate::plain::plain_scalar_views_spec(plains@) == plain@.scalars,
            crate::block::block_scalar_views_spec(blocks@) == block@.scalars,
            atoms@.len() == atomized@.atoms.len(),
            index <= atoms@.len(),
            quote_index <= quotes@.len(),
            plain_index <= plains@.len(),
            block_index <= blocks@.len(),
            tokens@.len() <= index,
            tokens@.len() <= token_limit,
            flow_stack@.len() <= flow_limit,
            maximum_flow_depth <= flow_limit,
            maximum_flow_depth <= MAX_PROFILE1_FLOW_DEPTH,
            token_limit <= MAX_PROFILE1_COMPLETED_TOKENS,
            flow_limit <= MAX_PROFILE1_FLOW_DEPTH,
            tail_expected == completed_token_scan_tail_spec(
                atomized@.atoms,
                quoted@.scalars,
                plain@.scalars,
                block@.scalars,
                atomized@.source_len_bytes,
                index as int,
                quote_index as int,
                plain_index as int,
                block_index as int,
                at_line_prefix,
                directive_mode,
                flow_stack@,
                maximum_flow_depth,
                completed_token_views_spec(tokens@),
                token_limit,
                flow_limit,
                (atoms@.len() + 1 - tokens@.len()) as nat,
            ),
            full_expected == tail_relation,
            full_expected == scan_profile1_completed_tokens_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
                limits@,
            ),
            tail_relation == match tail_expected {
                Err(error) => Err(error),
                Ok(success) => Ok(
                    completed_token_source_from_tail_spec(
                        atomized@,
                        layout@,
                        structural@,
                        quoted@,
                        plain@,
                        block@,
                        success,
                    ),
                ),
            },
            completed_token_flow_prefix_spec(completed_token_views_spec(tokens@), flow_stack@),
            crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                ==> completed_token_prefix_partition_spec(
                atomized@.atoms,
                completed_token_views_spec(tokens@),
                index as int,
            ),
            crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) ==> (
            atomized@.atoms.len() == 0 ==> atomized@.source_len_bytes == atomized@.bom_bytes) && (
            atomized@.atoms.len() > 0 ==> atomized@.atoms[0].span.start.byte_offset
                == atomized@.bom_bytes && atomized@.atoms[atomized@.atoms.len()
                - 1].span.end.byte_offset == atomized@.source_len_bytes && forall|atom_index: int|
                0 < atom_index < atomized@.atoms.len() ==> atomized@.atoms[atom_index - 1].span.end
                    == atomized@.atoms[atom_index].span.start),
        decreases atoms.len() - index,
    {
        let old_index = index;
        quote_index = advance_quoted_scalar_index(quotes, quote_index, index);
        plain_index = advance_plain_scalar_index(plains, plain_index, index);
        block_index = advance_block_scalar_index(blocks, block_index, index);
        proof {
            reveal(completed_token_scan_tail_spec);
        }
        let step = match next_completed_token(
            atoms,
            quotes,
            plains,
            blocks,
            index,
            quote_index,
            plain_index,
            block_index,
            at_line_prefix,
            directive_mode,
        ) {
            Ok(step) => step,
            Err(error) => {
                proof {
                    assert(tail_expected == Err(error@));
                    assert(full_expected == Err(error@));
                    assert(scan_profile1_completed_tokens_spec(
                        atomized@,
                        layout@,
                        structural@,
                        quoted@,
                        plain@,
                        block@,
                        limits@,
                    ) == Err(error@));
                }
                return Err(error);
            },
        };

        index = step.next_atom_index;
        quote_index = step.next_quote_index;
        plain_index = step.next_plain_index;
        block_index = step.next_block_index;
        at_line_prefix = step.next_at_line_prefix;
        directive_mode = step.next_directive_mode;
        let token = step.token;

        if token.start_atom_index != old_index as u64 || token.end_atom_index != index as u64
            || !completed_token_range_valid(atoms, &token) {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::InputScalarOverlap,
                token.byte_start,
            );
            proof {
                assert(tail_expected == Err(error@));
                assert(full_expected == Err(error@));
            }
            return Err(error);
        }
        let ghost previous_flow_stack = flow_stack@;
        match apply_completed_flow_kind(&mut flow_stack, token.kind, token.byte_start, flow_limit) {
            Ok(()) => {},
            Err(error) => {
                proof {
                    assert(tail_expected == Err(error@));
                    assert(full_expected == Err(error@));
                    assert(scan_profile1_completed_tokens_spec(
                        atomized@,
                        layout@,
                        structural@,
                        quoted@,
                        plain@,
                        block@,
                        limits@,
                    ) == Err(error@));
                }
                return Err(error);
            },
        }
        if flow_stack.len() as u64 > maximum_flow_depth {
            maximum_flow_depth = flow_stack.len() as u64;
        }
        if tokens.len() as u64 >= token_limit {
            let error = CompletedTokenError::at(
                CompletedTokenErrorKind::TokenLimitExceeded,
                token.byte_start,
            );
            proof {
                assert(tail_expected == Err(error@));
                assert(full_expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            assert(token@.start_atom_index == old_index);
            assert(token@.end_atom_index == index);
            reveal(completed_token_range_spec);
            assert(completed_token_range_spec(atomized@.atoms, token@));
            if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) {
                lemma_extend_completed_token_prefix(
                    atomized@.atoms,
                    completed_token_views_spec(tokens@),
                    token@,
                    old_index as int,
                );
            }
            lemma_extend_completed_flow_prefix(
                completed_token_views_spec(tokens@),
                previous_flow_stack,
                token@,
                flow_stack@,
            );
            lemma_completed_token_views_push(tokens@, token);
            reveal(completed_token_scan_tail_spec);
        }
        tokens.push(token);
    }

    if !flow_stack.is_empty() {
        let error = CompletedTokenError::at(
            CompletedTokenErrorKind::UnclosedFlowCollection,
            atomized.source_len_bytes(),
        );
        proof {
            reveal(completed_token_scan_tail_spec);
            assert(tail_expected == Err(error@));
            assert(full_expected == Err(error@));
        }
        return Err(error);
    }
    let source = CompletedTokenSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version(),
        layout_transformation_version: layout.transformation_version(),
        structural_transformation_version: structural.transformation_version(),
        quoted_transformation_version: quoted.transformation_version(),
        plain_transformation_version: plain.transformation_version(),
        block_transformation_version: block.transformation_version(),
        transformation_version: COMPLETED_TOKEN_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes(),
        bom_bytes: atomized.bom_bytes(),
        input_atom_count: atoms.len() as u64,
        maximum_flow_depth,
        tokens,
    };
    proof {
        reveal(completed_token_scan_tail_spec);
        let success = CompletedTokenTailSuccessView { maximum_flow_depth, tokens: source@.tokens };
        assert(tail_expected == Ok(success));
        reveal(completed_token_source_from_tail_spec);
        assert(source@ == completed_token_source_from_tail_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
            success,
        ));
        assert(full_expected == Ok(source@));
        reveal(completed_token_source_corresponds_spec);
        assert(exists|candidate_limits: CompletedTokenLimitsView|
            scan_profile1_completed_tokens_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
                candidate_limits,
            ) == Ok(source@)) by {
            assert(scan_profile1_completed_tokens_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                plain@,
                block@,
                limits@,
            ) == Ok(source@));
        }
        if atomized@.atoms.len() == 0 {
            reveal(scan_profile1_completed_tokens_spec);
            reveal(completed_token_views_spec);
        }
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) {
            reveal(completed_token_partition_spec);
            reveal(completed_token_prefix_partition_spec);
            if atomized@.atoms.len() == 0 {
                assert(source@.tokens.len() == 0);
            } else {
                assert(source@.tokens.len() > 0);
                assert(source@.tokens[0].start_atom_index == 0);
                assert(completed_token_range_spec(atomized@.atoms, source@.tokens[0]));
                reveal(completed_token_range_spec);
                assert(source@.tokens[0].byte_start == atomized@.bom_bytes);
                assert(source@.tokens[source@.tokens.len() - 1].end_atom_index
                    == atomized@.atoms.len());
                assert(completed_token_range_spec(
                    atomized@.atoms,
                    source@.tokens[source@.tokens.len() - 1],
                ));
                assert(source@.tokens[source@.tokens.len() - 1].byte_end
                    == atomized@.source_len_bytes);
            }
            assert(completed_token_partition_spec(atomized@, source@));
        }
        assert(flow_stack@ =~= Seq::<CompletedTokenKind>::empty());
        reveal(completed_token_flow_balanced_spec);
        assert(completed_token_flow_prefix_spec(source@.tokens, Seq::empty()));
        reveal(completed_token_absolute_limits_spec);
        assert(source@.tokens.len() <= token_limit);
        assert(completed_token_absolute_limits_spec(source@));
        reveal(completed_token_public_semantics_spec);
        assert(completed_token_public_semantics_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
            source@,
        ));
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
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
        ) && crate::block::block_scalar_source_well_formed_spec(
            atomized@,
            layout@,
            structural@,
            quoted@,
            plain@,
            block@,
        ) {
            reveal(completed_token_source_well_formed_spec);
        }
    }
    Ok(source)
}

} // verus!
