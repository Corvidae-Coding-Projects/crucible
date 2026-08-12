use crucible_yaml::{
    classify_core_plain_scalar, CoreIntegerBase, CorePlainScalarClass, CoreScalarErrorKind,
    CoreScalarLimits, CoreScalarRange,
};

fn code_points(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

fn classify(value: &str) -> CorePlainScalarClass {
    classify_core_plain_scalar(
        &code_points(value),
        CoreScalarLimits::new(crucible_yaml::MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS),
    )
    .expect("fixture is within the profile scalar cap")
}

#[test]
fn core_null_boolean_and_legacy_spellings_are_exact() {
    for value in ["", "~", "null", "Null", "NULL"] {
        assert_eq!(classify(value), CorePlainScalarClass::Null, "{value:?}");
    }
    for value in ["true", "True", "TRUE"] {
        assert_eq!(
            classify(value),
            CorePlainScalarClass::Boolean(true),
            "{value:?}"
        );
    }
    for value in ["false", "False", "FALSE"] {
        assert_eq!(
            classify(value),
            CorePlainScalarClass::Boolean(false),
            "{value:?}"
        );
    }
    for value in ["yes", "Yes", "YES", "no", "on", "off", "y", "n"] {
        assert_eq!(classify(value), CorePlainScalarClass::String, "{value:?}");
    }
}

#[test]
fn core_integer_bases_signs_and_digit_ranges_are_exact() {
    assert_eq!(
        classify("-019"),
        CorePlainScalarClass::Integer {
            negative: true,
            base: CoreIntegerBase::Decimal,
            digits: CoreScalarRange::new(1, 4),
        }
    );
    assert_eq!(
        classify("+12"),
        CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Decimal,
            digits: CoreScalarRange::new(1, 3),
        }
    );
    assert_eq!(
        classify("0123"),
        CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Decimal,
            digits: CoreScalarRange::new(0, 4),
        }
    );
    assert_eq!(
        classify("0o707"),
        CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Octal,
            digits: CoreScalarRange::new(2, 5),
        }
    );
    assert_eq!(
        classify("0x3aF"),
        CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Hexadecimal,
            digits: CoreScalarRange::new(2, 5),
        }
    );
    assert_eq!(classify("-0x2A"), CorePlainScalarClass::String);
    assert_eq!(classify("+0o17"), CorePlainScalarClass::String);
    assert_eq!(classify("0o8"), CorePlainScalarClass::String);
    assert_eq!(classify("0x"), CorePlainScalarClass::String);
}

#[test]
fn core_finite_float_ranges_and_special_values_are_exact() {
    assert_eq!(
        classify("-.5"),
        CorePlainScalarClass::FiniteFloat {
            negative: true,
            whole: CoreScalarRange::new(1, 1),
            fraction: Some(CoreScalarRange::new(2, 3)),
            exponent_negative: false,
            exponent: None,
        }
    );
    assert_eq!(
        classify("0."),
        CorePlainScalarClass::FiniteFloat {
            negative: false,
            whole: CoreScalarRange::new(0, 1),
            fraction: Some(CoreScalarRange::new(2, 2)),
            exponent_negative: false,
            exponent: None,
        }
    );
    assert_eq!(
        classify("+12.30e-04"),
        CorePlainScalarClass::FiniteFloat {
            negative: false,
            whole: CoreScalarRange::new(1, 3),
            fraction: Some(CoreScalarRange::new(4, 6)),
            exponent_negative: true,
            exponent: Some(CoreScalarRange::new(8, 10)),
        }
    );
    assert_eq!(
        classify("1e0"),
        CorePlainScalarClass::FiniteFloat {
            negative: false,
            whole: CoreScalarRange::new(0, 1),
            fraction: None,
            exponent_negative: false,
            exponent: Some(CoreScalarRange::new(2, 3)),
        }
    );
    assert_eq!(
        classify("-.Inf"),
        CorePlainScalarClass::Infinity { negative: true }
    );
    assert_eq!(
        classify("+.INF"),
        CorePlainScalarClass::Infinity { negative: false }
    );
    assert_eq!(
        classify("+.inf"),
        CorePlainScalarClass::Infinity { negative: false }
    );
    assert_eq!(classify(".NaN"), CorePlainScalarClass::NotANumber);
    assert_eq!(classify(".nan"), CorePlainScalarClass::NotANumber);
    assert_eq!(classify(".NAN"), CorePlainScalarClass::NotANumber);
    for value in [".", "1e", "1e+", ".inF", "nan", "+.NaN", "-.nan"] {
        assert_eq!(classify(value), CorePlainScalarClass::String, "{value:?}");
    }
}

#[test]
fn caller_lowered_scalar_cap_reports_the_first_excluded_code_point() {
    let error = classify_core_plain_scalar(&code_points("true"), CoreScalarLimits::new(3))
        .expect_err("the fourth code point is excluded");
    assert_eq!(error.kind(), CoreScalarErrorKind::InputLimitExceeded);
    assert_eq!(error.code_point_index(), 3);

    let error = classify_core_plain_scalar(&code_points("~"), CoreScalarLimits::new(0))
        .expect_err("the first code point is excluded");
    assert_eq!(error.kind(), CoreScalarErrorKind::InputLimitExceeded);
    assert_eq!(error.code_point_index(), 0);

    assert_eq!(
        classify_core_plain_scalar(&[], CoreScalarLimits::new(0)),
        Ok(CorePlainScalarClass::Null)
    );
}

#[test]
fn absolute_scalar_cap_reports_the_first_excluded_code_point_before_scanning() {
    let input = vec![0x61; crucible_yaml::MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS as usize + 1];
    let error = classify_core_plain_scalar(&input, CoreScalarLimits::new(u64::MAX))
        .expect_err("the absolute profile cap cannot be raised by a caller");
    assert_eq!(error.kind(), CoreScalarErrorKind::InputLimitExceeded);
    assert_eq!(
        error.code_point_index(),
        crucible_yaml::MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS
    );
}
