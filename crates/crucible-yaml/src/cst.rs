// The executable parser deliberately spells these operations out so each branch
// remains visible to the mirrored Verus specification and its proof state.
#![allow(clippy::implicit_saturating_add)]
#![allow(clippy::implicit_saturating_sub)]
#![allow(clippy::len_zero)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::question_mark)]
#![allow(clippy::too_many_arguments)]

use vstd::prelude::*;

use crate::atom::{AtomizedSource, LexicalAtom};
use crate::block::BlockScalarSource;
use crate::layout::LayoutSource;
use crate::plain::PlainScalarSource;
use crate::quoted::QuotedScalarSource;
use crate::structural::StructuralLexemeSource;
use crate::token::{
    canonical_completed_token_limits, scan_profile1_completed_tokens, CompletedToken,
    CompletedTokenKind, CompletedTokenPart, CompletedTokenPartKind, CompletedTokenSource,
    MAX_PROFILE1_COMPLETED_TOKENS,
};
use crate::CRUCIBLE_YAML_PROFILE_VERSION;

verus! {

pub const CST_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_CST_DOCUMENTS: u64 = 1_048_576;

pub const MAX_PROFILE1_CST_NODES: u64 = 1_048_576;

pub const MAX_PROFILE1_CST_SEQUENCE_ENTRIES: u64 = 1_048_576;

pub const MAX_PROFILE1_CST_MAPPING_ENTRIES: u64 = 1_048_576;

pub const MAX_PROFILE1_CST_DIRECTIVES: u64 = 1_048_576;

pub const MAX_PROFILE1_CST_WARNINGS: u64 = 1_048_576;

pub const MAX_PROFILE1_CST_DEPTH: u64 = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstLimits {
    max_documents: u64,
    max_nodes: u64,
    max_sequence_entries: u64,
    max_mapping_entries: u64,
    max_directives: u64,
    max_warnings: u64,
    max_depth: u64,
}

#[verifier::ext_equal]
pub struct CstLimitsView {
    pub max_documents: u64,
    pub max_nodes: u64,
    pub max_sequence_entries: u64,
    pub max_mapping_entries: u64,
    pub max_directives: u64,
    pub max_warnings: u64,
    pub max_depth: u64,
}

impl View for CstLimits {
    type V = CstLimitsView;

    closed spec fn view(&self) -> CstLimitsView {
        CstLimitsView {
            max_documents: self.max_documents,
            max_nodes: self.max_nodes,
            max_sequence_entries: self.max_sequence_entries,
            max_mapping_entries: self.max_mapping_entries,
            max_directives: self.max_directives,
            max_warnings: self.max_warnings,
            max_depth: self.max_depth,
        }
    }
}

impl CstLimits {
    pub const fn new(
        max_documents: u64,
        max_nodes: u64,
        max_sequence_entries: u64,
        max_mapping_entries: u64,
        max_directives: u64,
        max_warnings: u64,
        max_depth: u64,
    ) -> Self {
        Self {
            max_documents,
            max_nodes,
            max_sequence_entries,
            max_mapping_entries,
            max_directives,
            max_warnings,
            max_depth,
        }
    }

    pub const fn max_documents(&self) -> u64 {
        self.max_documents
    }

    pub const fn max_nodes(&self) -> u64 {
        self.max_nodes
    }

    pub const fn max_sequence_entries(&self) -> u64 {
        self.max_sequence_entries
    }

    pub const fn max_mapping_entries(&self) -> u64 {
        self.max_mapping_entries
    }

    pub const fn max_directives(&self) -> u64 {
        self.max_directives
    }

    pub const fn max_warnings(&self) -> u64 {
        self.max_warnings
    }

    pub const fn max_depth(&self) -> u64 {
        self.max_depth
    }
}

pub const fn canonical_cst_limits() -> CstLimits {
    CstLimits::new(
        MAX_PROFILE1_CST_DOCUMENTS,
        MAX_PROFILE1_CST_NODES,
        MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
        MAX_PROFILE1_CST_MAPPING_ENTRIES,
        MAX_PROFILE1_CST_DIRECTIVES,
        MAX_PROFILE1_CST_WARNINGS,
        MAX_PROFILE1_CST_DEPTH,
    )
}

pub open spec fn cst_effective_limit_spec(requested: u64, maximum: u64) -> u64 {
    if requested < maximum {
        requested
    } else {
        maximum
    }
}

fn effective_limit(requested: u64, maximum: u64) -> (effective: u64)
    ensures
        effective == cst_effective_limit_spec(requested, maximum),
        effective <= maximum,
{
    if requested < maximum {
        requested
    } else {
        maximum
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CstNodeKind {
    Empty,
    Scalar,
    Alias,
    Sequence,
    Mapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CstNodeStyle {
    Empty,
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
    Alias,
    Block,
    Flow,
    FlowPair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CstWarningKind {
    Yaml11Compatibility,
    FutureMinorVersion,
    ReservedDirective,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CstSyntaxOwnerKind {
    Directive,
    DocumentStartMarker,
    DocumentEndMarker,
    NodeProperty,
    NodeContent,
    NodeCollectionIndicator,
    SequenceEntryIndicator,
    MappingEntryIndicator,
    FlowEntryIndicator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstSyntaxOwner {
    token_index: u64,
    kind: CstSyntaxOwnerKind,
    record_index: u64,
}

#[verifier::ext_equal]
pub struct CstSyntaxOwnerView {
    pub token_index: u64,
    pub kind: CstSyntaxOwnerKind,
    pub record_index: u64,
}

impl View for CstSyntaxOwner {
    type V = CstSyntaxOwnerView;

    closed spec fn view(&self) -> CstSyntaxOwnerView {
        CstSyntaxOwnerView {
            token_index: self.token_index,
            kind: self.kind,
            record_index: self.record_index,
        }
    }
}

impl CstSyntaxOwner {
    pub fn token_index(&self) -> u64 {
        self.token_index
    }

    pub fn kind(&self) -> CstSyntaxOwnerKind {
        self.kind
    }

    pub fn record_index(&self) -> u64 {
        self.record_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CstErrorKind {
    InputCompletedTokenMismatch,
    DocumentLimitExceeded,
    NodeLimitExceeded,
    SequenceEntryLimitExceeded,
    MappingEntryLimitExceeded,
    DirectiveLimitExceeded,
    WarningLimitExceeded,
    DepthLimitExceeded,
    MissingDirectivesEnd,
    DuplicateYamlDirective,
    DuplicateTagHandle,
    UndeclaredTagHandle,
    UnsupportedYamlMajorVersion,
    DuplicateAnchorProperty,
    DuplicateTagProperty,
    MissingPropertySeparation,
    AliasHasPropertiesOrContent,
    MultilineImplicitKey,
    UnexpectedFlowEntry,
    MissingFlowEntry,
    MissingMappingValue,
    UnexpectedMappingValue,
    UnexpectedCollectionEnd,
    InvalidIndentation,
    UnexpectedDocumentMarker,
    UnexpectedToken,
    UnexpectedEndOfInput,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstError {
    kind: CstErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CstErrorView {
    pub kind: CstErrorKind,
    pub byte_offset: u64,
}

impl View for CstError {
    type V = CstErrorView;

    closed spec fn view(&self) -> CstErrorView {
        CstErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CstError {
    fn at(kind: CstErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (CstErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> CstErrorKind {
        self.kind
    }

    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstWarning {
    kind: CstWarningKind,
    document_index: u64,
    token_index: u64,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CstWarningView {
    pub kind: CstWarningKind,
    pub document_index: u64,
    pub token_index: u64,
    pub byte_offset: u64,
}

impl View for CstWarning {
    type V = CstWarningView;

    closed spec fn view(&self) -> CstWarningView {
        CstWarningView {
            kind: self.kind,
            document_index: self.document_index,
            token_index: self.token_index,
            byte_offset: self.byte_offset,
        }
    }
}

impl CstWarning {
    pub fn kind(&self) -> CstWarningKind {
        self.kind
    }

    pub fn document_index(&self) -> u64 {
        self.document_index
    }

    pub fn token_index(&self) -> u64 {
        self.token_index
    }

    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstSequenceEntry {
    node_index: u64,
    token_start: u64,
    token_end: u64,
    indicator_token: Option<u64>,
}

#[verifier::ext_equal]
pub struct CstSequenceEntryView {
    pub node_index: u64,
    pub token_start: u64,
    pub token_end: u64,
    pub indicator_token: Option<u64>,
}

impl View for CstSequenceEntry {
    type V = CstSequenceEntryView;

    closed spec fn view(&self) -> CstSequenceEntryView {
        CstSequenceEntryView {
            node_index: self.node_index,
            token_start: self.token_start,
            token_end: self.token_end,
            indicator_token: self.indicator_token,
        }
    }
}

impl CstSequenceEntry {
    pub fn node_index(&self) -> u64 {
        self.node_index
    }

    pub fn token_start(&self) -> (result: u64)
        ensures
            result == self@.token_start,
    {
        self.token_start
    }

    pub fn token_end(&self) -> (result: u64)
        ensures
            result == self@.token_end,
    {
        self.token_end
    }

    pub fn indicator_token(&self) -> Option<u64> {
        self.indicator_token
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstMappingEntry {
    key_node_index: u64,
    value_node_index: u64,
    token_start: u64,
    token_end: u64,
    explicit_key_token: Option<u64>,
    mapping_value_token: Option<u64>,
}

#[verifier::ext_equal]
pub struct CstMappingEntryView {
    pub key_node_index: u64,
    pub value_node_index: u64,
    pub token_start: u64,
    pub token_end: u64,
    pub explicit_key_token: Option<u64>,
    pub mapping_value_token: Option<u64>,
}

impl View for CstMappingEntry {
    type V = CstMappingEntryView;

    closed spec fn view(&self) -> CstMappingEntryView {
        CstMappingEntryView {
            key_node_index: self.key_node_index,
            value_node_index: self.value_node_index,
            token_start: self.token_start,
            token_end: self.token_end,
            explicit_key_token: self.explicit_key_token,
            mapping_value_token: self.mapping_value_token,
        }
    }
}

impl CstMappingEntry {
    pub fn key_node_index(&self) -> u64 {
        self.key_node_index
    }

    pub fn value_node_index(&self) -> u64 {
        self.value_node_index
    }

    pub fn token_start(&self) -> u64 {
        self.token_start
    }

    pub fn token_end(&self) -> u64 {
        self.token_end
    }

    pub fn explicit_key_token(&self) -> Option<u64> {
        self.explicit_key_token
    }

    pub fn mapping_value_token(&self) -> Option<u64> {
        self.mapping_value_token
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstNode {
    kind: CstNodeKind,
    style: CstNodeStyle,
    token_start: u64,
    token_end: u64,
    byte_start: u64,
    byte_end: u64,
    anchor_property_token: Option<u64>,
    tag_property_token: Option<u64>,
    scalar_or_alias_token: Option<u64>,
    collection_start_token: Option<u64>,
    collection_end_token: Option<u64>,
    entry_start: u64,
    entry_end: u64,
    empty_anchor_token: Option<u64>,
    empty_anchor_byte: Option<u64>,
}

#[verifier::ext_equal]
pub struct CstNodeView {
    pub kind: CstNodeKind,
    pub style: CstNodeStyle,
    pub token_start: u64,
    pub token_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub anchor_property_token: Option<u64>,
    pub tag_property_token: Option<u64>,
    pub scalar_or_alias_token: Option<u64>,
    pub collection_start_token: Option<u64>,
    pub collection_end_token: Option<u64>,
    pub entry_start: u64,
    pub entry_end: u64,
    pub empty_anchor_token: Option<u64>,
    pub empty_anchor_byte: Option<u64>,
}

impl View for CstNode {
    type V = CstNodeView;

    closed spec fn view(&self) -> CstNodeView {
        CstNodeView {
            kind: self.kind,
            style: self.style,
            token_start: self.token_start,
            token_end: self.token_end,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            anchor_property_token: self.anchor_property_token,
            tag_property_token: self.tag_property_token,
            scalar_or_alias_token: self.scalar_or_alias_token,
            collection_start_token: self.collection_start_token,
            collection_end_token: self.collection_end_token,
            entry_start: self.entry_start,
            entry_end: self.entry_end,
            empty_anchor_token: self.empty_anchor_token,
            empty_anchor_byte: self.empty_anchor_byte,
        }
    }
}

impl CstNode {
    pub fn kind(&self) -> CstNodeKind {
        self.kind
    }

    pub fn style(&self) -> CstNodeStyle {
        self.style
    }

    pub fn token_start(&self) -> (result: u64)
        ensures
            result == self@.token_start,
    {
        self.token_start
    }

    pub fn token_end(&self) -> (result: u64)
        ensures
            result == self@.token_end,
    {
        self.token_end
    }

    pub fn byte_start(&self) -> u64 {
        self.byte_start
    }

    pub fn byte_end(&self) -> u64 {
        self.byte_end
    }

    pub fn anchor_property_token(&self) -> Option<u64> {
        self.anchor_property_token
    }

    pub fn tag_property_token(&self) -> Option<u64> {
        self.tag_property_token
    }

    pub fn scalar_or_alias_token(&self) -> Option<u64> {
        self.scalar_or_alias_token
    }

    pub fn collection_start_token(&self) -> Option<u64> {
        self.collection_start_token
    }

    pub fn collection_end_token(&self) -> Option<u64> {
        self.collection_end_token
    }

    pub fn entry_start(&self) -> u64 {
        self.entry_start
    }

    pub fn entry_end(&self) -> u64 {
        self.entry_end
    }

    pub fn empty_anchor_token(&self) -> Option<u64> {
        self.empty_anchor_token
    }

    pub fn empty_anchor_byte(&self) -> Option<u64> {
        self.empty_anchor_byte
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstDocument {
    token_start: u64,
    token_end: u64,
    byte_start: u64,
    byte_end: u64,
    prefix_token_start: u64,
    prefix_token_end: u64,
    directive_start: u64,
    directive_end: u64,
    explicit_start_token_start: u64,
    explicit_start_token_end: u64,
    root_token_start: u64,
    root_token_end: u64,
    explicit_end_token_start: u64,
    explicit_end_token_end: u64,
    suffix_token_start: u64,
    suffix_token_end: u64,
    root_node_index: u64,
    explicit_start_token: Option<u64>,
    explicit_end_token: Option<u64>,
}

#[verifier::ext_equal]
pub struct CstDocumentView {
    pub token_start: u64,
    pub token_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub prefix_token_start: u64,
    pub prefix_token_end: u64,
    pub directive_start: u64,
    pub directive_end: u64,
    pub explicit_start_token_start: u64,
    pub explicit_start_token_end: u64,
    pub root_token_start: u64,
    pub root_token_end: u64,
    pub explicit_end_token_start: u64,
    pub explicit_end_token_end: u64,
    pub suffix_token_start: u64,
    pub suffix_token_end: u64,
    pub root_node_index: u64,
    pub explicit_start_token: Option<u64>,
    pub explicit_end_token: Option<u64>,
}

impl View for CstDocument {
    type V = CstDocumentView;

    closed spec fn view(&self) -> CstDocumentView {
        CstDocumentView {
            token_start: self.token_start,
            token_end: self.token_end,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            prefix_token_start: self.prefix_token_start,
            prefix_token_end: self.prefix_token_end,
            directive_start: self.directive_start,
            directive_end: self.directive_end,
            explicit_start_token_start: self.explicit_start_token_start,
            explicit_start_token_end: self.explicit_start_token_end,
            root_token_start: self.root_token_start,
            root_token_end: self.root_token_end,
            explicit_end_token_start: self.explicit_end_token_start,
            explicit_end_token_end: self.explicit_end_token_end,
            suffix_token_start: self.suffix_token_start,
            suffix_token_end: self.suffix_token_end,
            root_node_index: self.root_node_index,
            explicit_start_token: self.explicit_start_token,
            explicit_end_token: self.explicit_end_token,
        }
    }
}

impl CstDocument {
    pub fn token_start(&self) -> u64 {
        self.token_start
    }

    pub fn token_end(&self) -> u64 {
        self.token_end
    }

    pub fn byte_start(&self) -> u64 {
        self.byte_start
    }

    pub fn byte_end(&self) -> u64 {
        self.byte_end
    }

    pub fn prefix_token_start(&self) -> u64 {
        self.prefix_token_start
    }

    pub fn prefix_token_end(&self) -> u64 {
        self.prefix_token_end
    }

    pub fn directive_start(&self) -> u64 {
        self.directive_start
    }

    pub fn directive_end(&self) -> u64 {
        self.directive_end
    }

    pub fn explicit_start_token_start(&self) -> u64 {
        self.explicit_start_token_start
    }

    pub fn explicit_start_token_end(&self) -> u64 {
        self.explicit_start_token_end
    }

    pub fn root_token_start(&self) -> u64 {
        self.root_token_start
    }

    pub fn root_token_end(&self) -> u64 {
        self.root_token_end
    }

    pub fn explicit_end_token_start(&self) -> u64 {
        self.explicit_end_token_start
    }

    pub fn explicit_end_token_end(&self) -> u64 {
        self.explicit_end_token_end
    }

    pub fn suffix_token_start(&self) -> u64 {
        self.suffix_token_start
    }

    pub fn suffix_token_end(&self) -> u64 {
        self.suffix_token_end
    }

    pub fn root_node_index(&self) -> u64 {
        self.root_node_index
    }

    pub fn explicit_start_token(&self) -> Option<u64> {
        self.explicit_start_token
    }

    pub fn explicit_end_token(&self) -> Option<u64> {
        self.explicit_end_token
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CstSource {
    profile_version: u16,
    input_token_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_token_count: u64,
    directive_count: u64,
    maximum_depth: u64,
    documents: Vec<CstDocument>,
    nodes: Vec<CstNode>,
    sequence_entries: Vec<CstSequenceEntry>,
    mapping_entries: Vec<CstMappingEntry>,
    warnings: Vec<CstWarning>,
    syntax_owners: Vec<Option<CstSyntaxOwner>>,
}

#[verifier::ext_equal]
pub struct CstSourceView {
    pub profile_version: u16,
    pub input_token_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_token_count: u64,
    pub directive_count: u64,
    pub maximum_depth: u64,
    pub documents: Seq<CstDocumentView>,
    pub nodes: Seq<CstNodeView>,
    pub sequence_entries: Seq<CstSequenceEntryView>,
    pub mapping_entries: Seq<CstMappingEntryView>,
    pub warnings: Seq<CstWarningView>,
    pub syntax_owners: Seq<Option<CstSyntaxOwnerView>>,
}

pub open spec fn cst_document_views_spec(values: Seq<CstDocument>) -> Seq<CstDocumentView> {
    values.map_values(|value: CstDocument| value@)
}

pub open spec fn cst_node_views_spec(values: Seq<CstNode>) -> Seq<CstNodeView> {
    values.map_values(|value: CstNode| value@)
}

pub open spec fn cst_sequence_entry_views_spec(values: Seq<CstSequenceEntry>) -> Seq<
    CstSequenceEntryView,
> {
    values.map_values(|value: CstSequenceEntry| value@)
}

pub open spec fn cst_mapping_entry_views_spec(values: Seq<CstMappingEntry>) -> Seq<
    CstMappingEntryView,
> {
    values.map_values(|value: CstMappingEntry| value@)
}

pub open spec fn cst_warning_views_spec(values: Seq<CstWarning>) -> Seq<CstWarningView> {
    values.map_values(|value: CstWarning| value@)
}

pub open spec fn cst_syntax_owner_views_spec(values: Seq<Option<CstSyntaxOwner>>) -> Seq<
    Option<CstSyntaxOwnerView>,
> {
    values.map_values(
        |value: Option<CstSyntaxOwner>|
            match value {
                Some(owner) => Some(owner@),
                None => None,
            },
    )
}

proof fn lemma_cst_node_view_at(values: Seq<CstNode>, index: int)
    requires
        0 <= index < values.len(),
    ensures
        cst_node_views_spec(values)[index] == values[index]@,
{
    reveal(cst_node_views_spec);
}

proof fn lemma_cst_sequence_entry_view_at(values: Seq<CstSequenceEntry>, index: int)
    requires
        0 <= index < values.len(),
    ensures
        cst_sequence_entry_views_spec(values)[index] == values[index]@,
{
    reveal(cst_sequence_entry_views_spec);
}

proof fn lemma_cst_mapping_entry_view_at(values: Seq<CstMappingEntry>, index: int)
    requires
        0 <= index < values.len(),
    ensures
        cst_mapping_entry_views_spec(values)[index] == values[index]@,
{
    reveal(cst_mapping_entry_views_spec);
}

proof fn lemma_cst_syntax_owner_view_fields(value: CstSyntaxOwner)
    ensures
        value@.token_index == value.token_index,
        value@.kind == value.kind,
        value@.record_index == value.record_index,
{
}

proof fn lemma_cst_sequence_entry_view_fields(value: CstSequenceEntry)
    ensures
        value@.node_index == value.node_index,
        value@.token_start == value.token_start,
        value@.token_end == value.token_end,
        value@.indicator_token == value.indicator_token,
{
}

proof fn lemma_cst_mapping_entry_view_fields(value: CstMappingEntry)
    ensures
        value@.key_node_index == value.key_node_index,
        value@.value_node_index == value.value_node_index,
        value@.token_start == value.token_start,
        value@.token_end == value.token_end,
        value@.explicit_key_token == value.explicit_key_token,
        value@.mapping_value_token == value.mapping_value_token,
{
}

proof fn lemma_cst_node_view_fields(value: CstNode)
    ensures
        value@.kind == value.kind,
        value@.style == value.style,
        value@.token_start == value.token_start,
        value@.token_end == value.token_end,
        value@.anchor_property_token == value.anchor_property_token,
        value@.tag_property_token == value.tag_property_token,
        value@.scalar_or_alias_token == value.scalar_or_alias_token,
        value@.collection_start_token == value.collection_start_token,
        value@.collection_end_token == value.collection_end_token,
{
}

proof fn lemma_cst_document_view_fields(value: CstDocument)
    ensures
        value@.token_start == value.token_start,
        value@.token_end == value.token_end,
        value@.root_node_index == value.root_node_index,
        value@.explicit_start_token == value.explicit_start_token,
        value@.explicit_end_token == value.explicit_end_token,
{
}

pub proof fn lemma_cst_document_view_at(values: Seq<CstDocument>, index: int)
    requires
        0 <= index < values.len(),
    ensures
        cst_document_views_spec(values)[index] == values[index]@,
{
    reveal(cst_document_views_spec);
}

pub proof fn lemma_cst_warning_view_at(values: Seq<CstWarning>, index: int)
    requires
        0 <= index < values.len(),
    ensures
        cst_warning_views_spec(values)[index] == values[index]@,
{
    reveal(cst_warning_views_spec);
}

proof fn lemma_cst_syntax_owner_view_at(values: Seq<Option<CstSyntaxOwner>>, index: int)
    requires
        0 <= index < values.len(),
    ensures
        cst_syntax_owner_views_spec(values)[index] == match values[index] {
            Some(owner) => Some(owner@),
            None => None,
        },
{
    reveal(cst_syntax_owner_views_spec);
}

proof fn lemma_cst_syntax_owner_views_update(
    values: Seq<Option<CstSyntaxOwner>>,
    index: int,
    owner: CstSyntaxOwner,
)
    requires
        0 <= index < values.len(),
    ensures
        cst_syntax_owner_views_spec(values.update(index, Some(owner)))
            == cst_syntax_owner_views_spec(values).update(index, Some(owner@)),
{
    reveal(cst_syntax_owner_views_spec);
    assert(values.update(index, Some(owner)).map_values(
        |value: Option<CstSyntaxOwner>|
            match value {
                Some(value) => Some(value@),
                None => None,
            },
    ) =~= values.map_values(
        |value: Option<CstSyntaxOwner>|
            match value {
                Some(value) => Some(value@),
                None => None,
            },
    ).update(index, Some(owner@)));
}

impl View for CstSource {
    type V = CstSourceView;

    closed spec fn view(&self) -> CstSourceView {
        CstSourceView {
            profile_version: self.profile_version,
            input_token_transformation_version: self.input_token_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_token_count: self.input_token_count,
            directive_count: self.directive_count,
            maximum_depth: self.maximum_depth,
            documents: cst_document_views_spec(self.documents@),
            nodes: cst_node_views_spec(self.nodes@),
            sequence_entries: cst_sequence_entry_views_spec(self.sequence_entries@),
            mapping_entries: cst_mapping_entry_views_spec(self.mapping_entries@),
            warnings: cst_warning_views_spec(self.warnings@),
            syntax_owners: cst_syntax_owner_views_spec(self.syntax_owners@),
        }
    }
}

impl CstSource {
    pub fn profile_version(&self) -> u16 {
        self.profile_version
    }

    pub fn transformation_version(&self) -> u16 {
        self.transformation_version
    }

    pub fn source_len_bytes(&self) -> u64 {
        self.source_len_bytes
    }

    pub fn input_token_transformation_version(&self) -> u16 {
        self.input_token_transformation_version
    }

    pub fn input_token_count(&self) -> u64 {
        self.input_token_count
    }

    pub fn directive_count(&self) -> u64 {
        self.directive_count
    }

    pub fn maximum_depth(&self) -> u64 {
        self.maximum_depth
    }

    pub fn documents(&self) -> &[CstDocument] {
        self.documents.as_slice()
    }

    pub fn nodes(&self) -> &[CstNode] {
        self.nodes.as_slice()
    }

    pub fn sequence_entries(&self) -> &[CstSequenceEntry] {
        self.sequence_entries.as_slice()
    }

    pub fn mapping_entries(&self) -> &[CstMappingEntry] {
        self.mapping_entries.as_slice()
    }

    pub fn warnings(&self) -> &[CstWarning] {
        self.warnings.as_slice()
    }

    pub fn syntax_owners(&self) -> &[Option<CstSyntaxOwner>] {
        self.syntax_owners.as_slice()
    }
}

pub open spec fn cst_child_before_parent_spec(
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
) -> bool {
    (forall|node_index: int|
        0 <= node_index < nodes.len() && nodes[node_index].kind == CstNodeKind::Sequence
            ==> nodes[node_index].entry_start <= nodes[node_index].entry_end
            <= sequence_entries.len() && forall|entry_index: int|
            nodes[node_index].entry_start <= entry_index < nodes[node_index].entry_end
                ==> sequence_entries[entry_index].node_index < node_index) && (forall|
        node_index: int,
    |
        0 <= node_index < nodes.len() && nodes[node_index].kind == CstNodeKind::Mapping
            ==> nodes[node_index].entry_start <= nodes[node_index].entry_end
            <= mapping_entries.len() && forall|entry_index: int|
            nodes[node_index].entry_start <= entry_index < nodes[node_index].entry_end
                ==> mapping_entries[entry_index].key_node_index < node_index
                && mapping_entries[entry_index].value_node_index < node_index)
}

proof fn lemma_sequence_child_violation_breaks_cst(
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
    node_index: int,
    entry_index: int,
)
    requires
        0 <= node_index < nodes.len(),
        nodes[node_index].kind == CstNodeKind::Sequence,
        nodes[node_index].entry_start <= entry_index < nodes[node_index].entry_end,
        nodes[node_index].entry_end <= sequence_entries.len(),
        sequence_entries[entry_index].node_index >= node_index,
    ensures
        !cst_child_before_parent_spec(nodes, sequence_entries, mapping_entries),
{
    assert(!(forall|index: int|
        nodes[node_index].entry_start <= index < nodes[node_index].entry_end
            ==> sequence_entries[index].node_index < node_index)) by {
        assert(!(sequence_entries[entry_index].node_index < node_index));
    }
    assert(!(forall|index: int|
        0 <= index < nodes.len() && nodes[index].kind == CstNodeKind::Sequence
            ==> nodes[index].entry_start <= nodes[index].entry_end <= sequence_entries.len()
            && forall|child: int|
            nodes[index].entry_start <= child < nodes[index].entry_end
                ==> sequence_entries[child].node_index < index)) by {}
    reveal(cst_child_before_parent_spec);
}

proof fn lemma_mapping_child_violation_breaks_cst(
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
    node_index: int,
    entry_index: int,
)
    requires
        0 <= node_index < nodes.len(),
        nodes[node_index].kind == CstNodeKind::Mapping,
        nodes[node_index].entry_start <= entry_index < nodes[node_index].entry_end,
        nodes[node_index].entry_end <= mapping_entries.len(),
        mapping_entries[entry_index].key_node_index >= node_index
            || mapping_entries[entry_index].value_node_index >= node_index,
    ensures
        !cst_child_before_parent_spec(nodes, sequence_entries, mapping_entries),
{
    assert(!(mapping_entries[entry_index].key_node_index < node_index
        && mapping_entries[entry_index].value_node_index < node_index));
    assert(!(forall|index: int|
        nodes[node_index].entry_start <= index < nodes[node_index].entry_end
            ==> mapping_entries[index].key_node_index < node_index
            && mapping_entries[index].value_node_index < node_index)) by {}
    assert(!(forall|index: int|
        0 <= index < nodes.len() && nodes[index].kind == CstNodeKind::Mapping
            ==> nodes[index].entry_start <= nodes[index].entry_end <= mapping_entries.len()
            && forall|child: int|
            nodes[index].entry_start <= child < nodes[index].entry_end
                ==> mapping_entries[child].key_node_index < index
                && mapping_entries[child].value_node_index < index)) by {}
    reveal(cst_child_before_parent_spec);
}

fn cst_child_before_parent(
    nodes: &[CstNode],
    sequence_entries: &[CstSequenceEntry],
    mapping_entries: &[CstMappingEntry],
) -> (result: bool)
    ensures
        result == cst_child_before_parent_spec(
            cst_node_views_spec(nodes@),
            cst_sequence_entry_views_spec(sequence_entries@),
            cst_mapping_entry_views_spec(mapping_entries@),
        ),
{
    let ghost node_views = cst_node_views_spec(nodes@);
    let ghost sequence_views = cst_sequence_entry_views_spec(sequence_entries@);
    let ghost mapping_views = cst_mapping_entry_views_spec(mapping_entries@);
    proof {
        reveal(cst_node_views_spec);
        reveal(cst_sequence_entry_views_spec);
        reveal(cst_mapping_entry_views_spec);
        assert(node_views.len() == nodes.len());
        assert(sequence_views.len() == sequence_entries.len());
        assert(mapping_views.len() == mapping_entries.len());
    }
    let mut node_index = 0usize;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            node_views == cst_node_views_spec(nodes@),
            sequence_views == cst_sequence_entry_views_spec(sequence_entries@),
            mapping_views == cst_mapping_entry_views_spec(mapping_entries@),
            node_views.len() == nodes.len(),
            sequence_views.len() == sequence_entries.len(),
            mapping_views.len() == mapping_entries.len(),
            forall|prior: int|
                #![auto]
                0 <= prior < node_index && node_views[prior].kind == CstNodeKind::Sequence
                    ==> node_views[prior].entry_start <= node_views[prior].entry_end
                    <= sequence_views.len() && forall|entry_index: int|
                    node_views[prior].entry_start <= entry_index < node_views[prior].entry_end
                        ==> sequence_views[entry_index].node_index < prior,
            forall|prior: int|
                #![auto]
                0 <= prior < node_index && node_views[prior].kind == CstNodeKind::Mapping
                    ==> node_views[prior].entry_start <= node_views[prior].entry_end
                    <= mapping_views.len() && forall|entry_index: int|
                    node_views[prior].entry_start <= entry_index < node_views[prior].entry_end
                        ==> mapping_views[entry_index].key_node_index < prior
                        && mapping_views[entry_index].value_node_index < prior,
        decreases nodes.len() - node_index,
    {
        assert(node_index < node_views.len());
        assert(node_views[node_index as int] == nodes[node_index as int]@) by {
            lemma_cst_node_view_at(nodes@, node_index as int);
        }
        let node = &nodes[node_index];
        assert(*node == nodes@[node_index as int]);
        assert(node_views[node_index as int] == node@);
        if node.kind == CstNodeKind::Sequence {
            if node.entry_start > node.entry_end || node.entry_end > sequence_entries.len() as u64 {
                proof {
                    reveal(cst_child_before_parent_spec);
                    assert(!cst_child_before_parent_spec(
                        node_views,
                        sequence_views,
                        mapping_views,
                    ));
                }
                return false;
            }
            let mut entry_index = node.entry_start as usize;
            while entry_index < node.entry_end as usize
                invariant
                    node_views == cst_node_views_spec(nodes@),
                    sequence_views == cst_sequence_entry_views_spec(sequence_entries@),
                    mapping_views == cst_mapping_entry_views_spec(mapping_entries@),
                    sequence_views.len() == sequence_entries.len(),
                    node_index < node_views.len(),
                    node_views[node_index as int] == node@,
                    node_views[node_index as int].kind == CstNodeKind::Sequence,
                    node.entry_start <= entry_index <= node.entry_end,
                    node.entry_end <= sequence_entries.len(),
                    forall|prior: int|
                        #![auto]
                        node.entry_start <= prior < entry_index ==> sequence_views[prior].node_index
                            < node_index,
                decreases node.entry_end - entry_index,
            {
                assert(entry_index < sequence_entries.len());
                assert(entry_index < sequence_views.len());
                let entry = &sequence_entries[entry_index];
                assert(*entry == sequence_entries@[entry_index as int]);
                proof {
                    lemma_cst_sequence_entry_view_at(sequence_entries@, entry_index as int);
                    assert(cst_sequence_entry_views_spec(sequence_entries@)[entry_index as int]
                        == sequence_entries@[entry_index as int]@);
                }
                assert(sequence_views[entry_index as int]
                    == sequence_entries@[entry_index as int]@);
                if entry.node_index >= node_index as u64 {
                    proof {
                        assert(node_views[node_index as int] == node@);
                        assert(node_views[node_index as int].entry_start <= entry_index
                            < node_views[node_index as int].entry_end);
                        assert(!(sequence_views[entry_index as int].node_index < node_index));
                        lemma_sequence_child_violation_breaks_cst(
                            node_views,
                            sequence_views,
                            mapping_views,
                            node_index as int,
                            entry_index as int,
                        );
                    }
                    return false;
                }
                entry_index += 1;
            }
        } else if node.kind == CstNodeKind::Mapping {
            if node.entry_start > node.entry_end || node.entry_end > mapping_entries.len() as u64 {
                proof {
                    reveal(cst_child_before_parent_spec);
                    assert(!cst_child_before_parent_spec(
                        node_views,
                        sequence_views,
                        mapping_views,
                    ));
                }
                return false;
            }
            let mut entry_index = node.entry_start as usize;
            while entry_index < node.entry_end as usize
                invariant
                    node_views == cst_node_views_spec(nodes@),
                    sequence_views == cst_sequence_entry_views_spec(sequence_entries@),
                    mapping_views == cst_mapping_entry_views_spec(mapping_entries@),
                    mapping_views.len() == mapping_entries.len(),
                    node_index < node_views.len(),
                    node_views[node_index as int] == node@,
                    node_views[node_index as int].kind == CstNodeKind::Mapping,
                    node.entry_start <= entry_index <= node.entry_end,
                    node.entry_end <= mapping_entries.len(),
                    forall|prior: int|
                        #![auto]
                        node.entry_start <= prior < entry_index
                            ==> mapping_views[prior].key_node_index < node_index
                            && mapping_views[prior].value_node_index < node_index,
                decreases node.entry_end - entry_index,
            {
                assert(entry_index < mapping_entries.len());
                assert(entry_index < mapping_views.len());
                let entry = &mapping_entries[entry_index];
                assert(*entry == mapping_entries@[entry_index as int]);
                proof {
                    lemma_cst_mapping_entry_view_at(mapping_entries@, entry_index as int);
                    assert(cst_mapping_entry_views_spec(mapping_entries@)[entry_index as int]
                        == mapping_entries@[entry_index as int]@);
                }
                assert(mapping_views[entry_index as int] == mapping_entries@[entry_index as int]@);
                if entry.key_node_index >= node_index as u64 || entry.value_node_index
                    >= node_index as u64 {
                    proof {
                        assert(node_views[node_index as int] == node@);
                        assert(node_views[node_index as int].entry_start <= entry_index
                            < node_views[node_index as int].entry_end);
                        assert(!(mapping_views[entry_index as int].key_node_index < node_index
                            && mapping_views[entry_index as int].value_node_index < node_index));
                        lemma_mapping_child_violation_breaks_cst(
                            node_views,
                            sequence_views,
                            mapping_views,
                            node_index as int,
                            entry_index as int,
                        );
                    }
                    return false;
                }
                entry_index += 1;
            }
        }
        node_index += 1;
    }
    proof {
        reveal(cst_child_before_parent_spec);
    }
    true
}

pub open spec fn cst_entry_table_partition_from_spec(
    nodes: Seq<CstNodeView>,
    node_index: int,
    sequence_cursor: u64,
    mapping_cursor: u64,
    sequence_entry_count: u64,
    mapping_entry_count: u64,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if node_index >= nodes.len() {
        sequence_cursor == sequence_entry_count && mapping_cursor == mapping_entry_count
    } else if node_index < 0 || fuel == 0 {
        false
    } else {
        let node = nodes[node_index];
        if node.kind == CstNodeKind::Sequence {
            node.entry_start == sequence_cursor && node.entry_start <= node.entry_end
                && node.entry_end <= sequence_entry_count && cst_entry_table_partition_from_spec(
                nodes,
                node_index + 1,
                node.entry_end,
                mapping_cursor,
                sequence_entry_count,
                mapping_entry_count,
                (fuel - 1) as nat,
            )
        } else if node.kind == CstNodeKind::Mapping {
            node.entry_start == mapping_cursor && node.entry_start <= node.entry_end
                && node.entry_end <= mapping_entry_count && cst_entry_table_partition_from_spec(
                nodes,
                node_index + 1,
                sequence_cursor,
                node.entry_end,
                sequence_entry_count,
                mapping_entry_count,
                (fuel - 1) as nat,
            )
        } else {
            node.entry_start == 0 && node.entry_end == 0 && cst_entry_table_partition_from_spec(
                nodes,
                node_index + 1,
                sequence_cursor,
                mapping_cursor,
                sequence_entry_count,
                mapping_entry_count,
                (fuel - 1) as nat,
            )
        }
    }
}

pub open spec fn cst_entry_tables_uniquely_owned_spec(
    nodes: Seq<CstNodeView>,
    sequence_entry_count: u64,
    mapping_entry_count: u64,
) -> bool {
    cst_entry_table_partition_from_spec(
        nodes,
        0,
        0,
        0,
        sequence_entry_count,
        mapping_entry_count,
        (nodes.len() + 1) as nat,
    )
}

fn cst_entry_tables_uniquely_owned(
    nodes: &[CstNode],
    sequence_entry_count: usize,
    mapping_entry_count: usize,
) -> (result: bool)
    ensures
        result == cst_entry_tables_uniquely_owned_spec(
            cst_node_views_spec(nodes@),
            sequence_entry_count as u64,
            mapping_entry_count as u64,
        ),
{
    let ghost node_views = cst_node_views_spec(nodes@);
    let ghost expected = cst_entry_tables_uniquely_owned_spec(
        node_views,
        sequence_entry_count as u64,
        mapping_entry_count as u64,
    );
    let mut node_index = 0usize;
    let mut sequence_cursor = 0u64;
    let mut mapping_cursor = 0u64;
    let ghost mut fuel: nat = (nodes@.len() + 1) as nat;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            node_views == cst_node_views_spec(nodes@),
            expected == cst_entry_tables_uniquely_owned_spec(
                cst_node_views_spec(nodes@),
                sequence_entry_count as u64,
                mapping_entry_count as u64,
            ),
            fuel >= nodes@.len() - node_index + 1,
            expected == cst_entry_table_partition_from_spec(
                node_views,
                node_index as int,
                sequence_cursor,
                mapping_cursor,
                sequence_entry_count as u64,
                mapping_entry_count as u64,
                fuel,
            ),
        decreases nodes.len() - node_index,
    {
        assert(node_views[node_index as int] == nodes[node_index as int]@) by {
            lemma_cst_node_view_at(nodes@, node_index as int);
        }
        let node = &nodes[node_index];
        if node.kind == CstNodeKind::Sequence {
            if node.entry_start != sequence_cursor || node.entry_start > node.entry_end
                || node.entry_end > sequence_entry_count as u64 {
                proof {
                    reveal(cst_entry_table_partition_from_spec);
                    assert(!expected);
                }
                return false;
            }
            sequence_cursor = node.entry_end;
        } else if node.kind == CstNodeKind::Mapping {
            if node.entry_start != mapping_cursor || node.entry_start > node.entry_end
                || node.entry_end > mapping_entry_count as u64 {
                proof {
                    reveal(cst_entry_table_partition_from_spec);
                    assert(!expected);
                }
                return false;
            }
            mapping_cursor = node.entry_end;
        } else if node.entry_start != 0 || node.entry_end != 0 {
            proof {
                reveal(cst_entry_table_partition_from_spec);
                assert(!expected);
            }
            return false;
        }
        proof {
            reveal(cst_entry_table_partition_from_spec);
            fuel = (fuel - 1) as nat;
        }
        node_index += 1;
    }
    proof {
        reveal(cst_entry_table_partition_from_spec);
        reveal(cst_entry_tables_uniquely_owned_spec);
    }
    sequence_cursor == sequence_entry_count as u64 && mapping_cursor == mapping_entry_count as u64
}

pub open spec fn cst_style_matches_token_spec(
    style: CstNodeStyle,
    kind: CompletedTokenKind,
) -> bool {
    match style {
        CstNodeStyle::Plain => kind == CompletedTokenKind::PlainScalar,
        CstNodeStyle::SingleQuoted => kind == CompletedTokenKind::SingleQuotedScalar,
        CstNodeStyle::DoubleQuoted => kind == CompletedTokenKind::DoubleQuotedScalar,
        CstNodeStyle::Literal => kind == CompletedTokenKind::LiteralBlockScalar,
        CstNodeStyle::Folded => kind == CompletedTokenKind::FoldedBlockScalar,
        CstNodeStyle::Alias => kind == CompletedTokenKind::Alias,
        _ => false,
    }
}

pub open spec fn cst_byte_at_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    index: u64,
    source_len_bytes: u64,
) -> u64 {
    if index < tokens.len() {
        tokens[index as int].byte_start
    } else {
        source_len_bytes
    }
}

pub open spec fn cst_node_token_identity_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    source_len_bytes: u64,
    node: CstNodeView,
) -> bool {
    node.token_start <= node.token_end && node.token_end <= tokens.len() && node.byte_start
        <= node.byte_end <= source_len_bytes && node.byte_start == cst_byte_at_spec(
        tokens,
        node.token_start,
        source_len_bytes,
    ) && match node.anchor_property_token {
        Some(index) => node.token_start <= index < node.token_end && index < tokens.len()
            && tokens[index as int].kind == CompletedTokenKind::AnchorProperty,
        None => true,
    } && match node.tag_property_token {
        Some(index) => node.token_start <= index < node.token_end && index < tokens.len() && (
        tokens[index as int].kind == CompletedTokenKind::TagProperty || tokens[index as int].kind
            == CompletedTokenKind::VerbatimTagProperty),
        None => true,
    } && (node.anchor_property_token.is_none() || node.anchor_property_token
        != node.tag_property_token) && match node.kind {
        CstNodeKind::Empty => node.style == CstNodeStyle::Empty
            && node.scalar_or_alias_token.is_none() && node.entry_start == 0 && node.entry_end == 0
            && node.collection_start_token.is_none() && node.collection_end_token.is_none()
            && node.empty_anchor_token == Some(node.token_end) && node.empty_anchor_byte == Some(
            cst_byte_at_spec(tokens, node.token_end, source_len_bytes),
        ) && node.byte_end == cst_byte_at_spec(tokens, node.token_end, source_len_bytes),
        CstNodeKind::Scalar => node.scalar_or_alias_token.is_some()
            && node.empty_anchor_token.is_none() && node.empty_anchor_byte.is_none()
            && node.entry_start == 0 && node.entry_end == 0 && {
            node.collection_start_token.is_none() && node.collection_end_token.is_none() && {
                let index = node.scalar_or_alias_token.unwrap();
                node.token_start <= index < node.token_end && index < tokens.len()
                    && cst_style_matches_token_spec(node.style, tokens[index as int].kind)
                    && node.style != CstNodeStyle::Alias && node.byte_end == tokens[(node.token_end
                    - 1) as int].byte_end
            }
        },
        CstNodeKind::Alias => node.style == CstNodeStyle::Alias
            && node.anchor_property_token.is_none() && node.tag_property_token.is_none()
            && node.scalar_or_alias_token.is_some() && node.empty_anchor_token.is_none()
            && node.empty_anchor_byte.is_none() && node.entry_start == 0 && node.entry_end == 0 && {
            node.collection_start_token.is_none() && node.collection_end_token.is_none() && {
                let index = node.scalar_or_alias_token.unwrap();
                node.token_start <= index < node.token_end && index < tokens.len()
                    && tokens[index as int].kind == CompletedTokenKind::Alias && node.byte_end
                    == tokens[(node.token_end - 1) as int].byte_end
            }
        },
        CstNodeKind::Sequence => (node.style == CstNodeStyle::Block || node.style
            == CstNodeStyle::Flow) && node.scalar_or_alias_token.is_none()
            && node.empty_anchor_token.is_none() && node.empty_anchor_byte.is_none()
            && node.token_start < node.token_end && node.byte_end == tokens[(node.token_end
            - 1) as int].byte_end && if node.style == CstNodeStyle::Flow {
            node.collection_start_token.is_some() && node.collection_end_token.is_some()
                && node.token_start <= node.collection_start_token.unwrap() < node.token_end
                && node.token_start <= node.collection_end_token.unwrap() < node.token_end
                && tokens[node.collection_start_token.unwrap() as int].kind
                == CompletedTokenKind::FlowSequenceStart
                && tokens[node.collection_end_token.unwrap() as int].kind
                == CompletedTokenKind::FlowSequenceEnd
        } else {
            node.collection_start_token.is_none() && node.collection_end_token.is_none()
        },
        CstNodeKind::Mapping => (node.style == CstNodeStyle::Block || node.style
            == CstNodeStyle::Flow || node.style == CstNodeStyle::FlowPair)
            && node.scalar_or_alias_token.is_none() && node.empty_anchor_token.is_none()
            && node.empty_anchor_byte.is_none() && node.token_start < node.token_end
            && node.byte_end == tokens[(node.token_end - 1) as int].byte_end && if node.style
            == CstNodeStyle::Flow {
            node.collection_start_token.is_some() && node.collection_end_token.is_some()
                && node.token_start <= node.collection_start_token.unwrap() < node.token_end
                && node.token_start <= node.collection_end_token.unwrap() < node.token_end
                && tokens[node.collection_start_token.unwrap() as int].kind
                == CompletedTokenKind::FlowMappingStart
                && tokens[node.collection_end_token.unwrap() as int].kind
                == CompletedTokenKind::FlowMappingEnd
        } else {
            node.collection_start_token.is_none() && node.collection_end_token.is_none()
        },
    }
}

pub open spec fn cst_nodes_have_exact_token_identity_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    source_len_bytes: u64,
    nodes: Seq<CstNodeView>,
) -> bool {
    forall|index: int|
        #![auto]
        0 <= index < nodes.len() ==> cst_node_token_identity_spec(
            tokens,
            source_len_bytes,
            nodes[index],
        )
}

pub open spec fn cst_entry_ranges_spec(
    token_count: u64,
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
) -> bool {
    (forall|index: int|
        #![auto]
        0 <= index < sequence_entries.len() ==> sequence_entries[index].token_start
            < sequence_entries[index].token_end <= token_count && sequence_entries[index].node_index
            < nodes.len() && sequence_entries[index].token_start
            <= nodes[sequence_entries[index].node_index as int].token_start
            && nodes[sequence_entries[index].node_index as int].token_end
            <= sequence_entries[index].token_end) && (forall|index: int|
        #![auto]
        0 <= index < mapping_entries.len() ==> mapping_entries[index].token_start
            < mapping_entries[index].token_end <= token_count
            && mapping_entries[index].key_node_index < nodes.len()
            && mapping_entries[index].value_node_index < nodes.len()
            && mapping_entries[index].token_start
            <= nodes[mapping_entries[index].key_node_index as int].token_start
            && nodes[mapping_entries[index].key_node_index as int].token_end
            <= mapping_entries[index].token_end && mapping_entries[index].token_start
            <= nodes[mapping_entries[index].value_node_index as int].token_start
            && nodes[mapping_entries[index].value_node_index as int].token_end
            <= mapping_entries[index].token_end)
}

pub open spec fn cst_owner_at_spec(
    owners: Seq<Option<CstSyntaxOwnerView>>,
    token_index: u64,
    kind: CstSyntaxOwnerKind,
    record_index: u64,
) -> bool {
    token_index < owners.len() && owners[token_index as int] == Some(
        CstSyntaxOwnerView { token_index, kind, record_index },
    )
}

pub open spec fn cst_syntax_owner_record_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
    owner: CstSyntaxOwnerView,
) -> bool {
    owner.token_index < tokens.len() && match owner.kind {
        CstSyntaxOwnerKind::Directive => owner.record_index < documents.len()
            && documents[owner.record_index as int].directive_start <= owner.token_index
            < documents[owner.record_index as int].directive_end && (
        tokens[owner.token_index as int].kind == CompletedTokenKind::YamlDirective
            || tokens[owner.token_index as int].kind == CompletedTokenKind::TagDirective
            || tokens[owner.token_index as int].kind == CompletedTokenKind::ReservedDirective),
        CstSyntaxOwnerKind::DocumentStartMarker => owner.record_index < documents.len()
            && documents[owner.record_index as int].explicit_start_token == Some(owner.token_index)
            && tokens[owner.token_index as int].kind == CompletedTokenKind::DirectivesEnd,
        CstSyntaxOwnerKind::DocumentEndMarker => owner.record_index < documents.len()
            && documents[owner.record_index as int].explicit_end_token == Some(owner.token_index)
            && tokens[owner.token_index as int].kind == CompletedTokenKind::DocumentEnd,
        CstSyntaxOwnerKind::NodeProperty => owner.record_index < nodes.len() && (
        nodes[owner.record_index as int].anchor_property_token == Some(owner.token_index)
            || nodes[owner.record_index as int].tag_property_token == Some(owner.token_index)) && (
        tokens[owner.token_index as int].kind == CompletedTokenKind::AnchorProperty
            || tokens[owner.token_index as int].kind == CompletedTokenKind::TagProperty
            || tokens[owner.token_index as int].kind == CompletedTokenKind::VerbatimTagProperty),
        CstSyntaxOwnerKind::NodeContent => owner.record_index < nodes.len()
            && nodes[owner.record_index as int].scalar_or_alias_token == Some(owner.token_index)
            && (tokens[owner.token_index as int].kind == CompletedTokenKind::Alias
            || cst_style_matches_token_spec(
            nodes[owner.record_index as int].style,
            tokens[owner.token_index as int].kind,
        )),
        CstSyntaxOwnerKind::NodeCollectionIndicator => owner.record_index < nodes.len()
            && nodes[owner.record_index as int].style == CstNodeStyle::Flow && (
        nodes[owner.record_index as int].collection_start_token == Some(owner.token_index)
            || nodes[owner.record_index as int].collection_end_token == Some(owner.token_index))
            && if nodes[owner.record_index as int].kind == CstNodeKind::Sequence {
            tokens[owner.token_index as int].kind == CompletedTokenKind::FlowSequenceStart
                || tokens[owner.token_index as int].kind == CompletedTokenKind::FlowSequenceEnd
        } else {
            nodes[owner.record_index as int].kind == CstNodeKind::Mapping && (
            tokens[owner.token_index as int].kind == CompletedTokenKind::FlowMappingStart
                || tokens[owner.token_index as int].kind == CompletedTokenKind::FlowMappingEnd)
        },
        CstSyntaxOwnerKind::SequenceEntryIndicator => owner.record_index < sequence_entries.len()
            && sequence_entries[owner.record_index as int].indicator_token == Some(
            owner.token_index,
        ) && tokens[owner.token_index as int].kind == CompletedTokenKind::BlockSequenceEntry,
        CstSyntaxOwnerKind::MappingEntryIndicator => owner.record_index < mapping_entries.len() && (
        mapping_entries[owner.record_index as int].explicit_key_token == Some(owner.token_index)
            || mapping_entries[owner.record_index as int].mapping_value_token == Some(
            owner.token_index,
        )) && (tokens[owner.token_index as int].kind == CompletedTokenKind::ExplicitMappingKey
            || tokens[owner.token_index as int].kind == CompletedTokenKind::MappingValue),
        CstSyntaxOwnerKind::FlowEntryIndicator => owner.record_index < nodes.len()
            && nodes[owner.record_index as int].style == CstNodeStyle::Flow && (
        nodes[owner.record_index as int].kind == CstNodeKind::Sequence
            || nodes[owner.record_index as int].kind == CstNodeKind::Mapping)
            && nodes[owner.record_index as int].token_start < owner.token_index
            < nodes[owner.record_index as int].token_end && tokens[owner.token_index as int].kind
            == CompletedTokenKind::FlowEntry,
    }
}

pub open spec fn cst_document_references_owned_spec(
    document: CstDocumentView,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: u64,
) -> bool {
    (match document.explicit_start_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::DocumentStartMarker,
            index,
        ),
        None => true,
    }) && (match document.explicit_end_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::DocumentEndMarker,
            index,
        ),
        None => true,
    })
}

pub open spec fn cst_node_references_owned_spec(
    node: CstNodeView,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: u64,
) -> bool {
    (match node.anchor_property_token {
        Some(token) => cst_owner_at_spec(owners, token, CstSyntaxOwnerKind::NodeProperty, index),
        None => true,
    }) && (match node.tag_property_token {
        Some(token) => cst_owner_at_spec(owners, token, CstSyntaxOwnerKind::NodeProperty, index),
        None => true,
    }) && (match node.scalar_or_alias_token {
        Some(token) => cst_owner_at_spec(owners, token, CstSyntaxOwnerKind::NodeContent, index),
        None => true,
    }) && (match node.collection_start_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::NodeCollectionIndicator,
            index,
        ),
        None => true,
    }) && (match node.collection_end_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::NodeCollectionIndicator,
            index,
        ),
        None => true,
    })
}

pub open spec fn cst_sequence_entry_reference_owned_spec(
    entry: CstSequenceEntryView,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: u64,
) -> bool {
    match entry.indicator_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::SequenceEntryIndicator,
            index,
        ),
        None => true,
    }
}

pub open spec fn cst_mapping_entry_references_owned_spec(
    entry: CstMappingEntryView,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: u64,
) -> bool {
    (match entry.explicit_key_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::MappingEntryIndicator,
            index,
        ),
        None => true,
    }) && (match entry.mapping_value_token {
        Some(token) => cst_owner_at_spec(
            owners,
            token,
            CstSyntaxOwnerKind::MappingEntryIndicator,
            index,
        ),
        None => true,
    })
}

pub open spec fn cst_owner_slots_valid_from_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    token_index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if token_index >= tokens.len() {
        token_index == tokens.len() && owners.len() == tokens.len()
    } else if token_index < 0 || owners.len() != tokens.len() || fuel == 0 {
        false
    } else {
        (if cst_token_is_trivia_spec(tokens[token_index].kind) {
            owners[token_index].is_none()
        } else {
            owners[token_index].is_some() && owners[token_index].unwrap().token_index == token_index
                && cst_syntax_owner_record_spec(
                tokens,
                documents,
                nodes,
                sequence_entries,
                mapping_entries,
                owners[token_index].unwrap(),
            )
        }) && cst_owner_slots_valid_from_spec(
            tokens,
            documents,
            nodes,
            sequence_entries,
            mapping_entries,
            owners,
            token_index + 1,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn cst_document_references_owned_from_spec(
    documents: Seq<CstDocumentView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if index >= documents.len() {
        index == documents.len()
    } else if index < 0 || fuel == 0 {
        false
    } else {
        cst_document_references_owned_spec(documents[index], owners, index as u64)
            && cst_document_references_owned_from_spec(
            documents,
            owners,
            index + 1,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn cst_node_references_owned_from_spec(
    nodes: Seq<CstNodeView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if index >= nodes.len() {
        index == nodes.len()
    } else if index < 0 || fuel == 0 {
        false
    } else {
        cst_node_references_owned_spec(nodes[index], owners, index as u64)
            && cst_node_references_owned_from_spec(nodes, owners, index + 1, (fuel - 1) as nat)
    }
}

pub open spec fn cst_sequence_references_owned_from_spec(
    entries: Seq<CstSequenceEntryView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if index >= entries.len() {
        index == entries.len()
    } else if index < 0 || fuel == 0 {
        false
    } else {
        cst_sequence_entry_reference_owned_spec(entries[index], owners, index as u64)
            && cst_sequence_references_owned_from_spec(
            entries,
            owners,
            index + 1,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn cst_mapping_references_owned_from_spec(
    entries: Seq<CstMappingEntryView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if index >= entries.len() {
        index == entries.len()
    } else if index < 0 || fuel == 0 {
        false
    } else {
        cst_mapping_entry_references_owned_spec(entries[index], owners, index as u64)
            && cst_mapping_references_owned_from_spec(entries, owners, index + 1, (fuel - 1) as nat)
    }
}

pub open spec fn cst_references_have_exact_owners_spec(
    documents: Seq<CstDocumentView>,
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
) -> bool {
    cst_document_references_owned_from_spec(documents, owners, 0, (documents.len() + 1) as nat)
        && cst_node_references_owned_from_spec(nodes, owners, 0, (nodes.len() + 1) as nat)
        && cst_sequence_references_owned_from_spec(
        sequence_entries,
        owners,
        0,
        (sequence_entries.len() + 1) as nat,
    ) && cst_mapping_references_owned_from_spec(
        mapping_entries,
        owners,
        0,
        (mapping_entries.len() + 1) as nat,
    )
}

pub open spec fn cst_exact_syntax_ownership_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    nodes: Seq<CstNodeView>,
    sequence_entries: Seq<CstSequenceEntryView>,
    mapping_entries: Seq<CstMappingEntryView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
) -> bool {
    cst_owner_slots_valid_from_spec(
        tokens,
        documents,
        nodes,
        sequence_entries,
        mapping_entries,
        owners,
        0,
        (tokens.len() + 1) as nat,
    ) && cst_references_have_exact_owners_spec(
        documents,
        nodes,
        sequence_entries,
        mapping_entries,
        owners,
    )
}

pub open spec fn cst_document_record_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    source_len_bytes: u64,
    nodes: Seq<CstNodeView>,
    document: CstDocumentView,
) -> bool {
    document.token_start < document.token_end <= tokens.len() && document.byte_start
        <= document.byte_end <= source_len_bytes && document.byte_start == cst_byte_at_spec(
        tokens,
        document.token_start,
        source_len_bytes,
    ) && document.byte_end == tokens[(document.token_end - 1) as int].byte_end
        && document.token_start == document.prefix_token_start && document.prefix_token_start
        <= document.prefix_token_end && document.prefix_token_end == document.directive_start
        && document.directive_start <= document.directive_end && document.directive_end
        == document.explicit_start_token_start && document.explicit_start_token_start
        <= document.explicit_start_token_end && document.explicit_start_token_end
        == document.root_token_start && document.root_token_start <= document.root_token_end
        && document.root_token_end == document.explicit_end_token_start
        && document.explicit_end_token_start <= document.explicit_end_token_end
        && document.explicit_end_token_end == document.suffix_token_start
        && document.suffix_token_start <= document.suffix_token_end && document.suffix_token_end
        == document.token_end && document.root_node_index < nodes.len() && document.root_token_start
        == nodes[document.root_node_index as int].token_start && document.root_token_end
        == nodes[document.root_node_index as int].token_end && match document.explicit_start_token {
        Some(index) => document.explicit_start_token_start <= index
            < document.explicit_start_token_end && tokens[index as int].kind
            == CompletedTokenKind::DirectivesEnd,
        None => document.explicit_start_token_start == document.explicit_start_token_end,
    } && match document.explicit_end_token {
        Some(index) => document.explicit_end_token_start <= index < document.explicit_end_token_end
            && tokens[index as int].kind == CompletedTokenKind::DocumentEnd,
        None => document.explicit_end_token_start == document.explicit_end_token_end,
    }
}

pub open spec fn cst_warning_record_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    warning: CstWarningView,
) -> bool {
    warning.document_index < documents.len() && warning.token_index
        >= documents[warning.document_index as int].directive_start && warning.token_index
        < documents[warning.document_index as int].directive_end && warning.token_index
        < tokens.len() && warning.byte_offset == tokens[warning.token_index as int].byte_start
        && match warning.kind {
        CstWarningKind::Yaml11Compatibility => tokens[warning.token_index as int].kind
            == CompletedTokenKind::YamlDirective && tokens[warning.token_index as int].yaml_major
            == Some(1) && tokens[warning.token_index as int].yaml_minor == Some(1),
        CstWarningKind::FutureMinorVersion => tokens[warning.token_index as int].kind
            == CompletedTokenKind::YamlDirective && tokens[warning.token_index as int].yaml_major
            == Some(1) && tokens[warning.token_index as int].yaml_minor.is_some()
            && tokens[warning.token_index as int].yaml_minor.unwrap() > 2,
        CstWarningKind::ReservedDirective => tokens[warning.token_index as int].kind
            == CompletedTokenKind::ReservedDirective,
    }
}

pub open spec fn cst_documents_ordered_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    source_len_bytes: u64,
    documents: Seq<CstDocumentView>,
    nodes: Seq<CstNodeView>,
) -> bool {
    if documents.len() == 0 {
        nodes.len() == 0 && forall|token_index: int|
            #![auto]
            0 <= token_index < tokens.len() ==> cst_token_is_trivia_spec(tokens[token_index].kind)
    } else {
        nodes.len() > 0 && documents[0].token_start == 0 && documents[documents.len() - 1].token_end
            == tokens.len() && documents[documents.len() - 1].root_node_index + 1 == nodes.len()
            && forall|document_index: int|
            #![auto]
            0 <= document_index < documents.len() ==> cst_document_record_spec(
                tokens,
                source_len_bytes,
                nodes,
                documents[document_index],
            ) && (document_index + 1 < documents.len() ==> documents[document_index].token_end
                == documents[document_index + 1].token_start
                && documents[document_index].root_node_index < documents[document_index
                + 1].root_node_index)
    }
}

pub open spec fn cst_warnings_ordered_from_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    warnings: Seq<CstWarningView>,
    index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if index >= warnings.len() {
        index == warnings.len()
    } else if index < 0 || fuel == 0 {
        false
    } else {
        cst_warning_record_spec(tokens, documents, warnings[index]) && (index + 1 >= warnings.len()
            || warnings[index].token_index < warnings[index + 1].token_index)
            && cst_warnings_ordered_from_spec(
            tokens,
            documents,
            warnings,
            index + 1,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn cst_warnings_ordered_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    warnings: Seq<CstWarningView>,
) -> bool {
    cst_warnings_ordered_from_spec(tokens, documents, warnings, 0, (warnings.len() + 1) as nat)
}

pub open spec fn cst_documents_and_warnings_ordered_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    source_len_bytes: u64,
    documents: Seq<CstDocumentView>,
    nodes: Seq<CstNodeView>,
    warnings: Seq<CstWarningView>,
) -> bool {
    cst_documents_ordered_spec(tokens, source_len_bytes, documents, nodes)
        && cst_warnings_ordered_spec(tokens, documents, warnings)
}

pub open spec fn cst_source_respects_limits_spec(
    source: CstSourceView,
    limits: CstLimitsView,
) -> bool {
    source.documents.len() <= cst_effective_limit_spec(
        limits.max_documents,
        MAX_PROFILE1_CST_DOCUMENTS,
    ) && source.nodes.len() <= cst_effective_limit_spec(limits.max_nodes, MAX_PROFILE1_CST_NODES)
        && source.sequence_entries.len() <= cst_effective_limit_spec(
        limits.max_sequence_entries,
        MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
    ) && source.mapping_entries.len() <= cst_effective_limit_spec(
        limits.max_mapping_entries,
        MAX_PROFILE1_CST_MAPPING_ENTRIES,
    ) && source.directive_count <= cst_effective_limit_spec(
        limits.max_directives,
        MAX_PROFILE1_CST_DIRECTIVES,
    ) && source.warnings.len() <= cst_effective_limit_spec(
        limits.max_warnings,
        MAX_PROFILE1_CST_WARNINGS,
    ) && source.maximum_depth <= cst_effective_limit_spec(limits.max_depth, MAX_PROFILE1_CST_DEPTH)
}

pub open spec fn cst_public_semantics_spec(
    tokens: crate::token::CompletedTokenSourceView,
    source: CstSourceView,
) -> bool {
    source.profile_version == CRUCIBLE_YAML_PROFILE_VERSION
        && source.input_token_transformation_version == tokens.transformation_version
        && source.transformation_version == CST_TRANSFORMATION_VERSION && source.source_len_bytes
        == tokens.source_len_bytes && source.input_token_count == tokens.tokens.len()
        && cst_source_respects_limits_spec(
        source,
        CstLimitsView {
            max_documents: MAX_PROFILE1_CST_DOCUMENTS,
            max_nodes: MAX_PROFILE1_CST_NODES,
            max_sequence_entries: MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
            max_mapping_entries: MAX_PROFILE1_CST_MAPPING_ENTRIES,
            max_directives: MAX_PROFILE1_CST_DIRECTIVES,
            max_warnings: MAX_PROFILE1_CST_WARNINGS,
            max_depth: MAX_PROFILE1_CST_DEPTH,
        },
    ) && cst_child_before_parent_spec(source.nodes, source.sequence_entries, source.mapping_entries)
        && cst_entry_tables_uniquely_owned_spec(
        source.nodes,
        source.sequence_entries.len() as u64,
        source.mapping_entries.len() as u64,
    ) && cst_nodes_have_exact_token_identity_spec(
        tokens.tokens,
        tokens.source_len_bytes,
        source.nodes,
    ) && cst_entry_ranges_spec(
        tokens.tokens.len() as u64,
        source.nodes,
        source.sequence_entries,
        source.mapping_entries,
    ) && cst_documents_and_warnings_ordered_spec(
        tokens.tokens,
        tokens.source_len_bytes,
        source.documents,
        source.nodes,
        source.warnings,
    ) && cst_exact_syntax_ownership_spec(
        tokens.tokens,
        source.documents,
        source.nodes,
        source.sequence_entries,
        source.mapping_entries,
        source.syntax_owners,
    )
}

fn cst_token_kind_at(tokens: &[CompletedToken], index: usize) -> (kind: CompletedTokenKind)
    requires
        index < tokens.len(),
    ensures
        kind == crate::token::completed_token_views_spec(tokens@)[index as int].kind,
{
    let token = &tokens[index];
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, index as int);
    }
    token.kind()
}

fn cst_token_byte_start_at(tokens: &[CompletedToken], index: usize) -> (offset: u64)
    requires
        index < tokens.len(),
    ensures
        offset == crate::token::completed_token_views_spec(tokens@)[index as int].byte_start,
{
    let token = &tokens[index];
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, index as int);
    }
    token.byte_start()
}

fn cst_token_byte_end_at(tokens: &[CompletedToken], index: usize) -> (offset: u64)
    requires
        index < tokens.len(),
    ensures
        offset == crate::token::completed_token_views_spec(tokens@)[index as int].byte_end,
{
    let token = &tokens[index];
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, index as int);
    }
    token.byte_end()
}

fn cst_style_matches_token(style: CstNodeStyle, kind: CompletedTokenKind) -> (result: bool)
    ensures
        result == cst_style_matches_token_spec(style, kind),
{
    match style {
        CstNodeStyle::Plain => kind == CompletedTokenKind::PlainScalar,
        CstNodeStyle::SingleQuoted => kind == CompletedTokenKind::SingleQuotedScalar,
        CstNodeStyle::DoubleQuoted => kind == CompletedTokenKind::DoubleQuotedScalar,
        CstNodeStyle::Literal => kind == CompletedTokenKind::LiteralBlockScalar,
        CstNodeStyle::Folded => kind == CompletedTokenKind::FoldedBlockScalar,
        CstNodeStyle::Alias => kind == CompletedTokenKind::Alias,
        _ => false,
    }
}

fn cst_node_has_exact_token_identity(
    tokens: &[CompletedToken],
    source_len_bytes: u64,
    node: &CstNode,
) -> (result: bool)
    ensures
        result == cst_node_token_identity_spec(
            crate::token::completed_token_views_spec(tokens@),
            source_len_bytes,
            node@,
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        assert(token_views.len() == tokens.len());
        reveal(cst_node_token_identity_spec);
        reveal(cst_byte_at_spec);
    }
    if node.token_start > node.token_end || node.token_end > tokens.len() as u64 || node.byte_start
        > node.byte_end || node.byte_end > source_len_bytes {
        proof {
            reveal(cst_node_token_identity_spec);
            assert(!cst_node_token_identity_spec(token_views, source_len_bytes, node@));
        }
        return false;
    }
    let expected_start = if node.token_start < tokens.len() as u64 {
        cst_token_byte_start_at(tokens, node.token_start as usize)
    } else {
        source_len_bytes
    };
    if node.byte_start != expected_start {
        return false;
    }
    if let Some(index) = node.anchor_property_token {
        if index < node.token_start || index >= node.token_end || index >= tokens.len() as u64
            || cst_token_kind_at(tokens, index as usize) != CompletedTokenKind::AnchorProperty {
            return false;
        }
    }
    if let Some(index) = node.tag_property_token {
        if index < node.token_start || index >= node.token_end || index >= tokens.len() as u64 {
            return false;
        }
        let kind = cst_token_kind_at(tokens, index as usize);
        if kind != CompletedTokenKind::TagProperty && kind
            != CompletedTokenKind::VerbatimTagProperty {
            return false;
        }
    }
    if node.anchor_property_token.is_some() && node.anchor_property_token
        == node.tag_property_token {
        return false;
    }
    if node.kind == CstNodeKind::Empty {
        let anchor_byte = if node.token_end < tokens.len() as u64 {
            cst_token_byte_start_at(tokens, node.token_end as usize)
        } else {
            source_len_bytes
        };
        return node.style == CstNodeStyle::Empty && node.scalar_or_alias_token.is_none()
            && node.collection_start_token.is_none() && node.collection_end_token.is_none()
            && node.entry_start == 0 && node.entry_end == 0 && node.empty_anchor_token == Some(
            node.token_end,
        ) && node.empty_anchor_byte == Some(anchor_byte) && node.byte_end == anchor_byte;
    }
    if node.kind == CstNodeKind::Scalar {
        if node.scalar_or_alias_token.is_none() || node.empty_anchor_token.is_some()
            || node.empty_anchor_byte.is_some() || node.collection_start_token.is_some()
            || node.collection_end_token.is_some() || node.entry_start != 0 || node.entry_end != 0 {
            return false;
        }
        let index = node.scalar_or_alias_token.unwrap();
        if index < node.token_start || index >= node.token_end || index >= tokens.len() as u64
            || node.token_end == 0 {
            return false;
        }
        let kind = cst_token_kind_at(tokens, index as usize);
        let exact_end = cst_token_byte_end_at(tokens, (node.token_end - 1) as usize);
        let matches = cst_style_matches_token(node.style, kind) && node.style != CstNodeStyle::Alias
            && node.byte_end == exact_end;
        proof {
            assert(node.token_end > 0);
            assert(index < token_views.len());
            assert(kind == token_views[index as int].kind);
            assert(node.token_end - 1 < token_views.len());
            assert(exact_end == token_views[(node.token_end - 1) as int].byte_end);
            reveal(cst_node_token_identity_spec);
            assert(cst_node_token_identity_spec(token_views, source_len_bytes, node@) == matches);
        }
        return matches;
    }
    if node.kind == CstNodeKind::Alias {
        if node.style != CstNodeStyle::Alias || node.anchor_property_token.is_some()
            || node.tag_property_token.is_some() || node.scalar_or_alias_token.is_none()
            || node.empty_anchor_token.is_some() || node.empty_anchor_byte.is_some()
            || node.collection_start_token.is_some() || node.collection_end_token.is_some()
            || node.entry_start != 0 || node.entry_end != 0 {
            return false;
        }
        let index = node.scalar_or_alias_token.unwrap();
        if index < node.token_start || index >= node.token_end || index >= tokens.len() as u64
            || node.token_end == 0 {
            return false;
        }
        return cst_token_kind_at(tokens, index as usize) == CompletedTokenKind::Alias
            && node.byte_end == cst_token_byte_end_at(tokens, (node.token_end - 1) as usize);
    }
    if node.token_start >= node.token_end {
        return false;
    }
    let collection_shape = (node.style == CstNodeStyle::Block || node.style == CstNodeStyle::Flow
        || node.style == CstNodeStyle::FlowPair) && node.scalar_or_alias_token.is_none()
        && node.empty_anchor_token.is_none() && node.empty_anchor_byte.is_none() && node.byte_end
        == cst_token_byte_end_at(tokens, (node.token_end - 1) as usize);
    if !collection_shape || !(node.kind == CstNodeKind::Sequence || node.kind
        == CstNodeKind::Mapping) {
        return false;
    }
    if node.style == CstNodeStyle::Block {
        return node.collection_start_token.is_none() && node.collection_end_token.is_none();
    }
    if node.style == CstNodeStyle::FlowPair {
        return node.kind == CstNodeKind::Mapping && node.collection_start_token.is_none()
            && node.collection_end_token.is_none();
    }
    if node.collection_start_token.is_none() || node.collection_end_token.is_none() {
        return false;
    }
    let start = node.collection_start_token.unwrap();
    let end = node.collection_end_token.unwrap();
    if start < node.token_start || start >= node.token_end || end < node.token_start || end
        >= node.token_end {
        return false;
    }
    if node.kind == CstNodeKind::Sequence {
        cst_token_kind_at(tokens, start as usize) == CompletedTokenKind::FlowSequenceStart
            && cst_token_kind_at(tokens, end as usize) == CompletedTokenKind::FlowSequenceEnd
    } else {
        cst_token_kind_at(tokens, start as usize) == CompletedTokenKind::FlowMappingStart
            && cst_token_kind_at(tokens, end as usize) == CompletedTokenKind::FlowMappingEnd
    }
}

fn cst_nodes_have_exact_token_identity(
    tokens: &[CompletedToken],
    source_len_bytes: u64,
    nodes: &[CstNode],
) -> (result: bool)
    ensures
        result == cst_nodes_have_exact_token_identity_spec(
            crate::token::completed_token_views_spec(tokens@),
            source_len_bytes,
            cst_node_views_spec(nodes@),
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost node_views = cst_node_views_spec(nodes@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(cst_node_views_spec);
        assert(token_views.len() == tokens.len());
        assert(node_views.len() == nodes.len());
    }
    let mut index = 0usize;
    while index < nodes.len()
        invariant
            index <= nodes.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            node_views == cst_node_views_spec(nodes@),
            token_views.len() == tokens.len(),
            node_views.len() == nodes.len(),
            forall|prior: int|
                #![auto]
                0 <= prior < index ==> cst_node_token_identity_spec(
                    token_views,
                    source_len_bytes,
                    node_views[prior],
                ),
        decreases nodes.len() - index,
    {
        assert(node_views[index as int] == nodes[index as int]@) by {
            lemma_cst_node_view_at(nodes@, index as int);
        }
        let node = &nodes[index];
        if !cst_node_has_exact_token_identity(tokens, source_len_bytes, node) {
            proof {
                assert(!cst_node_token_identity_spec(
                    token_views,
                    source_len_bytes,
                    node_views[index as int],
                ));
                reveal(cst_nodes_have_exact_token_identity_spec);
                assert(!cst_nodes_have_exact_token_identity_spec(
                    token_views,
                    source_len_bytes,
                    node_views,
                ));
            }
            return false;
        }
        index += 1;
    }
    proof {
        reveal(cst_nodes_have_exact_token_identity_spec);
    }
    true
}

fn cst_node_token_start_at(nodes: &[CstNode], index: usize) -> (result: u64)
    requires
        index < nodes.len(),
    ensures
        result == cst_node_views_spec(nodes@)[index as int].token_start,
{
    let result = nodes[index].token_start();
    proof {
        lemma_cst_node_view_at(nodes@, index as int);
    }
    result
}

fn cst_node_token_end_at(nodes: &[CstNode], index: usize) -> (result: u64)
    requires
        index < nodes.len(),
    ensures
        result == cst_node_views_spec(nodes@)[index as int].token_end,
{
    let result = nodes[index].token_end();
    proof {
        lemma_cst_node_view_at(nodes@, index as int);
    }
    result
}

fn cst_entries_have_valid_ranges(
    token_count: usize,
    nodes: &[CstNode],
    sequence_entries: &[CstSequenceEntry],
    mapping_entries: &[CstMappingEntry],
) -> (result: bool)
    ensures
        result == cst_entry_ranges_spec(
            token_count as u64,
            cst_node_views_spec(nodes@),
            cst_sequence_entry_views_spec(sequence_entries@),
            cst_mapping_entry_views_spec(mapping_entries@),
        ),
{
    let ghost node_views = cst_node_views_spec(nodes@);
    let ghost sequence_views = cst_sequence_entry_views_spec(sequence_entries@);
    let ghost mapping_views = cst_mapping_entry_views_spec(mapping_entries@);
    proof {
        reveal(cst_sequence_entry_views_spec);
        reveal(cst_mapping_entry_views_spec);
        reveal(cst_node_views_spec);
        assert(sequence_views.len() == sequence_entries.len());
        assert(mapping_views.len() == mapping_entries.len());
        assert(node_views.len() == nodes.len());
    }
    let mut sequence_index = 0usize;
    while sequence_index < sequence_entries.len()
        invariant
            sequence_index <= sequence_entries.len(),
            sequence_views == cst_sequence_entry_views_spec(sequence_entries@),
            sequence_views.len() == sequence_entries.len(),
            node_views == cst_node_views_spec(nodes@),
            node_views.len() == nodes.len(),
            forall|prior: int|
                #![auto]
                0 <= prior < sequence_index ==> sequence_views[prior].token_start
                    < sequence_views[prior].token_end <= token_count
                    && sequence_views[prior].node_index < node_views.len()
                    && sequence_views[prior].token_start
                    <= node_views[sequence_views[prior].node_index as int].token_start
                    && node_views[sequence_views[prior].node_index as int].token_end
                    <= sequence_views[prior].token_end,
        decreases sequence_entries.len() - sequence_index,
    {
        assert(sequence_views[sequence_index as int] == sequence_entries[sequence_index as int]@)
            by {
            lemma_cst_sequence_entry_view_at(sequence_entries@, sequence_index as int);
            lemma_cst_sequence_entry_view_fields(sequence_entries@[sequence_index as int]);
        }
        let entry = sequence_entries[sequence_index];
        if entry.token_start >= entry.token_end || entry.token_end > token_count as u64
            || entry.node_index >= nodes.len() as u64 {
            proof {
                reveal(cst_entry_ranges_spec);
                assert(!cst_entry_ranges_spec(
                    token_count as u64,
                    node_views,
                    sequence_views,
                    mapping_views,
                ));
            }
            return false;
        }
        proof {
            lemma_cst_sequence_entry_view_fields(entry);
            assert(sequence_views[sequence_index as int] == entry@);
            assert((entry.node_index as usize) as u64 == entry.node_index);
            assert((entry.node_index as usize) as int == entry.node_index as int);
            lemma_cst_node_view_at(nodes@, entry.node_index as int);
            lemma_cst_node_view_fields(nodes@[entry.node_index as int]);
        }
        let child_start = cst_node_token_start_at(nodes, entry.node_index as usize);
        let child_end = cst_node_token_end_at(nodes, entry.node_index as usize);
        if entry.token_start > child_start || child_end > entry.token_end {
            proof {
                reveal(cst_entry_ranges_spec);
            }
            return false;
        }
        proof {
            assert(sequence_views[sequence_index as int].token_start
                < sequence_views[sequence_index as int].token_end <= token_count);
            assert(sequence_views[sequence_index as int].node_index < node_views.len());
            assert(sequence_views[sequence_index as int].token_start
                <= node_views[sequence_views[sequence_index as int].node_index as int].token_start);
            assert(node_views[sequence_views[sequence_index as int].node_index as int].token_end
                <= sequence_views[sequence_index as int].token_end);
        }
        sequence_index += 1;
    }
    let mut mapping_index = 0usize;
    while mapping_index < mapping_entries.len()
        invariant
            mapping_index <= mapping_entries.len(),
            mapping_views == cst_mapping_entry_views_spec(mapping_entries@),
            mapping_views.len() == mapping_entries.len(),
            node_views == cst_node_views_spec(nodes@),
            node_views.len() == nodes.len(),
            forall|prior: int|
                #![auto]
                0 <= prior < mapping_index ==> mapping_views[prior].token_start
                    < mapping_views[prior].token_end <= token_count
                    && mapping_views[prior].key_node_index < node_views.len()
                    && mapping_views[prior].value_node_index < node_views.len()
                    && mapping_views[prior].token_start
                    <= node_views[mapping_views[prior].key_node_index as int].token_start
                    && node_views[mapping_views[prior].key_node_index as int].token_end
                    <= mapping_views[prior].token_end && mapping_views[prior].token_start
                    <= node_views[mapping_views[prior].value_node_index as int].token_start
                    && node_views[mapping_views[prior].value_node_index as int].token_end
                    <= mapping_views[prior].token_end,
        decreases mapping_entries.len() - mapping_index,
    {
        assert(mapping_views[mapping_index as int] == mapping_entries[mapping_index as int]@) by {
            lemma_cst_mapping_entry_view_at(mapping_entries@, mapping_index as int);
            lemma_cst_mapping_entry_view_fields(mapping_entries@[mapping_index as int]);
        }
        let entry = mapping_entries[mapping_index];
        if entry.token_start >= entry.token_end || entry.token_end > token_count as u64
            || entry.key_node_index >= nodes.len() as u64 || entry.value_node_index
            >= nodes.len() as u64 {
            proof {
                reveal(cst_entry_ranges_spec);
                assert(!cst_entry_ranges_spec(
                    token_count as u64,
                    node_views,
                    sequence_views,
                    mapping_views,
                ));
            }
            return false;
        }
        proof {
            lemma_cst_mapping_entry_view_fields(entry);
            assert(mapping_views[mapping_index as int] == entry@);
            assert((entry.key_node_index as usize) as u64 == entry.key_node_index);
            assert((entry.value_node_index as usize) as u64 == entry.value_node_index);
            assert((entry.key_node_index as usize) as int == entry.key_node_index as int);
            assert((entry.value_node_index as usize) as int == entry.value_node_index as int);
            lemma_cst_node_view_at(nodes@, entry.key_node_index as int);
            lemma_cst_node_view_at(nodes@, entry.value_node_index as int);
            lemma_cst_node_view_fields(nodes@[entry.key_node_index as int]);
            lemma_cst_node_view_fields(nodes@[entry.value_node_index as int]);
        }
        let key_start = cst_node_token_start_at(nodes, entry.key_node_index as usize);
        let key_end = cst_node_token_end_at(nodes, entry.key_node_index as usize);
        let value_start = cst_node_token_start_at(nodes, entry.value_node_index as usize);
        let value_end = cst_node_token_end_at(nodes, entry.value_node_index as usize);
        if entry.token_start > key_start || key_end > entry.token_end || entry.token_start
            > value_start || value_end > entry.token_end {
            proof {
                reveal(cst_entry_ranges_spec);
            }
            return false;
        }
        proof {
            assert(mapping_views[mapping_index as int].token_start
                < mapping_views[mapping_index as int].token_end <= token_count);
            assert(mapping_views[mapping_index as int].key_node_index < node_views.len());
            assert(mapping_views[mapping_index as int].value_node_index < node_views.len());
            assert(mapping_views[mapping_index as int].token_start
                <= node_views[mapping_views[mapping_index as int].key_node_index as int].token_start);
            assert(node_views[mapping_views[mapping_index as int].key_node_index as int].token_end
                <= mapping_views[mapping_index as int].token_end);
            assert(mapping_views[mapping_index as int].token_start
                <= node_views[mapping_views[mapping_index as int].value_node_index as int].token_start);
            assert(node_views[mapping_views[mapping_index as int].value_node_index as int].token_end
                <= mapping_views[mapping_index as int].token_end);
        }
        mapping_index += 1;
    }
    proof {
        reveal(cst_entry_ranges_spec);
    }
    true
}

fn cst_document_record_is_valid(
    tokens: &[CompletedToken],
    source_len_bytes: u64,
    nodes: &[CstNode],
    document: &CstDocument,
) -> (result: bool)
    ensures
        result == cst_document_record_spec(
            crate::token::completed_token_views_spec(tokens@),
            source_len_bytes,
            cst_node_views_spec(nodes@),
            document@,
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost node_views = cst_node_views_spec(nodes@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(cst_node_views_spec);
        reveal(cst_document_record_spec);
        reveal(cst_byte_at_spec);
    }
    if document.token_start >= document.token_end || document.token_end > tokens.len() as u64
        || document.byte_start > document.byte_end || document.byte_end > source_len_bytes
        || document.token_start != document.prefix_token_start || document.prefix_token_start
        > document.prefix_token_end || document.prefix_token_end != document.directive_start
        || document.directive_start > document.directive_end || document.directive_end
        != document.explicit_start_token_start || document.explicit_start_token_start
        > document.explicit_start_token_end || document.explicit_start_token_end
        != document.root_token_start || document.root_token_start > document.root_token_end
        || document.root_token_end != document.explicit_end_token_start
        || document.explicit_end_token_start > document.explicit_end_token_end
        || document.explicit_end_token_end != document.suffix_token_start
        || document.suffix_token_start > document.suffix_token_end || document.suffix_token_end
        != document.token_end || document.root_node_index >= nodes.len() as u64 {
        return false;
    }
    if document.byte_start != cst_token_byte_start_at(tokens, document.token_start as usize)
        || document.byte_end != cst_token_byte_end_at(tokens, (document.token_end - 1) as usize) {
        return false;
    }
    proof {
        lemma_cst_node_view_at(nodes@, document.root_node_index as int);
    }
    let root = &nodes[document.root_node_index as usize];
    if document.root_token_start != root.token_start || document.root_token_end != root.token_end {
        return false;
    }
    if let Some(index) = document.explicit_start_token {
        if index < document.explicit_start_token_start || index >= document.explicit_start_token_end
            || cst_token_kind_at(tokens, index as usize) != CompletedTokenKind::DirectivesEnd {
            return false;
        }
    } else if document.explicit_start_token_start != document.explicit_start_token_end {
        return false;
    }
    if let Some(index) = document.explicit_end_token {
        if index < document.explicit_end_token_start || index >= document.explicit_end_token_end
            || cst_token_kind_at(tokens, index as usize) != CompletedTokenKind::DocumentEnd {
            return false;
        }
    } else if document.explicit_end_token_start != document.explicit_end_token_end {
        return false;
    }
    true
}

fn cst_warning_record_is_valid(
    tokens: &[CompletedToken],
    documents: &[CstDocument],
    warning: &CstWarning,
) -> (result: bool)
    ensures
        result == cst_warning_record_spec(
            crate::token::completed_token_views_spec(tokens@),
            cst_document_views_spec(documents@),
            warning@,
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost document_views = cst_document_views_spec(documents@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(cst_document_views_spec);
        reveal(cst_warning_record_spec);
    }
    if warning.document_index >= documents.len() as u64 || warning.token_index
        >= tokens.len() as u64 {
        return false;
    }
    let document = &documents[warning.document_index as usize];
    if warning.token_index < document.directive_start || warning.token_index
        >= document.directive_end || warning.byte_offset != cst_token_byte_start_at(
        tokens,
        warning.token_index as usize,
    ) {
        return false;
    }
    let token = &tokens[warning.token_index as usize];
    let kind_matches = match warning.kind {
        CstWarningKind::Yaml11Compatibility => token.kind() == CompletedTokenKind::YamlDirective
            && token.yaml_version() == Some((1, 1)),
        CstWarningKind::FutureMinorVersion => {
            if token.kind() != CompletedTokenKind::YamlDirective {
                false
            } else {
                match token.yaml_version() {
                    Some((major, minor)) => major == 1 && minor > 2,
                    None => false,
                }
            }
        },
        CstWarningKind::ReservedDirective => token.kind() == CompletedTokenKind::ReservedDirective,
    };
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, warning.token_index as int);
        lemma_cst_document_view_at(documents@, warning.document_index as int);
    }
    kind_matches
}

#[verifier::rlimit(50)]
fn cst_documents_and_warnings_are_ordered(
    tokens: &[CompletedToken],
    source_len_bytes: u64,
    documents: &[CstDocument],
    nodes: &[CstNode],
    warnings: &[CstWarning],
) -> (result: bool)
    ensures
        result == cst_documents_and_warnings_ordered_spec(
            crate::token::completed_token_views_spec(tokens@),
            source_len_bytes,
            cst_document_views_spec(documents@),
            cst_node_views_spec(nodes@),
            cst_warning_views_spec(warnings@),
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost document_views = cst_document_views_spec(documents@);
    let ghost node_views = cst_node_views_spec(nodes@);
    let ghost warning_views = cst_warning_views_spec(warnings@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(cst_document_views_spec);
        reveal(cst_node_views_spec);
        reveal(cst_warning_views_spec);
        assert(document_views.len() == documents.len());
        assert(node_views.len() == nodes.len());
        assert(warning_views.len() == warnings.len());
    }
    if documents.len() == 0 {
        if nodes.len() != 0 {
            proof {
                reveal(cst_documents_and_warnings_ordered_spec);
            }
            return false;
        }
        let mut token_index = 0usize;
        while token_index < tokens.len()
            invariant
                token_index <= tokens.len(),
                token_views == crate::token::completed_token_views_spec(tokens@),
                token_views.len() == tokens.len(),
                document_views == cst_document_views_spec(documents@),
                document_views.len() == documents.len(),
                documents.len() == 0,
                node_views == cst_node_views_spec(nodes@),
                node_views.len() == nodes.len(),
                nodes.len() == 0,
                warning_views == cst_warning_views_spec(warnings@),
                forall|prior: int|
                    #![auto]
                    0 <= prior < token_index ==> cst_token_is_trivia_spec(token_views[prior].kind),
            decreases tokens.len() - token_index,
        {
            if !is_trivia(tokens[token_index].kind()) {
                proof {
                    crate::token::lemma_completed_token_view_at(tokens@, token_index as int);
                    reveal(cst_documents_and_warnings_ordered_spec);
                    assert(!cst_token_is_trivia_spec(token_views[token_index as int].kind));
                    assert(!(forall|index: int|
                        #![auto]
                        0 <= index < token_views.len() ==> cst_token_is_trivia_spec(
                            token_views[index].kind,
                        ))) by {
                        assert(!cst_token_is_trivia_spec(token_views[token_index as int].kind));
                    }
                    assert(document_views.len() == 0);
                    assert(node_views.len() == 0);
                    assert(!cst_documents_and_warnings_ordered_spec(
                        token_views,
                        source_len_bytes,
                        document_views,
                        node_views,
                        warning_views,
                    ));
                }
                return false;
            }
            proof {
                crate::token::lemma_completed_token_view_at(tokens@, token_index as int);
            }
            token_index += 1;
        }
        proof {
            reveal(cst_documents_ordered_spec);
            assert(cst_documents_ordered_spec(
                token_views,
                source_len_bytes,
                document_views,
                node_views,
            ));
        }
    } else {
        if nodes.len() == 0 {
            proof {
                reveal(cst_documents_and_warnings_ordered_spec);
            }
            return false;
        }
        let last_document = &documents[documents.len() - 1];
        if documents[0].token_start != 0 || last_document.token_end != tokens.len() as u64
            || last_document.root_node_index >= nodes.len() as u64 || last_document.root_node_index
            + 1 != nodes.len() as u64 {
            proof {
                lemma_cst_document_view_at(documents@, (documents.len() - 1) as int);
                reveal(cst_documents_and_warnings_ordered_spec);
            }
            return false;
        }
        proof {
            lemma_cst_document_view_at(documents@, 0);
            lemma_cst_document_view_at(documents@, (documents.len() - 1) as int);
            lemma_cst_document_view_fields(documents@[0]);
            lemma_cst_document_view_fields(documents@[(documents.len() - 1) as int]);
            assert(document_views[0].token_start == 0);
            assert(document_views[document_views.len() - 1].token_end == token_views.len());
            assert(document_views[document_views.len() - 1].root_node_index + 1
                == node_views.len());
        }
        let mut document_index = 0usize;
        while document_index < documents.len()
            invariant
                document_index <= documents.len(),
                token_views == crate::token::completed_token_views_spec(tokens@),
                document_views == cst_document_views_spec(documents@),
                node_views == cst_node_views_spec(nodes@),
                token_views.len() == tokens.len(),
                document_views.len() == documents.len(),
                node_views.len() == nodes.len(),
                document_views.len() > 0,
                node_views.len() > 0,
                document_views[0].token_start == 0,
                document_views[document_views.len() - 1].token_end == token_views.len(),
                document_views[document_views.len() - 1].root_node_index + 1 == node_views.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < document_index ==> cst_document_record_spec(
                        token_views,
                        source_len_bytes,
                        node_views,
                        document_views[prior],
                    ) && (prior + 1 < document_views.len() ==> document_views[prior].token_end
                        == document_views[prior + 1].token_start
                        && document_views[prior].root_node_index < document_views[prior
                        + 1].root_node_index),
            decreases documents.len() - document_index,
        {
            proof {
                lemma_cst_document_view_at(documents@, document_index as int);
            }
            let document = documents[document_index];
            proof {
                lemma_cst_document_view_fields(document);
                assert(document_views[document_index as int] == document@);
            }
            if !cst_document_record_is_valid(tokens, source_len_bytes, nodes, &document) {
                proof {
                    reveal(cst_documents_and_warnings_ordered_spec);
                }
                return false;
            }
            if document_index + 1 < documents.len() && (document.token_end
                != documents[document_index + 1].token_start || document.root_node_index
                >= documents[document_index + 1].root_node_index) {
                proof {
                    lemma_cst_document_view_at(documents@, (document_index + 1) as int);
                    reveal(cst_documents_and_warnings_ordered_spec);
                }
                return false;
            }
            if document_index + 1 < documents.len() {
                proof {
                    lemma_cst_document_view_at(documents@, (document_index + 1) as int);
                }
            }
            proof {
                assert(cst_document_record_spec(
                    token_views,
                    source_len_bytes,
                    node_views,
                    document_views[document_index as int],
                ));
                assert(document_index + 1 >= document_views.len()
                    || document_views[document_index as int].token_end == document_views[(
                document_index + 1) as int].token_start
                    && document_views[document_index as int].root_node_index < document_views[(
                document_index + 1) as int].root_node_index);
            }
            document_index += 1;
        }
        proof {
            reveal(cst_documents_ordered_spec);
            assert(cst_documents_ordered_spec(
                token_views,
                source_len_bytes,
                document_views,
                node_views,
            ));
        }
    }
    let ghost warnings_expected = cst_warnings_ordered_from_spec(
        token_views,
        document_views,
        warning_views,
        0,
        (warning_views.len() + 1) as nat,
    );
    let mut warning_index = 0usize;
    let ghost mut warning_fuel: nat = (warning_views.len() + 1) as nat;
    while warning_index < warnings.len()
        invariant
            warning_index <= warnings.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            document_views == cst_document_views_spec(documents@),
            warning_views == cst_warning_views_spec(warnings@),
            warning_views.len() == warnings.len(),
            cst_documents_ordered_spec(token_views, source_len_bytes, document_views, node_views),
            warnings_expected == cst_warnings_ordered_from_spec(
                token_views,
                document_views,
                warning_views,
                0,
                (warning_views.len() + 1) as nat,
            ),
            warning_fuel >= warning_views.len() - warning_index + 1,
            warnings_expected == cst_warnings_ordered_from_spec(
                token_views,
                document_views,
                warning_views,
                warning_index as int,
                warning_fuel,
            ),
        decreases warnings.len() - warning_index,
    {
        proof {
            lemma_cst_warning_view_at(warnings@, warning_index as int);
            assert(warning_views[warning_index as int] == warnings[warning_index as int]@);
        }
        let warning = &warnings[warning_index];
        if !cst_warning_record_is_valid(tokens, documents, warning) {
            proof {
                assert(!cst_warning_record_spec(
                    token_views,
                    document_views,
                    warning_views[warning_index as int],
                ));
                reveal(cst_warnings_ordered_from_spec);
                assert(!warnings_expected);
                reveal(cst_warnings_ordered_spec);
                reveal(cst_documents_and_warnings_ordered_spec);
            }
            return false;
        }
        proof {
            assert(cst_warning_record_spec(token_views, document_views, warning@));
        }
        if warning_index + 1 < warnings.len() {
            if warning.token_index >= warnings[warning_index + 1].token_index {
                proof {
                    lemma_cst_warning_view_at(warnings@, (warning_index + 1) as int);
                    reveal(cst_warnings_ordered_from_spec);
                    assert(!warnings_expected);
                    reveal(cst_warnings_ordered_spec);
                    reveal(cst_documents_and_warnings_ordered_spec);
                }
                return false;
            }
            proof {
                lemma_cst_warning_view_at(warnings@, (warning_index + 1) as int);
                assert(warning_views[warning_index as int].token_index < warning_views[(
                warning_index + 1) as int].token_index);
            }
        }
        proof {
            reveal(cst_warnings_ordered_from_spec);
            warning_fuel = (warning_fuel - 1) as nat;
        }
        warning_index += 1;
    }
    proof {
        reveal(cst_warnings_ordered_from_spec);
        assert(warnings_expected);
        reveal(cst_warnings_ordered_spec);
        reveal(cst_documents_and_warnings_ordered_spec);
    }
    true
}

fn cst_source_respects_limits(source: &CstSource, limits: CstLimits) -> (result: bool)
    ensures
        result == cst_source_respects_limits_spec(source@, limits@),
{
    source.documents.len() as u64 <= effective_limit(
        limits.max_documents,
        MAX_PROFILE1_CST_DOCUMENTS,
    ) && source.nodes.len() as u64 <= effective_limit(limits.max_nodes, MAX_PROFILE1_CST_NODES)
        && source.sequence_entries.len() as u64 <= effective_limit(
        limits.max_sequence_entries,
        MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
    ) && source.mapping_entries.len() as u64 <= effective_limit(
        limits.max_mapping_entries,
        MAX_PROFILE1_CST_MAPPING_ENTRIES,
    ) && source.directive_count <= effective_limit(
        limits.max_directives,
        MAX_PROFILE1_CST_DIRECTIVES,
    ) && source.warnings.len() as u64 <= effective_limit(
        limits.max_warnings,
        MAX_PROFILE1_CST_WARNINGS,
    ) && source.maximum_depth <= effective_limit(limits.max_depth, MAX_PROFILE1_CST_DEPTH)
}

fn cst_owner_slot_matches(
    owners: &[Option<CstSyntaxOwner>],
    token_index: u64,
    kind: CstSyntaxOwnerKind,
    record_index: u64,
) -> (result: bool)
    ensures
        result == cst_owner_at_spec(
            cst_syntax_owner_views_spec(owners@),
            token_index,
            kind,
            record_index,
        ),
{
    if token_index >= owners.len() as u64 {
        return false;
    }
    match owners[token_index as usize] {
        Some(owner) => owner.token_index == token_index && owner.kind == kind && owner.record_index
            == record_index,
        None => false,
    }
}

fn cst_syntax_owner_record_is_valid(
    tokens: &[CompletedToken],
    documents: &[CstDocument],
    nodes: &[CstNode],
    sequence_entries: &[CstSequenceEntry],
    mapping_entries: &[CstMappingEntry],
    owner: CstSyntaxOwner,
) -> (result: bool)
    ensures
        result == cst_syntax_owner_record_spec(
            crate::token::completed_token_views_spec(tokens@),
            cst_document_views_spec(documents@),
            cst_node_views_spec(nodes@),
            cst_sequence_entry_views_spec(sequence_entries@),
            cst_mapping_entry_views_spec(mapping_entries@),
            owner@,
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost document_views = cst_document_views_spec(documents@);
    let ghost node_views = cst_node_views_spec(nodes@);
    let ghost sequence_views = cst_sequence_entry_views_spec(sequence_entries@);
    let ghost mapping_views = cst_mapping_entry_views_spec(mapping_entries@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(cst_document_views_spec);
        reveal(cst_node_views_spec);
        reveal(cst_sequence_entry_views_spec);
        reveal(cst_mapping_entry_views_spec);
        reveal(cst_syntax_owner_record_spec);
    }
    if owner.token_index >= tokens.len() as u64 {
        return false;
    }
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, owner.token_index as int);
    }
    let token_kind = tokens[owner.token_index as usize].kind();
    match owner.kind {
        CstSyntaxOwnerKind::Directive => {
            if owner.record_index >= documents.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_document_view_at(documents@, owner.record_index as int);
            }
            let document = &documents[owner.record_index as usize];
            document.directive_start <= owner.token_index && owner.token_index
                < document.directive_end && (token_kind == CompletedTokenKind::YamlDirective
                || token_kind == CompletedTokenKind::TagDirective || token_kind
                == CompletedTokenKind::ReservedDirective)
        },
        CstSyntaxOwnerKind::DocumentStartMarker => {
            if owner.record_index >= documents.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_document_view_at(documents@, owner.record_index as int);
            }
            documents[owner.record_index as usize].explicit_start_token == Some(owner.token_index)
                && token_kind == CompletedTokenKind::DirectivesEnd
        },
        CstSyntaxOwnerKind::DocumentEndMarker => {
            if owner.record_index >= documents.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_document_view_at(documents@, owner.record_index as int);
            }
            documents[owner.record_index as usize].explicit_end_token == Some(owner.token_index)
                && token_kind == CompletedTokenKind::DocumentEnd
        },
        CstSyntaxOwnerKind::NodeProperty => {
            if owner.record_index >= nodes.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_node_view_at(nodes@, owner.record_index as int);
            }
            (nodes[owner.record_index as usize].anchor_property_token == Some(owner.token_index)
                || nodes[owner.record_index as usize].tag_property_token == Some(owner.token_index))
                && (token_kind == CompletedTokenKind::AnchorProperty || token_kind
                == CompletedTokenKind::TagProperty || token_kind
                == CompletedTokenKind::VerbatimTagProperty)
        },
        CstSyntaxOwnerKind::NodeContent => {
            if owner.record_index >= nodes.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_node_view_at(nodes@, owner.record_index as int);
            }
            nodes[owner.record_index as usize].scalar_or_alias_token == Some(owner.token_index) && (
            token_kind == CompletedTokenKind::Alias || cst_style_matches_token(
                nodes[owner.record_index as usize].style,
                token_kind,
            ))
        },
        CstSyntaxOwnerKind::NodeCollectionIndicator => {
            if owner.record_index >= nodes.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_node_view_at(nodes@, owner.record_index as int);
            }
            let node = &nodes[owner.record_index as usize];
            if node.style != CstNodeStyle::Flow || !(node.collection_start_token == Some(
                owner.token_index,
            ) || node.collection_end_token == Some(owner.token_index)) {
                return false;
            }
            if node.kind == CstNodeKind::Sequence {
                token_kind == CompletedTokenKind::FlowSequenceStart || token_kind
                    == CompletedTokenKind::FlowSequenceEnd
            } else {
                node.kind == CstNodeKind::Mapping && (token_kind
                    == CompletedTokenKind::FlowMappingStart || token_kind
                    == CompletedTokenKind::FlowMappingEnd)
            }
        },
        CstSyntaxOwnerKind::SequenceEntryIndicator => {
            if owner.record_index >= sequence_entries.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_sequence_entry_view_at(sequence_entries@, owner.record_index as int);
            }
            sequence_entries[owner.record_index as usize].indicator_token == Some(owner.token_index)
                && token_kind == CompletedTokenKind::BlockSequenceEntry
        },
        CstSyntaxOwnerKind::MappingEntryIndicator => {
            if owner.record_index >= mapping_entries.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_mapping_entry_view_at(mapping_entries@, owner.record_index as int);
            }
            (mapping_entries[owner.record_index as usize].explicit_key_token == Some(
                owner.token_index,
            ) || mapping_entries[owner.record_index as usize].mapping_value_token == Some(
                owner.token_index,
            )) && (token_kind == CompletedTokenKind::ExplicitMappingKey || token_kind
                == CompletedTokenKind::MappingValue)
        },
        CstSyntaxOwnerKind::FlowEntryIndicator => {
            if owner.record_index >= nodes.len() as u64 {
                return false;
            }
            proof {
                lemma_cst_node_view_at(nodes@, owner.record_index as int);
            }
            let node = &nodes[owner.record_index as usize];
            node.style == CstNodeStyle::Flow && (node.kind == CstNodeKind::Sequence || node.kind
                == CstNodeKind::Mapping) && node.token_start < owner.token_index
                && owner.token_index < node.token_end && token_kind == CompletedTokenKind::FlowEntry
        },
    }
}

