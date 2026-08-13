//! Verified dispatch from authenticated CST scalar nodes to style-specific semantic decoders.
use crate::atom::AtomizedSource;
#[allow(unused_imports)]
use crate::atom::AtomizedSourceView;
use crate::block::BlockScalarSource;
#[allow(unused_imports)]
use crate::block::BlockScalarSourceView;
use crate::cst::{CstNode, CstNodeKind, CstNodeStyle, CstSource};
#[allow(unused_imports)]
use crate::cst::{CstNodeView, CstSourceView};
use crate::plain::PlainScalarSource;
#[allow(unused_imports)]
use crate::plain::PlainScalarSourceView;
use crate::quoted::QuotedScalarSource;
#[allow(unused_imports)]
use crate::quoted::QuotedScalarSourceView;
use crate::scalar_decode::{
    decode_profile1_block_scalar_content, decode_profile1_double_quoted_scalar_content,
    decode_profile1_plain_scalar_content, decode_profile1_single_quoted_scalar_content,
    DecodedScalarContent, ScalarDecodeError, ScalarDecodeErrorKind, ScalarDecodeLimits,
};
#[allow(unused_imports)]
use crate::scalar_decode::{
    DecodedScalarContentView, ScalarDecodeErrorView, ScalarDecodeLimitsView,
};
use crate::token::{CompletedTokenKind, CompletedTokenSource};
#[allow(unused_imports)]
use crate::token::{CompletedTokenSourceView, CompletedTokenView};
use vstd::prelude::*;

