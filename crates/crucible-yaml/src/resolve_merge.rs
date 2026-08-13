//! Verified, graph-preserving YAML merge-key expansion.
//!
//! The transform never materializes alias subtrees. It records one interval for every mapping and
//! copies only mapping-edge references. Explicit receiver entries precede inherited entries and
//! override every merge source; earlier mappings in a merge sequence override later mappings.
use crate::cst::{CstNodeKind, CstNodeStyle};
use crate::resolve_canonical_structural_key::{compare_byte_slices, CanonicalStructuralKeyRecord};
use crate::resolve_duplicate_key::{
    DuplicateFreeStructuralKeySource, DuplicateFreeStructuralKeySourceView,
};
use crate::resolve_node_table::{SemanticNodeKind, SemanticNodeSlot};
use crate::resolve_scalar_value::ResolvedScalar;
use crate::resolve_topology::SemanticTopologyNode;
use vstd::prelude::*;

verus! {

pub const MERGE_EXPANSION_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_MERGE_MAPPING_RECORDS: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES: u64 = crate::cst::MAX_PROFILE1_CST_MAPPING_ENTRIES;

pub const MAX_PROFILE1_EXPANDED_REFERENCES: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_MERGE_SOURCES: u64 = crate::cst::MAX_PROFILE1_CST_MAPPING_ENTRIES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MergeExpansionLimits {
    max_mappings: u64,
    max_expanded_mapping_entries: u64,
    max_expanded_references: u64,
    max_merge_sources: u64,
}

#[verifier::ext_equal]
pub struct MergeExpansionLimitsView {
    pub max_mappings: u64,
    pub max_expanded_mapping_entries: u64,
    pub max_expanded_references: u64,
    pub max_merge_sources: u64,
}

impl View for MergeExpansionLimits {
    type V = MergeExpansionLimitsView;

    closed spec fn view(&self) -> MergeExpansionLimitsView {
        MergeExpansionLimitsView {
            max_mappings: self.max_mappings,
            max_expanded_mapping_entries: self.max_expanded_mapping_entries,
            max_expanded_references: self.max_expanded_references,
            max_merge_sources: self.max_merge_sources,
        }
    }
}

impl MergeExpansionLimits {
    pub fn new(
        max_mappings: u64,
        max_expanded_mapping_entries: u64,
        max_expanded_references: u64,
        max_merge_sources: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (MergeExpansionLimitsView {
                max_mappings,
                max_expanded_mapping_entries,
                max_expanded_references,
                max_merge_sources,
            }),
    {
        Self {
            max_mappings,
            max_expanded_mapping_entries,
            max_expanded_references,
            max_merge_sources,
        }
    }

    pub fn max_mappings(&self) -> (value: u64)
        ensures
            value == self@.max_mappings,
    {
        self.max_mappings
    }

    pub fn max_expanded_mapping_entries(&self) -> (value: u64)
        ensures
            value == self@.max_expanded_mapping_entries,
    {
        self.max_expanded_mapping_entries
    }

    pub fn max_expanded_references(&self) -> (value: u64)
        ensures
            value == self@.max_expanded_references,
    {
        self.max_expanded_references
    }

    pub fn max_merge_sources(&self) -> (value: u64)
        ensures
            value == self@.max_merge_sources,
    {
        self.max_merge_sources
    }
}

pub fn canonical_merge_expansion_limits() -> (limits: MergeExpansionLimits)
    ensures
        limits@ == canonical_merge_expansion_limits_spec(),
{
    MergeExpansionLimits::new(
        MAX_PROFILE1_MERGE_MAPPING_RECORDS,
        MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
        MAX_PROFILE1_EXPANDED_REFERENCES,
        MAX_PROFILE1_MERGE_SOURCES,
    )
}

pub open spec fn canonical_merge_expansion_limits_spec() -> MergeExpansionLimitsView {
    MergeExpansionLimitsView {
        max_mappings: MAX_PROFILE1_MERGE_MAPPING_RECORDS,
        max_expanded_mapping_entries: MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
        max_expanded_references: MAX_PROFILE1_EXPANDED_REFERENCES,
        max_merge_sources: MAX_PROFILE1_MERGE_SOURCES,
    }
}

pub open spec fn merge_expansion_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (value: u64)
    ensures
        value == merge_expansion_effective_limit_spec(requested, absolute),
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
pub enum MergeExpansionErrorKind {
    InvalidMergeValue,
    MappingLimitExceeded,
    ExpandedMappingEntryLimitExceeded,
    ExpandedReferenceLimitExceeded,
    MergeSourceLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MergeExpansionError {
    kind: MergeExpansionErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct MergeExpansionErrorView {
    pub kind: MergeExpansionErrorKind,
    pub byte_offset: u64,
}

impl View for MergeExpansionError {
    type V = MergeExpansionErrorView;

    closed spec fn view(&self) -> MergeExpansionErrorView {
        MergeExpansionErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl MergeExpansionError {
    fn at(kind: MergeExpansionErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (MergeExpansionErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: MergeExpansionErrorKind)
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
pub struct ExpandedMappingRecord {
    node_index: u64,
    entry_start: u64,
    entry_end: u64,
}

#[verifier::ext_equal]
pub struct ExpandedMappingRecordView {
    pub node_index: u64,
    pub entry_start: u64,
    pub entry_end: u64,
}

impl View for ExpandedMappingRecord {
    type V = ExpandedMappingRecordView;

    closed spec fn view(&self) -> ExpandedMappingRecordView {
        ExpandedMappingRecordView {
            node_index: self.node_index,
            entry_start: self.entry_start,
            entry_end: self.entry_end,
        }
    }
}

impl ExpandedMappingRecord {
    fn new(node_index: u64, entry_start: u64, entry_end: u64) -> (record: Self)
        ensures
            record@ == (ExpandedMappingRecordView { node_index, entry_start, entry_end }),
    {
        Self { node_index, entry_start, entry_end }
    }

    pub fn node_index(&self) -> (value: u64)
        ensures
            value == self@.node_index,
    {
        self.node_index
    }

    pub fn entry_start(&self) -> (value: u64)
        ensures
            value == self@.entry_start,
    {
        self.entry_start
    }

    pub fn entry_end(&self) -> (value: u64)
        ensures
            value == self@.entry_end,
    {
        self.entry_end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpandedMappingEntry {
    key_node_index: u64,
    value_node_index: u64,
    source_mapping_node_index: u64,
    source_edge_index: u64,
    inherited: bool,
}

#[verifier::ext_equal]
pub struct ExpandedMappingEntryView {
    pub key_node_index: u64,
    pub value_node_index: u64,
    pub source_mapping_node_index: u64,
    pub source_edge_index: u64,
    pub inherited: bool,
}

impl View for ExpandedMappingEntry {
    type V = ExpandedMappingEntryView;

    closed spec fn view(&self) -> ExpandedMappingEntryView {
        ExpandedMappingEntryView {
            key_node_index: self.key_node_index,
            value_node_index: self.value_node_index,
            source_mapping_node_index: self.source_mapping_node_index,
            source_edge_index: self.source_edge_index,
            inherited: self.inherited,
        }
    }
}

impl ExpandedMappingEntry {
    fn explicit(
        key_node_index: u64,
        value_node_index: u64,
        source_mapping_node_index: u64,
        source_edge_index: u64,
    ) -> (entry: Self)
        ensures
            entry@ == (ExpandedMappingEntryView {
                key_node_index,
                value_node_index,
                source_mapping_node_index,
                source_edge_index,
                inherited: false,
            }),
    {
        Self {
            key_node_index,
            value_node_index,
            source_mapping_node_index,
            source_edge_index,
            inherited: false,
        }
    }

    fn inherited_from(source: &ExpandedMappingEntry) -> (entry: Self)
        ensures
            entry@ == (ExpandedMappingEntryView {
                key_node_index: source@.key_node_index,
                value_node_index: source@.value_node_index,
                source_mapping_node_index: source@.source_mapping_node_index,
                source_edge_index: source@.source_edge_index,
                inherited: true,
            }),
    {
        Self {
            key_node_index: source.key_node_index,
            value_node_index: source.value_node_index,
            source_mapping_node_index: source.source_mapping_node_index,
            source_edge_index: source.source_edge_index,
            inherited: true,
        }
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

    pub fn inherited(&self) -> (value: bool)
        ensures
            value == self@.inherited,
    {
        self.inherited
    }
}

pub open spec fn expanded_mapping_record_views_spec(values: Seq<ExpandedMappingRecord>) -> Seq<
    ExpandedMappingRecordView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn expanded_mapping_entry_views_spec(values: Seq<ExpandedMappingEntry>) -> Seq<
    ExpandedMappingEntryView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExpandedSemanticGraphSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    expanded_reference_count: u64,
    merge_source_count: u64,
    input: DuplicateFreeStructuralKeySource,
    mappings: Vec<ExpandedMappingRecord>,
    entries: Vec<ExpandedMappingEntry>,
}

#[verifier::ext_equal]
pub struct ExpandedSemanticGraphSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub expanded_reference_count: u64,
    pub merge_source_count: u64,
    pub input: DuplicateFreeStructuralKeySourceView,
    pub mappings: Seq<ExpandedMappingRecordView>,
    pub entries: Seq<ExpandedMappingEntryView>,
}

impl View for ExpandedSemanticGraphSource {
    type V = ExpandedSemanticGraphSourceView;

    closed spec fn view(&self) -> ExpandedSemanticGraphSourceView {
        ExpandedSemanticGraphSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            expanded_reference_count: self.expanded_reference_count,
            merge_source_count: self.merge_source_count,
            input: self.input@,
            mappings: expanded_mapping_record_views_spec(self.mappings@),
            entries: expanded_mapping_entry_views_spec(self.entries@),
        }
    }
}

impl ExpandedSemanticGraphSource {
    fn new(
        input: DuplicateFreeStructuralKeySource,
        expanded_reference_count: u64,
        merge_source_count: u64,
        mappings: Vec<ExpandedMappingRecord>,
        entries: Vec<ExpandedMappingEntry>,
    ) -> (source: Self)
        ensures
            source@ == (ExpandedSemanticGraphSourceView {
                profile_version: input@.profile_version,
                transformation_version: MERGE_EXPANSION_TRANSFORMATION_VERSION,
                source_len_bytes: input@.source_len_bytes,
                input_node_count: input@.input_node_count,
                expanded_reference_count,
                merge_source_count,
                input: input@,
                mappings: expanded_mapping_record_views_spec(mappings@),
                entries: expanded_mapping_entry_views_spec(entries@),
            }),
    {
        let profile_version = input.profile_version();
        let source_len_bytes = input.source_len_bytes();
        let input_node_count = input.input_node_count();
        Self {
            profile_version,
            transformation_version: MERGE_EXPANSION_TRANSFORMATION_VERSION,
            source_len_bytes,
            input_node_count,
            expanded_reference_count,
            merge_source_count,
            input,
            mappings,
            entries,
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

    pub fn merge_source_count(&self) -> (value: u64)
        ensures
            value == self@.merge_source_count,
    {
        self.merge_source_count
    }

    pub fn input(&self) -> (value: &DuplicateFreeStructuralKeySource)
        ensures
            value@ == self@.input,
    {
        &self.input
    }

    pub fn mappings(&self) -> (values: &[ExpandedMappingRecord])
        ensures
            expanded_mapping_record_views_spec(values@) == self@.mappings,
    {
        self.mappings.as_slice()
    }

    pub fn entries(&self) -> (values: &[ExpandedMappingEntry])
        ensures
            expanded_mapping_entry_views_spec(values@) == self@.entries,
    {
        self.entries.as_slice()
    }
}

#[verifier::ext_equal]
pub struct MergeExpansionBuildView {
    pub mappings: Seq<ExpandedMappingRecordView>,
    pub entries: Seq<ExpandedMappingEntryView>,
    pub merge_source_count: u64,
}

pub open spec fn merge_ascii_equal_tail_spec(
    content: Seq<u32>,
    expected: Seq<u8>,
    index: nat,
) -> bool
    decreases content.len() - index,
{
    if content.len() != expected.len() || index > content.len() {
        false
    } else if index == content.len() {
        true
    } else {
        content[index as int] == expected[index as int] as u32 && merge_ascii_equal_tail_spec(
            content,
            expected,
            (index + 1) as nat,
        )
    }
}

pub open spec fn decoded_merge_ascii_spec(
    content: Seq<crate::scalar_decode::DecodedContentScalarView>,
    _expected: Seq<u8>,
) -> bool {
    content.len() == 2 && content[0].code_point == b'<' as u32 && content[1].code_point
        == b'<' as u32
}

pub closed spec fn decoded_merge_ascii_tail_spec(
    content: Seq<crate::scalar_decode::DecodedContentScalarView>,
    expected: Seq<u8>,
    index: nat,
) -> bool
    decreases content.len() - index,
{
    if content.len() != expected.len() || index > content.len() {
        false
    } else if index == content.len() {
        true
    } else {
        content[index as int].code_point == expected[index as int] as u32
            && decoded_merge_ascii_tail_spec(content, expected, (index + 1) as nat)
    }
}

pub open spec fn tag_merge_ascii_spec(
    content: Seq<crate::resolve_tag::ResolvedTagCodePointView>,
    _expected: Seq<u8>,
) -> bool {
    content.len() == 23 && content[0].code_point == b't' as u32 && content[1].code_point
        == b'a' as u32 && content[2].code_point == b'g' as u32 && content[3].code_point
        == b':' as u32 && content[4].code_point == b'y' as u32 && content[5].code_point
        == b'a' as u32 && content[6].code_point == b'm' as u32 && content[7].code_point
        == b'l' as u32 && content[8].code_point == b'.' as u32 && content[9].code_point
        == b'o' as u32 && content[10].code_point == b'r' as u32 && content[11].code_point
        == b'g' as u32 && content[12].code_point == b',' as u32 && content[13].code_point
        == b'2' as u32 && content[14].code_point == b'0' as u32 && content[15].code_point
        == b'0' as u32 && content[16].code_point == b'2' as u32 && content[17].code_point
        == b':' as u32 && content[18].code_point == b'm' as u32 && content[19].code_point
        == b'e' as u32 && content[20].code_point == b'r' as u32 && content[21].code_point
        == b'g' as u32 && content[22].code_point == b'e' as u32
}

pub closed spec fn tag_merge_ascii_tail_spec(
    content: Seq<crate::resolve_tag::ResolvedTagCodePointView>,
    expected: Seq<u8>,
    index: nat,
) -> bool
    decreases content.len() - index,
{
    if content.len() != expected.len() || index > content.len() {
        false
    } else if index == content.len() {
        true
    } else {
        content[index as int].code_point == expected[index as int] as u32
            && tag_merge_ascii_tail_spec(content, expected, (index + 1) as nat)
    }
}

pub open spec fn resolved_scalar_is_merge_key_spec(
    scalar: crate::resolve_scalar_value::ResolvedScalarView,
) -> bool {
    match scalar.explicit_tag {
        Some(tag) => tag_merge_ascii_spec(
            tag.content,
            seq![
                b't',
                b'a',
                b'g',
                b':',
                b'y',
                b'a',
                b'm',
                b'l',
                b'.',
                b'o',
                b'r',
                b'g',
                b',',
                b'2',
                b'0',
                b'0',
                b'2',
                b':',
                b'm',
                b'e',
                b'r',
                b'g',
                b'e',
            ],
        ),
        None => scalar.presentation.style == CstNodeStyle::Plain
            && match scalar.presentation.decoded {
            Some(decoded) => decoded_merge_ascii_spec(decoded.content, seq![b'<', b'<']),
            None => false,
        },
    }
}

pub open spec fn node_is_merge_key_spec(
    key_node_index: u64,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
) -> Result<bool, MergeExpansionErrorView> {
    if key_node_index >= node_slots.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let slot = node_slots[key_node_index as int];
        if slot.kind != SemanticNodeKind::Scalar {
            Ok(false)
        } else {
            match slot.value_index {
                Some(value_index) => {
                    if value_index >= scalars.len() {
                        Err(
                            MergeExpansionErrorView {
                                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                byte_offset: slot.byte_start,
                            },
                        )
                    } else {
                        Ok(resolved_scalar_is_merge_key_spec(scalars[value_index as int]))
                    }
                },
                None => Ok(false),
            }
        }
    }
}

pub closed spec fn follow_alias_tail_spec(
    current: u64,
    fuel: nat,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
) -> Result<u64, MergeExpansionErrorView>
    decreases fuel,
{
    if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: if current < node_slots.len() {
                    node_slots[current as int].byte_start
                } else {
                    0
                },
            },
        )
    } else if current >= node_slots.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let slot = node_slots[current as int];
        if slot.kind != SemanticNodeKind::Alias {
            Ok(current)
        } else {
            match slot.alias_target_node_index {
                Some(target) => follow_alias_tail_spec(target, (fuel - 1) as nat, node_slots),
                None => Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: slot.byte_start,
                    },
                ),
            }
        }
    }
}

pub open spec fn follow_alias_spec(
    node_index: u64,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
) -> Result<u64, MergeExpansionErrorView> {
    follow_alias_tail_spec(node_index, node_slots.len() as nat, node_slots)
}

pub closed spec fn merge_sequence_sources_tail_spec(
    edge_index: nat,
    edge_end: nat,
    fuel: nat,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
) -> Result<Seq<u64>, MergeExpansionErrorView>
    decreases fuel,
{
    if edge_index > edge_end || edge_end > sequence_edges.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if edge_index == edge_end {
        Ok(Seq::empty())
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        match follow_alias_spec(sequence_edges[edge_index as int].child_node_index, node_slots) {
            Err(error) => Err(error),
            Ok(target) => {
                if target >= node_slots.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: 0,
                        },
                    )
                } else if node_slots[target as int].kind != SemanticNodeKind::Mapping {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InvalidMergeValue,
                            byte_offset: node_slots[target as int].byte_start,
                        },
                    )
                } else {
                    match merge_sequence_sources_tail_spec(
                        (edge_index + 1) as nat,
                        edge_end,
                        (fuel - 1) as nat,
                        sequence_edges,
                        node_slots,
                    ) {
                        Err(error) => Err(error),
                        Ok(tail) => Ok(Seq::empty().push(target) + tail),
                    }
                }
            },
        }
    }
}

