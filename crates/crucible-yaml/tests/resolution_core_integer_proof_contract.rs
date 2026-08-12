#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{convert_core_integer, CoreIntegerErrorKind, CoreIntegerLimits};
use vstd::prelude::*;

verus! {

proof fn lemma_short_core_integer_model_is_exact()
    ensures
        crucible_yaml::resolve_integer::convert_core_integer_spec(
            seq![0x34u32, 0x32u32],
            crucible_yaml::resolve_integer::CoreIntegerLimitsView {
                max_code_points: 2,
                max_limbs: 2,
            },
        ) == Ok(
            crucible_yaml::resolve_integer::CoreIntegerView { negative: false, limbs: seq![42u32] },
        ),
        crucible_yaml::resolve_integer::convert_core_integer_spec(
            seq![0x2du32, 0x30u32],
            crucible_yaml::resolve_integer::CoreIntegerLimitsView {
                max_code_points: 2,
                max_limbs: 0,
            },
        ) == Err(
            crucible_yaml::resolve_integer::CoreIntegerErrorView {
                kind: CoreIntegerErrorKind::MagnitudeLimitExceeded,
                code_point_index: 1,
            },
        ),
{
    reveal(crucible_yaml::resolve_integer::convert_core_integer_spec);
    reveal(crucible_yaml::resolve_integer::effective_core_integer_limb_limit_spec);
    reveal(crucible_yaml::resolve_integer::core_integer_digit_value_spec);
    reveal(crucible_yaml::resolve_integer::core_integer_multiplier_spec);
    reveal(crucible_yaml::resolve_integer::core_magnitude_mul_add_spec);
    reveal_with_fuel(crucible_yaml::resolve_integer::core_magnitude_mul_add_tail_spec, 3);
    reveal_with_fuel(crucible_yaml::resolve_integer::trim_core_magnitude_spec, 3);
    reveal_with_fuel(crucible_yaml::resolve_integer::core_integer_digits_spec, 3);
    reveal(crucible_yaml::resolve_integer::core_integer_zero_spec);
    reveal(crucible_yaml::resolve::classify_core_plain_scalar_spec);
    reveal(crucible_yaml::resolve::effective_core_scalar_limit_spec);
    reveal(crucible_yaml::resolve::classify_core_plain_scalar_unbounded_spec);
    reveal(crucible_yaml::resolve::core_null_spec);
    reveal(crucible_yaml::resolve::core_true_spec);
    reveal(crucible_yaml::resolve::core_false_spec);
    reveal(crucible_yaml::resolve::core_decimal_integer_spec);
    reveal(crucible_yaml::resolve::core_prefixed_integer_spec);
    reveal(crucible_yaml::resolve::core_sign_body_start_spec);
    reveal(crucible_yaml::resolve::core_digit_for_base_spec);
    reveal_with_fuel(crucible_yaml::resolve::core_digit_run_end_spec, 3);
}

#[test]
fn executable_nonempty_integer_conversion_has_the_exact_pure_result() {
    let input: &[u32] = &[0x34, 0x32];
    let limits = CoreIntegerLimits::new(2, 2);
    let _result = convert_core_integer(input, limits);
    proof {
        lemma_short_core_integer_model_is_exact();
        assert(crucible_yaml::resolve_integer::convert_core_integer_spec(input@, limits@) == Ok(
            crucible_yaml::resolve_integer::CoreIntegerView { negative: false, limbs: seq![42u32] },
        ));
    }
    match _result {
        Ok(_integer) => {
            assert(_integer@ == crucible_yaml::resolve_integer::CoreIntegerView {
                negative: false,
                limbs: seq![42u32],
            });
        },
        Err(_) => assert(false),
    }
}

#[test]
fn pure_zero_limb_limit_error_identifies_the_first_magnitude_digit() {
    proof {
        lemma_short_core_integer_model_is_exact();
        let input = seq![0x2du32, 0x30u32];
        let limits = crucible_yaml::resolve_integer::CoreIntegerLimitsView {
            max_code_points: 2,
            max_limbs: 0,
        };
        assert(crucible_yaml::resolve_integer::convert_core_integer_spec(input, limits) == Err(
            crucible_yaml::resolve_integer::CoreIntegerErrorView {
                kind: CoreIntegerErrorKind::MagnitudeLimitExceeded,
                code_point_index: 1,
            },
        ));
    }
}

} // verus!
