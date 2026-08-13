//! Verified canonical lowering from the resolved YAML graph into an alias-transparent DAG.
//!
//! Lowering owns the exact merge-expanded input, retains one stable record for every source node,
//! replaces alias nodes and collection edges with their resolved targets, and projects only the
//! effective post-merge mapping entries. It never materializes repeated alias subtrees.
use crate::resolve_merge::{
    ExpandedMappingEntry, ExpandedMappingRecord, ExpandedSemanticGraphSource,
};
use crate::resolve_node_table::{SemanticNodeKind, SemanticNodeSlot};
use crate::resolve_topology::{SemanticDocumentRoot, SemanticSequenceEdge, SemanticTopologyNode};
use vstd::prelude::*;

verus! {

pub const CANONICAL_LOWERING_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_CANONICAL_NODES: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES: u64 =
    crate::cst::MAX_PROFILE1_CST_SEQUENCE_ENTRIES;

pub const MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES: u64 =
    crate::cst::MAX_PROFILE1_CST_MAPPING_ENTRIES;

pub const MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS: u64 = crate::cst::MAX_PROFILE1_CST_DOCUMENTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalLoweringLimits {
    max_nodes: u64,
    max_sequence_entries: u64,
    max_mapping_entries: u64,
    max_document_roots: u64,
}

#[verifier::ext_equal]
pub struct CanonicalLoweringLimitsView {
    pub max_nodes: u64,
    pub max_sequence_entries: u64,
    pub max_mapping_entries: u64,
    pub max_document_roots: u64,
}

impl View for CanonicalLoweringLimits {
    type V = CanonicalLoweringLimitsView;

    closed spec fn view(&self) -> CanonicalLoweringLimitsView {
        CanonicalLoweringLimitsView {
            max_nodes: self.max_nodes,
            max_sequence_entries: self.max_sequence_entries,
            max_mapping_entries: self.max_mapping_entries,
            max_document_roots: self.max_document_roots,
        }
    }
}

impl CanonicalLoweringLimits {
    pub fn new(
        max_nodes: u64,
        max_sequence_entries: u64,
        max_mapping_entries: u64,
        max_document_roots: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (CanonicalLoweringLimitsView {
                max_nodes,
                max_sequence_entries,
                max_mapping_entries,
                max_document_roots,
            }),
    {
        Self { max_nodes, max_sequence_entries, max_mapping_entries, max_document_roots }
    }

    pub fn max_nodes(&self) -> (value: u64)
        ensures
            value == self@.max_nodes,
    {
        self.max_nodes
    }

    pub fn max_sequence_entries(&self) -> (value: u64)
        ensures
            value == self@.max_sequence_entries,
    {
        self.max_sequence_entries
    }

    pub fn max_mapping_entries(&self) -> (value: u64)
        ensures
            value == self@.max_mapping_entries,
    {
        self.max_mapping_entries
    }

    pub fn max_document_roots(&self) -> (value: u64)
        ensures
            value == self@.max_document_roots,
    {
        self.max_document_roots
    }
}

pub fn canonical_lowering_limits() -> (limits: CanonicalLoweringLimits)
    ensures
        limits@ == canonical_lowering_limits_spec(),
{
    CanonicalLoweringLimits::new(
        MAX_PROFILE1_CANONICAL_NODES,
        MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES,
        MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES,
        MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS,
    )
}

pub open spec fn canonical_lowering_limits_spec() -> CanonicalLoweringLimitsView {
    CanonicalLoweringLimitsView {
        max_nodes: MAX_PROFILE1_CANONICAL_NODES,
        max_sequence_entries: MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES,
        max_mapping_entries: MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES,
        max_document_roots: MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS,
    }
}

pub open spec fn canonical_lowering_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (value: u64)
    ensures
        value == canonical_lowering_effective_limit_spec(requested, absolute),
        value <= absolute,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CanonicalLoweringErrorKind {
    NodeLimitExceeded,
    SequenceEntryLimitExceeded,
    MappingEntryLimitExceeded,
    DocumentRootLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalLoweringError {
    kind: CanonicalLoweringErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CanonicalLoweringErrorView {
    pub kind: CanonicalLoweringErrorKind,
    pub byte_offset: u64,
}

impl View for CanonicalLoweringError {
    type V = CanonicalLoweringErrorView;

    closed spec fn view(&self) -> CanonicalLoweringErrorView {
        CanonicalLoweringErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CanonicalLoweringError {
    fn at(kind: CanonicalLoweringErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (CanonicalLoweringErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: CanonicalLoweringErrorKind)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CanonicalYamlNodeKind {
    Scalar,
    Sequence,
    Mapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalYamlNode {
    source_node_index: u64,
    resolved_node_index: u64,
    kind: CanonicalYamlNodeKind,
    byte_start: u64,
    byte_end: u64,
    scalar_index: Option<u64>,
    collection_index: Option<u64>,
    edge_start: u64,
    edge_end: u64,
}

#[verifier::ext_equal]
pub struct CanonicalYamlNodeView {
    pub source_node_index: u64,
    pub resolved_node_index: u64,
    pub kind: CanonicalYamlNodeKind,
    pub byte_start: u64,
    pub byte_end: u64,
    pub scalar_index: Option<u64>,
    pub collection_index: Option<u64>,
    pub edge_start: u64,
    pub edge_end: u64,
}

impl View for CanonicalYamlNode {
    type V = CanonicalYamlNodeView;

    closed spec fn view(&self) -> CanonicalYamlNodeView {
        CanonicalYamlNodeView {
            source_node_index: self.source_node_index,
            resolved_node_index: self.resolved_node_index,
            kind: self.kind,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            scalar_index: self.scalar_index,
            collection_index: self.collection_index,
            edge_start: self.edge_start,
            edge_end: self.edge_end,
        }
    }
}

impl CanonicalYamlNode {
    #[allow(clippy::too_many_arguments)]  // Every canonical record field remains explicit.
    fn new(
        source_node_index: u64,
        resolved_node_index: u64,
        kind: CanonicalYamlNodeKind,
        byte_start: u64,
        byte_end: u64,
        scalar_index: Option<u64>,
        collection_index: Option<u64>,
        edge_start: u64,
        edge_end: u64,
    ) -> (node: Self)
        ensures
            node@ == (CanonicalYamlNodeView {
                source_node_index,
                resolved_node_index,
                kind,
                byte_start,
                byte_end,
                scalar_index,
                collection_index,
                edge_start,
                edge_end,
            }),
    {
        Self {
            source_node_index,
            resolved_node_index,
            kind,
            byte_start,
            byte_end,
            scalar_index,
            collection_index,
            edge_start,
            edge_end,
        }
    }

    pub fn source_node_index(&self) -> (value: u64)
        ensures
            value == self@.source_node_index,
    {
        self.source_node_index
    }

    pub fn resolved_node_index(&self) -> (value: u64)
        ensures
            value == self@.resolved_node_index,
    {
        self.resolved_node_index
    }

    pub fn kind(&self) -> (value: CanonicalYamlNodeKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn byte_start(&self) -> (value: u64)
        ensures
            value == self@.byte_start,
    {
        self.byte_start
    }

    pub fn byte_end(&self) -> (value: u64)
        ensures
            value == self@.byte_end,
    {
        self.byte_end
    }

    pub fn scalar_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.scalar_index,
    {
        self.scalar_index
    }

    pub fn collection_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.collection_index,
    {
        self.collection_index
    }

    pub fn edge_start(&self) -> (value: u64)
        ensures
            value == self@.edge_start,
    {
        self.edge_start
    }

    pub fn edge_end(&self) -> (value: u64)
        ensures
            value == self@.edge_end,
    {
        self.edge_end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalSequenceEntry {
    source_parent_node_index: u64,
    source_edge_index: u64,
    value_node_index: u64,
}

#[verifier::ext_equal]
pub struct CanonicalSequenceEntryView {
    pub source_parent_node_index: u64,
    pub source_edge_index: u64,
    pub value_node_index: u64,
}

impl View for CanonicalSequenceEntry {
    type V = CanonicalSequenceEntryView;

    closed spec fn view(&self) -> CanonicalSequenceEntryView {
        CanonicalSequenceEntryView {
            source_parent_node_index: self.source_parent_node_index,
            source_edge_index: self.source_edge_index,
            value_node_index: self.value_node_index,
        }
    }
}

impl CanonicalSequenceEntry {
    fn new(source_parent_node_index: u64, source_edge_index: u64, value_node_index: u64) -> (entry:
        Self)
        ensures
            entry@ == (CanonicalSequenceEntryView {
                source_parent_node_index,
                source_edge_index,
                value_node_index,
            }),
    {
        Self { source_parent_node_index, source_edge_index, value_node_index }
    }

    pub fn source_parent_node_index(&self) -> (value: u64)
        ensures
            value == self@.source_parent_node_index,
    {
        self.source_parent_node_index
    }

    pub fn source_edge_index(&self) -> (value: u64)
        ensures
            value == self@.source_edge_index,
    {
        self.source_edge_index
    }

    pub fn value_node_index(&self) -> (value: u64)
        ensures
            value == self@.value_node_index,
    {
        self.value_node_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalMappingEntry {
    receiver_node_index: u64,
    source_mapping_node_index: u64,
    source_edge_index: u64,
    key_node_index: u64,
    value_node_index: u64,
    inherited: bool,
}

#[verifier::ext_equal]
pub struct CanonicalMappingEntryView {
    pub receiver_node_index: u64,
    pub source_mapping_node_index: u64,
    pub source_edge_index: u64,
    pub key_node_index: u64,
    pub value_node_index: u64,
    pub inherited: bool,
}

impl View for CanonicalMappingEntry {
    type V = CanonicalMappingEntryView;

    closed spec fn view(&self) -> CanonicalMappingEntryView {
        CanonicalMappingEntryView {
            receiver_node_index: self.receiver_node_index,
            source_mapping_node_index: self.source_mapping_node_index,
            source_edge_index: self.source_edge_index,
            key_node_index: self.key_node_index,
            value_node_index: self.value_node_index,
            inherited: self.inherited,
        }
    }
}

impl CanonicalMappingEntry {
    fn new(
        receiver_node_index: u64,
        source_mapping_node_index: u64,
        source_edge_index: u64,
        key_node_index: u64,
        value_node_index: u64,
        inherited: bool,
    ) -> (entry: Self)
        ensures
            entry@ == (CanonicalMappingEntryView {
                receiver_node_index,
                source_mapping_node_index,
                source_edge_index,
                key_node_index,
                value_node_index,
                inherited,
            }),
    {
        Self {
            receiver_node_index,
            source_mapping_node_index,
            source_edge_index,
            key_node_index,
            value_node_index,
            inherited,
        }
    }

    pub fn receiver_node_index(&self) -> (value: u64)
        ensures
            value == self@.receiver_node_index,
    {
        self.receiver_node_index
    }

    pub fn source_mapping_node_index(&self) -> (value: u64)
        ensures
            value == self@.source_mapping_node_index,
    {
        self.source_mapping_node_index
    }

    pub fn source_edge_index(&self) -> (value: u64)
        ensures
            value == self@.source_edge_index,
    {
        self.source_edge_index
    }

    pub fn key_node_index(&self) -> (value: u64)
        ensures
            value == self@.key_node_index,
    {
        self.key_node_index
    }

    pub fn value_node_index(&self) -> (value: u64)
        ensures
            value == self@.value_node_index,
    {
        self.value_node_index
    }

    pub fn inherited(&self) -> (value: bool)
        ensures
            value == self@.inherited,
    {
        self.inherited
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalDocumentRoot {
    document_index: u64,
    source_node_index: u64,
    value_node_index: u64,
    byte_start: u64,
}

#[verifier::ext_equal]
pub struct CanonicalDocumentRootView {
    pub document_index: u64,
    pub source_node_index: u64,
    pub value_node_index: u64,
    pub byte_start: u64,
}

impl View for CanonicalDocumentRoot {
    type V = CanonicalDocumentRootView;

    closed spec fn view(&self) -> CanonicalDocumentRootView {
        CanonicalDocumentRootView {
            document_index: self.document_index,
            source_node_index: self.source_node_index,
            value_node_index: self.value_node_index,
            byte_start: self.byte_start,
        }
    }
}

impl CanonicalDocumentRoot {
    fn new(
        document_index: u64,
        source_node_index: u64,
        value_node_index: u64,
        byte_start: u64,
    ) -> (root: Self)
        ensures
            root@ == (CanonicalDocumentRootView {
                document_index,
                source_node_index,
                value_node_index,
                byte_start,
            }),
    {
        Self { document_index, source_node_index, value_node_index, byte_start }
    }

    pub fn document_index(&self) -> (value: u64)
        ensures
            value == self@.document_index,
    {
        self.document_index
    }

    pub fn source_node_index(&self) -> (value: u64)
        ensures
            value == self@.source_node_index,
    {
        self.source_node_index
    }

    pub fn value_node_index(&self) -> (value: u64)
        ensures
            value == self@.value_node_index,
    {
        self.value_node_index
    }

    pub fn byte_start(&self) -> (value: u64)
        ensures
            value == self@.byte_start,
    {
        self.byte_start
    }
}

pub open spec fn canonical_yaml_node_views_spec(values: Seq<CanonicalYamlNode>) -> Seq<
    CanonicalYamlNodeView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn canonical_sequence_entry_views_spec(values: Seq<CanonicalSequenceEntry>) -> Seq<
    CanonicalSequenceEntryView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn canonical_mapping_entry_views_spec(values: Seq<CanonicalMappingEntry>) -> Seq<
    CanonicalMappingEntryView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn canonical_document_root_views_spec(values: Seq<CanonicalDocumentRoot>) -> Seq<
    CanonicalDocumentRootView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalYamlGraphSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    expanded_reference_count: u64,
    input: ExpandedSemanticGraphSource,
    nodes: Vec<CanonicalYamlNode>,
    sequence_entries: Vec<CanonicalSequenceEntry>,
    mapping_entries: Vec<CanonicalMappingEntry>,
    document_roots: Vec<CanonicalDocumentRoot>,
}

#[verifier::ext_equal]
pub struct CanonicalYamlGraphSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub expanded_reference_count: u64,
    pub input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    pub nodes: Seq<CanonicalYamlNodeView>,
    pub sequence_entries: Seq<CanonicalSequenceEntryView>,
    pub mapping_entries: Seq<CanonicalMappingEntryView>,
    pub document_roots: Seq<CanonicalDocumentRootView>,
}

impl View for CanonicalYamlGraphSource {
    type V = CanonicalYamlGraphSourceView;

    closed spec fn view(&self) -> CanonicalYamlGraphSourceView {
        CanonicalYamlGraphSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            expanded_reference_count: self.expanded_reference_count,
            input: self.input@,
            nodes: canonical_yaml_node_views_spec(self.nodes@),
            sequence_entries: canonical_sequence_entry_views_spec(self.sequence_entries@),
            mapping_entries: canonical_mapping_entry_views_spec(self.mapping_entries@),
            document_roots: canonical_document_root_views_spec(self.document_roots@),
        }
    }
}

impl CanonicalYamlGraphSource {
    fn new(
        input: ExpandedSemanticGraphSource,
        nodes: Vec<CanonicalYamlNode>,
        sequence_entries: Vec<CanonicalSequenceEntry>,
        mapping_entries: Vec<CanonicalMappingEntry>,
        document_roots: Vec<CanonicalDocumentRoot>,
    ) -> (source: Self)
        ensures
            source@ == (CanonicalYamlGraphSourceView {
                profile_version: input@.profile_version,
                transformation_version: CANONICAL_LOWERING_TRANSFORMATION_VERSION,
                source_len_bytes: input@.source_len_bytes,
                input_node_count: input@.input_node_count,
                expanded_reference_count: input@.expanded_reference_count,
                input: input@,
                nodes: canonical_yaml_node_views_spec(nodes@),
                sequence_entries: canonical_sequence_entry_views_spec(sequence_entries@),
                mapping_entries: canonical_mapping_entry_views_spec(mapping_entries@),
                document_roots: canonical_document_root_views_spec(document_roots@),
            }),
    {
        let profile_version = input.profile_version();
        let source_len_bytes = input.source_len_bytes();
        let input_node_count = input.input_node_count();
        let expanded_reference_count = input.expanded_reference_count();
        Self {
            profile_version,
            transformation_version: CANONICAL_LOWERING_TRANSFORMATION_VERSION,
            source_len_bytes,
            input_node_count,
            expanded_reference_count,
            input,
            nodes,
            sequence_entries,
            mapping_entries,
            document_roots,
        }
    }

    pub fn profile_version(&self) -> (value: u16)
        ensures
            value == self@.profile_version,
    {
        self.profile_version
    }

    pub fn transformation_version(&self) -> (value: u16)
        ensures
            value == self@.transformation_version,
    {
        self.transformation_version
    }

    pub fn source_len_bytes(&self) -> (value: u64)
        ensures
            value == self@.source_len_bytes,
    {
        self.source_len_bytes
    }

    pub fn input_node_count(&self) -> (value: u64)
        ensures
            value == self@.input_node_count,
    {
        self.input_node_count
    }

    pub fn expanded_reference_count(&self) -> (value: u64)
        ensures
            value == self@.expanded_reference_count,
    {
        self.expanded_reference_count
    }

    pub fn input(&self) -> (value: &ExpandedSemanticGraphSource)
        ensures
            value@ == self@.input,
    {
        &self.input
    }

    pub fn nodes(&self) -> (values: &[CanonicalYamlNode])
        ensures
            canonical_yaml_node_views_spec(values@) == self@.nodes,
    {
        self.nodes.as_slice()
    }

    pub fn sequence_entries(&self) -> (values: &[CanonicalSequenceEntry])
        ensures
            canonical_sequence_entry_views_spec(values@) == self@.sequence_entries,
    {
        self.sequence_entries.as_slice()
    }

    pub fn mapping_entries(&self) -> (values: &[CanonicalMappingEntry])
        ensures
            canonical_mapping_entry_views_spec(values@) == self@.mapping_entries,
    {
        self.mapping_entries.as_slice()
    }

    pub fn document_roots(&self) -> (values: &[CanonicalDocumentRoot])
        ensures
            canonical_document_root_views_spec(values@) == self@.document_roots,
    {
        self.document_roots.as_slice()
    }
}

#[verifier::ext_equal]
pub struct CanonicalLoweringBuildView {
    pub nodes: Seq<CanonicalYamlNodeView>,
    pub sequence_entries: Seq<CanonicalSequenceEntryView>,
    pub mapping_entries: Seq<CanonicalMappingEntryView>,
    pub document_roots: Seq<CanonicalDocumentRootView>,
}

pub closed spec fn canonical_follow_alias_tail_spec(
    current: u64,
    fuel: nat,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
) -> Result<u64, CanonicalLoweringErrorView>
    decreases fuel,
{
    if fuel == 0 {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: if current < slots.len() {
                    slots[current as int].byte_start
                } else {
                    0
                },
            },
        )
    } else if current >= slots.len() {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if slots[current as int].kind != SemanticNodeKind::Alias {
        Ok(current)
    } else {
        match slots[current as int].alias_target_node_index {
            Some(target) => canonical_follow_alias_tail_spec(target, (fuel - 1) as nat, slots),
            None => Err(
                CanonicalLoweringErrorView {
                    kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                    byte_offset: slots[current as int].byte_start,
                },
            ),
        }
    }
}

pub open spec fn canonical_follow_alias_spec(
    node_index: u64,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
) -> Result<u64, CanonicalLoweringErrorView> {
    canonical_follow_alias_tail_spec(node_index, slots.len() as nat, slots)
}

pub closed spec fn canonical_mapping_record_index_tail_spec(
    node_index: u64,
    index: nat,
    fuel: nat,
    mappings: Seq<crate::resolve_merge::ExpandedMappingRecordView>,
) -> Option<nat>
    decreases fuel,
{
    if index >= mappings.len() || fuel == 0 {
        None
    } else if mappings[index as int].node_index == node_index {
        Some(index)
    } else {
        canonical_mapping_record_index_tail_spec(
            node_index,
            (index + 1) as nat,
            (fuel - 1) as nat,
            mappings,
        )
    }
}

pub open spec fn canonical_mapping_record_index_spec(
    node_index: u64,
    mappings: Seq<crate::resolve_merge::ExpandedMappingRecordView>,
) -> Option<nat> {
    canonical_mapping_record_index_tail_spec(node_index, 0, mappings.len() as nat, mappings)
}

pub closed spec fn lower_sequence_edges_tail_spec(
    parent_node_index: u64,
    edge_index: nat,
    edge_end: nat,
    fuel: nat,
    output: Seq<CanonicalSequenceEntryView>,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    limit: u64,
) -> Result<Seq<CanonicalSequenceEntryView>, CanonicalLoweringErrorView>
    decreases fuel,
{
    if edge_index > edge_end || edge_end > edges.len() {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if edge_index == edge_end {
        Ok(output)
    } else if fuel == 0 {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let edge = edges[edge_index as int];
        match canonical_follow_alias_spec(edge.child_node_index, slots) {
            Err(error) => Err(error),
            Ok(value_node_index) => if edge.child_node_index >= slots.len() || value_node_index
                >= slots.len() {
                Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                )
            } else if output.len() >= limit {
                Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::SequenceEntryLimitExceeded,
                        byte_offset: slots[edge.child_node_index as int].byte_start,
                    },
                )
            } else {
                lower_sequence_edges_tail_spec(
                    parent_node_index,
                    (edge_index + 1) as nat,
                    edge_end,
                    (fuel - 1) as nat,
                    output.push(
                        CanonicalSequenceEntryView {
                            source_parent_node_index: parent_node_index,
                            source_edge_index: edge_index as u64,
                            value_node_index,
                        },
                    ),
                    slots,
                    edges,
                    limit,
                )
            },
        }
    }
}

pub closed spec fn lower_mapping_entries_tail_spec(
    receiver_node_index: u64,
    entry_index: nat,
    entry_end: nat,
    fuel: nat,
    output: Seq<CanonicalMappingEntryView>,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    entries: Seq<crate::resolve_merge::ExpandedMappingEntryView>,
    limit: u64,
) -> Result<Seq<CanonicalMappingEntryView>, CanonicalLoweringErrorView>
    decreases fuel,
{
    if entry_index > entry_end || entry_end > entries.len() {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if entry_index == entry_end {
        Ok(output)
    } else if fuel == 0 {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let entry = entries[entry_index as int];
        match canonical_follow_alias_spec(entry.key_node_index, slots) {
            Err(error) => Err(error),
            Ok(key_node_index) => match canonical_follow_alias_spec(entry.value_node_index, slots) {
                Err(error) => Err(error),
                Ok(value_node_index) => if entry.key_node_index >= slots.len() || key_node_index
                    >= slots.len() || value_node_index >= slots.len() {
                    Err(
                        CanonicalLoweringErrorView {
                            kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                            byte_offset: 0,
                        },
                    )
                } else if output.len() >= limit {
                    Err(
                        CanonicalLoweringErrorView {
                            kind: CanonicalLoweringErrorKind::MappingEntryLimitExceeded,
                            byte_offset: slots[entry.key_node_index as int].byte_start,
                        },
                    )
                } else {
                    lower_mapping_entries_tail_spec(
                        receiver_node_index,
                        (entry_index + 1) as nat,
                        entry_end,
                        (fuel - 1) as nat,
                        output.push(
                            CanonicalMappingEntryView {
                                receiver_node_index,
                                source_mapping_node_index: entry.source_mapping_node_index,
                                source_edge_index: entry.source_edge_index,
                                key_node_index,
                                value_node_index,
                                inherited: entry.inherited,
                            },
                        ),
                        slots,
                        entries,
                        limit,
                    )
                },
            },
        }
    }
}

pub open spec fn append_canonical_node_spec(
    source_node_index: u64,
    resolved_node_index: u64,
    build: CanonicalLoweringBuildView,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    topology_nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mappings: Seq<crate::resolve_merge::ExpandedMappingRecordView>,
    mapping_entries: Seq<crate::resolve_merge::ExpandedMappingEntryView>,
    limits: CanonicalLoweringLimitsView,
) -> Result<CanonicalLoweringBuildView, CanonicalLoweringErrorView> {
    if source_node_index >= slots.len() || source_node_index >= topology_nodes.len()
        || resolved_node_index >= slots.len() || resolved_node_index >= topology_nodes.len() {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let source = slots[source_node_index as int];
        let resolved = slots[resolved_node_index as int];
        let node_limit = canonical_lowering_effective_limit_spec(
            limits.max_nodes,
            MAX_PROFILE1_CANONICAL_NODES,
        );
        if build.nodes.len() >= node_limit {
            Err(
                CanonicalLoweringErrorView {
                    kind: CanonicalLoweringErrorKind::NodeLimitExceeded,
                    byte_offset: source.byte_start,
                },
            )
        } else if source.kind == SemanticNodeKind::Alias {
            if resolved_node_index >= build.nodes.len() {
                Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: source.byte_start,
                    },
                )
            } else {
                let target = build.nodes[resolved_node_index as int];
                Ok(
                    CanonicalLoweringBuildView {
                        nodes: build.nodes.push(
                            CanonicalYamlNodeView {
                                source_node_index,
                                resolved_node_index,
                                kind: target.kind,
                                byte_start: source.byte_start,
                                byte_end: source.byte_end,
                                scalar_index: target.scalar_index,
                                collection_index: target.collection_index,
                                edge_start: target.edge_start,
                                edge_end: target.edge_end,
                            },
                        ),
                        sequence_entries: build.sequence_entries,
                        mapping_entries: build.mapping_entries,
                        document_roots: build.document_roots,
                    },
                )
            }
        } else if resolved.kind == SemanticNodeKind::Scalar {
            match resolved.value_index {
                None => Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: source.byte_start,
                    },
                ),
                Some(scalar_index) => Ok(
                    CanonicalLoweringBuildView {
                        nodes: build.nodes.push(
                            CanonicalYamlNodeView {
                                source_node_index,
                                resolved_node_index,
                                kind: CanonicalYamlNodeKind::Scalar,
                                byte_start: source.byte_start,
                                byte_end: source.byte_end,
                                scalar_index: Some(scalar_index),
                                collection_index: None,
                                edge_start: 0,
                                edge_end: 0,
                            },
                        ),
                        sequence_entries: build.sequence_entries,
                        mapping_entries: build.mapping_entries,
                        document_roots: build.document_roots,
                    },
                ),
            }
        } else if resolved.kind == SemanticNodeKind::Sequence {
            match resolved.value_index {
                None => Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: source.byte_start,
                    },
                ),
                Some(collection_index) => {
                    let topology = topology_nodes[resolved_node_index as int];
                    let start = build.sequence_entries.len();
                    let sequence_limit = canonical_lowering_effective_limit_spec(
                        limits.max_sequence_entries,
                        MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES,
                    );
                    if topology.edge_start > topology.edge_end || topology.edge_end
                        > sequence_edges.len() {
                        Err(
                            CanonicalLoweringErrorView {
                                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                                byte_offset: source.byte_start,
                            },
                        )
                    } else {
                        match lower_sequence_edges_tail_spec(
                            resolved_node_index,
                            topology.edge_start as nat,
                            topology.edge_end as nat,
                            if topology.edge_start <= topology.edge_end {
                                (topology.edge_end - topology.edge_start) as nat
                            } else {
                                0nat
                            },
                            build.sequence_entries,
                            slots,
                            sequence_edges,
                            sequence_limit,
                        ) {
                            Err(error) => Err(error),
                            Ok(after) => Ok(
                                CanonicalLoweringBuildView {
                                    nodes: build.nodes.push(
                                        CanonicalYamlNodeView {
                                            source_node_index,
                                            resolved_node_index,
                                            kind: CanonicalYamlNodeKind::Sequence,
                                            byte_start: source.byte_start,
                                            byte_end: source.byte_end,
                                            scalar_index: None,
                                            collection_index: Some(collection_index),
                                            edge_start: start as u64,
                                            edge_end: after.len() as u64,
                                        },
                                    ),
                                    sequence_entries: after,
                                    mapping_entries: build.mapping_entries,
                                    document_roots: build.document_roots,
                                },
                            ),
                        }
                    }
                },
            }
        } else if resolved.kind == SemanticNodeKind::Mapping {
            match resolved.value_index {
                None => Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: source.byte_start,
                    },
                ),
                Some(collection_index) => match canonical_mapping_record_index_spec(
                    resolved_node_index,
                    mappings,
                ) {
                    None => Err(
                        CanonicalLoweringErrorView {
                            kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                            byte_offset: source.byte_start,
                        },
                    ),
                    Some(record_index) => {
                        let record = mappings[record_index as int];
                        let start = build.mapping_entries.len();
                        let mapping_limit = canonical_lowering_effective_limit_spec(
                            limits.max_mapping_entries,
                            MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES,
                        );
                        if record.entry_start > record.entry_end || record.entry_end
                            > mapping_entries.len() {
                            Err(
                                CanonicalLoweringErrorView {
                                    kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                                    byte_offset: source.byte_start,
                                },
                            )
                        } else {
                            match lower_mapping_entries_tail_spec(
                                resolved_node_index,
                                record.entry_start as nat,
                                record.entry_end as nat,
                                if record.entry_start <= record.entry_end {
                                    (record.entry_end - record.entry_start) as nat
                                } else {
                                    0nat
                                },
                                build.mapping_entries,
                                slots,
                                mapping_entries,
                                mapping_limit,
                            ) {
                                Err(error) => Err(error),
                                Ok(after) => Ok(
                                    CanonicalLoweringBuildView {
                                        nodes: build.nodes.push(
                                            CanonicalYamlNodeView {
                                                source_node_index,
                                                resolved_node_index,
                                                kind: CanonicalYamlNodeKind::Mapping,
                                                byte_start: source.byte_start,
                                                byte_end: source.byte_end,
                                                scalar_index: None,
                                                collection_index: Some(collection_index),
                                                edge_start: start as u64,
                                                edge_end: after.len() as u64,
                                            },
                                        ),
                                        sequence_entries: build.sequence_entries,
                                        mapping_entries: after,
                                        document_roots: build.document_roots,
                                    },
                                ),
                            }
                        }
                    },
                },
            }
        } else {
            Err(
                CanonicalLoweringErrorView {
                    kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                    byte_offset: source.byte_start,
                },
            )
        }
    }
}

pub closed spec fn lower_canonical_nodes_tail_spec(
    node_index: nat,
    fuel: nat,
    build: CanonicalLoweringBuildView,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    topology_nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mappings: Seq<crate::resolve_merge::ExpandedMappingRecordView>,
    mapping_entries: Seq<crate::resolve_merge::ExpandedMappingEntryView>,
    limits: CanonicalLoweringLimitsView,
) -> Result<CanonicalLoweringBuildView, CanonicalLoweringErrorView>
    decreases fuel,
{
    if node_index >= slots.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: slots[node_index as int].byte_start,
            },
        )
    } else {
        match canonical_follow_alias_spec(node_index as u64, slots) {
            Err(error) => Err(error),
            Ok(resolved_node_index) => match append_canonical_node_spec(
                node_index as u64,
                resolved_node_index,
                build,
                slots,
                topology_nodes,
                sequence_edges,
                mappings,
                mapping_entries,
                limits,
            ) {
                Err(error) => Err(error),
                Ok(after) => lower_canonical_nodes_tail_spec(
                    (node_index + 1) as nat,
                    (fuel - 1) as nat,
                    after,
                    slots,
                    topology_nodes,
                    sequence_edges,
                    mappings,
                    mapping_entries,
                    limits,
                ),
            },
        }
    }
}

