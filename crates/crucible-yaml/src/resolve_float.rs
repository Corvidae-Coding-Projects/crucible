//! Verified exact conversion of YAML 1.2.2 Core finite decimals.
//!
//! Coefficients and signed exponents use canonical little-endian decimal digits.  This preserves
//! arbitrary precision, admits linear-memory normalization, and never passes through IEEE-754.
use crate::resolve::{
    classify_core_plain_scalar, CorePlainScalarClass, CoreScalarErrorKind, CoreScalarLimits,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve::{CoreScalarLimitsView, CoreScalarRange};
use vstd::prelude::*;

verus! {

pub const CORE_FINITE_FLOAT_CONVERSION_VERSION: u16 = 1;

pub const MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS: u64 = 1_048_576;

pub const MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreFiniteFloatLimits {
    max_code_points: u64,
    max_coefficient_digits: u64,
    max_exponent_digits: u64,
}

#[verifier::ext_equal]
pub struct CoreFiniteFloatLimitsView {
    pub max_code_points: u64,
    pub max_coefficient_digits: u64,
    pub max_exponent_digits: u64,
}

impl View for CoreFiniteFloatLimits {
    type V = CoreFiniteFloatLimitsView;

    closed spec fn view(&self) -> CoreFiniteFloatLimitsView {
        CoreFiniteFloatLimitsView {
            max_code_points: self.max_code_points,
            max_coefficient_digits: self.max_coefficient_digits,
            max_exponent_digits: self.max_exponent_digits,
        }
    }
}

impl CoreFiniteFloatLimits {
    pub fn new(
        max_code_points: u64,
        max_coefficient_digits: u64,
        max_exponent_digits: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (CoreFiniteFloatLimitsView {
                max_code_points,
                max_coefficient_digits,
                max_exponent_digits,
            }),
    {
        Self { max_code_points, max_coefficient_digits, max_exponent_digits }
    }

    pub fn max_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_code_points,
    {
        self.max_code_points
    }

    pub fn max_coefficient_digits(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_coefficient_digits,
    {
        self.max_coefficient_digits
    }

    pub fn max_exponent_digits(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_exponent_digits,
    {
        self.max_exponent_digits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CoreFiniteFloatErrorKind {
    InputLimitExceeded,
    NotFiniteFloat,
    CoefficientLimitExceeded,
    ExponentLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreFiniteFloatError {
    kind: CoreFiniteFloatErrorKind,
    code_point_index: u64,
}

#[verifier::ext_equal]
pub struct CoreFiniteFloatErrorView {
    pub kind: CoreFiniteFloatErrorKind,
    pub code_point_index: u64,
}

impl View for CoreFiniteFloatError {
    type V = CoreFiniteFloatErrorView;

    closed spec fn view(&self) -> CoreFiniteFloatErrorView {
        CoreFiniteFloatErrorView { kind: self.kind, code_point_index: self.code_point_index }
    }
}

impl CoreFiniteFloatError {
    fn at(kind: CoreFiniteFloatErrorKind, code_point_index: u64) -> (error: Self)
        ensures
            error@ == (CoreFiniteFloatErrorView { kind, code_point_index }),
    {
        Self { kind, code_point_index }
    }

    pub fn kind(&self) -> (kind: CoreFiniteFloatErrorKind)
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

#[derive(Debug, PartialEq, Eq)]
pub struct CoreFiniteFloat {
    negative: bool,
    coefficient_digits_le: Vec<u8>,
    exponent_negative: bool,
    exponent_digits_le: Vec<u8>,
}

#[verifier::ext_equal]
pub struct CoreFiniteFloatView {
    pub negative: bool,
    pub coefficient_digits_le: Seq<u8>,
    pub exponent_negative: bool,
    pub exponent_digits_le: Seq<u8>,
}

impl View for CoreFiniteFloat {
    type V = CoreFiniteFloatView;

    closed spec fn view(&self) -> CoreFiniteFloatView {
        CoreFiniteFloatView {
            negative: self.negative,
            coefficient_digits_le: self.coefficient_digits_le@,
            exponent_negative: self.exponent_negative,
            exponent_digits_le: self.exponent_digits_le@,
        }
    }
}

impl CoreFiniteFloat {
    fn new(
        negative: bool,
        coefficient_digits_le: Vec<u8>,
        exponent_negative: bool,
        exponent_digits_le: Vec<u8>,
    ) -> (value: Self)
        ensures
            value@ == (CoreFiniteFloatView {
                negative,
                coefficient_digits_le: coefficient_digits_le@,
                exponent_negative,
                exponent_digits_le: exponent_digits_le@,
            }),
    {
        Self { negative, coefficient_digits_le, exponent_negative, exponent_digits_le }
    }

    pub fn negative(&self) -> (negative: bool)
        ensures
            negative == self@.negative,
    {
        self.negative
    }

    pub fn coefficient_digits_le(&self) -> (digits: &[u8])
        ensures
            digits@ == self@.coefficient_digits_le,
    {
        self.coefficient_digits_le.as_slice()
    }

    pub fn exponent_negative(&self) -> (negative: bool)
        ensures
            negative == self@.exponent_negative,
    {
        self.exponent_negative
    }

    pub fn exponent_digits_le(&self) -> (digits: &[u8])
        ensures
            digits@ == self@.exponent_digits_le,
    {
        self.exponent_digits_le.as_slice()
    }
}

pub open spec fn decimal_digits_bounded_spec(digits: Seq<u8>) -> bool {
    forall|index: int| 0 <= index < digits.len() ==> #[trigger] digits[index] < 10
}

pub open spec fn canonical_decimal_digits_le_spec(digits: Seq<u8>) -> bool {
    digits.len() > 0 && decimal_digits_bounded_spec(digits) && (digits.len() == 1
        || digits[digits.len() - 1] != 0)
}

pub open spec fn decimal_digits_zero_spec(digits: Seq<u8>) -> bool {
    digits.len() == 1 && digits[0] == 0
}

proof fn lemma_decimal_digit_bounded(digits: Seq<u8>, index: int)
    requires
        decimal_digits_bounded_spec(digits),
        0 <= index < digits.len(),
    ensures
        digits[index] < 10,
{
    reveal(decimal_digits_bounded_spec);
}

proof fn lemma_decimal_digits_bounded_push(digits: Seq<u8>, digit: u8)
    requires
        decimal_digits_bounded_spec(digits),
        digit < 10,
    ensures
        decimal_digits_bounded_spec(digits.push(digit)),
{
    reveal(decimal_digits_bounded_spec);
    assert forall|index: int| 0 <= index < digits.push(digit).len() implies #[trigger] digits.push(
        digit,
    )[index] < 10 by {
        if index < digits.len() {
            assert(digits.push(digit)[index] == digits[index]);
        } else {
            assert(index == digits.len());
            assert(digits.push(digit)[index] == digit);
        }
    }
}

pub open spec fn effective_float_coefficient_limit_spec(limits: CoreFiniteFloatLimitsView) -> u64 {
    if limits.max_coefficient_digits < MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS {
        limits.max_coefficient_digits
    } else {
        MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS
    }
}

pub open spec fn effective_float_exponent_limit_spec(limits: CoreFiniteFloatLimitsView) -> u64 {
    if limits.max_exponent_digits < MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS {
        limits.max_exponent_digits
    } else {
        MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS
    }
}

pub open spec fn decimal_range_digits_spec(input: Seq<u32>, range: CoreScalarRange) -> Seq<u8> {
    Seq::new(
        (range.end - range.start) as nat,
        |index: int| (input[range.start + index] - 0x30) as u8,
    )
}

pub open spec fn coefficient_digits_be_spec(
    input: Seq<u32>,
    whole: CoreScalarRange,
    fraction: Option<CoreScalarRange>,
) -> Seq<u8> {
    decimal_range_digits_spec(input, whole) + match fraction {
        Some(range) => decimal_range_digits_spec(input, range),
        None => Seq::empty(),
    }
}

pub open spec fn trim_trailing_decimal_zeros_spec(digits: Seq<u8>, fuel: nat) -> Seq<u8>
    decreases fuel,
{
    if fuel > 0 && digits.len() > 1 && digits[digits.len() - 1] == 0 {
        trim_trailing_decimal_zeros_spec(digits.drop_last(), (fuel - 1) as nat)
    } else {
        digits
    }
}

pub open spec fn trim_leading_decimal_zeros_spec(digits: Seq<u8>, fuel: nat) -> Seq<u8>
    decreases fuel,
{
    if fuel > 0 && digits.len() > 1 && digits[0] == 0 {
        trim_leading_decimal_zeros_spec(digits.drop_first(), (fuel - 1) as nat)
    } else {
        digits
    }
}

pub open spec fn reverse_digits_spec(digits: Seq<u8>) -> Seq<u8> {
    Seq::new(digits.len(), |index: int| digits[digits.len() - index - 1])
}

pub open spec fn canonicalize_coefficient_spec(digits_be: Seq<u8>) -> (Seq<u8>, u64, u64) {
    let without_trailing = trim_trailing_decimal_zeros_spec(digits_be, digits_be.len() as nat);
    let without_leading = trim_leading_decimal_zeros_spec(
        without_trailing,
        without_trailing.len() as nat,
    );
    (
        reverse_digits_spec(without_leading),
        (digits_be.len() - without_trailing.len()) as u64,
        (without_trailing.len() - without_leading.len()) as u64,
    )
}

pub open spec fn canonicalize_unsigned_decimal_spec(digits_be: Seq<u8>) -> Seq<u8> {
    reverse_digits_spec(trim_leading_decimal_zeros_spec(digits_be, digits_be.len() as nat))
}

pub open spec fn coefficient_source_index_spec(
    whole: CoreScalarRange,
    fraction: Option<CoreScalarRange>,
    coefficient_index: u64,
) -> u64 {
    let whole_len = (whole.end - whole.start) as u64;
    if coefficient_index < whole_len {
        (whole.start + coefficient_index) as u64
    } else {
        match fraction {
            Some(range) => (range.start + coefficient_index - whole_len) as u64,
            None => whole.end,
        }
    }
}

pub open spec fn small_decimal_digits_le_spec(value: u64) -> Seq<u8>
    decreases value,
{
    if value < 10 {
        Seq::empty().push(value as u8)
    } else {
        Seq::empty().push((value % 10) as u8) + small_decimal_digits_le_spec(value / 10)
    }
}

pub open spec fn compare_decimal_magnitude_le_spec(
    left: Seq<u8>,
    right: Seq<u8>,
    high: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if left.len() < right.len() {
        -1
    } else if left.len() > right.len() {
        1
    } else if high < 0 || fuel == 0 {
        0
    } else if left[high] < right[high] {
        -1
    } else if left[high] > right[high] {
        1
    } else {
        compare_decimal_magnitude_le_spec(left, right, high - 1, (fuel - 1) as nat)
    }
}

proof fn lemma_compare_decimal_magnitude_reverse_at(
    left: Seq<u8>,
    right: Seq<u8>,
    high: int,
    fuel: nat,
)
    requires
        left.len() == right.len(),
        high < left.len(),
    ensures
        compare_decimal_magnitude_le_spec(left, right, high, fuel)
            == -compare_decimal_magnitude_le_spec(right, left, high, fuel),
    decreases fuel,
{
    reveal(compare_decimal_magnitude_le_spec);
    if high >= 0 && fuel > 0 && left[high] == right[high] {
        lemma_compare_decimal_magnitude_reverse_at(left, right, high - 1, (fuel - 1) as nat);
    }
}

proof fn lemma_compare_decimal_magnitude_reverse(left: Seq<u8>, right: Seq<u8>)
    ensures
        compare_decimal_magnitude_le_spec(left, right, left.len() as int - 1, left.len() as nat)
            == -compare_decimal_magnitude_le_spec(
            right,
            left,
            right.len() as int - 1,
            right.len() as nat,
        ),
{
    if left.len() == right.len() {
        lemma_compare_decimal_magnitude_reverse_at(
            left,
            right,
            left.len() as int - 1,
            left.len() as nat,
        );
    } else {
        reveal(compare_decimal_magnitude_le_spec);
    }
}

pub open spec fn add_decimal_magnitudes_le_spec(
    left: Seq<u8>,
    right: Seq<u8>,
    index: int,
    carry: u8,
    fuel: nat,
) -> Seq<u8>
    decreases fuel,
{
    if fuel == 0 || (index >= left.len() && index >= right.len()) {
        if carry == 0 {
            Seq::empty()
        } else {
            Seq::empty().push(carry)
        }
    } else {
        let left_digit = if index < left.len() {
            left[index]
        } else {
            0
        };
        let right_digit = if index < right.len() {
            right[index]
        } else {
            0
        };
        let expanded = left_digit as int + right_digit as int + carry as int;
        Seq::empty().push((expanded % 10) as u8) + add_decimal_magnitudes_le_spec(
            left,
            right,
            index + 1,
            (expanded / 10) as u8,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn subtract_decimal_magnitudes_le_spec(
    larger: Seq<u8>,
    smaller: Seq<u8>,
    index: int,
    borrow: u8,
    fuel: nat,
) -> (Seq<u8>, u8)
    decreases fuel,
{
    if fuel == 0 || index >= larger.len() {
        (Seq::empty(), borrow)
    } else {
        let subtrahend = ((if index < smaller.len() {
            smaller[index]
        } else {
            0
        }) + borrow) as u8;
        let digit: u8 = if larger[index] >= subtrahend {
            (larger[index] - subtrahend) as u8
        } else {
            (10 + larger[index] - subtrahend) as u8
        };
        let next_borrow: u8 = if larger[index] >= subtrahend {
            0
        } else {
            1
        };
        let tail = subtract_decimal_magnitudes_le_spec(
            larger,
            smaller,
            index + 1,
            next_borrow,
            (fuel - 1) as nat,
        );
        (Seq::empty().push(digit) + tail.0, tail.1)
    }
}

pub open spec fn trim_high_decimal_zeros_le_spec(digits: Seq<u8>, fuel: nat) -> Seq<u8> {
    trim_trailing_decimal_zeros_spec(digits, fuel)
}

pub open spec fn apply_signed_small_decimal_raw_spec(
    negative: bool,
    digits: Seq<u8>,
    delta_negative: bool,
    delta: u64,
) -> (bool, Seq<u8>) {
    if delta == 0 {
        (
            if decimal_digits_zero_spec(digits) {
                false
            } else {
                negative
            },
            digits,
        )
    } else {
        let delta_digits = small_decimal_digits_le_spec(delta);
        if decimal_digits_zero_spec(digits) {
            (delta_negative, delta_digits)
        } else if negative == delta_negative {
            (
                negative,
                add_decimal_magnitudes_le_spec(
                    digits,
                    delta_digits,
                    0,
                    0,
                    (digits.len() + delta_digits.len() + 1) as nat,
                ),
            )
        } else {
            let comparison = compare_decimal_magnitude_le_spec(
                digits,
                delta_digits,
                digits.len() as int - 1,
                digits.len() as nat,
            );
            if comparison == 0 {
                (false, Seq::empty().push(0u8))
            } else if comparison > 0 {
                let difference = subtract_decimal_magnitudes_le_spec(
                    digits,
                    delta_digits,
                    0,
                    0,
                    digits.len() as nat,
                ).0;
                (negative, trim_high_decimal_zeros_le_spec(difference, difference.len() as nat))
            } else {
                let difference = subtract_decimal_magnitudes_le_spec(
                    delta_digits,
                    digits,
                    0,
                    0,
                    delta_digits.len() as nat,
                ).0;
                (
                    delta_negative,
                    trim_high_decimal_zeros_le_spec(difference, difference.len() as nat),
                )
            }
        }
    }
}

pub open spec fn apply_signed_small_decimal_spec(
    negative: bool,
    digits: Seq<u8>,
    delta_negative: bool,
    delta: u64,
) -> (bool, Seq<u8>) {
    let candidate = apply_signed_small_decimal_raw_spec(negative, digits, delta_negative, delta);
    (
        if decimal_digits_zero_spec(candidate.1) {
            false
        } else {
            candidate.0
        },
        candidate.1,
    )
}

pub open spec fn exponent_adjustment_anchor_spec(
    whole: CoreScalarRange,
    fraction: Option<CoreScalarRange>,
    exponent: Option<CoreScalarRange>,
    coefficient_len: u64,
) -> u64 {
    match exponent {
        Some(range) => range.start,
        None => match fraction {
            Some(range) => if range.end > range.start {
                (range.end - 1) as u64
            } else {
                coefficient_source_index_spec(whole, fraction, (coefficient_len - 1) as u64)
            },
            None => coefficient_source_index_spec(whole, fraction, (coefficient_len - 1) as u64),
        },
    }
}

pub open spec fn convert_core_finite_float_spec(
    input: Seq<u32>,
    limits: CoreFiniteFloatLimitsView,
) -> Result<CoreFiniteFloatView, CoreFiniteFloatErrorView> {
    let scalar_limits = CoreScalarLimitsView { max_code_points: limits.max_code_points };
    match crate::resolve::classify_core_plain_scalar_spec(input, scalar_limits) {
        Err(error) => Err(
            CoreFiniteFloatErrorView {
                kind: match error.kind {
                    CoreScalarErrorKind::InputLimitExceeded => CoreFiniteFloatErrorKind::InputLimitExceeded,
                },
                code_point_index: error.code_point_index,
            },
        ),
        Ok(class) => match class {
            CorePlainScalarClass::FiniteFloat {
                negative,
                whole,
                fraction,
                exponent_negative,
                exponent,
            } => {
                let coefficient_be = coefficient_digits_be_spec(input, whole, fraction);
                let prepared = canonicalize_coefficient_spec(coefficient_be);
                let coefficient_digits_le = prepared.0;
                let trailing_zeros: u64 = prepared.1;
                let leading_zeros: u64 = prepared.2;
                let coefficient_limit = effective_float_coefficient_limit_spec(limits);
                if coefficient_digits_le.len() > coefficient_limit {
                    Err(
                        CoreFiniteFloatErrorView {
                            kind: CoreFiniteFloatErrorKind::CoefficientLimitExceeded,
                            code_point_index: coefficient_source_index_spec(
                                whole,
                                fraction,
                                (leading_zeros + coefficient_limit) as u64,
                            ),
                        },
                    )
                } else {
                    let fraction_len: u64 = match fraction {
                        Some(range) => (range.end - range.start) as u64,
                        None => 0,
                    };
                    let exponent_be = match exponent {
                        Some(range) => decimal_range_digits_spec(input, range),
                        None => Seq::empty().push(0u8),
                    };
                    let raw_exponent_digits_le = canonicalize_unsigned_decimal_spec(exponent_be);
                    let normalized_exponent = if decimal_digits_zero_spec(coefficient_digits_le) {
                        (false, Seq::empty().push(0u8))
                    } else {
                        apply_signed_small_decimal_spec(
                            exponent_negative,
                            raw_exponent_digits_le,
                            fraction_len > trailing_zeros,
                            if fraction_len > trailing_zeros {
                                (fraction_len - trailing_zeros) as u64
                            } else {
                                (trailing_zeros - fraction_len) as u64
                            },
                        )
                    };
                    let exponent_limit = effective_float_exponent_limit_spec(limits);
                    if normalized_exponent.1.len() > exponent_limit {
                        let raw_leading: u64 = (exponent_be.len()
                            - raw_exponent_digits_le.len()) as u64;
                        let code_point_index = match exponent {
                            Some(range) => if raw_exponent_digits_le.len() > exponent_limit {
                                (range.start + raw_leading + exponent_limit) as u64
                            } else {
                                exponent_adjustment_anchor_spec(
                                    whole,
                                    fraction,
                                    exponent,
                                    coefficient_be.len() as u64,
                                )
                            },
                            None => exponent_adjustment_anchor_spec(
                                whole,
                                fraction,
                                exponent,
                                coefficient_be.len() as u64,
                            ),
                        };
                        Err(
                            CoreFiniteFloatErrorView {
                                kind: CoreFiniteFloatErrorKind::ExponentLimitExceeded,
                                code_point_index,
                            },
                        )
                    } else {
                        Ok(
                            CoreFiniteFloatView {
                                negative,
                                coefficient_digits_le,
                                exponent_negative: normalized_exponent.0,
                                exponent_digits_le: normalized_exponent.1,
                            },
                        )
                    }
                }
            },
            _ => Err(
                CoreFiniteFloatErrorView {
                    kind: CoreFiniteFloatErrorKind::NotFiniteFloat,
                    code_point_index: 0,
                },
            ),
        },
    }
}

// Executable helpers follow the same small recursive models above.  Their loops are intentionally
// independent from host numeric parsing so arbitrary-length values retain exact correspondence.
fn collect_range_digits(input: &[u32], start: usize, end: usize) -> (digits: Vec<u8>)
    requires
        start <= end <= input@.len(),
        forall|index: int| start <= index < end ==> 0x30 <= #[trigger] input@[index] <= 0x39,
    ensures
        digits@ == Seq::new(
            (end - start) as nat,
            |offset: int| (input@[start + offset] - 0x30) as u8,
        ),
        decimal_digits_bounded_spec(digits@),
{
    let mut digits = Vec::new();
    let mut index = start;
    assert(Seq::new(0, |offset: int| (input@[start + offset] - 0x30) as u8) =~= Seq::<u8>::empty());
    while index < end
        invariant
            start <= index <= end <= input@.len(),
            forall|candidate: int|
                start <= candidate < end ==> 0x30 <= #[trigger] input@[candidate] <= 0x39,
            digits@ == Seq::new(
                (index - start) as nat,
                |offset: int| (input@[start + offset] - 0x30) as u8,
            ),
            decimal_digits_bounded_spec(digits@),
        decreases end - index,
    {
        let code_point = input[index];
        assert(start <= index < end);
        assert(0x30 <= input@[index as int] <= 0x39);
        assert(0x30 <= code_point <= 0x39) by {
            assert(input@[index as int] == code_point);
        }
        proof {
            lemma_decimal_digits_bounded_push(digits@, (code_point - 0x30) as u8);
        }
        digits.push((code_point - 0x30) as u8);
        index += 1;
    }
    digits
}

fn append_range_digits(input: &[u32], start: usize, end: usize, digits: &mut Vec<u8>)
    requires
        start <= end <= input@.len(),
        forall|index: int| start <= index < end ==> 0x30 <= #[trigger] input@[index] <= 0x39,
        decimal_digits_bounded_spec(old(digits)@),
    ensures
        final(digits)@ == old(digits)@ + Seq::new(
            (end - start) as nat,
            |offset: int| (input@[start + offset] - 0x30) as u8,
        ),
        decimal_digits_bounded_spec(final(digits)@),
{
    let ghost original = old(digits)@;
    let mut index = start;
    assert(original + Seq::new(0, |offset: int| (input@[start + offset] - 0x30) as u8)
        =~= original);
    while index < end
        invariant
            start <= index <= end <= input@.len(),
            forall|candidate: int|
                start <= candidate < end ==> 0x30 <= #[trigger] input@[candidate] <= 0x39,
            digits@ == original + Seq::new(
                (index - start) as nat,
                |offset: int| (input@[start + offset] - 0x30) as u8,
            ),
            decimal_digits_bounded_spec(digits@),
        decreases end - index,
    {
        let code_point = input[index];
        assert(start <= index < end);
        assert(0x30 <= input@[index as int] <= 0x39);
        assert(0x30 <= code_point <= 0x39) by {
            assert(input@[index as int] == code_point);
        }
        proof {
            lemma_decimal_digits_bounded_push(digits@, (code_point - 0x30) as u8);
        }
        digits.push((code_point - 0x30) as u8);
        index += 1;
    }
}

fn trim_trailing_decimal_zeros(digits: &mut Vec<u8>)
    requires
        old(digits)@.len() > 0,
        decimal_digits_bounded_spec(old(digits)@),
    ensures
        final(digits)@ == trim_trailing_decimal_zeros_spec(old(digits)@, old(digits)@.len() as nat),
        final(digits)@.len() > 0,
        final(digits)@.len() <= old(digits)@.len(),
        decimal_digits_bounded_spec(final(digits)@),
        final(digits)@.len() == 1 || final(digits)@[final(digits)@.len() - 1] != 0,
{
    let ghost original = old(digits)@;
    let ghost expected = trim_trailing_decimal_zeros_spec(original, original.len() as nat);
    while digits.len() > 1 && digits[digits.len() - 1] == 0
        invariant
            digits@.len() > 0,
            digits@.len() <= original.len(),
            decimal_digits_bounded_spec(digits@),
            expected == trim_trailing_decimal_zeros_spec(digits@, digits@.len() as nat),
            expected == trim_trailing_decimal_zeros_spec(original, original.len() as nat),
        decreases digits.len(),
    {
        proof {
            reveal(trim_trailing_decimal_zeros_spec);
        }
        digits.pop();
        proof {
            reveal(decimal_digits_bounded_spec);
        }
    }
    proof {
        reveal(trim_trailing_decimal_zeros_spec);
    }
}

fn copy_decimal_suffix(digits: &[u8], start: usize) -> (suffix: Vec<u8>)
    requires
        start < digits@.len(),
        decimal_digits_bounded_spec(digits@),
    ensures
        suffix@ == digits@.subrange(start as int, digits@.len() as int),
        suffix@.len() > 0,
        decimal_digits_bounded_spec(suffix@),
{
    let mut suffix = Vec::new();
    let mut index = start;
    while index < digits.len()
        invariant
            start <= index <= digits@.len(),
            start < digits@.len(),
            decimal_digits_bounded_spec(digits@),
            suffix@ == digits@.subrange(start as int, index as int),
            decimal_digits_bounded_spec(suffix@),
        decreases digits.len() - index,
    {
        proof {
            lemma_decimal_digit_bounded(digits@, index as int);
            lemma_decimal_digits_bounded_push(suffix@, digits@[index as int]);
        }
        suffix.push(digits[index]);
        index += 1;
    }
    assert(suffix@.len() == digits@.len() - start);
    suffix
}

fn trim_leading_decimal_zeros(digits: &mut Vec<u8>)
    requires
        old(digits)@.len() > 0,
        decimal_digits_bounded_spec(old(digits)@),
    ensures
        final(digits)@ == trim_leading_decimal_zeros_spec(old(digits)@, old(digits)@.len() as nat),
        final(digits)@.len() > 0,
        final(digits)@.len() <= old(digits)@.len(),
        decimal_digits_bounded_spec(final(digits)@),
        final(digits)@.len() == 1 || final(digits)@[0] != 0,
{
    let ghost original = old(digits)@;
    let ghost expected = trim_leading_decimal_zeros_spec(original, original.len() as nat);
    let mut start = 0usize;
    assert(original.subrange(0, original.len() as int) =~= original);
    while start < digits.len() - 1 && digits[start] == 0
        invariant
            start < digits@.len(),
            digits@ == original,
            digits@.len() <= original.len(),
            decimal_digits_bounded_spec(digits@),
            expected == trim_leading_decimal_zeros_spec(
                digits@.subrange(start as int, digits@.len() as int),
                (digits@.len() - start) as nat,
            ),
            expected == trim_leading_decimal_zeros_spec(original, original.len() as nat),
        decreases digits.len() - start,
    {
        proof {
            reveal(trim_leading_decimal_zeros_spec);
            assert(digits@.subrange(start as int, digits@.len() as int).drop_first()
                =~= digits@.subrange(start as int + 1, digits@.len() as int));
        }
        start += 1;
    }
    proof {
        reveal(trim_leading_decimal_zeros_spec);
    }
    let retained = copy_decimal_suffix(digits.as_slice(), start);
    *digits = retained;
}

fn reverse_digits(digits: &[u8]) -> (reversed: Vec<u8>)
    ensures
        reversed@ == reverse_digits_spec(digits@),
        decimal_digits_bounded_spec(digits@) ==> decimal_digits_bounded_spec(reversed@),
        digits@.len() > 0 && (digits@.len() == 1 || digits@[0] != 0) && decimal_digits_bounded_spec(
            digits@,
        ) ==> canonical_decimal_digits_le_spec(reversed@),
{
    let mut reversed = Vec::new();
    let mut remaining = digits.len();
    while remaining > 0
        invariant
            remaining <= digits@.len(),
            reversed@ == Seq::new(
                (digits@.len() - remaining) as nat,
                |index: int| digits@[digits@.len() - index - 1],
            ),
            decimal_digits_bounded_spec(digits@) ==> decimal_digits_bounded_spec(reversed@),
        decreases remaining,
    {
        remaining -= 1;
        reversed.push(digits[remaining]);
        proof {
            reveal(decimal_digits_bounded_spec);
        }
    }
    proof {
        reveal(reverse_digits_spec);
        reveal(canonical_decimal_digits_le_spec);
    }
    reversed
}

fn small_decimal_digits_le(value: u64) -> (digits: Vec<u8>)
    requires
        value <= MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
    ensures
        digits@ == small_decimal_digits_le_spec(value),
        canonical_decimal_digits_le_spec(digits@),
{
    let ghost expected = small_decimal_digits_le_spec(value);
    let mut remaining = value;
    let mut digits = Vec::new();
    while remaining >= 10
        invariant
            remaining <= MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
            value > 0 ==> remaining > 0,
            decimal_digits_bounded_spec(digits@),
            expected == digits@ + small_decimal_digits_le_spec(remaining),
        decreases remaining,
    {
        assert(((remaining % 10) as u8) < 10);
        proof {
            lemma_decimal_digits_bounded_push(digits@, (remaining % 10) as u8);
        }
        digits.push((remaining % 10) as u8);
        proof {
            reveal(small_decimal_digits_le_spec);
            reveal(decimal_digits_bounded_spec);
        }
        remaining /= 10;
    }
    assert(remaining < 10);
    assert(value > 0 ==> remaining > 0);
    proof {
        lemma_decimal_digits_bounded_push(digits@, remaining as u8);
    }
    digits.push(remaining as u8);
    proof {
        reveal(small_decimal_digits_le_spec);
        reveal(decimal_digits_bounded_spec);
        reveal(canonical_decimal_digits_le_spec);
    }
    digits
}

fn compare_decimal_magnitude_le(left: &[u8], right: &[u8]) -> (comparison: i8)
    requires
        canonical_decimal_digits_le_spec(left@),
        canonical_decimal_digits_le_spec(right@),
    ensures
        comparison as int == compare_decimal_magnitude_le_spec(
            left@,
            right@,
            left@.len() as int - 1,
            left@.len() as nat,
        ),
        -1 <= comparison <= 1,
{
    if left.len() < right.len() {
        proof {
            reveal(compare_decimal_magnitude_le_spec);
            assert(compare_decimal_magnitude_le_spec(
                left@,
                right@,
                left@.len() as int - 1,
                left@.len() as nat,
            ) == -1);
        }
        return -1;
    }
    if left.len() > right.len() {
        proof {
            reveal(compare_decimal_magnitude_le_spec);
            assert(compare_decimal_magnitude_le_spec(
                left@,
                right@,
                left@.len() as int - 1,
                left@.len() as nat,
            ) == 1);
        }
        return 1;
    }
    let ghost expected = compare_decimal_magnitude_le_spec(
        left@,
        right@,
        left@.len() as int - 1,
        left@.len() as nat,
    );
    let mut remaining = left.len();
    while remaining > 0
        invariant
            remaining <= left@.len(),
            left@.len() == right@.len(),
            expected == compare_decimal_magnitude_le_spec(
                left@,
                right@,
                left@.len() as int - 1,
                left@.len() as nat,
            ),
            expected == compare_decimal_magnitude_le_spec(
                left@,
                right@,
                remaining as int - 1,
                remaining as nat,
            ),
        decreases remaining,
    {
        let index = remaining - 1;
        if left[index] < right[index] {
            proof {
                reveal(compare_decimal_magnitude_le_spec);
                assert(expected == -1);
            }
            return -1;
        }
        if left[index] > right[index] {
            proof {
                reveal(compare_decimal_magnitude_le_spec);
                assert(expected == 1);
            }
            return 1;
        }
        proof {
            reveal(compare_decimal_magnitude_le_spec);
        }
        remaining -= 1;
    }
    proof {
        reveal(compare_decimal_magnitude_le_spec);
    }
    0
}

fn add_decimal_magnitudes_le(left: &[u8], right: &[u8]) -> (sum: Vec<u8>)
    requires
        canonical_decimal_digits_le_spec(left@),
        canonical_decimal_digits_le_spec(right@),
    ensures
        sum@ == add_decimal_magnitudes_le_spec(
            left@,
            right@,
            0,
            0,
            (left@.len() + right@.len() + 1) as nat,
        ),
        canonical_decimal_digits_le_spec(sum@),
{
    assert(decimal_digits_bounded_spec(left@)) by {
        reveal(canonical_decimal_digits_le_spec);
    }
    assert(decimal_digits_bounded_spec(right@)) by {
        reveal(canonical_decimal_digits_le_spec);
    }
    assert(left@.len() > 0 && right@.len() > 0) by {
        reveal(canonical_decimal_digits_le_spec);
    }
    let ghost expected = add_decimal_magnitudes_le_spec(
        left@,
        right@,
        0,
        0,
        (left@.len() + right@.len() + 1) as nat,
    );
    let maximum = if left.len() > right.len() {
        left.len()
    } else {
        right.len()
    };
    let mut index = 0usize;
    let mut carry = 0u8;
    let mut output = Vec::new();
    while index < maximum
        invariant
            index <= maximum,
            maximum == if left@.len() > right@.len() {
                left@.len()
            } else {
                right@.len()
            },
            carry <= 1,
            canonical_decimal_digits_le_spec(left@),
            canonical_decimal_digits_le_spec(right@),
            decimal_digits_bounded_spec(left@),
            decimal_digits_bounded_spec(right@),
            decimal_digits_bounded_spec(output@),
            output@.len() == index,
            index == maximum && carry == 0 ==> output@.len() == 1 || output@[output@.len() - 1]
                != 0,
            expected == output@ + add_decimal_magnitudes_le_spec(
                left@,
                right@,
                index as int,
                carry,
                (left@.len() + right@.len() + 1 - index) as nat,
            ),
        decreases maximum - index,
    {
        let left_digit = if index < left.len() {
            proof {
                reveal(canonical_decimal_digits_le_spec);
                lemma_decimal_digit_bounded(left@, index as int);
            }
            left[index]
        } else {
            0
        };
        let right_digit = if index < right.len() {
            proof {
                reveal(canonical_decimal_digits_le_spec);
                lemma_decimal_digit_bounded(right@, index as int);
            }
            right[index]
        } else {
            0
        };
        assert(left_digit < 10 && right_digit < 10) by {
            reveal(canonical_decimal_digits_le_spec);
            reveal(decimal_digits_bounded_spec);
        }
        let expanded = left_digit as u16 + right_digit as u16 + carry as u16;
        if index + 1 == maximum && maximum > 1 {
            if left.len() == maximum {
                assert(index == left@.len() - 1);
                assert(left@[index as int] != 0) by {
                    reveal(canonical_decimal_digits_le_spec);
                }
                assert(left_digit != 0);
            } else {
                assert(right.len() == maximum);
                assert(index == right@.len() - 1);
                assert(right@[index as int] != 0) by {
                    reveal(canonical_decimal_digits_le_spec);
                }
                assert(right_digit != 0);
            }
            assert(expanded > 0);
            assert(expanded / 10 == 0 ==> expanded % 10 != 0);
        }
        proof {
            reveal(add_decimal_magnitudes_le_spec);
            lemma_decimal_digits_bounded_push(output@, (expanded % 10) as u8);
        }
        output.push((expanded % 10) as u8);
        carry = (expanded / 10) as u8;
        proof {
            reveal(decimal_digits_bounded_spec);
        }
        index += 1;
    }
    proof {
        reveal(add_decimal_magnitudes_le_spec);
    }
    if carry > 0 {
        proof {
            lemma_decimal_digits_bounded_push(output@, carry);
        }
        output.push(carry);
    }
    assert(output@.len() > 0);
    assert(output@.len() == 1 || output@[output@.len() - 1] != 0);
    proof {
        reveal(decimal_digits_bounded_spec);
        reveal(canonical_decimal_digits_le_spec);
    }
    output
}

fn subtract_decimal_magnitudes_le(larger: &[u8], smaller: &[u8]) -> (difference: Vec<u8>)
    requires
        canonical_decimal_digits_le_spec(larger@),
        canonical_decimal_digits_le_spec(smaller@),
        compare_decimal_magnitude_le_spec(
            larger@,
            smaller@,
            larger@.len() as int - 1,
            larger@.len() as nat,
        ) > 0,
    ensures
        difference@ == trim_high_decimal_zeros_le_spec(
            subtract_decimal_magnitudes_le_spec(larger@, smaller@, 0, 0, larger@.len() as nat).0,
            subtract_decimal_magnitudes_le_spec(
                larger@,
                smaller@,
                0,
                0,
                larger@.len() as nat,
            ).0.len() as nat,
        ),
        canonical_decimal_digits_le_spec(difference@),
{
    assert(decimal_digits_bounded_spec(larger@)) by {
        reveal(canonical_decimal_digits_le_spec);
    }
    assert(decimal_digits_bounded_spec(smaller@)) by {
        reveal(canonical_decimal_digits_le_spec);
    }
    let ghost raw_expected = subtract_decimal_magnitudes_le_spec(
        larger@,
        smaller@,
        0,
        0,
        larger@.len() as nat,
    );
    let mut output = Vec::new();
    let mut index = 0usize;
    let mut borrow = 0u8;
    while index < larger.len()
        invariant
            index <= larger@.len(),
            borrow <= 1,
            canonical_decimal_digits_le_spec(larger@),
            canonical_decimal_digits_le_spec(smaller@),
            decimal_digits_bounded_spec(larger@),
            decimal_digits_bounded_spec(smaller@),
            decimal_digits_bounded_spec(output@),
            output@.len() == index,
            raw_expected.0 == output@ + subtract_decimal_magnitudes_le_spec(
                larger@,
                smaller@,
                index as int,
                borrow,
                (larger@.len() - index) as nat,
            ).0,
            raw_expected.1 == subtract_decimal_magnitudes_le_spec(
                larger@,
                smaller@,
                index as int,
                borrow,
                (larger@.len() - index) as nat,
            ).1,
        decreases larger.len() - index,
    {
        let smaller_digit = if index < smaller.len() {
            proof {
                reveal(canonical_decimal_digits_le_spec);
                lemma_decimal_digit_bounded(smaller@, index as int);
            }
            smaller[index]
        } else {
            0
        };
        assert(smaller_digit < 10);
        assert(borrow <= 1);
        let subtrahend = smaller_digit + borrow;
        let larger_digit = larger[index];
        proof {
            reveal(canonical_decimal_digits_le_spec);
            lemma_decimal_digit_bounded(larger@, index as int);
        }
        let digit = if larger_digit >= subtrahend {
            larger_digit - subtrahend
        } else {
            10 + larger_digit - subtrahend
        };
        let next_borrow = if larger_digit >= subtrahend {
            0
        } else {
            1
        };
        proof {
            reveal(subtract_decimal_magnitudes_le_spec);
            lemma_decimal_digits_bounded_push(output@, digit);
        }
        output.push(digit);
        borrow = next_borrow;
        proof {
            reveal(decimal_digits_bounded_spec);
        }
        index += 1;
    }
    assert(output@.len() == larger@.len());
    assert(output@.len() > 0) by {
        reveal(canonical_decimal_digits_le_spec);
    }
    let ghost untrimmed = output@;
    trim_trailing_decimal_zeros(&mut output);
    proof {
        reveal(trim_high_decimal_zeros_le_spec);
        reveal(canonical_decimal_digits_le_spec);
    }
    output
}

fn apply_signed_small_decimal(
    negative: bool,
    digits: &[u8],
    delta_negative: bool,
    delta: u64,
) -> (result: (bool, Vec<u8>))
    requires
        canonical_decimal_digits_le_spec(digits@),
        delta <= MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS,
    ensures
        (result.0, result.1@) == apply_signed_small_decimal_spec(
            negative,
            digits@,
            delta_negative,
            delta,
        ),
        canonical_decimal_digits_le_spec(result.1@),
        decimal_digits_zero_spec(result.1@) ==> !result.0,
{
    if delta == 0 {
        let copied = vstd::slice::slice_to_vec(digits);
        proof {
            reveal(apply_signed_small_decimal_spec);
            reveal(apply_signed_small_decimal_raw_spec);
            reveal(decimal_digits_zero_spec);
        }
        let is_zero = digits.len() == 1 && digits[0] == 0;
        return ((!is_zero && negative), copied);
    }
    let delta_digits = small_decimal_digits_le(delta);
    if digits.len() == 1 && digits[0] == 0 {
        let is_zero = delta_digits.len() == 1 && delta_digits[0] == 0;
        proof {
            reveal(apply_signed_small_decimal_spec);
            reveal(apply_signed_small_decimal_raw_spec);
            reveal(decimal_digits_zero_spec);
        }
        return ((!is_zero && delta_negative), delta_digits);
    }
    if negative == delta_negative {
        let sum = add_decimal_magnitudes_le(digits, delta_digits.as_slice());
        let is_zero = sum.len() == 1 && sum[0] == 0;
        proof {
            reveal(apply_signed_small_decimal_spec);
            reveal(apply_signed_small_decimal_raw_spec);
            reveal(decimal_digits_zero_spec);
        }
        return ((!is_zero && negative), sum);
    }
    let comparison = compare_decimal_magnitude_le(digits, delta_digits.as_slice());
    if comparison == 0 {
        let zero = vec![0];
        proof {
            reveal(apply_signed_small_decimal_spec);
            reveal(apply_signed_small_decimal_raw_spec);
            reveal(decimal_digits_zero_spec);
            reveal(canonical_decimal_digits_le_spec);
            reveal(decimal_digits_bounded_spec);
        }
        return (false, zero);
    }
    if comparison > 0 {
        let difference = subtract_decimal_magnitudes_le(digits, delta_digits.as_slice());
        let is_zero = difference.len() == 1 && difference[0] == 0;
        proof {
            reveal(apply_signed_small_decimal_spec);
            reveal(apply_signed_small_decimal_raw_spec);
            reveal(decimal_digits_zero_spec);
        }
        ((!is_zero && negative), difference)
    } else {
        proof {
            lemma_compare_decimal_magnitude_reverse(digits@, delta_digits@);
        }
        let difference = subtract_decimal_magnitudes_le(delta_digits.as_slice(), digits);
        let is_zero = difference.len() == 1 && difference[0] == 0;
        proof {
            reveal(apply_signed_small_decimal_spec);
            reveal(apply_signed_small_decimal_raw_spec);
            reveal(decimal_digits_zero_spec);
        }
        ((!is_zero && delta_negative), difference)
    }
}

/// Convert one YAML 1.2.2 Core finite decimal to its exact canonical semantic value.
pub fn convert_core_finite_float(input: &[u32], limits: CoreFiniteFloatLimits) -> (result: Result<
    CoreFiniteFloat,
    CoreFiniteFloatError,
>)
    ensures
        convert_core_finite_float_spec(input@, limits@) == match result {
            Ok(value) => Ok(value@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(value) => {
                canonical_decimal_digits_le_spec(value@.coefficient_digits_le)
                    && canonical_decimal_digits_le_spec(value@.exponent_digits_le)
                    && value@.coefficient_digits_le.len() <= limits@.max_coefficient_digits
                    && value@.coefficient_digits_le.len()
                    <= MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS && value@.exponent_digits_le.len()
                    <= limits@.max_exponent_digits && value@.exponent_digits_le.len()
                    <= MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS && (decimal_digits_zero_spec(
                    value@.exponent_digits_le,
                ) ==> !value@.exponent_negative)
            },
            Err(_) => true,
        },
{
    let scalar_limits = CoreScalarLimits::new(limits.max_code_points);
    let class = match classify_core_plain_scalar(input, scalar_limits) {
        Ok(class) => class,
        Err(error) => {
            let converted = CoreFiniteFloatError::at(
                match error.kind() {
                    CoreScalarErrorKind::InputLimitExceeded => {
                        CoreFiniteFloatErrorKind::InputLimitExceeded
                    },
                },
                error.code_point_index(),
            );
            proof {
                reveal(convert_core_finite_float_spec);
            }
            return Err(converted);
        },
    };
    let (negative, whole, fraction, exponent_negative, exponent) = match class {
        CorePlainScalarClass::FiniteFloat {
            negative,
            whole,
            fraction,
            exponent_negative,
            exponent,
        } => (negative, whole, fraction, exponent_negative, exponent),
        _ => {
            let error = CoreFiniteFloatError::at(CoreFiniteFloatErrorKind::NotFiniteFloat, 0);
            proof {
                reveal(convert_core_finite_float_spec);
            }
            return Err(error);
        },
    };
    proof {
        crate::resolve::lemma_classified_core_finite_float_has_exact_digits(
            input@,
            scalar_limits@,
            negative,
            whole,
            fraction,
            exponent_negative,
            exponent,
        );
    }
    let whole_start = whole.start() as usize;
    let whole_end = whole.end() as usize;
    assert(whole_start as u64 == whole.start);
    assert(whole_end as u64 == whole.end);
    assert(whole_start <= whole_end <= input.len());
    let mut coefficient_be = collect_range_digits(input, whole_start, whole_end);
    if let Some(range) = fraction {
        let fraction_start = range.start() as usize;
        let fraction_end = range.end() as usize;
        assert(fraction_start as u64 == range.start);
        assert(fraction_end as u64 == range.end);
        assert(fraction_start <= fraction_end <= input.len());
        append_range_digits(input, fraction_start, fraction_end, &mut coefficient_be);
    }
    let ghost coefficient_be_model = coefficient_digits_be_spec(input@, whole, fraction);
    assert(coefficient_be@ == coefficient_be_model) by {
        reveal(coefficient_digits_be_spec);
        reveal(decimal_range_digits_spec);
    }
    assert(coefficient_be.len() > 0);
    let original_coefficient_len = coefficient_be.len();
    assert(original_coefficient_len == coefficient_be_model.len());
    trim_trailing_decimal_zeros(&mut coefficient_be);
    let trailing_zeros = original_coefficient_len - coefficient_be.len();
    let before_leading_len = coefficient_be.len();
    trim_leading_decimal_zeros(&mut coefficient_be);
    let leading_zeros = before_leading_len - coefficient_be.len();
    let coefficient_digits_le = reverse_digits(coefficient_be.as_slice());
    assert((coefficient_digits_le@, trailing_zeros as u64, leading_zeros as u64)
        == canonicalize_coefficient_spec(coefficient_be_model)) by {
        reveal(canonicalize_coefficient_spec);
    }
    let coefficient_limit = if limits.max_coefficient_digits
        < MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS {
        limits.max_coefficient_digits
    } else {
        MAX_PROFILE1_CORE_FLOAT_COEFFICIENT_DIGITS
    };
    if coefficient_digits_le.len() as u64 > coefficient_limit {
        let excluded = leading_zeros as u64 + coefficient_limit;
        let source_index = if excluded < whole.end() - whole.start() {
            whole.start() + excluded
        } else {
            match fraction {
                Some(range) => range.start() + excluded - (whole.end() - whole.start()),
                None => whole.end(),
            }
        };
        let error = CoreFiniteFloatError::at(
            CoreFiniteFloatErrorKind::CoefficientLimitExceeded,
            source_index,
        );
        proof {
            reveal(convert_core_finite_float_spec);
            reveal(effective_float_coefficient_limit_spec);
            reveal(canonicalize_coefficient_spec);
            reveal(coefficient_source_index_spec);
        }
        return Err(error);
    }
    let fraction_len = match fraction {
        Some(range) => range.end() - range.start(),
        None => 0,
    };
    let mut exponent_be = match exponent {
        Some(range) => {
            let exponent_start = range.start() as usize;
            let exponent_end = range.end() as usize;
            assert(exponent_start as u64 == range.start);
            assert(exponent_end as u64 == range.end);
            assert(exponent_start < exponent_end <= input.len());
            collect_range_digits(input, exponent_start, exponent_end)
        },
        None => { vec![0] },
    };
    let ghost exponent_be_model = match exponent {
        Some(range) => decimal_range_digits_spec(input@, range),
        None => Seq::empty().push(0u8),
    };
    assert(exponent_be@ == exponent_be_model) by {
        reveal(decimal_range_digits_spec);
    }
    let raw_exponent_be_len = exponent_be.len();
    assert(raw_exponent_be_len == exponent_be_model.len());
    trim_leading_decimal_zeros(&mut exponent_be);
    let raw_exponent_leading = raw_exponent_be_len - exponent_be.len();
    let raw_exponent_digits_le = reverse_digits(exponent_be.as_slice());
    assert(exponent_be@.len() == raw_exponent_digits_le@.len()) by {
        reveal(reverse_digits_spec);
    }
    assert(raw_exponent_leading as int == exponent_be_model.len() - raw_exponent_digits_le@.len());
    assert(raw_exponent_digits_le@ == canonicalize_unsigned_decimal_spec(exponent_be_model)) by {
        reveal(canonicalize_unsigned_decimal_spec);
    }
    let coefficient_is_zero = coefficient_digits_le.len() == 1 && coefficient_digits_le[0] == 0;
    let (normalized_exponent_negative, normalized_exponent_digits_le) = if coefficient_is_zero {
        (false, vec![0])
    } else {
        let delta_negative = fraction_len > trailing_zeros as u64;
        let delta = if delta_negative {
            fraction_len - trailing_zeros as u64
        } else {
            trailing_zeros as u64 - fraction_len
        };
        apply_signed_small_decimal(
            exponent_negative,
            raw_exponent_digits_le.as_slice(),
            delta_negative,
            delta,
        )
    };
    assert((normalized_exponent_negative, normalized_exponent_digits_le@)
        == if decimal_digits_zero_spec(coefficient_digits_le@) {
        (false, Seq::empty().push(0u8))
    } else {
        apply_signed_small_decimal_spec(
            exponent_negative,
            raw_exponent_digits_le@,
            fraction_len > trailing_zeros as u64,
            if fraction_len > trailing_zeros as u64 {
                (fraction_len - trailing_zeros as u64) as u64
            } else {
                (trailing_zeros as u64 - fraction_len) as u64
            },
        )
    }) by {
        reveal(decimal_digits_zero_spec);
    }
    let exponent_limit = if limits.max_exponent_digits < MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS {
        limits.max_exponent_digits
    } else {
        MAX_PROFILE1_CORE_FLOAT_EXPONENT_DIGITS
    };
    if normalized_exponent_digits_le.len() as u64 > exponent_limit {
        let ghost expected_source_index = match exponent {
            Some(range) => if raw_exponent_digits_le@.len() > exponent_limit {
                (range.start + (exponent_be_model.len() - raw_exponent_digits_le@.len())
                    + exponent_limit) as u64
            } else {
                exponent_adjustment_anchor_spec(
                    whole,
                    fraction,
                    exponent,
                    coefficient_be_model.len() as u64,
                )
            },
            None => exponent_adjustment_anchor_spec(
                whole,
                fraction,
                exponent,
                coefficient_be_model.len() as u64,
            ),
        };
        let source_index = match exponent {
            Some(range) if raw_exponent_digits_le.len() as u64 > exponent_limit => {
                range.start() + raw_exponent_leading as u64 + exponent_limit
            },
            Some(range) => range.start(),
            None => match fraction {
                Some(range) if range.end() > range.start() => range.end() - 1,
                _ => {
                    let last = original_coefficient_len as u64 - 1;
                    if last < whole.end() - whole.start() {
                        whole.start() + last
                    } else {
                        match fraction {
                            Some(range) => range.start() + last - (whole.end() - whole.start()),
                            None => whole.end(),
                        }
                    }
                },
            },
        };
        assert((raw_exponent_digits_le.len() as u64 > exponent_limit) == (
        raw_exponent_digits_le@.len() > exponent_limit));
        proof {
            match exponent {
                Some(range) => {
                    if raw_exponent_digits_le.len() as u64 > exponent_limit {
                        assert(raw_exponent_leading as int == exponent_be_model.len()
                            - raw_exponent_digits_le@.len());
                        assert(source_index == range.start + raw_exponent_leading as u64
                            + exponent_limit);
                        assert(expected_source_index == range.start + raw_exponent_leading as u64
                            + exponent_limit);
                        assert(source_index == expected_source_index);
                    } else {
                        reveal(exponent_adjustment_anchor_spec);
                        assert(source_index == range.start);
                        assert(expected_source_index == range.start);
                        assert(source_index == expected_source_index);
                    }
                },
                None => {
                    reveal(exponent_adjustment_anchor_spec);
                    match fraction {
                        Some(range) => {
                            if range.end > range.start {
                                assert(source_index == range.end - 1);
                                assert(expected_source_index == range.end - 1);
                                assert(source_index == expected_source_index);
                            } else {
                                reveal(coefficient_source_index_spec);
                                reveal(coefficient_digits_be_spec);
                                reveal(decimal_range_digits_spec);
                                assert(original_coefficient_len as int == whole.end - whole.start);
                                assert(original_coefficient_len > 0);
                                assert((original_coefficient_len as u64 - 1) < whole.end
                                    - whole.start);
                                assert(source_index == whole.start + original_coefficient_len as u64
                                    - 1);
                                assert(expected_source_index == whole.start
                                    + coefficient_be_model.len() as u64 - 1);
                                assert(original_coefficient_len as u64
                                    == coefficient_be_model.len() as u64);
                                assert(source_index == expected_source_index);
                            }
                        },
                        None => {
                            reveal(coefficient_source_index_spec);
                            reveal(coefficient_digits_be_spec);
                            reveal(decimal_range_digits_spec);
                            assert(original_coefficient_len as int == whole.end - whole.start);
                            assert(original_coefficient_len > 0);
                            assert((original_coefficient_len as u64 - 1) < whole.end - whole.start);
                            assert(source_index == whole.start + original_coefficient_len as u64
                                - 1);
                            assert(expected_source_index == whole.start
                                + coefficient_be_model.len() as u64 - 1);
                            assert(original_coefficient_len as u64
                                == coefficient_be_model.len() as u64);
                            assert(source_index == expected_source_index);
                        },
                    }
                },
            }
        }
        let error = CoreFiniteFloatError::at(
            CoreFiniteFloatErrorKind::ExponentLimitExceeded,
            source_index,
        );
        proof {
            reveal(convert_core_finite_float_spec);
            reveal(effective_float_exponent_limit_spec);
            reveal(exponent_adjustment_anchor_spec);
            reveal(coefficient_source_index_spec);
            reveal(canonicalize_unsigned_decimal_spec);
            reveal(canonicalize_coefficient_spec);
        }
        return Err(error);
    }
    let value = CoreFiniteFloat::new(
        negative,
        coefficient_digits_le,
        normalized_exponent_negative,
        normalized_exponent_digits_le,
    );
    proof {
        reveal(convert_core_finite_float_spec);
        reveal(effective_float_coefficient_limit_spec);
        reveal(effective_float_exponent_limit_spec);
        reveal(canonicalize_coefficient_spec);
        reveal(canonicalize_unsigned_decimal_spec);
        reveal(decimal_digits_zero_spec);
    }
    Ok(value)
}

} // verus!
