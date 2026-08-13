#[expect(
    unused_imports,
    reason = "the error kind is referenced only inside Verus proof code"
)]
use crucible_yaml::CoreScalarErrorKind;
use crucible_yaml::{
    classify_core_plain_scalar, CorePlainScalarClass, CoreScalarLimits, CoreScalarRange,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_core_scalar_classification_has_the_exact_pure_result() {
    let input: &[u32] = &[0x2b, 0x31, 0x32, 0x2e, 0x33, 0x30, 0x65, 0x2d, 0x30, 0x34];
    let limits = CoreScalarLimits::new(10);
    let whole = CoreScalarRange::new(1, 3);
    let fraction = CoreScalarRange::new(4, 6);
    let exponent = CoreScalarRange::new(8, 10);
    let _expected = CorePlainScalarClass::FiniteFloat {
        negative: false,
        whole,
        fraction: Some(fraction),
        exponent_negative: true,
        exponent: Some(exponent),
    };
    let _result = classify_core_plain_scalar(input, limits);
    proof {
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
        reveal(crucible_yaml::resolve::core_infinity_spec);
        reveal(crucible_yaml::resolve::core_nan_spec);
        reveal_with_fuel(crucible_yaml::resolve::core_digit_run_end_spec, 11);
        assert(crucible_yaml::resolve::classify_core_plain_scalar_spec(input@, limits@) == Ok(
            _expected,
        ));
    }
    assert(_result == Ok(_expected));
}

#[test]
fn pure_core_scalar_classification_rejects_the_exact_first_excluded_code_point() {
    proof {
        let input = seq![0x74u32, 0x72u32, 0x75u32, 0x65u32];
        let limits = crucible_yaml::resolve::CoreScalarLimitsView { max_code_points: 3 };
        assert(crucible_yaml::resolve::classify_core_plain_scalar_spec(input, limits) == Err(
            crucible_yaml::resolve::CoreScalarErrorView {
                kind: CoreScalarErrorKind::InputLimitExceeded,
                code_point_index: 3,
            },
        ));
    }
}

#[test]
fn pure_core_scalar_classification_does_not_launder_yaml_1_1_booleans() {
    proof {
        let yes = seq![0x79u32, 0x65u32, 0x73u32];
        let limits = crucible_yaml::resolve::CoreScalarLimitsView { max_code_points: 3 };
        assert(crucible_yaml::resolve::classify_core_plain_scalar_spec(yes, limits) == Ok(
            CorePlainScalarClass::String,
        ));
    }
}

} // verus!
