#![allow(unused_imports)]
#![allow(clippy::single_match)]

use crucible_yaml::{
    convert_core_special_float, CoreSpecialFloat, CoreSpecialFloatErrorKind, CoreSpecialFloatLimits,
};
use vstd::prelude::*;

verus! {

#[test]
fn executable_special_float_has_the_exact_pure_result() {
    let input: &[u32] = &[0x2d, 0x2e, 0x49, 0x6e, 0x66];
    let limits = CoreSpecialFloatLimits::new(5);
    let result = convert_core_special_float(input, limits);
    proof {
        assert(crucible_yaml::resolve_special_float::convert_core_special_float_spec(
            input@,
            limits@,
        ) == Ok(CoreSpecialFloat::NegativeInfinity));
    }
    match result {
        Ok(_value) => assert(_value == CoreSpecialFloat::NegativeInfinity),
        Err(_) => assert(false),
    }
}

#[test]
fn pure_special_float_limit_error_is_exact() {
    proof {
        assert(crucible_yaml::resolve_special_float::convert_core_special_float_spec(
            seq![0x2eu32, 0x4eu32, 0x61u32, 0x4eu32],
            crucible_yaml::resolve_special_float::CoreSpecialFloatLimitsView { max_code_points: 3 },
        ) == Err(
            crucible_yaml::resolve_special_float::CoreSpecialFloatErrorView {
                kind: CoreSpecialFloatErrorKind::InputLimitExceeded,
                code_point_index: 3,
            },
        ));
    }
}

} // verus!
