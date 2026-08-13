//! Verified compilation of versioned typed-field schemas.
//!
//! The compiler authenticates the schema graph consumed by schema-directed YAML lowering. It
//! preserves exact stable field identities and Unicode names, validates nested sequence and
//! mapping references, and rejects ambiguous field tables before any configuration is lowered.
use vstd::prelude::*;

verus! {

pub const TYPED_FIELD_SCHEMA_VERSION: u16 = 1;

pub const TYPED_FIELD_SCHEMA_COMPILATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_TYPED_SCHEMA_NODES: u64 = 1_048_576;

pub const MAX_PROFILE1_TYPED_SCHEMA_FIELDS: u64 = 1_048_576;

pub const MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedFieldSchemaLimits {
    max_schema_nodes: u64,
    max_schema_fields: u64,
    max_field_name_code_points: u64,
}

#[verifier::ext_equal]
pub struct TypedFieldSchemaLimitsView {
    pub max_schema_nodes: u64,
    pub max_schema_fields: u64,
    pub max_field_name_code_points: u64,
}

impl View for TypedFieldSchemaLimits {
    type V = TypedFieldSchemaLimitsView;

    closed spec fn view(&self) -> TypedFieldSchemaLimitsView {
        TypedFieldSchemaLimitsView {
            max_schema_nodes: self.max_schema_nodes,
            max_schema_fields: self.max_schema_fields,
            max_field_name_code_points: self.max_field_name_code_points,
        }
    }
}

impl TypedFieldSchemaLimits {
    pub fn new(
        max_schema_nodes: u64,
        max_schema_fields: u64,
        max_field_name_code_points: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (TypedFieldSchemaLimitsView {
                max_schema_nodes,
                max_schema_fields,
                max_field_name_code_points,
            }),
    {
        Self { max_schema_nodes, max_schema_fields, max_field_name_code_points }
    }

    pub fn max_schema_nodes(&self) -> (value: u64)
        ensures
            value == self@.max_schema_nodes,
    {
        self.max_schema_nodes
    }

    pub fn max_schema_fields(&self) -> (value: u64)
        ensures
            value == self@.max_schema_fields,
    {
        self.max_schema_fields
    }

    pub fn max_field_name_code_points(&self) -> (value: u64)
        ensures
            value == self@.max_field_name_code_points,
    {
        self.max_field_name_code_points
    }
}

pub fn canonical_typed_field_schema_limits() -> (limits: TypedFieldSchemaLimits)
    ensures
        limits@ == canonical_typed_field_schema_limits_spec(),
{
    TypedFieldSchemaLimits::new(
        MAX_PROFILE1_TYPED_SCHEMA_NODES,
        MAX_PROFILE1_TYPED_SCHEMA_FIELDS,
        MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
    )
}

pub open spec fn canonical_typed_field_schema_limits_spec() -> TypedFieldSchemaLimitsView {
    TypedFieldSchemaLimitsView {
        max_schema_nodes: MAX_PROFILE1_TYPED_SCHEMA_NODES,
        max_schema_fields: MAX_PROFILE1_TYPED_SCHEMA_FIELDS,
        max_field_name_code_points: MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
    }
}

pub open spec fn typed_field_schema_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

pub open spec fn effective_typed_schema_node_limit_spec(limits: TypedFieldSchemaLimitsView) -> u64 {
    typed_field_schema_effective_limit_spec(
        limits.max_schema_nodes,
        MAX_PROFILE1_TYPED_SCHEMA_NODES,
    )
}

pub open spec fn effective_typed_schema_field_limit_spec(
    limits: TypedFieldSchemaLimitsView,
) -> u64 {
    typed_field_schema_effective_limit_spec(
        limits.max_schema_fields,
        MAX_PROFILE1_TYPED_SCHEMA_FIELDS,
    )
}

pub open spec fn effective_typed_schema_name_limit_spec(limits: TypedFieldSchemaLimitsView) -> u64 {
    typed_field_schema_effective_limit_spec(
        limits.max_field_name_code_points,
        MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
    )
}

fn effective_limit(requested: u64, absolute: u64) -> (value: u64)
    ensures
        value == typed_field_schema_effective_limit_spec(requested, absolute),
        value <= absolute,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum TypedSchemaValueKind {
    Null,
    Boolean,
    Integer,
    FiniteFloat,
    PositiveInfinity,
    NegativeInfinity,
    NotANumber,
    String,
    CustomScalar,
    Sequence,
    CustomSequence,
    Mapping,
    CustomMapping,
}

pub open spec fn typed_schema_kind_is_mapping_spec(kind: TypedSchemaValueKind) -> bool {
    kind == TypedSchemaValueKind::Mapping || kind == TypedSchemaValueKind::CustomMapping
}

pub open spec fn typed_schema_kind_is_sequence_spec(kind: TypedSchemaValueKind) -> bool {
    kind == TypedSchemaValueKind::Sequence || kind == TypedSchemaValueKind::CustomSequence
}

pub open spec fn typed_schema_kind_is_scalar_spec(kind: TypedSchemaValueKind) -> bool {
    !typed_schema_kind_is_mapping_spec(kind) && !typed_schema_kind_is_sequence_spec(kind)
}

fn typed_schema_kind_is_mapping(kind: TypedSchemaValueKind) -> (value: bool)
    ensures
        value == typed_schema_kind_is_mapping_spec(kind),
{
    kind == TypedSchemaValueKind::Mapping || kind == TypedSchemaValueKind::CustomMapping
}

fn typed_schema_kind_is_sequence(kind: TypedSchemaValueKind) -> (value: bool)
    ensures
        value == typed_schema_kind_is_sequence_spec(kind),
{
    kind == TypedSchemaValueKind::Sequence || kind == TypedSchemaValueKind::CustomSequence
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedSchemaNode {
    kind: TypedSchemaValueKind,
    field_start: u64,
    field_end: u64,
    sequence_item_schema_node_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct TypedSchemaNodeView {
    pub kind: TypedSchemaValueKind,
    pub field_start: u64,
    pub field_end: u64,
    pub sequence_item_schema_node_index: Option<u64>,
}

impl View for TypedSchemaNode {
    type V = TypedSchemaNodeView;

    closed spec fn view(&self) -> TypedSchemaNodeView {
        TypedSchemaNodeView {
            kind: self.kind,
            field_start: self.field_start,
            field_end: self.field_end,
            sequence_item_schema_node_index: self.sequence_item_schema_node_index,
        }
    }
}

impl TypedSchemaNode {
    pub fn new(
        kind: TypedSchemaValueKind,
        field_start: u64,
        field_end: u64,
        sequence_item_schema_node_index: Option<u64>,
    ) -> (node: Self)
        ensures
            node@ == (TypedSchemaNodeView {
                kind,
                field_start,
                field_end,
                sequence_item_schema_node_index,
            }),
    {
        Self { kind, field_start, field_end, sequence_item_schema_node_index }
    }

    pub fn kind(&self) -> (value: TypedSchemaValueKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn field_start(&self) -> (value: u64)
        ensures
            value == self@.field_start,
    {
        self.field_start
    }

    pub fn field_end(&self) -> (value: u64)
        ensures
            value == self@.field_end,
    {
        self.field_end
    }

    pub fn sequence_item_schema_node_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.sequence_item_schema_node_index,
    {
        self.sequence_item_schema_node_index
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypedFieldDefinition {
    owner_schema_node_index: u64,
    field_id: u64,
    name: Vec<u32>,
    value_schema_node_index: u64,
    required: bool,
}

#[verifier::ext_equal]
pub struct TypedFieldDefinitionView {
    pub owner_schema_node_index: u64,
    pub field_id: u64,
    pub name: Seq<u32>,
    pub value_schema_node_index: u64,
    pub required: bool,
}

impl View for TypedFieldDefinition {
    type V = TypedFieldDefinitionView;

    closed spec fn view(&self) -> TypedFieldDefinitionView {
        TypedFieldDefinitionView {
            owner_schema_node_index: self.owner_schema_node_index,
            field_id: self.field_id,
            name: self.name@,
            value_schema_node_index: self.value_schema_node_index,
            required: self.required,
        }
    }
}

impl TypedFieldDefinition {
    pub fn new(
        owner_schema_node_index: u64,
        field_id: u64,
        name: Vec<u32>,
        value_schema_node_index: u64,
        required: bool,
    ) -> (field: Self)
        ensures
            field@ == (TypedFieldDefinitionView {
                owner_schema_node_index,
                field_id,
                name: name@,
                value_schema_node_index,
                required,
            }),
    {
        Self { owner_schema_node_index, field_id, name, value_schema_node_index, required }
    }

    pub fn owner_schema_node_index(&self) -> (value: u64)
        ensures
            value == self@.owner_schema_node_index,
    {
        self.owner_schema_node_index
    }

    pub fn field_id(&self) -> (value: u64)
        ensures
            value == self@.field_id,
    {
        self.field_id
    }

    pub fn name(&self) -> (value: &[u32])
        ensures
            value@ == self@.name,
    {
        self.name.as_slice()
    }

    pub fn value_schema_node_index(&self) -> (value: u64)
        ensures
            value == self@.value_schema_node_index,
    {
        self.value_schema_node_index
    }

    pub fn required(&self) -> (value: bool)
        ensures
            value == self@.required,
    {
        self.required
    }
}

pub open spec fn typed_schema_node_views_spec(values: Seq<TypedSchemaNode>) -> Seq<
    TypedSchemaNodeView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn typed_field_definition_views_spec(values: Seq<TypedFieldDefinition>) -> Seq<
    TypedFieldDefinitionView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypedFieldSchema {
    schema_version: u16,
    root_schema_node_index: u64,
    nodes: Vec<TypedSchemaNode>,
    fields: Vec<TypedFieldDefinition>,
}

#[verifier::ext_equal]
pub struct TypedFieldSchemaView {
    pub schema_version: u16,
    pub root_schema_node_index: u64,
    pub nodes: Seq<TypedSchemaNodeView>,
    pub fields: Seq<TypedFieldDefinitionView>,
}

impl View for TypedFieldSchema {
    type V = TypedFieldSchemaView;

    closed spec fn view(&self) -> TypedFieldSchemaView {
        TypedFieldSchemaView {
            schema_version: self.schema_version,
            root_schema_node_index: self.root_schema_node_index,
            nodes: typed_schema_node_views_spec(self.nodes@),
            fields: typed_field_definition_views_spec(self.fields@),
        }
    }
}

impl TypedFieldSchema {
    pub fn new(
        schema_version: u16,
        root_schema_node_index: u64,
        nodes: Vec<TypedSchemaNode>,
        fields: Vec<TypedFieldDefinition>,
    ) -> (schema: Self)
        ensures
            schema@ == (TypedFieldSchemaView {
                schema_version,
                root_schema_node_index,
                nodes: typed_schema_node_views_spec(nodes@),
                fields: typed_field_definition_views_spec(fields@),
            }),
    {
        Self { schema_version, root_schema_node_index, nodes, fields }
    }

    pub fn schema_version(&self) -> (value: u16)
        ensures
            value == self@.schema_version,
    {
        self.schema_version
    }

    pub fn root_schema_node_index(&self) -> (value: u64)
        ensures
            value == self@.root_schema_node_index,
    {
        self.root_schema_node_index
    }

    pub fn nodes(&self) -> (values: &[TypedSchemaNode])
        ensures
            typed_schema_node_views_spec(values@) == self@.nodes,
    {
        self.nodes.as_slice()
    }

    pub fn fields(&self) -> (values: &[TypedFieldDefinition])
        ensures
            typed_field_definition_views_spec(values@) == self@.fields,
    {
        self.fields.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum TypedFieldSchemaErrorKind {
    UnsupportedSchemaVersion,
    EmptySchema,
    SchemaNodeLimitExceeded,
    SchemaFieldLimitExceeded,
    InvalidRootSchemaNode,
    InvalidSchemaNodeShape,
    InvalidFieldPartition,
    InvalidSequenceItemSchemaNode,
    InvalidFieldOwner,
    InvalidFieldValueSchemaNode,
    InvalidFieldId,
    EmptyFieldName,
    InvalidFieldNameCodePoint,
    FieldNameCodePointLimitExceeded,
    DuplicateFieldId,
    DuplicateFieldName,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedFieldSchemaError {
    kind: TypedFieldSchemaErrorKind,
    schema_node_index: Option<u64>,
    schema_field_index: Option<u64>,
    name_code_point_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct TypedFieldSchemaErrorView {
    pub kind: TypedFieldSchemaErrorKind,
    pub schema_node_index: Option<u64>,
    pub schema_field_index: Option<u64>,
    pub name_code_point_index: Option<u64>,
}

impl View for TypedFieldSchemaError {
    type V = TypedFieldSchemaErrorView;

    closed spec fn view(&self) -> TypedFieldSchemaErrorView {
        TypedFieldSchemaErrorView {
            kind: self.kind,
            schema_node_index: self.schema_node_index,
            schema_field_index: self.schema_field_index,
            name_code_point_index: self.name_code_point_index,
        }
    }
}

impl TypedFieldSchemaError {
    fn at(
        kind: TypedFieldSchemaErrorKind,
        schema_node_index: Option<u64>,
        schema_field_index: Option<u64>,
        name_code_point_index: Option<u64>,
    ) -> (error: Self)
        ensures
            error@ == (TypedFieldSchemaErrorView {
                kind,
                schema_node_index,
                schema_field_index,
                name_code_point_index,
            }),
    {
        Self { kind, schema_node_index, schema_field_index, name_code_point_index }
    }

    pub fn kind(&self) -> (value: TypedFieldSchemaErrorKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn schema_node_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.schema_node_index,
    {
        self.schema_node_index
    }

    pub fn schema_field_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.schema_field_index,
    {
        self.schema_field_index
    }

    pub fn name_code_point_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.name_code_point_index,
    {
        self.name_code_point_index
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompiledTypedFieldSchema {
    schema_version: u16,
    compilation_version: u16,
    root_schema_node_index: u64,
    node_count: u64,
    field_count: u64,
    total_field_name_code_points: u64,
    schema: TypedFieldSchema,
}

#[verifier::ext_equal]
pub struct CompiledTypedFieldSchemaView {
    pub schema_version: u16,
    pub compilation_version: u16,
    pub root_schema_node_index: u64,
    pub node_count: u64,
    pub field_count: u64,
    pub total_field_name_code_points: u64,
    pub schema: TypedFieldSchemaView,
}

impl View for CompiledTypedFieldSchema {
    type V = CompiledTypedFieldSchemaView;

    closed spec fn view(&self) -> CompiledTypedFieldSchemaView {
        CompiledTypedFieldSchemaView {
            schema_version: self.schema_version,
            compilation_version: self.compilation_version,
            root_schema_node_index: self.root_schema_node_index,
            node_count: self.node_count,
            field_count: self.field_count,
            total_field_name_code_points: self.total_field_name_code_points,
            schema: self.schema@,
        }
    }
}

impl CompiledTypedFieldSchema {
    fn new(schema: TypedFieldSchema, total_field_name_code_points: u64) -> (compiled: Self)
        ensures
            compiled@ == (CompiledTypedFieldSchemaView {
                schema_version: schema@.schema_version,
                compilation_version: TYPED_FIELD_SCHEMA_COMPILATION_VERSION,
                root_schema_node_index: schema@.root_schema_node_index,
                node_count: schema@.nodes.len() as u64,
                field_count: schema@.fields.len() as u64,
                total_field_name_code_points,
                schema: schema@,
            }),
    {
        let schema_version = schema.schema_version();
        let root_schema_node_index = schema.root_schema_node_index();
        let node_count = schema.nodes().len() as u64;
        let field_count = schema.fields().len() as u64;
        Self {
            schema_version,
            compilation_version: TYPED_FIELD_SCHEMA_COMPILATION_VERSION,
            root_schema_node_index,
            node_count,
            field_count,
            total_field_name_code_points,
            schema,
        }
    }

    pub fn schema_version(&self) -> (value: u16)
        ensures
            value == self@.schema_version,
    {
        self.schema_version
    }

    pub fn compilation_version(&self) -> (value: u16)
        ensures
            value == self@.compilation_version,
    {
        self.compilation_version
    }

    pub fn root_schema_node_index(&self) -> (value: u64)
        ensures
            value == self@.root_schema_node_index,
    {
        self.root_schema_node_index
    }

    pub fn node_count(&self) -> (value: u64)
        ensures
            value == self@.node_count,
    {
        self.node_count
    }

    pub fn field_count(&self) -> (value: u64)
        ensures
            value == self@.field_count,
    {
        self.field_count
    }

    pub fn total_field_name_code_points(&self) -> (value: u64)
        ensures
            value == self@.total_field_name_code_points,
    {
        self.total_field_name_code_points
    }

    pub fn schema(&self) -> (value: &TypedFieldSchema)
        ensures
            value@ == self@.schema,
    {
        &self.schema
    }
}

pub open spec fn unicode_scalar_value_spec(code_point: u32) -> bool {
    code_point <= 0x10ffff && !(0xd800 <= code_point <= 0xdfff)
}

#[allow(clippy::manual_range_contains)]
fn unicode_scalar_value(code_point: u32) -> (valid: bool)
    ensures
        valid == unicode_scalar_value_spec(code_point),
{
    code_point <= 0x10ffff && !(0xd800 <= code_point && code_point <= 0xdfff)
}

pub closed spec fn validate_field_owners_tail_spec(
    fields: Seq<TypedFieldDefinitionView>,
    index: nat,
    end: nat,
    owner_schema_node_index: u64,
) -> Result<(), TypedFieldSchemaErrorView>
    decreases end - index,
{
    if index > end || end > fields.len() {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InternalInvariantViolation,
                schema_node_index: Some(owner_schema_node_index),
                schema_field_index: Some(index as u64),
                name_code_point_index: None,
            },
        )
    } else if index == end {
        Ok(())
    } else if fields[index as int].owner_schema_node_index != owner_schema_node_index {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InvalidFieldOwner,
                schema_node_index: Some(owner_schema_node_index),
                schema_field_index: Some(index as u64),
                name_code_point_index: None,
            },
        )
    } else {
        validate_field_owners_tail_spec(fields, (index + 1) as nat, end, owner_schema_node_index)
    }
}

pub closed spec fn validate_schema_nodes_tail_spec(
    nodes: Seq<TypedSchemaNodeView>,
    fields: Seq<TypedFieldDefinitionView>,
    node_index: nat,
    field_cursor: nat,
) -> Result<u64, TypedFieldSchemaErrorView>
    decreases nodes.len() - node_index,
{
    if node_index > nodes.len() || field_cursor > fields.len() {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InternalInvariantViolation,
                schema_node_index: Some(node_index as u64),
                schema_field_index: Some(field_cursor as u64),
                name_code_point_index: None,
            },
        )
    } else if node_index == nodes.len() {
        Ok(field_cursor as u64)
    } else {
        let node = nodes[node_index as int];
        if typed_schema_kind_is_mapping_spec(node.kind) {
            if node.sequence_item_schema_node_index.is_some() {
                Err(
                    TypedFieldSchemaErrorView {
                        kind: TypedFieldSchemaErrorKind::InvalidSchemaNodeShape,
                        schema_node_index: Some(node_index as u64),
                        schema_field_index: None,
                        name_code_point_index: None,
                    },
                )
            } else if node.field_start != field_cursor as u64 || node.field_start > node.field_end
                || node.field_end > fields.len() {
                Err(
                    TypedFieldSchemaErrorView {
                        kind: TypedFieldSchemaErrorKind::InvalidFieldPartition,
                        schema_node_index: Some(node_index as u64),
                        schema_field_index: Some(field_cursor as u64),
                        name_code_point_index: None,
                    },
                )
            } else {
                match validate_field_owners_tail_spec(
                    fields,
                    node.field_start as nat,
                    node.field_end as nat,
                    node_index as u64,
                ) {
                    Err(error) => Err(error),
                    Ok(()) => validate_schema_nodes_tail_spec(
                        nodes,
                        fields,
                        (node_index + 1) as nat,
                        node.field_end as nat,
                    ),
                }
            }
        } else if typed_schema_kind_is_sequence_spec(node.kind) {
            if node.field_start != 0 || node.field_end != 0
                || node.sequence_item_schema_node_index.is_none() {
                Err(
                    TypedFieldSchemaErrorView {
                        kind: TypedFieldSchemaErrorKind::InvalidSchemaNodeShape,
                        schema_node_index: Some(node_index as u64),
                        schema_field_index: None,
                        name_code_point_index: None,
                    },
                )
            } else {
                match node.sequence_item_schema_node_index {
                    None => Err(
                        TypedFieldSchemaErrorView {
                            kind: TypedFieldSchemaErrorKind::InternalInvariantViolation,
                            schema_node_index: Some(node_index as u64),
                            schema_field_index: None,
                            name_code_point_index: None,
                        },
                    ),
                    Some(item) => if item >= nodes.len() {
                        Err(
                            TypedFieldSchemaErrorView {
                                kind: TypedFieldSchemaErrorKind::InvalidSequenceItemSchemaNode,
                                schema_node_index: Some(node_index as u64),
                                schema_field_index: None,
                                name_code_point_index: None,
                            },
                        )
                    } else {
                        validate_schema_nodes_tail_spec(
                            nodes,
                            fields,
                            (node_index + 1) as nat,
                            field_cursor,
                        )
                    },
                }
            }
        } else if node.field_start != 0 || node.field_end != 0
            || node.sequence_item_schema_node_index.is_some() {
            Err(
                TypedFieldSchemaErrorView {
                    kind: TypedFieldSchemaErrorKind::InvalidSchemaNodeShape,
                    schema_node_index: Some(node_index as u64),
                    schema_field_index: None,
                    name_code_point_index: None,
                },
            )
        } else {
            validate_schema_nodes_tail_spec(nodes, fields, (node_index + 1) as nat, field_cursor)
        }
    }
}

pub closed spec fn validate_field_name_tail_spec(
    name: Seq<u32>,
    name_index: nat,
    total: u64,
    limit: u64,
    field_index: u64,
) -> Result<u64, TypedFieldSchemaErrorView>
    decreases name.len() - name_index,
{
    if name_index > name.len() {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InternalInvariantViolation,
                schema_node_index: None,
                schema_field_index: Some(field_index),
                name_code_point_index: Some(name_index as u64),
            },
        )
    } else if name_index == name.len() {
        Ok(total)
    } else if !unicode_scalar_value_spec(name[name_index as int]) {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InvalidFieldNameCodePoint,
                schema_node_index: None,
                schema_field_index: Some(field_index),
                name_code_point_index: Some(name_index as u64),
            },
        )
    } else if total >= limit {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::FieldNameCodePointLimitExceeded,
                schema_node_index: None,
                schema_field_index: Some(field_index),
                name_code_point_index: Some(name_index as u64),
            },
        )
    } else {
        validate_field_name_tail_spec(
            name,
            (name_index + 1) as nat,
            (total + 1) as u64,
            limit,
            field_index,
        )
    }
}

pub closed spec fn prior_field_id_duplicate_tail_spec(
    fields: Seq<TypedFieldDefinitionView>,
    field_index: nat,
    prior_index: nat,
) -> bool
    decreases field_index - prior_index,
{
    if prior_index >= field_index || field_index >= fields.len() {
        false
    } else if fields[prior_index as int].field_id == fields[field_index as int].field_id {
        true
    } else {
        prior_field_id_duplicate_tail_spec(fields, field_index, (prior_index + 1) as nat)
    }
}

pub closed spec fn prior_field_name_duplicate_tail_spec(
    fields: Seq<TypedFieldDefinitionView>,
    field_index: nat,
    prior_index: nat,
) -> bool
    decreases field_index - prior_index,
{
    if prior_index >= field_index || field_index >= fields.len() {
        false
    } else if fields[prior_index as int].owner_schema_node_index
        == fields[field_index as int].owner_schema_node_index && fields[prior_index as int].name
        == fields[field_index as int].name {
        true
    } else {
        prior_field_name_duplicate_tail_spec(fields, field_index, (prior_index + 1) as nat)
    }
}

pub closed spec fn validate_schema_fields_tail_spec(
    nodes: Seq<TypedSchemaNodeView>,
    fields: Seq<TypedFieldDefinitionView>,
    field_index: nat,
    total_name_code_points: u64,
    name_limit: u64,
) -> Result<u64, TypedFieldSchemaErrorView>
    decreases fields.len() - field_index,
{
    if field_index > fields.len() {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InternalInvariantViolation,
                schema_node_index: None,
                schema_field_index: Some(field_index as u64),
                name_code_point_index: None,
            },
        )
    } else if field_index == fields.len() {
        Ok(total_name_code_points)
    } else {
        let field = fields[field_index as int];
        if field.field_id == 0 {
            Err(
                TypedFieldSchemaErrorView {
                    kind: TypedFieldSchemaErrorKind::InvalidFieldId,
                    schema_node_index: Some(field.owner_schema_node_index),
                    schema_field_index: Some(field_index as u64),
                    name_code_point_index: None,
                },
            )
        } else if field.value_schema_node_index >= nodes.len() {
            Err(
                TypedFieldSchemaErrorView {
                    kind: TypedFieldSchemaErrorKind::InvalidFieldValueSchemaNode,
                    schema_node_index: Some(field.value_schema_node_index),
                    schema_field_index: Some(field_index as u64),
                    name_code_point_index: None,
                },
            )
        } else if field.name.len() == 0 {
            Err(
                TypedFieldSchemaErrorView {
                    kind: TypedFieldSchemaErrorKind::EmptyFieldName,
                    schema_node_index: Some(field.owner_schema_node_index),
                    schema_field_index: Some(field_index as u64),
                    name_code_point_index: None,
                },
            )
        } else {
            match validate_field_name_tail_spec(
                field.name,
                0,
                total_name_code_points,
                name_limit,
                field_index as u64,
            ) {
                Err(error) => Err(error),
                Ok(next_total) => if prior_field_id_duplicate_tail_spec(fields, field_index, 0) {
                    Err(
                        TypedFieldSchemaErrorView {
                            kind: TypedFieldSchemaErrorKind::DuplicateFieldId,
                            schema_node_index: Some(field.owner_schema_node_index),
                            schema_field_index: Some(field_index as u64),
                            name_code_point_index: None,
                        },
                    )
                } else if prior_field_name_duplicate_tail_spec(fields, field_index, 0) {
                    Err(
                        TypedFieldSchemaErrorView {
                            kind: TypedFieldSchemaErrorKind::DuplicateFieldName,
                            schema_node_index: Some(field.owner_schema_node_index),
                            schema_field_index: Some(field_index as u64),
                            name_code_point_index: None,
                        },
                    )
                } else {
                    validate_schema_fields_tail_spec(
                        nodes,
                        fields,
                        (field_index + 1) as nat,
                        next_total,
                        name_limit,
                    )
                },
            }
        }
    }
}

