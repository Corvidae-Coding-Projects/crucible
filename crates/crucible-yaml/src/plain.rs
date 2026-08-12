//! Verified plain-scalar boundaries and contextual tab admission for Crucible YAML profile 1.
//!
//! This context-sensitive lexer slice authenticates every preceding YAML transformation, excludes
//! quoted and block-scalar regions, recognizes YAML 1.2.2 plain-scalar presentation ranges, and
//! retains exact atom, byte, and line endpoints without decoding folded content.
use crate::atom::{
    AtomizedSource, LexicalAtom, LexicalAtomKind, YamlIndicator, MAX_PROFILE1_LEXICAL_ATOMS,
};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
#[allow(unused_imports)]
use crate::layout::LayoutSourceView;
use crate::layout::{analyze_profile1_layout, LayoutSource};
use crate::quoted::{
    canonical_quoted_scalar_limits, scan_profile1_quoted_scalars, QuotedScalarSource,
};
#[allow(unused_imports)]
use crate::quoted::{QuotedScalarSourceView, QuotedScalarView};
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

pub const PLAIN_SCALAR_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_PLAIN_SCALARS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_PLAIN_SCALAR_ATOMS: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlainScalarScanLimits {
    max_scalars: u64,
    max_scalar_atoms: u64,
}

#[verifier::ext_equal]
pub struct PlainScalarScanLimitsView {
    pub max_scalars: u64,
    pub max_scalar_atoms: u64,
}

impl View for PlainScalarScanLimits {
    type V = PlainScalarScanLimitsView;

    closed spec fn view(&self) -> PlainScalarScanLimitsView {
        PlainScalarScanLimitsView {
            max_scalars: self.max_scalars,
            max_scalar_atoms: self.max_scalar_atoms,
        }
    }
}

impl PlainScalarScanLimits {
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
pub enum PlainScalarErrorKind {
    InputQuotedMismatch,
    ScalarLimitExceeded,
    ScalarAtomLimitExceeded,
    InvalidPlainStart,
    InvalidPlainCharacter,
    TabInIndentation,
    InvalidBlockIndentation,
    ReservedIndicator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlainScalarError {
    kind: PlainScalarErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct PlainScalarErrorView {
    pub kind: PlainScalarErrorKind,
    pub byte_offset: u64,
}

impl View for PlainScalarError {
    type V = PlainScalarErrorView;

    closed spec fn view(&self) -> PlainScalarErrorView {
        PlainScalarErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl PlainScalarError {
    fn at(kind: PlainScalarErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (PlainScalarErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: PlainScalarErrorKind)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// One complete plain scalar from its first content atom through its final nonspace content atom.
///
/// ```compile_fail
/// use crucible_yaml::PlainScalar;
///
/// let forged = PlainScalar {
///     start_line_number: 2,
///     end_line_number: 1,
///     start_atom_index: 9,
///     end_atom_index: 3,
///     byte_start: 9,
///     byte_end: 3,
/// };
/// ```
pub struct PlainScalar {
    start_line_number: u64,
    end_line_number: u64,
    start_atom_index: u64,
    end_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
}

#[verifier::ext_equal]
pub struct PlainScalarView {
    pub start_line_number: u64,
    pub end_line_number: u64,
    pub start_atom_index: u64,
    pub end_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
}

impl View for PlainScalar {
    type V = PlainScalarView;

    closed spec fn view(&self) -> PlainScalarView {
        PlainScalarView {
            start_line_number: self.start_line_number,
            end_line_number: self.end_line_number,
            start_atom_index: self.start_atom_index,
            end_atom_index: self.end_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
        }
    }
}

impl DeepView for PlainScalar {
    type V = PlainScalarView;

    closed spec fn deep_view(&self) -> PlainScalarView {
        self@
    }
}

impl PlainScalar {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        self.start_line_number == other.start_line_number && self.end_line_number
            == other.end_line_number && self.start_atom_index == other.start_atom_index
            && self.end_atom_index == other.end_atom_index && self.byte_start == other.byte_start
            && self.byte_end == other.byte_end
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
pub struct PlainScalarSource {
    profile_version: u16,
    input_transformation_version: u16,
    layout_transformation_version: u16,
    structural_transformation_version: u16,
    quoted_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    input_atom_count: u64,
    input_line_count: u64,
    input_structural_lexeme_count: u64,
    input_quoted_scalar_count: u64,
    scalars: Vec<PlainScalar>,
}

#[verifier::ext_equal]
pub struct PlainScalarSourceView {
    pub profile_version: u16,
    pub input_transformation_version: u16,
    pub layout_transformation_version: u16,
    pub structural_transformation_version: u16,
    pub quoted_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub input_atom_count: u64,
    pub input_line_count: u64,
    pub input_structural_lexeme_count: u64,
    pub input_quoted_scalar_count: u64,
    pub scalars: Seq<PlainScalarView>,
}

pub open spec fn plain_scalar_views_spec(scalars: Seq<PlainScalar>) -> Seq<PlainScalarView> {
    Seq::new(scalars.len(), |index: int| scalars[index]@)
}

proof fn lemma_plain_scalar_views_push(scalars: Seq<PlainScalar>, scalar: PlainScalar)
    ensures
        plain_scalar_views_spec(scalars.push(scalar)) == plain_scalar_views_spec(scalars).push(
            scalar@,
        ),
{
    reveal(plain_scalar_views_spec);
    assert(plain_scalar_views_spec(scalars.push(scalar)) =~= plain_scalar_views_spec(scalars).push(
        scalar@,
    ));
}

impl View for PlainScalarSource {
    type V = PlainScalarSourceView;

    closed spec fn view(&self) -> PlainScalarSourceView {
        PlainScalarSourceView {
            profile_version: self.profile_version,
            input_transformation_version: self.input_transformation_version,
            layout_transformation_version: self.layout_transformation_version,
            structural_transformation_version: self.structural_transformation_version,
            quoted_transformation_version: self.quoted_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            input_atom_count: self.input_atom_count,
            input_line_count: self.input_line_count,
            input_structural_lexeme_count: self.input_structural_lexeme_count,
            input_quoted_scalar_count: self.input_quoted_scalar_count,
            scalars: plain_scalar_views_spec(self.scalars@),
        }
    }
}

impl PlainScalarSource {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.profile_version != other.profile_version || self.input_transformation_version
            != other.input_transformation_version || self.layout_transformation_version
            != other.layout_transformation_version || self.structural_transformation_version
            != other.structural_transformation_version || self.quoted_transformation_version
            != other.quoted_transformation_version || self.transformation_version
            != other.transformation_version || self.source_len_bytes != other.source_len_bytes
            || self.bom_bytes != other.bom_bytes || self.input_atom_count != other.input_atom_count
            || self.input_line_count != other.input_line_count || self.input_structural_lexeme_count
            != other.input_structural_lexeme_count || self.input_quoted_scalar_count
            != other.input_quoted_scalar_count {
            assert(self@ != other@);
            return false;
        }
        if self.scalars.len() != other.scalars.len() {
            proof {
                reveal(plain_scalar_views_spec);
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
                    reveal(plain_scalar_views_spec);
                    assert(self.scalars[index as int]@ != other.scalars[index as int]@);
                    assert(self@.scalars[index as int] != other@.scalars[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(plain_scalar_views_spec);
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

    pub fn quoted_transformation_version(&self) -> (version: u16)
        ensures
            version == self@.quoted_transformation_version,
    {
        self.quoted_transformation_version
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

    pub fn input_quoted_scalar_count(&self) -> (count: u64)
        ensures
            count == self@.input_quoted_scalar_count,
    {
        self.input_quoted_scalar_count
    }

    pub fn scalars(&self) -> (scalars: &[PlainScalar])
        ensures
            plain_scalar_views_spec(scalars@) == self@.scalars,
    {
        self.scalars.as_slice()
    }
}

pub open spec fn plain_scalar_range_spec(
    atoms: Seq<LexicalAtomView>,
    scalar: PlainScalarView,
) -> bool {
    scalar.start_atom_index < scalar.end_atom_index && scalar.end_atom_index <= atoms.len()
        && scalar.byte_start == atoms[scalar.start_atom_index as int].span.start.byte_offset
        && scalar.byte_end == atoms[(scalar.end_atom_index - 1) as int].span.end.byte_offset
        && scalar.start_line_number == atoms[scalar.start_atom_index as int].span.start.line
        && scalar.end_line_number == atoms[(scalar.end_atom_index - 1) as int].span.start.line
        && atoms[scalar.start_atom_index as int].kind != LexicalAtomKind::Space
        && atoms[scalar.start_atom_index as int].kind != LexicalAtomKind::Tab && atoms[(
    scalar.end_atom_index - 1) as int].kind != LexicalAtomKind::Space && atoms[(
    scalar.end_atom_index - 1) as int].kind != LexicalAtomKind::Tab && atoms[(scalar.end_atom_index
        - 1) as int].kind != LexicalAtomKind::LineFeed
}

pub open spec fn plain_scalar_sequence_ranges_spec(
    atoms: Seq<LexicalAtomView>,
    scalars: Seq<PlainScalarView>,
) -> bool {
    forall|index: int|
        0 <= index < scalars.len() ==> plain_scalar_range_spec(atoms, #[trigger] scalars[index])
            && (index > 0 ==> scalars[index - 1].end_atom_index <= scalars[index].start_atom_index
            && scalars[index - 1].byte_end <= scalars[index].byte_start)
}

proof fn lemma_earlier_plain_atom_ends_before_later_atom_starts(
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
        lemma_earlier_plain_atom_ends_before_later_atom_starts(atomized, earlier, later - 1);
        crate::atom::lemma_intrinsic_atomized_scalar_is_normalized(atomized, later - 1);
        reveal(crate::utf8::normalized_scalar_view_spec);
        assert(atomized.atoms[later - 1].span.start.byte_offset < atomized.atoms[later
            - 1].span.end.byte_offset);
        assert(atomized.atoms[later - 1].span.end == atomized.atoms[later].span.start);
    }
}

proof fn lemma_plain_scalar_sequence_ranges_push(
    atoms: Seq<LexicalAtomView>,
    scalars: Seq<PlainScalarView>,
    scalar: PlainScalarView,
)
    requires
        plain_scalar_sequence_ranges_spec(atoms, scalars),
        plain_scalar_range_spec(atoms, scalar),
        scalars.len() > 0 ==> scalars[scalars.len() - 1].end_atom_index <= scalar.start_atom_index,
        scalars.len() > 0 ==> scalars[scalars.len() - 1].byte_end <= scalar.byte_start,
    ensures
        plain_scalar_sequence_ranges_spec(atoms, scalars.push(scalar)),
{
    reveal(plain_scalar_sequence_ranges_spec);
    assert forall|index: int|
        0 <= index < scalars.push(scalar).len() implies plain_scalar_range_spec(
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

pub open spec fn plain_scalar_ranges_well_formed_spec(
    atomized: AtomizedSourceView,
    plain: PlainScalarSourceView,
) -> bool {
    plain.profile_version == CRUCIBLE_YAML_PROFILE_VERSION && plain.input_transformation_version
        == atomized.transformation_version && plain.transformation_version
        == PLAIN_SCALAR_TRANSFORMATION_VERSION && plain.source_len_bytes
        == atomized.source_len_bytes && plain.bom_bytes == atomized.bom_bytes
        && plain.input_atom_count == atomized.atoms.len() && plain_scalar_sequence_ranges_spec(
        atomized.atoms,
        plain.scalars,
    )
}

closed spec fn effective_scalar_limit_spec(limits: PlainScalarScanLimitsView) -> u64 {
    if limits.max_scalars < MAX_PROFILE1_PLAIN_SCALARS {
        limits.max_scalars
    } else {
        MAX_PROFILE1_PLAIN_SCALARS
    }
}

closed spec fn effective_scalar_atom_limit_spec(limits: PlainScalarScanLimitsView) -> u64 {
    if limits.max_scalar_atoms < MAX_PROFILE1_PLAIN_SCALAR_ATOMS {
        limits.max_scalar_atoms
    } else {
        MAX_PROFILE1_PLAIN_SCALAR_ATOMS
    }
}

pub open spec fn yaml_printable_plain_character_spec(code_point: u32) -> bool {
    crate::quoted::yaml_printable_character_spec(code_point)
}

closed spec fn yaml_indicator_code_point_spec(code_point: u32) -> bool {
    code_point == 0x2d || code_point == 0x3f || code_point == 0x3a || code_point == 0x2c
        || code_point == 0x5b || code_point == 0x5d || code_point == 0x7b || code_point == 0x7d
        || code_point == 0x23 || code_point == 0x26 || code_point == 0x2a || code_point == 0x21
        || code_point == 0x7c || code_point == 0x3e || code_point == 0x27 || code_point == 0x22
        || code_point == 0x25 || code_point == 0x40 || code_point == 0x60
}

fn yaml_indicator_code_point(code_point: u32) -> (indicator: bool)
    ensures
        indicator == yaml_indicator_code_point_spec(code_point),
{
    code_point == 0x2d || code_point == 0x3f || code_point == 0x3a || code_point == 0x2c
        || code_point == 0x5b || code_point == 0x5d || code_point == 0x7b || code_point == 0x7d
        || code_point == 0x23 || code_point == 0x26 || code_point == 0x2a || code_point == 0x21
        || code_point == 0x7c || code_point == 0x3e || code_point == 0x27 || code_point == 0x22
        || code_point == 0x25 || code_point == 0x40 || code_point == 0x60
}

closed spec fn plain_safe_atom_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    flow_depth: u64,
) -> bool {
    0 <= index < atoms.len() && atoms[index].kind != LexicalAtomKind::Space && atoms[index].kind
        != LexicalAtomKind::Tab && atoms[index].kind != LexicalAtomKind::LineFeed && (flow_depth
        == 0 || (atoms[index].code_point != 0x2c && atoms[index].code_point != 0x5b
        && atoms[index].code_point != 0x5d && atoms[index].code_point != 0x7b
        && atoms[index].code_point != 0x7d))
}

fn plain_safe_atom(atoms: &[LexicalAtom], index: usize, flow_depth: u64) -> (safe: bool)
    requires
        index <= atoms@.len(),
    ensures
        safe == plain_safe_atom_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            index as int,
            flow_depth,
        ),
{
    if index >= atoms.len() {
        return false;
    }
    let kind = atoms[index].kind();
    kind != LexicalAtomKind::Space && kind != LexicalAtomKind::Tab && kind
        != LexicalAtomKind::LineFeed && (flow_depth == 0 || {
        let code_point = atoms[index].code_point();
        code_point != 0x2c && code_point != 0x5b && code_point != 0x5d && code_point != 0x7b
            && code_point != 0x7d
    })
}

closed spec fn plain_start_allowed_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    flow_depth: u64,
) -> bool {
    0 <= start < atoms.len() && if atoms[start].code_point == 0x3f || atoms[start].code_point
        == 0x3a || atoms[start].code_point == 0x2d {
        plain_safe_atom_spec(atoms, start + 1, flow_depth)
    } else {
        !yaml_indicator_code_point_spec(atoms[start].code_point)
    }
}

fn plain_start_allowed(atoms: &[LexicalAtom], start: usize, flow_depth: u64) -> (allowed: bool)
    requires
        start < atoms@.len(),
    ensures
        allowed == plain_start_allowed_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            flow_depth,
        ),
{
    let code_point = atoms[start].code_point();
    if code_point == 0x3f || code_point == 0x3a || code_point == 0x2d {
        plain_safe_atom(atoms, start + 1, flow_depth)
    } else {
        !yaml_indicator_code_point(code_point)
    }
}

closed spec fn first_plain_terminating_colon_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    flow_depth: u64,
    fuel: nat,
) -> Option<u64>
    decreases fuel,
{
    if index >= end || index < 0 || end > atoms.len() {
        None
    } else if fuel == 0 {
        Some(index as u64)
    } else if atoms[index].code_point == 0x3a && !plain_safe_atom_spec(
        atoms,
        index + 1,
        flow_depth,
    ) {
        Some(index as u64)
    } else {
        first_plain_terminating_colon_spec(atoms, index + 1, end, flow_depth, (fuel - 1) as nat)
    }
}

fn first_plain_terminating_colon(
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
    flow_depth: u64,
) -> (terminator: Option<u64>)
    requires
        start <= end <= atoms@.len(),
    ensures
        terminator == first_plain_terminating_colon_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            flow_depth,
            (end - start) as nat,
        ),
        match terminator {
            Some(index) => start <= index < end,
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = first_plain_terminating_colon_spec(
        views,
        start as int,
        end as int,
        flow_depth,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == first_plain_terminating_colon_spec(
                views,
                index as int,
                end as int,
                flow_depth,
                (end - index) as nat,
            ),
            expected == first_plain_terminating_colon_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                flow_depth,
                (end - start) as nat,
            ),
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if atoms[index].code_point() == 0x3a && !plain_safe_atom(atoms, index + 1, flow_depth) {
            proof {
                reveal(first_plain_terminating_colon_spec);
                assert(expected == Some(index as u64));
            }
            return Some(index as u64);
        }
        proof {
            reveal(first_plain_terminating_colon_spec);
        }
        index += 1;
    }
    proof {
        reveal(first_plain_terminating_colon_spec);
    }
    None
}

#[allow(clippy::manual_range_contains)]
fn yaml_printable_plain_character(code_point: u32) -> (printable: bool)
    ensures
        printable == yaml_printable_plain_character_spec(code_point),
{
    code_point == 0x09 || code_point == 0x0a || (0x20 <= code_point && code_point <= 0x7e)
        || code_point == 0x85 || (0xa0 <= code_point && code_point <= 0xd7ff) || (0xe000
        <= code_point && code_point <= 0xfffd) || (0x10000 <= code_point && code_point <= 0x10ffff)
}

closed spec fn first_invalid_plain_atom_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Option<u64>
    decreases fuel,
{
    if index >= end || index >= atoms.len() || index < 0 {
        None
    } else if fuel == 0 {
        Some(index as u64)
    } else if !yaml_printable_plain_character_spec(atoms[index].code_point)
        || atoms[index].code_point == 0xfeff {
        Some(index as u64)
    } else {
        first_invalid_plain_atom_spec(atoms, index + 1, end, (fuel - 1) as nat)
    }
}

fn first_invalid_plain_atom(atoms: &[LexicalAtom], start: usize, end: usize) -> (invalid: Option<
    u64,
>)
    requires
        start <= end <= atoms@.len(),
    ensures
        invalid == first_invalid_plain_atom_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
        match invalid {
            Some(index) => start <= index < end,
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = first_invalid_plain_atom_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == first_invalid_plain_atom_spec(
                views,
                index as int,
                end as int,
                (end - index) as nat,
            ),
            expected == first_invalid_plain_atom_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ),
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if !yaml_printable_plain_character(atoms[index].code_point()) || atoms[index].code_point()
            == 0xfeff {
            proof {
                reveal(first_invalid_plain_atom_spec);
                assert(expected == Some(index as u64));
                assert(views == crate::atom::lexical_atom_views_spec(atoms@));
                assert(first_invalid_plain_atom_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    start as int,
                    end as int,
                    (end - start) as nat,
                ) == Some(index as u64));
            }
            return Some(index as u64);
        }
        proof {
            reveal(first_invalid_plain_atom_spec);
        }
        index += 1;
    }
    proof {
        reveal(first_invalid_plain_atom_spec);
    }
    None
}

closed spec fn last_plain_content_end_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
    fuel: nat,
) -> Option<u64>
    decreases fuel,
{
    if end <= start || end <= 0 || end > atoms.len() {
        None
    } else if fuel == 0 {
        None
    } else {
        let kind = atoms[end - 1].kind;
        if kind == LexicalAtomKind::Space || kind == LexicalAtomKind::Tab || kind
            == LexicalAtomKind::LineFeed {
            last_plain_content_end_spec(atoms, start, end - 1, (fuel - 1) as nat)
        } else {
            Some(end as u64)
        }
    }
}

fn last_plain_content_end(atoms: &[LexicalAtom], start: usize, end: usize) -> (content_end: Option<
    u64,
>)
    requires
        start <= end <= atoms@.len(),
    ensures
        content_end == last_plain_content_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
        match content_end {
            Some(index) => {
                start < index <= end && atoms[(index - 1) as int]@.kind != LexicalAtomKind::Space
                    && atoms[(index - 1) as int]@.kind != LexicalAtomKind::Tab && atoms[(index
                    - 1) as int]@.kind != LexicalAtomKind::LineFeed
            },
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = last_plain_content_end_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut cursor = end;
    while cursor > start
        invariant
            start <= cursor <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == last_plain_content_end_spec(
                views,
                start as int,
                cursor as int,
                (cursor - start) as nat,
            ),
            expected == last_plain_content_end_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ),
        decreases cursor - start,
    {
        let index = cursor - 1;
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let kind = atoms[index].kind();
        if kind != LexicalAtomKind::Space && kind != LexicalAtomKind::Tab && kind
            != LexicalAtomKind::LineFeed {
            proof {
                reveal(last_plain_content_end_spec);
                assert(expected == Some(cursor as u64));
            }
            return Some(cursor as u64);
        }
        proof {
            reveal(last_plain_content_end_spec);
        }
        cursor -= 1;
    }
    proof {
        reveal(last_plain_content_end_spec);
    }
    None
}

closed spec fn plain_content_end_spec(
    atoms: Seq<LexicalAtomView>,
    start: int,
    end: int,
    flow_depth: u64,
) -> Option<u64> {
    let colon = first_plain_terminating_colon_spec(
        atoms,
        start,
        end,
        flow_depth,
        (end - start) as nat,
    );
    let bounded_end = match colon {
        Some(index) => index as int,
        None => end,
    };
    last_plain_content_end_spec(atoms, start, bounded_end, (bounded_end - start) as nat)
}

fn plain_content_end(
    atoms: &[LexicalAtom],
    start: usize,
    end: usize,
    flow_depth: u64,
) -> (content_end: Option<u64>)
    requires
        start <= end <= atoms@.len(),
    ensures
        content_end == plain_content_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            flow_depth,
        ),
        match content_end {
            Some(index) => {
                start < index <= end && atoms[(index - 1) as int]@.kind != LexicalAtomKind::Space
                    && atoms[(index - 1) as int]@.kind != LexicalAtomKind::Tab && atoms[(index
                    - 1) as int]@.kind != LexicalAtomKind::LineFeed
            },
            None => true,
        },
{
    let terminator = first_plain_terminating_colon(atoms, start, end, flow_depth);
    let bounded_end = match terminator {
        Some(index) => index as usize,
        None => end,
    };
    let result = last_plain_content_end(atoms, start, bounded_end);
    proof {
        reveal(plain_content_end_spec);
    }
    result
}

closed spec fn candidate_is_line_feed_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::LineFeed
}

closed spec fn candidate_is_indentation_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::Indentation
}