pub closed spec fn lower_document_roots_tail_spec(
    root_index: nat,
    fuel: nat,
    build: CanonicalLoweringBuildView,
    roots: Seq<crate::resolve_topology::SemanticDocumentRootView>,
    slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    limits: CanonicalLoweringLimitsView,
) -> Result<CanonicalLoweringBuildView, CanonicalLoweringErrorView>
    decreases fuel,
{
    if root_index >= roots.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: roots[root_index as int].byte_start,
            },
        )
    } else {
        let root = roots[root_index as int];
        match canonical_follow_alias_spec(root.node_index, slots) {
            Err(error) => Err(error),
            Ok(value_node_index) => {
                let root_limit = canonical_lowering_effective_limit_spec(
                    limits.max_document_roots,
                    MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS,
                );
                if value_node_index >= build.nodes.len() {
                    Err(
                        CanonicalLoweringErrorView {
                            kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                            byte_offset: root.byte_start,
                        },
                    )
                } else if build.document_roots.len() >= root_limit {
                    Err(
                        CanonicalLoweringErrorView {
                            kind: CanonicalLoweringErrorKind::DocumentRootLimitExceeded,
                            byte_offset: root.byte_start,
                        },
                    )
                } else {
                    lower_document_roots_tail_spec(
                        (root_index + 1) as nat,
                        (fuel - 1) as nat,
                        CanonicalLoweringBuildView {
                            nodes: build.nodes,
                            sequence_entries: build.sequence_entries,
                            mapping_entries: build.mapping_entries,
                            document_roots: build.document_roots.push(
                                CanonicalDocumentRootView {
                                    document_index: root.document_index,
                                    source_node_index: root.node_index,
                                    value_node_index,
                                    byte_start: root.byte_start,
                                },
                            ),
                        },
                        roots,
                        slots,
                        limits,
                    )
                }
            },
        }
    }
}

