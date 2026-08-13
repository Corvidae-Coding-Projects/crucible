//! Verified alias-cycle rejection and semantic-depth composition.
//!
//! CST children are completed before their parents. Once every alias redirect also targets a
//! lower CST node index, every semantic edge strictly decreases that natural-number identity and
//! the graph is acyclic. The executable machine rejects the first nondecreasing alias edge before
//! caller traversal caps, then builds exact per-node depths and an explicit deepest-path stack.
use crate::atom::AtomizedSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::atom::AtomizedSourceView;
use crate::block::BlockScalarSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::block::BlockScalarSourceView;
use crate::cst::CstSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::cst::CstSourceView;
use crate::plain::PlainScalarSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::plain::PlainScalarSourceView;
use crate::quoted::QuotedScalarSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::quoted::QuotedScalarSourceView;
use crate::resolve_anchor::AnchorAliasLimits;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_anchor::AnchorAliasLimitsView;
use crate::resolve_node_table::{
    SemanticAliasRedirect, SemanticNodeKind, SemanticNodeTableError, SemanticNodeTableErrorKind,
    SemanticNodeTableLimits, SemanticNodeTableSource,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_node_table::{
    SemanticAliasRedirectView, SemanticNodeTableLimitsView, SemanticNodeTableSourceView,
};
use crate::resolve_scalar_table::SemanticScalarTableLimits;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_scalar_table::SemanticScalarTableLimitsView;
use crate::resolve_topology::SemanticTopologyLimits;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_topology::SemanticTopologyLimitsView;
use crate::token::CompletedTokenSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const ALIAS_CYCLE_RESOLUTION_VERSION: u16 = 1;

pub const MAX_PROFILE1_SEMANTIC_DEPTH: u64 = crate::cst::MAX_PROFILE1_CST_DEPTH;

pub const MAX_PROFILE1_SEMANTIC_WORK_STACK: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AliasCycleLimits {
    max_depth: u64,
    max_work_stack: u64,
}

#[verifier::ext_equal]
pub struct AliasCycleLimitsView {
    pub max_depth: u64,
    pub max_work_stack: u64,
}

impl View for AliasCycleLimits {
    type V = AliasCycleLimitsView;

    closed spec fn view(&self) -> AliasCycleLimitsView {
        AliasCycleLimitsView { max_depth: self.max_depth, max_work_stack: self.max_work_stack }
    }
}

impl AliasCycleLimits {
    pub fn new(max_depth: u64, max_work_stack: u64) -> (limits: Self)
        ensures
            limits@ == (AliasCycleLimitsView { max_depth, max_work_stack }),
    {
        Self { max_depth, max_work_stack }
    }

    pub fn max_depth(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_depth,
    {
        self.max_depth
    }

    pub fn max_work_stack(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_work_stack,
    {
        self.max_work_stack
    }
}

pub fn canonical_alias_cycle_limits() -> (limits: AliasCycleLimits)
    ensures
        limits@ == canonical_alias_cycle_limits_spec(),
{
    AliasCycleLimits::new(MAX_PROFILE1_SEMANTIC_DEPTH, MAX_PROFILE1_SEMANTIC_WORK_STACK)
}

pub open spec fn canonical_alias_cycle_limits_spec() -> AliasCycleLimitsView {
    AliasCycleLimitsView {
        max_depth: MAX_PROFILE1_SEMANTIC_DEPTH,
        max_work_stack: MAX_PROFILE1_SEMANTIC_WORK_STACK,
    }
}

pub open spec fn effective_alias_cycle_depth_limit_spec(limits: AliasCycleLimitsView) -> u64 {
    if limits.max_depth < MAX_PROFILE1_SEMANTIC_DEPTH {
        limits.max_depth
    } else {
        MAX_PROFILE1_SEMANTIC_DEPTH
    }
}

pub open spec fn effective_alias_cycle_work_stack_limit_spec(limits: AliasCycleLimitsView) -> u64 {
    if limits.max_work_stack < MAX_PROFILE1_SEMANTIC_WORK_STACK {
        limits.max_work_stack
    } else {
        MAX_PROFILE1_SEMANTIC_WORK_STACK
    }
}

fn effective_alias_cycle_depth_limit(limits: AliasCycleLimits) -> (limit: u64)
    ensures
        limit == effective_alias_cycle_depth_limit_spec(limits@),
{
    if limits.max_depth() < MAX_PROFILE1_SEMANTIC_DEPTH {
        limits.max_depth()
    } else {
        MAX_PROFILE1_SEMANTIC_DEPTH
    }
}

fn effective_alias_cycle_work_stack_limit(limits: AliasCycleLimits) -> (limit: u64)
    ensures
        limit == effective_alias_cycle_work_stack_limit_spec(limits@),
{
    if limits.max_work_stack() < MAX_PROFILE1_SEMANTIC_WORK_STACK {
        limits.max_work_stack()
    } else {
        MAX_PROFILE1_SEMANTIC_WORK_STACK
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum SemanticVisitState {
    Complete,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AcyclicSemanticGraphSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    input_alias_count: u64,
    max_depth_observed: u64,
    node_table: SemanticNodeTableSource,
    node_depths: Vec<u64>,
    visit_states: Vec<SemanticVisitState>,
    visit_order: Vec<u64>,
    deepest_path: Vec<u64>,
}

#[verifier::ext_equal]
pub struct AcyclicSemanticGraphSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub input_alias_count: u64,
    pub max_depth_observed: u64,
    pub node_table: SemanticNodeTableSourceView,
    pub node_depths: Seq<u64>,
    pub visit_states: Seq<SemanticVisitState>,
    pub visit_order: Seq<u64>,
    pub deepest_path: Seq<u64>,
}

impl View for AcyclicSemanticGraphSource {
    type V = AcyclicSemanticGraphSourceView;

    closed spec fn view(&self) -> AcyclicSemanticGraphSourceView {
        AcyclicSemanticGraphSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            input_alias_count: self.input_alias_count,
            max_depth_observed: self.max_depth_observed,
            node_table: self.node_table@,
            node_depths: self.node_depths@,
            visit_states: self.visit_states@,
            visit_order: self.visit_order@,
            deepest_path: self.deepest_path@,
        }
    }
}

impl AcyclicSemanticGraphSource {
    fn new(
        node_table: SemanticNodeTableSource,
        build: SemanticDepthBuild,
        deepest_path: Vec<u64>,
    ) -> (source: Self)
        ensures
            source@ == acyclic_semantic_graph_source_spec(node_table@, build@, deepest_path@),
    {
        Self {
            profile_version: node_table.profile_version(),
            transformation_version: ALIAS_CYCLE_RESOLUTION_VERSION,
            source_len_bytes: node_table.source_len_bytes(),
            input_node_count: node_table.nodes().len() as u64,
            input_alias_count: node_table.alias_redirects().len() as u64,
            max_depth_observed: build.max_depth_observed,
            node_table,
            node_depths: build.node_depths,
            visit_states: build.visit_states,
            visit_order: build.visit_order,
            deepest_path,
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

    pub fn input_alias_count(&self) -> (count: u64)
        ensures
            count == self@.input_alias_count,
    {
        self.input_alias_count
    }

    pub fn max_depth_observed(&self) -> (depth: u64)
        ensures
            depth == self@.max_depth_observed,
    {
        self.max_depth_observed
    }

    pub fn node_table(&self) -> (table: &SemanticNodeTableSource)
        ensures
            table@ == self@.node_table,
    {
        &self.node_table
    }

    pub fn node_depths(&self) -> (depths: &[u64])
        ensures
            depths@ == self@.node_depths,
    {
        self.node_depths.as_slice()
    }

    pub fn visit_states(&self) -> (states: &[SemanticVisitState])
        ensures
            states@ == self@.visit_states,
    {
        self.visit_states.as_slice()
    }

    pub fn visit_order(&self) -> (order: &[u64])
        ensures
            order@ == self@.visit_order,
    {
        self.visit_order.as_slice()
    }

    pub fn deepest_path(&self) -> (path: &[u64])
        ensures
            path@ == self@.deepest_path,
    {
        self.deepest_path.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum AliasCycleErrorKind {
    NodeTable(SemanticNodeTableErrorKind),
    AliasCycle,
    SemanticDepthLimitExceeded,
    WorkStackLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AliasCycleError {
    kind: AliasCycleErrorKind,
    byte_offset: u64,
    alias_node_index: Option<u64>,
    target_node_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct AliasCycleErrorView {
    pub kind: AliasCycleErrorKind,
    pub byte_offset: u64,
    pub alias_node_index: Option<u64>,
    pub target_node_index: Option<u64>,
}

impl View for AliasCycleError {
    type V = AliasCycleErrorView;

    closed spec fn view(&self) -> AliasCycleErrorView {
        AliasCycleErrorView {
            kind: self.kind,
            byte_offset: self.byte_offset,
            alias_node_index: self.alias_node_index,
            target_node_index: self.target_node_index,
        }
    }
}

impl AliasCycleError {
    fn at(kind: AliasCycleErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (AliasCycleErrorView {
                kind,
                byte_offset,
                alias_node_index: None,
                target_node_index: None,
            }),
    {
        Self { kind, byte_offset, alias_node_index: None, target_node_index: None }
    }

    fn cycle(redirect: &SemanticAliasRedirect) -> (error: Self)
        ensures
            error@ == alias_cycle_error_spec(redirect@),
    {
        Self {
            kind: AliasCycleErrorKind::AliasCycle,
            byte_offset: redirect.name_byte_start(),
            alias_node_index: Some(redirect.alias_node_index()),
            target_node_index: Some(redirect.target_node_index()),
        }
    }

    pub fn kind(&self) -> (kind: AliasCycleErrorKind)
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

    pub fn alias_node_index(&self) -> (index: Option<u64>)
        ensures
            index == self@.alias_node_index,
    {
        self.alias_node_index
    }

    pub fn target_node_index(&self) -> (index: Option<u64>)
        ensures
            index == self@.target_node_index,
    {
        self.target_node_index
    }
}

pub open spec fn map_semantic_node_table_error_spec(
    error: crate::resolve_node_table::SemanticNodeTableErrorView,
) -> AliasCycleErrorView {
    AliasCycleErrorView {
        kind: AliasCycleErrorKind::NodeTable(error.kind),
        byte_offset: error.byte_offset,
        alias_node_index: None,
        target_node_index: None,
    }
}

fn map_semantic_node_table_error(error: SemanticNodeTableError) -> (mapped: AliasCycleError)
    ensures
        mapped@ == map_semantic_node_table_error_spec(error@),
{
    AliasCycleError::at(AliasCycleErrorKind::NodeTable(error.kind()), error.byte_offset())
}

pub open spec fn alias_cycle_error_spec(
    redirect: SemanticAliasRedirectView,
) -> AliasCycleErrorView {
    AliasCycleErrorView {
        kind: AliasCycleErrorKind::AliasCycle,
        byte_offset: redirect.name_byte_start,
        alias_node_index: Some(redirect.alias_node_index),
        target_node_index: Some(redirect.target_node_index),
    }
}

pub open spec fn alias_redirect_targets_decrease_spec(
    redirects: Seq<SemanticAliasRedirectView>,
) -> bool {
    forall|index: int|
        0 <= index < redirects.len() ==> #[trigger] redirects[index].target_node_index
            < redirects[index].alias_node_index
}

pub closed spec fn first_nondecreasing_alias_redirect_spec(
    redirects: Seq<SemanticAliasRedirectView>,
    index: nat,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if index >= redirects.len() || fuel == 0 {
        None
    } else if redirects[index as int].target_node_index
        >= redirects[index as int].alias_node_index {
        Some(index as int)
    } else {
        first_nondecreasing_alias_redirect_spec(redirects, (index + 1) as nat, (fuel - 1) as nat)
    }
}

proof fn lemma_no_nondecreasing_redirect_from_index(
    redirects: Seq<SemanticAliasRedirectView>,
    index: nat,
    fuel: nat,
)
    requires
        index + fuel == redirects.len(),
        first_nondecreasing_alias_redirect_spec(redirects, index, fuel).is_none(),
    ensures
        forall|candidate: int|
            index <= candidate < redirects.len()
                ==> #[trigger] redirects[candidate].target_node_index
                < redirects[candidate].alias_node_index,
    decreases fuel,
{
    if fuel > 0 {
        reveal(first_nondecreasing_alias_redirect_spec);
        lemma_no_nondecreasing_redirect_from_index(
            redirects,
            (index + 1) as nat,
            (fuel - 1) as nat,
        );
    }
}

pub proof fn lemma_no_nondecreasing_redirect_means_all_decrease(
    redirects: Seq<SemanticAliasRedirectView>,
)
    requires
        first_nondecreasing_alias_redirect_spec(redirects, 0, redirects.len()).is_none(),
    ensures
        alias_redirect_targets_decrease_spec(redirects),
{
    lemma_no_nondecreasing_redirect_from_index(redirects, 0, redirects.len());
    reveal(alias_redirect_targets_decrease_spec);
}

fn first_nondecreasing_alias_redirect(redirects: &[SemanticAliasRedirect]) -> (found: Option<usize>)
    ensures
        first_nondecreasing_alias_redirect_spec(
            crate::resolve_node_table::semantic_alias_redirect_views_spec(redirects@),
            0,
            redirects@.len() as nat,
        ) == match found {
            Some(index) => Some(index as int),
            None => None,
        },
        match found {
            Some(index) => index < redirects@.len(),
            None => true,
        },
{
    let ghost views = crate::resolve_node_table::semantic_alias_redirect_views_spec(redirects@);
    let ghost expected = first_nondecreasing_alias_redirect_spec(views, 0, views.len());
    proof {
        reveal(crate::resolve_node_table::semantic_alias_redirect_views_spec);
        assert(views.len() == redirects@.len());
    }
    let mut index = 0usize;
    while index < redirects.len()
        invariant
            index <= redirects.len(),
            views.len() == redirects@.len(),
            expected == first_nondecreasing_alias_redirect_spec(views, 0, views.len()),
            views == crate::resolve_node_table::semantic_alias_redirect_views_spec(redirects@),
            expected == first_nondecreasing_alias_redirect_spec(
                views,
                index as nat,
                (redirects.len() - index) as nat,
            ),
        decreases redirects.len() - index,
    {
        proof {
            reveal(crate::resolve_node_table::semantic_alias_redirect_views_spec);
            assert(views[index as int] == redirects@[index as int]@);
        }
        if redirects[index].target_node_index() >= redirects[index].alias_node_index() {
            proof {
                reveal(first_nondecreasing_alias_redirect_spec);
                assert(views[index as int].target_node_index
                    >= views[index as int].alias_node_index);
                assert(expected == Some(index as int));
                assert(views.len() == redirects@.len());
                assert(first_nondecreasing_alias_redirect_spec(views, 0, views.len()) == Some(
                    index as int,
                ));
            }
            return Some(index);
        }
        proof {
            reveal(first_nondecreasing_alias_redirect_spec);
        }
        index += 1;
    }
    proof {
        reveal(first_nondecreasing_alias_redirect_spec);
    }
    None
}

pub open spec fn semantic_node_neighbor_count_spec(
    table: SemanticNodeTableSourceView,
    node_index: nat,
) -> nat {
    if node_index >= table.nodes.len() {
        0
    } else {
        let node = table.nodes[node_index as int];
        match node.kind {
            SemanticNodeKind::Scalar => 0,
            SemanticNodeKind::Alias => if node.alias_target_node_index.is_some() {
                1
            } else {
                0
            },
            SemanticNodeKind::Sequence => if node.edge_start <= node.edge_end && node.edge_end
                <= table.topology.sequence_edges.len() {
                (node.edge_end - node.edge_start) as nat
            } else {
                0
            },
            SemanticNodeKind::Mapping => if node.edge_start <= node.edge_end && node.edge_end
                <= table.topology.mapping_edges.len() && node.edge_end - node.edge_start
                <= usize::MAX / 2 {
                (2 * (node.edge_end - node.edge_start)) as nat
            } else {
                0
            },
        }
    }
}

pub open spec fn semantic_node_neighbor_spec(
    table: SemanticNodeTableSourceView,
    node_index: nat,
    ordinal: nat,
) -> Option<u64> {
    if node_index >= table.nodes.len() {
        None
    } else {
        let node = table.nodes[node_index as int];
        match node.kind {
            SemanticNodeKind::Scalar => None,
            SemanticNodeKind::Alias => if ordinal == 0 {
                node.alias_target_node_index
            } else {
                None
            },
            SemanticNodeKind::Sequence => if node.edge_start <= node.edge_end && node.edge_end
                <= table.topology.sequence_edges.len() && ordinal < node.edge_end
                - node.edge_start {
                Some(
                    table.topology.sequence_edges[(node.edge_start
                        + ordinal) as int].child_node_index,
                )
            } else {
                None
            },
            SemanticNodeKind::Mapping => if node.edge_start <= node.edge_end && node.edge_end
                <= table.topology.mapping_edges.len() && node.edge_end - node.edge_start
                <= usize::MAX / 2 && ordinal < 2 * (node.edge_end - node.edge_start) {
                let edge = table.topology.mapping_edges[(node.edge_start + ordinal / 2) as int];
                if ordinal % 2 == 0 {
                    Some(edge.key_node_index)
                } else {
                    Some(edge.value_node_index)
                }
            } else {
                None
            },
        }
    }
}

fn semantic_node_neighbor_count(table: &SemanticNodeTableSource, node_index: usize) -> (count:
    usize)
    requires
        node_index < table@.nodes.len(),
    ensures
        count as nat == semantic_node_neighbor_count_spec(table@, node_index as nat),
{
    let nodes = table.nodes();
    proof {
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(nodes@.len() == table@.nodes.len());
        assert(node_index < nodes@.len());
        assert(nodes@[node_index as int]@ == table@.nodes[node_index as int]);
    }
    let node = &nodes[node_index];
    let count = match node.kind() {
        SemanticNodeKind::Scalar => 0,
        SemanticNodeKind::Alias => if node.alias_target_node_index().is_some() {
            1
        } else {
            0
        },
        SemanticNodeKind::Sequence => {
            if node.edge_start() <= node.edge_end() && node.edge_end()
                <= table.topology().sequence_edges().len() as u64 {
                (node.edge_end() - node.edge_start()) as usize
            } else {
                0
            }
        },
        SemanticNodeKind::Mapping => {
            if node.edge_start() <= node.edge_end() && node.edge_end()
                <= table.topology().mapping_edges().len() as u64 && node.edge_end()
                - node.edge_start() <= (usize::MAX / 2) as u64 {
                (2 * (node.edge_end() - node.edge_start())) as usize
            } else {
                0
            }
        },
    };
    proof {
        reveal(semantic_node_neighbor_count_spec);
    }
    count
}

#[expect(clippy::manual_is_multiple_of, reason = "modulo spelling mirrors the verified key-value ordinal model")]  // `% 2` mirrors the pure key/value ordinal model.
fn semantic_node_neighbor(
    table: &SemanticNodeTableSource,
    node_index: usize,
    ordinal: usize,
) -> (neighbor: Option<u64>)
    requires
        node_index < table@.nodes.len(),
    ensures
        neighbor == semantic_node_neighbor_spec(table@, node_index as nat, ordinal as nat),
{
    let nodes = table.nodes();
    proof {
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(nodes@.len() == table@.nodes.len());
        assert(node_index < nodes@.len());
        assert(nodes@[node_index as int]@ == table@.nodes[node_index as int]);
    }
    let node = &nodes[node_index];
    let neighbor = match node.kind() {
        SemanticNodeKind::Scalar => None,
        SemanticNodeKind::Alias => if ordinal == 0 {
            node.alias_target_node_index()
        } else {
            None
        },
        SemanticNodeKind::Sequence => {
            if node.edge_start() <= node.edge_end() && node.edge_end()
                <= table.topology().sequence_edges().len() as u64 && (ordinal as u64)
                < node.edge_end() - node.edge_start() {
                Some(
                    table.topology().sequence_edges()[(node.edge_start() as usize)
                        + ordinal].child_node_index(),
                )
            } else {
                None
            }
        },
        SemanticNodeKind::Mapping => {
            if node.edge_start() <= node.edge_end() && node.edge_end()
                <= table.topology().mapping_edges().len() as u64 && node.edge_end()
                - node.edge_start() <= (usize::MAX / 2) as u64 && (ordinal as u64) < 2 * (
            node.edge_end() - node.edge_start()) {
                let edge = &table.topology().mapping_edges()[(node.edge_start() as usize) + ordinal
                    / 2];
                if ordinal % 2 == 0 {
                    Some(edge.key_node_index())
                } else {
                    Some(edge.value_node_index())
                }
            } else {
                None
            }
        },
    };
    proof {
        reveal(semantic_node_neighbor_spec);
    }
    neighbor
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SemanticChildDepth {
    depth: u64,
    child_node_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct SemanticChildDepthView {
    pub depth: nat,
    pub child_node_index: Option<u64>,
}

impl View for SemanticChildDepth {
    type V = SemanticChildDepthView;

    closed spec fn view(&self) -> SemanticChildDepthView {
        SemanticChildDepthView { depth: self.depth as nat, child_node_index: self.child_node_index }
    }
}

pub closed spec fn semantic_child_depth_tail_spec(
    table: SemanticNodeTableSourceView,
    node_depths: Seq<u64>,
    node_index: nat,
    ordinal: nat,
    fuel: nat,
    best: SemanticChildDepthView,
) -> Result<SemanticChildDepthView, AliasCycleErrorView>
    decreases fuel,
{
    let count = semantic_node_neighbor_count_spec(table, node_index);
    if ordinal >= count {
        Ok(best)
    } else if fuel == 0 || node_index >= table.nodes.len() {
        Err(
            AliasCycleErrorView {
                kind: AliasCycleErrorKind::InternalInvariantViolation,
                byte_offset: if node_index < table.nodes.len() {
                    table.nodes[node_index as int].byte_start
                } else {
                    table.source_len_bytes
                },
                alias_node_index: None,
                target_node_index: None,
            },
        )
    } else {
        match semantic_node_neighbor_spec(table, node_index, ordinal) {
            None => Err(
                AliasCycleErrorView {
                    kind: AliasCycleErrorKind::InternalInvariantViolation,
                    byte_offset: table.nodes[node_index as int].byte_start,
                    alias_node_index: None,
                    target_node_index: None,
                },
            ),
            Some(child) => if child >= node_depths.len() {
                Err(
                    AliasCycleErrorView {
                        kind: AliasCycleErrorKind::InternalInvariantViolation,
                        byte_offset: table.nodes[node_index as int].byte_start,
                        alias_node_index: None,
                        target_node_index: None,
                    },
                )
            } else {
                let child_depth = node_depths[child as int];
                semantic_child_depth_tail_spec(
                    table,
                    node_depths,
                    node_index,
                    (ordinal + 1) as nat,
                    (fuel - 1) as nat,
                    if child_depth > best.depth {
                        SemanticChildDepthView {
                            depth: child_depth as nat,
                            child_node_index: Some(child),
                        }
                    } else {
                        best
                    },
                )
            },
        }
    }
}

fn semantic_child_depth(
    table: &SemanticNodeTableSource,
    node_depths: &[u64],
    node_index: usize,
) -> (result: Result<SemanticChildDepth, AliasCycleError>)
    requires
        node_index < table@.nodes.len(),
    ensures
        semantic_child_depth_tail_spec(
            table@,
            node_depths@,
            node_index as nat,
            0,
            semantic_node_neighbor_count_spec(table@, node_index as nat),
            SemanticChildDepthView { depth: 0, child_node_index: None },
        ) == match result {
            Ok(depth) => Ok(depth@),
            Err(error) => Err(error@),
        },
{
    let nodes = table.nodes();
    proof {
        reveal(crate::resolve_node_table::semantic_node_slot_views_spec);
        assert(nodes@.len() == table@.nodes.len());
        assert(node_index < nodes@.len());
        assert(nodes@[node_index as int]@ == table@.nodes[node_index as int]);
    }
    let byte_offset = nodes[node_index].byte_start();
    proof {
        assert(byte_offset == table@.nodes[node_index as int].byte_start);
    }
    let count = semantic_node_neighbor_count(table, node_index);
    let ghost expected = semantic_child_depth_tail_spec(
        table@,
        node_depths@,
        node_index as nat,
        0,
        semantic_node_neighbor_count_spec(table@, node_index as nat),
        SemanticChildDepthView { depth: 0, child_node_index: None },
    );
    let mut ordinal = 0usize;
    let mut best = SemanticChildDepth { depth: 0, child_node_index: None };
    while ordinal < count
        invariant
            ordinal <= count,
            node_index < nodes.len(),
            nodes@.len() == table@.nodes.len(),
            byte_offset == table@.nodes[node_index as int].byte_start,
            count as nat == semantic_node_neighbor_count_spec(table@, node_index as nat),
            expected == semantic_child_depth_tail_spec(
                table@,
                node_depths@,
                node_index as nat,
                0,
                semantic_node_neighbor_count_spec(table@, node_index as nat),
                SemanticChildDepthView { depth: 0, child_node_index: None },
            ),
            expected == semantic_child_depth_tail_spec(
                table@,
                node_depths@,
                node_index as nat,
                ordinal as nat,
                (count - ordinal) as nat,
                best@,
            ),
        decreases count - ordinal,
    {
        let ghost neighbor = semantic_node_neighbor_spec(table@, node_index as nat, ordinal as nat);
        let child = match semantic_node_neighbor(table, node_index, ordinal) {
            Some(child) => child,
            None => {
                let error = AliasCycleError::at(
                    AliasCycleErrorKind::InternalInvariantViolation,
                    byte_offset,
                );
                proof {
                    reveal(semantic_child_depth_tail_spec);
                    assert(neighbor.is_none());
                    assert(expected == Err(error@));
                    assert(semantic_child_depth_tail_spec(
                        table@,
                        node_depths@,
                        node_index as nat,
                        0,
                        semantic_node_neighbor_count_spec(table@, node_index as nat),
                        SemanticChildDepthView { depth: 0, child_node_index: None },
                    ) == Err(error@));
                }
                return Err(error);
            },
        };
        if child >= node_depths.len() as u64 {
            let error = AliasCycleError::at(
                AliasCycleErrorKind::InternalInvariantViolation,
                byte_offset,
            );
            proof {
                reveal(semantic_child_depth_tail_spec);
                assert(neighbor == Some(child));
                assert(node_depths@.len() == node_depths.len());
                assert(node_depths.len() <= u64::MAX);
                assert(child >= node_depths@.len());
                assert(child as int >= node_depths@.len());
                assert(error@.byte_offset == table@.nodes[node_index as int].byte_start);
                assert(error@.kind == AliasCycleErrorKind::InternalInvariantViolation);
                assert(error@.alias_node_index.is_none());
                assert(error@.target_node_index.is_none());
                assert(expected == Err(error@));
                assert(semantic_child_depth_tail_spec(
                    table@,
                    node_depths@,
                    node_index as nat,
                    0,
                    semantic_node_neighbor_count_spec(table@, node_index as nat),
                    SemanticChildDepthView { depth: 0, child_node_index: None },
                ) == Err(error@));
            }
            return Err(error);
        }
        let child_depth = node_depths[child as usize];
        if child_depth > best.depth {
            best = SemanticChildDepth { depth: child_depth, child_node_index: Some(child) };
        }
        proof {
            reveal(semantic_child_depth_tail_spec);
        }
        ordinal += 1;
    }
    proof {
        reveal(semantic_child_depth_tail_spec);
    }
    Ok(best)
}

#[verifier::ext_equal]
pub struct SemanticDepthBuildView {
    pub node_depths: Seq<u64>,
    pub visit_states: Seq<SemanticVisitState>,
    pub visit_order: Seq<u64>,
    pub deepest_children: Seq<Option<u64>>,
    pub max_depth_observed: u64,
    pub max_depth_node: Option<u64>,
}

struct SemanticDepthBuild {
    node_depths: Vec<u64>,
    visit_states: Vec<SemanticVisitState>,
    visit_order: Vec<u64>,
    deepest_children: Vec<Option<u64>>,
    max_depth_observed: u64,
    max_depth_node: Option<u64>,
}

impl View for SemanticDepthBuild {
    type V = SemanticDepthBuildView;

    closed spec fn view(&self) -> SemanticDepthBuildView {
        SemanticDepthBuildView {
            node_depths: self.node_depths@,
            visit_states: self.visit_states@,
            visit_order: self.visit_order@,
            deepest_children: self.deepest_children@,
            max_depth_observed: self.max_depth_observed,
            max_depth_node: self.max_depth_node,
        }
    }
}

impl SemanticDepthBuild {
    fn empty() -> (build: Self)
        ensures
            build@ == empty_semantic_depth_build_spec(),
    {
        Self {
            node_depths: Vec::new(),
            visit_states: Vec::new(),
            visit_order: Vec::new(),
            deepest_children: Vec::new(),
            max_depth_observed: 0,
            max_depth_node: None,
        }
    }

    fn push(&mut self, node_index: u64, depth: u64, deepest_child: Option<u64>)
        ensures
            final(self)@ == semantic_depth_build_push_spec(
                old(self)@,
                node_index,
                depth,
                deepest_child,
            ),
    {
        self.node_depths.push(depth);
        self.visit_states.push(SemanticVisitState::Complete);
        self.visit_order.push(node_index);
        self.deepest_children.push(deepest_child);
        if depth > self.max_depth_observed {
            self.max_depth_observed = depth;
            self.max_depth_node = Some(node_index);
        }
    }
}

pub open spec fn empty_semantic_depth_build_spec() -> SemanticDepthBuildView {
    SemanticDepthBuildView {
        node_depths: Seq::empty(),
        visit_states: Seq::empty(),
        visit_order: Seq::empty(),
        deepest_children: Seq::empty(),
        max_depth_observed: 0,
        max_depth_node: None,
    }
}

pub open spec fn semantic_depth_build_push_spec(
    build: SemanticDepthBuildView,
    node_index: u64,
    depth: u64,
    deepest_child: Option<u64>,
) -> SemanticDepthBuildView {
    SemanticDepthBuildView {
        node_depths: build.node_depths.push(depth),
        visit_states: build.visit_states.push(SemanticVisitState::Complete),
        visit_order: build.visit_order.push(node_index),
        deepest_children: build.deepest_children.push(deepest_child),
        max_depth_observed: if depth > build.max_depth_observed {
            depth
        } else {
            build.max_depth_observed
        },
        max_depth_node: if depth > build.max_depth_observed {
            Some(node_index)
        } else {
            build.max_depth_node
        },
    }
}

pub open spec fn semantic_depth_node_step_spec(
    table: SemanticNodeTableSourceView,
    build: SemanticDepthBuildView,
    node_index: nat,
    limits: AliasCycleLimitsView,
) -> Result<SemanticDepthBuildView, AliasCycleErrorView> {
    if node_index >= table.nodes.len() || build.node_depths.len() != node_index {
        Err(
            AliasCycleErrorView {
                kind: AliasCycleErrorKind::InternalInvariantViolation,
                byte_offset: if node_index < table.nodes.len() {
                    table.nodes[node_index as int].byte_start
                } else {
                    table.source_len_bytes
                },
                alias_node_index: None,
                target_node_index: None,
            },
        )
    } else {
        match semantic_child_depth_tail_spec(
            table,
            build.node_depths,
            node_index,
            0,
            semantic_node_neighbor_count_spec(table, node_index),
            SemanticChildDepthView { depth: 0, child_node_index: None },
        ) {
            Err(error) => Err(error),
            Ok(child) => {
                if child.depth >= u64::MAX {
                    Err(
                        AliasCycleErrorView {
                            kind: AliasCycleErrorKind::InternalInvariantViolation,
                            byte_offset: table.nodes[node_index as int].byte_start,
                            alias_node_index: None,
                            target_node_index: None,
                        },
                    )
                } else {
                    let depth = child.depth + 1;
                    if depth > effective_alias_cycle_depth_limit_spec(limits) {
                        Err(
                            AliasCycleErrorView {
                                kind: AliasCycleErrorKind::SemanticDepthLimitExceeded,
                                byte_offset: table.nodes[node_index as int].byte_start,
                                alias_node_index: None,
                                target_node_index: None,
                            },
                        )
                    } else if depth > effective_alias_cycle_work_stack_limit_spec(limits) {
                        Err(
                            AliasCycleErrorView {
                                kind: AliasCycleErrorKind::WorkStackLimitExceeded,
                                byte_offset: table.nodes[node_index as int].byte_start,
                                alias_node_index: None,
                                target_node_index: None,
                            },
                        )
                    } else {
                        Ok(
                            semantic_depth_build_push_spec(
                                build,
                                node_index as u64,
                                depth as u64,
                                child.child_node_index,
                            ),
                        )
                    }
                }
            },
        }
    }
}

pub closed spec fn semantic_depth_table_tail_spec(
    table: SemanticNodeTableSourceView,
    node_index: nat,
    fuel: nat,
    build: SemanticDepthBuildView,
    limits: AliasCycleLimitsView,
) -> Result<SemanticDepthBuildView, AliasCycleErrorView>
    decreases fuel,
{
    if node_index >= table.nodes.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            AliasCycleErrorView {
                kind: AliasCycleErrorKind::InternalInvariantViolation,
                byte_offset: table.nodes[node_index as int].byte_start,
                alias_node_index: None,
                target_node_index: None,
            },
        )
    } else {
        match semantic_depth_node_step_spec(table, build, node_index, limits) {
            Err(error) => Err(error),
            Ok(next) => semantic_depth_table_tail_spec(
                table,
                (node_index + 1) as nat,
                (fuel - 1) as nat,
                next,
                limits,
            ),
        }
    }
}

fn semantic_depth_node_step(
    table: &SemanticNodeTableSource,
    build: &SemanticDepthBuild,
    node_index: usize,
    limits: AliasCycleLimits,
) -> (result: Result<SemanticChildDepth, AliasCycleError>)
    requires
        node_index < table@.nodes.len(),
        build@.node_depths.len() == node_index,
    ensures
        semantic_depth_node_step_spec(table@, build@, node_index as nat, limits@) == match result {
            Ok(child) => Ok(
                semantic_depth_build_push_spec(
                    build@,
                    node_index as u64,
                    (child.depth + 1) as u64,
                    child.child_node_index,
                ),
            ),
            Err(error) => Err(error@),
        },
        match result {
            Ok(child) => child.depth < u64::MAX,
            Err(_) => true,
        },
{
    let child = semantic_child_depth(table, build.node_depths.as_slice(), node_index)?;
    if child.depth == u64::MAX {
        return Err(
            AliasCycleError::at(
                AliasCycleErrorKind::InternalInvariantViolation,
                table.nodes()[node_index].byte_start(),
            ),
        );
    }
    let depth = child.depth + 1;
    if depth > effective_alias_cycle_depth_limit(limits) {
        return Err(
            AliasCycleError::at(
                AliasCycleErrorKind::SemanticDepthLimitExceeded,
                table.nodes()[node_index].byte_start(),
            ),
        );
    }
    if depth > effective_alias_cycle_work_stack_limit(limits) {
        return Err(
            AliasCycleError::at(
                AliasCycleErrorKind::WorkStackLimitExceeded,
                table.nodes()[node_index].byte_start(),
            ),
        );
    }
    assert(child.depth < u64::MAX);
    Ok(child)
}

pub closed spec fn semantic_deepest_path_tail_spec(
    deepest_children: Seq<Option<u64>>,
    current: Option<u64>,
    fuel: nat,
    path: Seq<u64>,
    source_len_bytes: u64,
) -> Result<Seq<u64>, AliasCycleErrorView>
    decreases fuel,
{
    match current {
        None => Ok(path),
        Some(node_index) => if fuel == 0 || node_index >= deepest_children.len() {
            Err(
                AliasCycleErrorView {
                    kind: AliasCycleErrorKind::InternalInvariantViolation,
                    byte_offset: source_len_bytes,
                    alias_node_index: None,
                    target_node_index: None,
                },
            )
        } else {
            let next = deepest_children[node_index as int];
            if next.is_some() && next.unwrap() >= node_index {
                Err(
                    AliasCycleErrorView {
                        kind: AliasCycleErrorKind::InternalInvariantViolation,
                        byte_offset: source_len_bytes,
                        alias_node_index: None,
                        target_node_index: None,
                    },
                )
            } else {
                semantic_deepest_path_tail_spec(
                    deepest_children,
                    next,
                    (fuel - 1) as nat,
                    path.push(node_index),
                    source_len_bytes,
                )
            }
        },
    }
}

fn semantic_deepest_path(
    deepest_children: &[Option<u64>],
    start: Option<u64>,
    source_len_bytes: u64,
) -> (result: Result<Vec<u64>, AliasCycleError>)
    ensures
        semantic_deepest_path_tail_spec(
            deepest_children@,
            start,
            deepest_children@.len() as nat,
            Seq::empty(),
            source_len_bytes,
        ) == match result {
            Ok(path) => Ok(path@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = semantic_deepest_path_tail_spec(
        deepest_children@,
        start,
        deepest_children@.len() as nat,
        Seq::empty(),
        source_len_bytes,
    );
    let mut path = Vec::new();
    let mut current = start;
    let mut fuel = deepest_children.len();
    while current.is_some()
        invariant
            fuel <= deepest_children.len(),
            expected == semantic_deepest_path_tail_spec(
                deepest_children@,
                start,
                deepest_children@.len() as nat,
                Seq::empty(),
                source_len_bytes,
            ),
            expected == semantic_deepest_path_tail_spec(
                deepest_children@,
                current,
                fuel as nat,
                path@,
                source_len_bytes,
            ),
        decreases fuel,
    {
        let node_index = current.unwrap();
        if fuel == 0 || node_index >= deepest_children.len() as u64 {
            let error = AliasCycleError::at(
                AliasCycleErrorKind::InternalInvariantViolation,
                source_len_bytes,
            );
            proof {
                reveal(semantic_deepest_path_tail_spec);
                assert(error@.kind == AliasCycleErrorKind::InternalInvariantViolation);
                assert(error@.byte_offset == source_len_bytes);
                assert(error@.alias_node_index.is_none());
                assert(error@.target_node_index.is_none());
                if fuel == 0 {
                    assert(expected == Err(error@));
                } else {
                    assert(deepest_children.len() <= u64::MAX);
                    assert(node_index >= deepest_children@.len());
                    assert(node_index as int >= deepest_children@.len());
                    assert(expected == Err(error@));
                }
                assert(semantic_deepest_path_tail_spec(
                    deepest_children@,
                    start,
                    deepest_children@.len() as nat,
                    Seq::empty(),
                    source_len_bytes,
                ) == Err(error@));
            }
            return Err(error);
        }
        let next = deepest_children[node_index as usize];
        proof {
            assert(deepest_children@[node_index as int] == next);
        }
        if let Some(next_index) = next {
            if next_index >= node_index {
                let error = AliasCycleError::at(
                    AliasCycleErrorKind::InternalInvariantViolation,
                    source_len_bytes,
                );
                proof {
                    reveal(semantic_deepest_path_tail_spec);
                    assert(deepest_children@[node_index as int] == Some(next_index));
                    assert(error@.kind == AliasCycleErrorKind::InternalInvariantViolation);
                    assert(error@.byte_offset == source_len_bytes);
                    assert(error@.alias_node_index.is_none());
                    assert(error@.target_node_index.is_none());
                    assert(expected == Err(error@));
                    assert(semantic_deepest_path_tail_spec(
                        deepest_children@,
                        start,
                        deepest_children@.len() as nat,
                        Seq::empty(),
                        source_len_bytes,
                    ) == Err(error@));
                }
                return Err(error);
            }
        }
        proof {
            reveal(semantic_deepest_path_tail_spec);
        }
        path.push(node_index);
        current = next;
        fuel -= 1;
    }
    proof {
        reveal(semantic_deepest_path_tail_spec);
    }
    Ok(path)
}

pub open spec fn acyclic_semantic_graph_source_spec(
    table: SemanticNodeTableSourceView,
    build: SemanticDepthBuildView,
    deepest_path: Seq<u64>,
) -> AcyclicSemanticGraphSourceView {
    AcyclicSemanticGraphSourceView {
        profile_version: table.profile_version,
        transformation_version: ALIAS_CYCLE_RESOLUTION_VERSION,
        source_len_bytes: table.source_len_bytes,
        input_node_count: table.nodes.len() as u64,
        input_alias_count: table.alias_redirects.len() as u64,
        max_depth_observed: build.max_depth_observed,
        node_table: table,
        node_depths: build.node_depths,
        visit_states: build.visit_states,
        visit_order: build.visit_order,
        deepest_path,
    }
}

pub open spec fn semantic_graph_edges_strictly_decrease_spec(
    table: SemanticNodeTableSourceView,
) -> bool {
    alias_redirect_targets_decrease_spec(table.alias_redirects) && forall|node_index: int|
        #![trigger table.topology.nodes[node_index]]
        0 <= node_index < table.topology.nodes.len() ==> {
            let node = table.topology.nodes[node_index];
            &&& node.cst_node_index == node_index as u64
            &&& (node.kind != crate::cst::CstNodeKind::Sequence || (node.edge_start <= node.edge_end
                <= table.topology.sequence_edges.len() && forall|edge_index: int|
                node.edge_start <= edge_index < node.edge_end
                    ==> #[trigger] table.topology.sequence_edges[edge_index].child_node_index
                    < node_index))
            &&& (node.kind != crate::cst::CstNodeKind::Mapping || (node.edge_start <= node.edge_end
                <= table.topology.mapping_edges.len() && forall|edge_index: int|
                node.edge_start <= edge_index < node.edge_end
                    ==> #[trigger] table.topology.mapping_edges[edge_index].key_node_index
                    < node_index && table.topology.mapping_edges[edge_index].value_node_index
                    < node_index))
        }
}

pub open spec fn finalize_alias_cycle_depth_spec(
    table: SemanticNodeTableSourceView,
    result: Result<SemanticDepthBuildView, AliasCycleErrorView>,
) -> Result<AcyclicSemanticGraphSourceView, AliasCycleErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => match semantic_deepest_path_tail_spec(
            build.deepest_children,
            build.max_depth_node,
            build.deepest_children.len(),
            Seq::empty(),
            table.source_len_bytes,
        ) {
            Err(error) => Err(error),
            Ok(path) => Ok(acyclic_semantic_graph_source_spec(table, build, path)),
        },
    }
}

pub open spec fn resolve_profile1_alias_cycles_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    node_limits: SemanticNodeTableLimitsView,
    cycle_limits: AliasCycleLimitsView,
) -> Result<AcyclicSemanticGraphSourceView, AliasCycleErrorView> {
    match crate::resolve_node_table::compose_profile1_semantic_node_table_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
    ) {
        Err(error) => Err(map_semantic_node_table_error_spec(error)),
        Ok(table) => {
            let cycle = first_nondecreasing_alias_redirect_spec(
                table.alias_redirects,
                0,
                table.alias_redirects.len(),
            );
            match cycle {
                Some(index) => Err(alias_cycle_error_spec(table.alias_redirects[index])),
                None => finalize_alias_cycle_depth_spec(
                    table,
                    semantic_depth_table_tail_spec(
                        table,
                        0,
                        table.nodes.len(),
                        empty_semantic_depth_build_spec(),
                        cycle_limits,
                    ),
                ),
            }
        },
    }
}

pub open spec fn acyclic_semantic_graph_source_well_formed_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    node_limits: SemanticNodeTableLimitsView,
    cycle_limits: AliasCycleLimitsView,
    source: AcyclicSemanticGraphSourceView,
) -> bool {
    crate::cst::cst_public_semantics_spec(completed, cst) && resolve_profile1_alias_cycles_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
        cycle_limits,
    ) == Ok(source) && semantic_graph_edges_strictly_decrease_spec(source.node_table)
}

pub proof fn lemma_alias_cycle_success_is_well_formed_and_acyclic(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    node_limits: SemanticNodeTableLimitsView,
    cycle_limits: AliasCycleLimitsView,
    source: AcyclicSemanticGraphSourceView,
)
    requires
        crate::cst::cst_public_semantics_spec(completed, cst),
        resolve_profile1_alias_cycles_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
        ) == Ok(source),
    ensures
        acyclic_semantic_graph_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
            source,
        ),
        semantic_graph_edges_strictly_decrease_spec(source.node_table),
{
    reveal(resolve_profile1_alias_cycles_spec);
    reveal(acyclic_semantic_graph_source_well_formed_spec);
    reveal(semantic_graph_edges_strictly_decrease_spec);
    reveal(alias_redirect_targets_decrease_spec);
    reveal(first_nondecreasing_alias_redirect_spec);

    let table_result = crate::resolve_node_table::compose_profile1_semantic_node_table_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
    );
    let table = match table_result {
        Ok(table) => table,
        Err(_) => {
            assert(false);
            source.node_table
        },
    };
    crate::resolve_node_table::lemma_semantic_node_table_success_is_well_formed(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
        table,
    );
    reveal(crate::resolve_node_table::compose_profile1_semantic_node_table_spec);
    crate::resolve_topology::lemma_semantic_topology_success_is_exact_and_well_formed(
        atomized,
        completed,
        cst,
        topology_limits,
        table.topology,
    );
    crate::resolve_topology::lemma_semantic_topology_well_formed_authenticates_cst(
        completed,
        cst,
        table.topology,
    );
    lemma_no_nondecreasing_redirect_means_all_decrease(table.alias_redirects);
    reveal(crate::resolve_topology::semantic_topology_source_well_formed_spec);
    reveal(crate::resolve_topology::semantic_topology_exact_source_spec);
    reveal(crate::resolve_topology::semantic_topology_nodes_spec);
    reveal(crate::resolve_topology::semantic_sequence_edges_spec);
    reveal(crate::resolve_topology::semantic_mapping_edges_spec);
}

pub proof fn lemma_acyclic_semantic_graph_well_formed_authenticates_exact_result(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    topology_limits: SemanticTopologyLimitsView,
    scalar_limits: SemanticScalarTableLimitsView,
    anchor_limits: AnchorAliasLimitsView,
    node_limits: SemanticNodeTableLimitsView,
    cycle_limits: AliasCycleLimitsView,
    source: AcyclicSemanticGraphSourceView,
)
    requires
        acyclic_semantic_graph_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
            source,
        ),
    ensures
        resolve_profile1_alias_cycles_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            topology_limits,
            scalar_limits,
            anchor_limits,
            node_limits,
            cycle_limits,
        ) == Ok(source),
{
    reveal(acyclic_semantic_graph_source_well_formed_spec);
}

