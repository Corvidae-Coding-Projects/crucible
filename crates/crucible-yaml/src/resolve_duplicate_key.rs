//! Verified rejection of duplicate explicit YAML mapping keys.
//!
//! Equality is exact canonical structural-key byte equality. The checker is iterative,
//! allocation-free, and reports the globally earliest later equal key by source byte. It completes
//! intrinsic duplicate discovery before applying caller-lowered accounting limits.
use crate::cst::CstNodeKind;
use crate::resolve_canonical_structural_key::{
    CanonicalStructuralKeyRecord, CanonicalStructuralKeySource,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_canonical_structural_key::{
    CanonicalStructuralKeyRecordView, CanonicalStructuralKeySourceView,
};
use crate::resolve_topology::{SemanticMappingEdge, SemanticTopologyNode};
use vstd::prelude::*;

verus! {

pub const DUPLICATE_KEY_REJECTION_VERSION: u16 = 1;

pub const MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES: u64 =
    crate::cst::MAX_PROFILE1_CST_MAPPING_ENTRIES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DuplicateKeyLimits {
    max_mappings: u64,
    max_mapping_entries: u64,
}

#[verifier::ext_equal]
pub struct DuplicateKeyLimitsView {
    pub max_mappings: u64,
    pub max_mapping_entries: u64,
}

impl View for DuplicateKeyLimits {
    type V = DuplicateKeyLimitsView;

    closed spec fn view(&self) -> DuplicateKeyLimitsView {
        DuplicateKeyLimitsView {
            max_mappings: self.max_mappings,
            max_mapping_entries: self.max_mapping_entries,
        }
    }
}

impl DuplicateKeyLimits {
    pub fn new(max_mappings: u64, max_mapping_entries: u64) -> (limits: Self)
        ensures
            limits@ == (DuplicateKeyLimitsView { max_mappings, max_mapping_entries }),
    {
        Self { max_mappings, max_mapping_entries }
    }

    pub fn max_mappings(&self) -> (value: u64)
        ensures
            value == self@.max_mappings,
    {
        self.max_mappings
    }

    pub fn max_mapping_entries(&self) -> (value: u64)
        ensures
            value == self@.max_mapping_entries,
    {
        self.max_mapping_entries
    }
}

pub fn canonical_duplicate_key_limits() -> (limits: DuplicateKeyLimits)
    ensures
        limits@ == canonical_duplicate_key_limits_spec(),
{
    DuplicateKeyLimits::new(
        MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS,
        MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES,
    )
}

pub open spec fn canonical_duplicate_key_limits_spec() -> DuplicateKeyLimitsView {
    DuplicateKeyLimitsView {
        max_mappings: MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS,
        max_mapping_entries: MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES,
    }
}

pub open spec fn duplicate_key_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (value: u64)
    ensures
        value == duplicate_key_effective_limit_spec(requested, absolute),
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum DuplicateKeyErrorKind {
    DuplicateExplicitKey,
    MappingLimitExceeded,
    MappingEntryLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DuplicateKeyError {
    kind: DuplicateKeyErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct DuplicateKeyErrorView {
    pub kind: DuplicateKeyErrorKind,
    pub byte_offset: u64,
}

impl View for DuplicateKeyError {
    type V = DuplicateKeyErrorView;

    closed spec fn view(&self) -> DuplicateKeyErrorView {
        DuplicateKeyErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl DuplicateKeyError {
    fn at(kind: DuplicateKeyErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (DuplicateKeyErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: DuplicateKeyErrorKind)
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

#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateFreeStructuralKeySource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    checked_mapping_count: u64,
    checked_mapping_entry_count: u64,
    structural_keys: CanonicalStructuralKeySource,
}

#[verifier::ext_equal]
pub struct DuplicateFreeStructuralKeySourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub checked_mapping_count: u64,
    pub checked_mapping_entry_count: u64,
    pub structural_keys: CanonicalStructuralKeySourceView,
}

impl View for DuplicateFreeStructuralKeySource {
    type V = DuplicateFreeStructuralKeySourceView;

    closed spec fn view(&self) -> DuplicateFreeStructuralKeySourceView {
        DuplicateFreeStructuralKeySourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            checked_mapping_count: self.checked_mapping_count,
            checked_mapping_entry_count: self.checked_mapping_entry_count,
            structural_keys: self.structural_keys@,
        }
    }
}

impl DuplicateFreeStructuralKeySource {
    fn new(
        structural_keys: CanonicalStructuralKeySource,
        checked_mapping_count: u64,
        checked_mapping_entry_count: u64,
    ) -> (source: Self)
        ensures
            source@ == (DuplicateFreeStructuralKeySourceView {
                profile_version: structural_keys@.profile_version,
                transformation_version: DUPLICATE_KEY_REJECTION_VERSION,
                source_len_bytes: structural_keys@.source_len_bytes,
                input_node_count: structural_keys@.input_node_count,
                checked_mapping_count,
                checked_mapping_entry_count,
                structural_keys: structural_keys@,
            }),
    {
        let profile_version = structural_keys.profile_version();
        let source_len_bytes = structural_keys.source_len_bytes();
        let input_node_count = structural_keys.input_node_count();
        Self {
            profile_version,
            transformation_version: DUPLICATE_KEY_REJECTION_VERSION,
            source_len_bytes,
            input_node_count,
            checked_mapping_count,
            checked_mapping_entry_count,
            structural_keys,
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

    pub fn checked_mapping_count(&self) -> (value: u64)
        ensures
            value == self@.checked_mapping_count,
    {
        self.checked_mapping_count
    }

    pub fn checked_mapping_entry_count(&self) -> (value: u64)
        ensures
            value == self@.checked_mapping_entry_count,
    {
        self.checked_mapping_entry_count
    }

    pub fn structural_keys(&self) -> (source: &CanonicalStructuralKeySource)
        ensures
            source@ == self@.structural_keys,
    {
        &self.structural_keys
    }
}

#[verifier::ext_equal]
pub struct DuplicateCheckBuildView {
    pub checked_mapping_count: u64,
    pub checked_mapping_entry_count: u64,
}

#[derive(Clone, Copy)]
struct DuplicateCheckBuild {
    checked_mapping_count: u64,
    checked_mapping_entry_count: u64,
}

impl View for DuplicateCheckBuild {
    type V = DuplicateCheckBuildView;

    closed spec fn view(&self) -> DuplicateCheckBuildView {
        DuplicateCheckBuildView {
            checked_mapping_count: self.checked_mapping_count,
            checked_mapping_entry_count: self.checked_mapping_entry_count,
        }
    }
}

impl DuplicateCheckBuild {
    fn empty() -> (build: Self)
        ensures
            build@ == (DuplicateCheckBuildView {
                checked_mapping_count: 0,
                checked_mapping_entry_count: 0,
            }),
    {
        Self { checked_mapping_count: 0, checked_mapping_entry_count: 0 }
    }
}

pub open spec fn mapping_key_indices_equal_spec(
    left_edge_index: nat,
    right_edge_index: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
) -> Result<bool, DuplicateKeyErrorView> {
    if left_edge_index >= edges.len() || right_edge_index >= edges.len() {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        let left_key = edges[left_edge_index as int].key_node_index;
        let right_key = edges[right_edge_index as int].key_node_index;
        if left_key >= records.len() || right_key >= records.len() {
            Err(
                DuplicateKeyErrorView {
                    kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node_byte,
                },
            )
        } else {
            let left_bytes = records[left_key as int].bytes;
            let right_bytes = records[right_key as int].bytes;
            Ok(
                crate::resolve_canonical_structural_key::compare_byte_views_tail_spec(
                    left_bytes,
                    right_bytes,
                    0,
                    if left_bytes.len() < right_bytes.len() {
                        left_bytes.len() as nat
                    } else {
                        right_bytes.len() as nat
                    },
                ) == 0,
            )
        }
    }
}

fn mapping_key_indices_equal(
    left_edge_index: usize,
    right_edge_index: usize,
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    node_byte: u64,
) -> (result: Result<bool, DuplicateKeyError>)
    ensures
        mapping_key_indices_equal_spec(
            left_edge_index as nat,
            right_edge_index as nat,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            node_byte,
        ) == match result {
            Ok(equal) => Ok(equal),
            Err(error) => Err(error@),
        },
{
    if left_edge_index >= edges.len() || right_edge_index >= edges.len() {
        proof {
            reveal(mapping_key_indices_equal_spec);
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::InternalInvariantViolation, node_byte),
        );
    }
    let left_key_u64 = edges[left_edge_index].key_node_index();
    let right_key_u64 = edges[right_edge_index].key_node_index();
    if left_key_u64 >= records.len() as u64 || right_key_u64 >= records.len() as u64 {
        proof {
            reveal(mapping_key_indices_equal_spec);
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            reveal(
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec,
            );
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::InternalInvariantViolation, node_byte),
        );
    }
    let left_key = left_key_u64 as usize;
    let right_key = right_key_u64 as usize;
    let order = crate::resolve_canonical_structural_key::compare_byte_slices(
        records[left_key].bytes(),
        records[right_key].bytes(),
    );
    proof {
        reveal(mapping_key_indices_equal_spec);
        reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
        reveal(crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec);
    }
    Ok(order == 0)
}

pub closed spec fn mapping_entry_has_prior_duplicate_tail_spec(
    current_edge_index: nat,
    prior_edge_index: nat,
    fuel: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
) -> Result<bool, DuplicateKeyErrorView>
    decreases fuel,
{
    if prior_edge_index > current_edge_index {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else if prior_edge_index == current_edge_index {
        Ok(false)
    } else if fuel == 0 {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        match mapping_key_indices_equal_spec(
            prior_edge_index,
            current_edge_index,
            edges,
            records,
            node_byte,
        ) {
            Err(error) => Err(error),
            Ok(true) => Ok(true),
            Ok(false) => mapping_entry_has_prior_duplicate_tail_spec(
                current_edge_index,
                (prior_edge_index + 1) as nat,
                (fuel - 1) as nat,
                edges,
                records,
                node_byte,
            ),
        }
    }
}

fn mapping_entry_has_prior_duplicate(
    edge_start: usize,
    current_edge_index: usize,
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    node_byte: u64,
) -> (result: Result<bool, DuplicateKeyError>)
    ensures
        mapping_entry_has_prior_duplicate_tail_spec(
            current_edge_index as nat,
            edge_start as nat,
            if edge_start <= current_edge_index {
                (current_edge_index - edge_start) as nat
            } else {
                0nat
            },
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            node_byte,
        ) == match result {
            Ok(found) => Ok(found),
            Err(error) => Err(error@),
        },
{
    if edge_start > current_edge_index {
        proof {
            reveal(mapping_entry_has_prior_duplicate_tail_spec);
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::InternalInvariantViolation, node_byte),
        );
    }
    let ghost expected = mapping_entry_has_prior_duplicate_tail_spec(
        current_edge_index as nat,
        edge_start as nat,
        (current_edge_index - edge_start) as nat,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@,
        ),
        node_byte,
    );
    let mut prior = edge_start;
    while prior < current_edge_index
        invariant
            edge_start <= prior <= current_edge_index,
            expected == mapping_entry_has_prior_duplicate_tail_spec(
                current_edge_index as nat,
                edge_start as nat,
                (current_edge_index - edge_start) as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
                node_byte,
            ),
            expected == mapping_entry_has_prior_duplicate_tail_spec(
                current_edge_index as nat,
                prior as nat,
                (current_edge_index - prior) as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
                node_byte,
            ),
        decreases current_edge_index - prior,
    {
        let equal = match mapping_key_indices_equal(
            prior,
            current_edge_index,
            edges,
            records,
            node_byte,
        ) {
            Err(error) => {
                proof {
                    reveal(mapping_entry_has_prior_duplicate_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(equal) => equal,
        };
        if equal {
            proof {
                reveal(mapping_entry_has_prior_duplicate_tail_spec);
                assert(expected == Ok(true));
            }
            return Ok(true);
        }
        proof {
            reveal(mapping_entry_has_prior_duplicate_tail_spec);
        }
        prior += 1;
    }
    proof {
        reveal(mapping_entry_has_prior_duplicate_tail_spec);
    }
    Ok(false)
}

pub closed spec fn mapping_has_no_duplicate_keys_tail_spec(
    edge_start: nat,
    edge_end: nat,
    current_edge_index: nat,
    fuel: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
) -> Result<(), DuplicateKeyErrorView>
    decreases fuel,
{
    if edge_start > edge_end || edge_end > edges.len() || edge_start > current_edge_index {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else if current_edge_index >= edge_end {
        Ok(())
    } else if fuel == 0 {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        match mapping_entry_has_prior_duplicate_tail_spec(
            current_edge_index,
            edge_start,
            (current_edge_index - edge_start) as nat,
            edges,
            records,
            node_byte,
        ) {
            Err(error) => Err(error),
            Ok(true) => {
                if current_edge_index >= edges.len() {
                    Err(
                        DuplicateKeyErrorView {
                            kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                            byte_offset: node_byte,
                        },
                    )
                } else {
                    let key_node = edges[current_edge_index as int].key_node_index;
                    if key_node >= nodes.len() {
                        Err(
                            DuplicateKeyErrorView {
                                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                                byte_offset: node_byte,
                            },
                        )
                    } else {
                        Err(
                            DuplicateKeyErrorView {
                                kind: DuplicateKeyErrorKind::DuplicateExplicitKey,
                                byte_offset: nodes[key_node as int].byte_start,
                            },
                        )
                    }
                }
            },
            Ok(false) => mapping_has_no_duplicate_keys_tail_spec(
                edge_start,
                edge_end,
                (current_edge_index + 1) as nat,
                (fuel - 1) as nat,
                edges,
                nodes,
                records,
                node_byte,
            ),
        }
    }
}

#[verifier::rlimit(50)]
fn mapping_has_no_duplicate_keys(
    edge_start: usize,
    edge_end: usize,
    edges: &[SemanticMappingEdge],
    nodes: &[SemanticTopologyNode],
    records: &[CanonicalStructuralKeyRecord],
    node_byte: u64,
) -> (result: Result<(), DuplicateKeyError>)
    ensures
        mapping_has_no_duplicate_keys_tail_spec(
            edge_start as nat,
            edge_end as nat,
            edge_start as nat,
            if edge_start <= edge_end {
                (edge_end - edge_start) as nat
            } else {
                0nat
            },
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            node_byte,
        ) == match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error@),
        },
{
    if edge_start > edge_end || edge_end > edges.len() {
        proof {
            reveal(mapping_has_no_duplicate_keys_tail_spec);
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::InternalInvariantViolation, node_byte),
        );
    }
    let ghost expected = mapping_has_no_duplicate_keys_tail_spec(
        edge_start as nat,
        edge_end as nat,
        edge_start as nat,
        (edge_end - edge_start) as nat,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
        crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
            records@,
        ),
        node_byte,
    );
    let mut current = edge_start;
    while current < edge_end
        invariant
            edge_start <= current <= edge_end,
            edge_end <= edges.len(),
            expected == mapping_has_no_duplicate_keys_tail_spec(
                edge_start as nat,
                edge_end as nat,
                edge_start as nat,
                (edge_end - edge_start) as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
                node_byte,
            ),
            expected == mapping_has_no_duplicate_keys_tail_spec(
                edge_start as nat,
                edge_end as nat,
                current as nat,
                (edge_end - current) as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                    records@,
                ),
                node_byte,
            ),
        decreases edge_end - current,
    {
        let duplicate = match mapping_entry_has_prior_duplicate(
            edge_start,
            current,
            edges,
            records,
            node_byte,
        ) {
            Err(error) => {
                proof {
                    reveal(mapping_has_no_duplicate_keys_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(found) => found,
        };
        if duplicate {
            let key_node_u64 = edges[current].key_node_index();
            if key_node_u64 >= nodes.len() as u64 {
                proof {
                    reveal(mapping_has_no_duplicate_keys_tail_spec);
                    reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                }
                return Err(
                    DuplicateKeyError::at(
                        DuplicateKeyErrorKind::InternalInvariantViolation,
                        node_byte,
                    ),
                );
            }
            let key_node = key_node_u64 as usize;
            let error = DuplicateKeyError::at(
                DuplicateKeyErrorKind::DuplicateExplicitKey,
                nodes[key_node].byte_start(),
            );
            proof {
                reveal(mapping_has_no_duplicate_keys_tail_spec);
                reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            reveal(mapping_has_no_duplicate_keys_tail_spec);
        }
        current += 1;
    }
    proof {
        reveal(mapping_has_no_duplicate_keys_tail_spec);
    }
    Ok(())
}

pub open spec fn minimum_duplicate_offset_spec(current: Option<u64>, candidate: u64) -> Option<
    u64,
> {
    match current {
        None => Some(candidate),
        Some(offset) => Some(
            if candidate < offset {
                candidate
            } else {
                offset
            },
        ),
    }
}

fn minimum_duplicate_offset(current: Option<u64>, candidate: u64) -> (value: Option<u64>)
    ensures
        value == minimum_duplicate_offset_spec(current, candidate),
{
    match current {
        None => Some(candidate),
        Some(offset) => Some(
            if candidate < offset {
                candidate
            } else {
                offset
            },
        ),
    }
}

pub open spec fn consider_duplicate_candidate_node_spec(
    node_index: nat,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    current: Option<u64>,
) -> Result<Option<u64>, DuplicateKeyErrorView> {
    if node_index >= nodes.len() || node_index >= records.len()
        || nodes[node_index as int].cst_node_index != node_index
        || records[node_index as int].node_index != node_index {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: if node_index < nodes.len() {
                    nodes[node_index as int].byte_start
                } else {
                    0
                },
            },
        )
    } else {
        let node = nodes[node_index as int];
        if node.kind != CstNodeKind::Mapping {
            Ok(current)
        } else if node.edge_start > node.edge_end || node.edge_end > edges.len() {
            Err(
                DuplicateKeyErrorView {
                    kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node.byte_start,
                },
            )
        } else {
            match mapping_has_no_duplicate_keys_tail_spec(
                node.edge_start as nat,
                node.edge_end as nat,
                node.edge_start as nat,
                (node.edge_end - node.edge_start) as nat,
                edges,
                nodes,
                records,
                node.byte_start,
            ) {
                Ok(()) => Ok(current),
                Err(error) => {
                    if error.kind == DuplicateKeyErrorKind::DuplicateExplicitKey {
                        Ok(minimum_duplicate_offset_spec(current, error.byte_offset))
                    } else {
                        Err(error)
                    }
                },
            }
        }
    }
}

#[verifier::rlimit(50)]
fn consider_duplicate_candidate_node(
    node_index: usize,
    nodes: &[SemanticTopologyNode],
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    current: Option<u64>,
) -> (result: Result<Option<u64>, DuplicateKeyError>)
    ensures
        consider_duplicate_candidate_node_spec(
            node_index as nat,
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            current,
        ) == match result {
            Ok(next) => Ok(next),
            Err(error) => Err(error@),
        },
{
    if node_index >= nodes.len() || node_index >= records.len()
        || nodes[node_index].cst_node_index() != node_index as u64
        || records[node_index].node_index() != node_index as u64 {
        let offset = if node_index < nodes.len() {
            nodes[node_index].byte_start()
        } else {
            0
        };
        proof {
            reveal(consider_duplicate_candidate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            reveal(
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec,
            );
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::InternalInvariantViolation, offset),
        );
    }
    let node = &nodes[node_index];
    if node.kind() != CstNodeKind::Mapping {
        proof {
            reveal(consider_duplicate_candidate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Ok(current);
    }
    let edge_start_u64 = node.edge_start();
    let edge_end_u64 = node.edge_end();
    if edge_start_u64 > edge_end_u64 || edge_end_u64 > edges.len() as u64 {
        proof {
            reveal(consider_duplicate_candidate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Err(
            DuplicateKeyError::at(
                DuplicateKeyErrorKind::InternalInvariantViolation,
                node.byte_start(),
            ),
        );
    }
    let edge_start = edge_start_u64 as usize;
    let edge_end = edge_end_u64 as usize;
    match mapping_has_no_duplicate_keys(
        edge_start,
        edge_end,
        edges,
        nodes,
        records,
        node.byte_start(),
    ) {
        Ok(()) => {
            proof {
                reveal(consider_duplicate_candidate_node_spec);
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            }
            Ok(current)
        },
        Err(error) => {
            if error.kind() == DuplicateKeyErrorKind::DuplicateExplicitKey {
                let next = minimum_duplicate_offset(current, error.byte_offset());
                proof {
                    reveal(consider_duplicate_candidate_node_spec);
                    reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                }
                Ok(next)
            } else {
                proof {
                    reveal(consider_duplicate_candidate_node_spec);
                    reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                }
                Err(error)
            }
        },
    }
}

pub closed spec fn earliest_duplicate_nodes_tail_spec(
    node_index: nat,
    fuel: nat,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    current: Option<u64>,
    source_len_bytes: u64,
) -> Result<Option<u64>, DuplicateKeyErrorView>
    decreases fuel,
{
    if node_index >= nodes.len() {
        Ok(current)
    } else if fuel == 0 {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: source_len_bytes,
            },
        )
    } else {
        match consider_duplicate_candidate_node_spec(node_index, nodes, edges, records, current) {
            Err(error) => Err(error),
            Ok(next) => earliest_duplicate_nodes_tail_spec(
                (node_index + 1) as nat,
                (fuel - 1) as nat,
                nodes,
                edges,
                records,
                next,
                source_len_bytes,
            ),
        }
    }
}

fn find_earliest_duplicate(source: &CanonicalStructuralKeySource) -> (result: Result<
    Option<u64>,
    DuplicateKeyError,
>)
    ensures
        earliest_duplicate_nodes_tail_spec(
            0,
            source@.scalar_keys.graph.node_table.topology.nodes.len() as nat,
            source@.scalar_keys.graph.node_table.topology.nodes,
            source@.scalar_keys.graph.node_table.topology.mapping_edges,
            source@.records,
            None,
            source@.source_len_bytes,
        ) == match result {
            Ok(candidate) => Ok(candidate),
            Err(error) => Err(error@),
        },
{
    let topology = source.scalar_keys().graph().node_table().topology();
    let nodes = topology.nodes();
    let edges = topology.mapping_edges();
    let records = source.records();
    let ghost expected = earliest_duplicate_nodes_tail_spec(
        0,
        source@.scalar_keys.graph.node_table.topology.nodes.len() as nat,
        source@.scalar_keys.graph.node_table.topology.nodes,
        source@.scalar_keys.graph.node_table.topology.mapping_edges,
        source@.records,
        None,
        source@.source_len_bytes,
    );
    let mut current = None;
    let mut node_index = 0usize;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            nodes.len() == source@.scalar_keys.graph.node_table.topology.nodes.len(),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@)
                == source@.scalar_keys.graph.node_table.topology.nodes,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@)
                == source@.scalar_keys.graph.node_table.topology.mapping_edges,
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ) == source@.records,
            expected == earliest_duplicate_nodes_tail_spec(
                0,
                source@.scalar_keys.graph.node_table.topology.nodes.len() as nat,
                source@.scalar_keys.graph.node_table.topology.nodes,
                source@.scalar_keys.graph.node_table.topology.mapping_edges,
                source@.records,
                None,
                source@.source_len_bytes,
            ),
            expected == earliest_duplicate_nodes_tail_spec(
                node_index as nat,
                (nodes.len() - node_index) as nat,
                source@.scalar_keys.graph.node_table.topology.nodes,
                source@.scalar_keys.graph.node_table.topology.mapping_edges,
                source@.records,
                current,
                source@.source_len_bytes,
            ),
        decreases nodes.len() - node_index,
    {
        current =
        match consider_duplicate_candidate_node(node_index, nodes, edges, records, current) {
            Err(error) => {
                proof {
                    reveal(earliest_duplicate_nodes_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(next) => next,
        };
        proof {
            reveal(earliest_duplicate_nodes_tail_spec);
        }
        node_index += 1;
    }
    proof {
        reveal(earliest_duplicate_nodes_tail_spec);
    }
    Ok(current)
}

pub open spec fn checked_mapping_after_limits_spec(
    node: crate::resolve_topology::SemanticTopologyNodeView,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    build: DuplicateCheckBuildView,
    limits: DuplicateKeyLimitsView,
) -> Result<DuplicateCheckBuildView, DuplicateKeyErrorView> {
    let mapping_limit = duplicate_key_effective_limit_spec(
        limits.max_mappings,
        MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS,
    );
    let entry_limit = duplicate_key_effective_limit_spec(
        limits.max_mapping_entries,
        MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES,
    );
    if build.checked_mapping_count >= mapping_limit {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::MappingLimitExceeded,
                byte_offset: node.byte_start,
            },
        )
    } else if build.checked_mapping_entry_count > entry_limit {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: node.byte_start,
            },
        )
    } else {
        let entry_count = (node.edge_end - node.edge_start) as u64;
        if entry_count > entry_limit - build.checked_mapping_entry_count {
            let first_excluded = node.edge_start + (entry_limit
                - build.checked_mapping_entry_count);
            if first_excluded >= edges.len() {
                Err(
                    DuplicateKeyErrorView {
                        kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node.byte_start,
                    },
                )
            } else {
                let key_node = edges[first_excluded as int].key_node_index;
                if key_node >= nodes.len() {
                    Err(
                        DuplicateKeyErrorView {
                            kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                            byte_offset: node.byte_start,
                        },
                    )
                } else {
                    Err(
                        DuplicateKeyErrorView {
                            kind: DuplicateKeyErrorKind::MappingEntryLimitExceeded,
                            byte_offset: nodes[key_node as int].byte_start,
                        },
                    )
                }
            }
        } else {
            Ok(
                DuplicateCheckBuildView {
                    checked_mapping_count: (build.checked_mapping_count + 1) as u64,
                    checked_mapping_entry_count: (build.checked_mapping_entry_count
                        + entry_count) as u64,
                },
            )
        }
    }
}

pub open spec fn check_duplicate_node_spec(
    node_index: nat,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    build: DuplicateCheckBuildView,
    limits: DuplicateKeyLimitsView,
) -> Result<DuplicateCheckBuildView, DuplicateKeyErrorView> {
    if node_index >= nodes.len() || node_index >= records.len()
        || nodes[node_index as int].cst_node_index != node_index
        || records[node_index as int].node_index != node_index {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: if node_index < nodes.len() {
                    nodes[node_index as int].byte_start
                } else {
                    0
                },
            },
        )
    } else {
        let node = nodes[node_index as int];
        if node.kind != CstNodeKind::Mapping {
            Ok(build)
        } else if node.edge_start > node.edge_end || node.edge_end > edges.len() {
            Err(
                DuplicateKeyErrorView {
                    kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node.byte_start,
                },
            )
        } else {
            match mapping_has_no_duplicate_keys_tail_spec(
                node.edge_start as nat,
                node.edge_end as nat,
                node.edge_start as nat,
                (node.edge_end - node.edge_start) as nat,
                edges,
                nodes,
                records,
                node.byte_start,
            ) {
                Err(error) => Err(error),
                Ok(()) => checked_mapping_after_limits_spec(node, edges, nodes, build, limits),
            }
        }
    }
}

#[verifier::rlimit(50)]
fn check_duplicate_node(
    node_index: usize,
    nodes: &[SemanticTopologyNode],
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    build: DuplicateCheckBuild,
    limits: DuplicateKeyLimits,
) -> (result: Result<DuplicateCheckBuild, DuplicateKeyError>)
    ensures
        check_duplicate_node_spec(
            node_index as nat,
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@),
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ),
            build@,
            limits@,
        ) == match result {
            Ok(next) => Ok(next@),
            Err(error) => Err(error@),
        },
{
    if node_index >= nodes.len() || node_index >= records.len()
        || nodes[node_index].cst_node_index() != node_index as u64
        || records[node_index].node_index() != node_index as u64 {
        let offset = if node_index < nodes.len() {
            nodes[node_index].byte_start()
        } else {
            0
        };
        proof {
            reveal(check_duplicate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            reveal(
                crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec,
            );
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::InternalInvariantViolation, offset),
        );
    }
    let node = &nodes[node_index];
    if node.kind() != CstNodeKind::Mapping {
        proof {
            reveal(check_duplicate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Ok(build);
    }
    let edge_start_u64 = node.edge_start();
    let edge_end_u64 = node.edge_end();
    if edge_start_u64 > edge_end_u64 || edge_end_u64 > edges.len() as u64 {
        proof {
            reveal(check_duplicate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Err(
            DuplicateKeyError::at(
                DuplicateKeyErrorKind::InternalInvariantViolation,
                node.byte_start(),
            ),
        );
    }
    let edge_start = edge_start_u64 as usize;
    let edge_end = edge_end_u64 as usize;
    if let Err(error) = mapping_has_no_duplicate_keys(
        edge_start,
        edge_end,
        edges,
        nodes,
        records,
        node.byte_start(),
    ) {
        proof {
            reveal(check_duplicate_node_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Err(error);
    }
    let mapping_limit = effective_limit(
        limits.max_mappings(),
        MAX_PROFILE1_DUPLICATE_CHECKED_MAPPINGS,
    );
    let entry_limit = effective_limit(
        limits.max_mapping_entries(),
        MAX_PROFILE1_DUPLICATE_CHECKED_MAPPING_ENTRIES,
    );
    if build.checked_mapping_count >= mapping_limit {
        proof {
            reveal(check_duplicate_node_spec);
            reveal(checked_mapping_after_limits_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Err(
            DuplicateKeyError::at(DuplicateKeyErrorKind::MappingLimitExceeded, node.byte_start()),
        );
    }
    if build.checked_mapping_entry_count > entry_limit {
        proof {
            reveal(check_duplicate_node_spec);
            reveal(checked_mapping_after_limits_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
        }
        return Err(
            DuplicateKeyError::at(
                DuplicateKeyErrorKind::InternalInvariantViolation,
                node.byte_start(),
            ),
        );
    }
    let entry_count = edge_end_u64 - edge_start_u64;
    if entry_count > entry_limit - build.checked_mapping_entry_count {
        let excluded_delta = entry_limit - build.checked_mapping_entry_count;
        let first_excluded_u64 = edge_start_u64 + excluded_delta;
        if first_excluded_u64 >= edges.len() as u64 {
            proof {
                reveal(check_duplicate_node_spec);
                reveal(checked_mapping_after_limits_spec);
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            }
            return Err(
                DuplicateKeyError::at(
                    DuplicateKeyErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                ),
            );
        }
        let first_excluded = first_excluded_u64 as usize;
        let key_node_u64 = edges[first_excluded].key_node_index();
        if key_node_u64 >= nodes.len() as u64 {
            proof {
                reveal(check_duplicate_node_spec);
                reveal(checked_mapping_after_limits_spec);
                reveal(crate::resolve_topology::semantic_topology_node_views_spec);
                reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            }
            return Err(
                DuplicateKeyError::at(
                    DuplicateKeyErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                ),
            );
        }
        let key_node = key_node_u64 as usize;
        let error = DuplicateKeyError::at(
            DuplicateKeyErrorKind::MappingEntryLimitExceeded,
            nodes[key_node].byte_start(),
        );
        proof {
            reveal(check_duplicate_node_spec);
            reveal(checked_mapping_after_limits_spec);
            reveal(crate::resolve_topology::semantic_topology_node_views_spec);
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
        }
        return Err(error);
    }
    let next = DuplicateCheckBuild {
        checked_mapping_count: build.checked_mapping_count + 1,
        checked_mapping_entry_count: build.checked_mapping_entry_count + entry_count,
    };
    proof {
        reveal(check_duplicate_node_spec);
        reveal(checked_mapping_after_limits_spec);
        reveal(crate::resolve_topology::semantic_topology_node_views_spec);
    }
    Ok(next)
}

pub closed spec fn duplicate_key_nodes_tail_spec(
    node_index: nat,
    fuel: nat,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    build: DuplicateCheckBuildView,
    limits: DuplicateKeyLimitsView,
    source_len_bytes: u64,
) -> Result<DuplicateCheckBuildView, DuplicateKeyErrorView>
    decreases fuel,
{
    if node_index >= nodes.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: source_len_bytes,
            },
        )
    } else {
        match check_duplicate_node_spec(node_index, nodes, edges, records, build, limits) {
            Err(error) => Err(error),
            Ok(next) => duplicate_key_nodes_tail_spec(
                (node_index + 1) as nat,
                (fuel - 1) as nat,
                nodes,
                edges,
                records,
                next,
                limits,
                source_len_bytes,
            ),
        }
    }
}

fn check_all_duplicate_keys(
    source: &CanonicalStructuralKeySource,
    limits: DuplicateKeyLimits,
) -> (result: Result<DuplicateCheckBuild, DuplicateKeyError>)
    ensures
        duplicate_key_nodes_tail_spec(
            0,
            source@.scalar_keys.graph.node_table.topology.nodes.len() as nat,
            source@.scalar_keys.graph.node_table.topology.nodes,
            source@.scalar_keys.graph.node_table.topology.mapping_edges,
            source@.records,
            DuplicateCheckBuildView { checked_mapping_count: 0, checked_mapping_entry_count: 0 },
            limits@,
            source@.source_len_bytes,
        ) == match result {
            Ok(build) => Ok(build@),
            Err(error) => Err(error@),
        },
{
    let topology = source.scalar_keys().graph().node_table().topology();
    let nodes = topology.nodes();
    let edges = topology.mapping_edges();
    let records = source.records();
    let mut build = DuplicateCheckBuild::empty();
    let ghost expected = duplicate_key_nodes_tail_spec(
        0,
        source@.scalar_keys.graph.node_table.topology.nodes.len() as nat,
        source@.scalar_keys.graph.node_table.topology.nodes,
        source@.scalar_keys.graph.node_table.topology.mapping_edges,
        source@.records,
        build@,
        limits@,
        source@.source_len_bytes,
    );
    let mut node_index = 0usize;
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            nodes.len() == source@.scalar_keys.graph.node_table.topology.nodes.len(),
            crate::resolve_topology::semantic_topology_node_views_spec(nodes@)
                == source@.scalar_keys.graph.node_table.topology.nodes,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@)
                == source@.scalar_keys.graph.node_table.topology.mapping_edges,
            crate::resolve_canonical_structural_key::canonical_structural_key_record_views_spec(
                records@,
            ) == source@.records,
            expected == duplicate_key_nodes_tail_spec(
                0,
                source@.scalar_keys.graph.node_table.topology.nodes.len() as nat,
                source@.scalar_keys.graph.node_table.topology.nodes,
                source@.scalar_keys.graph.node_table.topology.mapping_edges,
                source@.records,
                DuplicateCheckBuildView {
                    checked_mapping_count: 0,
                    checked_mapping_entry_count: 0,
                },
                limits@,
                source@.source_len_bytes,
            ),
            expected == duplicate_key_nodes_tail_spec(
                node_index as nat,
                (nodes.len() - node_index) as nat,
                source@.scalar_keys.graph.node_table.topology.nodes,
                source@.scalar_keys.graph.node_table.topology.mapping_edges,
                source@.records,
                build@,
                limits@,
                source@.source_len_bytes,
            ),
        decreases nodes.len() - node_index,
    {
        build =
        match check_duplicate_node(node_index, nodes, edges, records, build, limits) {
            Err(error) => {
                proof {
                    reveal(duplicate_key_nodes_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(next) => next,
        };
        proof {
            reveal(duplicate_key_nodes_tail_spec);
        }
        node_index += 1;
    }
    proof {
        reveal(duplicate_key_nodes_tail_spec);
    }
    Ok(build)
}

pub open spec fn duplicate_key_input_shape_spec(source: CanonicalStructuralKeySourceView) -> bool {
    source.input_node_count == source.records.len() && source.records.len()
        == source.scalar_keys.graph.node_table.topology.nodes.len()
        && source.scalar_keys.graph.node_table.topology.input_node_count
        == source.scalar_keys.graph.node_table.topology.nodes.len()
        && source.scalar_keys.graph.node_table.topology.input_mapping_entry_count
        == source.scalar_keys.graph.node_table.topology.mapping_edges.len()
}

pub open spec fn finalize_duplicate_key_check_spec(
    source: CanonicalStructuralKeySourceView,
    result: Result<DuplicateCheckBuildView, DuplicateKeyErrorView>,
) -> Result<DuplicateFreeStructuralKeySourceView, DuplicateKeyErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => Ok(
            DuplicateFreeStructuralKeySourceView {
                profile_version: source.profile_version,
                transformation_version: DUPLICATE_KEY_REJECTION_VERSION,
                source_len_bytes: source.source_len_bytes,
                input_node_count: source.input_node_count,
                checked_mapping_count: build.checked_mapping_count,
                checked_mapping_entry_count: build.checked_mapping_entry_count,
                structural_keys: source,
            },
        ),
    }
}

pub open spec fn mapping_keys_pairwise_distinct_spec(
    edge_start: nat,
    edge_end: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
) -> bool {
    edge_start <= edge_end <= edges.len() && forall|prior: nat, later: nat|
        edge_start <= prior < later < edge_end ==> #[trigger] mapping_key_indices_equal_spec(
            prior,
            later,
            edges,
            records,
            node_byte,
        ) == Ok(false)
}

pub proof fn lemma_equal_mapping_key_pair_is_not_distinct(
    edge_start: nat,
    edge_end: nat,
    prior: nat,
    later: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
)
    requires
        edge_start <= prior < later < edge_end <= edges.len(),
        mapping_key_indices_equal_spec(prior, later, edges, records, node_byte) == Ok(true),
    ensures
        !mapping_keys_pairwise_distinct_spec(edge_start, edge_end, edges, records, node_byte),
{
    reveal(mapping_keys_pairwise_distinct_spec);
    if mapping_keys_pairwise_distinct_spec(edge_start, edge_end, edges, records, node_byte) {
        assert(forall|candidate_prior: nat, candidate_later: nat|
            edge_start <= candidate_prior < candidate_later < edge_end
                ==> #[trigger] mapping_key_indices_equal_spec(
                candidate_prior,
                candidate_later,
                edges,
                records,
                node_byte,
            ) == Ok(false));
        assert(edge_start <= prior < later < edge_end);
        assert(mapping_key_indices_equal_spec(prior, later, edges, records, node_byte) == Ok(
            false,
        ));
    }
}

pub open spec fn duplicate_free_structural_keys_spec(
    source: CanonicalStructuralKeySourceView,
) -> bool {
    duplicate_key_input_shape_spec(source) && forall|node_index: int|
        0 <= node_index < source.scalar_keys.graph.node_table.topology.nodes.len() ==> {
            let node = source.scalar_keys.graph.node_table.topology.nodes[node_index];
            #[trigger] source.records[node_index].node_index == node_index && node.cst_node_index
                == node_index && (node.kind != CstNodeKind::Mapping
                || mapping_keys_pairwise_distinct_spec(
                node.edge_start as nat,
                node.edge_end as nat,
                source.scalar_keys.graph.node_table.topology.mapping_edges,
                source.records,
                node.byte_start,
            ))
        }
}

pub proof fn lemma_prior_duplicate_scan_false_is_pairwise(
    current_edge_index: nat,
    prior_edge_index: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
)
    requires
        prior_edge_index <= current_edge_index,
        mapping_entry_has_prior_duplicate_tail_spec(
            current_edge_index,
            prior_edge_index,
            (current_edge_index - prior_edge_index) as nat,
            edges,
            records,
            node_byte,
        ) == Ok(false),
    ensures
        forall|prior: nat|
            prior_edge_index <= prior < current_edge_index
                ==> #[trigger] mapping_key_indices_equal_spec(
                prior,
                current_edge_index,
                edges,
                records,
                node_byte,
            ) == Ok(false),
    decreases current_edge_index - prior_edge_index,
{
    if prior_edge_index < current_edge_index {
        reveal(mapping_entry_has_prior_duplicate_tail_spec);
        assert(mapping_key_indices_equal_spec(
            prior_edge_index,
            current_edge_index,
            edges,
            records,
            node_byte,
        ) == Ok(false));
        lemma_prior_duplicate_scan_false_is_pairwise(
            current_edge_index,
            (prior_edge_index + 1) as nat,
            edges,
            records,
            node_byte,
        );
        assert forall|prior: nat|
            prior_edge_index <= prior
                < current_edge_index implies #[trigger] mapping_key_indices_equal_spec(
            prior,
            current_edge_index,
            edges,
            records,
            node_byte,
        ) == Ok(false) by {
            if prior > prior_edge_index {
                assert((prior_edge_index + 1) as nat <= prior);
            }
        }
    }
}

pub proof fn lemma_successful_mapping_scan_is_pairwise(
    edge_start: nat,
    edge_end: nat,
    current_edge_index: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
)
    requires
        edge_start <= current_edge_index <= edge_end <= edges.len(),
        mapping_has_no_duplicate_keys_tail_spec(
            edge_start,
            edge_end,
            current_edge_index,
            (edge_end - current_edge_index) as nat,
            edges,
            nodes,
            records,
            node_byte,
        ) == Ok(()),
    ensures
        forall|prior: nat, later: nat|
            edge_start <= prior < later && current_edge_index <= later < edge_end
                ==> #[trigger] mapping_key_indices_equal_spec(
                prior,
                later,
                edges,
                records,
                node_byte,
            ) == Ok(false),
    decreases edge_end - current_edge_index,
{
    if current_edge_index < edge_end {
        reveal(mapping_has_no_duplicate_keys_tail_spec);
        assert(mapping_entry_has_prior_duplicate_tail_spec(
            current_edge_index,
            edge_start,
            (current_edge_index - edge_start) as nat,
            edges,
            records,
            node_byte,
        ) == Ok(false));
        lemma_prior_duplicate_scan_false_is_pairwise(
            current_edge_index,
            edge_start,
            edges,
            records,
            node_byte,
        );
        assert(mapping_has_no_duplicate_keys_tail_spec(
            edge_start,
            edge_end,
            (current_edge_index + 1) as nat,
            (edge_end - (current_edge_index + 1)) as nat,
            edges,
            nodes,
            records,
            node_byte,
        ) == Ok(()));
        lemma_successful_mapping_scan_is_pairwise(
            edge_start,
            edge_end,
            (current_edge_index + 1) as nat,
            edges,
            nodes,
            records,
            node_byte,
        );
        assert forall|prior: nat, later: nat|
            edge_start <= prior < later && current_edge_index <= later
                < edge_end implies #[trigger] mapping_key_indices_equal_spec(
            prior,
            later,
            edges,
            records,
            node_byte,
        ) == Ok(false) by {
            if later > current_edge_index {
                assert((current_edge_index + 1) as nat <= later);
            }
        }
    }
}