closed spec fn candidate_is_separation_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::Separation
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

closed spec fn candidate_is_reserved_spec(candidate: StructuralLexemeView) -> bool {
    candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::ReservedAt)
        || candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::ReservedGraveAccent)
}

closed spec fn candidate_terminates_plain_spec(
    candidate: StructuralLexemeView,
    flow_depth: u64,
) -> bool {
    candidate.kind == StructuralCandidateRole::Comment || candidate.kind
        == StructuralCandidateRole::Indicator(YamlIndicator::MappingValue) || (flow_depth > 0 && (
    candidate_is_flow_start_spec(candidate) || candidate_is_flow_end_spec(candidate)
        || candidate.kind == StructuralCandidateRole::FlowEntry))
}

fn candidate_terminates_plain(candidate: &StructuralLexeme, flow_depth: u64) -> (terminates: bool)
    ensures
        terminates == candidate_terminates_plain_spec(candidate@, flow_depth),
{
    let role = candidate.candidate_role();
    role == StructuralCandidateRole::Comment || role == StructuralCandidateRole::Indicator(
        YamlIndicator::MappingValue,
    ) || (flow_depth > 0 && (role == StructuralCandidateRole::FlowSequenceStart || role
        == StructuralCandidateRole::FlowMappingStart || role
        == StructuralCandidateRole::FlowSequenceEnd || role
        == StructuralCandidateRole::FlowMappingEnd || role == StructuralCandidateRole::FlowEntry))
}

#[verifier::ext_equal]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct PlainContextView {
    flow_depth: u64,
    line_indentation: u64,
    at_line_start: bool,
    after_node: bool,
    block_mode: u8,
    block_parent_indentation: u64,
    block_content_indentation: u64,
    block_line_active: bool,
    property_payload_mode: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlainContext {
    flow_depth: u64,
    line_indentation: u64,
    at_line_start: bool,
    after_node: bool,
    block_mode: u8,
    block_parent_indentation: u64,
    block_content_indentation: u64,
    block_line_active: bool,
    property_payload_mode: u8,
}

impl View for PlainContext {
    type V = PlainContextView;

    closed spec fn view(&self) -> PlainContextView {
        PlainContextView {
            flow_depth: self.flow_depth,
            line_indentation: self.line_indentation,
            at_line_start: self.at_line_start,
            after_node: self.after_node,
            block_mode: self.block_mode,
            block_parent_indentation: self.block_parent_indentation,
            block_content_indentation: self.block_content_indentation,
            block_line_active: self.block_line_active,
            property_payload_mode: self.property_payload_mode,
        }
    }
}

closed spec fn initial_plain_context_spec() -> PlainContextView {
    PlainContextView {
        flow_depth: 0,
        line_indentation: 0,
        at_line_start: true,
        after_node: false,
        block_mode: 0,
        block_parent_indentation: 0,
        block_content_indentation: 0,
        block_line_active: false,
        property_payload_mode: 0,
    }
}

