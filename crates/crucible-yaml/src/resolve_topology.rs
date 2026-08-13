//! Verified, bounded projection of authenticated CST topology into semantic tables.
use crate::atom::AtomizedSource;
#[allow(unused_imports)]
use crate::atom::AtomizedSourceView;
use crate::cst::{CstDocument, CstMappingEntry, CstNode, CstNodeKind, CstSequenceEntry, CstSource};
#[allow(unused_imports)]
use crate::cst::{
    CstDocumentView, CstMappingEntryView, CstNodeView, CstSequenceEntryView, CstSourceView,
};
use crate::token::CompletedTokenSource;
#[allow(unused_imports)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const SEMANTIC_TOPOLOGY_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS: u64 = 1_048_576;

pub const MAX_PROFILE1_SEMANTIC_NODES: u64 = 1_048_576;

pub const MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES: u64 = 1_048_576;

pub const MAX_PROFILE1_SEMANTIC_MAPPING_EDGES: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticTopologyLimits {
    max_document_roots: u64,
    max_nodes: u64,
    max_sequence_edges: u64,
    max_mapping_edges: u64,
}

#[verifier::ext_equal]
pub struct SemanticTopologyLimitsView {
    pub max_document_roots: u64,
    pub max_nodes: u64,
    pub max_sequence_edges: u64,
    pub max_mapping_edges: u64,
}

impl View for SemanticTopologyLimits {
    type V = SemanticTopologyLimitsView;

    closed spec fn view(&self) -> SemanticTopologyLimitsView {
        SemanticTopologyLimitsView {
            max_document_roots: self.max_document_roots,
            max_nodes: self.max_nodes,
            max_sequence_edges: self.max_sequence_edges,
            max_mapping_edges: self.max_mapping_edges,
        }
    }
}

