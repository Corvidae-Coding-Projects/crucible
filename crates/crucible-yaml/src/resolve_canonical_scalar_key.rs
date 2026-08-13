#![allow(clippy::question_mark, clippy::single_match)]
//! Verified collision-free canonical byte identities for resolved scalar nodes.
//!
//! Presentation style and spelling are excluded; resolved semantic tags and canonical values are
//! encoded with explicit variant markers and fixed-width length delimiters. Every byte retains a
//! source diagnostic anchor. Per-key and aggregate limits are checked before each allocation.
use crate::atom::AtomizedSource;
#[allow(unused_imports)]
use crate::atom::AtomizedSourceView;
use crate::block::BlockScalarSource;
#[allow(unused_imports)]
use crate::block::BlockScalarSourceView;
use crate::cst::CstSource;
#[allow(unused_imports)]
use crate::cst::CstSourceView;
use crate::plain::PlainScalarSource;
#[allow(unused_imports)]
use crate::plain::PlainScalarSourceView;
use crate::quoted::QuotedScalarSource;
#[allow(unused_imports)]
use crate::quoted::QuotedScalarSourceView;
use crate::resolve_alias_cycle::{
    AcyclicSemanticGraphSource, AliasCycleError, AliasCycleErrorKind, AliasCycleLimits,
};
#[allow(unused_imports)]
use crate::resolve_alias_cycle::{AcyclicSemanticGraphSourceView, AliasCycleLimitsView};
use crate::resolve_anchor::AnchorAliasLimits;
#[allow(unused_imports)]
use crate::resolve_anchor::AnchorAliasLimitsView;
use crate::resolve_node_table::SemanticNodeTableLimits;
#[allow(unused_imports)]
use crate::resolve_node_table::SemanticNodeTableLimitsView;
use crate::resolve_scalar_table::SemanticScalarTableLimits;
#[allow(unused_imports)]
use crate::resolve_scalar_table::SemanticScalarTableLimitsView;
use crate::resolve_scalar_value::{ResolvedScalar, ResolvedScalarTag, ResolvedScalarValue};
#[allow(unused_imports)]
use crate::resolve_scalar_value::{ResolvedScalarValueView, ResolvedScalarView};
use crate::resolve_topology::SemanticTopologyLimits;
#[allow(unused_imports)]
use crate::resolve_topology::SemanticTopologyLimitsView;
use crate::token::CompletedTokenSource;
#[allow(unused_imports)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const CANONICAL_SCALAR_KEY_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES: u64 = 1_048_576;

pub const MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalScalarKeyLimits {
    max_records: u64,
    max_key_bytes: u64,
    max_total_key_bytes: u64,
}

#[verifier::ext_equal]
pub struct CanonicalScalarKeyLimitsView {
    pub max_records: u64,
    pub max_key_bytes: u64,
    pub max_total_key_bytes: u64,
}

impl View for CanonicalScalarKeyLimits {
    type V = CanonicalScalarKeyLimitsView;

    closed spec fn view(&self) -> CanonicalScalarKeyLimitsView {
        CanonicalScalarKeyLimitsView {
            max_records: self.max_records,
            max_key_bytes: self.max_key_bytes,
            max_total_key_bytes: self.max_total_key_bytes,
        }
    }
}

impl CanonicalScalarKeyLimits {
    pub fn new(max_records: u64, max_key_bytes: u64, max_total_key_bytes: u64) -> (limits: Self)
        ensures
            limits@ == (CanonicalScalarKeyLimitsView {
                max_records,
                max_key_bytes,
                max_total_key_bytes,
            }),
    {
        Self { max_records, max_key_bytes, max_total_key_bytes }
    }

    pub fn max_records(&self) -> (value: u64)
        ensures
            value == self@.max_records,
    {
        self.max_records
    }

    pub fn max_key_bytes(&self) -> (value: u64)
        ensures
            value == self@.max_key_bytes,
    {
        self.max_key_bytes
    }

    pub fn max_total_key_bytes(&self) -> (value: u64)
        ensures
            value == self@.max_total_key_bytes,
    {
        self.max_total_key_bytes
    }
}

pub fn canonical_scalar_key_limits() -> (limits: CanonicalScalarKeyLimits)
    ensures
        limits@ == canonical_scalar_key_limits_spec(),
{
    CanonicalScalarKeyLimits::new(
        MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS,
        MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
        MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
    )
}

pub open spec fn canonical_scalar_key_limits_spec() -> CanonicalScalarKeyLimitsView {
    CanonicalScalarKeyLimitsView {
        max_records: MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS,
        max_key_bytes: MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
        max_total_key_bytes: MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
    }
}

pub open spec fn canonical_scalar_key_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn canonical_scalar_key_effective_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit == canonical_scalar_key_effective_limit_spec(requested, absolute),
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalKeyByte {
    value: u8,
    source_byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CanonicalKeyByteView {
    pub value: u8,
    pub source_byte_offset: u64,
}

impl View for CanonicalKeyByte {
    type V = CanonicalKeyByteView;

    closed spec fn view(&self) -> CanonicalKeyByteView {
        CanonicalKeyByteView { value: self.value, source_byte_offset: self.source_byte_offset }
    }
}

impl CanonicalKeyByte {
    pub(crate) fn new(value: u8, source_byte_offset: u64) -> (byte: Self)
        ensures
            byte@ == (CanonicalKeyByteView { value, source_byte_offset }),
    {
        Self { value, source_byte_offset }
    }

    pub fn value(&self) -> (value: u8)
        ensures
            value == self@.value,
    {
        self.value
    }

    pub fn source_byte_offset(&self) -> (offset: u64)
        ensures
            offset == self@.source_byte_offset,
    {
        self.source_byte_offset
    }
}

pub open spec fn canonical_key_byte_views_spec(values: Seq<CanonicalKeyByte>) -> Seq<
    CanonicalKeyByteView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub(crate) proof fn lemma_canonical_key_byte_views_push(
    values: Seq<CanonicalKeyByte>,
    value: CanonicalKeyByte,
)
    ensures
        canonical_key_byte_views_spec(values.push(value)) == canonical_key_byte_views_spec(
            values,
        ).push(value@),
{
    reveal(canonical_key_byte_views_spec);
    assert(canonical_key_byte_views_spec(values.push(value)) =~= canonical_key_byte_views_spec(
        values,
    ).push(value@));
}

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalScalarKeyRecord {
    node_index: u64,
    byte_start: u64,
    bytes: Vec<CanonicalKeyByte>,
}

#[verifier::ext_equal]
pub struct CanonicalScalarKeyRecordView {
    pub node_index: u64,
    pub byte_start: u64,
    pub bytes: Seq<CanonicalKeyByteView>,
}

impl View for CanonicalScalarKeyRecord {
    type V = CanonicalScalarKeyRecordView;

    closed spec fn view(&self) -> CanonicalScalarKeyRecordView {
        CanonicalScalarKeyRecordView {
            node_index: self.node_index,
            byte_start: self.byte_start,
            bytes: canonical_key_byte_views_spec(self.bytes@),
        }
    }
}

impl CanonicalScalarKeyRecord {
    fn new(node_index: u64, byte_start: u64, bytes: Vec<CanonicalKeyByte>) -> (record: Self)
        ensures
            record@ == (CanonicalScalarKeyRecordView {
                node_index,
                byte_start,
                bytes: canonical_key_byte_views_spec(bytes@),
            }),
    {
        Self { node_index, byte_start, bytes }
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

    pub fn bytes(&self) -> (bytes: &[CanonicalKeyByte])
        ensures
            canonical_key_byte_views_spec(bytes@) == self@.bytes,
    {
        self.bytes.as_slice()
    }
}

pub open spec fn canonical_scalar_key_record_views_spec(
    values: Seq<CanonicalScalarKeyRecord>,
) -> Seq<CanonicalScalarKeyRecordView> {
    Seq::new(values.len(), |index: int| values[index]@)
}

proof fn lemma_canonical_scalar_key_record_views_push(
    values: Seq<CanonicalScalarKeyRecord>,
    value: CanonicalScalarKeyRecord,
)
    ensures
        canonical_scalar_key_record_views_spec(values.push(value))
            == canonical_scalar_key_record_views_spec(values).push(value@),
{
    reveal(canonical_scalar_key_record_views_spec);
    assert(canonical_scalar_key_record_views_spec(values.push(value))
        =~= canonical_scalar_key_record_views_spec(values).push(value@));
}

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalScalarKeySource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    total_key_bytes: u64,
    graph: AcyclicSemanticGraphSource,
    records: Vec<CanonicalScalarKeyRecord>,
}

#[verifier::ext_equal]
pub struct CanonicalScalarKeySourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub total_key_bytes: u64,
    pub graph: AcyclicSemanticGraphSourceView,
    pub records: Seq<CanonicalScalarKeyRecordView>,
}

