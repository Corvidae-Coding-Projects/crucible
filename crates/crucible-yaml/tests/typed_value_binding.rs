use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, bind_profile1_typed_yaml_value,
    canonical_alias_cycle_limits, canonical_block_scalar_limits, canonical_completed_token_limits,
    canonical_cst_limits, canonical_duplicate_key_limits, canonical_lowering_limits,
    canonical_merge_expansion_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_key_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    canonical_typed_field_schema_limits, compile_typed_field_schema,
    compose_profile1_canonical_structural_keys, decode_profile1, expand_profile1_merge_keys,
    lower_profile1_canonical_graph, parse_profile1_cst, reject_profile1_duplicate_keys,
    scan_profile1_block_scalars, scan_profile1_completed_tokens, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AnchorAliasLimits,
    AtomizeLimits, BomPolicy, CanonicalYamlGraphSource, DecodeLimits, TypedFieldSchema,
    TypedSchemaNode, TypedSchemaValueKind, TypedValueBindingErrorKind, MAX_PROFILE1_ALIAS_BINDINGS,
    MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES,
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

fn schema_for_all_kinds() -> crucible_yaml::CompiledTypedFieldSchema {
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
    let nodes = kinds
        .into_iter()
        .map(|kind| match kind {
            TypedSchemaValueKind::Sequence | TypedSchemaValueKind::CustomSequence => {
                TypedSchemaNode::new(kind, 0, 0, Some(7))
            }
            _ => TypedSchemaNode::new(kind, 0, 0, None),
        })
        .collect();
    compile_typed_field_schema(
        TypedFieldSchema::new(1, 0, nodes, vec![]),
        canonical_typed_field_schema_limits(),
    )
    .unwrap()
}

#[test]
fn binds_every_exact_core_and_custom_scalar_and_collection_kind() {
    let graph = canonical_graph(
        b"--- null\n--- true\n--- 42\n--- 1.25\n--- .inf\n--- -.inf\n--- .nan\n--- text\n--- !local tagged\n--- [item]\n--- !local [item]\n--- {key: value}\n--- !local {key: value}\n",
    );
    let schema = schema_for_all_kinds();
    let expected = [
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

    assert_eq!(graph.document_roots().len(), expected.len());
    for (schema_node_index, (root, kind)) in graph.document_roots().iter().zip(expected).enumerate()
    {
        let binding = bind_profile1_typed_yaml_value(
            &graph,
            &schema,
            root.value_node_index(),
            schema_node_index as u64,
        )
        .unwrap();
        assert_eq!(binding.yaml_node_index(), root.value_node_index());
        assert_eq!(binding.schema_node_index(), schema_node_index as u64);
        assert_eq!(binding.kind(), kind);
        assert_eq!(
            binding.byte_start(),
            graph.nodes()[root.value_node_index() as usize].byte_start()
        );
        if schema_node_index <= 8 {
            assert!(binding.scalar_index().is_some());
            assert_eq!(binding.collection_index(), None);
        } else {
            assert_eq!(binding.scalar_index(), None);
            assert!(binding.collection_index().is_some());
        }
    }
}

#[test]
fn exact_tags_and_nonfinite_signs_cannot_be_laundered_between_schema_kinds() {
    let graph = canonical_graph(
        b"--- text\n--- !local tagged\n--- .inf\n--- -.inf\n--- [x]\n--- !local [x]\n",
    );
    let schema = schema_for_all_kinds();
    let mismatches = [(0, 8), (1, 7), (2, 5), (3, 4), (4, 10), (5, 9)];
    for (document_index, schema_node_index) in mismatches {
        let root = &graph.document_roots()[document_index];
        let error = bind_profile1_typed_yaml_value(
            &graph,
            &schema,
            root.value_node_index(),
            schema_node_index,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            TypedValueBindingErrorKind::YamlValueKindMismatch
        );
        assert_eq!(
            error.byte_offset(),
            graph.nodes()[root.value_node_index() as usize].byte_start()
        );
        assert_eq!(error.yaml_node_index(), root.value_node_index());
        assert_eq!(error.schema_node_index(), schema_node_index);
    }
}

#[test]
fn index_precedence_and_exact_offsets_are_typed_and_deterministic() {
    let graph = canonical_graph(b"value\n");
    let schema = schema_for_all_kinds();
    let root = graph.document_roots()[0];

    let error = bind_profile1_typed_yaml_value(&graph, &schema, u64::MAX, u64::MAX).unwrap_err();
    assert_eq!(
        error.kind(),
        TypedValueBindingErrorKind::YamlNodeIndexOutOfRange
    );
    assert_eq!(error.byte_offset(), graph.source_len_bytes());
    assert_eq!(error.yaml_node_index(), u64::MAX);
    assert_eq!(error.schema_node_index(), u64::MAX);

    let error = bind_profile1_typed_yaml_value(&graph, &schema, root.value_node_index(), u64::MAX)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TypedValueBindingErrorKind::SchemaNodeIndexOutOfRange
    );
    assert_eq!(
        error.byte_offset(),
        graph.nodes()[root.value_node_index() as usize].byte_start()
    );
}
