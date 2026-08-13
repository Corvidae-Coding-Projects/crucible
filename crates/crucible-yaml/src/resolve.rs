//! Verified YAML 1.2.2 Core Schema plain-scalar classification.
//!
//! This module owns only the deterministic classification boundary.  Arbitrary-width numeric
//! conversion and CST-wide semantic resolution build on the exact source ranges returned here.
use vstd::prelude::*;

verus! {

pub const CORE_SCALAR_CLASSIFIER_VERSION: u16 = 1;

pub const MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CoreIntegerBase {
    Decimal,
    Octal,
    Hexadecimal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub struct CoreScalarRange {
    pub start: u64,
    pub end: u64,
}

impl CoreScalarRange {
    pub fn new(start: u64, end: u64) -> (range: Self)
        ensures
            range == (CoreScalarRange { start, end }),
    {
        Self { start, end }
    }

    pub fn start(&self) -> (start: u64)
        ensures
            start == self.start,
    {
        self.start
    }

    pub fn end(&self) -> (end: u64)
        ensures
            end == self.end,
    {
        self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum CorePlainScalarClass {
    Null,
    Boolean(bool),
    Integer { negative: bool, base: CoreIntegerBase, digits: CoreScalarRange },
    FiniteFloat {
        negative: bool,
        whole: CoreScalarRange,
        fraction: Option<CoreScalarRange>,
        exponent_negative: bool,
        exponent: Option<CoreScalarRange>,
    },
    Infinity { negative: bool },
    NotANumber,
    String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreScalarLimits {
    max_code_points: u64,
}

#[verifier::ext_equal]
pub struct CoreScalarLimitsView {
    pub max_code_points: u64,
}

impl View for CoreScalarLimits {
    type V = CoreScalarLimitsView;

    closed spec fn view(&self) -> CoreScalarLimitsView {
        CoreScalarLimitsView { max_code_points: self.max_code_points }
    }
}

impl CoreScalarLimits {
    pub fn new(max_code_points: u64) -> (limits: Self)
        ensures
            limits@.max_code_points == max_code_points,
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
#[non_exhaustive]
pub enum CoreScalarErrorKind {
    InputLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreScalarError {
    kind: CoreScalarErrorKind,
    code_point_index: u64,
}

#[verifier::ext_equal]
pub struct CoreScalarErrorView {
    pub kind: CoreScalarErrorKind,
    pub code_point_index: u64,
}

impl View for CoreScalarError {
    type V = CoreScalarErrorView;

    closed spec fn view(&self) -> CoreScalarErrorView {
        CoreScalarErrorView { kind: self.kind, code_point_index: self.code_point_index }
    }
}

impl CoreScalarError {
    fn at(kind: CoreScalarErrorKind, code_point_index: u64) -> (error: Self)
        ensures
            error@ == (CoreScalarErrorView { kind, code_point_index }),
    {
        Self { kind, code_point_index }
    }

    pub fn kind(&self) -> (kind: CoreScalarErrorKind)
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

pub open spec fn effective_core_scalar_limit_spec(limits: CoreScalarLimitsView) -> u64 {
    if limits.max_code_points < MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS {
        limits.max_code_points
    } else {
        MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS
    }
}

pub open spec fn core_digit_for_base_spec(code_point: u32, base: CoreIntegerBase) -> bool {
    match base {
        CoreIntegerBase::Decimal => 0x30 <= code_point <= 0x39,
        CoreIntegerBase::Octal => 0x30 <= code_point <= 0x37,
        CoreIntegerBase::Hexadecimal => {
            0x30 <= code_point <= 0x39 || 0x41 <= code_point <= 0x46 || 0x61 <= code_point <= 0x66
        },
    }
}

#[expect(clippy::manual_range_contains, reason = "arithmetic spelling mirrors the Verus specification and proof obligations")]
fn core_digit_for_base(code_point: u32, base: CoreIntegerBase) -> (digit: bool)
    ensures
        digit == core_digit_for_base_spec(code_point, base),
{
    match base {
        CoreIntegerBase::Decimal => 0x30 <= code_point && code_point <= 0x39,
        CoreIntegerBase::Octal => 0x30 <= code_point && code_point <= 0x37,
        CoreIntegerBase::Hexadecimal => {
            (0x30 <= code_point && code_point <= 0x39) || (0x41 <= code_point && code_point <= 0x46)
                || (0x61 <= code_point && code_point <= 0x66)
        },
    }
}

pub open spec fn core_digit_run_end_spec(
    input: Seq<u32>,
    index: int,
    base: CoreIntegerBase,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < 0 || index >= input.len() || fuel == 0 {
        index
    } else if core_digit_for_base_spec(input[index], base) {
        core_digit_run_end_spec(input, index + 1, base, (fuel - 1) as nat)
    } else {
        index
    }
}

pub proof fn lemma_core_digit_run_to_target_has_only_digits(
    input: Seq<u32>,
    index: int,
    target: int,
    base: CoreIntegerBase,
    fuel: nat,
)
    requires
        0 <= index <= target <= input.len(),
        fuel >= (target - index) as nat,
        core_digit_run_end_spec(input, index, base, fuel) == target,
    ensures
        forall|candidate: int|
            index <= candidate < target ==> core_digit_for_base_spec(input[candidate], base),
    decreases fuel,
{
    if index < target {
        assert(fuel > 0);
        reveal(core_digit_run_end_spec);
        assert(core_digit_for_base_spec(input[index], base));
        lemma_core_digit_run_to_target_has_only_digits(
            input,
            index + 1,
            target,
            base,
            (fuel - 1) as nat,
        );
        assert forall|candidate: int| index <= candidate < target implies core_digit_for_base_spec(
            input[candidate],
            base,
        ) by {
            if candidate > index {
                assert(index + 1 <= candidate);
            }
        }
    }
}

pub proof fn lemma_core_digit_run_to_end_has_only_digits(
    input: Seq<u32>,
    index: int,
    base: CoreIntegerBase,
    fuel: nat,
)
    requires
        0 <= index <= input.len(),
        fuel >= (input.len() - index) as nat,
        core_digit_run_end_spec(input, index, base, fuel) == input.len(),
    ensures
        forall|candidate: int|
            index <= candidate < input.len() ==> core_digit_for_base_spec(input[candidate], base),
{
    lemma_core_digit_run_to_target_has_only_digits(input, index, input.len() as int, base, fuel);
}

fn core_digit_run_end(input: &[u32], start: usize, base: CoreIntegerBase) -> (end: usize)
    requires
        start <= input@.len(),
    ensures
        start <= end <= input@.len(),
        end as int == core_digit_run_end_spec(
            input@,
            start as int,
            base,
            (input@.len() - start) as nat,
        ),
{
    let ghost expected = core_digit_run_end_spec(
        input@,
        start as int,
        base,
        (input@.len() - start) as nat,
    );
    let mut index = start;
    while index < input.len()
        invariant
            start <= index <= input@.len(),
            expected == core_digit_run_end_spec(
                input@,
                index as int,
                base,
                (input@.len() - index) as nat,
            ),
            expected == core_digit_run_end_spec(
                input@,
                start as int,
                base,
                (input@.len() - start) as nat,
            ),
        decreases input.len() - index,
    {
        let code_point = input[index];
        if !core_digit_for_base(code_point, base) {
            proof {
                reveal(core_digit_run_end_spec);
                assert(expected == index as int);
                assert(core_digit_run_end_spec(
                    input@,
                    start as int,
                    base,
                    (input@.len() - start) as nat,
                ) == index as int);
            }
            return index;
        }
        proof {
            reveal(core_digit_run_end_spec);
        }
        index += 1;
    }
    proof {
        reveal(core_digit_run_end_spec);
        assert(core_digit_run_end_spec(input@, start as int, base, (input@.len() - start) as nat)
            == index as int);
    }
    index
}

pub open spec fn core_null_spec(input: Seq<u32>) -> bool {
    input.len() == 0 || (input.len() == 1 && input[0] == 0x7e) || (input.len() == 4 && ((input[0]
        == 0x6e && input[1] == 0x75 && input[2] == 0x6c && input[3] == 0x6c) || (input[0] == 0x4e
        && input[1] == 0x75 && input[2] == 0x6c && input[3] == 0x6c) || (input[0] == 0x4e
        && input[1] == 0x55 && input[2] == 0x4c && input[3] == 0x4c)))
}

fn core_null(input: &[u32]) -> (matches: bool)
    ensures
        matches == core_null_spec(input@),
{
    input.is_empty() || (input.len() == 1 && input[0] == 0x7e) || (input.len() == 4 && ((input[0]
        == 0x6e && input[1] == 0x75 && input[2] == 0x6c && input[3] == 0x6c) || (input[0] == 0x4e
        && input[1] == 0x75 && input[2] == 0x6c && input[3] == 0x6c) || (input[0] == 0x4e
        && input[1] == 0x55 && input[2] == 0x4c && input[3] == 0x4c)))
}

pub open spec fn core_true_spec(input: Seq<u32>) -> bool {
    input.len() == 4 && ((input[0] == 0x74 && input[1] == 0x72 && input[2] == 0x75 && input[3]
        == 0x65) || (input[0] == 0x54 && input[1] == 0x72 && input[2] == 0x75 && input[3] == 0x65)
        || (input[0] == 0x54 && input[1] == 0x52 && input[2] == 0x55 && input[3] == 0x45))
}

fn core_true(input: &[u32]) -> (matches: bool)
    ensures
        matches == core_true_spec(input@),
{
    input.len() == 4 && ((input[0] == 0x74 && input[1] == 0x72 && input[2] == 0x75 && input[3]
        == 0x65) || (input[0] == 0x54 && input[1] == 0x72 && input[2] == 0x75 && input[3] == 0x65)
        || (input[0] == 0x54 && input[1] == 0x52 && input[2] == 0x55 && input[3] == 0x45))
}

pub open spec fn core_false_spec(input: Seq<u32>) -> bool {
    input.len() == 5 && ((input[0] == 0x66 && input[1] == 0x61 && input[2] == 0x6c && input[3]
        == 0x73 && input[4] == 0x65) || (input[0] == 0x46 && input[1] == 0x61 && input[2] == 0x6c
        && input[3] == 0x73 && input[4] == 0x65) || (input[0] == 0x46 && input[1] == 0x41
        && input[2] == 0x4c && input[3] == 0x53 && input[4] == 0x45))
}

fn core_false(input: &[u32]) -> (matches: bool)
    ensures
        matches == core_false_spec(input@),
{
    input.len() == 5 && ((input[0] == 0x66 && input[1] == 0x61 && input[2] == 0x6c && input[3]
        == 0x73 && input[4] == 0x65) || (input[0] == 0x46 && input[1] == 0x61 && input[2] == 0x6c
        && input[3] == 0x73 && input[4] == 0x65) || (input[0] == 0x46 && input[1] == 0x41
        && input[2] == 0x4c && input[3] == 0x53 && input[4] == 0x45))
}

pub open spec fn core_sign_body_start_spec(input: Seq<u32>) -> int {
    if input.len() > 0 && (input[0] == 0x2d || input[0] == 0x2b) {
        1
    } else {
        0
    }
}

fn core_sign_body_start(input: &[u32]) -> (start: usize)
    ensures
        start as int == core_sign_body_start_spec(input@),
        start <= input@.len(),
{
    if !input.is_empty() && (input[0] == 0x2d || input[0] == 0x2b) {
        1
    } else {
        0
    }
}

pub open spec fn core_decimal_integer_spec(input: Seq<u32>) -> bool {
    let start = core_sign_body_start_spec(input);
    start < input.len() && core_digit_run_end_spec(
        input,
        start,
        CoreIntegerBase::Decimal,
        (input.len() - start) as nat,
    ) == input.len()
}

fn core_decimal_integer(input: &[u32]) -> (matches: bool)
    ensures
        matches == core_decimal_integer_spec(input@),
{
    let start = core_sign_body_start(input);
    let end = core_digit_run_end(input, start, CoreIntegerBase::Decimal);
    proof {
        reveal(core_decimal_integer_spec);
    }
    start < input.len() && end == input.len()
}

pub open spec fn core_prefixed_integer_spec(input: Seq<u32>, base: CoreIntegerBase) -> bool {
    let prefix: u32 = match base {
        CoreIntegerBase::Octal => 0x6f,
        CoreIntegerBase::Hexadecimal => 0x78,
        CoreIntegerBase::Decimal => 0,
    };
    base != CoreIntegerBase::Decimal && input.len() > 2 && input[0] == 0x30 && input[1] == prefix
        && core_digit_run_end_spec(input, 2, base, (input.len() - 2) as nat) == input.len()
}

fn core_prefixed_integer(input: &[u32], base: CoreIntegerBase) -> (matches: bool)
    ensures
        matches == core_prefixed_integer_spec(input@, base),
{
    let prefix: u32 = match base {
        CoreIntegerBase::Octal => 0x6f,
        CoreIntegerBase::Hexadecimal => 0x78,
        CoreIntegerBase::Decimal => 0,
    };
    if base == CoreIntegerBase::Decimal || input.len() <= 2 || input[0] != 0x30 || input[1]
        != prefix {
        proof {
            reveal(core_prefixed_integer_spec);
        }
        return false;
    }
    let end = core_digit_run_end(input, 2, base);
    proof {
        reveal(core_prefixed_integer_spec);
    }
    end == input.len()
}

pub open spec fn core_finite_float_spec(input: Seq<u32>) -> Option<CorePlainScalarClass> {
    let start = core_sign_body_start_spec(input);
    let negative = input.len() > 0 && input[0] == 0x2d;
    let whole_end = core_digit_run_end_spec(
        input,
        start,
        CoreIntegerBase::Decimal,
        (input.len() - start) as nat,
    );
    let has_whole = whole_end > start;
    let has_dot = whole_end < input.len() && input[whole_end] == 0x2e;
    let fraction_start = if has_dot {
        whole_end + 1
    } else {
        whole_end
    };
    let fraction_end = if has_dot {
        core_digit_run_end_spec(
            input,
            fraction_start,
            CoreIntegerBase::Decimal,
            (input.len() - fraction_start) as nat,
        )
    } else {
        whole_end
    };
    let has_fraction_digit = has_dot && fraction_end > fraction_start;
    let mantissa_valid = has_whole || has_fraction_digit;
    let exponent_marker = if has_dot {
        fraction_end
    } else {
        whole_end
    };
    let has_exponent = exponent_marker < input.len() && (input[exponent_marker] == 0x65
        || input[exponent_marker] == 0x45);
    let exponent_sign = if has_exponent && exponent_marker + 1 < input.len() && (
    input[exponent_marker + 1] == 0x2d || input[exponent_marker + 1] == 0x2b) {
        exponent_marker + 2
    } else if has_exponent {
        exponent_marker + 1
    } else {
        exponent_marker
    };
    let exponent_end = if has_exponent {
        core_digit_run_end_spec(
            input,
            exponent_sign,
            CoreIntegerBase::Decimal,
            (input.len() - exponent_sign) as nat,
        )
    } else {
        exponent_marker
    };
    let exponent_valid = !has_exponent || exponent_end > exponent_sign;
    let end = if has_exponent {
        exponent_end
    } else {
        exponent_marker
    };
    if mantissa_valid && exponent_valid && end == input.len() && (has_dot || has_exponent) {
        Some(
            CorePlainScalarClass::FiniteFloat {
                negative,
                whole: CoreScalarRange { start: start as u64, end: whole_end as u64 },
                fraction: if has_dot {
                    Some(CoreScalarRange { start: fraction_start as u64, end: fraction_end as u64 })
                } else {
                    None
                },
                exponent_negative: has_exponent && exponent_marker + 1 < input.len()
                    && input[exponent_marker + 1] == 0x2d,
                exponent: if has_exponent {
                    Some(CoreScalarRange { start: exponent_sign as u64, end: exponent_end as u64 })
                } else {
                    None
                },
            },
        )
    } else {
        None
    }
}

pub proof fn lemma_core_finite_float_has_exact_digits(
    input: Seq<u32>,
    negative: bool,
    whole: CoreScalarRange,
    fraction: Option<CoreScalarRange>,
    exponent_negative: bool,
    exponent: Option<CoreScalarRange>,
)
    requires
        input.len() <= MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS,
        core_finite_float_spec(input) == Some(
            CorePlainScalarClass::FiniteFloat {
                negative,
                whole,
                fraction,
                exponent_negative,
                exponent,
            },
        ),
    ensures
        whole.start <= whole.end <= input.len(),
        forall|index: int|
            whole.start <= index < whole.end ==> core_digit_for_base_spec(
                input[index],
                CoreIntegerBase::Decimal,
            ),
        match fraction {
            Some(range) => {
                range.start <= range.end <= input.len() && forall|index: int|
                    range.start <= index < range.end ==> core_digit_for_base_spec(
                        input[index],
                        CoreIntegerBase::Decimal,
                    )
            },
            None => true,
        },
        match exponent {
            Some(range) => {
                range.start < range.end <= input.len() && forall|index: int|
                    range.start <= index < range.end ==> core_digit_for_base_spec(
                        input[index],
                        CoreIntegerBase::Decimal,
                    )
            },
            None => true,
        },
        whole.start < whole.end || match fraction {
            Some(range) => range.start < range.end,
            None => false,
        },
{
    let start = core_sign_body_start_spec(input);
    let whole_end = core_digit_run_end_spec(
        input,
        start,
        CoreIntegerBase::Decimal,
        (input.len() - start) as nat,
    );
    let has_dot = whole_end < input.len() && input[whole_end] == 0x2e;
    let fraction_start = if has_dot {
        whole_end + 1
    } else {
        whole_end
    };
    let fraction_end = if has_dot {
        core_digit_run_end_spec(
            input,
            fraction_start,
            CoreIntegerBase::Decimal,
            (input.len() - fraction_start) as nat,
        )
    } else {
        whole_end
    };
    let exponent_marker = if has_dot {
        fraction_end
    } else {
        whole_end
    };
    let has_exponent = exponent_marker < input.len() && (input[exponent_marker] == 0x65
        || input[exponent_marker] == 0x45);
    let exponent_start = if has_exponent && exponent_marker + 1 < input.len() && (
    input[exponent_marker + 1] == 0x2d || input[exponent_marker + 1] == 0x2b) {
        exponent_marker + 2
    } else if has_exponent {
        exponent_marker + 1
    } else {
        exponent_marker
    };
    reveal(core_finite_float_spec);
    assert(0 <= start <= whole_end <= input.len());
    assert(whole.start == start as u64);
    assert(whole.end == whole_end as u64);
    lemma_core_digit_run_to_target_has_only_digits(
        input,
        start,
        whole_end,
        CoreIntegerBase::Decimal,
        (input.len() - start) as nat,
    );
    assert forall|index: int| whole.start <= index < whole.end implies core_digit_for_base_spec(
        input[index],
        CoreIntegerBase::Decimal,
    ) by {
        assert(start <= index < whole_end);
    }
    if has_dot {
        let range = fraction.unwrap();
        assert(fraction == Some(range));
        assert(range.start == fraction_start as u64);
        assert(range.end == fraction_end as u64);
        assert(0 <= fraction_start <= fraction_end <= input.len());
        assert(core_digit_run_end_spec(
            input,
            fraction_start,
            CoreIntegerBase::Decimal,
            (input.len() - fraction_start) as nat,
        ) == fraction_end);
        lemma_core_digit_run_to_target_has_only_digits(
            input,
            fraction_start,
            fraction_end,
            CoreIntegerBase::Decimal,
            (input.len() - fraction_start) as nat,
        );
        assert forall|index: int| range.start <= index < range.end implies core_digit_for_base_spec(
            input[index],
            CoreIntegerBase::Decimal,
        ) by {
            assert(fraction_start <= index < fraction_end);
        }
    }
    if has_exponent {
        let range = exponent.unwrap();
        let exponent_end = input.len() as int;
        assert(exponent == Some(range));
        assert(range.start == exponent_start as u64);
        assert(range.end == exponent_end as u64);
        assert(0 <= exponent_start < exponent_end <= input.len());
        assert(core_digit_run_end_spec(
            input,
            exponent_start,
            CoreIntegerBase::Decimal,
            (input.len() - exponent_start) as nat,
        ) == exponent_end);
        lemma_core_digit_run_to_target_has_only_digits(
            input,
            exponent_start,
            exponent_end,
            CoreIntegerBase::Decimal,
            (input.len() - exponent_start) as nat,
        );
        assert forall|index: int| range.start <= index < range.end implies core_digit_for_base_spec(
            input[index],
            CoreIntegerBase::Decimal,
        ) by {
            assert(exponent_start <= index < exponent_end);
        }
    }
}

fn core_finite_float(input: &[u32]) -> (class: Option<CorePlainScalarClass>)
    ensures
        class == core_finite_float_spec(input@),
{
    let start = core_sign_body_start(input);
    let negative = !input.is_empty() && input[0] == 0x2d;
    let whole_end = core_digit_run_end(input, start, CoreIntegerBase::Decimal);
    let has_whole = whole_end > start;
    let has_dot = whole_end < input.len() && input[whole_end] == 0x2e;
    let fraction_start = if has_dot {
        whole_end + 1
    } else {
        whole_end
    };
    let fraction_end = if has_dot {
        core_digit_run_end(input, fraction_start, CoreIntegerBase::Decimal)
    } else {
        whole_end
    };
    let has_fraction_digit = has_dot && fraction_end > fraction_start;
    let mantissa_valid = has_whole || has_fraction_digit;
    let exponent_marker = if has_dot {
        fraction_end
    } else {
        whole_end
    };
    let has_exponent = exponent_marker < input.len() && (input[exponent_marker] == 0x65
        || input[exponent_marker] == 0x45);
    let exponent_sign = if has_exponent && exponent_marker + 1 < input.len() && (
    input[exponent_marker + 1] == 0x2d || input[exponent_marker + 1] == 0x2b) {
        exponent_marker + 2
    } else if has_exponent {
        exponent_marker + 1
    } else {
        exponent_marker
    };
    let exponent_end = if has_exponent {
        core_digit_run_end(input, exponent_sign, CoreIntegerBase::Decimal)
    } else {
        exponent_marker
    };
    let exponent_valid = !has_exponent || exponent_end > exponent_sign;
    let end = if has_exponent {
        exponent_end
    } else {
        exponent_marker
    };
    proof {
        reveal(core_finite_float_spec);
    }
    if mantissa_valid && exponent_valid && end == input.len() && (has_dot || has_exponent) {
        Some(
            CorePlainScalarClass::FiniteFloat {
                negative,
                whole: CoreScalarRange::new(start as u64, whole_end as u64),
                fraction: if has_dot {
                    Some(CoreScalarRange::new(fraction_start as u64, fraction_end as u64))
                } else {
                    None
                },
                exponent_negative: has_exponent && exponent_marker + 1 < input.len()
                    && input[exponent_marker + 1] == 0x2d,
                exponent: if has_exponent {
                    Some(CoreScalarRange::new(exponent_sign as u64, exponent_end as u64))
                } else {
                    None
                },
            },
        )
    } else {
        None
    }
}

pub open spec fn core_infinity_spec(input: Seq<u32>) -> Option<bool> {
    let start = core_sign_body_start_spec(input);
    if input.len() == start + 4 && input[start] == 0x2e && ((input[start + 1] == 0x69 && input[start
        + 2] == 0x6e && input[start + 3] == 0x66) || (input[start + 1] == 0x49 && input[start + 2]
        == 0x6e && input[start + 3] == 0x66) || (input[start + 1] == 0x49 && input[start + 2]
        == 0x4e && input[start + 3] == 0x46)) {
        Some(start == 1 && input[0] == 0x2d)
    } else {
        None
    }
}

fn core_infinity(input: &[u32]) -> (negative: Option<bool>)
    ensures
        negative == core_infinity_spec(input@),
{
    let start = core_sign_body_start(input);
    if input.len() == start + 4 && input[start] == 0x2e && ((input[start + 1] == 0x69 && input[start
        + 2] == 0x6e && input[start + 3] == 0x66) || (input[start + 1] == 0x49 && input[start + 2]
        == 0x6e && input[start + 3] == 0x66) || (input[start + 1] == 0x49 && input[start + 2]
        == 0x4e && input[start + 3] == 0x46)) {
        Some(start == 1 && input[0] == 0x2d)
    } else {
        None
    }
}

pub open spec fn core_nan_spec(input: Seq<u32>) -> bool {
    input.len() == 4 && input[0] == 0x2e && ((input[1] == 0x6e && input[2] == 0x61 && input[3]
        == 0x6e) || (input[1] == 0x4e && input[2] == 0x61 && input[3] == 0x4e) || (input[1] == 0x4e
        && input[2] == 0x41 && input[3] == 0x4e))
}

fn core_nan(input: &[u32]) -> (matches: bool)
    ensures
        matches == core_nan_spec(input@),
{
    input.len() == 4 && input[0] == 0x2e && ((input[1] == 0x6e && input[2] == 0x61 && input[3]
        == 0x6e) || (input[1] == 0x4e && input[2] == 0x61 && input[3] == 0x4e) || (input[1] == 0x4e
        && input[2] == 0x41 && input[3] == 0x4e))
}

pub open spec fn classify_core_plain_scalar_unbounded_spec(
    input: Seq<u32>,
) -> CorePlainScalarClass {
    if core_null_spec(input) {
        CorePlainScalarClass::Null
    } else if core_true_spec(input) {
        CorePlainScalarClass::Boolean(true)
    } else if core_false_spec(input) {
        CorePlainScalarClass::Boolean(false)
    } else if core_decimal_integer_spec(input) {
        let start = core_sign_body_start_spec(input);
        CorePlainScalarClass::Integer {
            negative: start == 1 && input[0] == 0x2d,
            base: CoreIntegerBase::Decimal,
            digits: CoreScalarRange { start: start as u64, end: input.len() as u64 },
        }
    } else if core_prefixed_integer_spec(input, CoreIntegerBase::Octal) {
        CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Octal,
            digits: CoreScalarRange { start: 2, end: input.len() as u64 },
        }
    } else if core_prefixed_integer_spec(input, CoreIntegerBase::Hexadecimal) {
        CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Hexadecimal,
            digits: CoreScalarRange { start: 2, end: input.len() as u64 },
        }
    } else {
        match core_finite_float_spec(input) {
            Some(class) => class,
            None => match core_infinity_spec(input) {
                Some(negative) => CorePlainScalarClass::Infinity { negative },
                None => if core_nan_spec(input) {
                    CorePlainScalarClass::NotANumber
                } else {
                    CorePlainScalarClass::String
                },
            },
        }
    }
}

pub open spec fn classify_core_plain_scalar_spec(
    input: Seq<u32>,
    limits: CoreScalarLimitsView,
) -> Result<CorePlainScalarClass, CoreScalarErrorView> {
    let effective_limit = effective_core_scalar_limit_spec(limits);
    if input.len() > effective_limit {
        Err(
            CoreScalarErrorView {
                kind: CoreScalarErrorKind::InputLimitExceeded,
                code_point_index: effective_limit,
            },
        )
    } else {
        Ok(classify_core_plain_scalar_unbounded_spec(input))
    }
}

pub proof fn lemma_classified_core_integer_has_exact_digits(
    input: Seq<u32>,
    limits: CoreScalarLimitsView,
    negative: bool,
    base: CoreIntegerBase,
    digits: CoreScalarRange,
)
    requires
        classify_core_plain_scalar_spec(input, limits) == Ok(
            CorePlainScalarClass::Integer { negative, base, digits },
        ),
    ensures
        digits.start < digits.end <= input.len(),
        forall|index: int|
            digits.start <= index < digits.end ==> core_digit_for_base_spec(input[index], base),
{
    reveal(classify_core_plain_scalar_spec);
    reveal(effective_core_scalar_limit_spec);
    assert(input.len() <= MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS);
    reveal(classify_core_plain_scalar_unbounded_spec);
    assert(!core_null_spec(input));
    assert(!core_true_spec(input));
    assert(!core_false_spec(input));
    assert(core_decimal_integer_spec(input) || core_prefixed_integer_spec(
        input,
        CoreIntegerBase::Octal,
    ) || core_prefixed_integer_spec(input, CoreIntegerBase::Hexadecimal));
    if core_decimal_integer_spec(input) {
        let start = core_sign_body_start_spec(input);
        assert(base == CoreIntegerBase::Decimal);
        assert(digits.start == start as u64);
        assert(digits.end == input.len() as u64);
        reveal(core_decimal_integer_spec);
        lemma_core_digit_run_to_end_has_only_digits(
            input,
            start,
            CoreIntegerBase::Decimal,
            (input.len() - start) as nat,
        );
    } else if core_prefixed_integer_spec(input, CoreIntegerBase::Octal) {
        assert(base == CoreIntegerBase::Octal);
        assert(digits.start == 2);
        assert(digits.end == input.len() as u64);
        reveal(core_prefixed_integer_spec);
        lemma_core_digit_run_to_end_has_only_digits(
            input,
            2,
            CoreIntegerBase::Octal,
            (input.len() - 2) as nat,
        );
    } else {
        assert(core_prefixed_integer_spec(input, CoreIntegerBase::Hexadecimal));
        assert(base == CoreIntegerBase::Hexadecimal);
        assert(digits.start == 2);
        assert(digits.end == input.len() as u64);
        reveal(core_prefixed_integer_spec);
        lemma_core_digit_run_to_end_has_only_digits(
            input,
            2,
            CoreIntegerBase::Hexadecimal,
            (input.len() - 2) as nat,
        );
    }
}

pub proof fn lemma_classified_core_finite_float_has_exact_digits(
    input: Seq<u32>,
    limits: CoreScalarLimitsView,
    negative: bool,
    whole: CoreScalarRange,
    fraction: Option<CoreScalarRange>,
    exponent_negative: bool,
    exponent: Option<CoreScalarRange>,
)
    requires
        classify_core_plain_scalar_spec(input, limits) == Ok(
            CorePlainScalarClass::FiniteFloat {
                negative,
                whole,
                fraction,
                exponent_negative,
                exponent,
            },
        ),
    ensures
        whole.start <= whole.end <= input.len(),
        forall|index: int|
            whole.start <= index < whole.end ==> core_digit_for_base_spec(
                input[index],
                CoreIntegerBase::Decimal,
            ),
        match fraction {
            Some(range) => {
                range.start <= range.end <= input.len() && forall|index: int|
                    range.start <= index < range.end ==> core_digit_for_base_spec(
                        input[index],
                        CoreIntegerBase::Decimal,
                    )
            },
            None => true,
        },
        match exponent {
            Some(range) => {
                range.start < range.end <= input.len() && forall|index: int|
                    range.start <= index < range.end ==> core_digit_for_base_spec(
                        input[index],
                        CoreIntegerBase::Decimal,
                    )
            },
            None => true,
        },
        whole.start < whole.end || match fraction {
            Some(range) => range.start < range.end,
            None => false,
        },
{
    reveal(classify_core_plain_scalar_spec);
    reveal(classify_core_plain_scalar_unbounded_spec);
    assert(core_finite_float_spec(input) == Some(
        CorePlainScalarClass::FiniteFloat {
            negative,
            whole,
            fraction,
            exponent_negative,
            exponent,
        },
    ));
    lemma_core_finite_float_has_exact_digits(
        input,
        negative,
        whole,
        fraction,
        exponent_negative,
        exponent,
    );
}

fn classify_core_plain_scalar_unbounded(input: &[u32]) -> (class: CorePlainScalarClass)
    ensures
        class == classify_core_plain_scalar_unbounded_spec(input@),
{
    if core_null(input) {
        return CorePlainScalarClass::Null;
    }
    if core_true(input) {
        return CorePlainScalarClass::Boolean(true);
    }
    if core_false(input) {
        return CorePlainScalarClass::Boolean(false);
    }
    if core_decimal_integer(input) {
        let start = core_sign_body_start(input);
        proof {
            reveal(classify_core_plain_scalar_unbounded_spec);
        }
        return CorePlainScalarClass::Integer {
            negative: start == 1 && input[0] == 0x2d,
            base: CoreIntegerBase::Decimal,
            digits: CoreScalarRange::new(start as u64, input.len() as u64),
        };
    }
    if core_prefixed_integer(input, CoreIntegerBase::Octal) {
        proof {
            reveal(classify_core_plain_scalar_unbounded_spec);
        }
        return CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Octal,
            digits: CoreScalarRange::new(2, input.len() as u64),
        };
    }
    if core_prefixed_integer(input, CoreIntegerBase::Hexadecimal) {
        proof {
            reveal(classify_core_plain_scalar_unbounded_spec);
        }
        return CorePlainScalarClass::Integer {
            negative: false,
            base: CoreIntegerBase::Hexadecimal,
            digits: CoreScalarRange::new(2, input.len() as u64),
        };
    }
    if let Some(class) = core_finite_float(input) {
        proof {
            reveal(classify_core_plain_scalar_unbounded_spec);
        }
        return class;
    }
    if let Some(negative) = core_infinity(input) {
        proof {
            reveal(classify_core_plain_scalar_unbounded_spec);
        }
        return CorePlainScalarClass::Infinity { negative };
    }
    let is_nan = core_nan(input);
    proof {
        reveal(classify_core_plain_scalar_unbounded_spec);
    }
    if is_nan {
        CorePlainScalarClass::NotANumber
    } else {
        CorePlainScalarClass::String
    }
}

/// Classify decoded plain-scalar content under the YAML 1.2.2 Core Schema.
///
/// The function does not perform arbitrary-width numeric conversion.  It returns exact half-open
/// digit subranges for that next verified transformation and never applies YAML 1.1 boolean rules.
pub fn classify_core_plain_scalar(input: &[u32], limits: CoreScalarLimits) -> (result: Result<
    CorePlainScalarClass,
    CoreScalarError,
>)
    ensures
        classify_core_plain_scalar_spec(input@, limits@) == match result {
            Ok(class) => Ok(class),
            Err(error) => Err(error@),
        },
{
    let effective_limit = if limits.max_code_points < MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS {
        limits.max_code_points
    } else {
        MAX_PROFILE1_RESOLVED_SCALAR_CODE_POINTS
    };
    if input.len() as u64 > effective_limit {
        let error = CoreScalarError::at(CoreScalarErrorKind::InputLimitExceeded, effective_limit);
        proof {
            reveal(classify_core_plain_scalar_spec);
            reveal(effective_core_scalar_limit_spec);
        }
        return Err(error);
    }
    let class = classify_core_plain_scalar_unbounded(input);
    proof {
        reveal(classify_core_plain_scalar_spec);
        reveal(effective_core_scalar_limit_spec);
    }
    Ok(class)
}

} // verus!
