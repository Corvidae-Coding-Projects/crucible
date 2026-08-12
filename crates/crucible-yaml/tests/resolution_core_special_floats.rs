use crucible_yaml::{
    convert_core_special_float, CoreSpecialFloat, CoreSpecialFloatErrorKind,
    CoreSpecialFloatLimits, MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
};

fn code_points(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

fn convert(value: &str) -> CoreSpecialFloat {
    convert_core_special_float(
        &code_points(value),
        CoreSpecialFloatLimits::new(MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS),
    )
    .expect("fixture is a bounded Core infinity or NaN")
}

#[test]
fn every_core_infinity_spelling_has_one_signed_value() {
    for spelling in [".inf", ".Inf", ".INF", "+.inf", "+.Inf", "+.INF"] {
        assert_eq!(convert(spelling), CoreSpecialFloat::PositiveInfinity);
    }
    for spelling in ["-.inf", "-.Inf", "-.INF"] {
        assert_eq!(convert(spelling), CoreSpecialFloat::NegativeInfinity);
    }
}

#[test]
fn every_core_nan_spelling_has_one_canonical_value() {
    for spelling in [".nan", ".NaN", ".NAN"] {
        assert_eq!(convert(spelling), CoreSpecialFloat::NotANumber);
    }
}

#[test]
fn finite_and_nonfloat_scalars_are_typed_errors() {
    for spelling in ["1.0", "1", "true", "yes", "+.nan", "-.NaN", ".infinity"] {
        let error =
            convert_core_special_float(&code_points(spelling), CoreSpecialFloatLimits::new(32))
                .expect_err("fixture is not a Core infinity or NaN");
        assert_eq!(error.kind(), CoreSpecialFloatErrorKind::NotSpecialFloat);
        assert_eq!(error.code_point_index(), 0);
    }
}

#[test]
fn caller_input_limit_reports_the_first_excluded_code_point() {
    let error = convert_core_special_float(&code_points("-.Inf"), CoreSpecialFloatLimits::new(4))
        .expect_err("the final input code point is excluded");
    assert_eq!(error.kind(), CoreSpecialFloatErrorKind::InputLimitExceeded);
    assert_eq!(error.code_point_index(), 4);
}