pub open spec fn lower_profile1_canonical_graph_spec(
    input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: CanonicalLoweringLimitsView,
) -> Result<CanonicalYamlGraphSourceView, CanonicalLoweringErrorView> {
    let table = input.input.structural_keys.scalar_keys.graph.node_table;
    let topology = table.topology;
    let initial = CanonicalLoweringBuildView {
        nodes: Seq::empty(),
        sequence_entries: Seq::empty(),
        mapping_entries: Seq::empty(),
        document_roots: Seq::empty(),
    };
    match lower_canonical_nodes_tail_spec(
        0,
        table.nodes.len() as nat,
        initial,
        table.nodes,
        topology.nodes,
        topology.sequence_edges,
        input.mappings,
        input.entries,
        limits,
    ) {
        Err(error) => Err(error),
        Ok(nodes_build) => match lower_document_roots_tail_spec(
            0,
            topology.document_roots.len() as nat,
            nodes_build,
            topology.document_roots,
            table.nodes,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(build) => Ok(
                CanonicalYamlGraphSourceView {
                    profile_version: input.profile_version,
                    transformation_version: CANONICAL_LOWERING_TRANSFORMATION_VERSION,
                    source_len_bytes: input.source_len_bytes,
                    input_node_count: input.input_node_count,
                    expanded_reference_count: input.expanded_reference_count,
                    input,
                    nodes: build.nodes,
                    sequence_entries: build.sequence_entries,
                    mapping_entries: build.mapping_entries,
                    document_roots: build.document_roots,
                },
            ),
        },
    }
}

