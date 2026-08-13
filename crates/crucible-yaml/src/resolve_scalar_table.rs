//! Verified aggregate scalar-value table for authenticated CST nodes.
//!
//! Entries retain their exact CST node indices, and aggregate accounting uses decoded content
//! code points rather than presentation bytes.
use crate::atom::AtomizedSource;
#[allow(unused_imports)]
use crate::atom::AtomizedSourceView;
use crate::block::BlockScalarSource;
#[allow(unused_imports)]
use crate::block::BlockScalarSourceView;
use crate::cst::{CstNodeKind, CstSource};
#[allow(unused_imports)]
use crate::cst::{CstNodeView, CstSourceView};
use crate::plain::PlainScalarSource;
#[allow(unused_imports)]
use crate::plain::PlainScalarSourceView;
use crate::quoted::QuotedScalarSource;
#[allow(unused_imports)]
use crate::quoted::QuotedScalarSourceView;
use crate::resolve_scalar_value::{
    resolve_profile1_cst_node_scalar_value, ResolvedScalar, ScalarValueError, ScalarValueErrorKind,
    ScalarValueLimits,
};
#[allow(unused_imports)]
use crate::resolve_scalar_value::{
    ResolvedScalarView, ScalarValueErrorView, ScalarValueLimitsView,
};
use crate::token::CompletedTokenSource;
#[allow(unused_imports)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const SEMANTIC_SCALAR_TABLE_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticScalarTableLimits {
    max_scalars: u64,
    max_total_content_code_points: u64,
    max_scalar_content_code_points: u64,
    max_tag_code_points: u64,
    max_integer_limbs: u64,
    max_float_coefficient_digits: u64,
    max_float_exponent_digits: u64,
}

#[verifier::ext_equal]
pub struct SemanticScalarTableLimitsView {
    pub max_scalars: u64,
    pub max_total_content_code_points: u64,
    pub max_scalar_content_code_points: u64,
    pub max_tag_code_points: u64,
    pub max_integer_limbs: u64,
    pub max_float_coefficient_digits: u64,
    pub max_float_exponent_digits: u64,
}

impl View for SemanticScalarTableLimits {
    type V = SemanticScalarTableLimitsView;

    closed spec fn view(&self) -> SemanticScalarTableLimitsView {
        SemanticScalarTableLimitsView {
            max_scalars: self.max_scalars,
            max_total_content_code_points: self.max_total_content_code_points,
            max_scalar_content_code_points: self.max_scalar_content_code_points,
            max_tag_code_points: self.max_tag_code_points,
            max_integer_limbs: self.max_integer_limbs,
            max_float_coefficient_digits: self.max_float_coefficient_digits,
            max_float_exponent_digits: self.max_float_exponent_digits,
        }
    }
}