pub open spec fn merge_sources_for_value_spec(
    value_node_index: u64,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    topology_nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
) -> Result<Seq<u64>, MergeExpansionErrorView> {
    match follow_alias_spec(value_node_index, node_slots) {
        Err(error) => Err(error),
        Ok(target) => {
            if target >= node_slots.len() || target >= topology_nodes.len() {
                Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                )
            } else {
                match node_slots[target as int].kind {
                    SemanticNodeKind::Mapping => Ok(Seq::empty().push(target)),
                    SemanticNodeKind::Sequence => {
                        let node = topology_nodes[target as int];
                        if node.kind != CstNodeKind::Sequence || node.edge_start > node.edge_end
                            || node.edge_end > sequence_edges.len() {
                            Err(
                                MergeExpansionErrorView {
                                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                    byte_offset: node.byte_start,
                                },
                            )
                        } else {
                            merge_sequence_sources_tail_spec(
                                node.edge_start as nat,
                                node.edge_end as nat,
                                (node.edge_end - node.edge_start) as nat,
                                sequence_edges,
                                node_slots,
                            )
                        }
                    },
                    _ => Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InvalidMergeValue,
                            byte_offset: node_slots[target as int].byte_start,
                        },
                    ),
                }
            }
        },
    }
}

pub closed spec fn validate_merge_sequence_tail_spec(
    edge_index: nat,
    edge_end: nat,
    fuel: nat,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
) -> Result<(), MergeExpansionErrorView>
    decreases fuel,
{
    if edge_index > edge_end || edge_end > sequence_edges.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if edge_index == edge_end {
        Ok(())
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        match follow_alias_spec(sequence_edges[edge_index as int].child_node_index, node_slots) {
            Err(error) => Err(error),
            Ok(target) => {
                if target >= node_slots.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: 0,
                        },
                    )
                } else if node_slots[target as int].kind != SemanticNodeKind::Mapping {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InvalidMergeValue,
                            byte_offset: node_slots[target as int].byte_start,
                        },
                    )
                } else {
                    validate_merge_sequence_tail_spec(
                        (edge_index + 1) as nat,
                        edge_end,
                        (fuel - 1) as nat,
                        sequence_edges,
                        node_slots,
                    )
                }
            },
        }
    }
}

pub open spec fn validate_merge_value_spec(
    value_node_index: u64,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    topology_nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
) -> Result<(), MergeExpansionErrorView> {
    match follow_alias_spec(value_node_index, node_slots) {
        Err(error) => Err(error),
        Ok(target) => {
            if target >= node_slots.len() || target >= topology_nodes.len() {
                Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                )
            } else {
                match node_slots[target as int].kind {
                    SemanticNodeKind::Mapping => Ok(()),
                    SemanticNodeKind::Sequence => {
                        let node = topology_nodes[target as int];
                        if node.kind != CstNodeKind::Sequence || node.edge_start > node.edge_end
                            || node.edge_end > sequence_edges.len() {
                            Err(
                                MergeExpansionErrorView {
                                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                    byte_offset: node.byte_start,
                                },
                            )
                        } else {
                            validate_merge_sequence_tail_spec(
                                node.edge_start as nat,
                                node.edge_end as nat,
                                (node.edge_end - node.edge_start) as nat,
                                sequence_edges,
                                node_slots,
                            )
                        }
                    },
                    _ => Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InvalidMergeValue,
                            byte_offset: node_slots[target as int].byte_start,
                        },
                    ),
                }
            }
        },
    }
}

pub closed spec fn preflight_mapping_edges_tail_spec(
    edge_index: nat,
    edge_end: nat,
    fuel: nat,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
    topology_nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mapping_edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
) -> Result<(), MergeExpansionErrorView>
    decreases fuel,
{
    if edge_index > edge_end || edge_end > mapping_edges.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if edge_index == edge_end {
        Ok(())
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let edge = mapping_edges[edge_index as int];
        match node_is_merge_key_spec(edge.key_node_index, node_slots, scalars) {
            Err(error) => Err(error),
            Ok(is_merge) => {
                let validation = if is_merge {
                    validate_merge_value_spec(
                        edge.value_node_index,
                        node_slots,
                        topology_nodes,
                        sequence_edges,
                    )
                } else {
                    Ok(())
                };
                match validation {
                    Err(error) => Err(error),
                    Ok(()) => preflight_mapping_edges_tail_spec(
                        (edge_index + 1) as nat,
                        edge_end,
                        (fuel - 1) as nat,
                        node_slots,
                        scalars,
                        topology_nodes,
                        sequence_edges,
                        mapping_edges,
                    ),
                }
            },
        }
    }
}

pub closed spec fn preflight_merge_nodes_tail_spec(
    node_index: nat,
    fuel: nat,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
    topology_nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mapping_edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
) -> Result<(), MergeExpansionErrorView>
    decreases fuel,
{
    if node_index >= topology_nodes.len() {
        Ok(())
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let node = topology_nodes[node_index as int];
        let validation = if node.kind == CstNodeKind::Mapping {
            if node.edge_start > node.edge_end || node.edge_end > mapping_edges.len() {
                Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: node.byte_start,
                    },
                )
            } else {
                preflight_mapping_edges_tail_spec(
                    node.edge_start as nat,
                    node.edge_end as nat,
                    (node.edge_end - node.edge_start) as nat,
                    node_slots,
                    scalars,
                    topology_nodes,
                    sequence_edges,
                    mapping_edges,
                )
            }
        } else {
            Ok(())
        };
        match validation {
            Err(error) => Err(error),
            Ok(()) => preflight_merge_nodes_tail_spec(
                (node_index + 1) as nat,
                (fuel - 1) as nat,
                node_slots,
                scalars,
                topology_nodes,
                sequence_edges,
                mapping_edges,
            ),
        }
    }
}

pub open spec fn merge_expansion_input_shape_spec(
    input: DuplicateFreeStructuralKeySourceView,
) -> bool {
    let structural = input.structural_keys;
    let table = structural.scalar_keys.graph.node_table;
    let topology = table.topology;
    topology.nodes.len() == table.nodes.len() && structural.records.len() == topology.nodes.len()
}

pub open spec fn preflight_merge_shapes_spec(input: DuplicateFreeStructuralKeySourceView) -> Result<
    (),
    MergeExpansionErrorView,
> {
    let structural = input.structural_keys;
    let table = structural.scalar_keys.graph.node_table;
    let topology = table.topology;
    if !merge_expansion_input_shape_spec(input) {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: input.source_len_bytes,
            },
        )
    } else {
        preflight_merge_nodes_tail_spec(
            0,
            topology.nodes.len() as nat,
            table.nodes,
            table.scalars.scalars,
            topology.nodes,
            topology.sequence_edges,
            topology.mapping_edges,
        )
    }
}

pub open spec fn canonical_merge_key_equal_spec(
    left_node_index: u64,
    right_node_index: u64,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
) -> Result<bool, MergeExpansionErrorView> {
    if left_node_index >= records.len() || right_node_index >= records.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let left = records[left_node_index as int].bytes;
        let right = records[right_node_index as int].bytes;
        Ok(
            crate::resolve_canonical_structural_key::compare_byte_views_tail_spec(
                left,
                right,
                0,
                if left.len() < right.len() {
                    left.len() as nat
                } else {
                    right.len() as nat
                },
            ) == 0,
        )
    }
}

pub closed spec fn expanded_entry_key_present_tail_spec(
    key_node_index: u64,
    index: nat,
    end: nat,
    fuel: nat,
    entries: Seq<ExpandedMappingEntryView>,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
) -> Result<bool, MergeExpansionErrorView>
    decreases fuel,
{
    if key_node_index >= records.len() || index > end || end > entries.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if index == end {
        Ok(false)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        match canonical_merge_key_equal_spec(
            key_node_index,
            entries[index as int].key_node_index,
            records,
        ) {
            Err(error) => Err(error),
            Ok(true) => Ok(true),
            Ok(false) => expanded_entry_key_present_tail_spec(
                key_node_index,
                (index + 1) as nat,
                end,
                (fuel - 1) as nat,
                entries,
                records,
            ),
        }
    }
}

pub open spec fn admit_expanded_entry_spec(
    entry: ExpandedMappingEntryView,
    key_byte: u64,
    build: MergeExpansionBuildView,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView> {
    let entry_limit = merge_expansion_effective_limit_spec(
        limits.max_expanded_mapping_entries,
        MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
    );
    match admit_expanded_entry_limit_spec(entry, key_byte, build.entries, entry_limit) {
        Err(error) => Err(error),
        Ok(entries) => Ok(
            MergeExpansionBuildView {
                mappings: build.mappings,
                entries,
                merge_source_count: build.merge_source_count,
            },
        ),
    }
}

pub open spec fn admit_expanded_entry_limit_spec(
    entry: ExpandedMappingEntryView,
    key_byte: u64,
    entries: Seq<ExpandedMappingEntryView>,
    entry_limit: u64,
) -> Result<Seq<ExpandedMappingEntryView>, MergeExpansionErrorView> {
    if entries.len() >= entry_limit {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::ExpandedMappingEntryLimitExceeded,
                byte_offset: key_byte,
            },
        )
    } else {
        Ok(entries.push(entry))
    }
}

pub closed spec fn mapping_record_index_tail_spec(
    node_index: u64,
    index: nat,
    fuel: nat,
    mappings: Seq<ExpandedMappingRecordView>,
) -> Option<nat>
    decreases fuel,
{
    if index >= mappings.len() || fuel == 0 {
        None
    } else if mappings[index as int].node_index == node_index {
        Some(index)
    } else {
        mapping_record_index_tail_spec(node_index, (index + 1) as nat, (fuel - 1) as nat, mappings)
    }
}

pub closed spec fn append_mapping_source_entries_tail_spec(
    source_index: nat,
    source_end: nat,
    destination_start: nat,
    fuel: nat,
    build: MergeExpansionBuildView,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView>
    decreases fuel,
{
    if source_index > source_end || source_end > build.entries.len() || destination_start
        > build.entries.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if source_index == source_end {
        Ok(build)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let source_entry = build.entries[source_index as int];
        match expanded_entry_key_present_tail_spec(
            source_entry.key_node_index,
            destination_start,
            build.entries.len() as nat,
            (build.entries.len() - destination_start) as nat,
            build.entries,
            records,
        ) {
            Err(error) => Err(error),
            Ok(true) => append_mapping_source_entries_tail_spec(
                (source_index + 1) as nat,
                source_end,
                destination_start,
                (fuel - 1) as nat,
                build,
                records,
                nodes,
                limits,
            ),
            Ok(false) => {
                if source_entry.key_node_index >= nodes.len() || source_entry.value_node_index
                    >= nodes.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: 0,
                        },
                    )
                } else {
                    let inherited = ExpandedMappingEntryView {
                        key_node_index: source_entry.key_node_index,
                        value_node_index: source_entry.value_node_index,
                        source_mapping_node_index: source_entry.source_mapping_node_index,
                        source_edge_index: source_entry.source_edge_index,
                        inherited: true,
                    };
                    match admit_expanded_entry_spec(
                        inherited,
                        nodes[source_entry.key_node_index as int].byte_start,
                        build,
                        limits,
                    ) {
                        Err(error) => Err(error),
                        Ok(next) => append_mapping_source_entries_tail_spec(
                            (source_index + 1) as nat,
                            source_end,
                            destination_start,
                            (fuel - 1) as nat,
                            next,
                            records,
                            nodes,
                            limits,
                        ),
                    }
                }
            },
        }
    }
}

pub open spec fn append_mapping_source_spec(
    source_mapping_node_index: u64,
    destination_start: nat,
    build: MergeExpansionBuildView,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView> {
    match mapping_record_index_tail_spec(
        source_mapping_node_index,
        0,
        build.mappings.len() as nat,
        build.mappings,
    ) {
        None => Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: if source_mapping_node_index < nodes.len() {
                    nodes[source_mapping_node_index as int].byte_start
                } else {
                    0
                },
            },
        ),
        Some(record_index) => {
            let record = build.mappings[record_index as int];
            if record.entry_start > record.entry_end || record.entry_end > build.entries.len()
                || destination_start > build.entries.len() {
                Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                )
            } else {
                append_mapping_source_entries_tail_spec(
                    record.entry_start as nat,
                    record.entry_end as nat,
                    destination_start,
                    (record.entry_end - record.entry_start) as nat,
                    build,
                    records,
                    nodes,
                    limits,
                )
            }
        },
    }
}

pub closed spec fn append_explicit_mapping_edges_tail_spec(
    mapping_node_index: u64,
    edge_index: nat,
    edge_end: nat,
    fuel: nat,
    build: MergeExpansionBuildView,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    mapping_edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView>
    decreases fuel,
{
    if edge_index > edge_end || edge_end > mapping_edges.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if edge_index == edge_end {
        Ok(build)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let edge = mapping_edges[edge_index as int];
        match node_is_merge_key_spec(edge.key_node_index, node_slots, scalars) {
            Err(error) => Err(error),
            Ok(true) => append_explicit_mapping_edges_tail_spec(
                mapping_node_index,
                (edge_index + 1) as nat,
                edge_end,
                (fuel - 1) as nat,
                build,
                node_slots,
                scalars,
                nodes,
                mapping_edges,
                limits,
            ),
            Ok(false) => {
                if edge.key_node_index >= nodes.len() || edge.value_node_index >= nodes.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: if mapping_node_index < nodes.len() {
                                nodes[mapping_node_index as int].byte_start
                            } else {
                                0
                            },
                        },
                    )
                } else {
                    let entry = ExpandedMappingEntryView {
                        key_node_index: edge.key_node_index,
                        value_node_index: edge.value_node_index,
                        source_mapping_node_index: mapping_node_index,
                        source_edge_index: edge_index as u64,
                        inherited: false,
                    };
                    match admit_expanded_entry_spec(
                        entry,
                        nodes[edge.key_node_index as int].byte_start,
                        build,
                        limits,
                    ) {
                        Err(error) => Err(error),
                        Ok(next) => append_explicit_mapping_edges_tail_spec(
                            mapping_node_index,
                            (edge_index + 1) as nat,
                            edge_end,
                            (fuel - 1) as nat,
                            next,
                            node_slots,
                            scalars,
                            nodes,
                            mapping_edges,
                            limits,
                        ),
                    }
                }
            },
        }
    }
}

pub closed spec fn append_merge_sources_tail_spec(
    source_index: nat,
    fuel: nat,
    sources: Seq<u64>,
    destination_start: nat,
    build: MergeExpansionBuildView,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView>
    decreases fuel,
{
    if source_index >= sources.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let source = sources[source_index as int];
        if source >= nodes.len() {
            Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                    byte_offset: 0,
                },
            )
        } else {
            let source_limit = merge_expansion_effective_limit_spec(
                limits.max_merge_sources,
                MAX_PROFILE1_MERGE_SOURCES,
            );
            if build.merge_source_count >= source_limit {
                Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::MergeSourceLimitExceeded,
                        byte_offset: nodes[source as int].byte_start,
                    },
                )
            } else {
                let charged = MergeExpansionBuildView {
                    mappings: build.mappings,
                    entries: build.entries,
                    merge_source_count: (build.merge_source_count + 1) as u64,
                };
                match append_mapping_source_spec(
                    source,
                    destination_start,
                    charged,
                    records,
                    nodes,
                    limits,
                ) {
                    Err(error) => Err(error),
                    Ok(next) => append_merge_sources_tail_spec(
                        (source_index + 1) as nat,
                        (fuel - 1) as nat,
                        sources,
                        destination_start,
                        next,
                        records,
                        nodes,
                        limits,
                    ),
                }
            }
        }
    }
}

