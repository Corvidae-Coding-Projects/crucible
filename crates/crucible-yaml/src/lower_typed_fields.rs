//! Verified recognized/unknown mapping-field partition and required-field validation.
//!
//! This is the mapping submachine for graph-wide schema-directed lowering. It authenticates one
//! canonical YAML mapping against one compiled mapping-schema node, rejects non-string and unknown
//! keys without coercion, validates every recognized value through the typed-value binder, rejects
//! missing required fields, and emits records in schema order independent of YAML presentation
//! order.
use crate::lower::CanonicalYamlGraphSource;
#[allow(unused_imports)]
use crate::lower::CanonicalYamlGraphSourceView;
#[allow(unused_imports)]
use crate::lower_typed::TypedYamlValueBindingView;
use crate::lower_typed::{
    bind_profile1_typed_yaml_value, TypedValueBindingErrorKind, TypedYamlValueBinding,
};
use crate::resolve_scalar_value::{ResolvedScalarTag, ResolvedScalarValue};
#[allow(unused_imports)]
use crate::resolve_scalar_value::{ResolvedScalarValueView, ResolvedScalarView};
use crate::scalar_decode::DecodedContentScalar;
#[allow(unused_imports)]
use crate::scalar_decode::DecodedContentScalarView;
use crate::schema::{CompiledTypedFieldSchema, TypedFieldDefinition, TypedSchemaValueKind};
#[allow(unused_imports)]
use crate::schema::{CompiledTypedFieldSchemaView, TypedFieldDefinitionView};
use vstd::prelude::*;