proof fn lemma_canonical_yaml_node_views_push(
    values: Seq<CanonicalYamlNode>,
    value: CanonicalYamlNode,
)
    ensures
        canonical_yaml_node_views_spec(values.push(value)) == canonical_yaml_node_views_spec(
            values,
        ).push(value@),
{
    reveal(canonical_yaml_node_views_spec);
    assert(canonical_yaml_node_views_spec(values.push(value)) =~= canonical_yaml_node_views_spec(
        values,
    ).push(value@));
}

proof fn lemma_canonical_sequence_entry_views_push(
    values: Seq<CanonicalSequenceEntry>,
    value: CanonicalSequenceEntry,
)
    ensures
        canonical_sequence_entry_views_spec(values.push(value))
            == canonical_sequence_entry_views_spec(values).push(value@),
{
    reveal(canonical_sequence_entry_views_spec);
    assert(canonical_sequence_entry_views_spec(values.push(value))
        =~= canonical_sequence_entry_views_spec(values).push(value@));
}

proof fn lemma_canonical_mapping_entry_views_push(
    values: Seq<CanonicalMappingEntry>,
    value: CanonicalMappingEntry,
)
    ensures
        canonical_mapping_entry_views_spec(values.push(value))
            == canonical_mapping_entry_views_spec(values).push(value@),
{
    reveal(canonical_mapping_entry_views_spec);
    assert(canonical_mapping_entry_views_spec(values.push(value))
        =~= canonical_mapping_entry_views_spec(values).push(value@));
}

proof fn lemma_canonical_document_root_views_push(
    values: Seq<CanonicalDocumentRoot>,
    value: CanonicalDocumentRoot,
)
    ensures
        canonical_document_root_views_spec(values.push(value))
            == canonical_document_root_views_spec(values).push(value@),
{
    reveal(canonical_document_root_views_spec);
    assert(canonical_document_root_views_spec(values.push(value))
        =~= canonical_document_root_views_spec(values).push(value@));
}

fn canonical_follow_alias(node_index: u64, slots: &[SemanticNodeSlot]) -> (result: Result<
    u64,
    CanonicalLoweringError,
>)
    ensures
        match result {
            Ok(index) => index < slots@.len(),
            Err(_) => true,
        },
        canonical_follow_alias_spec(
            node_index,
            crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
        ) == match result {
            Ok(value) => Ok(value),
            Err(error) => Err(error@),
        },
{
    let mut current = node_index;
    let mut fuel = slots.len();
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost expected = canonical_follow_alias_tail_spec(
        node_index,
        slots@.len() as nat,
        slot_views,
    );
    proof {
        reveal(canonical_follow_alias_spec);
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(slot_views.len() == slots@.len());
    }
    while fuel > 0
        invariant
            fuel <= slots.len(),
            slot_views.len() == slots@.len(),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            expected == canonical_follow_alias_tail_spec(current, fuel as nat, slot_views),
            canonical_follow_alias_spec(node_index, slot_views) == expected,
        decreases fuel,
    {
        if current >= slots.len() as u64 {
            proof {
                reveal(canonical_follow_alias_tail_spec);
                reveal(canonical_follow_alias_spec);
                assert(current >= slot_views.len());
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::InternalInvariantViolation,
                    0,
                ),
            );
        }
        let slot = &slots[current as usize];
        let kind = slot.kind();
        proof {
            assert(0int <= current as int);
            assert((current as int) < slots.len() as int);
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slot_views[current as int] == slots[current as int]@);
            assert(slot_views[current as int].kind == kind);
        }
        if kind != SemanticNodeKind::Alias {
            proof {
                reveal(canonical_follow_alias_tail_spec);
                reveal(canonical_follow_alias_spec);
                assert(expected == Ok(current));
            }
            return Ok(current);
        }
        let alias_target = slot.alias_target_node_index();
        proof {
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slot_views[current as int].alias_target_node_index == alias_target);
        }
        let target = match alias_target {
            Some(target) => target,
            None => {
                let byte_offset = slot.byte_start();
                proof {
                    reveal(canonical_follow_alias_tail_spec);
                    reveal(canonical_follow_alias_spec);
                    reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
                    assert(expected == Err(
                        CanonicalLoweringErrorView {
                            kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                            byte_offset,
                        },
                    ));
                }
                return Err(
                    CanonicalLoweringError::at(
                        CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset,
                    ),
                );
            },
        };
        proof {
            reveal(canonical_follow_alias_tail_spec);
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(expected == canonical_follow_alias_tail_spec(
                target,
                (fuel - 1) as nat,
                slot_views,
            ));
        }
        current = target;
        fuel -= 1;
    }
    let offset = if current < slots.len() as u64 {
        slots[current as usize].byte_start()
    } else {
        0
    };
    proof {
        reveal(canonical_follow_alias_tail_spec);
        reveal(canonical_follow_alias_spec);
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(expected == Err(
            CanonicalLoweringErrorView {
                kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                byte_offset: offset,
            },
        ));
    }
    Err(CanonicalLoweringError::at(CanonicalLoweringErrorKind::InternalInvariantViolation, offset))
}