fn cst_node_references_owned(
    node: &CstNode,
    owners: &[Option<CstSyntaxOwner>],
    index: u64,
) -> (result: bool)
    ensures
        result == cst_node_references_owned_spec(
            node@,
            cst_syntax_owner_views_spec(owners@),
            index,
        ),
{
    if let Some(token) = node.anchor_property_token {
        if !cst_owner_slot_matches(owners, token, CstSyntaxOwnerKind::NodeProperty, index) {
            return false;
        }
    }
    if let Some(token) = node.tag_property_token {
        if !cst_owner_slot_matches(owners, token, CstSyntaxOwnerKind::NodeProperty, index) {
            return false;
        }
    }
    if let Some(token) = node.scalar_or_alias_token {
        if !cst_owner_slot_matches(owners, token, CstSyntaxOwnerKind::NodeContent, index) {
            return false;
        }
    }
    if let Some(token) = node.collection_start_token {
        if !cst_owner_slot_matches(
            owners,
            token,
            CstSyntaxOwnerKind::NodeCollectionIndicator,
            index,
        ) {
            return false;
        }
    }
    if let Some(token) = node.collection_end_token {
        if !cst_owner_slot_matches(
            owners,
            token,
            CstSyntaxOwnerKind::NodeCollectionIndicator,
            index,
        ) {
            return false;
        }
    }
    proof {
        reveal(cst_node_references_owned_spec);
    }
    true
}