verus! {

pub const TYPED_MAPPING_FIELD_PARTITION_VERSION: u16 = 1;

pub const MAX_PROFILE1_TYPED_MAPPING_FIELDS: u64 = crate::cst::MAX_PROFILE1_CST_MAPPING_ENTRIES;

pub const MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS: u64 =
    crate::scalar_decode::MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum TypedMappingUnknownFieldPolicy {
    Reject,
    Preserve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedMappingFieldLimits {
    max_fields: u64,
    max_key_code_points: u64,
}

#[verifier::ext_equal]
pub struct TypedMappingFieldLimitsView {
    pub max_fields: u64,
    pub max_key_code_points: u64,
}

impl View for TypedMappingFieldLimits {
    type V = TypedMappingFieldLimitsView;

    closed spec fn view(&self) -> TypedMappingFieldLimitsView {
        TypedMappingFieldLimitsView {
            max_fields: self.max_fields,
            max_key_code_points: self.max_key_code_points,
        }
    }
}

impl TypedMappingFieldLimits {
    pub fn new(max_fields: u64, max_key_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (TypedMappingFieldLimitsView { max_fields, max_key_code_points }),
    {
        Self { max_fields, max_key_code_points }
    }

    pub fn max_fields(&self) -> (value: u64)
        ensures
            value == self@.max_fields,
    {
        self.max_fields
    }

    pub fn max_key_code_points(&self) -> (value: u64)
        ensures
            value == self@.max_key_code_points,
    {
        self.max_key_code_points
    }
}

pub open spec fn canonical_typed_mapping_field_limits_spec() -> TypedMappingFieldLimitsView {
    TypedMappingFieldLimitsView {
        max_fields: MAX_PROFILE1_TYPED_MAPPING_FIELDS,
        max_key_code_points: MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
    }
}

pub fn canonical_typed_mapping_field_limits() -> (limits: TypedMappingFieldLimits)
    ensures
        limits@ == canonical_typed_mapping_field_limits_spec(),
{
    TypedMappingFieldLimits::new(
        MAX_PROFILE1_TYPED_MAPPING_FIELDS,
        MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
    )
}

pub open spec fn typed_mapping_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (value: u64)
    ensures
        value == typed_mapping_effective_limit_spec(requested, absolute),
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
pub enum TypedMappingFieldErrorKind {
    MappingKindMismatch,
    MappingKeyNotString,
    UnknownField,
    DuplicateRecognizedField,
    MissingRequiredField,
    ValueKindMismatch,
    FieldLimitExceeded,
    KeyCodePointLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedMappingFieldError {
    kind: TypedMappingFieldErrorKind,
    byte_offset: u64,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    mapping_entry_index: Option<u64>,
    schema_field_index: Option<u64>,
    key_code_point_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct TypedMappingFieldErrorView {
    pub kind: TypedMappingFieldErrorKind,
    pub byte_offset: u64,
    pub yaml_mapping_node_index: u64,
    pub schema_mapping_node_index: u64,
    pub mapping_entry_index: Option<u64>,
    pub schema_field_index: Option<u64>,
    pub key_code_point_index: Option<u64>,
}

impl View for TypedMappingFieldError {
    type V = TypedMappingFieldErrorView;

    closed spec fn view(&self) -> TypedMappingFieldErrorView {
        TypedMappingFieldErrorView {
            kind: self.kind,
            byte_offset: self.byte_offset,
            yaml_mapping_node_index: self.yaml_mapping_node_index,
            schema_mapping_node_index: self.schema_mapping_node_index,
            mapping_entry_index: self.mapping_entry_index,
            schema_field_index: self.schema_field_index,
            key_code_point_index: self.key_code_point_index,
        }
    }
}

impl TypedMappingFieldError {
    #[allow(clippy::too_many_arguments)]
    fn at(
        kind: TypedMappingFieldErrorKind,
        byte_offset: u64,
        yaml_mapping_node_index: u64,
        schema_mapping_node_index: u64,
        mapping_entry_index: Option<u64>,
        schema_field_index: Option<u64>,
        key_code_point_index: Option<u64>,
    ) -> (error: Self)
        ensures
            error@ == (TypedMappingFieldErrorView {
                kind,
                byte_offset,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                mapping_entry_index,
                schema_field_index,
                key_code_point_index,
            }),
    {
        Self {
            kind,
            byte_offset,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            mapping_entry_index,
            schema_field_index,
            key_code_point_index,
        }
    }

    pub fn kind(&self) -> (value: TypedMappingFieldErrorKind)
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

    pub fn yaml_mapping_node_index(&self) -> (value: u64)
        ensures
            value == self@.yaml_mapping_node_index,
    {
        self.yaml_mapping_node_index
    }

    pub fn schema_mapping_node_index(&self) -> (value: u64)
        ensures
            value == self@.schema_mapping_node_index,
    {
        self.schema_mapping_node_index
    }

    pub fn mapping_entry_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.mapping_entry_index,
    {
        self.mapping_entry_index
    }

    pub fn schema_field_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.schema_field_index,
    {
        self.schema_field_index
    }

    pub fn key_code_point_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.key_code_point_index,
    {
        self.key_code_point_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedMappingField {
    mapping_entry_index: u64,
    schema_field_index: u64,
    field_id: u64,
    key_yaml_node_index: u64,
    value_yaml_node_index: u64,
    inherited: bool,
    binding: TypedYamlValueBinding,
}

#[verifier::ext_equal]
pub struct TypedMappingFieldView {
    pub mapping_entry_index: u64,
    pub schema_field_index: u64,
    pub field_id: u64,
    pub key_yaml_node_index: u64,
    pub value_yaml_node_index: u64,
    pub inherited: bool,
    pub binding: TypedYamlValueBindingView,
}

impl View for TypedMappingField {
    type V = TypedMappingFieldView;

    closed spec fn view(&self) -> TypedMappingFieldView {
        TypedMappingFieldView {
            mapping_entry_index: self.mapping_entry_index,
            schema_field_index: self.schema_field_index,
            field_id: self.field_id,
            key_yaml_node_index: self.key_yaml_node_index,
            value_yaml_node_index: self.value_yaml_node_index,
            inherited: self.inherited,
            binding: self.binding@,
        }
    }
}

impl TypedMappingField {
    #[allow(clippy::too_many_arguments)]
    fn new(
        mapping_entry_index: u64,
        schema_field_index: u64,
        field_id: u64,
        key_yaml_node_index: u64,
        value_yaml_node_index: u64,
        inherited: bool,
        binding: TypedYamlValueBinding,
    ) -> (field: Self)
        ensures
            field@ == (TypedMappingFieldView {
                mapping_entry_index,
                schema_field_index,
                field_id,
                key_yaml_node_index,
                value_yaml_node_index,
                inherited,
                binding: binding@,
            }),
    {
        Self {
            mapping_entry_index,
            schema_field_index,
            field_id,
            key_yaml_node_index,
            value_yaml_node_index,
            inherited,
            binding,
        }
    }

    pub fn mapping_entry_index(&self) -> (value: u64)
        ensures
            value == self@.mapping_entry_index,
    {
        self.mapping_entry_index
    }

    pub fn schema_field_index(&self) -> (value: u64)
        ensures
            value == self@.schema_field_index,
    {
        self.schema_field_index
    }

    pub fn field_id(&self) -> (value: u64)
        ensures
            value == self@.field_id,
    {
        self.field_id
    }

    pub fn key_yaml_node_index(&self) -> (value: u64)
        ensures
            value == self@.key_yaml_node_index,
    {
        self.key_yaml_node_index
    }

    pub fn value_yaml_node_index(&self) -> (value: u64)
        ensures
            value == self@.value_yaml_node_index,
    {
        self.value_yaml_node_index
    }

    pub fn inherited(&self) -> (value: bool)
        ensures
            value == self@.inherited,
    {
        self.inherited
    }

    pub fn binding(&self) -> (value: &TypedYamlValueBinding)
        ensures
            value@ == self@.binding,
    {
        &self.binding
    }
}

pub open spec fn typed_mapping_field_views_spec(values: Seq<TypedMappingField>) -> Seq<
    TypedMappingFieldView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypedMappingUnknownField {
    mapping_entry_index: u64,
    key_yaml_node_index: u64,
    value_yaml_node_index: u64,
    inherited: bool,
    byte_start: u64,
    byte_end: u64,
    key_code_points: Vec<u32>,
}

#[verifier::ext_equal]
pub struct TypedMappingUnknownFieldView {
    pub mapping_entry_index: u64,
    pub key_yaml_node_index: u64,
    pub value_yaml_node_index: u64,
    pub inherited: bool,
    pub byte_start: u64,
    pub byte_end: u64,
    pub key_code_points: Seq<u32>,
}

impl View for TypedMappingUnknownField {
    type V = TypedMappingUnknownFieldView;

    closed spec fn view(&self) -> TypedMappingUnknownFieldView {
        TypedMappingUnknownFieldView {
            mapping_entry_index: self.mapping_entry_index,
            key_yaml_node_index: self.key_yaml_node_index,
            value_yaml_node_index: self.value_yaml_node_index,
            inherited: self.inherited,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            key_code_points: self.key_code_points@,
        }
    }
}

impl TypedMappingUnknownField {
    #[allow(clippy::too_many_arguments)]
    fn new(
        mapping_entry_index: u64,
        key_yaml_node_index: u64,
        value_yaml_node_index: u64,
        inherited: bool,
        byte_start: u64,
        byte_end: u64,
        key_code_points: Vec<u32>,
    ) -> (field: Self)
        ensures
            field@ == (TypedMappingUnknownFieldView {
                mapping_entry_index,
                key_yaml_node_index,
                value_yaml_node_index,
                inherited,
                byte_start,
                byte_end,
                key_code_points: key_code_points@,
            }),
    {
        Self {
            mapping_entry_index,
            key_yaml_node_index,
            value_yaml_node_index,
            inherited,
            byte_start,
            byte_end,
            key_code_points,
        }
    }

    pub fn mapping_entry_index(&self) -> u64 {
        self.mapping_entry_index
    }

    pub fn key_yaml_node_index(&self) -> u64 {
        self.key_yaml_node_index
    }

    pub fn value_yaml_node_index(&self) -> u64 {
        self.value_yaml_node_index
    }

    pub fn inherited(&self) -> bool {
        self.inherited
    }

    pub fn byte_start(&self) -> u64 {
        self.byte_start
    }

    pub fn byte_end(&self) -> u64 {
        self.byte_end
    }

    pub fn key_code_points(&self) -> &[u32] {
        self.key_code_points.as_slice()
    }
}

pub open spec fn typed_mapping_unknown_field_views_spec(
    values: Seq<TypedMappingUnknownField>,
) -> Seq<TypedMappingUnknownFieldView> {
    Seq::new(values.len(), |index: int| values[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypedMappingFieldPartition {
    transformation_version: u16,
    canonical_profile_version: u16,
    schema_version: u16,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    unknown_field_policy: TypedMappingUnknownFieldPolicy,
    mapping_binding: TypedYamlValueBinding,
    total_key_code_points: u64,
    fields: Vec<TypedMappingField>,
    unknown_fields: Vec<TypedMappingUnknownField>,
}

#[verifier::ext_equal]
pub struct TypedMappingFieldPartitionView {
    pub transformation_version: u16,
    pub canonical_profile_version: u16,
    pub schema_version: u16,
    pub yaml_mapping_node_index: u64,
    pub schema_mapping_node_index: u64,
    pub unknown_field_policy: TypedMappingUnknownFieldPolicy,
    pub mapping_binding: TypedYamlValueBindingView,
    pub total_key_code_points: u64,
    pub fields: Seq<TypedMappingFieldView>,
    pub unknown_fields: Seq<TypedMappingUnknownFieldView>,
}

impl View for TypedMappingFieldPartition {
    type V = TypedMappingFieldPartitionView;

    closed spec fn view(&self) -> TypedMappingFieldPartitionView {
        TypedMappingFieldPartitionView {
            transformation_version: self.transformation_version,
            canonical_profile_version: self.canonical_profile_version,
            schema_version: self.schema_version,
            yaml_mapping_node_index: self.yaml_mapping_node_index,
            schema_mapping_node_index: self.schema_mapping_node_index,
            unknown_field_policy: self.unknown_field_policy,
            mapping_binding: self.mapping_binding@,
            total_key_code_points: self.total_key_code_points,
            fields: typed_mapping_field_views_spec(self.fields@),
            unknown_fields: typed_mapping_unknown_field_views_spec(self.unknown_fields@),
        }
    }
}

impl TypedMappingFieldPartition {
    #[allow(clippy::too_many_arguments)]
    fn new(
        canonical_profile_version: u16,
        schema_version: u16,
        yaml_mapping_node_index: u64,
        schema_mapping_node_index: u64,
        unknown_field_policy: TypedMappingUnknownFieldPolicy,
        mapping_binding: TypedYamlValueBinding,
        total_key_code_points: u64,
        fields: Vec<TypedMappingField>,
        unknown_fields: Vec<TypedMappingUnknownField>,
    ) -> (partition: Self)
        ensures
            partition@ == (TypedMappingFieldPartitionView {
                transformation_version: TYPED_MAPPING_FIELD_PARTITION_VERSION,
                canonical_profile_version,
                schema_version,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                unknown_field_policy,
                mapping_binding: mapping_binding@,
                total_key_code_points,
                fields: typed_mapping_field_views_spec(fields@),
                unknown_fields: typed_mapping_unknown_field_views_spec(unknown_fields@),
            }),
    {
        Self {
            transformation_version: TYPED_MAPPING_FIELD_PARTITION_VERSION,
            canonical_profile_version,
            schema_version,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            mapping_binding,
            total_key_code_points,
            fields,
            unknown_fields,
        }
    }

    pub fn transformation_version(&self) -> (value: u16)
        ensures
            value == self@.transformation_version,
    {
        self.transformation_version
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

    pub fn yaml_mapping_node_index(&self) -> (value: u64)
        ensures
            value == self@.yaml_mapping_node_index,
    {
        self.yaml_mapping_node_index
    }

    pub fn schema_mapping_node_index(&self) -> (value: u64)
        ensures
            value == self@.schema_mapping_node_index,
    {
        self.schema_mapping_node_index
    }

    pub fn unknown_field_policy(&self) -> TypedMappingUnknownFieldPolicy {
        self.unknown_field_policy
    }

    pub fn mapping_binding(&self) -> (value: &TypedYamlValueBinding)
        ensures
            value@ == self@.mapping_binding,
    {
        &self.mapping_binding
    }

    pub fn total_key_code_points(&self) -> (value: u64)
        ensures
            value == self@.total_key_code_points,
    {
        self.total_key_code_points
    }

    pub fn fields(&self) -> (values: &[TypedMappingField])
        ensures
            typed_mapping_field_views_spec(values@) == self@.fields,
    {
        self.fields.as_slice()
    }

    pub fn unknown_fields(&self) -> (values: &[TypedMappingUnknownField])
        ensures
            typed_mapping_unknown_field_views_spec(values@) == self@.unknown_fields,
    {
        self.unknown_fields.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MappingKeyCodePoint {
    code_point: u32,
    byte_start: u64,
}

#[verifier::ext_equal]
#[allow(dead_code)]
pub struct MappingKeyCodePointView {
    pub code_point: u32,
    pub byte_start: u64,
}

impl View for MappingKeyCodePoint {
    type V = MappingKeyCodePointView;

    closed spec fn view(&self) -> MappingKeyCodePointView {
        MappingKeyCodePointView { code_point: self.code_point, byte_start: self.byte_start }
    }
}

spec fn mapping_key_code_point_views_spec(values: Seq<MappingKeyCodePoint>) -> Seq<
    MappingKeyCodePointView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

spec fn decoded_mapping_key_spec(content: Seq<DecodedContentScalarView>) -> Seq<
    MappingKeyCodePointView,
> {
    Seq::new(
        content.len(),
        |index: int|
            MappingKeyCodePointView {
                code_point: content[index].code_point,
                byte_start: content[index].byte_start,
            },
    )
}

pub closed spec fn mapping_key_for_node_spec(
    graph: CanonicalYamlGraphSourceView,
    key_node_index: u64,
) -> Option<Seq<MappingKeyCodePointView>> {
    if key_node_index >= graph.nodes.len() {
        None
    } else {
        let node = graph.nodes[key_node_index as int];
        match node.scalar_index {
            Some(scalar_index) => {
                let scalars = crate::lower_typed::canonical_yaml_graph_scalars_spec(graph);
                if node.kind != crate::lower::CanonicalYamlNodeKind::Scalar || scalar_index
                    >= scalars.len() {
                    None
                } else {
                    let scalar = scalars[scalar_index as int];
                    if scalar.node_index != node.resolved_node_index || scalar.tag
                        != ResolvedScalarTag::CoreString || scalar.value
                        != ResolvedScalarValueView::String {
                        None
                    } else {
                        match scalar.presentation.decoded {
                            Some(decoded) => Some(decoded_mapping_key_spec(decoded.content)),
                            None => None,
                        }
                    }
                }
            },
            None => None,
        }
    }
}

fn append_mapping_key_code_points(content: &[DecodedContentScalar]) -> (points: Vec<
    MappingKeyCodePoint,
>)
    ensures
        mapping_key_code_point_views_spec(points@) == decoded_mapping_key_spec(
            crate::scalar_decode::decoded_content_scalar_views_spec(content@),
        ),
{
    let mut points = Vec::new();
    let mut index = 0;
    while index < content.len()
        invariant
            index <= content.len(),
            mapping_key_code_point_views_spec(points@) == decoded_mapping_key_spec(
                crate::scalar_decode::decoded_content_scalar_views_spec(content@),
            ).take(index as int),
        decreases content.len() - index,
    {
        let point = MappingKeyCodePoint {
            code_point: content[index].code_point(),
            byte_start: content[index].byte_start(),
        };
        let ghost before = points@;
        points.push(point);
        proof {
            reveal(mapping_key_code_point_views_spec);
            reveal(decoded_mapping_key_spec);
            crate::scalar_decode::decoded_content_scalar_views_spec(content@).lemma_take_succ_push(
                index as int,
            );
            assert(mapping_key_code_point_views_spec(points@) == mapping_key_code_point_views_spec(
                before,
            ).push(point@));
        }
        index += 1;
    }
    points
}

#[allow(clippy::match_like_matches_macro)]
fn mapping_key_for_node(graph: &CanonicalYamlGraphSource, key_node_index: u64) -> (result: Option<
    Vec<MappingKeyCodePoint>,
>)
    ensures
        match result {
            Some(points) => mapping_key_for_node_spec(graph@, key_node_index) == Some(
                mapping_key_code_point_views_spec(points@),
            ),
            None => mapping_key_for_node_spec(graph@, key_node_index).is_none(),
        },
{
    let nodes = graph.nodes();
    if key_node_index >= nodes.len() as u64 {
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(mapping_key_for_node_spec);
        }
        return None;
    }
    let node = &nodes[key_node_index as usize];
    let scalar_index = match node.scalar_index() {
        Some(index) => index,
        None => {
            proof {
                reveal(crate::lower::canonical_yaml_node_views_spec);
                reveal(mapping_key_for_node_spec);
            }
            return None;
        },
    };
    let table = graph.input().input().structural_keys().scalar_keys().graph().node_table();
    let scalars = table.scalars().scalars();
    if node.kind() != crate::lower::CanonicalYamlNodeKind::Scalar || scalar_index
        >= scalars.len() as u64 {
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
            reveal(crate::lower_typed::canonical_yaml_graph_scalars_spec);
            reveal(mapping_key_for_node_spec);
        }
        return None;
    }
    let scalar = &scalars[scalar_index as usize];
    let value_is_string = match scalar.value() {
        ResolvedScalarValue::String => true,
        _ => false,
    };
    if scalar.node_index() != node.resolved_node_index() || scalar.tag()
        != ResolvedScalarTag::CoreString || !value_is_string {
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
            reveal(crate::lower_typed::canonical_yaml_graph_scalars_spec);
            reveal(mapping_key_for_node_spec);
        }
        return None;
    }
    let decoded = match scalar.presentation().decoded() {
        Some(decoded) => decoded,
        None => {
            proof {
                reveal(crate::lower::canonical_yaml_node_views_spec);
                reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
                reveal(crate::lower_typed::canonical_yaml_graph_scalars_spec);
                reveal(mapping_key_for_node_spec);
            }
            return None;
        },
    };
    let points = append_mapping_key_code_points(decoded.content());
    proof {
        reveal(crate::lower::canonical_yaml_node_views_spec);
        reveal(crate::resolve_scalar_table::semantic_scalar_views_spec);
        reveal(crate::lower_typed::canonical_yaml_graph_scalars_spec);
        reveal(mapping_key_for_node_spec);
        reveal(decoded_mapping_key_spec);
        assert(mapping_key_for_node_spec(graph@, key_node_index) == Some(
            mapping_key_code_point_views_spec(points@),
        ));
    }
    Some(points)
}

pub open spec fn mapping_key_code_points_spec(points: Seq<MappingKeyCodePointView>) -> Seq<u32> {
    Seq::new(points.len(), |index: int| points[index].code_point)
}

fn copy_mapping_key_code_points(points: &[MappingKeyCodePoint]) -> (code_points: Vec<u32>)
    ensures
        code_points@ == mapping_key_code_points_spec(mapping_key_code_point_views_spec(points@)),
{
    let mut code_points = Vec::new();
    let mut index = 0;
    while index < points.len()
        invariant
            index <= points.len(),
            code_points@ == mapping_key_code_points_spec(
                mapping_key_code_point_views_spec(points@),
            ).take(index as int),
        decreases points.len() - index,
    {
        let ghost before = code_points@;
        code_points.push(points[index].code_point);
        proof {
            reveal(mapping_key_code_points_spec);
            reveal(mapping_key_code_point_views_spec);
            mapping_key_code_points_spec(
                mapping_key_code_point_views_spec(points@),
            ).lemma_take_succ_push(index as int);
            assert(code_points@ == before.push(points[index as int]@.code_point));
        }
        index += 1;
    }
    code_points
}

spec fn key_matches_name_spec(points: Seq<MappingKeyCodePointView>, name: Seq<u32>) -> bool {
    mapping_key_code_points_spec(points) == name
}

fn key_matches_name(points: &[MappingKeyCodePoint], name: &[u32]) -> (matches: bool)
    ensures
        matches == key_matches_name_spec(mapping_key_code_point_views_spec(points@), name@),
{
    if points.len() != name.len() {
        return false;
    }
    let mut index = 0;
    while index < points.len()
        invariant
            index <= points.len(),
            points.len() == name.len(),
            forall|prior: int|
                0 <= prior < index ==> mapping_key_code_point_views_spec(points@)[prior].code_point
                    == name@[prior],
        decreases points.len() - index,
    {
        if points[index].code_point != name[index] {
            return false;
        }
        index += 1;
    }
    assert(mapping_key_code_points_spec(mapping_key_code_point_views_spec(points@)) == name@);
    true
}

pub closed spec fn find_schema_field_spec(
    fields: Seq<TypedFieldDefinitionView>,
    start: nat,
    end: nat,
    key: Seq<MappingKeyCodePointView>,
) -> Option<u64>
    decreases end - start,
{
    if start >= end || end > fields.len() {
        None
    } else if key_matches_name_spec(key, fields[start as int].name) {
        Some(start as u64)
    } else {
        find_schema_field_spec(fields, start + 1, end, key)
    }
}

fn find_schema_field(
    fields: &[TypedFieldDefinition],
    start: usize,
    end: usize,
    key: &[MappingKeyCodePoint],
) -> (found: Option<u64>)
    requires
        start <= end <= fields.len(),
    ensures
        found == find_schema_field_spec(
            crate::schema::typed_field_definition_views_spec(fields@),
            start as nat,
            end as nat,
            mapping_key_code_point_views_spec(key@),
        ),
        match found {
            Some(index) => start <= index < end,
            None => true,
        },
{
    let ghost all_fields = crate::schema::typed_field_definition_views_spec(fields@);
    let ghost key_view = mapping_key_code_point_views_spec(key@);
    let ghost expected = find_schema_field_spec(all_fields, start as nat, end as nat, key_view);
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= fields.len(),
            all_fields == crate::schema::typed_field_definition_views_spec(fields@),
            key_view == mapping_key_code_point_views_spec(key@),
            expected == find_schema_field_spec(
                crate::schema::typed_field_definition_views_spec(fields@),
                start as nat,
                end as nat,
                mapping_key_code_point_views_spec(key@),
            ),
            expected == find_schema_field_spec(all_fields, index as nat, end as nat, key_view),
        decreases end - index,
    {
        if key_matches_name(key, fields[index].name()) {
            proof {
                assert(0 <= (index as int) < fields@.len());
                reveal(find_schema_field_spec);
                reveal(crate::schema::typed_field_definition_views_spec);
                assert(all_fields[index as int] == fields[index as int]@);
                assert(key_matches_name_spec(key_view, fields[index as int]@.name));
                assert(key_matches_name_spec(key_view, all_fields[index as int].name));
                assert(expected == Some(index as u64));
                assert(expected == find_schema_field_spec(
                    crate::schema::typed_field_definition_views_spec(fields@),
                    start as nat,
                    end as nat,
                    mapping_key_code_point_views_spec(key@),
                ));
                assert(find_schema_field_spec(
                    crate::schema::typed_field_definition_views_spec(fields@),
                    start as nat,
                    end as nat,
                    mapping_key_code_point_views_spec(key@),
                ) == Some(index as u64));
            }
            return Some(index as u64);
        }
        proof {
            reveal(find_schema_field_spec);
            reveal(crate::schema::typed_field_definition_views_spec);
        }
        index += 1;
    }
    proof {
        reveal(find_schema_field_spec);
        assert(expected.is_none());
    }
    None
}

spec fn candidate_for_schema_field_from_spec(
    candidates: Seq<TypedMappingFieldView>,
    candidate_index: nat,
    schema_field_index: u64,
) -> Option<TypedMappingFieldView>
    decreases candidates.len() - candidate_index,
{
    if candidate_index >= candidates.len() {
        None
    } else if candidates[candidate_index as int].schema_field_index == schema_field_index {
        Some(candidates[candidate_index as int])
    } else {
        candidate_for_schema_field_from_spec(candidates, candidate_index + 1, schema_field_index)
    }
}

pub closed spec fn candidate_for_schema_field_spec(
    candidates: Seq<TypedMappingFieldView>,
    schema_field_index: u64,
) -> Option<TypedMappingFieldView> {
    candidate_for_schema_field_from_spec(candidates, 0, schema_field_index)
}

fn candidate_for_schema_field(candidates: &[TypedMappingField], schema_field_index: u64) -> (found:
    Option<TypedMappingField>)
    ensures
        match found {
            Some(field) => candidate_for_schema_field_spec(
                typed_mapping_field_views_spec(candidates@),
                schema_field_index,
            ) == Some(field@),
            None => candidate_for_schema_field_spec(
                typed_mapping_field_views_spec(candidates@),
                schema_field_index,
            ).is_none(),
        },
{
    let ghost all_candidates = typed_mapping_field_views_spec(candidates@);
    let ghost expected = candidate_for_schema_field_spec(all_candidates, schema_field_index);
    let mut index = 0;
    while index < candidates.len()
        invariant
            index <= candidates.len(),
            all_candidates == typed_mapping_field_views_spec(candidates@),
            expected == candidate_for_schema_field_spec(
                typed_mapping_field_views_spec(candidates@),
                schema_field_index,
            ),
            expected == candidate_for_schema_field_from_spec(
                all_candidates,
                index as nat,
                schema_field_index,
            ),
        decreases candidates.len() - index,
    {
        if candidates[index].schema_field_index() == schema_field_index {
            proof {
                assert(0 <= (index as int) < candidates@.len());
                reveal(candidate_for_schema_field_spec);
                reveal(candidate_for_schema_field_from_spec);
                reveal(typed_mapping_field_views_spec);
                assert(expected == Some(candidates[index as int]@));
                assert(expected == candidate_for_schema_field_spec(
                    typed_mapping_field_views_spec(candidates@),
                    schema_field_index,
                ));
                assert(candidate_for_schema_field_spec(
                    typed_mapping_field_views_spec(candidates@),
                    schema_field_index,
                ) == Some(candidates[index as int]@));
            }
            return Some(candidates[index]);
        }
        proof {
            reveal(candidate_for_schema_field_from_spec);
        }
        index += 1;
    }
    proof {
        reveal(candidate_for_schema_field_spec);
        reveal(candidate_for_schema_field_from_spec);
        assert(expected.is_none());
    }
    None
}

pub open spec fn typed_mapping_error_spec(
    kind: TypedMappingFieldErrorKind,
    byte_offset: u64,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    mapping_entry_index: Option<u64>,
    schema_field_index: Option<u64>,
    key_code_point_index: Option<u64>,
) -> TypedMappingFieldErrorView {
    TypedMappingFieldErrorView {
        kind,
        byte_offset,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        mapping_entry_index,
        schema_field_index,
        key_code_point_index,
    }
}

pub open spec fn typed_schema_kind_is_mapping_spec(kind: TypedSchemaValueKind) -> bool {
    match kind {
        TypedSchemaValueKind::Mapping | TypedSchemaValueKind::CustomMapping => true,
        _ => false,
    }
}

#[allow(clippy::match_like_matches_macro)]
fn typed_schema_kind_is_mapping(kind: TypedSchemaValueKind) -> (is_mapping: bool)
    ensures
        is_mapping == typed_schema_kind_is_mapping_spec(kind),
{
    match kind {
        TypedSchemaValueKind::Mapping | TypedSchemaValueKind::CustomMapping => true,
        _ => false,
    }
}

pub open spec fn scan_mapping_entries_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    entry_index: nat,
    entry_end: nat,
    schema_field_start: nat,
    schema_field_end: nat,
    field_limit: u64,
    key_limit: u64,
    unknown_field_policy: TypedMappingUnknownFieldPolicy,
    candidates: Seq<TypedMappingFieldView>,
    unknown_fields: Seq<TypedMappingUnknownFieldView>,
    total_key_code_points: u64,
) -> Result<
    (Seq<TypedMappingFieldView>, Seq<TypedMappingUnknownFieldView>, u64),
    TypedMappingFieldErrorView,
>
    decreases entry_end - entry_index,
{
    if entry_index >= entry_end || entry_end > graph.mapping_entries.len() {
        if entry_index == entry_end && entry_end <= graph.mapping_entries.len() {
            Ok((candidates, unknown_fields, total_key_code_points))
        } else {
            Err(
                typed_mapping_error_spec(
                    TypedMappingFieldErrorKind::InternalInvariantViolation,
                    graph.nodes[yaml_mapping_node_index as int].byte_start,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    Some(entry_index as u64),
                    None,
                    None,
                ),
            )
        }
    } else {
        let entry = graph.mapping_entries[entry_index as int];
        let key_start = if entry.key_node_index < graph.nodes.len() {
            graph.nodes[entry.key_node_index as int].byte_start
        } else {
            graph.source_len_bytes
        };
        if entry.receiver_node_index != yaml_mapping_node_index || entry.key_node_index
            >= graph.nodes.len() || entry.value_node_index >= graph.nodes.len() {
            Err(
                typed_mapping_error_spec(
                    TypedMappingFieldErrorKind::InternalInvariantViolation,
                    key_start,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    Some(entry_index as u64),
                    None,
                    None,
                ),
            )
        } else {
            match mapping_key_for_node_spec(graph, entry.key_node_index) {
                None => Err(
                    typed_mapping_error_spec(
                        TypedMappingFieldErrorKind::MappingKeyNotString,
                        key_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        Some(entry_index as u64),
                        None,
                        None,
                    ),
                ),
                Some(key) => match find_schema_field_spec(
                    schema.schema.fields,
                    schema_field_start,
                    schema_field_end,
                    key,
                ) {
                    None => if unknown_field_policy == TypedMappingUnknownFieldPolicy::Reject {
                        Err(
                            typed_mapping_error_spec(
                                TypedMappingFieldErrorKind::UnknownField,
                                key_start,
                                yaml_mapping_node_index,
                                schema_mapping_node_index,
                                Some(entry_index as u64),
                                None,
                                None,
                            ),
                        )
                    } else if candidates.len() + unknown_fields.len() >= field_limit {
                        Err(
                            typed_mapping_error_spec(
                                TypedMappingFieldErrorKind::FieldLimitExceeded,
                                key_start,
                                yaml_mapping_node_index,
                                schema_mapping_node_index,
                                Some(entry_index as u64),
                                None,
                                None,
                            ),
                        )
                    } else if key.len() > key_limit - total_key_code_points {
                        let excluded = (key_limit - total_key_code_points) as nat;
                        Err(
                            typed_mapping_error_spec(
                                TypedMappingFieldErrorKind::KeyCodePointLimitExceeded,
                                key[excluded as int].byte_start,
                                yaml_mapping_node_index,
                                schema_mapping_node_index,
                                Some(entry_index as u64),
                                None,
                                Some(excluded as u64),
                            ),
                        )
                    } else {
                        let unknown = TypedMappingUnknownFieldView {
                            mapping_entry_index: entry_index as u64,
                            key_yaml_node_index: entry.key_node_index,
                            value_yaml_node_index: entry.value_node_index,
                            inherited: entry.inherited,
                            byte_start: graph.nodes[entry.key_node_index as int].byte_start,
                            byte_end: graph.nodes[entry.key_node_index as int].byte_end,
                            key_code_points: mapping_key_code_points_spec(key),
                        };
                        scan_mapping_entries_spec(
                            graph,
                            schema,
                            yaml_mapping_node_index,
                            schema_mapping_node_index,
                            entry_index + 1,
                            entry_end,
                            schema_field_start,
                            schema_field_end,
                            field_limit,
                            key_limit,
                            unknown_field_policy,
                            candidates,
                            unknown_fields.push(unknown),
                            (total_key_code_points as int + key.len()) as u64,
                        )
                    },
                    Some(schema_field_index) => {
                        let prior = candidate_for_schema_field_spec(candidates, schema_field_index);
                        if prior.is_some() {
                            Err(
                                typed_mapping_error_spec(
                                    TypedMappingFieldErrorKind::DuplicateRecognizedField,
                                    key_start,
                                    yaml_mapping_node_index,
                                    schema_mapping_node_index,
                                    Some(entry_index as u64),
                                    Some(schema_field_index),
                                    None,
                                ),
                            )
                        } else {
                            let schema_field = schema.schema.fields[schema_field_index as int];
                            match crate::lower_typed::bind_profile1_typed_yaml_value_spec(
                                graph,
                                schema,
                                entry.value_node_index,
                                schema_field.value_schema_node_index,
                            ) {
                                Err(binding_error) => Err(
                                    typed_mapping_error_spec(
                                        if binding_error.kind
                                            == TypedValueBindingErrorKind::YamlValueKindMismatch {
                                            TypedMappingFieldErrorKind::ValueKindMismatch
                                        } else {
                                            TypedMappingFieldErrorKind::InternalInvariantViolation
                                        },
                                        binding_error.byte_offset,
                                        yaml_mapping_node_index,
                                        schema_mapping_node_index,
                                        Some(entry_index as u64),
                                        Some(schema_field_index),
                                        None,
                                    ),
                                ),
                                Ok(binding) => if candidates.len() + unknown_fields.len()
                                    >= field_limit {
                                    Err(
                                        typed_mapping_error_spec(
                                            TypedMappingFieldErrorKind::FieldLimitExceeded,
                                            key_start,
                                            yaml_mapping_node_index,
                                            schema_mapping_node_index,
                                            Some(entry_index as u64),
                                            Some(schema_field_index),
                                            None,
                                        ),
                                    )
                                } else if key.len() > key_limit - total_key_code_points {
                                    let excluded = (key_limit - total_key_code_points) as nat;
                                    Err(
                                        typed_mapping_error_spec(
                                            TypedMappingFieldErrorKind::KeyCodePointLimitExceeded,
                                            key[excluded as int].byte_start,
                                            yaml_mapping_node_index,
                                            schema_mapping_node_index,
                                            Some(entry_index as u64),
                                            Some(schema_field_index),
                                            Some(excluded as u64),
                                        ),
                                    )
                                } else {
                                    let candidate = TypedMappingFieldView {
                                        mapping_entry_index: entry_index as u64,
                                        schema_field_index,
                                        field_id: schema_field.field_id,
                                        key_yaml_node_index: entry.key_node_index,
                                        value_yaml_node_index: entry.value_node_index,
                                        inherited: entry.inherited,
                                        binding,
                                    };
                                    scan_mapping_entries_spec(
                                        graph,
                                        schema,
                                        yaml_mapping_node_index,
                                        schema_mapping_node_index,
                                        entry_index + 1,
                                        entry_end,
                                        schema_field_start,
                                        schema_field_end,
                                        field_limit,
                                        key_limit,
                                        unknown_field_policy,
                                        candidates.push(candidate),
                                        unknown_fields,
                                        (total_key_code_points as int + key.len()) as u64,
                                    )
                                },
                            }
                        }
                    },
                },
            }
        }
    }
}

pub open spec fn emit_schema_order_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    schema_field_index: nat,
    schema_field_end: nat,
    candidates: Seq<TypedMappingFieldView>,
    output: Seq<TypedMappingFieldView>,
) -> Result<Seq<TypedMappingFieldView>, TypedMappingFieldErrorView>
    decreases schema_field_end - schema_field_index,
{
    if schema_field_index >= schema_field_end || schema_field_end > schema.schema.fields.len() {
        if schema_field_index == schema_field_end && schema_field_end
            <= schema.schema.fields.len() {
            Ok(output)
        } else {
            Err(
                typed_mapping_error_spec(
                    TypedMappingFieldErrorKind::InternalInvariantViolation,
                    graph.nodes[yaml_mapping_node_index as int].byte_start,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    None,
                    Some(schema_field_index as u64),
                    None,
                ),
            )
        }
    } else {
        let field = schema.schema.fields[schema_field_index as int];
        match candidate_for_schema_field_spec(candidates, schema_field_index as u64) {
            Some(candidate) => emit_schema_order_spec(
                graph,
                schema,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                schema_field_index + 1,
                schema_field_end,
                candidates,
                output.push(candidate),
            ),
            None => if field.required {
                Err(
                    typed_mapping_error_spec(
                        TypedMappingFieldErrorKind::MissingRequiredField,
                        graph.nodes[yaml_mapping_node_index as int].byte_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        None,
                        Some(schema_field_index as u64),
                        None,
                    ),
                )
            } else {
                emit_schema_order_spec(
                    graph,
                    schema,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_field_index + 1,
                    schema_field_end,
                    candidates,
                    output,
                )
            },
        }
    }
}

spec fn partition_after_emission_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    unknown_field_policy: TypedMappingUnknownFieldPolicy,
    mapping_binding: TypedYamlValueBindingView,
    total_key_code_points: u64,
    unknown_fields: Seq<TypedMappingUnknownFieldView>,
    emitted: Result<Seq<TypedMappingFieldView>, TypedMappingFieldErrorView>,
) -> Result<TypedMappingFieldPartitionView, TypedMappingFieldErrorView> {
    match emitted {
        Err(error) => Err(error),
        Ok(fields) => Ok(
            TypedMappingFieldPartitionView {
                transformation_version: TYPED_MAPPING_FIELD_PARTITION_VERSION,
                canonical_profile_version: graph.profile_version,
                schema_version: schema.schema_version,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                unknown_field_policy,
                mapping_binding,
                total_key_code_points,
                fields,
                unknown_fields,
            },
        ),
    }
}

spec fn partition_after_scan_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    schema_field_start: nat,
    schema_field_end: nat,
    unknown_field_policy: TypedMappingUnknownFieldPolicy,
    mapping_binding: TypedYamlValueBindingView,
    scanned: Result<
        (Seq<TypedMappingFieldView>, Seq<TypedMappingUnknownFieldView>, u64),
        TypedMappingFieldErrorView,
    >,
) -> Result<TypedMappingFieldPartitionView, TypedMappingFieldErrorView> {
    match scanned {
        Err(error) => Err(error),
        Ok((candidates, unknown_fields, total_key_code_points)) => partition_after_emission_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            mapping_binding,
            total_key_code_points,
            unknown_fields,
            emit_schema_order_spec(
                graph,
                schema,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                schema_field_start,
                schema_field_end,
                candidates,
                Seq::empty(),
            ),
        ),
    }
}

pub closed spec fn partition_profile1_typed_mapping_fields_with_policy_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    unknown_field_policy: TypedMappingUnknownFieldPolicy,
    limits: TypedMappingFieldLimitsView,
) -> Result<TypedMappingFieldPartitionView, TypedMappingFieldErrorView> {
    match crate::lower_typed::bind_profile1_typed_yaml_value_spec(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
    ) {
        Err(binding_error) => Err(
            typed_mapping_error_spec(
                if binding_error.kind == TypedValueBindingErrorKind::YamlValueKindMismatch {
                    TypedMappingFieldErrorKind::MappingKindMismatch
                } else {
                    TypedMappingFieldErrorKind::InternalInvariantViolation
                },
                binding_error.byte_offset,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                None,
                None,
                None,
            ),
        ),
        Ok(mapping_binding) => {
            let node = graph.nodes[yaml_mapping_node_index as int];
            let schema_node = schema.schema.nodes[schema_mapping_node_index as int];
            if node.kind != crate::lower::CanonicalYamlNodeKind::Mapping
                || !typed_schema_kind_is_mapping_spec(schema_node.kind) {
                Err(
                    typed_mapping_error_spec(
                        TypedMappingFieldErrorKind::MappingKindMismatch,
                        node.byte_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        None,
                        None,
                        None,
                    ),
                )
            } else if schema_node.field_start > schema_node.field_end || schema_node.field_end
                > schema.schema.fields.len() {
                Err(
                    typed_mapping_error_spec(
                        TypedMappingFieldErrorKind::InternalInvariantViolation,
                        node.byte_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        None,
                        Some(schema_node.field_start),
                        None,
                    ),
                )
            } else {
                let field_limit = typed_mapping_effective_limit_spec(
                    limits.max_fields,
                    MAX_PROFILE1_TYPED_MAPPING_FIELDS,
                );
                let key_limit = typed_mapping_effective_limit_spec(
                    limits.max_key_code_points,
                    MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
                );
                partition_after_scan_spec(
                    graph,
                    schema,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_node.field_start as nat,
                    schema_node.field_end as nat,
                    unknown_field_policy,
                    mapping_binding,
                    scan_mapping_entries_spec(
                        graph,
                        schema,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        node.edge_start as nat,
                        node.edge_end as nat,
                        schema_node.field_start as nat,
                        schema_node.field_end as nat,
                        field_limit,
                        key_limit,
                        unknown_field_policy,
                        Seq::empty(),
                        Seq::empty(),
                        0,
                    ),
                )
            }
        },
    }
}

pub closed spec fn partition_profile1_typed_mapping_fields_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    limits: TypedMappingFieldLimitsView,
) -> Result<TypedMappingFieldPartitionView, TypedMappingFieldErrorView> {
    partition_profile1_typed_mapping_fields_with_policy_spec(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        TypedMappingUnknownFieldPolicy::Reject,
        limits,
    )
}

pub open spec fn typed_mapping_key_code_points_for_node_spec(
    graph: CanonicalYamlGraphSourceView,
    key_node_index: u64,
) -> Option<Seq<u32>> {
    if key_node_index >= graph.nodes.len() {
        None
    } else {
        let node = graph.nodes[key_node_index as int];
        match node.scalar_index {
            None => None,
            Some(scalar_index) => {
                let scalars = crate::lower_typed::canonical_yaml_graph_scalars_spec(graph);
                if node.kind != crate::lower::CanonicalYamlNodeKind::Scalar || scalar_index
                    >= scalars.len() {
                    None
                } else {
                    let scalar = scalars[scalar_index as int];
                    if scalar.node_index != node.resolved_node_index || scalar.tag
                        != ResolvedScalarTag::CoreString || scalar.value
                        != ResolvedScalarValueView::String {
                        None
                    } else {
                        match scalar.presentation.decoded {
                            None => None,
                            Some(decoded) => Some(
                                Seq::new(
                                    decoded.content.len(),
                                    |index: int| { decoded.content[index].code_point },
                                ),
                            ),
                        }
                    }
                }
            },
        }
    }
}

pub open spec fn typed_mapping_field_record_semantics_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    partition: TypedMappingFieldPartitionView,
    field: TypedMappingFieldView,
) -> bool {
    partition.yaml_mapping_node_index < graph.nodes.len() && partition.schema_mapping_node_index
        < schema.schema.nodes.len() && field.mapping_entry_index < graph.mapping_entries.len()
        && field.schema_field_index < schema.schema.fields.len() && {
        let mapping_node = graph.nodes[partition.yaml_mapping_node_index as int];
        let schema_node = schema.schema.nodes[partition.schema_mapping_node_index as int];
        let entry = graph.mapping_entries[field.mapping_entry_index as int];
        let definition = schema.schema.fields[field.schema_field_index as int];
        mapping_node.edge_start <= field.mapping_entry_index < mapping_node.edge_end
            && schema_node.field_start <= field.schema_field_index < schema_node.field_end
            && entry.receiver_node_index == partition.yaml_mapping_node_index && field.field_id
            == definition.field_id && field.key_yaml_node_index == entry.key_node_index
            && field.value_yaml_node_index == entry.value_node_index && field.inherited
            == entry.inherited && typed_mapping_key_code_points_for_node_spec(
            graph,
            entry.key_node_index,
        ) == Some(definition.name) && crate::lower_typed::bind_profile1_typed_yaml_value_spec(
            graph,
            schema,
            entry.value_node_index,
            definition.value_schema_node_index,
        ) == Ok(field.binding)
    }
}

pub open spec fn typed_mapping_unknown_field_record_semantics_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    partition: TypedMappingFieldPartitionView,
    unknown: TypedMappingUnknownFieldView,
) -> bool {
    partition.yaml_mapping_node_index < graph.nodes.len() && partition.schema_mapping_node_index
        < schema.schema.nodes.len() && unknown.mapping_entry_index < graph.mapping_entries.len()
        && unknown.key_yaml_node_index < graph.nodes.len() && {
        let mapping_node = graph.nodes[partition.yaml_mapping_node_index as int];
        let schema_node = schema.schema.nodes[partition.schema_mapping_node_index as int];
        let entry = graph.mapping_entries[unknown.mapping_entry_index as int];
        let key_node = graph.nodes[unknown.key_yaml_node_index as int];
        mapping_node.edge_start <= unknown.mapping_entry_index < mapping_node.edge_end
            && entry.receiver_node_index == partition.yaml_mapping_node_index
            && unknown.key_yaml_node_index == entry.key_node_index && unknown.value_yaml_node_index
            == entry.value_node_index && unknown.inherited == entry.inherited && unknown.byte_start
            == key_node.byte_start && unknown.byte_end == key_node.byte_end
            && typed_mapping_key_code_points_for_node_spec(graph, entry.key_node_index) == Some(
            unknown.key_code_points,
        ) && forall|schema_field_index: int|
            schema_node.field_start <= schema_field_index < schema_node.field_end
                ==> schema.schema.fields[schema_field_index].name != unknown.key_code_points
    }
}

pub open spec fn typed_mapping_partition_covers_entry_spec(
    partition: TypedMappingFieldPartitionView,
    entry_index: int,
) -> bool {
    (exists|field_index: int|
        0 <= field_index < partition.fields.len()
            && partition.fields[field_index].mapping_entry_index == entry_index) || (exists|
        unknown_index: int,
    |
        0 <= unknown_index < partition.unknown_fields.len()
            && partition.unknown_fields[unknown_index].mapping_entry_index == entry_index)
}

pub open spec fn typed_mapping_partition_contains_schema_field_spec(
    partition: TypedMappingFieldPartitionView,
    schema_field_index: int,
) -> bool {
    exists|field_index: int|
        0 <= field_index < partition.fields.len()
            && partition.fields[field_index].schema_field_index == schema_field_index
}

pub open spec fn typed_mapping_fields_strict_schema_order_spec(
    fields: Seq<TypedMappingFieldView>,
) -> bool
    decreases fields.len(),
{
    if fields.len() <= 1 {
        true
    } else {
        fields[0].schema_field_index < fields[1].schema_field_index
            && typed_mapping_fields_strict_schema_order_spec(fields.drop_first())
    }
}

pub open spec fn typed_mapping_required_fields_covered_spec(
    schema: CompiledTypedFieldSchemaView,
    schema_mapping_node_index: u64,
    fields: Seq<TypedMappingFieldView>,
) -> bool {
    schema_mapping_node_index < schema.schema.nodes.len() && {
        let schema_node = schema.schema.nodes[schema_mapping_node_index as int];
        forall|schema_field_index: int|
            schema_node.field_start <= schema_field_index < schema_node.field_end
                && schema.schema.fields[schema_field_index].required ==> exists|field_index: int|
                0 <= field_index < fields.len() && #[trigger] fields[field_index].schema_field_index
                    == schema_field_index
    }
}

pub open spec fn typed_mapping_field_partition_semantics_spec(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    partition: TypedMappingFieldPartitionView,
) -> bool {
    partition.transformation_version == TYPED_MAPPING_FIELD_PARTITION_VERSION
        && partition.canonical_profile_version == graph.profile_version && partition.schema_version
        == schema.schema_version && partition.yaml_mapping_node_index < graph.nodes.len()
        && partition.schema_mapping_node_index < schema.schema.nodes.len()
        && crate::lower_typed::bind_profile1_typed_yaml_value_spec(
        graph,
        schema,
        partition.yaml_mapping_node_index,
        partition.schema_mapping_node_index,
    ) == Ok(partition.mapping_binding) && (partition.unknown_field_policy
        == TypedMappingUnknownFieldPolicy::Preserve || partition.unknown_fields.len() == 0)
        && typed_mapping_fields_strict_schema_order_spec(partition.fields)
        && typed_mapping_required_fields_covered_spec(
        schema,
        partition.schema_mapping_node_index,
        partition.fields,
    ) && {
        let mapping_node = graph.nodes[partition.yaml_mapping_node_index as int];
        let schema_node = schema.schema.nodes[partition.schema_mapping_node_index as int];
        exists|candidates: Seq<TypedMappingFieldView>, field_limit: u64, key_limit: u64|
            field_limit <= MAX_PROFILE1_TYPED_MAPPING_FIELDS && key_limit
                <= MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS && scan_mapping_entries_spec(
                graph,
                schema,
                partition.yaml_mapping_node_index,
                partition.schema_mapping_node_index,
                mapping_node.edge_start as nat,
                mapping_node.edge_end as nat,
                schema_node.field_start as nat,
                schema_node.field_end as nat,
                field_limit,
                key_limit,
                partition.unknown_field_policy,
                Seq::empty(),
                Seq::empty(),
                0,
            ) == Ok((candidates, partition.unknown_fields, partition.total_key_code_points))
                && emit_schema_order_spec(
                graph,
                schema,
                partition.yaml_mapping_node_index,
                partition.schema_mapping_node_index,
                schema_node.field_start as nat,
                schema_node.field_end as nat,
                candidates,
                Seq::empty(),
            ) == Ok(partition.fields)
    }
}

proof fn lemma_candidate_from_some_matches_schema_field(
    candidates: Seq<TypedMappingFieldView>,
    candidate_index: nat,
    schema_field_index: u64,
    candidate: TypedMappingFieldView,
)
    requires
        candidate_index <= candidates.len(),
        candidate_for_schema_field_from_spec(candidates, candidate_index, schema_field_index)
            == Some(candidate),
    ensures
        candidate.schema_field_index == schema_field_index,
    decreases candidates.len() - candidate_index,
{
    reveal(candidate_for_schema_field_from_spec);
    if candidate_index < candidates.len() && candidates[candidate_index as int].schema_field_index
        != schema_field_index {
        lemma_candidate_from_some_matches_schema_field(
            candidates,
            candidate_index + 1,
            schema_field_index,
            candidate,
        );
    }
}

proof fn lemma_candidate_some_matches_schema_field(
    candidates: Seq<TypedMappingFieldView>,
    schema_field_index: u64,
    candidate: TypedMappingFieldView,
)
    requires
        candidate_for_schema_field_spec(candidates, schema_field_index) == Some(candidate),
    ensures
        candidate.schema_field_index == schema_field_index,
{
    reveal(candidate_for_schema_field_spec);
    lemma_candidate_from_some_matches_schema_field(candidates, 0, schema_field_index, candidate);
}

proof fn lemma_reject_scan_preserves_unknown_fields(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    entry_index: nat,
    entry_end: nat,
    schema_field_start: nat,
    schema_field_end: nat,
    field_limit: u64,
    key_limit: u64,
    candidates: Seq<TypedMappingFieldView>,
    unknown_fields: Seq<TypedMappingUnknownFieldView>,
    total_key_code_points: u64,
    result_candidates: Seq<TypedMappingFieldView>,
    result_unknown_fields: Seq<TypedMappingUnknownFieldView>,
    result_total_key_code_points: u64,
)
    requires
        scan_mapping_entries_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            entry_index,
            entry_end,
            schema_field_start,
            schema_field_end,
            field_limit,
            key_limit,
            TypedMappingUnknownFieldPolicy::Reject,
            candidates,
            unknown_fields,
            total_key_code_points,
        ) == Ok((result_candidates, result_unknown_fields, result_total_key_code_points)),
    ensures
        result_unknown_fields == unknown_fields,
    decreases entry_end - entry_index,
{
    reveal(scan_mapping_entries_spec);
    if entry_index < entry_end && entry_end <= graph.mapping_entries.len() {
        let entry = graph.mapping_entries[entry_index as int];
        if entry.receiver_node_index == yaml_mapping_node_index && entry.key_node_index
            < graph.nodes.len() && entry.value_node_index < graph.nodes.len() {
            match mapping_key_for_node_spec(graph, entry.key_node_index) {
                Some(key) => match find_schema_field_spec(
                    schema.schema.fields,
                    schema_field_start,
                    schema_field_end,
                    key,
                ) {
                    Some(schema_field_index) => {
                        let prior = candidate_for_schema_field_spec(candidates, schema_field_index);
                        if prior.is_none() {
                            let schema_field = schema.schema.fields[schema_field_index as int];
                            match crate::lower_typed::bind_profile1_typed_yaml_value_spec(
                                graph,
                                schema,
                                entry.value_node_index,
                                schema_field.value_schema_node_index,
                            ) {
                                Ok(binding) => {
                                    if candidates.len() + unknown_fields.len() < field_limit
                                        && key.len() <= key_limit - total_key_code_points {
                                        let candidate = TypedMappingFieldView {
                                            mapping_entry_index: entry_index as u64,
                                            schema_field_index,
                                            field_id: schema_field.field_id,
                                            key_yaml_node_index: entry.key_node_index,
                                            value_yaml_node_index: entry.value_node_index,
                                            inherited: entry.inherited,
                                            binding,
                                        };
                                        lemma_reject_scan_preserves_unknown_fields(
                                            graph,
                                            schema,
                                            yaml_mapping_node_index,
                                            schema_mapping_node_index,
                                            entry_index + 1,
                                            entry_end,
                                            schema_field_start,
                                            schema_field_end,
                                            field_limit,
                                            key_limit,
                                            candidates.push(candidate),
                                            unknown_fields,
                                            (total_key_code_points as int + key.len()) as u64,
                                            result_candidates,
                                            result_unknown_fields,
                                            result_total_key_code_points,
                                        );
                                    }
                                },
                                Err(_) => {},
                            }
                        }
                    },
                    None => {},
                },
                None => {},
            }
        }
    }
}