fn initial_plain_context() -> (context: PlainContext)
    ensures
        context@ == initial_plain_context_spec(),
{
    PlainContext {
        flow_depth: 0,
        line_indentation: 0,
        at_line_start: true,
        after_node: false,
        block_mode: 0,
        block_parent_indentation: 0,
        block_content_indentation: 0,
        block_line_active: false,
        property_payload_mode: 0,
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct PlainBodySuccessView {
    scalar: PlainScalarView,
    next_candidate_index: int,
    line_indentation: u64,
    at_line_start: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlainBodySuccess {
    scalar: PlainScalar,
    next_candidate_index: usize,
    line_indentation: u64,
    at_line_start: bool,
}

impl View for PlainBodySuccess {
    type V = PlainBodySuccessView;

    closed spec fn view(&self) -> PlainBodySuccessView {
        PlainBodySuccessView {
            scalar: self.scalar@,
            next_candidate_index: self.next_candidate_index as int,
            line_indentation: self.line_indentation,
            at_line_start: self.at_line_start,
        }
    }
}

fn make_plain_scalar(
    atoms: &[LexicalAtom],
    start_atom_index: usize,
    end_atom_index: usize,
) -> (scalar: PlainScalar)
    requires
        start_atom_index < end_atom_index <= atoms@.len(),
        atoms[start_atom_index as int]@.kind != LexicalAtomKind::Space,
        atoms[start_atom_index as int]@.kind != LexicalAtomKind::Tab,
        atoms[(end_atom_index - 1) as int]@.kind != LexicalAtomKind::Space,
        atoms[(end_atom_index - 1) as int]@.kind != LexicalAtomKind::Tab,
        atoms[(end_atom_index - 1) as int]@.kind != LexicalAtomKind::LineFeed,
    ensures
        scalar@ == (PlainScalarView {
            start_line_number: atoms[start_atom_index as int]@.span.start.line,
            end_line_number: atoms[(end_atom_index - 1) as int]@.span.start.line,
            start_atom_index: start_atom_index as u64,
            end_atom_index: end_atom_index as u64,
            byte_start: atoms[start_atom_index as int]@.span.start.byte_offset,
            byte_end: atoms[(end_atom_index - 1) as int]@.span.end.byte_offset,
        }),
        plain_scalar_range_spec(crate::atom::lexical_atom_views_spec(atoms@), scalar@),
{
    PlainScalar {
        start_line_number: atoms[start_atom_index].span().start().line(),
        end_line_number: atoms[end_atom_index - 1].span().start().line(),
        start_atom_index: start_atom_index as u64,
        end_atom_index: end_atom_index as u64,
        byte_start: atoms[start_atom_index].span().start().byte_offset(),
        byte_end: atoms[end_atom_index - 1].span().end().byte_offset(),
    }
}

closed spec fn plain_error_spec(kind: PlainScalarErrorKind, byte_offset: u64) -> Result<
    PlainBodySuccessView,
    PlainScalarErrorView,
> {
    Err(PlainScalarErrorView { kind, byte_offset })
}

closed spec fn plain_scalar_view_from_atoms_spec(
    atoms: Seq<LexicalAtomView>,
    start_atom_index: int,
    end_atom_index: int,
) -> PlainScalarView {
    PlainScalarView {
        start_line_number: atoms[start_atom_index].span.start.line,
        end_line_number: atoms[end_atom_index - 1].span.start.line,
        start_atom_index: start_atom_index as u64,
        end_atom_index: end_atom_index as u64,
        byte_start: atoms[start_atom_index].span.start.byte_offset,
        byte_end: atoms[end_atom_index - 1].span.end.byte_offset,
    }
}

closed spec fn plain_body_success_spec(
    atoms: Seq<LexicalAtomView>,
    start_atom_index: int,
    end_atom_index: int,
    next_candidate_index: int,
    line_indentation: u64,
    at_line_start: bool,
) -> Result<PlainBodySuccessView, PlainScalarErrorView> {
    Ok(
        PlainBodySuccessView {
            scalar: plain_scalar_view_from_atoms_spec(atoms, start_atom_index, end_atom_index),
            next_candidate_index,
            line_indentation,
            at_line_start,
        },
    )
}

closed spec fn indentation_width_spec(candidate: StructuralLexemeView) -> u64 {
    if candidate.start_atom_index <= candidate.end_atom_index {
        (candidate.end_atom_index - candidate.start_atom_index) as u64
    } else {
        0
    }
}

closed spec fn plain_scalar_body_spec(
    atoms: Seq<LexicalAtomView>,
    candidates: Seq<StructuralLexemeView>,
    candidate_index: int,
    start_atom_index: int,
    last_content_end: int,
    parent_indentation: u64,
    flow_depth: u64,
    line_indentation: u64,
    pending_line: bool,
    scalar_atom_limit: u64,
    fuel: nat,
) -> Result<PlainBodySuccessView, PlainScalarErrorView>
    decreases fuel,
{
    if candidate_index >= candidates.len() {
        plain_body_success_spec(
            atoms,
            start_atom_index,
            last_content_end,
            candidate_index,
            line_indentation,
            pending_line,
        )
    } else if fuel == 0 || candidate_index < 0 {
        plain_error_spec(PlainScalarErrorKind::InputQuotedMismatch, 0)
    } else {
        let candidate = candidates[candidate_index];
        if candidate.start_atom_index >= candidate.end_atom_index || candidate.end_atom_index
            > atoms.len() || candidate.start_atom_index < last_content_end {
            plain_error_spec(PlainScalarErrorKind::InputQuotedMismatch, candidate.byte_start)
        } else if candidate_is_separation_spec(candidate) {
            plain_scalar_body_spec(
                atoms,
                candidates,
                candidate_index + 1,
                start_atom_index,
                last_content_end,
                parent_indentation,
                flow_depth,
                line_indentation,
                pending_line,
                scalar_atom_limit,
                (fuel - 1) as nat,
            )
        } else if candidate_is_line_feed_spec(candidate) {
            plain_scalar_body_spec(
                atoms,
                candidates,
                candidate_index + 1,
                start_atom_index,
                last_content_end,
                parent_indentation,
                flow_depth,
                0,
                true,
                scalar_atom_limit,
                (fuel - 1) as nat,
            )
        } else if pending_line && candidate_is_indentation_spec(candidate) {
            plain_scalar_body_spec(
                atoms,
                candidates,
                candidate_index + 1,
                start_atom_index,
                last_content_end,
                parent_indentation,
                flow_depth,
                indentation_width_spec(candidate),
                true,
                scalar_atom_limit,
                (fuel - 1) as nat,
            )
        } else if pending_line && flow_depth == 0 && line_indentation <= parent_indentation {
            if atoms[candidate.start_atom_index as int].kind == LexicalAtomKind::Tab {
                plain_error_spec(
                    PlainScalarErrorKind::TabInIndentation,
                    atoms[candidate.start_atom_index as int].span.start.byte_offset,
                )
            } else {
                plain_body_success_spec(
                    atoms,
                    start_atom_index,
                    last_content_end,
                    candidate_index,
                    line_indentation,
                    true,
                )
            }
        } else if candidate_terminates_plain_spec(candidate, flow_depth) {
            plain_body_success_spec(
                atoms,
                start_atom_index,
                last_content_end,
                candidate_index,
                line_indentation,
                false,
            )
        } else {
            match first_invalid_plain_atom_spec(
                atoms,
                candidate.start_atom_index as int,
                candidate.end_atom_index as int,
                (candidate.end_atom_index - candidate.start_atom_index) as nat,
            ) {
                Some(invalid) => plain_error_spec(
                    PlainScalarErrorKind::InvalidPlainCharacter,
                    atoms[invalid as int].span.start.byte_offset,
                ),
                None => match plain_content_end_spec(
                    atoms,
                    candidate.start_atom_index as int,
                    candidate.end_atom_index as int,
                    flow_depth,
                ) {
                    None => plain_scalar_body_spec(
                        atoms,
                        candidates,
                        candidate_index + 1,
                        start_atom_index,
                        last_content_end,
                        parent_indentation,
                        flow_depth,
                        line_indentation,
                        pending_line,
                        scalar_atom_limit,
                        (fuel - 1) as nat,
                    ),
                    Some(content_end) => {
                        if content_end - start_atom_index > scalar_atom_limit {
                            plain_error_spec(
                                PlainScalarErrorKind::ScalarAtomLimitExceeded,
                                atoms[start_atom_index
                                    + scalar_atom_limit as int].span.start.byte_offset,
                            )
                        } else {
                            plain_scalar_body_spec(
                                atoms,
                                candidates,
                                candidate_index + 1,
                                start_atom_index,
                                content_end as int,
                                parent_indentation,
                                flow_depth,
                                line_indentation,
                                false,
                                scalar_atom_limit,
                                (fuel - 1) as nat,
                            )
                        }
                    },
                },
            }
        }
    }
}

fn finish_plain_body(
    atoms: &[LexicalAtom],
    start_atom_index: usize,
    last_content_end: usize,
    next_candidate_index: usize,
    line_indentation: u64,
    at_line_start: bool,
) -> (success: PlainBodySuccess)
    requires
        start_atom_index < last_content_end <= atoms@.len(),
        atoms[start_atom_index as int]@.kind != LexicalAtomKind::Space,
        atoms[start_atom_index as int]@.kind != LexicalAtomKind::Tab,
        atoms[(last_content_end - 1) as int]@.kind != LexicalAtomKind::Space,
        atoms[(last_content_end - 1) as int]@.kind != LexicalAtomKind::Tab,
        atoms[(last_content_end - 1) as int]@.kind != LexicalAtomKind::LineFeed,
    ensures
        success@ == (PlainBodySuccessView {
            scalar: plain_scalar_view_from_atoms_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start_atom_index as int,
                last_content_end as int,
            ),
            next_candidate_index: next_candidate_index as int,
            line_indentation,
            at_line_start,
        }),
{
    PlainBodySuccess {
        scalar: make_plain_scalar(atoms, start_atom_index, last_content_end),
        next_candidate_index,
        line_indentation,
        at_line_start,
    }
}

#[verifier::rlimit(160)]
#[allow(clippy::too_many_arguments)]
fn scan_plain_scalar_body(
    atoms: &[LexicalAtom],
    candidates: &[StructuralLexeme],
    start_candidate_index: usize,
    start_atom_index: usize,
    first_content_end: usize,
    parent_indentation: u64,
    flow_depth: u64,
    scalar_atom_limit: u64,
) -> (result: Result<PlainBodySuccess, PlainScalarError>)
    requires
        start_candidate_index < candidates@.len(),
        start_atom_index < first_content_end <= atoms@.len(),
        candidates[start_candidate_index as int]@.start_atom_index <= start_atom_index,
        first_content_end <= candidates[start_candidate_index as int]@.end_atom_index,
        atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        candidates@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        atoms[start_atom_index as int]@.kind != LexicalAtomKind::Space,
        atoms[start_atom_index as int]@.kind != LexicalAtomKind::Tab,
        atoms[(first_content_end - 1) as int]@.kind != LexicalAtomKind::Space,
        atoms[(first_content_end - 1) as int]@.kind != LexicalAtomKind::Tab,
        atoms[(first_content_end - 1) as int]@.kind != LexicalAtomKind::LineFeed,
        scalar_atom_limit > 0,
    ensures
        plain_scalar_body_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::structural::structural_lexeme_views_spec(candidates@),
            start_candidate_index as int + 1,
            start_atom_index as int,
            first_content_end as int,
            parent_indentation,
            flow_depth,
            parent_indentation,
            false,
            scalar_atom_limit,
            (candidates@.len() - start_candidate_index) as nat,
        ) == match result {
            Ok(success) => Ok(success@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(success) => {
                plain_scalar_range_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    success@.scalar,
                ) && start_candidate_index < success.next_candidate_index <= candidates@.len()
                    && success@.scalar.start_atom_index == start_atom_index && (
                success.next_candidate_index < candidates@.len() ==> success@.scalar.end_atom_index
                    <= candidates[success.next_candidate_index as int]@.start_atom_index)
            },
            Err(_) => true,
        },
{
    assert(MAX_PROFILE1_LEXICAL_ATOMS < usize::MAX);
    assert(atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS as usize);
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost candidate_views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost expected = plain_scalar_body_spec(
        atom_views,
        candidate_views,
        start_candidate_index as int + 1,
        start_atom_index as int,
        first_content_end as int,
        parent_indentation,
        flow_depth,
        parent_indentation,
        false,
        scalar_atom_limit,
        (candidates@.len() - start_candidate_index) as nat,
    );
    let mut candidate_index = start_candidate_index + 1;
    let mut last_content_end = first_content_end;
    let mut line_indentation = parent_indentation;
    let mut pending_line = false;
    let ghost mut fuel: nat = (candidates@.len() - start_candidate_index) as nat;
    while candidate_index < candidates.len()
        invariant
            start_candidate_index < candidate_index <= candidates@.len(),
            start_atom_index < last_content_end <= atoms@.len(),
            candidates@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS as usize,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            candidate_views == crate::structural::structural_lexeme_views_spec(candidates@),
            atoms[start_atom_index as int]@.kind != LexicalAtomKind::Space,
            atoms[start_atom_index as int]@.kind != LexicalAtomKind::Tab,
            atoms[(last_content_end - 1) as int]@.kind != LexicalAtomKind::Space,
            atoms[(last_content_end - 1) as int]@.kind != LexicalAtomKind::Tab,
            atoms[(last_content_end - 1) as int]@.kind != LexicalAtomKind::LineFeed,
            fuel >= candidates@.len() - candidate_index + 1,
            expected == plain_scalar_body_spec(
                atom_views,
                candidate_views,
                candidate_index as int,
                start_atom_index as int,
                last_content_end as int,
                parent_indentation,
                flow_depth,
                line_indentation,
                pending_line,
                scalar_atom_limit,
                fuel,
            ),
            expected == plain_scalar_body_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                crate::structural::structural_lexeme_views_spec(candidates@),
                start_candidate_index as int + 1,
                start_atom_index as int,
                first_content_end as int,
                parent_indentation,
                flow_depth,
                parent_indentation,
                false,
                scalar_atom_limit,
                (candidates@.len() - start_candidate_index) as nat,
            ),
        decreases candidates.len() - candidate_index,
    {
        let candidate = &candidates[candidate_index];
        assert(candidate_views[candidate_index as int] == candidate@) by {
            reveal(crate::structural::structural_lexeme_views_spec);
        }
        let candidate_start_u64 = candidate.start_atom_index();
        let candidate_end_u64 = candidate.end_atom_index();
        if candidate_start_u64 >= candidate_end_u64 || candidate_end_u64 > atoms.len() as u64
            || candidate_start_u64 < last_content_end as u64 {
            let error = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                candidate.byte_start(),
            );
            proof {
                assert((candidate_index as int) < candidate_views.len());
                assert((candidate_index as int) >= 0);
                assert(fuel > 0);
                assert(candidate_views[candidate_index as int] == candidate@);
                assert(candidate@.start_atom_index >= candidate@.end_atom_index
                    || candidate@.end_atom_index > atom_views.len() || candidate@.start_atom_index
                    < last_content_end);
                reveal(plain_scalar_body_spec);
                reveal(plain_error_spec);
                assert(expected == Err(error@));
                assert(atom_views == crate::atom::lexical_atom_views_spec(atoms@));
                assert(candidate_views == crate::structural::structural_lexeme_views_spec(
                    candidates@,
                ));
                assert(plain_scalar_body_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    crate::structural::structural_lexeme_views_spec(candidates@),
                    start_candidate_index as int + 1,
                    start_atom_index as int,
                    first_content_end as int,
                    parent_indentation,
                    flow_depth,
                    parent_indentation,
                    false,
                    scalar_atom_limit,
                    (candidates@.len() - start_candidate_index) as nat,
                ) == Err(error@));
            }
            return Err(error);
        }
        assert(candidate_start_u64 <= candidate_end_u64 <= atoms.len() as u64);
        assert(atoms.len() < usize::MAX);
        let candidate_start = candidate_start_u64 as usize;
        let candidate_end = candidate_end_u64 as usize;
        assert(candidate_start as u64 == candidate_start_u64);
        assert(candidate_end as u64 == candidate_end_u64);
        let role = candidate.candidate_role();
        if role == StructuralCandidateRole::Separation {
            proof {
                reveal(plain_scalar_body_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::LineFeed {
            proof {
                reveal(plain_scalar_body_spec);
                fuel = (fuel - 1) as nat;
            }
            line_indentation = 0;
            pending_line = true;
            candidate_index += 1;
            continue;
        }
        if pending_line && role == StructuralCandidateRole::Indentation {
            proof {
                reveal(plain_scalar_body_spec);
                fuel = (fuel - 1) as nat;
            }
            line_indentation = candidate.end_atom_index() - candidate.start_atom_index();
            candidate_index += 1;
            continue;
        }
        if pending_line && flow_depth == 0 && line_indentation <= parent_indentation {
            if atoms[candidate_start].kind() == LexicalAtomKind::Tab {
                let error = PlainScalarError::at(
                    PlainScalarErrorKind::TabInIndentation,
                    atoms[candidate_start].span().start().byte_offset(),
                );
                proof {
                    reveal(plain_scalar_body_spec);
                    reveal(plain_error_spec);
                    assert(expected == Err(error@));
                }
                return Err(error);
            }
            let success = finish_plain_body(
                atoms,
                start_atom_index,
                last_content_end,
                candidate_index,
                line_indentation,
                true,
            );
            proof {
                reveal(plain_scalar_body_spec);
                reveal(plain_body_success_spec);
                assert(expected == Ok(success@));
                assert(atom_views == crate::atom::lexical_atom_views_spec(atoms@));
                assert(candidate_views == crate::structural::structural_lexeme_views_spec(
                    candidates@,
                ));
                assert(plain_scalar_body_spec(
                    crate::atom::lexical_atom_views_spec(atoms@),
                    crate::structural::structural_lexeme_views_spec(candidates@),
                    start_candidate_index as int + 1,
                    start_atom_index as int,
                    first_content_end as int,
                    parent_indentation,
                    flow_depth,
                    parent_indentation,
                    false,
                    scalar_atom_limit,
                    (candidates@.len() - start_candidate_index) as nat,
                ) == Ok(success@));
            }
            return Ok(success);
        }
        let terminates = candidate_terminates_plain(candidate, flow_depth);
        if terminates {
            let success = finish_plain_body(
                atoms,
                start_atom_index,
                last_content_end,
                candidate_index,
                line_indentation,
                false,
            );
            proof {
                assert(candidate_terminates_plain_spec(candidate@, flow_depth));
                assert(fuel > 0);
                reveal(plain_scalar_body_spec);
                reveal(plain_body_success_spec);
                assert(expected == Ok(success@));
            }
            return Ok(success);
        }
        let invalid_result = first_invalid_plain_atom(atoms, candidate_start, candidate_end);
        if let Some(invalid_u64) = invalid_result {
            assert(invalid_u64 < atoms.len() as u64);
            let invalid = invalid_u64 as usize;
            let error = PlainScalarError::at(
                PlainScalarErrorKind::InvalidPlainCharacter,
                atoms[invalid].span().start().byte_offset(),
            );
            proof {
                reveal(plain_scalar_body_spec);
                reveal(plain_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        let content_end_result = plain_content_end(
            atoms,
            candidate_start,
            candidate_end,
            flow_depth,
        );
        let content_end_u64 = match content_end_result {
            None => {
                proof {
                    reveal(plain_scalar_body_spec);
                    fuel = (fuel - 1) as nat;
                }
                candidate_index += 1;
                continue;
            },
            Some(content_end) => content_end,
        };
        assert(content_end_u64 > start_atom_index as u64);
        if content_end_u64 - start_atom_index as u64 > scalar_atom_limit {
            let excluded = start_atom_index + scalar_atom_limit as usize;
            let error = PlainScalarError::at(
                PlainScalarErrorKind::ScalarAtomLimitExceeded,
                atoms[excluded].span().start().byte_offset(),
            );
            proof {
                reveal(plain_scalar_body_spec);
                reveal(plain_error_spec);
                assert(expected == Err(error@));
            }
            return Err(error);
        }
        proof {
            reveal(plain_scalar_body_spec);
            fuel = (fuel - 1) as nat;
        }
        last_content_end = content_end_u64 as usize;
        pending_line = false;
        candidate_index += 1;
    }
    let success = finish_plain_body(
        atoms,
        start_atom_index,
        last_content_end,
        candidate_index,
        line_indentation,
        pending_line,
    );
    proof {
        reveal(plain_scalar_body_spec);
        reveal(plain_body_success_spec);
        assert(expected == Ok(success@));
    }
    Ok(success)
}

closed spec fn first_plain_content_start_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Option<u64>
    decreases fuel,
{
    if index >= end || index >= atoms.len() || index < 0 {
        None
    } else if fuel == 0 {
        None
    } else if atoms[index].kind == LexicalAtomKind::Space || atoms[index].kind
        == LexicalAtomKind::Tab || atoms[index].kind == LexicalAtomKind::LineFeed {
        first_plain_content_start_spec(atoms, index + 1, end, (fuel - 1) as nat)
    } else {
        Some(index as u64)
    }
}

fn first_plain_content_start(atoms: &[LexicalAtom], start: usize, end: usize) -> (content_start:
    Option<u64>)
    requires
        start <= end <= atoms@.len(),
    ensures
        content_start == first_plain_content_start_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
        match content_start {
            Some(index) => {
                start <= index < end && atoms[index as int]@.kind != LexicalAtomKind::Space
                    && atoms[index as int]@.kind != LexicalAtomKind::Tab
                    && atoms[index as int]@.kind != LexicalAtomKind::LineFeed && forall|later: int|
                    #![auto]
                    start <= later < end && atoms[later]@.kind != LexicalAtomKind::Space
                        && atoms[later]@.kind != LexicalAtomKind::Tab && atoms[later]@.kind
                        != LexicalAtomKind::LineFeed ==> index <= later
            },
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = first_plain_content_start_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == first_plain_content_start_spec(
                views,
                index as int,
                end as int,
                (end - index) as nat,
            ),
            expected == first_plain_content_start_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ),
            forall|prior: int|
                #![auto]
                start <= prior < index ==> atoms[prior]@.kind == LexicalAtomKind::Space
                    || atoms[prior]@.kind == LexicalAtomKind::Tab || atoms[prior]@.kind
                    == LexicalAtomKind::LineFeed,
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let kind = atoms[index].kind();
        if kind != LexicalAtomKind::Space && kind != LexicalAtomKind::Tab && kind
            != LexicalAtomKind::LineFeed {
            proof {
                reveal(first_plain_content_start_spec);
                assert(expected == Some(index as u64));
                assert forall|later: int|
                    #![auto]
                    start <= later < end && atoms[later]@.kind != LexicalAtomKind::Space
                        && atoms[later]@.kind != LexicalAtomKind::Tab && atoms[later]@.kind
                        != LexicalAtomKind::LineFeed implies index <= later by {
                    if later < index {
                        assert(atoms[later]@.kind == LexicalAtomKind::Space || atoms[later]@.kind
                            == LexicalAtomKind::Tab || atoms[later]@.kind
                            == LexicalAtomKind::LineFeed);
                    }
                }
            }
            return Some(index as u64);
        }
        proof {
            reveal(first_plain_content_start_spec);
        }
        index += 1;
    }
    proof {
        reveal(first_plain_content_start_spec);
    }
    None
}

closed spec fn block_header_indent_digit_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> Option<u8>
    decreases fuel,
{
    if index >= end || index >= atoms.len() || index < 0 || fuel == 0 {
        None
    } else if 0x31 <= atoms[index].code_point <= 0x39 {
        Some((atoms[index].code_point - 0x30) as u8)
    } else {
        block_header_indent_digit_spec(atoms, index + 1, end, (fuel - 1) as nat)
    }
}

#[allow(clippy::manual_range_contains)]
fn block_header_indent_digit(atoms: &[LexicalAtom], start: usize, end: usize) -> (digit: Option<u8>)
    requires
        start <= end <= atoms@.len(),
    ensures
        digit == block_header_indent_digit_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
        match digit {
            Some(value) => 1 <= value <= 9,
            None => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = block_header_indent_digit_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == block_header_indent_digit_spec(
                views,
                index as int,
                end as int,
                (end - index) as nat,
            ),
            expected == block_header_indent_digit_spec(
                crate::atom::lexical_atom_views_spec(atoms@),
                start as int,
                end as int,
                (end - start) as nat,
            ),
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let code_point = atoms[index].code_point();
        if 0x31 <= code_point && code_point <= 0x39 {
            let digit = (code_point - 0x30) as u8;
            proof {
                reveal(block_header_indent_digit_spec);
                assert(expected == Some(digit));
            }
            return Some(digit);
        }
        proof {
            reveal(block_header_indent_digit_spec);
        }
        index += 1;
    }
    proof {
        reveal(block_header_indent_digit_spec);
    }
    None
}

pub open spec fn plain_candidate_index_after_atom_spec(
    candidates: Seq<StructuralLexemeView>,
    index: int,
    atom_index: u64,
    fuel: nat,
) -> int
    decreases fuel,
{
    if index < candidates.len() && index >= 0 && fuel > 0 && candidates[index].start_atom_index
        < atom_index {
        plain_candidate_index_after_atom_spec(candidates, index + 1, atom_index, (fuel - 1) as nat)
    } else {
        index
    }
}

fn plain_candidate_index_after_atom(
    candidates: &[StructuralLexeme],
    start_index: usize,
    atom_index: u64,
) -> (index: usize)
    requires
        start_index <= candidates@.len(),
    ensures
        index as int == plain_candidate_index_after_atom_spec(
            crate::structural::structural_lexeme_views_spec(candidates@),
            start_index as int,
            atom_index,
            (candidates@.len() - start_index) as nat,
        ),
        start_index <= index <= candidates@.len(),
        index < candidates@.len() ==> atom_index <= candidates[index as int]@.start_atom_index,
{
    let ghost views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost expected = plain_candidate_index_after_atom_spec(
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
            expected == plain_candidate_index_after_atom_spec(
                views,
                index as int,
                atom_index,
                (candidates@.len() - index) as nat,
            ),
            expected == plain_candidate_index_after_atom_spec(
                crate::structural::structural_lexeme_views_spec(candidates@),
                start_index as int,
                atom_index,
                (candidates@.len() - start_index) as nat,
            ),
        decreases candidates.len() - index,
    {
        assert(views[index as int] == candidates[index as int]@) by {
            reveal(crate::structural::structural_lexeme_views_spec);
        }
        proof {
            reveal(plain_candidate_index_after_atom_spec);
        }
        index += 1;
    }
    proof {
        reveal(plain_candidate_index_after_atom_spec);
        if index < candidates.len() {
            assert(views[index as int] == candidates[index as int]@) by {
                reveal(crate::structural::structural_lexeme_views_spec);
            }
        }
    }
    index
}

closed spec fn candidate_is_json_mapping_colon_spec(
    atoms: Seq<LexicalAtomView>,
    candidate: StructuralLexemeView,
    context: PlainContextView,
) -> bool {
    candidate.kind == StructuralCandidateRole::Content && context.flow_depth > 0
        && context.after_node && candidate.start_atom_index < atoms.len()
        && atoms[candidate.start_atom_index as int].code_point == 0x3a
}

fn candidate_is_json_mapping_colon(
    atoms: &[LexicalAtom],
    candidate: &StructuralLexeme,
    context: PlainContext,
) -> (is_colon: bool)
    requires
        candidate@.start_atom_index < candidate@.end_atom_index,
        candidate@.end_atom_index <= atoms@.len(),
    ensures
        is_colon == candidate_is_json_mapping_colon_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            candidate@,
            context@,
        ),
{
    candidate.candidate_role() == StructuralCandidateRole::Content && context.flow_depth > 0
        && context.after_node && atoms[candidate.start_atom_index() as usize].code_point() == 0x3a
}

closed spec fn property_payload_continues_spec(
    candidate: StructuralLexemeView,
    context: PlainContextView,
) -> bool {
    context.property_payload_mode > 0 && !candidate_is_line_feed_spec(candidate) && (
    context.property_payload_mode == 4 || context.property_payload_mode == 5 || (
    !candidate_is_separation_spec(candidate) && !candidate_is_flow_start_spec(candidate)
        && !candidate_is_flow_end_spec(candidate) && candidate.kind
        != StructuralCandidateRole::FlowEntry && candidate.kind
        != StructuralCandidateRole::Indicator(YamlIndicator::MappingValue)))
}

fn property_payload_continues(candidate: &StructuralLexeme, context: PlainContext) -> (continues:
    bool)
    ensures
        continues == property_payload_continues_spec(candidate@, context@),
{
    let role = candidate.candidate_role();
    context.property_payload_mode > 0 && role != StructuralCandidateRole::LineFeed && (
    context.property_payload_mode == 4 || context.property_payload_mode == 5 || (role
        != StructuralCandidateRole::Separation && role != StructuralCandidateRole::FlowSequenceStart
        && role != StructuralCandidateRole::FlowMappingStart && role
        != StructuralCandidateRole::FlowSequenceEnd && role
        != StructuralCandidateRole::FlowMappingEnd && role != StructuralCandidateRole::FlowEntry
        && role != StructuralCandidateRole::Indicator(YamlIndicator::MappingValue)))
}

closed spec fn context_after_property_payload_spec(
    atoms: Seq<LexicalAtomView>,
    candidate: StructuralLexemeView,
    context: PlainContextView,
) -> PlainContextView {
    let begins_verbatim = context.property_payload_mode == 3
        && atoms[candidate.start_atom_index as int].code_point == 0x3c;
    let ends_verbatim = atoms[(candidate.end_atom_index - 1) as int].code_point == 0x3e;
    PlainContextView {
        at_line_start: false,
        property_payload_mode: if begins_verbatim && !ends_verbatim {
            4
        } else if context.property_payload_mode == 4 && ends_verbatim {
            3
        } else {
            context.property_payload_mode
        },
        ..context
    }
}

fn context_after_property_payload(
    atoms: &[LexicalAtom],
    candidate: &StructuralLexeme,
    context: PlainContext,
) -> (advanced: PlainContext)
    requires
        candidate@.start_atom_index < candidate@.end_atom_index,
        candidate@.end_atom_index <= atoms@.len(),
    ensures
        advanced@ == context_after_property_payload_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            candidate@,
            context@,
        ),
{
    let mut advanced = context;
    let begins_verbatim = context.property_payload_mode == 3
        && atoms[candidate.start_atom_index() as usize].code_point() == 0x3c;
    let ends_verbatim = atoms[(candidate.end_atom_index() - 1) as usize].code_point() == 0x3e;
    if begins_verbatim && !ends_verbatim {
        advanced.property_payload_mode = 4;
    } else if context.property_payload_mode == 4 && ends_verbatim {
        advanced.property_payload_mode = 3;
    }
    advanced.at_line_start = false;
    advanced
}

