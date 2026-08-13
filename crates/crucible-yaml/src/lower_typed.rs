//! Verified binding of one canonical YAML value to one typed-field schema node.
//!
//! This is the exact kind-authentication submachine used by graph-wide schema-directed lowering.
//! It distinguishes every Core scalar value, custom scalar tags, and Core/custom collection tags
//! without coercion or host numeric conversion.
#[allow(unused_imports)]
use crate::lower::CanonicalYamlGraphSourceView;
use crate::lower::{CanonicalYamlGraphSource, CanonicalYamlNodeKind};
use crate::resolve_collection_tag::ResolvedCollectionTag;
#[allow(unused_imports)]
use crate::resolve_collection_tag::ResolvedCollectionView;
use crate::resolve_scalar_value::{ResolvedScalarTag, ResolvedScalarValue};
#[allow(unused_imports)]
use crate::resolve_scalar_value::{ResolvedScalarValueView, ResolvedScalarView};
#[allow(unused_imports)]
use crate::schema::CompiledTypedFieldSchemaView;
use crate::schema::{CompiledTypedFieldSchema, TypedSchemaValueKind};
use vstd::prelude::*;

verus! {

pub const TYPED_YAML_VALUE_BINDING_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedYamlValueBinding {
    binding_version: u16,
    canonical_profile_version: u16,
    schema_version: u16,
    yaml_node_index: u64,
    resolved_yaml_node_index: u64,
    schema_node_index: u64,
    kind: TypedSchemaValueKind,
    byte_start: u64,
    byte_end: u64,
    scalar_index: Option<u64>,
    collection_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct TypedYamlValueBindingView {
    pub binding_version: u16,
    pub canonical_profile_version: u16,
    pub schema_version: u16,
    pub yaml_node_index: u64,
    pub resolved_yaml_node_index: u64,
    pub schema_node_index: u64,
    pub kind: TypedSchemaValueKind,
    pub byte_start: u64,
    pub byte_end: u64,
    pub scalar_index: Option<u64>,
    pub collection_index: Option<u64>,
}

impl View for TypedYamlValueBinding {
    type V = TypedYamlValueBindingView;

    closed spec fn view(&self) -> TypedYamlValueBindingView {
        TypedYamlValueBindingView {
            binding_version: self.binding_version,
            canonical_profile_version: self.canonical_profile_version,
            schema_version: self.schema_version,
            yaml_node_index: self.yaml_node_index,
            resolved_yaml_node_index: self.resolved_yaml_node_index,
            schema_node_index: self.schema_node_index,
            kind: self.kind,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            scalar_index: self.scalar_index,
            collection_index: self.collection_index,
        }
    }
}

impl TypedYamlValueBinding {
    #[allow(clippy::too_many_arguments)]
    fn new(
        canonical_profile_version: u16,
        schema_version: u16,
        yaml_node_index: u64,
        resolved_yaml_node_index: u64,
        schema_node_index: u64,
        kind: TypedSchemaValueKind,
        byte_start: u64,
        byte_end: u64,
        scalar_index: Option<u64>,
        collection_index: Option<u64>,
    ) -> (binding: Self)
        ensures
            binding@ == (TypedYamlValueBindingView {
                binding_version: TYPED_YAML_VALUE_BINDING_VERSION,
                canonical_profile_version,
                schema_version,
                yaml_node_index,
                resolved_yaml_node_index,
                schema_node_index,
                kind,
                byte_start,
                byte_end,
                scalar_index,
                collection_index,
            }),
    {
        Self {
            binding_version: TYPED_YAML_VALUE_BINDING_VERSION,
            canonical_profile_version,
            schema_version,
            yaml_node_index,
            resolved_yaml_node_index,
            schema_node_index,
            kind,
            byte_start,
            byte_end,
            scalar_index,
            collection_index,
        }
    }

    pub fn binding_version(&self) -> (value: u16)
        ensures
            value == self@.binding_version,
    {
        self.binding_version
    }

    pub fn canonical_profile_version(&self) -> (value: u16)
        ensures
            value == self@.canonical_profile_version,
    {
        self.canonical_profile_version
    }

    pub fn schema_version(&self) -> (value: u16)
        ensures
            value == self@.schema_version,
    {
        self.schema_version
    }

    pub fn yaml_node_index(&self) -> (value: u64)
        ensures
            value == self@.yaml_node_index,
    {
        self.yaml_node_index
    }

    pub fn resolved_yaml_node_index(&self) -> (value: u64)
        ensures
            value == self@.resolved_yaml_node_index,
    {
        self.resolved_yaml_node_index
    }

    pub fn schema_node_index(&self) -> (value: u64)
        ensures
            value == self@.schema_node_index,
    {
        self.schema_node_index
    }

    pub fn kind(&self) -> (value: TypedSchemaValueKind)
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum TypedValueBindingErrorKind {
    YamlNodeIndexOutOfRange,
    SchemaNodeIndexOutOfRange,
    YamlValueKindMismatch,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedValueBindingError {
    kind: TypedValueBindingErrorKind,
    byte_offset: u64,
    yaml_node_index: u64,
    schema_node_index: u64,
}

#[verifier::ext_equal]
pub struct TypedValueBindingErrorView {
    pub kind: TypedValueBindingErrorKind,
    pub byte_offset: u64,
    pub yaml_node_index: u64,
    pub schema_node_index: u64,
}

impl View for TypedValueBindingError {
    type V = TypedValueBindingErrorView;

    closed spec fn view(&self) -> TypedValueBindingErrorView {
        TypedValueBindingErrorView {
            kind: self.kind,
            byte_offset: self.byte_offset,
            yaml_node_index: self.yaml_node_index,
            schema_node_index: self.schema_node_index,
        }
    }
}

impl TypedValueBindingError {
    fn at(
        kind: TypedValueBindingErrorKind,
        byte_offset: u64,
        yaml_node_index: u64,
        schema_node_index: u64,
    ) -> (error: Self)
        ensures
            error@ == (TypedValueBindingErrorView {
                kind,
                byte_offset,
                yaml_node_index,
                schema_node_index,
            }),
    {
        Self { kind, byte_offset, yaml_node_index, schema_node_index }
    }

    pub fn kind(&self) -> (value: TypedValueBindingErrorKind)
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

    pub fn yaml_node_index(&self) -> (value: u64)
        ensures
            value == self@.yaml_node_index,
    {
        self.yaml_node_index
    }

    pub fn schema_node_index(&self) -> (value: u64)
        ensures
            value == self@.schema_node_index,
    {
        self.schema_node_index
    }
}

pub open spec fn typed_scalar_kind_matches_spec(
    kind: TypedSchemaValueKind,
    tag: ResolvedScalarTag,
    value: ResolvedScalarValueView,
) -> bool {
    match (kind, tag, value) {
        (
            TypedSchemaValueKind::Null,
            ResolvedScalarTag::CoreNull,
            ResolvedScalarValueView::Null,
        ) => { true },
        (
            TypedSchemaValueKind::Boolean,
            ResolvedScalarTag::CoreBoolean,
            ResolvedScalarValueView::Boolean(_),
        ) => true,
        (
            TypedSchemaValueKind::Integer,
            ResolvedScalarTag::CoreInteger,
            ResolvedScalarValueView::Integer(_),
        ) => true,
        (
            TypedSchemaValueKind::FiniteFloat,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValueView::FiniteFloat(_),
        ) => true,
        (
            TypedSchemaValueKind::PositiveInfinity,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValueView::PositiveInfinity,
        ) => true,
        (
            TypedSchemaValueKind::NegativeInfinity,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValueView::NegativeInfinity,
        ) => true,
        (
            TypedSchemaValueKind::NotANumber,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValueView::NotANumber,
        ) => true,
        (
            TypedSchemaValueKind::String,
            ResolvedScalarTag::CoreString,
            ResolvedScalarValueView::String,
        ) => true,
        (
            TypedSchemaValueKind::CustomScalar,
            ResolvedScalarTag::CustomGlobal,
            ResolvedScalarValueView::String,
        )
        | (
            TypedSchemaValueKind::CustomScalar,
            ResolvedScalarTag::CustomLocal,
            ResolvedScalarValueView::String,
        ) => true,
        _ => false,
    }
}

#[allow(clippy::match_like_matches_macro)]
fn typed_scalar_kind_matches(
    kind: TypedSchemaValueKind,
    tag: ResolvedScalarTag,
    value: &ResolvedScalarValue,
) -> (matches: bool)
    ensures
        matches == typed_scalar_kind_matches_spec(kind, tag, value@),
{
    match (kind, tag, value) {
        (
            TypedSchemaValueKind::Null,
            ResolvedScalarTag::CoreNull,
            ResolvedScalarValue::Null,
        ) => true,
        (
            TypedSchemaValueKind::Boolean,
            ResolvedScalarTag::CoreBoolean,
            ResolvedScalarValue::Boolean(_),
        ) => true,
        (
            TypedSchemaValueKind::Integer,
            ResolvedScalarTag::CoreInteger,
            ResolvedScalarValue::Integer(_),
        ) => true,
        (
            TypedSchemaValueKind::FiniteFloat,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValue::FiniteFloat(_),
        ) => true,
        (
            TypedSchemaValueKind::PositiveInfinity,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValue::PositiveInfinity,
        ) => true,
        (
            TypedSchemaValueKind::NegativeInfinity,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValue::NegativeInfinity,
        ) => true,
        (
            TypedSchemaValueKind::NotANumber,
            ResolvedScalarTag::CoreFloat,
            ResolvedScalarValue::NotANumber,
        ) => true,
        (
            TypedSchemaValueKind::String,
            ResolvedScalarTag::CoreString,
            ResolvedScalarValue::String,
        ) => true,
        (
            TypedSchemaValueKind::CustomScalar,
            ResolvedScalarTag::CustomGlobal,
            ResolvedScalarValue::String,
        )
        | (
            TypedSchemaValueKind::CustomScalar,
            ResolvedScalarTag::CustomLocal,
            ResolvedScalarValue::String,
        ) => true,
        _ => false,
    }
}

pub open spec fn typed_collection_kind_matches_spec(
    kind: TypedSchemaValueKind,
    node_kind: CanonicalYamlNodeKind,
    tag: ResolvedCollectionTag,
) -> bool {
    match (kind, node_kind, tag) {
        (
            TypedSchemaValueKind::Sequence,
            CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CoreSequence,
        ) => true,
        (
            TypedSchemaValueKind::CustomSequence,
            CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CustomGlobal,
        )
        | (
            TypedSchemaValueKind::CustomSequence,
            CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CustomLocal,
        ) => true,
        (
            TypedSchemaValueKind::Mapping,
            CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CoreMapping,
        ) => true,
        (
            TypedSchemaValueKind::CustomMapping,
            CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CustomGlobal,
        )
        | (
            TypedSchemaValueKind::CustomMapping,
            CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CustomLocal,
        ) => true,
        _ => false,
    }
}

#[allow(clippy::match_like_matches_macro)]
fn typed_collection_kind_matches(
    kind: TypedSchemaValueKind,
    node_kind: CanonicalYamlNodeKind,
    tag: ResolvedCollectionTag,
) -> (matches: bool)
    ensures
        matches == typed_collection_kind_matches_spec(kind, node_kind, tag),
{
    match (kind, node_kind, tag) {
        (
            TypedSchemaValueKind::Sequence,
            CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CoreSequence,
        ) => true,
        (
            TypedSchemaValueKind::CustomSequence,
            CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CustomGlobal,
        )
        | (
            TypedSchemaValueKind::CustomSequence,
            CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CustomLocal,
        ) => true,
        (
            TypedSchemaValueKind::Mapping,
            CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CoreMapping,
        ) => true,
        (
            TypedSchemaValueKind::CustomMapping,
            CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CustomGlobal,
        )
        | (
            TypedSchemaValueKind::CustomMapping,
            CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CustomLocal,
        ) => true,
        _ => false,
    }
}

pub open spec fn canonical_yaml_graph_scalars_spec(graph: CanonicalYamlGraphSourceView) -> Seq<
    ResolvedScalarView,
> {
    graph.input.input.structural_keys.scalar_keys.graph.node_table.scalars.scalars
}

pub open spec fn canonical_yaml_graph_collections_spec(graph: CanonicalYamlGraphSourceView) -> Seq<
    ResolvedCollectionView,
> {
    graph.input.input.structural_keys.scalar_keys.graph.node_table.collections
}

pub open spec fn binding_error_spec(
    kind: TypedValueBindingErrorKind,
    byte_offset: u64,
    yaml_node_index: u64,
    schema_node_index: u64,
) -> TypedValueBindingErrorView {
    TypedValueBindingErrorView { kind, byte_offset, yaml_node_index, schema_node_index }
}

pub open spec fn binding_result_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
    scalar_index: Option<u64>,
    collection_index: Option<u64>,
) -> TypedYamlValueBindingView {
    let node = graph.nodes[yaml_node_index as int];
    TypedYamlValueBindingView {
        binding_version: TYPED_YAML_VALUE_BINDING_VERSION,
        canonical_profile_version: graph.profile_version,
        schema_version: schema.schema_version,
        yaml_node_index,
        resolved_yaml_node_index: node.resolved_node_index,
        schema_node_index,
        kind: schema.schema.nodes[schema_node_index as int].kind,
        byte_start: node.byte_start,
        byte_end: node.byte_end,
        scalar_index,
        collection_index,
    }
}

pub open spec fn bind_profile1_typed_yaml_value_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
) -> Result<TypedYamlValueBindingView, TypedValueBindingErrorView> {
    if yaml_node_index >= graph.nodes.len() {
        Err(
            binding_error_spec(
                TypedValueBindingErrorKind::YamlNodeIndexOutOfRange,
                graph.source_len_bytes,
                yaml_node_index,
                schema_node_index,
            ),
        )
    } else {
        let node = graph.nodes[yaml_node_index as int];
        if schema_node_index >= schema.schema.nodes.len() {
            Err(
                binding_error_spec(
                    TypedValueBindingErrorKind::SchemaNodeIndexOutOfRange,
                    node.byte_start,
                    yaml_node_index,
                    schema_node_index,
                ),
            )
        } else if node.source_node_index != yaml_node_index || node.byte_start > node.byte_end
            || node.byte_end > graph.source_len_bytes || node.resolved_node_index
            >= graph.nodes.len() || schema.compilation_version
            != crate::schema::TYPED_FIELD_SCHEMA_COMPILATION_VERSION || schema.schema_version
            != schema.schema.schema_version || schema.node_count != schema.schema.nodes.len()
            || schema.field_count != schema.schema.fields.len() {
            Err(
                binding_error_spec(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start,
                    yaml_node_index,
                    schema_node_index,
                ),
            )
        } else {
            let schema_kind = schema.schema.nodes[schema_node_index as int].kind;
            let scalars = canonical_yaml_graph_scalars_spec(graph);
            let collections = canonical_yaml_graph_collections_spec(graph);
            match node.kind {
                CanonicalYamlNodeKind::Scalar => match node.scalar_index {
                    None => Err(
                        binding_error_spec(
                            TypedValueBindingErrorKind::InternalInvariantViolation,
                            node.byte_start,
                            yaml_node_index,
                            schema_node_index,
                        ),
                    ),
                    Some(scalar_index) => if node.collection_index.is_some() || node.edge_start
                        != node.edge_end || scalar_index >= scalars.len() {
                        Err(
                            binding_error_spec(
                                TypedValueBindingErrorKind::InternalInvariantViolation,
                                node.byte_start,
                                yaml_node_index,
                                schema_node_index,
                            ),
                        )
                    } else {
                        let scalar = scalars[scalar_index as int];
                        if scalar.node_index != node.resolved_node_index {
                            Err(
                                binding_error_spec(
                                    TypedValueBindingErrorKind::InternalInvariantViolation,
                                    node.byte_start,
                                    yaml_node_index,
                                    schema_node_index,
                                ),
                            )
                        } else if !typed_scalar_kind_matches_spec(
                            schema_kind,
                            scalar.tag,
                            scalar.value,
                        ) {
                            Err(
                                binding_error_spec(
                                    TypedValueBindingErrorKind::YamlValueKindMismatch,
                                    node.byte_start,
                                    yaml_node_index,
                                    schema_node_index,
                                ),
                            )
                        } else {
                            Ok(
                                binding_result_spec(
                                    graph,
                                    schema,
                                    yaml_node_index,
                                    schema_node_index,
                                    Some(scalar_index),
                                    None,
                                ),
                            )
                        }
                    },
                },
                CanonicalYamlNodeKind::Sequence => match node.collection_index {
                    None => Err(
                        binding_error_spec(
                            TypedValueBindingErrorKind::InternalInvariantViolation,
                            node.byte_start,
                            yaml_node_index,
                            schema_node_index,
                        ),
                    ),
                    Some(collection_index) => if node.scalar_index.is_some() || node.edge_start
                        > node.edge_end || node.edge_end > graph.sequence_entries.len()
                        || collection_index >= collections.len() {
                        Err(
                            binding_error_spec(
                                TypedValueBindingErrorKind::InternalInvariantViolation,
                                node.byte_start,
                                yaml_node_index,
                                schema_node_index,
                            ),
                        )
                    } else {
                        let collection = collections[collection_index as int];
                        if collection.node_index != node.resolved_node_index {
                            Err(
                                binding_error_spec(
                                    TypedValueBindingErrorKind::InternalInvariantViolation,
                                    node.byte_start,
                                    yaml_node_index,
                                    schema_node_index,
                                ),
                            )
                        } else if !typed_collection_kind_matches_spec(
                            schema_kind,
                            node.kind,
                            collection.tag,
                        ) {
                            Err(
                                binding_error_spec(
                                    TypedValueBindingErrorKind::YamlValueKindMismatch,
                                    node.byte_start,
                                    yaml_node_index,
                                    schema_node_index,
                                ),
                            )
                        } else {
                            Ok(
                                binding_result_spec(
                                    graph,
                                    schema,
                                    yaml_node_index,
                                    schema_node_index,
                                    None,
                                    Some(collection_index),
                                ),
                            )
                        }
                    },
                },
                CanonicalYamlNodeKind::Mapping => match node.collection_index {
                    None => Err(
                        binding_error_spec(
                            TypedValueBindingErrorKind::InternalInvariantViolation,
                            node.byte_start,
                            yaml_node_index,
                            schema_node_index,
                        ),
                    ),
                    Some(collection_index) => if node.scalar_index.is_some() || node.edge_start
                        > node.edge_end || node.edge_end > graph.mapping_entries.len()
                        || collection_index >= collections.len() {
                        Err(
                            binding_error_spec(
                                TypedValueBindingErrorKind::InternalInvariantViolation,
                                node.byte_start,
                                yaml_node_index,
                                schema_node_index,
                            ),
                        )
                    } else {
                        let collection = collections[collection_index as int];
                        if collection.node_index != node.resolved_node_index {
                            Err(
                                binding_error_spec(
                                    TypedValueBindingErrorKind::InternalInvariantViolation,
                                    node.byte_start,
                                    yaml_node_index,
                                    schema_node_index,
                                ),
                            )
                        } else if !typed_collection_kind_matches_spec(
                            schema_kind,
                            node.kind,
                            collection.tag,
                        ) {
                            Err(
                                binding_error_spec(
                                    TypedValueBindingErrorKind::YamlValueKindMismatch,
                                    node.byte_start,
                                    yaml_node_index,
                                    schema_node_index,
                                ),
                            )
                        } else {
                            Ok(
                                binding_result_spec(
                                    graph,
                                    schema,
                                    yaml_node_index,
                                    schema_node_index,
                                    None,
                                    Some(collection_index),
                                ),
                            )
                        }
                    },
                },
            }
        }
    }
}

#[verifier::rlimit(60)]
pub fn bind_profile1_typed_yaml_value(
    graph: &CanonicalYamlGraphSource,
    schema: &CompiledTypedFieldSchema,
    yaml_node_index: u64,
    schema_node_index: u64,
) -> (result: Result<TypedYamlValueBinding, TypedValueBindingError>)
    ensures
        bind_profile1_typed_yaml_value_spec(graph@, schema@, yaml_node_index, schema_node_index)
            == match result {
            Ok(binding) => Ok(binding@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = bind_profile1_typed_yaml_value_spec(
        graph@,
        schema@,
        yaml_node_index,
        schema_node_index,
    );
    let nodes = graph.nodes();
    if yaml_node_index >= nodes.len() as u64 {
        let error = TypedValueBindingError::at(
            TypedValueBindingErrorKind::YamlNodeIndexOutOfRange,
            graph.source_len_bytes(),
            yaml_node_index,
            schema_node_index,
        );
        proof {
            reveal(bind_profile1_typed_yaml_value_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let node = &nodes[yaml_node_index as usize];
    let schema_nodes = schema.schema().nodes();
    if schema_node_index >= schema_nodes.len() as u64 {
        let error = TypedValueBindingError::at(
            TypedValueBindingErrorKind::SchemaNodeIndexOutOfRange,
            node.byte_start(),
            yaml_node_index,
            schema_node_index,
        );
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(bind_profile1_typed_yaml_value_spec);
            reveal(binding_error_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let structural = graph.input().input().structural_keys();
    let table = structural.scalar_keys().graph().node_table();
    let scalars = table.scalars().scalars();
    let collections = table.collections();
    let invalid_common = node.source_node_index() != yaml_node_index || node.byte_start()
        > node.byte_end() || node.byte_end() > graph.source_len_bytes()
        || node.resolved_node_index() >= nodes.len() as u64 || schema.compilation_version()
        != crate::schema::TYPED_FIELD_SCHEMA_COMPILATION_VERSION || schema.schema_version()
        != schema.schema().schema_version() || schema.node_count() != schema_nodes.len() as u64
        || schema.field_count() != schema.schema().fields().len() as u64;
    if invalid_common {
        let error = TypedValueBindingError::at(
            TypedValueBindingErrorKind::InternalInvariantViolation,
            node.byte_start(),
            yaml_node_index,
            schema_node_index,
        );
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(crate::schema::typed_schema_node_views_spec);
            reveal(bind_profile1_typed_yaml_value_spec);
            reveal(binding_error_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let schema_kind = schema_nodes[schema_node_index as usize].kind();
    let binding = match node.kind() {
        CanonicalYamlNodeKind::Scalar => {
            let scalar_index = match node.scalar_index() {
                Some(index) => index,
                None => {
                    let error = TypedValueBindingError::at(
                        TypedValueBindingErrorKind::InternalInvariantViolation,
                        node.byte_start(),
                        yaml_node_index,
                        schema_node_index,
                    );
                    proof {
                        reveal(crate::lower::canonical_yaml_node_views_spec);
                        reveal(crate::schema::typed_schema_node_views_spec);
                        reveal(bind_profile1_typed_yaml_value_spec);
                        reveal(binding_error_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            if node.collection_index().is_some() || node.edge_start() != node.edge_end()
                || scalar_index >= scalars.len() as u64 {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            let scalar = &scalars[scalar_index as usize];
            if scalar.node_index() != node.resolved_node_index() {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
                    reveal(canonical_yaml_graph_scalars_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            if !typed_scalar_kind_matches(schema_kind, scalar.tag(), scalar.value()) {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::YamlValueKindMismatch,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
                    reveal(canonical_yaml_graph_scalars_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            TypedYamlValueBinding::new(
                graph.profile_version(),
                schema.schema_version(),
                yaml_node_index,
                node.resolved_node_index(),
                schema_node_index,
                schema_kind,
                node.byte_start(),
                node.byte_end(),
                Some(scalar_index),
                None,
            )
        },
        CanonicalYamlNodeKind::Sequence => {
            let collection_index = match node.collection_index() {
                Some(index) => index,
                None => {
                    let error = TypedValueBindingError::at(
                        TypedValueBindingErrorKind::InternalInvariantViolation,
                        node.byte_start(),
                        yaml_node_index,
                        schema_node_index,
                    );
                    proof {
                        reveal(crate::lower::canonical_yaml_node_views_spec);
                        reveal(crate::schema::typed_schema_node_views_spec);
                        reveal(bind_profile1_typed_yaml_value_spec);
                        reveal(binding_error_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            if node.scalar_index().is_some() || node.edge_start() > node.edge_end()
                || node.edge_end() > graph.sequence_entries().len() as u64 || collection_index
                >= collections.len() as u64 {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            let collection = &collections[collection_index as usize];
            if collection.node_index() != node.resolved_node_index() {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(crate::resolve_node_table::semantic_collection_views_spec);
                    reveal(canonical_yaml_graph_collections_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            if !typed_collection_kind_matches(schema_kind, node.kind(), collection.tag()) {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::YamlValueKindMismatch,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(crate::resolve_node_table::semantic_collection_views_spec);
                    reveal(canonical_yaml_graph_collections_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            TypedYamlValueBinding::new(
                graph.profile_version(),
                schema.schema_version(),
                yaml_node_index,
                node.resolved_node_index(),
                schema_node_index,
                schema_kind,
                node.byte_start(),
                node.byte_end(),
                None,
                Some(collection_index),
            )
        },
        CanonicalYamlNodeKind::Mapping => {
            let collection_index = match node.collection_index() {
                Some(index) => index,
                None => {
                    let error = TypedValueBindingError::at(
                        TypedValueBindingErrorKind::InternalInvariantViolation,
                        node.byte_start(),
                        yaml_node_index,
                        schema_node_index,
                    );
                    proof {
                        reveal(crate::lower::canonical_yaml_node_views_spec);
                        reveal(crate::schema::typed_schema_node_views_spec);
                        reveal(bind_profile1_typed_yaml_value_spec);
                        reveal(binding_error_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            if node.scalar_index().is_some() || node.edge_start() > node.edge_end()
                || node.edge_end() > graph.mapping_entries().len() as u64 || collection_index
                >= collections.len() as u64 {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            let collection = &collections[collection_index as usize];
            if collection.node_index() != node.resolved_node_index() {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::InternalInvariantViolation,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(crate::resolve_node_table::semantic_collection_views_spec);
                    reveal(canonical_yaml_graph_collections_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            if !typed_collection_kind_matches(schema_kind, node.kind(), collection.tag()) {
                let error = TypedValueBindingError::at(
                    TypedValueBindingErrorKind::YamlValueKindMismatch,
                    node.byte_start(),
                    yaml_node_index,
                    schema_node_index,
                );
                proof {
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(crate::schema::typed_schema_node_views_spec);
                    reveal(crate::resolve_node_table::semantic_collection_views_spec);
                    reveal(canonical_yaml_graph_collections_spec);
                    reveal(bind_profile1_typed_yaml_value_spec);
                    reveal(binding_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            TypedYamlValueBinding::new(
                graph.profile_version(),
                schema.schema_version(),
                yaml_node_index,
                node.resolved_node_index(),
                schema_node_index,
                schema_kind,
                node.byte_start(),
                node.byte_end(),
                None,
                Some(collection_index),
            )
        },
    };
    proof {
        reveal(crate::lower::canonical_yaml_node_views_spec);
        reveal(crate::schema::typed_schema_node_views_spec);
        reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
        reveal(crate::resolve_node_table::semantic_collection_views_spec);
        reveal(canonical_yaml_graph_scalars_spec);
        reveal(canonical_yaml_graph_collections_spec);
        reveal(bind_profile1_typed_yaml_value_spec);
        reveal(binding_result_spec);
        assert(expected == Ok(binding@));
    }
    Ok(binding)
}

pub open spec fn typed_yaml_value_binding_well_formed_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
    output: TypedYamlValueBindingView,
) -> bool {
    bind_profile1_typed_yaml_value_spec(graph, schema, yaml_node_index, schema_node_index) == Ok(
        output,
    )
}

pub proof fn lemma_typed_yaml_value_binding_success_is_well_formed(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
    output: TypedYamlValueBindingView,
)
    requires
        bind_profile1_typed_yaml_value_spec(graph, schema, yaml_node_index, schema_node_index)
            == Ok(output),
    ensures
        typed_yaml_value_binding_well_formed_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
            output,
        ),
{
    reveal(typed_yaml_value_binding_well_formed_spec);
}

pub proof fn lemma_authenticated_typed_yaml_value_binding_is_unique(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
    first: TypedYamlValueBindingView,
    second: TypedYamlValueBindingView,
)
    requires
        typed_yaml_value_binding_well_formed_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
            first,
        ),
        typed_yaml_value_binding_well_formed_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
            second,
        ),
    ensures
        first == second,
{
    reveal(typed_yaml_value_binding_well_formed_spec);
}

} // verus!