impl View for CanonicalScalarKeySource {
    type V = CanonicalScalarKeySourceView;

    closed spec fn view(&self) -> CanonicalScalarKeySourceView {
        CanonicalScalarKeySourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            total_key_bytes: self.total_key_bytes,
            graph: self.graph@,
            records: canonical_scalar_key_record_views_spec(self.records@),
        }
    }
}

impl CanonicalScalarKeySource {
    fn new(
        graph: AcyclicSemanticGraphSource,
        records: Vec<CanonicalScalarKeyRecord>,
        total_key_bytes: u64,
    ) -> (source: Self)
        ensures
            source@ == (CanonicalScalarKeySourceView {
                profile_version: graph@.profile_version,
                transformation_version: CANONICAL_SCALAR_KEY_TRANSFORMATION_VERSION,
                source_len_bytes: graph@.source_len_bytes,
                input_node_count: graph@.node_table.nodes.len() as u64,
                total_key_bytes,
                graph: graph@,
                records: canonical_scalar_key_record_views_spec(records@),
            }),
    {
        let input_node_count = graph.node_table().nodes().len() as u64;
        Self {
            profile_version: graph.profile_version(),
            transformation_version: CANONICAL_SCALAR_KEY_TRANSFORMATION_VERSION,
            source_len_bytes: graph.source_len_bytes(),
            input_node_count,
            total_key_bytes,
            graph,
            records,
        }
    }

    pub fn input_node_count(&self) -> (count: u64)
        ensures
            count == self@.input_node_count,
    {
        self.input_node_count
    }

    pub fn total_key_bytes(&self) -> (count: u64)
        ensures
            count == self@.total_key_bytes,
    {
        self.total_key_bytes
    }

    pub fn graph(&self) -> (graph: &AcyclicSemanticGraphSource)
        ensures
            graph@ == self@.graph,
    {
        &self.graph
    }