impl SemanticTopologyLimits {
    pub fn new(
        max_document_roots: u64,
        max_nodes: u64,
        max_sequence_edges: u64,
        max_mapping_edges: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (SemanticTopologyLimitsView {
                max_document_roots,
                max_nodes,
                max_sequence_edges,
                max_mapping_edges,
            }),
    {
        Self { max_document_roots, max_nodes, max_sequence_edges, max_mapping_edges }
    }

    pub fn max_document_roots(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_document_roots,
    {
        self.max_document_roots
    }

    pub fn max_nodes(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_nodes,
    {
        self.max_nodes
    }

    pub fn max_sequence_edges(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_sequence_edges,
    {
        self.max_sequence_edges
    }

    pub fn max_mapping_edges(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_mapping_edges,
    {
        self.max_mapping_edges
    }
}

pub fn canonical_semantic_topology_limits() -> (limits: SemanticTopologyLimits)
    ensures
        limits@ == (SemanticTopologyLimitsView {
            max_document_roots: MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS,
            max_nodes: MAX_PROFILE1_SEMANTIC_NODES,
            max_sequence_edges: MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES,
            max_mapping_edges: MAX_PROFILE1_SEMANTIC_MAPPING_EDGES,
        }),
{
    SemanticTopologyLimits::new(
        MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS,
        MAX_PROFILE1_SEMANTIC_NODES,
        MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES,
        MAX_PROFILE1_SEMANTIC_MAPPING_EDGES,
    )
}

pub open spec fn semantic_topology_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn semantic_topology_effective_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit == semantic_topology_effective_limit_spec(requested, absolute),
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticDocumentRoot {
    document_index: u64,
    node_index: u64,
    byte_start: u64,
}

#[verifier::ext_equal]
pub struct SemanticDocumentRootView {
    pub document_index: u64,
    pub node_index: u64,
    pub byte_start: u64,
}

impl View for SemanticDocumentRoot {
    type V = SemanticDocumentRootView;

    closed spec fn view(&self) -> SemanticDocumentRootView {
        SemanticDocumentRootView {
            document_index: self.document_index,
            node_index: self.node_index,
            byte_start: self.byte_start,
        }
    }
}

impl SemanticDocumentRoot {
    fn from_document(document: &CstDocument, document_index: u64) -> (root: Self)
        ensures
            root@ == semantic_document_root_spec(document@, document_index),
    {
        Self {
            document_index,
            node_index: document.root_node_index(),
            byte_start: document.byte_start(),
        }
    }

    pub fn document_index(&self) -> (index: u64)
        ensures
            index == self@.document_index,
    {
        self.document_index
    }

    pub fn node_index(&self) -> (index: u64)
        ensures
            index == self@.node_index,
    {
        self.node_index
    }

    pub fn byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.byte_start,
    {
        self.byte_start
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticTopologyNode {
    cst_node_index: u64,
    kind: CstNodeKind,
    byte_start: u64,
    byte_end: u64,
    edge_start: u64,
    edge_end: u64,
}

#[verifier::ext_equal]
pub struct SemanticTopologyNodeView {
    pub cst_node_index: u64,
    pub kind: CstNodeKind,
    pub byte_start: u64,
    pub byte_end: u64,
    pub edge_start: u64,
    pub edge_end: u64,
}

impl View for SemanticTopologyNode {
    type V = SemanticTopologyNodeView;

    closed spec fn view(&self) -> SemanticTopologyNodeView {
        SemanticTopologyNodeView {
            cst_node_index: self.cst_node_index,
            kind: self.kind,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            edge_start: self.edge_start,
            edge_end: self.edge_end,
        }
    }
}

impl SemanticTopologyNode {
    fn from_cst_node(node: &CstNode, cst_node_index: u64) -> (topology: Self)
        ensures
            topology@ == semantic_topology_node_spec(node@, cst_node_index),
    {
        Self {
            cst_node_index,
            kind: node.kind(),
            byte_start: node.byte_start(),
            byte_end: node.byte_end(),
            edge_start: node.entry_start(),
            edge_end: node.entry_end(),
        }
    }

    pub fn cst_node_index(&self) -> (index: u64)
        ensures
            index == self@.cst_node_index,
    {
        self.cst_node_index
    }

    pub fn kind(&self) -> (kind: CstNodeKind)
        ensures
            kind == self@.kind,
    {
        self.kind
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticSequenceEdge {
    cst_entry_index: u64,
    child_node_index: u64,
    token_start: u64,
    token_end: u64,
}

#[verifier::ext_equal]
pub struct SemanticSequenceEdgeView {
    pub cst_entry_index: u64,
    pub child_node_index: u64,
    pub token_start: u64,
    pub token_end: u64,
}

impl View for SemanticSequenceEdge {
    type V = SemanticSequenceEdgeView;

    closed spec fn view(&self) -> SemanticSequenceEdgeView {
        SemanticSequenceEdgeView {
            cst_entry_index: self.cst_entry_index,
            child_node_index: self.child_node_index,
            token_start: self.token_start,
            token_end: self.token_end,
        }
    }
}

impl SemanticSequenceEdge {
    fn from_cst_entry(entry: &CstSequenceEntry, cst_entry_index: u64) -> (edge: Self)
        ensures
            edge@ == semantic_sequence_edge_spec(entry@, cst_entry_index),
    {
        Self {
            cst_entry_index,
            child_node_index: entry.node_index(),
            token_start: entry.token_start(),
            token_end: entry.token_end(),
        }
    }

    pub fn cst_entry_index(&self) -> (index: u64)
        ensures
            index == self@.cst_entry_index,
    {
        self.cst_entry_index
    }

    pub fn child_node_index(&self) -> (index: u64)
        ensures
            index == self@.child_node_index,
    {
        self.child_node_index
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticMappingEdge {
    cst_entry_index: u64,
    key_node_index: u64,
    value_node_index: u64,
    token_start: u64,
    token_end: u64,
}

#[verifier::ext_equal]
pub struct SemanticMappingEdgeView {
    pub cst_entry_index: u64,
    pub key_node_index: u64,
    pub value_node_index: u64,
    pub token_start: u64,
    pub token_end: u64,
}

impl View for SemanticMappingEdge {
    type V = SemanticMappingEdgeView;

    closed spec fn view(&self) -> SemanticMappingEdgeView {
        SemanticMappingEdgeView {
            cst_entry_index: self.cst_entry_index,
            key_node_index: self.key_node_index,
            value_node_index: self.value_node_index,
            token_start: self.token_start,
            token_end: self.token_end,
        }
    }
}

impl SemanticMappingEdge {
    fn from_cst_entry(entry: &CstMappingEntry, cst_entry_index: u64) -> (edge: Self)
        ensures
            edge@ == semantic_mapping_edge_spec(entry@, cst_entry_index),
    {
        Self {
            cst_entry_index,
            key_node_index: entry.key_node_index(),
            value_node_index: entry.value_node_index(),
            token_start: entry.token_start(),
            token_end: entry.token_end(),
        }
    }

    pub fn cst_entry_index(&self) -> (index: u64)
        ensures
            index == self@.cst_entry_index,
    {
        self.cst_entry_index
    }

    pub fn key_node_index(&self) -> (index: u64)
        ensures
            index == self@.key_node_index,
    {
        self.key_node_index
    }

    pub fn value_node_index(&self) -> (index: u64)
        ensures
            index == self@.value_node_index,
    {
        self.value_node_index
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
}

pub open spec fn semantic_document_root_spec(
    document: CstDocumentView,
    document_index: u64,
) -> SemanticDocumentRootView {
    SemanticDocumentRootView {
        document_index,
        node_index: document.root_node_index,
        byte_start: document.byte_start,
    }
}

pub open spec fn semantic_topology_node_spec(
    node: CstNodeView,
    cst_node_index: u64,
) -> SemanticTopologyNodeView {
    SemanticTopologyNodeView {
        cst_node_index,
        kind: node.kind,
        byte_start: node.byte_start,
        byte_end: node.byte_end,
        edge_start: node.entry_start,
        edge_end: node.entry_end,
    }
}

pub open spec fn semantic_sequence_edge_spec(
    entry: CstSequenceEntryView,
    cst_entry_index: u64,
) -> SemanticSequenceEdgeView {
    SemanticSequenceEdgeView {
        cst_entry_index,
        child_node_index: entry.node_index,
        token_start: entry.token_start,
        token_end: entry.token_end,
    }
}

pub open spec fn semantic_mapping_edge_spec(
    entry: CstMappingEntryView,
    cst_entry_index: u64,
) -> SemanticMappingEdgeView {
    SemanticMappingEdgeView {
        cst_entry_index,
        key_node_index: entry.key_node_index,
        value_node_index: entry.value_node_index,
        token_start: entry.token_start,
        token_end: entry.token_end,
    }
}

pub open spec fn semantic_document_roots_spec(documents: Seq<CstDocumentView>) -> Seq<
    SemanticDocumentRootView,
> {
    Seq::new(
        documents.len(),
        |index: int| semantic_document_root_spec(documents[index], index as u64),
    )
}

pub open spec fn semantic_topology_nodes_spec(nodes: Seq<CstNodeView>) -> Seq<
    SemanticTopologyNodeView,
> {
    Seq::new(nodes.len(), |index: int| semantic_topology_node_spec(nodes[index], index as u64))
}

pub open spec fn semantic_sequence_edges_spec(entries: Seq<CstSequenceEntryView>) -> Seq<
    SemanticSequenceEdgeView,
> {
    Seq::new(entries.len(), |index: int| semantic_sequence_edge_spec(entries[index], index as u64))
}

pub open spec fn semantic_mapping_edges_spec(entries: Seq<CstMappingEntryView>) -> Seq<
    SemanticMappingEdgeView,
> {
    Seq::new(entries.len(), |index: int| semantic_mapping_edge_spec(entries[index], index as u64))
}

pub open spec fn semantic_document_root_views_spec(values: Seq<SemanticDocumentRoot>) -> Seq<
    SemanticDocumentRootView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn semantic_topology_node_views_spec(values: Seq<SemanticTopologyNode>) -> Seq<
    SemanticTopologyNodeView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn semantic_sequence_edge_views_spec(values: Seq<SemanticSequenceEdge>) -> Seq<
    SemanticSequenceEdgeView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn semantic_mapping_edge_views_spec(values: Seq<SemanticMappingEdge>) -> Seq<
    SemanticMappingEdgeView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

proof fn lemma_semantic_document_root_views_push(
    values: Seq<SemanticDocumentRoot>,
    value: SemanticDocumentRoot,
)
    ensures
        semantic_document_root_views_spec(values.push(value)) == semantic_document_root_views_spec(
            values,
        ).push(value@),
{
    reveal(semantic_document_root_views_spec);
    assert(semantic_document_root_views_spec(values.push(value))
        =~= semantic_document_root_views_spec(values).push(value@));
}

proof fn lemma_semantic_topology_node_views_push(
    values: Seq<SemanticTopologyNode>,
    value: SemanticTopologyNode,
)
    ensures
        semantic_topology_node_views_spec(values.push(value)) == semantic_topology_node_views_spec(
            values,
        ).push(value@),
{
    reveal(semantic_topology_node_views_spec);
    assert(semantic_topology_node_views_spec(values.push(value))
        =~= semantic_topology_node_views_spec(values).push(value@));
}

proof fn lemma_semantic_sequence_edge_views_push(
    values: Seq<SemanticSequenceEdge>,
    value: SemanticSequenceEdge,
)
    ensures
        semantic_sequence_edge_views_spec(values.push(value)) == semantic_sequence_edge_views_spec(
            values,
        ).push(value@),
{
    reveal(semantic_sequence_edge_views_spec);
    assert(semantic_sequence_edge_views_spec(values.push(value))
        =~= semantic_sequence_edge_views_spec(values).push(value@));
}

proof fn lemma_semantic_mapping_edge_views_push(
    values: Seq<SemanticMappingEdge>,
    value: SemanticMappingEdge,
)
    ensures
        semantic_mapping_edge_views_spec(values.push(value)) == semantic_mapping_edge_views_spec(
            values,
        ).push(value@),
{
    reveal(semantic_mapping_edge_views_spec);
    assert(semantic_mapping_edge_views_spec(values.push(value))
        =~= semantic_mapping_edge_views_spec(values).push(value@));
}

#[derive(Debug, PartialEq, Eq)]
pub struct SemanticTopologySource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_token_transformation_version: u16,
    input_cst_transformation_version: u16,
    input_document_count: u64,
    input_node_count: u64,
    input_sequence_entry_count: u64,
    input_mapping_entry_count: u64,
    document_roots: Vec<SemanticDocumentRoot>,
    nodes: Vec<SemanticTopologyNode>,
    sequence_edges: Vec<SemanticSequenceEdge>,
    mapping_edges: Vec<SemanticMappingEdge>,
}

#[verifier::ext_equal]
pub struct SemanticTopologySourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_token_transformation_version: u16,
    pub input_cst_transformation_version: u16,
    pub input_document_count: u64,
    pub input_node_count: u64,
    pub input_sequence_entry_count: u64,
    pub input_mapping_entry_count: u64,
    pub document_roots: Seq<SemanticDocumentRootView>,
    pub nodes: Seq<SemanticTopologyNodeView>,
    pub sequence_edges: Seq<SemanticSequenceEdgeView>,
    pub mapping_edges: Seq<SemanticMappingEdgeView>,
}

impl View for SemanticTopologySource {
    type V = SemanticTopologySourceView;

    closed spec fn view(&self) -> SemanticTopologySourceView {
        SemanticTopologySourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_token_transformation_version: self.input_token_transformation_version,
            input_cst_transformation_version: self.input_cst_transformation_version,
            input_document_count: self.input_document_count,
            input_node_count: self.input_node_count,
            input_sequence_entry_count: self.input_sequence_entry_count,
            input_mapping_entry_count: self.input_mapping_entry_count,
            document_roots: semantic_document_root_views_spec(self.document_roots@),
            nodes: semantic_topology_node_views_spec(self.nodes@),
            sequence_edges: semantic_sequence_edge_views_spec(self.sequence_edges@),
            mapping_edges: semantic_mapping_edge_views_spec(self.mapping_edges@),
        }
    }
}

impl SemanticTopologySource {
    fn new(
        completed: &CompletedTokenSource,
        cst: &CstSource,
        document_roots: Vec<SemanticDocumentRoot>,
        nodes: Vec<SemanticTopologyNode>,
        sequence_edges: Vec<SemanticSequenceEdge>,
        mapping_edges: Vec<SemanticMappingEdge>,
    ) -> (source: Self)
        ensures
            source@ == (SemanticTopologySourceView {
                profile_version: completed@.profile_version,
                transformation_version: SEMANTIC_TOPOLOGY_TRANSFORMATION_VERSION,
                source_len_bytes: completed@.source_len_bytes,
                input_token_transformation_version: completed@.transformation_version,
                input_cst_transformation_version: cst@.transformation_version,
                input_document_count: document_roots@.len() as u64,
                input_node_count: nodes@.len() as u64,
                input_sequence_entry_count: sequence_edges@.len() as u64,
                input_mapping_entry_count: mapping_edges@.len() as u64,
                document_roots: semantic_document_root_views_spec(document_roots@),
                nodes: semantic_topology_node_views_spec(nodes@),
                sequence_edges: semantic_sequence_edge_views_spec(sequence_edges@),
                mapping_edges: semantic_mapping_edge_views_spec(mapping_edges@),
            }),
    {
        let input_document_count = document_roots.len() as u64;
        let input_node_count = nodes.len() as u64;
        let input_sequence_entry_count = sequence_edges.len() as u64;
        let input_mapping_entry_count = mapping_edges.len() as u64;
        Self {
            profile_version: completed.profile_version(),
            transformation_version: SEMANTIC_TOPOLOGY_TRANSFORMATION_VERSION,
            source_len_bytes: completed.source_len_bytes(),
            input_token_transformation_version: completed.transformation_version(),
            input_cst_transformation_version: cst.transformation_version(),
            input_document_count,
            input_node_count,
            input_sequence_entry_count,
            input_mapping_entry_count,
            document_roots,
            nodes,
            sequence_edges,
            mapping_edges,
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

    pub fn document_roots(&self) -> (values: &[SemanticDocumentRoot])
        ensures
            semantic_document_root_views_spec(values@) == self@.document_roots,
    {
        self.document_roots.as_slice()
    }

    pub fn nodes(&self) -> (values: &[SemanticTopologyNode])
        ensures
            semantic_topology_node_views_spec(values@) == self@.nodes,
    {
        self.nodes.as_slice()
    }

    pub fn sequence_edges(&self) -> (values: &[SemanticSequenceEdge])
        ensures
            semantic_sequence_edge_views_spec(values@) == self@.sequence_edges,
    {
        self.sequence_edges.as_slice()
    }

    pub fn mapping_edges(&self) -> (values: &[SemanticMappingEdge])
        ensures
            semantic_mapping_edge_views_spec(values@) == self@.mapping_edges,
    {
        self.mapping_edges.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum SemanticTopologyErrorKind {
    InputCompletedTokenMismatch,
    InputCstMismatch,
    DocumentRootLimitExceeded,
    NodeLimitExceeded,
    SequenceEdgeLimitExceeded,
    MappingEdgeLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticTopologyError {
    kind: SemanticTopologyErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct SemanticTopologyErrorView {
    pub kind: SemanticTopologyErrorKind,
    pub byte_offset: u64,
}

impl View for SemanticTopologyError {
    type V = SemanticTopologyErrorView;

    closed spec fn view(&self) -> SemanticTopologyErrorView {
        SemanticTopologyErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl SemanticTopologyError {
    fn at(kind: SemanticTopologyErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (SemanticTopologyErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: SemanticTopologyErrorKind)
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

pub open spec fn semantic_edge_byte_start_spec(
    completed: CompletedTokenSourceView,
    token_index: u64,
) -> u64 {
    if token_index < completed.tokens.len() {
        completed.tokens[token_index as int].byte_start
    } else {
        completed.source_len_bytes
    }
}

pub open spec fn semantic_topology_exact_source_spec(
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
) -> SemanticTopologySourceView {
    SemanticTopologySourceView {
        profile_version: completed.profile_version,
        transformation_version: SEMANTIC_TOPOLOGY_TRANSFORMATION_VERSION,
        source_len_bytes: completed.source_len_bytes,
        input_token_transformation_version: completed.transformation_version,
        input_cst_transformation_version: cst.transformation_version,
        input_document_count: cst.documents.len() as u64,
        input_node_count: cst.nodes.len() as u64,
        input_sequence_entry_count: cst.sequence_entries.len() as u64,
        input_mapping_entry_count: cst.mapping_entries.len() as u64,
        document_roots: semantic_document_roots_spec(cst.documents),
        nodes: semantic_topology_nodes_spec(cst.nodes),
        sequence_edges: semantic_sequence_edges_spec(cst.sequence_entries),
        mapping_edges: semantic_mapping_edges_spec(cst.mapping_entries),
    }
}

pub open spec fn compose_profile1_semantic_topology_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: SemanticTopologyLimitsView,
) -> Result<SemanticTopologySourceView, SemanticTopologyErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            SemanticTopologyErrorView {
                kind: SemanticTopologyErrorKind::InputCompletedTokenMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() {
        Err(
            SemanticTopologyErrorView {
                kind: SemanticTopologyErrorKind::InputCstMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else {
        let document_limit = semantic_topology_effective_limit_spec(
            limits.max_document_roots,
            MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS,
        );
        let node_limit = semantic_topology_effective_limit_spec(
            limits.max_nodes,
            MAX_PROFILE1_SEMANTIC_NODES,
        );
        let sequence_limit = semantic_topology_effective_limit_spec(
            limits.max_sequence_edges,
            MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES,
        );
        let mapping_limit = semantic_topology_effective_limit_spec(
            limits.max_mapping_edges,
            MAX_PROFILE1_SEMANTIC_MAPPING_EDGES,
        );
        if cst.documents.len() > document_limit {
            Err(
                SemanticTopologyErrorView {
                    kind: SemanticTopologyErrorKind::DocumentRootLimitExceeded,
                    byte_offset: cst.documents[document_limit as int].byte_start,
                },
            )
        } else if cst.nodes.len() > node_limit {
            Err(
                SemanticTopologyErrorView {
                    kind: SemanticTopologyErrorKind::NodeLimitExceeded,
                    byte_offset: cst.nodes[node_limit as int].byte_start,
                },
            )
        } else if cst.sequence_entries.len() > sequence_limit {
            Err(
                SemanticTopologyErrorView {
                    kind: SemanticTopologyErrorKind::SequenceEdgeLimitExceeded,
                    byte_offset: semantic_edge_byte_start_spec(
                        completed,
                        cst.sequence_entries[sequence_limit as int].token_start,
                    ),
                },
            )
        } else if cst.mapping_entries.len() > mapping_limit {
            Err(
                SemanticTopologyErrorView {
                    kind: SemanticTopologyErrorKind::MappingEdgeLimitExceeded,
                    byte_offset: semantic_edge_byte_start_spec(
                        completed,
                        cst.mapping_entries[mapping_limit as int].token_start,
                    ),
                },
            )
        } else {
            Ok(semantic_topology_exact_source_spec(completed, cst))
        }
    }
}

pub open spec fn semantic_topology_source_well_formed_spec(
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    source: SemanticTopologySourceView,
) -> bool {
    crate::cst::cst_public_semantics_spec(completed, cst) && source
        == semantic_topology_exact_source_spec(completed, cst) && source.document_roots.len()
        <= MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS && source.nodes.len() <= MAX_PROFILE1_SEMANTIC_NODES
        && source.sequence_edges.len() <= MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES
        && source.mapping_edges.len() <= MAX_PROFILE1_SEMANTIC_MAPPING_EDGES
}

pub proof fn lemma_semantic_topology_success_is_exact_and_well_formed(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: SemanticTopologyLimitsView,
    source: SemanticTopologySourceView,
)
    requires
        crate::cst::cst_public_semantics_spec(completed, cst),
        compose_profile1_semantic_topology_spec(atomized, completed, cst, limits) == Ok(source),
    ensures
        semantic_topology_source_well_formed_spec(completed, cst, source),
{
    reveal(compose_profile1_semantic_topology_spec);
    reveal(semantic_topology_source_well_formed_spec);
    reveal(semantic_topology_exact_source_spec);
    reveal(semantic_document_roots_spec);
    reveal(semantic_topology_nodes_spec);
    reveal(semantic_sequence_edges_spec);
    reveal(semantic_mapping_edges_spec);
}

pub proof fn lemma_semantic_topology_well_formed_authenticates_cst(
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    source: SemanticTopologySourceView,
)
    requires
        semantic_topology_source_well_formed_spec(completed, cst, source),
    ensures
        crate::cst::cst_public_semantics_spec(completed, cst),
        source == semantic_topology_exact_source_spec(completed, cst),
{
    reveal(semantic_topology_source_well_formed_spec);
}

fn compose_document_roots(documents: &[CstDocument]) -> (roots: Vec<SemanticDocumentRoot>)
    ensures
        semantic_document_root_views_spec(roots@) == semantic_document_roots_spec(
            crate::cst::cst_document_views_spec(documents@),
        ),
{
    let ghost document_views = crate::cst::cst_document_views_spec(documents@);
    let mut roots = Vec::new();
    let mut index = 0usize;
    proof {
        reveal(semantic_document_root_views_spec);
        reveal(semantic_document_roots_spec);
    }
    while index < documents.len()
        invariant
            document_views == crate::cst::cst_document_views_spec(documents@),
            document_views.len() == documents@.len(),
            index <= documents@.len(),
            semantic_document_root_views_spec(roots@) == Seq::new(
                index as nat,
                |position: int|
                    semantic_document_root_spec(document_views[position], position as u64),
            ),
        decreases documents.len() - index,
    {
        proof {
            crate::cst::lemma_cst_document_view_at(documents@, index as int);
        }
        let root = SemanticDocumentRoot::from_document(&documents[index], index as u64);
        proof {
            lemma_semantic_document_root_views_push(roots@, root);
            assert(Seq::new(
                (index + 1) as nat,
                |position: int|
                    semantic_document_root_spec(document_views[position], position as u64),
            ) =~= Seq::new(
                index as nat,
                |position: int|
                    semantic_document_root_spec(document_views[position], position as u64),
            ).push(root@));
        }
        roots.push(root);
        index += 1;
    }
    proof {
        reveal(semantic_document_roots_spec);
    }
    roots
}

fn compose_nodes(nodes: &[CstNode]) -> (topology: Vec<SemanticTopologyNode>)
    ensures
        semantic_topology_node_views_spec(topology@) == semantic_topology_nodes_spec(
            crate::cst::cst_node_views_spec(nodes@),
        ),
{
    let ghost node_views = crate::cst::cst_node_views_spec(nodes@);
    let mut topology = Vec::new();
    let mut index = 0usize;
    proof {
        reveal(semantic_topology_node_views_spec);
        reveal(semantic_topology_nodes_spec);
    }
    while index < nodes.len()
        invariant
            node_views == crate::cst::cst_node_views_spec(nodes@),
            node_views.len() == nodes@.len(),
            index <= nodes@.len(),
            semantic_topology_node_views_spec(topology@) == Seq::new(
                index as nat,
                |position: int| semantic_topology_node_spec(node_views[position], position as u64),
            ),
        decreases nodes.len() - index,
    {
        proof {
            crate::cst::lemma_cst_node_view_at(nodes@, index as int);
        }
        let node = SemanticTopologyNode::from_cst_node(&nodes[index], index as u64);
        proof {
            lemma_semantic_topology_node_views_push(topology@, node);
            assert(Seq::new(
                (index + 1) as nat,
                |position: int| semantic_topology_node_spec(node_views[position], position as u64),
            ) =~= Seq::new(
                index as nat,
                |position: int| semantic_topology_node_spec(node_views[position], position as u64),
            ).push(node@));
        }
        topology.push(node);
        index += 1;
    }
    proof {
        reveal(semantic_topology_nodes_spec);
    }
    topology
}

fn compose_sequence_edges(entries: &[CstSequenceEntry]) -> (edges: Vec<SemanticSequenceEdge>)
    ensures
        semantic_sequence_edge_views_spec(edges@) == semantic_sequence_edges_spec(
            crate::cst::cst_sequence_entry_views_spec(entries@),
        ),
{
    let ghost entry_views = crate::cst::cst_sequence_entry_views_spec(entries@);
    let mut edges = Vec::new();
    let mut index = 0usize;
    proof {
        reveal(semantic_sequence_edge_views_spec);
        reveal(semantic_sequence_edges_spec);
    }
    while index < entries.len()
        invariant
            entry_views == crate::cst::cst_sequence_entry_views_spec(entries@),
            entry_views.len() == entries@.len(),
            index <= entries@.len(),
            semantic_sequence_edge_views_spec(edges@) == Seq::new(
                index as nat,
                |position: int| semantic_sequence_edge_spec(entry_views[position], position as u64),
            ),
        decreases entries.len() - index,
    {
        proof {
            crate::cst::lemma_cst_sequence_entry_view_at(entries@, index as int);
        }
        let edge = SemanticSequenceEdge::from_cst_entry(&entries[index], index as u64);
        proof {
            lemma_semantic_sequence_edge_views_push(edges@, edge);
            assert(Seq::new(
                (index + 1) as nat,
                |position: int| semantic_sequence_edge_spec(entry_views[position], position as u64),
            ) =~= Seq::new(
                index as nat,
                |position: int| semantic_sequence_edge_spec(entry_views[position], position as u64),
            ).push(edge@));
        }
        edges.push(edge);
        index += 1;
    }
    proof {
        reveal(semantic_sequence_edges_spec);
    }
    edges
}

fn compose_mapping_edges(entries: &[CstMappingEntry]) -> (edges: Vec<SemanticMappingEdge>)
    ensures
        semantic_mapping_edge_views_spec(edges@) == semantic_mapping_edges_spec(
            crate::cst::cst_mapping_entry_views_spec(entries@),
        ),
{
    let ghost entry_views = crate::cst::cst_mapping_entry_views_spec(entries@);
    let mut edges = Vec::new();
    let mut index = 0usize;
    proof {
        reveal(semantic_mapping_edge_views_spec);
        reveal(semantic_mapping_edges_spec);
    }
    while index < entries.len()
        invariant
            entry_views == crate::cst::cst_mapping_entry_views_spec(entries@),
            entry_views.len() == entries@.len(),
            index <= entries@.len(),
            semantic_mapping_edge_views_spec(edges@) == Seq::new(
                index as nat,
                |position: int| semantic_mapping_edge_spec(entry_views[position], position as u64),
            ),
        decreases entries.len() - index,
    {
        proof {
            crate::cst::lemma_cst_mapping_entry_view_at(entries@, index as int);
        }
        let edge = SemanticMappingEdge::from_cst_entry(&entries[index], index as u64);
        proof {
            lemma_semantic_mapping_edge_views_push(edges@, edge);
            assert(Seq::new(
                (index + 1) as nat,
                |position: int| semantic_mapping_edge_spec(entry_views[position], position as u64),
            ) =~= Seq::new(
                index as nat,
                |position: int| semantic_mapping_edge_spec(entry_views[position], position as u64),
            ).push(edge@));
        }
        edges.push(edge);
        index += 1;
    }
    proof {
        reveal(semantic_mapping_edges_spec);
    }
    edges
}

fn semantic_edge_byte_start(completed: &CompletedTokenSource, token_index: u64) -> (offset: u64)
    ensures
        offset == semantic_edge_byte_start_spec(completed@, token_index),
{
    let tokens = completed.tokens();
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        assert(completed@.tokens == crate::token::completed_token_views_spec(tokens@));
        assert(completed@.tokens.len() == tokens@.len());
    }
    if token_index < tokens.len() as u64 {
        assert(token_index <= usize::MAX as u64);
        let index = token_index as usize;
        proof {
            crate::token::lemma_completed_token_view_at(tokens@, index as int);
        }
        let offset = tokens[index].byte_start();
        proof {
            reveal(semantic_edge_byte_start_spec);
        }
        offset
    } else {
        let offset = completed.source_len_bytes();
        proof {
            reveal(semantic_edge_byte_start_spec);
        }
        offset
    }
}

pub fn compose_profile1_semantic_topology(
    atomized: &AtomizedSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    limits: SemanticTopologyLimits,
) -> (result: Result<SemanticTopologySource, SemanticTopologyError>)
    ensures
        compose_profile1_semantic_topology_spec(atomized@, completed@, cst@, limits@)
            == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let documents = cst.documents();
    let nodes = cst.nodes();
    let sequence_entries = cst.sequence_entries();
    let mapping_entries = cst.mapping_entries();
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(crate::cst::cst_document_views_spec);
        reveal(crate::cst::cst_node_views_spec);
        reveal(crate::cst::cst_sequence_entry_views_spec);
        reveal(crate::cst::cst_mapping_entry_views_spec);
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens.len() == tokens@.len());
        assert(cst@.documents.len() == documents@.len());
        assert(cst@.nodes.len() == nodes@.len());
        assert(cst@.sequence_entries.len() == sequence_entries@.len());
        assert(cst@.mapping_entries.len() == mapping_entries@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != atomized.bom_bytes()
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = SemanticTopologyError::at(
            SemanticTopologyErrorKind::InputCompletedTokenMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_topology_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 {
        let error = SemanticTopologyError::at(
            SemanticTopologyErrorKind::InputCstMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_topology_spec);
        }
        return Err(error);
    }
    let document_limit = semantic_topology_effective_limit(
        limits.max_document_roots(),
        MAX_PROFILE1_SEMANTIC_DOCUMENT_ROOTS,
    );
    let node_limit = semantic_topology_effective_limit(
        limits.max_nodes(),
        MAX_PROFILE1_SEMANTIC_NODES,
    );
    let sequence_limit = semantic_topology_effective_limit(
        limits.max_sequence_edges(),
        MAX_PROFILE1_SEMANTIC_SEQUENCE_EDGES,
    );
    let mapping_limit = semantic_topology_effective_limit(
        limits.max_mapping_edges(),
        MAX_PROFILE1_SEMANTIC_MAPPING_EDGES,
    );
    if documents.len() as u64 > document_limit {
        let index = document_limit as usize;
        proof {
            crate::cst::lemma_cst_document_view_at(documents@, index as int);
        }
        let error = SemanticTopologyError::at(
            SemanticTopologyErrorKind::DocumentRootLimitExceeded,
            documents[index].byte_start(),
        );
        proof {
            reveal(compose_profile1_semantic_topology_spec);
        }
        return Err(error);
    }
    if nodes.len() as u64 > node_limit {
        let index = node_limit as usize;
        proof {
            crate::cst::lemma_cst_node_view_at(nodes@, index as int);
        }
        let error = SemanticTopologyError::at(
            SemanticTopologyErrorKind::NodeLimitExceeded,
            nodes[index].byte_start(),
        );
        proof {
            reveal(compose_profile1_semantic_topology_spec);
        }
        return Err(error);
    }
    if sequence_entries.len() as u64 > sequence_limit {
        let index = sequence_limit as usize;
        proof {
            crate::cst::lemma_cst_sequence_entry_view_at(sequence_entries@, index as int);
        }
        let error = SemanticTopologyError::at(
            SemanticTopologyErrorKind::SequenceEdgeLimitExceeded,
            semantic_edge_byte_start(completed, sequence_entries[index].token_start()),
        );
        proof {
            reveal(compose_profile1_semantic_topology_spec);
        }
        return Err(error);
    }
    if mapping_entries.len() as u64 > mapping_limit {
        let index = mapping_limit as usize;
        proof {
            crate::cst::lemma_cst_mapping_entry_view_at(mapping_entries@, index as int);
        }
        let error = SemanticTopologyError::at(
            SemanticTopologyErrorKind::MappingEdgeLimitExceeded,
            semantic_edge_byte_start(completed, mapping_entries[index].token_start()),
        );
        proof {
            reveal(compose_profile1_semantic_topology_spec);
        }
        return Err(error);
    }
    let document_roots = compose_document_roots(documents);
    let topology_nodes = compose_nodes(nodes);
    let sequence_edges = compose_sequence_edges(sequence_entries);
    let mapping_edges = compose_mapping_edges(mapping_entries);
    proof {
        reveal(semantic_document_root_views_spec);
        reveal(semantic_topology_node_views_spec);
        reveal(semantic_sequence_edge_views_spec);
        reveal(semantic_mapping_edge_views_spec);
        reveal(semantic_document_roots_spec);
        reveal(semantic_topology_nodes_spec);
        reveal(semantic_sequence_edges_spec);
        reveal(semantic_mapping_edges_spec);
        assert(semantic_document_root_views_spec(document_roots@) == semantic_document_roots_spec(
            cst@.documents,
        ));
        assert(semantic_topology_node_views_spec(topology_nodes@) == semantic_topology_nodes_spec(
            cst@.nodes,
        ));
        assert(semantic_sequence_edge_views_spec(sequence_edges@) == semantic_sequence_edges_spec(
            cst@.sequence_entries,
        ));
        assert(semantic_mapping_edge_views_spec(mapping_edges@) == semantic_mapping_edges_spec(
            cst@.mapping_entries,
        ));
        assert(semantic_document_root_views_spec(document_roots@).len() == document_roots@.len());
        assert(semantic_document_roots_spec(cst@.documents).len() == cst@.documents.len());
        assert(semantic_topology_node_views_spec(topology_nodes@).len() == topology_nodes@.len());
        assert(semantic_topology_nodes_spec(cst@.nodes).len() == cst@.nodes.len());
        assert(semantic_sequence_edge_views_spec(sequence_edges@).len() == sequence_edges@.len());
        assert(semantic_sequence_edges_spec(cst@.sequence_entries).len()
            == cst@.sequence_entries.len());
        assert(semantic_mapping_edge_views_spec(mapping_edges@).len() == mapping_edges@.len());
        assert(semantic_mapping_edges_spec(cst@.mapping_entries).len()
            == cst@.mapping_entries.len());
        assert(document_roots@.len() == cst@.documents.len());
        assert(topology_nodes@.len() == cst@.nodes.len());
        assert(sequence_edges@.len() == cst@.sequence_entries.len());
        assert(mapping_edges@.len() == cst@.mapping_entries.len());
    }
    let source = SemanticTopologySource::new(
        completed,
        cst,
        document_roots,
        topology_nodes,
        sequence_edges,
        mapping_edges,
    );
    proof {
        reveal(compose_profile1_semantic_topology_spec);
        reveal(semantic_topology_exact_source_spec);
        assert(source@ == semantic_topology_exact_source_spec(completed@, cst@));
    }
    Ok(source)
}

} // verus!
