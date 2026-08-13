#![expect(
    unused_imports,
    reason = "imports are consumed by Verus proof checking after ordinary Rust erasure"
)]

use crucible_cli::{
    validate_configuration, ConfigurationLimits, ValidatedConfiguration, ValidatedConfigurationView,
};
use vstd::prelude::*;

verus! {

#[expect(dead_code, reason = "wrapper exists solely to expose the executable contract to Verus")]
fn successful_validation_authenticates_integrity_without_overclaiming_semantic_proof(
    input: &[u8],
    limits: ConfigurationLimits,
) -> (result: Result<ValidatedConfiguration, crucible_cli::ConfigurationError>)
    ensures
        match &result {
            Ok(configuration) => crucible_cli::validated_configuration_integrity_spec(
                input@,
                limits@,
                configuration@,
            ),
            Err(_) => true,
        },
{
    validate_configuration(input, limits)
}

proof fn a_forged_digest_cannot_satisfy_validated_configuration_integrity(
    input: Seq<u8>,
    limits: crucible_cli::ConfigurationLimitsView,
    valid: ValidatedConfigurationView,
    forged: ValidatedConfigurationView,
)
    requires
        crucible_cli::validated_configuration_integrity_spec(input, limits, valid),
        forged.canonical_bytes == valid.canonical_bytes,
        forged.digest != valid.digest,
    ensures
        !crucible_cli::validated_configuration_integrity_spec(input, limits, forged),
{
}

proof fn accepted_limits_are_caller_lowered_beneath_absolute_limits(
    input: Seq<u8>,
    limits: crucible_cli::ConfigurationLimitsView,
    valid: ValidatedConfigurationView,
)
    requires
        crucible_cli::validated_configuration_integrity_spec(input, limits, valid),
    ensures
        input.len() <= crucible_cli::MAX_CONFIGURATION_SOURCE_BYTES,
        valid.canonical_bytes.len() <= crucible_cli::MAX_CONFIGURATION_CANONICAL_BYTES,
        valid.typed_node_count <= crucible_cli::MAX_CONFIGURATION_TYPED_NODES,
        valid.work_count <= crucible_cli::MAX_CONFIGURATION_RENDER_TASKS,
{
    crucible_cli::lemma_validated_configuration_obeys_absolute_limits(input, limits, valid);
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(crucible_cli::CONFIGURATION_SCHEMA_VERSION, 1);
}