pub proof fn lemma_successful_node_scan_is_duplicate_free(
    node_index: nat,
    nodes: Seq<crate::resolve_topology::SemanticTopologyNodeView>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    build: DuplicateCheckBuildView,
    limits: DuplicateKeyLimitsView,
    source_len_bytes: u64,
    result: DuplicateCheckBuildView,
)
    requires
        node_index <= nodes.len(),
        nodes.len() == records.len(),
        duplicate_key_nodes_tail_spec(
            node_index,
            (nodes.len() - node_index) as nat,
            nodes,
            edges,
            records,
            build,
            limits,
            source_len_bytes,
        ) == Ok(result),
    ensures
        forall|candidate: int|
            node_index <= candidate < nodes.len() ==> {
                let node = nodes[candidate];
                #[trigger] records[candidate].node_index == candidate && node.cst_node_index
                    == candidate && (node.kind != CstNodeKind::Mapping
                    || mapping_keys_pairwise_distinct_spec(
                    node.edge_start as nat,
                    node.edge_end as nat,
                    edges,
                    records,
                    node.byte_start,
                ))
            },
    decreases nodes.len() - node_index,
{
    if node_index < nodes.len() {
        reveal(duplicate_key_nodes_tail_spec);
        let step = check_duplicate_node_spec(node_index, nodes, edges, records, build, limits);
        match step {
            Err(_) => {
                assert(false);
            },
            Ok(next) => {
                assert(check_duplicate_node_spec(node_index, nodes, edges, records, build, limits)
                    == Ok(next));
                assert(nodes[node_index as int].cst_node_index == node_index);
                assert(records[node_index as int].node_index == node_index);
                if nodes[node_index as int].kind == CstNodeKind::Mapping {
                    reveal(check_duplicate_node_spec);
                    let mapping_scan = mapping_has_no_duplicate_keys_tail_spec(
                        nodes[node_index as int].edge_start as nat,
                        nodes[node_index as int].edge_end as nat,
                        nodes[node_index as int].edge_start as nat,
                        (nodes[node_index as int].edge_end
                            - nodes[node_index as int].edge_start) as nat,
                        edges,
                        nodes,
                        records,
                        nodes[node_index as int].byte_start,
                    );
                    if mapping_scan != Ok(()) {
                        match mapping_scan {
                            Err(mapping_error) => {
                                assert(check_duplicate_node_spec(
                                    node_index,
                                    nodes,
                                    edges,
                                    records,
                                    build,
                                    limits,
                                ) == Err(mapping_error));
                                assert(false);
                            },
                            Ok(unit_value) => {
                                assert(unit_value == ());
                                assert(mapping_scan == Ok(unit_value));
                                assert(false);
                            },
                        }
                    }
                    assert(mapping_scan == Ok(()));
                    assert(nodes[node_index as int].edge_start <= nodes[node_index as int].edge_end
                        <= edges.len());
                    lemma_successful_mapping_scan_is_pairwise(
                        nodes[node_index as int].edge_start as nat,
                        nodes[node_index as int].edge_end as nat,
                        nodes[node_index as int].edge_start as nat,
                        edges,
                        nodes,
                        records,
                        nodes[node_index as int].byte_start,
                    );
                    reveal(mapping_keys_pairwise_distinct_spec);
                    assert(mapping_keys_pairwise_distinct_spec(
                        nodes[node_index as int].edge_start as nat,
                        nodes[node_index as int].edge_end as nat,
                        edges,
                        records,
                        nodes[node_index as int].byte_start,
                    ));
                }
                lemma_successful_node_scan_is_duplicate_free(
                    (node_index + 1) as nat,
                    nodes,
                    edges,
                    records,
                    next,
                    limits,
                    source_len_bytes,
                    result,
                );
                assert forall|candidate: int| node_index <= candidate < nodes.len() implies {
                    let node = nodes[candidate];
                    #[trigger] records[candidate].node_index == candidate && node.cst_node_index
                        == candidate && (node.kind != CstNodeKind::Mapping
                        || mapping_keys_pairwise_distinct_spec(
                        node.edge_start as nat,
                        node.edge_end as nat,
                        edges,
                        records,
                        node.byte_start,
                    ))
                } by {
                    if candidate == node_index {
                        assert(records[candidate].node_index == candidate);
                        assert(nodes[candidate].cst_node_index == candidate);
                    } else {
                        assert(candidate > node_index);
                        assert((node_index + 1) as nat <= candidate);
                        assert(records[candidate].node_index == candidate);
                    }
                }
            },
        }
    }
}