verus! {

pub const CST_SCALAR_DECODE_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstScalarDecodeLimits {
    max_content_code_points: u64,
}

#[verifier::ext_equal]
pub struct CstScalarDecodeLimitsView {
    pub max_content_code_points: u64,
}

impl View for CstScalarDecodeLimits {
    type V = CstScalarDecodeLimitsView;

    closed spec fn view(&self) -> CstScalarDecodeLimitsView {
        CstScalarDecodeLimitsView { max_content_code_points: self.max_content_code_points }
    }
}

impl CstScalarDecodeLimits {
    pub fn new(max_content_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (CstScalarDecodeLimitsView { max_content_code_points }),
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

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedCstScalar {
    node_index: u64,
    token_index: Option<u64>,
    style: CstNodeStyle,
    decoded: Option<DecodedScalarContent>,
}

#[verifier::ext_equal]
pub struct DecodedCstScalarView {
    pub node_index: u64,
    pub token_index: Option<u64>,
    pub style: CstNodeStyle,
    pub decoded: Option<DecodedScalarContentView>,
}

impl View for DecodedCstScalar {
    type V = DecodedCstScalarView;

    closed spec fn view(&self) -> DecodedCstScalarView {
        DecodedCstScalarView {
            node_index: self.node_index,
            token_index: self.token_index,
            style: self.style,
            decoded: match self.decoded {
                Some(ref content) => Some(content@),
                None => None,
            },
        }
    }
}

impl DecodedCstScalar {
    fn empty(node_index: u64) -> (scalar: Self)
        ensures
            scalar@ == (DecodedCstScalarView {
                node_index,
                token_index: None,
                style: CstNodeStyle::Empty,
                decoded: None,
            }),
    {
        Self { node_index, token_index: None, style: CstNodeStyle::Empty, decoded: None }
    }

    fn from_decoded(
        node_index: u64,
        token_index: u64,
        style: CstNodeStyle,
        decoded: DecodedScalarContent,
    ) -> (scalar: Self)
        ensures
            scalar@ == (DecodedCstScalarView {
                node_index,
                token_index: Some(token_index),
                style,
                decoded: Some(decoded@),
            }),
    {
        Self { node_index, token_index: Some(token_index), style, decoded: Some(decoded) }
    }

    pub fn node_index(&self) -> (index: u64)
        ensures
            index == self@.node_index,
    {
        self.node_index
    }

    pub fn token_index(&self) -> (index: Option<u64>)
        ensures
            index == self@.token_index,
    {
        self.token_index
    }

    pub fn style(&self) -> (style: CstNodeStyle)
        ensures
            style == self@.style,
    {
        self.style
    }

    pub fn decoded(&self) -> (decoded: Option<&DecodedScalarContent>)
        ensures
            match decoded {
                Some(content) => self@.decoded == Some(content@),
                None => self@.decoded.is_none(),
            },
    {
        self.decoded.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CstScalarDecodeErrorKind {
    InputCompletedTokenMismatch,
    InputCstMismatch,
    InputQuotedMismatch,
    InputPlainMismatch,
    InputBlockMismatch,
    NodeIndexOutOfRange,
    InvalidScalarToken,
    ContentLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CstScalarDecodeError {
    kind: CstScalarDecodeErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CstScalarDecodeErrorView {
    pub kind: CstScalarDecodeErrorKind,
    pub byte_offset: u64,
}

impl View for CstScalarDecodeError {
    type V = CstScalarDecodeErrorView;

    closed spec fn view(&self) -> CstScalarDecodeErrorView {
        CstScalarDecodeErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CstScalarDecodeError {
    fn at(kind: CstScalarDecodeErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (CstScalarDecodeErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: CstScalarDecodeErrorKind)
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

pub open spec fn scalar_decode_limits_spec(
    limits: CstScalarDecodeLimitsView,
) -> ScalarDecodeLimitsView {
    ScalarDecodeLimitsView { max_content_code_points: limits.max_content_code_points }
}

pub open spec fn map_scalar_decode_error_kind_spec(
    kind: ScalarDecodeErrorKind,
) -> CstScalarDecodeErrorKind {
    match kind {
        ScalarDecodeErrorKind::InputQuotedMismatch => CstScalarDecodeErrorKind::InputQuotedMismatch,
        ScalarDecodeErrorKind::InputPlainMismatch => CstScalarDecodeErrorKind::InputPlainMismatch,
        ScalarDecodeErrorKind::ContentLimitExceeded => CstScalarDecodeErrorKind::ContentLimitExceeded,
        ScalarDecodeErrorKind::ScalarIndexOutOfRange
        | ScalarDecodeErrorKind::ScalarStyleMismatch => CstScalarDecodeErrorKind::InvalidScalarToken,
    }
}

pub open spec fn wrap_scalar_decode_result_spec(
    result: Result<DecodedScalarContentView, ScalarDecodeErrorView>,
    node_index: u64,
    token_index: u64,
    style: CstNodeStyle,
) -> Result<Option<DecodedCstScalarView>, CstScalarDecodeErrorView> {
    match result {
        Ok(decoded) => Ok(
            Some(
                DecodedCstScalarView {
                    node_index,
                    token_index: Some(token_index),
                    style,
                    decoded: Some(decoded),
                },
            ),
        ),
        Err(error) => Err(
            CstScalarDecodeErrorView {
                kind: map_scalar_decode_error_kind_spec(error.kind),
                byte_offset: error.byte_offset,
            },
        ),
    }
}

pub open spec fn decode_cst_node_scalar_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    tokens: Seq<CompletedTokenView>,
    node: CstNodeView,
    node_index: u64,
    limits: CstScalarDecodeLimitsView,
) -> Result<Option<DecodedCstScalarView>, CstScalarDecodeErrorView> {
    if node.kind == CstNodeKind::Empty && node.style == CstNodeStyle::Empty {
        Ok(
            Some(
                DecodedCstScalarView {
                    node_index,
                    token_index: None,
                    style: CstNodeStyle::Empty,
                    decoded: None,
                },
            ),
        )
    } else if node.kind != CstNodeKind::Scalar {
        Ok(None)
    } else if node.scalar_or_alias_token.is_none() || node.scalar_or_alias_token.unwrap()
        >= tokens.len() {
        Err(
            CstScalarDecodeErrorView {
                kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                byte_offset: node.byte_start,
            },
        )
    } else {
        let token_index = node.scalar_or_alias_token.unwrap();
        let token = tokens[token_index as int];
        if token.scalar_index.is_none() {
            Err(
                CstScalarDecodeErrorView {
                    kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                    byte_offset: token.byte_start,
                },
            )
        } else {
            let scalar_index = token.scalar_index.unwrap();
            let decoder_limits = scalar_decode_limits_spec(limits);
            match node.style {
                CstNodeStyle::Plain => if token.kind != CompletedTokenKind::PlainScalar {
                    Err(
                        CstScalarDecodeErrorView {
                            kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                            byte_offset: token.byte_start,
                        },
                    )
                } else {
                    wrap_scalar_decode_result_spec(
                        crate::scalar_decode::decode_profile1_plain_scalar_content_spec(
                            atomized,
                            plain,
                            scalar_index,
                            decoder_limits,
                        ),
                        node_index,
                        token_index,
                        node.style,
                    )
                },
                CstNodeStyle::SingleQuoted => if token.kind
                    != CompletedTokenKind::SingleQuotedScalar {
                    Err(
                        CstScalarDecodeErrorView {
                            kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                            byte_offset: token.byte_start,
                        },
                    )
                } else {
                    wrap_scalar_decode_result_spec(
                        crate::scalar_decode::decode_profile1_single_quoted_scalar_content_spec(
                            atomized,
                            quoted,
                            scalar_index,
                            decoder_limits,
                        ),
                        node_index,
                        token_index,
                        node.style,
                    )
                },
                CstNodeStyle::DoubleQuoted => if token.kind
                    != CompletedTokenKind::DoubleQuotedScalar {
                    Err(
                        CstScalarDecodeErrorView {
                            kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                            byte_offset: token.byte_start,
                        },
                    )
                } else {
                    wrap_scalar_decode_result_spec(
                        crate::scalar_decode::decode_profile1_double_quoted_scalar_content_spec(
                            atomized,
                            quoted,
                            scalar_index,
                            decoder_limits,
                        ),
                        node_index,
                        token_index,
                        node.style,
                    )
                },
                CstNodeStyle::Literal | CstNodeStyle::Folded => if (node.style
                    == CstNodeStyle::Literal && token.kind
                    != CompletedTokenKind::LiteralBlockScalar) || (node.style
                    == CstNodeStyle::Folded && token.kind
                    != CompletedTokenKind::FoldedBlockScalar) {
                    Err(
                        CstScalarDecodeErrorView {
                            kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                            byte_offset: token.byte_start,
                        },
                    )
                } else if block.profile_version != atomized.profile_version
                    || block.input_transformation_version != atomized.transformation_version
                    || block.transformation_version
                    != crate::block::BLOCK_SCALAR_TRANSFORMATION_VERSION || block.source_len_bytes
                    != atomized.source_len_bytes || block.bom_bytes != atomized.bom_bytes
                    || block.input_atom_count != atomized.atoms.len() {
                    Err(
                        CstScalarDecodeErrorView {
                            kind: CstScalarDecodeErrorKind::InputBlockMismatch,
                            byte_offset: atomized.bom_bytes,
                        },
                    )
                } else {
                    wrap_scalar_decode_result_spec(
                        crate::scalar_decode::decode_profile1_block_scalar_content_spec(
                            block,
                            scalar_index,
                            decoder_limits,
                        ),
                        node_index,
                        token_index,
                        node.style,
                    )
                },
                _ => Err(
                    CstScalarDecodeErrorView {
                        kind: CstScalarDecodeErrorKind::InvalidScalarToken,
                        byte_offset: token.byte_start,
                    },
                ),
            }
        }
    }
}

pub open spec fn decode_profile1_cst_node_scalar_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    node_index: u64,
    limits: CstScalarDecodeLimitsView,
) -> Result<Option<DecodedCstScalarView>, CstScalarDecodeErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            CstScalarDecodeErrorView {
                kind: CstScalarDecodeErrorKind::InputCompletedTokenMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() {
        Err(
            CstScalarDecodeErrorView {
                kind: CstScalarDecodeErrorKind::InputCstMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if node_index >= cst.nodes.len() {
        Err(
            CstScalarDecodeErrorView {
                kind: CstScalarDecodeErrorKind::NodeIndexOutOfRange,
                byte_offset: atomized.source_len_bytes,
            },
        )
    } else {
        decode_cst_node_scalar_spec(
            atomized,
            quoted,
            plain,
            block,
            completed.tokens,
            cst.nodes[node_index as int],
            node_index,
            limits,
        )
    }
}

fn map_scalar_decode_error(error: ScalarDecodeError) -> (mapped: CstScalarDecodeError)
    ensures
        mapped@ == (CstScalarDecodeErrorView {
            kind: map_scalar_decode_error_kind_spec(error@.kind),
            byte_offset: error@.byte_offset,
        }),
{
    let kind = match error.kind() {
        ScalarDecodeErrorKind::InputQuotedMismatch => CstScalarDecodeErrorKind::InputQuotedMismatch,
        ScalarDecodeErrorKind::InputPlainMismatch => CstScalarDecodeErrorKind::InputPlainMismatch,
        ScalarDecodeErrorKind::ContentLimitExceeded => CstScalarDecodeErrorKind::ContentLimitExceeded,
        ScalarDecodeErrorKind::ScalarIndexOutOfRange
        | ScalarDecodeErrorKind::ScalarStyleMismatch => CstScalarDecodeErrorKind::InvalidScalarToken,
    };
    let mapped = CstScalarDecodeError::at(kind, error.byte_offset());
    proof {
        reveal(map_scalar_decode_error_kind_spec);
    }
    mapped
}

fn wrap_scalar_decode_result(
    result: Result<DecodedScalarContent, ScalarDecodeError>,
    node_index: u64,
    token_index: u64,
    style: CstNodeStyle,
) -> (wrapped: Result<Option<DecodedCstScalar>, CstScalarDecodeError>)
    ensures
        wrap_scalar_decode_result_spec(
            match result {
                Ok(ref decoded) => Ok(decoded@),
                Err(ref error) => Err(error@),
            },
            node_index,
            token_index,
            style,
        ) == match wrapped {
            Ok(Some(ref scalar)) => Ok(Some(scalar@)),
            Ok(None) => Ok(None),
            Err(ref error) => Err(error@),
        },
{
    match result {
        Ok(decoded) => Ok(
            Some(DecodedCstScalar::from_decoded(node_index, token_index, style, decoded)),
        ),
        Err(error) => Err(map_scalar_decode_error(error)),
    }
}

#[allow(clippy::too_many_arguments)]  // Mirrors the exact pure dispatch contract inputs.
fn decode_cst_node_scalar(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    tokens: &[crate::token::CompletedToken],
    node: &CstNode,
    node_index: u64,
    limits: CstScalarDecodeLimits,
) -> (result: Result<Option<DecodedCstScalar>, CstScalarDecodeError>)
    ensures
        decode_cst_node_scalar_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            crate::token::completed_token_views_spec(tokens@),
            node@,
            node_index,
            limits@,
        ) == match result {
            Ok(Some(ref scalar)) => Ok(Some(scalar@)),
            Ok(None) => Ok(None),
            Err(ref error) => Err(error@),
        },
{
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        assert(token_views.len() == tokens@.len());
    }
    let kind = node.kind();
    let style = node.style();
    if kind == CstNodeKind::Empty && style == CstNodeStyle::Empty {
        let scalar = DecodedCstScalar::empty(node_index);
        proof {
            reveal(decode_cst_node_scalar_spec);
        }
        return Ok(Some(scalar));
    }
    if kind != CstNodeKind::Scalar {
        proof {
            reveal(decode_cst_node_scalar_spec);
        }
        return Ok(None);
    }
    let token_index = match node.scalar_or_alias_token() {
        Some(index) => index,
        None => {
            let error = CstScalarDecodeError::at(
                CstScalarDecodeErrorKind::InvalidScalarToken,
                node.byte_start(),
            );
            proof {
                reveal(decode_cst_node_scalar_spec);
            }
            return Err(error);
        },
    };
    if token_index >= tokens.len() as u64 {
        let error = CstScalarDecodeError::at(
            CstScalarDecodeErrorKind::InvalidScalarToken,
            node.byte_start(),
        );
        proof {
            reveal(decode_cst_node_scalar_spec);
            crate::token::lemma_completed_token_views_len(tokens@);
        }
        return Err(error);
    }
    let token_offset = token_index as usize;
    let token = &tokens[token_offset];
    let token_kind = token.kind();
    let token_byte_start = token.byte_start();
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, token_offset as int);
        assert(token_views[token_offset as int] == token@);
    }
    let scalar_index = match token.scalar_index() {
        Some(index) => index,
        None => {
            let error = CstScalarDecodeError::at(
                CstScalarDecodeErrorKind::InvalidScalarToken,
                token_byte_start,
            );
            proof {
                reveal(decode_cst_node_scalar_spec);
            }
            return Err(error);
        },
    };
    let decoder_limits = ScalarDecodeLimits::new(limits.max_content_code_points());
    proof {
        reveal(scalar_decode_limits_spec);
        assert(decoder_limits@ == scalar_decode_limits_spec(limits@));
    }
    let decoded = match style {
        CstNodeStyle::Plain => {
            if token_kind != CompletedTokenKind::PlainScalar {
                let error = CstScalarDecodeError::at(
                    CstScalarDecodeErrorKind::InvalidScalarToken,
                    token_byte_start,
                );
                proof {
                    reveal(decode_cst_node_scalar_spec);
                }
                return Err(error);
            }
            decode_profile1_plain_scalar_content(atomized, plain, scalar_index, decoder_limits)
        },
        CstNodeStyle::SingleQuoted => {
            if token_kind != CompletedTokenKind::SingleQuotedScalar {
                let error = CstScalarDecodeError::at(
                    CstScalarDecodeErrorKind::InvalidScalarToken,
                    token_byte_start,
                );
                proof {
                    reveal(decode_cst_node_scalar_spec);
                }
                return Err(error);
            }
            decode_profile1_single_quoted_scalar_content(
                atomized,
                quoted,
                scalar_index,
                decoder_limits,
            )
        },
        CstNodeStyle::DoubleQuoted => {
            if token_kind != CompletedTokenKind::DoubleQuotedScalar {
                let error = CstScalarDecodeError::at(
                    CstScalarDecodeErrorKind::InvalidScalarToken,
                    token_byte_start,
                );
                proof {
                    reveal(decode_cst_node_scalar_spec);
                }
                return Err(error);
            }
            decode_profile1_double_quoted_scalar_content(
                atomized,
                quoted,
                scalar_index,
                decoder_limits,
            )
        },
        CstNodeStyle::Literal | CstNodeStyle::Folded => {
            if (style == CstNodeStyle::Literal && token_kind
                != CompletedTokenKind::LiteralBlockScalar) || (style == CstNodeStyle::Folded
                && token_kind != CompletedTokenKind::FoldedBlockScalar) {
                let error = CstScalarDecodeError::at(
                    CstScalarDecodeErrorKind::InvalidScalarToken,
                    token_byte_start,
                );
                proof {
                    reveal(decode_cst_node_scalar_spec);
                }
                return Err(error);
            }
            if block.profile_version() != atomized.profile_version()
                || block.input_transformation_version() != atomized.transformation_version()
                || block.transformation_version()
                != crate::block::BLOCK_SCALAR_TRANSFORMATION_VERSION || block.source_len_bytes()
                != atomized.source_len_bytes() || block.bom_bytes() != atomized.bom_bytes()
                || block.input_atom_count() != atomized.atoms().len() as u64 {
                let error = CstScalarDecodeError::at(
                    CstScalarDecodeErrorKind::InputBlockMismatch,
                    atomized.bom_bytes(),
                );
                proof {
                    reveal(decode_cst_node_scalar_spec);
                    reveal(crate::atom::lexical_atom_views_spec);
                }
                return Err(error);
            }
            decode_profile1_block_scalar_content(block, scalar_index, decoder_limits)
        },
        _ => {
            let error = CstScalarDecodeError::at(
                CstScalarDecodeErrorKind::InvalidScalarToken,
                token_byte_start,
            );
            proof {
                reveal(decode_cst_node_scalar_spec);
            }
            return Err(error);
        },
    };
    let result = wrap_scalar_decode_result(decoded, node_index, token_index, style);
    proof {
        reveal(decode_cst_node_scalar_spec);
        reveal(scalar_decode_limits_spec);
    }
    result
}

#[allow(clippy::too_many_arguments)]  // Every independently authenticated producer is explicit.
pub fn decode_profile1_cst_node_scalar(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    node_index: u64,
    limits: CstScalarDecodeLimits,
) -> (result: Result<Option<DecodedCstScalar>, CstScalarDecodeError>)
    ensures
        decode_profile1_cst_node_scalar_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            node_index,
            limits@,
        ) == match result {
            Ok(Some(ref scalar)) => Ok(Some(scalar@)),
            Ok(None) => Ok(None),
            Err(ref error) => Err(error@),
        },
{
    let bom_bytes = atomized.bom_bytes();
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let nodes = cst.nodes();
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(crate::cst::cst_node_views_spec);
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens == crate::token::completed_token_views_spec(tokens@));
        assert(completed@.tokens.len() == tokens@.len());
        assert(cst@.nodes == crate::cst::cst_node_views_spec(nodes@));
        assert(cst@.nodes.len() == nodes@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != bom_bytes
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = CstScalarDecodeError::at(
            CstScalarDecodeErrorKind::InputCompletedTokenMismatch,
            bom_bytes,
        );
        proof {
            reveal(decode_profile1_cst_node_scalar_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 {
        let error = CstScalarDecodeError::at(CstScalarDecodeErrorKind::InputCstMismatch, bom_bytes);
        proof {
            reveal(decode_profile1_cst_node_scalar_spec);
        }
        return Err(error);
    }
    if node_index >= nodes.len() as u64 {
        let error = CstScalarDecodeError::at(
            CstScalarDecodeErrorKind::NodeIndexOutOfRange,
            atomized.source_len_bytes(),
        );
        proof {
            reveal(decode_profile1_cst_node_scalar_spec);
        }
        return Err(error);
    }
    let runtime_index = node_index as usize;
    let node = &nodes[runtime_index];
    proof {
        crate::cst::lemma_cst_node_view_at(nodes@, runtime_index as int);
        assert(cst@.nodes[node_index as int] == node@);
    }
    let result = decode_cst_node_scalar(
        atomized,
        quoted,
        plain,
        block,
        tokens,
        node,
        node_index,
        limits,
    );
    proof {
        reveal(decode_profile1_cst_node_scalar_spec);
    }
    result
}

} // verus!