pub open spec fn compile_typed_field_schema_spec(
    schema: TypedFieldSchemaView,
    limits: TypedFieldSchemaLimitsView,
) -> Result<CompiledTypedFieldSchemaView, TypedFieldSchemaErrorView> {
    let node_limit = effective_typed_schema_node_limit_spec(limits);
    let field_limit = effective_typed_schema_field_limit_spec(limits);
    let name_limit = effective_typed_schema_name_limit_spec(limits);
    if schema.schema_version != TYPED_FIELD_SCHEMA_VERSION {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::UnsupportedSchemaVersion,
                schema_node_index: None,
                schema_field_index: None,
                name_code_point_index: None,
            },
        )
    } else if schema.nodes.len() == 0 {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::EmptySchema,
                schema_node_index: Some(0),
                schema_field_index: None,
                name_code_point_index: None,
            },
        )
    } else if schema.nodes.len() > node_limit {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::SchemaNodeLimitExceeded,
                schema_node_index: Some(node_limit),
                schema_field_index: None,
                name_code_point_index: None,
            },
        )
    } else if schema.fields.len() > field_limit {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::SchemaFieldLimitExceeded,
                schema_node_index: None,
                schema_field_index: Some(field_limit),
                name_code_point_index: None,
            },
        )
    } else if schema.root_schema_node_index >= schema.nodes.len() {
        Err(
            TypedFieldSchemaErrorView {
                kind: TypedFieldSchemaErrorKind::InvalidRootSchemaNode,
                schema_node_index: Some(schema.root_schema_node_index),
                schema_field_index: None,
                name_code_point_index: None,
            },
        )
    } else {
        match validate_schema_nodes_tail_spec(schema.nodes, schema.fields, 0, 0) {
            Err(error) => Err(error),
            Ok(field_cursor) => if field_cursor != schema.fields.len() {
                Err(
                    TypedFieldSchemaErrorView {
                        kind: TypedFieldSchemaErrorKind::InvalidFieldPartition,
                        schema_node_index: None,
                        schema_field_index: Some(field_cursor),
                        name_code_point_index: None,
                    },
                )
            } else {
                match validate_schema_fields_tail_spec(
                    schema.nodes,
                    schema.fields,
                    0,
                    0,
                    name_limit,
                ) {
                    Err(error) => Err(error),
                    Ok(total) => Ok(
                        CompiledTypedFieldSchemaView {
                            schema_version: schema.schema_version,
                            compilation_version: TYPED_FIELD_SCHEMA_COMPILATION_VERSION,
                            root_schema_node_index: schema.root_schema_node_index,
                            node_count: schema.nodes.len() as u64,
                            field_count: schema.fields.len() as u64,
                            total_field_name_code_points: total,
                            schema,
                        },
                    ),
                }
            },
        }
    }
}

