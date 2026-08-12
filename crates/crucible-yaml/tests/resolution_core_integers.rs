use crucible_yaml::{
    convert_core_integer, CoreIntegerErrorKind, CoreIntegerLimits, CORE_INTEGER_MAGNITUDE_RADIX,
    MAX_PROFILE1_CORE_INTEGER_LIMBS, MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
};

fn code_points(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

fn convert(value: &str) -> crucible_yaml::CoreInteger {
    convert_core_integer(
        &code_points(value),
        CoreIntegerLimits::new(
            MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
            MAX_PROFILE1_CORE_INTEGER_LIMBS,
        ),
    )
    .expect("fixture is a bounded Core integer")
}

#[test]
fn decimal_conversion_is_arbitrary_width_and_canonical() {
    let integer = convert("340282366920938463463374607431768211456");
    assert!(!integer.negative());
    assert_eq!(
        integer.limbs(),
        &[768_211_456, 374_607_431, 938_463_463, 282_366_920, 340]
    );
    assert_eq!(CORE_INTEGER_MAGNITUDE_RADIX, 1_000_000_000);
}

#[test]
fn decimal_octal_and_hexadecimal_share_one_magnitude_representation() {
    let decimal = convert("65535");
    let octal = convert("0o177777");
    let hexadecimal = convert("0xFFFF");
    assert_eq!(decimal, octal);
    assert_eq!(octal, hexadecimal);

    let two_to_64 = convert("0x10000000000000000");
    assert_eq!(two_to_64.limbs(), &[709_551_616, 446_744_073, 18]);
}

#[test]
fn sign_and_zero_normalization_are_exact() {
    let negative = convert("-00042");
    assert!(negative.negative());
    assert_eq!(negative.limbs(), &[42]);

    for spelling in ["0", "-0", "+000", "0o000", "0x000"] {
        let zero = convert(spelling);
        assert!(!zero.negative(), "{spelling:?}");
        assert_eq!(zero.limbs(), &[0], "{spelling:?}");
    }
}

#[test]
fn non_integer_core_scalars_are_typed_errors() {
    for spelling in ["true", "1.0", "yes", "-0x2a", "0o8"] {
        let error = convert_core_integer(&code_points(spelling), CoreIntegerLimits::new(32, 32))
            .expect_err("fixture is not a Core integer");
        assert_eq!(
            error.kind(),
            CoreIntegerErrorKind::NotInteger,
            "{spelling:?}"
        );
        assert_eq!(error.code_point_index(), 0, "{spelling:?}");
    }
}

#[test]
fn caller_limits_report_the_first_excluded_input_or_limb() {
    let error = convert_core_integer(
        &code_points("123"),
        CoreIntegerLimits::new(2, MAX_PROFILE1_CORE_INTEGER_LIMBS),
    )
    .expect_err("the third code point is excluded");
    assert_eq!(error.kind(), CoreIntegerErrorKind::InputLimitExceeded);
    assert_eq!(error.code_point_index(), 2);

    let error = convert_core_integer(&code_points("1000000000"), CoreIntegerLimits::new(10, 1))
        .expect_err("the final digit requires the second canonical limb");
    assert_eq!(error.kind(), CoreIntegerErrorKind::MagnitudeLimitExceeded);
    assert_eq!(error.code_point_index(), 9);

    let error = convert_core_integer(&code_points("-0"), CoreIntegerLimits::new(2, 0))
        .expect_err("even canonical zero requires one limb");
    assert_eq!(error.kind(), CoreIntegerErrorKind::MagnitudeLimitExceeded);
    assert_eq!(error.code_point_index(), 1);
}

#[test]
fn deterministic_cross_base_sample_matches_an_independent_u128_oracle() {
    let mut high = 0x243f_6a88_85a3_08d3u64;
    let mut low = 0x1319_8a2e_0370_7344u64;
    for _ in 0..512 {
        high = high.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        low = low.wrapping_mul(1_442_695_040_888_963_407).wrapping_add(33);
        let value = (u128::from(high) << 64) | u128::from(low);
        let mut remaining = value;
        let mut expected = Vec::new();
        if remaining == 0 {
            expected.push(0);
        } else {
            while remaining > 0 {
                expected.push((remaining % u128::from(CORE_INTEGER_MAGNITUDE_RADIX)) as u32);
                remaining /= u128::from(CORE_INTEGER_MAGNITUDE_RADIX);
            }
        }
        for spelling in [
            value.to_string(),
            format!("0o{value:o}"),
            format!("0x{value:X}"),
        ] {
            let converted = convert(&spelling);
            assert!(!converted.negative(), "{spelling}");
            assert_eq!(converted.limbs(), expected, "{spelling}");
        }
    }
}
