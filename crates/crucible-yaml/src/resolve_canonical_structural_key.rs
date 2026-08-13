#![allow(clippy::question_mark, clippy::single_match)]
//! Verified canonical identities for every resolved YAML semantic node.
//!
//! Scalar identities reuse the presentation-independent scalar encoding. Aliases are exactly
//! transparent. Sequences encode children in source order, while mappings encode a lexicographically
//! sorted sequence of canonical key/value identity pairs. Collection tags, including the complete
//! resolved spelling of custom tags, are part of the identity.
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
use crate::resolve_alias_cycle::AliasCycleLimits;
#[allow(unused_imports)]
use crate::resolve_alias_cycle::AliasCycleLimitsView;
use crate::resolve_anchor::AnchorAliasLimits;
#[allow(unused_imports)]
use crate::resolve_anchor::AnchorAliasLimitsView;
use crate::resolve_canonical_scalar_key::{
    compose_profile1_canonical_scalar_keys, CanonicalKeyByte, CanonicalScalarKeyError,
    CanonicalScalarKeyErrorKind, CanonicalScalarKeyLimits, CanonicalScalarKeySource,
};
#[allow(unused_imports)]
use crate::resolve_canonical_scalar_key::{
    CanonicalKeyByteView, CanonicalScalarKeyErrorView, CanonicalScalarKeyLimitsView,
    CanonicalScalarKeySourceView,
};
#[allow(unused_imports)]
use crate::resolve_collection_tag::ResolvedCollectionView;
use crate::resolve_collection_tag::{ResolvedCollection, ResolvedCollectionTag};
#[allow(unused_imports)]
use crate::resolve_node_table::SemanticNodeTableLimitsView;
use crate::resolve_node_table::{SemanticNodeKind, SemanticNodeTableLimits};
use crate::resolve_scalar_table::SemanticScalarTableLimits;
#[allow(unused_imports)]
use crate::resolve_scalar_table::SemanticScalarTableLimitsView;
#[allow(unused_imports)]
use crate::resolve_topology::SemanticTopologyLimitsView;
use crate::resolve_topology::{SemanticMappingEdge, SemanticTopologyLimits};
use crate::token::CompletedTokenSource;
#[allow(unused_imports)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS: u64 = crate::cst::MAX_PROFILE1_CST_NODES;

pub const MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES: u64 = 1_048_576;

pub const MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES: u64 = 1_048_576;

pub const MAX_PROFILE1_MAPPING_SORT_ENTRIES: u64 = crate::cst::MAX_PROFILE1_CST_MAPPING_ENTRIES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalStructuralKeyLimits {
    max_records: u64,
    max_key_bytes: u64,
    max_total_key_bytes: u64,
    max_mapping_sort_entries: u64,
}

#[verifier::ext_equal]
pub struct CanonicalStructuralKeyLimitsView {
    pub max_records: u64,
    pub max_key_bytes: u64,
    pub max_total_key_bytes: u64,
    pub max_mapping_sort_entries: u64,
}

impl View for CanonicalStructuralKeyLimits {
    type V = CanonicalStructuralKeyLimitsView;

    closed spec fn view(&self) -> CanonicalStructuralKeyLimitsView {
        CanonicalStructuralKeyLimitsView {
            max_records: self.max_records,
            max_key_bytes: self.max_key_bytes,
            max_total_key_bytes: self.max_total_key_bytes,
            max_mapping_sort_entries: self.max_mapping_sort_entries,
        }
    }
}

impl CanonicalStructuralKeyLimits {
    pub fn new(
        max_records: u64,
        max_key_bytes: u64,
        max_total_key_bytes: u64,
        max_mapping_sort_entries: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (CanonicalStructuralKeyLimitsView {
                max_records,
                max_key_bytes,
                max_total_key_bytes,
                max_mapping_sort_entries,
            }),
    {
        Self { max_records, max_key_bytes, max_total_key_bytes, max_mapping_sort_entries }
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

    pub fn max_mapping_sort_entries(&self) -> (value: u64)
        ensures
            value == self@.max_mapping_sort_entries,
    {
        self.max_mapping_sort_entries
    }
}

pub fn canonical_structural_key_limits() -> (limits: CanonicalStructuralKeyLimits)
    ensures
        limits@ == canonical_structural_key_limits_spec(),
{
    CanonicalStructuralKeyLimits::new(
        MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
        MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
        MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
        MAX_PROFILE1_MAPPING_SORT_ENTRIES,
    )
}

pub open spec fn canonical_structural_key_limits_spec() -> CanonicalStructuralKeyLimitsView {
    CanonicalStructuralKeyLimitsView {
        max_records: MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
        max_key_bytes: MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
        max_total_key_bytes: MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
        max_mapping_sort_entries: MAX_PROFILE1_MAPPING_SORT_ENTRIES,
    }
}

pub open spec fn structural_key_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (limit: u64)
    ensures
        limit == structural_key_effective_limit_spec(requested, absolute),
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalStructuralKeyRecord {
    node_index: u64,
    byte_start: u64,
    bytes: Vec<CanonicalKeyByte>,
}

#[verifier::ext_equal]
pub struct CanonicalStructuralKeyRecordView {
    pub node_index: u64,
    pub byte_start: u64,
    pub bytes: Seq<CanonicalKeyByteView>,
}

impl View for CanonicalStructuralKeyRecord {
    type V = CanonicalStructuralKeyRecordView;

    closed spec fn view(&self) -> CanonicalStructuralKeyRecordView {
        CanonicalStructuralKeyRecordView {
            node_index: self.node_index,
            byte_start: self.byte_start,
            bytes: crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(self.bytes@),
        }
    }
}

impl CanonicalStructuralKeyRecord {
    fn new(node_index: u64, byte_start: u64, bytes: Vec<CanonicalKeyByte>) -> (record: Self)
        ensures
            record@ == (CanonicalStructuralKeyRecordView {
                node_index,
                byte_start,
                bytes: crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
            }),
    {
        Self { node_index, byte_start, bytes }
    }

    pub fn node_index(&self) -> (value: u64)
        ensures
            value == self@.node_index,
    {
        self.node_index
    }

    pub fn byte_start(&self) -> (value: u64)
        ensures
            value == self@.byte_start,
    {
        self.byte_start
    }

    pub fn bytes(&self) -> (bytes: &[CanonicalKeyByte])
        ensures
            crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@)
                == self@.bytes,
    {
        self.bytes.as_slice()
    }
}

pub open spec fn canonical_structural_key_record_views_spec(
    values: Seq<CanonicalStructuralKeyRecord>,
) -> Seq<CanonicalStructuralKeyRecordView> {
    Seq::new(values.len(), |index: int| values[index]@)
}

proof fn lemma_canonical_structural_key_record_views_push(
    values: Seq<CanonicalStructuralKeyRecord>,
    value: CanonicalStructuralKeyRecord,
)
    ensures
        canonical_structural_key_record_views_spec(values.push(value))
            == canonical_structural_key_record_views_spec(values).push(value@),
{
    reveal(canonical_structural_key_record_views_spec);
    assert(canonical_structural_key_record_views_spec(values.push(value))
        =~= canonical_structural_key_record_views_spec(values).push(value@));
}

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalStructuralKeySource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_node_count: u64,
    total_key_bytes: u64,
    scalar_keys: CanonicalScalarKeySource,
    records: Vec<CanonicalStructuralKeyRecord>,
}

#[verifier::ext_equal]
pub struct CanonicalStructuralKeySourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_node_count: u64,
    pub total_key_bytes: u64,
    pub scalar_keys: CanonicalScalarKeySourceView,
    pub records: Seq<CanonicalStructuralKeyRecordView>,
}

impl View for CanonicalStructuralKeySource {
    type V = CanonicalStructuralKeySourceView;

    closed spec fn view(&self) -> CanonicalStructuralKeySourceView {
        CanonicalStructuralKeySourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_node_count: self.input_node_count,
            total_key_bytes: self.total_key_bytes,
            scalar_keys: self.scalar_keys@,
            records: canonical_structural_key_record_views_spec(self.records@),
        }
    }
}