fn canonical_mapping_record_index(node_index: u64, mappings: &[ExpandedMappingRecord]) -> (result:
    Option<usize>)
    ensures
        match result {
            Some(index) => index < mappings@.len(),
            None => true,
        },
        match result {
            Some(index) => canonical_mapping_record_index_spec(
                node_index,
                crate::resolve_merge::expanded_mapping_record_views_spec(mappings@),
            ) == Some(index as nat),
            None => canonical_mapping_record_index_spec(
                node_index,
                crate::resolve_merge::expanded_mapping_record_views_spec(mappings@),
            ).is_none(),
        },
{
    let ghost mapping_views = crate::resolve_merge::expanded_mapping_record_views_spec(mappings@);
    let ghost expected = canonical_mapping_record_index_tail_spec(
        node_index,
        0,
        mappings@.len() as nat,
        mapping_views,
    );
    proof {
        reveal(canonical_mapping_record_index_spec);
    }
    let mut index = 0usize;
    let mut _fuel = mappings.len();
    while index < mappings.len()
        invariant
            index <= mappings.len(),
            _fuel == mappings.len() - index,
            mapping_views == crate::resolve_merge::expanded_mapping_record_views_spec(mappings@),
            canonical_mapping_record_index_spec(node_index, mapping_views) == expected,
            expected == canonical_mapping_record_index_tail_spec(
                node_index,
                index as nat,
                _fuel as nat,
                mapping_views,
            ),
        decreases _fuel,
    {
        proof {
            reveal(canonical_mapping_record_index_tail_spec);
            reveal(crate::resolve_merge::expanded_mapping_record_views_spec);
            assert(mapping_views[index as int] == mappings[index as int]@);
        }
        if mappings[index].node_index() == node_index {
            proof {
                reveal(canonical_mapping_record_index_tail_spec);
                assert(expected == Some(index as nat));
            }
            return Some(index);
        }
        index += 1;
        _fuel -= 1;
    }
    proof {
        reveal(canonical_mapping_record_index_tail_spec);
    }
    None
}