pub open spec fn finish_earliest_duplicate_check_spec(
    source: CanonicalStructuralKeySourceView,
    limits: DuplicateKeyLimitsView,
    candidate_result: Result<Option<u64>, DuplicateKeyErrorView>,
) -> Result<DuplicateFreeStructuralKeySourceView, DuplicateKeyErrorView> {
    match candidate_result {
        Err(error) => Err(error),
        Ok(Some(byte_offset)) => Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::DuplicateExplicitKey,
                byte_offset,
            },
        ),
        Ok(None) => finalize_duplicate_key_check_spec(
            source,
            duplicate_key_nodes_tail_spec(
                0,
                source.scalar_keys.graph.node_table.topology.nodes.len() as nat,
                source.scalar_keys.graph.node_table.topology.nodes,
                source.scalar_keys.graph.node_table.topology.mapping_edges,
                source.records,
                DuplicateCheckBuildView {
                    checked_mapping_count: 0,
                    checked_mapping_entry_count: 0,
                },
                limits,
                source.source_len_bytes,
            ),
        ),
    }
}

pub open spec fn reject_profile1_duplicate_keys_spec(
    source: CanonicalStructuralKeySourceView,
    limits: DuplicateKeyLimitsView,
) -> Result<DuplicateFreeStructuralKeySourceView, DuplicateKeyErrorView> {
    if !duplicate_key_input_shape_spec(source) {
        Err(
            DuplicateKeyErrorView {
                kind: DuplicateKeyErrorKind::InternalInvariantViolation,
                byte_offset: source.source_len_bytes,
            },
        )
    } else {
        finish_earliest_duplicate_check_spec(
            source,
            limits,
            earliest_duplicate_nodes_tail_spec(
                0,
                source.scalar_keys.graph.node_table.topology.nodes.len() as nat,
                source.scalar_keys.graph.node_table.topology.nodes,
                source.scalar_keys.graph.node_table.topology.mapping_edges,
                source.records,
                None,
                source.source_len_bytes,
            ),
        )
    }
}