pub closed spec fn append_merge_mapping_edges_tail_spec(
    edge_index: nat,
    edge_end: nat,
    fuel: nat,
    destination_start: nat,
    build: MergeExpansionBuildView,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mapping_edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView>
    decreases fuel,
{
    if edge_index > edge_end || edge_end > mapping_edges.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if edge_index == edge_end {
        Ok(build)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let edge = mapping_edges[edge_index as int];
        match node_is_merge_key_spec(edge.key_node_index, node_slots, scalars) {
            Err(error) => Err(error),
            Ok(false) => append_merge_mapping_edges_tail_spec(
                (edge_index + 1) as nat,
                edge_end,
                (fuel - 1) as nat,
                destination_start,
                build,
                node_slots,
                scalars,
                nodes,
                sequence_edges,
                mapping_edges,
                records,
                limits,
            ),
            Ok(true) => match merge_sources_for_value_spec(
                edge.value_node_index,
                node_slots,
                nodes,
                sequence_edges,
            ) {
                Err(error) => Err(error),
                Ok(sources) => match append_merge_sources_tail_spec(
                    0,
                    sources.len() as nat,
                    sources,
                    destination_start,
                    build,
                    records,
                    nodes,
                    limits,
                ) {
                    Err(error) => Err(error),
                    Ok(next) => append_merge_mapping_edges_tail_spec(
                        (edge_index + 1) as nat,
                        edge_end,
                        (fuel - 1) as nat,
                        destination_start,
                        next,
                        node_slots,
                        scalars,
                        nodes,
                        sequence_edges,
                        mapping_edges,
                        records,
                        limits,
                    ),
                },
            },
        }
    }
}

pub open spec fn expand_one_mapping_spec(
    node_index: nat,
    build: MergeExpansionBuildView,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mapping_edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView> {
    if node_index >= nodes.len() {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let node = nodes[node_index as int];
        let mapping_limit = merge_expansion_effective_limit_spec(
            limits.max_mappings,
            MAX_PROFILE1_MERGE_MAPPING_RECORDS,
        );
        if build.mappings.len() >= mapping_limit {
            Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::MappingLimitExceeded,
                    byte_offset: node.byte_start,
                },
            )
        } else if node.edge_start > node.edge_end || node.edge_end > mapping_edges.len() {
            Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                    byte_offset: node.byte_start,
                },
            )
        } else {
            let destination_start = build.entries.len() as nat;
            match append_explicit_mapping_edges_tail_spec(
                node_index as u64,
                node.edge_start as nat,
                node.edge_end as nat,
                (node.edge_end - node.edge_start) as nat,
                build,
                node_slots,
                scalars,
                nodes,
                mapping_edges,
                limits,
            ) {
                Err(error) => Err(error),
                Ok(explicit) => match append_merge_mapping_edges_tail_spec(
                    node.edge_start as nat,
                    node.edge_end as nat,
                    (node.edge_end - node.edge_start) as nat,
                    destination_start,
                    explicit,
                    node_slots,
                    scalars,
                    nodes,
                    sequence_edges,
                    mapping_edges,
                    records,
                    limits,
                ) {
                    Err(error) => Err(error),
                    Ok(expanded) => Ok(
                        MergeExpansionBuildView {
                            mappings: expanded.mappings.push(
                                ExpandedMappingRecordView {
                                    node_index: node_index as u64,
                                    entry_start: destination_start as u64,
                                    entry_end: expanded.entries.len() as u64,
                                },
                            ),
                            entries: expanded.entries,
                            merge_source_count: expanded.merge_source_count,
                        },
                    ),
                },
            }
        }
    }
}

pub closed spec fn expand_merge_nodes_tail_spec(
    node_index: nat,
    fuel: nat,
    build: MergeExpansionBuildView,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    scalars: Seq<crate::resolve_scalar_value::ResolvedScalarView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mapping_edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<crate::resolve_canonical_structural_key::CanonicalStructuralKeyRecordView>,
    limits: MergeExpansionLimitsView,
) -> Result<MergeExpansionBuildView, MergeExpansionErrorView>
    decreases fuel,
{
    if node_index >= nodes.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else if nodes[node_index as int].kind != CstNodeKind::Mapping {
        expand_merge_nodes_tail_spec(
            (node_index + 1) as nat,
            (fuel - 1) as nat,
            build,
            node_slots,
            scalars,
            nodes,
            sequence_edges,
            mapping_edges,
            records,
            limits,
        )
    } else {
        match expand_one_mapping_spec(
            node_index,
            build,
            node_slots,
            scalars,
            nodes,
            sequence_edges,
            mapping_edges,
            records,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(next) => expand_merge_nodes_tail_spec(
                (node_index + 1) as nat,
                (fuel - 1) as nat,
                next,
                node_slots,
                scalars,
                nodes,
                sequence_edges,
                mapping_edges,
                records,
                limits,
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Structural)]
pub enum ExpandedReferenceFrameView {
    Node(u64),
    Sequence { next_edge: u64, edge_end: u64 },
    Mapping { next_entry: u64, entry_end: u64, visit_value: bool },
}

pub closed spec fn count_expanded_reference_work_tail_spec(
    stack: Seq<ExpandedReferenceFrameView>,
    fuel: nat,
    references: u64,
    reference_limit: u64,
    source_len_bytes: u64,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mappings: Seq<ExpandedMappingRecordView>,
    entries: Seq<ExpandedMappingEntryView>,
) -> Result<u64, MergeExpansionErrorView>
    decreases fuel,
{
    if stack.len() == 0 {
        Ok(references)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: source_len_bytes,
            },
        )
    } else {
        let frame = stack.last();
        let rest = stack.subrange(0, stack.len() as int - 1);
        match frame {
            ExpandedReferenceFrameView::Node(node_index) => {
                if node_index >= node_slots.len() || node_index >= nodes.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: source_len_bytes,
                        },
                    )
                } else if references >= reference_limit {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::ExpandedReferenceLimitExceeded,
                            byte_offset: nodes[node_index as int].byte_start,
                        },
                    )
                } else {
                    let next_references = (references + 1) as u64;
                    let next_stack = match node_slots[node_index as int].kind {
                        SemanticNodeKind::Scalar => Ok(rest),
                        SemanticNodeKind::Alias => match node_slots[node_index as int].alias_target_node_index {
                            Some(target) => Ok(rest.push(ExpandedReferenceFrameView::Node(target))),
                            None => Err(
                                MergeExpansionErrorView {
                                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                    byte_offset: nodes[node_index as int].byte_start,
                                },
                            ),
                        },
                        SemanticNodeKind::Sequence => {
                            let node = nodes[node_index as int];
                            if node.edge_start > node.edge_end || node.edge_end
                                > sequence_edges.len() {
                                Err(
                                    MergeExpansionErrorView {
                                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                        byte_offset: node.byte_start,
                                    },
                                )
                            } else {
                                Ok(
                                    rest.push(
                                        ExpandedReferenceFrameView::Sequence {
                                            next_edge: node.edge_start,
                                            edge_end: node.edge_end,
                                        },
                                    ),
                                )
                            }
                        },
                        SemanticNodeKind::Mapping => match mapping_record_index_tail_spec(
                            node_index,
                            0,
                            mappings.len() as nat,
                            mappings,
                        ) {
                            None => Err(
                                MergeExpansionErrorView {
                                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                    byte_offset: nodes[node_index as int].byte_start,
                                },
                            ),
                            Some(record_index) => {
                                let record = mappings[record_index as int];
                                if record.entry_start > record.entry_end || record.entry_end
                                    > entries.len() {
                                    Err(
                                        MergeExpansionErrorView {
                                            kind:
                                                MergeExpansionErrorKind::InternalInvariantViolation,
                                            byte_offset: nodes[node_index as int].byte_start,
                                        },
                                    )
                                } else {
                                    Ok(
                                        rest.push(
                                            ExpandedReferenceFrameView::Mapping {
                                                next_entry: record.entry_start,
                                                entry_end: record.entry_end,
                                                visit_value: false,
                                            },
                                        ),
                                    )
                                }
                            },
                        },
                    };
                    match next_stack {
                        Err(error) => Err(error),
                        Ok(next) => count_expanded_reference_work_tail_spec(
                            next,
                            (fuel - 1) as nat,
                            next_references,
                            reference_limit,
                            source_len_bytes,
                            node_slots,
                            nodes,
                            sequence_edges,
                            mappings,
                            entries,
                        ),
                    }
                }
            },
            ExpandedReferenceFrameView::Sequence { next_edge, edge_end } => {
                if next_edge >= edge_end {
                    count_expanded_reference_work_tail_spec(
                        rest,
                        (fuel - 1) as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        node_slots,
                        nodes,
                        sequence_edges,
                        mappings,
                        entries,
                    )
                } else if edge_end > sequence_edges.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: source_len_bytes,
                        },
                    )
                } else {
                    count_expanded_reference_work_tail_spec(
                        rest.push(
                            ExpandedReferenceFrameView::Sequence {
                                next_edge: (next_edge + 1) as u64,
                                edge_end,
                            },
                        ).push(
                            ExpandedReferenceFrameView::Node(
                                sequence_edges[next_edge as int].child_node_index,
                            ),
                        ),
                        (fuel - 1) as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        node_slots,
                        nodes,
                        sequence_edges,
                        mappings,
                        entries,
                    )
                }
            },
            ExpandedReferenceFrameView::Mapping { next_entry, entry_end, visit_value } => {
                if next_entry >= entry_end {
                    count_expanded_reference_work_tail_spec(
                        rest,
                        (fuel - 1) as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        node_slots,
                        nodes,
                        sequence_edges,
                        mappings,
                        entries,
                    )
                } else if entry_end > entries.len() {
                    Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: source_len_bytes,
                        },
                    )
                } else {
                    let entry = entries[next_entry as int];
                    let next = if visit_value {
                        rest.push(
                            ExpandedReferenceFrameView::Mapping {
                                next_entry: (next_entry + 1) as u64,
                                entry_end,
                                visit_value: false,
                            },
                        ).push(ExpandedReferenceFrameView::Node(entry.value_node_index))
                    } else {
                        rest.push(
                            ExpandedReferenceFrameView::Mapping {
                                next_entry,
                                entry_end,
                                visit_value: true,
                            },
                        ).push(ExpandedReferenceFrameView::Node(entry.key_node_index))
                    };
                    count_expanded_reference_work_tail_spec(
                        next,
                        (fuel - 1) as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        node_slots,
                        nodes,
                        sequence_edges,
                        mappings,
                        entries,
                    )
                }
            },
        }
    }
}

pub closed spec fn count_expanded_reference_roots_tail_spec(
    root_index: nat,
    fuel: nat,
    references: u64,
    reference_limit: u64,
    source_len_bytes: u64,
    roots: Seq<crate::resolve_topology::SemanticDocumentRootView>,
    node_slots: Seq<crate::resolve_node_table::SemanticNodeSlotView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    sequence_edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    mappings: Seq<ExpandedMappingRecordView>,
    entries: Seq<ExpandedMappingEntryView>,
) -> Result<u64, MergeExpansionErrorView>
    decreases fuel,
{
    if root_index >= roots.len() {
        Ok(references)
    } else if fuel == 0 {
        Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: source_len_bytes,
            },
        )
    } else {
        match count_expanded_reference_work_tail_spec(
            Seq::empty().push(
                ExpandedReferenceFrameView::Node(roots[root_index as int].node_index),
            ),
            ((reference_limit as nat + 1) * 3 + 3) as nat,
            references,
            reference_limit,
            source_len_bytes,
            node_slots,
            nodes,
            sequence_edges,
            mappings,
            entries,
        ) {
            Err(error) => Err(error),
            Ok(next_references) => count_expanded_reference_roots_tail_spec(
                (root_index + 1) as nat,
                (fuel - 1) as nat,
                next_references,
                reference_limit,
                source_len_bytes,
                roots,
                node_slots,
                nodes,
                sequence_edges,
                mappings,
                entries,
            ),
        }
    }
}

pub open spec fn finalize_merge_expansion_spec(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
    build: MergeExpansionBuildView,
) -> Result<ExpandedSemanticGraphSourceView, MergeExpansionErrorView> {
    let table = input.structural_keys.scalar_keys.graph.node_table;
    let topology = table.topology;
    let reference_limit = merge_expansion_effective_limit_spec(
        limits.max_expanded_references,
        MAX_PROFILE1_EXPANDED_REFERENCES,
    );
    match count_expanded_reference_roots_tail_spec(
        0,
        topology.document_roots.len() as nat,
        0,
        reference_limit,
        topology.source_len_bytes,
        topology.document_roots,
        table.nodes,
        topology.nodes,
        topology.sequence_edges,
        build.mappings,
        build.entries,
    ) {
        Err(error) => Err(error),
        Ok(expanded_reference_count) => Ok(
            ExpandedSemanticGraphSourceView {
                profile_version: input.profile_version,
                transformation_version: MERGE_EXPANSION_TRANSFORMATION_VERSION,
                source_len_bytes: input.source_len_bytes,
                input_node_count: input.input_node_count,
                expanded_reference_count,
                merge_source_count: build.merge_source_count,
                input,
                mappings: build.mappings,
                entries: build.entries,
            },
        ),
    }
}

fn decoded_content_is_merge(content: &[crate::scalar_decode::DecodedContentScalar]) -> (result:
    bool)
    ensures
        result == decoded_merge_ascii_spec(
            crate::scalar_decode::decoded_content_scalar_views_spec(content@),
            Seq::empty(),
        ),
{
    if content.len() != 2 {
        return false;
    }
    let first = content[0].code_point();
    let second = content[1].code_point();
    proof {
        reveal(decoded_merge_ascii_spec);
        reveal(crate::scalar_decode::decoded_content_scalar_views_spec);
    }
    first == b'<' as u32 && second == b'<' as u32
}

fn tag_content_is_merge(content: &[crate::resolve_tag::ResolvedTagCodePoint]) -> (result: bool)
    ensures
        result == tag_merge_ascii_spec(
            crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
            Seq::empty(),
        ),
{
    if content.len() != 23 {
        return false;
    }
    let result = content[0].code_point() == b't' as u32 && content[1].code_point() == b'a' as u32
        && content[2].code_point() == b'g' as u32 && content[3].code_point() == b':' as u32
        && content[4].code_point() == b'y' as u32 && content[5].code_point() == b'a' as u32
        && content[6].code_point() == b'm' as u32 && content[7].code_point() == b'l' as u32
        && content[8].code_point() == b'.' as u32 && content[9].code_point() == b'o' as u32
        && content[10].code_point() == b'r' as u32 && content[11].code_point() == b'g' as u32
        && content[12].code_point() == b',' as u32 && content[13].code_point() == b'2' as u32
        && content[14].code_point() == b'0' as u32 && content[15].code_point() == b'0' as u32
        && content[16].code_point() == b'2' as u32 && content[17].code_point() == b':' as u32
        && content[18].code_point() == b'm' as u32 && content[19].code_point() == b'e' as u32
        && content[20].code_point() == b'r' as u32 && content[21].code_point() == b'g' as u32
        && content[22].code_point() == b'e' as u32;
    proof {
        reveal(tag_merge_ascii_spec);
        reveal(crate::resolve_tag::resolved_tag_code_point_views_spec);
    }
    result
}

fn scalar_is_merge_key(scalar: &ResolvedScalar) -> (result: bool)
    ensures
        result == resolved_scalar_is_merge_key_spec(scalar@),
{
    match scalar.explicit_tag() {
        Some(tag) => {
            let result = tag_content_is_merge(tag.content());
            proof {
                reveal(resolved_scalar_is_merge_key_spec);
                reveal(tag_merge_ascii_spec);
            }
            result
        },
        None => {
            if scalar.presentation().style() != CstNodeStyle::Plain {
                proof {
                    reveal(resolved_scalar_is_merge_key_spec);
                }
                return false;
            }
            match scalar.presentation().decoded() {
                Some(decoded) => {
                    let result = decoded_content_is_merge(decoded.content());
                    proof {
                        reveal(resolved_scalar_is_merge_key_spec);
                        reveal(decoded_merge_ascii_spec);
                    }
                    result
                },
                None => {
                    proof {
                        reveal(resolved_scalar_is_merge_key_spec);
                    }
                    false
                },
            }
        },
    }
}