#[expect(clippy::too_many_arguments, reason = "independent proof inputs remain explicit in the executable-to-spec contract")]
pub fn resolve_profile1_alias_cycles(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    topology_limits: SemanticTopologyLimits,
    scalar_limits: SemanticScalarTableLimits,
    anchor_limits: AnchorAliasLimits,
    node_limits: SemanticNodeTableLimits,
    cycle_limits: AliasCycleLimits,
) -> (result: Result<AcyclicSemanticGraphSource, AliasCycleError>)
    ensures
        resolve_profile1_alias_cycles_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            topology_limits@,
            scalar_limits@,
            anchor_limits@,
            node_limits@,
            cycle_limits@,
        ) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let table = match crate::resolve_node_table::compose_profile1_semantic_node_table(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        topology_limits,
        scalar_limits,
        anchor_limits,
        node_limits,
    ) {
        Err(error) => return Err(map_semantic_node_table_error(error)),
        Ok(table) => table,
    };
    let redirects = table.alias_redirects();
    let cycle = first_nondecreasing_alias_redirect(redirects);
    if let Some(index) = cycle {
        proof {
            reveal(crate::resolve_node_table::semantic_alias_redirect_views_spec);
            reveal(resolve_profile1_alias_cycles_spec);
        }
        return Err(AliasCycleError::cycle(&redirects[index]));
    }
    let mut build = SemanticDepthBuild::empty();
    let ghost expected_depths = semantic_depth_table_tail_spec(
        table@,
        0,
        table@.nodes.len(),
        build@,
        cycle_limits@,
    );
    proof {
        reveal(resolve_profile1_alias_cycles_spec);
        assert(first_nondecreasing_alias_redirect_spec(
            table@.alias_redirects,
            0,
            table@.alias_redirects.len(),
        ).is_none());
        assert(resolve_profile1_alias_cycles_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            topology_limits@,
            scalar_limits@,
            anchor_limits@,
            node_limits@,
            cycle_limits@,
        ) == finalize_alias_cycle_depth_spec(table@, expected_depths));
    }
    let node_count = table.nodes().len();
    let mut node_index = 0usize;
    while node_index < node_count
        invariant
            node_index <= node_count,
            node_count == table@.nodes.len(),
            build@.node_depths.len() == node_index,
            expected_depths == semantic_depth_table_tail_spec(
                table@,
                node_index as nat,
                (node_count - node_index) as nat,
                build@,
                cycle_limits@,
            ),
            resolve_profile1_alias_cycles_spec(
                atomized@,
                quoted@,
                plain@,
                block@,
                completed@,
                cst@,
                topology_limits@,
                scalar_limits@,
                anchor_limits@,
                node_limits@,
                cycle_limits@,
            ) == finalize_alias_cycle_depth_spec(table@, expected_depths),
        decreases node_count - node_index,
    {
        let child = match semantic_depth_node_step(&table, &build, node_index, cycle_limits) {
            Err(error) => {
                proof {
                    reveal(semantic_depth_table_tail_spec);
                    reveal(finalize_alias_cycle_depth_spec);
                }
                return Err(error);
            },
            Ok(child) => child,
        };
        proof {
            assert(semantic_depth_node_step_spec(table@, build@, node_index as nat, cycle_limits@)
                == Ok(
                semantic_depth_build_push_spec(
                    build@,
                    node_index as u64,
                    (child.depth + 1) as u64,
                    child.child_node_index,
                ),
            ));
            assert(child@.depth == child.depth as nat);
            reveal(semantic_depth_node_step_spec);
            assert(child.depth < u64::MAX);
        }
        let depth = child.depth + 1;
        proof {
            reveal(semantic_depth_table_tail_spec);
        }
        build.push(node_index as u64, depth, child.child_node_index);
        node_index += 1;
    }
    proof {
        reveal(semantic_depth_table_tail_spec);
    }
    let deepest_path = match semantic_deepest_path(
        build.deepest_children.as_slice(),
        build.max_depth_node,
        table.source_len_bytes(),
    ) {
        Err(error) => {
            proof {
                reveal(finalize_alias_cycle_depth_spec);
            }
            return Err(error);
        },
        Ok(path) => path,
    };
    let source = AcyclicSemanticGraphSource::new(table, build, deepest_path);
    proof {
        reveal(finalize_alias_cycle_depth_spec);
    }
    Ok(source)
}

} // verus!