proof fn lemma_emit_schema_order_preserves_prefix(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    schema_field_index: nat,
    schema_field_end: nat,
    candidates: Seq<TypedMappingFieldView>,
    output: Seq<TypedMappingFieldView>,
    result: Seq<TypedMappingFieldView>,
)
    requires
        schema_field_index <= schema_field_end <= schema.schema.fields.len(),
        emit_schema_order_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            schema_field_index,
            schema_field_end,
            candidates,
            output,
        ) == Ok(result),
    ensures
        exists|suffix: Seq<TypedMappingFieldView>| result == output + suffix,
    decreases schema_field_end - schema_field_index,
{
    reveal(emit_schema_order_spec);
    if schema_field_index < schema_field_end {
        match candidate_for_schema_field_spec(candidates, schema_field_index as u64) {
            Some(candidate) => {
                lemma_emit_schema_order_preserves_prefix(
                    graph,
                    schema,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_field_index + 1,
                    schema_field_end,
                    candidates,
                    output.push(candidate),
                    result,
                );
                let suffix = choose|suffix: Seq<TypedMappingFieldView>|
                    result == output.push(candidate) + suffix;
                assert(output.push(candidate) + suffix =~= output + (Seq::empty().push(candidate)
                    + suffix));
                assert(exists|combined_suffix: Seq<TypedMappingFieldView>|
                    combined_suffix == Seq::empty().push(candidate) + suffix && result == output
                        + combined_suffix);
            },
            None => {
                if !schema.schema.fields[schema_field_index as int].required {
                    lemma_emit_schema_order_preserves_prefix(
                        graph,
                        schema,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        schema_field_index + 1,
                        schema_field_end,
                        candidates,
                        output,
                        result,
                    );
                }
            },
        }
    } else {
        assert(result == output);
        assert(output + Seq::empty() =~= output);
        assert(exists|suffix: Seq<TypedMappingFieldView>|
            suffix == Seq::empty() && result == output + suffix);
    }
}