fn node_is_merge_key(
    key_node_index: u64,
    node_slots: &[SemanticNodeSlot],
    scalars: &[ResolvedScalar],
) -> (result: Result<bool, MergeExpansionError>)
    ensures
        node_is_merge_key_spec(
            key_node_index,
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
        ) == match result {
            Ok(value) => Ok(value),
            Err(error) => Err(error@),
        },
{
    if key_node_index >= node_slots.len() as u64 {
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let slot = &node_slots[key_node_index as usize];
    if slot.kind() != SemanticNodeKind::Scalar {
        return Ok(false);
    }
    match slot.value_index() {
        Some(value_index) => {
            if value_index >= scalars.len() as u64 {
                return Err(
                    MergeExpansionError::at(
                        MergeExpansionErrorKind::InternalInvariantViolation,
                        slot.byte_start(),
                    ),
                );
            }
            Ok(scalar_is_merge_key(&scalars[value_index as usize]))
        },
        None => Ok(false),
    }
}

fn follow_alias(node_index: u64, node_slots: &[SemanticNodeSlot]) -> (result: Result<
    u64,
    MergeExpansionError,
>)
    ensures
        match result {
            Ok(index) => index < node_slots@.len(),
            Err(_) => true,
        },
        follow_alias_spec(
            node_index,
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
        ) == match result {
            Ok(index) => Ok(index),
            Err(error) => Err(error@),
        },
{
    let mut current = node_index;
    let mut fuel = node_slots.len();
    let ghost slots = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost expected = follow_alias_tail_spec(node_index, node_slots@.len() as nat, slots);
    proof {
        reveal(follow_alias_spec);
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(slots.len() == node_slots@.len());
    }
    while fuel > 0
        invariant
            fuel <= node_slots.len(),
            slots.len() == node_slots@.len(),
            slots == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            expected == follow_alias_tail_spec(current, fuel as nat, slots),
            follow_alias_spec(node_index, slots) == expected,
        decreases fuel,
    {
        if current >= node_slots.len() as u64 {
            proof {
                reveal(follow_alias_tail_spec);
                reveal(follow_alias_spec);
                assert(current >= slots.len());
                assert(expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0),
            );
        }
        let slot = &node_slots[current as usize];
        let kind = slot.kind();
        proof {
            assert(0int <= current as int);
            assert((current as int) < node_slots.len() as int);
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slots[current as int] == node_slots[current as int]@);
            assert(slots[current as int].kind == kind);
        }
        if kind != SemanticNodeKind::Alias {
            proof {
                reveal(follow_alias_tail_spec);
                reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
                reveal(follow_alias_spec);
                assert(expected == Ok(current));
            }
            return Ok(current);
        }
        let alias_target = slot.alias_target_node_index();
        proof {
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slots[current as int].alias_target_node_index == alias_target);
        }
        let target = match alias_target {
            Some(target) => target,
            None => {
                let byte_offset = slot.byte_start();
                proof {
                    reveal(follow_alias_tail_spec);
                    reveal(follow_alias_spec);
                    reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
                    assert(expected == Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset,
                        },
                    ));
                }
                return Err(
                    MergeExpansionError::at(
                        MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset,
                    ),
                );
            },
        };
        proof {
            reveal(follow_alias_tail_spec);
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(expected == follow_alias_tail_spec(target, (fuel - 1) as nat, slots));
        }
        current = target;
        fuel -= 1;
    }
    let offset = if current < node_slots.len() as u64 {
        node_slots[current as usize].byte_start()
    } else {
        0
    };
    proof {
        reveal(follow_alias_tail_spec);
        reveal(follow_alias_spec);
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(expected == Err(
            MergeExpansionErrorView {
                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                byte_offset: offset,
            },
        ));
    }
    Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, offset))
}

fn validate_merge_value(
    value_node_index: u64,
    node_slots: &[SemanticNodeSlot],
    topology_nodes: &[SemanticTopologyNode],
    sequence_edges: &[crate::resolve_topology::SemanticSequenceEdge],
) -> (result: Result<(), MergeExpansionError>)
    ensures
        validate_merge_value_spec(
            value_node_index,
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
        ) == match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error@),
        },
{
    let ghost slots = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost nodes = crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@);
    let ghost edges = crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@);
    let target = follow_alias(value_node_index, node_slots)?;
    if target >= node_slots.len() as u64 || target >= topology_nodes.len() as u64 {
        proof {
            reveal(validate_merge_value_spec);
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let target_kind = node_slots[target as usize].kind();
    proof {
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(slots[target as int] == node_slots[target as int]@);
        assert(slots[target as int].kind == target_kind);
    }
    match target_kind {
        SemanticNodeKind::Mapping => {
            proof {
                reveal(validate_merge_value_spec);
            }
            Ok(())
        },
        SemanticNodeKind::Sequence => {
            let node = &topology_nodes[target as usize];
            let node_kind = node.kind();
            let edge_start_u64 = node.edge_start();
            let edge_end_u64 = node.edge_end();
            proof {
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                assert(nodes[target as int] == topology_nodes[target as int]@);
            }
            if node_kind != CstNodeKind::Sequence || edge_start_u64 > edge_end_u64 || edge_end_u64
                > sequence_edges.len() as u64 {
                proof {
                    reveal(validate_merge_value_spec);
                }
                return Err(
                    MergeExpansionError::at(
                        MergeExpansionErrorKind::InternalInvariantViolation,
                        node.byte_start(),
                    ),
                );
            }
            let mut edge_index = edge_start_u64 as usize;
            let edge_end = edge_end_u64 as usize;
            let ghost expected = validate_merge_sequence_tail_spec(
                edge_start_u64 as nat,
                edge_end_u64 as nat,
                (edge_end_u64 - edge_start_u64) as nat,
                edges,
                slots,
            );
            proof {
                reveal(validate_merge_value_spec);
                assert(validate_merge_value_spec(value_node_index, slots, nodes, edges)
                    == expected);
            }
            while edge_index < edge_end
                invariant
                    edge_index <= edge_end,
                    edge_end <= sequence_edges.len(),
                    slots == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
                    nodes == crate::resolve_topology::semantic_topology_node_views_spec(
                        topology_nodes@,
                    ),
                    edges == crate::resolve_topology::semantic_sequence_edge_views_spec(
                        sequence_edges@,
                    ),
                    expected == validate_merge_sequence_tail_spec(
                        edge_index as nat,
                        edge_end as nat,
                        (edge_end - edge_index) as nat,
                        edges,
                        slots,
                    ),
                    validate_merge_value_spec(value_node_index, slots, nodes, edges) == expected,
                decreases edge_end - edge_index,
            {
                let child_node_index = sequence_edges[edge_index].child_node_index();
                proof {
                    reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
                    assert(edges[edge_index as int] == sequence_edges[edge_index as int]@);
                }
                let child = match follow_alias(child_node_index, node_slots) {
                    Ok(child) => child,
                    Err(error) => {
                        proof {
                            reveal(validate_merge_sequence_tail_spec);
                            assert(expected == Err(error@));
                            reveal(validate_merge_value_spec);
                            assert(validate_merge_value_spec(
                                value_node_index,
                                crate::resolve_node_table::semantic_node_slot_views_spec(
                                    node_slots@,
                                ),
                                crate::resolve_topology::semantic_topology_node_views_spec(
                                    topology_nodes@,
                                ),
                                crate::resolve_topology::semantic_sequence_edge_views_spec(
                                    sequence_edges@,
                                ),
                            ) == Err(error@));
                        }
                        return Err(error);
                    },
                };
                if child >= node_slots.len() as u64 {
                    proof {
                        reveal(validate_merge_sequence_tail_spec);
                        assert(expected == Err(
                            MergeExpansionErrorView {
                                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                byte_offset: 0,
                            },
                        ));
                        reveal(validate_merge_value_spec);
                        assert(validate_merge_value_spec(
                            value_node_index,
                            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
                            crate::resolve_topology::semantic_topology_node_views_spec(
                                topology_nodes@,
                            ),
                            crate::resolve_topology::semantic_sequence_edge_views_spec(
                                sequence_edges@,
                            ),
                        ) == Err(
                            MergeExpansionErrorView {
                                kind: MergeExpansionErrorKind::InternalInvariantViolation,
                                byte_offset: 0,
                            },
                        ));
                    }
                    return Err(
                        MergeExpansionError::at(
                            MergeExpansionErrorKind::InternalInvariantViolation,
                            0,
                        ),
                    );
                }
                let child_kind = node_slots[child as usize].kind();
                let child_byte = node_slots[child as usize].byte_start();
                proof {
                    reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
                    assert(slots[child as int] == node_slots[child as int]@);
                }
                if child_kind != SemanticNodeKind::Mapping {
                    proof {
                        reveal(validate_merge_sequence_tail_spec);
                        assert(expected == Err(
                            MergeExpansionErrorView {
                                kind: MergeExpansionErrorKind::InvalidMergeValue,
                                byte_offset: child_byte,
                            },
                        ));
                        reveal(validate_merge_value_spec);
                        assert(validate_merge_value_spec(
                            value_node_index,
                            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
                            crate::resolve_topology::semantic_topology_node_views_spec(
                                topology_nodes@,
                            ),
                            crate::resolve_topology::semantic_sequence_edge_views_spec(
                                sequence_edges@,
                            ),
                        ) == Err(
                            MergeExpansionErrorView {
                                kind: MergeExpansionErrorKind::InvalidMergeValue,
                                byte_offset: child_byte,
                            },
                        ));
                    }
                    return Err(
                        MergeExpansionError::at(
                            MergeExpansionErrorKind::InvalidMergeValue,
                            child_byte,
                        ),
                    );
                }
                proof {
                    reveal(validate_merge_sequence_tail_spec);
                    assert(expected == validate_merge_sequence_tail_spec(
                        (edge_index + 1) as nat,
                        edge_end as nat,
                        (edge_end - edge_index - 1) as nat,
                        edges,
                        slots,
                    ));
                }
                edge_index += 1;
            }
            proof {
                reveal(validate_merge_sequence_tail_spec);
                assert(expected == Ok(()));
                reveal(validate_merge_value_spec);
            }
            Ok(())
        },
        _ => {
            let byte_offset = node_slots[target as usize].byte_start();
            proof {
                reveal(validate_merge_value_spec);
            }
            Err(MergeExpansionError::at(MergeExpansionErrorKind::InvalidMergeValue, byte_offset))
        },
    }
}

fn merge_sources_for_value(
    value_node_index: u64,
    node_slots: &[SemanticNodeSlot],
    topology_nodes: &[SemanticTopologyNode],
    sequence_edges: &[crate::resolve_topology::SemanticSequenceEdge],
) -> (result: Result<Vec<u64>, MergeExpansionError>)
    ensures
        merge_sources_for_value_spec(
            value_node_index,
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
        ) == match result {
            Ok(sources) => Ok(sources@),
            Err(error) => Err(error@),
        },
{
    let ghost slots = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost nodes = crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@);
    let ghost edges = crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@);
    let target = match follow_alias(value_node_index, node_slots) {
        Ok(target) => target,
        Err(error) => {
            proof {
                reveal(merge_sources_for_value_spec);
            }
            return Err(error);
        },
    };
    if target >= node_slots.len() as u64 || target >= topology_nodes.len() as u64 {
        proof {
            reveal(merge_sources_for_value_spec);
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let target_kind = node_slots[target as usize].kind();
    proof {
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(slots[target as int] == node_slots[target as int]@);
    }
    if target_kind == SemanticNodeKind::Mapping {
        let sources = vec![target];
        proof {
            reveal(merge_sources_for_value_spec);
            assert(sources@ == Seq::empty().push(target));
        }
        return Ok(sources);
    }
    if target_kind != SemanticNodeKind::Sequence {
        let byte_offset = node_slots[target as usize].byte_start();
        proof {
            reveal(merge_sources_for_value_spec);
        }
        return Err(
            MergeExpansionError::at(MergeExpansionErrorKind::InvalidMergeValue, byte_offset),
        );
    }
    let node = &topology_nodes[target as usize];
    let node_kind = node.kind();
    let edge_start_u64 = node.edge_start();
    let edge_end_u64 = node.edge_end();
    let node_byte = node.byte_start();
    proof {
        reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        assert(nodes[target as int] == topology_nodes[target as int]@);
    }
    if node_kind != CstNodeKind::Sequence || edge_start_u64 > edge_end_u64 || edge_end_u64
        > sequence_edges.len() as u64 {
        proof {
            reveal(merge_sources_for_value_spec);
        }
        return Err(
            MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, node_byte),
        );
    }
    let ghost full_expected = merge_sequence_sources_tail_spec(
        edge_start_u64 as nat,
        edge_end_u64 as nat,
        (edge_end_u64 - edge_start_u64) as nat,
        edges,
        slots,
    );
    proof {
        reveal(merge_sources_for_value_spec);
    }
    let mut sources = Vec::new();
    let mut edge_index = edge_start_u64 as usize;
    let edge_end = edge_end_u64 as usize;
    while edge_index < edge_end
        invariant
            edge_index <= edge_end,
            edge_end <= sequence_edges.len(),
            slots == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            nodes == crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@),
            edges == crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            full_expected == match merge_sequence_sources_tail_spec(
                edge_index as nat,
                edge_end as nat,
                (edge_end - edge_index) as nat,
                edges,
                slots,
            ) {
                Ok(tail) => Ok(sources@ + tail),
                Err(error) => Err(error),
            },
            merge_sources_for_value_spec(value_node_index, slots, nodes, edges) == full_expected,
            merge_sources_for_value_spec(
                value_node_index,
                crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
                crate::resolve_topology::semantic_topology_node_views_spec(topology_nodes@),
                crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            ) == full_expected,
        decreases edge_end - edge_index,
    {
        let child_node_index = sequence_edges[edge_index].child_node_index();
        proof {
            reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
            assert(edges[edge_index as int] == sequence_edges[edge_index as int]@);
        }
        let child = match follow_alias(child_node_index, node_slots) {
            Ok(child) => child,
            Err(error) => {
                proof {
                    reveal(merge_sequence_sources_tail_spec);
                    assert(full_expected == Err(error@));
                }
                return Err(error);
            },
        };
        if child >= node_slots.len() as u64 {
            proof {
                reveal(merge_sequence_sources_tail_spec);
                assert(full_expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0),
            );
        }
        let child_kind = node_slots[child as usize].kind();
        let child_byte = node_slots[child as usize].byte_start();
        proof {
            reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
            assert(slots[child as int] == node_slots[child as int]@);
        }
        if child_kind != SemanticNodeKind::Mapping {
            proof {
                reveal(merge_sequence_sources_tail_spec);
                assert(full_expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InvalidMergeValue,
                        byte_offset: child_byte,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(MergeExpansionErrorKind::InvalidMergeValue, child_byte),
            );
        }
        let ghost before = sources@;
        sources.push(child);
        proof {
            reveal(merge_sequence_sources_tail_spec);
            assert(sources@ == before.push(child));
            assert forall|tail: Seq<u64>|
                #![auto]
                before + (Seq::empty().push(child) + tail) =~= sources@ + tail by {};
        }
        edge_index += 1;
    }
    proof {
        reveal(merge_sequence_sources_tail_spec);
        assert(full_expected == Ok(sources@));
    }
    Ok(sources)
}

fn preflight_merge_shapes(source: &DuplicateFreeStructuralKeySource) -> (result: Result<
    (),
    MergeExpansionError,
>)
    ensures
        preflight_merge_shapes_spec(source@) == match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error@),
        },
{
    let structural = source.structural_keys();
    let table = structural.scalar_keys().graph().node_table();
    let topology = table.topology();
    let nodes = topology.nodes();
    let node_slots = table.nodes();
    let scalars = table.scalars().scalars();
    let sequence_edges = topology.sequence_edges();
    let mapping_edges = topology.mapping_edges();
    if nodes.len() != node_slots.len() || structural.records().len() != nodes.len() {
        proof {
            reveal(preflight_merge_shapes_spec);
            reveal(merge_expansion_input_shape_spec);
        }
        return Err(
            MergeExpansionError::at(
                MergeExpansionErrorKind::InternalInvariantViolation,
                source.source_len_bytes(),
            ),
        );
    }
    let ghost slots_view = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost scalars_view = crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@);
    let ghost nodes_view = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_view = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost mapping_view = crate::resolve_topology::semantic_mapping_edge_views_spec(
        mapping_edges@,
    );
    let ghost expected = preflight_merge_nodes_tail_spec(
        0,
        nodes@.len() as nat,
        slots_view,
        scalars_view,
        nodes_view,
        sequence_view,
        mapping_view,
    );
    proof {
        reveal(preflight_merge_shapes_spec);
        reveal(merge_expansion_input_shape_spec);
    }
    let mut node_index = 0usize;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            slots_view == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            scalars_view == crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            nodes_view == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            sequence_view == crate::resolve_topology::semantic_sequence_edge_views_spec(
                sequence_edges@,
            ),
            mapping_view == crate::resolve_topology::semantic_mapping_edge_views_spec(
                mapping_edges@,
            ),
            preflight_merge_shapes_spec(source@) == expected,
            expected == preflight_merge_nodes_tail_spec(
                node_index as nat,
                (nodes.len() - node_index) as nat,
                slots_view,
                scalars_view,
                nodes_view,
                sequence_view,
                mapping_view,
            ),
        decreases nodes.len() - node_index,
    {
        let node = &nodes[node_index];
        let node_kind = node.kind();
        let edge_start_u64 = node.edge_start();
        let edge_end_u64 = node.edge_end();
        let node_byte = node.byte_start();
        proof {
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            assert(nodes_view[node_index as int] == nodes[node_index as int]@);
        }
        if node_kind == CstNodeKind::Mapping {
            if edge_start_u64 > edge_end_u64 || edge_end_u64 > mapping_edges.len() as u64 {
                proof {
                    reveal(preflight_merge_nodes_tail_spec);
                    assert(expected == Err(
                        MergeExpansionErrorView {
                            kind: MergeExpansionErrorKind::InternalInvariantViolation,
                            byte_offset: node_byte,
                        },
                    ));
                }
                return Err(
                    MergeExpansionError::at(
                        MergeExpansionErrorKind::InternalInvariantViolation,
                        node_byte,
                    ),
                );
            }
            let mut edge_index = edge_start_u64 as usize;
            let edge_end = edge_end_u64 as usize;
            let ghost mapping_expected = preflight_mapping_edges_tail_spec(
                edge_start_u64 as nat,
                edge_end_u64 as nat,
                (edge_end_u64 - edge_start_u64) as nat,
                slots_view,
                scalars_view,
                nodes_view,
                sequence_view,
                mapping_view,
            );
            proof {
                reveal(preflight_merge_nodes_tail_spec);
                assert(expected == match mapping_expected {
                    Err(error) => Err(error),
                    Ok(()) => preflight_merge_nodes_tail_spec(
                        (node_index + 1) as nat,
                        (nodes.len() - node_index - 1) as nat,
                        slots_view,
                        scalars_view,
                        nodes_view,
                        sequence_view,
                        mapping_view,
                    ),
                });
            }
            while edge_index < edge_end
                invariant
                    edge_index <= edge_end,
                    edge_end <= mapping_edges.len(),
                    slots_view == crate::resolve_node_table::semantic_node_slot_views_spec(
                        node_slots@,
                    ),
                    scalars_view == crate::resolve_scalar_table::semantic_scalar_views_spec(
                        scalars@,
                    ),
                    nodes_view == crate::resolve_topology::semantic_topology_node_views_spec(
                        nodes@,
                    ),
                    sequence_view == crate::resolve_topology::semantic_sequence_edge_views_spec(
                        sequence_edges@,
                    ),
                    mapping_view == crate::resolve_topology::semantic_mapping_edge_views_spec(
                        mapping_edges@,
                    ),
                    preflight_merge_shapes_spec(source@) == expected,
                    mapping_expected == preflight_mapping_edges_tail_spec(
                        edge_index as nat,
                        edge_end as nat,
                        (edge_end - edge_index) as nat,
                        slots_view,
                        scalars_view,
                        nodes_view,
                        sequence_view,
                        mapping_view,
                    ),
                    expected == match mapping_expected {
                        Err(error) => Err(error),
                        Ok(()) => preflight_merge_nodes_tail_spec(
                            (node_index + 1) as nat,
                            (nodes.len() - node_index - 1) as nat,
                            slots_view,
                            scalars_view,
                            nodes_view,
                            sequence_view,
                            mapping_view,
                        ),
                    },
                decreases edge_end - edge_index,
            {
                let edge = &mapping_edges[edge_index];
                let key_node_index = edge.key_node_index();
                let value_node_index = edge.value_node_index();
                proof {
                    reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                    assert(mapping_view[edge_index as int] == mapping_edges[edge_index as int]@);
                }
                let is_merge = match node_is_merge_key(key_node_index, node_slots, scalars) {
                    Ok(value) => value,
                    Err(error) => {
                        proof {
                            reveal(preflight_mapping_edges_tail_spec);
                            assert(mapping_expected == Err(error@));
                            assert(expected == Err(error@));
                        }
                        return Err(error);
                    },
                };
                if is_merge {
                    match validate_merge_value(
                        value_node_index,
                        node_slots,
                        nodes,
                        sequence_edges,
                    ) {
                        Ok(()) => {},
                        Err(error) => {
                            proof {
                                reveal(preflight_mapping_edges_tail_spec);
                                assert(mapping_expected == Err(error@));
                                assert(expected == Err(error@));
                            }
                            return Err(error);
                        },
                    }
                }
                proof {
                    reveal(preflight_mapping_edges_tail_spec);
                    assert(mapping_expected == preflight_mapping_edges_tail_spec(
                        (edge_index + 1) as nat,
                        edge_end as nat,
                        (edge_end - edge_index - 1) as nat,
                        slots_view,
                        scalars_view,
                        nodes_view,
                        sequence_view,
                        mapping_view,
                    ));
                }
                edge_index += 1;
            }
            proof {
                reveal(preflight_mapping_edges_tail_spec);
                assert(mapping_expected == Ok(()));
            }
        }
        proof {
            reveal(preflight_merge_nodes_tail_spec);
        }
        node_index += 1;
    }
    proof {
        reveal(preflight_merge_nodes_tail_spec);
        assert(expected == Ok(()));
    }
    Ok(())
}

