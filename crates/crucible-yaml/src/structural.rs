//! Verified lossless structural lexemes for Crucible YAML profile 1.
//!
//! This stage partitions the verified line layout into directives, document markers, comments,
//! separation, structural indicators, and still-ambiguous content. It deliberately preserves
//! scalar material; the following context-sensitive token scanner owns quoted, plain, and block
//! scalar boundaries and contextual tab legality.
use crate::atom::{
    AtomizedSource, LexicalAtom, LexicalAtomKind, YamlIndicator, MAX_PROFILE1_LEXICAL_ATOMS,
};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::layout::{
    analyze_profile1_layout, LayoutLimits, LayoutSource, MAX_PROFILE1_INDENTATION_COLUMNS,
    MAX_PROFILE1_LAYOUT_LINES,
};
#[allow(unused_imports)]
use crate::layout::{LayoutLimitsView, LayoutSourceView};
use crate::utf8::CRUCIBLE_YAML_PROFILE_VERSION;
use vstd::prelude::*;

verus! {

pub const STRUCTURAL_LEXEME_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_STRUCTURAL_LEXEMES: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructuralScanLimits {
    max_lexemes: u64,
}

#[verifier::ext_equal]
pub struct StructuralScanLimitsView {
    pub max_lexemes: u64,
}

impl View for StructuralScanLimits {
    type V = StructuralScanLimitsView;

    closed spec fn view(&self) -> StructuralScanLimitsView {
        StructuralScanLimitsView { max_lexemes: self.max_lexemes }
    }
}

impl StructuralScanLimits {
    pub fn new(max_lexemes: u64) -> (limits: Self)
        ensures
            limits@.max_lexemes == max_lexemes,
    {
        Self { max_lexemes }
    }

    pub fn max_lexemes(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_lexemes,
    {
        self.max_lexemes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum StructuralScanErrorKind {
    InputLayoutMismatch,
    LexemeLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructuralScanError {
    kind: StructuralScanErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct StructuralScanErrorView {
    pub kind: StructuralScanErrorKind,
    pub byte_offset: u64,
}

impl View for StructuralScanError {
    type V = StructuralScanErrorView;

    closed spec fn view(&self) -> StructuralScanErrorView {
        StructuralScanErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl StructuralScanError {
    fn at(kind: StructuralScanErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (StructuralScanErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: StructuralScanErrorKind)
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

/// A provisional structural role retained for the completed context-sensitive YAML scanner.
///
/// These roles are not final YAML token classifications. Punctuation inside quoted, plain, or
/// block scalars may carry a comment, flow, separation, or indicator candidate role here and be
/// reinterpreted as scalar content once the following scanner has complete YAML context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum StructuralCandidateRole {
    Indentation,
    Separation,
    LineFeed,
    Comment,
    Directive,
    DocumentStart,
    DocumentEnd,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    FlowEntry,
    Indicator(YamlIndicator),
    Content,
}

type StructuralLexemeKind = StructuralCandidateRole;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A nonempty half-open atom range with one lossless structural role.
///
/// ```compile_fail
/// use crucible_yaml::{StructuralCandidateRole, StructuralLexeme};
///
/// let forged = StructuralLexeme {
///     kind: StructuralCandidateRole::Content,
///     line_number: 0,
///     start_atom_index: 9,
///     end_atom_index: 2,
///     byte_start: 8,
///     byte_end: 1,
/// };
/// ```
pub struct StructuralLexeme {
    kind: StructuralLexemeKind,
    line_number: u64,
    start_atom_index: u64,
    end_atom_index: u64,
    byte_start: u64,
    byte_end: u64,
}

#[verifier::ext_equal]
pub struct StructuralLexemeView {
    pub kind: StructuralCandidateRole,
    pub line_number: u64,
    pub start_atom_index: u64,
    pub end_atom_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
}

impl View for StructuralLexeme {
    type V = StructuralLexemeView;

    closed spec fn view(&self) -> StructuralLexemeView {
        StructuralLexemeView {
            kind: self.kind,
            line_number: self.line_number,
            start_atom_index: self.start_atom_index,
            end_atom_index: self.end_atom_index,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
        }
    }
}

impl DeepView for StructuralLexeme {
    type V = StructuralLexemeView;

    closed spec fn deep_view(&self) -> StructuralLexemeView {
        self@
    }
}

impl StructuralLexeme {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        self.kind == other.kind && self.line_number == other.line_number && self.start_atom_index
            == other.start_atom_index && self.end_atom_index == other.end_atom_index
            && self.byte_start == other.byte_start && self.byte_end == other.byte_end
    }

    pub fn candidate_role(&self) -> (role: StructuralCandidateRole)
        ensures
            role == self@.kind,
    {
        self.kind
    }

    pub fn line_number(&self) -> (line: u64)
        ensures
            line == self@.line_number,
    {
        self.line_number
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
pub struct StructuralLexemeSource {
    profile_version: u16,
    input_transformation_version: u16,
    layout_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    input_atom_count: u64,
    input_line_count: u64,
    lexemes: Vec<StructuralLexeme>,
}

#[verifier::ext_equal]
pub struct StructuralLexemeSourceView {
    pub profile_version: u16,
    pub input_transformation_version: u16,
    pub layout_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub input_atom_count: u64,
    pub input_line_count: u64,
    pub lexemes: Seq<StructuralLexemeView>,
}

pub open spec fn structural_lexeme_views_spec(lexemes: Seq<StructuralLexeme>) -> Seq<
    StructuralLexemeView,
> {
    Seq::new(lexemes.len(), |index: int| lexemes[index]@)
}

impl View for StructuralLexemeSource {
    type V = StructuralLexemeSourceView;

    closed spec fn view(&self) -> StructuralLexemeSourceView {
        StructuralLexemeSourceView {
            profile_version: self.profile_version,
            input_transformation_version: self.input_transformation_version,
            layout_transformation_version: self.layout_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            input_atom_count: self.input_atom_count,
            input_line_count: self.input_line_count,
            lexemes: structural_lexeme_views_spec(self.lexemes@),
        }
    }
}

impl StructuralLexemeSource {
    pub(crate) fn same_as(&self, other: &Self) -> (equal: bool)
        ensures
            equal == (self@ == other@),
    {
        if self.profile_version != other.profile_version || self.input_transformation_version
            != other.input_transformation_version || self.layout_transformation_version
            != other.layout_transformation_version || self.transformation_version
            != other.transformation_version || self.source_len_bytes != other.source_len_bytes
            || self.bom_bytes != other.bom_bytes || self.input_atom_count != other.input_atom_count
            || self.input_line_count != other.input_line_count {
            assert(self@ != other@);
            return false;
        }
        if self.lexemes.len() != other.lexemes.len() {
            proof {
                reveal(structural_lexeme_views_spec);
                assert(self@.lexemes.len() != other@.lexemes.len());
                assert(self@ != other@);
            }
            return false;
        }
        let mut index: usize = 0;
        while index < self.lexemes.len()
            invariant
                self.lexemes.len() == other.lexemes.len(),
                index <= self.lexemes.len(),
                forall|prior: int|
                    #![auto]
                    0 <= prior < index ==> self.lexemes[prior]@ == other.lexemes[prior]@,
            decreases self.lexemes.len() - index,
        {
            if !self.lexemes[index].same_as(&other.lexemes[index]) {
                proof {
                    reveal(structural_lexeme_views_spec);
                    assert(self.lexemes[index as int]@ != other.lexemes[index as int]@);
                    assert(self@.lexemes[index as int] != other@.lexemes[index as int]);
                    assert(self@ != other@);
                }
                return false;
            }
            index += 1;
        }
        proof {
            reveal(structural_lexeme_views_spec);
            assert(self@.lexemes =~= other@.lexemes);
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

    pub fn lexemes(&self) -> (lexemes: &[StructuralLexeme])
        ensures
            structural_lexeme_views_spec(lexemes@) == self@.lexemes,
    {
        self.lexemes.as_slice()
    }
}

#[verifier::ext_equal]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct StructuralCursorView {
    pub atom_index: int,
    pub line_number: u64,
    pub at_content_start: bool,
    pub indented: bool,
    pub previous_separation: bool,
}

#[derive(Clone, Copy)]
struct StructuralCursor {
    atom_index: usize,
    line_number: u64,
    at_content_start: bool,
    indented: bool,
    previous_separation: bool,
}

impl View for StructuralCursor {
    type V = StructuralCursorView;

    closed spec fn view(&self) -> StructuralCursorView {
        StructuralCursorView {
            atom_index: self.atom_index as int,
            line_number: self.line_number,
            at_content_start: self.at_content_start,
            indented: self.indented,
            previous_separation: self.previous_separation,
        }
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct StructuralStepView {
    kind: StructuralLexemeKind,
    line_number: u64,
    start_atom_index: int,
    end_atom_index: int,
    next: StructuralCursorView,
}

struct StructuralStep {
    kind: StructuralLexemeKind,
    line_number: u64,
    start_atom_index: usize,
    end_atom_index: usize,
    next: StructuralCursor,
}

impl View for StructuralStep {
    type V = StructuralStepView;

    closed spec fn view(&self) -> StructuralStepView {
        StructuralStepView {
            kind: self.kind,
            line_number: self.line_number,
            start_atom_index: self.start_atom_index as int,
            end_atom_index: self.end_atom_index as int,
            next: self.next@,
        }
    }
}

closed spec fn effective_lexeme_limit_spec(limits: StructuralScanLimitsView) -> u64 {
    if limits.max_lexemes < MAX_PROFILE1_STRUCTURAL_LEXEMES {
        limits.max_lexemes
    } else {
        MAX_PROFILE1_STRUCTURAL_LEXEMES
    }
}

pub closed spec fn canonical_layout_limits_spec() -> LayoutLimitsView {
    LayoutLimitsView {
        max_lines: MAX_PROFILE1_LAYOUT_LINES,
        max_indentation_columns: MAX_PROFILE1_INDENTATION_COLUMNS,
    }
}

/// Returns the canonical layout limits used to authenticate a structural scan input.
pub fn canonical_structural_layout_limits() -> (limits: LayoutLimits)
    ensures
        limits@ == canonical_layout_limits_spec(),
        limits@.max_lines == MAX_PROFILE1_LAYOUT_LINES,
        limits@.max_indentation_columns == MAX_PROFILE1_INDENTATION_COLUMNS,
{
    let limits = LayoutLimits::new(MAX_PROFILE1_LAYOUT_LINES, MAX_PROFILE1_INDENTATION_COLUMNS);
    proof {
        reveal(canonical_layout_limits_spec);
    }
    limits
}

pub open spec fn canonical_structural_scan_limits_spec() -> StructuralScanLimitsView {
    StructuralScanLimitsView { max_lexemes: MAX_PROFILE1_STRUCTURAL_LEXEMES }
}

/// Returns the absolute structural-candidate limits used to authenticate downstream input.
pub fn canonical_structural_scan_limits() -> (limits: StructuralScanLimits)
    ensures
        limits@ == canonical_structural_scan_limits_spec(),
{
    StructuralScanLimits::new(MAX_PROFILE1_STRUCTURAL_LEXEMES)
}

pub open spec fn atom_is_indicator_spec(atom: LexicalAtomView, indicator: YamlIndicator) -> bool {
    atom.kind == LexicalAtomKind::Indicator(indicator)
}

pub open spec fn atom_is_white_spec(atom: LexicalAtomView) -> bool {
    atom.kind == LexicalAtomKind::Space || atom.kind == LexicalAtomKind::Tab
}

pub open spec fn followed_by_white_break_or_end_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
) -> bool {
    index + 1 >= atoms.len() || atom_is_white_spec(atoms[index + 1]) || atoms[index + 1].kind
        == LexicalAtomKind::LineFeed
}

pub open spec fn document_start_at_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
) -> bool {
    cursor.at_content_start && !cursor.indented && cursor.atom_index + 3 <= atoms.len()
        && atom_is_indicator_spec(atoms[cursor.atom_index], YamlIndicator::BlockSequenceEntry)
        && atom_is_indicator_spec(atoms[cursor.atom_index + 1], YamlIndicator::BlockSequenceEntry)
        && atom_is_indicator_spec(atoms[cursor.atom_index + 2], YamlIndicator::BlockSequenceEntry)
        && (cursor.atom_index + 3 == atoms.len() || atom_is_white_spec(atoms[cursor.atom_index + 3])
        || atoms[cursor.atom_index + 3].kind == LexicalAtomKind::LineFeed)
}

pub open spec fn document_end_at_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
) -> bool {
    cursor.at_content_start && !cursor.indented && cursor.atom_index + 3 <= atoms.len()
        && atoms[cursor.atom_index].code_point == 0x2e && atoms[cursor.atom_index + 1].code_point
        == 0x2e && atoms[cursor.atom_index + 2].code_point == 0x2e && (cursor.atom_index + 3
        == atoms.len() || atom_is_white_spec(atoms[cursor.atom_index + 3])
        || atoms[cursor.atom_index + 3].kind == LexicalAtomKind::LineFeed)
}

pub open spec fn single_structural_kind_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
) -> Option<StructuralCandidateRole> {
    let atom = atoms[cursor.atom_index];
    if atom.kind == LexicalAtomKind::LineFeed {
        Some(StructuralLexemeKind::LineFeed)
    } else if cursor.at_content_start && atom.kind == LexicalAtomKind::Space {
        Some(StructuralLexemeKind::Indentation)
    } else if atom_is_white_spec(atom) && !(cursor.at_content_start && atom.kind
        == LexicalAtomKind::Tab) {
        Some(StructuralLexemeKind::Separation)
    } else if atom_is_indicator_spec(atom, YamlIndicator::Comment) && (cursor.at_content_start
        || cursor.previous_separation) {
        Some(StructuralLexemeKind::Comment)
    } else {
        match atom.kind {
            LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart) => {
                Some(StructuralLexemeKind::FlowSequenceStart)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd) => {
                Some(StructuralLexemeKind::FlowSequenceEnd)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart) => {
                Some(StructuralLexemeKind::FlowMappingStart)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd) => {
                Some(StructuralLexemeKind::FlowMappingEnd)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) => {
                Some(StructuralLexemeKind::FlowEntry)
            },
            LexicalAtomKind::Indicator(indicator) => {
                let quote_candidate = indicator == YamlIndicator::SingleQuotedScalar || indicator
                    == YamlIndicator::DoubleQuotedScalar;
                let rejected_by_context = !quote_candidate && (indicator == YamlIndicator::Comment
                    || ((indicator == YamlIndicator::BlockSequenceEntry || indicator
                    == YamlIndicator::ExplicitMappingKey) && (!(cursor.at_content_start
                    || cursor.previous_separation) || !followed_by_white_break_or_end_spec(
                    atoms,
                    cursor.atom_index,
                ))) || (indicator == YamlIndicator::MappingValue
                    && !followed_by_white_break_or_end_spec(atoms, cursor.atom_index)));
                if rejected_by_context {
                    None
                } else if quote_candidate || indicator == YamlIndicator::MappingValue
                    || cursor.at_content_start || cursor.previous_separation {
                    Some(StructuralLexemeKind::Indicator(indicator))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

pub open spec fn line_tail_end_spec(atoms: Seq<LexicalAtomView>, index: int) -> int
    decreases atoms.len() - index,
{
    if index < atoms.len() && atoms[index].kind != LexicalAtomKind::LineFeed {
        line_tail_end_spec(atoms, index + 1)
    } else {
        index
    }
}

pub open spec fn indentation_run_end_spec(atoms: Seq<LexicalAtomView>, index: int) -> int
    decreases atoms.len() - index,
{
    if index < atoms.len() && atoms[index].kind == LexicalAtomKind::Space {
        indentation_run_end_spec(atoms, index + 1)
    } else {
        index
    }
}

pub open spec fn separation_run_end_spec(atoms: Seq<LexicalAtomView>, index: int) -> int
    decreases atoms.len() - index,
{
    if index < atoms.len() && atom_is_white_spec(atoms[index]) {
        separation_run_end_spec(atoms, index + 1)
    } else {
        index
    }
}

pub open spec fn content_run_end_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
) -> int
    decreases atoms.len() - cursor.atom_index,
{
    if cursor.atom_index < atoms.len() && single_structural_kind_spec(atoms, cursor).is_none() {
        content_run_end_spec(
            atoms,
            StructuralCursorView {
                atom_index: cursor.atom_index + 1,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: cursor.indented,
                previous_separation: false,
            },
        )
    } else {
        cursor.atom_index
    }
}

closed spec fn next_structural_step_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
) -> StructuralStepView {
    let atom = atoms[cursor.atom_index];
    if atom.kind == LexicalAtomKind::LineFeed {
        StructuralStepView {
            kind: StructuralLexemeKind::LineFeed,
            line_number: cursor.line_number,
            start_atom_index: cursor.atom_index,
            end_atom_index: cursor.atom_index + 1,
            next: StructuralCursorView {
                atom_index: cursor.atom_index + 1,
                line_number: (cursor.line_number + 1) as u64,
                at_content_start: true,
                indented: false,
                previous_separation: false,
            },
        }
    } else if cursor.at_content_start && !cursor.indented && atom_is_indicator_spec(
        atom,
        YamlIndicator::Directive,
    ) {
        let end = line_tail_end_spec(atoms, cursor.atom_index);
        StructuralStepView {
            kind: StructuralLexemeKind::Directive,
            line_number: cursor.line_number,
            start_atom_index: cursor.atom_index,
            end_atom_index: end,
            next: StructuralCursorView {
                atom_index: end,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: false,
                previous_separation: false,
            },
        }
    } else if document_start_at_spec(atoms, cursor) {
        StructuralStepView {
            kind: StructuralLexemeKind::DocumentStart,
            line_number: cursor.line_number,
            start_atom_index: cursor.atom_index,
            end_atom_index: cursor.atom_index + 3,
            next: StructuralCursorView {
                atom_index: cursor.atom_index + 3,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: false,
                previous_separation: false,
            },
        }
    } else if document_end_at_spec(atoms, cursor) {
        StructuralStepView {
            kind: StructuralLexemeKind::DocumentEnd,
            line_number: cursor.line_number,
            start_atom_index: cursor.atom_index,
            end_atom_index: cursor.atom_index + 3,
            next: StructuralCursorView {
                atom_index: cursor.atom_index + 3,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: false,
                previous_separation: false,
            },
        }
    } else {
        match single_structural_kind_spec(atoms, cursor) {
            Some(StructuralLexemeKind::Indentation) => {
                let end = indentation_run_end_spec(atoms, cursor.atom_index);
                StructuralStepView {
                    kind: StructuralLexemeKind::Indentation,
                    line_number: cursor.line_number,
                    start_atom_index: cursor.atom_index,
                    end_atom_index: end,
                    next: StructuralCursorView {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: true,
                        indented: true,
                        previous_separation: true,
                    },
                }
            },
            Some(StructuralLexemeKind::Separation) => {
                let end = separation_run_end_spec(atoms, cursor.atom_index);
                StructuralStepView {
                    kind: StructuralLexemeKind::Separation,
                    line_number: cursor.line_number,
                    start_atom_index: cursor.atom_index,
                    end_atom_index: end,
                    next: StructuralCursorView {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: true,
                    },
                }
            },
            Some(StructuralLexemeKind::Comment) => {
                let end = line_tail_end_spec(atoms, cursor.atom_index);
                StructuralStepView {
                    kind: StructuralLexemeKind::Comment,
                    line_number: cursor.line_number,
                    start_atom_index: cursor.atom_index,
                    end_atom_index: end,
                    next: StructuralCursorView {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: false,
                    },
                }
            },
            Some(kind) => StructuralStepView {
                kind,
                line_number: cursor.line_number,
                start_atom_index: cursor.atom_index,
                end_atom_index: cursor.atom_index + 1,
                next: StructuralCursorView {
                    atom_index: cursor.atom_index + 1,
                    line_number: cursor.line_number,
                    at_content_start: false,
                    indented: cursor.indented,
                    previous_separation: false,
                },
            },
            None => {
                let end = content_run_end_spec(
                    atoms,
                    StructuralCursorView {
                        atom_index: cursor.atom_index + 1,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: false,
                    },
                );
                StructuralStepView {
                    kind: StructuralLexemeKind::Content,
                    line_number: cursor.line_number,
                    start_atom_index: cursor.atom_index,
                    end_atom_index: end,
                    next: StructuralCursorView {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: false,
                    },
                }
            },
        }
    }
}

closed spec fn lexeme_for_step_spec(
    atoms: Seq<LexicalAtomView>,
    step: StructuralStepView,
) -> StructuralLexemeView {
    StructuralLexemeView {
        kind: step.kind,
        line_number: step.line_number,
        start_atom_index: step.start_atom_index as u64,
        end_atom_index: step.end_atom_index as u64,
        byte_start: atoms[step.start_atom_index].span.start.byte_offset,
        byte_end: atoms[step.end_atom_index - 1].span.end.byte_offset,
    }
}

proof fn lemma_line_tail_end_bounds(atoms: Seq<LexicalAtomView>, index: int)
    requires
        0 <= index < atoms.len(),
        atoms[index].kind != LexicalAtomKind::LineFeed,
    ensures
        index < line_tail_end_spec(atoms, index) <= atoms.len(),
    decreases atoms.len() - index,
{
    assert(line_tail_end_spec(atoms, index) == line_tail_end_spec(atoms, index + 1));
    if index + 1 < atoms.len() && atoms[index + 1].kind != LexicalAtomKind::LineFeed {
        lemma_line_tail_end_bounds(atoms, index + 1);
    } else {
        assert(line_tail_end_spec(atoms, index + 1) == index + 1);
    }
}

proof fn lemma_indentation_run_end_bounds(atoms: Seq<LexicalAtomView>, index: int)
    requires
        0 <= index < atoms.len(),
        atoms[index].kind == LexicalAtomKind::Space,
    ensures
        index < indentation_run_end_spec(atoms, index) <= atoms.len(),
    decreases atoms.len() - index,
{
    assert(indentation_run_end_spec(atoms, index) == indentation_run_end_spec(atoms, index + 1));
    if index + 1 < atoms.len() && atoms[index + 1].kind == LexicalAtomKind::Space {
        lemma_indentation_run_end_bounds(atoms, index + 1);
    } else {
        assert(indentation_run_end_spec(atoms, index + 1) == index + 1);
    }
}

proof fn lemma_separation_run_end_bounds(atoms: Seq<LexicalAtomView>, index: int)
    requires
        0 <= index < atoms.len(),
        atom_is_white_spec(atoms[index]),
    ensures
        index < separation_run_end_spec(atoms, index) <= atoms.len(),
    decreases atoms.len() - index,
{
    assert(separation_run_end_spec(atoms, index) == separation_run_end_spec(atoms, index + 1));
    if index + 1 < atoms.len() && atom_is_white_spec(atoms[index + 1]) {
        lemma_separation_run_end_bounds(atoms, index + 1);
    } else {
        assert(separation_run_end_spec(atoms, index + 1) == index + 1);
    }
}

proof fn lemma_content_run_end_bounds(atoms: Seq<LexicalAtomView>, cursor: StructuralCursorView)
    requires
        0 <= cursor.atom_index <= atoms.len(),
        !cursor.at_content_start,
        !cursor.previous_separation,
    ensures
        cursor.atom_index <= content_run_end_spec(atoms, cursor) <= atoms.len(),
    decreases atoms.len() - cursor.atom_index,
{
    if cursor.atom_index < atoms.len() && single_structural_kind_spec(atoms, cursor).is_none() {
        lemma_content_run_end_bounds(
            atoms,
            StructuralCursorView {
                atom_index: cursor.atom_index + 1,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: cursor.indented,
                previous_separation: false,
            },
        );
    }
}

proof fn lemma_next_structural_step_progress(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
)
    requires
        0 <= cursor.atom_index < atoms.len(),
        cursor.line_number < MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        ({
            let step = next_structural_step_spec(atoms, cursor);
            step.start_atom_index == cursor.atom_index && cursor.atom_index < step.end_atom_index
                <= atoms.len() && step.next.atom_index == step.end_atom_index && step.line_number
                == cursor.line_number && cursor.line_number <= step.next.line_number
                && step.next.line_number <= cursor.line_number + 1
        }),
{
    let atom = atoms[cursor.atom_index];
    if atom.kind == LexicalAtomKind::LineFeed {
    } else if cursor.at_content_start && !cursor.indented && atom_is_indicator_spec(
        atom,
        YamlIndicator::Directive,
    ) {
        lemma_line_tail_end_bounds(atoms, cursor.atom_index);
    } else if document_start_at_spec(atoms, cursor) {
    } else if document_end_at_spec(atoms, cursor) {
    } else {
        match single_structural_kind_spec(atoms, cursor) {
            Some(StructuralLexemeKind::Indentation) => {
                lemma_indentation_run_end_bounds(atoms, cursor.atom_index);
            },
            Some(StructuralLexemeKind::Separation) => {
                lemma_separation_run_end_bounds(atoms, cursor.atom_index);
            },
            Some(StructuralLexemeKind::Comment) => {
                lemma_line_tail_end_bounds(atoms, cursor.atom_index);
            },
            Some(_) => {},
            None => {
                let next = StructuralCursorView {
                    atom_index: cursor.atom_index + 1,
                    line_number: cursor.line_number,
                    at_content_start: false,
                    indented: cursor.indented,
                    previous_separation: false,
                };
                lemma_content_run_end_bounds(atoms, next);
            },
        }
    }
}

closed spec fn structural_scan_tail_spec(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
    built: Seq<StructuralLexemeView>,
    lexeme_limit: u64,
    fuel: nat,
) -> Result<Seq<StructuralLexemeView>, StructuralScanErrorView>
    decreases fuel,
{
    if cursor.atom_index < 0 {
        Err(
            StructuralScanErrorView {
                kind: StructuralScanErrorKind::InputLayoutMismatch,
                byte_offset: 0,
            },
        )
    } else if cursor.atom_index >= atoms.len() {
        Ok(built)
    } else if fuel == 0 {
        Err(
            StructuralScanErrorView {
                kind: StructuralScanErrorKind::InputLayoutMismatch,
                byte_offset: atoms[cursor.atom_index].span.start.byte_offset,
            },
        )
    } else {
        let step = next_structural_step_spec(atoms, cursor);
        if built.len() >= lexeme_limit {
            Err(
                StructuralScanErrorView {
                    kind: StructuralScanErrorKind::LexemeLimitExceeded,
                    byte_offset: atoms[cursor.atom_index].span.start.byte_offset,
                },
            )
        } else {
            structural_scan_tail_spec(
                atoms,
                step.next,
                built.push(lexeme_for_step_spec(atoms, step)),
                lexeme_limit,
                (fuel - 1) as nat,
            )
        }
    }
}

pub closed spec fn scan_profile1_structural_lexemes_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    limits: StructuralScanLimitsView,
) -> Result<StructuralLexemeSourceView, StructuralScanErrorView> {
    match crate::layout::analyze_profile1_layout_spec(atomized, canonical_layout_limits_spec()) {
        Err(error) => Err(
            StructuralScanErrorView {
                kind: StructuralScanErrorKind::InputLayoutMismatch,
                byte_offset: error.byte_offset,
            },
        ),
        Ok(canonical) => {
            if canonical != layout {
                Err(
                    StructuralScanErrorView {
                        kind: StructuralScanErrorKind::InputLayoutMismatch,
                        byte_offset: atomized.bom_bytes,
                    },
                )
            } else {
                let cursor = StructuralCursorView {
                    atom_index: 0,
                    line_number: 0,
                    at_content_start: true,
                    indented: false,
                    previous_separation: false,
                };
                match structural_scan_tail_spec(
                    atomized.atoms,
                    cursor,
                    Seq::empty(),
                    effective_lexeme_limit_spec(limits),
                    atomized.atoms.len(),
                ) {
                    Ok(lexemes) => Ok(
                        StructuralLexemeSourceView {
                            profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                            input_transformation_version: atomized.transformation_version,
                            layout_transformation_version: layout.transformation_version,
                            transformation_version: STRUCTURAL_LEXEME_TRANSFORMATION_VERSION,
                            source_len_bytes: atomized.source_len_bytes,
                            bom_bytes: atomized.bom_bytes,
                            input_atom_count: atomized.atoms.len() as u64,
                            input_line_count: layout.lines.len() as u64,
                            lexemes,
                        },
                    ),
                    Err(error) => Err(error),
                }
            }
        },
    }
}

pub closed spec fn structural_lexeme_source_corresponds_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    lexemes: StructuralLexemeSourceView,
) -> bool {
    exists|limits: StructuralScanLimitsView|
        scan_profile1_structural_lexemes_spec(atomized, layout, limits) == Ok(lexemes)
}

/// One candidate range is nonempty, bounded, and carries the exact boundary-atom byte span.
pub open spec fn structural_candidate_range_spec(
    atoms: Seq<LexicalAtomView>,
    candidate: StructuralLexemeView,
) -> bool {
    candidate.start_atom_index < candidate.end_atom_index && candidate.end_atom_index <= atoms.len()
        && candidate.byte_start == atoms[candidate.start_atom_index as int].span.start.byte_offset
        && candidate.byte_end == atoms[(candidate.end_atom_index - 1) as int].span.end.byte_offset
        && match candidate.kind {
        StructuralCandidateRole::Indicator(indicator) => {
            atoms[candidate.start_atom_index as int].kind == LexicalAtomKind::Indicator(indicator)
        },
        _ => true,
    }
}

/// A candidate prefix exactly and monotonically partitions atoms through `consumed_atoms`.
pub open spec fn structural_candidate_prefix_partition_spec(
    atoms: Seq<LexicalAtomView>,
    candidates: Seq<StructuralLexemeView>,
    consumed_atoms: int,
) -> bool {
    0 <= consumed_atoms <= atoms.len() && (candidates.len() == 0 ==> consumed_atoms == 0) && (
    candidates.len() > 0 ==> candidates[0].start_atom_index == 0 && candidates[candidates.len()
        - 1].end_atom_index == consumed_atoms) && forall|index: int|
        0 <= index < candidates.len() ==> structural_candidate_range_spec(
            atoms,
            #[trigger] candidates[index],
        ) && (index > 0 ==> candidates[index - 1].end_atom_index
            == candidates[index].start_atom_index && candidates[index - 1].byte_end
            == candidates[index].byte_start && candidates[index - 1].line_number
            <= candidates[index].line_number)
}

/// The completed candidate source exactly partitions every atom and original post-BOM byte.
pub open spec fn structural_lexeme_partition_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    source: StructuralLexemeSourceView,
) -> bool {
    source.source_len_bytes == atomized.source_len_bytes && source.bom_bytes == atomized.bom_bytes
        && source.input_atom_count == atomized.atoms.len() && source.input_line_count
        == layout.lines.len() && structural_candidate_prefix_partition_spec(
        atomized.atoms,
        source.lexemes,
        atomized.atoms.len() as int,
    ) && (atomized.atoms.len() == 0 ==> source.lexemes.len() == 0 && source.source_len_bytes
        == source.bom_bytes) && (atomized.atoms.len() > 0 ==> source.lexemes.len() > 0
        && source.lexemes[0].byte_start == source.bom_bytes && source.lexemes[source.lexemes.len()
        - 1].byte_end == source.source_len_bytes)
}

pub closed spec fn structural_lexeme_source_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    lexemes: StructuralLexemeSourceView,
) -> bool {
    crate::atom::atomized_source_intrinsically_well_formed_spec(atomized)
        && crate::layout::layout_source_well_formed_spec(atomized, layout)
        && structural_lexeme_source_corresponds_spec(atomized, layout, lexemes)
        && structural_lexeme_partition_spec(atomized, layout, lexemes)
}

/// Semantic structural validity cannot hide an invalid line-layout input.
pub proof fn lemma_structural_well_formed_requires_layout(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    lexemes: StructuralLexemeSourceView,
)
    requires
        structural_lexeme_source_well_formed_spec(atomized, layout, lexemes),
    ensures
        crate::layout::layout_source_well_formed_spec(atomized, layout),
{
    reveal(structural_lexeme_source_well_formed_spec);
}

/// Semantic structural validity exposes the full lossless partition contract to downstream proofs.
pub proof fn lemma_structural_well_formed_has_exact_partition(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    lexemes: StructuralLexemeSourceView,
)
    requires
        structural_lexeme_source_well_formed_spec(atomized, layout, lexemes),
    ensures
        structural_lexeme_partition_spec(atomized, layout, lexemes),
{
    reveal(structural_lexeme_source_well_formed_spec);
}

fn atom_is_indicator(atom: &LexicalAtom, indicator: YamlIndicator) -> (matches: bool)
    ensures
        matches == (atom@.kind == LexicalAtomKind::Indicator(indicator)),
{
    atom.is_indicator(indicator)
}

fn atom_is_white(atom: &LexicalAtom) -> (white: bool)
    ensures
        white == (atom@.kind == LexicalAtomKind::Space || atom@.kind == LexicalAtomKind::Tab),
{
    atom.is_white()
}

fn followed_by_white_break_or_end(atoms: &[LexicalAtom], index: usize) -> (followed: bool)
    requires
        index < atoms@.len(),
    ensures
        followed == followed_by_white_break_or_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            index as int,
        ),
{
    let followed = if index + 1 >= atoms.len() {
        true
    } else {
        atom_is_white(&atoms[index + 1]) || atoms[index + 1].kind() == LexicalAtomKind::LineFeed
    };
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        assert(followed == followed_by_white_break_or_end_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            index as int,
        ));
    }
    followed
}

fn document_start_at(atoms: &[LexicalAtom], cursor: StructuralCursor) -> (matches: bool)
    requires
        cursor@.atom_index < atoms@.len(),
    ensures
        matches == document_start_at_spec(crate::atom::lexical_atom_views_spec(atoms@), cursor@),
{
    let index = cursor.atom_index;
    let matches = cursor.at_content_start && !cursor.indented && atoms.len() - index >= 3
        && atom_is_indicator(&atoms[index], YamlIndicator::BlockSequenceEntry) && atom_is_indicator(
        &atoms[index + 1],
        YamlIndicator::BlockSequenceEntry,
    ) && atom_is_indicator(&atoms[index + 2], YamlIndicator::BlockSequenceEntry) && (atoms.len()
        - index == 3 || atom_is_white(&atoms[index + 3]) || atoms[index + 3].kind()
        == LexicalAtomKind::LineFeed);
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        assert(matches == document_start_at_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            cursor@,
        ));
    }
    matches
}

fn document_end_at(atoms: &[LexicalAtom], cursor: StructuralCursor) -> (matches: bool)
    requires
        cursor@.atom_index < atoms@.len(),
    ensures
        matches == document_end_at_spec(crate::atom::lexical_atom_views_spec(atoms@), cursor@),
{
    let index = cursor.atom_index;
    let matches = cursor.at_content_start && !cursor.indented && atoms.len() - index >= 3
        && atoms[index].code_point() == 0x2e && atoms[index + 1].code_point() == 0x2e && atoms[index
        + 2].code_point() == 0x2e && (atoms.len() - index == 3 || atom_is_white(&atoms[index + 3])
        || atoms[index + 3].kind() == LexicalAtomKind::LineFeed);
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        assert(matches == document_end_at_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            cursor@,
        ));
    }
    matches
}