fn cst_sequence_entry_reference_owned(
    entry: &CstSequenceEntry,
    owners: &[Option<CstSyntaxOwner>],
    index: u64,
) -> (result: bool)
    ensures
        result == cst_sequence_entry_reference_owned_spec(
            entry@,
            cst_syntax_owner_views_spec(owners@),
            index,
        ),
{
    match entry.indicator_token {
        Some(token) => cst_owner_slot_matches(
            owners,
            token,
            CstSyntaxOwnerKind::SequenceEntryIndicator,
            index,
        ),
        None => true,
    }
}

fn cst_mapping_entry_references_owned(
    entry: &CstMappingEntry,
    owners: &[Option<CstSyntaxOwner>],
    index: u64,
) -> (result: bool)
    ensures
        result == cst_mapping_entry_references_owned_spec(
            entry@,
            cst_syntax_owner_views_spec(owners@),
            index,
        ),
{
    if let Some(token) = entry.explicit_key_token {
        if !cst_owner_slot_matches(
            owners,
            token,
            CstSyntaxOwnerKind::MappingEntryIndicator,
            index,
        ) {
            return false;
        }
    }
    if let Some(token) = entry.mapping_value_token {
        if !cst_owner_slot_matches(
            owners,
            token,
            CstSyntaxOwnerKind::MappingEntryIndicator,
            index,
        ) {
            return false;
        }
    }
    proof {
        reveal(cst_mapping_entry_references_owned_spec);
    }
    true
}

