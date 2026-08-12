//! Verified line and indentation layout for Crucible YAML profile 1.
//!
//! This transformation turns scalar-preserving lexical atoms into exact line descriptors. It is
//! deliberately narrower than token formation: comments, directives, flow state, and scalar
//! boundaries remain the responsibility of the following context-sensitive token scanner.
use crate::atom::{AtomizedSource, LexicalAtom, LexicalAtomKind, MAX_PROFILE1_LEXICAL_ATOMS};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::utf8::CRUCIBLE_YAML_PROFILE_VERSION;
use vstd::prelude::*;

verus! {

pub const LINE_LAYOUT_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_LAYOUT_LINES: u64 = MAX_PROFILE1_LEXICAL_ATOMS;

pub const MAX_PROFILE1_INDENTATION_COLUMNS: u64 = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutLimits {
    max_lines: u64,
    max_indentation_columns: u64,
}

#[verifier::ext_equal]
pub struct LayoutLimitsView {
    pub max_lines: u64,
    pub max_indentation_columns: u64,
}

impl View for LayoutLimits {
    type V = LayoutLimitsView;

    closed spec fn view(&self) -> LayoutLimitsView {
        LayoutLimitsView {
            max_lines: self.max_lines,
            max_indentation_columns: self.max_indentation_columns,
        }
    }
}

impl LayoutLimits {
    pub fn new(max_lines: u64, max_indentation_columns: u64) -> (limits: Self)
        ensures
            limits@.max_lines == max_lines,
            limits@.max_indentation_columns == max_indentation_columns,
    {
        Self { max_lines, max_indentation_columns }
    }

    pub fn max_lines(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_lines,
    {
        self.max_lines
    }

    pub fn max_indentation_columns(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_indentation_columns,
    {
        self.max_indentation_columns
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayoutErrorKind {
    InputAtomLimitExceeded,
    LineLimitExceeded,
    IndentationLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutError {
    kind: LayoutErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct LayoutErrorView {
    pub kind: LayoutErrorKind,
    pub byte_offset: u64,
}

impl View for LayoutError {
    type V = LayoutErrorView;

    closed spec fn view(&self) -> LayoutErrorView {
        LayoutErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl LayoutError {
    fn at(kind: LayoutErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (LayoutErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: LayoutErrorKind)
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
/// An exact half-open atom range for one source line.
///
/// `end_atom_index` excludes the line feed. When `is_terminated` is true, that line feed is the
/// atom at `end_atom_index`.
///
/// ```compile_fail
/// use crucible_yaml::LayoutLine;
///
/// let forged = LayoutLine {
///     line_number: 9,
///     start_atom_index: 4,
///     content_atom_index: 3,
///     end_atom_index: 2,
///     terminated: true,
///     indentation_columns: u64::MAX,
///     byte_start: 8,
///     content_byte_start: 7,
///     byte_end: 6,
/// };
/// ```
pub struct LayoutLine {
    line_number: u64,
    start_atom_index: u64,
    content_atom_index: u64,
    end_atom_index: u64,
    terminated: bool,
    indentation_columns: u64,
    byte_start: u64,
    content_byte_start: u64,
    byte_end: u64,
}

#[verifier::ext_equal]
pub struct LayoutLineView {
    pub line_number: u64,
    pub start_atom_index: u64,
    pub content_atom_index: u64,
    pub end_atom_index: u64,
    pub terminated: bool,
    pub indentation_columns: u64,
    pub byte_start: u64,
    pub content_byte_start: u64,
    pub byte_end: u64,
}

impl View for LayoutLine {
    type V = LayoutLineView;

    closed spec fn view(&self) -> LayoutLineView {
        LayoutLineView {
            line_number: self.line_number,
            start_atom_index: self.start_atom_index,
            content_atom_index: self.content_atom_index,
            end_atom_index: self.end_atom_index,
            terminated: self.terminated,
            indentation_columns: self.indentation_columns,
            byte_start: self.byte_start,
            content_byte_start: self.content_byte_start,
            byte_end: self.byte_end,
        }
    }
}

impl DeepView for LayoutLine {
    type V = LayoutLineView;

    closed spec fn deep_view(&self) -> LayoutLineView {
        self@
    }
}

impl LayoutLine {
    pub fn line_number(&self) -> (number: u64)
        ensures
            number == self@.line_number,
    {
        self.line_number
    }

    pub fn start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.start_atom_index,
    {
        self.start_atom_index
    }

    pub fn content_atom_index(&self) -> (index: u64)
        ensures
            index == self@.content_atom_index,
    {
        self.content_atom_index
    }

    pub fn end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.end_atom_index,
    {
        self.end_atom_index
    }

    pub fn is_terminated(&self) -> (terminated: bool)
        ensures
            terminated == self@.terminated,
    {
        self.terminated
    }

    pub fn indentation_columns(&self) -> (columns: u64)
        ensures
            columns == self@.indentation_columns,
    {
        self.indentation_columns
    }

    pub fn byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.byte_start,
    {
        self.byte_start
    }

    pub fn content_byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.content_byte_start,
    {
        self.content_byte_start
    }

    pub fn byte_end(&self) -> (offset: u64)
        ensures
            offset == self@.byte_end,
    {
        self.byte_end
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutSource {
    profile_version: u16,
    input_transformation_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    lines: Vec<LayoutLine>,
}

#[verifier::ext_equal]
pub struct LayoutSourceView {
    pub profile_version: u16,
    pub input_transformation_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub lines: Seq<LayoutLineView>,
}

pub open spec fn layout_line_views_spec(lines: Seq<LayoutLine>) -> Seq<LayoutLineView> {
    Seq::new(lines.len(), |index: int| lines[index]@)
}

impl View for LayoutSource {
    type V = LayoutSourceView;

    closed spec fn view(&self) -> LayoutSourceView {
        LayoutSourceView {
            profile_version: self.profile_version,
            input_transformation_version: self.input_transformation_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            lines: layout_line_views_spec(self.lines@),
        }
    }
}

impl LayoutSource {
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

    pub fn lines(&self) -> (lines: &[LayoutLine])
        ensures
            layout_line_views_spec(lines@) == self@.lines,
    {
        self.lines.as_slice()
    }
}

#[verifier::ext_equal]
#[allow(dead_code)]
struct LayoutScanStateView {
    lines: Seq<LayoutLineView>,
    line_start: int,
    content_start: int,
    indentation_columns: u64,
    line_number: u64,
    at_indentation: bool,
}

closed spec fn effective_line_limit_spec(limits: LayoutLimitsView) -> u64 {
    if limits.max_lines < MAX_PROFILE1_LAYOUT_LINES {
        limits.max_lines
    } else {
        MAX_PROFILE1_LAYOUT_LINES
    }
}

closed spec fn effective_indentation_limit_spec(limits: LayoutLimitsView) -> u64 {
    if limits.max_indentation_columns < MAX_PROFILE1_INDENTATION_COLUMNS {
        limits.max_indentation_columns
    } else {
        MAX_PROFILE1_INDENTATION_COLUMNS
    }
}

closed spec fn initial_layout_scan_state_spec() -> LayoutScanStateView {
    LayoutScanStateView {
        lines: Seq::empty(),
        line_start: 0,
        content_start: 0,
        indentation_columns: 0,
        line_number: 0,
        at_indentation: true,
    }
}

closed spec fn layout_line_spec(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    state: LayoutScanStateView,
    end_atom_index: int,
    terminated: bool,
) -> LayoutLineView {
    LayoutLineView {
        line_number: state.line_number,
        start_atom_index: state.line_start as u64,
        content_atom_index: state.content_start as u64,
        end_atom_index: end_atom_index as u64,
        terminated,
        indentation_columns: state.indentation_columns,
        byte_start: atoms[state.line_start].span.start.byte_offset,
        content_byte_start: if state.content_start < end_atom_index {
            atoms[state.content_start].span.start.byte_offset
        } else if terminated {
            atoms[end_atom_index].span.start.byte_offset
        } else {
            source_len_bytes
        },
        byte_end: if terminated {
            atoms[end_atom_index].span.end.byte_offset
        } else {
            source_len_bytes
        },
    }
}

closed spec fn finish_layout_scan_spec(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    state: LayoutScanStateView,
) -> LayoutScanStateView {
    if state.line_start < atoms.len() {
        LayoutScanStateView {
            lines: state.lines.push(
                layout_line_spec(atoms, source_len_bytes, state, atoms.len() as int, false),
            ),
            line_start: atoms.len() as int,
            content_start: atoms.len() as int,
            indentation_columns: 0,
            line_number: (state.line_number + 1) as u64,
            at_indentation: true,
        }
    } else {
        state
    }
}

closed spec fn layout_scan_tail_spec(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
) -> Result<LayoutScanStateView, LayoutErrorView>
    decreases atoms.len() - index,
{
    if index >= atoms.len() {
        Ok(finish_layout_scan_spec(atoms, source_len_bytes, state))
    } else {
        let atom = atoms[index];
        if state.at_indentation && atom.kind == LexicalAtomKind::Space {
            if state.indentation_columns >= indentation_limit {
                Err(
                    LayoutErrorView {
                        kind: LayoutErrorKind::IndentationLimitExceeded,
                        byte_offset: atom.span.start.byte_offset,
                    },
                )
            } else {
                layout_scan_tail_spec(
                    atoms,
                    source_len_bytes,
                    index + 1,
                    LayoutScanStateView {
                        lines: state.lines,
                        line_start: state.line_start,
                        content_start: index + 1,
                        indentation_columns: (state.indentation_columns + 1) as u64,
                        line_number: state.line_number,
                        at_indentation: true,
                    },
                    line_limit,
                    indentation_limit,
                )
            }
        } else if atom.kind == LexicalAtomKind::LineFeed {
            let next_index = index + 1;
            let next_line_number = (state.line_number + 1) as u64;
            let next_state = LayoutScanStateView {
                lines: state.lines.push(
                    layout_line_spec(atoms, source_len_bytes, state, index, true),
                ),
                line_start: next_index,
                content_start: next_index,
                indentation_columns: 0,
                line_number: next_line_number,
                at_indentation: true,
            };
            if next_index < atoms.len() && next_line_number >= line_limit {
                Err(
                    LayoutErrorView {
                        kind: LayoutErrorKind::LineLimitExceeded,
                        byte_offset: atoms[next_index].span.start.byte_offset,
                    },
                )
            } else {
                layout_scan_tail_spec(
                    atoms,
                    source_len_bytes,
                    next_index,
                    next_state,
                    line_limit,
                    indentation_limit,
                )
            }
        } else {
            layout_scan_tail_spec(
                atoms,
                source_len_bytes,
                index + 1,
                LayoutScanStateView {
                    lines: state.lines,
                    line_start: state.line_start,
                    content_start: state.content_start,
                    indentation_columns: state.indentation_columns,
                    line_number: state.line_number,
                    at_indentation: false,
                },
                line_limit,
                indentation_limit,
            )
        }
    }
}

pub closed spec fn analyze_profile1_layout_spec(
    atomized: AtomizedSourceView,
    limits: LayoutLimitsView,
) -> Result<LayoutSourceView, LayoutErrorView> {
    if atomized.atoms.len() > MAX_PROFILE1_LEXICAL_ATOMS {
        Err(
            LayoutErrorView {
                kind: LayoutErrorKind::InputAtomLimitExceeded,
                byte_offset:
                    atomized.atoms[MAX_PROFILE1_LEXICAL_ATOMS as int].span.start.byte_offset,
            },
        )
    } else {
        let line_limit = effective_line_limit_spec(limits);
        let indentation_limit = effective_indentation_limit_spec(limits);
        if atomized.atoms.len() > 0 && line_limit == 0 {
            Err(
                LayoutErrorView {
                    kind: LayoutErrorKind::LineLimitExceeded,
                    byte_offset: atomized.atoms[0].span.start.byte_offset,
                },
            )
        } else {
            match layout_scan_tail_spec(
                atomized.atoms,
                atomized.source_len_bytes,
                0,
                initial_layout_scan_state_spec(),
                line_limit,
                indentation_limit,
            ) {
                Ok(state) => Ok(
                    LayoutSourceView {
                        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                        input_transformation_version: atomized.transformation_version,
                        transformation_version: LINE_LAYOUT_TRANSFORMATION_VERSION,
                        source_len_bytes: atomized.source_len_bytes,
                        bom_bytes: atomized.bom_bytes,
                        lines: state.lines,
                    },
                ),
                Err(error) => Err(error),
            }
        }
    }
}

pub closed spec fn layout_source_corresponds_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
) -> bool {
    exists|limits: LayoutLimitsView| analyze_profile1_layout_spec(atomized, limits) == Ok(layout)
}

pub closed spec fn layout_source_well_formed_spec(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
) -> bool {
    crate::atom::atomized_source_intrinsically_well_formed_spec(atomized)
        && layout_source_corresponds_spec(atomized, layout)
}

/// Semantic layout validity necessarily retains intrinsic validity of its atom source.
pub proof fn lemma_layout_well_formed_requires_intrinsic_atom_source(
    atomized: AtomizedSourceView,
    layout: LayoutSourceView,
)
    requires
        layout_source_well_formed_spec(atomized, layout),
    ensures
        crate::atom::atomized_source_intrinsically_well_formed_spec(atomized),
{
    reveal(layout_source_well_formed_spec);
}

/// The defensive atom cap takes precedence over every caller-supplied layout limit.
pub proof fn lemma_layout_input_atom_limit_error(
    atomized: AtomizedSourceView,
    limits: LayoutLimitsView,
)
    requires
        atomized.atoms.len() > MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        analyze_profile1_layout_spec(atomized, limits) == Err(
            LayoutErrorView {
                kind: LayoutErrorKind::InputAtomLimitExceeded,
                byte_offset:
                    atomized.atoms[MAX_PROFILE1_LEXICAL_ATOMS as int].span.start.byte_offset,
            },
        ),
{
    reveal(analyze_profile1_layout_spec);
}

proof fn lemma_layout_scan_cannot_error_when_limits_cover_remaining_atoms(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
)
    requires
        0 <= index <= atoms.len(),
        state.line_number <= line_limit,
        index < atoms.len() ==> state.line_number < line_limit,
        state.indentation_columns <= indentation_limit,
        state.indentation_columns + (atoms.len() - index) <= indentation_limit,
        state.line_number + (atoms.len() - index) <= line_limit,
    ensures
        exists|completed: LayoutScanStateView|
            layout_scan_tail_spec(
                atoms,
                source_len_bytes,
                index,
                state,
                line_limit,
                indentation_limit,
            ) == Ok(completed),
    decreases atoms.len() - index,
{
    reveal(layout_scan_tail_spec);
    if index >= atoms.len() {
        assert(layout_scan_tail_spec(
            atoms,
            source_len_bytes,
            index,
            state,
            line_limit,
            indentation_limit,
        ) == Ok(finish_layout_scan_spec(atoms, source_len_bytes, state)));
    } else {
        let atom = atoms[index];
        if state.at_indentation && atom.kind == LexicalAtomKind::Space {
            assert(state.indentation_columns < indentation_limit);
            let next_state = LayoutScanStateView {
                lines: state.lines,
                line_start: state.line_start,
                content_start: index + 1,
                indentation_columns: (state.indentation_columns + 1) as u64,
                line_number: state.line_number,
                at_indentation: true,
            };
            lemma_layout_scan_cannot_error_when_limits_cover_remaining_atoms(
                atoms,
                source_len_bytes,
                index + 1,
                next_state,
                line_limit,
                indentation_limit,
            );
            let completed = choose|candidate: LayoutScanStateView|
                layout_scan_tail_spec(
                    atoms,
                    source_len_bytes,
                    index + 1,
                    next_state,
                    line_limit,
                    indentation_limit,
                ) == Ok(candidate);
            assert(layout_scan_tail_spec(
                atoms,
                source_len_bytes,
                index,
                state,
                line_limit,
                indentation_limit,
            ) == Ok(completed));
        } else if atom.kind == LexicalAtomKind::LineFeed {
            let next_index = index + 1;
            let next_line_number = (state.line_number + 1) as u64;
            let next_state = LayoutScanStateView {
                lines: state.lines.push(
                    layout_line_spec(atoms, source_len_bytes, state, index, true),
                ),
                line_start: next_index,
                content_start: next_index,
                indentation_columns: 0,
                line_number: next_line_number,
                at_indentation: true,
            };
            if next_index < atoms.len() {
                assert(next_line_number < line_limit);
                lemma_layout_scan_cannot_error_when_limits_cover_remaining_atoms(
                    atoms,
                    source_len_bytes,
                    next_index,
                    next_state,
                    line_limit,
                    indentation_limit,
                );
            } else {
                lemma_layout_scan_cannot_error_when_limits_cover_remaining_atoms(
                    atoms,
                    source_len_bytes,
                    next_index,
                    next_state,
                    line_limit,
                    indentation_limit,
                );
            }
            let completed = choose|candidate: LayoutScanStateView|
                layout_scan_tail_spec(
                    atoms,
                    source_len_bytes,
                    next_index,
                    next_state,
                    line_limit,
                    indentation_limit,
                ) == Ok(candidate);
            assert(layout_scan_tail_spec(
                atoms,
                source_len_bytes,
                index,
                state,
                line_limit,
                indentation_limit,
            ) == Ok(completed));
        } else {
            let next_state = LayoutScanStateView {
                lines: state.lines,
                line_start: state.line_start,
                content_start: state.content_start,
                indentation_columns: state.indentation_columns,
                line_number: state.line_number,
                at_indentation: false,
            };
            lemma_layout_scan_cannot_error_when_limits_cover_remaining_atoms(
                atoms,
                source_len_bytes,
                index + 1,
                next_state,
                line_limit,
                indentation_limit,
            );
            let completed = choose|candidate: LayoutScanStateView|
                layout_scan_tail_spec(
                    atoms,
                    source_len_bytes,
                    index + 1,
                    next_state,
                    line_limit,
                    indentation_limit,
                ) == Ok(candidate);
            assert(layout_scan_tail_spec(
                atoms,
                source_len_bytes,
                index,
                state,
                line_limit,
                indentation_limit,
            ) == Ok(completed));
        }
    }
}

/// A short stream is admitted when both caller limits cover its complete atom count.
pub proof fn lemma_short_atom_stream_fits_layout_limits(
    atomized: AtomizedSourceView,
    limits: LayoutLimitsView,
)
    requires
        atomized.atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        atomized.atoms.len() <= limits.max_lines,
        atomized.atoms.len() <= limits.max_indentation_columns,
        atomized.atoms.len() <= MAX_PROFILE1_LAYOUT_LINES,
        atomized.atoms.len() <= MAX_PROFILE1_INDENTATION_COLUMNS,
    ensures
        exists|layout: LayoutSourceView|
            analyze_profile1_layout_spec(atomized, limits) == Ok(layout),
{
    reveal(analyze_profile1_layout_spec);
    reveal(effective_line_limit_spec);
    reveal(effective_indentation_limit_spec);
    let line_limit = effective_line_limit_spec(limits);
    let indentation_limit = effective_indentation_limit_spec(limits);
    if atomized.atoms.len() > 0 {
        assert(line_limit > 0);
    }
    reveal(initial_layout_scan_state_spec);
    lemma_layout_scan_cannot_error_when_limits_cover_remaining_atoms(
        atomized.atoms,
        atomized.source_len_bytes,
        0,
        initial_layout_scan_state_spec(),
        line_limit,
        indentation_limit,
    );
}

proof fn lemma_analyze_spec_from_scan_error(
    atomized: AtomizedSourceView,
    limits: LayoutLimitsView,
    line_limit: u64,
    indentation_limit: u64,
    error: LayoutErrorView,
)
    requires
        atomized.atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        atomized.atoms.len() == 0 || line_limit > 0,
        line_limit == effective_line_limit_spec(limits),
        indentation_limit == effective_indentation_limit_spec(limits),
        layout_scan_tail_spec(
            atomized.atoms,
            atomized.source_len_bytes,
            0,
            initial_layout_scan_state_spec(),
            line_limit,
            indentation_limit,
        ) == Err(error),
    ensures
        analyze_profile1_layout_spec(atomized, limits) == Err(error),
{
    reveal(analyze_profile1_layout_spec);
}

proof fn lemma_analyze_spec_from_scan_success(
    atomized: AtomizedSourceView,
    limits: LayoutLimitsView,
    line_limit: u64,
    indentation_limit: u64,
    state: LayoutScanStateView,
)
    requires
        atomized.atoms.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
        atomized.atoms.len() == 0 || line_limit > 0,
        line_limit == effective_line_limit_spec(limits),
        indentation_limit == effective_indentation_limit_spec(limits),
        layout_scan_tail_spec(
            atomized.atoms,
            atomized.source_len_bytes,
            0,
            initial_layout_scan_state_spec(),
            line_limit,
            indentation_limit,
        ) == Ok(state),
    ensures
        analyze_profile1_layout_spec(atomized, limits) == Ok(
            LayoutSourceView {
                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                input_transformation_version: atomized.transformation_version,
                transformation_version: LINE_LAYOUT_TRANSFORMATION_VERSION,
                source_len_bytes: atomized.source_len_bytes,
                bom_bytes: atomized.bom_bytes,
                lines: state.lines,
            },
        ),
{
    reveal(analyze_profile1_layout_spec);
}

proof fn lemma_layout_scan_space_error(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
)
    requires
        0 <= index < atoms.len(),
        state.at_indentation,
        atoms[index].kind == LexicalAtomKind::Space,
        state.indentation_columns >= indentation_limit,
    ensures
        layout_scan_tail_spec(atoms, source_len_bytes, index, state, line_limit, indentation_limit)
            == Err(
            LayoutErrorView {
                kind: LayoutErrorKind::IndentationLimitExceeded,
                byte_offset: atoms[index].span.start.byte_offset,
            },
        ),
{
    reveal(layout_scan_tail_spec);
}

proof fn lemma_layout_scan_space_step(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
)
    requires
        0 <= index < atoms.len(),
        state.at_indentation,
        atoms[index].kind == LexicalAtomKind::Space,
        state.indentation_columns < indentation_limit,
    ensures
        layout_scan_tail_spec(atoms, source_len_bytes, index, state, line_limit, indentation_limit)
            == layout_scan_tail_spec(
            atoms,
            source_len_bytes,
            index + 1,
            LayoutScanStateView {
                lines: state.lines,
                line_start: state.line_start,
                content_start: index + 1,
                indentation_columns: (state.indentation_columns + 1) as u64,
                line_number: state.line_number,
                at_indentation: true,
            },
            line_limit,
            indentation_limit,
        ),
{
    reveal(layout_scan_tail_spec);
}

proof fn lemma_layout_scan_content_step(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
)
    requires
        0 <= index < atoms.len(),
        !(state.at_indentation && atoms[index].kind == LexicalAtomKind::Space),
        atoms[index].kind != LexicalAtomKind::LineFeed,
    ensures
        layout_scan_tail_spec(atoms, source_len_bytes, index, state, line_limit, indentation_limit)
            == layout_scan_tail_spec(
            atoms,
            source_len_bytes,
            index + 1,
            LayoutScanStateView {
                lines: state.lines,
                line_start: state.line_start,
                content_start: state.content_start,
                indentation_columns: state.indentation_columns,
                line_number: state.line_number,
                at_indentation: false,
            },
            line_limit,
            indentation_limit,
        ),
{
    reveal(layout_scan_tail_spec);
}

proof fn lemma_layout_scan_line_error(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
)
    requires
        0 <= index < atoms.len(),
        atoms[index].kind == LexicalAtomKind::LineFeed,
        index + 1 < atoms.len(),
        (state.line_number + 1) as u64 >= line_limit,
    ensures
        layout_scan_tail_spec(atoms, source_len_bytes, index, state, line_limit, indentation_limit)
            == Err(
            LayoutErrorView {
                kind: LayoutErrorKind::LineLimitExceeded,
                byte_offset: atoms[index + 1].span.start.byte_offset,
            },
        ),
{
    reveal(layout_scan_tail_spec);
}

proof fn lemma_layout_scan_line_step(
    atoms: Seq<LexicalAtomView>,
    source_len_bytes: u64,
    index: int,
    state: LayoutScanStateView,
    line_limit: u64,
    indentation_limit: u64,
)
    requires
        0 <= index < atoms.len(),
        atoms[index].kind == LexicalAtomKind::LineFeed,
        !(index + 1 < atoms.len() && (state.line_number + 1) as u64 >= line_limit),
    ensures
        layout_scan_tail_spec(atoms, source_len_bytes, index, state, line_limit, indentation_limit)
            == layout_scan_tail_spec(
            atoms,
            source_len_bytes,
            index + 1,
            LayoutScanStateView {
                lines: state.lines.push(
                    layout_line_spec(atoms, source_len_bytes, state, index, true),
                ),
                line_start: index + 1,
                content_start: index + 1,
                indentation_columns: 0,
                line_number: (state.line_number + 1) as u64,
                at_indentation: true,
            },
            line_limit,
            indentation_limit,
        ),
{
    reveal(layout_scan_tail_spec);
}

proof fn lemma_layout_line_views_push(lines: Seq<LayoutLine>, line: LayoutLine)
    ensures
        layout_line_views_spec(lines.push(line)) == layout_line_views_spec(lines).push(line@),
{
    reveal(layout_line_views_spec);
    assert forall|index: int|
        0 <= index < lines.push(line).len() implies #[trigger] layout_line_views_spec(
        lines.push(line),
    )[index] == layout_line_views_spec(lines).push(line@)[index] by {
        if index < lines.len() {
            assert(lines.push(line)[index] == lines[index]);
        } else {
            assert(index == lines.len());
            assert(lines.push(line)[index] == line);
        }
    }
}

#[derive(Clone, Copy)]
struct LineBuildState {
    line_number: u64,
    line_start: usize,
    content_start: usize,
    indentation_columns: u64,
}

fn make_layout_line(
    atoms: &[LexicalAtom],
    source_len_bytes: u64,
    state: LineBuildState,
    end_atom_index: usize,
    terminated: bool,
) -> (line: LayoutLine)
    requires
        state.line_start < atoms.len(),
        state.line_start <= state.content_start <= end_atom_index <= atoms.len(),
        state.indentation_columns == (state.content_start - state.line_start) as u64,
        terminated == (end_atom_index < atoms.len()),
    ensures
        line@ == layout_line_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            source_len_bytes,
            LayoutScanStateView {
                lines: Seq::empty(),
                line_start: state.line_start as int,
                content_start: state.content_start as int,
                indentation_columns: state.indentation_columns,
                line_number: state.line_number,
                at_indentation: false,
            },
            end_atom_index as int,
            terminated,
        ),
{
    let byte_start = atoms[state.line_start].span().start().byte_offset();
    let content_byte_start = if state.content_start < end_atom_index {
        atoms[state.content_start].span().start().byte_offset()
    } else if terminated {
        atoms[end_atom_index].span().start().byte_offset()
    } else {
        source_len_bytes
    };
    let byte_end = if terminated {
        atoms[end_atom_index].span().end().byte_offset()
    } else {
        source_len_bytes
    };
    LayoutLine {
        line_number: state.line_number,
        start_atom_index: state.line_start as u64,
        content_atom_index: state.content_start as u64,
        end_atom_index: end_atom_index as u64,
        terminated,
        indentation_columns: state.indentation_columns,
        byte_start,
        content_byte_start,
        byte_end,
    }
}

#[verifier::rlimit(120)]
#[verifier::spinoff_prover]
pub fn analyze_profile1_layout(atomized: &AtomizedSource, limits: LayoutLimits) -> (result: Result<
    LayoutSource,
    LayoutError,
>)
    ensures
        analyze_profile1_layout_spec(atomized@, limits@) == match result {
            Ok(layout) => Ok(layout@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(layout) => {
                layout_source_corresponds_spec(atomized@, layout@) && (
                crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@)
                    ==> layout_source_well_formed_spec(atomized@, layout@)) && layout@.lines.len()
                    <= limits@.max_lines && layout@.lines.len() <= MAX_PROFILE1_LAYOUT_LINES
                    && layout@.profile_version == CRUCIBLE_YAML_PROFILE_VERSION
                    && layout@.input_transformation_version == atomized@.transformation_version
                    && layout@.transformation_version == LINE_LAYOUT_TRANSFORMATION_VERSION
                    && layout@.source_len_bytes == atomized@.source_len_bytes && layout@.bom_bytes
                    == atomized@.bom_bytes
            },
            Err(_) => true,
        },
{
    let atoms = atomized.atoms();
    if atoms.len() as u64 > MAX_PROFILE1_LEXICAL_ATOMS {
        let rejected = &atoms[MAX_PROFILE1_LEXICAL_ATOMS as usize];
        let error = LayoutError::at(
            LayoutErrorKind::InputAtomLimitExceeded,
            rejected.span().start().byte_offset(),
        );
        proof {
            reveal(analyze_profile1_layout_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return Err(error);
    }
    let line_limit = if limits.max_lines < MAX_PROFILE1_LAYOUT_LINES {
        limits.max_lines
    } else {
        MAX_PROFILE1_LAYOUT_LINES
    };
    let indentation_limit = if limits.max_indentation_columns < MAX_PROFILE1_INDENTATION_COLUMNS {
        limits.max_indentation_columns
    } else {
        MAX_PROFILE1_INDENTATION_COLUMNS
    };
    proof {
        reveal(effective_line_limit_spec);
        reveal(effective_indentation_limit_spec);
        assert(line_limit == effective_line_limit_spec(limits@));
        assert(indentation_limit == effective_indentation_limit_spec(limits@));
    }
    if !atoms.is_empty() && line_limit == 0 {
        let error = LayoutError::at(
            LayoutErrorKind::LineLimitExceeded,
            atoms[0].span().start().byte_offset(),
        );
        proof {
            reveal(analyze_profile1_layout_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return Err(error);
    }
    let mut lines: Vec<LayoutLine> = Vec::new();
    let mut index: usize = 0;
    let mut line_start: usize = 0;
    let mut content_start: usize = 0;
    let mut indentation_columns: u64 = 0;
    let mut line_number: u64 = 0;
    let mut at_indentation = true;
    let ghost expected_scan_result = layout_scan_tail_spec(
        atomized@.atoms,
        atomized@.source_len_bytes,
        0,
        initial_layout_scan_state_spec(),
        line_limit,
        indentation_limit,
    );
    proof {
        reveal(initial_layout_scan_state_spec);
        reveal(layout_line_views_spec);
        assert(layout_line_views_spec(lines@) =~= Seq::<LayoutLineView>::empty());
        assert(LayoutScanStateView {
            lines: layout_line_views_spec(lines@),
            line_start: line_start as int,
            content_start: content_start as int,
            indentation_columns,
            line_number,
            at_indentation,
        } == initial_layout_scan_state_spec());
    }
    while index < atoms.len()
        invariant
            atoms.len() as u64 <= MAX_PROFILE1_LEXICAL_ATOMS,
            crate::atom::lexical_atom_views_spec(atoms@) == atomized@.atoms,
            index <= atoms.len(),
            line_start <= content_start <= index,
            indentation_columns == (content_start - line_start) as u64,
            indentation_columns <= indentation_limit,
            indentation_limit == effective_indentation_limit_spec(limits@),
            indentation_limit <= limits@.max_indentation_columns,
            indentation_limit <= MAX_PROFILE1_INDENTATION_COLUMNS,
            line_number == lines@.len(),
            line_number <= line_limit,
            index < atoms.len() ==> line_number < line_limit,
            line_start < atoms.len() ==> line_number < line_limit,
            line_limit == effective_line_limit_spec(limits@),
            line_limit <= limits@.max_lines,
            line_limit <= MAX_PROFILE1_LAYOUT_LINES,
            at_indentation ==> content_start == index,
            line_start <= atoms.len(),
            expected_scan_result == layout_scan_tail_spec(
                atomized@.atoms,
                atomized@.source_len_bytes,
                0,
                initial_layout_scan_state_spec(),
                line_limit,
                indentation_limit,
            ),
            expected_scan_result == layout_scan_tail_spec(
                atomized@.atoms,
                atomized@.source_len_bytes,
                index as int,
                LayoutScanStateView {
                    lines: layout_line_views_spec(lines@),
                    line_start: line_start as int,
                    content_start: content_start as int,
                    indentation_columns,
                    line_number,
                    at_indentation,
                },
                line_limit,
                indentation_limit,
            ),
        decreases atoms.len() - index,
    {
        let atom = &atoms[index];
        let kind = atom.kind();
        let ghost old_index = index as int;
        let ghost old_state = LayoutScanStateView {
            lines: layout_line_views_spec(lines@),
            line_start: line_start as int,
            content_start: content_start as int,
            indentation_columns,
            line_number,
            at_indentation,
        };
        assert(atom@ == atomized@.atoms[index as int]) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        assert(old_index == index as int);
        assert(kind == atom@.kind);
        assert(atom@.kind == atomized@.atoms[index as int].kind);
        assert(kind == atomized@.atoms[old_index].kind);
        assert(old_state.at_indentation == at_indentation);
        assert(old_state.indentation_columns == indentation_columns);
        assert(old_state.line_number == line_number);
        proof {
            assert(expected_scan_result == layout_scan_tail_spec(
                atomized@.atoms,
                atomized@.source_len_bytes,
                index as int,
                old_state,
                line_limit,
                indentation_limit,
            ));
            reveal(layout_scan_tail_spec);
        }
        match (at_indentation, kind) {
            (true, LexicalAtomKind::Space) => {
                assert(atomized@.atoms[old_index].kind == LexicalAtomKind::Space);
                if indentation_columns >= indentation_limit {
                    let error = LayoutError::at(
                        LayoutErrorKind::IndentationLimitExceeded,
                        atom.span().start().byte_offset(),
                    );
                    proof {
                        lemma_layout_scan_space_error(
                            atomized@.atoms,
                            atomized@.source_len_bytes,
                            old_index,
                            old_state,
                            line_limit,
                            indentation_limit,
                        );
                        assert(expected_scan_result == Err(error@));
                        lemma_analyze_spec_from_scan_error(
                            atomized@,
                            limits@,
                            line_limit,
                            indentation_limit,
                            error@,
                        );
                    }
                    return Err(error);
                }
                proof {
                    lemma_layout_scan_space_step(
                        atomized@.atoms,
                        atomized@.source_len_bytes,
                        old_index,
                        old_state,
                        line_limit,
                        indentation_limit,
                    );
                }
                indentation_columns += 1;
                index += 1;
                content_start = index;
                proof {
                    assert(LayoutScanStateView {
                        lines: layout_line_views_spec(lines@),
                        line_start: line_start as int,
                        content_start: content_start as int,
                        indentation_columns,
                        line_number,
                        at_indentation,
                    } == LayoutScanStateView {
                        lines: old_state.lines,
                        line_start: old_state.line_start,
                        content_start: index as int,
                        indentation_columns: (old_state.indentation_columns + 1) as u64,
                        line_number: old_state.line_number,
                        at_indentation: true,
                    });
                }
            },
            (_, LexicalAtomKind::LineFeed) => {
                assert(atomized@.atoms[old_index].kind == LexicalAtomKind::LineFeed);
                let line = make_layout_line(
                    atoms,
                    atomized.source_len_bytes(),
                    LineBuildState { line_number, line_start, content_start, indentation_columns },
                    index,
                    true,
                );
                proof {
                    assert(line@ == layout_line_spec(
                        atomized@.atoms,
                        atomized@.source_len_bytes,
                        old_state,
                        index as int,
                        true,
                    )) by {
                        reveal(layout_line_spec);
                    }
                    lemma_layout_line_views_push(lines@, line);
                }
                lines.push(line);
                line_number += 1;
                index += 1;
                line_start = index;
                content_start = index;
                indentation_columns = 0;
                at_indentation = true;
                if index < atoms.len() && line_number >= line_limit {
                    let error = LayoutError::at(
                        LayoutErrorKind::LineLimitExceeded,
                        atoms[index].span().start().byte_offset(),
                    );
                    proof {
                        lemma_layout_scan_line_error(
                            atomized@.atoms,
                            atomized@.source_len_bytes,
                            old_index,
                            old_state,
                            line_limit,
                            indentation_limit,
                        );
                        assert(expected_scan_result == Err(error@));
                        lemma_analyze_spec_from_scan_error(
                            atomized@,
                            limits@,
                            line_limit,
                            indentation_limit,
                            error@,
                        );
                    }
                    return Err(error);
                }
                proof {
                    lemma_layout_scan_line_step(
                        atomized@.atoms,
                        atomized@.source_len_bytes,
                        old_index,
                        old_state,
                        line_limit,
                        indentation_limit,
                    );
                    assert(layout_line_views_spec(lines@) == old_state.lines.push(line@));
                    assert(LayoutScanStateView {
                        lines: layout_line_views_spec(lines@),
                        line_start: line_start as int,
                        content_start: content_start as int,
                        indentation_columns,
                        line_number,
                        at_indentation,
                    } == LayoutScanStateView {
                        lines: old_state.lines.push(line@),
                        line_start: index as int,
                        content_start: index as int,
                        indentation_columns: 0,
                        line_number: (old_state.line_number + 1) as u64,
                        at_indentation: true,
                    });
                }
            },
            _ => {
                proof {
                    lemma_layout_scan_content_step(
                        atomized@.atoms,
                        atomized@.source_len_bytes,
                        old_index,
                        old_state,
                        line_limit,
                        indentation_limit,
                    );
                }
                at_indentation = false;
                index += 1;
                proof {
                    assert(LayoutScanStateView {
                        lines: layout_line_views_spec(lines@),
                        line_start: line_start as int,
                        content_start: content_start as int,
                        indentation_columns,
                        line_number,
                        at_indentation,
                    } == LayoutScanStateView {
                        lines: old_state.lines,
                        line_start: old_state.line_start,
                        content_start: old_state.content_start,
                        indentation_columns: old_state.indentation_columns,
                        line_number: old_state.line_number,
                        at_indentation: false,
                    });
                }
            },
        }
        proof {
            reveal(layout_scan_tail_spec);
            assert(expected_scan_result == layout_scan_tail_spec(
                atomized@.atoms,
                atomized@.source_len_bytes,
                index as int,
                LayoutScanStateView {
                    lines: layout_line_views_spec(lines@),
                    line_start: line_start as int,
                    content_start: content_start as int,
                    indentation_columns,
                    line_number,
                    at_indentation,
                },
                line_limit,
                indentation_limit,
            ));
        }
    }

    let ghost pre_finish_state = LayoutScanStateView {
        lines: layout_line_views_spec(lines@),
        line_start: line_start as int,
        content_start: content_start as int,
        indentation_columns,
        line_number,
        at_indentation,
    };
    proof {
        assert(index == atoms.len());
        assert(expected_scan_result == layout_scan_tail_spec(
            atomized@.atoms,
            atomized@.source_len_bytes,
            index as int,
            pre_finish_state,
            line_limit,
            indentation_limit,
        ));
        reveal(layout_scan_tail_spec);
        assert(expected_scan_result == Ok(
            finish_layout_scan_spec(atomized@.atoms, atomized@.source_len_bytes, pre_finish_state),
        ));
    }
    if line_start < atoms.len() {
        let line = make_layout_line(
            atoms,
            atomized.source_len_bytes(),
            LineBuildState { line_number, line_start, content_start, indentation_columns },
            atoms.len(),
            false,
        );
        proof {
            assert(line@ == layout_line_spec(
                atomized@.atoms,
                atomized@.source_len_bytes,
                pre_finish_state,
                atomized@.atoms.len() as int,
                false,
            )) by {
                reveal(layout_line_spec);
            }
            lemma_layout_line_views_push(lines@, line);
        }
        lines.push(line);
        line_number += 1;
    }
    let _completed_line_count = line_number;
    let layout = LayoutSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        input_transformation_version: atomized.transformation_version(),
        transformation_version: LINE_LAYOUT_TRANSFORMATION_VERSION,
        source_len_bytes: atomized.source_len_bytes(),
        bom_bytes: atomized.bom_bytes(),
        lines,
    };
    proof {
        let completed_state = finish_layout_scan_spec(
            atomized@.atoms,
            atomized@.source_len_bytes,
            pre_finish_state,
        );
        reveal(finish_layout_scan_spec);
        assert(layout_line_views_spec(lines@) == completed_state.lines);
        assert(expected_scan_result == Ok(completed_state));
        lemma_analyze_spec_from_scan_success(
            atomized@,
            limits@,
            line_limit,
            indentation_limit,
            completed_state,
        );
        assert(layout@ == LayoutSourceView {
            profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
            input_transformation_version: atomized@.transformation_version,
            transformation_version: LINE_LAYOUT_TRANSFORMATION_VERSION,
            source_len_bytes: atomized@.source_len_bytes,
            bom_bytes: atomized@.bom_bytes,
            lines: completed_state.lines,
        });
        assert(analyze_profile1_layout_spec(atomized@, limits@) == Ok(layout@));
        reveal(layout_source_corresponds_spec);
        assert(exists|candidate_limits: LayoutLimitsView|
            analyze_profile1_layout_spec(atomized@, candidate_limits) == Ok(layout@)) by {
            assert(analyze_profile1_layout_spec(atomized@, limits@) == Ok(layout@));
        }
        if crate::atom::atomized_source_intrinsically_well_formed_spec(atomized@) {
            reveal(layout_source_well_formed_spec);
        }
    }
    Ok(layout)
}

} // verus!