fn validate_field_owners(
    fields: &[TypedFieldDefinition],
    start: u64,
    end: u64,
    owner_schema_node_index: u64,
) -> (result: Result<(), TypedFieldSchemaError>)
    requires
        start <= end <= fields@.len(),
    ensures
        validate_field_owners_tail_spec(
            typed_field_definition_views_spec(fields@),
            start as nat,
            end as nat,
            owner_schema_node_index,
        ) == match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error@),
        },
{
    let ghost views = typed_field_definition_views_spec(fields@);
    let ghost expected = validate_field_owners_tail_spec(
        views,
        start as nat,
        end as nat,
        owner_schema_node_index,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= fields@.len(),
            views == typed_field_definition_views_spec(fields@),
            validate_field_owners_tail_spec(
                typed_field_definition_views_spec(fields@),
                start as nat,
                end as nat,
                owner_schema_node_index,
            ) == expected,
            expected == validate_field_owners_tail_spec(
                views,
                index as nat,
                end as nat,
                owner_schema_node_index,
            ),
        decreases end - index,
    {
        proof {
            assert(0 <= (index as int) < fields@.len());
            reveal(typed_field_definition_views_spec);
            assert(views[index as int] == fields[index as int]@);
            reveal(validate_field_owners_tail_spec);
        }
        if fields[index as usize].owner_schema_node_index() != owner_schema_node_index {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::InvalidFieldOwner,
                Some(owner_schema_node_index),
                Some(index),
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            reveal(validate_field_owners_tail_spec);
        }
        index += 1;
    }
    proof {
        reveal(validate_field_owners_tail_spec);
        assert(expected == Ok(()));
    }
    Ok(())
}

fn validate_schema_nodes(nodes: &[TypedSchemaNode], fields: &[TypedFieldDefinition]) -> (result:
    Result<u64, TypedFieldSchemaError>)
    requires
        nodes@.len() <= MAX_PROFILE1_TYPED_SCHEMA_NODES,
        fields@.len() <= MAX_PROFILE1_TYPED_SCHEMA_FIELDS,
    ensures
        validate_schema_nodes_tail_spec(
            typed_schema_node_views_spec(nodes@),
            typed_field_definition_views_spec(fields@),
            0,
            0,
        ) == match result {
            Ok(cursor) => Ok(cursor),
            Err(error) => Err(error@),
        },
{
    let ghost node_views = typed_schema_node_views_spec(nodes@);
    let ghost field_views = typed_field_definition_views_spec(fields@);
    let ghost expected = validate_schema_nodes_tail_spec(node_views, field_views, 0, 0);
    let mut node_index: u64 = 0;
    let mut field_cursor: u64 = 0;
    while node_index < nodes.len() as u64
        invariant
            node_index <= nodes@.len(),
            field_cursor <= fields@.len(),
            node_views == typed_schema_node_views_spec(nodes@),
            field_views == typed_field_definition_views_spec(fields@),
            validate_schema_nodes_tail_spec(
                typed_schema_node_views_spec(nodes@),
                typed_field_definition_views_spec(fields@),
                0,
                0,
            ) == expected,
            expected == validate_schema_nodes_tail_spec(
                node_views,
                field_views,
                node_index as nat,
                field_cursor as nat,
            ),
        decreases nodes.len() as u64 - node_index,
    {
        let node = &nodes[node_index as usize];
        proof {
            assert(0 <= (node_index as int) < nodes@.len());
            reveal(typed_schema_node_views_spec);
            assert(node_views[node_index as int] == node@);
            reveal(validate_schema_nodes_tail_spec);
        }
        if typed_schema_kind_is_mapping(node.kind()) {
            if node.sequence_item_schema_node_index().is_some() {
                let error = TypedFieldSchemaError::at(
                    TypedFieldSchemaErrorKind::InvalidSchemaNodeShape,
                    Some(node_index),
                    None,
                    None,
                );
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            if node.field_start() != field_cursor || node.field_start() > node.field_end()
                || node.field_end() > fields.len() as u64 {
                let error = TypedFieldSchemaError::at(
                    TypedFieldSchemaErrorKind::InvalidFieldPartition,
                    Some(node_index),
                    Some(field_cursor),
                    None,
                );
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            match validate_field_owners(fields, node.field_start(), node.field_end(), node_index) {
                Ok(()) => {},
                Err(error) => {
                    proof {
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            }
            field_cursor = node.field_end();
        } else if typed_schema_kind_is_sequence(node.kind()) {
            if node.field_start() != 0 || node.field_end() != 0
                || node.sequence_item_schema_node_index().is_none() {
                let error = TypedFieldSchemaError::at(
                    TypedFieldSchemaErrorKind::InvalidSchemaNodeShape,
                    Some(node_index),
                    None,
                    None,
                );
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            let item = match node.sequence_item_schema_node_index() {
                Some(value) => value,
                None => {
                    let error = TypedFieldSchemaError::at(
                        TypedFieldSchemaErrorKind::InternalInvariantViolation,
                        Some(node_index),
                        None,
                        None,
                    );
                    proof {
                        assert(expected == Err(error@));
                    }
                    return Err(error);
                },
            };
            if item >= nodes.len() as u64 {
                let error = TypedFieldSchemaError::at(
                    TypedFieldSchemaErrorKind::InvalidSequenceItemSchemaNode,
                    Some(node_index),
                    None,
                    None,
                );
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
        } else if node.field_start() != 0 || node.field_end() != 0
            || node.sequence_item_schema_node_index().is_some() {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::InvalidSchemaNodeShape,
                Some(node_index),
                None,
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            reveal(validate_schema_nodes_tail_spec);
        }
        node_index += 1;
    }
    proof {
        reveal(validate_schema_nodes_tail_spec);
        assert(expected == Ok(field_cursor));
    }
    Ok(field_cursor)
}

fn validate_field_name(name: &[u32], initial_total: u64, limit: u64, field_index: u64) -> (result:
    Result<u64, TypedFieldSchemaError>)
    requires
        initial_total <= limit <= MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
    ensures
        validate_field_name_tail_spec(name@, 0, initial_total, limit, field_index) == match result {
            Ok(total) => Ok(total),
            Err(error) => Err(error@),
        },
        match result {
            Ok(total) => initial_total <= total <= limit,
            Err(_) => true,
        },
{
    let ghost expected = validate_field_name_tail_spec(name@, 0, initial_total, limit, field_index);
    proof {
        assert(validate_field_name_tail_spec(name@, 0, initial_total, limit, field_index)
            == expected);
    }
    let mut name_index: u64 = 0;
    let mut total = initial_total;
    while name_index < name.len() as u64
        invariant
            name_index <= name@.len(),
            initial_total <= total <= limit,
            validate_field_name_tail_spec(name@, 0, initial_total, limit, field_index) == expected,
            expected == validate_field_name_tail_spec(
                name@,
                name_index as nat,
                total,
                limit,
                field_index,
            ),
        decreases name.len() as u64 - name_index,
    {
        proof {
            reveal(validate_field_name_tail_spec);
        }
        if !unicode_scalar_value(name[name_index as usize]) {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::InvalidFieldNameCodePoint,
                None,
                Some(field_index),
                Some(name_index),
            );
            proof {
                assert(expected == Err(error@));
                assert(validate_field_name_tail_spec(name@, 0, initial_total, limit, field_index)
                    == Err(error@));
            }
            return Err(error);
        }
        if total >= limit {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::FieldNameCodePointLimitExceeded,
                None,
                Some(field_index),
                Some(name_index),
            );
            proof {
                assert(expected == Err(error@));
                assert(validate_field_name_tail_spec(name@, 0, initial_total, limit, field_index)
                    == Err(error@));
            }
            return Err(error);
        }
        total += 1;
        name_index += 1;
    }
    proof {
        reveal(validate_field_name_tail_spec);
        assert(expected == Ok(total));
    }
    Ok(total)
}

fn field_names_equal(left: &[u32], right: &[u32]) -> (equal: bool)
    ensures
        equal == (left@ == right@),
{
    if left.len() != right.len() {
        proof {
            assert(left@.len() != right@.len());
        }
        return false;
    }
    let mut index = 0;
    while index < left.len()
        invariant
            index <= left@.len(),
            left@.len() == right@.len(),
            forall|prior: int| 0 <= prior < index ==> left@[prior] == right@[prior],
        decreases left.len() - index,
    {
        if left[index] != right[index] {
            assert(left@[index as int] != right@[index as int]);
            return false;
        }
        index += 1;
    }
    assert(left@ =~= right@);
    true
}

fn prior_field_id_duplicate(fields: &[TypedFieldDefinition], field_index: u64) -> (duplicate: bool)
    requires
        field_index < fields@.len(),
    ensures
        duplicate == prior_field_id_duplicate_tail_spec(
            typed_field_definition_views_spec(fields@),
            field_index as nat,
            0,
        ),
{
    let ghost views = typed_field_definition_views_spec(fields@);
    let ghost expected = prior_field_id_duplicate_tail_spec(views, field_index as nat, 0);
    let target = fields[field_index as usize].field_id();
    proof {
        assert(0 <= (field_index as int) < fields@.len());
        reveal(typed_field_definition_views_spec);
        assert(views[field_index as int] == fields[field_index as int]@);
        assert(target == views[field_index as int].field_id);
    }
    let mut prior: u64 = 0;
    while prior < field_index
        invariant
            prior <= field_index < fields@.len(),
            views == typed_field_definition_views_spec(fields@),
            target == views[field_index as int].field_id,
            prior_field_id_duplicate_tail_spec(
                typed_field_definition_views_spec(fields@),
                field_index as nat,
                0,
            ) == expected,
            expected == prior_field_id_duplicate_tail_spec(views, field_index as nat, prior as nat),
        decreases field_index - prior,
    {
        proof {
            assert(0 <= (prior as int) < fields@.len());
            assert(0 <= (field_index as int) < fields@.len());
            reveal(typed_field_definition_views_spec);
            assert(views[prior as int] == fields[prior as int]@);
            assert(views[field_index as int] == fields[field_index as int]@);
            reveal(prior_field_id_duplicate_tail_spec);
        }
        if fields[prior as usize].field_id() == target {
            assert(expected);
            return true;
        }
        proof {
            reveal(prior_field_id_duplicate_tail_spec);
        }
        prior += 1;
    }
    reveal(prior_field_id_duplicate_tail_spec);
    assert(!expected);
    false
}

fn prior_field_name_duplicate(fields: &[TypedFieldDefinition], field_index: u64) -> (duplicate:
    bool)
    requires
        field_index < fields@.len(),
    ensures
        duplicate == prior_field_name_duplicate_tail_spec(
            typed_field_definition_views_spec(fields@),
            field_index as nat,
            0,
        ),
{
    let ghost views = typed_field_definition_views_spec(fields@);
    let ghost expected = prior_field_name_duplicate_tail_spec(views, field_index as nat, 0);
    let target_owner = fields[field_index as usize].owner_schema_node_index();
    let target_name = fields[field_index as usize].name();
    proof {
        assert(0 <= (field_index as int) < fields@.len());
        reveal(typed_field_definition_views_spec);
        assert(views[field_index as int] == fields[field_index as int]@);
        assert(target_owner == views[field_index as int].owner_schema_node_index);
        assert(target_name@ == views[field_index as int].name);
    }
    let mut prior: u64 = 0;
    while prior < field_index
        invariant
            prior <= field_index < fields@.len(),
            views == typed_field_definition_views_spec(fields@),
            target_owner == views[field_index as int].owner_schema_node_index,
            target_name@ == views[field_index as int].name,
            prior_field_name_duplicate_tail_spec(
                typed_field_definition_views_spec(fields@),
                field_index as nat,
                0,
            ) == expected,
            expected == prior_field_name_duplicate_tail_spec(
                views,
                field_index as nat,
                prior as nat,
            ),
        decreases field_index - prior,
    {
        proof {
            assert(0 <= (prior as int) < fields@.len());
            assert(0 <= (field_index as int) < fields@.len());
            reveal(typed_field_definition_views_spec);
            assert(views[prior as int] == fields[prior as int]@);
            assert(views[field_index as int] == fields[field_index as int]@);
            reveal(prior_field_name_duplicate_tail_spec);
        }
        if fields[prior as usize].owner_schema_node_index() == target_owner && field_names_equal(
            fields[prior as usize].name(),
            target_name,
        ) {
            assert(expected);
            return true;
        }
        proof {
            reveal(prior_field_name_duplicate_tail_spec);
        }
        prior += 1;
    }
    reveal(prior_field_name_duplicate_tail_spec);
    assert(!expected);
    false
}

fn validate_schema_fields(
    nodes: &[TypedSchemaNode],
    fields: &[TypedFieldDefinition],
    name_limit: u64,
) -> (result: Result<u64, TypedFieldSchemaError>)
    requires
        nodes@.len() <= MAX_PROFILE1_TYPED_SCHEMA_NODES,
        fields@.len() <= MAX_PROFILE1_TYPED_SCHEMA_FIELDS,
        name_limit <= MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
    ensures
        validate_schema_fields_tail_spec(
            typed_schema_node_views_spec(nodes@),
            typed_field_definition_views_spec(fields@),
            0,
            0,
            name_limit,
        ) == match result {
            Ok(total) => Ok(total),
            Err(error) => Err(error@),
        },
{
    let ghost node_views = typed_schema_node_views_spec(nodes@);
    let ghost field_views = typed_field_definition_views_spec(fields@);
    let ghost expected = validate_schema_fields_tail_spec(
        node_views,
        field_views,
        0,
        0,
        name_limit,
    );
    let mut field_index: u64 = 0;
    let mut total: u64 = 0;
    while field_index < fields.len() as u64
        invariant
            field_index <= fields@.len(),
            total <= name_limit,
            name_limit <= MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
            node_views == typed_schema_node_views_spec(nodes@),
            field_views == typed_field_definition_views_spec(fields@),
            validate_schema_fields_tail_spec(
                typed_schema_node_views_spec(nodes@),
                typed_field_definition_views_spec(fields@),
                0,
                0,
                name_limit,
            ) == expected,
            expected == validate_schema_fields_tail_spec(
                node_views,
                field_views,
                field_index as nat,
                total,
                name_limit,
            ),
        decreases fields.len() as u64 - field_index,
    {
        let field = &fields[field_index as usize];
        proof {
            assert(0 <= (field_index as int) < fields@.len());
            reveal(typed_field_definition_views_spec);
            assert(field_views[field_index as int] == field@);
            reveal(validate_schema_fields_tail_spec);
        }
        if field.field_id() == 0 {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::InvalidFieldId,
                Some(field.owner_schema_node_index()),
                Some(field_index),
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if field.value_schema_node_index() >= nodes.len() as u64 {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::InvalidFieldValueSchemaNode,
                Some(field.value_schema_node_index()),
                Some(field_index),
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if field.name().is_empty() {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::EmptyFieldName,
                Some(field.owner_schema_node_index()),
                Some(field_index),
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let prior_total = total;
        proof {
            assert(prior_total <= name_limit <= MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS);
        }
        total =
        match validate_field_name(field.name(), prior_total, name_limit, field_index) {
            Ok(value) => {
                proof {
                    assert(prior_total <= value <= name_limit);
                }
                value
            },
            Err(error) => {
                proof {
                    assert(expected == Err(error@));
                }
                return Err(error);
            },
        };
        if prior_field_id_duplicate(fields, field_index) {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::DuplicateFieldId,
                Some(field.owner_schema_node_index()),
                Some(field_index),
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if prior_field_name_duplicate(fields, field_index) {
            let error = TypedFieldSchemaError::at(
                TypedFieldSchemaErrorKind::DuplicateFieldName,
                Some(field.owner_schema_node_index()),
                Some(field_index),
                None,
            );
            proof {
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            reveal(validate_schema_fields_tail_spec);
        }
        field_index += 1;
    }
    proof {
        reveal(validate_schema_fields_tail_spec);
        assert(expected == Ok(total));
    }
    Ok(total)
}

#[verifier::rlimit(80)]
pub fn compile_typed_field_schema(
    schema: TypedFieldSchema,
    limits: TypedFieldSchemaLimits,
) -> (result: Result<CompiledTypedFieldSchema, TypedFieldSchemaError>)
    ensures
        compile_typed_field_schema_spec(schema@, limits@) == match result {
            Ok(compiled) => Ok(compiled@),
            Err(error) => Err(error@),
        },
{
    let ghost schema_view = schema@;
    let ghost expected = compile_typed_field_schema_spec(schema_view, limits@);
    let node_limit = effective_limit(limits.max_schema_nodes(), MAX_PROFILE1_TYPED_SCHEMA_NODES);
    let field_limit = effective_limit(limits.max_schema_fields(), MAX_PROFILE1_TYPED_SCHEMA_FIELDS);
    let name_limit = effective_limit(
        limits.max_field_name_code_points(),
        MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS,
    );
    proof {
        reveal(compile_typed_field_schema_spec);
        reveal(effective_typed_schema_node_limit_spec);
        reveal(effective_typed_schema_field_limit_spec);
        reveal(effective_typed_schema_name_limit_spec);
    }
    if schema.schema_version() != TYPED_FIELD_SCHEMA_VERSION {
        let error = TypedFieldSchemaError::at(
            TypedFieldSchemaErrorKind::UnsupportedSchemaVersion,
            None,
            None,
            None,
        );
        assert(expected == Err(error@));
        return Err(error);
    }
    if schema.nodes().is_empty() {
        let error = TypedFieldSchemaError::at(
            TypedFieldSchemaErrorKind::EmptySchema,
            Some(0),
            None,
            None,
        );
        assert(expected == Err(error@));
        return Err(error);
    }
    if schema.nodes().len() as u64 > node_limit {
        let error = TypedFieldSchemaError::at(
            TypedFieldSchemaErrorKind::SchemaNodeLimitExceeded,
            Some(node_limit),
            None,
            None,
        );
        assert(expected == Err(error@));
        return Err(error);
    }
    if schema.fields().len() as u64 > field_limit {
        let error = TypedFieldSchemaError::at(
            TypedFieldSchemaErrorKind::SchemaFieldLimitExceeded,
            None,
            Some(field_limit),
            None,
        );
        assert(expected == Err(error@));
        return Err(error);
    }
    if schema.root_schema_node_index() >= schema.nodes().len() as u64 {
        let error = TypedFieldSchemaError::at(
            TypedFieldSchemaErrorKind::InvalidRootSchemaNode,
            Some(schema.root_schema_node_index()),
            None,
            None,
        );
        assert(expected == Err(error@));
        return Err(error);
    }
    let field_cursor = match validate_schema_nodes(schema.nodes(), schema.fields()) {
        Ok(value) => value,
        Err(error) => {
            assert(expected == Err(error@));
            return Err(error);
        },
    };
    if field_cursor != schema.fields().len() as u64 {
        let error = TypedFieldSchemaError::at(
            TypedFieldSchemaErrorKind::InvalidFieldPartition,
            None,
            Some(field_cursor),
            None,
        );
        assert(expected == Err(error@));
        return Err(error);
    }
    let total = match validate_schema_fields(schema.nodes(), schema.fields(), name_limit) {
        Ok(value) => value,
        Err(error) => {
            assert(expected == Err(error@));
            return Err(error);
        },
    };
    let compiled = CompiledTypedFieldSchema::new(schema, total);
    assert(expected == Ok(compiled@));
    Ok(compiled)
}

pub open spec fn compiled_typed_field_schema_well_formed_spec(
    input: TypedFieldSchemaView,
    limits: TypedFieldSchemaLimitsView,
    output: CompiledTypedFieldSchemaView,
) -> bool {
    compile_typed_field_schema_spec(input, limits) == Ok(output)
}

pub open spec fn compiled_typed_field_schema_preserves_input_identity_spec(
    input: TypedFieldSchemaView,
    output: CompiledTypedFieldSchemaView,
) -> bool {
    output.schema == input && output.schema_version == input.schema_version
        && output.root_schema_node_index == input.root_schema_node_index && output.node_count
        == input.nodes.len() && output.field_count == input.fields.len()
}

pub proof fn lemma_typed_field_schema_compilation_success_is_well_formed(
    input: TypedFieldSchemaView,
    limits: TypedFieldSchemaLimitsView,
    output: CompiledTypedFieldSchemaView,
)
    requires
        compile_typed_field_schema_spec(input, limits) == Ok(output),
    ensures
        compiled_typed_field_schema_well_formed_spec(input, limits, output),
{
    reveal(compiled_typed_field_schema_well_formed_spec);
}

pub proof fn lemma_authenticated_typed_field_schema_preserves_input_identity(
    input: TypedFieldSchemaView,
    limits: TypedFieldSchemaLimitsView,
    output: CompiledTypedFieldSchemaView,
)
    requires
        compiled_typed_field_schema_well_formed_spec(input, limits, output),
    ensures
        compiled_typed_field_schema_preserves_input_identity_spec(input, output),
{
    reveal(compiled_typed_field_schema_well_formed_spec);
    reveal(compiled_typed_field_schema_preserves_input_identity_spec);
    reveal(compile_typed_field_schema_spec);
}

pub proof fn lemma_authenticated_typed_field_schema_compilation_is_unique(
    input: TypedFieldSchemaView,
    limits: TypedFieldSchemaLimitsView,
    first: CompiledTypedFieldSchemaView,
    second: CompiledTypedFieldSchemaView,
)
    requires
        compiled_typed_field_schema_well_formed_spec(input, limits, first),
        compiled_typed_field_schema_well_formed_spec(input, limits, second),
    ensures
        first == second,
{
    reveal(compiled_typed_field_schema_well_formed_spec);
}

pub proof fn lemma_empty_mapping_typed_field_schema_compiles_exactly()
    ensures
        compile_typed_field_schema_spec(
            TypedFieldSchemaView {
                schema_version: 1,
                root_schema_node_index: 0,
                nodes: seq![
                    TypedSchemaNodeView {
                        kind: TypedSchemaValueKind::Mapping,
                        field_start: 0,
                        field_end: 0,
                        sequence_item_schema_node_index: None,
                    },
                ],
                fields: Seq::empty(),
            },
            TypedFieldSchemaLimitsView {
                max_schema_nodes: 1,
                max_schema_fields: 0,
                max_field_name_code_points: 0,
            },
        ) == Ok(
            CompiledTypedFieldSchemaView {
                schema_version: 1,
                compilation_version: TYPED_FIELD_SCHEMA_COMPILATION_VERSION,
                root_schema_node_index: 0,
                node_count: 1,
                field_count: 0,
                total_field_name_code_points: 0,
                schema: TypedFieldSchemaView {
                    schema_version: 1,
                    root_schema_node_index: 0,
                    nodes: seq![
                        TypedSchemaNodeView {
                            kind: TypedSchemaValueKind::Mapping,
                            field_start: 0,
                            field_end: 0,
                            sequence_item_schema_node_index: None,
                        },
                    ],
                    fields: Seq::empty(),
                },
            },
        ),
{
    reveal(compile_typed_field_schema_spec);
    reveal(effective_typed_schema_node_limit_spec);
    reveal(effective_typed_schema_field_limit_spec);
    reveal(effective_typed_schema_name_limit_spec);
    reveal(typed_field_schema_effective_limit_spec);
    reveal(typed_schema_kind_is_mapping_spec);
    reveal(typed_schema_kind_is_sequence_spec);
    reveal_with_fuel(validate_schema_nodes_tail_spec, 3);
    reveal_with_fuel(validate_field_owners_tail_spec, 2);
    reveal_with_fuel(validate_schema_fields_tail_spec, 2);
}

} // verus!