fn cst_exact_syntax_ownership(
    tokens: &[CompletedToken],
    documents: &[CstDocument],
    nodes: &[CstNode],
    sequence_entries: &[CstSequenceEntry],
    mapping_entries: &[CstMappingEntry],
    owners: &[Option<CstSyntaxOwner>],
) -> (result: bool)
    ensures
        result == cst_exact_syntax_ownership_spec(
            crate::token::completed_token_views_spec(tokens@),
            cst_document_views_spec(documents@),
            cst_node_views_spec(nodes@),
            cst_sequence_entry_views_spec(sequence_entries@),
            cst_mapping_entry_views_spec(mapping_entries@),
            cst_syntax_owner_views_spec(owners@),
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost document_views = cst_document_views_spec(documents@);
    let ghost node_views = cst_node_views_spec(nodes@);
    let ghost sequence_views = cst_sequence_entry_views_spec(sequence_entries@);
    let ghost mapping_views = cst_mapping_entry_views_spec(mapping_entries@);
    let ghost owner_views = cst_syntax_owner_views_spec(owners@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(cst_document_views_spec);
        reveal(cst_node_views_spec);
        reveal(cst_sequence_entry_views_spec);
        reveal(cst_mapping_entry_views_spec);
        reveal(cst_syntax_owner_views_spec);
    }
    if owners.len() != tokens.len() {
        proof {
            reveal(cst_owner_slots_valid_from_spec);
            assert(!cst_owner_slots_valid_from_spec(
                token_views,
                document_views,
                node_views,
                sequence_views,
                mapping_views,
                owner_views,
                0,
                (token_views.len() + 1) as nat,
            ));
            reveal(cst_exact_syntax_ownership_spec);
        }
        return false;
    }
    let ghost slots_expected = cst_owner_slots_valid_from_spec(
        token_views,
        document_views,
        node_views,
        sequence_views,
        mapping_views,
        owner_views,
        0,
        (token_views.len() + 1) as nat,
    );
    let mut token_index = 0usize;
    let ghost mut token_fuel: nat = (token_views.len() + 1) as nat;
    while token_index < tokens.len()
        invariant
            token_index <= tokens.len(),
            owners.len() == tokens.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            document_views == cst_document_views_spec(documents@),
            node_views == cst_node_views_spec(nodes@),
            sequence_views == cst_sequence_entry_views_spec(sequence_entries@),
            mapping_views == cst_mapping_entry_views_spec(mapping_entries@),
            owner_views == cst_syntax_owner_views_spec(owners@),
            token_views.len() == tokens.len(),
            owner_views.len() == owners.len(),
            slots_expected == cst_owner_slots_valid_from_spec(
                token_views,
                document_views,
                node_views,
                sequence_views,
                mapping_views,
                owner_views,
                0,
                (token_views.len() + 1) as nat,
            ),
            token_fuel >= token_views.len() - token_index + 1,
            slots_expected == cst_owner_slots_valid_from_spec(
                token_views,
                document_views,
                node_views,
                sequence_views,
                mapping_views,
                owner_views,
                token_index as int,
                token_fuel,
            ),
        decreases tokens.len() - token_index,
    {
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, token_index as int);
            lemma_cst_syntax_owner_view_at(owners@, token_index as int);
        }
        let current_kind = tokens[token_index].kind();
        proof {
            assert(current_kind == token_views[token_index as int].kind);
        }
        if is_trivia(current_kind) {
            if owners[token_index].is_some() {
                proof {
                    assert(cst_token_is_trivia_spec(token_views[token_index as int].kind));
                    assert(owner_views[token_index as int].is_some());
                    reveal(cst_owner_slots_valid_from_spec);
                    assert(!slots_expected);
                    reveal(cst_exact_syntax_ownership_spec);
                }
                return false;
            }
            proof {
                assert(owner_views[token_index as int].is_none());
                assert(cst_token_is_trivia_spec(token_views[token_index as int].kind));
                reveal(cst_owner_slots_valid_from_spec);
                token_fuel = (token_fuel - 1) as nat;
            }
            token_index += 1;
            continue;
        }
        let owner = match owners[token_index] {
            Some(owner) => owner,
            None => {
                proof {
                    assert(!cst_token_is_trivia_spec(token_views[token_index as int].kind));
                    assert(owner_views[token_index as int].is_none());
                    reveal(cst_owner_slots_valid_from_spec);
                    assert(!slots_expected);
                    reveal(cst_exact_syntax_ownership_spec);
                }
                return false;
            },
        };
        proof {
            lemma_cst_syntax_owner_view_fields(owner);
            assert(owner_views[token_index as int] == Some(owner@));
            assert(owner_views[token_index as int].unwrap() == owner@);
        }
        if owner.token_index != token_index as u64 {
            proof {
                assert(!cst_token_is_trivia_spec(token_views[token_index as int].kind));
                assert(owner@.token_index != token_index as int);
                reveal(cst_owner_slots_valid_from_spec);
                assert(!slots_expected);
                reveal(cst_exact_syntax_ownership_spec);
            }
            return false;
        }
        let owner_record_valid = cst_syntax_owner_record_is_valid(
            tokens,
            documents,
            nodes,
            sequence_entries,
            mapping_entries,
            owner,
        );
        proof {
            assert(owner_record_valid == cst_syntax_owner_record_spec(
                token_views,
                document_views,
                node_views,
                sequence_views,
                mapping_views,
                owner@,
            ));
        }
        if !owner_record_valid {
            proof {
                assert(!cst_token_is_trivia_spec(token_views[token_index as int].kind));
                assert(!cst_syntax_owner_record_spec(
                    token_views,
                    document_views,
                    node_views,
                    sequence_views,
                    mapping_views,
                    owner@,
                ));
                reveal(cst_owner_slots_valid_from_spec);
                assert(!slots_expected);
                reveal(cst_exact_syntax_ownership_spec);
            }
            return false;
        }
        proof {
            assert(!cst_token_is_trivia_spec(token_views[token_index as int].kind));
            assert(owner_views[token_index as int].is_some());
            assert(owner_views[token_index as int].unwrap().token_index == token_index as int);
            assert(cst_syntax_owner_record_spec(
                token_views,
                document_views,
                node_views,
                sequence_views,
                mapping_views,
                owner_views[token_index as int].unwrap(),
            ));
            reveal(cst_owner_slots_valid_from_spec);
            token_fuel = (token_fuel - 1) as nat;
        }
        token_index += 1;
    }
    proof {
        reveal(cst_owner_slots_valid_from_spec);
        assert(slots_expected);
    }
    let ghost documents_expected = cst_document_references_owned_from_spec(
        document_views,
        owner_views,
        0,
        (document_views.len() + 1) as nat,
    );
    let mut document_index = 0usize;
    let ghost mut document_fuel: nat = (document_views.len() + 1) as nat;
    while document_index < documents.len()
        invariant
            document_index <= documents.len(),
            document_views == cst_document_views_spec(documents@),
            document_views.len() == documents.len(),
            owner_views == cst_syntax_owner_views_spec(owners@),
            slots_expected,
            documents_expected == cst_document_references_owned_from_spec(
                document_views,
                owner_views,
                0,
                (document_views.len() + 1) as nat,
            ),
            document_fuel >= document_views.len() - document_index + 1,
            documents_expected == cst_document_references_owned_from_spec(
                document_views,
                owner_views,
                document_index as int,
                document_fuel,
            ),
        decreases documents.len() - document_index,
    {
        proof {
            lemma_cst_document_view_at(documents@, document_index as int);
        }
        let document = documents[document_index];
        proof {
            lemma_cst_document_view_fields(document);
            assert(document_views[document_index as int] == document@);
        }
        if let Some(token) = document.explicit_start_token {
            if !cst_owner_slot_matches(
                owners,
                token,
                CstSyntaxOwnerKind::DocumentStartMarker,
                document_index as u64,
            ) {
                proof {
                    assert(!cst_document_references_owned_spec(
                        document_views[document_index as int],
                        owner_views,
                        document_index as u64,
                    ));
                    reveal(cst_document_references_owned_spec);
                    reveal(cst_document_references_owned_from_spec);
                    assert(!documents_expected);
                    reveal(cst_references_have_exact_owners_spec);
                    assert(!cst_references_have_exact_owners_spec(
                        document_views,
                        node_views,
                        sequence_views,
                        mapping_views,
                        owner_views,
                    ));
                    reveal(cst_exact_syntax_ownership_spec);
                }
                return false;
            }
        }
        if let Some(token) = document.explicit_end_token {
            if !cst_owner_slot_matches(
                owners,
                token,
                CstSyntaxOwnerKind::DocumentEndMarker,
                document_index as u64,
            ) {
                proof {
                    assert(!cst_document_references_owned_spec(
                        document_views[document_index as int],
                        owner_views,
                        document_index as u64,
                    ));
                    reveal(cst_document_references_owned_spec);
                    reveal(cst_document_references_owned_from_spec);
                    assert(!documents_expected);
                    reveal(cst_references_have_exact_owners_spec);
                    assert(!cst_references_have_exact_owners_spec(
                        document_views,
                        node_views,
                        sequence_views,
                        mapping_views,
                        owner_views,
                    ));
                    reveal(cst_exact_syntax_ownership_spec);
                }
                return false;
            }
        }
        proof {
            reveal(cst_document_references_owned_spec);
            assert(cst_document_references_owned_spec(
                document_views[document_index as int],
                owner_views,
                document_index as u64,
            ));
            reveal(cst_document_references_owned_from_spec);
            document_fuel = (document_fuel - 1) as nat;
        }
        document_index += 1;
    }
    proof {
        reveal(cst_document_references_owned_from_spec);
        assert(documents_expected);
    }
    let ghost nodes_expected = cst_node_references_owned_from_spec(
        node_views,
        owner_views,
        0,
        (node_views.len() + 1) as nat,
    );
    let mut node_index = 0usize;
    let ghost mut node_fuel: nat = (node_views.len() + 1) as nat;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            node_views == cst_node_views_spec(nodes@),
            node_views.len() == nodes.len(),
            owner_views == cst_syntax_owner_views_spec(owners@),
            slots_expected,
            documents_expected,
            nodes_expected == cst_node_references_owned_from_spec(
                node_views,
                owner_views,
                0,
                (node_views.len() + 1) as nat,
            ),
            node_fuel >= node_views.len() - node_index + 1,
            nodes_expected == cst_node_references_owned_from_spec(
                node_views,
                owner_views,
                node_index as int,
                node_fuel,
            ),
        decreases nodes.len() - node_index,
    {
        let node = &nodes[node_index];
        proof {
            lemma_cst_node_view_at(nodes@, node_index as int);
        }
        if !cst_node_references_owned(node, owners, node_index as u64) {
            proof {
                assert(!cst_node_references_owned_spec(
                    node_views[node_index as int],
                    owner_views,
                    node_index as u64,
                ));
                reveal(cst_node_references_owned_from_spec);
                assert(!nodes_expected);
                reveal(cst_references_have_exact_owners_spec);
                assert(!cst_references_have_exact_owners_spec(
                    document_views,
                    node_views,
                    sequence_views,
                    mapping_views,
                    owner_views,
                ));
                reveal(cst_exact_syntax_ownership_spec);
            }
            return false;
        }
        proof {
            assert(cst_node_references_owned_spec(
                node_views[node_index as int],
                owner_views,
                node_index as u64,
            ));
            reveal(cst_node_references_owned_from_spec);
            node_fuel = (node_fuel - 1) as nat;
        }
        node_index += 1;
    }
    proof {
        reveal(cst_node_references_owned_from_spec);
        assert(nodes_expected);
    }
    let ghost sequences_expected = cst_sequence_references_owned_from_spec(
        sequence_views,
        owner_views,
        0,
        (sequence_views.len() + 1) as nat,
    );
    let mut sequence_index = 0usize;
    let ghost mut sequence_fuel: nat = (sequence_views.len() + 1) as nat;
    while sequence_index < sequence_entries.len()
        invariant
            sequence_index <= sequence_entries.len(),
            sequence_views == cst_sequence_entry_views_spec(sequence_entries@),
            sequence_views.len() == sequence_entries.len(),
            owner_views == cst_syntax_owner_views_spec(owners@),
            slots_expected,
            documents_expected,
            nodes_expected,
            sequences_expected == cst_sequence_references_owned_from_spec(
                sequence_views,
                owner_views,
                0,
                (sequence_views.len() + 1) as nat,
            ),
            sequence_fuel >= sequence_views.len() - sequence_index + 1,
            sequences_expected == cst_sequence_references_owned_from_spec(
                sequence_views,
                owner_views,
                sequence_index as int,
                sequence_fuel,
            ),
        decreases sequence_entries.len() - sequence_index,
    {
        proof {
            lemma_cst_sequence_entry_view_at(sequence_entries@, sequence_index as int);
        }
        if !cst_sequence_entry_reference_owned(
            &sequence_entries[sequence_index],
            owners,
            sequence_index as u64,
        ) {
            proof {
                assert(!cst_sequence_entry_reference_owned_spec(
                    sequence_views[sequence_index as int],
                    owner_views,
                    sequence_index as u64,
                ));
                reveal(cst_sequence_references_owned_from_spec);
                assert(!sequences_expected);
                reveal(cst_references_have_exact_owners_spec);
                assert(!cst_references_have_exact_owners_spec(
                    document_views,
                    node_views,
                    sequence_views,
                    mapping_views,
                    owner_views,
                ));
                reveal(cst_exact_syntax_ownership_spec);
            }
            return false;
        }
        proof {
            assert(cst_sequence_entry_reference_owned_spec(
                sequence_views[sequence_index as int],
                owner_views,
                sequence_index as u64,
            ));
            reveal(cst_sequence_references_owned_from_spec);
            sequence_fuel = (sequence_fuel - 1) as nat;
        }
        sequence_index += 1;
    }
    proof {
        reveal(cst_sequence_references_owned_from_spec);
        assert(sequences_expected);
    }
    let ghost mappings_expected = cst_mapping_references_owned_from_spec(
        mapping_views,
        owner_views,
        0,
        (mapping_views.len() + 1) as nat,
    );
    let mut mapping_index = 0usize;
    let ghost mut mapping_fuel: nat = (mapping_views.len() + 1) as nat;
    while mapping_index < mapping_entries.len()
        invariant
            mapping_index <= mapping_entries.len(),
            mapping_views == cst_mapping_entry_views_spec(mapping_entries@),
            mapping_views.len() == mapping_entries.len(),
            owner_views == cst_syntax_owner_views_spec(owners@),
            slots_expected,
            documents_expected,
            nodes_expected,
            sequences_expected,
            mappings_expected == cst_mapping_references_owned_from_spec(
                mapping_views,
                owner_views,
                0,
                (mapping_views.len() + 1) as nat,
            ),
            mapping_fuel >= mapping_views.len() - mapping_index + 1,
            mappings_expected == cst_mapping_references_owned_from_spec(
                mapping_views,
                owner_views,
                mapping_index as int,
                mapping_fuel,
            ),
        decreases mapping_entries.len() - mapping_index,
    {
        let entry = &mapping_entries[mapping_index];
        proof {
            lemma_cst_mapping_entry_view_at(mapping_entries@, mapping_index as int);
        }
        if !cst_mapping_entry_references_owned(entry, owners, mapping_index as u64) {
            proof {
                assert(!cst_mapping_entry_references_owned_spec(
                    mapping_views[mapping_index as int],
                    owner_views,
                    mapping_index as u64,
                ));
                reveal(cst_mapping_references_owned_from_spec);
                assert(!mappings_expected);
                reveal(cst_references_have_exact_owners_spec);
                assert(!cst_references_have_exact_owners_spec(
                    document_views,
                    node_views,
                    sequence_views,
                    mapping_views,
                    owner_views,
                ));
                reveal(cst_exact_syntax_ownership_spec);
            }
            return false;
        }
        proof {
            assert(cst_mapping_entry_references_owned_spec(
                mapping_views[mapping_index as int],
                owner_views,
                mapping_index as u64,
            ));
            reveal(cst_mapping_references_owned_from_spec);
            mapping_fuel = (mapping_fuel - 1) as nat;
        }
        mapping_index += 1;
    }
    proof {
        reveal(cst_mapping_references_owned_from_spec);
        assert(mappings_expected);
        reveal(cst_exact_syntax_ownership_spec);
        reveal(cst_references_have_exact_owners_spec);
    }
    true
}

