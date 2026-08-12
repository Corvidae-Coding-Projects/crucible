//! Verified style-specific scalar content decoding for Crucible YAML profile 1.
//!
//! Block scalars are already normalized by the authenticated block-scalar machine. Their decoder
//! copies that content into the shared representation, while the single-quoted decoder implements
//! quote doubling and flow-line folding directly. Both retain exact atom and byte provenance;
//! double-quoted and plain styles extend the same model.
use crate::atom::{AtomizedSource, LexicalAtom};
#[allow(unused_imports)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::block::{
    BlockScalarContentOrigin, BlockScalarContentScalar, BlockScalarSource, BlockScalarStyle,
};
#[allow(unused_imports)]
use crate::block::{BlockScalarContentScalarView, BlockScalarSourceView};
use crate::quoted::{QuotedScalar, QuotedScalarSource, QuotedScalarStyle};
#[allow(unused_imports)]
use crate::quoted::{QuotedScalarSourceView, QuotedScalarView};
use vstd::prelude::*;

verus! {

pub const SCALAR_DECODE_TRANSFORMATION_VERSION: u16 = 1;

pub const MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarDecodeLimits {
    max_content_code_points: u64,
}

#[verifier::ext_equal]
pub struct ScalarDecodeLimitsView {
    pub max_content_code_points: u64,
}

impl View for ScalarDecodeLimits {
    type V = ScalarDecodeLimitsView;

    closed spec fn view(&self) -> ScalarDecodeLimitsView {
        ScalarDecodeLimitsView { max_content_code_points: self.max_content_code_points }
    }
}

impl ScalarDecodeLimits {
    pub fn new(max_content_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (ScalarDecodeLimitsView { max_content_code_points }),
    {
        Self { max_content_code_points }
    }

    pub fn max_content_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_content_code_points,
    {
        self.max_content_code_points
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum DecodedScalarStyle {
    LiteralBlock,
    FoldedBlock,
    SingleQuoted,
    DoubleQuoted,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum DecodedContentOrigin {
    Direct,
    FoldedLineBreak,
    SingleQuoteDoubled,
    DoubleQuotedEscape,
    EscapedLineBreak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedContentScalar {
    code_point: u32,
    source_atom_start: u64,
    source_atom_end: u64,
    byte_start: u64,
    byte_end: u64,
    origin: DecodedContentOrigin,
}

#[verifier::ext_equal]
pub struct DecodedContentScalarView {
    pub code_point: u32,
    pub source_atom_start: u64,
    pub source_atom_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub origin: DecodedContentOrigin,
}

impl View for DecodedContentScalar {
    type V = DecodedContentScalarView;

    closed spec fn view(&self) -> DecodedContentScalarView {
        DecodedContentScalarView {
            code_point: self.code_point,
            source_atom_start: self.source_atom_start,
            source_atom_end: self.source_atom_end,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            origin: self.origin,
        }
    }
}

impl DecodedContentScalar {
    fn new(
        code_point: u32,
        source_atom_start: u64,
        source_atom_end: u64,
        byte_start: u64,
        byte_end: u64,
        origin: DecodedContentOrigin,
    ) -> (scalar: Self)
        ensures
            scalar@ == (DecodedContentScalarView {
                code_point,
                source_atom_start,
                source_atom_end,
                byte_start,
                byte_end,
                origin,
            }),
    {
        Self { code_point, source_atom_start, source_atom_end, byte_start, byte_end, origin }
    }

    pub fn code_point(&self) -> (code_point: u32)
        ensures
            code_point == self@.code_point,
    {
        self.code_point
    }

    pub fn source_atom_start(&self) -> (index: u64)
        ensures
            index == self@.source_atom_start,
    {
        self.source_atom_start
    }

    pub fn source_atom_end(&self) -> (index: u64)
        ensures
            index == self@.source_atom_end,
    {
        self.source_atom_end
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

    pub fn origin(&self) -> (origin: DecodedContentOrigin)
        ensures
            origin == self@.origin,
    {
        self.origin
    }
}

pub open spec fn decoded_content_scalar_views_spec(content: Seq<DecodedContentScalar>) -> Seq<
    DecodedContentScalarView,
> {
    Seq::new(content.len(), |index: int| content[index]@)
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedScalarContent {
    style: DecodedScalarStyle,
    content: Vec<DecodedContentScalar>,
}

#[verifier::ext_equal]
pub struct DecodedScalarContentView {
    pub style: DecodedScalarStyle,
    pub content: Seq<DecodedContentScalarView>,
}

impl View for DecodedScalarContent {
    type V = DecodedScalarContentView;

    closed spec fn view(&self) -> DecodedScalarContentView {
        DecodedScalarContentView {
            style: self.style,
            content: decoded_content_scalar_views_spec(self.content@),
        }
    }
}

impl DecodedScalarContent {
    fn new(style: DecodedScalarStyle, content: Vec<DecodedContentScalar>) -> (decoded: Self)
        ensures
            decoded@ == (DecodedScalarContentView {
                style,
                content: decoded_content_scalar_views_spec(content@),
            }),
    {
        Self { style, content }
    }

    pub fn style(&self) -> (style: DecodedScalarStyle)
        ensures
            style == self@.style,
    {
        self.style
    }

    pub fn content(&self) -> (content: &[DecodedContentScalar])
        ensures
            decoded_content_scalar_views_spec(content@) == self@.content,
    {
        self.content.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum ScalarDecodeErrorKind {
    InputQuotedMismatch,
    ScalarIndexOutOfRange,
    ScalarStyleMismatch,
    ContentLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarDecodeError {
    kind: ScalarDecodeErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct ScalarDecodeErrorView {
    pub kind: ScalarDecodeErrorKind,
    pub byte_offset: u64,
}

impl View for ScalarDecodeError {
    type V = ScalarDecodeErrorView;

    closed spec fn view(&self) -> ScalarDecodeErrorView {
        ScalarDecodeErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl ScalarDecodeError {
    fn at(kind: ScalarDecodeErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (ScalarDecodeErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: ScalarDecodeErrorKind)
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

pub open spec fn effective_scalar_content_limit_spec(limits: ScalarDecodeLimitsView) -> u64 {
    if limits.max_content_code_points < MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS {
        limits.max_content_code_points
    } else {
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS
    }
}

pub open spec fn decoded_block_style_spec(style: BlockScalarStyle) -> DecodedScalarStyle {
    match style {
        BlockScalarStyle::Literal => DecodedScalarStyle::LiteralBlock,
        BlockScalarStyle::Folded => DecodedScalarStyle::FoldedBlock,
    }
}

pub open spec fn decoded_block_origin_spec(
    origin: BlockScalarContentOrigin,
) -> DecodedContentOrigin {
    match origin {
        BlockScalarContentOrigin::Direct => DecodedContentOrigin::Direct,
        BlockScalarContentOrigin::FoldedLineBreak => DecodedContentOrigin::FoldedLineBreak,
    }
}

pub open spec fn decoded_block_content_item_spec(
    source: BlockScalarContentScalarView,
) -> DecodedContentScalarView {
    DecodedContentScalarView {
        code_point: source.code_point,
        source_atom_start: source.source_atom_index,
        source_atom_end: if source.source_atom_index == u64::MAX {
            u64::MAX
        } else {
            (source.source_atom_index + 1) as u64
        },
        byte_start: source.byte_start,
        byte_end: source.byte_end,
        origin: decoded_block_origin_spec(source.origin),
    }
}

pub open spec fn decoded_block_content_prefix_spec(
    source: Seq<BlockScalarContentScalarView>,
    end: nat,
) -> Seq<DecodedContentScalarView> {
    Seq::new(end, |index: int| decoded_block_content_item_spec(source[index]))
}

pub open spec fn decoded_block_content_spec(source: Seq<BlockScalarContentScalarView>) -> Seq<
    DecodedContentScalarView,
> {
    decoded_block_content_prefix_spec(source, source.len())
}

pub open spec fn decode_block_content_spec(
    style: BlockScalarStyle,
    source: Seq<BlockScalarContentScalarView>,
    limits: ScalarDecodeLimitsView,
) -> Result<DecodedScalarContentView, ScalarDecodeErrorView> {
    let effective_limit = effective_scalar_content_limit_spec(limits);
    if source.len() > effective_limit {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::ContentLimitExceeded,
                byte_offset: source[effective_limit as int].byte_start,
            },
        )
    } else {
        Ok(
            DecodedScalarContentView {
                style: decoded_block_style_spec(style),
                content: decoded_block_content_spec(source),
            },
        )
    }
}

pub open spec fn decode_profile1_block_scalar_content_spec(
    source: BlockScalarSourceView,
    scalar_index: u64,
    limits: ScalarDecodeLimitsView,
) -> Result<DecodedScalarContentView, ScalarDecodeErrorView> {
    if scalar_index >= source.scalars.len() {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::ScalarIndexOutOfRange,
                byte_offset: source.source_len_bytes,
            },
        )
    } else {
        let scalar = source.scalars[scalar_index as int];
        decode_block_content_spec(scalar.style, scalar.content, limits)
    }
}

pub open spec fn scalar_atom_white_spec(atom: LexicalAtomView) -> bool {
    atom.code_point == 0x20 || atom.code_point == 0x09
}

pub open spec fn skip_scalar_white_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> int
    decreases fuel,
{
    if fuel == 0 || index < 0 || end > atoms.len() || index >= end || !scalar_atom_white_spec(
        atoms[index],
    ) {
        index
    } else {
        skip_scalar_white_spec(atoms, index + 1, end, (fuel - 1) as nat)
    }
}

pub open spec fn single_quoted_break_group_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
) -> (Seq<int>, int)
    decreases fuel,
{
    if fuel == 0 || index < 0 || end > atoms.len() || index >= end || atoms[index].code_point
        != 0x0a {
        (Seq::empty(), index)
    } else {
        let after_white = skip_scalar_white_spec(atoms, index + 1, end, (end - index - 1) as nat);
        if after_white < end && atoms[after_white].code_point == 0x0a {
            let tail = single_quoted_break_group_spec(atoms, after_white, end, (fuel - 1) as nat);
            (Seq::empty().push(index) + tail.0, tail.1)
        } else {
            (Seq::empty().push(index), after_white)
        }
    }
}

proof fn lemma_skip_scalar_white_lower_bound(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
)
    requires
        0 <= index <= end <= atoms.len(),
    ensures
        index <= skip_scalar_white_spec(atoms, index, end, fuel),
    decreases fuel,
{
    reveal(skip_scalar_white_spec);
    if fuel > 0 && index < end && scalar_atom_white_spec(atoms[index]) {
        lemma_skip_scalar_white_lower_bound(atoms, index + 1, end, (fuel - 1) as nat);
    }
}

proof fn lemma_skip_scalar_white_progress(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    fuel: nat,
)
    requires
        0 <= index < end <= atoms.len(),
        fuel > 0,
        scalar_atom_white_spec(atoms[index]),
    ensures
        index < skip_scalar_white_spec(atoms, index, end, fuel),
{
    reveal(skip_scalar_white_spec);
    lemma_skip_scalar_white_lower_bound(atoms, index + 1, end, (fuel - 1) as nat);
}

pub open spec fn direct_atom_content_spec(
    atom: LexicalAtomView,
    atom_index: int,
    code_point: u32,
    origin: DecodedContentOrigin,
) -> DecodedContentScalarView {
    DecodedContentScalarView {
        code_point,
        source_atom_start: atom_index as u64,
        source_atom_end: (atom_index + 1) as u64,
        byte_start: atom.span.start.byte_offset,
        byte_end: atom.span.end.byte_offset,
        origin,
    }
}

pub open spec fn doubled_quote_content_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
) -> DecodedContentScalarView {
    DecodedContentScalarView {
        code_point: 0x27,
        source_atom_start: index as u64,
        source_atom_end: (index + 2) as u64,
        byte_start: atoms[index].span.start.byte_offset,
        byte_end: atoms[index + 1].span.end.byte_offset,
        origin: DecodedContentOrigin::SingleQuoteDoubled,
    }
}

pub open spec fn folded_break_content_spec(atoms: Seq<LexicalAtomView>, breaks: Seq<int>) -> Seq<
    DecodedContentScalarView,
> {
    if breaks.len() == 1 {
        Seq::empty().push(
            direct_atom_content_spec(
                atoms[breaks[0]],
                breaks[0],
                0x20,
                DecodedContentOrigin::FoldedLineBreak,
            ),
        )
    } else {
        Seq::new(
            (breaks.len() - 1) as nat,
            |offset: int|
                direct_atom_content_spec(
                    atoms[breaks[offset + 1]],
                    breaks[offset + 1],
                    0x0a,
                    DecodedContentOrigin::FoldedLineBreak,
                ),
        )
    }
}

pub open spec fn single_quoted_step_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    remaining: u64,
) -> Result<(Seq<DecodedContentScalarView>, int), ScalarDecodeErrorView> {
    let atom = atoms[index];
    if scalar_atom_white_spec(atom) {
        let after_white = skip_scalar_white_spec(atoms, index, end, (end - index) as nat);
        if after_white < end && atoms[after_white].code_point == 0x0a {
            Ok((Seq::empty(), after_white))
        } else {
            let item = direct_atom_content_spec(
                atom,
                index,
                atom.code_point,
                DecodedContentOrigin::Direct,
            );
            if remaining == 0 {
                Err(
                    ScalarDecodeErrorView {
                        kind: ScalarDecodeErrorKind::ContentLimitExceeded,
                        byte_offset: item.byte_start,
                    },
                )
            } else {
                Ok((Seq::empty().push(item), index + 1))
            }
        }
    } else if atom.code_point == 0x0a {
        let group = single_quoted_break_group_spec(atoms, index, end, (end - index) as nat);
        let additions = folded_break_content_spec(atoms, group.0);
        if additions.len() > remaining {
            Err(
                ScalarDecodeErrorView {
                    kind: ScalarDecodeErrorKind::ContentLimitExceeded,
                    byte_offset: additions[remaining as int].byte_start,
                },
            )
        } else {
            Ok((additions, group.1))
        }
    } else if atom.code_point == 0x27 {
        if index + 1 < end && atoms[index + 1].code_point == 0x27 {
            let item = doubled_quote_content_spec(atoms, index);
            if remaining == 0 {
                Err(
                    ScalarDecodeErrorView {
                        kind: ScalarDecodeErrorKind::ContentLimitExceeded,
                        byte_offset: item.byte_start,
                    },
                )
            } else {
                Ok((Seq::empty().push(item), index + 2))
            }
        } else {
            Err(
                ScalarDecodeErrorView {
                    kind: ScalarDecodeErrorKind::InputQuotedMismatch,
                    byte_offset: atom.span.start.byte_offset,
                },
            )
        }
    } else {
        let item = direct_atom_content_spec(
            atom,
            index,
            atom.code_point,
            DecodedContentOrigin::Direct,
        );
        if remaining == 0 {
            Err(
                ScalarDecodeErrorView {
                    kind: ScalarDecodeErrorKind::ContentLimitExceeded,
                    byte_offset: item.byte_start,
                },
            )
        } else {
            Ok((Seq::empty().push(item), index + 1))
        }
    }
}

pub open spec fn prepend_decoded_content_result_spec(
    prefix: Seq<DecodedContentScalarView>,
    result: Result<Seq<DecodedContentScalarView>, ScalarDecodeErrorView>,
) -> Result<Seq<DecodedContentScalarView>, ScalarDecodeErrorView> {
    match result {
        Ok(tail) => Ok(prefix + tail),
        Err(error) => Err(error),
    }
}

proof fn lemma_prepend_decoded_content_associative(
    first: Seq<DecodedContentScalarView>,
    second: Seq<DecodedContentScalarView>,
    result: Result<Seq<DecodedContentScalarView>, ScalarDecodeErrorView>,
)
    ensures
        prepend_decoded_content_result_spec(
            first,
            prepend_decoded_content_result_spec(second, result),
        ) == prepend_decoded_content_result_spec(first + second, result),
{
    reveal(prepend_decoded_content_result_spec);
    match result {
        Ok(tail) => {
            assert(first + (second + tail) =~= (first + second) + tail);
        },
        Err(_) => {},
    }
}

pub open spec fn decode_single_quoted_loop_spec(
    atoms: Seq<LexicalAtomView>,
    index: int,
    end: int,
    remaining: u64,
    fuel: nat,
) -> Result<Seq<DecodedContentScalarView>, ScalarDecodeErrorView>
    decreases fuel,
{
    if index >= end {
        Ok(Seq::empty())
    } else if fuel == 0 {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::InputQuotedMismatch,
                byte_offset: atoms[index].span.start.byte_offset,
            },
        )
    } else {
        match single_quoted_step_spec(atoms, index, end, remaining) {
            Err(error) => Err(error),
            Ok((prefix, next)) => prepend_decoded_content_result_spec(
                prefix,
                decode_single_quoted_loop_spec(
                    atoms,
                    next,
                    end,
                    (remaining - prefix.len()) as u64,
                    (fuel - 1) as nat,
                ),
            ),
        }
    }
}

pub open spec fn quoted_scalar_range_matches_atoms_spec(
    atoms: Seq<LexicalAtomView>,
    quote: QuotedScalarView,
) -> bool {
    quote.start_atom_index + 2 <= quote.end_atom_index && quote.end_atom_index <= atoms.len()
        && atoms[quote.start_atom_index as int].code_point == 0x27 && atoms[(quote.end_atom_index
        - 1) as int].code_point == 0x27 && quote.byte_start
        == atoms[quote.start_atom_index as int].span.start.byte_offset && quote.byte_end == atoms[(
    quote.end_atom_index - 1) as int].span.end.byte_offset
}

pub open spec fn decode_single_quoted_content_spec(
    atoms: Seq<LexicalAtomView>,
    quote: QuotedScalarView,
    limits: ScalarDecodeLimitsView,
) -> Result<DecodedScalarContentView, ScalarDecodeErrorView> {
    if quote.style != QuotedScalarStyle::Single {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::ScalarStyleMismatch,
                byte_offset: quote.byte_start,
            },
        )
    } else if !quoted_scalar_range_matches_atoms_spec(atoms, quote) {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::InputQuotedMismatch,
                byte_offset: quote.byte_start,
            },
        )
    } else {
        let start = quote.start_atom_index + 1;
        let end = quote.end_atom_index - 1;
        match decode_single_quoted_loop_spec(
            atoms,
            start as int,
            end as int,
            effective_scalar_content_limit_spec(limits),
            (end - start + 1) as nat,
        ) {
            Ok(content) => Ok(
                DecodedScalarContentView { style: DecodedScalarStyle::SingleQuoted, content },
            ),
            Err(error) => Err(error),
        }
    }
}

pub open spec fn decode_profile1_single_quoted_scalar_content_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    scalar_index: u64,
    limits: ScalarDecodeLimitsView,
) -> Result<DecodedScalarContentView, ScalarDecodeErrorView> {
    if quoted.profile_version != atomized.profile_version || quoted.input_transformation_version
        != atomized.transformation_version || quoted.source_len_bytes != atomized.source_len_bytes
        || quoted.bom_bytes != atomized.bom_bytes || quoted.input_atom_count
        != atomized.atoms.len() {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::InputQuotedMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if scalar_index >= quoted.scalars.len() {
        Err(
            ScalarDecodeErrorView {
                kind: ScalarDecodeErrorKind::ScalarIndexOutOfRange,
                byte_offset: atomized.source_len_bytes,
            },
        )
    } else {
        decode_single_quoted_content_spec(
            atomized.atoms,
            quoted.scalars[scalar_index as int],
            limits,
        )
    }
}

fn scalar_atom_white(atom: &LexicalAtom) -> (white: bool)
    ensures
        white == scalar_atom_white_spec(atom@),
{
    atom.code_point() == 0x20 || atom.code_point() == 0x09
}

fn skip_scalar_white(atoms: &[LexicalAtom], start: usize, end: usize) -> (next: usize)
    requires
        start <= end <= atoms@.len(),
    ensures
        next as int == skip_scalar_white_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
        start <= next <= end,
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = skip_scalar_white_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut index = start;
    while index < end && scalar_atom_white(&atoms[index])
        invariant
            start <= index <= end <= atoms@.len(),
            views == crate::atom::lexical_atom_views_spec(atoms@),
            expected == skip_scalar_white_spec(
                views,
                index as int,
                end as int,
                (end - index) as nat,
            ),
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        proof {
            reveal(skip_scalar_white_spec);
        }
        index += 1;
    }
    proof {
        reveal(skip_scalar_white_spec);
        if index < end {
            assert(views[index as int] == atoms[index as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
        }
    }
    index
}

pub open spec fn usize_indices_as_int_spec(indices: Seq<usize>) -> Seq<int> {
    Seq::new(indices.len(), |index: int| indices[index] as int)
}

proof fn lemma_usize_indices_as_int_push(indices: Seq<usize>, index: usize)
    ensures
        usize_indices_as_int_spec(indices.push(index)) == usize_indices_as_int_spec(indices).push(
            index as int,
        ),
{
    reveal(usize_indices_as_int_spec);
    assert(usize_indices_as_int_spec(indices.push(index)) =~= usize_indices_as_int_spec(
        indices,
    ).push(index as int));
}

fn single_quoted_break_group(atoms: &[LexicalAtom], start: usize, end: usize) -> (group: (
    Vec<usize>,
    usize,
))
    requires
        start < end <= atoms@.len(),
        atoms@[start as int]@.code_point == 0x0a,
    ensures
        (usize_indices_as_int_spec(group.0@), group.1 as int) == single_quoted_break_group_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            start as int,
            end as int,
            (end - start) as nat,
        ),
        group.0@.len() > 0,
        start < group.1 <= end,
        forall|position: int|
            0 <= position < group.0@.len() ==> start <= #[trigger] group.0@[position] < end,
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = single_quoted_break_group_spec(
        views,
        start as int,
        end as int,
        (end - start) as nat,
    );
    let mut breaks = Vec::new();
    let mut index = start;
    let mut _spec_fuel = end - start;
    while index < end && atoms[index].code_point() == 0x0a
        invariant
            start <= index <= end <= atoms@.len(),
            _spec_fuel >= end - index,
            views == crate::atom::lexical_atom_views_spec(atoms@),
            breaks@.len() > 0 || index == start,
            forall|position: int|
                0 <= position < breaks@.len() ==> start <= #[trigger] breaks@[position] < end,
            expected.0 == usize_indices_as_int_spec(breaks@) + single_quoted_break_group_spec(
                views,
                index as int,
                end as int,
                _spec_fuel as nat,
            ).0,
            expected.1 == single_quoted_break_group_spec(
                views,
                index as int,
                end as int,
                _spec_fuel as nat,
            ).1,
        decreases end - index,
    {
        assert(views[index as int] == atoms[index as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let ghost prior_breaks = breaks@;
        let ghost prior_group = single_quoted_break_group_spec(
            views,
            index as int,
            end as int,
            _spec_fuel as nat,
        );
        breaks.push(index);
        let after_white = skip_scalar_white(atoms, index + 1, end);
        proof {
            lemma_usize_indices_as_int_push(prior_breaks, index);
            assert(index + 1 <= end);
            assert(after_white as int == skip_scalar_white_spec(
                views,
                index as int + 1,
                end as int,
                (end - index - 1) as nat,
            ));
            reveal(single_quoted_break_group_spec);
            reveal(usize_indices_as_int_spec);
            if after_white < end && views[after_white as int].code_point == 0x0a {
                assert(prior_group.0 == Seq::empty().push(index as int)
                    + single_quoted_break_group_spec(
                    views,
                    after_white as int,
                    end as int,
                    (_spec_fuel - 1) as nat,
                ).0);
            }
            if after_white >= end || views[after_white as int].code_point != 0x0a {
                reveal(single_quoted_break_group_spec);
            }
        }
        index = after_white;
        _spec_fuel -= 1;
    }
    proof {
        reveal(single_quoted_break_group_spec);
        reveal(usize_indices_as_int_spec);
        assert(breaks@.len() > 0);
    }
    (breaks, index)
}

fn direct_atom_content(
    atom: &LexicalAtom,
    atom_index: usize,
    code_point: u32,
    origin: DecodedContentOrigin,
) -> (content: DecodedContentScalar)
    requires
        atom_index < u64::MAX,
    ensures
        content@ == direct_atom_content_spec(atom@, atom_index as int, code_point, origin),
{
    let content = DecodedContentScalar::new(
        code_point,
        atom_index as u64,
        atom_index as u64 + 1,
        atom.span().start().byte_offset(),
        atom.span().end().byte_offset(),
        origin,
    );
    proof {
        reveal(direct_atom_content_spec);
    }
    content
}

fn decoded_single_quoted_step(
    atoms: &[LexicalAtom],
    index: usize,
    end: usize,
    remaining: u64,
) -> (result: Result<(Vec<DecodedContentScalar>, usize), ScalarDecodeError>)
    requires
        index < end <= atoms@.len(),
        end <= u64::MAX,
    ensures
        single_quoted_step_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            index as int,
            end as int,
            remaining,
        ) == match result {
            Ok((content, next)) => Ok((decoded_content_scalar_views_spec(content@), next as int)),
            Err(error) => Err(error@),
        },
        match result {
            Ok((content, next)) => index < next <= end && content@.len() <= remaining,
            Err(_) => true,
        },
{
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let atom = &atoms[index];
    assert(views[index as int] == atom@) by {
        reveal(crate::atom::lexical_atom_views_spec);
    }
    if scalar_atom_white(atom) {
        let after_white = skip_scalar_white(atoms, index, end);
        assert(after_white > index) by {
            lemma_skip_scalar_white_progress(views, index as int, end as int, (end - index) as nat);
        }
        if after_white < end {
            assert(views[after_white as int] == atoms[after_white as int]@) by {
                reveal(crate::atom::lexical_atom_views_spec);
            }
        }
        if after_white < end && atoms[after_white].code_point() == 0x0a {
            proof {
                reveal(single_quoted_step_spec);
                reveal(scalar_atom_white_spec);
                reveal(decoded_content_scalar_views_spec);
                assert(single_quoted_step_spec(views, index as int, end as int, remaining) == Ok(
                    (Seq::empty(), after_white as int),
                ));
            }
            let content = Vec::new();
            assert(content@.len() == 0);
            assert(decoded_content_scalar_views_spec(content@) == Seq::empty()) by {
                reveal(decoded_content_scalar_views_spec);
            }
            return Ok((content, after_white));
        }
        let item = direct_atom_content(
            atom,
            index,
            atom.code_point(),
            DecodedContentOrigin::Direct,
        );
        if remaining == 0 {
            let error = ScalarDecodeError::at(
                ScalarDecodeErrorKind::ContentLimitExceeded,
                item.byte_start(),
            );
            proof {
                reveal(single_quoted_step_spec);
                reveal(scalar_atom_white_spec);
                reveal(direct_atom_content_spec);
            }
            return Err(error);
        }
        let mut content = Vec::new();
        proof {
            lemma_decoded_content_views_push(content@, item);
        }
        content.push(item);
        proof {
            reveal(single_quoted_step_spec);
            reveal(scalar_atom_white_spec);
            reveal(direct_atom_content_spec);
            assert(item@ == direct_atom_content_spec(
                atom@,
                index as int,
                atom@.code_point,
                DecodedContentOrigin::Direct,
            ));
            assert(decoded_content_scalar_views_spec(content@) == Seq::empty().push(item@));
        }
        return Ok((content, index + 1));
    }
    if atom.code_point() == 0x0a {
        let (breaks, next) = single_quoted_break_group(atoms, index, end);
        assert((usize_indices_as_int_spec(breaks@), next as int) == single_quoted_break_group_spec(
            views,
            index as int,
            end as int,
            (end - index) as nat,
        ));
        let addition_count = if breaks.len() == 1 {
            1
        } else {
            breaks.len() - 1
        };
        if addition_count as u64 > remaining {
            let excluded_break = if breaks.len() == 1 {
                breaks[0]
            } else {
                breaks[remaining as usize + 1]
            };
            let error = ScalarDecodeError::at(
                ScalarDecodeErrorKind::ContentLimitExceeded,
                atoms[excluded_break].span().start().byte_offset(),
            );
            proof {
                reveal(single_quoted_step_spec);
                reveal(folded_break_content_spec);
                reveal(direct_atom_content_spec);
            }
            return Err(error);
        }
        let mut content = Vec::new();
        if breaks.len() == 1 {
            let item = direct_atom_content(
                &atoms[breaks[0]],
                breaks[0],
                0x20,
                DecodedContentOrigin::FoldedLineBreak,
            );
            proof {
                lemma_decoded_content_views_push(content@, item);
            }
            content.push(item);
        } else {
            let mut break_index = 1usize;
            while break_index < breaks.len()
                invariant
                    1 <= break_index <= breaks@.len(),
                    end <= atoms@.len(),
                    forall|position: int|
                        0 <= position < breaks@.len() ==> index <= #[trigger] breaks@[position]
                            < end,
                    views == crate::atom::lexical_atom_views_spec(atoms@),
                    decoded_content_scalar_views_spec(content@) == Seq::new(
                        (break_index - 1) as nat,
                        |offset: int|
                            direct_atom_content_spec(
                                crate::atom::lexical_atom_views_spec(atoms@)[breaks@[offset
                                    + 1] as int],
                                breaks@[offset + 1] as int,
                                0x0a,
                                DecodedContentOrigin::FoldedLineBreak,
                            ),
                    ),
                decreases breaks.len() - break_index,
            {
                let source_index = breaks[break_index];
                assert(0 <= break_index as int && (break_index as int) < breaks@.len());
                assert(index <= breaks@[break_index as int] < end);
                assert(source_index == breaks@[break_index as int]);
                assert(source_index < end);
                assert(end <= atoms@.len());
                assert(source_index < atoms@.len());
                assert(views[source_index as int] == atoms[source_index as int]@) by {
                    reveal(crate::atom::lexical_atom_views_spec);
                }
                let item = direct_atom_content(
                    &atoms[source_index],
                    source_index,
                    0x0a,
                    DecodedContentOrigin::FoldedLineBreak,
                );
                proof {
                    lemma_decoded_content_views_push(content@, item);
                }
                content.push(item);
                break_index += 1;
            }
        }
        proof {
            assert(decoded_content_scalar_views_spec(content@) == folded_break_content_spec(
                views,
                usize_indices_as_int_spec(breaks@),
            )) by {
                reveal(folded_break_content_spec);
                reveal(usize_indices_as_int_spec);
                reveal(decoded_content_scalar_views_spec);
                if breaks@.len() == 1 {
                    assert(usize_indices_as_int_spec(breaks@)[0] == breaks@[0] as int);
                } else {
                    assert(decoded_content_scalar_views_spec(content@) =~= Seq::new(
                        (usize_indices_as_int_spec(breaks@).len() - 1) as nat,
                        |offset: int|
                            direct_atom_content_spec(
                                views[usize_indices_as_int_spec(breaks@)[offset + 1]],
                                usize_indices_as_int_spec(breaks@)[offset + 1],
                                0x0a,
                                DecodedContentOrigin::FoldedLineBreak,
                            ),
                    ));
                }
            }
            reveal(single_quoted_step_spec);
            reveal(scalar_atom_white_spec);
            reveal(folded_break_content_spec);
            reveal(usize_indices_as_int_spec);
            reveal(decoded_content_scalar_views_spec);
        }
        return Ok((content, next));
    }
    if atom.code_point() == 0x27 {
        if index + 1 >= end || atoms[index + 1].code_point() != 0x27 {
            let error = ScalarDecodeError::at(
                ScalarDecodeErrorKind::InputQuotedMismatch,
                atom.span().start().byte_offset(),
            );
            proof {
                reveal(single_quoted_step_spec);
            }
            return Err(error);
        }
        assert(views[(index + 1) as int] == atoms[(index + 1) as int]@) by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        let item = DecodedContentScalar::new(
            0x27,
            index as u64,
            index as u64 + 2,
            atom.span().start().byte_offset(),
            atoms[index + 1].span().end().byte_offset(),
            DecodedContentOrigin::SingleQuoteDoubled,
        );
        if remaining == 0 {
            let error = ScalarDecodeError::at(
                ScalarDecodeErrorKind::ContentLimitExceeded,
                item.byte_start(),
            );
            proof {
                reveal(single_quoted_step_spec);
                reveal(doubled_quote_content_spec);
            }
            return Err(error);
        }
        let mut content = Vec::new();
        proof {
            lemma_decoded_content_views_push(content@, item);
        }
        content.push(item);
        proof {
            reveal(single_quoted_step_spec);
            reveal(doubled_quote_content_spec);
            assert(item@ == doubled_quote_content_spec(views, index as int));
            assert(decoded_content_scalar_views_spec(content@) == Seq::empty().push(item@));
        }
        return Ok((content, index + 2));
    }
    let item = direct_atom_content(atom, index, atom.code_point(), DecodedContentOrigin::Direct);
    if remaining == 0 {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ContentLimitExceeded,
            item.byte_start(),
        );
        proof {
            reveal(single_quoted_step_spec);
        }
        return Err(error);
    }
    let mut content = Vec::new();
    proof {
        lemma_decoded_content_views_push(content@, item);
    }
    content.push(item);
    proof {
        reveal(single_quoted_step_spec);
        reveal(scalar_atom_white_spec);
        reveal(direct_atom_content_spec);
        assert(!scalar_atom_white_spec(atom@));
        assert(atom@.code_point != 0x0a);
        assert(atom@.code_point != 0x27);
        assert(decoded_content_scalar_views_spec(content@) == Seq::empty().push(item@));
        assert(single_quoted_step_spec(views, index as int, end as int, remaining) == Ok(
            (decoded_content_scalar_views_spec(content@), (index + 1) as int),
        ));
    }
    Ok((content, index + 1))
}

fn append_decoded_content(
    output: &mut Vec<DecodedContentScalar>,
    additions: &[DecodedContentScalar],
)
    ensures
        decoded_content_scalar_views_spec(final(output)@) == decoded_content_scalar_views_spec(
            old(output)@,
        ) + decoded_content_scalar_views_spec(additions@),
{
    let ghost original = old(output)@;
    let ghost addition_views = decoded_content_scalar_views_spec(additions@);
    let mut index = 0usize;
    while index < additions.len()
        invariant
            index <= additions@.len(),
            addition_views == decoded_content_scalar_views_spec(additions@),
            decoded_content_scalar_views_spec(output@) == decoded_content_scalar_views_spec(
                original,
            ) + addition_views.subrange(0, index as int),
        decreases additions.len() - index,
    {
        let item = additions[index];
        proof {
            assert(addition_views[index as int] == item@) by {
                reveal(decoded_content_scalar_views_spec);
            }
            lemma_decoded_content_views_push(output@, item);
            reveal(decoded_content_scalar_views_spec);
            assert(addition_views.subrange(0, index as int).push(item@) =~= addition_views.subrange(
                0,
                index as int + 1,
            ));
        }
        output.push(item);
        index += 1;
    }
}

fn decode_single_quoted_content(
    atoms: &[LexicalAtom],
    quote: &QuotedScalar,
    limits: ScalarDecodeLimits,
) -> (result: Result<DecodedScalarContent, ScalarDecodeError>)
    ensures
        decode_single_quoted_content_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            quote@,
            limits@,
        ) == match result {
            Ok(content) => Ok(content@),
            Err(error) => Err(error@),
        },
{
    if quote.style() != QuotedScalarStyle::Single {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ScalarStyleMismatch,
            quote.byte_start(),
        );
        proof {
            reveal(decode_single_quoted_content_spec);
        }
        return Err(error);
    }
    let start_atom = quote.start_atom_index();
    let end_atom = quote.end_atom_index();
    let range_matches = start_atom <= end_atom && end_atom <= atoms.len() as u64 && end_atom
        - start_atom >= 2 && atoms[start_atom as usize].code_point() == 0x27 && atoms[(end_atom
        - 1) as usize].code_point() == 0x27 && quote.byte_start()
        == atoms[start_atom as usize].span().start().byte_offset() && quote.byte_end() == atoms[(
    end_atom - 1) as usize].span().end().byte_offset();
    if !range_matches {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::InputQuotedMismatch,
            quote.byte_start(),
        );
        proof {
            reveal(decode_single_quoted_content_spec);
            reveal(quoted_scalar_range_matches_atoms_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return Err(error);
    }
    proof {
        reveal(quoted_scalar_range_matches_atoms_spec);
        reveal(crate::atom::lexical_atom_views_spec);
        assert(quoted_scalar_range_matches_atoms_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            quote@,
        ));
    }
    let start = start_atom as usize + 1;
    let end = end_atom as usize - 1;
    assert(start <= end <= atoms.len());
    assert(end <= u64::MAX);
    let limit = if limits.max_content_code_points
        < MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS {
        limits.max_content_code_points
    } else {
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS
    };
    assert(limit == effective_scalar_content_limit_spec(limits@)) by {
        reveal(effective_scalar_content_limit_spec);
    }
    let ghost views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost expected = decode_single_quoted_loop_spec(
        views,
        start as int,
        end as int,
        limit,
        (end - start + 1) as nat,
    );
    proof {
        assert(start as int == quote@.start_atom_index as int + 1);
        assert(end as int == quote@.end_atom_index as int - 1);
        reveal(decode_single_quoted_content_spec);
        assert(decode_single_quoted_content_spec(views, quote@, limits@) == match expected {
            Ok(content) => Ok(
                DecodedScalarContentView { style: DecodedScalarStyle::SingleQuoted, content },
            ),
            Err(error) => Err(error),
        });
    }
    let mut output = Vec::new();
    let mut index = start;
    let mut remaining = limit;
    let mut _fuel = end - start + 1;
    while index < end
        invariant
            start <= index <= end <= atoms@.len(),
            end <= u64::MAX,
            views == crate::atom::lexical_atom_views_spec(atoms@),
            _fuel >= end - index,
            remaining + output@.len() == limit,
            decoded_content_scalar_views_spec(output@).len() == output@.len(),
            expected == prepend_decoded_content_result_spec(
                decoded_content_scalar_views_spec(output@),
                decode_single_quoted_loop_spec(
                    views,
                    index as int,
                    end as int,
                    remaining,
                    _fuel as nat,
                ),
            ),
            decode_single_quoted_content_spec(views, quote@, limits@) == match expected {
                Ok(content) => Ok(
                    DecodedScalarContentView { style: DecodedScalarStyle::SingleQuoted, content },
                ),
                Err(error) => Err(error),
            },
        decreases end - index,
    {
        let step = decoded_single_quoted_step(atoms, index, end, remaining);
        match step {
            Err(error) => {
                proof {
                    reveal(decode_single_quoted_loop_spec);
                    reveal(prepend_decoded_content_result_spec);
                    assert(expected == Err(error@));
                    assert(decode_single_quoted_content_spec(views, quote@, limits@) == Err(
                        error@,
                    ));
                }
                return Err(error);
            },
            Ok((additions, next)) => {
                let addition_count = additions.len();
                let ghost prior_output = output@;
                let ghost addition_views = decoded_content_scalar_views_spec(additions@);
                let ghost tail_result = decode_single_quoted_loop_spec(
                    views,
                    next as int,
                    end as int,
                    (remaining - addition_views.len()) as u64,
                    (_fuel - 1) as nat,
                );
                assert(addition_count <= remaining);
                assert(index < next <= end);
                proof {
                    reveal(decode_single_quoted_loop_spec);
                    reveal(prepend_decoded_content_result_spec);
                    lemma_prepend_decoded_content_associative(
                        decoded_content_scalar_views_spec(prior_output),
                        addition_views,
                        tail_result,
                    );
                }
                append_decoded_content(&mut output, additions.as_slice());
                remaining -= addition_count as u64;
                index = next;
                _fuel -= 1;
                proof {
                    assert(decoded_content_scalar_views_spec(output@)
                        == decoded_content_scalar_views_spec(prior_output) + addition_views);
                }
            },
        }
    }
    proof {
        reveal(decode_single_quoted_loop_spec);
        reveal(prepend_decoded_content_result_spec);
    }
    let decoded = DecodedScalarContent::new(DecodedScalarStyle::SingleQuoted, output);
    proof {
        reveal(decode_single_quoted_content_spec);
        reveal(effective_scalar_content_limit_spec);
        reveal(quoted_scalar_range_matches_atoms_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    Ok(decoded)
}

/// Decode one authenticated single-quoted scalar with exact source provenance.
pub fn decode_profile1_single_quoted_scalar_content(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    scalar_index: u64,
    limits: ScalarDecodeLimits,
) -> (result: Result<DecodedScalarContent, ScalarDecodeError>)
    ensures
        decode_profile1_single_quoted_scalar_content_spec(atomized@, quoted@, scalar_index, limits@)
            == match result {
            Ok(content) => Ok(content@),
            Err(error) => Err(error@),
        },
{
    let metadata_matches = quoted.profile_version() == atomized.profile_version()
        && quoted.input_transformation_version() == atomized.transformation_version()
        && quoted.source_len_bytes() == atomized.source_len_bytes() && quoted.bom_bytes()
        == atomized.bom_bytes() && quoted.input_atom_count() == atomized.atoms().len() as u64;
    if !metadata_matches {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::InputQuotedMismatch,
            atomized.bom_bytes(),
        );
        proof {
            reveal(decode_profile1_single_quoted_scalar_content_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return Err(error);
    }
    let scalars = quoted.scalars();
    if scalar_index >= scalars.len() as u64 {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ScalarIndexOutOfRange,
            atomized.source_len_bytes(),
        );
        proof {
            reveal(decode_profile1_single_quoted_scalar_content_spec);
            reveal(crate::quoted::quoted_scalar_views_spec);
        }
        return Err(error);
    }
    let index = scalar_index as usize;
    assert(quoted@.scalars[index as int] == scalars[index as int]@) by {
        reveal(crate::quoted::quoted_scalar_views_spec);
    }
    let result = decode_single_quoted_content(atomized.atoms(), &scalars[index], limits);
    proof {
        reveal(decode_profile1_single_quoted_scalar_content_spec);
    }
    result
}

proof fn lemma_decoded_content_views_push(
    content: Seq<DecodedContentScalar>,
    scalar: DecodedContentScalar,
)
    ensures
        decoded_content_scalar_views_spec(content.push(scalar))
            == decoded_content_scalar_views_spec(content).push(scalar@),
{
    reveal(decoded_content_scalar_views_spec);
    assert(decoded_content_scalar_views_spec(content.push(scalar))
        =~= decoded_content_scalar_views_spec(content).push(scalar@));
}

fn copy_block_content(source: &[BlockScalarContentScalar]) -> (decoded: Vec<DecodedContentScalar>)
    ensures
        decoded_content_scalar_views_spec(decoded@) == decoded_block_content_spec(
            crate::block::block_content_views_spec(source@),
        ),
{
    let ghost source_views = crate::block::block_content_views_spec(source@);
    let mut decoded = Vec::new();
    let mut index = 0usize;
    while index < source.len()
        invariant
            index <= source@.len(),
            source_views == crate::block::block_content_views_spec(source@),
            decoded_content_scalar_views_spec(decoded@) == decoded_block_content_prefix_spec(
                source_views,
                index as nat,
            ),
        decreases source.len() - index,
    {
        assert(source_views[index as int] == source[index as int]@) by {
            reveal(crate::block::block_content_views_spec);
        }
        let item = &source[index];
        let origin = match item.origin() {
            BlockScalarContentOrigin::Direct => DecodedContentOrigin::Direct,
            BlockScalarContentOrigin::FoldedLineBreak => DecodedContentOrigin::FoldedLineBreak,
        };
        let source_atom_start = item.source_atom_index();
        let source_atom_end = if source_atom_start == u64::MAX {
            u64::MAX
        } else {
            source_atom_start + 1
        };
        let copied = DecodedContentScalar::new(
            item.code_point(),
            source_atom_start,
            source_atom_end,
            item.byte_start(),
            item.byte_end(),
            origin,
        );
        proof {
            reveal(decoded_block_content_prefix_spec);
            reveal(decoded_block_content_item_spec);
            reveal(decoded_block_origin_spec);
            lemma_decoded_content_views_push(decoded@, copied);
        }
        decoded.push(copied);
        index += 1;
    }
    proof {
        reveal(decoded_block_content_spec);
    }
    decoded
}

/// Copy one authenticated block scalar's normalized content into the shared semantic form.
pub fn decode_profile1_block_scalar_content(
    source: &BlockScalarSource,
    scalar_index: u64,
    limits: ScalarDecodeLimits,
) -> (result: Result<DecodedScalarContent, ScalarDecodeError>)
    ensures
        decode_profile1_block_scalar_content_spec(source@, scalar_index, limits@) == match result {
            Ok(content) => Ok(content@),
            Err(error) => Err(error@),
        },
{
    let scalars = source.scalars();
    if scalar_index >= scalars.len() as u64 {
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ScalarIndexOutOfRange,
            source.source_len_bytes(),
        );
        proof {
            reveal(decode_profile1_block_scalar_content_spec);
            reveal(crate::block::block_scalar_views_spec);
        }
        return Err(error);
    }
    let index = scalar_index as usize;
    assert(index < scalars.len());
    assert(source@.scalars[index as int] == scalars[index as int]@) by {
        reveal(crate::block::block_scalar_views_spec);
    }
    let scalar = &scalars[index];
    let source_content = scalar.content();
    let effective_limit = if limits.max_content_code_points
        < MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS {
        limits.max_content_code_points
    } else {
        MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS
    };
    if source_content.len() as u64 > effective_limit {
        let excluded = &source_content[effective_limit as usize];
        let error = ScalarDecodeError::at(
            ScalarDecodeErrorKind::ContentLimitExceeded,
            excluded.byte_start(),
        );
        proof {
            reveal(decode_profile1_block_scalar_content_spec);
            reveal(decode_block_content_spec);
            reveal(effective_scalar_content_limit_spec);
            reveal(crate::block::block_content_views_spec);
        }
        return Err(error);
    }
    let decoded = copy_block_content(source_content);
    let style = match scalar.style() {
        BlockScalarStyle::Literal => DecodedScalarStyle::LiteralBlock,
        BlockScalarStyle::Folded => DecodedScalarStyle::FoldedBlock,
    };
    let result = DecodedScalarContent::new(style, decoded);
    proof {
        reveal(decode_profile1_block_scalar_content_spec);
        reveal(decode_block_content_spec);
        reveal(effective_scalar_content_limit_spec);
        reveal(decoded_block_style_spec);
    }
    Ok(result)
}

} // verus!