impl SemanticScalarTableLimits {
    pub fn new(
        max_scalars: u64,
        max_total_content_code_points: u64,
        max_scalar_content_code_points: u64,
        max_tag_code_points: u64,
        max_integer_limbs: u64,
        max_float_coefficient_digits: u64,
        max_float_exponent_digits: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (SemanticScalarTableLimitsView {
                max_scalars,
                max_total_content_code_points,
                max_scalar_content_code_points,
                max_tag_code_points,
                max_integer_limbs,
                max_float_coefficient_digits,
                max_float_exponent_digits,
            }),
    {
        Self {
            max_scalars,
            max_total_content_code_points,
            max_scalar_content_code_points,
            max_tag_code_points,
            max_integer_limbs,
            max_float_coefficient_digits,
            max_float_exponent_digits,
        }
    }

    pub fn max_scalars(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_scalars,
    {
        self.max_scalars
    }

    pub fn max_total_content_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_total_content_code_points,
    {
        self.max_total_content_code_points
    }

    pub fn scalar_value_limits(&self) -> (limits: ScalarValueLimits)
        ensures
            limits@ == semantic_scalar_value_limits_spec(self@),
    {
        ScalarValueLimits::new(
            self.max_scalar_content_code_points,
            self.max_tag_code_points,
            self.max_integer_limbs,
            self.max_float_coefficient_digits,
            self.max_float_exponent_digits,
        )
    }
}

pub fn canonical_semantic_scalar_table_limits() -> (limits: SemanticScalarTableLimits)
    ensures
        limits@ == canonical_semantic_scalar_table_limits_spec(),
{
    SemanticScalarTableLimits::new(
        crate::cst::MAX_PROFILE1_CST_NODES,
        MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
        crate::scalar_decode::MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
        crate::resolve_tag::MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
        crate::resolve_integer::MAX_PROFILE1_CORE_INTEGER_LIMBS,
        crate::resolve_float::MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
        crate::resolve_float::MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    )
}

pub open spec fn canonical_semantic_scalar_table_limits_spec() -> SemanticScalarTableLimitsView {
    SemanticScalarTableLimitsView {
        max_scalars: crate::cst::MAX_PROFILE1_CST_NODES,
        max_total_content_code_points: MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
        max_scalar_content_code_points:
            crate::scalar_decode::MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS,
        max_tag_code_points: crate::resolve_tag::MAX_PROFILE1_RESOLVED_TAG_CODE_POINTS,
        max_integer_limbs: crate::resolve_integer::MAX_PROFILE1_CORE_INTEGER_LIMBS,
        max_float_coefficient_digits:
            crate::resolve_float::MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
        max_float_exponent_digits: crate::resolve_float::MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    }
}

pub open spec fn semantic_scalar_value_limits_spec(
    limits: SemanticScalarTableLimitsView,
) -> ScalarValueLimitsView {
    ScalarValueLimitsView {
        max_content_code_points: limits.max_scalar_content_code_points,
        max_tag_code_points: limits.max_tag_code_points,
        max_integer_limbs: limits.max_integer_limbs,
        max_float_coefficient_digits: limits.max_float_coefficient_digits,
        max_float_exponent_digits: limits.max_float_exponent_digits,
    }
}

pub open spec fn semantic_scalar_table_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn semantic_scalar_table_effective_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit == semantic_scalar_table_effective_limit_spec(requested, absolute),
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

pub open spec fn semantic_scalar_views_spec(values: Seq<ResolvedScalar>) -> Seq<
    ResolvedScalarView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

proof fn lemma_semantic_scalar_views_push(values: Seq<ResolvedScalar>, value: ResolvedScalar)
    ensures
        semantic_scalar_views_spec(values.push(value)) == semantic_scalar_views_spec(values).push(
            value@,
        ),
{
    reveal(semantic_scalar_views_spec);
    assert(semantic_scalar_views_spec(values.push(value)) =~= semantic_scalar_views_spec(
        values,
    ).push(value@));
}

pub open spec fn semantic_scalar_content_spec(scalar: ResolvedScalarView) -> Seq<
    crate::scalar_decode::DecodedContentScalarView,
> {
    match scalar.presentation.decoded {
        Some(decoded) => decoded.content,
        None => Seq::empty(),
    }
}

pub open spec fn semantic_scalar_table_scalar_node_indices_spec(
    scalars: Seq<ResolvedScalarView>,
) -> Seq<u64> {
    Seq::new(scalars.len(), |index: int| scalars[index].node_index)
}

proof fn lemma_semantic_scalar_table_scalar_node_indices_push(
    scalars: Seq<ResolvedScalarView>,
    scalar: ResolvedScalarView,
)
    ensures
        semantic_scalar_table_scalar_node_indices_spec(scalars.push(scalar))
            == semantic_scalar_table_scalar_node_indices_spec(scalars).push(scalar.node_index),
{
    reveal(semantic_scalar_table_scalar_node_indices_spec);
    assert(semantic_scalar_table_scalar_node_indices_spec(scalars.push(scalar))
        =~= semantic_scalar_table_scalar_node_indices_spec(scalars).push(scalar.node_index));
}

pub closed spec fn semantic_scalar_table_expected_node_indices_prefix_spec(
    nodes: Seq<CstNodeView>,
    end: nat,
    fuel: nat,
) -> Seq<u64>
    decreases fuel,
{
    if fuel == 0 || end == 0 || end > nodes.len() {
        Seq::empty()
    } else {
        let prior = semantic_scalar_table_expected_node_indices_prefix_spec(
            nodes,
            (end - 1) as nat,
            (fuel - 1) as nat,
        );
        let node = nodes[end - 1];
        if node.kind == CstNodeKind::Empty || node.kind == CstNodeKind::Scalar {
            prior.push((end - 1) as u64)
        } else {
            prior
        }
    }
}

pub open spec fn semantic_scalar_table_expected_node_indices_spec(nodes: Seq<CstNodeView>) -> Seq<
    u64,
> {
    semantic_scalar_table_expected_node_indices_prefix_spec(nodes, nodes.len(), nodes.len())
}

pub closed spec fn semantic_scalar_table_total_content_tail_spec(
    scalars: Seq<ResolvedScalarView>,
    index: nat,
    fuel: nat,
) -> int
    decreases fuel,
{
    if fuel == 0 || index >= scalars.len() {
        0
    } else {
        semantic_scalar_content_spec(scalars[index as int]).len() as int
            + semantic_scalar_table_total_content_tail_spec(
            scalars,
            (index + 1) as nat,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn semantic_scalar_table_total_content_spec(scalars: Seq<ResolvedScalarView>) -> int {
    semantic_scalar_table_total_content_tail_spec(scalars, 0, scalars.len())
}

proof fn lemma_semantic_scalar_table_total_content_tail_push(
    scalars: Seq<ResolvedScalarView>,
    scalar: ResolvedScalarView,
    index: nat,
    fuel: nat,
)
    requires
        index <= scalars.len(),
        fuel == scalars.len() - index,
    ensures
        semantic_scalar_table_total_content_tail_spec(
            scalars.push(scalar),
            index,
            (fuel + 1) as nat,
        ) == semantic_scalar_table_total_content_tail_spec(scalars, index, fuel)
            + semantic_scalar_content_spec(scalar).len() as int,
    decreases fuel,
{
    if fuel == 0 {
        assert(index == scalars.len());
        assert(scalars.push(scalar)[index as int] == scalar);
        reveal(semantic_scalar_table_total_content_tail_spec);
        assert(semantic_scalar_table_total_content_tail_spec(scalars, index, 0) == 0);
        assert(semantic_scalar_table_total_content_tail_spec(
            scalars.push(scalar),
            (index + 1) as nat,
            0,
        ) == 0);
    } else {
        assert(index < scalars.len());
        lemma_semantic_scalar_table_total_content_tail_push(
            scalars,
            scalar,
            (index + 1) as nat,
            (fuel - 1) as nat,
        );
        assert(scalars.push(scalar)[index as int] == scalars[index as int]);
        reveal(semantic_scalar_table_total_content_tail_spec);
        let head = semantic_scalar_content_spec(scalars[index as int]).len() as int;
        let tail = semantic_scalar_table_total_content_tail_spec(
            scalars,
            (index + 1) as nat,
            (fuel - 1) as nat,
        );
        let added = semantic_scalar_content_spec(scalar).len() as int;
        assert(head + (tail + added) == (head + tail) + added);
    }
}

proof fn lemma_semantic_scalar_table_total_content_push(
    scalars: Seq<ResolvedScalarView>,
    scalar: ResolvedScalarView,
)
    ensures
        semantic_scalar_table_total_content_spec(scalars.push(scalar))
            == semantic_scalar_table_total_content_spec(scalars) + semantic_scalar_content_spec(
            scalar,
        ).len() as int,
{
    reveal(semantic_scalar_table_total_content_spec);
    lemma_semantic_scalar_table_total_content_tail_push(scalars, scalar, 0, scalars.len());
}

#[derive(Debug, PartialEq, Eq)]
pub struct SemanticScalarTableSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_token_transformation_version: u16,
    input_cst_transformation_version: u16,
    input_node_count: u64,
    total_content_code_points: u64,
    scalars: Vec<ResolvedScalar>,
}

#[verifier::ext_equal]
pub struct SemanticScalarTableSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_token_transformation_version: u16,
    pub input_cst_transformation_version: u16,
    pub input_node_count: u64,
    pub total_content_code_points: u64,
    pub scalars: Seq<ResolvedScalarView>,
}

impl View for SemanticScalarTableSource {
    type V = SemanticScalarTableSourceView;

    closed spec fn view(&self) -> SemanticScalarTableSourceView {
        SemanticScalarTableSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_token_transformation_version: self.input_token_transformation_version,
            input_cst_transformation_version: self.input_cst_transformation_version,
            input_node_count: self.input_node_count,
            total_content_code_points: self.total_content_code_points,
            scalars: semantic_scalar_views_spec(self.scalars@),
        }
    }
}

impl SemanticScalarTableSource {
    fn new(
        completed: &CompletedTokenSource,
        cst: &CstSource,
        total_content_code_points: u64,
        scalars: Vec<ResolvedScalar>,
    ) -> (source: Self)
        ensures
            source@ == (SemanticScalarTableSourceView {
                profile_version: completed@.profile_version,
                transformation_version: SEMANTIC_SCALAR_TABLE_TRANSFORMATION_VERSION,
                source_len_bytes: completed@.source_len_bytes,
                input_token_transformation_version: completed@.transformation_version,
                input_cst_transformation_version: cst@.transformation_version,
                input_node_count: cst@.nodes.len() as u64,
                total_content_code_points,
                scalars: semantic_scalar_views_spec(scalars@),
            }),
    {
        Self {
            profile_version: completed.profile_version(),
            transformation_version: SEMANTIC_SCALAR_TABLE_TRANSFORMATION_VERSION,
            source_len_bytes: completed.source_len_bytes(),
            input_token_transformation_version: completed.transformation_version(),
            input_cst_transformation_version: cst.transformation_version(),
            input_node_count: cst.nodes().len() as u64,
            total_content_code_points,
            scalars,
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

    pub fn total_content_code_points(&self) -> (count: u64)
        ensures
            count == self@.total_content_code_points,
    {
        self.total_content_code_points
    }

    pub fn scalars(&self) -> (values: &[ResolvedScalar])
        ensures
            semantic_scalar_views_spec(values@) == self@.scalars,
    {
        self.scalars.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum SemanticScalarTableErrorKind {
    ScalarValue(ScalarValueErrorKind),
    ScalarLimitExceeded,
    TotalContentLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticScalarTableError {
    kind: SemanticScalarTableErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct SemanticScalarTableErrorView {
    pub kind: SemanticScalarTableErrorKind,
    pub byte_offset: u64,
}

impl View for SemanticScalarTableError {
    type V = SemanticScalarTableErrorView;

    closed spec fn view(&self) -> SemanticScalarTableErrorView {
        SemanticScalarTableErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl SemanticScalarTableError {
    fn at(kind: SemanticScalarTableErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (SemanticScalarTableErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: SemanticScalarTableErrorKind)
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

pub open spec fn map_semantic_scalar_value_error_spec(
    error: ScalarValueErrorView,
) -> SemanticScalarTableErrorView {
    SemanticScalarTableErrorView {
        kind: SemanticScalarTableErrorKind::ScalarValue(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_semantic_scalar_value_error(error: ScalarValueError) -> (mapped: SemanticScalarTableError)
    ensures
        mapped@ == map_semantic_scalar_value_error_spec(error@),
{
    let mapped = SemanticScalarTableError::at(
        SemanticScalarTableErrorKind::ScalarValue(error.kind()),
        error.byte_offset(),
    );
    proof {
        reveal(map_semantic_scalar_value_error_spec);
    }
    mapped
}

pub open spec fn semantic_scalar_table_structural_well_formed_spec(
    cst: CstSourceView,
    source: SemanticScalarTableSourceView,
) -> bool {
    source.profile_version == cst.profile_version && source.transformation_version
        == SEMANTIC_SCALAR_TABLE_TRANSFORMATION_VERSION && source.source_len_bytes
        == cst.source_len_bytes && source.input_token_transformation_version
        == cst.input_token_transformation_version && source.input_cst_transformation_version
        == cst.transformation_version && source.input_node_count == cst.nodes.len()
        && semantic_scalar_table_scalar_node_indices_spec(source.scalars)
        == semantic_scalar_table_expected_node_indices_spec(cst.nodes)
        && source.total_content_code_points as int == semantic_scalar_table_total_content_spec(
        source.scalars,
    ) && source.scalars.len() <= crate::cst::MAX_PROFILE1_CST_NODES
        && source.total_content_code_points <= MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS
}

pub proof fn lemma_semantic_scalar_table_well_formed_is_exact(
    cst: CstSourceView,
    source: SemanticScalarTableSourceView,
)
    requires
        semantic_scalar_table_structural_well_formed_spec(cst, source),
    ensures
        semantic_scalar_table_scalar_node_indices_spec(source.scalars)
            == semantic_scalar_table_expected_node_indices_spec(cst.nodes),
        source.total_content_code_points as int == semantic_scalar_table_total_content_spec(
            source.scalars,
        ),
{
    reveal(semantic_scalar_table_structural_well_formed_spec);
}

pub open spec fn semantic_scalar_table_inputs_match_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
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
        && quoted.profile_version == atomized.profile_version && quoted.input_transformation_version
        == atomized.transformation_version && quoted.transformation_version
        == crate::quoted::QUOTED_SCALAR_TRANSFORMATION_VERSION && quoted.source_len_bytes
        == atomized.source_len_bytes && quoted.bom_bytes == atomized.bom_bytes
        && quoted.input_atom_count == atomized.atoms.len() && plain.profile_version
        == atomized.profile_version && plain.input_transformation_version
        == atomized.transformation_version && plain.quoted_transformation_version
        == quoted.transformation_version && plain.transformation_version
        == crate::plain::PLAIN_SCALAR_TRANSFORMATION_VERSION && plain.source_len_bytes
        == atomized.source_len_bytes && plain.bom_bytes == atomized.bom_bytes
        && plain.input_atom_count == atomized.atoms.len() && plain.input_quoted_scalar_count
        == quoted.scalars.len() && block.profile_version == atomized.profile_version
        && block.input_transformation_version == atomized.transformation_version
        && block.quoted_transformation_version == quoted.transformation_version
        && block.plain_transformation_version == plain.transformation_version
        && block.transformation_version == crate::block::BLOCK_SCALAR_TRANSFORMATION_VERSION
        && block.source_len_bytes == atomized.source_len_bytes && block.bom_bytes
        == atomized.bom_bytes && block.input_atom_count == atomized.atoms.len()
        && block.input_quoted_scalar_count == quoted.scalars.len() && block.input_plain_scalar_count
        == plain.scalars.len()
}

#[derive(Debug, PartialEq, Eq)]
enum SemanticScalarTableStep {
    Skip,
    Scalar(ResolvedScalar),
}

#[verifier::ext_equal]
#[allow(dead_code)]
enum SemanticScalarTableStepView {
    Skip,
    Scalar(ResolvedScalarView),
}

impl View for SemanticScalarTableStep {
    type V = SemanticScalarTableStepView;

    closed spec fn view(&self) -> SemanticScalarTableStepView {
        match self {
            SemanticScalarTableStep::Skip => SemanticScalarTableStepView::Skip,
            SemanticScalarTableStep::Scalar(value) => SemanticScalarTableStepView::Scalar(value@),
        }
    }
}

closed spec fn semantic_scalar_table_step_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    index: nat,
    limits: ScalarValueLimitsView,
) -> Result<SemanticScalarTableStepView, SemanticScalarTableErrorView> {
    let node = cst.nodes[index as int];
    if node.kind != CstNodeKind::Empty && node.kind != CstNodeKind::Scalar {
        Ok(SemanticScalarTableStepView::Skip)
    } else {
        match crate::resolve_scalar_value::resolve_profile1_cst_node_scalar_value_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            index as u64,
            limits,
        ) {
            Err(error) => Err(map_semantic_scalar_value_error_spec(error)),
            Ok(None) => Err(
                SemanticScalarTableErrorView {
                    kind: SemanticScalarTableErrorKind::InternalInvariantViolation,
                    byte_offset: node.byte_start,
                },
            ),
            Ok(Some(scalar)) => Ok(SemanticScalarTableStepView::Scalar(scalar)),
        }
    }
}

#[allow(clippy::too_many_arguments)]  // Every authenticated scalar producer remains explicit.
fn semantic_scalar_table_step(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    index: usize,
    limits: ScalarValueLimits,
) -> (result: Result<SemanticScalarTableStep, SemanticScalarTableError>)
    requires
        index < cst@.nodes.len(),
    ensures
        semantic_scalar_table_step_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            index as nat,
            limits@,
        ) == match result {
            Ok(step) => Ok(step@),
            Err(error) => Err(error@),
        },
{
    let nodes = cst.nodes();
    proof {
        reveal(crate::cst::cst_node_views_spec);
        assert(cst@.nodes.len() == nodes@.len());
        crate::cst::lemma_cst_node_view_at(nodes@, index as int);
    }
    let node = &nodes[index];
    let kind = node.kind();
    if kind != CstNodeKind::Empty && kind != CstNodeKind::Scalar {
        proof {
            reveal(semantic_scalar_table_step_spec);
        }
        return Ok(SemanticScalarTableStep::Skip);
    }
    match resolve_profile1_cst_node_scalar_value(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        index as u64,
        limits,
    ) {
        Err(error) => {
            let mapped = map_semantic_scalar_value_error(error);
            proof {
                reveal(semantic_scalar_table_step_spec);
            }
            Err(mapped)
        },
        Ok(None) => {
            let error = SemanticScalarTableError::at(
                SemanticScalarTableErrorKind::InternalInvariantViolation,
                node.byte_start(),
            );
            proof {
                reveal(semantic_scalar_table_step_spec);
            }
            Err(error)
        },
        Ok(Some(value)) => {
            proof {
                reveal(semantic_scalar_table_step_spec);
            }
            Ok(SemanticScalarTableStep::Scalar(value))
        },
    }
}

#[verifier::ext_equal]
pub struct SemanticScalarTableBuildView {
    pub scalars: Seq<ResolvedScalarView>,
    pub total_content_code_points: nat,
}

pub closed spec fn compose_semantic_scalar_table_tail_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    index: nat,
    fuel: nat,
    build: SemanticScalarTableBuildView,
    limits: SemanticScalarTableLimitsView,
) -> Result<SemanticScalarTableBuildView, SemanticScalarTableErrorView>
    decreases fuel,
{
    if index >= cst.nodes.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            SemanticScalarTableErrorView {
                kind: SemanticScalarTableErrorKind::InternalInvariantViolation,
                byte_offset: cst.nodes[index as int].byte_start,
            },
        )
    } else {
        let node = cst.nodes[index as int];
        match semantic_scalar_table_step_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            index,
            semantic_scalar_value_limits_spec(limits),
        ) {
            Err(error) => Err(error),
            Ok(SemanticScalarTableStepView::Skip) => compose_semantic_scalar_table_tail_spec(
                atomized,
                quoted,
                plain,
                block,
                completed,
                cst,
                (index + 1) as nat,
                (fuel - 1) as nat,
                build,
                limits,
            ),
            Ok(SemanticScalarTableStepView::Scalar(scalar)) => {
                let scalar_limit = semantic_scalar_table_effective_limit_spec(
                    limits.max_scalars,
                    crate::cst::MAX_PROFILE1_CST_NODES,
                );
                let total_limit = semantic_scalar_table_effective_limit_spec(
                    limits.max_total_content_code_points,
                    MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
                );
                let content = semantic_scalar_content_spec(scalar);
                if build.scalars.len() >= scalar_limit {
                    Err(
                        SemanticScalarTableErrorView {
                            kind: SemanticScalarTableErrorKind::ScalarLimitExceeded,
                            byte_offset: node.byte_start,
                        },
                    )
                } else if content.len() > u64::MAX {
                    Err(
                        SemanticScalarTableErrorView {
                            kind: SemanticScalarTableErrorKind::InternalInvariantViolation,
                            byte_offset: node.byte_start,
                        },
                    )
                } else if build.total_content_code_points > total_limit as nat || content.len()
                    > total_limit as nat - build.total_content_code_points {
                    let excluded = if build.total_content_code_points <= total_limit as nat {
                        total_limit as nat - build.total_content_code_points
                    } else {
                        0
                    };
                    Err(
                        SemanticScalarTableErrorView {
                            kind: SemanticScalarTableErrorKind::TotalContentLimitExceeded,
                            byte_offset: if excluded < content.len() {
                                content[excluded as int].byte_start
                            } else {
                                node.byte_start
                            },
                        },
                    )
                } else {
                    compose_semantic_scalar_table_tail_spec(
                        atomized,
                        quoted,
                        plain,
                        block,
                        completed,
                        cst,
                        (index + 1) as nat,
                        (fuel - 1) as nat,
                        SemanticScalarTableBuildView {
                            scalars: build.scalars.push(scalar),
                            total_content_code_points: build.total_content_code_points
                                + content.len(),
                        },
                        limits,
                    )
                }
            },
        }
    }
}

pub open spec fn semantic_scalar_table_finalize_spec(
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    result: Result<SemanticScalarTableBuildView, SemanticScalarTableErrorView>,
) -> Result<SemanticScalarTableSourceView, SemanticScalarTableErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => Ok(
            SemanticScalarTableSourceView {
                profile_version: completed.profile_version,
                transformation_version: SEMANTIC_SCALAR_TABLE_TRANSFORMATION_VERSION,
                source_len_bytes: completed.source_len_bytes,
                input_token_transformation_version: completed.transformation_version,
                input_cst_transformation_version: cst.transformation_version,
                input_node_count: cst.nodes.len() as u64,
                total_content_code_points: build.total_content_code_points as u64,
                scalars: build.scalars,
            },
        ),
    }
}

pub open spec fn compose_profile1_semantic_scalar_table_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: SemanticScalarTableLimitsView,
) -> Result<SemanticScalarTableSourceView, SemanticScalarTableErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            SemanticScalarTableErrorView {
                kind: SemanticScalarTableErrorKind::ScalarValue(
                    ScalarValueErrorKind::ScalarDecode(
                        crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputCompletedTokenMismatch,
                    ),
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() {
        Err(
            SemanticScalarTableErrorView {
                kind: SemanticScalarTableErrorKind::ScalarValue(
                    ScalarValueErrorKind::ScalarDecode(
                        crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputCstMismatch,
                    ),
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if quoted.profile_version != atomized.profile_version
        || quoted.input_transformation_version != atomized.transformation_version
        || quoted.transformation_version != crate::quoted::QUOTED_SCALAR_TRANSFORMATION_VERSION
        || quoted.source_len_bytes != atomized.source_len_bytes || quoted.bom_bytes
        != atomized.bom_bytes || quoted.input_atom_count != atomized.atoms.len() {
        Err(
            SemanticScalarTableErrorView {
                kind: SemanticScalarTableErrorKind::ScalarValue(
                    ScalarValueErrorKind::ScalarDecode(
                        crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputQuotedMismatch,
                    ),
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if plain.profile_version != atomized.profile_version
        || plain.input_transformation_version != atomized.transformation_version
        || plain.quoted_transformation_version != quoted.transformation_version
        || plain.transformation_version != crate::plain::PLAIN_SCALAR_TRANSFORMATION_VERSION
        || plain.source_len_bytes != atomized.source_len_bytes || plain.bom_bytes
        != atomized.bom_bytes || plain.input_atom_count != atomized.atoms.len()
        || plain.input_quoted_scalar_count != quoted.scalars.len() {
        Err(
            SemanticScalarTableErrorView {
                kind: SemanticScalarTableErrorKind::ScalarValue(
                    ScalarValueErrorKind::ScalarDecode(
                        crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputPlainMismatch,
                    ),
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if block.profile_version != atomized.profile_version
        || block.input_transformation_version != atomized.transformation_version
        || block.quoted_transformation_version != quoted.transformation_version
        || block.plain_transformation_version != plain.transformation_version
        || block.transformation_version != crate::block::BLOCK_SCALAR_TRANSFORMATION_VERSION
        || block.source_len_bytes != atomized.source_len_bytes || block.bom_bytes
        != atomized.bom_bytes || block.input_atom_count != atomized.atoms.len()
        || block.input_quoted_scalar_count != quoted.scalars.len() || block.input_plain_scalar_count
        != plain.scalars.len() {
        Err(
            SemanticScalarTableErrorView {
                kind: SemanticScalarTableErrorKind::ScalarValue(
                    ScalarValueErrorKind::ScalarDecode(
                        crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputBlockMismatch,
                    ),
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else {
        semantic_scalar_table_finalize_spec(
            completed,
            cst,
            compose_semantic_scalar_table_tail_spec(
                atomized,
                quoted,
                plain,
                block,
                completed,
                cst,
                0,
                cst.nodes.len(),
                SemanticScalarTableBuildView {
                    scalars: Seq::empty(),
                    total_content_code_points: 0,
                },
                limits,
            ),
        )
    }
}

/// Exact public scalar-table semantics, including producer authentication and every resolved value.
pub open spec fn semantic_scalar_table_source_well_formed_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: SemanticScalarTableLimitsView,
    source: SemanticScalarTableSourceView,
) -> bool {
    crate::cst::cst_public_semantics_spec(completed, cst)
        && compose_profile1_semantic_scalar_table_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        limits,
    ) == Ok(source) && semantic_scalar_table_structural_well_formed_spec(cst, source)
}

/// Extract the authenticated exact composition and structural facts from public semantics.
pub proof fn lemma_semantic_scalar_table_well_formed_authenticates_exact_composition(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: SemanticScalarTableLimitsView,
    source: SemanticScalarTableSourceView,
)
    requires
        semantic_scalar_table_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
            source,
        ),
    ensures
        crate::cst::cst_public_semantics_spec(completed, cst),
        compose_profile1_semantic_scalar_table_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
        ) == Ok(source),
        semantic_scalar_table_structural_well_formed_spec(cst, source),
{
    reveal(semantic_scalar_table_source_well_formed_spec);
}

proof fn lemma_semantic_scalar_table_tail_success_is_exact(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    index: nat,
    fuel: nat,
    build: SemanticScalarTableBuildView,
    limits: SemanticScalarTableLimitsView,
    final_build: SemanticScalarTableBuildView,
)
    requires
        cst.nodes.len() <= crate::cst::MAX_PROFILE1_CST_NODES,
        index <= cst.nodes.len(),
        fuel == cst.nodes.len() - index,
        semantic_scalar_table_scalar_node_indices_spec(build.scalars)
            == semantic_scalar_table_expected_node_indices_prefix_spec(cst.nodes, index, index),
        build.total_content_code_points as int == semantic_scalar_table_total_content_spec(
            build.scalars,
        ),
        build.scalars.len() <= semantic_scalar_table_effective_limit_spec(
            limits.max_scalars,
            crate::cst::MAX_PROFILE1_CST_NODES,
        ),
        build.total_content_code_points <= semantic_scalar_table_effective_limit_spec(
            limits.max_total_content_code_points,
            MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
        ),
        compose_semantic_scalar_table_tail_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            index,
            fuel,
            build,
            limits,
        ) == Ok(final_build),
    ensures
        semantic_scalar_table_scalar_node_indices_spec(final_build.scalars)
            == semantic_scalar_table_expected_node_indices_spec(cst.nodes),
        final_build.total_content_code_points as int == semantic_scalar_table_total_content_spec(
            final_build.scalars,
        ),
        final_build.scalars.len() <= semantic_scalar_table_effective_limit_spec(
            limits.max_scalars,
            crate::cst::MAX_PROFILE1_CST_NODES,
        ),
        final_build.total_content_code_points <= semantic_scalar_table_effective_limit_spec(
            limits.max_total_content_code_points,
            MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
        ),
    decreases fuel,
{
    if index >= cst.nodes.len() {
        assert(index == cst.nodes.len());
        reveal(compose_semantic_scalar_table_tail_spec);
        reveal(semantic_scalar_table_expected_node_indices_spec);
        assert(final_build == build);
    } else {
        assert(fuel > 0);
        let step = semantic_scalar_table_step_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            index,
            semantic_scalar_value_limits_spec(limits),
        );
        reveal(compose_semantic_scalar_table_tail_spec);
        match step {
            Err(_) => {
                assert(false);
            },
            Ok(SemanticScalarTableStepView::Skip) => {
                reveal(semantic_scalar_table_step_spec);
                let node = cst.nodes[index as int];
                assert(node.kind != CstNodeKind::Empty && node.kind != CstNodeKind::Scalar);
                reveal(semantic_scalar_table_expected_node_indices_prefix_spec);
                lemma_semantic_scalar_table_tail_success_is_exact(
                    atomized,
                    quoted,
                    plain,
                    block,
                    completed,
                    cst,
                    (index + 1) as nat,
                    (fuel - 1) as nat,
                    build,
                    limits,
                    final_build,
                );
            },
            Ok(SemanticScalarTableStepView::Scalar(scalar)) => {
                reveal(semantic_scalar_table_step_spec);
                let node = cst.nodes[index as int];
                assert(node.kind == CstNodeKind::Empty || node.kind == CstNodeKind::Scalar);
                crate::resolve_scalar_value::lemma_resolved_scalar_success_retains_requested_node_index(

                    atomized,
                    quoted,
                    plain,
                    block,
                    completed,
                    cst,
                    index as u64,
                    semantic_scalar_value_limits_spec(limits),
                    scalar,
                );
                let content = semantic_scalar_content_spec(scalar);
                let next_build = SemanticScalarTableBuildView {
                    scalars: build.scalars.push(scalar),
                    total_content_code_points: build.total_content_code_points + content.len(),
                };
                assert(compose_semantic_scalar_table_tail_spec(
                    atomized,
                    quoted,
                    plain,
                    block,
                    completed,
                    cst,
                    (index + 1) as nat,
                    (fuel - 1) as nat,
                    next_build,
                    limits,
                ) == Ok(final_build));
                lemma_semantic_scalar_table_scalar_node_indices_push(build.scalars, scalar);
                reveal(semantic_scalar_table_expected_node_indices_prefix_spec);
                lemma_semantic_scalar_table_total_content_push(build.scalars, scalar);
                lemma_semantic_scalar_table_tail_success_is_exact(
                    atomized,
                    quoted,
                    plain,
                    block,
                    completed,
                    cst,
                    (index + 1) as nat,
                    (fuel - 1) as nat,
                    next_build,
                    limits,
                    final_build,
                );
            },
        }
    }
}

/// Extract exact CST coverage and aggregate accounting from every successful pure composition.
pub proof fn lemma_semantic_scalar_table_success_is_well_formed(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: SemanticScalarTableLimitsView,
    source: SemanticScalarTableSourceView,
)
    requires
        crate::cst::cst_public_semantics_spec(completed, cst),
        compose_profile1_semantic_scalar_table_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
        ) == Ok(source),
    ensures
        semantic_scalar_table_source_well_formed_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            limits,
            source,
        ),
{
    let result = compose_semantic_scalar_table_tail_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        0,
        cst.nodes.len(),
        SemanticScalarTableBuildView { scalars: Seq::empty(), total_content_code_points: 0 },
        limits,
    );
    reveal(compose_profile1_semantic_scalar_table_spec);
    reveal(semantic_scalar_table_finalize_spec);
    assert(result.is_ok());
    assert(exists|build: SemanticScalarTableBuildView| result == Ok(build));
    let final_build = choose|build: SemanticScalarTableBuildView| result == Ok(build);
    reveal(semantic_scalar_table_scalar_node_indices_spec);
    reveal(semantic_scalar_table_expected_node_indices_prefix_spec);
    reveal(semantic_scalar_table_total_content_spec);
    reveal(semantic_scalar_table_total_content_tail_spec);
    assert(semantic_scalar_table_scalar_node_indices_spec(Seq::empty()) == Seq::<u64>::empty());
    assert(semantic_scalar_table_expected_node_indices_prefix_spec(cst.nodes, 0, 0) == Seq::<
        u64,
    >::empty());
    lemma_semantic_scalar_table_tail_success_is_exact(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        0,
        cst.nodes.len(),
        SemanticScalarTableBuildView { scalars: Seq::empty(), total_content_code_points: 0 },
        limits,
        final_build,
    );
    reveal(crate::cst::cst_public_semantics_spec);
    reveal(crate::cst::cst_source_respects_limits_spec);
    reveal(semantic_scalar_table_source_well_formed_spec);
    reveal(semantic_scalar_table_structural_well_formed_spec);
    assert(source.scalars.len() <= crate::cst::MAX_PROFILE1_CST_NODES);
    assert(source.total_content_code_points <= MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS);
}

fn semantic_scalar_content(scalar: &ResolvedScalar) -> (content: Option<
    &[crate::scalar_decode::DecodedContentScalar],
>)
    ensures
        match content {
            Some(values) => semantic_scalar_content_spec(scalar@)
                == crate::scalar_decode::decoded_content_scalar_views_spec(values@),
            None => semantic_scalar_content_spec(scalar@).len() == 0,
        },
{
    match scalar.presentation().decoded() {
        Some(decoded) => Some(decoded.content()),
        None => None,
    }
}

#[allow(clippy::too_many_arguments)]  // Every authenticated scalar producer remains explicit.
pub fn compose_profile1_semantic_scalar_table(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    limits: SemanticScalarTableLimits,
) -> (result: Result<SemanticScalarTableSource, SemanticScalarTableError>)
    ensures
        compose_profile1_semantic_scalar_table_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            limits@,
        ) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let quote_scalars = quoted.scalars();
    let plain_scalars = plain.scalars();
    let nodes = cst.nodes();
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(crate::quoted::quoted_scalar_views_spec);
        reveal(crate::plain::plain_scalar_views_spec);
        reveal(crate::cst::cst_node_views_spec);
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens.len() == tokens@.len());
        assert(quoted@.scalars.len() == quote_scalars@.len());
        assert(plain@.scalars.len() == plain_scalars@.len());
        assert(cst@.nodes.len() == nodes@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != atomized.bom_bytes()
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = SemanticScalarTableError::at(
            SemanticScalarTableErrorKind::ScalarValue(
                ScalarValueErrorKind::ScalarDecode(
                    crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputCompletedTokenMismatch,
                ),
            ),
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_scalar_table_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 {
        let error = SemanticScalarTableError::at(
            SemanticScalarTableErrorKind::ScalarValue(
                ScalarValueErrorKind::ScalarDecode(
                    crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputCstMismatch,
                ),
            ),
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_scalar_table_spec);
        }
        return Err(error);
    }
    if quoted.profile_version() != atomized.profile_version()
        || quoted.input_transformation_version() != atomized.transformation_version()
        || quoted.transformation_version() != crate::quoted::QUOTED_SCALAR_TRANSFORMATION_VERSION
        || quoted.source_len_bytes() != atomized.source_len_bytes() || quoted.bom_bytes()
        != atomized.bom_bytes() || quoted.input_atom_count() != atoms.len() as u64 {
        let error = SemanticScalarTableError::at(
            SemanticScalarTableErrorKind::ScalarValue(
                ScalarValueErrorKind::ScalarDecode(
                    crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputQuotedMismatch,
                ),
            ),
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_scalar_table_spec);
        }
        return Err(error);
    }
    if plain.profile_version() != atomized.profile_version() || plain.input_transformation_version()
        != atomized.transformation_version() || plain.quoted_transformation_version()
        != quoted.transformation_version() || plain.transformation_version()
        != crate::plain::PLAIN_SCALAR_TRANSFORMATION_VERSION || plain.source_len_bytes()
        != atomized.source_len_bytes() || plain.bom_bytes() != atomized.bom_bytes()
        || plain.input_atom_count() != atoms.len() as u64 || plain.input_quoted_scalar_count()
        != quote_scalars.len() as u64 {
        let error = SemanticScalarTableError::at(
            SemanticScalarTableErrorKind::ScalarValue(
                ScalarValueErrorKind::ScalarDecode(
                    crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputPlainMismatch,
                ),
            ),
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_scalar_table_spec);
        }
        return Err(error);
    }
    if block.profile_version() != atomized.profile_version() || block.input_transformation_version()
        != atomized.transformation_version() || block.quoted_transformation_version()
        != quoted.transformation_version() || block.plain_transformation_version()
        != plain.transformation_version() || block.transformation_version()
        != crate::block::BLOCK_SCALAR_TRANSFORMATION_VERSION || block.source_len_bytes()
        != atomized.source_len_bytes() || block.bom_bytes() != atomized.bom_bytes()
        || block.input_atom_count() != atoms.len() as u64 || block.input_quoted_scalar_count()
        != quote_scalars.len() as u64 || block.input_plain_scalar_count()
        != plain_scalars.len() as u64 {
        let error = SemanticScalarTableError::at(
            SemanticScalarTableErrorKind::ScalarValue(
                ScalarValueErrorKind::ScalarDecode(
                    crate::resolve_scalar_node::CstScalarDecodeErrorKind::InputBlockMismatch,
                ),
            ),
            atomized.bom_bytes(),
        );
        proof {
            reveal(compose_profile1_semantic_scalar_table_spec);
        }
        return Err(error);
    }
    let requested_scalar_limit = limits.max_scalars();
    let scalar_limit = semantic_scalar_table_effective_limit(
        requested_scalar_limit,
        crate::cst::MAX_PROFILE1_CST_NODES,
    );
    let requested_total_limit = limits.max_total_content_code_points();
    let total_limit = semantic_scalar_table_effective_limit(
        requested_total_limit,
        MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
    );
    let scalar_limits = limits.scalar_value_limits();
    let mut scalars = Vec::new();
    let mut total_content_code_points = 0u64;
    let mut index = 0usize;
    let mut fuel = nodes.len();
    let ghost initial_build = SemanticScalarTableBuildView {
        scalars: Seq::empty(),
        total_content_code_points: 0,
    };
    let ghost expected = compose_semantic_scalar_table_tail_spec(
        atomized@,
        quoted@,
        plain@,
        block@,
        completed@,
        cst@,
        0,
        cst@.nodes.len(),
        initial_build,
        limits@,
    );
    proof {
        reveal(semantic_scalar_views_spec);
        reveal(compose_profile1_semantic_scalar_table_spec);
        assert(semantic_scalar_views_spec(scalars@) == Seq::<ResolvedScalarView>::empty());
        assert(compose_profile1_semantic_scalar_table_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            limits@,
        ) == semantic_scalar_table_finalize_spec(completed@, cst@, expected));
    }
    while index < nodes.len()
        invariant
            cst@.nodes == crate::cst::cst_node_views_spec(nodes@),
            index <= nodes.len(),
            fuel == nodes.len() - index,
            requested_scalar_limit == limits@.max_scalars,
            scalar_limit == semantic_scalar_table_effective_limit_spec(
                limits@.max_scalars,
                crate::cst::MAX_PROFILE1_CST_NODES,
            ),
            requested_total_limit == limits@.max_total_content_code_points,
            total_limit == semantic_scalar_table_effective_limit_spec(
                limits@.max_total_content_code_points,
                MAX_PROFILE1_TOTAL_SEMANTIC_SCALAR_CODE_POINTS,
            ),
            scalar_limits@ == semantic_scalar_value_limits_spec(limits@),
            total_content_code_points <= total_limit,
            compose_profile1_semantic_scalar_table_spec(
                atomized@,
                quoted@,
                plain@,
                block@,
                completed@,
                cst@,
                limits@,
            ) == semantic_scalar_table_finalize_spec(completed@, cst@, expected),
            compose_semantic_scalar_table_tail_spec(
                atomized@,
                quoted@,
                plain@,
                block@,
                completed@,
                cst@,
                index as nat,
                fuel as nat,
                SemanticScalarTableBuildView {
                    scalars: semantic_scalar_views_spec(scalars@),
                    total_content_code_points: total_content_code_points as nat,
                },
                limits@,
            ) == expected,
        decreases fuel,
    {
        if fuel == 0 {
            let error = SemanticScalarTableError::at(
                SemanticScalarTableErrorKind::InternalInvariantViolation,
                nodes[index].byte_start(),
            );
            proof {
                reveal(compose_semantic_scalar_table_tail_spec);
                reveal(compose_profile1_semantic_scalar_table_spec);
                reveal(semantic_scalar_table_finalize_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let node = &nodes[index];
        proof {
            crate::cst::lemma_cst_node_view_at(nodes@, index as int);
        }
        let step = semantic_scalar_table_step(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            index,
            scalar_limits,
        );
        let scalar = match step {
            Err(error) => {
                proof {
                    reveal(compose_semantic_scalar_table_tail_spec);
                    reveal(compose_profile1_semantic_scalar_table_spec);
                    reveal(semantic_scalar_table_finalize_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(SemanticScalarTableStep::Skip) => {
                proof {
                    reveal(compose_semantic_scalar_table_tail_spec);
                }
                index += 1;
                fuel -= 1;
                continue;
            },
            Ok(SemanticScalarTableStep::Scalar(value)) => value,
        };
        let content = semantic_scalar_content(&scalar);
        let content_len = match content {
            Some(values) => values.len(),
            None => 0,
        };
        proof {
            match content {
                Some(values) => {
                    reveal(crate::scalar_decode::decoded_content_scalar_views_spec);
                    assert(semantic_scalar_content_spec(scalar@).len() == values@.len());
                },
                None => {},
            }
            assert(semantic_scalar_content_spec(scalar@).len() == content_len);
        }
        if scalars.len() as u64 >= scalar_limit {
            let error = SemanticScalarTableError::at(
                SemanticScalarTableErrorKind::ScalarLimitExceeded,
                node.byte_start(),
            );
            proof {
                reveal(semantic_scalar_views_spec);
                assert(semantic_scalar_views_spec(scalars@).len() == scalars@.len());
                assert(scalars@.len() == scalars.len());
                assert(scalar_limit == semantic_scalar_table_effective_limit_spec(
                    limits@.max_scalars,
                    crate::cst::MAX_PROFILE1_CST_NODES,
                ));
                reveal(compose_semantic_scalar_table_tail_spec);
                reveal(compose_profile1_semantic_scalar_table_spec);
                reveal(semantic_scalar_table_finalize_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        assert(content_len <= usize::MAX);
        assert(usize::MAX as u128 <= u64::MAX as u128);
        let content_count = content_len as u64;
        if content_count > total_limit - total_content_code_points {
            let excluded = (total_limit - total_content_code_points) as usize;
            let offset = match content {
                Some(values) if excluded < values.len() => values[excluded].byte_start(),
                _ => node.byte_start(),
            };
            let error = SemanticScalarTableError::at(
                SemanticScalarTableErrorKind::TotalContentLimitExceeded,
                offset,
            );
            proof {
                reveal(compose_semantic_scalar_table_tail_spec);
                reveal(compose_profile1_semantic_scalar_table_spec);
                reveal(semantic_scalar_table_finalize_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            lemma_semantic_scalar_views_push(scalars@, scalar);
            reveal(semantic_scalar_views_spec);
            assert(semantic_scalar_views_spec(scalars@).len() == scalars@.len());
            assert(scalars@.len() == scalars.len());
            assert(scalar_limit == semantic_scalar_table_effective_limit_spec(
                limits@.max_scalars,
                crate::cst::MAX_PROFILE1_CST_NODES,
            ));
            reveal(compose_semantic_scalar_table_tail_spec);
            assert(compose_semantic_scalar_table_tail_spec(
                atomized@,
                quoted@,
                plain@,
                block@,
                completed@,
                cst@,
                (index + 1) as nat,
                (fuel - 1) as nat,
                SemanticScalarTableBuildView {
                    scalars: semantic_scalar_views_spec(scalars@).push(scalar@),
                    total_content_code_points: (total_content_code_points + content_count) as nat,
                },
                limits@,
            ) == expected);
        }
        scalars.push(scalar);
        total_content_code_points += content_count;
        index += 1;
        fuel -= 1;
    }
    proof {
        reveal(compose_semantic_scalar_table_tail_spec);
        assert(expected == Ok(
            SemanticScalarTableBuildView {
                scalars: semantic_scalar_views_spec(scalars@),
                total_content_code_points: total_content_code_points as nat,
            },
        ));
    }
    let source = SemanticScalarTableSource::new(completed, cst, total_content_code_points, scalars);
    proof {
        reveal(compose_profile1_semantic_scalar_table_spec);
        reveal(semantic_scalar_table_finalize_spec);
    }
    Ok(source)
}

} // verus!
