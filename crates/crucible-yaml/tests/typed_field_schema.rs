use crucible_yaml::{
    canonical_typed_field_schema_limits, compile_typed_field_schema, CompiledTypedFieldSchema,
    TypedFieldDefinition, TypedFieldSchema, TypedFieldSchemaErrorKind, TypedFieldSchemaLimits,
    TypedSchemaNode, TypedSchemaValueKind, MAX_PROFILE1_TYPED_SCHEMA_FIELDS,
    MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS, MAX_PROFILE1_TYPED_SCHEMA_NODES,
};

fn name(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

fn node(
    kind: TypedSchemaValueKind,
    field_start: u64,
    field_end: u64,
    sequence_item_schema_node_index: Option<u64>,
) -> TypedSchemaNode {
    TypedSchemaNode::new(
        kind,
        field_start,
        field_end,
        sequence_item_schema_node_index,
    )
}

fn field(
    owner_schema_node_index: u64,
    field_id: u64,
    field_name: &str,
    value_schema_node_index: u64,
    required: bool,
) -> TypedFieldDefinition {
    TypedFieldDefinition::new(
        owner_schema_node_index,
        field_id,
        name(field_name),
        value_schema_node_index,
        required,
    )
}

fn nested_schema() -> TypedFieldSchema {
    TypedFieldSchema::new(
        1,
        0,
        vec![
            node(TypedSchemaValueKind::Mapping, 0, 4, None),
            node(TypedSchemaValueKind::String, 0, 0, None),
            node(TypedSchemaValueKind::Boolean, 0, 0, None),
            node(TypedSchemaValueKind::Mapping, 4, 5, None),
            node(TypedSchemaValueKind::Integer, 0, 0, None),
            node(TypedSchemaValueKind::Sequence, 0, 0, Some(6)),
            node(TypedSchemaValueKind::Mapping, 5, 6, None),
        ],
        vec![
            field(0, 1, "name", 1, true),
            field(0, 2, "enabled", 2, true),
            field(0, 3, "budget", 3, false),
            field(0, 4, "targets", 5, true),
            field(3, 5, "cpu", 4, true),
            field(6, 6, "id", 1, true),
        ],
    )
}

fn compile(schema: TypedFieldSchema) -> CompiledTypedFieldSchema {
    compile_typed_field_schema(schema, canonical_typed_field_schema_limits()).unwrap()
}

#[test]
fn compiles_nested_typed_schema_without_erasing_field_identity_or_requiredness() {
    let compiled = compile(nested_schema());

    assert_eq!(compiled.schema_version(), 1);
    assert_eq!(compiled.root_schema_node_index(), 0);
    assert_eq!(compiled.node_count(), 7);
    assert_eq!(compiled.field_count(), 6);
    assert_eq!(compiled.total_field_name_code_points(), 29);
    assert_eq!(compiled.schema().nodes().len(), 7);
    assert_eq!(compiled.schema().fields().len(), 6);

    let root = &compiled.schema().nodes()[0];
    assert_eq!(root.kind(), TypedSchemaValueKind::Mapping);
    assert_eq!((root.field_start(), root.field_end()), (0, 4));
    assert_eq!(root.sequence_item_schema_node_index(), None);

    let targets = &compiled.schema().fields()[3];
    assert_eq!(targets.owner_schema_node_index(), 0);
    assert_eq!(targets.field_id(), 4);
    assert_eq!(targets.name(), name("targets"));
    assert_eq!(targets.value_schema_node_index(), 5);
    assert!(targets.required());

    let sequence = &compiled.schema().nodes()[5];
    assert_eq!(sequence.kind(), TypedSchemaValueKind::Sequence);
    assert_eq!(sequence.sequence_item_schema_node_index(), Some(6));
}

#[test]
fn rejects_schema_shape_owner_and_reference_laundering_with_exact_indices() {
    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Sequence, 0, 0, None)],
            vec![],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::InvalidSchemaNodeShape
    );
    assert_eq!(error.schema_node_index(), Some(0));

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![
                node(TypedSchemaValueKind::Mapping, 0, 1, None),
                node(TypedSchemaValueKind::String, 0, 0, None),
            ],
            vec![field(1, 1, "wrong-owner", 1, true)],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedFieldSchemaErrorKind::InvalidFieldOwner);
    assert_eq!(error.schema_node_index(), Some(0));
    assert_eq!(error.schema_field_index(), Some(0));

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Mapping, 0, 1, None)],
            vec![field(0, 1, "bad-value", 9, true)],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::InvalidFieldValueSchemaNode
    );
    assert_eq!(error.schema_field_index(), Some(0));
}

