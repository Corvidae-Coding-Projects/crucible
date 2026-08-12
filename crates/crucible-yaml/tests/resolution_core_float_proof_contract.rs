#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{convert_core_finite_float, CoreFiniteFloatErrorKind, CoreFiniteFloatLimits};
use vstd::prelude::*;

verus! {

proof fn lemma_short_core_float_models_are_exact()
    ensures
        crucible_yaml::resolve_float::convert_core_finite_float_spec(
            seq![0x31u32, 0x2eu32, 0x30u32],
            crucible_yaml::resolve_float::CoreFiniteFloatLimitsView {
                max_code_points: 3,
                max_coefficient_digits: 2,
                max_exponent_digits: 2,
            },
        ) == Ok(
            crucible_yaml::resolve_float::CoreFiniteFloatView {
                negative: false,
                coefficient_digits_le: seq![1u8],
                exponent_negative: false,
                exponent_digits_le: seq![0u8],
            },
        ),
        crucible_yaml::resolve_float::convert_core_finite_float_spec(
            seq![0x2eu32, 0x35u32],
            crucible_yaml::resolve_float::CoreFiniteFloatLimitsView {
                max_code_points: 2,
                max_coefficient_digits: 2,
                max_exponent_digits: 0,
            },
        ) == Err(
            crucible_yaml::resolve_float::CoreFiniteFloatErrorView {
                kind: CoreFiniteFloatErrorKind::ExponentLimitExceeded,
                code_point_index: 1,
            },
        ),
        crucible_yaml::resolve_float::convert_core_finite_float_spec(
            seq![0x31u32, 0x2eu32],
            crucible_yaml::resolve_float::CoreFiniteFloatLimitsView {
                max_code_points: 2,
                max_coefficient_digits: 2,
                max_exponent_digits: 0,
            },
        ) == Err(
            crucible_yaml::resolve_float::CoreFiniteFloatErrorView {
                kind: CoreFiniteFloatErrorKind::ExponentLimitExceeded,
                code_point_index: 0,
            },
        ),
{
    reveal(crucible_yaml::resolve_float::convert_core_finite_float_spec);
    reveal(crucible_yaml::resolve_float::effective_float_coefficient_limit_spec);
    reveal(crucible_yaml::resolve_float::effective_float_exponent_limit_spec);
    reveal(crucible_yaml::resolve_float::coefficient_digits_be_spec);
    reveal(crucible_yaml::resolve_float::decimal_range_digits_spec);
    reveal(crucible_yaml::resolve_float::canonicalize_coefficient_spec);
    reveal(crucible_yaml::resolve_float::canonicalize_unsigned_decimal_spec);
    reveal_with_fuel(crucible_yaml::resolve_float::trim_trailing_decimal_zeros_spec, 5);
    reveal_with_fuel(crucible_yaml::resolve_float::trim_leading_decimal_zeros_spec, 5);
    reveal(crucible_yaml::resolve_float::reverse_digits_spec);
    reveal(crucible_yaml::resolve_float::decimal_digits_zero_spec);
    reveal(crucible_yaml::resolve_float::apply_signed_small_decimal_spec);
    reveal(crucible_yaml::resolve_float::apply_signed_small_decimal_raw_spec);
    reveal_with_fuel(crucible_yaml::resolve_float::small_decimal_digits_le_spec, 5);
    reveal_with_fuel(crucible_yaml::resolve_float::add_decimal_magnitudes_le_spec, 5);
    reveal_with_fuel(crucible_yaml::resolve_float::subtract_decimal_magnitudes_le_spec, 5);
    reveal_with_fuel(crucible_yaml::resolve_float::compare_decimal_magnitude_le_spec, 5);
    reveal(crucible_yaml::resolve_float::trim_high_decimal_zeros_le_spec);
    reveal(crucible_yaml::resolve_float::exponent_adjustment_anchor_spec);
    reveal(crucible_yaml::resolve_float::coefficient_source_index_spec);
    reveal(crucible_yaml::resolve::classify_core_plain_scalar_spec);
    reveal(crucible_yaml::resolve::effective_core_scalar_limit_spec);
    reveal(crucible_yaml::resolve::classify_core_plain_scalar_unbounded_spec);
    reveal(crucible_yaml::resolve::core_null_spec);
    reveal(crucible_yaml::resolve::core_true_spec);
    reveal(crucible_yaml::resolve::core_false_spec);
    reveal(crucible_yaml::resolve::core_decimal_integer_spec);
    reveal(crucible_yaml::resolve::core_prefixed_integer_spec);
    reveal(crucible_yaml::resolve::core_finite_float_spec);
    reveal(crucible_yaml::resolve::core_sign_body_start_spec);
    reveal(crucible_yaml::resolve::core_digit_for_base_spec);
    reveal_with_fuel(crucible_yaml::resolve::core_digit_run_end_spec, 5);

    let input = seq![0x31u32, 0x2eu32, 0x30u32];
    let whole = crucible_yaml::resolve::CoreScalarRange { start: 0, end: 1 };
    let fraction = crucible_yaml::resolve::CoreScalarRange { start: 2, end: 3 };
    assert(crucible_yaml::resolve::core_finite_float_spec(input) == Some(
        crucible_yaml::resolve::CorePlainScalarClass::FiniteFloat {
            negative: false,
            whole,
            fraction: Some(fraction),
            exponent_negative: false,
            exponent: None,
        },
    ));
    assert(crucible_yaml::resolve::classify_core_plain_scalar_spec(
        input,
        crucible_yaml::resolve::CoreScalarLimitsView { max_code_points: 3 },
    ) == Ok(
        crucible_yaml::resolve::CorePlainScalarClass::FiniteFloat {
            negative: false,
            whole,
            fraction: Some(fraction),
            exponent_negative: false,
            exponent: None,
        },
    ));
    assert(crucible_yaml::resolve_float::coefficient_digits_be_spec(input, whole, Some(fraction))
        == seq![1u8, 0u8]);
    assert(crucible_yaml::resolve_float::canonicalize_coefficient_spec(seq![1u8, 0u8]) == (
        seq![1u8],
        1u64,
        0u64,
    ));
    assert(crucible_yaml::resolve_float::apply_signed_small_decimal_spec(false, seq![0u8], false, 0)
        == (false, seq![0u8]));
    assert(crucible_yaml::resolve_float::canonicalize_unsigned_decimal_spec(seq![0u8]) == seq![
        0u8,
    ]);
    assert(crucible_yaml::resolve_float::convert_core_finite_float_spec(
        input,
        crucible_yaml::resolve_float::CoreFiniteFloatLimitsView {
            max_code_points: 3,
            max_coefficient_digits: 2,
            max_exponent_digits: 2,
        },
    ) == Ok(
        crucible_yaml::resolve_float::CoreFiniteFloatView {
            negative: false,
            coefficient_digits_le: seq![1u8],
            exponent_negative: false,
            exponent_digits_le: seq![0u8],
        },
    ));
}

#[test]
fn executable_nonempty_finite_float_has_the_exact_normalized_pure_result() {
    let input: &[u32] = &[0x31, 0x2e, 0x30];
    let limits = CoreFiniteFloatLimits::new(3, 2, 2);
    let _result = convert_core_finite_float(input, limits);
    proof {
        lemma_short_core_float_models_are_exact();
        assert(crucible_yaml::resolve_float::convert_core_finite_float_spec(input@, limits@) == Ok(
            crucible_yaml::resolve_float::CoreFiniteFloatView {
                negative: false,
                coefficient_digits_le: seq![1u8],
                exponent_negative: false,
                exponent_digits_le: seq![0u8],
            },
        ));
    }
    match _result {
        Ok(_float) => {
            assert(_float@ == crucible_yaml::resolve_float::CoreFiniteFloatView {
                negative: false,
                coefficient_digits_le: seq![1u8],
                exponent_negative: false,
                exponent_digits_le: seq![0u8],
            });
        },
        Err(_) => assert(false),
    }
}

#[test]
fn pure_implicit_exponent_limit_error_has_the_exact_fraction_digit() {
    proof {
        lemma_short_core_float_models_are_exact();
        let input = seq![0x2eu32, 0x35u32];
        let limits = crucible_yaml::resolve_float::CoreFiniteFloatLimitsView {
            max_code_points: 2,
            max_coefficient_digits: 2,
            max_exponent_digits: 0,
        };
        assert(crucible_yaml::resolve_float::convert_core_finite_float_spec(input, limits) == Err(
            crucible_yaml::resolve_float::CoreFiniteFloatErrorView {
                kind: CoreFiniteFloatErrorKind::ExponentLimitExceeded,
                code_point_index: 1,
            },
        ));
    }
}

#[test]
fn pure_empty_fraction_anchors_an_implicit_exponent_limit_at_the_coefficient() {
    proof {
        lemma_short_core_float_models_are_exact();
        assert(crucible_yaml::resolve_float::convert_core_finite_float_spec(
            seq![0x31u32, 0x2eu32],
            crucible_yaml::resolve_float::CoreFiniteFloatLimitsView {
                max_code_points: 2,
                max_coefficient_digits: 2,
                max_exponent_digits: 0,
            },
        ) == Err(
            crucible_yaml::resolve_float::CoreFiniteFloatErrorView {
                kind: CoreFiniteFloatErrorKind::ExponentLimitExceeded,
                code_point_index: 0,
            },
        ));
    }
}

} // verus!