impl CanonicalStructuralKeySource {
    fn new(
        scalar_keys: CanonicalScalarKeySource,
        records: Vec<CanonicalStructuralKeyRecord>,
        total_key_bytes: u64,
    ) -> (source: Self)
        ensures
            source@ == (CanonicalStructuralKeySourceView {
                profile_version: scalar_keys@.graph.profile_version,
                transformation_version: CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION,
                source_len_bytes: scalar_keys@.graph.source_len_bytes,
                input_node_count: scalar_keys@.input_node_count,
                total_key_bytes,
                scalar_keys: scalar_keys@,
                records: canonical_structural_key_record_views_spec(records@),
            }),
    {
        let profile_version = scalar_keys.graph().profile_version();
        let source_len_bytes = scalar_keys.graph().source_len_bytes();
        let input_node_count = scalar_keys.input_node_count();
        Self {
            profile_version,
            transformation_version: CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION,
            source_len_bytes,
            input_node_count,
            total_key_bytes,
            scalar_keys,
            records,
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

    pub fn total_key_bytes(&self) -> (value: u64)
        ensures
            value == self@.total_key_bytes,
    {
        self.total_key_bytes
    }

    pub fn scalar_keys(&self) -> (source: &CanonicalScalarKeySource)
        ensures
            source@ == self@.scalar_keys,
    {
        &self.scalar_keys
    }

    pub fn records(&self) -> (records: &[CanonicalStructuralKeyRecord])
        ensures
            canonical_structural_key_record_views_spec(records@) == self@.records,
    {
        self.records.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CanonicalStructuralKeyErrorKind {
    ScalarKey(CanonicalScalarKeyErrorKind),
    RecordLimitExceeded,
    KeyByteLimitExceeded,
    TotalKeyByteLimitExceeded,
    MappingSortLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalStructuralKeyError {
    kind: CanonicalStructuralKeyErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CanonicalStructuralKeyErrorView {
    pub kind: CanonicalStructuralKeyErrorKind,
    pub byte_offset: u64,
}

impl View for CanonicalStructuralKeyError {
    type V = CanonicalStructuralKeyErrorView;

    closed spec fn view(&self) -> CanonicalStructuralKeyErrorView {
        CanonicalStructuralKeyErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CanonicalStructuralKeyError {
    fn at(kind: CanonicalStructuralKeyErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (CanonicalStructuralKeyErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (value: CanonicalStructuralKeyErrorKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn byte_offset(&self) -> (value: u64)
        ensures
            value == self@.byte_offset,
    {
        self.byte_offset
    }
}

pub open spec fn map_scalar_key_error_spec(
    error: CanonicalScalarKeyErrorView,
) -> CanonicalStructuralKeyErrorView {
    CanonicalStructuralKeyErrorView {
        kind: CanonicalStructuralKeyErrorKind::ScalarKey(error.kind),
        byte_offset: error.byte_offset,
    }
}

fn map_scalar_key_error(error: CanonicalScalarKeyError) -> (mapped: CanonicalStructuralKeyError)
    ensures
        mapped@ == map_scalar_key_error_spec(error@),
{
    CanonicalStructuralKeyError::at(
        CanonicalStructuralKeyErrorKind::ScalarKey(error.kind()),
        error.byte_offset(),
    )
}

struct StructuralKeyBuild {
    bytes: Vec<CanonicalKeyByte>,
    total_before: u64,
    node_byte: u64,
}

#[verifier::ext_equal]
pub struct StructuralKeyBuildView {
    pub bytes: Seq<CanonicalKeyByteView>,
    pub total_before: u64,
    pub node_byte: u64,
}

impl View for StructuralKeyBuild {
    type V = StructuralKeyBuildView;

    closed spec fn view(&self) -> StructuralKeyBuildView {
        StructuralKeyBuildView {
            bytes: crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(self.bytes@),
            total_before: self.total_before,
            node_byte: self.node_byte,
        }
    }
}

impl StructuralKeyBuild {
    fn empty(total_before: u64, node_byte: u64) -> (build: Self)
        ensures
            build@ == (StructuralKeyBuildView { bytes: Seq::empty(), total_before, node_byte }),
    {
        let build = Self { bytes: Vec::new(), total_before, node_byte };
        proof {
            reveal(crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec);
        }
        build
    }

    fn push_with_source(
        &mut self,
        value: u8,
        source_byte_offset: u64,
        limits: CanonicalStructuralKeyLimits,
    ) -> (result: Result<(), CanonicalStructuralKeyError>)
        ensures
            final(self)@.node_byte == old(self)@.node_byte,
            final(self)@.total_before == old(self)@.total_before,
            structural_key_push_with_source_spec(old(self)@, value, source_byte_offset, limits@)
                == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        let key_limit = effective_limit(
            limits.max_key_bytes(),
            MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
        );
        if self.bytes.len() as u64 >= key_limit {
            return Err(
                CanonicalStructuralKeyError::at(
                    CanonicalStructuralKeyErrorKind::KeyByteLimitExceeded,
                    self.node_byte,
                ),
            );
        }
        let total_limit = effective_limit(
            limits.max_total_key_bytes(),
            MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
        );
        if self.total_before >= total_limit || self.bytes.len() as u64 >= total_limit
            - self.total_before {
            return Err(
                CanonicalStructuralKeyError::at(
                    CanonicalStructuralKeyErrorKind::TotalKeyByteLimitExceeded,
                    self.node_byte,
                ),
            );
        }
        let byte = CanonicalKeyByte::new(value, source_byte_offset);
        proof {
            crate::resolve_canonical_scalar_key::lemma_canonical_key_byte_views_push(
                self.bytes@,
                byte,
            );
        }
        self.bytes.push(byte);
        proof {
            reveal(structural_key_push_with_source_spec);
        }
        Ok(())
    }

    fn push(&mut self, value: u8, limits: CanonicalStructuralKeyLimits) -> (result: Result<
        (),
        CanonicalStructuralKeyError,
    >)
        ensures
            final(self)@.node_byte == old(self)@.node_byte,
            final(self)@.total_before == old(self)@.total_before,
            structural_key_push_spec(old(self)@, value, limits@) == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        self.push_with_source(value, self.node_byte, limits)
    }

    fn append_u32(&mut self, value: u32, limits: CanonicalStructuralKeyLimits) -> (result: Result<
        (),
        CanonicalStructuralKeyError,
    >)
        ensures
            final(self)@.node_byte == old(self)@.node_byte,
            final(self)@.total_before == old(self)@.total_before,
            structural_append_u32_spec(old(self)@, value, limits@) == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        self.push((value >> 24) as u8, limits)?;
        self.push((value >> 16) as u8, limits)?;
        self.push((value >> 8) as u8, limits)?;
        self.push(value as u8, limits)
    }

    fn append_u32_with_source(
        &mut self,
        value: u32,
        source_byte_offset: u64,
        limits: CanonicalStructuralKeyLimits,
    ) -> (result: Result<(), CanonicalStructuralKeyError>)
        ensures
            final(self)@.node_byte == old(self)@.node_byte,
            final(self)@.total_before == old(self)@.total_before,
            structural_append_u32_with_source_spec(old(self)@, value, source_byte_offset, limits@)
                == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        self.push_with_source((value >> 24) as u8, source_byte_offset, limits)?;
        self.push_with_source((value >> 16) as u8, source_byte_offset, limits)?;
        self.push_with_source((value >> 8) as u8, source_byte_offset, limits)?;
        self.push_with_source(value as u8, source_byte_offset, limits)
    }

    fn append_u64(&mut self, value: u64, limits: CanonicalStructuralKeyLimits) -> (result: Result<
        (),
        CanonicalStructuralKeyError,
    >)
        ensures
            final(self)@.node_byte == old(self)@.node_byte,
            final(self)@.total_before == old(self)@.total_before,
            structural_append_u64_spec(old(self)@, value, limits@) == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        self.append_u32((value >> 32) as u32, limits)?;
        self.append_u32(value as u32, limits)
    }

    fn append_bytes(
        &mut self,
        Ghost(input): Ghost<StructuralKeyBuildView>,
        bytes: &[CanonicalKeyByte],
        limits: CanonicalStructuralKeyLimits,
    ) -> (result: Result<(), CanonicalStructuralKeyError>)
        requires
            old(self)@ == input,
        ensures
            final(self)@.node_byte == input.node_byte,
            final(self)@.total_before == input.total_before,
            structural_append_bytes_tail_spec(
                input,
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
                0,
                bytes@.len() as nat,
                limits@,
            ) == match result {
                Ok(()) => Ok(final(self)@),
                Err(error) => Err(error@),
            },
    {
        let ghost expected = structural_append_bytes_tail_spec(
            input,
            crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
            0,
            bytes@.len() as nat,
            limits@,
        );
        let mut index = 0usize;
        while index < bytes.len()
            invariant
                index <= bytes.len(),
                self@.node_byte == input.node_byte,
                self@.total_before == input.total_before,
                expected == structural_append_bytes_tail_spec(
                    input,
                    crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
                    0,
                    bytes@.len() as nat,
                    limits@,
                ),
                expected == structural_append_bytes_tail_spec(
                    self@,
                    crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
                    index as nat,
                    (bytes.len() - index) as nat,
                    limits@,
                ),
            decreases bytes.len() - index,
        {
            let step = self.push_with_source(
                bytes[index].value(),
                bytes[index].source_byte_offset(),
                limits,
            );
            match step {
                Err(error) => {
                    proof {
                        reveal(structural_append_bytes_tail_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
                Ok(()) => {},
            }
            proof {
                reveal(structural_append_bytes_tail_spec);
            }
            index += 1;
        }
        proof {
            reveal(structural_append_bytes_tail_spec);
        }
        Ok(())
    }
}

pub open spec fn structural_key_push_with_source_spec(
    build: StructuralKeyBuildView,
    value: u8,
    source_byte_offset: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    let key_limit = structural_key_effective_limit_spec(
        limits.max_key_bytes,
        MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_BYTES,
    );
    let total_limit = structural_key_effective_limit_spec(
        limits.max_total_key_bytes,
        MAX_PROFILE1_TOTAL_CANONICAL_STRUCTURAL_KEY_BYTES,
    );
    if build.bytes.len() >= key_limit {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::KeyByteLimitExceeded,
                byte_offset: build.node_byte,
            },
        )
    } else if build.total_before >= total_limit || build.bytes.len() >= total_limit
        - build.total_before {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::TotalKeyByteLimitExceeded,
                byte_offset: build.node_byte,
            },
        )
    } else {
        Ok(
            StructuralKeyBuildView {
                bytes: build.bytes.push(CanonicalKeyByteView { value, source_byte_offset }),
                total_before: build.total_before,
                node_byte: build.node_byte,
            },
        )
    }
}

pub open spec fn structural_key_push_spec(
    build: StructuralKeyBuildView,
    value: u8,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    structural_key_push_with_source_spec(build, value, build.node_byte, limits)
}

pub open spec fn structural_append_u32_spec(
    build: StructuralKeyBuildView,
    value: u32,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    match structural_key_push_spec(build, (value >> 24) as u8, limits) {
        Err(error) => Err(error),
        Ok(a) => match structural_key_push_spec(a, (value >> 16) as u8, limits) {
            Err(error) => Err(error),
            Ok(b) => match structural_key_push_spec(b, (value >> 8) as u8, limits) {
                Err(error) => Err(error),
                Ok(c) => structural_key_push_spec(c, value as u8, limits),
            },
        },
    }
}

pub open spec fn structural_append_u32_with_source_spec(
    build: StructuralKeyBuildView,
    value: u32,
    source_byte_offset: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    match structural_key_push_with_source_spec(
        build,
        (value >> 24) as u8,
        source_byte_offset,
        limits,
    ) {
        Err(error) => Err(error),
        Ok(a) => match structural_key_push_with_source_spec(
            a,
            (value >> 16) as u8,
            source_byte_offset,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(b) => match structural_key_push_with_source_spec(
                b,
                (value >> 8) as u8,
                source_byte_offset,
                limits,
            ) {
                Err(error) => Err(error),
                Ok(c) => structural_key_push_with_source_spec(
                    c,
                    value as u8,
                    source_byte_offset,
                    limits,
                ),
            },
        },
    }
}

pub open spec fn structural_append_u64_spec(
    build: StructuralKeyBuildView,
    value: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    match structural_append_u32_spec(build, (value >> 32) as u32, limits) {
        Err(error) => Err(error),
        Ok(next) => structural_append_u32_spec(next, value as u32, limits),
    }
}

pub closed spec fn structural_append_bytes_tail_spec(
    build: StructuralKeyBuildView,
    bytes: Seq<CanonicalKeyByteView>,
    index: nat,
    fuel: nat,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if index >= bytes.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: build.node_byte,
            },
        )
    } else {
        match structural_key_push_with_source_spec(
            build,
            bytes[index as int].value,
            bytes[index as int].source_byte_offset,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(next) => structural_append_bytes_tail_spec(
                next,
                bytes,
                (index + 1) as nat,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

pub closed spec fn compare_byte_views_tail_spec(
    left: Seq<CanonicalKeyByteView>,
    right: Seq<CanonicalKeyByteView>,
    index: nat,
    fuel: nat,
) -> i8
    decreases fuel,
{
    if index >= left.len() || index >= right.len() {
        if left.len() < right.len() {
            -1i8
        } else if left.len() > right.len() {
            1i8
        } else {
            0i8
        }
    } else if fuel == 0 {
        0i8
    } else if left[index as int].value < right[index as int].value {
        -1i8
    } else if left[index as int].value > right[index as int].value {
        1i8
    } else {
        compare_byte_views_tail_spec(left, right, (index + 1) as nat, (fuel - 1) as nat)
    }
}

fn compare_byte_slices(left: &[CanonicalKeyByte], right: &[CanonicalKeyByte]) -> (order: i8)
    ensures
        order == compare_byte_views_tail_spec(
            crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(left@),
            crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(right@),
            0,
            if left@.len() < right@.len() {
                left@.len() as nat
            } else {
                right@.len() as nat
            },
        ),
{
    let ghost expected = compare_byte_views_tail_spec(
        crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(left@),
        crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(right@),
        0,
        if left@.len() < right@.len() {
            left@.len() as nat
        } else {
            right@.len() as nat
        },
    );
    let mut index = 0usize;
    while index < left.len() && index < right.len()
        invariant
            index <= left.len(),
            index <= right.len(),
            expected == compare_byte_views_tail_spec(
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(left@),
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(right@),
                0,
                if left@.len() < right@.len() {
                    left@.len() as nat
                } else {
                    right@.len() as nat
                },
            ),
            expected == compare_byte_views_tail_spec(
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(left@),
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(right@),
                index as nat,
                if left.len() - index < right.len() - index {
                    (left.len() - index) as nat
                } else {
                    (right.len() - index) as nat
                },
            ),
        decreases left.len() - index,
    {
        if left[index].value() < right[index].value() {
            proof {
                reveal(compare_byte_views_tail_spec);
                reveal(crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec);
                assert(expected == -1i8);
            }
            return -1;
        }
        if left[index].value() > right[index].value() {
            proof {
                reveal(compare_byte_views_tail_spec);
                reveal(crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec);
                assert(expected == 1i8);
            }
            return 1;
        }
        proof {
            reveal(compare_byte_views_tail_spec);
            reveal(crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec);
        }
        index += 1;
    }
    proof {
        reveal(compare_byte_views_tail_spec);
        if left.len() < right.len() {
            assert(expected == -1i8);
        } else if left.len() > right.len() {
            assert(expected == 1i8);
        } else {
            assert(expected == 0i8);
        }
    }
    if left.len() < right.len() {
        -1
    } else if left.len() > right.len() {
        1
    } else {
        0
    }
}

pub open spec fn compare_mapping_edge_views_spec(
    left: crate::resolve_topology::SemanticMappingEdgeView,
    right: crate::resolve_topology::SemanticMappingEdgeView,
    records: Seq<CanonicalStructuralKeyRecordView>,
) -> Result<i8, CanonicalStructuralKeyErrorView> {
    if left.key_node_index >= records.len() || right.key_node_index >= records.len()
        || left.value_node_index >= records.len() || right.value_node_index >= records.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: 0,
            },
        )
    } else {
        let left_key = records[left.key_node_index as int].bytes;
        let right_key = records[right.key_node_index as int].bytes;
        let key_order = compare_byte_views_tail_spec(
            left_key,
            right_key,
            0,
            if left_key.len() < right_key.len() {
                left_key.len() as nat
            } else {
                right_key.len() as nat
            },
        );
        if key_order != 0 {
            Ok(key_order)
        } else {
            let left_value = records[left.value_node_index as int].bytes;
            let right_value = records[right.value_node_index as int].bytes;
            Ok(
                compare_byte_views_tail_spec(
                    left_value,
                    right_value,
                    0,
                    if left_value.len() < right_value.len() {
                        left_value.len() as nat
                    } else {
                        right_value.len() as nat
                    },
                ),
            )
        }
    }
}

fn compare_mapping_edges(
    left: &SemanticMappingEdge,
    right: &SemanticMappingEdge,
    records: &[CanonicalStructuralKeyRecord],
) -> (result: Result<i8, CanonicalStructuralKeyError>)
    ensures
        compare_mapping_edge_views_spec(
            left@,
            right@,
            canonical_structural_key_record_views_spec(records@),
        ) == match result {
            Ok(order) => Ok(order),
            Err(error) => Err(error@),
        },
{
    let left_key_u64 = left.key_node_index();
    let right_key_u64 = right.key_node_index();
    let left_value_u64 = left.value_node_index();
    let right_value_u64 = right.value_node_index();
    if left_key_u64 >= records.len() as u64 || right_key_u64 >= records.len() as u64
        || left_value_u64 >= records.len() as u64 || right_value_u64 >= records.len() as u64 {
        proof {
            reveal(compare_mapping_edge_views_spec);
            reveal(canonical_structural_key_record_views_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                0,
            ),
        );
    }
    let left_key = left_key_u64 as usize;
    let right_key = right_key_u64 as usize;
    let left_value = left_value_u64 as usize;
    let right_value = right_value_u64 as usize;
    let key_order = compare_byte_slices(records[left_key].bytes(), records[right_key].bytes());
    if key_order != 0 {
        proof {
            reveal(compare_mapping_edge_views_spec);
            reveal(canonical_structural_key_record_views_spec);
        }
        return Ok(key_order);
    }
    let value_order = compare_byte_slices(
        records[left_value].bytes(),
        records[right_value].bytes(),
    );
    proof {
        reveal(compare_mapping_edge_views_spec);
        reveal(canonical_structural_key_record_views_spec);
    }
    Ok(value_order)
}

pub closed spec fn merge_mapping_items_tail_spec(
    left: Seq<u64>,
    right: Seq<u64>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    left_index: nat,
    right_index: nat,
    fuel: nat,
    built: Seq<u64>,
    node_byte: u64,
) -> Result<Seq<u64>, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if left_index >= left.len() && right_index >= right.len() {
        Ok(built)
    } else if fuel == 0 {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else if left_index >= left.len() {
        merge_mapping_items_tail_spec(
            left,
            right,
            edges,
            records,
            left_index,
            (right_index + 1) as nat,
            (fuel - 1) as nat,
            built.push(right[right_index as int]),
            node_byte,
        )
    } else if right_index >= right.len() {
        merge_mapping_items_tail_spec(
            left,
            right,
            edges,
            records,
            (left_index + 1) as nat,
            right_index,
            (fuel - 1) as nat,
            built.push(left[left_index as int]),
            node_byte,
        )
    } else if left[left_index as int] >= edges.len() || right[right_index as int] >= edges.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        match compare_mapping_edge_views_spec(
            edges[left[left_index as int] as int],
            edges[right[right_index as int] as int],
            records,
        ) {
            Err(error) => Err(error),
            Ok(order) => if order <= 0 {
                merge_mapping_items_tail_spec(
                    left,
                    right,
                    edges,
                    records,
                    (left_index + 1) as nat,
                    right_index,
                    (fuel - 1) as nat,
                    built.push(left[left_index as int]),
                    node_byte,
                )
            } else {
                merge_mapping_items_tail_spec(
                    left,
                    right,
                    edges,
                    records,
                    left_index,
                    (right_index + 1) as nat,
                    (fuel - 1) as nat,
                    built.push(right[right_index as int]),
                    node_byte,
                )
            },
        }
    }
}

fn merge_mapping_items(
    left: &[u64],
    right: &[u64],
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    node_byte: u64,
) -> (result: Result<Vec<u64>, CanonicalStructuralKeyError>)
    ensures
        merge_mapping_items_tail_spec(
            left@,
            right@,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            0,
            0,
            (left@.len() + right@.len()) as nat,
            Seq::empty(),
            node_byte,
        ) == match result {
            Ok(merged) => Ok(merged@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(ref merged) => merged@.len() == left@.len() + right@.len(),
            Err(_) => true,
        },
{
    let ghost expected = merge_mapping_items_tail_spec(
        left@,
        right@,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        0,
        0,
        (left@.len() + right@.len()) as nat,
        Seq::empty(),
        node_byte,
    );
    let mut merged = Vec::new();
    let mut left_index = 0usize;
    let mut right_index = 0usize;
    while left_index < left.len() || right_index < right.len()
        invariant
            left_index <= left.len(),
            right_index <= right.len(),
            merged.len() == left_index + right_index,
            expected == merge_mapping_items_tail_spec(
                left@,
                right@,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                0,
                0,
                (left@.len() + right@.len()) as nat,
                Seq::empty(),
                node_byte,
            ),
            expected == merge_mapping_items_tail_spec(
                left@,
                right@,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                left_index as nat,
                right_index as nat,
                (left.len() + right.len() - left_index - right_index) as nat,
                merged@,
                node_byte,
            ),
        decreases left.len() + right.len() - left_index - right_index,
    {
        if left_index >= left.len() {
            proof {
                reveal(merge_mapping_items_tail_spec);
                assert(expected == merge_mapping_items_tail_spec(
                    left@,
                    right@,
                    crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                    canonical_structural_key_record_views_spec(records@),
                    left_index as nat,
                    (right_index + 1) as nat,
                    (left.len() + right.len() - left_index - right_index - 1) as nat,
                    merged@.push(right@[right_index as int]),
                    node_byte,
                ));
            }
            merged.push(right[right_index]);
            right_index += 1;
            continue;
        }
        if right_index >= right.len() {
            proof {
                reveal(merge_mapping_items_tail_spec);
                assert(expected == merge_mapping_items_tail_spec(
                    left@,
                    right@,
                    crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                    canonical_structural_key_record_views_spec(records@),
                    (left_index + 1) as nat,
                    right_index as nat,
                    (left.len() + right.len() - left_index - right_index - 1) as nat,
                    merged@.push(left@[left_index as int]),
                    node_byte,
                ));
            }
            merged.push(left[left_index]);
            left_index += 1;
            continue;
        }
        let left_edge_u64 = left[left_index];
        let right_edge_u64 = right[right_index];
        if left_edge_u64 >= edges.len() as u64 || right_edge_u64 >= edges.len() as u64 {
            proof {
                reveal(merge_mapping_items_tail_spec);
                reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                assert(expected == Err(
                    CanonicalStructuralKeyErrorView {
                        kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node_byte,
                    },
                ));
            }
            return Err(
                CanonicalStructuralKeyError::at(
                    CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    node_byte,
                ),
            );
        }
        let left_edge = left_edge_u64 as usize;
        let right_edge = right_edge_u64 as usize;
        proof {
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            assert(crate::resolve_topology::semantic_mapping_edge_views_spec(
                edges@,
            )[left_edge_u64 as int] == edges@[left_edge as int]@);
            assert(crate::resolve_topology::semantic_mapping_edge_views_spec(
                edges@,
            )[right_edge_u64 as int] == edges@[right_edge as int]@);
        }
        let compared = compare_mapping_edges(&edges[left_edge], &edges[right_edge], records);
        match compared {
            Err(error) => {
                proof {
                    reveal(merge_mapping_items_tail_spec);
                    reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(order) => {
                if order <= 0 {
                    proof {
                        reveal(merge_mapping_items_tail_spec);
                        reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                        assert(expected == merge_mapping_items_tail_spec(
                            left@,
                            right@,
                            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                            canonical_structural_key_record_views_spec(records@),
                            (left_index + 1) as nat,
                            right_index as nat,
                            (left.len() + right.len() - left_index - right_index - 1) as nat,
                            merged@.push(left@[left_index as int]),
                            node_byte,
                        ));
                    }
                    merged.push(left[left_index]);
                    left_index += 1;
                } else {
                    proof {
                        reveal(merge_mapping_items_tail_spec);
                        reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                        assert(expected == merge_mapping_items_tail_spec(
                            left@,
                            right@,
                            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                            canonical_structural_key_record_views_spec(records@),
                            left_index as nat,
                            (right_index + 1) as nat,
                            (left.len() + right.len() - left_index - right_index - 1) as nat,
                            merged@.push(right@[right_index as int]),
                            node_byte,
                        ));
                    }
                    merged.push(right[right_index]);
                    right_index += 1;
                }
            },
        }
    }
    proof {
        reveal(merge_mapping_items_tail_spec);
        assert(expected == Ok(merged@));
        assert(merged@.len() == left@.len() + right@.len());
    }
    Ok(merged)
}

pub closed spec fn sort_mapping_pass_tail_spec(
    items: Seq<u64>,
    width: nat,
    start: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    fuel: nat,
    built: Seq<u64>,
    node_byte: u64,
) -> Result<Seq<u64>, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if start >= items.len() {
        Ok(built)
    } else if fuel == 0 || width == 0 {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        let middle = if width >= items.len() - start {
            items.len()
        } else {
            start + width
        };
        let end = if width >= items.len() - middle {
            items.len()
        } else {
            middle + width
        };
        match merge_mapping_items_tail_spec(
            items.subrange(start as int, middle as int),
            items.subrange(middle as int, end as int),
            edges,
            records,
            0,
            0,
            (end - start) as nat,
            Seq::empty(),
            node_byte,
        ) {
            Err(error) => Err(error),
            Ok(run) => sort_mapping_pass_tail_spec(
                items,
                width,
                end,
                edges,
                records,
                (fuel - 1) as nat,
                built + run,
                node_byte,
            ),
        }
    }
}

pub closed spec fn sort_mapping_iter_tail_spec(
    items: Seq<u64>,
    width: nat,
    fuel: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
) -> Result<Seq<u64>, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if items.len() <= 1 || width >= items.len() {
        Ok(items)
    } else if fuel == 0 || width == 0 {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        match sort_mapping_pass_tail_spec(
            items,
            width,
            0,
            edges,
            records,
            items.len() as nat,
            Seq::empty(),
            node_byte,
        ) {
            Err(error) => Err(error),
            Ok(next) => {
                let next_width = if width >= items.len() - width {
                    items.len()
                } else {
                    width + width
                };
                sort_mapping_iter_tail_spec(
                    next,
                    next_width,
                    (fuel - 1) as nat,
                    edges,
                    records,
                    node_byte,
                )
            },
        }
    }
}

pub open spec fn sort_mapping_items_spec(
    items: Seq<u64>,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    node_byte: u64,
) -> Result<Seq<u64>, CanonicalStructuralKeyErrorView> {
    sort_mapping_iter_tail_spec(items, 1, items.len() as nat, edges, records, node_byte)
}

fn append_u64_slice(output: &mut Vec<u64>, values: &[u64])
    ensures
        final(output)@ == old(output)@ + values@,
{
    let mut index = 0usize;
    while index < values.len()
        invariant
            index <= values.len(),
            output@ == old(output)@ + values@.subrange(0, index as int),
        decreases values.len() - index,
    {
        output.push(values[index]);
        index += 1;
    }
}

#[verifier::rlimit(60)]
fn sort_mapping_pass(
    items: &[u64],
    width: usize,
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    node_byte: u64,
) -> (result: Result<Vec<u64>, CanonicalStructuralKeyError>)
    ensures
        sort_mapping_pass_tail_spec(
            items@,
            width as nat,
            0,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            items@.len() as nat,
            Seq::empty(),
            node_byte,
        ) == match result {
            Ok(next) => Ok(next@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(ref next) => next@.len() == items@.len(),
            Err(_) => true,
        },
{
    if items.is_empty() {
        proof {
            reveal(sort_mapping_pass_tail_spec);
        }
        return Ok(Vec::new());
    }
    if width == 0 {
        proof {
            reveal(sort_mapping_pass_tail_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                node_byte,
            ),
        );
    }
    let ghost expected = sort_mapping_pass_tail_spec(
        items@,
        width as nat,
        0,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        items@.len() as nat,
        Seq::empty(),
        node_byte,
    );
    let mut output = Vec::new();
    let mut start = 0usize;
    let mut _fuel = items.len();
    while start < items.len()
        invariant
            start <= items.len(),
            width > 0,
            _fuel + start >= items.len(),
            output.len() == start,
            expected == sort_mapping_pass_tail_spec(
                items@,
                width as nat,
                0,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                items@.len() as nat,
                Seq::empty(),
                node_byte,
            ),
            expected == sort_mapping_pass_tail_spec(
                items@,
                width as nat,
                start as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                _fuel as nat,
                output@,
                node_byte,
            ),
        decreases items.len() - start,
    {
        let middle = if width >= items.len() - start {
            items.len()
        } else {
            start + width
        };
        let end = if width >= items.len() - middle {
            items.len()
        } else {
            middle + width
        };
        let left = &items[start..middle];
        let right = &items[middle..end];
        proof {
            assert(_fuel > 0);
            assert(left@ == items@.subrange(start as int, middle as int));
            assert(right@ == items@.subrange(middle as int, end as int));
        }
        let run = match merge_mapping_items(left, right, edges, records, node_byte) {
            Err(error) => {
                proof {
                    reveal(sort_mapping_pass_tail_spec);
                    assert(expected == Err(error@));
                    assert(sort_mapping_pass_tail_spec(
                        items@,
                        width as nat,
                        0,
                        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                        canonical_structural_key_record_views_spec(records@),
                        items@.len() as nat,
                        Seq::empty(),
                        node_byte,
                    ) == Err(error@));
                }
                return Err(error);
            },
            Ok(run) => run,
        };
        append_u64_slice(&mut output, run.as_slice());
        proof {
            reveal(sort_mapping_pass_tail_spec);
        }
        start = end;
        _fuel -= 1;
    }
    proof {
        reveal(sort_mapping_pass_tail_spec);
    }
    Ok(output)
}

#[verifier::rlimit(80)]
fn sort_mapping_items(
    mut items: Vec<u64>,
    Ghost(input): Ghost<Seq<u64>>,
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    node_byte: u64,
) -> (result: Result<Vec<u64>, CanonicalStructuralKeyError>)
    requires
        items@ == input,
    ensures
        sort_mapping_items_spec(
            input,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            node_byte,
        ) == match result {
            Ok(sorted) => Ok(sorted@),
            Err(error) => Err(error@),
        },
{
    if items.len() <= 1 {
        proof {
            reveal(sort_mapping_items_spec);
            reveal(sort_mapping_iter_tail_spec);
        }
        return Ok(items);
    }
    let ghost expected = sort_mapping_items_spec(
        input,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        node_byte,
    );
    let original_len = items.len();
    let mut width = 1usize;
    let mut _fuel = original_len;
    while width < items.len()
        invariant
            items.len() == original_len,
            1 <= width <= original_len,
            _fuel + width >= original_len + 1,
            expected == sort_mapping_items_spec(
                input,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                node_byte,
            ),
            expected == sort_mapping_iter_tail_spec(
                items@,
                width as nat,
                _fuel as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                node_byte,
            ),
        decreases original_len - width,
    {
        let next = match sort_mapping_pass(items.as_slice(), width, edges, records, node_byte) {
            Err(error) => {
                proof {
                    assert(_fuel > 0);
                    reveal(sort_mapping_iter_tail_spec);
                    assert(expected == Err(error@));
                    assert(sort_mapping_items_spec(
                        input,
                        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                        canonical_structural_key_record_views_spec(records@),
                        node_byte,
                    ) == Err(error@));
                }
                return Err(error);
            },
            Ok(next) => next,
        };
        let next_width = if width >= items.len() - width {
            items.len()
        } else {
            width + width
        };
        proof {
            reveal(sort_mapping_iter_tail_spec);
        }
        items = next;
        width = next_width;
        _fuel -= 1;
    }
    proof {
        reveal(sort_mapping_iter_tail_spec);
    }
    Ok(items)
}

pub closed spec fn append_collection_tag_content_tail_spec(
    build: StructuralKeyBuildView,
    content: Seq<crate::resolve_tag::ResolvedTagCodePointView>,
    index: nat,
    fuel: nat,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if index >= content.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: build.node_byte,
            },
        )
    } else {
        match structural_append_u32_with_source_spec(
            build,
            content[index as int].code_point,
            content[index as int].byte_start,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(next) => append_collection_tag_content_tail_spec(
                next,
                content,
                (index + 1) as nat,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

fn append_collection_tag_content(
    build: &mut StructuralKeyBuild,
    Ghost(input): Ghost<StructuralKeyBuildView>,
    content: &[crate::resolve_tag::ResolvedTagCodePoint],
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    requires
        old(build)@ == input,
    ensures
        final(build)@.node_byte == input.node_byte,
        final(build)@.total_before == input.total_before,
        append_collection_tag_content_tail_spec(
            input,
            crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
            0,
            content@.len() as nat,
            limits@,
        ) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = append_collection_tag_content_tail_spec(
        input,
        crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
        0,
        content@.len() as nat,
        limits@,
    );
    let mut index = 0usize;
    while index < content.len()
        invariant
            index <= content.len(),
            build@.node_byte == input.node_byte,
            build@.total_before == input.total_before,
            expected == append_collection_tag_content_tail_spec(
                input,
                crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
                0,
                content@.len() as nat,
                limits@,
            ),
            expected == append_collection_tag_content_tail_spec(
                build@,
                crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
                index as nat,
                (content.len() - index) as nat,
                limits@,
            ),
        decreases content.len() - index,
    {
        let step = build.append_u32_with_source(
            content[index].code_point(),
            content[index].byte_start(),
            limits,
        );
        match step {
            Err(error) => {
                proof {
                    reveal(append_collection_tag_content_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(append_collection_tag_content_tail_spec);
            reveal(crate::resolve_tag::resolved_tag_code_point_views_spec);
        }
        index += 1;
    }
    proof {
        reveal(append_collection_tag_content_tail_spec);
    }
    Ok(())
}

pub open spec fn append_collection_tag_spec(
    build: StructuralKeyBuildView,
    collection: ResolvedCollectionView,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    match collection.tag {
        ResolvedCollectionTag::CoreSequence => structural_key_push_spec(build, 0x01, limits),
        ResolvedCollectionTag::CoreMapping => structural_key_push_spec(build, 0x02, limits),
        ResolvedCollectionTag::CustomGlobal | ResolvedCollectionTag::CustomLocal => {
            let marker = if collection.tag == ResolvedCollectionTag::CustomGlobal {
                0x03
            } else {
                0x04
            };
            match structural_key_push_spec(build, marker, limits) {
                Err(error) => Err(error),
                Ok(marked) => match collection.explicit_tag {
                    None => Err(
                        CanonicalStructuralKeyErrorView {
                            kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            byte_offset: marked.node_byte,
                        },
                    ),
                    Some(tag) => match structural_append_u64_spec(
                        marked,
                        tag.content.len() as u64,
                        limits,
                    ) {
                        Err(error) => Err(error),
                        Ok(length) => append_collection_tag_content_tail_spec(
                            length,
                            tag.content,
                            0,
                            tag.content.len() as nat,
                            limits,
                        ),
                    },
                },
            }
        },
    }
}

fn append_collection_tag(
    build: &mut StructuralKeyBuild,
    collection: &ResolvedCollection,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    ensures
        final(build)@.node_byte == old(build)@.node_byte,
        final(build)@.total_before == old(build)@.total_before,
        append_collection_tag_spec(old(build)@, collection@, limits@) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    match collection.tag() {
        ResolvedCollectionTag::CoreSequence => build.push(0x01, limits),
        ResolvedCollectionTag::CoreMapping => build.push(0x02, limits),
        ResolvedCollectionTag::CustomGlobal | ResolvedCollectionTag::CustomLocal => {
            if collection.tag() == ResolvedCollectionTag::CustomGlobal {
                build.push(0x03, limits)?;
            } else {
                build.push(0x04, limits)?;
            }
            let explicit = match collection.explicit_tag() {
                Some(tag) => tag,
                None => return Err(
                    CanonicalStructuralKeyError::at(
                        CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        build.node_byte,
                    ),
                ),
            };
            let content = explicit.content();
            build.append_u64(content.len() as u64, limits)?;
            let result = append_collection_tag_content(build, Ghost(build@), content, limits);
            proof {
                reveal(append_collection_tag_spec);
            }
            result
        },
    }
}

pub open spec fn begin_collection_spec(
    build: StructuralKeyBuildView,
    kind: u8,
    collection: ResolvedCollectionView,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    match structural_key_push_spec(build, 0x43, limits) {
        Err(error) => Err(error),
        Ok(a) => match structural_key_push_spec(a, 0x43, limits) {
            Err(error) => Err(error),
            Ok(b) => match structural_key_push_spec(b, 0x4b, limits) {
                Err(error) => Err(error),
                Ok(c) => match structural_key_push_spec(
                    c,
                    (CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION >> 8) as u8,
                    limits,
                ) {
                    Err(error) => Err(error),
                    Ok(d) => match structural_key_push_spec(
                        d,
                        CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION as u8,
                        limits,
                    ) {
                        Err(error) => Err(error),
                        Ok(e) => match structural_key_push_spec(e, kind, limits) {
                            Err(error) => Err(error),
                            Ok(f) => append_collection_tag_spec(f, collection, limits),
                        },
                    },
                },
            },
        },
    }
}

fn begin_collection(
    build: &mut StructuralKeyBuild,
    kind: u8,
    collection: &ResolvedCollection,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    ensures
        final(build)@.node_byte == old(build)@.node_byte,
        final(build)@.total_before == old(build)@.total_before,
        begin_collection_spec(old(build)@, kind, collection@, limits@) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    build.push(0x43, limits)?;
    build.push(0x43, limits)?;
    build.push(0x4b, limits)?;
    build.push((CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION >> 8) as u8, limits)?;
    build.push(CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION as u8, limits)?;
    build.push(kind, limits)?;
    append_collection_tag(build, collection, limits)
}

pub open spec fn encode_scalar_or_alias_spec(
    source: Seq<CanonicalKeyByteView>,
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView> {
    match structural_append_bytes_tail_spec(
        StructuralKeyBuildView { bytes: Seq::empty(), total_before, node_byte },
        source,
        0,
        source.len() as nat,
        limits,
    ) {
        Err(error) => Err(error),
        Ok(build) => Ok(build.bytes),
    }
}

fn encode_scalar_or_alias(
    source: &[CanonicalKeyByte],
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<Vec<CanonicalKeyByte>, CanonicalStructuralKeyError>)
    ensures
        encode_scalar_or_alias_spec(
            crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(source@),
            total_before,
            node_byte,
            limits@,
        ) == match result {
            Ok(bytes) => Ok(
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
            ),
            Err(error) => Err(error@),
        },
{
    let mut build = StructuralKeyBuild::empty(total_before, node_byte);
    build.append_bytes(Ghost(build@), source, limits)?;
    proof {
        reveal(encode_scalar_or_alias_spec);
    }
    Ok(build.bytes)
}

pub open spec fn append_length_delimited_record_spec(
    build: StructuralKeyBuildView,
    record: CanonicalStructuralKeyRecordView,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    match structural_append_u64_spec(build, record.bytes.len() as u64, limits) {
        Err(error) => Err(error),
        Ok(length) => structural_append_bytes_tail_spec(
            length,
            record.bytes,
            0,
            record.bytes.len() as nat,
            limits,
        ),
    }
}

fn append_length_delimited_record(
    build: &mut StructuralKeyBuild,
    Ghost(input): Ghost<StructuralKeyBuildView>,
    record: &CanonicalStructuralKeyRecord,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    requires
        old(build)@ == input,
    ensures
        final(build)@.node_byte == input.node_byte,
        final(build)@.total_before == input.total_before,
        append_length_delimited_record_spec(input, record@, limits@) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    build.append_u64(record.bytes().len() as u64, limits)?;
    let result = build.append_bytes(Ghost(build@), record.bytes(), limits);
    proof {
        reveal(append_length_delimited_record_spec);
    }
    result
}

pub closed spec fn encode_sequence_children_tail_spec(
    build: StructuralKeyBuildView,
    node_index: nat,
    edge_index: nat,
    edge_end: nat,
    edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    fuel: nat,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if edge_index >= edge_end {
        Ok(build)
    } else if fuel == 0 || edge_index >= edges.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: build.node_byte,
            },
        )
    } else {
        let child = edges[edge_index as int].child_node_index;
        if child >= node_index || child >= records.len() {
            Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    byte_offset: build.node_byte,
                },
            )
        } else {
            match append_length_delimited_record_spec(build, records[child as int], limits) {
                Err(error) => Err(error),
                Ok(next) => encode_sequence_children_tail_spec(
                    next,
                    node_index,
                    (edge_index + 1) as nat,
                    edge_end,
                    edges,
                    records,
                    (fuel - 1) as nat,
                    limits,
                ),
            }
        }
    }
}

pub open spec fn finish_structural_bytes_spec(
    result: Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView>,
) -> Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => Ok(build.bytes),
    }
}

pub open spec fn encode_sequence_spec(
    node_index: nat,
    collection: ResolvedCollectionView,
    edge_start: nat,
    edge_end: nat,
    edges: Seq<crate::resolve_topology::SemanticSequenceEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView> {
    if edge_start > edge_end || edge_end > edges.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        let initial = StructuralKeyBuildView { bytes: Seq::empty(), total_before, node_byte };
        match begin_collection_spec(initial, 0x31, collection, limits) {
            Err(error) => Err(error),
            Ok(header) => match structural_append_u64_spec(
                header,
                (edge_end - edge_start) as u64,
                limits,
            ) {
                Err(error) => Err(error),
                Ok(count) => finish_structural_bytes_spec(
                    encode_sequence_children_tail_spec(
                        count,
                        node_index,
                        edge_start,
                        edge_end,
                        edges,
                        records,
                        (edge_end - edge_start) as nat,
                        limits,
                    ),
                ),
            },
        }
    }
}

#[verifier::rlimit(50)]
#[allow(clippy::too_many_arguments)]
fn encode_sequence(
    node_index: usize,
    collection: &ResolvedCollection,
    edge_start: usize,
    edge_end: usize,
    edges: &[crate::resolve_topology::SemanticSequenceEdge],
    records: &[CanonicalStructuralKeyRecord],
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<Vec<CanonicalKeyByte>, CanonicalStructuralKeyError>)
    ensures
        encode_sequence_spec(
            node_index as nat,
            collection@,
            edge_start as nat,
            edge_end as nat,
            crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            total_before,
            node_byte,
            limits@,
        ) == match result {
            Ok(bytes) => Ok(
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
            ),
            Err(error) => Err(error@),
        },
{
    let ghost whole_expected = encode_sequence_spec(
        node_index as nat,
        collection@,
        edge_start as nat,
        edge_end as nat,
        crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        total_before,
        node_byte,
        limits@,
    );
    if edge_start > edge_end || edge_end > edges.len() {
        proof {
            reveal(encode_sequence_spec);
            assert(whole_expected == Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node_byte,
                },
            ));
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                node_byte,
            ),
        );
    }
    let mut build = StructuralKeyBuild::empty(total_before, node_byte);
    begin_collection(&mut build, 0x31, collection, limits)?;
    build.append_u64((edge_end - edge_start) as u64, limits)?;
    let ghost expected = encode_sequence_children_tail_spec(
        build@,
        node_index as nat,
        edge_start as nat,
        edge_end as nat,
        crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        (edge_end - edge_start) as nat,
        limits@,
    );
    proof {
        reveal(encode_sequence_spec);
        assert(whole_expected == finish_structural_bytes_spec(expected));
    }
    let mut edge_index = edge_start;
    while edge_index < edge_end
        invariant
            edge_start <= edge_index <= edge_end,
            edge_end <= edges.len(),
            build@.node_byte == node_byte,
            whole_expected == encode_sequence_spec(
                node_index as nat,
                collection@,
                edge_start as nat,
                edge_end as nat,
                crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                total_before,
                node_byte,
                limits@,
            ),
            whole_expected == finish_structural_bytes_spec(expected),
            expected == encode_sequence_children_tail_spec(
                build@,
                node_index as nat,
                edge_index as nat,
                edge_end as nat,
                crate::resolve_topology::semantic_sequence_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                (edge_end - edge_index) as nat,
                limits@,
            ),
        decreases edge_end - edge_index,
    {
        let child_index_u64 = edges[edge_index].child_node_index();
        if child_index_u64 >= node_index as u64 || child_index_u64 >= records.len() as u64 {
            proof {
                assert(edge_index < edge_end);
                assert((edge_end - edge_index) as nat > 0);
                assert(edge_index < edges.len());
                reveal(encode_sequence_children_tail_spec);
                reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
                assert(crate::resolve_topology::semantic_sequence_edge_views_spec(
                    edges@,
                )[edge_index as int].child_node_index == child_index_u64);
                reveal(canonical_structural_key_record_views_spec);
                assert(canonical_structural_key_record_views_spec(records@).len() == records.len());
                if child_index_u64 >= node_index as u64 {
                    assert(child_index_u64 as nat >= node_index as nat);
                } else {
                    assert(child_index_u64 >= records.len() as u64);
                    assert(child_index_u64 >= canonical_structural_key_record_views_spec(
                        records@,
                    ).len());
                }
                assert(expected == Err(
                    CanonicalStructuralKeyErrorView {
                        kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node_byte,
                    },
                ));
                assert(finish_structural_bytes_spec(expected) == Err(
                    CanonicalStructuralKeyErrorView {
                        kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node_byte,
                    },
                ));
            }
            return Err(
                CanonicalStructuralKeyError::at(
                    CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    node_byte,
                ),
            );
        }
        let child_index = child_index_u64 as usize;
        let ghost build_before_child = build@;
        let step = append_length_delimited_record(
            &mut build,
            Ghost(build_before_child),
            &records[child_index],
            limits,
        );
        match step {
            Err(error) => {
                proof {
                    reveal(encode_sequence_children_tail_spec);
                    reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
                    reveal(canonical_structural_key_record_views_spec);
                    assert(finish_structural_bytes_spec(expected) == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(encode_sequence_children_tail_spec);
            reveal(crate::resolve_topology::semantic_sequence_edge_views_spec);
            reveal(canonical_structural_key_record_views_spec);
        }
        edge_index += 1;
    }
    proof {
        reveal(encode_sequence_children_tail_spec);
        reveal(encode_sequence_spec);
        assert(finish_structural_bytes_spec(expected) == Ok(build@.bytes));
    }
    Ok(build.bytes)
}

pub open spec fn append_mapping_pair_spec(
    build: StructuralKeyBuildView,
    edge: crate::resolve_topology::SemanticMappingEdgeView,
    records: Seq<CanonicalStructuralKeyRecordView>,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView> {
    if edge.key_node_index >= records.len() || edge.value_node_index >= records.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: build.node_byte,
            },
        )
    } else {
        match append_length_delimited_record_spec(
            build,
            records[edge.key_node_index as int],
            limits,
        ) {
            Err(error) => Err(error),
            Ok(key) => append_length_delimited_record_spec(
                key,
                records[edge.value_node_index as int],
                limits,
            ),
        }
    }
}