closed spec fn prepare_block_context_spec(
    atoms: Seq<LexicalAtomView>,
    candidate: StructuralLexemeView,
    context: PlainContextView,
) -> Result<PlainContextView, PlainScalarErrorView> {
    if context.block_mode == 2 && !context.block_line_active && !candidate_is_indentation_spec(
        candidate,
    ) && !candidate_is_line_feed_spec(candidate) {
        if context.line_indentation > context.block_parent_indentation
            || atoms[candidate.start_atom_index as int].kind == LexicalAtomKind::Tab {
            Ok(
                PlainContextView {
                    block_content_indentation: if context.block_content_indentation == 0
                        && context.line_indentation > context.block_parent_indentation {
                        context.line_indentation
                    } else {
                        context.block_content_indentation
                    },
                    block_line_active: true,
                    at_line_start: false,
                    ..context
                },
            )
        } else {
            Ok(
                PlainContextView {
                    block_mode: 0,
                    block_content_indentation: 0,
                    block_line_active: false,
                    ..context
                },
            )
        }
    } else {
        Ok(context)
    }
}

fn prepare_block_context(
    atoms: &[LexicalAtom],
    candidate: &StructuralLexeme,
    context: PlainContext,
) -> (result: Result<PlainContext, PlainScalarError>)
    requires
        candidate@.start_atom_index < candidate@.end_atom_index,
        candidate@.end_atom_index <= atoms@.len(),
    ensures
        prepare_block_context_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            candidate@,
            context@,
        ) == match result {
            Ok(prepared) => Ok(prepared@),
            Err(error) => Err(error@),
        },
{
    if context.block_mode == 2 && !context.block_line_active && candidate.candidate_role()
        != StructuralCandidateRole::Indentation && candidate.candidate_role()
        != StructuralCandidateRole::LineFeed {
        if context.line_indentation > context.block_parent_indentation
            || atoms[candidate.start_atom_index() as usize].kind() == LexicalAtomKind::Tab {
            let mut prepared = context;
            if prepared.block_content_indentation == 0 && prepared.line_indentation
                > prepared.block_parent_indentation {
                prepared.block_content_indentation = prepared.line_indentation;
            }
            prepared.block_line_active = true;
            prepared.at_line_start = false;
            Ok(prepared)
        } else {
            let mut prepared = context;
            prepared.block_mode = 0;
            prepared.block_content_indentation = 0;
            prepared.block_line_active = false;
            Ok(prepared)
        }
    } else {
        Ok(context)
    }
}

