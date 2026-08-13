use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_alias_cycle_limits,
    canonical_block_scalar_limits, canonical_completed_token_limits, canonical_cst_limits,
    canonical_duplicate_key_limits, canonical_lowering_limits, canonical_merge_expansion_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_key_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    canonical_typed_field_schema_limits, canonical_typed_mapping_field_limits,
    compile_typed_field_schema, compose_profile1_canonical_structural_keys, decode_profile1,
    expand_profile1_merge_keys, lower_profile1_canonical_graph, parse_profile1_cst,
    partition_profile1_typed_mapping_fields, partition_profile1_typed_mapping_fields_with_policy,
    reject_profile1_duplicate_keys, scan_profile1_block_scalars, scan_profile1_completed_tokens,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AnchorAliasLimits, AtomizeLimits, BomPolicy, CanonicalYamlGraphSource, DecodeLimits,
    TypedFieldDefinition, TypedFieldSchema, TypedMappingFieldErrorKind, TypedMappingFieldLimits,
    TypedMappingUnknownFieldPolicy, TypedSchemaNode, TypedSchemaValueKind,
    MAX_PROFILE1_ALIAS_BINDINGS, MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_TYPED_MAPPING_FIELDS,
    MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS,
};

fn canonical_graph(input: &[u8]) -> CanonicalYamlGraphSource {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .unwrap();
    let atoms = atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS)).unwrap();
    let layout = analyze_profile1_layout(&atoms, canonical_structural_layout_limits()).unwrap();
    let structural =
        scan_profile1_structural_lexemes(&atoms, &layout, canonical_structural_scan_limits())
            .unwrap();
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &layout,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .unwrap();
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .unwrap();
    let block = scan_profile1_block_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    )
    .unwrap();
    let tokens = scan_profile1_completed_tokens(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    )
    .unwrap();
    let cst = parse_profile1_cst(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        &tokens,
        canonical_cst_limits(),
    )
    .unwrap();
    let structural_keys = compose_profile1_canonical_structural_keys(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        canonical_semantic_topology_limits(),
        canonical_semantic_scalar_table_limits(),
        AnchorAliasLimits::new(
            MAX_PROFILE1_ANCHOR_DECLARATIONS,
            MAX_PROFILE1_ALIAS_BINDINGS,
        ),
        canonical_semantic_node_table_limits(),
        canonical_alias_cycle_limits(),
        canonical_scalar_key_limits(),
        canonical_structural_key_limits(),
    )
    .unwrap();
    let duplicate_free =
        reject_profile1_duplicate_keys(structural_keys, canonical_duplicate_key_limits()).unwrap();
    let expanded =
        expand_profile1_merge_keys(duplicate_free, canonical_merge_expansion_limits()).unwrap();
    lower_profile1_canonical_graph(expanded, canonical_lowering_limits()).unwrap()
}

fn code_points(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn compiled_schema() -> crucible_yaml::CompiledTypedFieldSchema {
    let nodes = vec![
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 0, 5, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Integer, 0, 0, None),
        TypedSchemaNode::new(TypedSchemaValueKind::String, 0, 0, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 5, 7, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Boolean, 0, 0, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Sequence, 0, 0, Some(2)),
    ];
    let fields = vec![
        TypedFieldDefinition::new(0, 1, code_points("version"), 1, true),
        TypedFieldDefinition::new(0, 2, code_points("name"), 2, true),
        TypedFieldDefinition::new(0, 3, code_points("execution"), 3, true),
        TypedFieldDefinition::new(0, 4, code_points("modes"), 5, false),
        TypedFieldDefinition::new(0, 7, code_points("defaults"), 3, false),
        TypedFieldDefinition::new(3, 5, code_points("enabled"), 4, true),
        TypedFieldDefinition::new(3, 6, code_points("label"), 2, false),
    ];
    compile_typed_field_schema(
        TypedFieldSchema::new(1, 0, nodes, fields),
        canonical_typed_field_schema_limits(),
    )
    .unwrap()
}