fn append_mapping_pair(
    build: &mut StructuralKeyBuild,
    Ghost(input): Ghost<StructuralKeyBuildView>,
    edge: &SemanticMappingEdge,
    records: &[CanonicalStructuralKeyRecord],
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    requires
        old(build)@ == input,
    ensures
        final(build)@.node_byte == input.node_byte,
        final(build)@.total_before == input.total_before,
        append_mapping_pair_spec(
            input,
            edge@,
            canonical_structural_key_record_views_spec(records@),
            limits@,
        ) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let key_u64 = edge.key_node_index();
    let value_u64 = edge.value_node_index();
    if key_u64 >= records.len() as u64 || value_u64 >= records.len() as u64 {
        proof {
            reveal(append_mapping_pair_spec);
            reveal(canonical_structural_key_record_views_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                build.node_byte,
            ),
        );
    }
    let key_index = key_u64 as usize;
    let value_index = value_u64 as usize;
    append_length_delimited_record(build, Ghost(build@), &records[key_index], limits)?;
    let result = append_length_delimited_record(
        build,
        Ghost(build@),
        &records[value_index],
        limits,
    );
    proof {
        reveal(append_mapping_pair_spec);
        reveal(canonical_structural_key_record_views_spec);
    }
    result
}

pub open spec fn mapping_edge_index_range_spec(start: nat, end: nat) -> Seq<u64> {
    if start <= end {
        Seq::new((end - start) as nat, |index: int| (start + index) as u64)
    } else {
        Seq::empty()
    }
}

