//! Verified canonical conversion of YAML 1.2.2 Core infinities and NaN.
#[allow(unused_imports)]
use crate::resolve::CoreScalarLimitsView;
use crate::resolve::{
    classify_core_plain_scalar, CorePlainScalarClass, CoreScalarErrorKind, CoreScalarLimits,
};
use vstd::prelude::*;

verus! {

pub const CORE_SPECIAL_FLOAT_CONVERSION_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreSpecialFloatLimits {
    max_code_points: u64,
}

#[verifier::ext_equal]
pub struct CoreSpecialFloatLimitsView {
    pub max_code_points: u64,
}

impl View for CoreSpecialFloatLimits {
    type V = CoreSpecialFloatLimitsView;

    closed spec fn view(&self) -> CoreSpecialFloatLimitsView {
        CoreSpecialFloatLimitsView { max_code_points: self.max_code_points }
    }
}

impl CoreSpecialFloatLimits {
    pub fn new(max_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (CoreSpecialFloatLimitsView { max_code_points }),
    {
        Self { max_code_points }
    }

    pub fn max_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_code_points,
    {
        self.max_code_points
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CoreSpecialFloat {
    PositiveInfinity,
    NegativeInfinity,
    NotANumber,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CoreSpecialFloatErrorKind {
    InputLimitExceeded,
    NotSpecialFloat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreSpecialFloatError {
    kind: CoreSpecialFloatErrorKind,
    code_point_index: u64,
}

#[verifier::ext_equal]
pub struct CoreSpecialFloatErrorView {
    pub kind: CoreSpecialFloatErrorKind,
    pub code_point_index: u64,
}

impl View for CoreSpecialFloatError {
    type V = CoreSpecialFloatErrorView;

    closed spec fn view(&self) -> CoreSpecialFloatErrorView {
        CoreSpecialFloatErrorView { kind: self.kind, code_point_index: self.code_point_index }
    }
}

impl CoreSpecialFloatError {
    fn at(kind: CoreSpecialFloatErrorKind, code_point_index: u64) -> (error: Self)
        ensures
            error@ == (CoreSpecialFloatErrorView { kind, code_point_index }),
    {
        Self { kind, code_point_index }
    }

    pub fn kind(&self) -> (kind: CoreSpecialFloatErrorKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn code_point_index(&self) -> (index: u64)
        ensures
            index == self@.code_point_index,
    {
        self.code_point_index
    }
}

pub open spec fn convert_core_special_float_spec(
    input: Seq<u32>,
    limits: CoreSpecialFloatLimitsView,
) -> Result<CoreSpecialFloat, CoreSpecialFloatErrorView> {
    match crate::resolve::classify_core_plain_scalar_spec(
        input,
        CoreScalarLimitsView { max_code_points: limits.max_code_points },
    ) {
        Err(error) => Err(
            CoreSpecialFloatErrorView {
                kind: match error.kind {
                    CoreScalarErrorKind::InputLimitExceeded => {
                        CoreSpecialFloatErrorKind::InputLimitExceeded
                    },
                },
                code_point_index: error.code_point_index,
            },
        ),
        Ok(CorePlainScalarClass::Infinity { negative }) => Ok(
            if negative {
                CoreSpecialFloat::NegativeInfinity
            } else {
                CoreSpecialFloat::PositiveInfinity
            },
        ),
        Ok(CorePlainScalarClass::NotANumber) => Ok(CoreSpecialFloat::NotANumber),
        Ok(_) => Err(
            CoreSpecialFloatErrorView {
                kind: CoreSpecialFloatErrorKind::NotSpecialFloat,
                code_point_index: 0,
            },
        ),
    }
}

/// Convert one YAML 1.2.2 Core infinity or NaN spelling to its canonical semantic value.
pub fn convert_core_special_float(input: &[u32], limits: CoreSpecialFloatLimits) -> (result: Result<
    CoreSpecialFloat,
    CoreSpecialFloatError,
>)
    ensures
        convert_core_special_float_spec(input@, limits@) == match result {
            Ok(value) => Ok(value),
            Err(error) => Err(error@),
        },
{
    let scalar_limits = CoreScalarLimits::new(limits.max_code_points);
    let class = match classify_core_plain_scalar(input, scalar_limits) {
        Ok(class) => class,
        Err(error) => {
            let converted = CoreSpecialFloatError::at(
                match error.kind() {
                    CoreScalarErrorKind::InputLimitExceeded => {
                        CoreSpecialFloatErrorKind::InputLimitExceeded
                    },
                },
                error.code_point_index(),
            );
            proof {
                reveal(convert_core_special_float_spec);
            }
            return Err(converted);
        },
    };
    let value = match class {
        CorePlainScalarClass::Infinity { negative } => if negative {
            CoreSpecialFloat::NegativeInfinity
        } else {
            CoreSpecialFloat::PositiveInfinity
        },
        CorePlainScalarClass::NotANumber => CoreSpecialFloat::NotANumber,
        _ => {
            let error = CoreSpecialFloatError::at(CoreSpecialFloatErrorKind::NotSpecialFloat, 0);
            proof {
                reveal(convert_core_special_float_spec);
            }
            return Err(error);
        },
    };
    proof {
        reveal(convert_core_special_float_spec);
    }
    Ok(value)
}

} // verus!
