#[expect(
    unused_imports,
    reason = "these variants are referenced only inside Verus proof code"
)]
use crucible_yaml::{ResolvedCollectionTag, ResolvedScalarTag, TypedSchemaValueKind};
use vstd::prelude::*;

verus! {

proof fn successful_value_binding_has_exact_public_semantics(
    graph: crucible_yaml::lower::CanonicalYamlGraphSourceView,
    schema: crucible_yaml::schema::CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
    output: crucible_yaml::lower_typed::TypedYamlValueBindingView,
)
    requires
        crucible_yaml::lower_typed::bind_profile1_typed_yaml_value_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
        ) == Ok(output),
    ensures
        crucible_yaml::lower_typed::typed_yaml_value_binding_well_formed_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
            output,
        ),
{
    crucible_yaml::lower_typed::lemma_typed_yaml_value_binding_success_is_well_formed(
        graph,
        schema,
        yaml_node_index,
        schema_node_index,
        output,
    );
}

proof fn value_binding_rejects_output_substitution(
    graph: crucible_yaml::lower::CanonicalYamlGraphSourceView,
    schema: crucible_yaml::schema::CompiledTypedFieldSchemaView,
    yaml_node_index: u64,
    schema_node_index: u64,
    first: crucible_yaml::lower_typed::TypedYamlValueBindingView,
    second: crucible_yaml::lower_typed::TypedYamlValueBindingView,
)
    requires
        crucible_yaml::lower_typed::typed_yaml_value_binding_well_formed_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
            first,
        ),
        crucible_yaml::lower_typed::typed_yaml_value_binding_well_formed_spec(
            graph,
            schema,
            yaml_node_index,
            schema_node_index,
            second,
        ),
    ensures
        first == second,
{
    crucible_yaml::lower_typed::lemma_authenticated_typed_yaml_value_binding_is_unique(
        graph,
        schema,
        yaml_node_index,
        schema_node_index,
        first,
        second,
    );
}

#[test]
fn pure_scalar_and_collection_tag_compatibility_is_exact() {
    proof {
        assert(crucible_yaml::lower_typed::typed_scalar_kind_matches_spec(
            TypedSchemaValueKind::String,
            ResolvedScalarTag::CoreString,
            crucible_yaml::resolve_scalar_value::ResolvedScalarValueView::String,
        ));
        assert(!crucible_yaml::lower_typed::typed_scalar_kind_matches_spec(
            TypedSchemaValueKind::CustomScalar,
            ResolvedScalarTag::CoreString,
            crucible_yaml::resolve_scalar_value::ResolvedScalarValueView::String,
        ));
        assert(crucible_yaml::lower_typed::typed_scalar_kind_matches_spec(
            TypedSchemaValueKind::CustomScalar,
            ResolvedScalarTag::CustomGlobal,
            crucible_yaml::resolve_scalar_value::ResolvedScalarValueView::String,
        ));
        assert(!crucible_yaml::lower_typed::typed_scalar_kind_matches_spec(
            TypedSchemaValueKind::PositiveInfinity,
            ResolvedScalarTag::CoreFloat,
            crucible_yaml::resolve_scalar_value::ResolvedScalarValueView::NegativeInfinity,
        ));

        assert(crucible_yaml::lower_typed::typed_collection_kind_matches_spec(
            TypedSchemaValueKind::Sequence,
            crucible_yaml::CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CoreSequence,
        ));
        assert(!crucible_yaml::lower_typed::typed_collection_kind_matches_spec(
            TypedSchemaValueKind::Sequence,
            crucible_yaml::CanonicalYamlNodeKind::Sequence,
            ResolvedCollectionTag::CustomLocal,
        ));
        assert(crucible_yaml::lower_typed::typed_collection_kind_matches_spec(
            TypedSchemaValueKind::CustomMapping,
            crucible_yaml::CanonicalYamlNodeKind::Mapping,
            ResolvedCollectionTag::CustomGlobal,
        ));
    }
}

} // verus!