#[allow(clippy::too_many_arguments)]  // Mirrors the exact pure sequence-edge transition.
fn lower_sequence_edges(
    parent_node_index: u64,
    edge_start: usize,
    edge_end: usize,
    output: &mut Vec<CanonicalSequenceEntry>,
    slots: &[SemanticNodeSlot],
    edges: &[SemanticSequenceEdge],
    limit: u64,
) -> (result: Result<(), CanonicalLoweringError>)
    ensures
        lower_sequence_edges_tail_spec(
            parent_node_index,
            edge_start as nat,
            edge_end as nat,
            if edge_start <= edge_end {
                (edge_end - edge_start) as nat
            } else {
                0nat
            },
            canonical_sequence_entry_views_spec(old(output)@),
            crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
            limit,
        ) == match result {
            Ok(()) => Ok(canonical_sequence_entry_views_spec(final(output)@)),
            Err(error) => Err(error@),
        },
{
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost edge_views = crate::resolve_topology::semantic_sequence_edge_views_spec(edges@);
    let ghost initial_output = canonical_sequence_entry_views_spec(output@);
    let ghost expected = lower_sequence_edges_tail_spec(
        parent_node_index,
        edge_start as nat,
        edge_end as nat,
        if edge_start <= edge_end {
            (edge_end - edge_start) as nat
        } else {
            0nat
        },
        initial_output,
        slot_views,
        edge_views,
        limit,
    );
    if edge_start > edge_end || edge_end > edges.len() {
        proof {
            reveal(lower_sequence_edges_tail_spec);
        }
        return Err(
            CanonicalLoweringError::at(CanonicalLoweringErrorKind::InternalInvariantViolation, 0),
        );
    }
    let mut index = edge_start;
    while index < edge_end
        invariant
            edge_start <= index,
            index <= edge_end,
            edge_end <= edges.len(),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            edge_views == crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
            initial_output == canonical_sequence_entry_views_spec(old(output)@),
            lower_sequence_edges_tail_spec(
                parent_node_index,
                edge_start as nat,
                edge_end as nat,
                (edge_end - edge_start) as nat,
                initial_output,
                slot_views,
                edge_views,
                limit,
            ) == expected,
            expected == lower_sequence_edges_tail_spec(
                parent_node_index,
                index as nat,
                edge_end as nat,
                (edge_end - index) as nat,
                canonical_sequence_entry_views_spec(output@),
                slot_views,
                edge_views,
                limit,
            ),
        decreases edge_end - index,
    {
        let source_child = edges[index].child_node_index();
        proof {
            reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
            assert(edge_views[index as int] == edges[index as int]@);
            reveal(lower_sequence_edges_tail_spec);
        }
        let value_node_index = match canonical_follow_alias(source_child, slots) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if source_child >= slots.len() as u64 || value_node_index >= slots.len() as u64 {
            proof {
                reveal(lower_sequence_edges_tail_spec);
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::InternalInvariantViolation,
                    0,
                ),
            );
        }
        let child_byte = slots[source_child as usize].byte_start();
        proof {
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slot_views[source_child as int] == slots[source_child as int]@);
        }
        if output.len() as u64 >= limit {
            proof {
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::SequenceEntryLimitExceeded,
                        byte_offset: child_byte,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::SequenceEntryLimitExceeded,
                    child_byte,
                ),
            );
        }
        let entry = CanonicalSequenceEntry::new(parent_node_index, index as u64, value_node_index);
        let ghost before = output@;
        output.push(entry);
        proof {
            lemma_canonical_sequence_entry_views_push(before, entry);
            reveal(lower_sequence_edges_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(lower_sequence_edges_tail_spec);
        assert(expected == Ok(canonical_sequence_entry_views_spec(output@)));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Mirrors the exact pure effective-mapping transition.
fn lower_mapping_entries(
    receiver_node_index: u64,
    entry_start: usize,
    entry_end: usize,
    output: &mut Vec<CanonicalMappingEntry>,
    slots: &[SemanticNodeSlot],
    entries: &[ExpandedMappingEntry],
    limit: u64,
) -> (result: Result<(), CanonicalLoweringError>)
    ensures
        lower_mapping_entries_tail_spec(
            receiver_node_index,
            entry_start as nat,
            entry_end as nat,
            if entry_start <= entry_end {
                (entry_end - entry_start) as nat
            } else {
                0nat
            },
            canonical_mapping_entry_views_spec(old(output)@),
            crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            crate::resolve_merge::expanded_mapping_entry_views_spec(entries@),
            limit,
        ) == match result {
            Ok(()) => Ok(canonical_mapping_entry_views_spec(final(output)@)),
            Err(error) => Err(error@),
        },
{
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost entry_views = crate::resolve_merge::expanded_mapping_entry_views_spec(entries@);
    let ghost initial_output = canonical_mapping_entry_views_spec(output@);
    let ghost expected = lower_mapping_entries_tail_spec(
        receiver_node_index,
        entry_start as nat,
        entry_end as nat,
        if entry_start <= entry_end {
            (entry_end - entry_start) as nat
        } else {
            0nat
        },
        initial_output,
        slot_views,
        entry_views,
        limit,
    );
    if entry_start > entry_end || entry_end > entries.len() {
        proof {
            reveal(lower_mapping_entries_tail_spec);
        }
        return Err(
            CanonicalLoweringError::at(CanonicalLoweringErrorKind::InternalInvariantViolation, 0),
        );
    }
    let mut index = entry_start;
    while index < entry_end
        invariant
            entry_start <= index,
            index <= entry_end,
            entry_end <= entries.len(),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            entry_views == crate::resolve_merge::expanded_mapping_entry_views_spec(entries@),
            initial_output == canonical_mapping_entry_views_spec(old(output)@),
            lower_mapping_entries_tail_spec(
                receiver_node_index,
                entry_start as nat,
                entry_end as nat,
                (entry_end - entry_start) as nat,
                initial_output,
                slot_views,
                entry_views,
                limit,
            ) == expected,
            expected == lower_mapping_entries_tail_spec(
                receiver_node_index,
                index as nat,
                entry_end as nat,
                (entry_end - index) as nat,
                canonical_mapping_entry_views_spec(output@),
                slot_views,
                entry_views,
                limit,
            ),
        decreases entry_end - index,
    {
        let source_key = entries[index].key_node_index();
        let source_value = entries[index].value_node_index();
        proof {
            reveal(crate::resolve_merge::expanded_mapping_entry_views_spec);
            assert(entry_views[index as int] == entries[index as int]@);
            reveal(lower_mapping_entries_tail_spec);
        }
        let key_node_index = match canonical_follow_alias(source_key, slots) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        let value_node_index = match canonical_follow_alias(source_value, slots) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if source_key >= slots.len() as u64 || key_node_index >= slots.len() as u64
            || value_node_index >= slots.len() as u64 {
            proof {
                reveal(lower_mapping_entries_tail_spec);
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::InternalInvariantViolation,
                    0,
                ),
            );
        }
        let key_byte = slots[source_key as usize].byte_start();
        proof {
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slot_views[source_key as int] == slots[source_key as int]@);
        }
        if output.len() as u64 >= limit {
            proof {
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::MappingEntryLimitExceeded,
                        byte_offset: key_byte,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::MappingEntryLimitExceeded,
                    key_byte,
                ),
            );
        }
        let entry = CanonicalMappingEntry::new(
            receiver_node_index,
            entries[index].source_mapping_node_index(),
            entries[index].source_edge_index(),
            key_node_index,
            value_node_index,
            entries[index].inherited(),
        );
        let ghost before = output@;
        output.push(entry);
        proof {
            lemma_canonical_mapping_entry_views_push(before, entry);
            reveal(lower_mapping_entries_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(lower_mapping_entries_tail_spec);
        assert(expected == Ok(canonical_mapping_entry_views_spec(output@)));
    }
    Ok(())
}

// The node/root composition functions below retain the same exact operational model.
#[allow(clippy::too_many_arguments)]  // Every authenticated graph and build input remains explicit.
fn append_canonical_node(
    source_node_index: u64,
    resolved_node_index: u64,
    nodes_output: &mut Vec<CanonicalYamlNode>,
    sequence_output: &mut Vec<CanonicalSequenceEntry>,
    mapping_output: &mut Vec<CanonicalMappingEntry>,
    slots: &[SemanticNodeSlot],
    topology_nodes: &[SemanticTopologyNode],
    sequence_edges: &[SemanticSequenceEdge],
    mappings: &[ExpandedMappingRecord],
    mapping_entries: &[ExpandedMappingEntry],
    limits: CanonicalLoweringLimits,
) -> (result: Result<(), CanonicalLoweringError>)
    ensures
        match result {
            Ok(()) => final(nodes_output)@.len() == old(nodes_output)@.len() + 1,
            Err(_) => true,
        },
        append_canonical_node_spec(
            source_node_index,
            resolved_node_index,
            CanonicalLoweringBuildView {
                nodes: canonical_yaml_node_views_spec(old(nodes_output)@),
                sequence_entries: canonical_sequence_entry_views_spec(old(sequence_output)@),
                mapping_entries: canonical_mapping_entry_views_spec(old(mapping_output)@),
                document_roots: Seq::empty(),
            },
            crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            crate::resolve_merge::expanded_mapping_record_views_spec(mappings@),
            crate::resolve_merge::expanded_mapping_entry_views_spec(mapping_entries@),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                CanonicalLoweringBuildView {
                    nodes: canonical_yaml_node_views_spec(final(nodes_output)@),
                    sequence_entries: canonical_sequence_entry_views_spec(final(sequence_output)@),
                    mapping_entries: canonical_mapping_entry_views_spec(final(mapping_output)@),
                    document_roots: Seq::empty(),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost topology_views = crate::resolve_topology::semantic_topology_node_views_spec(
        topology_nodes@,
    );
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost mapping_views = crate::resolve_merge::expanded_mapping_record_views_spec(mappings@);
    let ghost entry_views = crate::resolve_merge::expanded_mapping_entry_views_spec(
        mapping_entries@,
    );
    let ghost initial_build = CanonicalLoweringBuildView {
        nodes: canonical_yaml_node_views_spec(nodes_output@),
        sequence_entries: canonical_sequence_entry_views_spec(sequence_output@),
        mapping_entries: canonical_mapping_entry_views_spec(mapping_output@),
        document_roots: Seq::empty(),
    };
    let ghost expected = append_canonical_node_spec(
        source_node_index,
        resolved_node_index,
        initial_build,
        slot_views,
        topology_views,
        sequence_views,
        mapping_views,
        entry_views,
        limits@,
    );
    if source_node_index >= slots.len() as u64 || source_node_index >= topology_nodes.len() as u64
        || resolved_node_index >= slots.len() as u64 || resolved_node_index
        >= topology_nodes.len() as u64 {
        proof {
            reveal(append_canonical_node_spec);
        }
        return Err(
            CanonicalLoweringError::at(CanonicalLoweringErrorKind::InternalInvariantViolation, 0),
        );
    }
    proof {
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(slot_views[source_node_index as int] == slots[source_node_index as int]@);
        assert(slot_views[resolved_node_index as int] == slots[resolved_node_index as int]@);
        reveal(append_canonical_node_spec);
    }
    let source = &slots[source_node_index as usize];
    let resolved = &slots[resolved_node_index as usize];
    let node_limit = effective_limit(limits.max_nodes(), MAX_PROFILE1_CANONICAL_NODES);
    if nodes_output.len() as u64 >= node_limit {
        return Err(
            CanonicalLoweringError::at(
                CanonicalLoweringErrorKind::NodeLimitExceeded,
                source.byte_start(),
            ),
        );
    }
    if source.kind() == SemanticNodeKind::Alias {
        if resolved_node_index >= nodes_output.len() as u64 {
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::InternalInvariantViolation,
                    source.byte_start(),
                ),
            );
        }
        let target = &nodes_output[resolved_node_index as usize];
        proof {
            reveal(canonical_yaml_node_views_spec);
            assert(initial_build.nodes[resolved_node_index as int]
                == nodes_output[resolved_node_index as int]@);
        }
        let node = CanonicalYamlNode::new(
            source_node_index,
            resolved_node_index,
            target.kind(),
            source.byte_start(),
            source.byte_end(),
            target.scalar_index(),
            target.collection_index(),
            target.edge_start(),
            target.edge_end(),
        );
        let ghost before = nodes_output@;
        nodes_output.push(node);
        proof {
            lemma_canonical_yaml_node_views_push(before, node);
            assert(expected == Ok(
                CanonicalLoweringBuildView {
                    nodes: canonical_yaml_node_views_spec(nodes_output@),
                    sequence_entries: canonical_sequence_entry_views_spec(sequence_output@),
                    mapping_entries: canonical_mapping_entry_views_spec(mapping_output@),
                    document_roots: Seq::empty(),
                },
            ));
        }
        return Ok(());
    }
    match resolved.kind() {
        SemanticNodeKind::Scalar => {
            let scalar_index = match resolved.value_index() {
                Some(index) => index,
                None => {
                    return Err(
                        CanonicalLoweringError::at(
                            CanonicalLoweringErrorKind::InternalInvariantViolation,
                            source.byte_start(),
                        ),
                    );
                },
            };
            let node = CanonicalYamlNode::new(
                source_node_index,
                resolved_node_index,
                CanonicalYamlNodeKind::Scalar,
                source.byte_start(),
                source.byte_end(),
                Some(scalar_index),
                None,
                0,
                0,
            );
            let ghost before = nodes_output@;
            nodes_output.push(node);
            proof {
                lemma_canonical_yaml_node_views_push(before, node);
            }
            Ok(())
        },
        SemanticNodeKind::Sequence => {
            let collection_index = match resolved.value_index() {
                Some(index) => index,
                None => {
                    return Err(
                        CanonicalLoweringError::at(
                            CanonicalLoweringErrorKind::InternalInvariantViolation,
                            source.byte_start(),
                        ),
                    );
                },
            };
            let topology = &topology_nodes[resolved_node_index as usize];
            proof {
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                assert(topology_views[resolved_node_index as int]
                    == topology_nodes[resolved_node_index as int]@);
            }
            let edge_start_u64 = topology.edge_start();
            let edge_end_u64 = topology.edge_end();
            if edge_start_u64 > edge_end_u64 || edge_end_u64 > sequence_edges.len() as u64 {
                return Err(
                    CanonicalLoweringError::at(
                        CanonicalLoweringErrorKind::InternalInvariantViolation,
                        source.byte_start(),
                    ),
                );
            }
            let start = sequence_output.len();
            let sequence_limit = effective_limit(
                limits.max_sequence_entries(),
                MAX_PROFILE1_CANONICAL_SEQUENCE_ENTRIES,
            );
            match lower_sequence_edges(
                resolved_node_index,
                edge_start_u64 as usize,
                edge_end_u64 as usize,
                sequence_output,
                slots,
                sequence_edges,
                sequence_limit,
            ) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            let node = CanonicalYamlNode::new(
                source_node_index,
                resolved_node_index,
                CanonicalYamlNodeKind::Sequence,
                source.byte_start(),
                source.byte_end(),
                None,
                Some(collection_index),
                start as u64,
                sequence_output.len() as u64,
            );
            let ghost before = nodes_output@;
            nodes_output.push(node);
            proof {
                lemma_canonical_yaml_node_views_push(before, node);
            }
            Ok(())
        },
        SemanticNodeKind::Mapping => {
            let collection_index = match resolved.value_index() {
                Some(index) => index,
                None => {
                    return Err(
                        CanonicalLoweringError::at(
                            CanonicalLoweringErrorKind::InternalInvariantViolation,
                            source.byte_start(),
                        ),
                    );
                },
            };
            let record_index = match canonical_mapping_record_index(resolved_node_index, mappings) {
                Some(index) => index,
                None => {
                    return Err(
                        CanonicalLoweringError::at(
                            CanonicalLoweringErrorKind::InternalInvariantViolation,
                            source.byte_start(),
                        ),
                    );
                },
            };
            let record = &mappings[record_index];
            proof {
                reveal(crate::resolve_merge::expanded_mapping_record_views_spec);
                assert(mapping_views[record_index as int] == mappings[record_index as int]@);
            }
            let entry_start_u64 = record.entry_start();
            let entry_end_u64 = record.entry_end();
            if entry_start_u64 > entry_end_u64 || entry_end_u64 > mapping_entries.len() as u64 {
                return Err(
                    CanonicalLoweringError::at(
                        CanonicalLoweringErrorKind::InternalInvariantViolation,
                        source.byte_start(),
                    ),
                );
            }
            let start = mapping_output.len();
            let mapping_limit = effective_limit(
                limits.max_mapping_entries(),
                MAX_PROFILE1_CANONICAL_MAPPING_ENTRIES,
            );
            match lower_mapping_entries(
                resolved_node_index,
                entry_start_u64 as usize,
                entry_end_u64 as usize,
                mapping_output,
                slots,
                mapping_entries,
                mapping_limit,
            ) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            let node = CanonicalYamlNode::new(
                source_node_index,
                resolved_node_index,
                CanonicalYamlNodeKind::Mapping,
                source.byte_start(),
                source.byte_end(),
                None,
                Some(collection_index),
                start as u64,
                mapping_output.len() as u64,
            );
            let ghost before = nodes_output@;
            nodes_output.push(node);
            proof {
                lemma_canonical_yaml_node_views_push(before, node);
            }
            Ok(())
        },
        SemanticNodeKind::Alias => Err(
            CanonicalLoweringError::at(
                CanonicalLoweringErrorKind::InternalInvariantViolation,
                source.byte_start(),
            ),
        ),
    }
}

