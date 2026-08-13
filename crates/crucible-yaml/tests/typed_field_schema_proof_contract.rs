#![allow(unused_imports)]

use vstd::prelude::*;

verus! {

proof fn successful_schema_compilation_has_exact_public_semantics(
    input: crucible_yaml::schema::TypedFieldSchemaView,
    limits: crucible_yaml::schema::TypedFieldSchemaLimitsView,
    output: crucible_yaml::schema::CompiledTypedFieldSchemaView,
)
    requires
        crucible_yaml::schema::compile_typed_field_schema_spec(input, limits) == Ok(output),
    ensures
        crucible_yaml::schema::compiled_typed_field_schema_well_formed_spec(input, limits, output),
{
    crucible_yaml::schema::lemma_typed_field_schema_compilation_success_is_well_formed(
        input,
        limits,
        output,
    );
}

proof fn authenticated_compilation_owns_the_exact_schema(
    input: crucible_yaml::schema::TypedFieldSchemaView,
    limits: crucible_yaml::schema::TypedFieldSchemaLimitsView,
    output: crucible_yaml::schema::CompiledTypedFieldSchemaView,
)
    requires
        crucible_yaml::schema::compiled_typed_field_schema_well_formed_spec(input, limits, output),
    ensures
        crucible_yaml::schema::compiled_typed_field_schema_preserves_input_identity_spec(
            input,
            output,
        ),
{
    crucible_yaml::schema::lemma_authenticated_typed_field_schema_preserves_input_identity(
        input,
        limits,
        output,
    );
}

proof fn schema_compilation_rejects_output_substitution(
    input: crucible_yaml::schema::TypedFieldSchemaView,
    limits: crucible_yaml::schema::TypedFieldSchemaLimitsView,
    first: crucible_yaml::schema::CompiledTypedFieldSchemaView,
    second: crucible_yaml::schema::CompiledTypedFieldSchemaView,
)
    requires
        crucible_yaml::schema::compiled_typed_field_schema_well_formed_spec(input, limits, first),
        crucible_yaml::schema::compiled_typed_field_schema_well_formed_spec(input, limits, second),
    ensures
        first == second,
{
    crucible_yaml::schema::lemma_authenticated_typed_field_schema_compilation_is_unique(
        input,
        limits,
        first,
        second,
    );
}

#[test]
fn pure_empty_mapping_schema_has_one_exact_compilation_result() {
    proof {
        let input = crucible_yaml::schema::TypedFieldSchemaView {
            schema_version: 1,
            root_schema_node_index: 0,
            nodes: seq![
                crucible_yaml::schema::TypedSchemaNodeView {
                    kind: crucible_yaml::TypedSchemaValueKind::Mapping,
                    field_start: 0,
                    field_end: 0,
                    sequence_item_schema_node_index: None,
                },
            ],
            fields: Seq::empty(),
        };
        let limits = crucible_yaml::schema::TypedFieldSchemaLimitsView {
            max_schema_nodes: 1,
            max_schema_fields: 0,
            max_field_name_code_points: 0,
        };
        let expected = crucible_yaml::schema::CompiledTypedFieldSchemaView {
            schema_version: 1,
            compilation_version: crucible_yaml::schema::TYPED_FIELD_SCHEMA_COMPILATION_VERSION,
            root_schema_node_index: 0,
            node_count: 1,
            field_count: 0,
            total_field_name_code_points: 0,
            schema: input,
        };
        crucible_yaml::schema::lemma_empty_mapping_typed_field_schema_compiles_exactly();
        assert(crucible_yaml::schema::compile_typed_field_schema_spec(input, limits) == Ok(
            expected,
        ));
    }
}

} // verus!
