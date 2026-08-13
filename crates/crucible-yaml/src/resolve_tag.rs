//! Verified explicit YAML tag-property resolution.
//!
//! YAML 1.2.2 tag percent escapes are identity-bearing presentation characters: they are
//! validated by completed-token formation, retained verbatim here, and never UTF-8-decoded.
use crate::atom::{AtomizedSource, LexicalAtom};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::cst::{CstDocument, CstNode, CstSource};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::cst::{CstDocumentView, CstNodeView, CstSourceView};
use crate::token::{
    CompletedToken, CompletedTokenKind, CompletedTokenPart, CompletedTokenPartKind,
    CompletedTokenSource,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::token::{CompletedTokenPartView, CompletedTokenSourceView, CompletedTokenView};
use vstd::prelude::*;

verus! {

pub const TAG_RESOLUTION_VERSION: u16 = 1;

pub const MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagResolutionLimits {
    max_tag_code_points: u64,
}

#[verifier::ext_equal]
pub struct TagResolutionLimitsView {
    pub max_tag_code_points: u64,
}

impl View for TagResolutionLimits {
    type V = TagResolutionLimitsView;

    closed spec fn view(&self) -> TagResolutionLimitsView {
        TagResolutionLimitsView { max_tag_code_points: self.max_tag_code_points }
    }
}

impl TagResolutionLimits {
    pub fn new(max_tag_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (TagResolutionLimitsView { max_tag_code_points }),
    {
        Self { max_tag_code_points }
    }

    pub fn max_tag_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_tag_code_points,
    {
        self.max_tag_code_points
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum ResolvedTagKind {
    NonSpecific,
    Local,
    Global,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum ResolvedTagOrigin {
    DefaultPrimaryPrefix,
    DefaultSecondaryPrefix,
    DirectivePrefix,
    TagSuffix,
    VerbatimPayload,
    NonSpecificIndicator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedTagCodePoint {
    code_point: u32,
    source_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
    origin: ResolvedTagOrigin,
}

#[verifier::ext_equal]
pub struct ResolvedTagCodePointView {
    pub code_point: u32,
    pub source_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub origin: ResolvedTagOrigin,
}

impl View for ResolvedTagCodePoint {
    type V = ResolvedTagCodePointView;

    closed spec fn view(&self) -> ResolvedTagCodePointView {
        ResolvedTagCodePointView {
            code_point: self.code_point,
            source_atom_index: self.source_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            origin: self.origin,
        }
    }
}

impl ResolvedTagCodePoint {
    fn new(
        code_point: u32,
        source_atom_index: u64,
        byte_start: u64,
        byte_end: u64,
        origin: ResolvedTagOrigin,
    ) -> (point: Self)
        ensures
            point@ == (ResolvedTagCodePointView {
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

    pub fn origin(&self) -> (origin: ResolvedTagOrigin)
        ensures
            origin == self@.origin,
    {
        self.origin
    }
}

pub open spec fn resolved_tag_code_point_views_spec(content: Seq<ResolvedTagCodePoint>) -> Seq<
    ResolvedTagCodePointView,
> {
    Seq::new(content.len(), |index: int| content[index]@)
}

proof fn lemma_resolved_tag_code_point_views_push(
    content: Seq<ResolvedTagCodePoint>,
    point: ResolvedTagCodePoint,
)
    ensures
        resolved_tag_code_point_views_spec(content.push(point))
            == resolved_tag_code_point_views_spec(content).push(point@),
{
    reveal(resolved_tag_code_point_views_spec);
    assert(resolved_tag_code_point_views_spec(content.push(point))
        =~= resolved_tag_code_point_views_spec(content).push(point@));
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedTagProperty {
    kind: ResolvedTagKind,
    token_index: u64,
    content: Vec<ResolvedTagCodePoint>,
}

#[verifier::ext_equal]
pub struct ResolvedTagPropertyView {
    pub kind: ResolvedTagKind,
    pub token_index: u64,
    pub content: Seq<ResolvedTagCodePointView>,
}

impl View for ResolvedTagProperty {
    type V = ResolvedTagPropertyView;

    closed spec fn view(&self) -> ResolvedTagPropertyView {
        ResolvedTagPropertyView {
            kind: self.kind,
            token_index: self.token_index,
            content: resolved_tag_code_point_views_spec(self.content@),
        }
    }
}

impl ResolvedTagProperty {
    fn new(kind: ResolvedTagKind, token_index: u64, content: Vec<ResolvedTagCodePoint>) -> (tag:
        Self)
        ensures
            tag@ == (ResolvedTagPropertyView {
                kind,
                token_index,
                content: resolved_tag_code_point_views_spec(content@),
            }),
    {
        Self { kind, token_index, content }
    }

    pub fn kind(&self) -> (kind: ResolvedTagKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn token_index(&self) -> (index: u64)
        ensures
            index == self@.token_index,
    {
        self.token_index
    }

    pub fn content(&self) -> (content: &[ResolvedTagCodePoint])
        ensures
            resolved_tag_code_point_views_spec(content@) == self@.content,
    {
        self.content.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum TagResolutionErrorKind {
    InputCompletedTokenMismatch,
    InputCstMismatch,
    NodeIndexOutOfRange,
    DocumentNotFound,
    InvalidTagToken,
    UndeclaredTagHandle,
    InvalidLocalTag,
    InvalidGlobalTagUri,
    TagCodePointLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagResolutionError {
    kind: TagResolutionErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct TagResolutionErrorView {
    pub kind: TagResolutionErrorKind,
    pub byte_offset: u64,
}

impl View for TagResolutionError {
    type V = TagResolutionErrorView;

    closed spec fn view(&self) -> TagResolutionErrorView {
        TagResolutionErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl TagResolutionError {
    fn at(kind: TagResolutionErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (TagResolutionErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: TagResolutionErrorKind)
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

pub open spec fn effective_tag_code_point_limit_spec(limits: TagResolutionLimitsView) -> u64 {
    if limits.max_tag_code_points < MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS {
        limits.max_tag_code_points
    } else {
        MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS
    }
}

pub open spec fn ascii_tag_scheme_first_spec(code_point: u32) -> bool {
    0x41 <= code_point <= 0x5a || 0x61 <= code_point <= 0x7a
}

pub open spec fn ascii_tag_scheme_continuation_spec(code_point: u32) -> bool {
    ascii_tag_scheme_first_spec(code_point) || 0x30 <= code_point <= 0x39 || code_point == 0x2b
        || code_point == 0x2d || code_point == 0x2e
}

pub open spec fn tag_uri_scheme_tail_spec(
    content: Seq<ResolvedTagCodePointView>,
    index: int,
    fuel: nat,
) -> bool
    decreases fuel,
{
    if fuel == 0 || index < 0 || index >= content.len() {
        false
    } else if content[index].code_point == 0x3a {
        true
    } else {
        ascii_tag_scheme_continuation_spec(content[index].code_point) && tag_uri_scheme_tail_spec(
            content,
            index + 1,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn global_tag_uri_spec(content: Seq<ResolvedTagCodePointView>) -> bool {
    content.len() >= 2 && ascii_tag_scheme_first_spec(content[0].code_point)
        && tag_uri_scheme_tail_spec(content, 1, content.len() as nat)
}

pub open spec fn finalize_resolved_tag_property_spec(
    content: Seq<ResolvedTagCodePointView>,
    token_index: u64,
    tag_byte_start: u64,
    limits: TagResolutionLimitsView,
) -> Result<ResolvedTagPropertyView, TagResolutionErrorView> {
    if content.len() == 0 {
        Err(
            TagResolutionErrorView {
                kind: TagResolutionErrorKind::InvalidTagToken,
                byte_offset: tag_byte_start,
            },
        )
    } else {
        let local = content[0].code_point == 0x21;
        if local && content.len() == 1 {
            Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::InvalidLocalTag,
                    byte_offset: content[0].byte_start,
                },
            )
        } else if !local && !global_tag_uri_spec(content) {
            Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::InvalidGlobalTagUri,
                    byte_offset: content[0].byte_start,
                },
            )
        } else {
            let effective_limit = effective_tag_code_point_limit_spec(limits);
            if content.len() > effective_limit {
                Err(
                    TagResolutionErrorView {
                        kind: TagResolutionErrorKind::TagCodePointLimitExceeded,
                        byte_offset: content[effective_limit as int].byte_start,
                    },
                )
            } else {
                Ok(
                    ResolvedTagPropertyView {
                        kind: if local {
                            ResolvedTagKind::Local
                        } else {
                            ResolvedTagKind::Global
                        },
                        token_index,
                        content,
                    },
                )
            }
        }
    }
}

pub open spec fn default_secondary_prefix_code_points_spec() -> Seq<u32> {
    seq![
        0x74u32,
        0x61u32,
        0x67u32,
        0x3au32,
        0x79u32,
        0x61u32,
        0x6du32,
        0x6cu32,
        0x2eu32,
        0x6fu32,
        0x72u32,
        0x67u32,
        0x2cu32,
        0x32u32,
        0x30u32,
        0x30u32,
        0x32u32,
        0x3au32,
    ]
}

pub open spec fn generated_tag_content_spec(
    code_points: Seq<u32>,
    source_atom_index: u64,
    byte_offset: u64,
    origin: ResolvedTagOrigin,
) -> Seq<ResolvedTagCodePointView> {
    Seq::new(
        code_points.len(),
        |index: int|
            ResolvedTagCodePointView {
                code_point: code_points[index],
                source_atom_index,
                byte_start: byte_offset,
                byte_end: byte_offset,
                origin,
            },
    )
}

pub open spec fn atom_range_tag_content_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
    origin: ResolvedTagOrigin,
) -> Seq<ResolvedTagCodePointView> {
    Seq::new(
        (end - start) as nat,
        |offset: int|
            ResolvedTagCodePointView {
                code_point: atoms[start + offset].code_point,
                source_atom_index: (start + offset) as u64,
                byte_start: atoms[start + offset].span.start.byte_offset,
                byte_end: atoms[start + offset].span.end.byte_offset,
                origin,
            },
    )
}

pub open spec fn tag_part_range_spec(
    atoms: Seq<LexicalAtomView>,
    part: CompletedTokenPartView,
) -> bool {
    part.start_atom_index < part.end_atom_index && part.end_atom_index <= atoms.len()
        && part.byte_start == atoms[part.start_atom_index as int].span.start.byte_offset
        && part.byte_end == atoms[(part.end_atom_index - 1) as int].span.end.byte_offset
}

pub open spec fn tag_part_within_token_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
    part: CompletedTokenPartView,
) -> bool {
    tag_part_range_spec(atoms, part) && token.start_atom_index <= part.start_atom_index
        && part.end_atom_index <= token.end_atom_index && token.byte_start <= part.byte_start
        && part.byte_end <= token.byte_end
}

pub open spec fn tag_part_code_points_spec(
    atoms: Seq<LexicalAtomView>,
    part: CompletedTokenPartView,
) -> Seq<u32> {
    Seq::new(
        (part.end_atom_index - part.start_atom_index) as nat,
        |offset: int| atoms[part.start_atom_index as int + offset].code_point,
    )
}

pub open spec fn tag_parts_have_same_spelling_spec(
    atoms: Seq<LexicalAtomView>,
    left: CompletedTokenPartView,
    right: CompletedTokenPartView,
) -> bool {
    tag_part_range_spec(atoms, left) && tag_part_range_spec(atoms, right)
        && tag_part_code_points_spec(atoms, left) == tag_part_code_points_spec(atoms, right)
}

pub open spec fn primary_tag_handle_spec(
    atoms: Seq<LexicalAtomView>,
    handle: CompletedTokenPartView,
) -> bool {
    tag_part_range_spec(atoms, handle) && handle.end_atom_index - handle.start_atom_index == 1
        && atoms[handle.start_atom_index as int].code_point == 0x21
}

pub open spec fn secondary_tag_handle_spec(
    atoms: Seq<LexicalAtomView>,
    handle: CompletedTokenPartView,
) -> bool {
    tag_part_range_spec(atoms, handle) && handle.end_atom_index - handle.start_atom_index == 2
        && atoms[handle.start_atom_index as int].code_point == 0x21 && atoms[(
    handle.start_atom_index + 1) as int].code_point == 0x21
}

pub open spec fn tag_directive_matches_handle_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
    handle: CompletedTokenPartView,
) -> bool {
    token.kind == CompletedTokenKind::TagDirective && token.parts.len() == 3 && token.parts[1].kind
        == CompletedTokenPartKind::TagHandle && token.parts[2].kind
        == CompletedTokenPartKind::TagPrefix && tag_part_within_token_spec(
        atoms,
        token,
        token.parts[1],
    ) && tag_part_within_token_spec(atoms, token, token.parts[2])
        && tag_parts_have_same_spelling_spec(atoms, token.parts[1], handle)
}

pub open spec fn document_contains_node_spec(document: CstDocumentView, node: CstNodeView) -> bool {
    document.root_token_start <= node.token_start && node.token_end <= document.root_token_end
}

pub open spec fn find_node_document_tail_spec(
    documents: Seq<CstDocumentView>,
    node: CstNodeView,
    index: int,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if fuel == 0 || index < 0 || index >= documents.len() {
        None
    } else if document_contains_node_spec(documents[index], node) {
        Some(index)
    } else {
        find_node_document_tail_spec(documents, node, index + 1, (fuel - 1) as nat)
    }
}

pub open spec fn find_tag_directive_prefix_tail_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
    index: int,
    end: int,
    handle: CompletedTokenPartView,
    fuel: nat,
) -> Option<CompletedTokenPartView>
    decreases fuel,
{
    if fuel == 0 || index < 0 || end > tokens.len() || index >= end {
        None
    } else {
        let token = tokens[index];
        if tag_directive_matches_handle_spec(atoms, token, handle) {
            Some(token.parts[2])
        } else {
            find_tag_directive_prefix_tail_spec(
                atoms,
                tokens,
                index + 1,
                end,
                handle,
                (fuel - 1) as nat,
            )
        }
    }
}

pub open spec fn resolve_non_specific_tag_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
    token_index: u64,
    limits: TagResolutionLimitsView,
) -> Result<ResolvedTagPropertyView, TagResolutionErrorView> {
    if token.start_atom_index >= token.end_atom_index || token.end_atom_index > atoms.len()
        || atoms[token.start_atom_index as int].code_point != 0x21 {
        Err(
            TagResolutionErrorView {
                kind: TagResolutionErrorKind::InvalidTagToken,
                byte_offset: token.byte_start,
            },
        )
    } else {
        let content = atom_range_tag_content_spec(
            atoms,
            token.start_atom_index as int,
            (token.start_atom_index + 1) as int,
            ResolvedTagOrigin::NonSpecificIndicator,
        );
        if effective_tag_code_point_limit_spec(limits) == 0 {
            Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::TagCodePointLimitExceeded,
                    byte_offset: content[0].byte_start,
                },
            )
        } else {
            Ok(ResolvedTagPropertyView { kind: ResolvedTagKind::NonSpecific, token_index, content })
        }
    }
}

pub open spec fn resolve_explicit_tag_token_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
    document: CstDocumentView,
    token_index: u64,
    limits: TagResolutionLimitsView,
) -> Result<ResolvedTagPropertyView, TagResolutionErrorView> {
    if token_index >= tokens.len() {
        Err(
            TagResolutionErrorView {
                kind: TagResolutionErrorKind::InvalidTagToken,
                byte_offset: document.byte_start,
            },
        )
    } else {
        let token = tokens[token_index as int];
        if token.kind == CompletedTokenKind::VerbatimTagProperty {
            if token.parts.len() != 1 || token.parts[0].kind
                != CompletedTokenPartKind::VerbatimTagPayload || !tag_part_within_token_spec(
                atoms,
                token,
                token.parts[0],
            ) {
                Err(
                    TagResolutionErrorView {
                        kind: TagResolutionErrorKind::InvalidTagToken,
                        byte_offset: token.byte_start,
                    },
                )
            } else {
                let payload = token.parts[0];
                finalize_resolved_tag_property_spec(
                    atom_range_tag_content_spec(
                        atoms,
                        payload.start_atom_index as int,
                        payload.end_atom_index as int,
                        ResolvedTagOrigin::VerbatimPayload,
                    ),
                    token_index,
                    token.byte_start,
                    limits,
                )
            }
        } else if token.kind != CompletedTokenKind::TagProperty {
            Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::InvalidTagToken,
                    byte_offset: token.byte_start,
                },
            )
        } else if token.parts.len() == 0 {
            resolve_non_specific_tag_spec(atoms, token, token_index, limits)
        } else if token.parts.len() != 2 || token.parts[0].kind != CompletedTokenPartKind::TagHandle
            || token.parts[1].kind != CompletedTokenPartKind::TagSuffix
            || !tag_part_within_token_spec(atoms, token, token.parts[0])
            || !tag_part_within_token_spec(atoms, token, token.parts[1]) {
            Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::InvalidTagToken,
                    byte_offset: token.byte_start,
                },
            )
        } else if document.directive_start > document.directive_end || document.directive_end
            > tokens.len() {
            Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::InvalidTagToken,
                    byte_offset: token.byte_start,
                },
            )
        } else {
            let handle = token.parts[0];
            let suffix = token.parts[1];
            let directive_prefix = find_tag_directive_prefix_tail_spec(
                atoms,
                tokens,
                document.directive_start as int,
                document.directive_end as int,
                handle,
                (document.directive_end - document.directive_start) as nat,
            );
            let prefix = match directive_prefix {
                Some(part) => atom_range_tag_content_spec(
                    atoms,
                    part.start_atom_index as int,
                    part.end_atom_index as int,
                    ResolvedTagOrigin::DirectivePrefix,
                ),
                None => if primary_tag_handle_spec(atoms, handle) {
                    generated_tag_content_spec(
                        seq![0x21u32],
                        token.start_atom_index,
                        token.byte_start,
                        ResolvedTagOrigin::DefaultPrimaryPrefix,
                    )
                } else if secondary_tag_handle_spec(atoms, handle) {
                    generated_tag_content_spec(
                        default_secondary_prefix_code_points_spec(),
                        token.start_atom_index,
                        token.byte_start,
                        ResolvedTagOrigin::DefaultSecondaryPrefix,
                    )
                } else {
                    Seq::empty()
                },
            };
            if directive_prefix.is_none() && !primary_tag_handle_spec(atoms, handle)
                && !secondary_tag_handle_spec(atoms, handle) {
                Err(
                    TagResolutionErrorView {
                        kind: TagResolutionErrorKind::UndeclaredTagHandle,
                        byte_offset: handle.byte_start,
                    },
                )
            } else {
                finalize_resolved_tag_property_spec(
                    prefix + atom_range_tag_content_spec(
                        atoms,
                        suffix.start_atom_index as int,
                        suffix.end_atom_index as int,
                        ResolvedTagOrigin::TagSuffix,
                    ),
                    token_index,
                    token.byte_start,
                    limits,
                )
            }
        }
    }
}

pub open spec fn tag_resolution_inputs_match_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
) -> bool {
    completed.profile_version == atomized.profile_version && completed.input_transformation_version
        == atomized.transformation_version && completed.transformation_version
        == crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION && completed.source_len_bytes
        == atomized.source_len_bytes && completed.bom_bytes == atomized.bom_bytes
        && completed.input_atom_count == atomized.atoms.len() && cst.profile_version
        == completed.profile_version && cst.input_token_transformation_version
        == completed.transformation_version && cst.transformation_version
        == crate::cst::CST_TRANSFORMATION_VERSION && cst.source_len_bytes
        == completed.source_len_bytes && cst.input_token_count == completed.tokens.len()
}

pub open spec fn resolve_profile1_node_tag_property_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    node_index: u64,
    limits: TagResolutionLimitsView,
) -> Result<Option<ResolvedTagPropertyView>, TagResolutionErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            TagResolutionErrorView {
                kind: TagResolutionErrorKind::InputCompletedTokenMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() {
        Err(
            TagResolutionErrorView {
                kind: TagResolutionErrorKind::InputCstMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if node_index >= cst.nodes.len() {
        Err(
            TagResolutionErrorView {
                kind: TagResolutionErrorKind::NodeIndexOutOfRange,
                byte_offset: atomized.source_len_bytes,
            },
        )
    } else {
        let node = cst.nodes[node_index as int];
        match find_node_document_tail_spec(cst.documents, node, 0, cst.documents.len() as nat) {
            None => Err(
                TagResolutionErrorView {
                    kind: TagResolutionErrorKind::DocumentNotFound,
                    byte_offset: node.byte_start,
                },
            ),
            Some(document_index) => match node.tag_property_token {
                None => Ok(None),
                Some(token_index) => match resolve_explicit_tag_token_spec(
                    atomized.atoms,
                    completed.tokens,
                    cst.documents[document_index],
                    token_index,
                    limits,
                ) {
                    Ok(tag) => Ok(Some(tag)),
                    Err(error) => Err(error),
                },
            },
        }
    }
}

fn tag_part_range(atoms: &[LexicalAtom], part: &CompletedTokenPart) -> (valid: bool)
    ensures
        valid == tag_part_range_spec(crate::atom::lexical_atom_views_spec(atoms@), part@),
{
    let start = part.start_atom_index();
    let end = part.end_atom_index();
    let valid = start < end && end <= atoms.len() as u64 && part.byte_start()
        == atoms[start as usize].span().start().byte_offset() && part.byte_end() == atoms[(end
        - 1) as usize].span().end().byte_offset();
    proof {
        reveal(tag_part_range_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    valid
}

fn tag_part_within_token(
    atoms: &[LexicalAtom],
    token: &CompletedToken,
    part: &CompletedTokenPart,
) -> (valid: bool)
    ensures
        valid == tag_part_within_token_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            token@,
            part@,
        ),
{
    let valid = tag_part_range(atoms, part) && token.start_atom_index() <= part.start_atom_index()
        && part.end_atom_index() <= token.end_atom_index() && token.byte_start()
        <= part.byte_start() && part.byte_end() <= token.byte_end();
    proof {
        reveal(tag_part_within_token_spec);
    }
    valid
}

fn tag_parts_have_same_spelling(
    atoms: &[LexicalAtom],
    left: &CompletedTokenPart,
    right: &CompletedTokenPart,
) -> (same: bool)
    ensures
        same == tag_parts_have_same_spelling_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            left@,
            right@,
        ),
{
    let left_start_atom = left.start_atom_index();
    let left_end_atom = left.end_atom_index();
    let right_start_atom = right.start_atom_index();
    let right_end_atom = right.end_atom_index();
    if !tag_part_range(atoms, left) || !tag_part_range(atoms, right) {
        proof {
            reveal(tag_parts_have_same_spelling_spec);
        }
        return false;
    }
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost left_points = tag_part_code_points_spec(views, left@);
    let ghost right_points = tag_part_code_points_spec(views, right@);
    assert(left_start_atom < left_end_atom);
    assert(right_start_atom < right_end_atom);
    assert(left_end_atom <= atoms.len() as u64);
    assert(right_end_atom <= atoms.len() as u64);
    assert(left_start_atom <= usize::MAX as u64);
    assert(right_start_atom <= usize::MAX as u64);
    let left_start = left_start_atom as usize;
    let right_start = right_start_atom as usize;
    let left_len = (left_end_atom - left_start_atom) as usize;
    let right_len = (right_end_atom - right_start_atom) as usize;
    assert(left_start <= atoms.len());
    assert(right_start <= atoms.len());
    assert(left_len <= atoms.len() - left_start);
    assert(right_len <= atoms.len() - right_start);
    proof {
        reveal(tag_part_code_points_spec);
        assert(left_start_atom as int == left_start as int);
        assert(right_start_atom as int == right_start as int);
        assert(left_points.len() == left_len);
        assert(right_points.len() == right_len);
    }
    if left_len != right_len {
        proof {
            reveal(tag_parts_have_same_spelling_spec);
            assert(left_points.len() != right_points.len());
            assert(left_points != right_points);
        }
        return false;
    }
    let mut offset = 0usize;
    while offset < left_len
        invariant
            left_start <= atoms@.len(),
            right_start <= atoms@.len(),
            atoms@.len() <= usize::MAX,
            left_len <= atoms@.len() - left_start,
            right_len <= atoms@.len() - right_start,
            left_len == right_len,
            offset <= left_len,
            left_start_atom == left@.start_atom_index,
            right_start_atom == right@.start_atom_index,
            left_start_atom as int == left_start as int,
            right_start_atom as int == right_start as int,
            views == crate::atom::lexical_atom_views_spec(atoms@),
            left_points == tag_part_code_points_spec(views, left@),
            right_points == tag_part_code_points_spec(views, right@),
            left_points.len() == left_len,
            right_points.len() == right_len,
            forall|prior: int|
                0 <= prior < offset ==> #[trigger] left_points[prior] == right_points[prior],
        decreases left_len - offset,
    {
        assert(left_start <= usize::MAX - offset);
        assert(right_start <= usize::MAX - offset);
        assert(left_start + offset < atoms@.len());
        assert(right_start + offset < atoms@.len());
        assert((left_start + offset) as int == left_start as int + offset as int);
        assert((right_start + offset) as int == right_start as int + offset as int);
        assert(views[(left_start + offset) as int] == atoms[(left_start + offset) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        assert(views[(right_start + offset) as int] == atoms[(right_start + offset) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        proof {
            reveal(tag_part_code_points_spec);
            assert(left@.start_atom_index as int + offset as int == (left_start + offset) as int);
            assert(right@.start_atom_index as int + offset as int == (right_start + offset) as int);
            assert(left_points[offset as int] == views[(left_start + offset) as int].code_point);
            assert(right_points[offset as int] == views[(right_start + offset) as int].code_point);
        }
        if atoms[left_start + offset].code_point() != atoms[right_start + offset].code_point() {
            proof {
                reveal(tag_parts_have_same_spelling_spec);
                assert(left_points[offset as int] != right_points[offset as int]);
                assert(left_points != right_points);
            }
            return false;
        }
        offset += 1;
    }
    proof {
        reveal(tag_parts_have_same_spelling_spec);
        assert(left_points =~= right_points);
    }
    true
}

fn primary_tag_handle(atoms: &[LexicalAtom], handle: &CompletedTokenPart) -> (primary: bool)
    ensures
        primary == primary_tag_handle_spec(crate::atom::lexical_atom_views_spec(atoms@), handle@),
{
    let valid = tag_part_range(atoms, handle);
    let start = handle.start_atom_index();
    let end = handle.end_atom_index();
    let primary = valid && end - start == 1 && atoms[start as usize].code_point() == 0x21;
    proof {
        reveal(primary_tag_handle_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    primary
}

fn secondary_tag_handle(atoms: &[LexicalAtom], handle: &CompletedTokenPart) -> (secondary: bool)
    ensures
        secondary == secondary_tag_handle_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            handle@,
        ),
{
    let valid = tag_part_range(atoms, handle);
    let start = handle.start_atom_index();
    let end = handle.end_atom_index();
    let secondary = valid && end - start == 2 && atoms[start as usize].code_point() == 0x21
        && atoms[start as usize + 1].code_point() == 0x21;
    proof {
        reveal(secondary_tag_handle_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    secondary
}

fn document_contains_node(document: &CstDocument, node: &CstNode) -> (contains: bool)
    ensures
        contains == document_contains_node_spec(document@, node@),
{
    let contains = document.root_token_start() <= node.token_start() && node.token_end()
        <= document.root_token_end();
    proof {
        reveal(document_contains_node_spec);
    }
    contains
}

fn find_node_document(documents: &[CstDocument], node: &CstNode) -> (found: Option<usize>)
    ensures
        find_node_document_tail_spec(
            crate::cst::cst_document_views_spec(documents@),
            node@,
            0,
            documents@.len() as nat,
        ) == match found {
            Some(index) => Some(index as int),
            None => None,
        },
        match found {
            Some(index) => index < documents@.len(),
            None => true,
        },
{
    let ghost views = crate::cst::cst_document_views_spec(documents@);
    let ghost expected = find_node_document_tail_spec(views, node@, 0, documents@.len() as nat);
    let mut index = 0usize;
    let mut _fuel = documents.len();
    while index < documents.len()
        invariant
            index <= documents@.len(),
            _fuel == documents@.len() - index,
            views == crate::cst::cst_document_views_spec(documents@),
            expected == find_node_document_tail_spec(views, node@, 0, documents@.len() as nat),
            expected == find_node_document_tail_spec(views, node@, index as int, _fuel as nat),
        decreases documents.len() - index,
    {
        assert(views[index as int] == documents[index as int]@) by {
            reveal(crate::cst::cst_document_views_spec);
        }
        if document_contains_node(&documents[index], node) {
            proof {
                reveal(find_node_document_tail_spec);
                assert(expected == Some(index as int));
            }
            return Some(index);
        }
        proof {
            reveal(find_node_document_tail_spec);
        }
        index += 1;
        _fuel -= 1;
    }
    proof {
        reveal(find_node_document_tail_spec);
    }
    None
}

fn tag_directive_matches_handle(
    atoms: &[LexicalAtom],
    token: &CompletedToken,
    handle: &CompletedTokenPart,
) -> (matches: bool)
    ensures
        matches == tag_directive_matches_handle_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            token@,
            handle@,
        ),
{
    let parts = token.parts();
    proof {
        crate::token::lemma_completed_token_part_views_len(parts@);
    }
    if token.kind() != CompletedTokenKind::TagDirective || parts.len() != 3 {
        proof {
            reveal(tag_directive_matches_handle_spec);
        }
        return false;
    }
    proof {
        crate::token::lemma_completed_token_part_view_at(parts@, 1);
        crate::token::lemma_completed_token_part_view_at(parts@, 2);
        assert(token@.parts[1] == parts@[1]@);
        assert(token@.parts[2] == parts@[2]@);
    }
    let matches = parts[1].kind() == CompletedTokenPartKind::TagHandle && parts[2].kind()
        == CompletedTokenPartKind::TagPrefix && tag_part_within_token(atoms, token, &parts[1])
        && tag_part_within_token(atoms, token, &parts[2]) && tag_parts_have_same_spelling(
        atoms,
        &parts[1],
        handle,
    );
    proof {
        reveal(tag_directive_matches_handle_spec);
    }
    matches
}

fn find_tag_directive_prefix(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    start: usize,
    end: usize,
    handle: &CompletedTokenPart,
) -> (found: Option<usize>)
    requires
        start <= end <= tokens@.len(),
    ensures
        find_tag_directive_prefix_tail_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::token::completed_token_views_spec(tokens@),
            start as int,
            end as int,
            handle@,
            (end - start) as nat,
        ) == match found {
            Some(index) => Some(
                crate::token::completed_token_views_spec(tokens@)[index as int].parts[2],
            ),
            None => None,
        },
        match found {
            Some(index) => start <= index < end && tokens@[index as int]@.parts.len() == 3
                && tag_directive_matches_handle_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                tokens@[index as int]@,
                handle@,
            ),
            None => true,
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost expected = find_tag_directive_prefix_tail_spec(
        atom_views,
        token_views,
        start as int,
        end as int,
        handle@,
        (end - start) as nat,
    );
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
    }
    let mut index = start;
    let mut _fuel = end - start;
    while index < end
        invariant
            start <= index <= end <= tokens@.len(),
            token_views.len() == tokens@.len(),
            _fuel == end - index,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            token_views == crate::token::completed_token_views_spec(tokens@),
            expected == find_tag_directive_prefix_tail_spec(
                atom_views,
                token_views,
                start as int,
                end as int,
                handle@,
                (end - start) as nat,
            ),
            expected == find_tag_directive_prefix_tail_spec(
                atom_views,
                token_views,
                index as int,
                end as int,
                handle@,
                _fuel as nat,
            ),
        decreases end - index,
    {
        assert(token_views[index as int] == tokens[index as int]@) by {
            crate::token::lemma_completed_token_view_at(tokens@, index as int);
        }
        let token = &tokens[index];
        if tag_directive_matches_handle(atoms, token, handle) {
            let _parts = token.parts();
            proof {
                crate::token::lemma_completed_token_part_views_len(_parts@);
                assert(_parts.len() == 3);
                crate::token::lemma_completed_token_part_view_at(_parts@, 1);
                crate::token::lemma_completed_token_part_view_at(_parts@, 2);
                reveal(find_tag_directive_prefix_tail_spec);
                assert(expected == Some(token_views[index as int].parts[2]));
                assert(tag_directive_matches_handle_spec(atom_views, token@, handle@));
            }
            return Some(index);
        }
        proof {
            reveal(find_tag_directive_prefix_tail_spec);
        }
        index += 1;
        _fuel -= 1;
    }
    proof {
        reveal(find_tag_directive_prefix_tail_spec);
    }
    None
}

fn append_atom_range_tag_content(
    output: &mut Vec<ResolvedTagCodePoint>,
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
    origin: ResolvedTagOrigin,
)
    requires
        start <= end <= atoms@.len(),
        end <= u64::MAX,
    ensures
        resolved_tag_code_point_views_spec(final(output)@) == resolved_tag_code_point_views_spec(
            old(output)@,
        ) + atom_range_tag_content_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            origin,
        ),
{
    let ghost original = old(output)@;
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            end <= u64::MAX,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            resolved_tag_code_point_views_spec(output@) == resolved_tag_code_point_views_spec(
                original,
            ) + atom_range_tag_content_spec(atom_views, start as int, index as int, origin),
        decreases end - index,
    {
        assert(atom_views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let atom = &atoms[index];
        let point = ResolvedTagCodePoint::new(
            atom.code_point(),
            index as u64,
            atom.span().start().byte_offset(),
            atom.span().end().byte_offset(),
            origin,
        );
        proof {
            lemma_resolved_tag_code_point_views_push(output@, point);
            reveal(atom_range_tag_content_spec);
            assert(atom_range_tag_content_spec(atom_views, start as int, (index + 1) as int, origin)
                =~= atom_range_tag_content_spec(
                atom_views,
                start as int,
                index as int,
                origin,
            ).push(point@));
        }
        output.push(point);
        index += 1;
    }
}

fn append_generated_tag_content(
    output: &mut Vec<ResolvedTagCodePoint>,
    code_points: &[u32],
    source_atom_index: u64,
    byte_offset: u64,
    origin: ResolvedTagOrigin,
)
    ensures
        resolved_tag_code_point_views_spec(final(output)@) == resolved_tag_code_point_views_spec(
            old(output)@,
        ) + generated_tag_content_spec(code_points@, source_atom_index, byte_offset, origin),
{
    let ghost original = old(output)@;
    let mut index = 0usize;
    while index < code_points.len()
        invariant
            index <= code_points@.len(),
            resolved_tag_code_point_views_spec(output@) == resolved_tag_code_point_views_spec(
                original,
            ) + generated_tag_content_spec(
                code_points@.subrange(0, index as int),
                source_atom_index,
                byte_offset,
                origin,
            ),
        decreases code_points.len() - index,
    {
        let point = ResolvedTagCodePoint::new(
            code_points[index],
            source_atom_index,
            byte_offset,
            byte_offset,
            origin,
        );
        proof {
            lemma_resolved_tag_code_point_views_push(output@, point);
            reveal(generated_tag_content_spec);
            assert(code_points@.subrange(0, index as int + 1) =~= code_points@.subrange(
                0,
                index as int,
            ).push(code_points@[index as int]));
            assert(generated_tag_content_spec(
                code_points@.subrange(0, index as int + 1),
                source_atom_index,
                byte_offset,
                origin,
            ) =~= generated_tag_content_spec(
                code_points@.subrange(0, index as int),
                source_atom_index,
                byte_offset,
                origin,
            ).push(point@));
        }
        output.push(point);
        index += 1;
    }
    proof {
        assert(code_points@.subrange(0, code_points@.len() as int) =~= code_points@);
    }
}

#[expect(
    clippy::vec_init_then_push,
    reason = "stepwise construction mirrors the exact Verus sequence used by the tag proof"
)]
fn default_secondary_prefix() -> (code_points: Vec<u32>)
    ensures
        code_points@ == default_secondary_prefix_code_points_spec(),
{
    let mut code_points = Vec::new();
    code_points.push(0x74);
    code_points.push(0x61);
    code_points.push(0x67);
    code_points.push(0x3a);
    code_points.push(0x79);
    code_points.push(0x61);
    code_points.push(0x6d);
    code_points.push(0x6c);
    code_points.push(0x2e);
    code_points.push(0x6f);
    code_points.push(0x72);
    code_points.push(0x67);
    code_points.push(0x2c);
    code_points.push(0x32);
    code_points.push(0x30);
    code_points.push(0x30);
    code_points.push(0x32);
    code_points.push(0x3a);
    proof {
        reveal(default_secondary_prefix_code_points_spec);
    }
    code_points
}

#[expect(clippy::manual_range_contains, reason = "arithmetic spelling mirrors the Verus specification and proof obligations")]  // Mirrors the arithmetic Verus specification directly.
fn ascii_tag_scheme_first(code_point: u32) -> (valid: bool)
    ensures
        valid == ascii_tag_scheme_first_spec(code_point),
{
    (0x41 <= code_point && code_point <= 0x5a) || (0x61 <= code_point && code_point <= 0x7a)
}

#[expect(clippy::manual_range_contains, reason = "arithmetic spelling mirrors the Verus specification and proof obligations")]  // Mirrors the arithmetic Verus specification directly.
fn ascii_tag_scheme_continuation(code_point: u32) -> (valid: bool)
    ensures
        valid == ascii_tag_scheme_continuation_spec(code_point),
{
    ascii_tag_scheme_first(code_point) || (0x30 <= code_point && code_point <= 0x39) || code_point
        == 0x2b || code_point == 0x2d || code_point == 0x2e
}

fn global_tag_uri(content: &[ResolvedTagCodePoint]) -> (valid: bool)
    ensures
        valid == global_tag_uri_spec(resolved_tag_code_point_views_spec(content@)),
{
    if content.len() < 2 || !ascii_tag_scheme_first(content[0].code_point()) {
        proof {
            reveal(global_tag_uri_spec);
            reveal(resolved_tag_code_point_views_spec);
        }
        return false;
    }
    let ghost views = resolved_tag_code_point_views_spec(content@);
    let ghost expected = tag_uri_scheme_tail_spec(views, 1, content@.len() as nat);
    let mut index = 1usize;
    let mut _fuel = content.len();
    while index < content.len() && content[index].code_point() != 0x3a
        invariant
            1 <= index <= content@.len(),
            _fuel >= content@.len() - index + 1,
            views == resolved_tag_code_point_views_spec(content@),
            expected == tag_uri_scheme_tail_spec(views, 1, content@.len() as nat),
            expected == tag_uri_scheme_tail_spec(views, index as int, _fuel as nat),
        decreases content.len() - index,
    {
        assert(views[index as int] == content[index as int]@) by {
            reveal(resolved_tag_code_point_views_spec);
        }
        if !ascii_tag_scheme_continuation(content[index].code_point()) {
            proof {
                assert(!ascii_tag_scheme_continuation_spec(views[index as int].code_point));
                reveal(tag_uri_scheme_tail_spec);
                assert(!tag_uri_scheme_tail_spec(views, index as int, _fuel as nat));
                assert(!expected);
                assert(expected == tag_uri_scheme_tail_spec(views, 1, content@.len() as nat));
                assert(views.len() == content@.len()) by {
                    reveal(resolved_tag_code_point_views_spec);
                }
                assert(!tag_uri_scheme_tail_spec(views, 1, views.len() as nat));
                reveal(global_tag_uri_spec);
                assert(!global_tag_uri_spec(views));
            }
            return false;
        }
        proof {
            reveal(tag_uri_scheme_tail_spec);
        }
        index += 1;
        _fuel -= 1;
    }
    proof {
        reveal(tag_uri_scheme_tail_spec);
        reveal(global_tag_uri_spec);
        if index < content@.len() {
            assert(views[index as int] == content[index as int]@) by {
                reveal(resolved_tag_code_point_views_spec);
            }
        }
    }
    index < content.len()
}

fn finalize_resolved_tag_property(
    content: Vec<ResolvedTagCodePoint>,
    token_index: u64,
    tag_byte_start: u64,
    limits: TagResolutionLimits,
) -> (result: Result<ResolvedTagProperty, TagResolutionError>)
    ensures
        finalize_resolved_tag_property_spec(
            resolved_tag_code_point_views_spec(content@),
            token_index,
            tag_byte_start,
            limits@,
        ) == match result {
            Ok(tag) => Ok(tag@),
            Err(error) => Err(error@),
        },
{
    if content.is_empty() {
        let error = TagResolutionError::at(TagResolutionErrorKind::InvalidTagToken, tag_byte_start);
        proof {
            reveal(finalize_resolved_tag_property_spec);
            reveal(resolved_tag_code_point_views_spec);
        }
        return Err(error);
    }
    assert(content@.len() > 0);
    assert(resolved_tag_code_point_views_spec(content@)[0] == content[0]@) by {
        reveal(resolved_tag_code_point_views_spec);
    }
    let local = content[0].code_point() == 0x21;
    if local && content.len() == 1 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidLocalTag,
            content[0].byte_start(),
        );
        proof {
            reveal(finalize_resolved_tag_property_spec);
        }
        return Err(error);
    }
    if !local && !global_tag_uri(content.as_slice()) {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidGlobalTagUri,
            content[0].byte_start(),
        );
        proof {
            reveal(finalize_resolved_tag_property_spec);
        }
        return Err(error);
    }
    let effective_limit = if limits.max_tag_code_points < MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS {
        limits.max_tag_code_points
    } else {
        MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS
    };
    if content.len() as u64 > effective_limit {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::TagCodePointLimitExceeded,
            content[effective_limit as usize].byte_start(),
        );
        proof {
            reveal(finalize_resolved_tag_property_spec);
            reveal(effective_tag_code_point_limit_spec);
            reveal(resolved_tag_code_point_views_spec);
        }
        return Err(error);
    }
    let kind = if local {
        ResolvedTagKind::Local
    } else {
        ResolvedTagKind::Global
    };
    let tag = ResolvedTagProperty::new(kind, token_index, content);
    proof {
        reveal(finalize_resolved_tag_property_spec);
        reveal(effective_tag_code_point_limit_spec);
    }
    Ok(tag)
}

fn resolve_non_specific_tag(
    atoms: &[LexicalAtom],
    token: &CompletedToken,
    token_index: u64,
    limits: TagResolutionLimits,
) -> (result: Result<ResolvedTagProperty, TagResolutionError>)
    ensures
        resolve_non_specific_tag_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            token@,
            token_index,
            limits@,
        ) == match result {
            Ok(tag) => Ok(tag@),
            Err(error) => Err(error@),
        },
{
    let start = token.start_atom_index();
    let end = token.end_atom_index();
    let byte_start = token.byte_start();
    if start >= end || end > atoms.len() as u64 || atoms[start as usize].code_point() != 0x21 {
        let error = TagResolutionError::at(TagResolutionErrorKind::InvalidTagToken, byte_start);
        proof {
            reveal(resolve_non_specific_tag_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return Err(error);
    }
    let mut content = Vec::new();
    append_atom_range_tag_content(
        &mut content,
        atoms,
        start as usize,
        start as usize + 1,
        ResolvedTagOrigin::NonSpecificIndicator,
    );
    let effective_limit = if limits.max_tag_code_points < MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS {
        limits.max_tag_code_points
    } else {
        MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS
    };
    if effective_limit == 0 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::TagCodePointLimitExceeded,
            content[0].byte_start(),
        );
        proof {
            reveal(resolve_non_specific_tag_spec);
            reveal(effective_tag_code_point_limit_spec);
        }
        return Err(error);
    }
    let tag = ResolvedTagProperty::new(ResolvedTagKind::NonSpecific, token_index, content);
    proof {
        reveal(resolve_non_specific_tag_spec);
        reveal(effective_tag_code_point_limit_spec);
    }
    Ok(tag)
}

#[expect(
    clippy::vec_init_then_push,
    reason = "stepwise primary-prefix construction preserves the exact Verus sequence proof"
)]
fn resolve_explicit_tag_token(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    document: &CstDocument,
    token_index: u64,
    limits: TagResolutionLimits,
) -> (result: Result<ResolvedTagProperty, TagResolutionError>)
    ensures
        resolve_explicit_tag_token_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::token::completed_token_views_spec(tokens@),
            document@,
            token_index,
            limits@,
        ) == match result {
            Ok(tag) => Ok(tag@),
            Err(error) => Err(error@),
        },
{
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
    }
    if token_index >= tokens.len() as u64 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidTagToken,
            document.byte_start(),
        );
        proof {
            reveal(resolve_explicit_tag_token_spec);
            crate::token::lemma_completed_token_views_len(tokens@);
        }
        return Err(error);
    }
    let index = token_index as usize;
    let token = &tokens[index];
    let token_kind = token.kind();
    let token_byte_start = token.byte_start();
    let parts = token.parts();
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, index as int);
        crate::token::lemma_completed_token_part_views_len(parts@);
        assert(crate::token::completed_token_views_spec(tokens@)[index as int] == token@);
        assert(token@.parts.len() == parts@.len());
    }
    if token_kind == CompletedTokenKind::VerbatimTagProperty {
        if parts.len() != 1 {
            let error = TagResolutionError::at(
                TagResolutionErrorKind::InvalidTagToken,
                token_byte_start,
            );
            proof {
                reveal(resolve_explicit_tag_token_spec);
            }
            return Err(error);
        }
        proof {
            crate::token::lemma_completed_token_part_view_at(parts@, 0);
            assert(token@.parts[0] == parts@[0]@);
        }
        if parts[0].kind() != CompletedTokenPartKind::VerbatimTagPayload || !tag_part_within_token(
            atoms,
            token,
            &parts[0],
        ) {
            let error = TagResolutionError::at(
                TagResolutionErrorKind::InvalidTagToken,
                token_byte_start,
            );
            proof {
                reveal(resolve_explicit_tag_token_spec);
            }
            return Err(error);
        }
        let payload_start_atom = parts[0].start_atom_index();
        let payload_end_atom = parts[0].end_atom_index();
        assert(payload_start_atom < payload_end_atom);
        assert(payload_end_atom <= atoms.len() as u64);
        assert(payload_start_atom <= usize::MAX as u64);
        assert(payload_end_atom <= usize::MAX as u64);
        let payload_start = payload_start_atom as usize;
        let payload_end = payload_end_atom as usize;
        assert(payload_start <= payload_end <= atoms@.len());
        let mut content = Vec::new();
        append_atom_range_tag_content(
            &mut content,
            atoms,
            payload_start,
            payload_end,
            ResolvedTagOrigin::VerbatimPayload,
        );
        let result = finalize_resolved_tag_property(content, token_index, token_byte_start, limits);
        proof {
            reveal(resolve_explicit_tag_token_spec);
        }
        return result;
    }
    if token_kind != CompletedTokenKind::TagProperty {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidTagToken,
            token_byte_start,
        );
        proof {
            reveal(resolve_explicit_tag_token_spec);
        }
        return Err(error);
    }
    if parts.is_empty() {
        let result = resolve_non_specific_tag(atoms, token, token_index, limits);
        proof {
            reveal(resolve_explicit_tag_token_spec);
        }
        return result;
    }
    if parts.len() != 2 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidTagToken,
            token_byte_start,
        );
        proof {
            reveal(resolve_explicit_tag_token_spec);
        }
        return Err(error);
    }
    proof {
        crate::token::lemma_completed_token_part_view_at(parts@, 0);
        crate::token::lemma_completed_token_part_view_at(parts@, 1);
        assert(token@.parts[0] == parts@[0]@);
        assert(token@.parts[1] == parts@[1]@);
    }
    if parts[0].kind() != CompletedTokenPartKind::TagHandle || parts[1].kind()
        != CompletedTokenPartKind::TagSuffix || !tag_part_within_token(atoms, token, &parts[0])
        || !tag_part_within_token(atoms, token, &parts[1]) {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidTagToken,
            token_byte_start,
        );
        proof {
            reveal(resolve_explicit_tag_token_spec);
        }
        return Err(error);
    }
    let directive_start = document.directive_start();
    let directive_end = document.directive_end();
    if directive_start > directive_end || directive_end > tokens.len() as u64 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InvalidTagToken,
            token_byte_start,
        );
        proof {
            reveal(resolve_explicit_tag_token_spec);
            crate::token::lemma_completed_token_views_len(tokens@);
        }
        return Err(error);
    }
    let handle = &parts[0];
    let suffix = &parts[1];
    let directive = find_tag_directive_prefix(
        atoms,
        tokens,
        directive_start as usize,
        directive_end as usize,
        handle,
    );
    let mut content = Vec::new();
    match directive {
        Some(directive_index) => {
            let directive_token = &tokens[directive_index];
            let directive_parts = directive_token.parts();
            proof {
                crate::token::lemma_completed_token_view_at(tokens@, directive_index as int);
                crate::token::lemma_completed_token_part_views_len(directive_parts@);
                crate::token::lemma_completed_token_part_view_at(directive_parts@, 2);
                assert(directive_parts.len() == 3);
            }
            let prefix = &directive_parts[2];
            let _prefix_valid = tag_part_within_token(atoms, directive_token, prefix);
            assert(_prefix_valid);
            let prefix_start_atom = prefix.start_atom_index();
            let prefix_end_atom = prefix.end_atom_index();
            assert(prefix_start_atom < prefix_end_atom);
            assert(prefix_end_atom <= atoms.len() as u64);
            assert(prefix_start_atom <= usize::MAX as u64);
            assert(prefix_end_atom <= usize::MAX as u64);
            append_atom_range_tag_content(
                &mut content,
                atoms,
                prefix_start_atom as usize,
                prefix_end_atom as usize,
                ResolvedTagOrigin::DirectivePrefix,
            );
        },
        None => {
            if primary_tag_handle(atoms, handle) {
                let mut prefix = Vec::new();
                prefix.push(0x21u32);
                append_generated_tag_content(
                    &mut content,
                    prefix.as_slice(),
                    token.start_atom_index(),
                    token_byte_start,
                    ResolvedTagOrigin::DefaultPrimaryPrefix,
                );
            } else if secondary_tag_handle(atoms, handle) {
                let prefix = default_secondary_prefix();
                append_generated_tag_content(
                    &mut content,
                    prefix.as_slice(),
                    token.start_atom_index(),
                    token_byte_start,
                    ResolvedTagOrigin::DefaultSecondaryPrefix,
                );
            } else {
                let error = TagResolutionError::at(
                    TagResolutionErrorKind::UndeclaredTagHandle,
                    handle.byte_start(),
                );
                proof {
                    reveal(resolve_explicit_tag_token_spec);
                }
                return Err(error);
            }
        },
    }
    let suffix_start_atom = suffix.start_atom_index();
    let suffix_end_atom = suffix.end_atom_index();
    assert(suffix_start_atom < suffix_end_atom);
    assert(suffix_end_atom <= atoms.len() as u64);
    assert(suffix_start_atom <= usize::MAX as u64);
    assert(suffix_end_atom <= usize::MAX as u64);
    append_atom_range_tag_content(
        &mut content,
        atoms,
        suffix_start_atom as usize,
        suffix_end_atom as usize,
        ResolvedTagOrigin::TagSuffix,
    );
    let result = finalize_resolved_tag_property(content, token_index, token_byte_start, limits);
    proof {
        reveal(resolve_explicit_tag_token_spec);
    }
    result
}