#[allow(clippy::too_many_arguments)]  // Mirrors the total pure node-lowering state.
fn lower_canonical_nodes(
    nodes_output: &mut Vec<CanonicalYamlNode>,
    sequence_output: &mut Vec<CanonicalSequenceEntry>,
    mapping_output: &mut Vec<CanonicalMappingEntry>,
    slots: &[SemanticNodeSlot],
    topology_nodes: &[SemanticTopologyNode],
    sequence_edges: &[SemanticSequenceEdge],
    mappings: &[ExpandedMappingRecord],
    mapping_entries: &[ExpandedMappingEntry],
    limits: CanonicalLoweringLimits,
) -> (result: Result<(), CanonicalLoweringError>)
    requires
        old(nodes_output)@.len() == 0,
        old(sequence_output)@.len() == 0,
        old(mapping_output)@.len() == 0,
    ensures
        lower_canonical_nodes_tail_spec(
            0,
            slots@.len() as nat,
            CanonicalLoweringBuildView {
                nodes: Seq::empty(),
                sequence_entries: Seq::empty(),
                mapping_entries: Seq::empty(),
                document_roots: Seq::empty(),
            },
            crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            crate::resolve_merge::expanded_mapping_record_views_spec(mappings@),
            crate::resolve_merge::expanded_mapping_entry_views_spec(mapping_entries@),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                CanonicalLoweringBuildView {
                    nodes: canonical_yaml_node_views_spec(final(nodes_output)@),
                    sequence_entries: canonical_sequence_entry_views_spec(final(sequence_output)@),
                    mapping_entries: canonical_mapping_entry_views_spec(final(mapping_output)@),
                    document_roots: Seq::empty(),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost topology_views = crate::resolve_topology::semantic_topology_node_views_spec(
        topology_nodes@,
    );
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost mapping_views = crate::resolve_merge::expanded_mapping_record_views_spec(mappings@);
    let ghost entry_views = crate::resolve_merge::expanded_mapping_entry_views_spec(
        mapping_entries@,
    );
    let ghost top_expected = lower_canonical_nodes_tail_spec(
        0,
        slots@.len() as nat,
        CanonicalLoweringBuildView {
            nodes: Seq::empty(),
            sequence_entries: Seq::empty(),
            mapping_entries: Seq::empty(),
            document_roots: Seq::empty(),
        },
        slot_views,
        topology_views,
        sequence_views,
        mapping_views,
        entry_views,
        limits@,
    );
    proof {
        reveal(canonical_yaml_node_views_spec);
        reveal(canonical_sequence_entry_views_spec);
        reveal(canonical_mapping_entry_views_spec);
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(canonical_yaml_node_views_spec(nodes_output@) == Seq::<
            CanonicalYamlNodeView,
        >::empty());
        assert(canonical_sequence_entry_views_spec(sequence_output@) == Seq::<
            CanonicalSequenceEntryView,
        >::empty());
        assert(canonical_mapping_entry_views_spec(mapping_output@) == Seq::<
            CanonicalMappingEntryView,
        >::empty());
        assert(slot_views.len() == slots@.len());
    }
    let mut index = 0usize;
    while index < slots.len()
        invariant
            index <= slots.len(),
            nodes_output@.len() == index,
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            topology_views == crate::resolve_topology::semantic_topology_node_views_spec(
                topology_nodes@,
            ),
            sequence_views == crate::resolve_topology::semantic_sequence_edge_views_spec(
                sequence_edges@,
            ),
            mapping_views == crate::resolve_merge::expanded_mapping_record_views_spec(mappings@),
            entry_views == crate::resolve_merge::expanded_mapping_entry_views_spec(
                mapping_entries@,
            ),
            lower_canonical_nodes_tail_spec(
                0,
                slots@.len() as nat,
                CanonicalLoweringBuildView {
                    nodes: Seq::empty(),
                    sequence_entries: Seq::empty(),
                    mapping_entries: Seq::empty(),
                    document_roots: Seq::empty(),
                },
                slot_views,
                topology_views,
                sequence_views,
                mapping_views,
                entry_views,
                limits@,
            ) == top_expected,
            top_expected == lower_canonical_nodes_tail_spec(
                index as nat,
                (slots.len() - index) as nat,
                CanonicalLoweringBuildView {
                    nodes: canonical_yaml_node_views_spec(nodes_output@),
                    sequence_entries: canonical_sequence_entry_views_spec(sequence_output@),
                    mapping_entries: canonical_mapping_entry_views_spec(mapping_output@),
                    document_roots: Seq::empty(),
                },
                slot_views,
                topology_views,
                sequence_views,
                mapping_views,
                entry_views,
                limits@,
            ),
        decreases slots.len() - index,
    {
        proof {
            reveal(lower_canonical_nodes_tail_spec);
        }
        let resolved = match canonical_follow_alias(index as u64, slots) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    assert(top_expected == Err(error@));
                }
                return Err(error);
            },
        };
        match append_canonical_node(
            index as u64,
            resolved,
            nodes_output,
            sequence_output,
            mapping_output,
            slots,
            topology_nodes,
            sequence_edges,
            mappings,
            mapping_entries,
            limits,
        ) {
            Ok(()) => {},
            Err(error) => {
                proof {
                    assert(top_expected == Err(error@));
                }
                return Err(error);
            },
        }
        proof {
            reveal(lower_canonical_nodes_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(lower_canonical_nodes_tail_spec);
        assert(top_expected == Ok(
            CanonicalLoweringBuildView {
                nodes: canonical_yaml_node_views_spec(nodes_output@),
                sequence_entries: canonical_sequence_entry_views_spec(sequence_output@),
                mapping_entries: canonical_mapping_entry_views_spec(mapping_output@),
                document_roots: Seq::empty(),
            },
        ));
    }
    Ok(())
}