    pub fn records(&self) -> (records: &[CanonicalScalarKeyRecord])
        ensures
            canonical_scalar_key_record_views_spec(records@) == self@.records,
    {
        self.records.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CanonicalScalarKeyErrorKind {
    AliasCycle(AliasCycleErrorKind),
    RecordLimitExceeded,
    KeyByteLimitExceeded,
    TotalKeyByteLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalScalarKeyError {
    kind: CanonicalScalarKeyErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CanonicalScalarKeyErrorView {
    pub kind: CanonicalScalarKeyErrorKind,
    pub byte_offset: u64,
}

impl View for CanonicalScalarKeyError {
    type V = CanonicalScalarKeyErrorView;

    closed spec fn view(&self) -> CanonicalScalarKeyErrorView {
        CanonicalScalarKeyErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CanonicalScalarKeyError {
    fn at(kind: CanonicalScalarKeyErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (CanonicalScalarKeyErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: CanonicalScalarKeyErrorKind)
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

pub open spec fn map_alias_cycle_error_spec(
    error: crate::resolve_alias_cycle::AliasCycleErrorView,
) -> CanonicalScalarKeyErrorView {
    CanonicalScalarKeyErrorView {
        kind: CanonicalScalarKeyErrorKind::AliasCycle(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_alias_cycle_error(error: AliasCycleError) -> (mapped: CanonicalScalarKeyError)
    ensures
        mapped@ == map_alias_cycle_error_spec(error@),
{
    CanonicalScalarKeyError::at(
        CanonicalScalarKeyErrorKind::AliasCycle(error.kind()),
        error.byte_offset(),
    )
}

#[verifier::ext_equal]
pub struct KeyBuildView {
    pub bytes: Seq<CanonicalKeyByteView>,
    pub total_before: u64,
}

struct KeyBuild {
    bytes: Vec<CanonicalKeyByte>,
    total_before: u64,
}

impl View for KeyBuild {
    type V = KeyBuildView;

    closed spec fn view(&self) -> KeyBuildView {
        KeyBuildView {
            bytes: canonical_key_byte_views_spec(self.bytes@),
            total_before: self.total_before,
        }
    }
}

impl KeyBuild {
    fn empty(total_before: u64) -> (build: Self)
        ensures
            build@ == (KeyBuildView { bytes: Seq::empty(), total_before }),
    {
        let build = Self { bytes: Vec::new(), total_before };
        proof {
            reveal(canonical_key_byte_views_spec);
        }
        build
    }

    fn push(
        &mut self,
        value: u8,
        source_byte_offset: u64,
        limits: CanonicalScalarKeyLimits,
    ) -> (result: Result<(), CanonicalScalarKeyError>)
        ensures
            canonical_key_push_spec(old(self)@, value, source_byte_offset, limits@)
                == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        let key_limit = canonical_scalar_key_effective_limit(
            limits.max_key_bytes(),
            MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
        );
        if self.bytes.len() as u64 >= key_limit {
            return Err(
                CanonicalScalarKeyError::at(
                    CanonicalScalarKeyErrorKind::KeyByteLimitExceeded,
                    source_byte_offset,
                ),
            );
        }
        let total_limit = canonical_scalar_key_effective_limit(
            limits.max_total_key_bytes(),
            MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
        );
        if self.total_before >= total_limit {
            return Err(
                CanonicalScalarKeyError::at(
                    CanonicalScalarKeyErrorKind::TotalKeyByteLimitExceeded,
                    source_byte_offset,
                ),
            );
        }
        if self.bytes.len() as u64 >= total_limit - self.total_before {
            return Err(
                CanonicalScalarKeyError::at(
                    CanonicalScalarKeyErrorKind::TotalKeyByteLimitExceeded,
                    source_byte_offset,
                ),
            );
        }
        let byte = CanonicalKeyByte::new(value, source_byte_offset);
        proof {
            lemma_canonical_key_byte_views_push(self.bytes@, byte);
        }
        self.bytes.push(byte);
        proof {
            reveal(canonical_key_push_spec);
        }
        Ok(())
    }
}

pub open spec fn canonical_key_push_spec(
    build: KeyBuildView,
    value: u8,
    source_byte_offset: u64,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    let key_limit = canonical_scalar_key_effective_limit_spec(
        limits.max_key_bytes,
        MAX_PROFILE1_CANONICAL_SCALAR_KEY_BYTES,
    );
    let total_limit = canonical_scalar_key_effective_limit_spec(
        limits.max_total_key_bytes,
        MAX_PROFILE1_TOTAL_CANONICAL_SCALAR_KEY_BYTES,
    );
    if build.bytes.len() >= key_limit {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::KeyByteLimitExceeded,
                byte_offset: source_byte_offset,
            },
        )
    } else if build.total_before >= total_limit {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::TotalKeyByteLimitExceeded,
                byte_offset: source_byte_offset,
            },
        )
    } else if build.bytes.len() >= total_limit - build.total_before {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::TotalKeyByteLimitExceeded,
                byte_offset: source_byte_offset,
            },
        )
    } else {
        Ok(
            KeyBuildView {
                bytes: build.bytes.push(CanonicalKeyByteView { value, source_byte_offset }),
                total_before: build.total_before,
            },
        )
    }
}

pub open spec fn canonical_append_u32_spec(
    build: KeyBuildView,
    value: u32,
    source: u64,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_key_push_spec(build, (value >> 24) as u8, source, limits) {
        Err(error) => Err(error),
        Ok(a) => match canonical_key_push_spec(a, (value >> 16) as u8, source, limits) {
            Err(error) => Err(error),
            Ok(b) => match canonical_key_push_spec(b, (value >> 8) as u8, source, limits) {
                Err(error) => Err(error),
                Ok(c) => canonical_key_push_spec(c, value as u8, source, limits),
            },
        },
    }
}

fn append_u32(
    build: &mut KeyBuild,
    value: u32,
    source: u64,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<(), CanonicalScalarKeyError>)
    ensures
        canonical_append_u32_spec(old(build)@, value, source, limits@) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    if let Err(error) = build.push((value >> 24) as u8, source, limits) {
        return Err(error);
    }
    if let Err(error) = build.push((value >> 16) as u8, source, limits) {
        return Err(error);
    }
    if let Err(error) = build.push((value >> 8) as u8, source, limits) {
        return Err(error);
    }
    build.push(value as u8, source, limits)
}

pub open spec fn canonical_append_u64_spec(
    build: KeyBuildView,
    value: u64,
    source: u64,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_append_u32_spec(build, (value >> 32) as u32, source, limits) {
        Err(error) => Err(error),
        Ok(next) => canonical_append_u32_spec(next, value as u32, source, limits),
    }
}

fn append_u64(
    build: &mut KeyBuild,
    value: u64,
    source: u64,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<(), CanonicalScalarKeyError>)
    ensures
        canonical_append_u64_spec(old(build)@, value, source, limits@) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    if let Err(error) = append_u32(build, (value >> 32) as u32, source, limits) {
        return Err(error);
    }
    append_u32(build, value as u32, source, limits)
}

pub open spec fn canonical_scalar_value_code_points_spec(scalar: ResolvedScalarView) -> Seq<u32> {
    match scalar.presentation.decoded {
        Some(decoded) => Seq::new(
            decoded.content.len(),
            |index: int| decoded.content[index].code_point,
        ),
        None => Seq::empty(),
    }
}

pub open spec fn canonical_scalar_value_sources_spec(scalar: ResolvedScalarView) -> Seq<u64> {
    match scalar.presentation.decoded {
        Some(decoded) => Seq::new(
            decoded.content.len(),
            |index: int| decoded.content[index].byte_start,
        ),
        None => Seq::empty(),
    }
}

pub closed spec fn canonical_append_same_source_u32_tail_spec(
    build: KeyBuildView,
    values: Seq<u32>,
    source: u64,
    index: nat,
    fuel: nat,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView>
    decreases fuel,
{
    if index >= values.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: source,
            },
        )
    } else {
        match canonical_append_u32_spec(build, values[index as int], source, limits) {
            Err(error) => Err(error),
            Ok(next) => canonical_append_same_source_u32_tail_spec(
                next,
                values,
                source,
                (index + 1) as nat,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

fn append_same_source_u32_values(
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    values: &[u32],
    source: u64,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_append_same_source_u32_tail_spec(
            input,
            values@,
            source,
            0,
            values@.len() as nat,
            limits@,
        ) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = canonical_append_same_source_u32_tail_spec(
        input,
        values@,
        source,
        0,
        values@.len() as nat,
        limits@,
    );
    let mut current = build;
    let mut index = 0usize;
    while index < values.len()
        invariant
            index <= values.len(),
            expected == canonical_append_same_source_u32_tail_spec(
                input,
                values@,
                source,
                0,
                values@.len() as nat,
                limits@,
            ),
            expected == canonical_append_same_source_u32_tail_spec(
                current@,
                values@,
                source,
                index as nat,
                (values.len() - index) as nat,
                limits@,
            ),
        decreases values.len() - index,
    {
        let step = append_u32(&mut current, values[index], source, limits);
        match step {
            Err(error) => {
                proof {
                    reveal(canonical_append_same_source_u32_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(canonical_append_same_source_u32_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(canonical_append_same_source_u32_tail_spec);
    }
    Ok(current)
}

pub closed spec fn canonical_append_decoded_content_tail_spec(
    build: KeyBuildView,
    content: Seq<crate::scalar_decode::DecodedContentScalarView>,
    index: nat,
    fuel: nat,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView>
    decreases fuel,
{
    if index >= content.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let point = content[index as int];
        match canonical_append_u32_spec(build, point.code_point, point.byte_start, limits) {
            Err(error) => Err(error),
            Ok(next) => canonical_append_decoded_content_tail_spec(
                next,
                content,
                (index + 1) as nat,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

fn append_decoded_content(
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    content: &[crate::scalar_decode::DecodedContentScalar],
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_append_decoded_content_tail_spec(
            input,
            crate::scalar_decode::decoded_content_scalar_views_spec(content@),
            0,
            content@.len() as nat,
            limits@,
        ) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = canonical_append_decoded_content_tail_spec(
        input,
        crate::scalar_decode::decoded_content_scalar_views_spec(content@),
        0,
        content@.len() as nat,
        limits@,
    );
    let mut current = build;
    let mut index = 0usize;
    while index < content.len()
        invariant
            index <= content.len(),
            expected == canonical_append_decoded_content_tail_spec(
                input,
                crate::scalar_decode::decoded_content_scalar_views_spec(content@),
                0,
                content@.len() as nat,
                limits@,
            ),
            expected == canonical_append_decoded_content_tail_spec(
                current@,
                crate::scalar_decode::decoded_content_scalar_views_spec(content@),
                index as nat,
                (content.len() - index) as nat,
                limits@,
            ),
        decreases content.len() - index,
    {
        proof {
            reveal(crate::scalar_decode::decoded_content_scalar_views_spec);
        }
        let step = append_u32(
            &mut current,
            content[index].code_point(),
            content[index].byte_start(),
            limits,
        );
        match step {
            Err(error) => {
                proof {
                    reveal(canonical_append_decoded_content_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(canonical_append_decoded_content_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(canonical_append_decoded_content_tail_spec);
    }
    Ok(current)
}

pub closed spec fn canonical_append_tag_content_tail_spec(
    build: KeyBuildView,
    content: Seq<crate::resolve_tag::ResolvedTagCodePointView>,
    index: nat,
    fuel: nat,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView>
    decreases fuel,
{
    if index >= content.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let point = content[index as int];
        match canonical_append_u32_spec(build, point.code_point, point.byte_start, limits) {
            Err(error) => Err(error),
            Ok(next) => canonical_append_tag_content_tail_spec(
                next,
                content,
                (index + 1) as nat,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

fn append_tag_content(
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    content: &[crate::resolve_tag::ResolvedTagCodePoint],
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_append_tag_content_tail_spec(
            input,
            crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
            0,
            content@.len() as nat,
            limits@,
        ) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = canonical_append_tag_content_tail_spec(
        input,
        crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
        0,
        content@.len() as nat,
        limits@,
    );
    let mut current = build;
    let mut index = 0usize;
    while index < content.len()
        invariant
            index <= content.len(),
            expected == canonical_append_tag_content_tail_spec(
                input,
                crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
                0,
                content@.len() as nat,
                limits@,
            ),
            expected == canonical_append_tag_content_tail_spec(
                current@,
                crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
                index as nat,
                (content.len() - index) as nat,
                limits@,
            ),
        decreases content.len() - index,
    {
        proof {
            reveal(crate::resolve_tag::resolved_tag_code_point_views_spec);
        }
        let step = append_u32(
            &mut current,
            content[index].code_point(),
            content[index].byte_start(),
            limits,
        );
        match step {
            Err(error) => {
                proof {
                    reveal(canonical_append_tag_content_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(canonical_append_tag_content_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(canonical_append_tag_content_tail_spec);
    }
    Ok(current)
}

pub closed spec fn canonical_encode_tag_identity_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match scalar.tag {
        ResolvedScalarTag::CustomGlobal | ResolvedScalarTag::CustomLocal => {
            match scalar.explicit_tag {
                None => Err(
                    CanonicalScalarKeyErrorView {
                        kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node_byte,
                    },
                ),
                Some(tag) => match canonical_append_u64_spec(
                    build,
                    tag.content.len() as u64,
                    node_byte,
                    limits,
                ) {
                    Err(error) => Err(error),
                    Ok(next) => canonical_append_tag_content_tail_spec(
                        next,
                        tag.content,
                        0,
                        tag.content.len(),
                        limits,
                    ),
                },
            }
        },
        _ => Ok(build),
    }
}

pub closed spec fn canonical_encode_scalar_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_encode_scalar_prefix_spec(scalar, node_byte, build, limits) {
        Err(error) => Err(error),
        Ok(prefix) => canonical_encode_scalar_value_spec(scalar, node_byte, prefix, limits),
    }
}

pub closed spec fn canonical_encode_scalar_prefix_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    let tag_marker = match scalar.tag {
        ResolvedScalarTag::CoreNull => 0x10u8,
        ResolvedScalarTag::CoreBoolean => 0x11u8,
        ResolvedScalarTag::CoreInteger => 0x12u8,
        ResolvedScalarTag::CoreFloat => 0x13u8,
        ResolvedScalarTag::CoreString => 0x14u8,
        ResolvedScalarTag::CustomGlobal => 0x15u8,
        ResolvedScalarTag::CustomLocal => 0x16u8,
    };
    match canonical_key_push_spec(build, 0x43, node_byte, limits) {
        Err(e) => Err(e),
        Ok(a) => match canonical_key_push_spec(a, 0x53, node_byte, limits) {
            Err(e) => Err(e),
            Ok(b) => match canonical_key_push_spec(b, 0x4b, node_byte, limits) {
                Err(e) => Err(e),
                Ok(c) => match canonical_key_push_spec(c, 1, node_byte, limits) {
                    Err(e) => Err(e),
                    Ok(d) => match canonical_key_push_spec(d, tag_marker, node_byte, limits) {
                        Err(e) => Err(e),
                        Ok(tagged) => canonical_encode_tag_identity_spec(
                            scalar,
                            node_byte,
                            tagged,
                            limits,
                        ),
                    },
                },
            },
        },
    }
}

pub closed spec fn canonical_encode_scalar_value_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match scalar.value {
        ResolvedScalarValueView::Null => canonical_key_push_spec(build, 0x20, node_byte, limits),
        ResolvedScalarValueView::Boolean(value) => match canonical_key_push_spec(
            build,
            0x21,
            node_byte,
            limits,
        ) {
            Err(e) => Err(e),
            Ok(a) => canonical_key_push_spec(
                a,
                if value {
                    1
                } else {
                    0
                },
                node_byte,
                limits,
            ),
        },
        ResolvedScalarValueView::Integer(integer) => canonical_encode_integer_value_spec(
            integer,
            node_byte,
            build,
            limits,
        ),
        ResolvedScalarValueView::FiniteFloat(float) => canonical_encode_float_value_spec(
            float,
            node_byte,
            build,
            limits,
        ),
        ResolvedScalarValueView::PositiveInfinity => canonical_key_push_spec(
            build,
            0x24,
            node_byte,
            limits,
        ),
        ResolvedScalarValueView::NegativeInfinity => canonical_key_push_spec(
            build,
            0x25,
            node_byte,
            limits,
        ),
        ResolvedScalarValueView::NotANumber => canonical_key_push_spec(
            build,
            0x26,
            node_byte,
            limits,
        ),
        ResolvedScalarValueView::String => canonical_encode_string_value_spec(
            scalar,
            node_byte,
            build,
            limits,
        ),
    }
}

pub closed spec fn canonical_encode_integer_value_spec(
    integer: crate::resolve_integer::CoreIntegerView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_key_push_spec(build, 0x22, node_byte, limits) {
        Err(e) => Err(e),
        Ok(a) => match canonical_key_push_spec(
            a,
            if integer.negative {
                1
            } else {
                0
            },
            node_byte,
            limits,
        ) {
            Err(e) => Err(e),
            Ok(b) => match canonical_append_u64_spec(
                b,
                integer.limbs.len() as u64,
                node_byte,
                limits,
            ) {
                Err(e) => Err(e),
                Ok(c) => canonical_append_same_source_u32_tail_spec(
                    c,
                    integer.limbs,
                    node_byte,
                    0,
                    integer.limbs.len(),
                    limits,
                ),
            },
        },
    }
}

pub closed spec fn canonical_encode_float_value_spec(
    float: crate::resolve_float::CoreFiniteFloatView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_key_push_spec(build, 0x23, node_byte, limits) {
        Err(e) => Err(e),
        Ok(a) => match canonical_key_push_spec(
            a,
            if float.negative {
                1
            } else {
                0
            },
            node_byte,
            limits,
        ) {
            Err(e) => Err(e),
            Ok(b) => match canonical_append_u64_spec(
                b,
                float.coefficient_digits_le.len() as u64,
                node_byte,
                limits,
            ) {
                Err(e) => Err(e),
                Ok(c) => match canonical_append_u8_values_tail_spec(
                    c,
                    float.coefficient_digits_le,
                    node_byte,
                    0,
                    float.coefficient_digits_le.len(),
                    limits,
                ) {
                    Err(e) => Err(e),
                    Ok(d) => match canonical_key_push_spec(
                        d,
                        if float.exponent_negative {
                            1
                        } else {
                            0
                        },
                        node_byte,
                        limits,
                    ) {
                        Err(e) => Err(e),
                        Ok(f) => match canonical_append_u64_spec(
                            f,
                            float.exponent_digits_le.len() as u64,
                            node_byte,
                            limits,
                        ) {
                            Err(e) => Err(e),
                            Ok(g) => canonical_append_u8_values_tail_spec(
                                g,
                                float.exponent_digits_le,
                                node_byte,
                                0,
                                float.exponent_digits_le.len(),
                                limits,
                            ),
                        },
                    },
                },
            },
        },
    }
}

pub closed spec fn canonical_encode_string_value_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: KeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_key_push_spec(build, 0x27, node_byte, limits) {
        Err(e) => Err(e),
        Ok(a) => match scalar.presentation.decoded {
            None => canonical_append_u64_spec(a, 0, node_byte, limits),
            Some(decoded) => match canonical_append_u64_spec(
                a,
                decoded.content.len() as u64,
                node_byte,
                limits,
            ) {
                Err(e) => Err(e),
                Ok(b) => canonical_append_decoded_content_tail_spec(
                    b,
                    decoded.content,
                    0,
                    decoded.content.len(),
                    limits,
                ),
            },
        },
    }
}

pub closed spec fn canonical_append_u8_values_tail_spec(
    build: KeyBuildView,
    values: Seq<u8>,
    source: u64,
    index: nat,
    fuel: nat,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<KeyBuildView, CanonicalScalarKeyErrorView>
    decreases fuel,
{
    if index >= values.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: source,
            },
        )
    } else {
        match canonical_key_push_spec(build, values[index as int], source, limits) {
            Err(e) => Err(e),
            Ok(next) => canonical_append_u8_values_tail_spec(
                next,
                values,
                source,
                (index + 1) as nat,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

fn append_u8_values(
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    values: &[u8],
    source: u64,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_append_u8_values_tail_spec(
            input,
            values@,
            source,
            0,
            values@.len() as nat,
            limits@,
        ) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = canonical_append_u8_values_tail_spec(
        input,
        values@,
        source,
        0,
        values@.len() as nat,
        limits@,
    );
    let mut current = build;
    let mut index = 0usize;
    while index < values.len()
        invariant
            index <= values.len(),
            expected == canonical_append_u8_values_tail_spec(
                input,
                values@,
                source,
                0,
                values@.len() as nat,
                limits@,
            ),
            expected == canonical_append_u8_values_tail_spec(
                current@,
                values@,
                source,
                index as nat,
                (values.len() - index) as nat,
                limits@,
            ),
        decreases values.len() - index,
    {
        let step = current.push(values[index], source, limits);
        match step {
            Err(error) => {
                proof {
                    reveal(canonical_append_u8_values_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(canonical_append_u8_values_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(canonical_append_u8_values_tail_spec);
    }
    Ok(current)
}

fn encode_scalar_prefix(
    scalar: &ResolvedScalar,
    node_byte: u64,
    build: KeyBuild,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    ensures
        canonical_encode_scalar_prefix_spec(scalar@, node_byte, build@, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let mut current = build;
    let tag_marker = match scalar.tag() {
        ResolvedScalarTag::CoreNull => 0x10,
        ResolvedScalarTag::CoreBoolean => 0x11,
        ResolvedScalarTag::CoreInteger => 0x12,
        ResolvedScalarTag::CoreFloat => 0x13,
        ResolvedScalarTag::CoreString => 0x14,
        ResolvedScalarTag::CustomGlobal => 0x15,
        ResolvedScalarTag::CustomLocal => 0x16,
    };
    if let Err(e) = current.push(0x43, node_byte, limits) {
        return Err(e);
    }
    if let Err(e) = current.push(0x53, node_byte, limits) {
        return Err(e);
    }
    if let Err(e) = current.push(0x4b, node_byte, limits) {
        return Err(e);
    }
    if let Err(e) = current.push(1, node_byte, limits) {
        return Err(e);
    }
    if let Err(e) = current.push(tag_marker, node_byte, limits) {
        return Err(e);
    }
    if matches!(
        scalar.tag(),
        ResolvedScalarTag::CustomGlobal | ResolvedScalarTag::CustomLocal
    ) {
        let tag = match scalar.explicit_tag() {
            Some(tag) => tag,
            None => return Err(
                CanonicalScalarKeyError::at(
                    CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                    node_byte,
                ),
            ),
        };
        let content = tag.content();
        if let Err(e) = append_u64(&mut current, content.len() as u64, node_byte, limits) {
            return Err(e);
        }
        let ghost tag_input = current@;
        current =
        match append_tag_content(current, Ghost(tag_input), content, limits) {
            Err(error) => return Err(error),
            Ok(next) => next,
        };
    }
    Ok(current)
}

fn encode_integer_value(
    integer: &crate::resolve_integer::CoreInteger,
    node_byte: u64,
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_encode_integer_value_spec(integer@, node_byte, input, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let mut current = build;
    if let Err(error) = current.push(0x22, node_byte, limits) {
        return Err(error);
    }
    if let Err(error) = current.push(
        if integer.negative() {
            1
        } else {
            0
        },
        node_byte,
        limits,
    ) {
        return Err(error);
    }
    let limbs = integer.limbs();
    if let Err(error) = append_u64(&mut current, limbs.len() as u64, node_byte, limits) {
        return Err(error);
    }
    let ghost input = current@;
    append_same_source_u32_values(current, Ghost(input), limbs, node_byte, limits)
}

fn encode_float_value(
    float: &crate::resolve_float::CoreFiniteFloat,
    node_byte: u64,
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_encode_float_value_spec(float@, node_byte, input, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let mut current = build;
    if let Err(error) = current.push(0x23, node_byte, limits) {
        return Err(error);
    }
    if let Err(error) = current.push(
        if float.negative() {
            1
        } else {
            0
        },
        node_byte,
        limits,
    ) {
        return Err(error);
    }
    let coefficient = float.coefficient_digits_le();
    if let Err(error) = append_u64(&mut current, coefficient.len() as u64, node_byte, limits) {
        return Err(error);
    }
    let ghost coefficient_input = current@;
    current =
    match append_u8_values(current, Ghost(coefficient_input), coefficient, node_byte, limits) {
        Err(error) => return Err(error),
        Ok(next) => next,
    };
    if let Err(error) = current.push(
        if float.exponent_negative() {
            1
        } else {
            0
        },
        node_byte,
        limits,
    ) {
        return Err(error);
    }
    let exponent = float.exponent_digits_le();
    if let Err(error) = append_u64(&mut current, exponent.len() as u64, node_byte, limits) {
        return Err(error);
    }
    let ghost exponent_input = current@;
    append_u8_values(current, Ghost(exponent_input), exponent, node_byte, limits)
}

fn encode_string_value(
    scalar: &ResolvedScalar,
    node_byte: u64,
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_encode_string_value_spec(scalar@, node_byte, input, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let mut current = build;
    if let Err(error) = current.push(0x27, node_byte, limits) {
        return Err(error);
    }
    match scalar.presentation().decoded() {
        None => {
            if let Err(error) = append_u64(&mut current, 0, node_byte, limits) {
                Err(error)
            } else {
                Ok(current)
            }
        },
        Some(decoded) => {
            let content = decoded.content();
            if let Err(error) = append_u64(&mut current, content.len() as u64, node_byte, limits) {
                return Err(error);
            }
            let ghost content_input = current@;
            append_decoded_content(current, Ghost(content_input), content, limits)
        },
    }
}

fn push_owned_key_byte(
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    value: u8,
    source: u64,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        canonical_key_push_spec(input, value, source, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let mut current = build;
    match current.push(value, source, limits) {
        Err(error) => Err(error),
        Ok(()) => Ok(current),
    }
}

fn encode_boolean_value(
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    value: bool,
    node_byte: u64,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        build@ == input,
    ensures
        match canonical_key_push_spec(input, 0x21, node_byte, limits@) {
            Err(error) => Err(error),
            Ok(next) => canonical_key_push_spec(
                next,
                if value {
                    1
                } else {
                    0
                },
                node_byte,
                limits@,
            ),
        } == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let first = match push_owned_key_byte(build, Ghost(input), 0x21, node_byte, limits) {
        Err(error) => return Err(error),
        Ok(next) => next,
    };
    let ghost second_input = first@;
    push_owned_key_byte(
        first,
        Ghost(second_input),
        if value {
            1
        } else {
            0
        },
        node_byte,
        limits,
    )
}

#[verifier::rlimit(50)]
fn encode_scalar_value(
    scalar: &ResolvedScalar,
    Ghost(scalar_view): Ghost<ResolvedScalarView>,
    node_byte: u64,
    build: KeyBuild,
    Ghost(input): Ghost<KeyBuildView>,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    requires
        scalar@ == scalar_view,
        build@ == input,
    ensures
        canonical_encode_scalar_value_spec(scalar_view, node_byte, input, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let current = build;
    proof {
        assert(current@ == input);
    }
    match scalar.value() {
        ResolvedScalarValue::Null => {
            let result = push_owned_key_byte(current, Ghost(input), 0x20, node_byte, limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::Null);
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::Boolean(value) => {
            let result = encode_boolean_value(current, Ghost(input), *value, node_byte, limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::Boolean(*value));
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::Integer(integer) => {
            let result = encode_integer_value(integer, node_byte, current, Ghost(input), limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::Integer(integer@));
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::FiniteFloat(float) => {
            let result = encode_float_value(float, node_byte, current, Ghost(input), limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::FiniteFloat(float@));
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::PositiveInfinity => {
            let result = push_owned_key_byte(current, Ghost(input), 0x24, node_byte, limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::PositiveInfinity);
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::NegativeInfinity => {
            let result = push_owned_key_byte(current, Ghost(input), 0x25, node_byte, limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::NegativeInfinity);
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::NotANumber => {
            let result = push_owned_key_byte(current, Ghost(input), 0x26, node_byte, limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::NotANumber);
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
        ResolvedScalarValue::String => {
            let result = encode_string_value(scalar, node_byte, current, Ghost(input), limits);
            proof {
                assert(scalar_view.value == ResolvedScalarValueView::String);
                reveal(canonical_encode_scalar_value_spec);
            }
            result
        },
    }
}

fn encode_scalar(
    scalar: &ResolvedScalar,
    node_byte: u64,
    build: KeyBuild,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<KeyBuild, CanonicalScalarKeyError>)
    ensures
        canonical_encode_scalar_spec(scalar@, node_byte, build@, limits@) == match result {
            Ok(final_build) => Ok(final_build@),
            Err(error) => Err(error@),
        },
{
    let prefix = encode_scalar_prefix(scalar, node_byte, build, limits);
    match prefix {
        Err(error) => {
            proof {
                reveal(canonical_encode_scalar_spec);
            }
            Err(error)
        },
        Ok(prefix_build) => {
            let ghost value_input = prefix_build@;
            let ghost scalar_view = scalar@;
            let value = encode_scalar_value(
                scalar,
                Ghost(scalar_view),
                node_byte,
                prefix_build,
                Ghost(value_input),
                limits,
            );
            proof {
                reveal(canonical_encode_scalar_spec);
            }
            value
        },
    }
}

#[verifier::ext_equal]
pub struct CanonicalScalarKeyBuildView {
    pub records: Seq<CanonicalScalarKeyRecordView>,
    pub total_key_bytes: u64,
}

struct CanonicalScalarKeyBuild {
    records: Vec<CanonicalScalarKeyRecord>,
    total_key_bytes: u64,
}

impl View for CanonicalScalarKeyBuild {
    type V = CanonicalScalarKeyBuildView;

    closed spec fn view(&self) -> CanonicalScalarKeyBuildView {
        CanonicalScalarKeyBuildView {
            records: canonical_scalar_key_record_views_spec(self.records@),
            total_key_bytes: self.total_key_bytes,
        }
    }
}

impl CanonicalScalarKeyBuild {
    fn empty() -> (build: Self)
        ensures
            build@ == (CanonicalScalarKeyBuildView { records: Seq::empty(), total_key_bytes: 0 }),
    {
        let build = Self { records: Vec::new(), total_key_bytes: 0 };
        proof {
            reveal(canonical_scalar_key_record_views_spec);
        }
        build
    }

    fn push(&mut self, record: CanonicalScalarKeyRecord) -> (result: Result<
        (),
        CanonicalScalarKeyError,
    >)
        ensures
            canonical_scalar_key_build_push_spec(old(self)@, record@) == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        if record.bytes().len() as u64 > u64::MAX - self.total_key_bytes {
            return Err(
                CanonicalScalarKeyError::at(
                    CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                    record.byte_start(),
                ),
            );
        }
        self.total_key_bytes += record.bytes().len() as u64;
        proof {
            lemma_canonical_scalar_key_record_views_push(self.records@, record);
        }
        self.records.push(record);
        proof {
            reveal(canonical_scalar_key_build_push_spec);
        }
        Ok(())
    }
}

pub open spec fn canonical_scalar_key_build_push_spec(
    build: CanonicalScalarKeyBuildView,
    record: CanonicalScalarKeyRecordView,
) -> Result<CanonicalScalarKeyBuildView, CanonicalScalarKeyErrorView> {
    if record.bytes.len() > u64::MAX - build.total_key_bytes {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: record.byte_start,
            },
        )
    } else {
        Ok(
            CanonicalScalarKeyBuildView {
                records: build.records.push(record),
                total_key_bytes: (build.total_key_bytes + record.bytes.len()) as u64,
            },
        )
    }
}

pub open spec fn canonical_scalar_record_step_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: CanonicalScalarKeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<CanonicalScalarKeyBuildView, CanonicalScalarKeyErrorView> {
    let record_limit = canonical_scalar_key_effective_limit_spec(
        limits.max_records,
        MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS,
    );
    if build.records.len() >= record_limit {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::RecordLimitExceeded,
                byte_offset: node_byte,
            },
        )
    } else {
        canonical_scalar_record_encode_spec(scalar, node_byte, build, limits)
    }
}

pub open spec fn canonical_scalar_record_encode_spec(
    scalar: ResolvedScalarView,
    node_byte: u64,
    build: CanonicalScalarKeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<CanonicalScalarKeyBuildView, CanonicalScalarKeyErrorView> {
    match canonical_encode_scalar_spec(
        scalar,
        node_byte,
        KeyBuildView { bytes: Seq::empty(), total_before: build.total_key_bytes },
        limits,
    ) {
        Err(error) => Err(error),
        Ok(key) => canonical_scalar_key_build_push_spec(
            build,
            CanonicalScalarKeyRecordView {
                node_index: scalar.node_index,
                byte_start: node_byte,
                bytes: key.bytes,
            },
        ),
    }
}

fn compose_canonical_scalar_record_encode(
    scalar: &ResolvedScalar,
    node_byte: u64,
    build: &mut CanonicalScalarKeyBuild,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<(), CanonicalScalarKeyError>)
    ensures
        canonical_scalar_record_encode_spec(scalar@, node_byte, old(build)@, limits@)
            == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let key = KeyBuild::empty(build.total_key_bytes);
    let encoded = encode_scalar(scalar, node_byte, key, limits);
    match encoded {
        Err(error) => {
            proof {
                reveal(canonical_scalar_record_encode_spec);
            }
            Err(error)
        },
        Ok(key) => {
            let record = CanonicalScalarKeyRecord::new(scalar.node_index(), node_byte, key.bytes);
            proof {
                reveal(canonical_scalar_record_encode_spec);
            }
            build.push(record)
        },
    }
}

fn compose_canonical_scalar_record_step(
    scalar: &ResolvedScalar,
    node_byte: u64,
    build: &mut CanonicalScalarKeyBuild,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<(), CanonicalScalarKeyError>)
    ensures
        canonical_scalar_record_step_spec(scalar@, node_byte, old(build)@, limits@)
            == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let record_limit = canonical_scalar_key_effective_limit(
        limits.max_records(),
        MAX_PROFILE1_CANONICAL_SCALAR_KEY_RECORDS,
    );
    if build.records.len() as u64 >= record_limit {
        let error = CanonicalScalarKeyError::at(
            CanonicalScalarKeyErrorKind::RecordLimitExceeded,
            node_byte,
        );
        proof {
            reveal(canonical_scalar_record_step_spec);
            assert(canonical_scalar_record_step_spec(scalar@, node_byte, build@, limits@) == Err(
                error@,
            ));
        }
        return Err(error);
    }
    let result = compose_canonical_scalar_record_encode(scalar, node_byte, build, limits);
    proof {
        reveal(canonical_scalar_record_step_spec);
    }
    result
}

pub closed spec fn canonical_scalar_table_tail_spec(
    graph: AcyclicSemanticGraphSourceView,
    scalar_index: nat,
    fuel: nat,
    build: CanonicalScalarKeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<CanonicalScalarKeyBuildView, CanonicalScalarKeyErrorView>
    decreases fuel,
{
    if scalar_index >= graph.node_table.scalars.scalars.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: graph.source_len_bytes,
            },
        )
    } else {
        match canonical_scalar_table_item_spec(graph, scalar_index, build, limits) {
            Err(error) => Err(error),
            Ok(next) => canonical_scalar_table_tail_spec(
                graph,
                (scalar_index + 1) as nat,
                (fuel - 1) as nat,
                next,
                limits,
            ),
        }
    }
}

pub open spec fn canonical_scalar_table_item_spec(
    graph: AcyclicSemanticGraphSourceView,
    scalar_index: nat,
    build: CanonicalScalarKeyBuildView,
    limits: CanonicalScalarKeyLimitsView,
) -> Result<CanonicalScalarKeyBuildView, CanonicalScalarKeyErrorView> {
    if scalar_index >= graph.node_table.scalars.scalars.len() {
        Err(
            CanonicalScalarKeyErrorView {
                kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                byte_offset: graph.source_len_bytes,
            },
        )
    } else {
        let scalar = graph.node_table.scalars.scalars[scalar_index as int];
        if scalar.node_index >= graph.node_table.nodes.len() {
            Err(
                CanonicalScalarKeyErrorView {
                    kind: CanonicalScalarKeyErrorKind::InternalInvariantViolation,
                    byte_offset: graph.source_len_bytes,
                },
            )
        } else {
            let node_byte = graph.node_table.nodes[scalar.node_index as int].byte_start;
            canonical_scalar_record_step_spec(scalar, node_byte, build, limits)
        }
    }
}

fn compose_canonical_scalar_table_item(
    graph: &AcyclicSemanticGraphSource,
    scalar_index: usize,
    build: &mut CanonicalScalarKeyBuild,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<(), CanonicalScalarKeyError>)
    ensures
        canonical_scalar_table_item_spec(graph@, scalar_index as nat, old(build)@, limits@)
            == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let scalars = graph.node_table().scalars().scalars();
    if scalar_index >= scalars.len() {
        let error = CanonicalScalarKeyError::at(
            CanonicalScalarKeyErrorKind::InternalInvariantViolation,
            graph.source_len_bytes(),
        );
        proof {
            reveal(canonical_scalar_table_item_spec);
        }
        return Err(error);
    }
    let scalar = &scalars[scalar_index];
    let nodes = graph.node_table().nodes();
    if scalar.node_index() >= nodes.len() as u64 {
        let error = CanonicalScalarKeyError::at(
            CanonicalScalarKeyErrorKind::InternalInvariantViolation,
            graph.source_len_bytes(),
        );
        proof {
            reveal(canonical_scalar_table_item_spec);
        }
        return Err(error);
    }
    let node_byte = nodes[scalar.node_index() as usize].byte_start();
    let result = compose_canonical_scalar_record_step(scalar, node_byte, build, limits);
    proof {
        reveal(canonical_scalar_table_item_spec);
    }
    result
}

#[verifier::rlimit(50)]
fn compose_canonical_scalar_table(
    graph: &AcyclicSemanticGraphSource,
    limits: CanonicalScalarKeyLimits,
) -> (result: Result<CanonicalScalarKeyBuild, CanonicalScalarKeyError>)
    ensures
        canonical_scalar_table_tail_spec(
            graph@,
            0,
            graph@.node_table.scalars.scalars.len(),
            CanonicalScalarKeyBuildView { records: Seq::empty(), total_key_bytes: 0 },
            limits@,
        ) == match result {
            Ok(build) => Ok(build@),
            Err(error) => Err(error@),
        },
{
    let scalars = graph.node_table().scalars().scalars();
    let mut build = CanonicalScalarKeyBuild::empty();
    let mut scalar_index = 0usize;
    let ghost expected = canonical_scalar_table_tail_spec(
        graph@,
        0,
        graph@.node_table.scalars.scalars.len(),
        build@,
        limits@,
    );
    while scalar_index < scalars.len()
        invariant
            scalar_index <= scalars.len(),
            scalars.len() == graph@.node_table.scalars.scalars.len(),
            expected == canonical_scalar_table_tail_spec(
                graph@,
                0,
                graph@.node_table.scalars.scalars.len(),
                CanonicalScalarKeyBuildView { records: Seq::empty(), total_key_bytes: 0 },
                limits@,
            ),
            expected == canonical_scalar_table_tail_spec(
                graph@,
                scalar_index as nat,
                (scalars.len() - scalar_index) as nat,
                build@,
                limits@,
            ),
        decreases scalars.len() - scalar_index,
    {
        let step = compose_canonical_scalar_table_item(graph, scalar_index, &mut build, limits);
        match step {
            Err(error) => {
                proof {
                    reveal(canonical_scalar_table_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(canonical_scalar_table_tail_spec);
        }
        scalar_index += 1;
    }
    proof {
        reveal(canonical_scalar_table_tail_spec);
    }
    Ok(build)
}

pub open spec fn finalize_canonical_scalar_key_spec(
    graph: AcyclicSemanticGraphSourceView,
    result: Result<CanonicalScalarKeyBuildView, CanonicalScalarKeyErrorView>,
) -> Result<CanonicalScalarKeySourceView, CanonicalScalarKeyErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => Ok(
            CanonicalScalarKeySourceView {
                profile_version: graph.profile_version,
                transformation_version: CANONICAL_SCALAR_KEY_TRANSFORMATION_VERSION,
                source_len_bytes: graph.source_len_bytes,
                input_node_count: graph.node_table.nodes.len() as u64,
                total_key_bytes: build.total_key_bytes,
                graph,
                records: build.records,
            },
        ),
    }
}

pub open spec fn compose_profile1_canonical_scalar_keys_spec(
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
    key_limits: CanonicalScalarKeyLimitsView,
) -> Result<CanonicalScalarKeySourceView, CanonicalScalarKeyErrorView> {
    match crate::resolve_alias_cycle::resolve_profile1_alias_cycles_spec(
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
    ) {
        Err(error) => Err(map_alias_cycle_error_spec(error)),
        Ok(graph) => finalize_canonical_scalar_key_spec(
            graph,
            canonical_scalar_table_tail_spec(
                graph,
                0,
                graph.node_table.scalars.scalars.len(),
                CanonicalScalarKeyBuildView { records: Seq::empty(), total_key_bytes: 0 },
                key_limits,
            ),
        ),
    }
}

pub open spec fn canonical_scalar_key_source_well_formed_spec(
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
    key_limits: CanonicalScalarKeyLimitsView,
    source: CanonicalScalarKeySourceView,
) -> bool {
    compose_profile1_canonical_scalar_keys_spec(
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
        key_limits,
    ) == Ok(source)
}

pub proof fn lemma_canonical_scalar_key_success_is_well_formed(
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
    key_limits: CanonicalScalarKeyLimitsView,
    source: CanonicalScalarKeySourceView,
)
    requires
        compose_profile1_canonical_scalar_keys_spec(
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
            key_limits,
        ) == Ok(source),
    ensures
        canonical_scalar_key_source_well_formed_spec(
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
            key_limits,
            source,
        ),
{
    reveal(canonical_scalar_key_source_well_formed_spec);
}

pub proof fn lemma_canonical_scalar_key_well_formed_authenticates_exact_result(
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
    key_limits: CanonicalScalarKeyLimitsView,
    source: CanonicalScalarKeySourceView,
)
    requires
        canonical_scalar_key_source_well_formed_spec(
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
            key_limits,
            source,
        ),
    ensures
        compose_profile1_canonical_scalar_keys_spec(
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
            key_limits,
        ) == Ok(source),
{
    reveal(canonical_scalar_key_source_well_formed_spec);
}

#[allow(clippy::too_many_arguments)]
pub fn compose_profile1_canonical_scalar_keys(
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
    key_limits: CanonicalScalarKeyLimits,
) -> (result: Result<CanonicalScalarKeySource, CanonicalScalarKeyError>)
    ensures
        compose_profile1_canonical_scalar_keys_spec(
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
            key_limits@,
        ) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let graph = match crate::resolve_alias_cycle::resolve_profile1_alias_cycles(
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
    ) {
        Err(error) => return Err(map_alias_cycle_error(error)),
        Ok(graph) => graph,
    };
    let build = match compose_canonical_scalar_table(&graph, key_limits) {
        Err(error) => {
            proof {
                reveal(compose_profile1_canonical_scalar_keys_spec);
                reveal(finalize_canonical_scalar_key_spec);
            }
            return Err(error);
        },
        Ok(build) => build,
    };
    let source = CanonicalScalarKeySource::new(graph, build.records, build.total_key_bytes);
    proof {
        reveal(compose_profile1_canonical_scalar_keys_spec);
        reveal(finalize_canonical_scalar_key_spec);
    }
    Ok(source)
}

} // verus!