proof fn lemma_strict_schema_order_push(
    fields: Seq<TypedMappingFieldView>,
    candidate: TypedMappingFieldView,
)
    requires
        typed_mapping_fields_strict_schema_order_spec(fields),
        forall|index: int|
            0 <= index < fields.len() ==> #[trigger] fields[index].schema_field_index
                < candidate.schema_field_index,
    ensures
        typed_mapping_fields_strict_schema_order_spec(fields.push(candidate)),
    decreases fields.len(),
{
    reveal(typed_mapping_fields_strict_schema_order_spec);
    if fields.len() == 0 {
        assert(fields.push(candidate).len() == 1);
        assert(typed_mapping_fields_strict_schema_order_spec(fields.push(candidate)));
    } else if fields.len() == 1 {
        assert(fields.push(candidate)[0] == fields[0]);
        assert(fields.push(candidate)[1] == candidate);
        assert(fields[0].schema_field_index < candidate.schema_field_index);
        assert(fields.push(candidate).drop_first().len() == 1);
        assert(typed_mapping_fields_strict_schema_order_spec(fields.push(candidate).drop_first()));
        assert(typed_mapping_fields_strict_schema_order_spec(fields.push(candidate)));
    } else if fields.len() > 1 {
        assert(fields.push(candidate).drop_first() =~= fields.drop_first().push(candidate));
        assert forall|index: int|
            0 <= index
                < fields.drop_first().len() implies #[trigger] fields.drop_first()[index].schema_field_index
            < candidate.schema_field_index by {
            assert(fields.drop_first()[index] == fields[index + 1]);
        }
        lemma_strict_schema_order_push(fields.drop_first(), candidate);
        assert(fields.push(candidate)[0] == fields[0]);
        assert(fields.push(candidate)[1] == fields[1]);
        assert(typed_mapping_fields_strict_schema_order_spec(fields.push(candidate)));
    }
}