struct CstBuilder {
    documents: Vec<CstDocument>,
    nodes: Vec<CstNode>,
    sequence_entries: Vec<CstSequenceEntry>,
    mapping_entries: Vec<CstMappingEntry>,
    warnings: Vec<CstWarning>,
    syntax_owner_slots: Vec<Option<CstSyntaxOwner>>,
    document_limit: u64,
    node_limit: u64,
    sequence_limit: u64,
    mapping_limit: u64,
    directive_limit: u64,
    warning_limit: u64,
    directive_count: u64,
    maximum_depth: u64,
    source_len_bytes: u64,
}

#[verifier::ext_equal]
pub struct CstBuilderView {
    pub documents: Seq<CstDocumentView>,
    pub nodes: Seq<CstNodeView>,
    pub sequence_entries: Seq<CstSequenceEntryView>,
    pub mapping_entries: Seq<CstMappingEntryView>,
    pub warnings: Seq<CstWarningView>,
    pub syntax_owner_slots: Seq<Option<CstSyntaxOwnerView>>,
    pub document_limit: u64,
    pub node_limit: u64,
    pub sequence_limit: u64,
    pub mapping_limit: u64,
    pub directive_limit: u64,
    pub warning_limit: u64,
    pub directive_count: u64,
    pub maximum_depth: u64,
    pub source_len_bytes: u64,
}

impl View for CstBuilder {
    type V = CstBuilderView;

    closed spec fn view(&self) -> CstBuilderView {
        CstBuilderView {
            documents: cst_document_views_spec(self.documents@),
            nodes: cst_node_views_spec(self.nodes@),
            sequence_entries: cst_sequence_entry_views_spec(self.sequence_entries@),
            mapping_entries: cst_mapping_entry_views_spec(self.mapping_entries@),
            warnings: cst_warning_views_spec(self.warnings@),
            syntax_owner_slots: cst_syntax_owner_views_spec(self.syntax_owner_slots@),
            document_limit: self.document_limit,
            node_limit: self.node_limit,
            sequence_limit: self.sequence_limit,
            mapping_limit: self.mapping_limit,
            directive_limit: self.directive_limit,
            warning_limit: self.warning_limit,
            directive_count: self.directive_count,
            maximum_depth: self.maximum_depth,
            source_len_bytes: self.source_len_bytes,
        }
    }
}

pub open spec fn cst_empty_builder_spec(
    token_count: nat,
    limits: CstLimitsView,
    source_len_bytes: u64,
) -> CstBuilderView {
    CstBuilderView {
        documents: Seq::empty(),
        nodes: Seq::empty(),
        sequence_entries: Seq::empty(),
        mapping_entries: Seq::empty(),
        warnings: Seq::empty(),
        syntax_owner_slots: Seq::new(token_count, |_index: int| None),
        document_limit: cst_effective_limit_spec(limits.max_documents, MAX_PROFILE1_CST_DOCUMENTS),
        node_limit: cst_effective_limit_spec(limits.max_nodes, MAX_PROFILE1_CST_NODES),
        sequence_limit: cst_effective_limit_spec(
            limits.max_sequence_entries,
            MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
        ),
        mapping_limit: cst_effective_limit_spec(
            limits.max_mapping_entries,
            MAX_PROFILE1_CST_MAPPING_ENTRIES,
        ),
        directive_limit: cst_effective_limit_spec(
            limits.max_directives,
            MAX_PROFILE1_CST_DIRECTIVES,
        ),
        warning_limit: cst_effective_limit_spec(limits.max_warnings, MAX_PROFILE1_CST_WARNINGS),
        directive_count: 0,
        maximum_depth: 0,
        source_len_bytes,
    }
}

pub open spec fn cst_claim_syntax_token_spec(
    builder: CstBuilderView,
    token_index: u64,
    kind: CstSyntaxOwnerKind,
    record_index: u64,
) -> Result<CstBuilderView, CstErrorView> {
    match cst_claim_syntax_owner_slots_spec(
        builder.syntax_owner_slots,
        token_index,
        kind,
        record_index,
    ) {
        Ok(slots) => Ok(CstBuilderView { syntax_owner_slots: slots, ..builder }),
        Err(error) => Err(error),
    }
}

pub open spec fn cst_claim_syntax_owner_slots_spec(
    slots: Seq<Option<CstSyntaxOwnerView>>,
    token_index: u64,
    kind: CstSyntaxOwnerKind,
    record_index: u64,
) -> Result<Seq<Option<CstSyntaxOwnerView>>, CstErrorView> {
    if token_index >= slots.len() || slots[token_index as int].is_some() {
        Err(CstErrorView { kind: CstErrorKind::InternalInvariantViolation, byte_offset: 0 })
    } else {
        Ok(
            slots.update(
                token_index as int,
                Some(CstSyntaxOwnerView { token_index, kind, record_index }),
            ),
        )
    }
}

impl CstBuilder {
    fn claim_syntax_token(
        &mut self,
        token_index: u64,
        kind: CstSyntaxOwnerKind,
        record_index: u64,
    ) -> (result: Result<(), CstError>)
        ensures
            final(self).documents@ == old(self).documents@,
            final(self).nodes@ == old(self).nodes@,
            final(self).sequence_entries@ == old(self).sequence_entries@,
            final(self).mapping_entries@ == old(self).mapping_entries@,
            final(self).warnings@ == old(self).warnings@,
            final(self).syntax_owner_slots.len() == old(self).syntax_owner_slots.len(),
            cst_claim_syntax_owner_slots_spec(
                cst_syntax_owner_views_spec(old(self).syntax_owner_slots@),
                token_index,
                kind,
                record_index,
            ) == match result {
                Ok(()) => Ok(cst_syntax_owner_views_spec(final(self).syntax_owner_slots@)),
                Err(error) => Err(error@),
            },
            final(self).document_limit == old(self).document_limit,
            final(self).node_limit == old(self).node_limit,
            final(self).sequence_limit == old(self).sequence_limit,
            final(self).mapping_limit == old(self).mapping_limit,
            final(self).directive_limit == old(self).directive_limit,
            final(self).warning_limit == old(self).warning_limit,
            final(self).directive_count == old(self).directive_count,
            final(self).maximum_depth == old(self).maximum_depth,
            final(self).source_len_bytes == old(self).source_len_bytes,
    {
        let ghost old_slots = self.syntax_owner_slots@;
        if token_index >= self.syntax_owner_slots.len() as u64
            || self.syntax_owner_slots[token_index as usize].is_some() {
            proof {
                reveal(cst_claim_syntax_owner_slots_spec);
            }
            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
        }
        let owner = CstSyntaxOwner { token_index, kind, record_index };
        self.syntax_owner_slots[token_index as usize] = Some(owner);
        proof {
            lemma_cst_syntax_owner_views_update(old_slots, token_index as int, owner);
            reveal(cst_claim_syntax_owner_slots_spec);
            reveal(cst_syntax_owner_views_spec);
        }
        Ok(())
    }

    fn push_node(&mut self, node: CstNode, offset: u64) -> (result: Result<u64, CstError>)
        ensures
            result.is_ok() ==> result.unwrap() < final(self).nodes.len(),
            final(self).syntax_owner_slots.len() == old(self).syntax_owner_slots.len(),
    {
        if self.nodes.len() as u64 >= self.node_limit {
            return Err(CstError::at(CstErrorKind::NodeLimitExceeded, offset));
        }
        let index = self.nodes.len() as u64;
        self.nodes.push(node);
        if let Some(token) = node.anchor_property_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::NodeProperty, index)?;
        }
        if let Some(token) = node.tag_property_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::NodeProperty, index)?;
        }
        if let Some(token) = node.scalar_or_alias_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::NodeContent, index)?;
        }
        if let Some(token) = node.collection_start_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::NodeCollectionIndicator, index)?;
        }
        if let Some(token) = node.collection_end_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::NodeCollectionIndicator, index)?;
        }
        Ok(index)
    }

    fn push_sequence_entry(&mut self, entry: CstSequenceEntry, offset: u64) -> (result: Result<
        (),
        CstError,
    >)
        ensures
            final(self).syntax_owner_slots.len() == old(self).syntax_owner_slots.len(),
    {
        if self.sequence_entries.len() as u64 >= self.sequence_limit {
            return Err(CstError::at(CstErrorKind::SequenceEntryLimitExceeded, offset));
        }
        let index = self.sequence_entries.len() as u64;
        self.sequence_entries.push(entry);
        if let Some(token) = entry.indicator_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::SequenceEntryIndicator, index)?;
        }
        Ok(())
    }

    fn push_mapping_entry(&mut self, entry: CstMappingEntry, offset: u64) -> (result: Result<
        (),
        CstError,
    >)
        ensures
            final(self).syntax_owner_slots.len() == old(self).syntax_owner_slots.len(),
    {
        if self.mapping_entries.len() as u64 >= self.mapping_limit {
            return Err(CstError::at(CstErrorKind::MappingEntryLimitExceeded, offset));
        }
        let index = self.mapping_entries.len() as u64;
        self.mapping_entries.push(entry);
        if let Some(token) = entry.explicit_key_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::MappingEntryIndicator, index)?;
        }
        if let Some(token) = entry.mapping_value_token {
            self.claim_syntax_token(token, CstSyntaxOwnerKind::MappingEntryIndicator, index)?;
        }
        Ok(())
    }

    fn push_warning(&mut self, warning: CstWarning) -> (result: Result<(), CstError>)
        ensures
            final(self).syntax_owner_slots.len() == old(self).syntax_owner_slots.len(),
    {
        if self.warnings.len() as u64 >= self.warning_limit {
            return Err(CstError::at(CstErrorKind::WarningLimitExceeded, warning.byte_offset));
        }
        self.warnings.push(warning);
        Ok(())
    }

    fn push_document(&mut self, document: CstDocument) -> (result: Result<(), CstError>)
        ensures
            final(self).syntax_owner_slots.len() == old(self).syntax_owner_slots.len(),
    {
        if self.documents.len() as u64 >= self.document_limit {
            return Err(CstError::at(CstErrorKind::DocumentLimitExceeded, document.byte_start));
        }
        self.documents.push(document);
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ParsedNode {
    node_index: u64,
    next_token: usize,
}

pub open spec fn cst_token_is_trivia_spec(kind: CompletedTokenKind) -> bool {
    kind == CompletedTokenKind::Indentation || kind == CompletedTokenKind::Separation || kind
        == CompletedTokenKind::Comment || kind == CompletedTokenKind::LineFeed || kind
        == CompletedTokenKind::DocumentByteOrderMark
}

pub open spec fn cst_skip_trivia_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    index: int,
    end: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || end < index || end > tokens.len() || fuel == 0 || index >= end
        || !cst_token_is_trivia_spec(tokens[index].kind) {
        index
    } else {
        cst_skip_trivia_spec(tokens, index + 1, end, (fuel - 1) as nat)
    }
}

pub open spec fn cst_token_is_scalar_spec(kind: CompletedTokenKind) -> bool {
    kind == CompletedTokenKind::PlainScalar || kind == CompletedTokenKind::SingleQuotedScalar
        || kind == CompletedTokenKind::DoubleQuotedScalar || kind
        == CompletedTokenKind::LiteralBlockScalar || kind == CompletedTokenKind::FoldedBlockScalar
}

pub open spec fn cst_scalar_style_spec(kind: CompletedTokenKind) -> CstNodeStyle {
    if kind == CompletedTokenKind::PlainScalar {
        CstNodeStyle::Plain
    } else if kind == CompletedTokenKind::SingleQuotedScalar {
        CstNodeStyle::SingleQuoted
    } else if kind == CompletedTokenKind::DoubleQuotedScalar {
        CstNodeStyle::DoubleQuoted
    } else if kind == CompletedTokenKind::LiteralBlockScalar {
        CstNodeStyle::Literal
    } else {
        CstNodeStyle::Folded
    }
}

pub open spec fn cst_token_column_spec(
    atoms: Seq<crate::atom::LexicalAtomView>,
    token: crate::token::CompletedTokenView,
) -> u64 {
    if token.start_atom_index < atoms.len() {
        atoms[token.start_atom_index as int].span.start.column
    } else {
        0
    }
}

pub open spec fn cst_same_line_spec(
    left: crate::token::CompletedTokenView,
    right: crate::token::CompletedTokenView,
) -> bool {
    left.start_line_number == right.start_line_number
}

fn is_trivia(kind: CompletedTokenKind) -> (result: bool)
    ensures
        result == cst_token_is_trivia_spec(kind),
{
    kind == CompletedTokenKind::Indentation || kind == CompletedTokenKind::Separation || kind
        == CompletedTokenKind::Comment || kind == CompletedTokenKind::LineFeed || kind
        == CompletedTokenKind::DocumentByteOrderMark
}

fn skip_trivia(tokens: &[CompletedToken], index: usize, end: usize) -> (result: usize)
    requires
        index <= end <= tokens.len(),
    ensures
        index <= result <= end,
        result as int == cst_skip_trivia_spec(
            crate::token::completed_token_views_spec(tokens@),
            index as int,
            end as int,
            (end - index) as nat + 1,
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
    }
    let ghost expected = cst_skip_trivia_spec(
        token_views,
        index as int,
        end as int,
        (end - index) as nat + 1,
    );
    let mut cursor = index;
    let ghost mut fuel: nat = (end - index) as nat + 1;
    while cursor < end && is_trivia(tokens[cursor].kind())
        invariant
            index <= cursor,
            cursor <= end,
            end <= tokens.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            token_views.len() == tokens.len(),
            fuel >= end - cursor + 1,
            expected == cst_skip_trivia_spec(token_views, cursor as int, end as int, fuel),
        decreases end - cursor,
    {
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, cursor as int);
            assert(token_views[cursor as int] == tokens@[cursor as int]@);
            assert(cst_token_is_trivia_spec(tokens@[cursor as int]@.kind));
            assert(cst_token_is_trivia_spec(token_views[cursor as int].kind));
            assert(fuel > 0);
            reveal(cst_skip_trivia_spec);
            assert(expected == cst_skip_trivia_spec(
                token_views,
                cursor as int + 1,
                end as int,
                (fuel - 1) as nat,
            ));
            fuel = (fuel - 1) as nat;
        }
        cursor += 1;
        proof {
            assert(expected == cst_skip_trivia_spec(token_views, cursor as int, end as int, fuel));
        }
    }
    proof {
        if cursor < end {
            crate::token::lemma_completed_token_view_at(tokens@, cursor as int);
        }
        reveal(cst_skip_trivia_spec);
    }
    cursor
}

fn byte_at(tokens: &[CompletedToken], index: usize, source_len_bytes: u64) -> (result: u64)
    ensures
        result == cst_byte_at_spec(
            crate::token::completed_token_views_spec(tokens@),
            index as u64,
            source_len_bytes,
        ),
{
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
    }
    if index < tokens.len() {
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, index as int);
        }
        tokens[index].byte_start()
    } else {
        source_len_bytes
    }
}

fn token_column(atoms: &[LexicalAtom], token: &CompletedToken) -> (result: u64)
    ensures
        result == cst_token_column_spec(crate::atom::lexical_atom_views_spec(atoms@), token@),
{
    let atom_index = token.start_atom_index();
    if atom_index < atoms.len() as u64 {
        let index = atom_index as usize;
        proof {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        atoms[index].span().start().column()
    } else {
        proof {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        0
    }
}

fn same_line(left: &CompletedToken, right: &CompletedToken) -> (result: bool)
    ensures
        result == cst_same_line_spec(left@, right@),
{
    left.start_line_number() == right.start_line_number()
}

fn token_is_scalar(kind: CompletedTokenKind) -> (result: bool)
    ensures
        result == cst_token_is_scalar_spec(kind),
{
    kind == CompletedTokenKind::PlainScalar || kind == CompletedTokenKind::SingleQuotedScalar
        || kind == CompletedTokenKind::DoubleQuotedScalar || kind
        == CompletedTokenKind::LiteralBlockScalar || kind == CompletedTokenKind::FoldedBlockScalar
}

fn scalar_style(kind: CompletedTokenKind) -> (result: CstNodeStyle)
    ensures
        result == cst_scalar_style_spec(kind),
{
    if kind == CompletedTokenKind::PlainScalar {
        CstNodeStyle::Plain
    } else if kind == CompletedTokenKind::SingleQuotedScalar {
        CstNodeStyle::SingleQuoted
    } else if kind == CompletedTokenKind::DoubleQuotedScalar {
        CstNodeStyle::DoubleQuoted
    } else if kind == CompletedTokenKind::LiteralBlockScalar {
        CstNodeStyle::Literal
    } else {
        CstNodeStyle::Folded
    }
}

fn part_of_kind<'a>(token: &'a CompletedToken, kind: CompletedTokenPartKind) -> Option<
    &'a CompletedTokenPart,
> {
    let parts = token.parts();
    let mut index = 0usize;
    while index < parts.len()
        invariant
            index <= parts.len(),
        decreases parts.len() - index,
    {
        if parts[index].kind() == kind {
            return Some(&parts[index]);
        }
        index += 1;
    }
    None
}

fn atom_ranges_equal(
    atoms: &[LexicalAtom],
    left_start: u64,
    left_end: u64,
    right_start: u64,
    right_end: u64,
) -> bool {
    if left_end < left_start || right_end < right_start || left_end - left_start != right_end
        - right_start || left_end > atoms.len() as u64 || right_end > atoms.len() as u64 {
        return false;
    }
    let mut offset = 0u64;
    while offset < left_end - left_start
        invariant
            offset <= left_end - left_start,
            left_end <= atoms.len(),
            right_end <= atoms.len(),
            left_end - left_start == right_end - right_start,
        decreases left_end - left_start - offset,
    {
        if atoms[(left_start + offset) as usize].code_point() != atoms[(right_start
            + offset) as usize].code_point() {
            return false;
        }
        offset += 1;
    }
    true
}

fn tag_handle_is_default(atoms: &[LexicalAtom], part: &CompletedTokenPart) -> bool {
    let start = part.start_atom_index() as usize;
    let end = part.end_atom_index() as usize;
    if start >= end || end > atoms.len() {
        return false;
    }
    end == start + 1 || end == start + 2 && atoms[start + 1].code_point() == 0x21
}

fn first_undeclared_tag_handle(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    start: usize,
    end: usize,
    handles: &[(u64, u64)],
) -> (result: Option<usize>)
    requires
        start <= end <= tokens.len(),
    ensures
        result.is_some() ==> start <= result.unwrap() < end,
{
    let mut index = start;
    while index < end
        invariant
            start <= index <= end,
            end <= tokens.len(),
        decreases end - index,
    {
        if tokens[index].kind() == CompletedTokenKind::TagProperty {
            if let Some(handle) = part_of_kind(&tokens[index], CompletedTokenPartKind::TagHandle) {
                if !tag_handle_is_default(atoms, handle) {
                    let mut declared = false;
                    let mut handle_index = 0usize;
                    while handle_index < handles.len()
                        invariant
                            handle_index <= handles.len(),
                        decreases handles.len() - handle_index,
                    {
                        if atom_ranges_equal(
                            atoms,
                            handles[handle_index].0,
                            handles[handle_index].1,
                            handle.start_atom_index(),
                            handle.end_atom_index(),
                        ) {
                            declared = true;
                            break;
                        }
                        handle_index += 1;
                    }
                    if !declared {
                        return Some(index);
                    }
                }
            }
        }
        index += 1;
    }
    None
}

fn empty_node(builder: &mut CstBuilder, tokens: &[CompletedToken], anchor: usize) -> (result:
    Result<u64, CstError>)
    requires
        anchor <= tokens.len(),
    ensures
        result.is_ok() ==> result.unwrap() < final(builder).nodes.len(),
        final(builder).syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
{
    let byte = byte_at(tokens, anchor, builder.source_len_bytes);
    builder.push_node(
        CstNode {
            kind: CstNodeKind::Empty,
            style: CstNodeStyle::Empty,
            token_start: anchor as u64,
            token_end: anchor as u64,
            byte_start: byte,
            byte_end: byte,
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start: 0,
            entry_end: 0,
            empty_anchor_token: Some(anchor as u64),
            empty_anchor_byte: Some(byte),
        },
        byte,
    )
}

fn single_pair_mapping(
    tokens: &[CompletedToken],
    key: u64,
    value: u64,
    token_start: usize,
    token_end: usize,
    explicit_key_token: Option<u64>,
    mapping_value_token: Option<u64>,
    builder: &mut CstBuilder,
) -> (result: Result<u64, CstError>)
    requires
        token_start <= token_end <= tokens.len(),
    ensures
        result.is_ok() ==> result.unwrap() < final(builder).nodes.len(),
        final(builder).syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
{
    let entry_start = builder.mapping_entries.len() as u64;
    let entry = CstMappingEntry {
        key_node_index: key,
        value_node_index: value,
        token_start: token_start as u64,
        token_end: token_end as u64,
        explicit_key_token,
        mapping_value_token,
    };
    let offset = byte_at(tokens, token_start, builder.source_len_bytes);
    match builder.push_mapping_entry(entry, offset) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    builder.push_node(
        CstNode {
            kind: CstNodeKind::Mapping,
            style: CstNodeStyle::FlowPair,
            token_start: token_start as u64,
            token_end: token_end as u64,
            byte_start: offset,
            byte_end: if token_end > token_start {
                tokens[token_end - 1].byte_end()
            } else {
                offset
            },
            anchor_property_token: None,
            tag_property_token: None,
            scalar_or_alias_token: None,
            collection_start_token: None,
            collection_end_token: None,
            entry_start,
            entry_end: builder.mapping_entries.len() as u64,
            empty_anchor_token: None,
            empty_anchor_byte: None,
        },
        offset,
    )
}

pub open spec fn cst_find_mapping_value_on_line_from_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    index: int,
    end: int,
    line: u64,
    flow_depth: u64,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index < 0 || end < index || end > tokens.len() || fuel == 0 || index >= end
        || tokens[index].start_line_number != line {
        None
    } else {
        let kind = tokens[index].kind;
        if kind == CompletedTokenKind::MappingValue && flow_depth == 0 {
            Some(index)
        } else {
            let next_depth = if kind == CompletedTokenKind::FlowSequenceStart || kind
                == CompletedTokenKind::FlowMappingStart {
                if flow_depth < u64::MAX {
                    (flow_depth + 1) as u64
                } else {
                    flow_depth
                }
            } else if kind == CompletedTokenKind::FlowSequenceEnd || kind
                == CompletedTokenKind::FlowMappingEnd {
                if flow_depth > 0 {
                    (flow_depth - 1) as u64
                } else {
                    flow_depth
                }
            } else {
                flow_depth
            };
            cst_find_mapping_value_on_line_from_spec(
                tokens,
                index + 1,
                end,
                line,
                next_depth,
                (fuel - 1) as nat,
            )
        }
    }
}

