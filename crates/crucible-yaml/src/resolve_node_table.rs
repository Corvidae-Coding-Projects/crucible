//! Verified semantic node-slot, collection, and alias-redirection table composition.
//!
//! The transformation is nonrecursive and preserves every CST identity and source range. Scalar
//! slots reference the independently verified scalar table, collection slots retain complete
//! resolved collection tags, and alias slots carry explicit redirects without copying targets.
use crate::atom::AtomizedSource;
#[allow(unused_imports)]
use crate::atom::AtomizedSourceView;
use crate::block::BlockScalarSource;
#[allow(unused_imports)]
use crate::block::BlockScalarSourceView;
use crate::cst::{CstNode, CstNodeKind, CstSource};
#[allow(unused_imports)]
use crate::cst::{CstNodeView, CstSourceView};
use crate::plain::PlainScalarSource;
#[allow(unused_imports)]
use crate::plain::PlainScalarSourceView;
use crate::quoted::QuotedScalarSource;
#[allow(unused_imports)]
use crate::quoted::QuotedScalarSourceView;
use crate::resolve_anchor::{
    AliasBinding, AnchorAliasError, AnchorAliasErrorKind, AnchorAliasLimits, AnchorAliasSource,
};
#[allow(unused_imports)]
use crate::resolve_anchor::{AliasBindingView, AnchorAliasLimitsView, AnchorAliasSourceView};
use crate::resolve_collection_tag::{
    resolve_profile1_cst_node_collection_tag, CollectionTagError, CollectionTagErrorKind,
    CollectionTagLimits, ResolvedCollection,
};
#[allow(unused_imports)]
use crate::resolve_collection_tag::{
    CollectionTagErrorView, CollectionTagLimitsView, ResolvedCollectionView,
};
use crate::resolve_scalar_table::{
    SemanticScalarTableError, SemanticScalarTableErrorKind, SemanticScalarTableLimits,
    SemanticScalarTableSource,
};
#[allow(unused_imports)]
use crate::resolve_scalar_table::{SemanticScalarTableLimitsView, SemanticScalarTableSourceView};
use crate::resolve_topology::{
    SemanticTopologyError, SemanticTopologyErrorKind, SemanticTopologyLimits,
    SemanticTopologySource,
};
#[allow(unused_imports)]
use crate::resolve_topology::{SemanticTopologyLimitsView, SemanticTopologySourceView};
use crate::token::CompletedTokenSource;
#[allow(unused_imports)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const SEMANTIC_NODE_TABLE_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_SEMANTIC_COLLECTIONS: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_SEMANTIC_ALIAS_REDIRECTS: u64 =
    crate::resolve_anchor::MAX_PROFILE1_ALIAS_BINDINGS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticNodeTableLimits {
    max_nodes: u64,
    max_collections: u64,
    max_alias_redirects: u64,
    max_collection_tag_code_points: u64,
}

#[verifier::ext_equal]
pub struct SemanticNodeTableLimitsView {
    pub max_nodes: u64,
    pub max_collections: u64,
    pub max_alias_redirects: u64,
    pub max_collection_tag_code_points: u64,
}

impl View for SemanticNodeTableLimits {
    type V = SemanticNodeTableLimitsView;

    closed spec fn view(&self) -> SemanticNodeTableLimitsView {
        SemanticNodeTableLimitsView {
            max_nodes: self.max_nodes,
            max_collections: self.max_collections,
            max_alias_redirects: self.max_alias_redirects,
            max_collection_tag_code_points: self.max_collection_tag_code_points,
        }
    }
}