pub closed spec fn build_mapping_items_tail_spec(
    node_index: nat,
    edge_index: nat,
    edge_end: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    fuel: nat,
    built: Seq<u64>,
    node_byte: u64,
) -> Result<Seq<u64>, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if edge_index >= edge_end {
        Ok(built)
    } else if fuel == 0 || edge_index >= edges.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        let edge = edges[edge_index as int];
        if edge.key_node_index >= node_index || edge.value_node_index >= node_index
            || edge.key_node_index >= records.len() || edge.value_node_index >= records.len() {
            Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node_byte,
                },
            )
        } else {
            build_mapping_items_tail_spec(
                node_index,
                (edge_index + 1) as nat,
                edge_end,
                edges,
                records,
                (fuel - 1) as nat,
                built.push(edge_index as u64),
                node_byte,
            )
        }
    }
}

pub closed spec fn encode_mapping_pairs_tail_spec(
    build: StructuralKeyBuildView,
    sorted: Seq<u64>,
    sorted_index: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    fuel: nat,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralKeyBuildView, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if sorted_index >= sorted.len() {
        Ok(build)
    } else if fuel == 0 || sorted[sorted_index as int] >= edges.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: build.node_byte,
            },
        )
    } else {
        match append_mapping_pair_spec(
            build,
            edges[sorted[sorted_index as int] as int],
            records,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(next) => encode_mapping_pairs_tail_spec(
                next,
                sorted,
                (sorted_index + 1) as nat,
                edges,
                records,
                (fuel - 1) as nat,
                limits,
            ),
        }
    }
}