fn mapping_record_index(node_index: u64, mappings: &[ExpandedMappingRecord]) -> (result: Option<
    usize,
>)
    ensures
        match result {
            Some(index) => index < mappings@.len(),
            None => true,
        },
        mapping_record_index_tail_spec(
            node_index,
            0,
            mappings@.len() as nat,
            expanded_mapping_record_views_spec(mappings@),
        ) == match result {
            Some(index) => Some(index as nat),
            None => None,
        },
{
    let ghost views = expanded_mapping_record_views_spec(mappings@);
    let ghost expected = mapping_record_index_tail_spec(
        node_index,
        0,
        mappings@.len() as nat,
        views,
    );
    let mut index = 0usize;
    while index < mappings.len()
        invariant
            index <= mappings.len(),
            views == expanded_mapping_record_views_spec(mappings@),
            mapping_record_index_tail_spec(
                node_index,
                0,
                mappings@.len() as nat,
                expanded_mapping_record_views_spec(mappings@),
            ) == expected,
            expected == mapping_record_index_tail_spec(
                node_index,
                index as nat,
                (mappings.len() - index) as nat,
                views,
            ),
        decreases mappings.len() - index,
    {
        let candidate = mappings[index].node_index();
        proof {
            reveal(expanded_mapping_record_views_spec);
            assert(views[index as int] == mappings[index as int]@);
        }
        if candidate == node_index {
            proof {
                reveal(mapping_record_index_tail_spec);
                assert(expected == Some(index as nat));
            }
            return Some(index);
        }
        proof {
            reveal(mapping_record_index_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(mapping_record_index_tail_spec);
    }
    None
}

fn entry_key_already_present(
    key_node_index: u64,
    destination_start: usize,
    entries: &[ExpandedMappingEntry],
    records: &[CanonicalStructuralKeyRecord],
) -> (result: Result<bool, MergeExpansionError>)
    ensures
        expanded_entry_key_present_tail_spec(
            key_node_index,
            destination_start as nat,
            entries@.len() as nat,
            if destination_start <= entries@.len() {
                (entries@.len() - destination_start) as nat
            } else {
                0nat
            },
            expanded_mapping_entry_views_spec(entries@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
        ) == match result {
            Ok(found) => Ok(found),
            Err(error) => Err(error@),
        },
{
    let ghost entry_views = expanded_mapping_entry_views_spec(entries@);
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    if key_node_index >= records.len() as u64 || destination_start > entries.len() {
        proof {
            reveal(expanded_entry_key_present_tail_spec);
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let candidate = records[key_node_index as usize].bytes();
    proof {
        reveal(crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec);
        assert(record_views[key_node_index as int] == records[key_node_index as int]@);
        assert(crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(candidate@)
            == record_views[key_node_index as int].bytes);
    }
    let ghost expected = expanded_entry_key_present_tail_spec(
        key_node_index,
        destination_start as nat,
        entries@.len() as nat,
        (entries@.len() - destination_start) as nat,
        entry_views,
        record_views,
    );
    let mut index = destination_start;
    while index < entries.len()
        invariant
            destination_start <= index,
            index <= entries.len(),
            key_node_index < records.len(),
            crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(candidate@)
                == record_views[key_node_index as int].bytes,
            entry_views == expanded_mapping_entry_views_spec(entries@),
            record_views
                == crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@),
            expanded_entry_key_present_tail_spec(
                key_node_index,
                destination_start as nat,
                entries@.len() as nat,
                (entries@.len() - destination_start) as nat,
                expanded_mapping_entry_views_spec(entries@),
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
            ) == expected,
            expected == expanded_entry_key_present_tail_spec(
                key_node_index,
                index as nat,
                entries@.len() as nat,
                (entries@.len() - index) as nat,
                entry_views,
                record_views,
            ),
        decreases entries.len() - index,
    {
        let existing_key = entries[index].key_node_index();
        proof {
            reveal(expanded_mapping_entry_views_spec);
            assert(entry_views[index as int] == entries[index as int]@);
        }
        if existing_key >= records.len() as u64 {
            proof {
                reveal(expanded_entry_key_present_tail_spec);
                reveal(canonical_merge_key_equal_spec);
                assert(expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0),
            );
        }
        let existing = records[existing_key as usize].bytes();
        let order = compare_byte_slices(candidate, existing);
        proof {
            reveal(
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec,
            );
            reveal(canonical_merge_key_equal_spec);
            assert(record_views[key_node_index as int] == records[key_node_index as int]@);
            assert(record_views[existing_key as int] == records[existing_key as int]@);
            assert(record_views[key_node_index as int].bytes
                == crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(candidate@));
            assert(record_views[existing_key as int].bytes
                == crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(existing@));
            assert(canonical_merge_key_equal_spec(key_node_index, existing_key, record_views) == Ok(
                order == 0,
            ));
        }
        if order == 0 {
            proof {
                reveal(expanded_entry_key_present_tail_spec);
                assert(expected == Ok(true));
            }
            return Ok(true);
        }
        proof {
            reveal(expanded_entry_key_present_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(expanded_entry_key_present_tail_spec);
    }
    Ok(false)
}

fn charge_and_push_entry(
    entry: ExpandedMappingEntry,
    key_byte: u64,
    entries: &mut Vec<ExpandedMappingEntry>,
    entry_limit: u64,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        final(entries)@.len() >= old(entries)@.len(),
        admit_expanded_entry_limit_spec(
            entry@,
            key_byte,
            expanded_mapping_entry_views_spec(old(entries)@),
            entry_limit,
        ) == match result {
            Ok(()) => Ok(expanded_mapping_entry_views_spec(final(entries)@)),
            Err(error) => Err(error@),
        },
{
    if entries.len() as u64 >= entry_limit {
        proof {
            reveal(admit_expanded_entry_limit_spec);
        }
        return Err(
            MergeExpansionError::at(
                MergeExpansionErrorKind::ExpandedMappingEntryLimitExceeded,
                key_byte,
            ),
        );
    }
    let ghost before = entries@;
    entries.push(entry);
    proof {
        reveal(admit_expanded_entry_limit_spec);
        reveal(expanded_mapping_entry_views_spec);
        assert(expanded_mapping_entry_views_spec(entries@) == expanded_mapping_entry_views_spec(
            before,
        ).push(entry@));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Every authenticated graph and build input remains explicit.
fn append_mapping_source(
    source_mapping_node_index: u64,
    destination_start: usize,
    merge_source_count: u64,
    mappings: &[ExpandedMappingRecord],
    entries: &mut Vec<ExpandedMappingEntry>,
    records: &[CanonicalStructuralKeyRecord],
    nodes: &[SemanticTopologyNode],
    limits: MergeExpansionLimits,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        final(entries)@.len() >= old(entries)@.len(),
        append_mapping_source_spec(
            source_mapping_node_index,
            destination_start as nat,
            MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(mappings@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count,
            },
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(final(entries)@),
                    merge_source_count,
                },
            ),
            Err(error) => Err(error@),
        },
{
    // These values are executable inputs to the exact ghost contract even when
    // this helper does not inspect them after proof erasure.
    let _ = merge_source_count;
    let entry_limit = effective_limit(
        limits.max_expanded_mapping_entries(),
        MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
    );
    let ghost mapping_views = expanded_mapping_record_views_spec(mappings@);
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost initial_entries = expanded_mapping_entry_views_spec(entries@);
    let ghost initial_build = MergeExpansionBuildView {
        mappings: mapping_views,
        entries: initial_entries,
        merge_source_count,
    };
    let ghost top_expected = append_mapping_source_spec(
        source_mapping_node_index,
        destination_start as nat,
        initial_build,
        record_views,
        node_views,
        limits@,
    );
    let record_index = match mapping_record_index(source_mapping_node_index, mappings) {
        Some(index) => index,
        None => {
            let offset = if source_mapping_node_index < nodes.len() as u64 {
                nodes[source_mapping_node_index as usize].byte_start()
            } else {
                0
            };
            proof {
                reveal(append_mapping_source_spec);
                assert(top_expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: offset,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(
                    MergeExpansionErrorKind::InternalInvariantViolation,
                    offset,
                ),
            );
        },
    };
    let source_start_u64 = mappings[record_index].entry_start();
    let source_end_u64 = mappings[record_index].entry_end();
    proof {
        reveal(expanded_mapping_record_views_spec);
        assert(mapping_views[record_index as int] == mappings[record_index as int]@);
        reveal(append_mapping_source_spec);
    }
    if source_start_u64 > source_end_u64 || source_end_u64 > entries.len() as u64 {
        proof {
            assert(top_expected == Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                    byte_offset: 0,
                },
            ));
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let source_start = source_start_u64 as usize;
    let source_end = source_end_u64 as usize;
    let source_snapshot_end = source_end;
    if destination_start > entries.len() {
        proof {
            assert(top_expected == Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                    byte_offset: 0,
                },
            ));
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let ghost expected = append_mapping_source_entries_tail_spec(
        source_start as nat,
        source_end as nat,
        destination_start as nat,
        (source_end - source_start) as nat,
        initial_build,
        record_views,
        node_views,
        limits@,
    );
    proof {
        assert(top_expected == expected);
    }
    let mut source_index = source_start;
    while source_index < source_snapshot_end
        invariant
            source_start <= source_index,
            source_index <= source_snapshot_end,
            source_snapshot_end <= entries.len(),
            destination_start <= entries.len(),
            entries@.len() >= old(entries)@.len(),
            mapping_views == expanded_mapping_record_views_spec(mappings@),
            record_views
                == crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            initial_entries == expanded_mapping_entry_views_spec(old(entries)@),
            entry_limit == merge_expansion_effective_limit_spec(
                limits@.max_expanded_mapping_entries,
                MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
            ),
            top_expected == expected,
            top_expected == append_mapping_source_spec(
                source_mapping_node_index,
                destination_start as nat,
                MergeExpansionBuildView {
                    mappings: mapping_views,
                    entries: initial_entries,
                    merge_source_count,
                },
                record_views,
                node_views,
                limits@,
            ),
            expected == append_mapping_source_entries_tail_spec(
                source_index as nat,
                source_snapshot_end as nat,
                destination_start as nat,
                (source_snapshot_end - source_index) as nat,
                MergeExpansionBuildView {
                    mappings: mapping_views,
                    entries: expanded_mapping_entry_views_spec(entries@),
                    merge_source_count,
                },
                record_views,
                node_views,
                limits@,
            ),
        decreases source_snapshot_end - source_index,
    {
        let source_entry = entries[source_index];
        let key_node_index = source_entry.key_node_index();
        let value_node_index = source_entry.value_node_index();
        proof {
            reveal(expanded_mapping_entry_views_spec);
            assert(expanded_mapping_entry_views_spec(entries@)[source_index as int]
                == entries[source_index as int]@);
        }
        let present = match entry_key_already_present(
            key_node_index,
            destination_start,
            entries.as_slice(),
            records,
        ) {
            Ok(present) => present,
            Err(error) => {
                proof {
                    reveal(append_mapping_source_entries_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if present {
            proof {
                reveal(append_mapping_source_entries_tail_spec);
                assert(expected == append_mapping_source_entries_tail_spec(
                    (source_index + 1) as nat,
                    source_snapshot_end as nat,
                    destination_start as nat,
                    (source_snapshot_end - source_index - 1) as nat,
                    MergeExpansionBuildView {
                        mappings: mapping_views,
                        entries: expanded_mapping_entry_views_spec(entries@),
                        merge_source_count,
                    },
                    record_views,
                    node_views,
                    limits@,
                ));
            }
            source_index += 1;
            continue;
        }
        if key_node_index >= nodes.len() as u64 || value_node_index >= nodes.len() as u64 {
            proof {
                reveal(append_mapping_source_entries_tail_spec);
                assert(expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset: 0,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0),
            );
        }
        let key_byte = nodes[key_node_index as usize].byte_start();
        let inherited = ExpandedMappingEntry::inherited_from(&source_entry);
        let ghost before_charge = expanded_mapping_entry_views_spec(entries@);
        proof {
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            assert(node_views[key_node_index as int] == nodes[key_node_index as int]@);
            assert(node_views[key_node_index as int].byte_start == key_byte);
        }
        match charge_and_push_entry(inherited, key_byte, entries, entry_limit) {
            Ok(()) => {},
            Err(error) => {
                proof {
                    reveal(append_mapping_source_entries_tail_spec);
                    reveal(admit_expanded_entry_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        }
        proof {
            reveal(append_mapping_source_entries_tail_spec);
            reveal(admit_expanded_entry_spec);
            assert(admit_expanded_entry_limit_spec(inherited@, key_byte, before_charge, entry_limit)
                == Ok(expanded_mapping_entry_views_spec(entries@)));
            assert(expected == append_mapping_source_entries_tail_spec(
                (source_index + 1) as nat,
                source_snapshot_end as nat,
                destination_start as nat,
                (source_snapshot_end - source_index - 1) as nat,
                MergeExpansionBuildView {
                    mappings: mapping_views,
                    entries: expanded_mapping_entry_views_spec(entries@),
                    merge_source_count,
                },
                record_views,
                node_views,
                limits@,
            ));
        }
        source_index += 1;
    }
    proof {
        reveal(append_mapping_source_entries_tail_spec);
        assert(expected == Ok(
            MergeExpansionBuildView {
                mappings: mapping_views,
                entries: expanded_mapping_entry_views_spec(entries@),
                merge_source_count,
            },
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Mirrors the complete pure explicit-edge transition.
fn append_explicit_mapping_edges(
    mapping_node_index: u64,
    edge_start: usize,
    edge_end: usize,
    mappings: &[ExpandedMappingRecord],
    entries: &mut Vec<ExpandedMappingEntry>,
    merge_source_count: u64,
    node_slots: &[SemanticNodeSlot],
    scalars: &[ResolvedScalar],
    nodes: &[SemanticTopologyNode],
    mapping_edges: &[crate::resolve_topology::SemanticMappingEdge],
    limits: MergeExpansionLimits,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        append_explicit_mapping_edges_tail_spec(
            mapping_node_index,
            edge_start as nat,
            edge_end as nat,
            if edge_start <= edge_end {
                (edge_end - edge_start) as nat
            } else {
                0nat
            },
            MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(mappings@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count,
            },
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(final(entries)@),
                    merge_source_count,
                },
            ),
            Err(error) => Err(error@),
        },
{
    // Retain the authenticated build prefix in the executable signature; the
    // helper's pure model consumes both values even though proof erasure does not.
    let _ = mappings;
    let _ = merge_source_count;
    let ghost mapping_views = expanded_mapping_record_views_spec(mappings@);
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost scalar_views = crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost edge_views = crate::resolve_topology::semantic_mapping_edge_views_spec(
        mapping_edges@,
    );
    let ghost initial_entries = expanded_mapping_entry_views_spec(entries@);
    let ghost expected = append_explicit_mapping_edges_tail_spec(
        mapping_node_index,
        edge_start as nat,
        edge_end as nat,
        if edge_start <= edge_end {
            (edge_end - edge_start) as nat
        } else {
            0nat
        },
        MergeExpansionBuildView {
            mappings: mapping_views,
            entries: initial_entries,
            merge_source_count,
        },
        slot_views,
        scalar_views,
        node_views,
        edge_views,
        limits@,
    );
    if edge_start > edge_end || edge_end > mapping_edges.len() {
        proof {
            reveal(append_explicit_mapping_edges_tail_spec);
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let entry_limit = effective_limit(
        limits.max_expanded_mapping_entries(),
        MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
    );
    let mut edge_index = edge_start;
    while edge_index < edge_end
        invariant
            edge_start <= edge_index,
            edge_index <= edge_end,
            edge_end <= mapping_edges.len(),
            mapping_views == expanded_mapping_record_views_spec(mappings@),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            scalar_views == crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            edge_views == crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            initial_entries == expanded_mapping_entry_views_spec(old(entries)@),
            entry_limit == merge_expansion_effective_limit_spec(
                limits@.max_expanded_mapping_entries,
                MAX_PROFILE1_EXPANDED_MAPPING_ENTRIES,
            ),
            append_explicit_mapping_edges_tail_spec(
                mapping_node_index,
                edge_start as nat,
                edge_end as nat,
                (edge_end - edge_start) as nat,
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(old(entries)@),
                    merge_source_count,
                },
                crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
                crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
                crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
                crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
                limits@,
            ) == expected,
            expected == append_explicit_mapping_edges_tail_spec(
                mapping_node_index,
                edge_index as nat,
                edge_end as nat,
                (edge_end - edge_index) as nat,
                MergeExpansionBuildView {
                    mappings: mapping_views,
                    entries: expanded_mapping_entry_views_spec(entries@),
                    merge_source_count,
                },
                slot_views,
                scalar_views,
                node_views,
                edge_views,
                limits@,
            ),
        decreases edge_end - edge_index,
    {
        let edge = &mapping_edges[edge_index];
        let key = edge.key_node_index();
        let value = edge.value_node_index();
        proof {
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            assert(edge_views[edge_index as int] == mapping_edges[edge_index as int]@);
        }
        let is_merge = match node_is_merge_key(key, node_slots, scalars) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    reveal(append_explicit_mapping_edges_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if is_merge {
            proof {
                reveal(append_explicit_mapping_edges_tail_spec);
                assert(expected == append_explicit_mapping_edges_tail_spec(
                    mapping_node_index,
                    (edge_index + 1) as nat,
                    edge_end as nat,
                    (edge_end - edge_index - 1) as nat,
                    MergeExpansionBuildView {
                        mappings: mapping_views,
                        entries: expanded_mapping_entry_views_spec(entries@),
                        merge_source_count,
                    },
                    slot_views,
                    scalar_views,
                    node_views,
                    edge_views,
                    limits@,
                ));
            }
            edge_index += 1;
            continue;
        }
        if key >= nodes.len() as u64 || value >= nodes.len() as u64 {
            let byte_offset = if mapping_node_index < nodes.len() as u64 {
                nodes[mapping_node_index as usize].byte_start()
            } else {
                0
            };
            proof {
                reveal(append_explicit_mapping_edges_tail_spec);
                assert(expected == Err(
                    MergeExpansionErrorView {
                        kind: MergeExpansionErrorKind::InternalInvariantViolation,
                        byte_offset,
                    },
                ));
            }
            return Err(
                MergeExpansionError::at(
                    MergeExpansionErrorKind::InternalInvariantViolation,
                    byte_offset,
                ),
            );
        }
        let key_byte = nodes[key as usize].byte_start();
        let entry = ExpandedMappingEntry::explicit(
            key,
            value,
            mapping_node_index,
            edge_index as u64,
        );
        let ghost before = expanded_mapping_entry_views_spec(entries@);
        proof {
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            assert(node_views[key as int] == nodes[key as int]@);
        }
        match charge_and_push_entry(entry, key_byte, entries, entry_limit) {
            Ok(()) => {},
            Err(error) => {
                proof {
                    reveal(append_explicit_mapping_edges_tail_spec);
                    reveal(admit_expanded_entry_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        }
        proof {
            reveal(append_explicit_mapping_edges_tail_spec);
            reveal(admit_expanded_entry_spec);
            assert(admit_expanded_entry_limit_spec(entry@, key_byte, before, entry_limit) == Ok(
                expanded_mapping_entry_views_spec(entries@),
            ));
        }
        edge_index += 1;
    }
    proof {
        reveal(append_explicit_mapping_edges_tail_spec);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Mirrors the complete pure merge-source transition.
fn append_merge_sources(
    sources: &[u64],
    destination_start: usize,
    mappings: &[ExpandedMappingRecord],
    entries: &mut Vec<ExpandedMappingEntry>,
    merge_source_count: &mut u64,
    records: &[CanonicalStructuralKeyRecord],
    nodes: &[SemanticTopologyNode],
    limits: MergeExpansionLimits,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        append_merge_sources_tail_spec(
            0,
            sources@.len() as nat,
            sources@,
            destination_start as nat,
            MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(mappings@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count: *old(merge_source_count),
            },
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(final(entries)@),
                    merge_source_count: *final(merge_source_count),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost mapping_views = expanded_mapping_record_views_spec(mappings@);
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost initial_entries = expanded_mapping_entry_views_spec(entries@);
    let ghost initial_count = *merge_source_count;
    let ghost expected = append_merge_sources_tail_spec(
        0,
        sources@.len() as nat,
        sources@,
        destination_start as nat,
        MergeExpansionBuildView {
            mappings: mapping_views,
            entries: initial_entries,
            merge_source_count: initial_count,
        },
        record_views,
        node_views,
        limits@,
    );
    let source_limit = effective_limit(limits.max_merge_sources(), MAX_PROFILE1_MERGE_SOURCES);
    let mut source_index = 0usize;
    while source_index < sources.len()
        invariant
            source_index <= sources.len(),
            mapping_views == expanded_mapping_record_views_spec(mappings@),
            record_views
                == crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            initial_entries == expanded_mapping_entry_views_spec(old(entries)@),
            initial_count == *old(merge_source_count),
            source_limit == merge_expansion_effective_limit_spec(
                limits@.max_merge_sources,
                MAX_PROFILE1_MERGE_SOURCES,
            ),
            append_merge_sources_tail_spec(
                0,
                sources@.len() as nat,
                sources@,
                destination_start as nat,
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(old(entries)@),
                    merge_source_count: *old(merge_source_count),
                },
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
                crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
                limits@,
            ) == expected,
            expected == append_merge_sources_tail_spec(
                source_index as nat,
                (sources.len() - source_index) as nat,
                sources@,
                destination_start as nat,
                MergeExpansionBuildView {
                    mappings: mapping_views,
                    entries: expanded_mapping_entry_views_spec(entries@),
                    merge_source_count: *merge_source_count,
                },
                record_views,
                node_views,
                limits@,
            ),
        decreases sources.len() - source_index,
    {
        let source = sources[source_index];
        if source >= nodes.len() as u64 {
            proof {
                reveal(append_merge_sources_tail_spec);
            }
            return Err(
                MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0),
            );
        }
        let source_byte = nodes[source as usize].byte_start();
        if *merge_source_count >= source_limit {
            proof {
                reveal(append_merge_sources_tail_spec);
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                assert(node_views[source as int] == nodes[source as int]@);
            }
            return Err(
                MergeExpansionError::at(
                    MergeExpansionErrorKind::MergeSourceLimitExceeded,
                    source_byte,
                ),
            );
        }
        *merge_source_count += 1;
        match append_mapping_source(
            source,
            destination_start,
            *merge_source_count,
            mappings,
            entries,
            records,
            nodes,
            limits,
        ) {
            Ok(()) => {},
            Err(error) => {
                proof {
                    reveal(append_merge_sources_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        }
        proof {
            reveal(append_merge_sources_tail_spec);
        }
        source_index += 1;
    }
    proof {
        reveal(append_merge_sources_tail_spec);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Keeps every proof-relevant graph view independently named.
fn append_merge_mapping_edges(
    edge_start: usize,
    edge_end: usize,
    destination_start: usize,
    mappings: &[ExpandedMappingRecord],
    entries: &mut Vec<ExpandedMappingEntry>,
    merge_source_count: &mut u64,
    node_slots: &[SemanticNodeSlot],
    scalars: &[ResolvedScalar],
    nodes: &[SemanticTopologyNode],
    sequence_edges: &[crate::resolve_topology::SemanticSequenceEdge],
    mapping_edges: &[crate::resolve_topology::SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    limits: MergeExpansionLimits,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        append_merge_mapping_edges_tail_spec(
            edge_start as nat,
            edge_end as nat,
            if edge_start <= edge_end {
                (edge_end - edge_start) as nat
            } else {
                0nat
            },
            destination_start as nat,
            MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(mappings@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count: *old(merge_source_count),
            },
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(final(entries)@),
                    merge_source_count: *final(merge_source_count),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost mapping_views = expanded_mapping_record_views_spec(mappings@);
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost scalar_views = crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost edge_views = crate::resolve_topology::semantic_mapping_edge_views_spec(
        mapping_edges@,
    );
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    let ghost initial_entries = expanded_mapping_entry_views_spec(entries@);
    let ghost initial_count = *merge_source_count;
    let ghost expected = append_merge_mapping_edges_tail_spec(
        edge_start as nat,
        edge_end as nat,
        if edge_start <= edge_end {
            (edge_end - edge_start) as nat
        } else {
            0nat
        },
        destination_start as nat,
        MergeExpansionBuildView {
            mappings: mapping_views,
            entries: initial_entries,
            merge_source_count: initial_count,
        },
        slot_views,
        scalar_views,
        node_views,
        sequence_views,
        edge_views,
        record_views,
        limits@,
    );
    if edge_start > edge_end || edge_end > mapping_edges.len() {
        proof {
            reveal(append_merge_mapping_edges_tail_spec);
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let mut edge_index = edge_start;
    while edge_index < edge_end
        invariant
            edge_start <= edge_index,
            edge_index <= edge_end,
            edge_end <= mapping_edges.len(),
            mapping_views == expanded_mapping_record_views_spec(mappings@),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            scalar_views == crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            sequence_views == crate::resolve_topology::semantic_sequence_edge_views_spec(
                sequence_edges@,
            ),
            edge_views == crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            record_views
                == crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@),
            initial_entries == expanded_mapping_entry_views_spec(old(entries)@),
            initial_count == *old(merge_source_count),
            append_merge_mapping_edges_tail_spec(
                edge_start as nat,
                edge_end as nat,
                (edge_end - edge_start) as nat,
                destination_start as nat,
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(old(entries)@),
                    merge_source_count: *old(merge_source_count),
                },
                crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
                crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
                crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
                crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
                crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
                limits@,
            ) == expected,
            expected == append_merge_mapping_edges_tail_spec(
                edge_index as nat,
                edge_end as nat,
                (edge_end - edge_index) as nat,
                destination_start as nat,
                MergeExpansionBuildView {
                    mappings: mapping_views,
                    entries: expanded_mapping_entry_views_spec(entries@),
                    merge_source_count: *merge_source_count,
                },
                slot_views,
                scalar_views,
                node_views,
                sequence_views,
                edge_views,
                record_views,
                limits@,
            ),
        decreases edge_end - edge_index,
    {
        let edge = &mapping_edges[edge_index];
        let key = edge.key_node_index();
        let value = edge.value_node_index();
        proof {
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            assert(edge_views[edge_index as int] == mapping_edges[edge_index as int]@);
        }
        let is_merge = match node_is_merge_key(key, node_slots, scalars) {
            Ok(value) => value,
            Err(error) => {
                proof {
                    reveal(append_merge_mapping_edges_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if is_merge {
            let sources = match merge_sources_for_value(value, node_slots, nodes, sequence_edges) {
                Ok(sources) => sources,
                Err(error) => {
                    proof {
                        reveal(append_merge_mapping_edges_tail_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            match append_merge_sources(
                sources.as_slice(),
                destination_start,
                mappings,
                entries,
                merge_source_count,
                records,
                nodes,
                limits,
            ) {
                Ok(()) => {},
                Err(error) => {
                    proof {
                        reveal(append_merge_mapping_edges_tail_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            }
        }
        proof {
            reveal(append_merge_mapping_edges_tail_spec);
        }
        edge_index += 1;
    }
    proof {
        reveal(append_merge_mapping_edges_tail_spec);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Every authenticated producer remains an explicit input.
fn expand_one_mapping(
    node_index: usize,
    mappings: &mut Vec<ExpandedMappingRecord>,
    entries: &mut Vec<ExpandedMappingEntry>,
    merge_source_count: &mut u64,
    node_slots: &[SemanticNodeSlot],
    scalars: &[ResolvedScalar],
    nodes: &[SemanticTopologyNode],
    sequence_edges: &[crate::resolve_topology::SemanticSequenceEdge],
    mapping_edges: &[crate::resolve_topology::SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    limits: MergeExpansionLimits,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        expand_one_mapping_spec(
            node_index as nat,
            MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(old(mappings)@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count: *old(merge_source_count),
            },
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(final(mappings)@),
                    entries: expanded_mapping_entry_views_spec(final(entries)@),
                    merge_source_count: *final(merge_source_count),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost scalar_views = crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost edge_views = crate::resolve_topology::semantic_mapping_edge_views_spec(
        mapping_edges@,
    );
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    let ghost initial_build = MergeExpansionBuildView {
        mappings: expanded_mapping_record_views_spec(mappings@),
        entries: expanded_mapping_entry_views_spec(entries@),
        merge_source_count: *merge_source_count,
    };
    let ghost expected = expand_one_mapping_spec(
        node_index as nat,
        initial_build,
        slot_views,
        scalar_views,
        node_views,
        sequence_views,
        edge_views,
        record_views,
        limits@,
    );
    if node_index >= nodes.len() {
        proof {
            reveal(expand_one_mapping_spec);
        }
        return Err(MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, 0));
    }
    let node = &nodes[node_index];
    let node_byte = node.byte_start();
    let edge_start_u64 = node.edge_start();
    let edge_end_u64 = node.edge_end();
    proof {
        reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        assert(node_views[node_index as int] == nodes[node_index as int]@);
        reveal(expand_one_mapping_spec);
    }
    let mapping_limit = effective_limit(limits.max_mappings(), MAX_PROFILE1_MERGE_MAPPING_RECORDS);
    if mappings.len() as u64 >= mapping_limit {
        proof {
            assert(expected == Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::MappingLimitExceeded,
                    byte_offset: node_byte,
                },
            ));
        }
        return Err(
            MergeExpansionError::at(MergeExpansionErrorKind::MappingLimitExceeded, node_byte),
        );
    }
    if edge_start_u64 > edge_end_u64 || edge_end_u64 > mapping_edges.len() as u64 {
        proof {
            assert(expected == Err(
                MergeExpansionErrorView {
                    kind: MergeExpansionErrorKind::InternalInvariantViolation,
                    byte_offset: node_byte,
                },
            ));
        }
        return Err(
            MergeExpansionError::at(MergeExpansionErrorKind::InternalInvariantViolation, node_byte),
        );
    }
    let edge_start = edge_start_u64 as usize;
    let edge_end = edge_end_u64 as usize;
    let destination_start = entries.len();
    match append_explicit_mapping_edges(
        node_index as u64,
        edge_start,
        edge_end,
        mappings.as_slice(),
        entries,
        *merge_source_count,
        node_slots,
        scalars,
        nodes,
        mapping_edges,
        limits,
    ) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    match append_merge_mapping_edges(
        edge_start,
        edge_end,
        destination_start,
        mappings.as_slice(),
        entries,
        merge_source_count,
        node_slots,
        scalars,
        nodes,
        sequence_edges,
        mapping_edges,
        records,
        limits,
    ) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    let ghost mappings_before = mappings@;
    let record = ExpandedMappingRecord::new(
        node_index as u64,
        destination_start as u64,
        entries.len() as u64,
    );
    let ghost record_view = record@;
    mappings.push(record);
    proof {
        reveal(expanded_mapping_record_views_spec);
        assert(expanded_mapping_record_views_spec(mappings@) == expanded_mapping_record_views_spec(
            mappings_before,
        ).push(record_view));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]  // Mirrors the total pure node-expansion state.
fn expand_merge_nodes(
    mappings: &mut Vec<ExpandedMappingRecord>,
    entries: &mut Vec<ExpandedMappingEntry>,
    merge_source_count: &mut u64,
    node_slots: &[SemanticNodeSlot],
    scalars: &[ResolvedScalar],
    nodes: &[SemanticTopologyNode],
    sequence_edges: &[crate::resolve_topology::SemanticSequenceEdge],
    mapping_edges: &[crate::resolve_topology::SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    limits: MergeExpansionLimits,
) -> (result: Result<(), MergeExpansionError>)
    ensures
        expand_merge_nodes_tail_spec(
            0,
            nodes@.len() as nat,
            MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(old(mappings)@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count: *old(merge_source_count),
            },
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            limits@,
        ) == match result {
            Ok(()) => Ok(
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(final(mappings)@),
                    entries: expanded_mapping_entry_views_spec(final(entries)@),
                    merge_source_count: *final(merge_source_count),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost scalar_views = crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost edge_views = crate::resolve_topology::semantic_mapping_edge_views_spec(
        mapping_edges@,
    );
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    let ghost initial_build = MergeExpansionBuildView {
        mappings: expanded_mapping_record_views_spec(mappings@),
        entries: expanded_mapping_entry_views_spec(entries@),
        merge_source_count: *merge_source_count,
    };
    let ghost expected = expand_merge_nodes_tail_spec(
        0,
        nodes@.len() as nat,
        initial_build,
        slot_views,
        scalar_views,
        node_views,
        sequence_views,
        edge_views,
        record_views,
        limits@,
    );
    let mut node_index = 0usize;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            scalar_views == crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            sequence_views == crate::resolve_topology::semantic_sequence_edge_views_spec(
                sequence_edges@,
            ),
            edge_views == crate::resolve_topology::semantic_mapping_edge_views_spec(mapping_edges@),
            record_views
                == crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@),
            initial_build == (MergeExpansionBuildView {
                mappings: expanded_mapping_record_views_spec(old(mappings)@),
                entries: expanded_mapping_entry_views_spec(old(entries)@),
                merge_source_count: *old(merge_source_count),
            }),
            expand_merge_nodes_tail_spec(
                0,
                nodes@.len() as nat,
                initial_build,
                slot_views,
                scalar_views,
                node_views,
                sequence_views,
                edge_views,
                record_views,
                limits@,
            ) == expected,
            expected == expand_merge_nodes_tail_spec(
                node_index as nat,
                (nodes.len() - node_index) as nat,
                MergeExpansionBuildView {
                    mappings: expanded_mapping_record_views_spec(mappings@),
                    entries: expanded_mapping_entry_views_spec(entries@),
                    merge_source_count: *merge_source_count,
                },
                slot_views,
                scalar_views,
                node_views,
                sequence_views,
                edge_views,
                record_views,
                limits@,
            ),
        decreases nodes.len() - node_index,
    {
        let kind = nodes[node_index].kind();
        proof {
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            assert(node_views[node_index as int] == nodes[node_index as int]@);
        }
        if kind == CstNodeKind::Mapping {
            match expand_one_mapping(
                node_index,
                mappings,
                entries,
                merge_source_count,
                node_slots,
                scalars,
                nodes,
                sequence_edges,
                mapping_edges,
                records,
                limits,
            ) {
                Ok(()) => {},
                Err(error) => {
                    proof {
                        reveal(expand_merge_nodes_tail_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            }
        }
        proof {
            reveal(expand_merge_nodes_tail_spec);
        }
        node_index += 1;
    }
    proof {
        reveal(expand_merge_nodes_tail_spec);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum ExpandedReferenceFrame {
    Node(u64),
    Sequence { next_edge: u64, edge_end: u64 },
    Mapping { next_entry: u64, entry_end: u64, visit_value: bool },
}

impl View for ExpandedReferenceFrame {
    type V = ExpandedReferenceFrameView;

    closed spec fn view(&self) -> ExpandedReferenceFrameView {
        match self {
            ExpandedReferenceFrame::Node(index) => ExpandedReferenceFrameView::Node(*index),
            ExpandedReferenceFrame::Sequence { next_edge, edge_end } => {
                ExpandedReferenceFrameView::Sequence { next_edge: *next_edge, edge_end: *edge_end }
            },
            ExpandedReferenceFrame::Mapping { next_entry, entry_end, visit_value } => {
                ExpandedReferenceFrameView::Mapping {
                    next_entry: *next_entry,
                    entry_end: *entry_end,
                    visit_value: *visit_value,
                }
            },
        }
    }
}

closed spec fn expanded_reference_frame_views_spec(frames: Seq<ExpandedReferenceFrame>) -> Seq<
    ExpandedReferenceFrameView,
> {
    Seq::new(frames.len(), |index: int| frames[index]@)
}

fn push_reference_frame(stack: &mut Vec<ExpandedReferenceFrame>, frame: ExpandedReferenceFrame)
    ensures
        expanded_reference_frame_views_spec(final(stack)@) == expanded_reference_frame_views_spec(
            old(stack)@,
        ).push(frame@),
{
    let ghost before = stack@;
    stack.push(frame);
    proof {
        reveal(expanded_reference_frame_views_spec);
        assert(expanded_reference_frame_views_spec(stack@) == expanded_reference_frame_views_spec(
            before,
        ).push(frame@));
    }
}

#[allow(clippy::too_many_arguments)]  // Keeps traversal inputs aligned with the exact pure model.
fn count_expanded_reference_root(
    root_node_index: u64,
    starting_references: u64,
    source_len_bytes: u64,
    nodes: &[SemanticTopologyNode],
    sequence_edges: &[crate::resolve_topology::SemanticSequenceEdge],
    node_slots: &[SemanticNodeSlot],
    mappings: &[ExpandedMappingRecord],
    entries: &[ExpandedMappingEntry],
    reference_limit: u64,
) -> (result: Result<u64, MergeExpansionError>)
    requires
        reference_limit <= MAX_PROFILE1_EXPANDED_REFERENCES,
        starting_references <= reference_limit,
    ensures
        count_expanded_reference_work_tail_spec(
            Seq::empty().push(ExpandedReferenceFrameView::Node(root_node_index)),
            ((reference_limit as nat + 1) * 3 + 3) as nat,
            starting_references,
            reference_limit,
            source_len_bytes,
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_sequence_edge_views_spec(sequence_edges@),
            expanded_mapping_record_views_spec(mappings@),
            expanded_mapping_entry_views_spec(entries@),
        ) == match result {
            Ok(references) => Ok(references),
            Err(error) => Err(error@),
        },
        match result {
            Ok(references) => references <= reference_limit,
            Err(_) => true,
        },
{
    let fuel_base = match reference_limit.checked_add(1) {
        Some(value) => value,
        None => return Err(
            MergeExpansionError::at(
                MergeExpansionErrorKind::InternalInvariantViolation,
                source_len_bytes,
            ),
        ),
    };
    let fuel_triple = match fuel_base.checked_mul(3) {
        Some(value) => value,
        None => return Err(
            MergeExpansionError::at(
                MergeExpansionErrorKind::InternalInvariantViolation,
                source_len_bytes,
            ),
        ),
    };
    let work_limit = match fuel_triple.checked_add(3) {
        Some(value) => value,
        None => return Err(
            MergeExpansionError::at(
                MergeExpansionErrorKind::InternalInvariantViolation,
                source_len_bytes,
            ),
        ),
    };
    let mut work_fuel = work_limit;
    let mut references = starting_references;
    let mut stack: Vec<ExpandedReferenceFrame> = Vec::new();
    push_reference_frame(&mut stack, ExpandedReferenceFrame::Node(root_node_index));
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost mapping_views = expanded_mapping_record_views_spec(mappings@);
    let ghost entry_views = expanded_mapping_entry_views_spec(entries@);
    let ghost top_expected = count_expanded_reference_work_tail_spec(
        Seq::empty().push(ExpandedReferenceFrameView::Node(root_node_index)),
        ((reference_limit as nat + 1) * 3 + 3) as nat,
        starting_references,
        reference_limit,
        source_len_bytes,
        slot_views,
        node_views,
        sequence_views,
        mapping_views,
        entry_views,
    );
    proof {
        assert(work_limit as nat == (reference_limit as nat + 1) * 3 + 3);
        reveal(expanded_reference_frame_views_spec);
        assert(expanded_reference_frame_views_spec(stack@) == Seq::empty().push(
            ExpandedReferenceFrameView::Node(root_node_index),
        ));
    }
    while !stack.is_empty()
        invariant
            references <= reference_limit,
            work_fuel <= work_limit,
            work_limit as nat == (reference_limit as nat + 1) * 3 + 3,
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            sequence_views == crate::resolve_topology::semantic_sequence_edge_views_spec(
                sequence_edges@,
            ),
            mapping_views == expanded_mapping_record_views_spec(mappings@),
            entry_views == expanded_mapping_entry_views_spec(entries@),
            count_expanded_reference_work_tail_spec(
                Seq::empty().push(ExpandedReferenceFrameView::Node(root_node_index)),
                ((reference_limit as nat + 1) * 3 + 3) as nat,
                starting_references,
                reference_limit,
                source_len_bytes,
                slot_views,
                node_views,
                sequence_views,
                mapping_views,
                entry_views,
            ) == top_expected,
            top_expected == count_expanded_reference_work_tail_spec(
                expanded_reference_frame_views_spec(stack@),
                work_fuel as nat,
                references,
                reference_limit,
                source_len_bytes,
                slot_views,
                node_views,
                sequence_views,
                mapping_views,
                entry_views,
            ),
        decreases work_fuel,
    {
        if work_fuel == 0 {
            proof {
                reveal(count_expanded_reference_work_tail_spec);
            }
            return Err(
                MergeExpansionError::at(
                    MergeExpansionErrorKind::InternalInvariantViolation,
                    source_len_bytes,
                ),
            );
        }
        let ghost step_fuel = work_fuel;
        let ghost step_references = references;
        let ghost stack_before = expanded_reference_frame_views_spec(stack@);
        proof {
            assert(top_expected == count_expanded_reference_work_tail_spec(
                stack_before,
                step_fuel as nat,
                step_references,
                reference_limit,
                source_len_bytes,
                slot_views,
                node_views,
                sequence_views,
                mapping_views,
                entry_views,
            ));
        }
        work_fuel -= 1;
        let frame = match stack.pop() {
            Some(frame) => frame,
            None => return Err(
                MergeExpansionError::at(
                    MergeExpansionErrorKind::InternalInvariantViolation,
                    source_len_bytes,
                ),
            ),
        };
        let ghost rest_after_pop = expanded_reference_frame_views_spec(stack@);
        proof {
            reveal(expanded_reference_frame_views_spec);
            assert(stack_before == rest_after_pop.push(frame@));
            assert(stack_before.last() == frame@);
            assert(stack_before.subrange(0, stack_before.len() as int - 1) == rest_after_pop);
        }
        match frame {
            ExpandedReferenceFrame::Node(node_index) => {
                if node_index >= node_slots.len() as u64 || node_index >= nodes.len() as u64 {
                    return Err(
                        MergeExpansionError::at(
                            MergeExpansionErrorKind::InternalInvariantViolation,
                            source_len_bytes,
                        ),
                    );
                }
                if references >= reference_limit {
                    return Err(
                        MergeExpansionError::at(
                            MergeExpansionErrorKind::ExpandedReferenceLimitExceeded,
                            nodes[node_index as usize].byte_start(),
                        ),
                    );
                }
                let ghost references_before_increment = references;
                references += 1;
                proof {
                    assert(references == (references_before_increment + 1) as u64);
                }
                let node_kind = node_slots[node_index as usize].kind();
                proof {
                    reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
                    reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                    assert(slot_views[node_index as int] == node_slots[node_index as int]@);
                    assert(node_views[node_index as int] == nodes[node_index as int]@);
                }
                match node_kind {
                    SemanticNodeKind::Scalar => {
                        proof {
                            assert(frame@ == ExpandedReferenceFrameView::Node(node_index));
                            assert(stack_before.len() > 0);
                            assert(step_fuel > 0);
                            assert(stack_before == rest_after_pop.push(
                                ExpandedReferenceFrameView::Node(node_index),
                            ));
                            assert(work_fuel + 1 == step_fuel);
                            assert(references == (step_references + 1) as u64);
                            assert(step_references < reference_limit);
                            assert(slot_views[node_index as int].kind == SemanticNodeKind::Scalar);
                            reveal(count_expanded_reference_work_tail_spec);
                            assert(top_expected == count_expanded_reference_work_tail_spec(
                                expanded_reference_frame_views_spec(stack@),
                                work_fuel as nat,
                                references,
                                reference_limit,
                                source_len_bytes,
                                slot_views,
                                node_views,
                                sequence_views,
                                mapping_views,
                                entry_views,
                            ));
                        }
                    },
                    SemanticNodeKind::Alias => {
                        match node_slots[node_index as usize].alias_target_node_index() {
                            Some(target) => push_reference_frame(
                                &mut stack,
                                ExpandedReferenceFrame::Node(target),
                            ),
                            None => return Err(
                                MergeExpansionError::at(
                                    MergeExpansionErrorKind::InternalInvariantViolation,
                                    nodes[node_index as usize].byte_start(),
                                ),
                            ),
                        }
                    },
                    SemanticNodeKind::Sequence => {
                        let node = &nodes[node_index as usize];
                        let edge_start = node.edge_start();
                        let edge_end = node.edge_end();
                        if edge_start > edge_end || edge_end > sequence_edges.len() as u64 {
                            return Err(
                                MergeExpansionError::at(
                                    MergeExpansionErrorKind::InternalInvariantViolation,
                                    node.byte_start(),
                                ),
                            );
                        }
                        push_reference_frame(
                            &mut stack,
                            ExpandedReferenceFrame::Sequence { next_edge: edge_start, edge_end },
                        );
                    },
                    SemanticNodeKind::Mapping => {
                        let record_index = match mapping_record_index(node_index, mappings) {
                            Some(index) => index,
                            None => return Err(
                                MergeExpansionError::at(
                                    MergeExpansionErrorKind::InternalInvariantViolation,
                                    nodes[node_index as usize].byte_start(),
                                ),
                            ),
                        };
                        let start = mappings[record_index].entry_start();
                        let end = mappings[record_index].entry_end();
                        proof {
                            reveal(expanded_mapping_record_views_spec);
                            assert(mapping_views[record_index as int]
                                == mappings[record_index as int]@);
                        }
                        if start > end || end > entries.len() as u64 {
                            return Err(
                                MergeExpansionError::at(
                                    MergeExpansionErrorKind::InternalInvariantViolation,
                                    nodes[node_index as usize].byte_start(),
                                ),
                            );
                        }
                        push_reference_frame(
                            &mut stack,
                            ExpandedReferenceFrame::Mapping {
                                next_entry: start,
                                entry_end: end,
                                visit_value: false,
                            },
                        );
                    },
                }
                proof {
                    reveal(count_expanded_reference_work_tail_spec);
                    reveal(expanded_reference_frame_views_spec);
                    assert(top_expected == count_expanded_reference_work_tail_spec(
                        expanded_reference_frame_views_spec(stack@),
                        work_fuel as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        slot_views,
                        node_views,
                        sequence_views,
                        mapping_views,
                        entry_views,
                    ));
                }
            },
            ExpandedReferenceFrame::Sequence { next_edge, edge_end } => {
                if next_edge < edge_end {
                    if edge_end > sequence_edges.len() as u64 {
                        return Err(
                            MergeExpansionError::at(
                                MergeExpansionErrorKind::InternalInvariantViolation,
                                source_len_bytes,
                            ),
                        );
                    }
                    push_reference_frame(
                        &mut stack,
                        ExpandedReferenceFrame::Sequence { next_edge: next_edge + 1, edge_end },
                    );
                    let child = sequence_edges[next_edge as usize].child_node_index();
                    proof {
                        reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
                        assert(sequence_views[next_edge as int]
                            == sequence_edges[next_edge as int]@);
                    }
                    push_reference_frame(&mut stack, ExpandedReferenceFrame::Node(child));
                }
                proof {
                    reveal(count_expanded_reference_work_tail_spec);
                    reveal(expanded_reference_frame_views_spec);
                    assert(top_expected == count_expanded_reference_work_tail_spec(
                        expanded_reference_frame_views_spec(stack@),
                        work_fuel as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        slot_views,
                        node_views,
                        sequence_views,
                        mapping_views,
                        entry_views,
                    ));
                }
            },
            ExpandedReferenceFrame::Mapping { next_entry, entry_end, visit_value } => {
                if next_entry < entry_end {
                    if entry_end > entries.len() as u64 {
                        return Err(
                            MergeExpansionError::at(
                                MergeExpansionErrorKind::InternalInvariantViolation,
                                source_len_bytes,
                            ),
                        );
                    }
                    let entry = &entries[next_entry as usize];
                    let key = entry.key_node_index();
                    let value = entry.value_node_index();
                    proof {
                        reveal(expanded_mapping_entry_views_spec);
                        assert(entry_views[next_entry as int] == entries[next_entry as int]@);
                    }
                    if visit_value {
                        push_reference_frame(
                            &mut stack,
                            ExpandedReferenceFrame::Mapping {
                                next_entry: next_entry + 1,
                                entry_end,
                                visit_value: false,
                            },
                        );
                        push_reference_frame(&mut stack, ExpandedReferenceFrame::Node(value));
                    } else {
                        push_reference_frame(
                            &mut stack,
                            ExpandedReferenceFrame::Mapping {
                                next_entry,
                                entry_end,
                                visit_value: true,
                            },
                        );
                        push_reference_frame(&mut stack, ExpandedReferenceFrame::Node(key));
                    }
                }
                proof {
                    assert(frame@ == ExpandedReferenceFrameView::Mapping {
                        next_entry,
                        entry_end,
                        visit_value,
                    });
                    assert(stack_before == rest_after_pop.push(frame@));
                    reveal(count_expanded_reference_work_tail_spec);
                    reveal(expanded_reference_frame_views_spec);
                    assert(top_expected == count_expanded_reference_work_tail_spec(
                        expanded_reference_frame_views_spec(stack@),
                        work_fuel as nat,
                        references,
                        reference_limit,
                        source_len_bytes,
                        slot_views,
                        node_views,
                        sequence_views,
                        mapping_views,
                        entry_views,
                    ));
                }
            },
        }
        proof {
            reveal(count_expanded_reference_work_tail_spec);
            reveal(expanded_reference_frame_views_spec);
        }
    }
    proof {
        reveal(count_expanded_reference_work_tail_spec);
        reveal(expanded_reference_frame_views_spec);
    }
    Ok(references)
}

fn count_expanded_references(
    topology: &crate::resolve_topology::SemanticTopologySource,
    node_slots: &[SemanticNodeSlot],
    mappings: &[ExpandedMappingRecord],
    entries: &[ExpandedMappingEntry],
    reference_limit: u64,
) -> (result: Result<u64, MergeExpansionError>)
    requires
        reference_limit <= MAX_PROFILE1_EXPANDED_REFERENCES,
    ensures
        count_expanded_reference_roots_tail_spec(
            0,
            topology@.document_roots.len() as nat,
            0,
            reference_limit,
            topology@.source_len_bytes,
            topology@.document_roots,
            crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            topology@.nodes,
            topology@.sequence_edges,
            expanded_mapping_record_views_spec(mappings@),
            expanded_mapping_entry_views_spec(entries@),
        ) == match result {
            Ok(references) => Ok(references),
            Err(error) => Err(error@),
        },
{
    let nodes = topology.nodes();
    let sequence_edges = topology.sequence_edges();
    let roots = topology.document_roots();
    let ghost root_views = crate::resolve_topology::semantic_document_root_views_spec(roots@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost mapping_views = expanded_mapping_record_views_spec(mappings@);
    let ghost entry_views = expanded_mapping_entry_views_spec(entries@);
    let ghost expected = count_expanded_reference_roots_tail_spec(
        0,
        roots@.len() as nat,
        0,
        reference_limit,
        topology@.source_len_bytes,
        root_views,
        slot_views,
        node_views,
        sequence_views,
        mapping_views,
        entry_views,
    );
    let mut references = 0u64;
    let mut root_index = 0usize;
    while root_index < roots.len()
        invariant
            root_index <= roots.len(),
            references <= reference_limit,
            reference_limit <= MAX_PROFILE1_EXPANDED_REFERENCES,
            root_views == crate::resolve_topology::semantic_document_root_views_spec(roots@),
            node_views == crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            sequence_views == crate::resolve_topology::semantic_sequence_edge_views_spec(
                sequence_edges@,
            ),
            slot_views == crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@),
            mapping_views == expanded_mapping_record_views_spec(mappings@),
            entry_views == expanded_mapping_entry_views_spec(entries@),
            topology@.document_roots == root_views,
            topology@.nodes == node_views,
            topology@.sequence_edges == sequence_views,
            count_expanded_reference_roots_tail_spec(
                0,
                topology@.document_roots.len() as nat,
                0,
                reference_limit,
                topology@.source_len_bytes,
                topology@.document_roots,
                slot_views,
                topology@.nodes,
                topology@.sequence_edges,
                mapping_views,
                entry_views,
            ) == expected,
            expected == count_expanded_reference_roots_tail_spec(
                root_index as nat,
                (roots.len() - root_index) as nat,
                references,
                reference_limit,
                topology@.source_len_bytes,
                root_views,
                slot_views,
                node_views,
                sequence_views,
                mapping_views,
                entry_views,
            ),
        decreases roots.len() - root_index,
    {
        let root_node_index = roots[root_index].node_index();
        proof {
            reveal(crate::resolve_topology::semantic_document_root_views_spec);
            assert(root_views[root_index as int] == roots[root_index as int]@);
        }
        references =
        match count_expanded_reference_root(
            root_node_index,
            references,
            topology.source_len_bytes(),
            nodes,
            sequence_edges,
            node_slots,
            mappings,
            entries,
            reference_limit,
        ) {
            Ok(references) => references,
            Err(error) => {
                proof {
                    reveal(count_expanded_reference_roots_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        proof {
            reveal(count_expanded_reference_roots_tail_spec);
        }
        root_index += 1;
    }
    proof {
        reveal(count_expanded_reference_roots_tail_spec);
    }
    Ok(references)
}

#[verifier::rlimit(80)]
pub fn expand_profile1_merge_keys(
    source: DuplicateFreeStructuralKeySource,
    limits: MergeExpansionLimits,
) -> (result: Result<ExpandedSemanticGraphSource, MergeExpansionError>)
    ensures
        expand_profile1_merge_keys_spec(source@, limits@) == match result {
            Ok(output) => Ok(output@),
            Err(error) => Err(error@),
        },
{
    let ghost input_view = source@;
    let ghost expected = expand_profile1_merge_keys_spec(input_view, limits@);
    match preflight_merge_shapes(&source) {
        Ok(()) => {},
        Err(error) => {
            proof {
                reveal(expand_profile1_merge_keys_spec);
            }
            return Err(error);
        },
    }

    let reference_limit = effective_limit(
        limits.max_expanded_references(),
        MAX_PROFILE1_EXPANDED_REFERENCES,
    );

    let structural = source.structural_keys();
    let table = structural.scalar_keys().graph().node_table();
    let topology = table.topology();
    let nodes = topology.nodes();
    let node_slots = table.nodes();
    let scalars = table.scalars().scalars();
    let mapping_edges = topology.mapping_edges();
    let sequence_edges = topology.sequence_edges();
    let records = structural.records();
    let ghost slot_views = crate::resolve_node_table::semantic_node_slot_views_spec(node_slots@);
    let ghost scalar_views = crate::resolve_scalar_table::semantic_scalar_views_spec(scalars@);
    let ghost node_views = crate::resolve_topology::semantic_topology_node_views_spec(nodes@);
    let ghost sequence_views = crate::resolve_topology::semantic_sequence_edge_views_spec(
        sequence_edges@,
    );
    let ghost edge_views = crate::resolve_topology::semantic_mapping_edge_views_spec(
        mapping_edges@,
    );
    let ghost record_views =
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
        records@,
    );
    let mut mappings: Vec<ExpandedMappingRecord> = Vec::new();
    let mut entries: Vec<ExpandedMappingEntry> = Vec::new();
    let mut merge_sources = 0u64;
    let ghost initial_build = MergeExpansionBuildView {
        mappings: expanded_mapping_record_views_spec(mappings@),
        entries: expanded_mapping_entry_views_spec(entries@),
        merge_source_count: merge_sources,
    };
    let ghost expansion_expected = expand_merge_nodes_tail_spec(
        0,
        nodes@.len() as nat,
        initial_build,
        slot_views,
        scalar_views,
        node_views,
        sequence_views,
        edge_views,
        record_views,
        limits@,
    );
    proof {
        reveal(expand_profile1_merge_keys_spec);
        reveal(expanded_mapping_record_views_spec);
        reveal(expanded_mapping_entry_views_spec);
        assert(initial_build == (MergeExpansionBuildView {
            mappings: Seq::empty(),
            entries: Seq::empty(),
            merge_source_count: 0,
        }));
        assert(input_view.structural_keys.scalar_keys.graph.node_table.nodes == slot_views);
        assert(input_view.structural_keys.scalar_keys.graph.node_table.scalars.scalars
            == scalar_views);
        assert(input_view.structural_keys.scalar_keys.graph.node_table.topology.nodes
            == node_views);
        assert(input_view.structural_keys.scalar_keys.graph.node_table.topology.sequence_edges
            == sequence_views);
        assert(input_view.structural_keys.scalar_keys.graph.node_table.topology.mapping_edges
            == edge_views);
        assert(input_view.structural_keys.records == record_views);
        assert(expected == match expansion_expected {
            Err(error) => Err(error),
            Ok(build) => finalize_merge_expansion_spec(input_view, limits@, build),
        });
    }
    match expand_merge_nodes(
        &mut mappings,
        &mut entries,
        &mut merge_sources,
        node_slots,
        scalars,
        nodes,
        sequence_edges,
        mapping_edges,
        records,
        limits,
    ) {
        Ok(()) => {},
        Err(error) => {
            proof {
                reveal(expand_profile1_merge_keys_spec);
                assert(expansion_expected == Err(error@));
                assert(expected == Err(error@));
            }
            return Err(error);
        },
    }
    let ghost build = MergeExpansionBuildView {
        mappings: expanded_mapping_record_views_spec(mappings@),
        entries: expanded_mapping_entry_views_spec(entries@),
        merge_source_count: merge_sources,
    };
    proof {
        assert(expansion_expected == Ok(build));
    }
    let ghost final_expected = finalize_merge_expansion_spec(input_view, limits@, build);
    proof {
        assert(expected == final_expected);
        assert(reference_limit == merge_expansion_effective_limit_spec(
            limits@.max_expanded_references,
            MAX_PROFILE1_EXPANDED_REFERENCES,
        ));
    }
    let ghost reference_expected = count_expanded_reference_roots_tail_spec(
        0,
        topology@.document_roots.len() as nat,
        0,
        reference_limit,
        topology@.source_len_bytes,
        topology@.document_roots,
        slot_views,
        topology@.nodes,
        topology@.sequence_edges,
        build.mappings,
        build.entries,
    );
    proof {
        reveal(finalize_merge_expansion_spec);
        assert(input_view.structural_keys.scalar_keys.graph.node_table == table@);
        assert(table@.topology == topology@);
        assert(table@.nodes == slot_views);
        assert(final_expected == match reference_expected {
            Err(error) => Err(error),
            Ok(expanded_reference_count) => Ok(
                ExpandedSemanticGraphSourceView {
                    profile_version: input_view.profile_version,
                    transformation_version: MERGE_EXPANSION_TRANSFORMATION_VERSION,
                    source_len_bytes: input_view.source_len_bytes,
                    input_node_count: input_view.input_node_count,
                    expanded_reference_count,
                    merge_source_count: build.merge_source_count,
                    input: input_view,
                    mappings: build.mappings,
                    entries: build.entries,
                },
            ),
        });
    }

    let references = match count_expanded_references(
        topology,
        node_slots,
        mappings.as_slice(),
        entries.as_slice(),
        reference_limit,
    ) {
        Ok(references) => references,
        Err(error) => {
            proof {
                reveal(expand_profile1_merge_keys_spec);
                reveal(finalize_merge_expansion_spec);
                assert(reference_expected == Err(error@));
                assert(final_expected == Err(error@));
                assert(expected == Err(error@));
            }
            return Err(error);
        },
    };
    let ghost output_view = ExpandedSemanticGraphSourceView {
        profile_version: input_view.profile_version,
        transformation_version: MERGE_EXPANSION_TRANSFORMATION_VERSION,
        source_len_bytes: input_view.source_len_bytes,
        input_node_count: input_view.input_node_count,
        expanded_reference_count: references,
        merge_source_count: build.merge_source_count,
        input: input_view,
        mappings: build.mappings,
        entries: build.entries,
    };
    proof {
        assert(reference_expected == Ok(references));
        assert(final_expected == Ok(output_view));
    }

    let output = ExpandedSemanticGraphSource::new(
        source,
        references,
        merge_sources,
        mappings,
        entries,
    );
    proof {
        reveal(expand_profile1_merge_keys_spec);
        reveal(finalize_merge_expansion_spec);
        assert(output@ == output_view);
        assert(final_expected == Ok(output_view));
        assert(expected == Ok(output@));
    }
    Ok(output)
}

pub open spec fn expand_profile1_merge_keys_spec(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
) -> Result<ExpandedSemanticGraphSourceView, MergeExpansionErrorView> {
    match preflight_merge_shapes_spec(input) {
        Err(error) => Err(error),
        Ok(()) => {
            let table = input.structural_keys.scalar_keys.graph.node_table;
            let topology = table.topology;
            match expand_merge_nodes_tail_spec(
                0,
                topology.nodes.len() as nat,
                MergeExpansionBuildView {
                    mappings: Seq::empty(),
                    entries: Seq::empty(),
                    merge_source_count: 0,
                },
                table.nodes,
                table.scalars.scalars,
                topology.nodes,
                topology.sequence_edges,
                topology.mapping_edges,
                input.structural_keys.records,
                limits,
            ) {
                Err(error) => Err(error),
                Ok(build) => finalize_merge_expansion_spec(input, limits, build),
            }
        },
    }
}

pub open spec fn expanded_semantic_graph_source_well_formed_spec(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
    output: ExpandedSemanticGraphSourceView,
) -> bool {
    expand_profile1_merge_keys_spec(input, limits) == Ok(output)
}

pub open spec fn expanded_semantic_graph_source_preserves_input_identity_spec(
    input: DuplicateFreeStructuralKeySourceView,
    output: ExpandedSemanticGraphSourceView,
) -> bool {
    output.profile_version == input.profile_version && output.transformation_version
        == MERGE_EXPANSION_TRANSFORMATION_VERSION && output.source_len_bytes
        == input.source_len_bytes && output.input_node_count == input.input_node_count
        && output.input == input
}

pub proof fn lemma_merge_expansion_success_is_well_formed(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
    output: ExpandedSemanticGraphSourceView,
)
    requires
        expand_profile1_merge_keys_spec(input, limits) == Ok(output),
    ensures
        expanded_semantic_graph_source_well_formed_spec(input, limits, output),
{
    reveal(expanded_semantic_graph_source_well_formed_spec);
}

pub proof fn lemma_merge_expansion_well_formed_authenticates_exact_result(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
    output: ExpandedSemanticGraphSourceView,
)
    requires
        expanded_semantic_graph_source_well_formed_spec(input, limits, output),
    ensures
        expand_profile1_merge_keys_spec(input, limits) == Ok(output),
{
    reveal(expanded_semantic_graph_source_well_formed_spec);
}

pub proof fn lemma_authenticated_merge_expansion_preserves_input_identity(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
    output: ExpandedSemanticGraphSourceView,
)
    requires
        expanded_semantic_graph_source_well_formed_spec(input, limits, output),
    ensures
        expanded_semantic_graph_source_preserves_input_identity_spec(input, output),
{
    reveal(expanded_semantic_graph_source_well_formed_spec);
    reveal(expand_profile1_merge_keys_spec);
    match preflight_merge_shapes_spec(input) {
        Err(_) => {
            assert(false);
        },
        Ok(()) => {
            let table = input.structural_keys.scalar_keys.graph.node_table;
            let topology = table.topology;
            let expansion = expand_merge_nodes_tail_spec(
                0,
                topology.nodes.len() as nat,
                MergeExpansionBuildView {
                    mappings: Seq::empty(),
                    entries: Seq::empty(),
                    merge_source_count: 0,
                },
                table.nodes,
                table.scalars.scalars,
                topology.nodes,
                topology.sequence_edges,
                topology.mapping_edges,
                input.structural_keys.records,
                limits,
            );
            match expansion {
                Err(_) => {
                    assert(false);
                },
                Ok(build) => {
                    reveal(finalize_merge_expansion_spec);
                    let reference_limit = merge_expansion_effective_limit_spec(
                        limits.max_expanded_references,
                        MAX_PROFILE1_EXPANDED_REFERENCES,
                    );
                    let references = count_expanded_reference_roots_tail_spec(
                        0,
                        topology.document_roots.len() as nat,
                        0,
                        reference_limit,
                        topology.source_len_bytes,
                        topology.document_roots,
                        table.nodes,
                        topology.nodes,
                        topology.sequence_edges,
                        build.mappings,
                        build.entries,
                    );
                    match references {
                        Err(_) => {
                            assert(false);
                        },
                        Ok(_) => {
                            reveal(expanded_semantic_graph_source_preserves_input_identity_spec);
                        },
                    }
                },
            }
        },
    }
}

pub proof fn lemma_authenticated_merge_expansion_is_unique(
    input: DuplicateFreeStructuralKeySourceView,
    limits: MergeExpansionLimitsView,
    first: ExpandedSemanticGraphSourceView,
    second: ExpandedSemanticGraphSourceView,
)
    requires
        expanded_semantic_graph_source_well_formed_spec(input, limits, first),
        expanded_semantic_graph_source_well_formed_spec(input, limits, second),
    ensures
        first == second,
{
    reveal(expanded_semantic_graph_source_well_formed_spec);
}

} // verus!