proof fn lemma_emit_schema_order_is_strict(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    schema_field_index: nat,
    schema_field_end: nat,
    candidates: Seq<TypedMappingFieldView>,
    output: Seq<TypedMappingFieldView>,
    result: Seq<TypedMappingFieldView>,
)
    requires
        schema_field_index <= schema_field_end <= schema.schema.fields.len(),
        schema_field_end <= u64::MAX,
        emit_schema_order_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            schema_field_index,
            schema_field_end,
            candidates,
            output,
        ) == Ok(result),
        typed_mapping_fields_strict_schema_order_spec(output),
        forall|index: int|
            0 <= index < output.len() ==> #[trigger] output[index].schema_field_index
                < schema_field_index,
    ensures
        typed_mapping_fields_strict_schema_order_spec(result),
    decreases schema_field_end - schema_field_index,
{
    reveal(emit_schema_order_spec);
    if schema_field_index < schema_field_end {
        match candidate_for_schema_field_spec(candidates, schema_field_index as u64) {
            Some(candidate) => {
                assert(schema_field_index as u64 as nat == schema_field_index);
                lemma_candidate_some_matches_schema_field(
                    candidates,
                    schema_field_index as u64,
                    candidate,
                );
                assert forall|index: int|
                    0 <= index < output.len() implies #[trigger] output[index].schema_field_index
                    < candidate.schema_field_index by {};
                lemma_strict_schema_order_push(output, candidate);
                assert forall|index: int|
                    0 <= index < output.push(candidate).len() implies #[trigger] output.push(
                    candidate,
                )[index].schema_field_index < schema_field_index + 1 by {};
                lemma_emit_schema_order_is_strict(
                    graph,
                    schema,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_field_index + 1,
                    schema_field_end,
                    candidates,
                    output.push(candidate),
                    result,
                );
            },
            None => {
                if !schema.schema.fields[schema_field_index as int].required {
                    assert forall|index: int|
                        0 <= index
                            < output.len() implies #[trigger] output[index].schema_field_index
                        < schema_field_index + 1 by {};
                    lemma_emit_schema_order_is_strict(
                        graph,
                        schema,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        schema_field_index + 1,
                        schema_field_end,
                        candidates,
                        output,
                        result,
                    );
                }
            },
        }
    }
}