fn single_structural_kind(atoms: &[LexicalAtom], cursor: StructuralCursor) -> (kind: Option<
    StructuralLexemeKind,
>)
    requires
        cursor@.atom_index < atoms@.len(),
    ensures
        kind == single_structural_kind_spec(crate::atom::lexical_atom_views_spec(atoms@), cursor@),
{
    let index = cursor.atom_index;
    let atom = &atoms[index];
    let atom_kind = atom.kind();
    let result = if atom_kind == LexicalAtomKind::LineFeed {
        Some(StructuralLexemeKind::LineFeed)
    } else if cursor.at_content_start && atom_kind == LexicalAtomKind::Space {
        Some(StructuralLexemeKind::Indentation)
    } else if atom_is_white(atom) && !(cursor.at_content_start && atom_kind
        == LexicalAtomKind::Tab) {
        Some(StructuralLexemeKind::Separation)
    } else if atom_is_indicator(atom, YamlIndicator::Comment) && (cursor.at_content_start
        || cursor.previous_separation) {
        Some(StructuralLexemeKind::Comment)
    } else {
        match atom_kind {
            LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart) => {
                Some(StructuralLexemeKind::FlowSequenceStart)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd) => {
                Some(StructuralLexemeKind::FlowSequenceEnd)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart) => {
                Some(StructuralLexemeKind::FlowMappingStart)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd) => {
                Some(StructuralLexemeKind::FlowMappingEnd)
            },
            LexicalAtomKind::Indicator(YamlIndicator::FlowEntry) => {
                Some(StructuralLexemeKind::FlowEntry)
            },
            LexicalAtomKind::Indicator(indicator) => {
                let quote_candidate = indicator == YamlIndicator::SingleQuotedScalar || indicator
                    == YamlIndicator::DoubleQuotedScalar;
                let rejected_by_context = !quote_candidate && (indicator == YamlIndicator::Comment
                    || ((indicator == YamlIndicator::BlockSequenceEntry || indicator
                    == YamlIndicator::ExplicitMappingKey) && (!(cursor.at_content_start
                    || cursor.previous_separation) || !followed_by_white_break_or_end(
                    atoms,
                    index,
                ))) || (indicator == YamlIndicator::MappingValue && !followed_by_white_break_or_end(
                    atoms,
                    index,
                )));
                if rejected_by_context {
                    None
                } else if quote_candidate || indicator == YamlIndicator::MappingValue
                    || cursor.at_content_start || cursor.previous_separation {
                    Some(StructuralLexemeKind::Indicator(indicator))
                } else {
                    None
                }
            },
            _ => None,
        }
    };
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        assert(result == single_structural_kind_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            cursor@,
        ));
    }
    result
}