#[verifier::rlimit(50)]
fn append_sorted_mapping_pairs(
    build: &mut StructuralKeyBuild,
    Ghost(input): Ghost<StructuralKeyBuildView>,
    sorted: &[u64],
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    requires
        old(build)@ == input,
    ensures
        final(build)@.node_byte == input.node_byte,
        final(build)@.total_before == input.total_before,
        encode_mapping_pairs_tail_spec(
            input,
            sorted@,
            0,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            sorted@.len() as nat,
            limits@,
        ) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = encode_mapping_pairs_tail_spec(
        input,
        sorted@,
        0,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        sorted@.len() as nat,
        limits@,
    );
    let mut sorted_index = 0usize;
    while sorted_index < sorted.len()
        invariant
            sorted_index <= sorted.len(),
            build@.node_byte == input.node_byte,
            build@.total_before == input.total_before,
            expected == encode_mapping_pairs_tail_spec(
                input,
                sorted@,
                0,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                sorted@.len() as nat,
                limits@,
            ),
            expected == encode_mapping_pairs_tail_spec(
                build@,
                sorted@,
                sorted_index as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                (sorted.len() - sorted_index) as nat,
                limits@,
            ),
        decreases sorted.len() - sorted_index,
    {
        let item_u64 = sorted[sorted_index];
        if item_u64 >= edges.len() as u64 {
            proof {
                reveal(encode_mapping_pairs_tail_spec);
                assert(expected == Err(
                    CanonicalStructuralKeyErrorView {
                        kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        byte_offset: build@.node_byte,
                    },
                ));
            }
            return Err(
                CanonicalStructuralKeyError::at(
                    CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    build.node_byte,
                ),
            );
        }
        let item = item_u64 as usize;
        proof {
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            assert(crate::resolve_topology::semantic_mapping_edge_views_spec(
                edges@,
            )[item_u64 as int] == edges@[item as int]@);
        }
        let ghost build_before_pair = build@;
        let step = append_mapping_pair(
            build,
            Ghost(build_before_pair),
            &edges[item],
            records,
            limits,
        );
        match step {
            Err(error) => {
                proof {
                    reveal(encode_mapping_pairs_tail_spec);
                    reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(encode_mapping_pairs_tail_spec);
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
        }
        sorted_index += 1;
    }
    proof {
        reveal(encode_mapping_pairs_tail_spec);
    }
    Ok(())
}

pub open spec fn finish_sorted_mapping_spec(
    sorted_result: Result<Seq<u64>, CanonicalStructuralKeyErrorView>,
    collection: ResolvedCollectionView,
    entry_count: u64,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView> {
    match sorted_result {
        Err(error) => Err(error),
        Ok(sorted) => {
            let initial = StructuralKeyBuildView { bytes: Seq::empty(), total_before, node_byte };
            match begin_collection_spec(initial, 0x32, collection, limits) {
                Err(error) => Err(error),
                Ok(header) => match structural_append_u64_spec(header, entry_count, limits) {
                    Err(error) => Err(error),
                    Ok(count) => finish_structural_bytes_spec(
                        encode_mapping_pairs_tail_spec(
                            count,
                            sorted,
                            0,
                            edges,
                            records,
                            sorted.len() as nat,
                            limits,
                        ),
                    ),
                },
            }
        },
    }
}

pub open spec fn finish_mapping_items_spec(
    items_result: Result<Seq<u64>, CanonicalStructuralKeyErrorView>,
    collection: ResolvedCollectionView,
    entry_count: u64,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView> {
    match items_result {
        Err(error) => Err(error),
        Ok(items) => finish_sorted_mapping_spec(
            sort_mapping_items_spec(items, edges, records, node_byte),
            collection,
            entry_count,
            edges,
            records,
            total_before,
            node_byte,
            limits,
        ),
    }
}

pub open spec fn encode_mapping_spec(
    node_index: nat,
    collection: ResolvedCollectionView,
    edge_start: nat,
    edge_end: nat,
    edges: Seq<crate::resolve_topology::SemanticMappingEdgeView>,
    records: Seq<CanonicalStructuralKeyRecordView>,
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView> {
    if edge_start > edge_end || edge_end > edges.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: node_byte,
            },
        )
    } else {
        let entry_count = (edge_end - edge_start) as u64;
        let sort_limit = structural_key_effective_limit_spec(
            limits.max_mapping_sort_entries,
            MAX_PROFILE1_MAPPING_SORT_ENTRIES,
        );
        if entry_count > sort_limit {
            Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::MappingSortLimitExceeded,
                    byte_offset: node_byte,
                },
            )
        } else {
            finish_mapping_items_spec(
                build_mapping_items_tail_spec(
                    node_index,
                    edge_start,
                    edge_end,
                    edges,
                    records,
                    (edge_end - edge_start) as nat,
                    Seq::empty(),
                    node_byte,
                ),
                collection,
                entry_count,
                edges,
                records,
                total_before,
                node_byte,
                limits,
            )
        }
    }
}