proof fn lemma_emit_schema_order_covers_required(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    schema_field_index: nat,
    schema_field_end: nat,
    candidates: Seq<TypedMappingFieldView>,
    output: Seq<TypedMappingFieldView>,
    result: Seq<TypedMappingFieldView>,
)
    requires
        schema_field_index <= schema_field_end <= schema.schema.fields.len(),
        schema_field_end <= u64::MAX,
        emit_schema_order_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            schema_field_index,
            schema_field_end,
            candidates,
            output,
        ) == Ok(result),
    ensures
        forall|required_index: int|
            schema_field_index <= required_index < schema_field_end
                && schema.schema.fields[required_index].required ==> exists|field_index: int|
                0 <= field_index < result.len() && #[trigger] result[field_index].schema_field_index
                    == required_index,
    decreases schema_field_end - schema_field_index,
{
    reveal(emit_schema_order_spec);
    if schema_field_index < schema_field_end {
        match candidate_for_schema_field_spec(candidates, schema_field_index as u64) {
            Some(candidate) => {
                assert(schema_field_index as u64 as nat == schema_field_index);
                lemma_candidate_some_matches_schema_field(
                    candidates,
                    schema_field_index as u64,
                    candidate,
                );
                lemma_emit_schema_order_preserves_prefix(
                    graph,
                    schema,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_field_index + 1,
                    schema_field_end,
                    candidates,
                    output.push(candidate),
                    result,
                );
                lemma_emit_schema_order_covers_required(
                    graph,
                    schema,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_field_index + 1,
                    schema_field_end,
                    candidates,
                    output.push(candidate),
                    result,
                );
                let suffix = choose|suffix: Seq<TypedMappingFieldView>|
                    result == output.push(candidate) + suffix;
                assert(result == output.push(candidate) + suffix);
                assert(result.len() == output.len() + 1 + suffix.len());
                assert(output.len() < result.len());
                assert(result[output.len() as int] == candidate);
                assert forall|required_index: int|
                    schema_field_index <= required_index < schema_field_end
                        && schema.schema.fields[required_index].required implies exists|
                    field_index: int,
                |
                    0 <= field_index < result.len()
                        && #[trigger] result[field_index].schema_field_index == required_index by {
                    if required_index == schema_field_index {
                        let witness = output.len() as int;
                        assert(0 <= witness < result.len());
                        assert(candidate.schema_field_index as nat == schema_field_index);
                        assert(result[witness].schema_field_index as int == required_index);
                        assert(exists|field_index: int|
                            0 <= field_index < result.len()
                                && #[trigger] result[field_index].schema_field_index
                                == required_index);
                    }
                }
            },
            None => {
                if !schema.schema.fields[schema_field_index as int].required {
                    lemma_emit_schema_order_covers_required(
                        graph,
                        schema,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        schema_field_index + 1,
                        schema_field_end,
                        candidates,
                        output,
                        result,
                    );
                }
            },
        }
    }
}

proof fn lemma_successful_typed_mapping_field_partition_has_binding(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    limits: TypedMappingFieldLimitsView,
    partition: TypedMappingFieldPartitionView,
)
    requires
        partition_profile1_typed_mapping_fields_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            limits,
        ) == Ok(partition),
    ensures
        exists|binding: TypedYamlValueBindingView|
            crate::lower_typed::bind_profile1_typed_yaml_value_spec(
                graph,
                schema,
                yaml_mapping_node_index,
                schema_mapping_node_index,
            ) == Ok(binding),
{
    reveal(partition_profile1_typed_mapping_fields_spec);
    reveal(partition_profile1_typed_mapping_fields_with_policy_spec);
    reveal(partition_after_scan_spec);
    reveal(partition_after_emission_spec);
    let binding_result = crate::lower_typed::bind_profile1_typed_yaml_value_spec(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
    );
    match binding_result {
        Err(binding_error) => {
            let partition_error = typed_mapping_error_spec(
                if binding_error.kind == TypedValueBindingErrorKind::YamlValueKindMismatch {
                    TypedMappingFieldErrorKind::MappingKindMismatch
                } else {
                    TypedMappingFieldErrorKind::InternalInvariantViolation
                },
                binding_error.byte_offset,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                None,
                None,
                None,
            );
            assert(partition_profile1_typed_mapping_fields_spec(
                graph,
                schema,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                limits,
            ) == Err(partition_error));
            assert(false);
        },
        Ok(binding) => {
            assert(exists|witness: TypedYamlValueBindingView| binding_result == Ok(witness)) by {
                assert(binding_result == Ok(binding));
            };
        },
    }
}

pub proof fn lemma_successful_typed_mapping_field_partition_has_semantics(
    graph: CanonicalYamlGraphSourceView,
    schema: CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    limits: TypedMappingFieldLimitsView,
    partition: TypedMappingFieldPartitionView,
)
    requires
        partition_profile1_typed_mapping_fields_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            limits,
        ) == Ok(partition),
    ensures
        typed_mapping_field_partition_semantics_spec(graph, schema, partition),
{
    reveal(partition_profile1_typed_mapping_fields_spec);
    reveal(partition_profile1_typed_mapping_fields_with_policy_spec);
    reveal(partition_after_scan_spec);
    reveal(partition_after_emission_spec);
    let binding = crate::lower_typed::bind_profile1_typed_yaml_value_spec(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
    );
    lemma_successful_typed_mapping_field_partition_has_binding(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        limits,
        partition,
    );
    let mapping_binding = choose|witness: TypedYamlValueBindingView| binding == Ok(witness);
    assert(binding == Ok(mapping_binding));
    let mapping_node = graph.nodes[yaml_mapping_node_index as int];
    let schema_node = schema.schema.nodes[schema_mapping_node_index as int];
    let field_limit = typed_mapping_effective_limit_spec(
        limits.max_fields,
        MAX_PROFILE1_TYPED_MAPPING_FIELDS,
    );
    let key_limit = typed_mapping_effective_limit_spec(
        limits.max_key_code_points,
        MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
    );
    let scanned = scan_mapping_entries_spec(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        mapping_node.edge_start as nat,
        mapping_node.edge_end as nat,
        schema_node.field_start as nat,
        schema_node.field_end as nat,
        field_limit,
        key_limit,
        TypedMappingUnknownFieldPolicy::Reject,
        Seq::empty(),
        Seq::empty(),
        0,
    );
    assert(exists|witness: (Seq<TypedMappingFieldView>, Seq<TypedMappingUnknownFieldView>, u64)|
        scanned == Ok(witness)) by {
        match scanned {
            Err(_) => {
                assert(false);
            },
            Ok(scanned_value) => {
                assert(exists|
                    witness: (Seq<TypedMappingFieldView>, Seq<TypedMappingUnknownFieldView>, u64),
                |
                    scanned == Ok(witness)) by {
                    assert(scanned == Ok(scanned_value));
                };
            },
        }
    };
    let scanned_value = choose|
        witness: (Seq<TypedMappingFieldView>, Seq<TypedMappingUnknownFieldView>, u64),
    |
        scanned == Ok(witness);
    let (candidates, unknown_fields, total_key_code_points) = scanned_value;
    assert(scanned == Ok((candidates, unknown_fields, total_key_code_points)));
    lemma_reject_scan_preserves_unknown_fields(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        mapping_node.edge_start as nat,
        mapping_node.edge_end as nat,
        schema_node.field_start as nat,
        schema_node.field_end as nat,
        field_limit,
        key_limit,
        Seq::empty(),
        Seq::empty(),
        0,
        candidates,
        unknown_fields,
        total_key_code_points,
    );
    assert(unknown_fields == Seq::<TypedMappingUnknownFieldView>::empty());
    assert(unknown_fields == partition.unknown_fields);
    assert(total_key_code_points == partition.total_key_code_points);
    assert(emit_schema_order_spec(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        schema_node.field_start as nat,
        schema_node.field_end as nat,
        candidates,
        Seq::empty(),
    ) == Ok(partition.fields));
    lemma_emit_schema_order_is_strict(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        schema_node.field_start as nat,
        schema_node.field_end as nat,
        candidates,
        Seq::empty(),
        partition.fields,
    );
    lemma_emit_schema_order_covers_required(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        schema_node.field_start as nat,
        schema_node.field_end as nat,
        candidates,
        Seq::empty(),
        partition.fields,
    );
    reveal(typed_mapping_fields_strict_schema_order_spec);
    reveal(typed_mapping_required_fields_covered_spec);
    reveal(typed_mapping_field_partition_semantics_spec);
    assert(field_limit <= MAX_PROFILE1_TYPED_MAPPING_FIELDS);
    assert(key_limit <= MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS);
    assert(exists|
        witness_candidates: Seq<TypedMappingFieldView>,
        witness_field_limit: u64,
        witness_key_limit: u64,
    |
        #![auto]
        witness_field_limit <= MAX_PROFILE1_TYPED_MAPPING_FIELDS && witness_key_limit
            <= MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS && scan_mapping_entries_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            mapping_node.edge_start as nat,
            mapping_node.edge_end as nat,
            schema_node.field_start as nat,
            schema_node.field_end as nat,
            witness_field_limit,
            witness_key_limit,
            TypedMappingUnknownFieldPolicy::Reject,
            Seq::empty(),
            Seq::empty(),
            0,
        ) == Ok((witness_candidates, partition.unknown_fields, partition.total_key_code_points))
            && emit_schema_order_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            schema_node.field_start as nat,
            schema_node.field_end as nat,
            witness_candidates,
            Seq::empty(),
        ) == Ok(partition.fields)) by {
        assert(scan_mapping_entries_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            mapping_node.edge_start as nat,
            mapping_node.edge_end as nat,
            schema_node.field_start as nat,
            schema_node.field_end as nat,
            field_limit,
            key_limit,
            TypedMappingUnknownFieldPolicy::Reject,
            Seq::empty(),
            Seq::empty(),
            0,
        ) == Ok((candidates, partition.unknown_fields, partition.total_key_code_points)));
    };
    assert(partition.transformation_version == TYPED_MAPPING_FIELD_PARTITION_VERSION);
    assert(partition.canonical_profile_version == graph.profile_version);
    assert(partition.schema_version == schema.schema_version);
    assert(partition.yaml_mapping_node_index == yaml_mapping_node_index);
    assert(partition.schema_mapping_node_index == schema_mapping_node_index);
    assert(partition.yaml_mapping_node_index < graph.nodes.len());
    assert(partition.schema_mapping_node_index < schema.schema.nodes.len());
    assert(binding == Ok(partition.mapping_binding));
    assert(partition.unknown_field_policy == TypedMappingUnknownFieldPolicy::Reject);
    assert(partition.unknown_fields.len() == 0);
    assert(typed_mapping_fields_strict_schema_order_spec(partition.fields));
    assert(typed_mapping_required_fields_covered_spec(
        schema,
        partition.schema_mapping_node_index,
        partition.fields,
    ));
    assert(typed_mapping_field_partition_semantics_spec(graph, schema, partition));
}