fn lower_document_roots(
    output: &mut Vec<CanonicalDocumentRoot>,
    roots: &[SemanticDocumentRoot],
    slots: &[SemanticNodeSlot],
    nodes: &[CanonicalYamlNode],
    limits: CanonicalLoweringLimits,
    Ghost(prior): Ghost<CanonicalLoweringBuildView>,
) -> (result: Result<(), CanonicalLoweringError>)
    requires
        old(output)@.len() == 0,
        prior.nodes == canonical_yaml_node_views_spec(nodes@),
        prior.document_roots.len() == 0,
    ensures
        lower_document_roots_tail_spec(
            0,
            roots@.len() as nat,
            prior,
            crate::resolve_topology::semantic_document_root_views_spec(roots@),
            crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                CanonicalLoweringBuildView {
                    nodes: prior.nodes,
                    sequence_entries: prior.sequence_entries,
                    mapping_entries: prior.mapping_entries,
                    document_roots: canonical_document_root_views_spec(final(output)@),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost root_views = crate::resolve_topology::semantic_document_root_views_spec(roots@);
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost expected = lower_document_roots_tail_spec(
        0,
        roots@.len() as nat,
        prior,
        root_views,
        slot_views,
        limits@,
    );
    let root_limit = effective_limit(
        limits.max_document_roots(),
        MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS,
    );
    proof {
        reveal(canonical_document_root_views_spec);
        reveal(canonical_yaml_node_views_spec);
        reveal(crate::resolve_topology::semantic_document_root_views_spec);
        assert(canonical_document_root_views_spec(output@) == Seq::<
            CanonicalDocumentRootView,
        >::empty());
        assert(root_views.len() == roots@.len());
        assert(prior.document_roots == Seq::<CanonicalDocumentRootView>::empty());
        assert(prior.document_roots == canonical_document_root_views_spec(output@));
        assert(prior.nodes.len() == nodes@.len());
    }
    let mut index = 0usize;
    while index < roots.len()
        invariant
            index <= roots.len(),
            output@.len() == index,
            root_views == crate::resolve_topology::semantic_document_root_views_spec(roots@),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(slots@),
            prior.nodes == canonical_yaml_node_views_spec(nodes@),
            prior.nodes.len() == nodes@.len(),
            lower_document_roots_tail_spec(
                0,
                roots@.len() as nat,
                prior,
                root_views,
                slot_views,
                limits@,
            ) == expected,
            root_limit == canonical_lowering_effective_limit_spec(
                limits@.max_document_roots,
                MAX_PROFILE1_CANONICAL_DOCUMENT_ROOTS,
            ),
            expected == lower_document_roots_tail_spec(
                index as nat,
                (roots.len() - index) as nat,
                CanonicalLoweringBuildView {
                    nodes: prior.nodes,
                    sequence_entries: prior.sequence_entries,
                    mapping_entries: prior.mapping_entries,
                    document_roots: canonical_document_root_views_spec(output@),
                },
                root_views,
                slot_views,
                limits@,
            ),
        decreases roots.len() - index,
    {
        let source_node_index = roots[index].node_index();
        let byte_start = roots[index].byte_start();
        proof {
            reveal(crate::resolve_topology::semantic_document_root_views_spec);
            assert(root_views[index as int] == roots[index as int]@);
            reveal(lower_document_roots_tail_spec);
        }
        let value_node_index = match canonical_follow_alias(source_node_index, slots) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if value_node_index >= nodes.len() as u64 {
            proof {
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::InternalInvariantViolation,
                        byte_offset: byte_start,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::InternalInvariantViolation,
                    byte_start,
                ),
            );
        }
        if output.len() as u64 >= root_limit {
            proof {
                assert(expected == Err(
                    CanonicalLoweringErrorView {
                        kind: CanonicalLoweringErrorKind::DocumentRootLimitExceeded,
                        byte_offset: byte_start,
                    },
                ));
            }
            return Err(
                CanonicalLoweringError::at(
                    CanonicalLoweringErrorKind::DocumentRootLimitExceeded,
                    byte_start,
                ),
            );
        }
        let root = CanonicalDocumentRoot::new(
            roots[index].document_index(),
            source_node_index,
            value_node_index,
            byte_start,
        );
        let ghost before = output@;
        output.push(root);
        proof {
            lemma_canonical_document_root_views_push(before, root);
            reveal(lower_document_roots_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(lower_document_roots_tail_spec);
        assert(expected == Ok(
            CanonicalLoweringBuildView {
                nodes: prior.nodes,
                sequence_entries: prior.sequence_entries,
                mapping_entries: prior.mapping_entries,
                document_roots: canonical_document_root_views_spec(output@),
            },
        ));
    }
    Ok(())
}

#[verifier::rlimit(80)]
pub fn lower_profile1_canonical_graph(
    input: ExpandedSemanticGraphSource,
    limits: CanonicalLoweringLimits,
) -> (result: Result<CanonicalYamlGraphSource, CanonicalLoweringError>)
    ensures
        lower_profile1_canonical_graph_spec(input@, limits@) == match result {
            Ok(output) => Ok(output@),
            Err(error) => Err(error@),
        },
{
    let ghost input_view = input@;
    let ghost expected = lower_profile1_canonical_graph_spec(input_view, limits@);
    let structural = input.input().structural_keys();
    let table = structural.scalar_keys().graph().node_table();
    let topology = table.topology();
    let slots = table.nodes();
    let topology_nodes = topology.nodes();
    let sequence_edges = topology.sequence_edges();
    let mappings = input.mappings();
    let mapping_entries = input.entries();
    let roots = topology.document_roots();
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(slots@);
    let ghost topology_views = crate::resolve_topology::semantic_topology_node_views_spec(
        topology_nodes@,
    );
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost mapping_views = crate::resolve_merge::expanded_mapping_record_views_spec(mappings@);
    let ghost entry_views = crate::resolve_merge::expanded_mapping_entry_views_spec(
        mapping_entries@,
    );
    let ghost root_views = crate::resolve_topology::semantic_document_root_views_spec(roots@);
    let ghost initial = CanonicalLoweringBuildView {
        nodes: Seq::empty(),
        sequence_entries: Seq::empty(),
        mapping_entries: Seq::empty(),
        document_roots: Seq::empty(),
    };
    let ghost nodes_expected = lower_canonical_nodes_tail_spec(
        0,
        slots@.len() as nat,
        initial,
        slot_views,
        topology_views,
        sequence_views,
        mapping_views,
        entry_views,
        limits@,
    );
    proof {
        reveal(lower_profile1_canonical_graph_spec);
        assert(input_view.input.structural_keys.scalar_keys.graph.node_table == table@);
        assert(table@.topology == topology@);
        assert(table@.nodes == slot_views);
        assert(topology@.nodes == topology_views);
        assert(topology@.sequence_edges == sequence_views);
        assert(input_view.mappings == mapping_views);
        assert(input_view.entries == entry_views);
        assert(topology@.document_roots == root_views);
        assert(expected == match nodes_expected {
            Err(error) => Err(error),
            Ok(build) => match lower_document_roots_tail_spec(
                0,
                root_views.len() as nat,
                build,
                root_views,
                slot_views,
                limits@,
            ) {
                Err(error) => Err(error),
                Ok(final_build) => Ok(
                    CanonicalYamlGraphSourceView {
                        profile_version: input_view.profile_version,
                        transformation_version: CANONICAL_LOWERING_TRANSFORMATION_VERSION,
                        source_len_bytes: input_view.source_len_bytes,
                        input_node_count: input_view.input_node_count,
                        expanded_reference_count: input_view.expanded_reference_count,
                        input: input_view,
                        nodes: final_build.nodes,
                        sequence_entries: final_build.sequence_entries,
                        mapping_entries: final_build.mapping_entries,
                        document_roots: final_build.document_roots,
                    },
                ),
            },
        });
    }

    let mut nodes = Vec::new();
    let mut sequence_entries = Vec::new();
    let mut canonical_mapping_entries = Vec::new();
    match lower_canonical_nodes(
        &mut nodes,
        &mut sequence_entries,
        &mut canonical_mapping_entries,
        slots,
        topology_nodes,
        sequence_edges,
        mappings,
        mapping_entries,
        limits,
    ) {
        Ok(()) => {},
        Err(error) => {
            proof {
                assert(nodes_expected == Err(error@));
            }
            return Err(error);
        },
    }
    let ghost nodes_build = CanonicalLoweringBuildView {
        nodes: canonical_yaml_node_views_spec(nodes@),
        sequence_entries: canonical_sequence_entry_views_spec(sequence_entries@),
        mapping_entries: canonical_mapping_entry_views_spec(canonical_mapping_entries@),
        document_roots: Seq::empty(),
    };
    proof {
        assert(nodes_expected == Ok(nodes_build));
    }
    let ghost roots_expected = lower_document_roots_tail_spec(
        0,
        roots@.len() as nat,
        nodes_build,
        root_views,
        slot_views,
        limits@,
    );
    let mut document_roots = Vec::new();
    match lower_document_roots(
        &mut document_roots,
        roots,
        slots,
        nodes.as_slice(),
        limits,
        Ghost(nodes_build),
    ) {
        Ok(()) => {},
        Err(error) => {
            proof {
                assert(roots_expected == Err(error@));
            }
            return Err(error);
        },
    }
    let ghost final_build = CanonicalLoweringBuildView {
        nodes: canonical_yaml_node_views_spec(nodes@),
        sequence_entries: canonical_sequence_entry_views_spec(sequence_entries@),
        mapping_entries: canonical_mapping_entry_views_spec(canonical_mapping_entries@),
        document_roots: canonical_document_root_views_spec(document_roots@),
    };
    proof {
        assert(roots_expected == Ok(final_build));
    }
    let output = CanonicalYamlGraphSource::new(
        input,
        nodes,
        sequence_entries,
        canonical_mapping_entries,
        document_roots,
    );
    proof {
        reveal(lower_profile1_canonical_graph_spec);
        assert(expected == Ok(output@));
    }
    Ok(output)
}

pub open spec fn canonical_yaml_graph_source_well_formed_spec(
    input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: CanonicalLoweringLimitsView,
    output: CanonicalYamlGraphSourceView,
) -> bool {
    lower_profile1_canonical_graph_spec(input, limits) == Ok(output)
}

pub open spec fn canonical_yaml_graph_source_preserves_input_identity_spec(
    input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    output: CanonicalYamlGraphSourceView,
) -> bool {
    output.profile_version == input.profile_version && output.transformation_version
        == CANONICAL_LOWERING_TRANSFORMATION_VERSION && output.source_len_bytes
        == input.source_len_bytes && output.input_node_count == input.input_node_count
        && output.expanded_reference_count == input.expanded_reference_count && output.input
        == input
}

pub proof fn lemma_canonical_lowering_success_is_well_formed(
    input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: CanonicalLoweringLimitsView,
    output: CanonicalYamlGraphSourceView,
)
    requires
        lower_profile1_canonical_graph_spec(input, limits) == Ok(output),
    ensures
        canonical_yaml_graph_source_well_formed_spec(input, limits, output),
{
    reveal(canonical_yaml_graph_source_well_formed_spec);
}

pub proof fn lemma_authenticated_canonical_graph_preserves_input_identity(
    input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: CanonicalLoweringLimitsView,
    output: CanonicalYamlGraphSourceView,
)
    requires
        canonical_yaml_graph_source_well_formed_spec(input, limits, output),
    ensures
        canonical_yaml_graph_source_preserves_input_identity_spec(input, output),
{
    reveal(canonical_yaml_graph_source_well_formed_spec);
    reveal(lower_profile1_canonical_graph_spec);
    let table = input.input.structural_keys.scalar_keys.graph.node_table;
    let topology = table.topology;
    let initial = CanonicalLoweringBuildView {
        nodes: Seq::empty(),
        sequence_entries: Seq::empty(),
        mapping_entries: Seq::empty(),
        document_roots: Seq::empty(),
    };
    let nodes_result = lower_canonical_nodes_tail_spec(
        0,
        table.nodes.len() as nat,
        initial,
        table.nodes,
        topology.nodes,
        topology.sequence_edges,
        input.mappings,
        input.entries,
        limits,
    );
    match nodes_result {
        Err(_) => {
            assert(false);
        },
        Ok(build) => {
            let roots_result = lower_document_roots_tail_spec(
                0,
                topology.document_roots.len() as nat,
                build,
                topology.document_roots,
                table.nodes,
                limits,
            );
            match roots_result {
                Err(_) => {
                    assert(false);
                },
                Ok(_) => {
                    reveal(canonical_yaml_graph_source_preserves_input_identity_spec);
                },
            }
        },
    }
}

pub proof fn lemma_authenticated_canonical_lowering_is_unique(
    input: crate::resolve_merge::ExpandedSemanticGraphSourceView,
    limits: CanonicalLoweringLimitsView,
    first: CanonicalYamlGraphSourceView,
    second: CanonicalYamlGraphSourceView,
)
    requires
        canonical_yaml_graph_source_well_formed_spec(input, limits, first),
        canonical_yaml_graph_source_well_formed_spec(input, limits, second),
    ensures
        first == second,
{
    reveal(canonical_yaml_graph_source_well_formed_spec);
}

} // verus!
