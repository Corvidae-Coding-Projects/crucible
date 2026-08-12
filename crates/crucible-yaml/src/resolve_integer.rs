//! Verified arbitrary-width conversion of YAML 1.2.2 Core integers.
//!
//! Magnitudes use canonical little-endian base-1,000,000,000 limbs.  Conversion never passes
//! through a host-width integer and applies its effective limb cap to each canonical per-digit
//! result before the next digit is admitted.
#[allow(unused_imports)]
use crate::resolve::CoreScalarLimitsView;
use crate::resolve::{
    classify_core_plain_scalar, CoreIntegerBase, CorePlainScalarClass, CoreScalarErrorKind,
    CoreScalarLimits,
};
use vstd::prelude::*;

verus! {

pub const CORE_INTEGER_CONVERSION_VERSION: u16 = 1;

pub const CORE_INTEGER_MAGNITUDE_RADIX: u32 = 1_000_000_000;

pub const MAX_PROFILE1_CORE_INTEGER_LIMBS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreIntegerLimits {
    max_code_points: u64,
    max_limbs: u64,
}

#[verifier::ext_equal]
pub struct CoreIntegerLimitsView {
    pub max_code_points: u64,
    pub max_limbs: u64,
}

impl View for CoreIntegerLimits {
    type V = CoreIntegerLimitsView;

    closed spec fn view(&self) -> CoreIntegerLimitsView {
        CoreIntegerLimitsView { max_code_points: self.max_code_points, max_limbs: self.max_limbs }
    }
}

impl CoreIntegerLimits {
    pub fn new(max_code_points: u64, max_limbs: u64) -> (limits: Self)
        ensures
            limits@ == (CoreIntegerLimitsView { max_code_points, max_limbs }),
    {
        Self { max_code_points, max_limbs }
    }

    pub fn max_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_code_points,
    {
        self.max_code_points
    }

    pub fn max_limbs(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_limbs,
    {
        self.max_limbs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CoreIntegerErrorKind {
    InputLimitExceeded,
    NotInteger,
    MagnitudeLimitExceeded,
    FuelExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreIntegerError {
    kind: CoreIntegerErrorKind,
    code_point_index: u64,
}

#[verifier::ext_equal]
pub struct CoreIntegerErrorView {
    pub kind: CoreIntegerErrorKind,
    pub code_point_index: u64,
}

impl View for CoreIntegerError {
    type V = CoreIntegerErrorView;

    closed spec fn view(&self) -> CoreIntegerErrorView {
        CoreIntegerErrorView { kind: self.kind, code_point_index: self.code_point_index }
    }
}

impl CoreIntegerError {
    fn at(kind: CoreIntegerErrorKind, code_point_index: u64) -> (error: Self)
        ensures
            error@ == (CoreIntegerErrorView { kind, code_point_index }),
    {
        Self { kind, code_point_index }
    }

    pub fn kind(&self) -> (kind: CoreIntegerErrorKind)
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
pub struct CoreInteger {
    negative: bool,
    limbs: Vec<u32>,
}

#[verifier::ext_equal]
pub struct CoreIntegerView {
    pub negative: bool,
    pub limbs: Seq<u32>,
}

impl View for CoreInteger {
    type V = CoreIntegerView;

    closed spec fn view(&self) -> CoreIntegerView {
        CoreIntegerView { negative: self.negative, limbs: self.limbs@ }
    }
}

impl CoreInteger {
    fn new(negative: bool, limbs: Vec<u32>) -> (integer: Self)
        ensures
            integer@ == (CoreIntegerView { negative, limbs: limbs@ }),
    {
        Self { negative, limbs }
    }

    pub fn negative(&self) -> (negative: bool)
        ensures
            negative == self@.negative,
    {
        self.negative
    }

    pub fn limbs(&self) -> (limbs: &[u32])
        ensures
            limbs@ == self@.limbs,
    {
        self.limbs.as_slice()
    }
}

pub open spec fn effective_core_integer_limb_limit_spec(limits: CoreIntegerLimitsView) -> u64 {
    if limits.max_limbs < MAX_PROFILE1_CORE_INTEGER_LIMBS {
        limits.max_limbs
    } else {
        MAX_PROFILE1_CORE_INTEGER_LIMBS
    }
}

pub open spec fn core_integer_digit_value_spec(code_point: u32, base: CoreIntegerBase) -> Option<
    u32,
> {
    match base {
        CoreIntegerBase::Decimal => if 0x30 <= code_point <= 0x39 {
            Some((code_point - 0x30) as u32)
        } else {
            None
        },
        CoreIntegerBase::Octal => if 0x30 <= code_point <= 0x37 {
            Some((code_point - 0x30) as u32)
        } else {
            None
        },
        CoreIntegerBase::Hexadecimal => if 0x30 <= code_point <= 0x39 {
            Some((code_point - 0x30) as u32)
        } else if 0x41 <= code_point <= 0x46 {
            Some((code_point - 0x41 + 10) as u32)
        } else if 0x61 <= code_point <= 0x66 {
            Some((code_point - 0x61 + 10) as u32)
        } else {
            None
        },
    }
}

#[allow(clippy::manual_range_contains)]
fn core_integer_digit_value(code_point: u32, base: CoreIntegerBase) -> (value: Option<u32>)
    ensures
        value == core_integer_digit_value_spec(code_point, base),
{
    match base {
        CoreIntegerBase::Decimal => if 0x30 <= code_point && code_point <= 0x39 {
            Some(code_point - 0x30)
        } else {
            None
        },
        CoreIntegerBase::Octal => if 0x30 <= code_point && code_point <= 0x37 {
            Some(code_point - 0x30)
        } else {
            None
        },
        CoreIntegerBase::Hexadecimal => if 0x30 <= code_point && code_point <= 0x39 {
            Some(code_point - 0x30)
        } else if 0x41 <= code_point && code_point <= 0x46 {
            Some(code_point - 0x41 + 10)
        } else if 0x61 <= code_point && code_point <= 0x66 {
            Some(code_point - 0x61 + 10)
        } else {
            None
        },
    }
}

pub open spec fn core_integer_multiplier_spec(base: CoreIntegerBase) -> u32 {
    match base {
        CoreIntegerBase::Decimal => 10,
        CoreIntegerBase::Octal => 8,
        CoreIntegerBase::Hexadecimal => 16,
    }
}

fn core_integer_multiplier(base: CoreIntegerBase) -> (multiplier: u32)
    ensures
        multiplier == core_integer_multiplier_spec(base),
        2 <= multiplier <= 16,
{
    match base {
        CoreIntegerBase::Decimal => 10,
        CoreIntegerBase::Octal => 8,
        CoreIntegerBase::Hexadecimal => 16,
    }
}

pub open spec fn core_magnitude_mul_add_tail_spec(
    limbs: Seq<u32>,
    index: int,
    multiplier: u32,
    carry: u32,
    fuel: nat,
) -> Seq<u32>
    decreases fuel,
{
    if index < 0 || index >= limbs.len() || fuel == 0 {
        if carry == 0 {
            Seq::empty()
        } else {
            Seq::empty().push(carry)
        }
    } else {
        let expanded = limbs[index] as int * multiplier as int + carry as int;
        Seq::empty().push((expanded % CORE_INTEGER_MAGNITUDE_RADIX as int) as u32)
            + core_magnitude_mul_add_tail_spec(
            limbs,
            index + 1,
            multiplier,
            (expanded / CORE_INTEGER_MAGNITUDE_RADIX as int) as u32,
            (fuel - 1) as nat,
        )
    }
}

pub open spec fn core_magnitude_mul_add_spec(limbs: Seq<u32>, multiplier: u32, addend: u32) -> Seq<
    u32,
> {
    core_magnitude_mul_add_tail_spec(limbs, 0, multiplier, addend, limbs.len() as nat)
}

pub open spec fn core_magnitude_digits_bounded_spec(limbs: Seq<u32>) -> bool {
    forall|index: int|
        0 <= index < limbs.len() ==> #[trigger] limbs[index] < CORE_INTEGER_MAGNITUDE_RADIX
}

fn core_magnitude_mul_add(limbs: &[u32], multiplier: u32, addend: u32) -> (result: Vec<u32>)
    requires
        limbs@.len() > 0,
        core_magnitude_digits_bounded_spec(limbs@),
        2 <= multiplier <= 16,
        addend < multiplier,
    ensures
        result@ == core_magnitude_mul_add_spec(limbs@, multiplier, addend),
        result@.len() > 0,
        core_magnitude_digits_bounded_spec(result@),
        result@.len() <= limbs@.len() + 1,
{
    let ghost expected = core_magnitude_mul_add_spec(limbs@, multiplier, addend);
    let mut output: Vec<u32> = Vec::new();
    let mut index: usize = 0;
    let mut carry = addend;
    while index < limbs.len()
        invariant
            0 <= index <= limbs@.len(),
            limbs@.len() > 0,
            core_magnitude_digits_bounded_spec(limbs@),
            2 <= multiplier <= 16,
            carry < multiplier,
            output@.len() == index,
            core_magnitude_digits_bounded_spec(output@),
            expected == output@ + core_magnitude_mul_add_tail_spec(
                limbs@,
                index as int,
                multiplier,
                carry,
                (limbs@.len() - index) as nat,
            ),
            expected == core_magnitude_mul_add_spec(limbs@, multiplier, addend),
        decreases limbs.len() - index,
    {
        assert(limbs@[index as int] < CORE_INTEGER_MAGNITUDE_RADIX) by {
            reveal(core_magnitude_digits_bounded_spec);
        }
        let limb = limbs[index];
        assert((limb as int) * (multiplier as int) + (carry as int) < (
        CORE_INTEGER_MAGNITUDE_RADIX as int) * (multiplier as int)) by (nonlinear_arith)
            requires
                limb < CORE_INTEGER_MAGNITUDE_RADIX,
                carry < multiplier,
                multiplier > 0,
        ;
        assert((CORE_INTEGER_MAGNITUDE_RADIX as int) * (multiplier as int) <= 16_000_000_000int)
            by (nonlinear_arith)
            requires
                multiplier <= 16,
        ;
        let product = limb as u64 * multiplier as u64;
        let expanded = product + carry as u64;
        assert(expanded < CORE_INTEGER_MAGNITUDE_RADIX as u64 * multiplier as u64);
        let digit = (expanded % CORE_INTEGER_MAGNITUDE_RADIX as u64) as u32;
        let next_carry = (expanded / CORE_INTEGER_MAGNITUDE_RADIX as u64) as u32;
        assert(digit < CORE_INTEGER_MAGNITUDE_RADIX);
        assert(next_carry < multiplier);
        proof {
            reveal(core_magnitude_mul_add_tail_spec);
        }
        output.push(digit);
        proof {
            reveal(core_magnitude_digits_bounded_spec);
        }
        carry = next_carry;
        index += 1;
    }
    proof {
        reveal(core_magnitude_mul_add_tail_spec);
    }
    if carry > 0 {
        assert(carry < CORE_INTEGER_MAGNITUDE_RADIX);
        output.push(carry);
        proof {
            reveal(core_magnitude_digits_bounded_spec);
        }
    }
    proof {
        reveal(core_magnitude_digits_bounded_spec);
    }
    output
}

pub open spec fn trim_core_magnitude_spec(limbs: Seq<u32>, fuel: nat) -> Seq<u32>
    decreases fuel,
{
    if fuel > 0 && limbs.len() > 1 && limbs[limbs.len() - 1] == 0 {
        trim_core_magnitude_spec(limbs.drop_last(), (fuel - 1) as nat)
    } else {
        limbs
    }
}

fn trim_core_magnitude(limbs: &mut Vec<u32>)
    requires
        old(limbs)@.len() > 0,
        core_magnitude_digits_bounded_spec(old(limbs)@),
    ensures
        final(limbs)@ == trim_core_magnitude_spec(old(limbs)@, old(limbs)@.len() as nat),
        final(limbs)@.len() > 0,
        core_magnitude_digits_bounded_spec(final(limbs)@),
        final(limbs)@.len() <= old(limbs)@.len(),
        final(limbs)@.len() == 1 || final(limbs)@[final(limbs)@.len() - 1] != 0,
{
    let ghost original = old(limbs)@;
    let ghost expected = trim_core_magnitude_spec(original, original.len() as nat);
    while limbs.len() > 1 && limbs[limbs.len() - 1] == 0
        invariant
            limbs@.len() > 0,
            limbs@.len() <= original.len(),
            core_magnitude_digits_bounded_spec(limbs@),
            expected == trim_core_magnitude_spec(limbs@, limbs@.len() as nat),
            expected == trim_core_magnitude_spec(original, original.len() as nat),
        decreases limbs.len(),
    {
        proof {
            reveal(trim_core_magnitude_spec);
        }
        limbs.pop();
        proof {
            reveal(core_magnitude_digits_bounded_spec);
        }
    }
    proof {
        reveal(trim_core_magnitude_spec);
    }
}

pub open spec fn core_integer_digits_spec(
    input: Seq<u32>,
    index: int,
    end: int,
    base: CoreIntegerBase,
    limbs: Seq<u32>,
    limb_limit: u64,
    fuel: nat,
) -> Result<Seq<u32>, CoreIntegerErrorView>
    decreases fuel,
{
    if index < 0 || end < index || end > input.len() {
        Err(
            CoreIntegerErrorView {
                kind: CoreIntegerErrorKind::NotInteger,
                code_point_index: if index < 0 {
                    0
                } else {
                    index as u64
                },
            },
        )
    } else if index == end {
        Ok(limbs)
    } else if fuel == 0 {
        Err(
            CoreIntegerErrorView {
                kind: CoreIntegerErrorKind::FuelExhausted,
                code_point_index: index as u64,
            },
        )
    } else {
        match core_integer_digit_value_spec(input[index], base) {
            None => Err(
                CoreIntegerErrorView {
                    kind: CoreIntegerErrorKind::NotInteger,
                    code_point_index: index as u64,
                },
            ),
            Some(digit) => {
                let multiplied = core_magnitude_mul_add_spec(
                    limbs,
                    core_integer_multiplier_spec(base),
                    digit,
                );
                let next = trim_core_magnitude_spec(multiplied, multiplied.len() as nat);
                if next.len() > limb_limit {
                    Err(
                        CoreIntegerErrorView {
                            kind: CoreIntegerErrorKind::MagnitudeLimitExceeded,
                            code_point_index: index as u64,
                        },
                    )
                } else {
                    core_integer_digits_spec(
                        input,
                        index + 1,
                        end,
                        base,
                        next,
                        limb_limit,
                        (fuel - 1) as nat,
                    )
                }
            },
        }
    }
}

pub open spec fn core_integer_zero_spec(limbs: Seq<u32>) -> bool {
    limbs.len() == 1 && limbs[0] == 0
}

pub open spec fn convert_core_integer_spec(
    input: Seq<u32>,
    limits: CoreIntegerLimitsView,
) -> Result<CoreIntegerView, CoreIntegerErrorView> {
    let scalar_limits = CoreScalarLimitsView { max_code_points: limits.max_code_points };
    match crate::resolve::classify_core_plain_scalar_spec(input, scalar_limits) {
        Err(error) => Err(
            CoreIntegerErrorView {
                kind: match error.kind {
                    CoreScalarErrorKind::InputLimitExceeded => CoreIntegerErrorKind::InputLimitExceeded,
                },
                code_point_index: error.code_point_index,
            },
        ),
        Ok(class) => match class {
            CorePlainScalarClass::Integer { negative, base, digits } => {
                let limb_limit = effective_core_integer_limb_limit_spec(limits);
                if limb_limit == 0 {
                    Err(
                        CoreIntegerErrorView {
                            kind: CoreIntegerErrorKind::MagnitudeLimitExceeded,
                            code_point_index: digits.start,
                        },
                    )
                } else {
                    match core_integer_digits_spec(
                        input,
                        digits.start as int,
                        digits.end as int,
                        base,
                        Seq::empty().push(0u32),
                        limb_limit,
                        (digits.end - digits.start) as nat,
                    ) {
                        Err(error) => Err(error),
                        Ok(limbs) => Ok(
                            CoreIntegerView {
                                negative: negative && !core_integer_zero_spec(limbs),
                                limbs,
                            },
                        ),
                    }
                }
            },
            _ => Err(
                CoreIntegerErrorView {
                    kind: CoreIntegerErrorKind::NotInteger,
                    code_point_index: 0,
                },
            ),
        },
    }
}

fn convert_core_integer_digits(
    input: &[u32],
    start: usize,
    end: usize,
    base: CoreIntegerBase,
    limb_limit: u64,
) -> (result: Result<Vec<u32>, CoreIntegerError>)
    requires
        start < end <= input@.len(),
        limb_limit > 0,
        limb_limit <= MAX_PROFILE1_CORE_INTEGER_LIMBS,
        forall|index: int|
            start <= index < end ==> crate::resolve::core_digit_for_base_spec(input@[index], base),
    ensures
        core_integer_digits_spec(
            input@,
            start as int,
            end as int,
            base,
            Seq::empty().push(0u32),
            limb_limit,
            (end - start) as nat,
        ) == match result {
            Ok(limbs) => Ok(limbs@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(limbs) => {
                limbs@.len() > 0 && limbs@.len() <= limb_limit
                    && core_magnitude_digits_bounded_spec(limbs@) && (limbs@.len() == 1
                    || limbs@[limbs@.len() - 1] != 0)
            },
            Err(_) => true,
        },
{
    let multiplier = core_integer_multiplier(base);
    let mut limbs: Vec<u32> = Vec::new();
    limbs.push(0);
    let ghost original_spec = core_integer_digits_spec(
        input@,
        start as int,
        end as int,
        base,
        Seq::empty().push(0u32),
        limb_limit,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= input@.len(),
            limb_limit > 0,
            limb_limit <= MAX_PROFILE1_CORE_INTEGER_LIMBS,
            2 <= multiplier <= 16,
            multiplier == core_integer_multiplier_spec(base),
            limbs@.len() > 0,
            limbs@.len() <= limb_limit,
            core_magnitude_digits_bounded_spec(limbs@),
            limbs@.len() == 1 || limbs@[limbs@.len() - 1] != 0,
            forall|candidate: int|
                index <= candidate < end ==> crate::resolve::core_digit_for_base_spec(
                    input@[candidate],
                    base,
                ),
            original_spec == core_integer_digits_spec(
                input@,
                index as int,
                end as int,
                base,
                limbs@,
                limb_limit,
                (end - index) as nat,
            ),
            original_spec == core_integer_digits_spec(
                input@,
                start as int,
                end as int,
                base,
                Seq::empty().push(0u32),
                limb_limit,
                (end - start) as nat,
            ),
        decreases end - index,
    {
        let digit = match core_integer_digit_value(input[index], base) {
            Some(digit) => digit,
            None => {
                assert(false) by {
                    assert(crate::resolve::core_digit_for_base_spec(input@[index as int], base));
                    reveal(crate::resolve::core_digit_for_base_spec);
                    reveal(core_integer_digit_value_spec);
                }
                return Err(CoreIntegerError::at(CoreIntegerErrorKind::NotInteger, index as u64));
            },
        };
        assert(digit < multiplier) by {
            reveal(core_integer_digit_value_spec);
            reveal(core_integer_multiplier_spec);
        }
        let mut next = core_magnitude_mul_add(limbs.as_slice(), multiplier, digit);
        trim_core_magnitude(&mut next);
        proof {
            reveal(core_integer_digits_spec);
        }
        if next.len() as u64 > limb_limit {
            let error = CoreIntegerError::at(
                CoreIntegerErrorKind::MagnitudeLimitExceeded,
                index as u64,
            );
            return Err(error);
        }
        limbs = next;
        index += 1;
    }
    proof {
        reveal(core_integer_digits_spec);
    }
    Ok(limbs)
}

/// Convert a YAML 1.2.2 Core integer into a canonical arbitrary-width magnitude.
pub fn convert_core_integer(input: &[u32], limits: CoreIntegerLimits) -> (result: Result<
    CoreInteger,
    CoreIntegerError,
>)
    ensures
        convert_core_integer_spec(input@, limits@) == match result {
            Ok(integer) => Ok(integer@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(integer) => {
                integer@.limbs.len() > 0 && integer@.limbs.len() <= limits@.max_limbs
                    && integer@.limbs.len() <= MAX_PROFILE1_CORE_INTEGER_LIMBS
                    && core_magnitude_digits_bounded_spec(integer@.limbs) && (integer@.limbs.len()
                    == 1 || integer@.limbs[integer@.limbs.len() - 1] != 0) && (
                !core_integer_zero_spec(integer@.limbs) || !integer@.negative)
            },
            Err(_) => true,
        },
{
    let scalar_limits = CoreScalarLimits::new(limits.max_code_points);
    let class = match classify_core_plain_scalar(input, scalar_limits) {
        Ok(class) => class,
        Err(error) => {
            let converted = CoreIntegerError::at(
                match error.kind() {
                    CoreScalarErrorKind::InputLimitExceeded => {
                        CoreIntegerErrorKind::InputLimitExceeded
                    },
                },
                error.code_point_index(),
            );
            proof {
                reveal(convert_core_integer_spec);
            }
            return Err(converted);
        },
    };
    let (negative, base, digits) = match class {
        CorePlainScalarClass::Integer { negative, base, digits } => (negative, base, digits),
        _ => {
            let error = CoreIntegerError::at(CoreIntegerErrorKind::NotInteger, 0);
            proof {
                reveal(convert_core_integer_spec);
            }
            return Err(error);
        },
    };
    let limb_limit = if limits.max_limbs < MAX_PROFILE1_CORE_INTEGER_LIMBS {
        limits.max_limbs
    } else {
        MAX_PROFILE1_CORE_INTEGER_LIMBS
    };
    if limb_limit == 0 {
        let error = CoreIntegerError::at(
            CoreIntegerErrorKind::MagnitudeLimitExceeded,
            digits.start(),
        );
        proof {
            reveal(convert_core_integer_spec);
            reveal(effective_core_integer_limb_limit_spec);
        }
        return Err(error);
    }
    let start = digits.start() as usize;
    let end = digits.end() as usize;
    proof {
        crate::resolve::lemma_classified_core_integer_has_exact_digits(
            input@,
            scalar_limits@,
            negative,
            base,
            digits,
        );
    }
    assert(start < end <= input.len());
    let limbs = match convert_core_integer_digits(input, start, end, base, limb_limit) {
        Ok(limbs) => limbs,
        Err(error) => {
            proof {
                reveal(convert_core_integer_spec);
                reveal(effective_core_integer_limb_limit_spec);
            }
            return Err(error);
        },
    };
    let is_zero = limbs.len() == 1 && limbs[0] == 0;
    let integer = CoreInteger::new(negative && !is_zero, limbs);
    proof {
        reveal(convert_core_integer_spec);
        reveal(effective_core_integer_limb_limit_spec);
        reveal(core_integer_zero_spec);
    }
    Ok(integer)
}

} // verus!