#[verifier::rlimit(60)]
#[allow(clippy::too_many_arguments)]
fn encode_mapping(
    node_index: usize,
    collection: &ResolvedCollection,
    edge_start: usize,
    edge_end: usize,
    edges: &[SemanticMappingEdge],
    records: &[CanonicalStructuralKeyRecord],
    total_before: u64,
    node_byte: u64,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<Vec<CanonicalKeyByte>, CanonicalStructuralKeyError>)
    ensures
        encode_mapping_spec(
            node_index as nat,
            collection@,
            edge_start as nat,
            edge_end as nat,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            total_before,
            node_byte,
            limits@,
        ) == match result {
            Ok(bytes) => Ok(
                crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
            ),
            Err(error) => Err(error@),
        },
{
    let ghost whole_expected = encode_mapping_spec(
        node_index as nat,
        collection@,
        edge_start as nat,
        edge_end as nat,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        total_before,
        node_byte,
        limits@,
    );
    if edge_start > edge_end || edge_end > edges.len() {
        proof {
            reveal(encode_mapping_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                node_byte,
            ),
        );
    }
    let entry_count = (edge_end - edge_start) as u64;
    let sort_limit = effective_limit(
        limits.max_mapping_sort_entries(),
        MAX_PROFILE1_MAPPING_SORT_ENTRIES,
    );
    if entry_count > sort_limit {
        proof {
            reveal(encode_mapping_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::MappingSortLimitExceeded,
                node_byte,
            ),
        );
    }
    let mut items = Vec::new();
    let ghost items_expected = build_mapping_items_tail_spec(
        node_index as nat,
        edge_start as nat,
        edge_end as nat,
        crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
        canonical_structural_key_record_views_spec(records@),
        (edge_end - edge_start) as nat,
        Seq::empty(),
        node_byte,
    );
    proof {
        reveal(encode_mapping_spec);
        assert(whole_expected == finish_mapping_items_spec(
            items_expected,
            collection@,
            entry_count,
            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
            canonical_structural_key_record_views_spec(records@),
            total_before,
            node_byte,
            limits@,
        ));
    }
    let mut edge_index = edge_start;
    while edge_index < edge_end
        invariant
            edge_start <= edge_index <= edge_end,
            edge_end <= edges.len(),
            items@ == mapping_edge_index_range_spec(edge_start as nat, edge_index as nat),
            whole_expected == encode_mapping_spec(
                node_index as nat,
                collection@,
                edge_start as nat,
                edge_end as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                total_before,
                node_byte,
                limits@,
            ),
            whole_expected == finish_mapping_items_spec(
                items_expected,
                collection@,
                entry_count,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                total_before,
                node_byte,
                limits@,
            ),
            items_expected == build_mapping_items_tail_spec(
                node_index as nat,
                edge_index as nat,
                edge_end as nat,
                crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                canonical_structural_key_record_views_spec(records@),
                (edge_end - edge_index) as nat,
                items@,
                node_byte,
            ),
        decreases edge_end - edge_index,
    {
        let edge = &edges[edge_index];
        if edge.key_node_index() >= node_index as u64 || edge.value_node_index()
            >= node_index as u64 || edge.key_node_index() >= records.len() as u64
            || edge.value_node_index() >= records.len() as u64 {
            proof {
                reveal(build_mapping_items_tail_spec);
                reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
                reveal(canonical_structural_key_record_views_spec);
                assert(items_expected == Err(
                    CanonicalStructuralKeyErrorView {
                        kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node_byte,
                    },
                ));
                match items_expected {
                    Err(item_error) => {
                        assert(item_error == (CanonicalStructuralKeyErrorView {
                            kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            byte_offset: node_byte,
                        }));
                        reveal(finish_mapping_items_spec);
                        assert(finish_mapping_items_spec(
                            items_expected,
                            collection@,
                            entry_count,
                            crate::resolve_topology::semantic_mapping_edge_views_spec(edges@),
                            canonical_structural_key_record_views_spec(records@),
                            total_before,
                            node_byte,
                            limits@,
                        ) == Err(item_error));
                    },
                    Ok(_) => {
                        assert(false);
                    },
                }
                assert(whole_expected == Err(
                    CanonicalStructuralKeyErrorView {
                        kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        byte_offset: node_byte,
                    },
                ));
                reveal(encode_mapping_spec);
            }
            return Err(
                CanonicalStructuralKeyError::at(
                    CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    node_byte,
                ),
            );
        }
        items.push(edge_index as u64);
        proof {
            reveal(build_mapping_items_tail_spec);
            reveal(crate::resolve_topology::semantic_mapping_edge_views_spec);
            reveal(canonical_structural_key_record_views_spec);
            reveal(mapping_edge_index_range_spec);
            assert(items@ =~= mapping_edge_index_range_spec(
                edge_start as nat,
                (edge_index + 1) as nat,
            ));
        }
        edge_index += 1;
    }
    proof {
        reveal(build_mapping_items_tail_spec);
        assert(items_expected == Ok(items@));
    }
    let ghost item_views = items@;
    let sorted = sort_mapping_items(items, Ghost(item_views), edges, records, node_byte)?;
    let mut build = StructuralKeyBuild::empty(total_before, node_byte);
    begin_collection(&mut build, 0x32, collection, limits)?;
    build.append_u64(entry_count, limits)?;
    let ghost build_before_pairs = build@;
    append_sorted_mapping_pairs(
        &mut build,
        Ghost(build_before_pairs),
        &sorted,
        edges,
        records,
        limits,
    )?;
    proof {
        reveal(encode_mapping_pairs_tail_spec);
        reveal(finish_sorted_mapping_spec);
        reveal(encode_mapping_spec);
    }
    Ok(build.bytes)
}