#[test]
fn rejects_version_root_unicode_and_partition_errors_deterministically() {
    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            2,
            0,
            vec![node(TypedSchemaValueKind::Mapping, 0, 0, None)],
            vec![],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::UnsupportedSchemaVersion
    );

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            3,
            vec![node(TypedSchemaValueKind::Mapping, 0, 0, None)],
            vec![],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::InvalidRootSchemaNode
    );
    assert_eq!(error.schema_node_index(), Some(3));

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Mapping, 0, 0, None)],
            vec![field(0, 1, "unowned", 0, true)],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::InvalidFieldPartition
    );
    assert_eq!(error.schema_field_index(), Some(0));

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Mapping, 0, 1, None)],
            vec![TypedFieldDefinition::new(0, 1, vec![0xd800], 0, true)],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::InvalidFieldNameCodePoint
    );
    assert_eq!(error.schema_field_index(), Some(0));
    assert_eq!(error.name_code_point_index(), Some(0));
}

#[test]
fn accepts_the_complete_exact_value_kind_vocabulary() {
    let kinds = [
        TypedSchemaValueKind::Null,
        TypedSchemaValueKind::Boolean,
        TypedSchemaValueKind::Integer,
        TypedSchemaValueKind::FiniteFloat,
        TypedSchemaValueKind::PositiveInfinity,
        TypedSchemaValueKind::NegativeInfinity,
        TypedSchemaValueKind::NotANumber,
        TypedSchemaValueKind::String,
        TypedSchemaValueKind::CustomScalar,
        TypedSchemaValueKind::Sequence,
        TypedSchemaValueKind::CustomSequence,
        TypedSchemaValueKind::Mapping,
        TypedSchemaValueKind::CustomMapping,
    ];
    let mut nodes = vec![node(TypedSchemaValueKind::Mapping, 0, 13, None)];
    let mut fields = Vec::new();
    for (index, kind) in kinds.into_iter().enumerate() {
        let schema_node_index = index as u64 + 1;
        let schema_node = match kind {
            TypedSchemaValueKind::Sequence | TypedSchemaValueKind::CustomSequence => {
                node(kind, 0, 0, Some(1))
            }
            TypedSchemaValueKind::Mapping | TypedSchemaValueKind::CustomMapping => {
                node(kind, 13, 13, None)
            }
            _ => node(kind, 0, 0, None),
        };
        nodes.push(schema_node);
        fields.push(TypedFieldDefinition::new(
            0,
            schema_node_index,
            name(&format!("field-{schema_node_index}")),
            schema_node_index,
            false,
        ));
    }

    let compiled = compile(TypedFieldSchema::new(1, 0, nodes, fields));
    for (index, kind) in kinds.into_iter().enumerate() {
        assert_eq!(compiled.schema().nodes()[index + 1].kind(), kind);
    }
}

