use crucible_cli::{
    validate_configuration, ConfigurationError, ConfigurationLimits, ValidatedConfiguration,
};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "wrapper exposes the executable projection contract to Verus")]
fn successful_validation_produces_a_well_formed_execution_projection(
    input: &[u8],
    limits: ConfigurationLimits,
) -> (result: Result<ValidatedConfiguration, ConfigurationError>)
    ensures
        match &result {
            Ok(configuration) => crucible_cli::effective_execution_configuration_well_formed_spec(
                configuration@.execution,
            ),
            Err(_) => true,
        },
{
    validate_configuration(input, limits)
}

proof fn well_formed_execution_projection_has_positive_bounded_controls(
    execution: crucible_cli::EffectiveExecutionConfigurationView,
)
    requires
        crucible_cli::effective_execution_configuration_well_formed_spec(execution),
    ensures
        execution.command.len() > 0,
        execution.timeout_ms > 0,
        execution.memory_mb > 0,
        execution.max_processes > 0,
        execution.max_output_mb > 0,
{
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(crucible_cli::CONFIGURATION_SCHEMA_VERSION, 1);
}