pub open spec fn cst_find_mapping_value_on_line_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    start: int,
    end: int,
    fuel: nat,
) -> Option<int> {
    if start < 0 || end < start || end > tokens.len() || start >= end {
        None
    } else {
        cst_find_mapping_value_on_line_from_spec(
            tokens,
            start,
            end,
            tokens[start].start_line_number,
            0,
            fuel,
        )
    }
}

fn find_mapping_value_on_line(tokens: &[CompletedToken], start: usize, end: usize) -> (result:
    Option<usize>)
    requires
        start <= end <= tokens.len(),
    ensures
        result.is_some() ==> start <= result.unwrap() < end,
        match result {
            Some(index) => cst_find_mapping_value_on_line_spec(
                crate::token::completed_token_views_spec(tokens@),
                start as int,
                end as int,
                (end - start) as nat + 1,
            ) == Some(index as int),
            None => cst_find_mapping_value_on_line_spec(
                crate::token::completed_token_views_spec(tokens@),
                start as int,
                end as int,
                (end - start) as nat + 1,
            ).is_none(),
        },
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
    }
    if start >= end {
        proof {
            reveal(cst_find_mapping_value_on_line_spec);
        }
        return None;
    }
    let line = tokens[start].start_line_number();
    let ghost expected = cst_find_mapping_value_on_line_spec(
        token_views,
        start as int,
        end as int,
        (end - start) as nat + 1,
    );
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, start as int);
        reveal(cst_find_mapping_value_on_line_spec);
        assert(expected == cst_find_mapping_value_on_line_from_spec(
            token_views,
            start as int,
            end as int,
            line,
            0,
            (end - start) as nat + 1,
        ));
    }
    let mut index = start;
    let mut flow_depth = 0u64;
    let ghost mut fuel: nat = (end - start) as nat + 1;
    while index < end && tokens[index].start_line_number() == line
        invariant
            start <= index,
            index <= end,
            end <= tokens.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            token_views.len() == tokens.len(),
            fuel >= end - index + 1,
            expected == cst_find_mapping_value_on_line_spec(
                token_views,
                start as int,
                end as int,
                (end - start) as nat + 1,
            ),
            expected == cst_find_mapping_value_on_line_from_spec(
                token_views,
                index as int,
                end as int,
                line,
                flow_depth,
                fuel,
            ),
        decreases end - index,
    {
        let kind = tokens[index].kind();
        if kind == CompletedTokenKind::MappingValue && flow_depth == 0 {
            proof {
                crate::token::lemma_completed_token_view_at(tokens@, index as int);
                assert(token_views[index as int] == tokens@[index as int]@);
                assert(token_views[index as int].start_line_number == line);
                assert(token_views[index as int].kind == kind);
                assert(fuel > 0);
                reveal(cst_find_mapping_value_on_line_from_spec);
                assert(expected == Some(index as int));
                assert(cst_find_mapping_value_on_line_spec(
                    token_views,
                    start as int,
                    end as int,
                    (end - start) as nat + 1,
                ) == Some(index as int));
            }
            return Some(index);
        }
        let next_flow_depth = if kind == CompletedTokenKind::FlowSequenceStart || kind
            == CompletedTokenKind::FlowMappingStart {
            if flow_depth < u64::MAX {
                flow_depth + 1
            } else {
                flow_depth
            }
        } else if kind == CompletedTokenKind::FlowSequenceEnd || kind
            == CompletedTokenKind::FlowMappingEnd {
            if flow_depth > 0 {
                flow_depth - 1
            } else {
                flow_depth
            }
        } else {
            flow_depth
        };
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, index as int);
            assert(token_views[index as int] == tokens@[index as int]@);
            assert(token_views[index as int].start_line_number == line);
            assert(token_views[index as int].kind == kind);
            assert(fuel > 0);
            reveal(cst_find_mapping_value_on_line_from_spec);
            assert(expected == cst_find_mapping_value_on_line_from_spec(
                token_views,
                index as int + 1,
                end as int,
                line,
                next_flow_depth,
                (fuel - 1) as nat,
            ));
            fuel = (fuel - 1) as nat;
        }
        flow_depth = next_flow_depth;
        index += 1;
        proof {
            assert(expected == cst_find_mapping_value_on_line_from_spec(
                token_views,
                index as int,
                end as int,
                line,
                flow_depth,
                fuel,
            ));
        }
    }
    proof {
        if index < end {
            crate::token::lemma_completed_token_view_at(tokens@, index as int);
            assert(token_views[index as int] == tokens@[index as int]@);
            assert(token_views[index as int].start_line_number != line);
        }
        assert(fuel > 0);
        reveal(cst_find_mapping_value_on_line_from_spec);
    }
    None
}

pub open spec fn cst_find_explicit_mapping_value_from_spec(
    atoms: Seq<crate::atom::LexicalAtomView>,
    tokens: Seq<crate::token::CompletedTokenView>,
    index: int,
    end: int,
    indentation: u64,
    flow_depth: u64,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index < 0 || end < index || end > tokens.len() || fuel == 0 || index >= end {
        None
    } else {
        let kind = tokens[index].kind;
        if kind == CompletedTokenKind::MappingValue && flow_depth == 0 && cst_token_column_spec(
            atoms,
            tokens[index],
        ) == indentation {
            Some(index)
        } else {
            let next_depth = if kind == CompletedTokenKind::FlowSequenceStart || kind
                == CompletedTokenKind::FlowMappingStart {
                if flow_depth < u64::MAX {
                    (flow_depth + 1) as u64
                } else {
                    flow_depth
                }
            } else if kind == CompletedTokenKind::FlowSequenceEnd || kind
                == CompletedTokenKind::FlowMappingEnd {
                if flow_depth > 0 {
                    (flow_depth - 1) as u64
                } else {
                    flow_depth
                }
            } else {
                flow_depth
            };
            cst_find_explicit_mapping_value_from_spec(
                atoms,
                tokens,
                index + 1,
                end,
                indentation,
                next_depth,
                (fuel - 1) as nat,
            )
        }
    }
}

pub open spec fn cst_find_explicit_mapping_value_spec(
    atoms: Seq<crate::atom::LexicalAtomView>,
    tokens: Seq<crate::token::CompletedTokenView>,
    start: int,
    end: int,
    indentation: u64,
    fuel: nat,
) -> Option<int> {
    if start < 0 || end < start || end > tokens.len() {
        None
    } else {
        match cst_find_mapping_value_on_line_spec(tokens, start, end, fuel) {
            Some(index) => Some(index),
            None => cst_find_explicit_mapping_value_from_spec(
                atoms,
                tokens,
                start,
                end,
                indentation,
                0,
                fuel,
            ),
        }
    }
}

fn find_explicit_mapping_value(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    start: usize,
    end: usize,
    indentation: u64,
) -> (result: Option<usize>)
    requires
        start <= end <= tokens.len(),
    ensures
        result.is_some() ==> start <= result.unwrap() < end,
        match result {
            Some(index) => cst_find_explicit_mapping_value_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::token::completed_token_views_spec(tokens@),
                start as int,
                end as int,
                indentation,
                (end - start) as nat + 1,
            ) == Some(index as int),
            None => cst_find_explicit_mapping_value_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::token::completed_token_views_spec(tokens@),
                start as int,
                end as int,
                indentation,
                (end - start) as nat + 1,
            ).is_none(),
        },
{
    let same_line_result = find_mapping_value_on_line(tokens, start, end);
    if let Some(same_line) = same_line_result {
        proof {
            reveal(cst_find_explicit_mapping_value_spec);
        }
        return Some(same_line);
    }
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    let ghost expected = cst_find_explicit_mapping_value_spec(
        atom_views,
        token_views,
        start as int,
        end as int,
        indentation,
        (end - start) as nat + 1,
    );
    proof {
        reveal(cst_find_explicit_mapping_value_spec);
        assert(cst_find_mapping_value_on_line_spec(
            token_views,
            start as int,
            end as int,
            (end - start) as nat + 1,
        ).is_none());
        assert(expected == cst_find_explicit_mapping_value_from_spec(
            atom_views,
            token_views,
            start as int,
            end as int,
            indentation,
            0,
            (end - start) as nat + 1,
        ));
    }
    let mut index = start;
    let mut flow_depth = 0u64;
    let ghost mut fuel: nat = (end - start) as nat + 1;
    while index < end
        invariant
            start <= index <= end,
            end <= tokens.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            token_views.len() == tokens.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            atom_views.len() == atoms.len(),
            fuel >= end - index + 1,
            expected == cst_find_explicit_mapping_value_spec(
                atom_views,
                token_views,
                start as int,
                end as int,
                indentation,
                (end - start) as nat + 1,
            ),
            expected == cst_find_explicit_mapping_value_from_spec(
                atom_views,
                token_views,
                index as int,
                end as int,
                indentation,
                flow_depth,
                fuel,
            ),
        decreases end - index,
    {
        let kind = tokens[index].kind();
        let column = token_column(atoms, &tokens[index]);
        if kind == CompletedTokenKind::MappingValue && flow_depth == 0 && column == indentation {
            proof {
                crate::token::lemma_completed_token_view_at(tokens@, index as int);
                assert(token_views[index as int] == tokens@[index as int]@);
                assert(token_views[index as int].kind == kind);
                assert(cst_token_column_spec(atom_views, token_views[index as int]) == column);
                assert(fuel > 0);
                reveal(cst_find_explicit_mapping_value_from_spec);
                assert(expected == Some(index as int));
            }
            return Some(index);
        }
        let next_flow_depth = if kind == CompletedTokenKind::FlowSequenceStart || kind
            == CompletedTokenKind::FlowMappingStart {
            if flow_depth < u64::MAX {
                flow_depth + 1
            } else {
                flow_depth
            }
        } else if kind == CompletedTokenKind::FlowSequenceEnd || kind
            == CompletedTokenKind::FlowMappingEnd {
            if flow_depth > 0 {
                flow_depth - 1
            } else {
                flow_depth
            }
        } else {
            flow_depth
        };
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, index as int);
            assert(token_views[index as int] == tokens@[index as int]@);
            assert(token_views[index as int].kind == kind);
            assert(cst_token_column_spec(atom_views, token_views[index as int]) == column);
            assert(fuel > 0);
            reveal(cst_find_explicit_mapping_value_from_spec);
            assert(expected == cst_find_explicit_mapping_value_from_spec(
                atom_views,
                token_views,
                index as int + 1,
                end as int,
                indentation,
                next_flow_depth,
                (fuel - 1) as nat,
            ));
            fuel = (fuel - 1) as nat;
        }
        flow_depth = next_flow_depth;
        index += 1;
        proof {
            assert(expected == cst_find_explicit_mapping_value_from_spec(
                atom_views,
                token_views,
                index as int,
                end as int,
                indentation,
                flow_depth,
                fuel,
            ));
        }
    }
    proof {
        assert(fuel > 0);
        reveal(cst_find_explicit_mapping_value_from_spec);
    }
    None
}

pub open spec fn cst_token_is_property_spec(kind: CompletedTokenKind) -> bool {
    kind == CompletedTokenKind::AnchorProperty || kind == CompletedTokenKind::TagProperty || kind
        == CompletedTokenKind::VerbatimTagProperty
}

fn token_is_property(kind: CompletedTokenKind) -> (result: bool)
    ensures
        result == cst_token_is_property_spec(kind),
{
    kind == CompletedTokenKind::AnchorProperty || kind == CompletedTokenKind::TagProperty || kind
        == CompletedTokenKind::VerbatimTagProperty
}

pub open spec fn cst_block_property_only_end_from_spec(
    atoms: Seq<crate::atom::LexicalAtomView>,
    tokens: Seq<crate::token::CompletedTokenView>,
    start: int,
    end: int,
    parent_indentation: u64,
    property_line: u64,
    syntax: int,
    property_end: int,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if start < 0 || property_end < start || syntax < property_end || end < syntax || end
        > tokens.len() || fuel == 0 {
        None
    } else if syntax < end && cst_token_is_property_spec(tokens[syntax].kind) {
        let next_property_end = syntax + 1;
        let next_syntax = cst_skip_trivia_spec(
            tokens,
            next_property_end,
            end,
            (end - next_property_end) as nat + 1,
        );
        cst_block_property_only_end_from_spec(
            atoms,
            tokens,
            start,
            end,
            parent_indentation,
            property_line,
            next_syntax,
            next_property_end,
            (fuel - 1) as nat,
        )
    } else if property_end <= start {
        None
    } else if syntax >= end {
        Some(property_end)
    } else {
        let indentationless_sequence = tokens[syntax].kind == CompletedTokenKind::BlockSequenceEntry
            && cst_token_column_spec(atoms, tokens[syntax]) == parent_indentation;
        if tokens[syntax].start_line_number > property_line && cst_token_column_spec(
            atoms,
            tokens[syntax],
        ) <= parent_indentation && !indentationless_sequence {
            Some(property_end)
        } else {
            None
        }
    }
}

pub open spec fn cst_block_property_only_end_spec(
    atoms: Seq<crate::atom::LexicalAtomView>,
    tokens: Seq<crate::token::CompletedTokenView>,
    start: int,
    end: int,
    parent_indentation: u64,
    fuel: nat,
) -> Option<int> {
    if start < 0 || end < start || end > tokens.len() || start >= end
        || !cst_token_is_property_spec(tokens[start].kind) {
        None
    } else {
        cst_block_property_only_end_from_spec(
            atoms,
            tokens,
            start,
            end,
            parent_indentation,
            tokens[start].start_line_number,
            start,
            start,
            fuel,
        )
    }
}

fn block_property_only_end(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    start: usize,
    end: usize,
    parent_indentation: u64,
) -> (result: Option<usize>)
    requires
        start <= end <= tokens.len(),
    ensures
        result.is_some() ==> start < result.unwrap() <= end,
        match result {
            Some(index) => cst_block_property_only_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::token::completed_token_views_spec(tokens@),
                start as int,
                end as int,
                parent_indentation,
                (end - start) as nat + 1,
            ) == Some(index as int),
            None => cst_block_property_only_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::token::completed_token_views_spec(tokens@),
                start as int,
                end as int,
                parent_indentation,
                (end - start) as nat + 1,
            ).is_none(),
        },
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    if start >= end || !token_is_property(tokens[start].kind()) {
        proof {
            if start < end {
                crate::token::lemma_completed_token_view_at(tokens@, start as int);
            }
            reveal(cst_block_property_only_end_spec);
        }
        return None;
    }
    let property_line = tokens[start].start_line_number();
    let ghost expected = cst_block_property_only_end_spec(
        atom_views,
        token_views,
        start as int,
        end as int,
        parent_indentation,
        (end - start) as nat + 1,
    );
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, start as int);
        reveal(cst_block_property_only_end_spec);
        assert(expected == cst_block_property_only_end_from_spec(
            atom_views,
            token_views,
            start as int,
            end as int,
            parent_indentation,
            property_line,
            start as int,
            start as int,
            (end - start) as nat + 1,
        ));
    }
    let mut syntax = start;
    let mut property_end = start;
    let ghost mut fuel: nat = (end - start) as nat + 1;
    while syntax < end && token_is_property(tokens[syntax].kind())
        invariant
            start <= property_end <= syntax <= end,
            end <= tokens.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            token_views.len() == tokens.len(),
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            atom_views.len() == atoms.len(),
            fuel >= end - syntax + 1,
            expected == cst_block_property_only_end_spec(
                atom_views,
                token_views,
                start as int,
                end as int,
                parent_indentation,
                (end - start) as nat + 1,
            ),
            expected == cst_block_property_only_end_from_spec(
                atom_views,
                token_views,
                start as int,
                end as int,
                parent_indentation,
                property_line,
                syntax as int,
                property_end as int,
                fuel,
            ),
        decreases end - syntax,
    {
        let next_property_end = syntax + 1;
        let next_syntax = skip_trivia(tokens, next_property_end, end);
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, syntax as int);
            assert(token_views[syntax as int] == tokens@[syntax as int]@);
            assert(cst_token_is_property_spec(token_views[syntax as int].kind));
            assert(fuel > 0);
            reveal(cst_block_property_only_end_from_spec);
            assert(expected == cst_block_property_only_end_from_spec(
                atom_views,
                token_views,
                start as int,
                end as int,
                parent_indentation,
                property_line,
                next_syntax as int,
                next_property_end as int,
                (fuel - 1) as nat,
            ));
            fuel = (fuel - 1) as nat;
        }
        property_end = next_property_end;
        syntax = next_syntax;
        proof {
            assert(expected == cst_block_property_only_end_from_spec(
                atom_views,
                token_views,
                start as int,
                end as int,
                parent_indentation,
                property_line,
                syntax as int,
                property_end as int,
                fuel,
            ));
        }
    }
    proof {
        if syntax < end {
            crate::token::lemma_completed_token_view_at(tokens@, syntax as int);
            assert(token_views[syntax as int] == tokens@[syntax as int]@);
            assert(!cst_token_is_property_spec(token_views[syntax as int].kind));
        }
        assert(fuel > 0);
    }
    if property_end <= start {
        proof {
            reveal(cst_block_property_only_end_from_spec);
            assert(expected.is_none());
        }
        return None;
    }
    if syntax >= end {
        proof {
            reveal(cst_block_property_only_end_from_spec);
            assert(expected == Some(property_end as int));
        }
        return Some(property_end);
    }
    let kind = tokens[syntax].kind();
    let syntax_line = tokens[syntax].start_line_number();
    let column = token_column(atoms, &tokens[syntax]);
    let indentationless_sequence = kind == CompletedTokenKind::BlockSequenceEntry && column
        == parent_indentation;
    let result = if syntax_line > property_line && column <= parent_indentation
        && !indentationless_sequence {
        Some(property_end)
    } else {
        None
    };
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, syntax as int);
        assert(token_views[syntax as int] == tokens@[syntax as int]@);
        assert(token_views[syntax as int].kind == kind);
        assert(token_views[syntax as int].start_line_number == syntax_line);
        assert(cst_token_column_spec(atom_views, token_views[syntax as int]) == column);
        assert(fuel > 0);
        assert(!cst_token_is_property_spec(token_views[syntax as int].kind));
        reveal(cst_block_property_only_end_from_spec);
    }
    result
}

pub open spec fn cst_block_value_allows_collection_from_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    syntax: int,
    end: int,
    colon_line: u64,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if syntax < 0 || end < syntax || end > tokens.len() || fuel == 0 {
        false
    } else if syntax < end && cst_token_is_property_spec(tokens[syntax].kind) {
        let next_syntax = cst_skip_trivia_spec(
            tokens,
            syntax + 1,
            end,
            (end - (syntax + 1)) as nat + 1,
        );
        cst_block_value_allows_collection_from_spec(
            tokens,
            next_syntax,
            end,
            colon_line,
            (fuel - 1) as nat,
        )
    } else {
        syntax < end && tokens[syntax].start_line_number > colon_line
    }
}

pub open spec fn cst_block_value_allows_collection_spec(
    tokens: Seq<crate::token::CompletedTokenView>,
    start: int,
    end: int,
    colon: int,
    fuel: nat,
) -> bool {
    if start < 0 || end < start || end > tokens.len() || colon < 0 || colon >= tokens.len() {
        false
    } else {
        cst_block_value_allows_collection_from_spec(
            tokens,
            start,
            end,
            tokens[colon].start_line_number,
            fuel,
        )
    }
}

fn block_value_allows_collection(
    tokens: &[CompletedToken],
    start: usize,
    end: usize,
    colon: usize,
) -> (allowed: bool)
    requires
        start <= end <= tokens.len(),
        colon < tokens.len(),
    ensures
        allowed == cst_block_value_allows_collection_spec(
            crate::token::completed_token_views_spec(tokens@),
            start as int,
            end as int,
            colon as int,
            (end - start) as nat + 1,
        ),
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        crate::token::lemma_completed_token_view_at(tokens@, colon as int);
    }
    let colon_line = tokens[colon].start_line_number();
    let ghost expected = cst_block_value_allows_collection_spec(
        token_views,
        start as int,
        end as int,
        colon as int,
        (end - start) as nat + 1,
    );
    proof {
        reveal(cst_block_value_allows_collection_spec);
        assert(expected == cst_block_value_allows_collection_from_spec(
            token_views,
            start as int,
            end as int,
            colon_line,
            (end - start) as nat + 1,
        ));
    }
    let mut syntax = start;
    let ghost mut fuel: nat = (end - start) as nat + 1;
    while syntax < end && token_is_property(tokens[syntax].kind())
        invariant
            start <= syntax <= end,
            end <= tokens.len(),
            token_views == crate::token::completed_token_views_spec(tokens@),
            token_views.len() == tokens.len(),
            fuel >= end - syntax + 1,
            expected == cst_block_value_allows_collection_spec(
                token_views,
                start as int,
                end as int,
                colon as int,
                (end - start) as nat + 1,
            ),
            expected == cst_block_value_allows_collection_from_spec(
                token_views,
                syntax as int,
                end as int,
                colon_line,
                fuel,
            ),
        decreases end - syntax,
    {
        let next_syntax = skip_trivia(tokens, syntax + 1, end);
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, syntax as int);
            assert(token_views[syntax as int] == tokens@[syntax as int]@);
            assert(cst_token_is_property_spec(token_views[syntax as int].kind));
            assert(fuel > 0);
            reveal(cst_block_value_allows_collection_from_spec);
            assert(expected == cst_block_value_allows_collection_from_spec(
                token_views,
                next_syntax as int,
                end as int,
                colon_line,
                (fuel - 1) as nat,
            ));
            fuel = (fuel - 1) as nat;
        }
        syntax = next_syntax;
        proof {
            assert(expected == cst_block_value_allows_collection_from_spec(
                token_views,
                syntax as int,
                end as int,
                colon_line,
                fuel,
            ));
        }
    }
    proof {
        if syntax < end {
            crate::token::lemma_completed_token_view_at(tokens@, syntax as int);
            assert(token_views[syntax as int] == tokens@[syntax as int]@);
            assert(!cst_token_is_property_spec(token_views[syntax as int].kind));
        }
        assert(fuel > 0);
        reveal(cst_block_value_allows_collection_from_spec);
    }
    syntax < end && tokens[syntax].start_line_number() > colon_line
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseTaskKind {
    Node,
    FlowSequence,
    FlowMapping,
    BlockSequence,
    BlockMapping,
}

struct ParseTask {
    kind: ParseTaskKind,
    state: u8,
    token_start: usize,
    cursor: usize,
    end: usize,
    opener: usize,
    indentation: u64,
    depth_left: u64,
    allow_block_mapping: bool,
    anchor_property_token: Option<u64>,
    tag_property_token: Option<u64>,
    pending_sequence: Vec<CstSequenceEntry>,
    pending_mapping: Vec<CstMappingEntry>,
    flow_entry_tokens: Vec<u64>,
    entry_token_start: usize,
    entry_token_end: usize,
    key_node_index: u64,
    colon_token: usize,
    node_token_end: usize,
    explicit_key: bool,
}

fn node_task(start: usize, end: usize, allow_block_mapping: bool, depth_left: u64) -> ParseTask {
    ParseTask {
        kind: ParseTaskKind::Node,
        state: 0,
        token_start: start,
        cursor: start,
        end,
        opener: start,
        indentation: 0,
        depth_left,
        allow_block_mapping,
        anchor_property_token: None,
        tag_property_token: None,
        pending_sequence: Vec::new(),
        pending_mapping: Vec::new(),
        flow_entry_tokens: Vec::new(),
        entry_token_start: start,
        entry_token_end: start,
        key_node_index: 0,
        colon_token: start,
        node_token_end: start,
        explicit_key: false,
    }
}