#[test]
fn rejects_empty_schemas_and_invalid_sequence_field_identity_and_name_records() {
    let error = compile_typed_field_schema(
        TypedFieldSchema::new(1, 0, vec![], vec![]),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedFieldSchemaErrorKind::EmptySchema);

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Sequence, 0, 0, Some(4))],
            vec![],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::InvalidSequenceItemSchemaNode
    );
    assert_eq!(error.schema_node_index(), Some(0));

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Mapping, 0, 1, None)],
            vec![TypedFieldDefinition::new(0, 0, name("zero"), 0, true)],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedFieldSchemaErrorKind::InvalidFieldId);
    assert_eq!(error.schema_field_index(), Some(0));

    let error = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![node(TypedSchemaValueKind::Mapping, 0, 1, None)],
            vec![TypedFieldDefinition::new(0, 1, vec![], 0, true)],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedFieldSchemaErrorKind::EmptyFieldName);
    assert_eq!(error.schema_field_index(), Some(0));
}

#[test]
fn rejects_duplicate_stable_ids_globally_and_duplicate_names_within_one_mapping() {
    let duplicate_name = TypedFieldSchema::new(
        1,
        0,
        vec![
            node(TypedSchemaValueKind::Mapping, 0, 2, None),
            node(TypedSchemaValueKind::String, 0, 0, None),
        ],
        vec![field(0, 1, "same", 1, true), field(0, 2, "same", 1, false)],
    );
    let error = compile_typed_field_schema(duplicate_name, canonical_typed_field_schema_limits())
        .unwrap_err();
    assert_eq!(error.kind(), TypedFieldSchemaErrorKind::DuplicateFieldName);
    assert_eq!(error.schema_field_index(), Some(1));

    let duplicate_id = TypedFieldSchema::new(
        1,
        0,
        vec![
            node(TypedSchemaValueKind::Mapping, 0, 1, None),
            node(TypedSchemaValueKind::Mapping, 1, 2, None),
            node(TypedSchemaValueKind::String, 0, 0, None),
        ],
        vec![field(0, 9, "left", 2, true), field(1, 9, "right", 2, true)],
    );
    let error = compile_typed_field_schema(duplicate_id, canonical_typed_field_schema_limits())
        .unwrap_err();
    assert_eq!(error.kind(), TypedFieldSchemaErrorKind::DuplicateFieldId);
    assert_eq!(error.schema_field_index(), Some(1));
}

#[test]
fn caller_limits_accept_inclusive_boundaries_and_reject_the_first_excluded_item() {
    let exact = TypedFieldSchemaLimits::new(7, 6, 29);
    let compiled = compile_typed_field_schema(nested_schema(), exact).unwrap();
    assert_eq!(compiled.node_count(), 7);
    assert_eq!(compiled.field_count(), 6);
    assert_eq!(compiled.total_field_name_code_points(), 29);

    let error = compile_typed_field_schema(nested_schema(), TypedFieldSchemaLimits::new(6, 6, 29))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::SchemaNodeLimitExceeded
    );
    assert_eq!(error.schema_node_index(), Some(6));

    let error = compile_typed_field_schema(nested_schema(), TypedFieldSchemaLimits::new(7, 5, 29))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::SchemaFieldLimitExceeded
    );
    assert_eq!(error.schema_field_index(), Some(5));

    let error = compile_typed_field_schema(nested_schema(), TypedFieldSchemaLimits::new(7, 6, 28))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedFieldSchemaErrorKind::FieldNameCodePointLimitExceeded
    );
    assert_eq!(error.schema_field_index(), Some(5));
    assert_eq!(error.name_code_point_index(), Some(1));

    let clamped = compile_typed_field_schema(
        nested_schema(),
        TypedFieldSchemaLimits::new(u64::MAX, u64::MAX, u64::MAX),
    )
    .unwrap();
    assert_eq!(clamped.node_count(), 7);
    assert_eq!(clamped.field_count(), 6);

    let canonical = canonical_typed_field_schema_limits();
    assert_eq!(
        canonical.max_schema_nodes(),
        MAX_PROFILE1_TYPED_SCHEMA_NODES
    );
    assert_eq!(
        canonical.max_schema_fields(),
        MAX_PROFILE1_TYPED_SCHEMA_FIELDS
    );
    assert_eq!(
        canonical.max_field_name_code_points(),
        MAX_PROFILE1_TYPED_SCHEMA_NAME_CODE_POINTS
    );
}
