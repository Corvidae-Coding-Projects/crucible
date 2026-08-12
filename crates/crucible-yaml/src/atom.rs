//! Verified scalar-by-scalar lexical atomization for Crucible YAML profile 1.
//!
//! Atomization is the context-free first stage of lexing. It preserves every validated decoded
//! scalar and source span while assigning whitespace, line-feed, indicator, or content meaning.
//! Context-sensitive token formation, indentation, comments, and scalar bodies consume these
//! atoms in the following lexer stage.
#[allow(unused_imports)]
use crate::utf8::{DecodedScalarView, DecodedSourceView};
use crate::utf8::{
    DecodedSource, SourceSpan, SourceSpanView, CRUCIBLE_YAML_PROFILE_VERSION,
    MAX_PROFILE1_DECODED_SCALARS,
};
use vstd::prelude::*;

verus! {

pub const LEXICAL_ATOM_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_LEXICAL_ATOMS: u64 = MAX_PROFILE1_DECODED_SCALARS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YamlIndicator {
    BlockSequenceEntry,
    ExplicitMappingKey,
    MappingValue,
    FlowEntry,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    Comment,
    Anchor,
    Alias,
    Tag,
    LiteralBlockScalar,
    FoldedBlockScalar,
    SingleQuotedScalar,
    DoubleQuotedScalar,
    Directive,
    ReservedAt,
    ReservedGraveAccent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexicalAtomKind {
    LineFeed,
    Space,
    Tab,
    Indicator(YamlIndicator),
    Content,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtomizeLimits {
    max_atoms: u64,
}

#[verifier::ext_equal]
pub struct AtomizeLimitsView {
    pub max_atoms: u64,
}

impl View for AtomizeLimits {
    type V = AtomizeLimitsView;

    closed spec fn view(&self) -> AtomizeLimitsView {
        AtomizeLimitsView { max_atoms: self.max_atoms }
    }
}

impl AtomizeLimits {
    pub fn new(max_atoms: u64) -> (limits: Self)
        ensures
            limits@.max_atoms == max_atoms,
    {
        Self { max_atoms }
    }

    pub fn max_atoms(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_atoms,
    {
        self.max_atoms
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AtomizeErrorKind {
    AtomLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtomizeError {
    kind: AtomizeErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct AtomizeErrorView {
    pub kind: AtomizeErrorKind,
    pub byte_offset: u64,
}

impl View for AtomizeError {
    type V = AtomizeErrorView;

    closed spec fn view(&self) -> AtomizeErrorView {
        AtomizeErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl AtomizeError {
    fn at(kind: AtomizeErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (AtomizeErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: AtomizeErrorKind)
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
/// One validated decoded scalar classified for context-sensitive YAML lexing.
///
/// ```compile_fail
/// use crucible_yaml::{LexicalAtom, LexicalAtomKind, SourcePosition, SourceSpan};
///
/// let forged = LexicalAtom {
///     kind: LexicalAtomKind::Content,
///     code_point: 0xd800,
///     span: SourceSpan {
///         start: SourcePosition { byte_offset: 3, line: 0, column: 3 },
///         end: SourcePosition { byte_offset: 2, line: 0, column: 2 },
///     },
/// };
/// ```
pub struct LexicalAtom {
    kind: LexicalAtomKind,
    code_point: u32,
    span: SourceSpan,
}

#[verifier::ext_equal]
pub struct LexicalAtomView {
    pub kind: LexicalAtomKind,
    pub code_point: u32,
    pub span: SourceSpanView,
}

impl View for LexicalAtom {
    type V = LexicalAtomView;

    closed spec fn view(&self) -> LexicalAtomView {
        LexicalAtomView { kind: self.kind, code_point: self.code_point, span: self.span@ }
    }
}

impl DeepView for LexicalAtom {
    type V = LexicalAtomView;

    closed spec fn deep_view(&self) -> LexicalAtomView {
        self@
    }
}

impl LexicalAtom {
    fn new(kind: LexicalAtomKind, code_point: u32, span: SourceSpan) -> (atom: Self)
        ensures
            atom@ == (LexicalAtomView { kind, code_point, span: span@ }),
    {
        Self { kind, code_point, span }
    }

    pub fn kind(&self) -> (kind: LexicalAtomKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn code_point(&self) -> (code_point: u32)
        ensures
            code_point == self@.code_point,
    {
        self.code_point
    }

    pub fn span(&self) -> (span: &SourceSpan)
        ensures
            span@ == self@.span,
    {
        &self.span
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AtomizedSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    bom_bytes: u64,
    atoms: Vec<LexicalAtom>,
}

#[verifier::ext_equal]
pub struct AtomizedSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub bom_bytes: u64,
    pub atoms: Seq<LexicalAtomView>,
}

pub open spec fn lexical_atom_views_spec(atoms: Seq<LexicalAtom>) -> Seq<LexicalAtomView> {
    Seq::new(atoms.len(), |index: int| atoms[index]@)
}

impl View for AtomizedSource {
    type V = AtomizedSourceView;

    closed spec fn view(&self) -> AtomizedSourceView {
        AtomizedSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            bom_bytes: self.bom_bytes,
            atoms: lexical_atom_views_spec(self.atoms@),
        }
    }
}

impl AtomizedSource {
    pub fn profile_version(&self) -> (version: u16)
        ensures
            version == self@.profile_version,
    {
        self.profile_version
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

    pub fn atoms(&self) -> (atoms: &[LexicalAtom])
        ensures
            lexical_atom_views_spec(atoms@) == self@.atoms,
    {
        self.atoms.as_slice()
    }
}

pub open spec fn lexical_atom_kind_spec(code_point: u32) -> LexicalAtomKind {
    match code_point {
        0x0a => LexicalAtomKind::LineFeed,
        0x20 => LexicalAtomKind::Space,
        0x09 => LexicalAtomKind::Tab,
        0x2d => LexicalAtomKind::Indicator(YamlIndicator::BlockSequenceEntry),
        0x3f => LexicalAtomKind::Indicator(YamlIndicator::ExplicitMappingKey),
        0x3a => LexicalAtomKind::Indicator(YamlIndicator::MappingValue),
        0x2c => LexicalAtomKind::Indicator(YamlIndicator::FlowEntry),
        0x5b => LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart),
        0x5d => LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd),
        0x7b => LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart),
        0x7d => LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd),
        0x23 => LexicalAtomKind::Indicator(YamlIndicator::Comment),
        0x26 => LexicalAtomKind::Indicator(YamlIndicator::Anchor),
        0x2a => LexicalAtomKind::Indicator(YamlIndicator::Alias),
        0x21 => LexicalAtomKind::Indicator(YamlIndicator::Tag),
        0x7c => LexicalAtomKind::Indicator(YamlIndicator::LiteralBlockScalar),
        0x3e => LexicalAtomKind::Indicator(YamlIndicator::FoldedBlockScalar),
        0x27 => LexicalAtomKind::Indicator(YamlIndicator::SingleQuotedScalar),
        0x22 => LexicalAtomKind::Indicator(YamlIndicator::DoubleQuotedScalar),
        0x25 => LexicalAtomKind::Indicator(YamlIndicator::Directive),
        0x40 => LexicalAtomKind::Indicator(YamlIndicator::ReservedAt),
        0x60 => LexicalAtomKind::Indicator(YamlIndicator::ReservedGraveAccent),
        _ => LexicalAtomKind::Content,
    }
}

pub open spec fn lexical_atom_for_scalar_spec(scalar: DecodedScalarView) -> LexicalAtomView {
    LexicalAtomView {
        kind: lexical_atom_kind_spec(scalar.code_point),
        code_point: scalar.code_point,
        span: scalar.span,
    }
}

pub open spec fn lexical_atoms_for_scalars_spec(scalars: Seq<DecodedScalarView>) -> Seq<
    LexicalAtomView,
> {
    Seq::new(scalars.len(), |index: int| lexical_atom_for_scalar_spec(scalars[index]))
}

closed spec fn atomized_prefix_spec(
    scalars: Seq<DecodedScalarView>,
    atoms: Seq<LexicalAtomView>,
) -> bool {
    atoms.len() <= scalars.len() && forall|index: int|
        0 <= index < atoms.len() ==> atoms[index] == lexical_atom_for_scalar_spec(
            #[trigger] scalars[index],
        )
}

/// Exact scalar-for-atom correspondence, independent of whether a ghost decoded view is valid.
pub closed spec fn atomized_source_corresponds_spec(
    decoded: DecodedSourceView,
    atomized: AtomizedSourceView,
) -> bool {
    atomized.profile_version == CRUCIBLE_YAML_PROFILE_VERSION && atomized.transformation_version
        == LEXICAL_ATOM_TRANSFORMATION_VERSION && atomized.source_len_bytes
        == decoded.source_len_bytes && atomized.bom_bytes == decoded.bom_bytes && atomized.atoms
        == lexical_atoms_for_scalars_spec(decoded.scalars)
}

/// Semantic validity of an atom stream and the decoded source from which it was derived.
pub closed spec fn atomized_source_well_formed_spec(
    decoded: DecodedSourceView,
    atomized: AtomizedSourceView,
) -> bool {
    crate::utf8::decoded_source_well_formed_spec(decoded) && atomized_source_corresponds_spec(
        decoded,
        atomized,
    )
}

/// Intrinsic validity of an atomized source, witnessed by some valid decoded source.
pub closed spec fn atomized_source_intrinsically_well_formed_spec(
    atomized: AtomizedSourceView,
) -> bool {
    exists|decoded: DecodedSourceView| atomized_source_well_formed_spec(decoded, atomized)
}

/// Forget the decoded-source witness while retaining intrinsic atom-source validity.
pub proof fn lemma_atomized_well_formed_is_intrinsic(
    decoded: DecodedSourceView,
    atomized: AtomizedSourceView,
)
    requires
        atomized_source_well_formed_spec(decoded, atomized),
    ensures
        atomized_source_intrinsically_well_formed_spec(atomized),
{
    reveal(atomized_source_intrinsically_well_formed_spec);
    assert(exists|candidate: DecodedSourceView|
        atomized_source_well_formed_spec(candidate, atomized)) by {
        assert(atomized_source_well_formed_spec(decoded, atomized));
    }
}

/// Valid decoded input plus exact atom correspondence yields a semantically valid atom source.
pub proof fn lemma_atomized_correspondence_preserves_validity(
    decoded: DecodedSourceView,
    atomized: AtomizedSourceView,
)
    requires
        crate::utf8::decoded_source_well_formed_spec(decoded),
        atomized_source_corresponds_spec(decoded, atomized),
    ensures
        atomized_source_well_formed_spec(decoded, atomized),
{
    reveal(atomized_source_well_formed_spec);
}

/// A semantically valid atom source cannot hide an invalid decoded-source ghost view.
pub proof fn lemma_atomized_well_formed_has_valid_decoded_source(
    decoded: DecodedSourceView,
    atomized: AtomizedSourceView,
)
    requires
        atomized_source_well_formed_spec(decoded, atomized),
    ensures
        crate::utf8::decoded_source_well_formed_spec(decoded),
{
    reveal(atomized_source_well_formed_spec);
}

/// Every scalar underlying a semantically valid atom source is a normalized Unicode scalar.
pub proof fn lemma_atomized_well_formed_scalar_is_normalized(
    decoded: DecodedSourceView,
    atomized: AtomizedSourceView,
    index: int,
)
    requires
        atomized_source_well_formed_spec(decoded, atomized),
        0 <= index < decoded.scalars.len(),
    ensures
        crate::utf8::normalized_scalar_view_spec(decoded.scalars[index]),
{
    lemma_atomized_well_formed_has_valid_decoded_source(decoded, atomized);
    crate::utf8::lemma_decoded_source_well_formed_scalar_is_normalized(decoded, index);
}

/// Every atom in an intrinsically valid source denotes a normalized Unicode scalar and span.
pub proof fn lemma_intrinsic_atomized_scalar_is_normalized(atomized: AtomizedSourceView, index: int)
    requires
        atomized_source_intrinsically_well_formed_spec(atomized),
        0 <= index < atomized.atoms.len(),
    ensures
        crate::utf8::normalized_scalar_view_spec(
            DecodedScalarView {
                code_point: atomized.atoms[index].code_point,
                span: atomized.atoms[index].span,
            },
        ),
{
    reveal(atomized_source_intrinsically_well_formed_spec);
    let decoded = choose|candidate: DecodedSourceView|
        atomized_source_well_formed_spec(candidate, atomized);
    assert(atomized_source_well_formed_spec(decoded, atomized));
    reveal(atomized_source_well_formed_spec);
    reveal(atomized_source_corresponds_spec);
    assert(atomized.atoms == lexical_atoms_for_scalars_spec(decoded.scalars));
    assert(atomized.atoms.len() == decoded.scalars.len());
    lemma_atomized_well_formed_scalar_is_normalized(decoded, atomized, index);
    reveal(lexical_atoms_for_scalars_spec);
    reveal(lexical_atom_for_scalar_spec);
}

pub closed spec fn atomize_profile1_spec(
    decoded: DecodedSourceView,
    limits: AtomizeLimitsView,
) -> Result<AtomizedSourceView, AtomizeErrorView> {
    let effective_atom_limit = if limits.max_atoms < MAX_PROFILE1_LEXICAL_ATOMS {
        limits.max_atoms
    } else {
        MAX_PROFILE1_LEXICAL_ATOMS
    };
    if decoded.scalars.len() > effective_atom_limit {
        Err(
            AtomizeErrorView {
                kind: AtomizeErrorKind::AtomLimitExceeded,
                byte_offset: decoded.scalars[effective_atom_limit as int].span.start.byte_offset,
            },
        )
    } else {
        Ok(
            AtomizedSourceView {
                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                transformation_version: LEXICAL_ATOM_TRANSFORMATION_VERSION,
                source_len_bytes: decoded.source_len_bytes,
                bom_bytes: decoded.bom_bytes,
                atoms: lexical_atoms_for_scalars_spec(decoded.scalars),
            },
        )
    }
}

/// Evaluate the total atomization result when both caller and profile caps admit every scalar.
pub proof fn lemma_atomize_within_limits(decoded: DecodedSourceView, limits: AtomizeLimitsView)
    requires
        decoded.scalars.len() <= limits.max_atoms,
        decoded.scalars.len() <= MAX_PROFILE1_LEXICAL_ATOMS,
    ensures
        atomize_profile1_spec(decoded, limits) == Ok(
            AtomizedSourceView {
                profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
                transformation_version: LEXICAL_ATOM_TRANSFORMATION_VERSION,
                source_len_bytes: decoded.source_len_bytes,
                bom_bytes: decoded.bom_bytes,
                atoms: lexical_atoms_for_scalars_spec(decoded.scalars),
            },
        ),
{
    reveal(atomize_profile1_spec);
}

/// Evaluate the exact first-rejected scalar when the effective atom cap is exceeded.
pub proof fn lemma_atomize_limit_error(decoded: DecodedSourceView, limits: AtomizeLimitsView)
    requires
        decoded.scalars.len() > if limits.max_atoms < MAX_PROFILE1_LEXICAL_ATOMS {
            limits.max_atoms
        } else {
            MAX_PROFILE1_LEXICAL_ATOMS
        },
    ensures
        atomize_profile1_spec(decoded, limits) == Err(
            AtomizeErrorView {
                kind: AtomizeErrorKind::AtomLimitExceeded,
                byte_offset: decoded.scalars[(if limits.max_atoms < MAX_PROFILE1_LEXICAL_ATOMS {
                    limits.max_atoms
                } else {
                    MAX_PROFILE1_LEXICAL_ATOMS
                }) as int].span.start.byte_offset,
            },
        ),
{
    reveal(atomize_profile1_spec);
}

pub fn classify_lexical_atom(code_point: u32) -> (kind: LexicalAtomKind)
    ensures
        kind == lexical_atom_kind_spec(code_point),
{
    match code_point {
        0x0a => LexicalAtomKind::LineFeed,
        0x20 => LexicalAtomKind::Space,
        0x09 => LexicalAtomKind::Tab,
        0x2d => LexicalAtomKind::Indicator(YamlIndicator::BlockSequenceEntry),
        0x3f => LexicalAtomKind::Indicator(YamlIndicator::ExplicitMappingKey),
        0x3a => LexicalAtomKind::Indicator(YamlIndicator::MappingValue),
        0x2c => LexicalAtomKind::Indicator(YamlIndicator::FlowEntry),
        0x5b => LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceStart),
        0x5d => LexicalAtomKind::Indicator(YamlIndicator::FlowSequenceEnd),
        0x7b => LexicalAtomKind::Indicator(YamlIndicator::FlowMappingStart),
        0x7d => LexicalAtomKind::Indicator(YamlIndicator::FlowMappingEnd),
        0x23 => LexicalAtomKind::Indicator(YamlIndicator::Comment),
        0x26 => LexicalAtomKind::Indicator(YamlIndicator::Anchor),
        0x2a => LexicalAtomKind::Indicator(YamlIndicator::Alias),
        0x21 => LexicalAtomKind::Indicator(YamlIndicator::Tag),
        0x7c => LexicalAtomKind::Indicator(YamlIndicator::LiteralBlockScalar),
        0x3e => LexicalAtomKind::Indicator(YamlIndicator::FoldedBlockScalar),
        0x27 => LexicalAtomKind::Indicator(YamlIndicator::SingleQuotedScalar),
        0x22 => LexicalAtomKind::Indicator(YamlIndicator::DoubleQuotedScalar),
        0x25 => LexicalAtomKind::Indicator(YamlIndicator::Directive),
        0x40 => LexicalAtomKind::Indicator(YamlIndicator::ReservedAt),
        0x60 => LexicalAtomKind::Indicator(YamlIndicator::ReservedGraveAccent),
        _ => LexicalAtomKind::Content,
    }
}

proof fn lemma_empty_atomized_prefix(scalars: Seq<DecodedScalarView>)
    ensures
        atomized_prefix_spec(scalars, Seq::empty()),
{
    reveal(atomized_prefix_spec);
}

proof fn lemma_extend_atomized_prefix(
    scalars: Seq<DecodedScalarView>,
    atoms: Seq<LexicalAtomView>,
    atom: LexicalAtomView,
)
    requires
        atomized_prefix_spec(scalars, atoms),
        atoms.len() < scalars.len(),
        atom == lexical_atom_for_scalar_spec(scalars[atoms.len() as int]),
    ensures
        atomized_prefix_spec(scalars, atoms.push(atom)),
{
    reveal(atomized_prefix_spec);
    assert forall|index: int| 0 <= index < atoms.push(atom).len() implies atoms.push(atom)[index]
        == lexical_atom_for_scalar_spec(#[trigger] scalars[index]) by {
        if index < atoms.len() {
            assert(atoms.push(atom)[index] == atoms[index]);
        } else {
            assert(index == atoms.len());
            assert(atoms.push(atom)[index] == atom);
        }
    }
}

proof fn lemma_lexical_atom_views_push(atoms: Seq<LexicalAtom>, atom: LexicalAtom)
    ensures
        lexical_atom_views_spec(atoms.push(atom)) == lexical_atom_views_spec(atoms).push(atom@),
{
    reveal(lexical_atom_views_spec);
    assert forall|index: int|
        0 <= index < atoms.push(atom).len() implies #[trigger] lexical_atom_views_spec(
        atoms.push(atom),
    )[index] == lexical_atom_views_spec(atoms).push(atom@)[index] by {
        if index < atoms.len() {
            assert(atoms.push(atom)[index] == atoms[index]);
        } else {
            assert(index == atoms.len());
            assert(atoms.push(atom)[index] == atom);
        }
    }
}

#[verifier::spinoff_prover]
proof fn lemma_complete_atomized_prefix(
    scalars: Seq<DecodedScalarView>,
    atoms: Seq<LexicalAtomView>,
)
    requires
        atomized_prefix_spec(scalars, atoms),
        atoms.len() == scalars.len(),
    ensures
        atoms == lexical_atoms_for_scalars_spec(scalars),
{
    reveal(atomized_prefix_spec);
    reveal(lexical_atoms_for_scalars_spec);
    assert forall|index: int| 0 <= index < atoms.len() implies #[trigger] atoms[index]
        == lexical_atoms_for_scalars_spec(scalars)[index] by {
        assert(atoms[index] == lexical_atom_for_scalar_spec(scalars[index]));
    }
}

#[verifier::rlimit(100)]
#[verifier::spinoff_prover]
pub fn atomize_profile1(decoded: &DecodedSource, limits: AtomizeLimits) -> (result: Result<
    AtomizedSource,
    AtomizeError,
>)
    ensures
        atomize_profile1_spec(decoded@, limits@) == match result {
            Ok(atomized) => Ok(atomized@),
            Err(error) => Err(error@),
        },
        match result {
            Ok(atomized) => {
                atomized_source_corresponds_spec(decoded@, atomized@) && (
                crate::utf8::decoded_source_well_formed_spec(decoded@)
                    ==> atomized_source_well_formed_spec(decoded@, atomized@))
                    && atomized@.atoms.len() <= limits@.max_atoms && atomized@.atoms.len()
                    <= MAX_PROFILE1_LEXICAL_ATOMS
            },
            Err(_) => true,
        },
{
    let effective_atom_limit = if limits.max_atoms < MAX_PROFILE1_LEXICAL_ATOMS {
        limits.max_atoms
    } else {
        MAX_PROFILE1_LEXICAL_ATOMS
    };
    let scalars = decoded.scalars();
    if scalars.len() as u64 > effective_atom_limit {
        let rejected = &scalars[effective_atom_limit as usize];
        let rejected_span = rejected.span();
        let error = AtomizeError::at(
            AtomizeErrorKind::AtomLimitExceeded,
            rejected_span.start().byte_offset(),
        );
        proof {
            reveal(atomize_profile1_spec);
            reveal(crate::utf8::decoded_scalar_views_spec);
        }
        return Err(error);
    }
    let mut atoms: Vec<LexicalAtom> = Vec::new();
    let mut index: usize = 0;
    proof {
        lemma_empty_atomized_prefix(decoded@.scalars);
    }
    while index < scalars.len()
        invariant
            index <= scalars.len(),
            scalars.len() as u64 <= effective_atom_limit,
            effective_atom_limit <= limits@.max_atoms,
            effective_atom_limit <= MAX_PROFILE1_LEXICAL_ATOMS,
            crate::utf8::decoded_scalar_views_spec(scalars@) == decoded@.scalars,
            atoms@.len() == index,
            atomized_prefix_spec(decoded@.scalars, lexical_atom_views_spec(atoms@)),
        decreases scalars.len() - index,
    {
        let scalar = &scalars[index];
        let code_point = scalar.code_point();
        let span = *scalar.span();
        let kind = classify_lexical_atom(code_point);
        let atom = LexicalAtom::new(kind, code_point, span);
        assert(scalar@ == decoded@.scalars[index as int]) by {
            reveal(crate::utf8::decoded_scalar_views_spec);
        }
        assert(atom@ == lexical_atom_for_scalar_spec(decoded@.scalars[index as int]));
        proof {
            lemma_extend_atomized_prefix(decoded@.scalars, lexical_atom_views_spec(atoms@), atom@);
            lemma_lexical_atom_views_push(atoms@, atom);
        }
        atoms.push(atom);
        index += 1;
    }

    proof {
        lemma_complete_atomized_prefix(decoded@.scalars, lexical_atom_views_spec(atoms@));
    }
    let atomized = AtomizedSource {
        profile_version: CRUCIBLE_YAML_PROFILE_VERSION,
        transformation_version: LEXICAL_ATOM_TRANSFORMATION_VERSION,
        source_len_bytes: decoded.source_len_bytes(),
        bom_bytes: decoded.bom_bytes(),
        atoms,
    };
    proof {
        reveal(atomized_source_corresponds_spec);
        reveal(atomize_profile1_spec);
        if crate::utf8::decoded_source_well_formed_spec(decoded@) {
            lemma_atomized_correspondence_preserves_validity(decoded@, atomized@);
        }
    }
    Ok(atomized)
}

} // verus!