#[test]
fn recognized_fields_are_partitioned_in_schema_order_with_exact_value_bindings() {
    let graph = canonical_graph(
        b"execution: {label: demo, enabled: true}\nname: example\nversion: 1\nmodes: [managed, native]\n",
    );
    let schema = compiled_schema();
    let root = graph.document_roots()[0].value_node_index();
    let partition = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap();

    assert_eq!(partition.yaml_mapping_node_index(), root);
    assert_eq!(partition.schema_mapping_node_index(), 0);
    assert_eq!(
        partition
            .fields()
            .iter()
            .map(|field| field.field_id())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        partition
            .fields()
            .iter()
            .map(|field| field.binding().kind())
            .collect::<Vec<_>>(),
        vec![
            TypedSchemaValueKind::Integer,
            TypedSchemaValueKind::String,
            TypedSchemaValueKind::Mapping,
            TypedSchemaValueKind::Sequence,
        ]
    );

    let execution = partition
        .fields()
        .iter()
        .find(|field| field.field_id() == 3)
        .unwrap();
    let nested = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        execution.value_yaml_node_index(),
        3,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap();
    assert_eq!(
        nested
            .fields()
            .iter()
            .map(|field| field.field_id())
            .collect::<Vec<_>>(),
        vec![5, 6]
    );
}

#[test]
fn unknown_and_missing_required_fields_have_typed_exact_diagnostics() {
    let schema = compiled_schema();
    let unknown_input =
        b"version: 1\nname: example\nexecution: {enabled: true}\nunexpected: dangerous\n";
    let graph = canonical_graph(unknown_input);
    let root = graph.document_roots()[0].value_node_index();
    let error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedMappingFieldErrorKind::UnknownField);
    assert_eq!(
        error.byte_offset(),
        unknown_input
            .windows(b"unexpected".len())
            .position(|window| window == b"unexpected")
            .unwrap() as u64
    );
    assert_eq!(
        error.mapping_entry_index(),
        Some(graph.nodes()[root as usize].edge_start() + 3)
    );
    assert_eq!(error.schema_field_index(), None);

    let missing = canonical_graph(b"version: 1\nexecution: {enabled: true}\n");
    let root = missing.document_roots()[0].value_node_index();
    let error = partition_profile1_typed_mapping_fields(
        &missing,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedMappingFieldErrorKind::MissingRequiredField
    );
    assert_eq!(
        error.byte_offset(),
        missing.nodes()[root as usize].byte_start()
    );
    assert_eq!(error.mapping_entry_index(), None);
    assert_eq!(error.schema_field_index(), Some(1));
}

#[test]
fn mapping_keys_and_values_are_checked_without_coercion() {
    let schema = compiled_schema();
    let key_input = b"? [version]\n: 1\nname: example\nexecution: {enabled: true}\n";
    let graph = canonical_graph(key_input);
    let root = graph.document_roots()[0].value_node_index();
    let error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedMappingFieldErrorKind::MappingKeyNotString
    );

    let value_input = b"version: wrong\nname: example\nexecution: {enabled: true}\n";
    let graph = canonical_graph(value_input);
    let root = graph.document_roots()[0].value_node_index();
    let error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedMappingFieldErrorKind::ValueKindMismatch);
    assert_eq!(error.schema_field_index(), Some(0));
    assert_eq!(error.byte_offset(), 9);
}