pub open spec fn duplicate_free_structural_key_source_well_formed_spec(
    input: CanonicalStructuralKeySourceView,
    limits: DuplicateKeyLimitsView,
    output: DuplicateFreeStructuralKeySourceView,
) -> bool {
    reject_profile1_duplicate_keys_spec(input, limits) == Ok(output)
        && duplicate_free_structural_keys_spec(output.structural_keys)
}

pub proof fn lemma_duplicate_key_success_has_pairwise_distinct_mappings(
    input: CanonicalStructuralKeySourceView,
    limits: DuplicateKeyLimitsView,
    output: DuplicateFreeStructuralKeySourceView,
)
    requires
        reject_profile1_duplicate_keys_spec(input, limits) == Ok(output),
    ensures
        duplicate_free_structural_keys_spec(output.structural_keys),
{
    reveal(reject_profile1_duplicate_keys_spec);
    if !duplicate_key_input_shape_spec(input) {
        assert(false);
    }
    let candidates = earliest_duplicate_nodes_tail_spec(
        0,
        input.scalar_keys.graph.node_table.topology.nodes.len() as nat,
        input.scalar_keys.graph.node_table.topology.nodes,
        input.scalar_keys.graph.node_table.topology.mapping_edges,
        input.records,
        None,
        input.source_len_bytes,
    );
    match candidates {
        Err(_) => {
            assert(false);
        },
        Ok(Some(_)) => {
            assert(false);
        },
        Ok(None) => {},
    }
    reveal(finish_earliest_duplicate_check_spec);
    let scan = duplicate_key_nodes_tail_spec(
        0,
        input.scalar_keys.graph.node_table.topology.nodes.len() as nat,
        input.scalar_keys.graph.node_table.topology.nodes,
        input.scalar_keys.graph.node_table.topology.mapping_edges,
        input.records,
        DuplicateCheckBuildView { checked_mapping_count: 0, checked_mapping_entry_count: 0 },
        limits,
        input.source_len_bytes,
    );
    match scan {
        Err(_) => {
            assert(false);
        },
        Ok(build) => {
            reveal(finalize_duplicate_key_check_spec);
            assert(output.structural_keys == input);
            lemma_successful_node_scan_is_duplicate_free(
                0,
                input.scalar_keys.graph.node_table.topology.nodes,
                input.scalar_keys.graph.node_table.topology.mapping_edges,
                input.records,
                DuplicateCheckBuildView {
                    checked_mapping_count: 0,
                    checked_mapping_entry_count: 0,
                },
                limits,
                input.source_len_bytes,
                build,
            );
            reveal(duplicate_free_structural_keys_spec);
        },
    }
}