#[verifier::ext_equal]
pub struct StructuralRecordBuildView {
    pub records: Seq<CanonicalStructuralKeyRecordView>,
    pub total_key_bytes: u64,
}

struct StructuralRecordBuild {
    records: Vec<CanonicalStructuralKeyRecord>,
    total_key_bytes: u64,
}

impl View for StructuralRecordBuild {
    type V = StructuralRecordBuildView;

    closed spec fn view(&self) -> StructuralRecordBuildView {
        StructuralRecordBuildView {
            records: canonical_structural_key_record_views_spec(self.records@),
            total_key_bytes: self.total_key_bytes,
        }
    }
}

impl StructuralRecordBuild {
    fn empty() -> (build: Self)
        ensures
            build@ == (StructuralRecordBuildView { records: Seq::empty(), total_key_bytes: 0 }),
    {
        let build = Self { records: Vec::new(), total_key_bytes: 0 };
        proof {
            reveal(canonical_structural_key_record_views_spec);
        }
        build
    }
}

pub open spec fn finish_structural_record_spec(
    node_index: nat,
    node_byte: u64,
    build: StructuralRecordBuildView,
    encoded: Result<Seq<CanonicalKeyByteView>, CanonicalStructuralKeyErrorView>,
) -> Result<StructuralRecordBuildView, CanonicalStructuralKeyErrorView> {
    match encoded {
        Err(error) => Err(error),
        Ok(bytes) => if bytes.len() > u64::MAX - build.total_key_bytes {
            Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node_byte,
                },
            )
        } else {
            Ok(
                StructuralRecordBuildView {
                    records: build.records.push(
                        CanonicalStructuralKeyRecordView {
                            node_index: node_index as u64,
                            byte_start: node_byte,
                            bytes,
                        },
                    ),
                    total_key_bytes: (build.total_key_bytes + bytes.len()) as u64,
                },
            )
        },
    }
}

pub open spec fn structural_node_step_spec(
    scalar_keys: CanonicalScalarKeySourceView,
    node_index: nat,
    build: StructuralRecordBuildView,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralRecordBuildView, CanonicalStructuralKeyErrorView> {
    let nodes = scalar_keys.graph.node_table.nodes;
    let record_limit = structural_key_effective_limit_spec(
        limits.max_records,
        MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
    );
    if node_index >= nodes.len() {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: scalar_keys.graph.source_len_bytes,
            },
        )
    } else {
        let node = nodes[node_index as int];
        let node_byte = node.byte_start;
        if build.records.len() >= record_limit {
            Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::RecordLimitExceeded,
                    byte_offset: node_byte,
                },
            )
        } else if node.cst_node_index != node_index {
            Err(
                CanonicalStructuralKeyErrorView {
                    kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                    byte_offset: node_byte,
                },
            )
        } else {
            match node.kind {
                SemanticNodeKind::Scalar => match node.value_index {
                    None => Err(
                        CanonicalStructuralKeyErrorView {
                            kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            byte_offset: node_byte,
                        },
                    ),
                    Some(scalar_index) => if scalar_index >= scalar_keys.records.len() {
                        Err(
                            CanonicalStructuralKeyErrorView {
                                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                                byte_offset: node_byte,
                            },
                        )
                    } else {
                        finish_structural_record_spec(
                            node_index,
                            node_byte,
                            build,
                            encode_scalar_or_alias_spec(
                                scalar_keys.records[scalar_index as int].bytes,
                                build.total_key_bytes,
                                node_byte,
                                limits,
                            ),
                        )
                    },
                },
                SemanticNodeKind::Alias => match node.alias_target_node_index {
                    None => Err(
                        CanonicalStructuralKeyErrorView {
                            kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            byte_offset: node_byte,
                        },
                    ),
                    Some(target) => if target >= node_index || target >= build.records.len() {
                        Err(
                            CanonicalStructuralKeyErrorView {
                                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                                byte_offset: node_byte,
                            },
                        )
                    } else {
                        finish_structural_record_spec(
                            node_index,
                            node_byte,
                            build,
                            encode_scalar_or_alias_spec(
                                build.records[target as int].bytes,
                                build.total_key_bytes,
                                node_byte,
                                limits,
                            ),
                        )
                    },
                },
                SemanticNodeKind::Sequence | SemanticNodeKind::Mapping => match node.value_index {
                    None => Err(
                        CanonicalStructuralKeyErrorView {
                            kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            byte_offset: node_byte,
                        },
                    ),
                    Some(collection_index) => if collection_index
                        >= scalar_keys.graph.node_table.collections.len() {
                        Err(
                            CanonicalStructuralKeyErrorView {
                                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                                byte_offset: node_byte,
                            },
                        )
                    } else if node.kind == SemanticNodeKind::Sequence {
                        finish_structural_record_spec(
                            node_index,
                            node_byte,
                            build,
                            encode_sequence_spec(
                                node_index,
                                scalar_keys.graph.node_table.collections[collection_index as int],
                                node.edge_start as nat,
                                node.edge_end as nat,
                                scalar_keys.graph.node_table.topology.sequence_edges,
                                build.records,
                                build.total_key_bytes,
                                node_byte,
                                limits,
                            ),
                        )
                    } else {
                        finish_structural_record_spec(
                            node_index,
                            node_byte,
                            build,
                            encode_mapping_spec(
                                node_index,
                                scalar_keys.graph.node_table.collections[collection_index as int],
                                node.edge_start as nat,
                                node.edge_end as nat,
                                scalar_keys.graph.node_table.topology.mapping_edges,
                                build.records,
                                build.total_key_bytes,
                                node_byte,
                                limits,
                            ),
                        )
                    },
                },
            }
        }
    }
}

fn finish_structural_record(
    node_index: usize,
    node_byte: u64,
    build: &mut StructuralRecordBuild,
    encoded: Result<Vec<CanonicalKeyByte>, CanonicalStructuralKeyError>,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    ensures
        finish_structural_record_spec(
            node_index as nat,
            node_byte,
            old(build)@,
            match encoded {
                Ok(ref bytes) => Ok(
                    crate::resolve_canonical_scalar_key::canonical_key_byte_views_spec(bytes@),
                ),
                Err(error) => Err(error@),
            },
        ) == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let bytes = match encoded {
        Err(error) => {
            proof {
                reveal(finish_structural_record_spec);
            }
            return Err(error);
        },
        Ok(bytes) => bytes,
    };
    if bytes.len() as u64 > u64::MAX - build.total_key_bytes {
        proof {
            reveal(finish_structural_record_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                node_byte,
            ),
        );
    }
    build.total_key_bytes += bytes.len() as u64;
    let record = CanonicalStructuralKeyRecord::new(node_index as u64, node_byte, bytes);
    proof {
        lemma_canonical_structural_key_record_views_push(build.records@, record);
    }
    build.records.push(record);
    proof {
        reveal(finish_structural_record_spec);
    }
    Ok(())
}

