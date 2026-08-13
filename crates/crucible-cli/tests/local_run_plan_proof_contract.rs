use crucible_cli::{
    prepare_local_execution, EffectiveExecutionConfiguration, LocalExecutionPlan, LocalRunPlanError,
};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "wrapper exposes the executable plan contract to Verus")]
fn accepted_plan_satisfies_the_public_execution_contract(
    configuration: &EffectiveExecutionConfiguration,
) -> (result: Result<LocalExecutionPlan, LocalRunPlanError>)
    requires
        crucible_cli::effective_execution_configuration_well_formed_spec(configuration@),
    ensures
        result is Ok ==> crucible_cli::local_execution_plan_well_formed_spec(result.unwrap()@),
        result is Ok ==> crucible_cli::local_execution_plan_matches_configuration_spec(
            configuration@,
            result.unwrap()@,
        ),
{
    prepare_local_execution(configuration)
}

} // verus!
fn main() {}