#[test]
fn merge_expansion_and_source_reordering_preserve_schema_identity_and_provenance() {
    let schema = compiled_schema();
    let direct = canonical_graph(
        b"version: 1\nname: example\nexecution: {enabled: true}\nmodes: [managed]\n",
    );
    let merged = canonical_graph(
        b"defaults: &defaults {label: run, enabled: true}\nmodes: [managed]\nexecution: {<<: *defaults}\nname: example\nversion: 1\n",
    );

    let direct_root = direct.document_roots()[0].value_node_index();
    let merged_root = merged.document_roots()[0].value_node_index();
    let direct_partition = partition_profile1_typed_mapping_fields(
        &direct,
        &schema,
        direct_root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap();
    let merged_partition = partition_profile1_typed_mapping_fields(
        &merged,
        &schema,
        merged_root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap();
    assert_eq!(
        direct_partition
            .fields()
            .iter()
            .map(|field| field.field_id())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        merged_partition
            .fields()
            .iter()
            .map(|field| field.field_id())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 7]
    );
    let execution = merged_partition
        .fields()
        .iter()
        .find(|field| field.field_id() == 3)
        .unwrap();
    let nested = partition_profile1_typed_mapping_fields(
        &merged,
        &schema,
        execution.value_yaml_node_index(),
        3,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap();
    assert_eq!(
        nested
            .fields()
            .iter()
            .map(|field| field.field_id())
            .collect::<Vec<_>>(),
        vec![5, 6]
    );
    assert!(nested.fields().iter().all(|field| field.inherited()));
}

#[test]
fn field_and_key_code_point_caps_are_independent_exact_and_cannot_be_raised() {
    assert_eq!(MAX_PROFILE1_TYPED_MAPPING_FIELDS, 1_048_576);
    assert_eq!(MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS, 1_048_576);
    let input = b"version: 1\nname: example\nexecution: {enabled: true}\nmodes: [managed]\n";
    let graph = canonical_graph(input);
    let schema = compiled_schema();
    let root = graph.document_roots()[0].value_node_index();

    let error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        TypedMappingFieldLimits::new(3, MAX_PROFILE1_TYPED_MAPPING_KEY_CODE_POINTS),
    )
    .unwrap_err();
    assert_eq!(error.kind(), TypedMappingFieldErrorKind::FieldLimitExceeded);
    assert_eq!(error.byte_offset(), 52);
    assert_eq!(error.schema_field_index(), Some(3));

    let error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        TypedMappingFieldLimits::new(MAX_PROFILE1_TYPED_MAPPING_FIELDS, 7),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedMappingFieldErrorKind::KeyCodePointLimitExceeded
    );
    assert_eq!(error.byte_offset(), 11);

    let canonical = canonical_typed_mapping_field_limits();
    let raised = TypedMappingFieldLimits::new(u64::MAX, u64::MAX);
    let canonical_partition =
        partition_profile1_typed_mapping_fields(&graph, &schema, root, 0, canonical).unwrap();
    let raised_partition =
        partition_profile1_typed_mapping_fields(&graph, &schema, root, 0, raised).unwrap();
    assert_eq!(canonical_partition, raised_partition);
}

#[test]
fn explicit_compatibility_policy_preserves_unknown_fields_losslessly() {
    let input =
        b"future: {nested: [one, two]}\nversion: 1\nname: example\nexecution: {enabled: true}\n";
    let graph = canonical_graph(input);
    let schema = compiled_schema();
    let root = graph.document_roots()[0].value_node_index();

    let partition = partition_profile1_typed_mapping_fields_with_policy(
        &graph,
        &schema,
        root,
        0,
        TypedMappingUnknownFieldPolicy::Preserve,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap();

    assert_eq!(partition.fields().len(), 3);
    assert_eq!(partition.unknown_fields().len(), 1);
    let unknown = &partition.unknown_fields()[0];
    let entry = &graph.mapping_entries()[unknown.mapping_entry_index() as usize];
    assert_eq!(unknown.key_yaml_node_index(), entry.key_node_index());
    assert_eq!(unknown.value_yaml_node_index(), entry.value_node_index());
    assert_eq!(unknown.inherited(), entry.inherited());
    assert_eq!(unknown.key_code_points(), code_points("future"));
    assert_eq!(unknown.byte_start(), 0);
    assert_eq!(unknown.byte_end(), 6);

    let default_error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap_err();
    assert_eq!(
        default_error.kind(),
        TypedMappingFieldErrorKind::UnknownField
    );
}

#[test]
fn mapping_partition_rejects_non_mapping_inputs_even_when_generic_binding_matches() {
    let graph = canonical_graph(b"plain\n");
    let schema = compile_typed_field_schema(
        TypedFieldSchema::new(
            1,
            0,
            vec![TypedSchemaNode::new(
                TypedSchemaValueKind::String,
                0,
                0,
                None,
            )],
            vec![],
        ),
        canonical_typed_field_schema_limits(),
    )
    .unwrap();
    let root = graph.document_roots()[0].value_node_index();
    let error = partition_profile1_typed_mapping_fields(
        &graph,
        &schema,
        root,
        0,
        canonical_typed_mapping_field_limits(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedMappingFieldErrorKind::MappingKindMismatch
    );
}