#[verifier::rlimit(150)]
fn compose_structural_record_item(
    scalar_keys: &CanonicalScalarKeySource,
    node_index: usize,
    build: &mut StructuralRecordBuild,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<(), CanonicalStructuralKeyError>)
    ensures
        structural_node_step_spec(scalar_keys@, node_index as nat, old(build)@, limits@)
            == match result {
            Ok(()) => Ok(final(build)@),
            Err(error) => Err(error@),
        },
{
    let graph = scalar_keys.graph();
    let table = graph.node_table();
    let nodes = table.nodes();
    if node_index >= nodes.len() {
        proof {
            reveal(structural_node_step_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                scalar_keys.graph().source_len_bytes(),
            ),
        );
    }
    let node = &nodes[node_index];
    let node_byte = node.byte_start();
    let record_limit = effective_limit(
        limits.max_records(),
        MAX_PROFILE1_CANONICAL_STRUCTURAL_KEY_RECORDS,
    );
    if build.records.len() as u64 >= record_limit {
        proof {
            reveal(structural_node_step_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::RecordLimitExceeded,
                node_byte,
            ),
        );
    }
    if node.cst_node_index() != node_index as u64 {
        proof {
            reveal(structural_node_step_spec);
        }
        return Err(
            CanonicalStructuralKeyError::at(
                CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                node_byte,
            ),
        );
    }
    let topology = table.topology();
    let scalar_records = scalar_keys.records();
    let collections = table.collections();
    let encoded = match node.kind() {
        SemanticNodeKind::Scalar => {
            let scalar_index_u64 = match node.value_index() {
                Some(index) => index,
                None => {
                    proof {
                        reveal(structural_node_step_spec);
                    }
                    return Err(
                        CanonicalStructuralKeyError::at(
                            CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            node_byte,
                        ),
                    );
                },
            };
            if scalar_index_u64 >= scalar_records.len() as u64 {
                proof {
                    reveal(structural_node_step_spec);
                }
                return Err(
                    CanonicalStructuralKeyError::at(
                        CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        node_byte,
                    ),
                );
            }
            encode_scalar_or_alias(
                scalar_records[scalar_index_u64 as usize].bytes(),
                build.total_key_bytes,
                node_byte,
                limits,
            )
        },
        SemanticNodeKind::Alias => {
            let target_u64 = match node.alias_target_node_index() {
                Some(index) => index,
                None => {
                    proof {
                        reveal(structural_node_step_spec);
                    }
                    return Err(
                        CanonicalStructuralKeyError::at(
                            CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            node_byte,
                        ),
                    );
                },
            };
            if target_u64 >= node_index as u64 || target_u64 >= build.records.len() as u64 {
                proof {
                    reveal(structural_node_step_spec);
                }
                return Err(
                    CanonicalStructuralKeyError::at(
                        CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        node_byte,
                    ),
                );
            }
            encode_scalar_or_alias(
                build.records[target_u64 as usize].bytes(),
                build.total_key_bytes,
                node_byte,
                limits,
            )
        },
        SemanticNodeKind::Sequence | SemanticNodeKind::Mapping => {
            let collection_index_u64 = match node.value_index() {
                Some(index) => index,
                None => {
                    proof {
                        reveal(structural_node_step_spec);
                    }
                    return Err(
                        CanonicalStructuralKeyError::at(
                            CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            node_byte,
                        ),
                    );
                },
            };
            if collection_index_u64 >= collections.len() as u64 {
                proof {
                    reveal(structural_node_step_spec);
                }
                return Err(
                    CanonicalStructuralKeyError::at(
                        CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                        node_byte,
                    ),
                );
            }
            let collection = &collections[collection_index_u64 as usize];
            if node.kind() == SemanticNodeKind::Sequence {
                if node.edge_start() > node.edge_end() || node.edge_end()
                    > topology.sequence_edges().len() as u64 {
                    proof {
                        reveal(structural_node_step_spec);
                        reveal(encode_sequence_spec);
                    }
                    return Err(
                        CanonicalStructuralKeyError::at(
                            CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            node_byte,
                        ),
                    );
                }
                encode_sequence(
                    node_index,
                    collection,
                    node.edge_start() as usize,
                    node.edge_end() as usize,
                    topology.sequence_edges(),
                    build.records.as_slice(),
                    build.total_key_bytes,
                    node_byte,
                    limits,
                )
            } else {
                if node.edge_start() > node.edge_end() || node.edge_end()
                    > topology.mapping_edges().len() as u64 {
                    proof {
                        reveal(structural_node_step_spec);
                        reveal(encode_mapping_spec);
                    }
                    return Err(
                        CanonicalStructuralKeyError::at(
                            CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                            node_byte,
                        ),
                    );
                }
                encode_mapping(
                    node_index,
                    collection,
                    node.edge_start() as usize,
                    node.edge_end() as usize,
                    topology.mapping_edges(),
                    build.records.as_slice(),
                    build.total_key_bytes,
                    node_byte,
                    limits,
                )
            }
        },
    };
    let result = finish_structural_record(node_index, node_byte, build, encoded);
    proof {
        reveal(structural_node_step_spec);
    }
    result
}

pub closed spec fn structural_records_tail_spec(
    scalar_keys: CanonicalScalarKeySourceView,
    node_index: nat,
    fuel: nat,
    build: StructuralRecordBuildView,
    limits: CanonicalStructuralKeyLimitsView,
) -> Result<StructuralRecordBuildView, CanonicalStructuralKeyErrorView>
    decreases fuel,
{
    if node_index >= scalar_keys.graph.node_table.nodes.len() {
        Ok(build)
    } else if fuel == 0 {
        Err(
            CanonicalStructuralKeyErrorView {
                kind: CanonicalStructuralKeyErrorKind::InternalInvariantViolation,
                byte_offset: scalar_keys.graph.source_len_bytes,
            },
        )
    } else {
        match structural_node_step_spec(scalar_keys, node_index, build, limits) {
            Err(error) => Err(error),
            Ok(next) => structural_records_tail_spec(
                scalar_keys,
                (node_index + 1) as nat,
                (fuel - 1) as nat,
                next,
                limits,
            ),
        }
    }
}

#[verifier::rlimit(60)]
fn compose_structural_records(
    scalar_keys: &CanonicalScalarKeySource,
    limits: CanonicalStructuralKeyLimits,
) -> (result: Result<StructuralRecordBuild, CanonicalStructuralKeyError>)
    ensures
        structural_records_tail_spec(
            scalar_keys@,
            0,
            scalar_keys@.graph.node_table.nodes.len() as nat,
            StructuralRecordBuildView { records: Seq::empty(), total_key_bytes: 0 },
            limits@,
        ) == match result {
            Ok(build) => Ok(build@),
            Err(error) => Err(error@),
        },
{
    let graph = scalar_keys.graph();
    let table = graph.node_table();
    let nodes = table.nodes();
    let mut build = StructuralRecordBuild::empty();
    let mut node_index = 0usize;
    let ghost expected = structural_records_tail_spec(
        scalar_keys@,
        0,
        scalar_keys@.graph.node_table.nodes.len() as nat,
        build@,
        limits@,
    );
    while node_index < nodes.len()
        invariant
            node_index <= nodes.len(),
            nodes.len() == scalar_keys@.graph.node_table.nodes.len(),
            expected == structural_records_tail_spec(
                scalar_keys@,
                0,
                scalar_keys@.graph.node_table.nodes.len() as nat,
                StructuralRecordBuildView { records: Seq::empty(), total_key_bytes: 0 },
                limits@,
            ),
            expected == structural_records_tail_spec(
                scalar_keys@,
                node_index as nat,
                (nodes.len() - node_index) as nat,
                build@,
                limits@,
            ),
        decreases nodes.len() - node_index,
    {
        let step = compose_structural_record_item(scalar_keys, node_index, &mut build, limits);
        match step {
            Err(error) => {
                proof {
                    reveal(structural_records_tail_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
            Ok(()) => {},
        }
        proof {
            reveal(structural_records_tail_spec);
        }
        node_index += 1;
    }
    proof {
        reveal(structural_records_tail_spec);
    }
    Ok(build)
}

pub open spec fn finalize_canonical_structural_key_spec(
    scalar_keys: CanonicalScalarKeySourceView,
    result: Result<StructuralRecordBuildView, CanonicalStructuralKeyErrorView>,
) -> Result<CanonicalStructuralKeySourceView, CanonicalStructuralKeyErrorView> {
    match result {
        Err(error) => Err(error),
        Ok(build) => Ok(
            CanonicalStructuralKeySourceView {
                profile_version: scalar_keys.graph.profile_version,
                transformation_version: CANONICAL_STRUCTURAL_KEY_TRANSFORMATION_VERSION,
                source_len_bytes: scalar_keys.graph.source_len_bytes,
                input_node_count: scalar_keys.input_node_count,
                total_key_bytes: build.total_key_bytes,
                scalar_keys,
                records: build.records,
            },
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub open spec fn compose_profile1_canonical_structural_keys_spec(
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
    scalar_key_limits: CanonicalScalarKeyLimitsView,
    structural_limits: CanonicalStructuralKeyLimitsView,
) -> Result<CanonicalStructuralKeySourceView, CanonicalStructuralKeyErrorView> {
    match crate::resolve_canonical_scalar_key::compose_profile1_canonical_scalar_keys_spec(
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
        scalar_key_limits,
    ) {
        Err(error) => Err(map_scalar_key_error_spec(error)),
        Ok(scalar_keys) => finalize_canonical_structural_key_spec(
            scalar_keys,
            structural_records_tail_spec(
                scalar_keys,
                0,
                scalar_keys.graph.node_table.nodes.len() as nat,
                StructuralRecordBuildView { records: Seq::empty(), total_key_bytes: 0 },
                structural_limits,
            ),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub open spec fn canonical_structural_key_source_well_formed_spec(
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
    scalar_key_limits: CanonicalScalarKeyLimitsView,
    structural_limits: CanonicalStructuralKeyLimitsView,
    source: CanonicalStructuralKeySourceView,
) -> bool {
    compose_profile1_canonical_structural_keys_spec(
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
        scalar_key_limits,
        structural_limits,
    ) == Ok(source)
}

#[allow(clippy::too_many_arguments)]
pub proof fn lemma_canonical_structural_key_success_is_well_formed(
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
    scalar_key_limits: CanonicalScalarKeyLimitsView,
    structural_limits: CanonicalStructuralKeyLimitsView,
    source: CanonicalStructuralKeySourceView,
)
    requires
        compose_profile1_canonical_structural_keys_spec(
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
            scalar_key_limits,
            structural_limits,
        ) == Ok(source),
    ensures
        canonical_structural_key_source_well_formed_spec(
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
            scalar_key_limits,
            structural_limits,
            source,
        ),
{
    reveal(canonical_structural_key_source_well_formed_spec);
}

#[allow(clippy::too_many_arguments)]
pub proof fn lemma_canonical_structural_key_well_formed_authenticates_exact_result(
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
    scalar_key_limits: CanonicalScalarKeyLimitsView,
    structural_limits: CanonicalStructuralKeyLimitsView,
    source: CanonicalStructuralKeySourceView,
)
    requires
        canonical_structural_key_source_well_formed_spec(
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
            scalar_key_limits,
            structural_limits,
            source,
        ),
    ensures
        compose_profile1_canonical_structural_keys_spec(
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
            scalar_key_limits,
            structural_limits,
        ) == Ok(source),
{
    reveal(canonical_structural_key_source_well_formed_spec);
}

#[allow(clippy::too_many_arguments)]
pub fn compose_profile1_canonical_structural_keys(
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
    scalar_key_limits: CanonicalScalarKeyLimits,
    structural_limits: CanonicalStructuralKeyLimits,
) -> (result: Result<CanonicalStructuralKeySource, CanonicalStructuralKeyError>)
    ensures
        compose_profile1_canonical_structural_keys_spec(
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
            scalar_key_limits@,
            structural_limits@,
        ) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let scalar_keys = match compose_profile1_canonical_scalar_keys(
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
        scalar_key_limits,
    ) {
        Err(error) => {
            proof {
                reveal(compose_profile1_canonical_structural_keys_spec);
            }
            return Err(map_scalar_key_error(error));
        },
        Ok(source) => source,
    };
    let build = match compose_structural_records(&scalar_keys, structural_limits) {
        Err(error) => {
            proof {
                reveal(compose_profile1_canonical_structural_keys_spec);
                reveal(finalize_canonical_structural_key_spec);
            }
            return Err(error);
        },
        Ok(build) => build,
    };
    let source = CanonicalStructuralKeySource::new(
        scalar_keys,
        build.records,
        build.total_key_bytes,
    );
    proof {
        reveal(compose_profile1_canonical_structural_keys_spec);
        reveal(finalize_canonical_structural_key_spec);
    }
    Ok(source)
}

} // verus!
