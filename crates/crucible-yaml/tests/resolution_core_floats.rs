use crucible_yaml::{
    convert_core_finite_float, CoreFiniteFloat, CoreFiniteFloatErrorKind, CoreFiniteFloatLimits,
    MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS, MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
    MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
};

fn code_points(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

fn convert(value: &str) -> CoreFiniteFloat {
    convert_core_finite_float(
        &code_points(value),
        CoreFiniteFloatLimits::new(
            MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
            MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
            MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS,
        ),
    )
    .expect("fixture is a bounded Core finite float")
}

fn decimal_digits_le(value: &str) -> Vec<u8> {
    value.bytes().rev().map(|byte| byte - b'0').collect()
}

#[test]
fn equivalent_decimal_spellings_have_one_exact_value() {
    let expected = convert("1.0");
    for spelling in ["1e0", "10e-1", "0.10e1", "1.0000"] {
        assert_eq!(convert(spelling), expected, "{spelling:?}");
    }
    assert!(!expected.negative());
    assert_eq!(expected.coefficient_digits_le(), &[1]);
    assert!(!expected.exponent_negative());
    assert_eq!(expected.exponent_digits_le(), &[0]);

    let expected = convert("123.0");
    for spelling in ["1.23e2", "123e0", ".0012300e5"] {
        assert_eq!(convert(spelling), expected, "{spelling:?}");
    }
    assert_eq!(expected.coefficient_digits_le(), &[3, 2, 1]);
    assert_eq!(expected.exponent_digits_le(), &[0]);
}

#[test]
fn coefficient_and_exponent_are_independent_of_host_width() {
    let large = convert("340282366920938463463374607431768211456.0");
    assert_eq!(
        large.coefficient_digits_le(),
        decimal_digits_le("340282366920938463463374607431768211456")
    );
    assert_eq!(large.exponent_digits_le(), &[0]);

    let huge_positive_exponent = convert("1e100000000000000000000");
    assert!(!huge_positive_exponent.exponent_negative());
    assert_eq!(
        huge_positive_exponent.exponent_digits_le(),
        decimal_digits_le("100000000000000000000")
    );

    let huge_negative_exponent = convert("1e-100000000000000000000");
    assert!(huge_negative_exponent.exponent_negative());
    assert_eq!(
        huge_negative_exponent.exponent_digits_le(),
        decimal_digits_le("100000000000000000000")
    );
}

#[test]
fn long_leading_zero_runs_normalize_exactly() {
    let mut spelling = String::with_capacity(100_002);
    spelling.push('.');
    spelling.extend(std::iter::repeat_n('0', 100_000));
    spelling.push('1');

    let value = convert(&spelling);
    assert_eq!(value.coefficient_digits_le(), &[1]);
    assert!(value.exponent_negative());
    assert_eq!(value.exponent_digits_le(), decimal_digits_le("100001"));
}

#[test]
fn negative_zero_remains_distinct_but_has_one_canonical_scale() {
    for spelling in ["-0.0", "-0e999", "-0.000e-999"] {
        let zero = convert(spelling);
        assert!(zero.negative(), "{spelling:?}");
        assert_eq!(zero.coefficient_digits_le(), &[0], "{spelling:?}");
        assert!(!zero.exponent_negative(), "{spelling:?}");
        assert_eq!(zero.exponent_digits_le(), &[0], "{spelling:?}");
    }
    assert_ne!(convert("-0.0"), convert("0.0"));
}

#[test]
fn nonfinite_and_nonfloat_core_scalars_are_typed_errors() {
    for spelling in ["1", "true", "yes", ".inf", "-.Inf", ".NaN", "1e"] {
        let error = convert_core_finite_float(
            &code_points(spelling),
            CoreFiniteFloatLimits::new(32, 32, 32),
        )
        .expect_err("fixture is not a Core finite float");
        assert_eq!(error.kind(), CoreFiniteFloatErrorKind::NotFiniteFloat);
        assert_eq!(error.code_point_index(), 0);
    }
}

#[test]
fn every_caller_lowered_limit_has_an_exact_source_index() {
    let error = convert_core_finite_float(&code_points("1.0"), CoreFiniteFloatLimits::new(2, 8, 8))
        .expect_err("the third input code point is excluded");
    assert_eq!(error.kind(), CoreFiniteFloatErrorKind::InputLimitExceeded);
    assert_eq!(error.code_point_index(), 2);

    let error = convert_core_finite_float(
        &code_points("1000000001.0"),
        CoreFiniteFloatLimits::new(12, 1, 8),
    )
    .expect_err("the canonical coefficient requires ten decimal digits");
    assert_eq!(
        error.kind(),
        CoreFiniteFloatErrorKind::CoefficientLimitExceeded
    );
    assert_eq!(error.code_point_index(), 1);

    let error = convert_core_finite_float(
        &code_points("1e1000000000"),
        CoreFiniteFloatLimits::new(12, 8, 1),
    )
    .expect_err("the canonical exponent requires ten decimal digits");
    assert_eq!(
        error.kind(),
        CoreFiniteFloatErrorKind::ExponentLimitExceeded
    );
    assert_eq!(error.code_point_index(), 3);

    let error = convert_core_finite_float(&code_points(".5"), CoreFiniteFloatLimits::new(2, 8, 0))
        .expect_err("the implicit negative exponent requires one limb");
    assert_eq!(
        error.kind(),
        CoreFiniteFloatErrorKind::ExponentLimitExceeded
    );
    assert_eq!(error.code_point_index(), 1);

    let error = convert_core_finite_float(&code_points("1."), CoreFiniteFloatLimits::new(2, 8, 0))
        .expect_err("the implicit zero exponent still occupies one semantic digit");
    assert_eq!(
        error.kind(),
        CoreFiniteFloatErrorKind::ExponentLimitExceeded
    );
    assert_eq!(error.code_point_index(), 0);
}