closed spec fn context_after_line_feed_spec(context: PlainContextView) -> PlainContextView {
    PlainContextView {
        line_indentation: 0,
        at_line_start: true,
        block_mode: if context.block_mode == 1 {
            2
        } else {
            context.block_mode
        },
        block_line_active: false,
        after_node: if context.property_payload_mode == 2 {
            true
        } else {
            context.after_node
        },
        property_payload_mode: 0,
        ..context
    }
}

closed spec fn context_after_plain_spec(
    context: PlainContextView,
    body: PlainBodySuccessView,
) -> PlainContextView {
    PlainContextView {
        line_indentation: body.line_indentation,
        at_line_start: body.at_line_start,
        after_node: true,
        property_payload_mode: 0,
        ..context
    }
}

closed spec fn scan_plain_tail_spec(
    atoms: Seq<LexicalAtomView>,
    candidates: Seq<StructuralLexemeView>,
    quotes: Seq<QuotedScalarView>,
    candidate_index: int,
    quote_index: int,
    context: PlainContextView,
    built: Seq<PlainScalarView>,
    scalar_limit: u64,
    scalar_atom_limit: u64,
    fuel: nat,
) -> Result<Seq<PlainScalarView>, PlainScalarErrorView>
    decreases fuel,
{
    if candidate_index >= candidates.len() {
        if quote_index == quotes.len() {
            Ok(built)
        } else {
            Err(
                PlainScalarErrorView {
                    kind: PlainScalarErrorKind::InputQuotedMismatch,
                    byte_offset: 0,
                },
            )
        }
    } else if candidate_index < 0 || quote_index < 0 || quote_index > quotes.len() || fuel == 0 {
        Err(
            PlainScalarErrorView {
                kind: PlainScalarErrorKind::InputQuotedMismatch,
                byte_offset: 0,
            },
        )
    } else {
        let candidate = candidates[candidate_index];
        if candidate.start_atom_index >= candidate.end_atom_index || candidate.end_atom_index
            > atoms.len() {
            Err(
                PlainScalarErrorView {
                    kind: PlainScalarErrorKind::InputQuotedMismatch,
                    byte_offset: candidate.byte_start,
                },
            )
        } else {
            match prepare_block_context_spec(atoms, candidate, context) {
                Err(error) => Err(error),
                Ok(prepared) => {
                    if prepared.block_mode == 1 && !candidate_is_line_feed_spec(candidate) {
                        let digit = if candidate.kind == StructuralCandidateRole::Comment {
                            None
                        } else {
                            block_header_indent_digit_spec(
                                atoms,
                                candidate.start_atom_index as int,
                                candidate.end_atom_index as int,
                                (candidate.end_atom_index - candidate.start_atom_index) as nat,
                            )
                        };
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            match digit {
                                Some(value) => PlainContextView {
                                    block_content_indentation: if prepared.block_parent_indentation
                                        <= MAX_PROFILE1_LEXICAL_ATOMS - 9 {
                                        (prepared.block_parent_indentation + value as u64) as u64
                                    } else {
                                        MAX_PROFILE1_LEXICAL_ATOMS
                                    },
                                    at_line_start: false,
                                    ..prepared
                                },
                                None => PlainContextView { at_line_start: false, ..prepared },
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if prepared.block_mode == 2 && prepared.block_line_active {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            if candidate_is_line_feed_spec(candidate) {
                                context_after_line_feed_spec(prepared)
                            } else {
                                PlainContextView { at_line_start: false, ..prepared }
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if prepared.at_line_start && prepared.flow_depth == 0
                        && prepared.line_indentation == 0
                        && atoms[candidate.start_atom_index as int].kind == LexicalAtomKind::Tab {
                        Err(
                            PlainScalarErrorView {
                                kind: PlainScalarErrorKind::TabInIndentation,
                                byte_offset:
                                    atoms[candidate.start_atom_index as int].span.start.byte_offset,
                            },
                        )
                    } else if property_payload_continues_spec(candidate, prepared) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            context_after_property_payload_spec(atoms, candidate, prepared),
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate_is_line_feed_spec(candidate) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            context_after_line_feed_spec(prepared),
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate_is_indentation_spec(candidate) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                line_indentation: indentation_width_spec(candidate),
                                at_line_start: true,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate_is_separation_spec(candidate) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                at_line_start: false,
                                after_node: if prepared.property_payload_mode == 2 {
                                    true
                                } else {
                                    prepared.after_node
                                },
                                property_payload_mode: 0,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if quote_index < quotes.len() && quotes[quote_index].start_atom_index
                        < candidate.start_atom_index {
                        Err(
                            PlainScalarErrorView {
                                kind: PlainScalarErrorKind::InputQuotedMismatch,
                                byte_offset: candidate.byte_start,
                            },
                        )
                    } else if quote_index < quotes.len() && quotes[quote_index].start_atom_index
                        == candidate.start_atom_index {
                        let quote = quotes[quote_index];
                        if quote.end_atom_index <= quote.start_atom_index || quote.end_atom_index
                            > atoms.len() {
                            Err(
                                PlainScalarErrorView {
                                    kind: PlainScalarErrorKind::InputQuotedMismatch,
                                    byte_offset: quote.byte_start,
                                },
                            )
                        } else {
                            let next = plain_candidate_index_after_atom_spec(
                                candidates,
                                candidate_index + 1,
                                quote.end_atom_index,
                                (candidates.len() - candidate_index - 1) as nat,
                            );
                            scan_plain_tail_spec(
                                atoms,
                                candidates,
                                quotes,
                                next,
                                quote_index + 1,
                                PlainContextView {
                                    at_line_start: false,
                                    after_node: true,
                                    property_payload_mode: 0,
                                    ..prepared
                                },
                                built,
                                scalar_limit,
                                scalar_atom_limit,
                                (fuel - 1) as nat,
                            )
                        }
                    } else if candidate_is_json_mapping_colon_spec(atoms, candidate, prepared)
                        && candidate.end_atom_index == candidate.start_atom_index + 1 {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                at_line_start: false,
                                after_node: false,
                                property_payload_mode: 0,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate.kind == StructuralCandidateRole::Content {
                        let content_base = if candidate_is_json_mapping_colon_spec(
                            atoms,
                            candidate,
                            prepared,
                        ) {
                            (candidate.start_atom_index + 1) as int
                        } else {
                            candidate.start_atom_index as int
                        };
                        let raw_first = first_plain_content_start_spec(
                            atoms,
                            content_base,
                            candidate.end_atom_index as int,
                            (candidate.end_atom_index as int - content_base) as nat,
                        );
                        let document_bom = match raw_first {
                            Some(start) => prepared.at_line_start && start < atoms.len()
                                && atoms[start as int].code_point == 0xfeff,
                            None => false,
                        };
                        let first = match raw_first {
                            Some(start) if document_bom && prepared.line_indentation == 0 => {
                                let after = first_plain_content_start_spec(
                                    atoms,
                                    start as int + 1,
                                    candidate.end_atom_index as int,
                                    (candidate.end_atom_index - start - 1) as nat,
                                );
                                match after {
                                    Some(next) if atoms[next as int].code_point == 0x25 => None,
                                    _ => after,
                                }
                            },
                            _ => raw_first,
                        };
                        match first {
                            None => scan_plain_tail_spec(
                                atoms,
                                candidates,
                                quotes,
                                candidate_index + 1,
                                quote_index,
                                PlainContextView {
                                    at_line_start: document_bom,
                                    property_payload_mode: if document_bom
                                        && prepared.line_indentation == 0 {
                                        5
                                    } else {
                                        prepared.property_payload_mode
                                    },
                                    ..prepared
                                },
                                built,
                                scalar_limit,
                                scalar_atom_limit,
                                (fuel - 1) as nat,
                            ),
                            Some(start) => {
                                if start >= atoms.len() {
                                    Err(
                                        PlainScalarErrorView {
                                            kind: PlainScalarErrorKind::InputQuotedMismatch,
                                            byte_offset: candidate.byte_start,
                                        },
                                    )
                                } else if !plain_start_allowed_spec(
                                    atoms,
                                    start as int,
                                    prepared.flow_depth,
                                ) {
                                    Err(
                                        PlainScalarErrorView {
                                            kind: PlainScalarErrorKind::InvalidPlainStart,
                                            byte_offset: atoms[start as int].span.start.byte_offset,
                                        },
                                    )
                                } else {
                                    match first_invalid_plain_atom_spec(
                                        atoms,
                                        start as int,
                                        candidate.end_atom_index as int,
                                        (candidate.end_atom_index - start) as nat,
                                    ) {
                                        Some(invalid) => Err(
                                            PlainScalarErrorView {
                                                kind: PlainScalarErrorKind::InvalidPlainCharacter,
                                                byte_offset:
                                                    atoms[invalid as int].span.start.byte_offset,
                                            },
                                        ),
                                        None => {
                                            let content_end = plain_content_end_spec(
                                                atoms,
                                                start as int,
                                                candidate.end_atom_index as int,
                                                prepared.flow_depth,
                                            );
                                            match content_end {
                                                None => Err(
                                                    PlainScalarErrorView {
                                                        kind:
                                                            PlainScalarErrorKind::InputQuotedMismatch,
                                                        byte_offset: candidate.byte_start,
                                                    },
                                                ),
                                                Some(end) => {
                                                    if built.len() >= scalar_limit {
                                                        Err(
                                                            PlainScalarErrorView {
                                                                kind:
                                                                    PlainScalarErrorKind::ScalarLimitExceeded,
                                                                byte_offset:
                                                                    atoms[start as int].span.start.byte_offset,
                                                            },
                                                        )
                                                    } else if scalar_atom_limit == 0 || end - start
                                                        > scalar_atom_limit {
                                                        Err(
                                                            PlainScalarErrorView {
                                                                kind:
                                                                    PlainScalarErrorKind::ScalarAtomLimitExceeded,
                                                                byte_offset: atoms[(start
                                                                    + scalar_atom_limit) as int].span.start.byte_offset,
                                                            },
                                                        )
                                                    } else {
                                                        match plain_scalar_body_spec(
                                                            atoms,
                                                            candidates,
                                                            candidate_index + 1,
                                                            start as int,
                                                            end as int,
                                                            prepared.line_indentation,
                                                            prepared.flow_depth,
                                                            prepared.line_indentation,
                                                            false,
                                                            scalar_atom_limit,
                                                            (candidates.len()
                                                                - candidate_index) as nat,
                                                        ) {
                                                            Err(error) => Err(error),
                                                            Ok(body) => scan_plain_tail_spec(
                                                                atoms,
                                                                candidates,
                                                                quotes,
                                                                body.next_candidate_index,
                                                                quote_index,
                                                                context_after_plain_spec(
                                                                    prepared,
                                                                    body,
                                                                ),
                                                                built.push(body.scalar),
                                                                scalar_limit,
                                                                scalar_atom_limit,
                                                                (fuel - 1) as nat,
                                                            ),
                                                        }
                                                    }
                                                },
                                            }
                                        },
                                    }
                                }
                            },
                        }
                    } else if candidate_is_reserved_spec(candidate) {
                        Err(
                            PlainScalarErrorView {
                                kind: PlainScalarErrorKind::ReservedIndicator,
                                byte_offset: candidate.byte_start,
                            },
                        )
                    } else if candidate.kind == StructuralCandidateRole::Indicator(
                        YamlIndicator::SingleQuotedScalar,
                    ) || candidate.kind == StructuralCandidateRole::Indicator(
                        YamlIndicator::DoubleQuotedScalar,
                    ) {
                        Err(
                            PlainScalarErrorView {
                                kind: PlainScalarErrorKind::InputQuotedMismatch,
                                byte_offset: candidate.byte_start,
                            },
                        )
                    } else if candidate_is_block_scalar_spec(candidate) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                block_mode: 1,
                                block_parent_indentation: prepared.line_indentation,
                                block_content_indentation: 0,
                                block_line_active: false,
                                at_line_start: false,
                                after_node: true,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate.kind == StructuralCandidateRole::Indicator(
                        YamlIndicator::Anchor,
                    ) || candidate.kind == StructuralCandidateRole::Indicator(YamlIndicator::Tag)
                        || candidate.kind == StructuralCandidateRole::Indicator(
                        YamlIndicator::Alias,
                    ) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                at_line_start: false,
                                property_payload_mode: if candidate.kind
                                    == StructuralCandidateRole::Indicator(YamlIndicator::Alias) {
                                    2
                                } else if candidate.kind == StructuralCandidateRole::Indicator(
                                    YamlIndicator::Tag,
                                ) {
                                    3
                                } else {
                                    1
                                },
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate_is_flow_start_spec(candidate) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                flow_depth: if prepared.flow_depth < MAX_PROFILE1_LEXICAL_ATOMS {
                                    (prepared.flow_depth + 1) as u64
                                } else {
                                    prepared.flow_depth
                                },
                                at_line_start: false,
                                after_node: false,
                                property_payload_mode: 0,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else if candidate_is_flow_end_spec(candidate) {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                flow_depth: if prepared.flow_depth > 0 {
                                    (prepared.flow_depth - 1) as u64
                                } else {
                                    0
                                },
                                at_line_start: false,
                                after_node: true,
                                property_payload_mode: 0,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    } else {
                        scan_plain_tail_spec(
                            atoms,
                            candidates,
                            quotes,
                            candidate_index + 1,
                            quote_index,
                            PlainContextView {
                                at_line_start: false,
                                after_node: if candidate.kind == StructuralCandidateRole::FlowEntry
                                    || candidate.kind == StructuralCandidateRole::Indicator(
                                    YamlIndicator::MappingValue,
                                ) || candidate.kind == StructuralCandidateRole::Indicator(
                                    YamlIndicator::BlockSequenceEntry,
                                ) || candidate.kind == StructuralCandidateRole::Indicator(
                                    YamlIndicator::ExplicitMappingKey,
                                ) {
                                    false
                                } else {
                                    prepared.after_node
                                },
                                property_payload_mode: 0,
                                ..prepared
                            },
                            built,
                            scalar_limit,
                            scalar_atom_limit,
                            (fuel - 1) as nat,
                        )
                    }
                },
            }
        }
    }
}

pub open spec fn canonical_plain_layout_limits_spec() -> crate::layout::LayoutLimitsView {
    crate::structural::canonical_layout_limits_spec()
}

pub open spec fn canonical_plain_structural_limits_spec() -> crate::structural::StructuralScanLimitsView {
    crate::structural::canonical_structural_scan_limits_spec()
}

pub open spec fn canonical_plain_quoted_limits_spec() -> crate::quoted::QuotedScalarScanLimitsView {
    crate::quoted::canonical_quoted_scalar_limits_spec()
}

pub open spec fn canonical_plain_scalar_limits_spec() -> PlainScalarScanLimitsView {
    PlainScalarScanLimitsView {
        max_scalars: MAX_PROFILE1_PLAIN_SCALARS,
        max_scalar_atoms: MAX_PROFILE1_PLAIN_SCALAR_ATOMS,
    }
}

pub fn canonical_plain_scalar_limits() -> (limits: PlainScalarScanLimits)
    ensures
        limits@ == canonical_plain_scalar_limits_spec(),
{
    PlainScalarScanLimits::new(MAX_PROFILE1_PLAIN_SCALARS, MAX_PROFILE1_PLAIN_SCALAR_ATOMS)
}

pub closed spec fn scan_profile1_plain_scalars_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    limits: PlainScalarScanLimitsView,
) -> Result<PlainScalarSourceView, PlainScalarErrorView> {
    match crate::layout::analyze_profile1_layout_spec(
        atomized,
        canonical_plain_layout_limits_spec(),
    ) {
        Err(error) => Err(
            PlainScalarErrorView {
                kind: PlainScalarErrorKind::InputQuotedMismatch,
                byte_offset: error.byte_offset,
            },
        ),
        Ok(canonical_layout) => {
            if canonical_layout != layout {
                Err(
                    PlainScalarErrorView {
                        kind: PlainScalarErrorKind::InputQuotedMismatch,
                        byte_offset: atomized.bom_bytes,
                    },
                )
            } else {
                match crate::structural::scan_profile1_structural_lexemes_spec(
                    atomized,
                    layout,
                    canonical_plain_structural_limits_spec(),
                ) {
                    Err(error) => Err(
                        PlainScalarErrorView {
                            kind: PlainScalarErrorKind::InputQuotedMismatch,
                            byte_offset: error.byte_offset,
                        },
                    ),
                    Ok(canonical_structural) => {
                        if canonical_structural != structural {
                            Err(
                                PlainScalarErrorView {
                                    kind: PlainScalarErrorKind::InputQuotedMismatch,
                                    byte_offset: atomized.bom_bytes,
                                },
                            )
                        } else {
                            match crate::quoted::scan_profile1_quoted_scalars_spec(
                                atomized,
                                layout,
                                structural,
                                canonical_plain_quoted_limits_spec(),
                            ) {
                                Err(error) => Err(
                                    PlainScalarErrorView {
                                        kind: PlainScalarErrorKind::InputQuotedMismatch,
                                        byte_offset: error.byte_offset,
                                    },
                                ),
                                Ok(canonical_quoted) => {
                                    if canonical_quoted != quoted {
                                        Err(
                                            PlainScalarErrorView {
                                                kind: PlainScalarErrorKind::InputQuotedMismatch,
                                                byte_offset: atomized.bom_bytes,
                                            },
                                        )
                                    } else {
                                        match scan_plain_tail_spec(
                                            atomized.atoms,
                                            structural.lexemes,
                                            quoted.scalars,
                                            0,
                                            0,
                                            initial_plain_context_spec(),
                                            Seq::empty(),
                                            effective_scalar_limit_spec(limits),
                                            effective_scalar_atom_limit_spec(limits),
                                            (structural.lexemes.len() + 1) as nat,
                                        ) {
                                            Err(error) => Err(error),
                                            Ok(scalars) => Ok(
                                                PlainScalarSourceView {
                                                    profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                                                    input_transformation_version:
                                                        atomized.transformation_version,
                                                    layout_transformation_version:
                                                        layout.transformation_version,
                                                    structural_transformation_version:
                                                        structural.transformation_version,
                                                    quoted_transformation_version:
                                                        quoted.transformation_version,
                                                    transformation_version:
                                                        PLAIN_SCALAR_TRANSFORMATION_VERSION,
                                                    source_len_bytes: atomized.source_len_bytes,
                                                    bom_bytes: atomized.bom_bytes,
                                                    input_atom_count: atomized.atoms.len() as u64,
                                                    input_line_count: layout.lines.len() as u64,
                                                    input_structural_lexeme_count:
                                                        structural.lexemes.len() as u64,
                                                    input_quoted_scalar_count:
                                                        quoted.scalars.len() as u64,
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
        },
    }
}

pub closed spec fn plain_scalar_source_corresponds_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
) -> bool {
    exists|limits: PlainScalarScanLimitsView|
        scan_profile1_plain_scalars_spec(atomized, layout, structural, quoted, limits) == Ok(plain)
}

pub closed spec fn plain_scalar_source_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
) -> bool {
    crate::atom::atomized_source_intrinsically_well_formed_spec(atomized)
        && crate::layout::layout_source_well_formed_spec(atomized, layout)
        && crate::structural::structural_lexeme_source_well_formed_spec(
        atomized,
        layout,
        structural,
    ) && crate::quoted::quoted_scalar_source_well_formed_spec(atomized, layout, structural, quoted)
        && plain_scalar_source_corresponds_spec(atomized, layout, structural, quoted, plain)
        && plain_scalar_ranges_well_formed_spec(atomized, plain)
}

pub proof fn lemma_plain_well_formed_has_exact_ranges(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
)
    requires
        plain_scalar_source_well_formed_spec(atomized, layout, structural, quoted, plain),
    ensures
        plain_scalar_ranges_well_formed_spec(atomized, plain),
{
    reveal(plain_scalar_source_well_formed_spec);
}

/// A canonical empty atom, structural, and quote stream admits one exact empty plain source.
pub proof fn lemma_empty_input_fits_plain_scalar_scan_limits(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    limits: PlainScalarScanLimitsView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(atomized, canonical_plain_layout_limits_spec())
            == Ok(layout),
        crate::structural::scan_profile1_structural_lexemes_spec(
            atomized,
            layout,
            canonical_plain_structural_limits_spec(),
        ) == Ok(structural),
        crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized,
            layout,
            structural,
            canonical_plain_quoted_limits_spec(),
        ) == Ok(quoted),
        atomized.atoms.len() == 0,
        structural.lexemes.len() == 0,
        quoted.scalars.len() == 0,
    ensures
        exists|source: PlainScalarSourceView|
            scan_profile1_plain_scalars_spec(atomized, layout, structural, quoted, limits) == Ok(
                source,
            ),
{
    let source = PlainScalarSourceView {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version,
        layout_transformation_version: layout.transformation_version,
        structural_transformation_version: structural.transformation_version,
        quoted_transformation_version: quoted.transformation_version,
        transformation_version: PLAIN_SCALAR_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes,
        bom_bytes: atomized.bom_bytes,
        input_atom_count: 0,
        input_line_count: layout.lines.len() as u64,
        input_structural_lexeme_count: 0,
        input_quoted_scalar_count: 0,
        scalars: Seq::empty(),
    };
    reveal(scan_profile1_plain_scalars_spec);
    reveal(scan_plain_tail_spec);
    assert(scan_profile1_plain_scalars_spec(atomized, layout, structural, quoted, limits) == Ok(
        source,
    ));
}

pub proof fn lemma_empty_plain_scan_has_no_scalars(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    quoted: QuotedScalarSourceView,
    limits: PlainScalarScanLimitsView,
    plain: PlainScalarSourceView,
)
    requires
        atomized.atoms.len() == 0,
        scan_profile1_plain_scalars_spec(atomized, layout, structural, quoted, limits) == Ok(plain),
    ensures
        plain.scalars.len() == 0,
{
    reveal(scan_profile1_plain_scalars_spec);
    reveal(scan_plain_tail_spec);
}

#[verifier::rlimit(50)]
#[verifier::spinoff_prover]
proof fn lemma_adjacent_plain_candidate_starts_increase(
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

proof fn lemma_plain_candidate_starts_increase(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    structural: StructuralLexemeSourceView,
    earlier: int,
    later: int,
)
    requires
        crate::structural::structural_lexeme_source_well_formed_spec(atomized, layout, structural),
        0 <= earlier < later < structural.lexemes.len(),
    ensures
        structural.lexemes[earlier].start_atom_index < structural.lexemes[later].start_atom_index,
    decreases later - earlier,
{
    if later == earlier + 1 {
        lemma_adjacent_plain_candidate_starts_increase(atomized, layout, structural, earlier);
    } else {
        lemma_plain_candidate_starts_increase(atomized, layout, structural, earlier, later - 1);
        lemma_adjacent_plain_candidate_starts_increase(atomized, layout, structural, later - 1);
    }
}

/// Scans canonical upstream evidence for completed plain-scalar presentation ranges.
#[verifier::rlimit(240)]
#[verifier::spinoff_prover]
pub fn scan_profile1_plain_scalars(
    atomized: &AtomizedSource,
    layout: &LayoutSource,
    structural: &StructuralLexemeSource,
    quoted: &QuotedScalarSource,
    limits: PlainScalarScanLimits,
) -> (result: Result<PlainScalarSource, PlainScalarError>)
    ensures
        scan_profile1_plain_scalars_spec(atomized@, layout@, structural@, quoted@, limits@)
            == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(source) => {
                plain_scalar_source_corresponds_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    source@,
                ) && ((crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                    && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
                    && crate::structural::structural_lexeme_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                ) && crate::quoted::quoted_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                )) ==> plain_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                    source@,
                )) && ((crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                    && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
                    && crate::structural::structural_lexeme_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                ) && crate::quoted::quoted_scalar_source_well_formed_spec(
                    atomized@,
                    layout@,
                    structural@,
                    quoted@,
                )) ==> plain_scalar_ranges_well_formed_spec(atomized@, source@))
                    && source@.scalars.len() <= limits@.max_scalars && source@.scalars.len()
                    <= MAX_PROFILE1_PLAIN_SCALARS
            },
            Err(_) => true,
        },
{
    let canonical_layout_limits = canonical_structural_layout_limits();
    let canonical_layout = match analyze_profile1_layout(atomized, canonical_layout_limits) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_plain_scalars_spec);
                reveal(canonical_plain_layout_limits_spec);
                assert(crate::layout::analyze_profile1_layout_spec(
                    atomized@,
                    canonical_plain_layout_limits_spec(),
                ) == Err(error@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_layout.same_as(layout) {
        let mismatch = PlainScalarError::at(
            PlainScalarErrorKind::InputQuotedMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_plain_scalars_spec);
            reveal(canonical_plain_layout_limits_spec);
            assert(canonical_layout@ != layout@);
            assert(crate::layout::analyze_profile1_layout_spec(
                atomized@,
                canonical_plain_layout_limits_spec(),
            ) == Ok(canonical_layout@));
        }
        return Err(mismatch);
    }
    assert(canonical_layout@ == layout@);
    proof {
        reveal(canonical_plain_layout_limits_spec);
        assert(crate::layout::analyze_profile1_layout_spec(
            atomized@,
            canonical_plain_layout_limits_spec(),
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
            let mismatch = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_plain_scalars_spec);
                reveal(canonical_plain_structural_limits_spec);
                assert(crate::structural::scan_profile1_structural_lexemes_spec(
                    atomized@,
                    layout@,
                    canonical_plain_structural_limits_spec(),
                ) == Err(error@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_structural.same_as(structural) {
        let mismatch = PlainScalarError::at(
            PlainScalarErrorKind::InputQuotedMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_plain_scalars_spec);
            reveal(canonical_plain_structural_limits_spec);
            assert(canonical_structural@ != structural@);
            assert(crate::structural::scan_profile1_structural_lexemes_spec(
                atomized@,
                layout@,
                canonical_plain_structural_limits_spec(),
            ) == Ok(canonical_structural@));
        }
        return Err(mismatch);
    }
    assert(canonical_structural@ == structural@);
    proof {
        reveal(canonical_plain_structural_limits_spec);
        assert(crate::structural::scan_profile1_structural_lexemes_spec(
            atomized@,
            layout@,
            canonical_plain_structural_limits_spec(),
        ) == Ok(structural@));
    }
    let canonical_quote_limits = canonical_quoted_scalar_limits();
    let canonical_quoted = match scan_profile1_quoted_scalars(
        atomized,
        layout,
        structural,
        canonical_quote_limits,
    ) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_plain_scalars_spec);
                reveal(canonical_plain_quoted_limits_spec);
                assert(crate::quoted::scan_profile1_quoted_scalars_spec(
                    atomized@,
                    layout@,
                    structural@,
                    canonical_plain_quoted_limits_spec(),
                ) == Err(error@));
            }
            return Err(mismatch);
        },
    };
    if !canonical_quoted.same_as(quoted) {
        let mismatch = PlainScalarError::at(
            PlainScalarErrorKind::InputQuotedMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_plain_scalars_spec);
            reveal(canonical_plain_quoted_limits_spec);
            assert(canonical_quoted@ != quoted@);
            assert(crate::quoted::scan_profile1_quoted_scalars_spec(
                atomized@,
                layout@,
                structural@,
                canonical_plain_quoted_limits_spec(),
            ) == Ok(canonical_quoted@));
        }
        return Err(mismatch);
    }
    assert(canonical_quoted@ == quoted@);
    proof {
        reveal(canonical_plain_quoted_limits_spec);
        assert(crate::quoted::scan_profile1_quoted_scalars_spec(
            atomized@,
            layout@,
            structural@,
            canonical_plain_quoted_limits_spec(),
        ) == Ok(quoted@));
    }
    let scalar_limit = if limits.max_scalars() < MAX_PROFILE1_PLAIN_SCALARS {
        limits.max_scalars()
    } else {
        MAX_PROFILE1_PLAIN_SCALARS
    };
    let scalar_atom_limit = if limits.max_scalar_atoms() < MAX_PROFILE1_PLAIN_SCALAR_ATOMS {
        limits.max_scalar_atoms()
    } else {
        MAX_PROFILE1_PLAIN_SCALAR_ATOMS
    };
    proof {
        reveal(effective_scalar_limit_spec);
        reveal(effective_scalar_atom_limit_spec);
        assert(scalar_limit == effective_scalar_limit_spec(limits@));
        assert(scalar_atom_limit == effective_scalar_atom_limit_spec(limits@));
    }
    let atoms = atomized.atoms();
    let candidates = structural.lexemes();
    let quotes = quoted.scalars();
    assert(atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
    assert(candidates@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
    if atoms.len() as u64 > MAX_PROFILE1_LEXICAL_ATOMS || candidates.len() as u64
        > MAX_PROFILE1_LEXICAL_ATOMS {
        return Err(
            PlainScalarError::at(PlainScalarErrorKind::InputQuotedMismatch, atomized.bom_bytes()),
        );
    }
    assert(atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
    assert(candidates@.len() <= MAX_PROFILE1_LEXICAL_ATOMS);
    let mut scalars: Vec<PlainScalar> = Vec::new();
    let mut candidate_index: usize = 0;
    let mut quote_index: usize = 0;
    let mut context = initial_plain_context();
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost candidate_views = crate::structural::structural_lexeme_views_spec(candidates@);
    let ghost quote_views = crate::quoted::quoted_scalar_views_spec(quotes@);
    let ghost semantic_inputs = crate::atom::atomized_source_intrinsically_well_formed_spec(
        atomized@,
    ) && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
        && crate::structural::structural_lexeme_source_well_formed_spec(
        atomized@,
        layout@,
        structural@,
    ) && crate::quoted::quoted_scalar_source_well_formed_spec(
        atomized@,
        layout@,
        structural@,
        quoted@,
    );
    let ghost mut fuel: nat = (candidates@.len() + 1) as nat;
    let ghost expected = scan_plain_tail_spec(
        atom_views,
        candidate_views,
        quote_views,
        0,
        0,
        initial_plain_context_spec(),
        Seq::empty(),
        scalar_limit,
        scalar_atom_limit,
        fuel,
    );
    proof {
        reveal(plain_scalar_views_spec);
        reveal(plain_scalar_sequence_ranges_spec);
        assert(plain_scalar_views_spec(scalars@) =~= Seq::<PlainScalarView>::empty());
        assert(plain_scalar_sequence_ranges_spec(atom_views, plain_scalar_views_spec(scalars@)));
        assert(atom_views == atomized@.atoms);
        assert(candidate_views == structural@.lexemes);
        assert(quote_views == quoted@.scalars);
        assert(expected == scan_plain_tail_spec(
            atomized@.atoms,
            structural@.lexemes,
            quoted@.scalars,
            0,
            0,
            initial_plain_context_spec(),
            Seq::empty(),
            effective_scalar_limit_spec(limits@),
            effective_scalar_atom_limit_spec(limits@),
            (structural@.lexemes.len() + 1) as nat,
        ));
    }
    assert(MAX_PROFILE1_LEXICAL_ATOMS < usize::MAX);
    while candidate_index < candidates.len()
        invariant
            candidate_index <= candidates@.len(),
            quote_index <= quotes@.len(),
            context@.flow_depth <= MAX_PROFILE1_LEXICAL_ATOMS,
            scalars@.len() <= scalar_limit,
            atoms@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            candidates@.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            atom_views == atomized@.atoms,
            candidate_views == crate::structural::structural_lexeme_views_spec(candidates@),
            candidate_views == structural@.lexemes,
            quote_views == crate::quoted::quoted_scalar_views_spec(quotes@),
            quote_views == quoted@.scalars,
            semantic_inputs == (crate::atom::atomized_source_intrinsically_well_formed_spec(
                atomized@,
            ) && crate::layout::layout_source_well_formed_spec(atomized@, layout@)
                && crate::structural::structural_lexeme_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
            ) && crate::quoted::quoted_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
            )),
            plain_scalar_views_spec(scalars@).len() == scalars@.len(),
            semantic_inputs ==> plain_scalar_sequence_ranges_spec(
                atom_views,
                plain_scalar_views_spec(scalars@),
            ),
            semantic_inputs && scalars@.len() > 0 && candidate_index < candidates@.len()
                ==> plain_scalar_views_spec(scalars@)[scalars@.len() - 1].end_atom_index
                <= candidate_views[candidate_index as int].start_atom_index,
            crate::layout::analyze_profile1_layout_spec(
                atomized@,
                canonical_plain_layout_limits_spec(),
            ) == Ok(layout@),
            crate::structural::scan_profile1_structural_lexemes_spec(
                atomized@,
                layout@,
                canonical_plain_structural_limits_spec(),
            ) == Ok(structural@),
            crate::quoted::scan_profile1_quoted_scalars_spec(
                atomized@,
                layout@,
                structural@,
                canonical_plain_quoted_limits_spec(),
            ) == Ok(quoted@),
            scalar_limit == effective_scalar_limit_spec(limits@),
            scalar_atom_limit == effective_scalar_atom_limit_spec(limits@),
            fuel >= candidates@.len() - candidate_index + 1,
            expected == scan_plain_tail_spec(
                atomized@.atoms,
                structural@.lexemes,
                quoted@.scalars,
                0,
                0,
                initial_plain_context_spec(),
                Seq::empty(),
                effective_scalar_limit_spec(limits@),
                effective_scalar_atom_limit_spec(limits@),
                (structural@.lexemes.len() + 1) as nat,
            ),
            expected == scan_plain_tail_spec(
                atom_views,
                candidate_views,
                quote_views,
                candidate_index as int,
                quote_index as int,
                context@,
                plain_scalar_views_spec(scalars@),
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
        proof {
            if semantic_inputs && scalars@.len() > 0 && candidate_index + 1 < candidates@.len() {
                lemma_adjacent_plain_candidate_starts_increase(
                    atomized@,
                    layout@,
                    structural@,
                    candidate_index as int,
                );
            }
        }
        let candidate_start_u64 = candidate.start_atom_index();
        let candidate_end_u64 = candidate.end_atom_index();
        if candidate_start_u64 >= candidate_end_u64 || candidate_end_u64 > atoms.len() as u64 {
            let error = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_plain_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_plain_scalars_spec);
            }
            return Err(error);
        }
        let candidate_start = candidate_start_u64 as usize;
        let candidate_end = candidate_end_u64 as usize;
        context =
        match prepare_block_context(atoms, candidate, context) {
            Ok(prepared) => prepared,
            Err(error) => {
                proof {
                    reveal(scan_plain_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_plain_scalars_spec);
                }
                return Err(error);
            },
        };
        let role = candidate.candidate_role();
        if context.block_mode == 1 && role != StructuralCandidateRole::LineFeed {
            if role != StructuralCandidateRole::Comment {
                if let Some(digit) = block_header_indent_digit(
                    atoms,
                    candidate_start,
                    candidate_end,
                ) {
                    if context.block_parent_indentation <= MAX_PROFILE1_LEXICAL_ATOMS - 9 {
                        context.block_content_indentation = context.block_parent_indentation
                            + digit as u64;
                    } else {
                        context.block_content_indentation = MAX_PROFILE1_LEXICAL_ATOMS;
                    }
                }
            }
            context.at_line_start = false;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if context.block_mode == 2 && context.block_line_active {
            if role == StructuralCandidateRole::LineFeed {
                context.line_indentation = 0;
                context.at_line_start = true;
                context.block_line_active = false;
                if context.property_payload_mode == 2 {
                    context.after_node = true;
                }
                context.property_payload_mode = 0;
            } else {
                context.at_line_start = false;
            }
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if context.at_line_start && context.flow_depth == 0 && context.line_indentation == 0
            && atoms[candidate_start].kind() == LexicalAtomKind::Tab {
            let error = PlainScalarError::at(
                PlainScalarErrorKind::TabInIndentation,
                atoms[candidate_start].span().start().byte_offset(),
            );
            proof {
                reveal(scan_plain_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_plain_scalars_spec);
            }
            return Err(error);
        }
        if property_payload_continues(candidate, context) {
            context = context_after_property_payload(atoms, candidate, context);
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::LineFeed {
            context.line_indentation = 0;
            context.at_line_start = true;
            if context.block_mode == 1 {
                context.block_mode = 2;
            }
            context.block_line_active = false;
            if context.property_payload_mode == 2 {
                context.after_node = true;
            }
            context.property_payload_mode = 0;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::Indentation {
            context.line_indentation = candidate_end_u64 - candidate_start_u64;
            context.at_line_start = true;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::Separation {
            if context.property_payload_mode == 2 {
                context.after_node = true;
            }
            context.property_payload_mode = 0;
            context.at_line_start = false;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if quote_index < quotes.len() && quotes[quote_index].start_atom_index()
            < candidate_start_u64 {
            assert(quote_views[quote_index as int] == quotes[quote_index as int]@) by {
                reveal(crate::quoted::quoted_scalar_views_spec);
            }
            let error = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_plain_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_plain_scalars_spec);
            }
            return Err(error);
        }
        if quote_index < quotes.len() && quotes[quote_index].start_atom_index()
            == candidate_start_u64 {
            assert(quote_views[quote_index as int] == quotes[quote_index as int]@) by {
                reveal(crate::quoted::quoted_scalar_views_spec);
            }
            let quote = &quotes[quote_index];
            if quote.end_atom_index() <= quote.start_atom_index() || quote.end_atom_index()
                > atoms.len() as u64 {
                let error = PlainScalarError::at(
                    PlainScalarErrorKind::InputQuotedMismatch,
                    quote.byte_start(),
                );
                proof {
                    reveal(scan_plain_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_plain_scalars_spec);
                }
                return Err(error);
            }
            let ghost old_candidate_index = candidate_index;
            candidate_index =
            plain_candidate_index_after_atom(
                candidates,
                candidate_index + 1,
                quote.end_atom_index(),
            );
            quote_index += 1;
            context.at_line_start = false;
            context.after_node = true;
            context.property_payload_mode = 0;
            proof {
                reveal(scan_plain_tail_spec);
                if semantic_inputs && scalars@.len() > 0 && candidate_index < candidates@.len() {
                    lemma_plain_candidate_starts_increase(
                        atomized@,
                        layout@,
                        structural@,
                        old_candidate_index as int,
                        candidate_index as int,
                    );
                }
                fuel = (fuel - 1) as nat;
            }
            continue;
        }
        let json_mapping_colon = candidate_is_json_mapping_colon(atoms, candidate, context);
        if json_mapping_colon && candidate_end_u64 == candidate_start_u64 + 1 {
            context.at_line_start = false;
            context.after_node = false;
            context.property_payload_mode = 0;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::Content {
            let content_base = if json_mapping_colon {
                candidate_start + 1
            } else {
                candidate_start
            };
            let raw_first = first_plain_content_start(atoms, content_base, candidate_end);
            let document_bom = match raw_first {
                Some(start) => context.at_line_start && atoms[start as usize].code_point()
                    == 0xfeff,
                None => false,
            };
            let first = match raw_first {
                Some(start) if document_bom && context.line_indentation == 0 => {
                    let after = first_plain_content_start(atoms, start as usize + 1, candidate_end);
                    match after {
                        Some(next) if atoms[next as usize].code_point() == 0x25 => None,
                        _ => after,
                    }
                },
                _ => raw_first,
            };
            let start_u64 = match first {
                Some(start) => start,
                None => {
                    context.at_line_start = document_bom;
                    if document_bom && context.line_indentation == 0 {
                        context.property_payload_mode = 5;
                    }
                    proof {
                        reveal(scan_plain_tail_spec);
                        fuel = (fuel - 1) as nat;
                    }
                    candidate_index += 1;
                    continue;
                },
            };
            if !plain_start_allowed(atoms, start_u64 as usize, context.flow_depth) {
                let error = PlainScalarError::at(
                    PlainScalarErrorKind::InvalidPlainStart,
                    atoms[start_u64 as usize].span().start().byte_offset(),
                );
                proof {
                    reveal(scan_plain_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_plain_scalars_spec);
                }
                return Err(error);
            }
            if let Some(invalid_u64) = first_invalid_plain_atom(
                atoms,
                start_u64 as usize,
                candidate_end,
            ) {
                let invalid = invalid_u64 as usize;
                let error = PlainScalarError::at(
                    PlainScalarErrorKind::InvalidPlainCharacter,
                    atoms[invalid].span().start().byte_offset(),
                );
                proof {
                    reveal(scan_plain_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_plain_scalars_spec);
                }
                return Err(error);
            }
            let end_u64 = match plain_content_end(
                atoms,
                start_u64 as usize,
                candidate_end,
                context.flow_depth,
            ) {
                Some(end) => end,
                None => {
                    let error = PlainScalarError::at(
                        PlainScalarErrorKind::InputQuotedMismatch,
                        candidate.byte_start(),
                    );
                    proof {
                        reveal(scan_plain_tail_spec);
                        assert(expected == Err(error@));
                        reveal(scan_profile1_plain_scalars_spec);
                    }
                    return Err(error);
                },
            };
            assert(start_u64 < end_u64);
            if scalars.len() as u64 >= scalar_limit {
                let error = PlainScalarError::at(
                    PlainScalarErrorKind::ScalarLimitExceeded,
                    atoms[start_u64 as usize].span().start().byte_offset(),
                );
                proof {
                    reveal(scan_plain_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_plain_scalars_spec);
                }
                return Err(error);
            }
            if scalar_atom_limit == 0 || end_u64 - start_u64 > scalar_atom_limit {
                let error = PlainScalarError::at(
                    PlainScalarErrorKind::ScalarAtomLimitExceeded,
                    atoms[(start_u64 + scalar_atom_limit) as usize].span().start().byte_offset(),
                );
                proof {
                    reveal(scan_plain_tail_spec);
                    assert(expected == Err(error@));
                    reveal(scan_profile1_plain_scalars_spec);
                }
                return Err(error);
            }
            let body = match scan_plain_scalar_body(
                atoms,
                candidates,
                candidate_index,
                start_u64 as usize,
                end_u64 as usize,
                context.line_indentation,
                context.flow_depth,
                scalar_atom_limit,
            ) {
                Ok(body) => body,
                Err(error) => {
                    proof {
                        reveal(scan_plain_tail_spec);
                        assert(expected == Err(error@));
                        reveal(scan_profile1_plain_scalars_spec);
                    }
                    return Err(error);
                },
            };
            let ghost old_scalars = scalars@;
            candidate_index = body.next_candidate_index;
            context.line_indentation = body.line_indentation;
            context.at_line_start = body.at_line_start;
            context.after_node = true;
            context.property_payload_mode = 0;
            proof {
                reveal(scan_plain_tail_spec);
                if semantic_inputs {
                    assert(plain_scalar_range_spec(atom_views, body.scalar@));
                    if old_scalars.len() > 0 {
                        let previous = plain_scalar_views_spec(old_scalars)[old_scalars.len() - 1];
                        assert(previous.end_atom_index <= candidate@.start_atom_index);
                        assert(candidate@.start_atom_index <= body.scalar@.start_atom_index);
                        assert(previous.end_atom_index <= body.scalar@.start_atom_index);
                        assert(0 < previous.end_atom_index);
                        assert(body.scalar@.start_atom_index < atom_views.len());
                        lemma_earlier_plain_atom_ends_before_later_atom_starts(
                            atomized@,
                            previous.end_atom_index as int - 1,
                            body.scalar@.start_atom_index as int,
                        );
                        assert(previous.byte_end <= body.scalar@.byte_start);
                    }
                    lemma_plain_scalar_sequence_ranges_push(
                        atom_views,
                        plain_scalar_views_spec(old_scalars),
                        body.scalar@,
                    );
                }
                lemma_plain_scalar_views_push(old_scalars, body.scalar);
                fuel = (fuel - 1) as nat;
            }
            scalars.push(body.scalar);
            continue;
        }
        if role == StructuralCandidateRole::Indicator(YamlIndicator::ReservedAt) || role
            == StructuralCandidateRole::Indicator(YamlIndicator::ReservedGraveAccent) {
            let error = PlainScalarError::at(
                PlainScalarErrorKind::ReservedIndicator,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_plain_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_plain_scalars_spec);
            }
            return Err(error);
        }
        if role == StructuralCandidateRole::Indicator(YamlIndicator::SingleQuotedScalar) || role
            == StructuralCandidateRole::Indicator(YamlIndicator::DoubleQuotedScalar) {
            let error = PlainScalarError::at(
                PlainScalarErrorKind::InputQuotedMismatch,
                candidate.byte_start(),
            );
            proof {
                reveal(scan_plain_tail_spec);
                assert(expected == Err(error@));
                reveal(scan_profile1_plain_scalars_spec);
            }
            return Err(error);
        }
        if role == StructuralCandidateRole::Indicator(YamlIndicator::LiteralBlockScalar) || role
            == StructuralCandidateRole::Indicator(YamlIndicator::FoldedBlockScalar) {
            context.block_mode = 1;
            context.block_parent_indentation = context.line_indentation;
            context.block_content_indentation = 0;
            context.block_line_active = false;
            context.at_line_start = false;
            context.after_node = true;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::Indicator(YamlIndicator::Anchor) || role
            == StructuralCandidateRole::Indicator(YamlIndicator::Tag) || role
            == StructuralCandidateRole::Indicator(YamlIndicator::Alias) {
            context.property_payload_mode = if role == StructuralCandidateRole::Indicator(
                YamlIndicator::Alias,
            ) {
                2
            } else if role == StructuralCandidateRole::Indicator(YamlIndicator::Tag) {
                3
            } else {
                1
            };
            context.at_line_start = false;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::FlowSequenceStart || role
            == StructuralCandidateRole::FlowMappingStart {
            if context.flow_depth < MAX_PROFILE1_LEXICAL_ATOMS {
                context.flow_depth += 1;
            }
            context.at_line_start = false;
            context.after_node = false;
            context.property_payload_mode = 0;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        if role == StructuralCandidateRole::FlowSequenceEnd || role
            == StructuralCandidateRole::FlowMappingEnd {
            if context.flow_depth > 0 {
                context.flow_depth -= 1;
            }
            context.at_line_start = false;
            context.after_node = true;
            context.property_payload_mode = 0;
            proof {
                reveal(scan_plain_tail_spec);
                fuel = (fuel - 1) as nat;
            }
            candidate_index += 1;
            continue;
        }
        context.at_line_start = false;
        if role == StructuralCandidateRole::FlowEntry || role == StructuralCandidateRole::Indicator(
            YamlIndicator::MappingValue,
        ) || role == StructuralCandidateRole::Indicator(YamlIndicator::BlockSequenceEntry) || role
            == StructuralCandidateRole::Indicator(YamlIndicator::ExplicitMappingKey) {
            context.after_node = false;
        }
        context.property_payload_mode = 0;
        proof {
            reveal(scan_plain_tail_spec);
            fuel = (fuel - 1) as nat;
        }
        candidate_index += 1;
    }
    if quote_index != quotes.len() {
        let error = PlainScalarError::at(PlainScalarErrorKind::InputQuotedMismatch, 0);
        proof {
            reveal(scan_plain_tail_spec);
            assert(expected == Err(error@));
            reveal(scan_profile1_plain_scalars_spec);
        }
        return Err(error);
    }
    let source = PlainScalarSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version(),
        layout_transformation_version: layout.transformation_version(),
        structural_transformation_version: structural.transformation_version(),
        quoted_transformation_version: quoted.transformation_version(),
        transformation_version: PLAIN_SCALAR_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes(),
        bom_bytes: atomized.bom_bytes(),
        input_atom_count: atoms.len() as u64,
        input_line_count: layout.lines().len() as u64,
        input_structural_lexeme_count: candidates.len() as u64,
        input_quoted_scalar_count: quotes.len() as u64,
        scalars,
    };
    proof {
        reveal(scan_plain_tail_spec);
        assert(expected == Ok(source@.scalars));
        reveal(scan_profile1_plain_scalars_spec);
        assert(scan_profile1_plain_scalars_spec(atomized@, layout@, structural@, quoted@, limits@)
            == Ok(source@));
        reveal(plain_scalar_source_corresponds_spec);
        assert(exists|candidate_limits: PlainScalarScanLimitsView|
            scan_profile1_plain_scalars_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                candidate_limits,
            ) == Ok(source@)) by {
            assert(scan_profile1_plain_scalars_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                limits@,
            ) == Ok(source@));
        }
        if semantic_inputs {
            assert(plain_scalar_sequence_ranges_spec(atomized@.atoms, source@.scalars));
            reveal(plain_scalar_ranges_well_formed_spec);
            assert(plain_scalar_ranges_well_formed_spec(atomized@, source@));
            reveal(plain_scalar_source_well_formed_spec);
            assert(plain_scalar_source_well_formed_spec(
                atomized@,
                layout@,
                structural@,
                quoted@,
                source@,
            ));
        }
        assert(source@.scalars.len() <= scalar_limit);
        assert(scalar_limit <= limits@.max_scalars);
        assert(scalar_limit <= MAX_PROFILE1_PLAIN_SCALARS);
    }
    Ok(source)
}

} // verus!
