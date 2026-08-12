//! Verified quoted-scalar boundary and escape validation for Crucible YAML profile 1.
//!
//! This context-sensitive lexer substage authenticates the structural-candidate partition,
//! recognizes single- and double-quoted scalar boundaries, validates double-quoted escapes, and
//! retains exact raw atom and byte ranges. Flow folding and semantic escape decoding remain
//! presentation-to-content work for the completed token/resolution pipeline.
use crate::atom::{
    AtomizedSource, LexicalAtom, LexicalAtomKind, YamlIndicator, MAX_PROFILE1_LEXICAL_ATOMS,
};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
#[allow(unused_imports)]
use crate::layout::LayoutSourceView;
use crate::layout::{analyze_profile1_layout, LayoutSource};
use crate::structural::{
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    scan_profile1_structural_lexemes, StructuralCandidateRole, StructuralLexeme,
    StructuralLexemeSource,
};
#[allow(unused_imports)]
use crate::structural::{StructuralLexemeSourceView, StructuralLexemeView};
use crate::utf8::CRUCIBLE_YAML_PROFILE_VERSION;
use vstd::prelude::*;

verus! {

pub const QUOTED_SCALAR_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_QUOTED_SCALARS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_QUOTED_SCALAR_ATOMS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuotedScalarScanLimits {
    max_scalars: u64,
    max_scalar_atoms: u64,
}

#[verifier::ext_equal]
pub struct QuotedScalarScanLimitsView {
    pub max_scalars: u64,
    pub max_scalar_atoms: u64,
}

impl View for QuotedScalarScanLimits {
    type V = QuotedScalarScanLimitsView;

    closed spec fn view(&self) -> QuotedScalarScanLimitsView {
        QuotedScalarScanLimitsView {
            max_scalars: self.max_scalars,
            max_scalar_atoms: self.max_scalar_atoms,
        }
    }
}

impl QuotedScalarScanLimits {
    pub fn new(max_scalars: u64, max_scalar_atoms: u64) -> (limits: Self)
        ensures
            limits@.max_scalars == max_scalars,
            limits@.max_scalar_atoms == max_scalar_atoms,
    {
        Self { max_scalars, max_scalar_atoms }
    }

    pub fn max_scalars(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_scalars,
    {
        self.max_scalars
    }

    pub fn max_scalar_atoms(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_scalar_atoms,
    {
        self.max_scalar_atoms
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum QuotedScalarErrorKind {
    InputStructuralMismatch,
    ScalarLimitExceeded,
    ScalarAtomLimitExceeded,
    UnterminatedQuotedScalar,
    InvalidEscape,
    InvalidHexDigit,
    InvalidEscapedCodePoint,
    InvalidQuotedCharacter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuotedScalarError {
    kind: QuotedScalarErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct QuotedScalarErrorView {
    pub kind: QuotedScalarErrorKind,
    pub byte_offset: u64,
}

impl View for QuotedScalarError {
    type V = QuotedScalarErrorView;

    closed spec fn view(&self) -> QuotedScalarErrorView {
        QuotedScalarErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl QuotedScalarError {
    fn at(kind: QuotedScalarErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (QuotedScalarErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: QuotedScalarErrorKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn byte_offset(&self) -> (offset: u64)
        ensures
            offset == self@.byte_offset,
    {
        self.byte_offset
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum QuotedScalarStyle {
    Single,
    Double,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// One complete quoted scalar, including both quote delimiters.
///
/// ```compile_fail
/// use crucible_yaml::{QuotedScalar, QuotedScalarStyle};
///
/// let forged = QuotedScalar {
///     style: QuotedScalarStyle::Double,
///     start_line_number: 2,
///     end_line_number: 1,
///     start_atom_index: 9,
///     end_atom_index: 3,
///     byte_start: 9,
///     byte_end: 3,
/// };
/// ```
pub struct QuotedScalar {
    style: QuotedScalarStyle,
    start_line_number: u64,
    end_line_number: u64,
    start_atom_index: u64,
    end_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
}

#[verifier::ext_equal]
pub struct QuotedScalarView {
    pub style: QuotedScalarStyle,
    pub start_line_number: u64,
    pub end_line_number: u64,
    pub start_atom_index: u64,
    pub end_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
}

impl View for QuotedScalar {
    type V = QuotedScalarView;

    closed spec fn view(&self) -> QuotedScalarView {
        QuotedScalarView {
            style: self.style,
            start_line_number: self.start_line_number,
            end_line_number: self.end_line_number,
            start_atom_index: self.start_atom_index,
            end_atom_index: self.end_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
        }
    }
}

impl DeepView for QuotedScalar {
    type V = QuotedScalarView;

    closed spec fn deep_view(&self) -> QuotedScalarView {
        self@
    }
}

impl QuotedScalar {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        self.style == other.style && self.start_line_number == other.start_line_number
            && self.end_line_number == other.end_line_number && self.start_atom_index
            == other.start_atom_index && self.end_atom_index == other.end_atom_index
            && self.byte_start == other.byte_start && self.byte_end == other.byte_end
    }

    pub fn style(&self) -> (style: QuotedScalarStyle)
        ensures
            style == self@.style,
    {
        self.style
    }

    pub fn start_line_number(&self) -> (line: u64)
        ensures
            line == self@.start_line_number,
    {
        self.start_line_number
    }

    pub fn end_line_number(&self) -> (line: u64)
        ensures
            line == self@.end_line_number,
    {
        self.end_line_number
    }

    pub fn start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.start_atom_index,
    {
        self.start_atom_index
    }

    pub fn end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.end_atom_index,
    {
        self.end_atom_index
    }

    pub fn byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.byte_start,
    {
        self.byte_start
    }

    pub fn byte_end(&self) -> (offset: u64)
        ensures
            offset == self@.byte_end,
    {
        self.byte_end
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QuotedScalarSource {
    profile_version: u16,
    input_transformation_version: u16,
    layout_transformation_version: u16,
    structural_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    input_atom_count: u64,
    input_line_count: u64,
    input_structural_lexeme_count: u64,
    scalars: Vec<QuotedScalar>,
}

#[verifier::ext_equal]
pub struct QuotedScalarSourceView {
    pub profile_version: u16,
    pub input_transformation_version: u16,
    pub layout_transformation_version: u16,
    pub structural_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub input_atom_count: u64,
    pub input_line_count: u64,
    pub input_structural_lexeme_count: u64,
    pub scalars: Seq<QuotedScalarView>,
}

/// One quoted scalar has a nonempty, in-bounds half-open atom range, exact byte and line
/// endpoints, and delimiters matching its recorded style.
pub open spec fn quoted_scalar_range_spec(
    atoms: Seq<LexicalAtomView>,
    scalar: QuotedScalarView,
) -> bool {
    scalar.start_atom_index < scalar.end_atom_index && scalar.end_atom_index <= atoms.len()
        && scalar.byte_start == atoms[scalar.start_atom_index as int].span.start.byte_offset
        && scalar.byte_end == atoms[(scalar.end_atom_index - 1) as int].span.end.byte_offset
        && scalar.start_line_number == atoms[scalar.start_atom_index as int].span.start.line
        && scalar.end_line_number == atoms[(scalar.end_atom_index - 1) as int].span.start.line
        && atoms[scalar.start_atom_index as int].code_point == if scalar.style
        == QuotedScalarStyle::Single {
        0x27u32
    } else {
        0x22u32
    } && atoms[(scalar.end_atom_index - 1) as int].code_point == if scalar.style
        == QuotedScalarStyle::Single {
        0x27u32
    } else {
        0x22u32
    }
}

/// Every scalar range is exact and adjacent output ranges are ordered and non-overlapping.
pub open spec fn quoted_scalar_sequence_ranges_spec(
    atoms: Seq<LexicalAtomView>,
    scalars: Seq<QuotedScalarView>,
) -> bool {
    forall|index: int|
        0 <= index < scalars.len() ==> quoted_scalar_range_spec(atoms, #[trigger] scalars[index])
            && (index > 0 ==> scalars[index - 1].end_atom_index <= scalars[index].start_atom_index
            && scalars[index - 1].byte_end <= scalars[index].byte_start)
}

/// Public lossless range contract exposed to every downstream YAML phase.
pub open spec fn quoted_scalar_ranges_well_formed_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
) -> bool {
    quoted.profile_version == CRUCIBLE_YAML_PROFILE_VERSION && quoted.input_transformation_version
        == atomized.transformation_version && quoted.transformation_version
        == QUOTED_SCALAR_TRANSFORMATION_VERSION && quoted.source_len_bytes
        == atomized.source_len_bytes && quoted.bom_bytes == atomized.bom_bytes
        && quoted.input_atom_count == atomized.atoms.len() && quoted_scalar_sequence_ranges_spec(
        atomized.atoms,
        quoted.scalars,
    )
}

pub open spec fn quoted_scalar_views_spec(scalars: Seq<QuotedScalar>) -> Seq<QuotedScalarView> {
    Seq::new(scalars.len(), |index: int| scalars[index]@)
}

proof fn lemma_quoted_scalar_views_push(scalars: Seq<QuotedScalar>, scalar: QuotedScalar)
    ensures
        quoted_scalar_views_spec(scalars.push(scalar)) == quoted_scalar_views_spec(scalars).push(
            scalar@,
        ),
{
    reveal(quoted_scalar_views_spec);
    assert(quoted_scalar_views_spec(scalars.push(scalar)) =~= quoted_scalar_views_spec(
        scalars,
    ).push(scalar@));
}

proof fn lemma_earlier_atom_ends_before_later_atom_starts(
    atomized: AtomizedSourceView,
    earlier: int,
    later: int,
)
    requires
        crate::atom::atomized_source_intrinsically_well_formed_spec(atomized),
        0 <= earlier < later < atomized.atoms.len(),
    ensures
        atomized.atoms[earlier].span.end.byte_offset
            <= atomized.atoms[later].span.start.byte_offset,
    decreases later - earlier,
{
    crate::atom::lemma_intrinsic_atomized_spans_partition_source(atomized);
    if later == earlier + 1 {
        assert(atomized.atoms[earlier].span.end == atomized.atoms[later].span.start);
    } else {
        lemma_earlier_atom_ends_before_later_atom_starts(atomized, earlier, later - 1);
        crate::atom::lemma_intrinsic_atomized_scalar_is_normalized(atomized, later - 1);
        reveal(crate::utf8::normalized_scalar_view_spec);
        assert(atomized.atoms[later - 1].span.start.byte_offset < atomized.atoms[later
            - 1].span.end.byte_offset);
        assert(atomized.atoms[later - 1].span.end == atomized.atoms[later].span.start);
    }
}

proof fn lemma_quoted_scalar_sequence_ranges_push(
    atoms: Seq<LexicalAtomView>,
    scalars: Seq<QuotedScalarView>,
    scalar: QuotedScalarView,
)
    requires
        quoted_scalar_sequence_ranges_spec(atoms, scalars),
        quoted_scalar_range_spec(atoms, scalar),
        scalars.len() > 0 ==> scalars[scalars.len() - 1].end_atom_index <= scalar.start_atom_index,
        scalars.len() > 0 ==> scalars[scalars.len() - 1].byte_end <= scalar.byte_start,
    ensures
        quoted_scalar_sequence_ranges_spec(atoms, scalars.push(scalar)),
{
    reveal(quoted_scalar_sequence_ranges_spec);
    assert forall|index: int|
        0 <= index < scalars.push(scalar).len() implies quoted_scalar_range_spec(
        atoms,
        #[trigger] scalars.push(scalar)[index],
    ) && (index > 0 ==> scalars.push(scalar)[index - 1].end_atom_index <= scalars.push(
        scalar,
    )[index].start_atom_index && scalars.push(scalar)[index - 1].byte_end <= scalars.push(
        scalar,
    )[index].byte_start) by {
        if index < scalars.len() {
            assert(scalars.push(scalar)[index] == scalars[index]);
            if index > 0 {
                assert(scalars.push(scalar)[index - 1] == scalars[index - 1]);
            }
        } else {
            assert(index == scalars.len());
            assert(scalars.push(scalar)[index] == scalar);
            if index > 0 {
                assert(scalars.push(scalar)[index - 1] == scalars[scalars.len() - 1]);
            }
        }
    }
}

impl View for QuotedScalarSource {
    type V = QuotedScalarSourceView;

    closed spec fn view(&self) -> QuotedScalarSourceView {
        QuotedScalarSourceView {
            profile_version: self.profile_version,
            input_transformation_version: self.input_transformation_version,
            layout_transformation_version: self.layout_transformation_version,
            structural_transformation_version: self.structural_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            input_atom_count: self.input_atom_count,
            input_line_count: self.input_line_count,
            input_structural_lexeme_count: self.input_structural_lexeme_count,
            scalars: quoted_scalar_views_spec(self.scalars@),
        }
    }
}

impl QuotedScalarSource {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.profile_version != other.profile_version || self.input_transformation_version
            != other.input_transformation_version || self.layout_transformation_version
            != other.layout_transformation_version || self.structural_transformation_version
            != other.structural_transformation_version || self.transformation_version
            != other.transformation_version || self.source_len_bytes != other.source_len_bytes
            || self.bom_bytes != other.bom_bytes || self.input_atom_count != other.input_atom_count
            || self.input_line_count != other.input_line_count || self.input_structural_lexeme_count
            != other.input_structural_lexeme_count {
            assert(self@ != other@);
            return false;
        }
        if self.scalars.len() != other.scalars.len() {
            proof {
                reveal(quoted_scalar_views_spec);
                assert(self@.scalars.len() != other@.scalars.len());
                assert(self@ != other@);
            }
            return false;
        }
        let mut index: usize = 0;
        while index < self.scalars.len()
            invariant
                self.scalars.len() == other.scalars.len(),
                index <= self.scalars.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < index ==> self.scalars[prior]@ == other.scalars[prior]@,
            decreases self.scalars.len() - index,
        {
            if !self.scalars[index].same_as(&other.scalars[index]) {
                proof {
                    reveal(quoted_scalar_views_spec);
                    assert(self.scalars[index as int]@ != other.scalars[index as int]@);
                    assert(self@.scalars[index as int] != other@.scalars[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(quoted_scalar_views_spec);
            assert(self@.scalars =~= other@.scalars);
        }
        true
    }

    pub fn profile_version(&self) -> (version: u16)
        ensures
            version == self@.profile_version,
    {
        self.profile_version
    }

    pub fn input_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.input_transformation_version,
    {
        self.input_transformation_version
    }

    pub fn layout_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.layout_transformation_version,
    {
        self.layout_transformation_version
    }

    pub fn structural_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.structural_transformation_version,
    {
        self.structural_transformation_version
    }

    pub fn transformation_version(&self) -> (version: u16)
        ensures
            version == self@.transformation_version,
    {
        self.transformation_version
    }

    pub fn source_len_bytes(&self) -> (length: u64)
        ensures
            length == self@.source_len_bytes,
    {
        self.source_len_bytes
    }

    pub fn bom_bytes(&self) -> (length: u64)
        ensures
            length == self@.bom_bytes,
    {
        self.bom_bytes
    }

    pub fn input_atom_count(&self) -> (count: u64)
        ensures
            count == self@.input_atom_count,
    {
        self.input_atom_count
    }

    pub fn input_line_count(&self) -> (count: u64)
        ensures
            count == self@.input_line_count,
    {
        self.input_line_count
    }

    pub fn input_structural_lexeme_count(&self) -> (count: u64)
        ensures
            count == self@.input_structural_lexeme_count,
    {
        self.input_structural_lexeme_count
    }

    pub fn scalars(&self) -> (scalars: &[QuotedScalar])
        ensures
            quoted_scalar_views_spec(scalars@) == self@.scalars,
    {
        self.scalars.as_slice()
    }
}

closed spec fn effective_scalar_limit_spec(limits: QuotedScalarScanLimitsView) -> u64 {
    if limits.max_scalars < MAX_PROFILE1_QUOTED_SCALARS {
        limits.max_scalars
    } else {
        MAX_PROFILE1_QUOTED_SCALARS
    }
}

closed spec fn effective_scalar_atom_limit_spec(limits: QuotedScalarScanLimitsView) -> u64 {
    if limits.max_scalar_atoms < MAX_PROFILE1_QUOTED_SCALAR_ATOMS {
        limits.max_scalar_atoms
    } else {
        MAX_PROFILE1_QUOTED_SCALAR_ATOMS
    }
}

pub open spec fn hex_digit_value_spec(code_point: u32) -> Option<u32> {
    if 0x30 <= code_point <= 0x39 {
        Some((code_point - 0x30) as u32)
    } else if 0x41 <= code_point <= 0x46 {
        Some((code_point - 0x41 + 10) as u32)
    } else if 0x61 <= code_point <= 0x66 {
        Some((code_point - 0x61 + 10) as u32)
    } else {
        None
    }
}

#[allow(clippy::manual_range_contains)]  // Mirrors the arithmetic Verus specification directly.
fn hex_digit_value(code_point: u32) -> (value: Option<u32>)
    ensures
        value == hex_digit_value_spec(code_point),
{
    if 0x30 <= code_point && code_point <= 0x39 {
        Some(code_point - 0x30)
    } else if 0x41 <= code_point && code_point <= 0x46 {
        Some(code_point - 0x41 + 10)
    } else if 0x61 <= code_point && code_point <= 0x66 {
        Some(code_point - 0x61 + 10)
    } else {
        None
    }
}

pub open spec fn simple_double_escape_spec(code_point: u32) -> bool {
    code_point == 0x30 || code_point == 0x61 || code_point == 0x62 || code_point == 0x74
        || code_point == 0x09 || code_point == 0x6e || code_point == 0x76 || code_point == 0x66
        || code_point == 0x72 || code_point == 0x65 || code_point == 0x20 || code_point == 0x22
        || code_point == 0x2f || code_point == 0x5c || code_point == 0x4e || code_point == 0x5f
        || code_point == 0x4c || code_point == 0x50
}

fn simple_double_escape(code_point: u32) -> (simple: bool)
    ensures
        simple == simple_double_escape_spec(code_point),
{
    code_point == 0x30 || code_point == 0x61 || code_point == 0x62 || code_point == 0x74
        || code_point == 0x09 || code_point == 0x6e || code_point == 0x76 || code_point == 0x66
        || code_point == 0x72 || code_point == 0x65 || code_point == 0x20 || code_point == 0x22
        || code_point == 0x2f || code_point == 0x5c || code_point == 0x4e || code_point == 0x5f
        || code_point == 0x4c || code_point == 0x50
}

pub open spec fn escaped_unicode_scalar_spec(code_point: u32) -> bool {
    code_point <= 0x10ffff && !(0xd800 <= code_point <= 0xdfff)
}

pub open spec fn yaml_printable_character_spec(code_point: u32) -> bool {
    code_point == 0x09 || code_point == 0x0a || (0x20 <= code_point <= 0x7e) || code_point == 0x85
        || (0xa0 <= code_point <= 0xd7ff) || (0xe000 <= code_point <= 0xfffd) || (0x10000
        <= code_point <= 0x10ffff)
}

#[allow(clippy::manual_range_contains)]  // Mirrors the arithmetic Verus specification directly.
fn yaml_printable_character(code_point: u32) -> (printable: bool)
    ensures
        printable == yaml_printable_character_spec(code_point),
{
    code_point == 0x09 || code_point == 0x0a || (0x20 <= code_point && code_point <= 0x7e)
        || code_point == 0x85 || (0xa0 <= code_point && code_point <= 0xd7ff) || (0xe000
        <= code_point && code_point <= 0xfffd) || (0x10000 <= code_point && code_point <= 0x10ffff)
}

#[allow(clippy::manual_range_contains)]  // Mirrors the arithmetic Verus specification directly.
fn escaped_unicode_scalar(code_point: u32) -> (valid: bool)
    ensures
        valid == escaped_unicode_scalar_spec(code_point),
{
    code_point <= 0x10ffff && !(0xd800 <= code_point && code_point <= 0xdfff)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
enum DoubleEscapeState {
    Normal,
    AfterSlash { slash_byte_offset: u64 },
    Hex { remaining: u8, value: u32, slash_byte_offset: u64 },
}

#[verifier::ext_equal]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct DoubleEscapeStateView {
    tag: u8,
    remaining: u8,
    value: u32,
    slash_byte_offset: u64,
}

impl View for DoubleEscapeState {
    type V = DoubleEscapeStateView;

    closed spec fn view(&self) -> DoubleEscapeStateView {
        match self {
            DoubleEscapeState::Normal => DoubleEscapeStateView {
                tag: 0,
                remaining: 0,
                value: 0,
                slash_byte_offset: 0,
            },
            DoubleEscapeState::AfterSlash { slash_byte_offset } => DoubleEscapeStateView {
                tag: 1,
                remaining: 0,
                value: 0,
                slash_byte_offset: *slash_byte_offset,
            },
            DoubleEscapeState::Hex { remaining, value, slash_byte_offset } => {
                DoubleEscapeStateView {
                    tag: 2,
                    remaining: *remaining,
                    value: *value,
                    slash_byte_offset: *slash_byte_offset,
                }
            },
        }
    }
}

closed spec fn quote_error_spec(kind: QuotedScalarErrorKind, byte_offset: u64) -> Result<
    QuotedScalarView,
    QuotedScalarErrorView,
> {
    Err(QuotedScalarErrorView { kind, byte_offset })
}

closed spec fn quoted_scalar_body_spec(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    style: QuotedScalarStyle,
    start_atom_index: int,
    start_line_number: u64,
    index: int,
    line_number: u64,
    single_pending_quote: bool,
    double_escape: DoubleEscapeStateView,
    scalar_atom_limit: u64,
    fuel: nat,
) -> Result<QuotedScalarView, QuotedScalarErrorView>
    decreases fuel,
{
    if style == QuotedScalarStyle::Single && single_pending_quote && (index >= atoms.len()
        || atoms[index].code_point != 0x27) {
        Ok(
            QuotedScalarView {
                style,
                start_line_number: atoms[start_atom_index].span.start.line,
                end_line_number: atoms[index - 1].span.start.line,
                start_atom_index: start_atom_index as u64,
                end_atom_index: index as u64,
                byte_start: atoms[start_atom_index].span.start.byte_offset,
                byte_end: atoms[index - 1].span.end.byte_offset,
            },
        )
    } else if index >= atoms.len() {
        if style == QuotedScalarStyle::Double && double_escape.tag == 1 {
            quote_error_spec(QuotedScalarErrorKind::InvalidEscape, source_len_bytes)
        } else if style == QuotedScalarStyle::Double && double_escape.tag == 2 {
            quote_error_spec(QuotedScalarErrorKind::InvalidHexDigit, source_len_bytes)
        } else {
            quote_error_spec(QuotedScalarErrorKind::UnterminatedQuotedScalar, source_len_bytes)
        }
    } else if fuel == 0 {
        quote_error_spec(
            QuotedScalarErrorKind::InputStructuralMismatch,
            atoms[index].span.start.byte_offset,
        )
    } else if index - start_atom_index >= scalar_atom_limit {
        quote_error_spec(
            QuotedScalarErrorKind::ScalarAtomLimitExceeded,
            atoms[index].span.start.byte_offset,
        )
    } else {
        let atom = atoms[index];
        if !yaml_printable_character_spec(atom.code_point) {
            quote_error_spec(
                QuotedScalarErrorKind::InvalidQuotedCharacter,
                atom.span.start.byte_offset,
            )
        } else if style == QuotedScalarStyle::Single {
            if single_pending_quote {
                quoted_scalar_body_spec(
                    atoms,
                    source_len_bytes,
                    style,
                    start_atom_index,
                    start_line_number,
                    index + 1,
                    line_number,
                    false,
                    double_escape,
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                )
            } else if atom.code_point == 0x27 {
                quoted_scalar_body_spec(
                    atoms,
                    source_len_bytes,
                    style,
                    start_atom_index,
                    start_line_number,
                    index + 1,
                    line_number,
                    true,
                    double_escape,
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                )
            } else {
                quoted_scalar_body_spec(
                    atoms,
                    source_len_bytes,
                    style,
                    start_atom_index,
                    start_line_number,
                    index + 1,
                    if atom.code_point == 0x0a {
                        (line_number + 1) as u64
                    } else {
                        line_number
                    },
                    false,
                    double_escape,
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                )
            }
        } else if double_escape.tag == 0 {
            if atom.code_point == 0x22 {
                Ok(
                    QuotedScalarView {
                        style,
                        start_line_number: atoms[start_atom_index].span.start.line,
                        end_line_number: atom.span.start.line,
                        start_atom_index: start_atom_index as u64,
                        end_atom_index: (index + 1) as u64,
                        byte_start: atoms[start_atom_index].span.start.byte_offset,
                        byte_end: atom.span.end.byte_offset,
                    },
                )
            } else {
                quoted_scalar_body_spec(
                    atoms,
                    source_len_bytes,
                    style,
                    start_atom_index,
                    start_line_number,
                    index + 1,
                    if atom.code_point == 0x0a {
                        (line_number + 1) as u64
                    } else {
                        line_number
                    },
                    false,
                    if atom.code_point == 0x5c {
                        DoubleEscapeStateView {
                            tag: 1,
                            remaining: 0,
                            value: 0,
                            slash_byte_offset: atom.span.start.byte_offset,
                        }
                    } else {
                        double_escape
                    },
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                )
            }
        } else if double_escape.tag == 1 {
            if atom.code_point == 0x78 || atom.code_point == 0x75 || atom.code_point == 0x55 {
                quoted_scalar_body_spec(
                    atoms,
                    source_len_bytes,
                    style,
                    start_atom_index,
                    start_line_number,
                    index + 1,
                    line_number,
                    false,
                    DoubleEscapeStateView {
                        tag: 2,
                        remaining: if atom.code_point == 0x78 {
                            2
                        } else if atom.code_point == 0x75 {
                            4
                        } else {
                            8
                        },
                        value: 0,
                        slash_byte_offset: double_escape.slash_byte_offset,
                    },
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                )
            } else if simple_double_escape_spec(atom.code_point) || atom.code_point == 0x0a {
                quoted_scalar_body_spec(
                    atoms,
                    source_len_bytes,
                    style,
                    start_atom_index,
                    start_line_number,
                    index + 1,
                    if atom.code_point == 0x0a {
                        (line_number + 1) as u64
                    } else {
                        line_number
                    },
                    false,
                    DoubleEscapeStateView { tag: 0, remaining: 0, value: 0, slash_byte_offset: 0 },
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                )
            } else {
                quote_error_spec(QuotedScalarErrorKind::InvalidEscape, atom.span.start.byte_offset)
            }
        } else {
            match hex_digit_value_spec(atom.code_point) {
                None => quote_error_spec(
                    QuotedScalarErrorKind::InvalidHexDigit,
                    atom.span.start.byte_offset,
                ),
                Some(digit) => {
                    let value = (double_escape.value * 16 + digit) as u32;
                    if double_escape.remaining == 1 && !escaped_unicode_scalar_spec(value) {
                        quote_error_spec(
                            QuotedScalarErrorKind::InvalidEscapedCodePoint,
                            double_escape.slash_byte_offset,
                        )
                    } else {
                        quoted_scalar_body_spec(
                            atoms,
                            source_len_bytes,
                            style,
                            start_atom_index,
                            start_line_number,
                            index + 1,
                            line_number,
                            false,
                            if double_escape.remaining == 1 {
                                DoubleEscapeStateView {
                                    tag: 0,
                                    remaining: 0,
                                    value: 0,
                                    slash_byte_offset: 0,
                                }
                            } else {
                                DoubleEscapeStateView {
                                    tag: 2,
                                    remaining: (double_escape.remaining - 1) as u8,
                                    value,
                                    slash_byte_offset: double_escape.slash_byte_offset,
                                }
                            },
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    }
                },
            }
        }
    }
}

pub open spec fn quote_style_for_candidate_spec(candidate: StructuralLexemeView) -> Option<
    QuotedScalarStyle,
> {
    if candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::SingleQuotedScalar) {
        Some(QuotedScalarStyle::Single)
    } else if candidate.kind == StructuralCandidateRole::Indicator(
        YamlIndicator::DoubleQuotedScalar,
    ) {
        Some(QuotedScalarStyle::Double)
    } else {
        None
    }
}

#[verifier::ext_equal]
#[derive(Clone, Copy)]
#[allow(dead_code)]  // Fields are consumed by Verus specifications, not ordinary Rust code.
struct QuotedContextView {
    flow_depth: u64,
    line_indentation: u64,
    at_line_start: bool,
    plain_active: bool,
    plain_pending_line: bool,
    plain_parent_indentation: u64,
    block_mode: u8,
    block_parent_indentation: u64,
    block_content_indentation: u64,
    block_line_active: bool,
    after_node: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QuotedContext {
    flow_depth: u64,
    line_indentation: u64,
    at_line_start: bool,
    plain_active: bool,
    plain_pending_line: bool,
    plain_parent_indentation: u64,
    block_mode: u8,
    block_parent_indentation: u64,
    block_content_indentation: u64,
    block_line_active: bool,
    after_node: bool,
}

impl View for QuotedContext {
    type V = QuotedContextView;

    closed spec fn view(&self) -> QuotedContextView {
        QuotedContextView {
            flow_depth: self.flow_depth,
            line_indentation: self.line_indentation,
            at_line_start: self.at_line_start,
            plain_active: self.plain_active,
            plain_pending_line: self.plain_pending_line,
            plain_parent_indentation: self.plain_parent_indentation,
            block_mode: self.block_mode,
            block_parent_indentation: self.block_parent_indentation,
            block_content_indentation: self.block_content_indentation,
            block_line_active: self.block_line_active,
            after_node: self.after_node,
        }
    }
}

closed spec fn initial_quoted_context_spec() -> QuotedContextView {
    QuotedContextView {
        flow_depth: 0,
        line_indentation: 0,
        at_line_start: true,
        plain_active: false,
        plain_pending_line: false,
        plain_parent_indentation: 0,
        block_mode: 0,
        block_parent_indentation: 0,
        block_content_indentation: 0,
        block_line_active: false,
        after_node: false,
    }
}

fn initial_quoted_context() -> (context: QuotedContext)
    ensures
        context@ == initial_quoted_context_spec(),
{
    QuotedContext {
        flow_depth: 0,
        line_indentation: 0,
        at_line_start: true,
        plain_active: false,
        plain_pending_line: false,
        plain_parent_indentation: 0,
        block_mode: 0,
        block_parent_indentation: 0,
        block_content_indentation: 0,
        block_line_active: false,
        after_node: false,
    }
}

closed spec fn candidate_is_indentation_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::Indentation
}

closed spec fn candidate_is_line_feed_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::LineFeed
}

closed spec fn prepare_quoted_context_spec(
    candidate: StructuralLexemeView,
    context: QuotedContextView,
) -> QuotedContextView {
    if candidate_is_indentation_spec(candidate) || candidate_is_line_feed_spec(candidate) {
        context
    } else {
        let block_prepared = if context.block_mode == 2 && !context.block_line_active
            && context.at_line_start {
            let meets_parent = context.line_indentation > context.block_parent_indentation;
            let meets_content = context.block_content_indentation == 0 || context.line_indentation
                >= context.block_content_indentation;
            if meets_parent && meets_content {
                QuotedContextView {
                    block_content_indentation: if context.block_content_indentation == 0 {
                        context.line_indentation
                    } else {
                        context.block_content_indentation
                    },
                    block_line_active: true,
                    at_line_start: false,
                    ..context
                }
            } else {
                QuotedContextView {
                    block_mode: 0,
                    block_content_indentation: 0,
                    block_line_active: false,
                    ..context
                }
            }
        } else {
            context
        };
        if block_prepared.block_mode == 0 && block_prepared.plain_pending_line {
            if block_prepared.line_indentation > block_prepared.plain_parent_indentation {
                QuotedContextView {
                    plain_active: true,
                    plain_pending_line: false,
                    at_line_start: false,
                    ..block_prepared
                }
            } else {
                QuotedContextView {
                    plain_active: false,
                    plain_pending_line: false,
                    ..block_prepared
                }
            }
        } else {
            block_prepared
        }
    }
}

fn prepare_quoted_context(candidate: &StructuralLexeme, context: QuotedContext) -> (prepared:
    QuotedContext)
    ensures
        prepared@ == prepare_quoted_context_spec(candidate@, context@),
{
    let role = candidate.candidate_role();
    if role == StructuralCandidateRole::Indentation || role == StructuralCandidateRole::LineFeed {
        return context;
    }
    let mut prepared = context;
    if prepared.block_mode == 2 && !prepared.block_line_active && prepared.at_line_start {
        let meets_parent = prepared.line_indentation > prepared.block_parent_indentation;
        let meets_content = prepared.block_content_indentation == 0 || prepared.line_indentation
            >= prepared.block_content_indentation;
        if meets_parent && meets_content {
            if prepared.block_content_indentation == 0 {
                prepared.block_content_indentation = prepared.line_indentation;
            }
            prepared.block_line_active = true;
            prepared.at_line_start = false;
        } else {
            prepared.block_mode = 0;
            prepared.block_content_indentation = 0;
            prepared.block_line_active = false;
        }
    }
    if prepared.block_mode == 0 && prepared.plain_pending_line {
        if prepared.line_indentation > prepared.plain_parent_indentation {
            prepared.plain_active = true;
            prepared.at_line_start = false;
        } else {
            prepared.plain_active = false;
        }
        prepared.plain_pending_line = false;
    }
    prepared
}

closed spec fn quote_can_start_spec(
    atoms: Seq<LexicalAtomView>,
    start_atom_index: u64,
    context: QuotedContextView,
) -> bool {
    !context.plain_active && !context.plain_pending_line && context.block_mode == 0
        && start_atom_index < atoms.len() && (start_atom_index == 0 || {
        let previous = atoms[start_atom_index as int - 1];
        previous.kind == LexicalAtomKind::LineFeed || previous.kind == LexicalAtomKind::Space
            || previous.kind == LexicalAtomKind::Tab || previous.kind == LexicalAtomKind::Indicator(
            YamlIndicator::FlowSequenceStart,
        ) || previous.kind == LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart)
            || previous.kind == LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) || (
        context.flow_depth > 0 && previous.kind == LexicalAtomKind::Indicator(
            YamlIndicator::MappingValue,
        ))
    })
}

closed spec fn context_after_quoted_scalar_spec(context: QuotedContextView) -> QuotedContextView {
    QuotedContextView {
        plain_active: false,
        plain_pending_line: false,
        block_mode: 0,
        block_line_active: false,
        at_line_start: false,
        after_node: true,
        ..context
    }
}

fn context_after_quoted_scalar(context: QuotedContext) -> (next: QuotedContext)
    ensures
        next@ == context_after_quoted_scalar_spec(context@),
{
    let mut next = context;
    next.plain_active = false;
    next.plain_pending_line = false;
    next.block_mode = 0;
    next.block_line_active = false;
    next.at_line_start = false;
    next.after_node = true;
    next
}

closed spec fn candidate_is_flow_start_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::FlowSequenceStart || candidate.kind
        == StructuralCandidateRole::FlowMappingStart
}

closed spec fn candidate_is_flow_end_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::FlowSequenceEnd || candidate.kind
        == StructuralCandidateRole::FlowMappingEnd
}

closed spec fn candidate_is_block_scalar_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar)
        || candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar)
}

closed spec fn candidate_is_quote_spec(candidate: StructuralLexemeView) -> bool {
    quote_style_for_candidate_spec(candidate).is_some()
}

closed spec fn advance_quoted_context_spec(
    atoms: Seq<LexicalAtomView>,
    candidate: StructuralLexemeView,
    context: QuotedContextView,
) -> QuotedContextView {
    if candidate_is_line_feed_spec(candidate) {
        QuotedContextView {
            line_indentation: 0,
            at_line_start: true,
            plain_pending_line: context.plain_active || context.plain_pending_line,
            block_mode: if context.block_mode == 1 {
                2
            } else {
                context.block_mode
            },
            block_line_active: false,
            ..context
        }
    } else if candidate_is_indentation_spec(candidate) {
        QuotedContextView {
            line_indentation: if candidate.start_atom_index <= candidate.end_atom_index {
                (candidate.end_atom_index - candidate.start_atom_index) as u64
            } else {
                0
            },
            at_line_start: true,
            ..context
        }
    } else if context.block_mode == 1 {
        QuotedContextView { at_line_start: false, ..context }
    } else if context.block_mode == 2 && context.block_line_active {
        QuotedContextView { at_line_start: false, ..context }
    } else if context.plain_active {
        if candidate.kind == StructuralCandidateRole::Comment {
            QuotedContextView {
                plain_active: false,
                plain_pending_line: false,
                after_node: true,
                at_line_start: false,
                ..context
            }
        } else if candidate.kind == StructuralCandidateRole::Indicator(
            YamlIndicator::MappingValue,
        ) {
            QuotedContextView {
                plain_active: false,
                plain_pending_line: false,
                after_node: false,
                at_line_start: false,
                ..context
            }
        } else if context.flow_depth > 0 && candidate_is_flow_start_spec(candidate) {
            QuotedContextView {
                flow_depth: if context.flow_depth < MAX_PROFILE1_LEXICAL_ATOMS {
                    (context.flow_depth + 1) as u64
                } else {
                    context.flow_depth
                },
                plain_active: false,
                plain_pending_line: false,
                after_node: false,
                at_line_start: false,
                ..context
            }
        } else if context.flow_depth > 0 && candidate.kind == StructuralCandidateRole::FlowEntry {
            QuotedContextView {
                plain_active: false,
                plain_pending_line: false,
                after_node: false,
                at_line_start: false,
                ..context
            }
        } else if context.flow_depth > 0 && candidate_is_flow_end_spec(candidate) {
            QuotedContextView {
                flow_depth: (context.flow_depth - 1) as u64,
                plain_active: false,
                plain_pending_line: false,
                after_node: true,
                at_line_start: false,
                ..context
            }
        } else {
            QuotedContextView { at_line_start: false, after_node: false, ..context }
        }
    } else if candidate_is_block_scalar_spec(candidate) {
        QuotedContextView {
            block_mode: 1,
            block_parent_indentation: context.line_indentation,
            block_content_indentation: 0,
            block_line_active: false,
            at_line_start: false,
            after_node: true,
            ..context
        }
    } else if candidate_is_flow_start_spec(candidate) {
        QuotedContextView {
            flow_depth: if context.flow_depth < MAX_PROFILE1_LEXICAL_ATOMS {
                (context.flow_depth + 1) as u64
            } else {
                context.flow_depth
            },
            at_line_start: false,
            after_node: false,
            ..context
        }
    } else if context.flow_depth > 0 && candidate.kind == StructuralCandidateRole::FlowEntry {
        QuotedContextView { at_line_start: false, after_node: false, ..context }
    } else if context.flow_depth > 0 && candidate_is_flow_end_spec(candidate) {
        QuotedContextView {
            flow_depth: (context.flow_depth - 1) as u64,
            at_line_start: false,
            after_node: true,
            ..context
        }
    } else if candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::MappingValue) || (
    candidate.kind == StructuralCandidateRole::Content && context.flow_depth > 0
        && context.after_node && candidate.start_atom_index < atoms.len()
        && candidate.end_atom_index == candidate.start_atom_index + 1
        && atoms[candidate.start_atom_index as int].code_point == 0x3a) {
        QuotedContextView { at_line_start: false, after_node: false, ..context }
    } else if candidate.kind == StructuralCandidateRole::Content || candidate_is_quote_spec(
        candidate,
    ) {
        QuotedContextView {
            plain_active: true,
            plain_pending_line: false,
            plain_parent_indentation: context.line_indentation,
            at_line_start: false,
            after_node: false,
            ..context
        }
    } else {
        QuotedContextView { at_line_start: false, ..context }
    }
}

fn advance_quoted_context(
    atoms: &[LexicalAtom],
    candidate: &StructuralLexeme,
    context: QuotedContext,
) -> (next: QuotedContext)
    requires
        candidate@.start_atom_index < candidate@.end_atom_index,
        candidate@.end_atom_index <= atoms@.len(),
        context@.flow_depth <= MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        next@ == advance_quoted_context_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            candidate@,
            context@,
        ),
        next@.flow_depth <= MAX_PROFILE1_LEXICAL_ATOMS,
{
    let role = candidate.candidate_role();
    let mut next = context;
    if role == StructuralCandidateRole::LineFeed {
        next.line_indentation = 0;
        next.at_line_start = true;
        next.plain_pending_line = next.plain_active || next.plain_pending_line;
        if next.block_mode == 1 {
            next.block_mode = 2;
        }
        next.block_line_active = false;
        return next;
    }
    if role == StructuralCandidateRole::Indentation {
        next.line_indentation = candidate.end_atom_index() - candidate.start_atom_index();
        next.at_line_start = true;
        return next;
    }
    if next.block_mode == 1 || (next.block_mode == 2 && next.block_line_active) {
        next.at_line_start = false;
        return next;
    }
    if next.plain_active {
        if role == StructuralCandidateRole::Comment {
            next.plain_active = false;
            next.plain_pending_line = false;
            next.after_node = true;
        } else if role == StructuralCandidateRole::Indicator(YamlIndicator::MappingValue) {
            next.plain_active = false;
            next.plain_pending_line = false;
            next.after_node = false;
        } else if next.flow_depth > 0 && (role == StructuralCandidateRole::FlowSequenceStart || role
            == StructuralCandidateRole::FlowMappingStart) {
            if next.flow_depth < MAX_PROFILE1_LEXICAL_ATOMS {
                next.flow_depth += 1;
            }
            next.plain_active = false;
            next.plain_pending_line = false;
            next.after_node = false;
        } else if next.flow_depth > 0 && role == StructuralCandidateRole::FlowEntry {
            next.plain_active = false;
            next.plain_pending_line = false;
            next.after_node = false;
        } else if next.flow_depth > 0 && (role == StructuralCandidateRole::FlowSequenceEnd || role
            == StructuralCandidateRole::FlowMappingEnd) {
            next.flow_depth -= 1;
            next.plain_active = false;
            next.plain_pending_line = false;
            next.after_node = true;
        } else {
            next.after_node = false;
        }
        next.at_line_start = false;
        return next;
    }
    if role == StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar) || role
        == StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar) {
        next.block_mode = 1;
        next.block_parent_indentation = next.line_indentation;
        next.block_content_indentation = 0;
        next.block_line_active = false;
        next.after_node = true;
    } else if role == StructuralCandidateRole::FlowSequenceStart || role
        == StructuralCandidateRole::FlowMappingStart {
        if next.flow_depth < MAX_PROFILE1_LEXICAL_ATOMS {
            next.flow_depth += 1;
        }
        next.after_node = false;
    } else if next.flow_depth > 0 && role == StructuralCandidateRole::FlowEntry {
        next.after_node = false;
    } else if next.flow_depth > 0 && (role == StructuralCandidateRole::FlowSequenceEnd || role
        == StructuralCandidateRole::FlowMappingEnd) {
        next.flow_depth -= 1;
        next.after_node = true;
    } else if role == StructuralCandidateRole::Indicator(YamlIndicator::MappingValue) || (role
        == StructuralCandidateRole::Content && next.flow_depth > 0 && next.after_node
        && candidate.end_atom_index() == candidate.start_atom_index() + 1
        && atoms[candidate.start_atom_index() as usize].code_point() == 0x3a) {
        next.after_node = false;
    } else if role == StructuralCandidateRole::Content
        || matches!(
        role,
        StructuralCandidateRole::Indicator(YamlIndicator::SingleQuotedScalar)
            | StructuralCandidateRole::Indicator(YamlIndicator::DoubleQuotedScalar)
    ) {
        next.plain_active = true;
        next.plain_pending_line = false;
        next.plain_parent_indentation = next.line_indentation;
        next.after_node = false;
    }
    next.at_line_start = false;
    next
}

fn make_quoted_scalar(
    atoms: &[LexicalAtom],
    style: QuotedScalarStyle,
    start_atom_index: usize,
    end_atom_index: usize,
) -> (scalar: QuotedScalar)
    requires
        start_atom_index < end_atom_index <= atoms@.len(),
        atoms[start_atom_index as int]@.code_point == if style == QuotedScalarStyle::Single {
            0x27u32
        } else {
            0x22u32
        },
        atoms[(end_atom_index - 1) as int]@.code_point == if style == QuotedScalarStyle::Single {
            0x27u32
        } else {
            0x22u32
        },
    ensures
        scalar@ == (QuotedScalarView {
            style,
            start_line_number: atoms[start_atom_index as int]@.span.start.line,
            end_line_number: atoms[(end_atom_index - 1) as int]@.span.start.line,
            start_atom_index: start_atom_index as u64,
            end_atom_index: end_atom_index as u64,
            byte_start: atoms[start_atom_index as int]@.span.start.byte_offset,
            byte_end: atoms[(end_atom_index - 1) as int]@.span.end.byte_offset,
        }),
        quoted_scalar_range_spec(crate::atom::lexical_atom_views_spec(atoms@), scalar@),
{
    QuotedScalar {
        style,
        start_line_number: atoms[start_atom_index].span().start().line(),
        end_line_number: atoms[end_atom_index - 1].span().start().line(),
        start_atom_index: start_atom_index as u64,
        end_atom_index: end_atom_index as u64,
        byte_start: atoms[start_atom_index].span().start().byte_offset(),
        byte_end: atoms[end_atom_index - 1].span().end().byte_offset(),
    }
}

#[verifier::rlimit(200)]
#[allow(clippy::if_same_then_else, unused_assignments, unused_variables)]
// The line counter is consumed by the pure-machine correspondence and loop invariants.
fn scan_quoted_scalar_body(
    atoms: &[LexicalAtom],
    source_len_bytes: u64,
    style: QuotedScalarStyle,
    start_atom_index: usize,
    start_line_number: u64,
    scalar_atom_limit: u64,
) -> (result: Result<QuotedScalar, QuotedScalarError>)
    requires
        start_atom_index < atoms@.len(),
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        start_line_number <= MAX_PROFILE1_LEXICAL_ATOMS,
        scalar_atom_limit > 0,
        atoms[start_atom_index as int]@.code_point == if style == QuotedScalarStyle::Single {
            0x27u32
        } else {
            0x22u32
        },
    ensures
        quoted_scalar_body_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            source_len_bytes,
            style,
            start_atom_index as int,
            start_line_number,
            start_atom_index as int + 1,
            start_line_number,
            false,
            DoubleEscapeState::Normal@,
            scalar_atom_limit,
            (atoms@.len() - start_atom_index) as nat,
        ) == match result {
            Ok(scalar) => Ok(scalar@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(scalar) => {
                quoted_scalar_range_spec(crate::atom::lexical_atom_views_spec(atoms@), scalar@)
                    && scalar@.start_atom_index == start_atom_index as u64
            },
            Err(_) => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    assert(MAX_PROFILE1_LEXICAL_ATOMS < usize::MAX);
    let mut index = start_atom_index + 1;
    let mut line_number = start_line_number;
    let mut single_pending_quote = false;
    let mut double_escape = DoubleEscapeState::Normal;
    let ghost expected = quoted_scalar_body_spec(
        views,
        source_len_bytes,
        style,
        start_atom_index as int,
        start_line_number,
        index as int,
        line_number,
        single_pending_quote,
        double_escape@,
        scalar_atom_limit,
        (atoms@.len() - start_atom_index) as nat,
    );
    assert(expected == quoted_scalar_body_spec(
        crate::atom::lexical_atom_views_spec(atoms@),
        source_len_bytes,
        style,
        start_atom_index as int,
        start_line_number,
        start_atom_index as int + 1,
        start_line_number,
        false,
        DoubleEscapeState::Normal@,
        scalar_atom_limit,
        (atoms@.len() - start_atom_index) as nat,
    ));
    while index < atoms.len()
        invariant
            start_atom_index < index <= atoms@.len(),
            atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            start_line_number <= MAX_PROFILE1_LEXICAL_ATOMS,
            views == crate::atom::lexical_atom_views_spec(atoms@),
            views[start_atom_index as int].code_point == if style == QuotedScalarStyle::Single {
                0x27u32
            } else {
                0x22u32
            },
            start_line_number <= line_number,
            line_number - start_line_number <= index - start_atom_index,
            line_number <= start_line_number + index - start_atom_index,
            style == QuotedScalarStyle::Double ==> !single_pending_quote,
            single_pending_quote ==> views[index as int - 1].code_point == 0x27,
            expected == quoted_scalar_body_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                source_len_bytes,
                style,
                start_atom_index as int,
                start_line_number,
                start_atom_index as int + 1,
                start_line_number,
                false,
                DoubleEscapeState::Normal@,
                scalar_atom_limit,
                (atoms@.len() - start_atom_index) as nat,
            ),
            match double_escape {
                DoubleEscapeState::Hex { remaining, value, .. } => {
                    1 <= remaining <= 8 && (if remaining == 8 {
                        value == 0
                    } else if remaining == 7 {
                        value <= 0x0000000f
                    } else if remaining == 6 {
                        value <= 0x000000ff
                    } else if remaining == 5 {
                        value <= 0x00000fff
                    } else if remaining == 4 {
                        value <= 0x0000ffff
                    } else if remaining == 3 {
                        value <= 0x000fffff
                    } else if remaining == 2 {
                        value <= 0x00ffffff
                    } else {
                        value <= 0x0fffffff
                    })
                },
                _ => true,
            },
            expected == quoted_scalar_body_spec(
                views,
                source_len_bytes,
                style,
                start_atom_index as int,
                start_line_number,
                index as int,
                line_number,
                single_pending_quote,
                double_escape@,
                scalar_atom_limit,
                (atoms@.len() - index + 1) as nat,
            ),
        decreases atoms.len() - index,
    {
        let atom = &atoms[index];
        let code_point = atom.code_point();
        assert(views[index as int] == atom@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if style == QuotedScalarStyle::Single && single_pending_quote && code_point != 0x27 {
            let scalar = make_quoted_scalar(atoms, style, start_atom_index, index);
            proof {
                reveal(quoted_scalar_body_spec);
                assert(expected == Ok(scalar@));
            }
            return Ok(scalar);
        }
        if (index - start_atom_index) as u64 >= scalar_atom_limit {
            let error = QuotedScalarError::at(
                QuotedScalarErrorKind::ScalarAtomLimitExceeded,
                atom.span().start().byte_offset(),
            );
            proof {
                reveal(quoted_scalar_body_spec);
                reveal(quote_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if !yaml_printable_character(code_point) {
            let error = QuotedScalarError::at(
                QuotedScalarErrorKind::InvalidQuotedCharacter,
                atom.span().start().byte_offset(),
            );
            proof {
                reveal(quoted_scalar_body_spec);
                reveal(quote_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        if style == QuotedScalarStyle::Single {
            proof {
                reveal(quoted_scalar_body_spec);
            }
            if single_pending_quote {
                single_pending_quote = false;
            } else if code_point == 0x27 {
                single_pending_quote = true;
            } else if code_point == 0x0a {
                assert(line_number < u64::MAX) by {
                    assert(index - start_atom_index < atoms@.len());
                    assert(atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
                    assert(start_line_number <= MAX_PROFILE1_LEXICAL_ATOMS);
                    assert(line_number <= start_line_number + index - start_atom_index);
                    assert(line_number <= 2 * MAX_PROFILE1_LEXICAL_ATOMS);
                    assert(2 * MAX_PROFILE1_LEXICAL_ATOMS < u64::MAX);
                }
                line_number += 1;
            }
            index += 1;
            if single_pending_quote {
                assert(code_point == 0x27);
                assert(views[index as int - 1].code_point == 0x27);
            }
            assert(expected == quoted_scalar_body_spec(
                views,
                source_len_bytes,
                style,
                start_atom_index as int,
                start_line_number,
                index as int,
                line_number,
                single_pending_quote,
                double_escape@,
                scalar_atom_limit,
                (atoms@.len() - index + 1) as nat,
            ));
        } else {
            match double_escape {
                DoubleEscapeState::Normal => {
                    if code_point == 0x22 {
                        let scalar = make_quoted_scalar(atoms, style, start_atom_index, index + 1);
                        proof {
                            reveal(quoted_scalar_body_spec);
                            assert(expected == Ok(scalar@));
                        }
                        return Ok(scalar);
                    }
                    proof {
                        reveal(quoted_scalar_body_spec);
                    }
                    if code_point == 0x5c {
                        double_escape =
                        DoubleEscapeState::AfterSlash {
                            slash_byte_offset: atom.span().start().byte_offset(),
                        };
                    } else if code_point == 0x0a {
                        assert(line_number < u64::MAX) by {
                            assert(index - start_atom_index < atoms@.len());
                            assert(atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
                            assert(start_line_number <= MAX_PROFILE1_LEXICAL_ATOMS);
                            assert(line_number <= start_line_number + index - start_atom_index);
                            assert(line_number <= 2 * MAX_PROFILE1_LEXICAL_ATOMS);
                            assert(2 * MAX_PROFILE1_LEXICAL_ATOMS < u64::MAX);
                        }
                        line_number += 1;
                    }
                    index += 1;
                    assert(expected == quoted_scalar_body_spec(
                        views,
                        source_len_bytes,
                        style,
                        start_atom_index as int,
                        start_line_number,
                        index as int,
                        line_number,
                        single_pending_quote,
                        double_escape@,
                        scalar_atom_limit,
                        (atoms@.len() - index + 1) as nat,
                    ));
                },
                DoubleEscapeState::AfterSlash { slash_byte_offset } => {
                    if code_point == 0x78 || code_point == 0x75 || code_point == 0x55 {
                        proof {
                            reveal(quoted_scalar_body_spec);
                        }
                        double_escape =
                        DoubleEscapeState::Hex {
                            remaining: if code_point == 0x78 {
                                2
                            } else if code_point == 0x75 {
                                4
                            } else {
                                8
                            },
                            value: 0,
                            slash_byte_offset,
                        };
                        index += 1;
                        assert(expected == quoted_scalar_body_spec(
                            views,
                            source_len_bytes,
                            style,
                            start_atom_index as int,
                            start_line_number,
                            index as int,
                            line_number,
                            single_pending_quote,
                            double_escape@,
                            scalar_atom_limit,
                            (atoms@.len() - index + 1) as nat,
                        ));
                    } else if simple_double_escape(code_point) || code_point == 0x0a {
                        proof {
                            reveal(quoted_scalar_body_spec);
                        }
                        if code_point == 0x0a {
                            assert(line_number < u64::MAX) by {
                                assert(index - start_atom_index < atoms@.len());
                                assert(atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
                                assert(start_line_number <= MAX_PROFILE1_LEXICAL_ATOMS);
                                assert(line_number <= start_line_number + index - start_atom_index);
                                assert(line_number <= 2 * MAX_PROFILE1_LEXICAL_ATOMS);
                                assert(2 * MAX_PROFILE1_LEXICAL_ATOMS < u64::MAX);
                            }
                            line_number += 1;
                        }
                        double_escape = DoubleEscapeState::Normal;
                        index += 1;
                        assert(expected == quoted_scalar_body_spec(
                            views,
                            source_len_bytes,
                            style,
                            start_atom_index as int,
                            start_line_number,
                            index as int,
                            line_number,
                            single_pending_quote,
                            double_escape@,
                            scalar_atom_limit,
                            (atoms@.len() - index + 1) as nat,
                        ));
                    } else {
                        let error = QuotedScalarError::at(
                            QuotedScalarErrorKind::InvalidEscape,
                            atom.span().start().byte_offset(),
                        );
                        proof {
                            reveal(quoted_scalar_body_spec);
                            reveal(quote_error_spec);
                            assert(expected == Err(error@));
                        }
                        return Err(error);
                    }
                },
                DoubleEscapeState::Hex { remaining, value, slash_byte_offset } => {
                    let digit = match hex_digit_value(code_point) {
                        Some(digit) => digit,
                        None => {
                            let error = QuotedScalarError::at(
                                QuotedScalarErrorKind::InvalidHexDigit,
                                atom.span().start().byte_offset(),
                            );
                            proof {
                                reveal(quoted_scalar_body_spec);
                                reveal(quote_error_spec);
                                assert(expected == Err(error@));
                            }
                            return Err(error);
                        },
                    };
                    assert(digit <= 15) by {
                        reveal(hex_digit_value_spec);
                    }
                    assert(value <= 0x0fffffff);
                    let expanded = value as u64 * 16 + digit as u64;
                    assert(expanded <= u32::MAX);
                    let next_value = expanded as u32;
                    assert(next_value == (value * 16 + digit) as u32);
                    if remaining == 1 && !escaped_unicode_scalar(next_value) {
                        let error = QuotedScalarError::at(
                            QuotedScalarErrorKind::InvalidEscapedCodePoint,
                            slash_byte_offset,
                        );
                        proof {
                            reveal(quoted_scalar_body_spec);
                            reveal(quote_error_spec);
                            assert(expected == Err(error@));
                        }
                        return Err(error);
                    }
                    proof {
                        reveal(quoted_scalar_body_spec);
                    }
                    if remaining > 1 {
                        if remaining == 8 {
                            assert(next_value <= 0x0000000f);
                        } else if remaining == 7 {
                            assert(next_value <= 0x000000ff);
                        } else if remaining == 6 {
                            assert(next_value <= 0x00000fff);
                        } else if remaining == 5 {
                            assert(next_value <= 0x0000ffff);
                        } else if remaining == 4 {
                            assert(next_value <= 0x000fffff);
                        } else if remaining == 3 {
                            assert(next_value <= 0x00ffffff);
                        } else {
                            assert(remaining == 2);
                            assert(next_value <= 0x0fffffff);
                        }
                    }
                    double_escape =
                    if remaining == 1 {
                        DoubleEscapeState::Normal
                    } else {
                        DoubleEscapeState::Hex {
                            remaining: remaining - 1,
                            value: next_value,
                            slash_byte_offset,
                        }
                    };
                    index += 1;
                    assert(expected == quoted_scalar_body_spec(
                        views,
                        source_len_bytes,
                        style,
                        start_atom_index as int,
                        start_line_number,
                        index as int,
                        line_number,
                        single_pending_quote,
                        double_escape@,
                        scalar_atom_limit,
                        (atoms@.len() - index + 1) as nat,
                    ));
                },
            }
        }
    }
    if style == QuotedScalarStyle::Single && single_pending_quote {
        let scalar = make_quoted_scalar(atoms, style, start_atom_index, index);
        proof {
            reveal(quoted_scalar_body_spec);
            assert(expected == Ok(scalar@));
        }
        Ok(scalar)
    } else {
        let kind = match double_escape {
            DoubleEscapeState::AfterSlash { .. } if style == QuotedScalarStyle::Double => {
                QuotedScalarErrorKind::InvalidEscape
            },
            DoubleEscapeState::Hex { .. } if style == QuotedScalarStyle::Double => {
                QuotedScalarErrorKind::InvalidHexDigit
            },
            _ => QuotedScalarErrorKind::UnterminatedQuotedScalar,
        };
        let error = QuotedScalarError::at(kind, source_len_bytes);
        proof {
            reveal(quoted_scalar_body_spec);
            reveal(quote_error_spec);
            assert(expected == Err(error@));
        }
        Err(error)
    }
}

pub open spec fn candidate_index_after_atom_spec(
    candidates: Seq<StructuralLexemeView>,
    index: int,
    atom_index: u64,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < candidates.len() && fuel > 0 && candidates[index].start_atom_index < atom_index {
        candidate_index_after_atom_spec(candidates, index + 1, atom_index, (fuel - 1) as nat)
    } else {
        index
    }
}

closed spec fn quoted_scalar_scan_tail_spec(
    atoms: Seq<LexicalAtomView>,
    candidates: Seq<StructuralLexemeView>,
    source_len_bytes: u64,
    candidate_index: int,
    context: QuotedContextView,
    built: Seq<QuotedScalarView>,
    scalar_limit: u64,
    scalar_atom_limit: u64,
    fuel: nat,
) -> Result<Seq<QuotedScalarView>, QuotedScalarErrorView>
    decreases fuel,
{
    if candidate_index >= candidates.len() {
        Ok(built)
    } else if fuel == 0 || candidate_index < 0 {
        Err(
            QuotedScalarErrorView {
                kind: QuotedScalarErrorKind::InputStructuralMismatch,
                byte_offset: 0,
            },
        )
    } else {
        let candidate = candidates[candidate_index];
        if candidate.start_atom_index >= candidate.end_atom_index || candidate.end_atom_index
            > atoms.len() || candidate.line_number > MAX_PROFILE1_LEXICAL_ATOMS {
            Err(
                QuotedScalarErrorView {
                    kind: QuotedScalarErrorKind::InputStructuralMismatch,
                    byte_offset: candidate.byte_start,
                },
            )
        } else {
            let prepared = prepare_quoted_context_spec(candidate, context);
            match quote_style_for_candidate_spec(candidate) {
                None => quoted_scalar_scan_tail_spec(
                    atoms,
                    candidates,
                    source_len_bytes,
                    candidate_index + 1,
                    advance_quoted_context_spec(atoms, candidate, prepared),
                    built,
                    scalar_limit,
                    scalar_atom_limit,
                    (fuel - 1) as nat,
                ),
                Some(style) => {
                    if !quote_can_start_spec(atoms, candidate.start_atom_index, prepared) {
                        quoted_scalar_scan_tail_spec(
                            atoms,
                            candidates,
                            source_len_bytes,
                            candidate_index + 1,
                            advance_quoted_context_spec(atoms, candidate, prepared),
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if atoms[candidate.start_atom_index as int].code_point != if style
                        == QuotedScalarStyle::Single {
                        0x27u32
                    } else {
                        0x22u32
                    } {
                        Err(
                            QuotedScalarErrorView {
                                kind: QuotedScalarErrorKind::InputStructuralMismatch,
                                byte_offset: candidate.byte_start,
                            },
                        )
                    } else if built.len() >= scalar_limit {
                        Err(
                            QuotedScalarErrorView {
                                kind: QuotedScalarErrorKind::ScalarLimitExceeded,
                                byte_offset: candidate.byte_start,
                            },
                        )
                    } else if scalar_atom_limit == 0 {
                        Err(
                            QuotedScalarErrorView {
                                kind: QuotedScalarErrorKind::ScalarAtomLimitExceeded,
                                byte_offset: candidate.byte_start,
                            },
                        )
                    } else {
                        match quoted_scalar_body_spec(
                            atoms,
                            source_len_bytes,
                            style,
                            candidate.start_atom_index as int,
                            candidate.line_number,
                            candidate.start_atom_index as int + 1,
                            candidate.line_number,
                            false,
                            DoubleEscapeStateView {
                                tag: 0,
                                remaining: 0,
                                value: 0,
                                slash_byte_offset: 0,
                            },
                            scalar_atom_limit,
                            (atoms.len() - candidate.start_atom_index) as nat,
                        ) {
                            Err(error) => Err(error),
                            Ok(scalar) => {
                                let next = candidate_index_after_atom_spec(
                                    candidates,
                                    candidate_index + 1,
                                    scalar.end_atom_index,
                                    (candidates.len() - candidate_index - 1) as nat,
                                );
                                quoted_scalar_scan_tail_spec(
                                    atoms,
                                    candidates,
                                    source_len_bytes,
                                    next,
                                    context_after_quoted_scalar_spec(prepared),
                                    built.push(scalar),
                                    scalar_limit,
                                    scalar_atom_limit,
                                    (fuel - 1) as nat,
                                )
                            },
                        }
                    }
                },
            }
        }
    }
}

pub open spec fn canonical_quote_layout_limits_spec() -> crate::layout::LayoutLimitsView {
    crate::structural::canonical_layout_limits_spec()
}

pub open spec fn canonical_quote_structural_limits_spec() -> crate::structural::StructuralScanLimitsView {
    crate::structural::canonical_structural_scan_limits_spec()
}

pub open spec fn canonical_quoted_scalar_limits_spec() -> QuotedScalarScanLimitsView {
    QuotedScalarScanLimitsView {
        max_scalars: MAX_PROFILE1_QUOTED_SCALARS,
        max_scalar_atoms: MAX_PROFILE1_QUOTED_SCALAR_ATOMS,
    }
}

/// Returns the absolute quoted-scalar limits used to authenticate downstream input.
pub fn canonical_quoted_scalar_limits() -> (limits: QuotedScalarScanLimits)
    ensures
        limits@ == canonical_quoted_scalar_limits_spec(),
{
    QuotedScalarScanLimits::new(MAX_PROFILE1_QUOTED_SCALARS, MAX_PROFILE1_QUOTED_SCALAR_ATOMS)
}

pub closed spec fn scan_profile1_quoted_scalars_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    limits: QuotedScalarScanLimitsView,
) -> Result<QuotedScalarSourceView, QuotedScalarErrorView> {
    match crate::layout::analyze_profile1_layout_spec(
        atomized,
        canonical_quote_layout_limits_spec(),
    ) {
        Err(error) => Err(
            QuotedScalarErrorView {
                kind: QuotedScalarErrorKind::InputStructuralMismatch,
                byte_offset: error.byte_offset,
            },
        ),
        Ok(canonical_layout) => {
            if canonical_layout != layout {
                Err(
                    QuotedScalarErrorView {
                        kind: QuotedScalarErrorKind::InputStructuralMismatch,
                        byte_offset: atomized.bom_bytes,
                    },
                )
            } else {
                match crate::structural::scan_profile1_structural_lexemes_spec(
                    atomized,
                    layout,
                    canonical_quote_structural_limits_spec(),
                ) {
                    Err(error) => Err(
                        QuotedScalarErrorView {
                            kind: QuotedScalarErrorKind::InputStructuralMismatch,
                            byte_offset: error.byte_offset,
                        },
                    ),
                    Ok(canonical_structural) => {
                        if canonical_structural != structural {
                            Err(
                                QuotedScalarErrorView {
                                    kind: QuotedScalarErrorKind::InputStructuralMismatch,
                                    byte_offset: atomized.bom_bytes,
                                },
                            )
                        } else {
                            match quoted_scalar_scan_tail_spec(
                                atomized.atoms,
                                structural.lexemes,
                                atomized.source_len_bytes,
                                0,
                                initial_quoted_context_spec(),
                                Seq::empty(),
                                effective_scalar_limit_spec(limits),
                                effective_scalar_atom_limit_spec(limits),
                                structural.lexemes.len(),
                            ) {
                                Err(error) => Err(error),
                                Ok(scalars) => Ok(
                                    QuotedScalarSourceView {
                                        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                                        input_transformation_version:
                                            atomized.transformation_version,
                                        layout_transformation_version:
                                            layout.transformation_version,
                                        structural_transformation_version:
                                            structural.transformation_version,
                                        transformation_version:
                                            QUOTED_SCALAR_TRANSFORMATION_VERSION,
                                        source_len_bytes: atomized.source_len_bytes,
                                        bom_bytes: atomized.bom_bytes,
                                        input_atom_count: atomized.atoms.len() as u64,
                                        input_line_count: layout.lines.len() as u64,
                                        input_structural_lexeme_count:
                                            structural.lexemes.len() as u64,
                                        scalars,
                                    },
                                ),
                            }
                        }
                    },
                }
            }
        },
    }
}

pub closed spec fn quoted_scalar_source_corresponds_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
) -> bool {
    exists|limits: QuotedScalarScanLimitsView|
        scan_profile1_quoted_scalars_spec(atomized, layout, structural, limits) == Ok(quoted)
}

pub closed spec fn quoted_scalar_source_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
) -> bool {
    crate::atom::atomized_source_intrinsically_well_formed_spec(atomized)
        && crate::layout::layout_source_well_formed_spec(atomized, layout)
        && crate::structural::structural_lexeme_source_well_formed_spec(
        atomized,
        layout,
        structural,
    ) && quoted_scalar_source_corresponds_spec(atomized, layout, structural, quoted)
        && quoted_scalar_ranges_well_formed_spec(atomized, quoted)
}

/// Semantic quoted-scalar validity exposes exact, ordered, non-overlapping source ranges.
pub proof fn lemma_quoted_well_formed_has_exact_ranges(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
)
    requires
        quoted_scalar_source_well_formed_spec(atomized, layout, structural, quoted),
    ensures
        quoted_scalar_ranges_well_formed_spec(atomized, quoted),
{
    reveal(quoted_scalar_source_well_formed_spec);
}

/// A canonical empty atom and structural stream always admits one exact empty quote source.
pub proof fn lemma_empty_input_fits_quoted_scalar_scan_limits(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    limits: QuotedScalarScanLimitsView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(atomized, canonical_quote_layout_limits_spec())
            == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            canonical_quote_structural_limits_spec(),
        ) == Ok(structural),
        atomized.atoms.len() == 0,
        structural.lexemes.len() == 0,
    ensures
        exists|source: QuotedScalarSourceView|
            scan_profile1_quoted_scalars_spec(atomized, layout, structural, limits) == Ok(source),
{
    let source = QuotedScalarSourceView {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version,
        layout_transformation_version: layout.transformation_version,
        structural_transformation_version: structural.transformation_version,
        transformation_version: QUOTED_SCALAR_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes,
        bom_bytes: atomized.bom_bytes,
        input_atom_count: 0,
        input_line_count: layout.lines.len() as u64,
        input_structural_lexeme_count: 0,
        scalars: Seq::empty(),
    };
    reveal(scan_profile1_quoted_scalars_spec);
    reveal(quoted_scalar_scan_tail_spec);
    assert(scan_profile1_quoted_scalars_spec(atomized, layout, structural, limits) == Ok(source));
}

/// Canonical success on an empty structural stream contains no quoted scalar ranges.
pub proof fn lemma_empty_quoted_scan_has_no_scalars(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    limits: QuotedScalarScanLimitsView,
    quoted: QuotedScalarSourceView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(atomized, canonical_quote_layout_limits_spec())
            == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            canonical_quote_structural_limits_spec(),
        ) == Ok(structural),
        structural.lexemes.len() == 0,
        scan_profile1_quoted_scalars_spec(atomized, layout, structural, limits) == Ok(quoted),
    ensures
        quoted.scalars.len() == 0,
{
    reveal(scan_profile1_quoted_scalars_spec);
    reveal(quoted_scalar_scan_tail_spec);
}

fn quote_style_for_candidate(candidate: &StructuralLexeme) -> (style: Option<QuotedScalarStyle>)
    ensures
        style == quote_style_for_candidate_spec(candidate@),
{
    match candidate.candidate_role() {
        StructuralCandidateRole::Indicator(YamlIndicator::SingleQuotedScalar) => {
            Some(QuotedScalarStyle::Single)
        },
        StructuralCandidateRole::Indicator(YamlIndicator::DoubleQuotedScalar) => {
            Some(QuotedScalarStyle::Double)
        },
        _ => None,
    }
}

fn quote_can_start(
    atoms: &[LexicalAtom],
    start_atom_index: usize,
    context: QuotedContext,
) -> (can_start: bool)
    requires
        start_atom_index < atoms@.len(),
    ensures
        can_start == quote_can_start_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start_atom_index as u64,
            context@,
        ),
{
    if context.plain_active || context.plain_pending_line || context.block_mode != 0 {
        return false;
    }
    if start_atom_index == 0 {
        return true;
    }
    let previous_kind = atoms[start_atom_index - 1].kind();
    previous_kind == LexicalAtomKind::LineFeed || previous_kind == LexicalAtomKind::Space
        || previous_kind == LexicalAtomKind::Tab || previous_kind == LexicalAtomKind::Indicator(
        YamlIndicator::FlowSequenceStart,
    ) || previous_kind == LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart)
        || previous_kind == LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) || (
    context.flow_depth > 0 && previous_kind == LexicalAtomKind::Indicator(
        YamlIndicator::MappingValue,
    ))
}

fn candidate_index_after_atom(
    candidates: &[StructuralLexeme],
    start_index: usize,
    atom_index: u64,
) -> (index: usize)
    requires
        start_index <= candidates@.len(),
    ensures
        index as int == candidate_index_after_atom_spec(
            crate::structural::structural_lexeme_views_spec(candidates@),
            start_index as int,
            atom_index,
            (candidates@.len() - start_index) as nat,
        ),
        start_index <= index <= candidates@.len(),
        index < candidates@.len() ==> atom_index <= candidates[index as int]@.start_atom_index,
{
    let ghost views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost expected = candidate_index_after_atom_spec(
        views,
        start_index as int,
        atom_index,
        (candidates@.len() - start_index) as nat,
    );
    let mut index = start_index;
    while index < candidates.len() && candidates[index].start_atom_index() < atom_index
        invariant
            start_index <= index <= candidates@.len(),
            views == crate::structural::structural_lexeme_views_spec(candidates@),
            expected == candidate_index_after_atom_spec(
                views,
                index as int,
                atom_index,
                (candidates@.len() - index) as nat,
            ),
        decreases candidates.len() - index,
    {
        assert(views[index as int] == candidates[index as int]@) by {
            reveal(crate::structural::structural_lexeme_views_spec);
        }
        proof {
            reveal(candidate_index_after_atom_spec);
        }
        index += 1;
    }
    proof {
        reveal(candidate_index_after_atom_spec);
        if index < candidates.len() {
            assert(views[index as int] == candidates[index as int]@) by {
                reveal(crate::structural::structural_lexeme_views_spec);
            }
        }
    }
    index
}

#[verifier::rlimit(50)]
#[verifier::spinoff_prover]
proof fn lemma_adjacent_structural_candidate_starts_increase(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    index: int,
)
    requires
        crate::structural::structural_lexeme_source_well_formed_spec(atomized, layout, structural),
        0 <= index,
        index + 1 < structural.lexemes.len(),
    ensures
        structural.lexemes[index].start_atom_index < structural.lexemes[index + 1].start_atom_index,
{
    crate::structural::lemma_structural_well_formed_has_exact_partition(
        atomized,
        layout,
        structural,
    );
    reveal(crate::structural::structural_lexeme_partition_spec);
    reveal(crate::structural::structural_candidate_prefix_partition_spec);
    reveal(crate::structural::structural_candidate_range_spec);
    assert(structural.lexemes[index].start_atom_index < structural.lexemes[index].end_atom_index);
    assert(structural.lexemes[index].end_atom_index == structural.lexemes[index
        + 1].start_atom_index);
}

#[verifier::rlimit(240)]
#[verifier::spinoff_prover]
pub fn scan_profile1_quoted_scalars(
    atomized: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
    limits: QuotedScalarScanLimits,
) -> (result: Result<QuotedScalarSource, QuotedScalarError>)
    ensures
        scan_profile1_quoted_scalars_spec(atomized@, layout@, structural@, limits@)
            == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(source) => {
                quoted_scalar_source_corresponds_spec(atomized@, layout@, structural@, source@) && (
                (crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                    && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
                    && crate::structural::structural_lexeme_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                )) ==> quoted_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    source@,
                )) && source@.scalars.len() <= limits@.max_scalars && source@.scalars.len()
                    <= MAX_PROFILE1_QUOTED_SCALARS
            },
            Err(_) => true,
        },
{
    let canonical_layout_limits = canonical_structural_layout_limits();
    let canonical_layout = match analyze_profile1_layout(atomized, canonical_layout_limits) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = QuotedScalarError::at(
                QuotedScalarErrorKind::InputStructuralMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_quoted_scalars_spec);
                reveal(canonical_quote_layout_limits_spec);
                assert(crate::layout::analyze_profile1_layout_spec(
                    atomized@,
                    canonical_quote_layout_limits_spec(),
                ) == Err(error@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_layout.same_as(layout) {
        let mismatch = QuotedScalarError::at(
            QuotedScalarErrorKind::InputStructuralMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_quoted_scalars_spec);
            reveal(canonical_quote_layout_limits_spec);
            assert(canonical_layout@ != layout@);
            assert(crate::layout::analyze_profile1_layout_spec(
                atomized@,
                canonical_quote_layout_limits_spec(),
            ) == Ok(canonical_layout@));
        }
        return Err(mismatch);
    }
    assert(canonical_layout@ == layout@);
    proof {
        reveal(canonical_quote_layout_limits_spec);
        assert(crate::layout::analyze_profile1_layout_spec(
            atomized@,
            canonical_quote_layout_limits_spec(),
        ) == Ok(layout@));
        crate::layout::lemma_layout_success_input_within_atom_cap(
            atomized@,
            canonical_layout_limits@,
            canonical_layout@,
        );
        assert(atomized@.atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
    }
    let canonical_structural_limits = canonical_structural_scan_limits();
    let canonical_structural = match scan_profile1_structural_lexemes(
        atomized,
        layout,
        canonical_structural_limits,
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = QuotedScalarError::at(
                QuotedScalarErrorKind::InputStructuralMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_quoted_scalars_spec);
                reveal(canonical_quote_structural_limits_spec);
                assert(crate::structural::scan_profile1_structural_lexemes_spec(
                    atomized@,
                    layout@,
                    canonical_quote_structural_limits_spec(),
                ) == Err(error@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_structural.same_as(structural) {
        let mismatch = QuotedScalarError::at(
            QuotedScalarErrorKind::InputStructuralMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_quoted_scalars_spec);
            reveal(canonical_quote_structural_limits_spec);
            assert(canonical_structural@ != structural@);
            assert(crate::structural::scan_profile1_structural_lexemes_spec(
                atomized@,
                layout@,
                canonical_quote_structural_limits_spec(),
            ) == Ok(canonical_structural@));
        }
        return Err(mismatch);
    }
    assert(canonical_structural@ == structural@);
    proof {
        reveal(canonical_quote_structural_limits_spec);
        assert(crate::structural::scan_profile1_structural_lexemes_spec(
            atomized@,
            layout@,
            canonical_quote_structural_limits_spec(),
        ) == Ok(structural@));
    }
    let scalar_limit = if limits.max_scalars < MAX_PROFILE1_QUOTED_SCALARS {
        limits.max_scalars
    } else {
        MAX_PROFILE1_QUOTED_SCALARS
    };
    let scalar_atom_limit = if limits.max_scalar_atoms < MAX_PROFILE1_QUOTED_SCALAR_ATOMS {
        limits.max_scalar_atoms
    } else {
        MAX_PROFILE1_QUOTED_SCALAR_ATOMS
    };
    proof {
        reveal(effective_scalar_limit_spec);
        reveal(effective_scalar_atom_limit_spec);
        assert(scalar_limit == effective_scalar_limit_spec(limits@));
        assert(scalar_atom_limit == effective_scalar_atom_limit_spec(limits@));
    }
    let atoms = atomized.atoms();
    let candidates = structural.lexemes();
    let mut scalars: Vec<QuotedScalar> = Vec::new();
    let mut candidate_index: usize = 0;
    let mut context = initial_quoted_context();
    let ghost mut fuel: nat = candidates@.len();
    let ghost candidate_views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost semantic_inputs = crate::structural::structural_lexeme_source_well_formed_spec(
        atomized@,
        layout@,
        structural@,
    );
    let ghost expected = quoted_scalar_scan_tail_spec(
        atom_views,
        candidate_views,
        atomized@.source_len_bytes,
        0,
        initial_quoted_context_spec(),
        Seq::empty(),
        scalar_limit,
        scalar_atom_limit,
        fuel,
    );
    proof {
        reveal(quoted_scalar_views_spec);
        reveal(quoted_scalar_sequence_ranges_spec);
        assert(quoted_scalar_views_spec(scalars@) =~= Seq::<QuotedScalarView>::empty());
        assert(quoted_scalar_sequence_ranges_spec(atom_views, quoted_scalar_views_spec(scalars@)));
        assert(expected == quoted_scalar_scan_tail_spec(
            atomized@.atoms,
            structural@.lexemes,
            atomized@.source_len_bytes,
            0,
            initial_quoted_context_spec(),
            Seq::empty(),
            effective_scalar_limit_spec(limits@),
            effective_scalar_atom_limit_spec(limits@),
            structural@.lexemes.len(),
        ));
    }
    while candidate_index < candidates.len()
        invariant
            candidate_index <= candidates@.len(),
            scalars@.len() <= scalar_limit,
            context@.flow_depth <= MAX_PROFILE1_LEXICAL_ATOMS,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            atom_views == atomized@.atoms,
            atom_views.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            candidate_views == crate::structural::structural_lexeme_views_spec(candidates@),
            candidate_views == structural@.lexemes,
            semantic_inputs == crate::structural::structural_lexeme_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
            ),
            quoted_scalar_views_spec(scalars@).len() == scalars@.len(),
            semantic_inputs ==> quoted_scalar_sequence_ranges_spec(
                atom_views,
                quoted_scalar_views_spec(scalars@),
            ),
            semantic_inputs && scalars@.len() > 0 && candidate_index < candidates@.len()
                ==> quoted_scalar_views_spec(scalars@)[scalars@.len() - 1].end_atom_index
                <= candidate_views[candidate_index as int].start_atom_index,
            crate::layout::analyze_profile1_layout_spec(
                atomized@,
                canonical_quote_layout_limits_spec(),
            ) == Ok(layout@),
            crate::structural::scan_profile1_structural_lexemes_spec(
                atomized@,
                layout@,
                canonical_quote_structural_limits_spec(),
            ) == Ok(structural@),
            scalar_limit == effective_scalar_limit_spec(limits@),
            scalar_atom_limit == effective_scalar_atom_limit_spec(limits@),
            fuel >= candidates@.len() - candidate_index,
            expected == quoted_scalar_scan_tail_spec(
                atomized@.atoms,
                structural@.lexemes,
                atomized@.source_len_bytes,
                0,
                initial_quoted_context_spec(),
                Seq::empty(),
                effective_scalar_limit_spec(limits@),
                effective_scalar_atom_limit_spec(limits@),
                structural@.lexemes.len(),
            ),
            expected == quoted_scalar_scan_tail_spec(
                atom_views,
                candidate_views,
                atomized@.source_len_bytes,
                candidate_index as int,
                context@,
                quoted_scalar_views_spec(scalars@),
                scalar_limit,
                scalar_atom_limit,
                fuel,
            ),
        decreases candidates.len() - candidate_index,
    {
        let candidate = &candidates[candidate_index];
        assert(candidate_views[candidate_index as int] == candidate@) by {
            reveal(crate::structural::structural_lexeme_views_spec);
        }
        let start_atom_index = candidate.start_atom_index();
        let end_atom_index = candidate.end_atom_index();
        if start_atom_index >= end_atom_index || end_atom_index > atoms.len() as u64
            || candidate.line_number() > MAX_PROFILE1_LEXICAL_ATOMS {
            let mismatch = QuotedScalarError::at(
                QuotedScalarErrorKind::InputStructuralMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(quoted_scalar_scan_tail_spec);
                assert(expected == Err(mismatch@));
                reveal(scan_profile1_quoted_scalars_spec);
            }
            return Err(mismatch);
        }
        context = prepare_quoted_context(candidate, context);
        let style = match quote_style_for_candidate(candidate) {
            None => {
                proof {
                    reveal(quoted_scalar_scan_tail_spec);
                    if semantic_inputs && scalars@.len() > 0 && candidate_index + 1
                        < candidates@.len() {
                        lemma_adjacent_structural_candidate_starts_increase(
                            atomized@,
                            layout@,
                            structural@,
                            candidate_index as int,
                        );
                    }
                    fuel = (fuel - 1) as nat;
                }
                context = advance_quoted_context(atoms, candidate, context);
                candidate_index += 1;
                continue;
            },
            Some(style) => style,
        };
        if !quote_can_start(atoms, start_atom_index as usize, context) {
            proof {
                reveal(quoted_scalar_scan_tail_spec);
                if semantic_inputs && scalars@.len() > 0 && candidate_index + 1
                    < candidates@.len() {
                    lemma_adjacent_structural_candidate_starts_increase(
                        atomized@,
                        layout@,
                        structural@,
                        candidate_index as int,
                    );
                }
                fuel = (fuel - 1) as nat;
            }
            context = advance_quoted_context(atoms, candidate, context);
            candidate_index += 1;
            continue;
        }
        let expected_delimiter = if style == QuotedScalarStyle::Single {
            0x27u32
        } else {
            0x22u32
        };
        if atoms[start_atom_index as usize].code_point() != expected_delimiter {
            let mismatch = QuotedScalarError::at(
                QuotedScalarErrorKind::InputStructuralMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(quoted_scalar_scan_tail_spec);
                assert(expected == Err(mismatch@));
                reveal(scan_profile1_quoted_scalars_spec);
            }
            return Err(mismatch);
        }
        if scalars.len() as u64 >= scalar_limit {
            let error = QuotedScalarError::at(
                QuotedScalarErrorKind::ScalarLimitExceeded,
                candidate.byte_start(),
            );
            proof {
                reveal(quoted_scalar_scan_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_quoted_scalars_spec);
            }
            return Err(error);
        }
        if scalar_atom_limit == 0 {
            let error = QuotedScalarError::at(
                QuotedScalarErrorKind::ScalarAtomLimitExceeded,
                candidate.byte_start(),
            );
            proof {
                reveal(quoted_scalar_scan_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_quoted_scalars_spec);
            }
            return Err(error);
        }
        let scalar = match scan_quoted_scalar_body(
            atoms,
            atomized.source_len_bytes(),
            style,
            start_atom_index as usize,
            candidate.line_number(),
            scalar_atom_limit,
        ) {
            Ok(scalar) => scalar,
            Err(error) => {
                proof {
                    reveal(quoted_scalar_scan_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_quoted_scalars_spec);
                }
                return Err(error);
            },
        };
        let scalar_end_atom_index = scalar.end_atom_index();
        let ghost old_scalars = scalars@;
        proof {
            reveal(quoted_scalar_scan_tail_spec);
            if semantic_inputs {
                assert(quoted_scalar_range_spec(atom_views, scalar@));
                if old_scalars.len() > 0 {
                    let previous = quoted_scalar_views_spec(old_scalars)[old_scalars.len() - 1];
                    assert(previous.end_atom_index <= candidate@.start_atom_index);
                    assert(scalar@.start_atom_index == candidate@.start_atom_index);
                    crate::structural::lemma_structural_well_formed_requires_layout(
                        atomized@,
                        layout@,
                        structural@,
                    );
                    crate::layout::lemma_layout_well_formed_requires_intrinsic_atom_source(
                        atomized@,
                        layout@,
                    );
                    lemma_earlier_atom_ends_before_later_atom_starts(
                        atomized@,
                        previous.end_atom_index as int - 1,
                        scalar@.start_atom_index as int,
                    );
                    assert(previous.byte_end <= scalar@.byte_start);
                }
                lemma_quoted_scalar_sequence_ranges_push(
                    atom_views,
                    quoted_scalar_views_spec(old_scalars),
                    scalar@,
                );
            }
            lemma_quoted_scalar_views_push(old_scalars, scalar);
            fuel = (fuel - 1) as nat;
        }
        scalars.push(scalar);
        context = context_after_quoted_scalar(context);
        candidate_index =
        candidate_index_after_atom(candidates, candidate_index + 1, scalar_end_atom_index);
    }
    let source = QuotedScalarSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version(),
        layout_transformation_version: layout.transformation_version(),
        structural_transformation_version: structural.transformation_version(),
        transformation_version: QUOTED_SCALAR_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes(),
        bom_bytes: atomized.bom_bytes(),
        input_atom_count: atoms.len() as u64,
        input_line_count: layout.lines().len() as u64,
        input_structural_lexeme_count: candidates.len() as u64,
        scalars,
    };
    proof {
        reveal(quoted_scalar_scan_tail_spec);
        assert(expected == Ok(source@.scalars));
        assert(source@.profile_version == CRUCIBLE_YAML_PROFILE_VERSION);
        assert(source@.input_transformation_version == atomized@.transformation_version);
        assert(source@.layout_transformation_version == layout@.transformation_version);
        assert(source@.structural_transformation_version == structural@.transformation_version);
        assert(source@.source_len_bytes == atomized@.source_len_bytes);
        assert(source@.bom_bytes == atomized@.bom_bytes);
        assert(source@.input_atom_count == atomized@.atoms.len());
        assert(source@.input_line_count == layout@.lines.len());
        assert(source@.input_structural_lexeme_count == structural@.lexemes.len());
        reveal(scan_profile1_quoted_scalars_spec);
        assert(scan_profile1_quoted_scalars_spec(atomized@, layout@, structural@, limits@) == Ok(
            source@,
        ));
        reveal(quoted_scalar_source_corresponds_spec);
        assert(exists|candidate_limits: QuotedScalarScanLimitsView|
            scan_profile1_quoted_scalars_spec(atomized@, layout@, structural@, candidate_limits)
                == Ok(source@)) by {
            assert(scan_profile1_quoted_scalars_spec(atomized@, layout@, structural@, limits@)
                == Ok(source@));
        }
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
            && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
            && crate::structural::structural_lexeme_source_well_formed_spec(
            atomized@,
            layout@,
            structural@,
        ) {
            assert(semantic_inputs);
            assert(quoted_scalar_sequence_ranges_spec(atomized@.atoms, source@.scalars));
            reveal(quoted_scalar_ranges_well_formed_spec);
            assert(quoted_scalar_ranges_well_formed_spec(atomized@, source@));
            reveal(quoted_scalar_source_well_formed_spec);
        }
    }
    Ok(source)
}

} // verus!
