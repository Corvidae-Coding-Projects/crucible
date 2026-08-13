use crucible_yaml::{
    partition_profile1_typed_mapping_fields, CanonicalYamlGraphSource, CompiledTypedFieldSchema,
    TypedMappingFieldError, TypedMappingFieldLimits, TypedMappingFieldPartition,
};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "used by Verus proof contracts after ordinary Rust erasure")]
fn executable_partition_has_the_exact_total_pure_result(
    graph: &CanonicalYamlGraphSource,
    schema: &CompiledTypedFieldSchema,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    limits: TypedMappingFieldLimits,
) -> (result: Result<TypedMappingFieldPartition, TypedMappingFieldError>)
    ensures
        crucible_yaml::lower_typed_fields::partition_profile1_typed_mapping_fields_spec(
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
    partition_profile1_typed_mapping_fields(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        limits,
    )
}

proof fn successful_partition_exposes_structural_semantics(
    graph: crucible_yaml::lower::CanonicalYamlGraphSourceView,
    schema: crucible_yaml::schema::CompiledTypedFieldSchemaView,
    yaml_mapping_node_index: u64,
    schema_mapping_node_index: u64,
    limits: crucible_yaml::TypedMappingFieldLimitsView,
    partition: crucible_yaml::TypedMappingFieldPartitionView,
)
    requires
        crucible_yaml::lower_typed_fields::partition_profile1_typed_mapping_fields_spec(
            graph,
            schema,
            yaml_mapping_node_index,
            schema_mapping_node_index,
            limits,
        ) == Ok(partition),
    ensures
        crucible_yaml::lower_typed_fields::typed_mapping_field_partition_semantics_spec(
            graph,
            schema,
            partition,
        ),
{
    crucible_yaml::lower_typed_fields::lemma_successful_typed_mapping_field_partition_has_semantics(
        graph,
        schema,
        yaml_mapping_node_index,
        schema_mapping_node_index,
        limits,
        partition,
    );
}

proof fn forged_out_of_order_partition_is_rejected(
    graph: crucible_yaml::lower::CanonicalYamlGraphSourceView,
    schema: crucible_yaml::schema::CompiledTypedFieldSchemaView,
    partition: crucible_yaml::TypedMappingFieldPartitionView,
)
    requires
        partition.fields.len() >= 2,
        partition.fields[0].schema_field_index >= partition.fields[1].schema_field_index,
    ensures
        !crucible_yaml::lower_typed_fields::typed_mapping_field_partition_semantics_spec(
            graph,
            schema,
            partition,
        ),
{
    reveal(crucible_yaml::lower_typed_fields::typed_mapping_field_partition_semantics_spec);
}

proof fn forged_required_field_omission_is_rejected(
    graph: crucible_yaml::lower::CanonicalYamlGraphSourceView,
    schema: crucible_yaml::schema::CompiledTypedFieldSchemaView,
    partition: crucible_yaml::TypedMappingFieldPartitionView,
    required_index: int,
)
    requires
        0 <= required_index < schema.schema.fields.len(),
        schema.schema.fields[required_index].required,
        partition.schema_mapping_node_index < schema.schema.nodes.len(),
        schema.schema.nodes[partition.schema_mapping_node_index as int].field_start
            <= required_index
            < schema.schema.nodes[partition.schema_mapping_node_index as int].field_end,
        forall|field_index: int|
            0 <= field_index < partition.fields.len()
                ==> partition.fields[field_index].schema_field_index as int != required_index,
    ensures
        !crucible_yaml::lower_typed_fields::typed_mapping_field_partition_semantics_spec(
            graph,
            schema,
            partition,
        ),
{
    reveal(crucible_yaml::lower_typed_fields::typed_mapping_field_partition_semantics_spec);
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(crucible_yaml::TYPED_MAPPING_FIELD_PARTITION_VERSION, 1);
}