#[allow(clippy::manual_map)]
fn finish_iterative_sequence(
    tokens: &[CompletedToken],
    task: ParseTask,
    closer: Option<usize>,
    builder: &mut CstBuilder,
) -> (result: Result<ParsedNode, CstError>)
    ensures
        final(builder).syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
{
    let entry_start = builder.sequence_entries.len() as u64;
    let mut pending_index = 0usize;
    while pending_index < task.pending_sequence.len()
        invariant
            pending_index <= task.pending_sequence.len(),
            builder.syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
        decreases task.pending_sequence.len() - pending_index,
    {
        let offset = byte_at(
            tokens,
            task.pending_sequence[pending_index].token_start as usize,
            builder.source_len_bytes,
        );
        match builder.push_sequence_entry(task.pending_sequence[pending_index], offset) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
        pending_index += 1;
    }
    let (token_end, byte_end, next_token, offset) = match closer {
        Some(index) => {
            if task.token_start >= tokens.len() || index >= tokens.len() {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
            (index + 1, tokens[index].byte_end(), index + 1, tokens[index].byte_start())
        },
        None => {
            if task.token_start >= tokens.len() || task.opener >= tokens.len() {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
            let token_end = task.node_token_end;
            let byte_end = if token_end > task.token_start && token_end <= tokens.len() {
                tokens[token_end - 1].byte_end()
            } else {
                tokens[task.opener].byte_end()
            };
            (token_end, byte_end, task.cursor, tokens[task.opener].byte_start())
        },
    };
    let node = CstNode {
        kind: CstNodeKind::Sequence,
        style: if closer.is_some() {
            CstNodeStyle::Flow
        } else {
            CstNodeStyle::Block
        },
        token_start: task.token_start as u64,
        token_end: token_end as u64,
        byte_start: tokens[task.token_start].byte_start(),
        byte_end,
        anchor_property_token: task.anchor_property_token,
        tag_property_token: task.tag_property_token,
        scalar_or_alias_token: None,
        collection_start_token: if closer.is_some() {
            Some(task.opener as u64)
        } else {
            None
        },
        collection_end_token: match closer {
            Some(index) => Some(index as u64),
            None => None,
        },
        entry_start,
        entry_end: builder.sequence_entries.len() as u64,
        empty_anchor_token: None,
        empty_anchor_byte: None,
    };
    let node_index = match builder.push_node(node, offset) {
        Ok(node_index) => node_index,
        Err(error) => return Err(error),
    };
    let mut flow_entry_index = 0usize;
    while flow_entry_index < task.flow_entry_tokens.len()
        invariant
            flow_entry_index <= task.flow_entry_tokens.len(),
            builder.syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
        decreases task.flow_entry_tokens.len() - flow_entry_index,
    {
        if let Err(error) = builder.claim_syntax_token(
            task.flow_entry_tokens[flow_entry_index],
            CstSyntaxOwnerKind::FlowEntryIndicator,
            node_index,
        ) {
            return Err(error);
        }
        flow_entry_index += 1;
    }
    Ok(ParsedNode { node_index, next_token })
}

#[allow(clippy::manual_map)]
fn finish_iterative_mapping(
    tokens: &[CompletedToken],
    task: ParseTask,
    closer: Option<usize>,
    builder: &mut CstBuilder,
) -> (result: Result<ParsedNode, CstError>)
    ensures
        final(builder).syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
{
    let entry_start = builder.mapping_entries.len() as u64;
    let mut pending_index = 0usize;
    while pending_index < task.pending_mapping.len()
        invariant
            pending_index <= task.pending_mapping.len(),
            builder.syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
        decreases task.pending_mapping.len() - pending_index,
    {
        let offset = byte_at(
            tokens,
            task.pending_mapping[pending_index].token_start as usize,
            builder.source_len_bytes,
        );
        match builder.push_mapping_entry(task.pending_mapping[pending_index], offset) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
        pending_index += 1;
    }
    let (token_end, byte_end, next_token, offset) = match closer {
        Some(index) => {
            if task.token_start >= tokens.len() || index >= tokens.len() {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
            (index + 1, tokens[index].byte_end(), index + 1, tokens[index].byte_start())
        },
        None => {
            if task.token_start >= tokens.len() || task.opener >= tokens.len() {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
            let token_end = task.node_token_end;
            let byte_end = if token_end > task.token_start && token_end <= tokens.len() {
                tokens[token_end - 1].byte_end()
            } else {
                tokens[task.opener].byte_end()
            };
            (token_end, byte_end, task.cursor, tokens[task.opener].byte_start())
        },
    };
    let node = CstNode {
        kind: CstNodeKind::Mapping,
        style: if closer.is_some() {
            CstNodeStyle::Flow
        } else {
            CstNodeStyle::Block
        },
        token_start: task.token_start as u64,
        token_end: token_end as u64,
        byte_start: tokens[task.token_start].byte_start(),
        byte_end,
        anchor_property_token: task.anchor_property_token,
        tag_property_token: task.tag_property_token,
        scalar_or_alias_token: None,
        collection_start_token: if closer.is_some() {
            Some(task.opener as u64)
        } else {
            None
        },
        collection_end_token: match closer {
            Some(index) => Some(index as u64),
            None => None,
        },
        entry_start,
        entry_end: builder.mapping_entries.len() as u64,
        empty_anchor_token: None,
        empty_anchor_byte: None,
    };
    let node_index = match builder.push_node(node, offset) {
        Ok(node_index) => node_index,
        Err(error) => return Err(error),
    };
    let mut flow_entry_index = 0usize;
    while flow_entry_index < task.flow_entry_tokens.len()
        invariant
            flow_entry_index <= task.flow_entry_tokens.len(),
            builder.syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
        decreases task.flow_entry_tokens.len() - flow_entry_index,
    {
        if let Err(error) = builder.claim_syntax_token(
            task.flow_entry_tokens[flow_entry_index],
            CstSyntaxOwnerKind::FlowEntryIndicator,
            node_index,
        ) {
            return Err(error);
        }
        flow_entry_index += 1;
    }
    Ok(ParsedNode { node_index, next_token })
}

fn push_iterative_task(
    tasks: &mut Vec<ParseTask>,
    task: ParseTask,
    depth_limit: u64,
    offset: u64,
) -> (result: Result<(), CstError>)
    requires
        depth_limit <= MAX_PROFILE1_CST_DEPTH,
    ensures
        result.is_ok() ==> final(tasks).len() == old(tasks).len() + 1,
        result.is_ok() ==> final(tasks).len() <= depth_limit + 2,
{
    if tasks.len() > depth_limit as usize + 1 {
        return Err(CstError::at(CstErrorKind::DepthLimitExceeded, offset));
    }
    tasks.push(task);
    Ok(())
}

#[verifier::rlimit(500)]
fn parse_node_iterative(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    start: usize,
    end: usize,
    allow_block_mapping: bool,
    depth_limit: u64,
    builder: &mut CstBuilder,
) -> (result: Result<ParsedNode, CstError>)
    requires
        start <= end <= tokens.len(),
        end <= MAX_PROFILE1_COMPLETED_TOKENS,
        depth_limit <= MAX_PROFILE1_CST_DEPTH,
    ensures
        result.is_ok() ==> result.unwrap().node_index < final(builder).nodes.len(),
        result.is_ok() ==> start <= result.unwrap().next_token <= end,
        final(builder).syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
{
    let mut tasks: Vec<ParseTask> = Vec::new();
    tasks.push(node_task(start, end, allow_block_mapping, depth_limit));
    let mut completed: Option<ParsedNode> = None;
    let mut fuel = (MAX_PROFILE1_COMPLETED_TOKENS + 1) * (MAX_PROFILE1_CST_DEPTH + 1) * 32;
    loop
        invariant
            depth_limit <= MAX_PROFILE1_CST_DEPTH,
            tasks.len() <= depth_limit + 2,
            builder.syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
            fuel <= (MAX_PROFILE1_COMPLETED_TOKENS + 1) * (MAX_PROFILE1_CST_DEPTH + 1) * 32,
        decreases fuel,
    {
        if tasks.len() == 0 {
            return match completed {
                Some(parsed) => {
                    if parsed.next_token < start || parsed.next_token > end || parsed.node_index
                        >= builder.nodes.len() as u64 {
                        Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0))
                    } else {
                        Ok(parsed)
                    }
                },
                None => Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
            };
        }
        if fuel == 0 {
            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
        }
        fuel -= 1;
        let mut task = match tasks.pop() {
            Some(value) => value,
            None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
        };
        if task.end > tokens.len() || task.cursor > task.end || task.token_start > task.end {
            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
        }
        if task.kind == ParseTaskKind::Node {
            let mut index = skip_trivia(tokens, task.cursor, task.end);
            if index >= task.end {
                completed =
                match empty_node(builder, tokens, index) {
                    Ok(node_index) => Some(ParsedNode { node_index, next_token: index }),
                    Err(error) => return Err(error),
                };
                continue;
            }
            let token_start = index;
            let mut anchor_property_token = None;
            let mut tag_property_token = None;
            while index < task.end && (tokens[index].kind() == CompletedTokenKind::AnchorProperty
                || tokens[index].kind() == CompletedTokenKind::TagProperty || tokens[index].kind()
                == CompletedTokenKind::VerbatimTagProperty)
                invariant
                    token_start <= index <= task.end,
                    task.end <= tokens.len(),
                    builder.syntax_owner_slots.len() == old(builder).syntax_owner_slots.len(),
                decreases task.end - index,
            {
                if tokens[index].kind() == CompletedTokenKind::AnchorProperty {
                    if anchor_property_token.is_some() {
                        return Err(
                            CstError::at(
                                CstErrorKind::DuplicateAnchorProperty,
                                tokens[index].byte_start(),
                            ),
                        );
                    }
                    anchor_property_token = Some(index as u64);
                } else {
                    if tag_property_token.is_some() {
                        return Err(
                            CstError::at(
                                CstErrorKind::DuplicateTagProperty,
                                tokens[index].byte_start(),
                            ),
                        );
                    }
                    tag_property_token = Some(index as u64);
                }
                let after_property = index + 1;
                index = skip_trivia(tokens, after_property, task.end);
                if index < task.end && index == after_property {
                    return Err(
                        CstError::at(
                            CstErrorKind::MissingPropertySeparation,
                            tokens[index].byte_start(),
                        ),
                    );
                }
            }
            if index >= task.end {
                let byte = byte_at(tokens, index, builder.source_len_bytes);
                let node = CstNode {
                    kind: CstNodeKind::Empty,
                    style: CstNodeStyle::Empty,
                    token_start: token_start as u64,
                    token_end: index as u64,
                    byte_start: tokens[token_start].byte_start(),
                    byte_end: byte,
                    anchor_property_token,
                    tag_property_token,
                    scalar_or_alias_token: None,
                    collection_start_token: None,
                    collection_end_token: None,
                    entry_start: 0,
                    entry_end: 0,
                    empty_anchor_token: Some(index as u64),
                    empty_anchor_byte: Some(byte),
                };
                completed =
                match builder.push_node(node, byte) {
                    Ok(node_index) => Some(ParsedNode { node_index, next_token: index }),
                    Err(error) => return Err(error),
                };
                continue;
            }
            let kind = tokens[index].kind();
            if kind == CompletedTokenKind::Alias {
                if anchor_property_token.is_some() || tag_property_token.is_some() {
                    return Err(
                        CstError::at(
                            CstErrorKind::AliasHasPropertiesOrContent,
                            tokens[index].byte_start(),
                        ),
                    );
                }
                let node = CstNode {
                    kind: CstNodeKind::Alias,
                    style: CstNodeStyle::Alias,
                    token_start: token_start as u64,
                    token_end: (index + 1) as u64,
                    byte_start: tokens[token_start].byte_start(),
                    byte_end: tokens[index].byte_end(),
                    anchor_property_token: None,
                    tag_property_token: None,
                    scalar_or_alias_token: Some(index as u64),
                    collection_start_token: None,
                    collection_end_token: None,
                    entry_start: 0,
                    entry_end: 0,
                    empty_anchor_token: None,
                    empty_anchor_byte: None,
                };
                completed =
                match builder.push_node(node, tokens[index].byte_start()) {
                    Ok(node_index) => Some(ParsedNode { node_index, next_token: index + 1 }),
                    Err(error) => return Err(error),
                };
                continue;
            }
            if token_is_scalar(kind) {
                let after = skip_trivia(tokens, index + 1, task.end);
                if task.allow_block_mapping && after < task.end && tokens[after].kind()
                    == CompletedTokenKind::MappingValue && same_line(
                    &tokens[index],
                    &tokens[after],
                ) {
                    if task.depth_left == 0 {
                        return Err(
                            CstError::at(
                                CstErrorKind::DepthLimitExceeded,
                                tokens[after].byte_start(),
                            ),
                        );
                    }
                    if task.depth_left > depth_limit {
                        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                    }
                    let opened_depth = depth_limit - task.depth_left + 1;
                    if opened_depth > builder.maximum_depth {
                        builder.maximum_depth = opened_depth;
                    }
                    task.kind = ParseTaskKind::BlockMapping;
                    task.state = 0;
                    task.token_start = token_start;
                    task.cursor = index;
                    task.opener = index;
                    task.indentation = token_column(atoms, &tokens[index]);
                    task.depth_left -= 1;
                    task.anchor_property_token = anchor_property_token;
                    task.tag_property_token = tag_property_token;
                    task.node_token_end = index;
                    tasks.push(task);
                    continue;
                }
                let node = CstNode {
                    kind: CstNodeKind::Scalar,
                    style: scalar_style(kind),
                    token_start: token_start as u64,
                    token_end: (index + 1) as u64,
                    byte_start: tokens[token_start].byte_start(),
                    byte_end: tokens[index].byte_end(),
                    anchor_property_token,
                    tag_property_token,
                    scalar_or_alias_token: Some(index as u64),
                    collection_start_token: None,
                    collection_end_token: None,
                    entry_start: 0,
                    entry_end: 0,
                    empty_anchor_token: None,
                    empty_anchor_byte: None,
                };
                completed =
                match builder.push_node(node, tokens[index].byte_start()) {
                    Ok(node_index) => Some(ParsedNode { node_index, next_token: index + 1 }),
                    Err(error) => return Err(error),
                };
                continue;
            }
            let specialized = if kind == CompletedTokenKind::FlowSequenceStart {
                ParseTaskKind::FlowSequence
            } else if kind == CompletedTokenKind::FlowMappingStart {
                ParseTaskKind::FlowMapping
            } else if kind == CompletedTokenKind::BlockSequenceEntry && task.allow_block_mapping {
                ParseTaskKind::BlockSequence
            } else if (kind == CompletedTokenKind::ExplicitMappingKey || kind
                == CompletedTokenKind::MappingValue) && task.allow_block_mapping {
                ParseTaskKind::BlockMapping
            } else {
                return Err(CstError::at(CstErrorKind::UnexpectedToken, tokens[index].byte_start()));
            };
            if task.depth_left == 0 {
                return Err(
                    CstError::at(CstErrorKind::DepthLimitExceeded, tokens[index].byte_start()),
                );
            }
            if task.depth_left > depth_limit {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
            let opened_depth = depth_limit - task.depth_left + 1;
            if opened_depth > builder.maximum_depth {
                builder.maximum_depth = opened_depth;
            }
            task.kind = specialized;
            task.state = 0;
            task.token_start = token_start;
            task.cursor = if specialized == ParseTaskKind::FlowSequence || specialized
                == ParseTaskKind::FlowMapping {
                skip_trivia(tokens, index + 1, task.end)
            } else {
                index
            };
            task.opener = index;
            task.indentation = token_column(atoms, &tokens[index]);
            task.depth_left -= 1;
            task.anchor_property_token = anchor_property_token;
            task.tag_property_token = tag_property_token;
            task.node_token_end = index + 1;
            tasks.push(task);
            continue;
        }
        if task.kind == ParseTaskKind::FlowSequence {
            if task.state == 0 {
                task.cursor = skip_trivia(tokens, task.cursor, task.end);
                if task.cursor >= task.end {
                    return Err(
                        CstError::at(CstErrorKind::UnexpectedEndOfInput, builder.source_len_bytes),
                    );
                }
                if tokens[task.cursor].kind() == CompletedTokenKind::FlowSequenceEnd {
                    let closer = task.cursor;
                    completed =
                    Some(
                        match finish_iterative_sequence(tokens, task, Some(closer), builder) {
                            Ok(parsed) => parsed,
                            Err(error) => return Err(error),
                        },
                    );
                    continue;
                }
                if tokens[task.cursor].kind() == CompletedTokenKind::FlowEntry {
                    return Err(
                        CstError::at(
                            CstErrorKind::UnexpectedFlowEntry,
                            tokens[task.cursor].byte_start(),
                        ),
                    );
                }
                task.entry_token_start = task.cursor;
                task.explicit_key = tokens[task.cursor].kind()
                    == CompletedTokenKind::ExplicitMappingKey;
                if task.explicit_key {
                    task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
                }
                if task.cursor >= task.end || tokens[task.cursor].kind()
                    == CompletedTokenKind::MappingValue || tokens[task.cursor].kind()
                    == CompletedTokenKind::FlowEntry || tokens[task.cursor].kind()
                    == CompletedTokenKind::FlowSequenceEnd {
                    task.key_node_index = match empty_node(builder, tokens, task.cursor) {
                        Ok(node) => node,
                        Err(error) => return Err(error),
                    };
                    task.state = 2;
                    tasks.push(task);
                } else {
                    let child_offset = tokens[task.cursor].byte_start();
                    let child = node_task(task.cursor, task.end, false, task.depth_left);
                    task.state = 1;
                    tasks.push(task);
                    if let Err(error) = push_iterative_task(
                        &mut tasks,
                        child,
                        depth_limit,
                        child_offset,
                    ) {
                        return Err(error);
                    }
                }
                continue;
            }
            if task.state == 1 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.key_node_index = child.node_index;
                task.cursor = skip_trivia(tokens, child.next_token, task.end);
                task.state = 2;
                tasks.push(task);
                continue;
            }
            if task.state == 2 {
                if task.cursor < task.end && tokens[task.cursor].kind()
                    == CompletedTokenKind::MappingValue {
                    if task.entry_token_start >= task.end {
                        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                    }
                    if !task.explicit_key && tokens[task.entry_token_start].start_line_number()
                        != tokens[task.cursor].start_line_number() {
                        return Err(
                            CstError::at(
                                CstErrorKind::MultilineImplicitKey,
                                tokens[task.cursor].byte_start(),
                            ),
                        );
                    }
                    task.colon_token = task.cursor;
                    task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
                    if task.cursor >= task.end || tokens[task.cursor].kind()
                        == CompletedTokenKind::FlowEntry || tokens[task.cursor].kind()
                        == CompletedTokenKind::FlowSequenceEnd {
                        let value = match empty_node(builder, tokens, task.cursor) {
                            Ok(node) => node,
                            Err(error) => return Err(error),
                        };
                        let pair_end = if task.cursor > task.colon_token {
                            task.cursor
                        } else {
                            if task.colon_token >= task.end {
                                return Err(
                                    CstError::at(CstErrorKind::InternalInvariantViolation, 0),
                                );
                            }
                            task.colon_token + 1
                        };
                        if task.entry_token_start > pair_end || pair_end > task.end {
                            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                        }
                        let pair = match single_pair_mapping(
                            tokens,
                            task.key_node_index,
                            value,
                            task.entry_token_start,
                            pair_end,
                            if task.explicit_key {
                                Some(task.entry_token_start as u64)
                            } else {
                                None
                            },
                            Some(task.colon_token as u64),
                            builder,
                        ) {
                            Ok(node) => node,
                            Err(error) => return Err(error),
                        };
                        task.pending_sequence.push(
                            CstSequenceEntry {
                                node_index: pair,
                                token_start: task.entry_token_start as u64,
                                token_end: task.cursor as u64,
                                indicator_token: None,
                            },
                        );
                        task.state = 4;
                        tasks.push(task);
                    } else {
                        let child_offset = tokens[task.cursor].byte_start();
                        let child = node_task(task.cursor, task.end, false, task.depth_left);
                        task.state = 3;
                        tasks.push(task);
                        if let Err(error) = push_iterative_task(
                            &mut tasks,
                            child,
                            depth_limit,
                            child_offset,
                        ) {
                            return Err(error);
                        }
                    }
                } else {
                    let entry_node = if task.explicit_key {
                        if task.entry_token_start > task.cursor {
                            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                        }
                        let value = match empty_node(builder, tokens, task.cursor) {
                            Ok(node) => node,
                            Err(error) => return Err(error),
                        };
                        match single_pair_mapping(
                            tokens,
                            task.key_node_index,
                            value,
                            task.entry_token_start,
                            task.cursor,
                            Some(task.entry_token_start as u64),
                            None,
                            builder,
                        ) {
                            Ok(node) => node,
                            Err(error) => return Err(error),
                        }
                    } else {
                        task.key_node_index
                    };
                    task.pending_sequence.push(
                        CstSequenceEntry {
                            node_index: entry_node,
                            token_start: task.entry_token_start as u64,
                            token_end: task.cursor as u64,
                            indicator_token: None,
                        },
                    );
                    task.state = 4;
                    tasks.push(task);
                }
                continue;
            }
            if task.state == 3 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.cursor = skip_trivia(tokens, child.next_token, task.end);
                if task.entry_token_start > task.cursor {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                let pair = match single_pair_mapping(
                    tokens,
                    task.key_node_index,
                    child.node_index,
                    task.entry_token_start,
                    task.cursor,
                    if task.explicit_key {
                        Some(task.entry_token_start as u64)
                    } else {
                        None
                    },
                    Some(task.colon_token as u64),
                    builder,
                ) {
                    Ok(node) => node,
                    Err(error) => return Err(error),
                };
                task.pending_sequence.push(
                    CstSequenceEntry {
                        node_index: pair,
                        token_start: task.entry_token_start as u64,
                        token_end: task.cursor as u64,
                        indicator_token: None,
                    },
                );
                task.state = 4;
                tasks.push(task);
                continue;
            }
            if task.cursor >= task.end {
                return Err(
                    CstError::at(CstErrorKind::UnexpectedEndOfInput, builder.source_len_bytes),
                );
            }
            if tokens[task.cursor].kind() == CompletedTokenKind::FlowSequenceEnd {
                let closer = task.cursor;
                completed =
                Some(
                    match finish_iterative_sequence(tokens, task, Some(closer), builder) {
                        Ok(parsed) => parsed,
                        Err(error) => return Err(error),
                    },
                );
                continue;
            }
            if tokens[task.cursor].kind() != CompletedTokenKind::FlowEntry {
                return Err(
                    CstError::at(CstErrorKind::MissingFlowEntry, tokens[task.cursor].byte_start()),
                );
            }
            task.flow_entry_tokens.push(task.cursor as u64);
            task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
            if task.cursor < task.end && tokens[task.cursor].kind()
                == CompletedTokenKind::FlowEntry {
                return Err(
                    CstError::at(
                        CstErrorKind::UnexpectedFlowEntry,
                        tokens[task.cursor].byte_start(),
                    ),
                );
            }
            task.state = 0;
            tasks.push(task);
            continue;
        }
        if task.kind == ParseTaskKind::FlowMapping {
            if task.state == 0 {
                task.cursor = skip_trivia(tokens, task.cursor, task.end);
                if task.cursor >= task.end {
                    return Err(
                        CstError::at(CstErrorKind::UnexpectedEndOfInput, builder.source_len_bytes),
                    );
                }
                if tokens[task.cursor].kind() == CompletedTokenKind::FlowMappingEnd {
                    let closer = task.cursor;
                    completed =
                    Some(
                        match finish_iterative_mapping(tokens, task, Some(closer), builder) {
                            Ok(parsed) => parsed,
                            Err(error) => return Err(error),
                        },
                    );
                    continue;
                }
                if tokens[task.cursor].kind() == CompletedTokenKind::FlowEntry {
                    return Err(
                        CstError::at(
                            CstErrorKind::UnexpectedFlowEntry,
                            tokens[task.cursor].byte_start(),
                        ),
                    );
                }
                task.entry_token_start = task.cursor;
                task.explicit_key = tokens[task.cursor].kind()
                    == CompletedTokenKind::ExplicitMappingKey;
                if task.explicit_key {
                    task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
                }
                if task.cursor >= task.end || tokens[task.cursor].kind()
                    == CompletedTokenKind::MappingValue || tokens[task.cursor].kind()
                    == CompletedTokenKind::FlowEntry || tokens[task.cursor].kind()
                    == CompletedTokenKind::FlowMappingEnd {
                    task.key_node_index = match empty_node(builder, tokens, task.cursor) {
                        Ok(node) => node,
                        Err(error) => return Err(error),
                    };
                    task.state = 2;
                    tasks.push(task);
                } else {
                    let child_offset = tokens[task.cursor].byte_start();
                    let child = node_task(task.cursor, task.end, false, task.depth_left);
                    task.state = 1;
                    tasks.push(task);
                    if let Err(error) = push_iterative_task(
                        &mut tasks,
                        child,
                        depth_limit,
                        child_offset,
                    ) {
                        return Err(error);
                    }
                }
                continue;
            }
            if task.state == 1 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.key_node_index = child.node_index;
                task.cursor = skip_trivia(tokens, child.next_token, task.end);
                task.state = 2;
                tasks.push(task);
                continue;
            }
            if task.state == 2 {
                if task.cursor < task.end && tokens[task.cursor].kind()
                    == CompletedTokenKind::MappingValue {
                    if task.entry_token_start >= task.end {
                        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                    }
                    if !task.explicit_key && tokens[task.entry_token_start].start_line_number()
                        != tokens[task.cursor].start_line_number() {
                        return Err(
                            CstError::at(
                                CstErrorKind::MultilineImplicitKey,
                                tokens[task.cursor].byte_start(),
                            ),
                        );
                    }
                    task.colon_token = task.cursor;
                    task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
                    if task.cursor >= task.end || tokens[task.cursor].kind()
                        == CompletedTokenKind::FlowEntry || tokens[task.cursor].kind()
                        == CompletedTokenKind::FlowMappingEnd {
                        let value = match empty_node(builder, tokens, task.cursor) {
                            Ok(node) => node,
                            Err(error) => return Err(error),
                        };
                        task.pending_mapping.push(
                            CstMappingEntry {
                                key_node_index: task.key_node_index,
                                value_node_index: value,
                                token_start: task.entry_token_start as u64,
                                token_end: task.cursor as u64,
                                explicit_key_token: if task.explicit_key {
                                    Some(task.entry_token_start as u64)
                                } else {
                                    None
                                },
                                mapping_value_token: Some(task.colon_token as u64),
                            },
                        );
                        task.state = 4;
                        tasks.push(task);
                    } else {
                        let child_offset = tokens[task.cursor].byte_start();
                        let child = node_task(task.cursor, task.end, false, task.depth_left);
                        task.state = 3;
                        tasks.push(task);
                        if let Err(error) = push_iterative_task(
                            &mut tasks,
                            child,
                            depth_limit,
                            child_offset,
                        ) {
                            return Err(error);
                        }
                    }
                } else {
                    let value = match empty_node(builder, tokens, task.cursor) {
                        Ok(node) => node,
                        Err(error) => return Err(error),
                    };
                    task.pending_mapping.push(
                        CstMappingEntry {
                            key_node_index: task.key_node_index,
                            value_node_index: value,
                            token_start: task.entry_token_start as u64,
                            token_end: task.cursor as u64,
                            explicit_key_token: if task.explicit_key {
                                Some(task.entry_token_start as u64)
                            } else {
                                None
                            },
                            mapping_value_token: None,
                        },
                    );
                    task.state = 4;
                    tasks.push(task);
                }
                continue;
            }
            if task.state == 3 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.cursor = skip_trivia(tokens, child.next_token, task.end);
                task.pending_mapping.push(
                    CstMappingEntry {
                        key_node_index: task.key_node_index,
                        value_node_index: child.node_index,
                        token_start: task.entry_token_start as u64,
                        token_end: task.cursor as u64,
                        explicit_key_token: if task.explicit_key {
                            Some(task.entry_token_start as u64)
                        } else {
                            None
                        },
                        mapping_value_token: Some(task.colon_token as u64),
                    },
                );
                task.state = 4;
                tasks.push(task);
                continue;
            }
            if task.cursor >= task.end {
                return Err(
                    CstError::at(CstErrorKind::UnexpectedEndOfInput, builder.source_len_bytes),
                );
            }
            if tokens[task.cursor].kind() == CompletedTokenKind::FlowMappingEnd {
                let closer = task.cursor;
                completed =
                Some(
                    match finish_iterative_mapping(tokens, task, Some(closer), builder) {
                        Ok(parsed) => parsed,
                        Err(error) => return Err(error),
                    },
                );
                continue;
            }
            if tokens[task.cursor].kind() != CompletedTokenKind::FlowEntry {
                return Err(
                    CstError::at(CstErrorKind::MissingFlowEntry, tokens[task.cursor].byte_start()),
                );
            }
            task.flow_entry_tokens.push(task.cursor as u64);
            task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
            if task.cursor < task.end && tokens[task.cursor].kind()
                == CompletedTokenKind::FlowEntry {
                return Err(
                    CstError::at(
                        CstErrorKind::UnexpectedFlowEntry,
                        tokens[task.cursor].byte_start(),
                    ),
                );
            }
            task.state = 0;
            tasks.push(task);
            continue;
        }
        if task.kind == ParseTaskKind::BlockSequence {
            if task.state == 0 {
                task.cursor = skip_trivia(tokens, task.cursor, task.end);
                if task.cursor >= task.end || tokens[task.cursor].kind()
                    != CompletedTokenKind::BlockSequenceEntry || token_column(
                    atoms,
                    &tokens[task.cursor],
                ) != task.indentation {
                    if task.pending_sequence.len() == 0 {
                        return Err(
                            CstError::at(
                                CstErrorKind::UnexpectedToken,
                                byte_at(tokens, task.opener, builder.source_len_bytes),
                            ),
                        );
                    }
                    completed =
                    Some(
                        match finish_iterative_sequence(tokens, task, None, builder) {
                            Ok(parsed) => parsed,
                            Err(error) => return Err(error),
                        },
                    );
                    continue;
                }
                let dash = task.cursor;
                let dash_line = tokens[dash].start_line_number();
                task.cursor = skip_trivia(tokens, dash + 1, task.end);
                task.entry_token_start = dash;
                task.entry_token_end = dash + 1;
                if task.cursor >= task.end || tokens[task.cursor].start_line_number() > dash_line
                    && token_column(atoms, &tokens[task.cursor]) <= task.indentation
                    || tokens[task.cursor].kind() == CompletedTokenKind::BlockSequenceEntry
                    && token_column(atoms, &tokens[task.cursor]) == task.indentation {
                    let node = match empty_node(builder, tokens, task.cursor) {
                        Ok(node) => node,
                        Err(error) => return Err(error),
                    };
                    task.pending_sequence.push(
                        CstSequenceEntry {
                            node_index: node,
                            token_start: dash as u64,
                            token_end: if task.cursor > dash + 1 {
                                task.cursor as u64
                            } else {
                                (dash + 1) as u64
                            },
                            indicator_token: Some(dash as u64),
                        },
                    );
                    task.state = 2;
                    tasks.push(task);
                } else {
                    let child_offset = tokens[task.cursor].byte_start();
                    let child_end = match block_property_only_end(
                        atoms,
                        tokens,
                        task.cursor,
                        task.end,
                        task.indentation,
                    ) {
                        Some(end) => end,
                        None => task.end,
                    };
                    let child = node_task(task.cursor, child_end, true, task.depth_left);
                    task.state = 1;
                    tasks.push(task);
                    if let Err(error) = push_iterative_task(
                        &mut tasks,
                        child,
                        depth_limit,
                        child_offset,
                    ) {
                        return Err(error);
                    }
                }
                continue;
            }
            if task.state == 1 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end || task.colon_token >= task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.cursor = child.next_token;
                task.entry_token_end = child.next_token;
                if task.entry_token_end > task.node_token_end {
                    task.node_token_end = task.entry_token_end;
                }
                task.pending_sequence.push(
                    CstSequenceEntry {
                        node_index: child.node_index,
                        token_start: task.entry_token_start as u64,
                        token_end: task.entry_token_end as u64,
                        indicator_token: Some(task.entry_token_start as u64),
                    },
                );
                task.state = 2;
                tasks.push(task);
                continue;
            }
            let next = skip_trivia(tokens, task.cursor, task.end);
            if next >= task.end {
                task.cursor = next;
                task.state = 0;
                tasks.push(task);
                continue;
            }
            let next_column = token_column(atoms, &tokens[next]);
            if tokens[next].kind() == CompletedTokenKind::BlockSequenceEntry && next_column
                == task.indentation {
                task.cursor = next;
                task.state = 0;
                tasks.push(task);
                continue;
            }
            if next_column > task.indentation {
                return Err(
                    CstError::at(CstErrorKind::InvalidIndentation, tokens[next].byte_start()),
                );
            }
            task.cursor = next;
            task.state = 0;
            tasks.push(task);
            continue;
        }
        if task.kind == ParseTaskKind::BlockMapping {
            if task.state == 0 {
                task.cursor = skip_trivia(tokens, task.cursor, task.end);
                if task.cursor >= task.end || token_column(atoms, &tokens[task.cursor])
                    != task.indentation {
                    if task.pending_mapping.len() == 0 {
                        return Err(
                            CstError::at(
                                CstErrorKind::UnexpectedToken,
                                byte_at(tokens, task.opener, builder.source_len_bytes),
                            ),
                        );
                    }
                    completed =
                    Some(
                        match finish_iterative_mapping(tokens, task, None, builder) {
                            Ok(parsed) => parsed,
                            Err(error) => return Err(error),
                        },
                    );
                    continue;
                }
                task.entry_token_start = task.cursor;
                task.explicit_key = tokens[task.cursor].kind()
                    == CompletedTokenKind::ExplicitMappingKey;
                if task.explicit_key {
                    task.cursor = skip_trivia(tokens, task.cursor + 1, task.end);
                }
                let colon = if task.explicit_key {
                    find_explicit_mapping_value(
                        atoms,
                        tokens,
                        task.cursor,
                        task.end,
                        task.indentation,
                    )
                } else {
                    find_mapping_value_on_line(tokens, task.cursor, task.end)
                };
                if colon.is_none() {
                    if !task.explicit_key {
                        if task.pending_mapping.len() == 0 {
                            return Err(
                                CstError::at(
                                    CstErrorKind::UnexpectedToken,
                                    byte_at(tokens, task.cursor, builder.source_len_bytes),
                                ),
                            );
                        }
                        completed =
                        Some(
                            match finish_iterative_mapping(tokens, task, None, builder) {
                                Ok(parsed) => parsed,
                                Err(error) => return Err(error),
                            },
                        );
                        continue;
                    }
                    if task.cursor >= task.end || tokens[task.cursor].start_line_number()
                        > tokens[task.entry_token_start].start_line_number() && token_column(
                        atoms,
                        &tokens[task.cursor],
                    ) <= task.indentation {
                        task.key_node_index = match empty_node(builder, tokens, task.cursor) {
                            Ok(node) => node,
                            Err(error) => return Err(error),
                        };
                        task.state = 5;
                        tasks.push(task);
                    } else {
                        let child_offset = tokens[task.cursor].byte_start();
                        let allow_block_collection = block_value_allows_collection(
                            tokens,
                            task.cursor,
                            task.end,
                            task.entry_token_start,
                        );
                        let child = node_task(
                            task.cursor,
                            task.end,
                            allow_block_collection,
                            task.depth_left,
                        );
                        task.state = 4;
                        tasks.push(task);
                        if let Err(error) = push_iterative_task(
                            &mut tasks,
                            child,
                            depth_limit,
                            child_offset,
                        ) {
                            return Err(error);
                        }
                    }
                    continue;
                }
                task.colon_token = colon.unwrap();
                if task.cursor >= task.colon_token {
                    task.key_node_index = match empty_node(builder, tokens, task.colon_token) {
                        Ok(node) => node,
                        Err(error) => return Err(error),
                    };
                    task.state = 2;
                    tasks.push(task);
                } else {
                    let child_offset = tokens[task.cursor].byte_start();
                    let allow_block_collection = task.explicit_key && block_value_allows_collection(
                        tokens,
                        task.cursor,
                        task.colon_token,
                        task.entry_token_start,
                    );
                    let child = node_task(
                        task.cursor,
                        task.colon_token,
                        allow_block_collection,
                        task.depth_left,
                    );
                    task.state = 1;
                    tasks.push(task);
                    if let Err(error) = push_iterative_task(
                        &mut tasks,
                        child,
                        depth_limit,
                        child_offset,
                    ) {
                        return Err(error);
                    }
                }
                continue;
            }
            if task.state == 1 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.colon_token || task.colon_token > task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                let remaining = skip_trivia(tokens, child.next_token, task.colon_token);
                if remaining != task.colon_token {
                    return Err(
                        CstError::at(CstErrorKind::UnexpectedToken, tokens[remaining].byte_start()),
                    );
                }
                task.key_node_index = child.node_index;
                task.state = 2;
                tasks.push(task);
                continue;
            }
            if task.state == 2 {
                if task.colon_token >= task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.cursor = skip_trivia(tokens, task.colon_token + 1, task.end);
                let indentationless_sequence = task.cursor < task.end
                    && tokens[task.cursor].start_line_number()
                    > tokens[task.colon_token].start_line_number() && tokens[task.cursor].kind()
                    == CompletedTokenKind::BlockSequenceEntry && token_column(
                    atoms,
                    &tokens[task.cursor],
                ) == task.indentation;
                if task.cursor >= task.end || !indentationless_sequence
                    && tokens[task.cursor].start_line_number()
                    > tokens[task.colon_token].start_line_number() && token_column(
                    atoms,
                    &tokens[task.cursor],
                ) <= task.indentation {
                    let value = match empty_node(builder, tokens, task.cursor) {
                        Ok(node) => node,
                        Err(error) => return Err(error),
                    };
                    task.entry_token_end = if task.cursor > task.colon_token + 1 {
                        task.cursor
                    } else {
                        task.colon_token + 1
                    };
                    task.pending_mapping.push(
                        CstMappingEntry {
                            key_node_index: task.key_node_index,
                            value_node_index: value,
                            token_start: task.entry_token_start as u64,
                            token_end: task.entry_token_end as u64,
                            explicit_key_token: if task.explicit_key {
                                Some(task.entry_token_start as u64)
                            } else {
                                None
                            },
                            mapping_value_token: Some(task.colon_token as u64),
                        },
                    );
                    task.state = 6;
                    tasks.push(task);
                } else {
                    let child_offset = tokens[task.cursor].byte_start();
                    let child_end = match block_property_only_end(
                        atoms,
                        tokens,
                        task.cursor,
                        task.end,
                        task.indentation,
                    ) {
                        Some(end) => end,
                        None => task.end,
                    };
                    let allow_block_collection = block_value_allows_collection(
                        tokens,
                        task.cursor,
                        child_end,
                        task.colon_token,
                    );
                    let child = node_task(
                        task.cursor,
                        child_end,
                        allow_block_collection,
                        task.depth_left,
                    );
                    task.state = 3;
                    tasks.push(task);
                    if let Err(error) = push_iterative_task(
                        &mut tasks,
                        child,
                        depth_limit,
                        child_offset,
                    ) {
                        return Err(error);
                    }
                }
                continue;
            }
            if task.state == 3 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end || task.colon_token >= task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.cursor = child.next_token;
                task.entry_token_end = if task.cursor > task.colon_token + 1 {
                    task.cursor
                } else {
                    task.colon_token + 1
                };
                task.pending_mapping.push(
                    CstMappingEntry {
                        key_node_index: task.key_node_index,
                        value_node_index: child.node_index,
                        token_start: task.entry_token_start as u64,
                        token_end: task.entry_token_end as u64,
                        explicit_key_token: if task.explicit_key {
                            Some(task.entry_token_start as u64)
                        } else {
                            None
                        },
                        mapping_value_token: Some(task.colon_token as u64),
                    },
                );
                task.state = 6;
                tasks.push(task);
                continue;
            }
            if task.state == 4 {
                let child = match completed.take() {
                    Some(parsed) => parsed,
                    None => return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0)),
                };
                if child.next_token > task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                task.key_node_index = child.node_index;
                task.cursor = skip_trivia(tokens, child.next_token, task.end);
                task.state = 5;
                tasks.push(task);
                continue;
            }
            if task.state == 5 {
                if task.entry_token_start >= task.end {
                    return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
                }
                let value = match empty_node(builder, tokens, task.cursor) {
                    Ok(node) => node,
                    Err(error) => return Err(error),
                };
                task.entry_token_end = if task.cursor > task.entry_token_start + 1 {
                    task.cursor
                } else {
                    task.entry_token_start + 1
                };
                task.pending_mapping.push(
                    CstMappingEntry {
                        key_node_index: task.key_node_index,
                        value_node_index: value,
                        token_start: task.entry_token_start as u64,
                        token_end: task.entry_token_end as u64,
                        explicit_key_token: Some(task.entry_token_start as u64),
                        mapping_value_token: None,
                    },
                );
                task.state = 6;
                tasks.push(task);
                continue;
            }
            if task.entry_token_end > task.node_token_end {
                task.node_token_end = task.entry_token_end;
            }
            let next = skip_trivia(tokens, task.cursor, task.end);
            if next >= task.end {
                task.cursor = next;
                task.state = 0;
                tasks.push(task);
                continue;
            }
            let next_column = token_column(atoms, &tokens[next]);
            if next_column == task.indentation && (tokens[next].kind()
                == CompletedTokenKind::ExplicitMappingKey || find_mapping_value_on_line(
                tokens,
                next,
                task.end,
            ).is_some()) {
                task.cursor = next;
                task.state = 0;
                tasks.push(task);
                continue;
            }
            if next_column > task.indentation {
                return Err(
                    CstError::at(CstErrorKind::InvalidIndentation, tokens[next].byte_start()),
                );
            }
            task.cursor = next;
            task.state = 0;
            tasks.push(task);
            continue;
        }
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
}