pub proof fn lemma_duplicate_key_success_is_well_formed(
    input: CanonicalStructuralKeySourceView,
    limits: DuplicateKeyLimitsView,
    output: DuplicateFreeStructuralKeySourceView,
)
    requires
        reject_profile1_duplicate_keys_spec(input, limits) == Ok(output),
    ensures
        duplicate_free_structural_key_source_well_formed_spec(input, limits, output),
{
    lemma_duplicate_key_success_has_pairwise_distinct_mappings(input, limits, output);
    reveal(duplicate_free_structural_key_source_well_formed_spec);
}

pub proof fn lemma_duplicate_key_well_formed_authenticates_exact_result(
    input: CanonicalStructuralKeySourceView,
    limits: DuplicateKeyLimitsView,
    output: DuplicateFreeStructuralKeySourceView,
)
    requires
        duplicate_free_structural_key_source_well_formed_spec(input, limits, output),
    ensures
        reject_profile1_duplicate_keys_spec(input, limits) == Ok(output),
{
    reveal(duplicate_free_structural_key_source_well_formed_spec);
}

pub fn reject_profile1_duplicate_keys(
    source: CanonicalStructuralKeySource,
    limits: DuplicateKeyLimits,
) -> (result: Result<DuplicateFreeStructuralKeySource, DuplicateKeyError>)
    ensures
        reject_profile1_duplicate_keys_spec(source@, limits@) == match result {
            Ok(output) => Ok(output@),
            Err(error) => Err(error@),
        },
{
    let topology = source.scalar_keys().graph().node_table().topology();
    if source.input_node_count() != source.records().len() as u64 || source.records().len()
        != topology.nodes().len() || topology.input_node_count() != topology.nodes().len() as u64
        || topology.input_mapping_entry_count() != topology.mapping_edges().len() as u64 {
        proof {
            reveal(reject_profile1_duplicate_keys_spec);
            reveal(duplicate_key_input_shape_spec);
        }
        return Err(
            DuplicateKeyError::at(
                DuplicateKeyErrorKind::InternalInvariantViolation,
                source.source_len_bytes(),
            ),
        );
    }
    let duplicate = match find_earliest_duplicate(&source) {
        Err(error) => {
            proof {
                reveal(reject_profile1_duplicate_keys_spec);
                reveal(duplicate_key_input_shape_spec);
                reveal(finish_earliest_duplicate_check_spec);
            }
            return Err(error);
        },
        Ok(candidate) => candidate,
    };
    if let Some(byte_offset) = duplicate {
        let error = DuplicateKeyError::at(DuplicateKeyErrorKind::DuplicateExplicitKey, byte_offset);
        proof {
            reveal(reject_profile1_duplicate_keys_spec);
            reveal(duplicate_key_input_shape_spec);
            reveal(finish_earliest_duplicate_check_spec);
        }
        return Err(error);
    }
    let build = match check_all_duplicate_keys(&source, limits) {
        Err(error) => {
            proof {
                reveal(reject_profile1_duplicate_keys_spec);
                reveal(duplicate_key_input_shape_spec);
                reveal(finish_earliest_duplicate_check_spec);
                reveal(finalize_duplicate_key_check_spec);
            }
            return Err(error);
        },
        Ok(build) => build,
    };
    let output = DuplicateFreeStructuralKeySource::new(
        source,
        build.checked_mapping_count,
        build.checked_mapping_entry_count,
    );
    proof {
        reveal(reject_profile1_duplicate_keys_spec);
        reveal(duplicate_key_input_shape_spec);
        reveal(finish_earliest_duplicate_check_spec);
        reveal(finalize_duplicate_key_check_spec);
    }
    Ok(output)
}

} // verus!