impl SemanticNodeTableLimits {
    pub fn new(
        max_nodes: u64,
        max_collections: u64,
        max_alias_redirects: u64,
        max_collection_tag_code_points: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (SemanticNodeTableLimitsView {
                max_nodes,
                max_collections,
                max_alias_redirects,
                max_collection_tag_code_points,
            }),
    {
        Self { max_nodes, max_collections, max_alias_redirects, max_collection_tag_code_points }
    }

    pub fn max_nodes(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_nodes,
    {
        self.max_nodes
    }

    pub fn max_collections(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_collections,
    {
        self.max_collections
    }

    pub fn max_alias_redirects(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_alias_redirects,
    {
        self.max_alias_redirects
    }

    pub fn collection_tag_limits(&self) -> (limits: CollectionTagLimits)
        ensures
            limits@ == semantic_node_table_collection_tag_limits_spec(self@),
    {
        CollectionTagLimits::new(self.max_collection_tag_code_points)
    }
}

pub fn canonical_semantic_node_table_limits() -> (limits: SemanticNodeTableLimits)
    ensures
        limits@ == canonical_semantic_node_table_limits_spec(),
{
    SemanticNodeTableLimits::new(
        MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES,
        MAX_PROFILE1_SEMANTIC_COLLECTIONS,
        MAX_PROFILE1_SEMANTIC_ALIAS_REDIRECTS,
        crate::resolve_tag::MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
    )
}

pub open spec fn canonical_semantic_node_table_limits_spec() -> SemanticNodeTableLimitsView {
    SemanticNodeTableLimitsView {
        max_nodes: MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES,
        max_collections: MAX_PROFILE1_SEMANTIC_COLLECTIONS,
        max_alias_redirects: MAX_PROFILE1_SEMANTIC_ALIAS_REDIRECTS,
        max_collection_tag_code_points: crate::resolve_tag::MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
    }
}

pub open spec fn semantic_node_table_collection_tag_limits_spec(
    limits: SemanticNodeTableLimitsView,
) -> CollectionTagLimitsView {
    CollectionTagLimitsView { max_tag_code_points: limits.max_collection_tag_code_points }
}

pub open spec fn semantic_node_table_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn semantic_node_table_effective_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit == semantic_node_table_effective_limit_spec(requested, absolute),
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum SemanticNodeKind {
    Scalar,
    Sequence,
    Mapping,
    Alias,
}

pub open spec fn semantic_node_kind_spec(kind: CstNodeKind) -> SemanticNodeKind {
    match kind {
        CstNodeKind::Empty | CstNodeKind::Scalar => SemanticNodeKind::Scalar,
        CstNodeKind::Sequence => SemanticNodeKind::Sequence,
        CstNodeKind::Mapping => SemanticNodeKind::Mapping,
        CstNodeKind::Alias => SemanticNodeKind::Alias,
    }
}

fn semantic_node_kind(kind: CstNodeKind) -> (semantic: SemanticNodeKind)
    ensures
        semantic == semantic_node_kind_spec(kind),
{
    match kind {
        CstNodeKind::Empty | CstNodeKind::Scalar => SemanticNodeKind::Scalar,
        CstNodeKind::Sequence => SemanticNodeKind::Sequence,
        CstNodeKind::Mapping => SemanticNodeKind::Mapping,
        CstNodeKind::Alias => SemanticNodeKind::Alias,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticNodeSlot {
    cst_node_index: u64,
    kind: SemanticNodeKind,
    token_start: u64,
    token_end: u64,
    byte_start: u64,
    byte_end: u64,
    anchor_property_token: Option<u64>,
    tag_property_token: Option<u64>,
    edge_start: u64,
    edge_end: u64,
    value_index: Option<u64>,
    alias_target_node_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct SemanticNodeSlotView {
    pub cst_node_index: u64,
    pub kind: SemanticNodeKind,
    pub token_start: u64,
    pub token_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub anchor_property_token: Option<u64>,
    pub tag_property_token: Option<u64>,
    pub edge_start: u64,
    pub edge_end: u64,
    pub value_index: Option<u64>,
    pub alias_target_node_index: Option<u64>,
}

impl View for SemanticNodeSlot {
    type V = SemanticNodeSlotView;

    closed spec fn view(&self) -> SemanticNodeSlotView {
        SemanticNodeSlotView {
            cst_node_index: self.cst_node_index,
            kind: self.kind,
            token_start: self.token_start,
            token_end: self.token_end,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            anchor_property_token: self.anchor_property_token,
            tag_property_token: self.tag_property_token,
            edge_start: self.edge_start,
            edge_end: self.edge_end,
            value_index: self.value_index,
            alias_target_node_index: self.alias_target_node_index,
        }
    }
}

pub open spec fn semantic_node_slot_spec(
    node: CstNodeView,
    cst_node_index: u64,
    value_index: Option<u64>,
    alias_target_node_index: Option<u64>,
) -> SemanticNodeSlotView {
    SemanticNodeSlotView {
        cst_node_index,
        kind: semantic_node_kind_spec(node.kind),
        token_start: node.token_start,
        token_end: node.token_end,
        byte_start: node.byte_start,
        byte_end: node.byte_end,
        anchor_property_token: node.anchor_property_token,
        tag_property_token: node.tag_property_token,
        edge_start: node.entry_start,
        edge_end: node.entry_end,
        value_index,
        alias_target_node_index,
    }
}

impl SemanticNodeSlot {
    fn from_cst_node(
        node: &CstNode,
        cst_node_index: u64,
        value_index: Option<u64>,
        alias_target_node_index: Option<u64>,
    ) -> (slot: Self)
        ensures
            slot@ == semantic_node_slot_spec(
                node@,
                cst_node_index,
                value_index,
                alias_target_node_index,
            ),
    {
        Self {
            cst_node_index,
            kind: semantic_node_kind(node.kind()),
            token_start: node.token_start(),
            token_end: node.token_end(),
            byte_start: node.byte_start(),
            byte_end: node.byte_end(),
            anchor_property_token: node.anchor_property_token(),
            tag_property_token: node.tag_property_token(),
            edge_start: node.entry_start(),
            edge_end: node.entry_end(),
            value_index,
            alias_target_node_index,
        }
    }

    pub fn cst_node_index(&self) -> (index: u64)
        ensures
            index == self@.cst_node_index,
    {
        self.cst_node_index
    }

    pub fn kind(&self) -> (kind: SemanticNodeKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn token_start(&self) -> (index: u64)
        ensures
            index == self@.token_start,
    {
        self.token_start
    }

    pub fn token_end(&self) -> (index: u64)
        ensures
            index == self@.token_end,
    {
        self.token_end
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

    pub fn anchor_property_token(&self) -> (index: Option<u64>)
        ensures
            index == self@.anchor_property_token,
    {
        self.anchor_property_token
    }

    pub fn tag_property_token(&self) -> (index: Option<u64>)
        ensures
            index == self@.tag_property_token,
    {
        self.tag_property_token
    }

    pub fn edge_start(&self) -> (index: u64)
        ensures
            index == self@.edge_start,
    {
        self.edge_start
    }

    pub fn edge_end(&self) -> (index: u64)
        ensures
            index == self@.edge_end,
    {
        self.edge_end
    }

    pub fn value_index(&self) -> (index: Option<u64>)
        ensures
            index == self@.value_index,
    {
        self.value_index
    }

    pub fn alias_target_node_index(&self) -> (index: Option<u64>)
        ensures
            index == self@.alias_target_node_index,
    {
        self.alias_target_node_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticAliasRedirect {
    binding_index: u64,
    document_index: u64,
    alias_node_index: u64,
    alias_token_index: u64,
    target_anchor_index: u64,
    target_node_index: u64,
    name_start_atom_index: u64,
    name_end_atom_index: u64,
    name_byte_start: u64,
    name_byte_end: u64,
}

#[verifier::ext_equal]
pub struct SemanticAliasRedirectView {
    pub binding_index: u64,
    pub document_index: u64,
    pub alias_node_index: u64,
    pub alias_token_index: u64,
    pub target_anchor_index: u64,
    pub target_node_index: u64,
    pub name_start_atom_index: u64,
    pub name_end_atom_index: u64,
    pub name_byte_start: u64,
    pub name_byte_end: u64,
}

impl View for SemanticAliasRedirect {
    type V = SemanticAliasRedirectView;

    closed spec fn view(&self) -> SemanticAliasRedirectView {
        SemanticAliasRedirectView {
            binding_index: self.binding_index,
            document_index: self.document_index,
            alias_node_index: self.alias_node_index,
            alias_token_index: self.alias_token_index,
            target_anchor_index: self.target_anchor_index,
            target_node_index: self.target_node_index,
            name_start_atom_index: self.name_start_atom_index,
            name_end_atom_index: self.name_end_atom_index,
            name_byte_start: self.name_byte_start,
            name_byte_end: self.name_byte_end,
        }
    }
}

pub open spec fn semantic_alias_redirect_spec(
    binding: AliasBindingView,
    binding_index: u64,
) -> SemanticAliasRedirectView {
    SemanticAliasRedirectView {
        binding_index,
        document_index: binding.document_index,
        alias_node_index: binding.alias_node_index,
        alias_token_index: binding.alias_token_index,
        target_anchor_index: binding.target_anchor_index,
        target_node_index: binding.target_node_index,
        name_start_atom_index: binding.name_start_atom_index,
        name_end_atom_index: binding.name_end_atom_index,
        name_byte_start: binding.name_byte_start,
        name_byte_end: binding.name_byte_end,
    }
}

impl SemanticAliasRedirect {
    fn from_binding(binding: &AliasBinding, binding_index: u64) -> (redirect: Self)
        ensures
            redirect@ == semantic_alias_redirect_spec(binding@, binding_index),
    {
        Self {
            binding_index,
            document_index: binding.document_index(),
            alias_node_index: binding.alias_node_index(),
            alias_token_index: binding.alias_token_index(),
            target_anchor_index: binding.target_anchor_index(),
            target_node_index: binding.target_node_index(),
            name_start_atom_index: binding.name_start_atom_index(),
            name_end_atom_index: binding.name_end_atom_index(),
            name_byte_start: binding.name_byte_start(),
            name_byte_end: binding.name_byte_end(),
        }
    }

    pub fn binding_index(&self) -> (index: u64)
        ensures
            index == self@.binding_index,
    {
        self.binding_index
    }

    pub fn document_index(&self) -> (index: u64)
        ensures
            index == self@.document_index,
    {
        self.document_index
    }

    pub fn alias_node_index(&self) -> (index: u64)
        ensures
            index == self@.alias_node_index,
    {
        self.alias_node_index
    }

    pub fn alias_token_index(&self) -> (index: u64)
        ensures
            index == self@.alias_token_index,
    {
        self.alias_token_index
    }

    pub fn target_anchor_index(&self) -> (index: u64)
        ensures
            index == self@.target_anchor_index,
    {
        self.target_anchor_index
    }

    pub fn target_node_index(&self) -> (index: u64)
        ensures
            index == self@.target_node_index,
    {
        self.target_node_index
    }

    pub fn name_start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.name_start_atom_index,
    {
        self.name_start_atom_index
    }

    pub fn name_end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.name_end_atom_index,
    {
        self.name_end_atom_index
    }

    pub fn name_byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.name_byte_start,
    {
        self.name_byte_start
    }

    pub fn name_byte_end(&self) -> (offset: u64)
        ensures
            offset == self@.name_byte_end,
    {
        self.name_byte_end
    }
}

pub open spec fn semantic_node_slot_views_spec(values: Seq<SemanticNodeSlot>) -> Seq<
    SemanticNodeSlotView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn semantic_collection_views_spec(values: Seq<ResolvedCollection>) -> Seq<
    ResolvedCollectionView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn semantic_alias_redirect_views_spec(values: Seq<SemanticAliasRedirect>) -> Seq<
    SemanticAliasRedirectView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn semantic_alias_redirects_spec(bindings: Seq<AliasBindingView>) -> Seq<
    SemanticAliasRedirectView,
> {
    Seq::new(
        bindings.len(),
        |index: int| semantic_alias_redirect_spec(bindings[index], index as u64),
    )
}

proof fn lemma_semantic_node_slot_views_push(values: Seq<SemanticNodeSlot>, value: SemanticNodeSlot)
    ensures
        semantic_node_slot_views_spec(values.push(value)) == semantic_node_slot_views_spec(
            values,
        ).push(value@),
{
    reveal(semantic_node_slot_views_spec);
    assert(semantic_node_slot_views_spec(values.push(value)) =~= semantic_node_slot_views_spec(
        values,
    ).push(value@));
}

proof fn lemma_semantic_collection_views_push(
    values: Seq<ResolvedCollection>,
    value: ResolvedCollection,
)
    ensures
        semantic_collection_views_spec(values.push(value)) == semantic_collection_views_spec(
            values,
        ).push(value@),
{
    reveal(semantic_collection_views_spec);
    assert(semantic_collection_views_spec(values.push(value)) =~= semantic_collection_views_spec(
        values,
    ).push(value@));
}

proof fn lemma_semantic_alias_redirect_views_push(
    values: Seq<SemanticAliasRedirect>,
    value: SemanticAliasRedirect,
)
    ensures
        semantic_alias_redirect_views_spec(values.push(value))
            == semantic_alias_redirect_views_spec(values).push(value@),
{
    reveal(semantic_alias_redirect_views_spec);
    assert(semantic_alias_redirect_views_spec(values.push(value))
        =~= semantic_alias_redirect_views_spec(values).push(value@));
}

#[derive(Debug, PartialEq, Eq)]
pub struct SemanticNodeTableSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    input_scalar_count: u64,
    input_anchor_count: u64,
    input_alias_count: u64,
    topology: SemanticTopologySource,
    scalars: SemanticScalarTableSource,
    anchors: AnchorAliasSource,
    nodes: Vec<SemanticNodeSlot>,
    collections: Vec<ResolvedCollection>,
    alias_redirects: Vec<SemanticAliasRedirect>,
}

#[verifier::ext_equal]
pub struct SemanticNodeTableSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub input_scalar_count: u64,
    pub input_anchor_count: u64,
    pub input_alias_count: u64,
    pub topology: SemanticTopologySourceView,
    pub scalars: SemanticScalarTableSourceView,
    pub anchors: AnchorAliasSourceView,
    pub nodes: Seq<SemanticNodeSlotView>,
    pub collections: Seq<ResolvedCollectionView>,
    pub alias_redirects: Seq<SemanticAliasRedirectView>,
}

impl View for SemanticNodeTableSource {
    type V = SemanticNodeTableSourceView;

    closed spec fn view(&self) -> SemanticNodeTableSourceView {
        SemanticNodeTableSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            input_scalar_count: self.input_scalar_count,
            input_anchor_count: self.input_anchor_count,
            input_alias_count: self.input_alias_count,
            topology: self.topology@,
            scalars: self.scalars@,
            anchors: self.anchors@,
            nodes: semantic_node_slot_views_spec(self.nodes@),
            collections: semantic_collection_views_spec(self.collections@),
            alias_redirects: semantic_alias_redirect_views_spec(self.alias_redirects@),
        }
    }
}

impl SemanticNodeTableSource {
    fn new(
        completed: &CompletedTokenSource,
        topology: SemanticTopologySource,
        scalars: SemanticScalarTableSource,
        anchors: AnchorAliasSource,
        nodes: Vec<SemanticNodeSlot>,
        collections: Vec<ResolvedCollection>,
        alias_redirects: Vec<SemanticAliasRedirect>,
    ) -> (source: Self)
        ensures
            source@ == (SemanticNodeTableSourceView {
                profile_version: completed@.profile_version,
                transformation_version: SEMANTIC_NODE_TABLE_TRANSFORMATION_VERSION,
                source_len_bytes: completed@.source_len_bytes,
                input_node_count: topology@.nodes.len() as u64,
                input_scalar_count: scalars@.scalars.len() as u64,
                input_anchor_count: anchors@.anchors.len() as u64,
                input_alias_count: anchors@.aliases.len() as u64,
                topology: topology@,
                scalars: scalars@,
                anchors: anchors@,
                nodes: semantic_node_slot_views_spec(nodes@),
                collections: semantic_collection_views_spec(collections@),
                alias_redirects: semantic_alias_redirect_views_spec(alias_redirects@),
            }),
    {
        let input_node_count = topology.nodes().len() as u64;
        let input_scalar_count = scalars.scalars().len() as u64;
        let input_anchor_count = anchors.anchors().len() as u64;
        let input_alias_count = anchors.aliases().len() as u64;
        Self {
            profile_version: completed.profile_version(),
            transformation_version: SEMANTIC_NODE_TABLE_TRANSFORMATION_VERSION,
            source_len_bytes: completed.source_len_bytes(),
            input_node_count,
            input_scalar_count,
            input_anchor_count,
            input_alias_count,
            topology,
            scalars,
            anchors,
            nodes,
            collections,
            alias_redirects,
        }
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

    pub fn source_len_bytes(&self) -> (length: u64)
        ensures
            length == self@.source_len_bytes,
    {
        self.source_len_bytes
    }

    pub fn input_node_count(&self) -> (count: u64)
        ensures
            count == self@.input_node_count,
    {
        self.input_node_count
    }

    pub fn input_scalar_count(&self) -> (count: u64)
        ensures
            count == self@.input_scalar_count,
    {
        self.input_scalar_count
    }

    pub fn input_anchor_count(&self) -> (count: u64)
        ensures
            count == self@.input_anchor_count,
    {
        self.input_anchor_count
    }

    pub fn input_alias_count(&self) -> (count: u64)
        ensures
            count == self@.input_alias_count,
    {
        self.input_alias_count
    }

    pub fn topology(&self) -> (source: &SemanticTopologySource)
        ensures
            source@ == self@.topology,
    {
        &self.topology
    }

    pub fn scalars(&self) -> (source: &SemanticScalarTableSource)
        ensures
            source@ == self@.scalars,
    {
        &self.scalars
    }

    pub fn anchors(&self) -> (source: &AnchorAliasSource)
        ensures
            source@ == self@.anchors,
    {
        &self.anchors
    }

    pub fn nodes(&self) -> (values: &[SemanticNodeSlot])
        ensures
            semantic_node_slot_views_spec(values@) == self@.nodes,
    {
        self.nodes.as_slice()
    }

    pub fn collections(&self) -> (values: &[ResolvedCollection])
        ensures
            semantic_collection_views_spec(values@) == self@.collections,
    {
        self.collections.as_slice()
    }

    pub fn alias_redirects(&self) -> (values: &[SemanticAliasRedirect])
        ensures
            semantic_alias_redirect_views_spec(values@) == self@.alias_redirects,
    {
        self.alias_redirects.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum SemanticNodeTableErrorKind {
    InputCompletedTokenMismatch,
    InputCstMismatch,
    Topology(SemanticTopologyErrorKind),
    ScalarTable(SemanticScalarTableErrorKind),
    AnchorAlias(AnchorAliasErrorKind),
    CollectionTag(CollectionTagErrorKind),
    NodeLimitExceeded,
    CollectionLimitExceeded,
    AliasRedirectLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticNodeTableError {
    kind: SemanticNodeTableErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct SemanticNodeTableErrorView {
    pub kind: SemanticNodeTableErrorKind,
    pub byte_offset: u64,
}

impl View for SemanticNodeTableError {
    type V = SemanticNodeTableErrorView;

    closed spec fn view(&self) -> SemanticNodeTableErrorView {
        SemanticNodeTableErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl SemanticNodeTableError {
    fn at(kind: SemanticNodeTableErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (SemanticNodeTableErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: SemanticNodeTableErrorKind)
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

pub open spec fn map_collection_tag_error_spec(
    error: CollectionTagErrorView,
) -> SemanticNodeTableErrorView {
    SemanticNodeTableErrorView {
        kind: SemanticNodeTableErrorKind::CollectionTag(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_collection_tag_error(error: CollectionTagError) -> (mapped: SemanticNodeTableError)
    ensures
        mapped@ == map_collection_tag_error_spec(error@),
{
    let mapped = SemanticNodeTableError::at(
        SemanticNodeTableErrorKind::CollectionTag(error.kind()),
        error.byte_offset(),
    );
    proof {
        reveal(map_collection_tag_error_spec);
    }
    mapped
}

pub open spec fn map_semantic_topology_error_spec(
    error: crate::resolve_topology::SemanticTopologyErrorView,
) -> SemanticNodeTableErrorView {
    SemanticNodeTableErrorView {
        kind: SemanticNodeTableErrorKind::Topology(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_semantic_topology_error(error: SemanticTopologyError) -> (mapped: SemanticNodeTableError)
    ensures
        mapped@ == map_semantic_topology_error_spec(error@),
{
    SemanticNodeTableError::at(
        SemanticNodeTableErrorKind::Topology(error.kind()),
        error.byte_offset(),
    )
}

pub open spec fn map_semantic_scalar_table_error_spec(
    error: crate::resolve_scalar_table::SemanticScalarTableErrorView,
) -> SemanticNodeTableErrorView {
    SemanticNodeTableErrorView {
        kind: SemanticNodeTableErrorKind::ScalarTable(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_semantic_scalar_table_error(error: SemanticScalarTableError) -> (mapped:
    SemanticNodeTableError)
    ensures
        mapped@ == map_semantic_scalar_table_error_spec(error@),
{
    SemanticNodeTableError::at(
        SemanticNodeTableErrorKind::ScalarTable(error.kind()),
        error.byte_offset(),
    )
}

pub open spec fn map_anchor_alias_error_spec(
    error: crate::resolve_anchor::AnchorAliasErrorView,
) -> SemanticNodeTableErrorView {
    SemanticNodeTableErrorView {
        kind: SemanticNodeTableErrorKind::AnchorAlias(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_anchor_alias_error(error: AnchorAliasError) -> (mapped: SemanticNodeTableError)
    ensures
        mapped@ == map_anchor_alias_error_spec(error@),
{
    SemanticNodeTableError::at(
        SemanticNodeTableErrorKind::AnchorAlias(error.kind()),
        error.byte_offset(),
    )
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct SemanticNodeTableStepStateView {
    node_count: nat,
    collection_count: nat,
    alias_redirect_count: nat,
    scalar_cursor: nat,
    alias_cursor: nat,
}

#[derive(Debug, PartialEq, Eq)]
struct SemanticNodeTableStep {
    slot: SemanticNodeSlot,
    collection: Option<ResolvedCollection>,
    alias_redirect: Option<SemanticAliasRedirect>,
    next_scalar_cursor: usize,
    next_alias_cursor: usize,
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct SemanticNodeTableStepView {
    slot: SemanticNodeSlotView,
    collection: Option<ResolvedCollectionView>,
    alias_redirect: Option<SemanticAliasRedirectView>,
    next_scalar_cursor: nat,
    next_alias_cursor: nat,
}

impl View for SemanticNodeTableStep {
    type V = SemanticNodeTableStepView;

    closed spec fn view(&self) -> SemanticNodeTableStepView {
        SemanticNodeTableStepView {
            slot: self.slot@,
            collection: match self.collection {
                Some(ref value) => Some(value@),
                None => None,
            },
            alias_redirect: match self.alias_redirect {
                Some(ref value) => Some(value@),
                None => None,
            },
            next_scalar_cursor: self.next_scalar_cursor as nat,
            next_alias_cursor: self.next_alias_cursor as nat,
        }
    }
}

closed spec fn semantic_node_table_step_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    scalars: SemanticScalarTableSourceView,
    anchors: AnchorAliasSourceView,
    index: nat,
    state: SemanticNodeTableStepStateView,
    limits: SemanticNodeTableLimitsView,
) -> Result<SemanticNodeTableStepView, SemanticNodeTableErrorView> {
    let node = cst.nodes[index as int];
    let node_limit = semantic_node_table_effective_limit_spec(
        limits.max_nodes,
        MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES,
    );
    let collection_limit = semantic_node_table_effective_limit_spec(
        limits.max_collections,
        MAX_PROFILE1_SEMANTIC_COLLECTIONS,
    );
    let alias_limit = semantic_node_table_effective_limit_spec(
        limits.max_alias_redirects,
        MAX_PROFILE1_SEMANTIC_ALIAS_REDIRECTS,
    );
    if node.kind == CstNodeKind::Scalar || node.kind == CstNodeKind::Empty {
        if state.scalar_cursor >= scalars.scalars.len()
            || scalars.scalars[state.scalar_cursor as int].node_index != index as u64 {
            Err(
                SemanticNodeTableErrorView {
                    kind: SemanticNodeTableErrorKind::InternalInvariantViolation,
                    byte_offset: node.byte_start,
                },
            )
        } else if state.node_count >= node_limit {
            Err(
                SemanticNodeTableErrorView {
                    kind: SemanticNodeTableErrorKind::NodeLimitExceeded,
                    byte_offset: node.byte_start,
                },
            )
        } else {
            Ok(
                SemanticNodeTableStepView {
                    slot: semantic_node_slot_spec(
                        node,
                        index as u64,
                        Some(state.scalar_cursor as u64),
                        None,
                    ),
                    collection: None,
                    alias_redirect: None,
                    next_scalar_cursor: (state.scalar_cursor + 1) as nat,
                    next_alias_cursor: state.alias_cursor,
                },
            )
        }
    } else if node.kind == CstNodeKind::Sequence || node.kind == CstNodeKind::Mapping {
        match crate::resolve_collection_tag::resolve_profile1_cst_node_collection_tag_spec(
            atomized,
            completed,
            cst,
            index as u64,
            semantic_node_table_collection_tag_limits_spec(limits),
        ) {
            Err(error) => Err(map_collection_tag_error_spec(error)),
            Ok(None) => Err(
                SemanticNodeTableErrorView {
                    kind: SemanticNodeTableErrorKind::InternalInvariantViolation,
                    byte_offset: node.byte_start,
                },
            ),
            Ok(Some(collection)) => if state.node_count >= node_limit {
                Err(
                    SemanticNodeTableErrorView {
                        kind: SemanticNodeTableErrorKind::NodeLimitExceeded,
                        byte_offset: node.byte_start,
                    },
                )
            } else if state.collection_count >= collection_limit {
                Err(
                    SemanticNodeTableErrorView {
                        kind: SemanticNodeTableErrorKind::CollectionLimitExceeded,
                        byte_offset: node.byte_start,
                    },
                )
            } else {
                Ok(
                    SemanticNodeTableStepView {
                        slot: semantic_node_slot_spec(
                            node,
                            index as u64,
                            Some(state.collection_count as u64),
                            None,
                        ),
                        collection: Some(collection),
                        alias_redirect: None,
                        next_scalar_cursor: state.scalar_cursor,
                        next_alias_cursor: state.alias_cursor,
                    },
                )
            },
        }
    } else if state.alias_cursor >= anchors.aliases.len()
        || anchors.aliases[state.alias_cursor as int].alias_node_index != index as u64
        || anchors.aliases[state.alias_cursor as int].target_node_index >= cst.nodes.len() {
        Err(
            SemanticNodeTableErrorView {
                kind: SemanticNodeTableErrorKind::InternalInvariantViolation,
                byte_offset: node.byte_start,
            },
        )
    } else {
        let binding = anchors.aliases[state.alias_cursor as int];
        if state.node_count >= node_limit {
            Err(
                SemanticNodeTableErrorView {
                    kind: SemanticNodeTableErrorKind::NodeLimitExceeded,
                    byte_offset: node.byte_start,
                },
            )
        } else if state.alias_redirect_count >= alias_limit {
            Err(
                SemanticNodeTableErrorView {
                    kind: SemanticNodeTableErrorKind::AliasRedirectLimitExceeded,
                    byte_offset: binding.name_byte_start,
                },
            )
        } else {
            Ok(
                SemanticNodeTableStepView {
                    slot: semantic_node_slot_spec(
                        node,
                        index as u64,
                        None,
                        Some(binding.target_node_index),
                    ),
                    collection: None,
                    alias_redirect: Some(
                        semantic_alias_redirect_spec(binding, state.alias_cursor as u64),
                    ),
                    next_scalar_cursor: state.scalar_cursor,
                    next_alias_cursor: (state.alias_cursor + 1) as nat,
                },
            )
        }
    }
}

proof fn lemma_semantic_node_table_step_success_preserves_cursor_bounds(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    scalars: SemanticScalarTableSourceView,
    anchors: AnchorAliasSourceView,
    index: nat,
    state: SemanticNodeTableStepStateView,
    limits: SemanticNodeTableLimitsView,
    step: SemanticNodeTableStepView,
)
    requires
        index < cst.nodes.len(),
        state.scalar_cursor <= scalars.scalars.len(),
        state.alias_cursor <= anchors.aliases.len(),
        semantic_node_table_step_spec(
            atomized,
            completed,
            cst,
            scalars,
            anchors,
            index,
            state,
            limits,
        ) == Ok(step),
    ensures
        step.next_scalar_cursor <= scalars.scalars.len(),
        step.next_alias_cursor <= anchors.aliases.len(),
{
    reveal(semantic_node_table_step_spec);
}

#[allow(clippy::too_many_arguments)]  // Every independently verified producer remains explicit.
fn semantic_node_table_step(
    atomized: &AtomizedSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    scalars: &SemanticScalarTableSource,
    anchors: &AnchorAliasSource,
    index: usize,
    node_count: usize,
    collection_count: usize,
    alias_redirect_count: usize,
    scalar_cursor: usize,
    alias_cursor: usize,
    limits: SemanticNodeTableLimits,
) -> (result: Result<SemanticNodeTableStep, SemanticNodeTableError>)
    requires
        index < cst@.nodes.len(),
    ensures
        semantic_node_table_step_spec(
            atomized@,
            completed@,
            cst@,
            scalars@,
            anchors@,
            index as nat,
            SemanticNodeTableStepStateView {
                node_count: node_count as nat,
                collection_count: collection_count as nat,
                alias_redirect_count: alias_redirect_count as nat,
                scalar_cursor: scalar_cursor as nat,
                alias_cursor: alias_cursor as nat,
            },
            limits@,
        ) == match result {
            Ok(step) => Ok(step@),
            Err(error) => Err(error@),
        },
{
    let nodes = cst.nodes();
    let scalar_values = scalars.scalars();
    let aliases = anchors.aliases();
    proof {
        reveal(crate::cst::cst_node_views_spec);
        assert(cst@.nodes.len() == nodes@.len());
        crate::cst::lemma_cst_node_view_at(nodes@, index as int);
    }
    let node = &nodes[index];
    let kind = node.kind();
    let node_limit = semantic_node_table_effective_limit(
        limits.max_nodes(),
        MAX_PROFILE1_SEMANTIC_NODE_TABLE_NODES,
    );
    if kind == CstNodeKind::Scalar || kind == CstNodeKind::Empty {
        if scalar_cursor >= scalar_values.len() || scalar_values[scalar_cursor].node_index()
            != index as u64 {
            let error = SemanticNodeTableError::at(
                SemanticNodeTableErrorKind::InternalInvariantViolation,
                node.byte_start(),
            );
            proof {
                reveal(semantic_node_table_step_spec);
            }
            return Err(error);
        }
        if node_count as u64 >= node_limit {
            let error = SemanticNodeTableError::at(
                SemanticNodeTableErrorKind::NodeLimitExceeded,
                node.byte_start(),
            );
            proof {
                reveal(semantic_node_table_step_spec);
            }
            return Err(error);
        }
        let slot = SemanticNodeSlot::from_cst_node(
            node,
            index as u64,
            Some(scalar_cursor as u64),
            None,
        );
        proof {
            reveal(semantic_node_table_step_spec);
        }
        return Ok(
            SemanticNodeTableStep {
                slot,
                collection: None,
                alias_redirect: None,
                next_scalar_cursor: scalar_cursor + 1,
                next_alias_cursor: alias_cursor,
            },
        );
    }
    if kind == CstNodeKind::Sequence || kind == CstNodeKind::Mapping {
        let tag_limits = limits.collection_tag_limits();
        let collection = match resolve_profile1_cst_node_collection_tag(
            atomized,
            completed,
            cst,
            index as u64,
            tag_limits,
        ) {
            Err(error) => {
                let mapped = map_collection_tag_error(error);
                proof {
                    reveal(semantic_node_table_step_spec);
                }
                return Err(mapped);
            },
            Ok(None) => {
                let error = SemanticNodeTableError::at(
                    SemanticNodeTableErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                );
                proof {
                    reveal(semantic_node_table_step_spec);
                }
                return Err(error);
            },
            Ok(Some(value)) => value,
        };
        if node_count as u64 >= node_limit {
            let error = SemanticNodeTableError::at(
                SemanticNodeTableErrorKind::NodeLimitExceeded,
                node.byte_start(),
            );
            proof {
                reveal(semantic_node_table_step_spec);
            }
            return Err(error);
        }
        let collection_limit = semantic_node_table_effective_limit(
            limits.max_collections(),
            MAX_PROFILE1_SEMANTIC_COLLECTIONS,
        );
        if collection_count as u64 >= collection_limit {
            let error = SemanticNodeTableError::at(
                SemanticNodeTableErrorKind::CollectionLimitExceeded,
                node.byte_start(),
            );
            proof {
                reveal(semantic_node_table_step_spec);
            }
            return Err(error);
        }
        let slot = SemanticNodeSlot::from_cst_node(
            node,
            index as u64,
            Some(collection_count as u64),
            None,
        );
        proof {
            reveal(semantic_node_table_step_spec);
        }
        return Ok(
            SemanticNodeTableStep {
                slot,
                collection: Some(collection),
                alias_redirect: None,
                next_scalar_cursor: scalar_cursor,
                next_alias_cursor: alias_cursor,
            },
        );
    }
    if alias_cursor >= aliases.len() || aliases[alias_cursor].alias_node_index() != index as u64
        || aliases[alias_cursor].target_node_index() >= nodes.len() as u64 {
        let error = SemanticNodeTableError::at(
            SemanticNodeTableErrorKind::InternalInvariantViolation,
            node.byte_start(),
        );
        proof {
            reveal(semantic_node_table_step_spec);
        }
        return Err(error);
    }
    if node_count as u64 >= node_limit {
        let error = SemanticNodeTableError::at(
            SemanticNodeTableErrorKind::NodeLimitExceeded,
            node.byte_start(),
        );
        proof {
            reveal(semantic_node_table_step_spec);
        }
        return Err(error);
    }
    let binding = &aliases[alias_cursor];
    let alias_limit = semantic_node_table_effective_limit(
        limits.max_alias_redirects(),
        MAX_PROFILE1_SEMANTIC_ALIAS_REDIRECTS,
    );
    if alias_redirect_count as u64 >= alias_limit {
        let error = SemanticNodeTableError::at(
            SemanticNodeTableErrorKind::AliasRedirectLimitExceeded,
            binding.name_byte_start(),
        );
        proof {
            reveal(semantic_node_table_step_spec);
        }
        return Err(error);
    }
    let slot = SemanticNodeSlot::from_cst_node(
        node,
        index as u64,
        None,
        Some(binding.target_node_index()),
    );
    let redirect = SemanticAliasRedirect::from_binding(binding, alias_cursor as u64);
    proof {
        reveal(semantic_node_table_step_spec);
    }
    Ok(
        SemanticNodeTableStep {
            slot,
            collection: None,
            alias_redirect: Some(redirect),
            next_scalar_cursor: scalar_cursor,
            next_alias_cursor: alias_cursor + 1,
        },
    )
}

#[verifier::ext_equal]
pub struct SemanticNodeTableBuildView {
    pub nodes: Seq<SemanticNodeSlotView>,
    pub collections: Seq<ResolvedCollectionView>,
    pub alias_redirects: Seq<SemanticAliasRedirectView>,
    pub scalar_cursor: nat,
    pub alias_cursor: nat,
}

closed spec fn semantic_node_table_apply_step_spec(
    build: SemanticNodeTableBuildView,
    step: SemanticNodeTableStepView,
) -> SemanticNodeTableBuildView {
    SemanticNodeTableBuildView {
        nodes: build.nodes.push(step.slot),
        collections: match step.collection {
            Some(collection) => build.collections.push(collection),
            None => build.collections,
        },
        alias_redirects: match step.alias_redirect {
            Some(redirect) => build.alias_redirects.push(redirect),
            None => build.alias_redirects,
        },
        scalar_cursor: step.next_scalar_cursor,
        alias_cursor: step.next_alias_cursor,
    }
}

struct SemanticNodeTableBuild {
    nodes: Vec<SemanticNodeSlot>,
    collections: Vec<ResolvedCollection>,
    alias_redirects: Vec<SemanticAliasRedirect>,
    scalar_cursor: usize,
    alias_cursor: usize,
}

impl View for SemanticNodeTableBuild {
    type V = SemanticNodeTableBuildView;

    closed spec fn view(&self) -> SemanticNodeTableBuildView {
        SemanticNodeTableBuildView {
            nodes: semantic_node_slot_views_spec(self.nodes@),
            collections: semantic_collection_views_spec(self.collections@),
            alias_redirects: semantic_alias_redirect_views_spec(self.alias_redirects@),
            scalar_cursor: self.scalar_cursor as nat,
            alias_cursor: self.alias_cursor as nat,
        }
    }
}

impl SemanticNodeTableBuild {
    fn empty() -> (build: Self)
        ensures
            build@ == (SemanticNodeTableBuildView {
                nodes: Seq::empty(),
                collections: Seq::empty(),
                alias_redirects: Seq::empty(),
                scalar_cursor: 0,
                alias_cursor: 0,
            }),
    {
        let build = Self {
            nodes: Vec::new(),
            collections: Vec::new(),
            alias_redirects: Vec::new(),
            scalar_cursor: 0,
            alias_cursor: 0,
        };
        proof {
            reveal(semantic_node_slot_views_spec);
            reveal(semantic_collection_views_spec);
            reveal(semantic_alias_redirect_views_spec);
        }
        build
    }

    #[allow(non_shorthand_field_patterns)]  // Verus macro expansion retains explicit field names.
    fn apply_step(&mut self, step: SemanticNodeTableStep)
        ensures
            final(self)@ == semantic_node_table_apply_step_spec(old(self)@, step@),
    {
        let SemanticNodeTableStep {
            slot,
            collection,
            alias_redirect,
            next_scalar_cursor,
            next_alias_cursor,
        } = step;
        proof {
            lemma_semantic_node_slot_views_push(self.nodes@, slot);
        }
        self.nodes.push(slot);
        if let Some(value) = collection {
            proof {
                lemma_semantic_collection_views_push(self.collections@, value);
            }
            self.collections.push(value);
        }
        if let Some(value) = alias_redirect {
            proof {
                lemma_semantic_alias_redirect_views_push(self.alias_redirects@, value);
            }
            self.alias_redirects.push(value);
        }
        self.scalar_cursor = next_scalar_cursor;
        self.alias_cursor = next_alias_cursor;
        proof {
            reveal(semantic_node_table_apply_step_spec);
        }
    }
}

pub closed spec fn compose_semantic_node_table_tail_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    scalars: SemanticScalarTableSourceView,
    anchors: AnchorAliasSourceView,
    index: nat,
    fuel: nat,
    build: SemanticNodeTableBuildView,
    limits: SemanticNodeTableLimitsView,
) -> Result<SemanticNodeTableBuildView, SemanticNodeTableErrorView>
    decreases fuel,
{
    if index >= cst.nodes.len() {
        if build.scalar_cursor == scalars.scalars.len() && build.alias_cursor
            == anchors.aliases.len() {
            Ok(build)
        } else {
            Err(
                SemanticNodeTableErrorView {
                    kind: SemanticNodeTableErrorKind::InternalInvariantViolation,
                    byte_offset: completed.source_len_bytes,
                },
            )
        }
    } else if fuel == 0 {
        Err(
            SemanticNodeTableErrorView {
                kind: SemanticNodeTableErrorKind::InternalInvariantViolation,
                byte_offset: cst.nodes[index as int].byte_start,
            },
        )
    } else {
        match semantic_node_table_step_spec(
            atomized,
            completed,
            cst,
            scalars,
            anchors,
            index,
            SemanticNodeTableStepStateView {
                node_count: build.nodes.len() as nat,
                collection_count: build.collections.len() as nat,
                alias_redirect_count: build.alias_redirects.len() as nat,
                scalar_cursor: build.scalar_cursor,
                alias_cursor: build.alias_cursor,
            },
            limits,
        ) {
            Err(error) => Err(error),
            Ok(step) => compose_semantic_node_table_tail_spec(
                atomized,
                completed,
                cst,
                scalars,
                anchors,
                (index + 1) as nat,
                (fuel - 1) as nat,
                semantic_node_table_apply_step_spec(build, step),
                limits,
            ),
        }
    }
}

pub open spec fn semantic_node_table_finalize_spec(
    completed: CompletedTokenSourceView,
    topology: SemanticTopologySourceView,
    scalars: SemanticScalarTableSourceView,
    anchors: AnchorAliasSourceView,
    result: Result<SemanticNodeTableBuildView, SemanticNodeTableErrorView>,
) -> Result<SemanticNodeTableSourceView, SemanticNodeTableErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => Ok(
            SemanticNodeTableSourceView {
                profile_version: completed.profile_version,
                transformation_version: SEMANTIC_NODE_TABLE_TRANSFORMATION_VERSION,
                source_len_bytes: completed.source_len_bytes,
                input_node_count: topology.nodes.len() as u64,
                input_scalar_count: scalars.scalars.len() as u64,
                input_anchor_count: anchors.anchors.len() as u64,
                input_alias_count: anchors.aliases.len() as u64,
                topology,
                scalars,
                anchors,
                nodes: build.nodes,
                collections: build.collections,
                alias_redirects: build.alias_redirects,
            },
        ),
    }
}

pub open spec fn compose_profile1_semantic_node_table_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    limits: SemanticNodeTableLimitsView,
) -> Result<SemanticNodeTableSourceView, SemanticNodeTableErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            SemanticNodeTableErrorView {
                kind: SemanticNodeTableErrorKind::InputCompletedTokenMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() {
        Err(
            SemanticNodeTableErrorView {
                kind: SemanticNodeTableErrorKind::InputCstMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else {
        match crate::resolve_topology::compose_profile1_semantic_topology_spec(
            atomized,
            completed,
            cst,
            topology_limits,
        ) {
            Err(error) => Err(map_semantic_topology_error_spec(error)),
            Ok(
                topology,
            ) => match crate::resolve_scalar_table::compose_profile1_semantic_scalar_table_spec(
                atomized,
                quoted,
                plain,
                block,
                completed,
                cst,
                scalar_limits,
            ) {
                Err(error) => Err(map_semantic_scalar_table_error_spec(error)),
                Ok(scalars) => match crate::resolve_anchor::resolve_profile1_anchor_aliases_spec(
                    atomized,
                    completed,
                    cst,
                    anchor_limits,
                ) {
                    Err(error) => Err(map_anchor_alias_error_spec(error)),
                    Ok(anchors) => semantic_node_table_finalize_spec(
                        completed,
                        topology,
                        scalars,
                        anchors,
                        compose_semantic_node_table_tail_spec(
                            atomized,
                            completed,
                            cst,
                            scalars,
                            anchors,
                            0,
                            cst.nodes.len(),
                            SemanticNodeTableBuildView {
                                nodes: Seq::empty(),
                                collections: Seq::empty(),
                                alias_redirects: Seq::empty(),
                                scalar_cursor: 0,
                                alias_cursor: 0,
                            },
                            limits,
                        ),
                    ),
                },
            },
        }
    }
}

/// Exact public semantics authenticate every producer and the total pure aggregate result.
pub open spec fn semantic_node_table_source_well_formed_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    limits: SemanticNodeTableLimitsView,
    source: SemanticNodeTableSourceView,
) -> bool {
    crate::cst::cst_public_semantics_spec(completed, cst)
        && compose_profile1_semantic_node_table_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        limits,
    ) == Ok(source)
}

pub proof fn lemma_semantic_node_table_success_is_well_formed(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    limits: SemanticNodeTableLimitsView,
    source: SemanticNodeTableSourceView,
)
    requires
        crate::cst::cst_public_semantics_spec(completed, cst),
        compose_profile1_semantic_node_table_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            limits,
        ) == Ok(source),
    ensures
        semantic_node_table_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            limits,
            source,
        ),
{
    reveal(semantic_node_table_source_well_formed_spec);
}

pub proof fn lemma_semantic_node_table_well_formed_authenticates_exact_composition(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    limits: SemanticNodeTableLimitsView,
    source: SemanticNodeTableSourceView,
)
    requires
        semantic_node_table_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            limits,
            source,
        ),
    ensures
        compose_profile1_semantic_node_table_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            limits,
        ) == Ok(source),
        crate::cst::cst_public_semantics_spec(completed, cst),
{
    reveal(semantic_node_table_source_well_formed_spec);
}

#[allow(clippy::too_many_arguments)]  // Every independently verified producer remains explicit.
#[allow(non_shorthand_field_patterns)]  // Verus macro expansion retains explicit field names.
pub fn compose_profile1_semantic_node_table(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    topology_limits: SemanticTopologyLimits,
    scalar_limits: SemanticScalarTableLimits,
    anchor_limits: AnchorAliasLimits,
    limits: SemanticNodeTableLimits,
) -> (result: Result<SemanticNodeTableSource, SemanticNodeTableError>)
    ensures
        compose_profile1_semantic_node_table_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            topology_limits@,
            scalar_limits@,
            anchor_limits@,
            limits@,
        ) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let cst_nodes = cst.nodes();
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::cst::cst_node_views_spec);
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens.len() == tokens@.len());
        assert(cst@.nodes.len() == cst_nodes@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != atomized.bom_bytes()
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = SemanticNodeTableError::at(
            SemanticNodeTableErrorKind::InputCompletedTokenMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_node_table_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 {
        let error = SemanticNodeTableError::at(
            SemanticNodeTableErrorKind::InputCstMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_node_table_spec);
        }
        return Err(error);
    }
    let topology = match crate::resolve_topology::compose_profile1_semantic_topology(
        atomized,
        completed,
        cst,
        topology_limits,
    ) {
        Err(error) => {
            let mapped = map_semantic_topology_error(error);
            proof {
                reveal(compose_profile1_semantic_node_table_spec);
            }
            return Err(mapped);
        },
        Ok(source) => source,
    };
    let scalars = match crate::resolve_scalar_table::compose_profile1_semantic_scalar_table(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        scalar_limits,
    ) {
        Err(error) => {
            let mapped = map_semantic_scalar_table_error(error);
            proof {
                reveal(compose_profile1_semantic_node_table_spec);
            }
            return Err(mapped);
        },
        Ok(source) => source,
    };
    let anchors = match crate::resolve_anchor::resolve_profile1_anchor_aliases(
        atomized,
        completed,
        cst,
        anchor_limits,
    ) {
        Err(error) => {
            let mapped = map_anchor_alias_error(error);
            proof {
                reveal(compose_profile1_semantic_node_table_spec);
            }
            return Err(mapped);
        },
        Ok(source) => source,
    };
    let scalar_values = scalars.scalars();
    let alias_bindings = anchors.aliases();
    let mut build = SemanticNodeTableBuild::empty();
    let mut index = 0usize;
    let mut _fuel = cst_nodes.len();
    let ghost expected = compose_semantic_node_table_tail_spec(
        atomized@,
        completed@,
        cst@,
        scalars@,
        anchors@,
        0,
        cst@.nodes.len(),
        build@,
        limits@,
    );
    proof {
        reveal(compose_profile1_semantic_node_table_spec);
        assert(compose_profile1_semantic_node_table_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            topology_limits@,
            scalar_limits@,
            anchor_limits@,
            limits@,
        ) == semantic_node_table_finalize_spec(
            completed@,
            topology@,
            scalars@,
            anchors@,
            expected,
        ));
    }
    while index < cst_nodes.len()
        invariant
            index <= cst_nodes.len(),
            _fuel == cst_nodes.len() - index,
            cst@.nodes.len() == cst_nodes@.len(),
            crate::resolve_scalar_table::semantic_scalar_views_spec(scalar_values@)
                == scalars@.scalars,
            crate::resolve_anchor::alias_binding_views_spec(alias_bindings@) == anchors@.aliases,
            build.scalar_cursor <= scalar_values.len(),
            build.alias_cursor <= alias_bindings.len(),
            build@.nodes.len() == index,
            compose_semantic_node_table_tail_spec(
                atomized@,
                completed@,
                cst@,
                scalars@,
                anchors@,
                index as nat,
                _fuel as nat,
                build@,
                limits@,
            ) == expected,
            compose_profile1_semantic_node_table_spec(
                atomized@,
                quoted@,
                plain@,
                block@,
                completed@,
                cst@,
                topology_limits@,
                scalar_limits@,
                anchor_limits@,
                limits@,
            ) == semantic_node_table_finalize_spec(
                completed@,
                topology@,
                scalars@,
                anchors@,
                expected,
            ),
        decreases _fuel,
    {
        let step = match semantic_node_table_step(
            atomized,
            completed,
            cst,
            &scalars,
            &anchors,
            index,
            build.nodes.len(),
            build.collections.len(),
            build.alias_redirects.len(),
            build.scalar_cursor,
            build.alias_cursor,
            limits,
        ) {
            Err(error) => {
                proof {
                    reveal(compose_semantic_node_table_tail_spec);
                    reveal(compose_profile1_semantic_node_table_spec);
                    reveal(semantic_node_table_finalize_spec);
                }
                return Err(error);
            },
            Ok(step) => step,
        };
        proof {
            lemma_semantic_node_table_step_success_preserves_cursor_bounds(
                atomized@,
                completed@,
                cst@,
                scalars@,
                anchors@,
                index as nat,
                SemanticNodeTableStepStateView {
                    node_count: build.nodes.len() as nat,
                    collection_count: build.collections.len() as nat,
                    alias_redirect_count: build.alias_redirects.len() as nat,
                    scalar_cursor: build.scalar_cursor as nat,
                    alias_cursor: build.alias_cursor as nat,
                },
                limits@,
                step@,
            );
            reveal(compose_semantic_node_table_tail_spec);
            assert(compose_semantic_node_table_tail_spec(
                atomized@,
                completed@,
                cst@,
                scalars@,
                anchors@,
                (index + 1) as nat,
                (_fuel - 1) as nat,
                semantic_node_table_apply_step_spec(build@, step@),
                limits@,
            ) == expected);
        }
        build.apply_step(step);
        index += 1;
        _fuel -= 1;
    }
    if build.scalar_cursor != scalar_values.len() || build.alias_cursor != alias_bindings.len() {
        let error = SemanticNodeTableError::at(
            SemanticNodeTableErrorKind::InternalInvariantViolation,
            completed.source_len_bytes(),
        );
        proof {
            reveal(compose_semantic_node_table_tail_spec);
            reveal(compose_profile1_semantic_node_table_spec);
            reveal(semantic_node_table_finalize_spec);
        }
        return Err(error);
    }
    proof {
        reveal(compose_semantic_node_table_tail_spec);
    }
    let source = SemanticNodeTableSource::new(
        completed,
        topology,
        scalars,
        anchors,
        build.nodes,
        build.collections,
        build.alias_redirects,
    );
    proof {
        reveal(compose_profile1_semantic_node_table_spec);
        reveal(semantic_node_table_finalize_spec);
    }
    Ok(source)
}

} // verus!