fn line_tail_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start < atoms.len(),
        atoms[start as int]@.kind != LexicalAtomKind::LineFeed,
    ensures
        start < end <= atoms@.len(),
        end == line_tail_end_spec(crate::atom::lexical_atom_views_spec(atoms@), start as int),
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let mut end = start;
    let mut stopped = false;
    while end < atoms.len() && !stopped
        invariant
            start <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            line_tail_end_spec(views, start as int) == line_tail_end_spec(views, end as int),
            stopped ==> end < atoms@.len() && views[end as int].kind == LexicalAtomKind::LineFeed,
        decreases
                atoms.len() - end,
                if stopped {
                    0int
                } else {
                    1int
                },
    {
        let current_kind = atoms[end].kind();
        assert(current_kind == atoms[end as int]@.kind);
        assert(views[end as int] == atoms[end as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if current_kind == LexicalAtomKind::LineFeed {
            stopped = true;
        } else {
            assert(views[end as int].kind != LexicalAtomKind::LineFeed);
            assert(line_tail_end_spec(views, end as int) == line_tail_end_spec(
                views,
                (end + 1) as int,
            ));
            end += 1;
        }
    }
    assert(end == atoms.len() || stopped);
    if stopped {
        assert(line_tail_end_spec(views, end as int) == end as int);
    } else {
        assert(end == atoms.len());
        assert(line_tail_end_spec(views, end as int) == end as int);
    }
    assert(line_tail_end_spec(views, start as int) == end as int);
    assert(start < end) by {
        if start == end {
            assert(stopped);
            assert(views[start as int].kind == LexicalAtomKind::LineFeed);
            assert(atoms[start as int]@.kind != LexicalAtomKind::LineFeed);
        }
    }
    end
}

fn indentation_run_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start < atoms.len(),
        atoms[start as int]@.kind == LexicalAtomKind::Space,
    ensures
        start < end <= atoms@.len(),
        end == indentation_run_end_spec(crate::atom::lexical_atom_views_spec(atoms@), start as int),
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let mut end = start;
    let mut stopped = false;
    while end < atoms.len() && !stopped
        invariant
            start <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            indentation_run_end_spec(views, start as int) == indentation_run_end_spec(
                views,
                end as int,
            ),
            stopped ==> end < atoms@.len() && views[end as int].kind != LexicalAtomKind::Space,
        decreases
                atoms.len() - end,
                if stopped {
                    0int
                } else {
                    1int
                },
    {
        let current_kind = atoms[end].kind();
        assert(current_kind == atoms[end as int]@.kind);
        assert(views[end as int] == atoms[end as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if current_kind != LexicalAtomKind::Space {
            stopped = true;
        } else {
            assert(views[end as int].kind == LexicalAtomKind::Space);
            assert(indentation_run_end_spec(views, end as int) == indentation_run_end_spec(
                views,
                (end + 1) as int,
            ));
            end += 1;
        }
    }
    assert(end == atoms.len() || stopped);
    if stopped {
        assert(indentation_run_end_spec(views, end as int) == end as int);
    } else {
        assert(end == atoms.len());
        assert(indentation_run_end_spec(views, end as int) == end as int);
    }
    assert(indentation_run_end_spec(views, start as int) == end as int);
    assert(start < end) by {
        if start == end {
            assert(stopped);
            assert(views[start as int].kind != LexicalAtomKind::Space);
            assert(atoms[start as int]@.kind == LexicalAtomKind::Space);
        }
    }
    end
}

fn separation_run_end(atoms: &[LexicalAtom], start: usize) -> (end: usize)
    requires
        start < atoms.len(),
        atom_is_white_spec(crate::atom::lexical_atom_views_spec(atoms@)[start as int]),
    ensures
        start < end <= atoms@.len(),
        end == separation_run_end_spec(crate::atom::lexical_atom_views_spec(atoms@), start as int),
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let mut end = start;
    let mut stopped = false;
    while end < atoms.len() && !stopped
        invariant
            start <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            separation_run_end_spec(views, start as int) == separation_run_end_spec(
                views,
                end as int,
            ),
            stopped ==> end < atoms@.len() && !atom_is_white_spec(views[end as int]),
        decreases
                atoms.len() - end,
                if stopped {
                    0int
                } else {
                    1int
                },
    {
        let current_is_white = atom_is_white(&atoms[end]);
        assert(views[end as int] == atoms[end as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        if !current_is_white {
            stopped = true;
        } else {
            assert(atom_is_white_spec(atoms[end as int]@));
            assert(atom_is_white_spec(views[end as int]));
            assert(separation_run_end_spec(views, end as int) == separation_run_end_spec(
                views,
                (end + 1) as int,
            ));
            end += 1;
        }
    }
    assert(end == atoms.len() || stopped);
    if stopped {
        assert(separation_run_end_spec(views, end as int) == end as int);
    } else {
        assert(end == atoms.len());
        assert(separation_run_end_spec(views, end as int) == end as int);
    }
    assert(separation_run_end_spec(views, start as int) == end as int);
    assert(start < end) by {
        if start == end {
            assert(stopped);
            assert(!atom_is_white_spec(views[start as int]));
        }
    }
    end
}

fn content_run_end(atoms: &[LexicalAtom], start: StructuralCursor) -> (end: usize)
    requires
        start@.atom_index <= atoms@.len(),
        !start@.at_content_start,
        !start@.previous_separation,
    ensures
        start@.atom_index <= end <= atoms@.len(),
        end == content_run_end_spec(crate::atom::lexical_atom_views_spec(atoms@), start@),
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let mut cursor = start;
    while cursor.atom_index < atoms.len() && single_structural_kind(atoms, cursor).is_none()
        invariant
            start@.atom_index <= cursor@.atom_index,
            cursor@.atom_index <= atoms@.len(),
            cursor@.line_number == start@.line_number,
            !cursor@.at_content_start,
            cursor@.indented == start@.indented,
            !cursor@.previous_separation,
            views == crate::atom::lexical_atom_views_spec(atoms@),
            content_run_end_spec(views, start@) == content_run_end_spec(views, cursor@),
        decreases atoms.len() - cursor.atom_index,
    {
        proof {
            reveal(content_run_end_spec);
        }
        cursor.atom_index += 1;
    }
    proof {
        reveal(content_run_end_spec);
    }
    cursor.atom_index
}

fn next_structural_step(atoms: &[LexicalAtom], cursor: StructuralCursor) -> (step: StructuralStep)
    requires
        cursor@.atom_index < atoms@.len(),
        cursor@.line_number < MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        step@ == next_structural_step_spec(crate::atom::lexical_atom_views_spec(atoms@), cursor@),
        cursor@.atom_index < step@.end_atom_index <= atoms@.len(),
        step@.next.atom_index == step@.end_atom_index,
        step@.line_number == cursor@.line_number,
        cursor@.line_number <= step@.next.line_number,
{
    let index = cursor.atom_index;
    let atom = &atoms[index];
    let kind = atom.kind();
    let step = if kind == LexicalAtomKind::LineFeed {
        StructuralStep {
            kind: StructuralLexemeKind::LineFeed,
            line_number: cursor.line_number,
            start_atom_index: index,
            end_atom_index: index + 1,
            next: StructuralCursor {
                atom_index: index + 1,
                line_number: cursor.line_number + 1,
                at_content_start: true,
                indented: false,
                previous_separation: false,
            },
        }
    } else if cursor.at_content_start && !cursor.indented && atom_is_indicator(
        atom,
        YamlIndicator::Directive,
    ) {
        let end = line_tail_end(atoms, index);
        StructuralStep {
            kind: StructuralLexemeKind::Directive,
            line_number: cursor.line_number,
            start_atom_index: index,
            end_atom_index: end,
            next: StructuralCursor {
                atom_index: end,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: false,
                previous_separation: false,
            },
        }
    } else if document_start_at(atoms, cursor) {
        StructuralStep {
            kind: StructuralLexemeKind::DocumentStart,
            line_number: cursor.line_number,
            start_atom_index: index,
            end_atom_index: index + 3,
            next: StructuralCursor {
                atom_index: index + 3,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: false,
                previous_separation: false,
            },
        }
    } else if document_end_at(atoms, cursor) {
        StructuralStep {
            kind: StructuralLexemeKind::DocumentEnd,
            line_number: cursor.line_number,
            start_atom_index: index,
            end_atom_index: index + 3,
            next: StructuralCursor {
                atom_index: index + 3,
                line_number: cursor.line_number,
                at_content_start: false,
                indented: false,
                previous_separation: false,
            },
        }
    } else {
        match single_structural_kind(atoms, cursor) {
            Some(StructuralLexemeKind::Indentation) => {
                let end = indentation_run_end(atoms, index);
                StructuralStep {
                    kind: StructuralLexemeKind::Indentation,
                    line_number: cursor.line_number,
                    start_atom_index: index,
                    end_atom_index: end,
                    next: StructuralCursor {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: true,
                        indented: true,
                        previous_separation: true,
                    },
                }
            },
            Some(StructuralLexemeKind::Separation) => {
                let end = separation_run_end(atoms, index);
                StructuralStep {
                    kind: StructuralLexemeKind::Separation,
                    line_number: cursor.line_number,
                    start_atom_index: index,
                    end_atom_index: end,
                    next: StructuralCursor {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: true,
                    },
                }
            },
            Some(StructuralLexemeKind::Comment) => {
                let end = line_tail_end(atoms, index);
                StructuralStep {
                    kind: StructuralLexemeKind::Comment,
                    line_number: cursor.line_number,
                    start_atom_index: index,
                    end_atom_index: end,
                    next: StructuralCursor {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: false,
                    },
                }
            },
            Some(structural_kind) => StructuralStep {
                kind: structural_kind,
                line_number: cursor.line_number,
                start_atom_index: index,
                end_atom_index: index + 1,
                next: StructuralCursor {
                    atom_index: index + 1,
                    line_number: cursor.line_number,
                    at_content_start: false,
                    indented: cursor.indented,
                    previous_separation: false,
                },
            },
            None => {
                let content_cursor = StructuralCursor {
                    atom_index: index + 1,
                    line_number: cursor.line_number,
                    at_content_start: false,
                    indented: cursor.indented,
                    previous_separation: false,
                };
                let end = content_run_end(atoms, content_cursor);
                StructuralStep {
                    kind: StructuralLexemeKind::Content,
                    line_number: cursor.line_number,
                    start_atom_index: index,
                    end_atom_index: end,
                    next: StructuralCursor {
                        atom_index: end,
                        line_number: cursor.line_number,
                        at_content_start: false,
                        indented: cursor.indented,
                        previous_separation: false,
                    },
                }
            },
        }
    };
    proof {
        lemma_next_structural_step_progress(crate::atom::lexical_atom_views_spec(atoms@), cursor@);
        reveal(next_structural_step_spec);
        reveal(crate::atom::lexical_atom_views_spec);
        assert(step@ == next_structural_step_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            cursor@,
        ));
    }
    step
}

fn make_structural_lexeme(atoms: &[LexicalAtom], step: &StructuralStep) -> (lexeme:
    StructuralLexeme)
    requires
        0 <= step@.start_atom_index < step@.end_atom_index <= atoms@.len(),
    ensures
        lexeme@ == lexeme_for_step_spec(crate::atom::lexical_atom_views_spec(atoms@), step@),
{
    let start = step.start_atom_index;
    let end = step.end_atom_index;
    let lexeme = StructuralLexeme {
        kind: step.kind,
        line_number: step.line_number,
        start_atom_index: start as u64,
        end_atom_index: end as u64,
        byte_start: atoms[start].span().start().byte_offset(),
        byte_end: atoms[end - 1].span().end().byte_offset(),
    };
    proof {
        reveal(lexeme_for_step_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    lexeme
}

proof fn lemma_structural_lexeme_views_push(
    lexemes: Seq<StructuralLexeme>,
    lexeme: StructuralLexeme,
)
    ensures
        structural_lexeme_views_spec(lexemes.push(lexeme)) == structural_lexeme_views_spec(
            lexemes,
        ).push(lexeme@),
{
    reveal(structural_lexeme_views_spec);
    assert(structural_lexeme_views_spec(lexemes.push(lexeme)) =~= structural_lexeme_views_spec(
        lexemes,
    ).push(lexeme@));
}

proof fn lemma_empty_structural_candidate_prefix(atoms: Seq<LexicalAtomView>)
    ensures
        structural_candidate_prefix_partition_spec(atoms, Seq::empty(), 0),
{
    reveal(structural_candidate_prefix_partition_spec);
}

/// Successful structural scanning of an empty atom stream contains no candidate ranges.
pub proof fn lemma_empty_structural_scan_has_no_lexemes(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    limits: StructuralScanLimitsView,
    structural: StructuralLexemeSourceView,
)
    requires
        atomized.atoms.len() == 0,
        scan_profile1_structural_lexemes_spec(atomized, layout, limits) == Ok(structural),
    ensures
        structural.lexemes.len() == 0,
{
    reveal(scan_profile1_structural_lexemes_spec);
    reveal(structural_scan_tail_spec);
}

#[verifier::spinoff_prover]
#[verifier::rlimit(100)]
proof fn lemma_extend_structural_candidate_prefix(
    atoms: Seq<LexicalAtomView>,
    built: Seq<StructuralLexemeView>,
    candidate: StructuralLexemeView,
    consumed_atoms: int,
)
    requires
        structural_candidate_prefix_partition_spec(atoms, built, consumed_atoms),
        structural_candidate_range_spec(atoms, candidate),
        candidate.start_atom_index == consumed_atoms,
        built.len() > 0 ==> built[built.len() - 1].line_number <= candidate.line_number,
        forall|index: int|
            0 < index < atoms.len() ==> atoms[index - 1].span.end == atoms[index].span.start,
    ensures
        structural_candidate_prefix_partition_spec(
            atoms,
            built.push(candidate),
            candidate.end_atom_index as int,
        ),
{
    reveal(structural_candidate_prefix_partition_spec);
    reveal(structural_candidate_range_spec);
    if built.len() > 0 {
        assert(built[built.len() - 1].end_atom_index == consumed_atoms);
        assert(0 < consumed_atoms < atoms.len());
        assert(built[built.len() - 1].byte_end == atoms[consumed_atoms - 1].span.end.byte_offset);
        assert(candidate.byte_start == atoms[consumed_atoms].span.start.byte_offset);
        assert(atoms[consumed_atoms - 1].span.end == atoms[consumed_atoms].span.start);
    }
    assert forall|index: int|
        0 <= index < built.push(candidate).len() implies structural_candidate_range_spec(
        atoms,
        #[trigger] built.push(candidate)[index],
    ) && (index > 0 ==> built.push(candidate)[index - 1].end_atom_index == built.push(
        candidate,
    )[index].start_atom_index && built.push(candidate)[index - 1].byte_end == built.push(
        candidate,
    )[index].byte_start && built.push(candidate)[index - 1].line_number <= built.push(
        candidate,
    )[index].line_number) by {
        if index < built.len() {
            assert(built.push(candidate)[index] == built[index]);
            if index > 0 {
                assert(built.push(candidate)[index - 1] == built[index - 1]);
            }
        } else {
            assert(index == built.len());
            assert(built.push(candidate)[index] == candidate);
            if index > 0 {
                assert(built.push(candidate)[index - 1] == built[built.len() - 1]);
            }
        }
    }
}

proof fn lemma_structural_tail_fits_remaining_atoms(
    atoms: Seq<LexicalAtomView>,
    cursor: StructuralCursorView,
    built: Seq<StructuralLexemeView>,
    lexeme_limit: u64,
    fuel: nat,
)
    requires
        0 <= cursor.atom_index <= atoms.len(),
        built.len() + (atoms.len() - cursor.atom_index) <= lexeme_limit,
        atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        cursor.line_number <= cursor.atom_index,
        atoms.len() - cursor.atom_index <= fuel,
    ensures
        exists|completed: Seq<StructuralLexemeView>|
            structural_scan_tail_spec(atoms, cursor, built, lexeme_limit, fuel) == Ok(completed),
    decreases atoms.len() - cursor.atom_index,
{
    reveal(structural_scan_tail_spec);
    if cursor.atom_index < atoms.len() {
        assert(fuel > 0);
        assert(cursor.line_number < MAX_PROFILE1_LEXICAL_ATOMS);
        let step = next_structural_step_spec(atoms, cursor);
        lemma_next_structural_step_progress(atoms, cursor);
        lemma_structural_tail_fits_remaining_atoms(
            atoms,
            step.next,
            built.push(lexeme_for_step_spec(atoms, step)),
            lexeme_limit,
            (fuel - 1) as nat,
        );
        let completed = choose|candidate: Seq<StructuralLexemeView>|
            structural_scan_tail_spec(
                atoms,
                step.next,
                built.push(lexeme_for_step_spec(atoms, step)),
                lexeme_limit,
                (fuel - 1) as nat,
            ) == Ok(candidate);
        assert(structural_scan_tail_spec(atoms, cursor, built, lexeme_limit, fuel) == Ok(
            completed,
        ));
    }
}

proof fn lemma_tail_error_is_scan_error(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    limits: StructuralScanLimitsView,
    error: StructuralScanErrorView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(atomized, canonical_layout_limits_spec()) == Ok(
            layout,
        ),
        structural_scan_tail_spec(
            atomized.atoms,
            StructuralCursorView {
                atom_index: 0,
                line_number: 0,
                at_content_start: true,
                indented: false,
                previous_separation: false,
            },
            Seq::empty(),
            effective_lexeme_limit_spec(limits),
            atomized.atoms.len(),
        ) == Err(error),
    ensures
        scan_profile1_structural_lexemes_spec(atomized, layout, limits) == Err(error),
{
    reveal(scan_profile1_structural_lexemes_spec);
}

pub proof fn lemma_short_well_formed_input_fits_structural_scan_limits(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
    limits: StructuralScanLimitsView,
)
    requires
        crate::layout::analyze_profile1_layout_spec(atomized, canonical_layout_limits_spec()) == Ok(
            layout,
        ),
        atomized.atoms.len() <= limits.max_lexemes,
        atomized.atoms.len() <= MAX_PROFILE1_STRUCTURAL_LEXEMES,
    ensures
        exists|source: StructuralLexemeSourceView|
            scan_profile1_structural_lexemes_spec(atomized, layout, limits) == Ok(source),
{
    reveal(scan_profile1_structural_lexemes_spec);
    reveal(effective_lexeme_limit_spec);
    let cursor = StructuralCursorView {
        atom_index: 0,
        line_number: 0,
        at_content_start: true,
        indented: false,
        previous_separation: false,
    };
    lemma_structural_tail_fits_remaining_atoms(
        atomized.atoms,
        cursor,
        Seq::empty(),
        effective_lexeme_limit_spec(limits),
        atomized.atoms.len(),
    );
}

#[verifier::rlimit(180)]
#[verifier::spinoff_prover]
pub fn scan_profile1_structural_lexemes(
    atomized: &AtomizedSource,
    layout: &LayoutSource,
    limits: StructuralScanLimits,
) -> (result: Result<StructuralLexemeSource, StructuralScanError>)
    ensures
        scan_profile1_structural_lexemes_spec(atomized@, layout@, limits@) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(source) => {
                structural_lexeme_source_corresponds_spec(atomized@, layout@, source@) && ((
                crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                    && crate::layout::layout_source_well_formed_spec(atomized@, layout@))
                    ==> structural_lexeme_source_well_formed_spec(atomized@, layout@, source@))
                    && source@.lexemes.len() <= limits@.max_lexemes && source@.lexemes.len()
                    <= MAX_PROFILE1_STRUCTURAL_LEXEMES && source@.input_atom_count
                    == atomized@.atoms.len() && source@.input_line_count == layout@.lines.len()
            },
            Err(_) => true,
        },
{
    let canonical_limits = canonical_structural_layout_limits();
    let canonical = match analyze_profile1_layout(atomized, canonical_limits) {
        Ok(source) => source,
        Err(error) => {
            let mismatch = StructuralScanError::at(
                StructuralScanErrorKind::InputLayoutMismatch,
                error.byte_offset(),
            );
            proof {
                reveal(scan_profile1_structural_lexemes_spec);
            }
            return Err(mismatch);
        },
    };
    proof {
        crate::layout::lemma_layout_success_input_within_atom_cap(
            atomized@,
            canonical_limits@,
            canonical@,
        );
    }
    if !canonical.same_as(layout) {
        let mismatch = StructuralScanError::at(
            StructuralScanErrorKind::InputLayoutMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(scan_profile1_structural_lexemes_spec);
        }
        return Err(mismatch);
    }
    assert(canonical@ == layout@);
    proof {
        assert(crate::layout::analyze_profile1_layout_spec(
            atomized@,
            canonical_layout_limits_spec(),
        ) == Ok(canonical@));
        assert(crate::layout::analyze_profile1_layout_spec(
            atomized@,
            canonical_layout_limits_spec(),
        ) == Ok(layout@));
    }
    let lexeme_limit = if limits.max_lexemes < MAX_PROFILE1_STRUCTURAL_LEXEMES {
        limits.max_lexemes
    } else {
        MAX_PROFILE1_STRUCTURAL_LEXEMES
    };
    proof {
        reveal(effective_lexeme_limit_spec);
        assert(lexeme_limit == effective_lexeme_limit_spec(limits@));
    }
    let atoms = atomized.atoms();
    let mut lexemes: Vec<StructuralLexeme> = Vec::new();
    let mut cursor = StructuralCursor {
        atom_index: 0,
        line_number: 0,
        at_content_start: true,
        indented: false,
        previous_separation: false,
    };
    let ghost initial_cursor = cursor@;
    let ghost expected = structural_scan_tail_spec(
        atomized@.atoms,
        initial_cursor,
        Seq::empty(),
        lexeme_limit,
        atomized@.atoms.len(),
    );
    proof {
        reveal(structural_lexeme_views_spec);
        assert(structural_lexeme_views_spec(lexemes@) =~= Seq::<StructuralLexemeView>::empty());
        lemma_empty_structural_candidate_prefix(atomized@.atoms);
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) {
            crate::atom::lemma_intrinsic_atomized_spans_partition_source(atomized@);
        }
        assert((atomized@.atoms.len() - lexemes@.len()) as nat == atomized@.atoms.len());
        assert(expected == structural_scan_tail_spec(
            atomized@.atoms,
            cursor@,
            structural_lexeme_views_spec(lexemes@),
            lexeme_limit,
            (atomized@.atoms.len() - lexemes@.len()) as nat,
        ));
    }
    while cursor.atom_index < atoms.len()
        invariant
            crate::layout::analyze_profile1_layout_spec(atomized@, canonical_layout_limits_spec())
                == Ok(layout@),
            expected == structural_scan_tail_spec(
                atomized@.atoms,
                StructuralCursorView {
                    atom_index: 0,
                    line_number: 0,
                    at_content_start: true,
                    indented: false,
                    previous_separation: false,
                },
                Seq::empty(),
                effective_lexeme_limit_spec(limits@),
                atomized@.atoms.len(),
            ),
            crate::atom::lexical_atom_views_spec(atoms@) == atomized@.atoms,
            cursor@.atom_index <= atoms@.len(),
            atomized@.atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
            cursor@.line_number <= cursor@.atom_index,
            lexemes@.len() <= lexeme_limit,
            lexemes@.len() <= cursor@.atom_index,
            structural_lexeme_views_spec(lexemes@).len() == lexemes@.len(),
            crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                ==> structural_candidate_prefix_partition_spec(
                atomized@.atoms,
                structural_lexeme_views_spec(lexemes@),
                cursor@.atom_index,
            ),
            crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) && lexemes@.len()
                > 0 ==> structural_lexeme_views_spec(lexemes@)[lexemes@.len() - 1].line_number
                <= cursor@.line_number,
            crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) ==> (
            atomized@.atoms.len() == 0 ==> atomized@.source_len_bytes == atomized@.bom_bytes) && (
            atomized@.atoms.len() > 0 ==> atomized@.atoms[0].span.start.byte_offset
                == atomized@.bom_bytes && atomized@.atoms[atomized@.atoms.len()
                - 1].span.end.byte_offset == atomized@.source_len_bytes && forall|index: int|
                0 < index < atomized@.atoms.len() ==> atomized@.atoms[index - 1].span.end
                    == atomized@.atoms[index].span.start),
            expected == structural_scan_tail_spec(
                atomized@.atoms,
                cursor@,
                structural_lexeme_views_spec(lexemes@),
                lexeme_limit,
                (atomized@.atoms.len() - lexemes@.len()) as nat,
            ),
        decreases atoms.len() - cursor.atom_index,
    {
        let step = next_structural_step(atoms, cursor);
        proof {
            reveal(structural_scan_tail_spec);
        }
        if lexemes.len() as u64 >= lexeme_limit {
            let error = StructuralScanError::at(
                StructuralScanErrorKind::LexemeLimitExceeded,
                atoms[cursor.atom_index].span().start().byte_offset(),
            );
            proof {
                assert(expected == Err(error@));
                lemma_tail_error_is_scan_error(atomized@, layout@, limits@, error@);
            }
            return Err(error);
        }
        let lexeme = make_structural_lexeme(atoms, &step);
        proof {
            if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) {
                reveal(structural_candidate_range_spec);
                reveal(lexeme_for_step_spec);
                assert(structural_candidate_range_spec(atomized@.atoms, lexeme@));
                lemma_extend_structural_candidate_prefix(
                    atomized@.atoms,
                    structural_lexeme_views_spec(lexemes@),
                    lexeme@,
                    cursor@.atom_index,
                );
            }
            lemma_structural_lexeme_views_push(lexemes@, lexeme);
        }
        lexemes.push(lexeme);
        cursor = step.next;
        proof {
            reveal(structural_scan_tail_spec);
        }
    }
    let source = StructuralLexemeSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version(),
        layout_transformation_version: layout.transformation_version(),
        transformation_version: STRUCTURAL_LEXEME_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes(),
        bom_bytes: atomized.bom_bytes(),
        input_atom_count: atoms.len() as u64,
        input_line_count: layout.lines().len() as u64,
        lexemes,
    };
    proof {
        reveal(structural_scan_tail_spec);
        reveal(scan_profile1_structural_lexemes_spec);
        assert(scan_profile1_structural_lexemes_spec(atomized@, layout@, limits@) == Ok(source@));
        reveal(structural_lexeme_source_corresponds_spec);
        assert(exists|candidate_limits: StructuralScanLimitsView|
            scan_profile1_structural_lexemes_spec(atomized@, layout@, candidate_limits) == Ok(
                source@,
            )) by {
            assert(scan_profile1_structural_lexemes_spec(atomized@, layout@, limits@) == Ok(
                source@,
            ));
        }
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
            && crate::layout::layout_source_well_formed_spec(atomized@, layout@) {
            reveal(structural_lexeme_partition_spec);
            reveal(structural_candidate_prefix_partition_spec);
            reveal(structural_candidate_range_spec);
            if atomized@.atoms.len() == 0 {
                assert(source@.lexemes.len() == 0);
            } else {
                assert(source@.lexemes.len() > 0);
                assert(source@.lexemes[0].start_atom_index == 0);
                assert(source@.lexemes[0].byte_start == atomized@.atoms[0].span.start.byte_offset);
                assert(source@.lexemes[source@.lexemes.len() - 1].end_atom_index
                    == atomized@.atoms.len());
                assert(source@.lexemes[source@.lexemes.len() - 1].byte_end
                    == atomized@.atoms[atomized@.atoms.len() - 1].span.end.byte_offset);
            }
            assert(structural_lexeme_partition_spec(atomized@, layout@, source@));
            reveal(structural_lexeme_source_well_formed_spec);
        }
    }
    Ok(source)
}

} // verus!