pub fn resolve_profile1_node_tag_property(
    atomized: &AtomizedSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    node_index: u64,
    limits: TagResolutionLimits,
) -> (result: Result<Option<ResolvedTagProperty>, TagResolutionError>)
    ensures
        resolve_profile1_node_tag_property_spec(atomized@, completed@, cst@, node_index, limits@)
            == match result {
            Ok(Some(tag)) => Ok(Some(tag@)),
            Ok(None) => Ok(None),
            Err(error) => Err(error@),
        },
{
    let bom_bytes = atomized.bom_bytes();
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let documents = cst.documents();
    let nodes = cst.nodes();
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::cst::cst_node_views_spec);
        reveal(crate::cst::cst_document_views_spec);
        reveal(crate::atom::lexical_atom_views_spec);
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens.len() == tokens@.len());
        assert(cst@.documents.len() == documents@.len());
        assert(cst@.nodes.len() == nodes@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != bom_bytes
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::InputCompletedTokenMismatch,
            bom_bytes,
        );
        proof {
            reveal(resolve_profile1_node_tag_property_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 {
        let error = TagResolutionError::at(TagResolutionErrorKind::InputCstMismatch, bom_bytes);
        proof {
            reveal(resolve_profile1_node_tag_property_spec);
        }
        return Err(error);
    }
    proof {
        reveal(tag_resolution_inputs_match_spec);
        assert(tag_resolution_inputs_match_spec(atomized@, completed@, cst@));
    }
    if node_index >= nodes.len() as u64 {
        let error = TagResolutionError::at(
            TagResolutionErrorKind::NodeIndexOutOfRange,
            atomized.source_len_bytes(),
        );
        proof {
            reveal(resolve_profile1_node_tag_property_spec);
        }
        return Err(error);
    }
    let node_offset = node_index as usize;
    let node = &nodes[node_offset];
    proof {
        crate::cst::lemma_cst_node_view_at(nodes@, node_offset as int);
        assert(cst@.nodes[node_index as int] == node@);
    }
    let found_document = find_node_document(documents, node);
    let document_index = match found_document {
        Some(index) => index,
        None => {
            let error = TagResolutionError::at(
                TagResolutionErrorKind::DocumentNotFound,
                node.byte_start(),
            );
            proof {
                reveal(resolve_profile1_node_tag_property_spec);
                assert(find_node_document_tail_spec(
                    cst@.documents,
                    node@,
                    0,
                    cst@.documents.len() as nat,
                ) == None);
            }
            return Err(error);
        },
    };
    proof {
        crate::cst::lemma_cst_document_view_at(documents@, document_index as int);
        assert(cst@.documents[document_index as int] == documents@[document_index as int]@);
        assert(find_node_document_tail_spec(cst@.documents, node@, 0, cst@.documents.len() as nat)
            == Some(document_index as int));
    }
    let token_index = match node.tag_property_token() {
        Some(index) => index,
        None => {
            proof {
                reveal(resolve_profile1_node_tag_property_spec);
            }
            return Ok(None);
        },
    };
    let resolved = resolve_explicit_tag_token(
        atoms,
        tokens,
        &documents[document_index],
        token_index,
        limits,
    );
    proof {
        reveal(resolve_profile1_node_tag_property_spec);
    }
    match resolved {
        Ok(tag) => Ok(Some(tag)),
        Err(error) => Err(error),
    }
}

} // verus!