pub fn parse_profile1_cst(
    atomized: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    supplied_tokens: &CompletedTokenSource,
    limits: CstLimits,
) -> (result: Result<CstSource, CstError>)
    ensures
        result.is_ok() ==> cst_child_before_parent_spec(
            result.unwrap()@.nodes,
            result.unwrap()@.sequence_entries,
            result.unwrap()@.mapping_entries,
        ),
        result.is_ok() ==> cst_entry_tables_uniquely_owned_spec(
            result.unwrap()@.nodes,
            result.unwrap()@.sequence_entries.len() as u64,
            result.unwrap()@.mapping_entries.len() as u64,
        ),
        result.is_ok() ==> cst_nodes_have_exact_token_identity_spec(
            supplied_tokens@.tokens,
            supplied_tokens@.source_len_bytes,
            result.unwrap()@.nodes,
        ),
        result.is_ok() ==> cst_entry_ranges_spec(
            supplied_tokens@.tokens.len() as u64,
            result.unwrap()@.nodes,
            result.unwrap()@.sequence_entries,
            result.unwrap()@.mapping_entries,
        ),
        result.is_ok() ==> cst_documents_and_warnings_ordered_spec(
            supplied_tokens@.tokens,
            supplied_tokens@.source_len_bytes,
            result.unwrap()@.documents,
            result.unwrap()@.nodes,
            result.unwrap()@.warnings,
        ),
        result.is_ok() ==> cst_exact_syntax_ownership_spec(
            supplied_tokens@.tokens,
            result.unwrap()@.documents,
            result.unwrap()@.nodes,
            result.unwrap()@.sequence_entries,
            result.unwrap()@.mapping_entries,
            result.unwrap()@.syntax_owners,
        ),
        result.is_ok() ==> cst_source_respects_limits_spec(result.unwrap()@, limits@),
        result.is_ok() ==> cst_public_semantics_spec(supplied_tokens@, result.unwrap()@),
{
    let canonical_tokens = match scan_profile1_completed_tokens(
        atomized,
        layout,
        structural,
        quoted,
        plain,
        block,
        canonical_completed_token_limits(),
    ) {
        Ok(tokens) => tokens,
        Err(error) => {
            return Err(
                CstError::at(CstErrorKind::InputCompletedTokenMismatch, error.byte_offset()),
            );
        },
    };
    if !canonical_tokens.same_as(supplied_tokens) {
        return Err(CstError::at(CstErrorKind::InputCompletedTokenMismatch, atomized.bom_bytes()));
    }
    let document_limit = effective_limit(limits.max_documents, MAX_PROFILE1_CST_DOCUMENTS);
    let node_limit = effective_limit(limits.max_nodes, MAX_PROFILE1_CST_NODES);
    let sequence_limit = effective_limit(
        limits.max_sequence_entries,
        MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
    );
    let mapping_limit = effective_limit(
        limits.max_mapping_entries,
        MAX_PROFILE1_CST_MAPPING_ENTRIES,
    );
    let directive_limit = effective_limit(limits.max_directives, MAX_PROFILE1_CST_DIRECTIVES);
    let warning_limit = effective_limit(limits.max_warnings, MAX_PROFILE1_CST_WARNINGS);
    let depth_limit = effective_limit(limits.max_depth, MAX_PROFILE1_CST_DEPTH);
    let tokens = supplied_tokens.tokens();
    let atoms = atomized.atoms();
    if tokens.len() > MAX_PROFILE1_COMPLETED_TOKENS as usize {
        return Err(CstError::at(CstErrorKind::InputCompletedTokenMismatch, atomized.bom_bytes()));
    }
    let mut syntax_owner_slots: Vec<Option<CstSyntaxOwner>> = Vec::new();
    let mut syntax_owner_slot_index = 0usize;
    while syntax_owner_slot_index < tokens.len()
        invariant
            syntax_owner_slot_index <= tokens.len(),
            syntax_owner_slots.len() == syntax_owner_slot_index,
        decreases tokens.len() - syntax_owner_slot_index,
    {
        syntax_owner_slots.push(None);
        syntax_owner_slot_index += 1;
    }
    let mut builder = CstBuilder {
        documents: Vec::new(),
        nodes: Vec::new(),
        sequence_entries: Vec::new(),
        mapping_entries: Vec::new(),
        warnings: Vec::new(),
        syntax_owner_slots,
        document_limit,
        node_limit,
        sequence_limit,
        mapping_limit,
        directive_limit,
        warning_limit,
        directive_count: 0,
        maximum_depth: 0,
        source_len_bytes: supplied_tokens.source_len_bytes(),
    };
    let mut index = 0usize;
    let mut stream_fuel = tokens.len();
    while index < tokens.len()
        invariant
            index <= tokens.len(),
            tokens.len() <= MAX_PROFILE1_COMPLETED_TOKENS,
            depth_limit <= MAX_PROFILE1_CST_DEPTH,
            builder.syntax_owner_slots.len() == tokens.len(),
        decreases stream_fuel,
    {
        if stream_fuel == 0 {
            return Err(CstError::at(CstErrorKind::UnexpectedEndOfInput, builder.source_len_bytes));
        }
        stream_fuel -= 1;
        let document_token_start = index;
        let mut syntax = skip_trivia(tokens, index, tokens.len());
        if syntax >= tokens.len() {
            break;
        }
        if tokens[syntax].kind() == CompletedTokenKind::DocumentEnd {
            return Err(
                CstError::at(CstErrorKind::UnexpectedDocumentMarker, tokens[syntax].byte_start()),
            );
        }
        let directive_start = syntax;
        let mut saw_directive = false;
        let mut saw_yaml = false;
        let mut handles: Vec<(u64, u64)> = Vec::new();
        let mut directive_fuel = tokens.len();
        while syntax < tokens.len() && (tokens[syntax].kind() == CompletedTokenKind::YamlDirective
            || tokens[syntax].kind() == CompletedTokenKind::TagDirective || tokens[syntax].kind()
            == CompletedTokenKind::ReservedDirective)
            invariant
                syntax <= tokens.len(),
                builder.syntax_owner_slots.len() == tokens.len(),
            decreases directive_fuel,
        {
            if directive_fuel == 0 {
                return Err(
                    CstError::at(CstErrorKind::UnexpectedEndOfInput, builder.source_len_bytes),
                );
            }
            directive_fuel -= 1;
            assert(syntax < tokens.len());
            if builder.directive_count >= builder.directive_limit {
                return Err(
                    CstError::at(CstErrorKind::DirectiveLimitExceeded, tokens[syntax].byte_start()),
                );
            }
            builder.directive_count += 1;
            if let Err(error) = builder.claim_syntax_token(
                syntax as u64,
                CstSyntaxOwnerKind::Directive,
                builder.documents.len() as u64,
            ) {
                return Err(error);
            }
            saw_directive = true;
            let kind = tokens[syntax].kind();
            if kind == CompletedTokenKind::YamlDirective {
                if saw_yaml {
                    return Err(
                        CstError::at(
                            CstErrorKind::DuplicateYamlDirective,
                            tokens[syntax].byte_start(),
                        ),
                    );
                }
                saw_yaml = true;
                if let Some((major, minor)) = tokens[syntax].yaml_version() {
                    if major != 1 {
                        let offset = match part_of_kind(
                            &tokens[syntax],
                            CompletedTokenPartKind::YamlMajor,
                        ) {
                            Some(part) => part.byte_start(),
                            None => tokens[syntax].byte_start(),
                        };
                        return Err(CstError::at(CstErrorKind::UnsupportedYamlMajorVersion, offset));
                    }
                    if minor == 1 {
                        let warning = CstWarning {
                            kind: CstWarningKind::Yaml11Compatibility,
                            document_index: builder.documents.len() as u64,
                            token_index: syntax as u64,
                            byte_offset: tokens[syntax].byte_start(),
                        };
                        match builder.push_warning(warning) {
                            Ok(()) => {},
                            Err(error) => return Err(error),
                        }
                    } else if minor > 2 {
                        let warning = CstWarning {
                            kind: CstWarningKind::FutureMinorVersion,
                            document_index: builder.documents.len() as u64,
                            token_index: syntax as u64,
                            byte_offset: tokens[syntax].byte_start(),
                        };
                        match builder.push_warning(warning) {
                            Ok(()) => {},
                            Err(error) => return Err(error),
                        }
                    }
                }
            } else if kind == CompletedTokenKind::TagDirective {
                if let Some(handle) = part_of_kind(
                    &tokens[syntax],
                    CompletedTokenPartKind::TagHandle,
                ) {
                    let mut handle_index = 0usize;
                    while handle_index < handles.len()
                        invariant
                            handle_index <= handles.len(),
                            syntax < tokens.len(),
                        decreases handles.len() - handle_index,
                    {
                        if atom_ranges_equal(
                            atoms,
                            handles[handle_index].0,
                            handles[handle_index].1,
                            handle.start_atom_index(),
                            handle.end_atom_index(),
                        ) {
                            return Err(
                                CstError::at(
                                    CstErrorKind::DuplicateTagHandle,
                                    tokens[syntax].byte_start(),
                                ),
                            );
                        }
                        handle_index += 1;
                    }
                    handles.push((handle.start_atom_index(), handle.end_atom_index()));
                }
            } else {
                let warning = CstWarning {
                    kind: CstWarningKind::ReservedDirective,
                    document_index: builder.documents.len() as u64,
                    token_index: syntax as u64,
                    byte_offset: tokens[syntax].byte_start(),
                };
                match builder.push_warning(warning) {
                    Ok(()) => {},
                    Err(error) => return Err(error),
                }
            }
            syntax = skip_trivia(tokens, syntax + 1, tokens.len());
        }
        let directive_end = syntax;
        let explicit_start_token_start = directive_end;
        let explicit_start_token = if syntax < tokens.len() && tokens[syntax].kind()
            == CompletedTokenKind::DirectivesEnd {
            let marker = syntax;
            if let Err(error) = builder.claim_syntax_token(
                marker as u64,
                CstSyntaxOwnerKind::DocumentStartMarker,
                builder.documents.len() as u64,
            ) {
                return Err(error);
            }
            syntax = skip_trivia(tokens, syntax + 1, tokens.len());
            Some(marker as u64)
        } else {
            if saw_directive {
                return Err(
                    CstError::at(
                        CstErrorKind::MissingDirectivesEnd,
                        byte_at(tokens, syntax, builder.source_len_bytes),
                    ),
                );
            }
            None
        };
        let content_start = syntax;
        let explicit_start_token_end = content_start;
        assert(content_start <= tokens.len());
        let mut boundary = syntax;
        while boundary < tokens.len() && tokens[boundary].kind() != CompletedTokenKind::DocumentEnd
            && tokens[boundary].kind() != CompletedTokenKind::DirectivesEnd
            invariant
                content_start <= boundary,
                boundary <= tokens.len(),
            decreases tokens.len() - boundary,
        {
            boundary += 1;
        }
        if let Some(undeclared) = first_undeclared_tag_handle(
            atoms,
            tokens,
            content_start,
            boundary,
            handles.as_slice(),
        ) {
            return Err(
                CstError::at(CstErrorKind::UndeclaredTagHandle, tokens[undeclared].byte_start()),
            );
        }
        let root = if skip_trivia(tokens, content_start, boundary) >= boundary {
            match empty_node(&mut builder, tokens, boundary) {
                Ok(node) => node,
                Err(error) => return Err(error),
            }
        } else {
            let parsed = match parse_node_iterative(
                atoms,
                tokens,
                content_start,
                boundary,
                true,
                depth_limit,
                &mut builder,
            ) {
                Ok(parsed) => parsed,
                Err(error) => return Err(error),
            };
            let remaining = skip_trivia(tokens, parsed.next_token, boundary);
            if remaining != boundary {
                return Err(
                    CstError::at(CstErrorKind::UnexpectedToken, tokens[remaining].byte_start()),
                );
            }
            parsed.node_index
        };
        if root >= builder.nodes.len() as u64 {
            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
        }
        let root_token_start = builder.nodes[root as usize].token_start;
        let root_token_end = builder.nodes[root as usize].token_end;
        if root_token_end > tokens.len() as u64 {
            return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
        }
        let explicit_end_token = if boundary < tokens.len() && tokens[boundary].kind()
            == CompletedTokenKind::DocumentEnd {
            if let Err(error) = builder.claim_syntax_token(
                boundary as u64,
                CstSyntaxOwnerKind::DocumentEndMarker,
                builder.documents.len() as u64,
            ) {
                return Err(error);
            }
            Some(boundary as u64)
        } else {
            None
        };
        let explicit_end_token_start = root_token_end as usize;
        let explicit_end_token_end = if explicit_end_token.is_some() {
            boundary + 1
        } else {
            explicit_end_token_start
        };
        index =
        if explicit_end_token.is_some() {
            skip_trivia(tokens, boundary + 1, tokens.len())
        } else {
            boundary
        };
        let suffix_token_start = explicit_end_token_end;
        let document_token_end = index;
        let byte_start = byte_at(tokens, document_token_start, builder.source_len_bytes);
        let byte_end = if document_token_end > document_token_start {
            tokens[document_token_end - 1].byte_end()
        } else {
            byte_start
        };
        let document = CstDocument {
            token_start: document_token_start as u64,
            token_end: document_token_end as u64,
            byte_start,
            byte_end,
            prefix_token_start: document_token_start as u64,
            prefix_token_end: directive_start as u64,
            directive_start: directive_start as u64,
            directive_end: directive_end as u64,
            explicit_start_token_start: explicit_start_token_start as u64,
            explicit_start_token_end: explicit_start_token_end as u64,
            root_token_start,
            root_token_end,
            explicit_end_token_start: explicit_end_token_start as u64,
            explicit_end_token_end: explicit_end_token_end as u64,
            suffix_token_start: suffix_token_start as u64,
            suffix_token_end: document_token_end as u64,
            root_node_index: root,
            explicit_start_token,
            explicit_end_token,
        };
        match builder.push_document(document) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
    }
    let mut syntax_owner_index = 0usize;
    while syntax_owner_index < tokens.len()
        invariant
            syntax_owner_index <= tokens.len(),
            builder.syntax_owner_slots.len() == tokens.len(),
        decreases tokens.len() - syntax_owner_index,
    {
        if is_trivia(tokens[syntax_owner_index].kind()) {
            if builder.syntax_owner_slots[syntax_owner_index].is_some() {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
        } else {
            if builder.syntax_owner_slots[syntax_owner_index].is_none() {
                return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
            }
        }
        syntax_owner_index += 1;
    }
    let source = CstSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_token_transformation_version: supplied_tokens.transformation_version(),
        transformation_version: CST_TRANSFORMATION_VERSION,
        source_len_bytes: supplied_tokens.source_len_bytes(),
        input_token_count: tokens.len() as u64,
        directive_count: builder.directive_count,
        maximum_depth: builder.maximum_depth,
        documents: builder.documents,
        nodes: builder.nodes,
        sequence_entries: builder.sequence_entries,
        mapping_entries: builder.mapping_entries,
        warnings: builder.warnings,
        syntax_owners: builder.syntax_owner_slots,
    };
    if !cst_child_before_parent(
        source.nodes.as_slice(),
        source.sequence_entries.as_slice(),
        source.mapping_entries.as_slice(),
    ) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    if !cst_entry_tables_uniquely_owned(
        source.nodes.as_slice(),
        source.sequence_entries.len(),
        source.mapping_entries.len(),
    ) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    if !cst_nodes_have_exact_token_identity(
        tokens,
        source.source_len_bytes,
        source.nodes.as_slice(),
    ) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    if !cst_entries_have_valid_ranges(
        tokens.len(),
        source.nodes.as_slice(),
        source.sequence_entries.as_slice(),
        source.mapping_entries.as_slice(),
    ) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    if !cst_documents_and_warnings_are_ordered(
        tokens,
        source.source_len_bytes,
        source.documents.as_slice(),
        source.nodes.as_slice(),
        source.warnings.as_slice(),
    ) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    if !cst_exact_syntax_ownership(
        tokens,
        source.documents.as_slice(),
        source.nodes.as_slice(),
        source.sequence_entries.as_slice(),
        source.mapping_entries.as_slice(),
        source.syntax_owners.as_slice(),
    ) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    if !cst_source_respects_limits(&source, limits) {
        return Err(CstError::at(CstErrorKind::InternalInvariantViolation, 0));
    }
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        assert(crate::token::completed_token_views_spec(tokens@).len() == tokens.len());
        assert(supplied_tokens@.tokens.len() == tokens.len());
        reveal(cst_source_respects_limits_spec);
        reveal(cst_effective_limit_spec);
        assert(source@.documents.len() <= MAX_PROFILE1_CST_DOCUMENTS);
        assert(source@.nodes.len() <= MAX_PROFILE1_CST_NODES);
        assert(source@.sequence_entries.len() <= MAX_PROFILE1_CST_SEQUENCE_ENTRIES);
        assert(source@.mapping_entries.len() <= MAX_PROFILE1_CST_MAPPING_ENTRIES);
        assert(source@.directive_count <= MAX_PROFILE1_CST_DIRECTIVES);
        assert(source@.warnings.len() <= MAX_PROFILE1_CST_WARNINGS);
        assert(source@.maximum_depth <= MAX_PROFILE1_CST_DEPTH);
        assert(cst_source_respects_limits_spec(
            source@,
            CstLimitsView {
                max_documents: MAX_PROFILE1_CST_DOCUMENTS,
                max_nodes: MAX_PROFILE1_CST_NODES,
                max_sequence_entries: MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
                max_mapping_entries: MAX_PROFILE1_CST_MAPPING_ENTRIES,
                max_directives: MAX_PROFILE1_CST_DIRECTIVES,
                max_warnings: MAX_PROFILE1_CST_WARNINGS,
                max_depth: MAX_PROFILE1_CST_DEPTH,
            },
        ));
        assert(source@.profile_version == CRUCIBLE_YAML_PROFILE_VERSION);
        assert(source@.input_token_transformation_version
            == supplied_tokens@.transformation_version);
        assert(source@.transformation_version == CST_TRANSFORMATION_VERSION);
        assert(source@.source_len_bytes == supplied_tokens@.source_len_bytes);
        assert(source@.input_token_count == supplied_tokens@.tokens.len());
        reveal(cst_public_semantics_spec);
        assert(cst_public_semantics_spec(supplied_tokens@, source@));
    }
    Ok(source)
}

} // verus!