#[verifier::rlimit(80)]
pub fn partition_profile1_typed_mapping_fields_with_policy(
    graph: &CanonicalYamlGraphSource,
    schema: &CompiledTypedFieldSchema,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    unknown_field_policy: TypedMappingUnknownFieldPolicy,
    limits: TypedMappingFieldLimits,
) -> (result: Result<TypedMappingFieldPartition, TypedMappingFieldError>)
    ensures
        partition_profile1_typed_mapping_fields_with_policy_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            limits@,
        ) == match result {
            Ok(partition) => Ok(partition@),
            Err(error) => Err(error@),
        },
{
    let ghost expected = partition_profile1_typed_mapping_fields_with_policy_spec(
        graph@,
        schema@,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        unknown_field_policy,
        limits@,
    );
    let mapping_binding = match bind_profile1_typed_yaml_value(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
    ) {
        Ok(binding) => binding,
        Err(binding_error) => {
            let kind = if binding_error.kind()
                == TypedValueBindingErrorKind::YamlValueKindMismatch {
                TypedMappingFieldErrorKind::MappingKindMismatch
            } else {
                TypedMappingFieldErrorKind::InternalInvariantViolation
            };
            let error = TypedMappingFieldError::at(
                kind,
                binding_error.byte_offset(),
                yaml_mapping_node_index,
                schema_mapping_node_index,
                None,
                None,
                None,
            );
            proof {
                reveal(partition_profile1_typed_mapping_fields_with_policy_spec);
                reveal(typed_mapping_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        },
    };
    proof {
        assert(crate::lower_typed::bind_profile1_typed_yaml_value_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
        ) == Ok(mapping_binding@));
        reveal(crate::lower_typed::bind_profile1_typed_yaml_value_spec);
        reveal(partition_profile1_typed_mapping_fields_with_policy_spec);
    }
    let nodes = graph.nodes();
    let node = &nodes[yaml_mapping_node_index as usize];
    let schema_node = &schema.schema().nodes()[schema_mapping_node_index as usize];
    if node.kind() != crate::lower::CanonicalYamlNodeKind::Mapping || !typed_schema_kind_is_mapping(
        schema_node.kind(),
    ) {
        let error = TypedMappingFieldError::at(
            TypedMappingFieldErrorKind::MappingKindMismatch,
            node.byte_start(),
            yaml_mapping_node_index,
            schema_mapping_node_index,
            None,
            None,
            None,
        );
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(crate::schema::typed_schema_node_views_spec);
            reveal(typed_mapping_error_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    if schema_node.field_start() > schema_node.field_end() || schema_node.field_end()
        > schema.schema().fields().len() as u64 {
        let error = TypedMappingFieldError::at(
            TypedMappingFieldErrorKind::InternalInvariantViolation,
            node.byte_start(),
            yaml_mapping_node_index,
            schema_mapping_node_index,
            None,
            Some(schema_node.field_start()),
            None,
        );
        proof {
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(crate::schema::typed_schema_node_views_spec);
            reveal(typed_mapping_error_spec);
            assert(expected == Err(error@));
        }
        return Err(error);
    }
    let field_start = schema_node.field_start() as usize;
    let field_end = schema_node.field_end() as usize;
    let entry_start = node.edge_start() as usize;
    let entry_end = node.edge_end() as usize;
    let entries = graph.mapping_entries();
    let schema_fields = schema.schema().fields();
    let field_limit = effective_limit(limits.max_fields(), MAX_PROFILE1_TYPED_MAPPING_FIELDS);
    let key_limit = effective_limit(
        limits.max_key_code_points(),
        MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
    );
    proof {
        reveal(crate::lower::canonical_yaml_node_views_spec);
        reveal(crate::lower::canonical_mapping_entry_views_spec);
        reveal(crate::schema::typed_schema_node_views_spec);
        reveal(crate::schema::typed_field_definition_views_spec);
        reveal(crate::lower_typed::bind_profile1_typed_yaml_value_spec);
        reveal(partition_profile1_typed_mapping_fields_with_policy_spec);
        reveal(typed_mapping_effective_limit_spec);
        reveal(typed_mapping_field_views_spec);
        assert(graph@.nodes[yaml_mapping_node_index as int] == node@);
        assert(schema@.schema.nodes[schema_mapping_node_index as int] == schema_node@);
        assert(entry_end <= entries.len());
        assert(field_end <= schema_fields.len());
        assert(field_start as u64 == schema_node@.field_start);
        assert(field_end as u64 == schema_node@.field_end);
        assert(entry_start as u64 == node@.edge_start);
        assert(entry_end as u64 == node@.edge_end);
        assert(field_limit == typed_mapping_effective_limit_spec(
            limits@.max_fields,
            MAX_PROFILE1_TYPED_MAPPING_FIELDS,
        ));
        assert(key_limit == typed_mapping_effective_limit_spec(
            limits@.max_key_code_points,
            MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
        ));
    }
    let mut candidates: Vec<TypedMappingField> = Vec::new();
    let mut unknown_fields: Vec<TypedMappingUnknownField> = Vec::new();
    let mut total_key_code_points = 0u64;
    let mut entry_index = entry_start;
    proof {
        reveal(partition_profile1_typed_mapping_fields_with_policy_spec);
        reveal(crate::lower_typed::bind_profile1_typed_yaml_value_spec);
        reveal(typed_schema_kind_is_mapping_spec);
        reveal(typed_mapping_field_views_spec);
        assert(crate::lower_typed::bind_profile1_typed_yaml_value_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
        ) == Ok(mapping_binding@));
        assert(graph@.nodes[yaml_mapping_node_index as int] == node@);
        assert(schema@.schema.nodes[schema_mapping_node_index as int] == schema_node@);
        assert(node@.kind == crate::lower::CanonicalYamlNodeKind::Mapping);
        assert(typed_schema_kind_is_mapping_spec(schema_node@.kind));
        assert(schema_node@.field_start <= schema_node@.field_end);
        assert(schema_node@.field_end <= schema@.schema.fields.len());
        assert(field_start as nat == schema_node@.field_start as nat);
        assert(field_end as nat == schema_node@.field_end as nat);
        assert(entry_index as nat == node@.edge_start as nat);
        assert(entry_end as nat == node@.edge_end as nat);
        assert(field_limit == typed_mapping_effective_limit_spec(
            limits@.max_fields,
            MAX_PROFILE1_TYPED_MAPPING_FIELDS,
        ));
        assert(key_limit == typed_mapping_effective_limit_spec(
            limits@.max_key_code_points,
            MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
        ));
        assert(typed_mapping_field_views_spec(candidates@) == Seq::empty());
        reveal(typed_mapping_unknown_field_views_spec);
        assert(typed_mapping_unknown_field_views_spec(unknown_fields@) == Seq::empty());
        assert(total_key_code_points == 0);
        assert(expected == partition_profile1_typed_mapping_fields_with_policy_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            limits@,
        ));
        assert(partition_profile1_typed_mapping_fields_with_policy_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            limits@,
        ) == partition_after_scan_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            field_start as nat,
            field_end as nat,
            unknown_field_policy,
            mapping_binding@,
            scan_mapping_entries_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                entry_index as nat,
                entry_end as nat,
                field_start as nat,
                field_end as nat,
                field_limit,
                key_limit,
                unknown_field_policy,
                typed_mapping_field_views_spec(candidates@),
                typed_mapping_unknown_field_views_spec(unknown_fields@),
                total_key_code_points,
            ),
        ));
    }
    while entry_index < entry_end
        invariant
            entry_start <= entry_index <= entry_end <= entries.len(),
            field_start <= field_end <= schema_fields.len(),
            crate::lower::canonical_yaml_node_views_spec(nodes@) == graph@.nodes,
            crate::lower::canonical_mapping_entry_views_spec(entries@) == graph@.mapping_entries,
            crate::schema::typed_field_definition_views_spec(schema_fields@)
                == schema@.schema.fields,
            expected == partition_profile1_typed_mapping_fields_with_policy_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                unknown_field_policy,
                limits@,
            ),
            candidates.len() + unknown_fields.len() <= field_limit,
            total_key_code_points <= key_limit,
            expected == partition_after_scan_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                field_start as nat,
                field_end as nat,
                unknown_field_policy,
                mapping_binding@,
                scan_mapping_entries_spec(
                    graph@,
                    schema@,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    entry_index as nat,
                    entry_end as nat,
                    field_start as nat,
                    field_end as nat,
                    field_limit,
                    key_limit,
                    unknown_field_policy,
                    typed_mapping_field_views_spec(candidates@),
                    typed_mapping_unknown_field_views_spec(unknown_fields@),
                    total_key_code_points,
                ),
            ),
        decreases entry_end - entry_index,
    {
        let entry = &entries[entry_index];
        let key_node_index = entry.key_node_index();
        let key_start = if key_node_index < nodes.len() as u64 {
            nodes[key_node_index as usize].byte_start()
        } else {
            graph.source_len_bytes()
        };
        proof {
            reveal(crate::lower::canonical_mapping_entry_views_spec);
            assert(graph@.mapping_entries[entry_index as int] == entry@);
            if key_node_index < graph@.nodes.len() {
                reveal(crate::lower::canonical_yaml_node_views_spec);
                assert(graph@.nodes[key_node_index as int] == nodes[key_node_index as int]@);
            }
        }
        if entry.receiver_node_index() != yaml_mapping_node_index || key_node_index
            >= nodes.len() as u64 || entry.value_node_index() >= nodes.len() as u64 {
            let error = TypedMappingFieldError::at(
                TypedMappingFieldErrorKind::InternalInvariantViolation,
                key_start,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                Some(entry_index as u64),
                None,
                None,
            );
            proof {
                reveal(crate::lower::canonical_mapping_entry_views_spec);
                reveal(crate::lower::canonical_yaml_node_views_spec);
                reveal(scan_mapping_entries_spec);
                reveal(partition_after_scan_spec);
                reveal(typed_mapping_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let key = match mapping_key_for_node(graph, key_node_index) {
            Some(key) => key,
            None => {
                let error = TypedMappingFieldError::at(
                    TypedMappingFieldErrorKind::MappingKeyNotString,
                    key_start,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    Some(entry_index as u64),
                    None,
                    None,
                );
                proof {
                    reveal(crate::lower::canonical_mapping_entry_views_spec);
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(scan_mapping_entries_spec);
                    reveal(partition_after_scan_spec);
                    reveal(typed_mapping_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        let schema_field_index = match find_schema_field(
            schema_fields,
            field_start,
            field_end,
            &key,
        ) {
            Some(index) => index,
            None => {
                if unknown_field_policy == TypedMappingUnknownFieldPolicy::Reject {
                    let error = TypedMappingFieldError::at(
                        TypedMappingFieldErrorKind::UnknownField,
                        key_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        Some(entry_index as u64),
                        None,
                        None,
                    );
                    proof {
                        reveal(crate::lower::canonical_mapping_entry_views_spec);
                        reveal(crate::lower::canonical_yaml_node_views_spec);
                        reveal(scan_mapping_entries_spec);
                        reveal(partition_after_scan_spec);
                        reveal(typed_mapping_error_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                }
                if unknown_fields.len() as u64 >= field_limit || candidates.len() as u64
                    >= field_limit - unknown_fields.len() as u64 {
                    let error = TypedMappingFieldError::at(
                        TypedMappingFieldErrorKind::FieldLimitExceeded,
                        key_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        Some(entry_index as u64),
                        None,
                        None,
                    );
                    proof {
                        reveal(crate::lower::canonical_mapping_entry_views_spec);
                        reveal(scan_mapping_entries_spec);
                        reveal(partition_after_scan_spec);
                        reveal(typed_mapping_error_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                }
                if key.len() as u64 > key_limit - total_key_code_points {
                    let excluded = (key_limit - total_key_code_points) as usize;
                    let error = TypedMappingFieldError::at(
                        TypedMappingFieldErrorKind::KeyCodePointLimitExceeded,
                        key[excluded].byte_start,
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        Some(entry_index as u64),
                        None,
                        Some(excluded as u64),
                    );
                    proof {
                        reveal(crate::lower::canonical_mapping_entry_views_spec);
                        reveal(scan_mapping_entries_spec);
                        reveal(partition_after_scan_spec);
                        reveal(typed_mapping_error_spec);
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                }
                let code_points = copy_mapping_key_code_points(&key);
                let unknown = TypedMappingUnknownField::new(
                    entry_index as u64,
                    key_node_index,
                    entry.value_node_index(),
                    entry.inherited(),
                    nodes[key_node_index as usize].byte_start(),
                    nodes[key_node_index as usize].byte_end(),
                    code_points,
                );
                let ghost before = unknown_fields@;
                unknown_fields.push(unknown);
                total_key_code_points += key.len() as u64;
                proof {
                    reveal(crate::lower::canonical_mapping_entry_views_spec);
                    reveal(crate::lower::canonical_yaml_node_views_spec);
                    reveal(scan_mapping_entries_spec);
                    reveal(partition_after_scan_spec);
                    reveal(typed_mapping_unknown_field_views_spec);
                    assert(typed_mapping_unknown_field_views_spec(unknown_fields@)
                        == typed_mapping_unknown_field_views_spec(before).push(unknown@));
                }
                entry_index += 1;
                continue;
            },
        };
        proof {
            reveal(crate::schema::typed_field_definition_views_spec);
            assert(schema@.schema.fields[schema_field_index as int]
                == schema_fields[schema_field_index as int]@);
        }
        if candidate_for_schema_field(&candidates, schema_field_index).is_some() {
            let error = TypedMappingFieldError::at(
                TypedMappingFieldErrorKind::DuplicateRecognizedField,
                key_start,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                Some(entry_index as u64),
                Some(schema_field_index),
                None,
            );
            proof {
                reveal(crate::lower::canonical_mapping_entry_views_spec);
                reveal(crate::lower::canonical_yaml_node_views_spec);
                reveal(scan_mapping_entries_spec);
                reveal(partition_after_scan_spec);
                reveal(typed_mapping_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let schema_field = &schema_fields[schema_field_index as usize];
        let binding = match bind_profile1_typed_yaml_value(
            graph,
            schema,
            entry.value_node_index(),
            schema_field.value_schema_node_index(),
        ) {
            Ok(binding) => binding,
            Err(binding_error) => {
                let kind = if binding_error.kind()
                    == TypedValueBindingErrorKind::YamlValueKindMismatch {
                    TypedMappingFieldErrorKind::ValueKindMismatch
                } else {
                    TypedMappingFieldErrorKind::InternalInvariantViolation
                };
                let error = TypedMappingFieldError::at(
                    kind,
                    binding_error.byte_offset(),
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    Some(entry_index as u64),
                    Some(schema_field_index),
                    None,
                );
                proof {
                    reveal(crate::lower::canonical_mapping_entry_views_spec);
                    reveal(crate::schema::typed_field_definition_views_spec);
                    reveal(crate::lower_typed::bind_profile1_typed_yaml_value_spec);
                    reveal(scan_mapping_entries_spec);
                    reveal(partition_after_scan_spec);
                    reveal(typed_mapping_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if unknown_fields.len() as u64 >= field_limit || candidates.len() as u64 >= field_limit
            - unknown_fields.len() as u64 {
            let error = TypedMappingFieldError::at(
                TypedMappingFieldErrorKind::FieldLimitExceeded,
                key_start,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                Some(entry_index as u64),
                Some(schema_field_index),
                None,
            );
            proof {
                reveal(crate::lower::canonical_mapping_entry_views_spec);
                reveal(crate::schema::typed_field_definition_views_spec);
                reveal(scan_mapping_entries_spec);
                reveal(partition_after_scan_spec);
                reveal(typed_mapping_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if key.len() as u64 > key_limit - total_key_code_points {
            let excluded = (key_limit - total_key_code_points) as usize;
            let error = TypedMappingFieldError::at(
                TypedMappingFieldErrorKind::KeyCodePointLimitExceeded,
                key[excluded].byte_start,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                Some(entry_index as u64),
                Some(schema_field_index),
                Some(excluded as u64),
            );
            proof {
                reveal(crate::lower::canonical_mapping_entry_views_spec);
                reveal(crate::schema::typed_field_definition_views_spec);
                reveal(scan_mapping_entries_spec);
                reveal(partition_after_scan_spec);
                reveal(typed_mapping_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let candidate = TypedMappingField::new(
            entry_index as u64,
            schema_field_index,
            schema_field.field_id(),
            key_node_index,
            entry.value_node_index(),
            entry.inherited(),
            binding,
        );
        let ghost before = candidates@;
        candidates.push(candidate);
        total_key_code_points += key.len() as u64;
        proof {
            reveal(crate::lower::canonical_mapping_entry_views_spec);
            reveal(crate::lower::canonical_yaml_node_views_spec);
            reveal(crate::schema::typed_field_definition_views_spec);
            reveal(crate::lower_typed::bind_profile1_typed_yaml_value_spec);
            reveal(scan_mapping_entries_spec);
            reveal(partition_after_scan_spec);
            reveal(typed_mapping_field_views_spec);
            assert(typed_mapping_field_views_spec(candidates@) == typed_mapping_field_views_spec(
                before,
            ).push(candidate@));
        }
        entry_index += 1;
    }
    proof {
        reveal(scan_mapping_entries_spec);
        reveal(partition_after_scan_spec);
        assert(scan_mapping_entries_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            entry_index as nat,
            entry_end as nat,
            field_start as nat,
            field_end as nat,
            field_limit,
            key_limit,
            unknown_field_policy,
            typed_mapping_field_views_spec(candidates@),
            typed_mapping_unknown_field_views_spec(unknown_fields@),
            total_key_code_points,
        ) == Ok(
            (
                typed_mapping_field_views_spec(candidates@),
                typed_mapping_unknown_field_views_spec(unknown_fields@),
                total_key_code_points,
            ),
        ));
    }
    let mut output: Vec<TypedMappingField> = Vec::new();
    let mut schema_field_index = field_start;
    proof {
        reveal(typed_mapping_field_views_spec);
        assert(partition_after_scan_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            field_start as nat,
            field_end as nat,
            unknown_field_policy,
            mapping_binding@,
            scan_mapping_entries_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                entry_index as nat,
                entry_end as nat,
                field_start as nat,
                field_end as nat,
                field_limit,
                key_limit,
                unknown_field_policy,
                typed_mapping_field_views_spec(candidates@),
                typed_mapping_unknown_field_views_spec(unknown_fields@),
                total_key_code_points,
            ),
        ) == partition_after_emission_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            mapping_binding@,
            total_key_code_points,
            typed_mapping_unknown_field_views_spec(unknown_fields@),
            emit_schema_order_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                schema_field_index as nat,
                field_end as nat,
                typed_mapping_field_views_spec(candidates@),
                typed_mapping_field_views_spec(output@),
            ),
        ));
        assert(expected == partition_after_emission_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            unknown_field_policy,
            mapping_binding@,
            total_key_code_points,
            typed_mapping_unknown_field_views_spec(unknown_fields@),
            emit_schema_order_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                schema_field_index as nat,
                field_end as nat,
                typed_mapping_field_views_spec(candidates@),
                typed_mapping_field_views_spec(output@),
            ),
        ));
    }
    while schema_field_index < field_end
        invariant
            field_start <= schema_field_index <= field_end <= schema_fields.len(),
            crate::lower::canonical_yaml_node_views_spec(nodes@) == graph@.nodes,
            graph@.nodes[yaml_mapping_node_index as int] == node@,
            crate::schema::typed_field_definition_views_spec(schema_fields@)
                == schema@.schema.fields,
            expected == partition_profile1_typed_mapping_fields_with_policy_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                unknown_field_policy,
                limits@,
            ),
            expected == partition_after_emission_spec(
                graph@,
                schema@,
                yaml_mapping_node_index,
                schema_mapping_node_index,
                unknown_field_policy,
                mapping_binding@,
                total_key_code_points,
                typed_mapping_unknown_field_views_spec(unknown_fields@),
                emit_schema_order_spec(
                    graph@,
                    schema@,
                    yaml_mapping_node_index,
                    schema_mapping_node_index,
                    schema_field_index as nat,
                    field_end as nat,
                    typed_mapping_field_views_spec(candidates@),
                    typed_mapping_field_views_spec(output@),
                ),
            ),
        decreases field_end - schema_field_index,
    {
        let candidate = candidate_for_schema_field(&candidates, schema_field_index as u64);
        proof {
            reveal(crate::schema::typed_field_definition_views_spec);
            assert(schema@.schema.fields[schema_field_index as int]
                == schema_fields[schema_field_index as int]@);
        }
        match candidate {
            Some(field) => {
                let ghost before = output@;
                output.push(field);
                proof {
                    reveal(emit_schema_order_spec);
                    reveal(partition_after_emission_spec);
                    reveal(typed_mapping_field_views_spec);
                    assert(typed_mapping_field_views_spec(output@)
                        == typed_mapping_field_views_spec(before).push(field@));
                }
            },
            None => {
                if schema_fields[schema_field_index].required() {
                    let error = TypedMappingFieldError::at(
                        TypedMappingFieldErrorKind::MissingRequiredField,
                        node.byte_start(),
                        yaml_mapping_node_index,
                        schema_mapping_node_index,
                        None,
                        Some(schema_field_index as u64),
                        None,
                    );
                    proof {
                        reveal(crate::lower::canonical_yaml_node_views_spec);
                        reveal(crate::schema::typed_field_definition_views_spec);
                        assert(candidate_for_schema_field_spec(
                            typed_mapping_field_views_spec(candidates@),
                            schema_field_index as u64,
                        ).is_none());
                        assert(schema@.schema.fields[schema_field_index as int].required);
                        assert(graph@.nodes[yaml_mapping_node_index as int].byte_start
                            == node@.byte_start);
                        reveal(emit_schema_order_spec);
                        reveal(partition_after_emission_spec);
                        reveal(typed_mapping_error_spec);
                        assert(emit_schema_order_spec(
                            graph@,
                            schema@,
                            yaml_mapping_node_index,
                            schema_mapping_node_index,
                            schema_field_index as nat,
                            field_end as nat,
                            typed_mapping_field_views_spec(candidates@),
                            typed_mapping_field_views_spec(output@),
                        ) == Err(error@));
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                }
                proof {
                    reveal(emit_schema_order_spec);
                    reveal(partition_after_emission_spec);
                }
            },
        }
        schema_field_index += 1;
    }
    proof {
        reveal(emit_schema_order_spec);
        reveal(partition_after_emission_spec);
    }
    let partition = TypedMappingFieldPartition::new(
        graph.profile_version(),
        schema.schema_version(),
        yaml_mapping_node_index,
        schema_mapping_node_index,
        unknown_field_policy,
        mapping_binding,
        total_key_code_points,
        output,
        unknown_fields,
    );
    Ok(partition)
}

pub fn partition_profile1_typed_mapping_fields(
    graph: &CanonicalYamlGraphSource,
    schema: &CompiledTypedFieldSchema,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    limits: TypedMappingFieldLimits,
) -> (result: Result<TypedMappingFieldPartition, TypedMappingFieldError>)
    ensures
        partition_profile1_typed_mapping_fields_spec(
            graph@,
            schema@,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            limits@,
        ) == match result {
            Ok(partition) => Ok(partition@),
            Err(error) => Err(error@),
        },
{
    proof {
        reveal(partition_profile1_typed_mapping_fields_spec);
    }
    partition_profile1_typed_mapping_fields_with_policy(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        TypedMappingUnknownFieldPolicy::Reject,
        limits,
    )
}

} // verus!
