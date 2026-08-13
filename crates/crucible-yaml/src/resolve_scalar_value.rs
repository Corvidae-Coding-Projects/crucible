//! Verified YAML 1.2.2 Core semantic resolution for authenticated CST scalar nodes.
use crate::atom::AtomizedSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::atom::AtomizedSourceView;
use crate::block::BlockScalarSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::block::BlockScalarSourceView;
use crate::cst::{CstNode, CstNodeKind, CstNodeStyle, CstSource};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::cst::{CstNodeView, CstSourceView};
use crate::plain::PlainScalarSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::plain::PlainScalarSourceView;
use crate::quoted::QuotedScalarSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::quoted::QuotedScalarSourceView;
use crate::resolve::{
    classify_core_plain_scalar, CorePlainScalarClass, CoreScalarErrorKind, CoreScalarLimits,
};
use crate::resolve_float::{
    convert_core_finite_float, CoreFiniteFloat, CoreFiniteFloatErrorKind, CoreFiniteFloatLimits,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_float::{CoreFiniteFloatLimitsView, CoreFiniteFloatView};
use crate::resolve_integer::{
    convert_core_integer, CoreInteger, CoreIntegerErrorKind, CoreIntegerLimits,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_integer::{CoreIntegerLimitsView, CoreIntegerView};
use crate::resolve_scalar_node::{
    decode_profile1_cst_node_scalar, CstScalarDecodeError, CstScalarDecodeErrorKind,
    CstScalarDecodeLimits, DecodedCstScalar,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_scalar_node::{
    CstScalarDecodeErrorView, CstScalarDecodeLimitsView, DecodedCstScalarView,
};
use crate::resolve_tag::{
    resolve_profile1_node_tag_property, ResolvedTagKind, ResolvedTagProperty, TagResolutionError,
    TagResolutionErrorKind, TagResolutionLimits,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::resolve_tag::{
    ResolvedTagCodePointView, ResolvedTagPropertyView, TagResolutionErrorView,
    TagResolutionLimitsView,
};
use crate::scalar_decode::{DecodedContentScalar, DecodedScalarStyle};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::scalar_decode::{DecodedContentScalarView, DecodedScalarContentView};
use crate::token::CompletedTokenSource;
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const SCALAR_VALUE_RESOLUTION_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarValueLimits {
    max_content_code_points: u64,
    max_tag_code_points: u64,
    max_integer_limbs: u64,
    max_float_coefficient_digits: u64,
    max_float_exponent_digits: u64,
}

#[verifier::ext_equal]
pub struct ScalarValueLimitsView {
    pub max_content_code_points: u64,
    pub max_tag_code_points: u64,
    pub max_integer_limbs: u64,
    pub max_float_coefficient_digits: u64,
    pub max_float_exponent_digits: u64,
}

impl View for ScalarValueLimits {
    type V = ScalarValueLimitsView;

    closed spec fn view(&self) -> ScalarValueLimitsView {
        ScalarValueLimitsView {
            max_content_code_points: self.max_content_code_points,
            max_tag_code_points: self.max_tag_code_points,
            max_integer_limbs: self.max_integer_limbs,
            max_float_coefficient_digits: self.max_float_coefficient_digits,
            max_float_exponent_digits: self.max_float_exponent_digits,
        }
    }
}

impl ScalarValueLimits {
    pub fn new(
        max_content_code_points: u64,
        max_tag_code_points: u64,
        max_integer_limbs: u64,
        max_float_coefficient_digits: u64,
        max_float_exponent_digits: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (ScalarValueLimitsView {
                max_content_code_points,
                max_tag_code_points,
                max_integer_limbs,
                max_float_coefficient_digits,
                max_float_exponent_digits,
            }),
    {
        Self {
            max_content_code_points,
            max_tag_code_points,
            max_integer_limbs,
            max_float_coefficient_digits,
            max_float_exponent_digits,
        }
    }

    pub fn max_content_code_points(&self) -> (value: u64)
        ensures
            value == self@.max_content_code_points,
    {
        self.max_content_code_points
    }

    pub fn max_tag_code_points(&self) -> (value: u64)
        ensures
            value == self@.max_tag_code_points,
    {
        self.max_tag_code_points
    }

    pub fn max_integer_limbs(&self) -> (value: u64)
        ensures
            value == self@.max_integer_limbs,
    {
        self.max_integer_limbs
    }

    pub fn max_float_coefficient_digits(&self) -> (value: u64)
        ensures
            value == self@.max_float_coefficient_digits,
    {
        self.max_float_coefficient_digits
    }

    pub fn max_float_exponent_digits(&self) -> (value: u64)
        ensures
            value == self@.max_float_exponent_digits,
    {
        self.max_float_exponent_digits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum ResolvedScalarTag {
    CoreNull,
    CoreBoolean,
    CoreInteger,
    CoreFloat,
    CoreString,
    CustomGlobal,
    CustomLocal,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolvedScalarValue {
    Null,
    Boolean(bool),
    Integer(CoreInteger),
    FiniteFloat(CoreFiniteFloat),
    PositiveInfinity,
    NegativeInfinity,
    NotANumber,
    String,
}

#[verifier::ext_equal]
pub enum ResolvedScalarValueView {
    Null,
    Boolean(bool),
    Integer(CoreIntegerView),
    FiniteFloat(CoreFiniteFloatView),
    PositiveInfinity,
    NegativeInfinity,
    NotANumber,
    String,
}

impl View for ResolvedScalarValue {
    type V = ResolvedScalarValueView;

    open spec fn view(&self) -> ResolvedScalarValueView {
        match self {
            ResolvedScalarValue::Null => ResolvedScalarValueView::Null,
            ResolvedScalarValue::Boolean(value) => ResolvedScalarValueView::Boolean(*value),
            ResolvedScalarValue::Integer(value) => ResolvedScalarValueView::Integer(value@),
            ResolvedScalarValue::FiniteFloat(value) => {
                ResolvedScalarValueView::FiniteFloat(value@)
            },
            ResolvedScalarValue::PositiveInfinity => ResolvedScalarValueView::PositiveInfinity,
            ResolvedScalarValue::NegativeInfinity => ResolvedScalarValueView::NegativeInfinity,
            ResolvedScalarValue::NotANumber => ResolvedScalarValueView::NotANumber,
            ResolvedScalarValue::String => ResolvedScalarValueView::String,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedScalar {
    node_index: u64,
    tag: ResolvedScalarTag,
    explicit_tag: Option<ResolvedTagProperty>,
    presentation: DecodedCstScalar,
    value: ResolvedScalarValue,
}

#[verifier::ext_equal]
pub struct ResolvedScalarView {
    pub node_index: u64,
    pub tag: ResolvedScalarTag,
    pub explicit_tag: Option<ResolvedTagPropertyView>,
    pub presentation: DecodedCstScalarView,
    pub value: ResolvedScalarValueView,
}

impl View for ResolvedScalar {
    type V = ResolvedScalarView;

    closed spec fn view(&self) -> ResolvedScalarView {
        ResolvedScalarView {
            node_index: self.node_index,
            tag: self.tag,
            explicit_tag: match self.explicit_tag {
                Some(ref tag) => Some(tag@),
                None => None,
            },
            presentation: self.presentation@,
            value: self.value@,
        }
    }
}

impl ResolvedScalar {
    fn new(
        node_index: u64,
        tag: ResolvedScalarTag,
        explicit_tag: Option<ResolvedTagProperty>,
        presentation: DecodedCstScalar,
        value: ResolvedScalarValue,
    ) -> (resolved: Self)
        ensures
            resolved@ == (ResolvedScalarView {
                node_index,
                tag,
                explicit_tag: match explicit_tag {
                    Some(ref property) => Some(property@),
                    None => None,
                },
                presentation: presentation@,
                value: value@,
            }),
    {
        Self { node_index, tag, explicit_tag, presentation, value }
    }

    pub fn node_index(&self) -> (index: u64)
        ensures
            index == self@.node_index,
    {
        self.node_index
    }

    pub fn tag(&self) -> (tag: ResolvedScalarTag)
        ensures
            tag == self@.tag,
    {
        self.tag
    }

    pub fn explicit_tag(&self) -> (tag: Option<&ResolvedTagProperty>)
        ensures
            match tag {
                Some(value) => self@.explicit_tag == Some(value@),
                None => self@.explicit_tag.is_none(),
            },
    {
        self.explicit_tag.as_ref()
    }

    pub fn presentation(&self) -> (presentation: &DecodedCstScalar)
        ensures
            presentation@ == self@.presentation,
    {
        &self.presentation
    }

    pub fn value(&self) -> (value: &ResolvedScalarValue)
        ensures
            value@ == self@.value,
    {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum ScalarValueErrorKind {
    ScalarDecode(CstScalarDecodeErrorKind),
    TagResolution(TagResolutionErrorKind),
    InvalidScalarPresentation,
    InvalidExplicitScalarTagValue,
    ScalarTagKindMismatch,
    ScalarClassificationLimitExceeded,
    IntegerMagnitudeLimitExceeded,
    FloatCoefficientLimitExceeded,
    FloatExponentLimitExceeded,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarValueError {
    kind: ScalarValueErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct ScalarValueErrorView {
    pub kind: ScalarValueErrorKind,
    pub byte_offset: u64,
}

impl View for ScalarValueError {
    type V = ScalarValueErrorView;

    closed spec fn view(&self) -> ScalarValueErrorView {
        ScalarValueErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl ScalarValueError {
    fn at(kind: ScalarValueErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (ScalarValueErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: ScalarValueErrorKind)
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
pub enum ExplicitScalarTagClass {
    NonSpecific,
    CoreNull,
    CoreBoolean,
    CoreInteger,
    CoreFloat,
    CoreString,
    CoreSequence,
    CoreMapping,
    CustomGlobal,
    CustomLocal,
}

pub open spec fn decoded_content_code_points_spec(content: Seq<DecodedContentScalarView>) -> Seq<
    u32,
> {
    Seq::new(content.len(), |index: int| content[index].code_point)
}

pub open spec fn decoded_scalar_code_points_spec(decoded: DecodedCstScalarView) -> Seq<u32> {
    match decoded.decoded {
        Some(content) => decoded_content_code_points_spec(content.content),
        None => Seq::empty(),
    }
}

pub open spec fn scalar_style_matches_spec(
    cst_style: CstNodeStyle,
    decoded_style: DecodedScalarStyle,
) -> bool {
    match (cst_style, decoded_style) {
        (CstNodeStyle::Plain, DecodedScalarStyle::Plain)
        | (CstNodeStyle::SingleQuoted, DecodedScalarStyle::SingleQuoted)
        | (CstNodeStyle::DoubleQuoted, DecodedScalarStyle::DoubleQuoted)
        | (CstNodeStyle::Literal, DecodedScalarStyle::LiteralBlock)
        | (CstNodeStyle::Folded, DecodedScalarStyle::FoldedBlock) => true,
        _ => false,
    }
}

pub open spec fn decoded_scalar_matches_node_spec(
    decoded: DecodedCstScalarView,
    node: CstNodeView,
    node_index: u64,
) -> bool {
    decoded.node_index == node_index && if node.kind == CstNodeKind::Empty && node.style
        == CstNodeStyle::Empty {
        decoded.style == CstNodeStyle::Empty && decoded.token_index.is_none()
            && decoded.decoded.is_none()
    } else {
        node.kind == CstNodeKind::Scalar && decoded.style == node.style && decoded.token_index
            == node.scalar_or_alias_token && decoded.decoded.is_some() && scalar_style_matches_spec(
            node.style,
            decoded.decoded.unwrap().style,
        )
    }
}

pub open spec fn tag_matches_node_spec(
    tag: Option<ResolvedTagPropertyView>,
    node: CstNodeView,
) -> bool {
    match (tag, node.tag_property_token) {
        (None, None) => true,
        (Some(property), Some(token_index)) => property.token_index == token_index,
        _ => false,
    }
}

pub open spec fn tag_content_matches_ascii_spec(
    content: Seq<ResolvedTagCodePointView>,
    expected: Seq<u8>,
) -> bool {
    content.len() == expected.len() && forall|index: int|
        0 <= index < content.len() ==> content[index].code_point == expected[index] as u32
}

pub open spec fn explicit_scalar_tag_class_spec(
    tag: ResolvedTagPropertyView,
) -> ExplicitScalarTagClass {
    if tag.kind == ResolvedTagKind::NonSpecific {
        ExplicitScalarTagClass::NonSpecific
    } else if tag.kind == ResolvedTagKind::Local {
        ExplicitScalarTagClass::CustomLocal
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:null"@) {
        ExplicitScalarTagClass::CoreNull
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:bool"@) {
        ExplicitScalarTagClass::CoreBoolean
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:int"@) {
        ExplicitScalarTagClass::CoreInteger
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:float"@) {
        ExplicitScalarTagClass::CoreFloat
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:str"@) {
        ExplicitScalarTagClass::CoreString
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:seq"@) {
        ExplicitScalarTagClass::CoreSequence
    } else if tag_content_matches_ascii_spec(tag.content, b"tag:yaml.org,2002:map"@) {
        ExplicitScalarTagClass::CoreMapping
    } else {
        ExplicitScalarTagClass::CustomGlobal
    }
}

pub open spec fn explicit_scalar_tag_compatible_spec(
    tag_class: ExplicitScalarTagClass,
    scalar_class: CorePlainScalarClass,
) -> bool {
    match tag_class {
        ExplicitScalarTagClass::CoreNull => { matches!(scalar_class, CorePlainScalarClass::Null) },
        ExplicitScalarTagClass::CoreBoolean => {
            matches!(scalar_class, CorePlainScalarClass::Boolean(_))
        },
        ExplicitScalarTagClass::CoreInteger => {
            matches!(scalar_class, CorePlainScalarClass::Integer { .. })
        },
        ExplicitScalarTagClass::CoreFloat => matches!(
            scalar_class,
            CorePlainScalarClass::FiniteFloat { .. }
                | CorePlainScalarClass::Infinity { .. }
                | CorePlainScalarClass::NotANumber
        ),
        _ => false,
    }
}

pub open spec fn scalar_value_anchor_spec(
    decoded: DecodedCstScalarView,
    node: CstNodeView,
    code_point_index: u64,
) -> u64 {
    match decoded.decoded {
        Some(content) => if code_point_index < content.content.len() {
            content.content[code_point_index as int].byte_start
        } else {
            node.byte_start
        },
        None => node.byte_start,
    }
}

pub open spec fn explicit_tag_anchor_spec(tag: ResolvedTagPropertyView, node: CstNodeView) -> u64 {
    if tag.content.len() > 0 {
        tag.content[0].byte_start
    } else {
        node.byte_start
    }
}

pub open spec fn resolved_scalar_view_spec(
    presentation: DecodedCstScalarView,
    tag: ResolvedScalarTag,
    explicit_tag: Option<ResolvedTagPropertyView>,
    value: ResolvedScalarValueView,
) -> ResolvedScalarView {
    ResolvedScalarView {
        node_index: presentation.node_index,
        tag,
        explicit_tag,
        presentation,
        value,
    }
}

pub open spec fn integer_error_kind_spec(kind: CoreIntegerErrorKind) -> ScalarValueErrorKind {
    match kind {
        CoreIntegerErrorKind::InputLimitExceeded => {
            ScalarValueErrorKind::ScalarClassificationLimitExceeded
        },
        CoreIntegerErrorKind::MagnitudeLimitExceeded => {
            ScalarValueErrorKind::IntegerMagnitudeLimitExceeded
        },
        CoreIntegerErrorKind::NotInteger | CoreIntegerErrorKind::FuelExhausted => {
            ScalarValueErrorKind::InternalInvariantViolation
        },
    }
}

pub open spec fn finite_float_error_kind_spec(
    kind: CoreFiniteFloatErrorKind,
) -> ScalarValueErrorKind {
    match kind {
        CoreFiniteFloatErrorKind::InputLimitExceeded => {
            ScalarValueErrorKind::ScalarClassificationLimitExceeded
        },
        CoreFiniteFloatErrorKind::CoefficientLimitExceeded => {
            ScalarValueErrorKind::FloatCoefficientLimitExceeded
        },
        CoreFiniteFloatErrorKind::ExponentLimitExceeded => {
            ScalarValueErrorKind::FloatExponentLimitExceeded
        },
        CoreFiniteFloatErrorKind::NotFiniteFloat => {
            ScalarValueErrorKind::InternalInvariantViolation
        },
    }
}

pub open spec fn resolve_classified_scalar_spec(
    input: Seq<u32>,
    class: CorePlainScalarClass,
    presentation: DecodedCstScalarView,
    explicit_tag: Option<ResolvedTagPropertyView>,
    node: CstNodeView,
    limits: ScalarValueLimitsView,
) -> Result<ResolvedScalarView, ScalarValueErrorView> {
    match class {
        CorePlainScalarClass::Null => Ok(
            resolved_scalar_view_spec(
                presentation,
                ResolvedScalarTag::CoreNull,
                explicit_tag,
                ResolvedScalarValueView::Null,
            ),
        ),
        CorePlainScalarClass::Boolean(value) => Ok(
            resolved_scalar_view_spec(
                presentation,
                ResolvedScalarTag::CoreBoolean,
                explicit_tag,
                ResolvedScalarValueView::Boolean(value),
            ),
        ),
        CorePlainScalarClass::Integer {
            ..
        } => match crate::resolve_integer::convert_core_integer_spec(
            input,
            CoreIntegerLimitsView {
                max_code_points: limits.max_content_code_points,
                max_limbs: limits.max_integer_limbs,
            },
        ) {
            Ok(integer) => Ok(
                resolved_scalar_view_spec(
                    presentation,
                    ResolvedScalarTag::CoreInteger,
                    explicit_tag,
                    ResolvedScalarValueView::Integer(integer),
                ),
            ),
            Err(error) => Err(
                ScalarValueErrorView {
                    kind: integer_error_kind_spec(error.kind),
                    byte_offset: scalar_value_anchor_spec(
                        presentation,
                        node,
                        error.code_point_index,
                    ),
                },
            ),
        },
        CorePlainScalarClass::FiniteFloat {
            ..
        } => match crate::resolve_float::convert_core_finite_float_spec(
            input,
            CoreFiniteFloatLimitsView {
                max_code_points: limits.max_content_code_points,
                max_coefficient_digits: limits.max_float_coefficient_digits,
                max_exponent_digits: limits.max_float_exponent_digits,
            },
        ) {
            Ok(value) => Ok(
                resolved_scalar_view_spec(
                    presentation,
                    ResolvedScalarTag::CoreFloat,
                    explicit_tag,
                    ResolvedScalarValueView::FiniteFloat(value),
                ),
            ),
            Err(error) => Err(
                ScalarValueErrorView {
                    kind: finite_float_error_kind_spec(error.kind),
                    byte_offset: scalar_value_anchor_spec(
                        presentation,
                        node,
                        error.code_point_index,
                    ),
                },
            ),
        },
        CorePlainScalarClass::Infinity { negative } => Ok(
            resolved_scalar_view_spec(
                presentation,
                ResolvedScalarTag::CoreFloat,
                explicit_tag,
                if negative {
                    ResolvedScalarValueView::NegativeInfinity
                } else {
                    ResolvedScalarValueView::PositiveInfinity
                },
            ),
        ),
        CorePlainScalarClass::NotANumber => Ok(
            resolved_scalar_view_spec(
                presentation,
                ResolvedScalarTag::CoreFloat,
                explicit_tag,
                ResolvedScalarValueView::NotANumber,
            ),
        ),
        CorePlainScalarClass::String => Ok(
            resolved_scalar_view_spec(
                presentation,
                ResolvedScalarTag::CoreString,
                explicit_tag,
                ResolvedScalarValueView::String,
            ),
        ),
    }
}

pub open spec fn resolve_decoded_scalar_value_spec(
    presentation: DecodedCstScalarView,
    explicit_tag: Option<ResolvedTagPropertyView>,
    node: CstNodeView,
    node_index: u64,
    limits: ScalarValueLimitsView,
) -> Result<ResolvedScalarView, ScalarValueErrorView> {
    if !decoded_scalar_matches_node_spec(presentation, node, node_index) || !tag_matches_node_spec(
        explicit_tag,
        node,
    ) {
        Err(
            ScalarValueErrorView {
                kind: ScalarValueErrorKind::InvalidScalarPresentation,
                byte_offset: node.byte_start,
            },
        )
    } else {
        let input = decoded_scalar_code_points_spec(presentation);
        match explicit_tag {
            Some(tag) => {
                let tag_class = explicit_scalar_tag_class_spec(tag);
                match tag_class {
                    ExplicitScalarTagClass::NonSpecific | ExplicitScalarTagClass::CoreString => Ok(
                        resolved_scalar_view_spec(
                            presentation,
                            ResolvedScalarTag::CoreString,
                            Some(tag),
                            ResolvedScalarValueView::String,
                        ),
                    ),
                    ExplicitScalarTagClass::CustomGlobal => Ok(
                        resolved_scalar_view_spec(
                            presentation,
                            ResolvedScalarTag::CustomGlobal,
                            Some(tag),
                            ResolvedScalarValueView::String,
                        ),
                    ),
                    ExplicitScalarTagClass::CustomLocal => Ok(
                        resolved_scalar_view_spec(
                            presentation,
                            ResolvedScalarTag::CustomLocal,
                            Some(tag),
                            ResolvedScalarValueView::String,
                        ),
                    ),
                    ExplicitScalarTagClass::CoreSequence
                    | ExplicitScalarTagClass::CoreMapping => Err(
                        ScalarValueErrorView {
                            kind: ScalarValueErrorKind::ScalarTagKindMismatch,
                            byte_offset: explicit_tag_anchor_spec(tag, node),
                        },
                    ),
                    _ => match crate::resolve::classify_core_plain_scalar_spec(
                        input,
                        crate::resolve::CoreScalarLimitsView {
                            max_code_points: limits.max_content_code_points,
                        },
                    ) {
                        Err(error) => Err(
                            ScalarValueErrorView {
                                kind: ScalarValueErrorKind::ScalarClassificationLimitExceeded,
                                byte_offset: scalar_value_anchor_spec(
                                    presentation,
                                    node,
                                    error.code_point_index,
                                ),
                            },
                        ),
                        Ok(class) => {
                            let compatible = explicit_scalar_tag_compatible_spec(tag_class, class);
                            if compatible {
                                resolve_classified_scalar_spec(
                                    input,
                                    class,
                                    presentation,
                                    Some(tag),
                                    node,
                                    limits,
                                )
                            } else {
                                Err(
                                    ScalarValueErrorView {
                                        kind: ScalarValueErrorKind::InvalidExplicitScalarTagValue,
                                        byte_offset: scalar_value_anchor_spec(
                                            presentation,
                                            node,
                                            0,
                                        ),
                                    },
                                )
                            }
                        },
                    },
                }
            },
            None => if node.kind == CstNodeKind::Empty {
                Ok(
                    resolved_scalar_view_spec(
                        presentation,
                        ResolvedScalarTag::CoreNull,
                        None,
                        ResolvedScalarValueView::Null,
                    ),
                )
            } else if node.style == CstNodeStyle::Plain {
                match crate::resolve::classify_core_plain_scalar_spec(
                    input,
                    crate::resolve::CoreScalarLimitsView {
                        max_code_points: limits.max_content_code_points,
                    },
                ) {
                    Err(error) => Err(
                        ScalarValueErrorView {
                            kind: ScalarValueErrorKind::ScalarClassificationLimitExceeded,
                            byte_offset: scalar_value_anchor_spec(
                                presentation,
                                node,
                                error.code_point_index,
                            ),
                        },
                    ),
                    Ok(class) => resolve_classified_scalar_spec(
                        input,
                        class,
                        presentation,
                        None,
                        node,
                        limits,
                    ),
                }
            } else {
                Ok(
                    resolved_scalar_view_spec(
                        presentation,
                        ResolvedScalarTag::CoreString,
                        None,
                        ResolvedScalarValueView::String,
                    ),
                )
            },
        }
    }
}

pub open spec fn map_scalar_decode_error_spec(
    error: CstScalarDecodeErrorView,
) -> ScalarValueErrorView {
    ScalarValueErrorView {
        kind: ScalarValueErrorKind::ScalarDecode(error.kind),
        byte_offset: error.byte_offset,
    }
}

pub open spec fn map_tag_resolution_error_spec(
    error: TagResolutionErrorView,
) -> ScalarValueErrorView {
    ScalarValueErrorView {
        kind: ScalarValueErrorKind::TagResolution(error.kind),
        byte_offset: error.byte_offset,
    }
}

pub open spec fn resolve_profile1_cst_node_scalar_value_spec(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    node_index: u64,
    limits: ScalarValueLimitsView,
) -> Result<Option<ResolvedScalarView>, ScalarValueErrorView> {
    match crate::resolve_scalar_node::decode_profile1_cst_node_scalar_spec(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        node_index,
        CstScalarDecodeLimitsView { max_content_code_points: limits.max_content_code_points },
    ) {
        Err(error) => Err(map_scalar_decode_error_spec(error)),
        Ok(None) => Ok(None),
        Ok(Some(presentation)) => if node_index >= cst.nodes.len() {
            Err(
                ScalarValueErrorView {
                    kind: ScalarValueErrorKind::InternalInvariantViolation,
                    byte_offset: atomized.source_len_bytes,
                },
            )
        } else {
            match crate::resolve_tag::resolve_profile1_node_tag_property_spec(
                atomized,
                completed,
                cst,
                node_index,
                TagResolutionLimitsView { max_tag_code_points: limits.max_tag_code_points },
            ) {
                Err(error) => Err(map_tag_resolution_error_spec(error)),
                Ok(tag) => match resolve_decoded_scalar_value_spec(
                    presentation,
                    tag,
                    cst.nodes[node_index as int],
                    node_index,
                    limits,
                ) {
                    Ok(resolved) => Ok(Some(resolved)),
                    Err(error) => Err(error),
                },
            }
        },
    }
}

pub proof fn lemma_resolved_scalar_success_retains_requested_node_index(
    atomized: AtomizedSourceView,
    quoted: QuotedScalarSourceView,
    plain: PlainScalarSourceView,
    block: BlockScalarSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    node_index: u64,
    limits: ScalarValueLimitsView,
    scalar: ResolvedScalarView,
)
    requires
        resolve_profile1_cst_node_scalar_value_spec(
            atomized,
            quoted,
            plain,
            block,
            completed,
            cst,
            node_index,
            limits,
        ) == Ok(Some(scalar)),
    ensures
        scalar.node_index == node_index,
{
    reveal(resolve_profile1_cst_node_scalar_value_spec);
    reveal(resolve_decoded_scalar_value_spec);
}

fn decoded_scalar_matches_node(
    decoded: &DecodedCstScalar,
    node: &CstNode,
    node_index: u64,
) -> (matches: bool)
    ensures
        matches == decoded_scalar_matches_node_spec(decoded@, node@, node_index),
{
    if decoded.node_index() != node_index {
        proof {
            reveal(decoded_scalar_matches_node_spec);
        }
        return false;
    }
    let kind = node.kind();
    let node_style = node.style();
    let decoded_style = decoded.style();
    if kind == CstNodeKind::Empty && node_style == CstNodeStyle::Empty {
        let matches = decoded_style == CstNodeStyle::Empty && decoded.token_index().is_none()
            && decoded.decoded().is_none();
        proof {
            reveal(decoded_scalar_matches_node_spec);
        }
        return matches;
    }
    if kind != CstNodeKind::Scalar || decoded_style != node_style || decoded.token_index()
        != node.scalar_or_alias_token() {
        proof {
            reveal(decoded_scalar_matches_node_spec);
        }
        return false;
    }
    let content = match decoded.decoded() {
        Some(content) => content,
        None => {
            proof {
                reveal(decoded_scalar_matches_node_spec);
            }
            return false;
        },
    };
    let content_style = content.style();
    let matches =
        matches!((node_style, content_style),
        (CstNodeStyle::Plain, DecodedScalarStyle::Plain)
        | (CstNodeStyle::SingleQuoted, DecodedScalarStyle::SingleQuoted)
        | (CstNodeStyle::DoubleQuoted, DecodedScalarStyle::DoubleQuoted)
        | (CstNodeStyle::Literal, DecodedScalarStyle::LiteralBlock)
        | (CstNodeStyle::Folded, DecodedScalarStyle::FoldedBlock));
    proof {
        reveal(decoded_scalar_matches_node_spec);
        reveal(scalar_style_matches_spec);
    }
    matches
}

fn tag_matches_node(tag: Option<&ResolvedTagProperty>, node: &CstNode) -> (matches: bool)
    ensures
        matches == tag_matches_node_spec(
            match tag {
                Some(value) => Some(value@),
                None => None,
            },
            node@,
        ),
{
    let matches = match (tag, node.tag_property_token()) {
        (None, None) => true,
        (Some(property), Some(token_index)) => property.token_index() == token_index,
        _ => false,
    };
    proof {
        reveal(tag_matches_node_spec);
    }
    matches
}

fn tag_content_matches_ascii(
    content: &[crate::resolve_tag::ResolvedTagCodePoint],
    expected: &[u8],
) -> (matches: bool)
    ensures
        matches == tag_content_matches_ascii_spec(
            crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
            expected@,
        ),
{
    if content.len() != expected.len() {
        proof {
            reveal(tag_content_matches_ascii_spec);
            reveal(crate::resolve_tag::resolved_tag_code_point_views_spec);
        }
        return false;
    }
    let ghost views = crate::resolve_tag::resolved_tag_code_point_views_spec(content@);
    let mut index = 0usize;
    while index < content.len()
        invariant
            content@.len() == expected@.len(),
            views == crate::resolve_tag::resolved_tag_code_point_views_spec(content@),
            views.len() == content@.len(),
            0 <= index <= content@.len(),
            forall|prior: int|
                0 <= prior < index ==> views[prior].code_point == expected@[prior] as u32,
        decreases content.len() - index,
    {
        assert(views[index as int] == content[index as int]@) by {
            reveal(crate::resolve_tag::resolved_tag_code_point_views_spec);
        }
        if content[index].code_point() != expected[index] as u32 {
            proof {
                reveal(tag_content_matches_ascii_spec);
            }
            return false;
        }
        index += 1;
    }
    proof {
        reveal(tag_content_matches_ascii_spec);
    }
    true
}

pub(crate) fn explicit_scalar_tag_class(tag: &ResolvedTagProperty) -> (class:
    ExplicitScalarTagClass)
    ensures
        class == explicit_scalar_tag_class_spec(tag@),
{
    let kind = tag.kind();
    if kind == ResolvedTagKind::NonSpecific {
        proof {
            reveal(explicit_scalar_tag_class_spec);
        }
        return ExplicitScalarTagClass::NonSpecific;
    }
    if kind == ResolvedTagKind::Local {
        proof {
            reveal(explicit_scalar_tag_class_spec);
        }
        return ExplicitScalarTagClass::CustomLocal;
    }
    let content = tag.content();
    let class = if tag_content_matches_ascii(content, b"tag:yaml.org,2002:null") {
        ExplicitScalarTagClass::CoreNull
    } else if tag_content_matches_ascii(content, b"tag:yaml.org,2002:bool") {
        ExplicitScalarTagClass::CoreBoolean
    } else if tag_content_matches_ascii(content, b"tag:yaml.org,2002:int") {
        ExplicitScalarTagClass::CoreInteger
    } else if tag_content_matches_ascii(content, b"tag:yaml.org,2002:float") {
        ExplicitScalarTagClass::CoreFloat
    } else if tag_content_matches_ascii(content, b"tag:yaml.org,2002:str") {
        ExplicitScalarTagClass::CoreString
    } else if tag_content_matches_ascii(content, b"tag:yaml.org,2002:seq") {
        ExplicitScalarTagClass::CoreSequence
    } else if tag_content_matches_ascii(content, b"tag:yaml.org,2002:map") {
        ExplicitScalarTagClass::CoreMapping
    } else {
        ExplicitScalarTagClass::CustomGlobal
    };
    proof {
        reveal(explicit_scalar_tag_class_spec);
    }
    class
}

fn decoded_content_code_points(content: &[DecodedContentScalar]) -> (output: Vec<u32>)
    ensures
        output@ == decoded_content_code_points_spec(
            crate::scalar_decode::decoded_content_scalar_views_spec(content@),
        ),
{
    let ghost views = crate::scalar_decode::decoded_content_scalar_views_spec(content@);
    let mut output = Vec::new();
    let mut index = 0usize;
    while index < content.len()
        invariant
            views == crate::scalar_decode::decoded_content_scalar_views_spec(content@),
            views.len() == content@.len(),
            0 <= index <= content@.len(),
            output@ == Seq::new(index as nat, |position: int| views[position].code_point),
        decreases content.len() - index,
    {
        assert(views[index as int] == content[index as int]@) by {
            reveal(crate::scalar_decode::decoded_content_scalar_views_spec);
        }
        let code_point = content[index].code_point();
        proof {
            assert(Seq::new((index + 1) as nat, |position: int| views[position].code_point)
                =~= Seq::new(index as nat, |position: int| views[position].code_point).push(
                code_point,
            ));
        }
        output.push(code_point);
        index += 1;
    }
    proof {
        reveal(decoded_content_code_points_spec);
    }
    output
}

fn scalar_value_anchor(
    decoded: &DecodedCstScalar,
    node: &CstNode,
    code_point_index: u64,
) -> (offset: u64)
    ensures
        offset == scalar_value_anchor_spec(decoded@, node@, code_point_index),
{
    let content = match decoded.decoded() {
        Some(value) => value.content(),
        None => {
            let offset = node.byte_start();
            proof {
                reveal(scalar_value_anchor_spec);
            }
            return offset;
        },
    };
    if code_point_index < content.len() as u64 {
        let index = code_point_index as usize;
        let offset = content[index].byte_start();
        proof {
            reveal(scalar_value_anchor_spec);
            reveal(crate::scalar_decode::decoded_content_scalar_views_spec);
        }
        offset
    } else {
        let offset = node.byte_start();
        proof {
            reveal(scalar_value_anchor_spec);
        }
        offset
    }
}

pub(crate) fn explicit_tag_anchor(tag: &ResolvedTagProperty, node: &CstNode) -> (offset: u64)
    ensures
        offset == explicit_tag_anchor_spec(tag@, node@),
{
    let content = tag.content();
    if content.is_empty() {
        let offset = node.byte_start();
        proof {
            reveal(explicit_tag_anchor_spec);
        }
        offset
    } else {
        let offset = content[0].byte_start();
        proof {
            reveal(explicit_tag_anchor_spec);
            reveal(crate::resolve_tag::resolved_tag_code_point_views_spec);
        }
        offset
    }
}

fn explicit_scalar_tag_compatible(
    tag_class: ExplicitScalarTagClass,
    scalar_class: CorePlainScalarClass,
) -> (compatible: bool)
    ensures
        compatible == explicit_scalar_tag_compatible_spec(tag_class, scalar_class),
{
    let compatible = match tag_class {
        ExplicitScalarTagClass::CoreNull => matches!(scalar_class, CorePlainScalarClass::Null),
        ExplicitScalarTagClass::CoreBoolean => {
            matches!(scalar_class, CorePlainScalarClass::Boolean(_))
        },
        ExplicitScalarTagClass::CoreInteger => {
            matches!(scalar_class, CorePlainScalarClass::Integer { .. })
        },
        ExplicitScalarTagClass::CoreFloat => matches!(
            scalar_class,
            CorePlainScalarClass::FiniteFloat { .. }
                | CorePlainScalarClass::Infinity { .. }
                | CorePlainScalarClass::NotANumber
        ),
        _ => false,
    };
    proof {
        reveal(explicit_scalar_tag_compatible_spec);
    }
    compatible
}

fn integer_error_kind(kind: CoreIntegerErrorKind) -> (mapped: ScalarValueErrorKind)
    ensures
        mapped == integer_error_kind_spec(kind),
{
    let mapped = match kind {
        CoreIntegerErrorKind::InputLimitExceeded => {
            ScalarValueErrorKind::ScalarClassificationLimitExceeded
        },
        CoreIntegerErrorKind::MagnitudeLimitExceeded => {
            ScalarValueErrorKind::IntegerMagnitudeLimitExceeded
        },
        CoreIntegerErrorKind::NotInteger | CoreIntegerErrorKind::FuelExhausted => {
            ScalarValueErrorKind::InternalInvariantViolation
        },
    };
    proof {
        reveal(integer_error_kind_spec);
    }
    mapped
}

fn finite_float_error_kind(kind: CoreFiniteFloatErrorKind) -> (mapped: ScalarValueErrorKind)
    ensures
        mapped == finite_float_error_kind_spec(kind),
{
    let mapped = match kind {
        CoreFiniteFloatErrorKind::InputLimitExceeded => {
            ScalarValueErrorKind::ScalarClassificationLimitExceeded
        },
        CoreFiniteFloatErrorKind::CoefficientLimitExceeded => {
            ScalarValueErrorKind::FloatCoefficientLimitExceeded
        },
        CoreFiniteFloatErrorKind::ExponentLimitExceeded => {
            ScalarValueErrorKind::FloatExponentLimitExceeded
        },
        CoreFiniteFloatErrorKind::NotFiniteFloat => {
            ScalarValueErrorKind::InternalInvariantViolation
        },
    };
    proof {
        reveal(finite_float_error_kind_spec);
    }
    mapped
}

fn make_scalar(
    presentation: DecodedCstScalar,
    tag: ResolvedScalarTag,
    explicit_tag: Option<ResolvedTagProperty>,
    value: ResolvedScalarValue,
) -> (resolved: ResolvedScalar)
    ensures
        resolved@ == resolved_scalar_view_spec(
            presentation@,
            tag,
            match explicit_tag {
                Some(ref property) => Some(property@),
                None => None,
            },
            value@,
        ),
{
    let node_index = presentation.node_index();
    let resolved = ResolvedScalar::new(node_index, tag, explicit_tag, presentation, value);
    proof {
        reveal(resolved_scalar_view_spec);
    }
    resolved
}

fn resolve_classified_scalar(
    input: &[u32],
    class: CorePlainScalarClass,
    presentation: DecodedCstScalar,
    explicit_tag: Option<ResolvedTagProperty>,
    node: &CstNode,
    limits: ScalarValueLimits,
) -> (result: Result<ResolvedScalar, ScalarValueError>)
    ensures
        resolve_classified_scalar_spec(
            input@,
            class,
            presentation@,
            match explicit_tag {
                Some(ref property) => Some(property@),
                None => None,
            },
            node@,
            limits@,
        ) == match result {
            Ok(value) => Ok(value@),
            Err(error) => Err(error@),
        },
{
    match class {
        CorePlainScalarClass::Null => {
            let result = make_scalar(
                presentation,
                ResolvedScalarTag::CoreNull,
                explicit_tag,
                ResolvedScalarValue::Null,
            );
            proof {
                reveal(resolve_classified_scalar_spec);
            }
            Ok(result)
        },
        CorePlainScalarClass::Boolean(value) => {
            let result = make_scalar(
                presentation,
                ResolvedScalarTag::CoreBoolean,
                explicit_tag,
                ResolvedScalarValue::Boolean(value),
            );
            proof {
                reveal(resolve_classified_scalar_spec);
            }
            Ok(result)
        },
        CorePlainScalarClass::Integer { .. } => {
            let conversion_limits = CoreIntegerLimits::new(
                limits.max_content_code_points(),
                limits.max_integer_limbs(),
            );
            match convert_core_integer(input, conversion_limits) {
                Ok(integer) => {
                    let result = make_scalar(
                        presentation,
                        ResolvedScalarTag::CoreInteger,
                        explicit_tag,
                        ResolvedScalarValue::Integer(integer),
                    );
                    proof {
                        reveal(resolve_classified_scalar_spec);
                    }
                    Ok(result)
                },
                Err(error) => {
                    let code_point_index = error.code_point_index();
                    let byte_offset = scalar_value_anchor(&presentation, node, code_point_index);
                    let mapped = ScalarValueError::at(
                        integer_error_kind(error.kind()),
                        byte_offset,
                    );
                    proof {
                        reveal(resolve_classified_scalar_spec);
                    }
                    Err(mapped)
                },
            }
        },
        CorePlainScalarClass::FiniteFloat { .. } => {
            let conversion_limits = CoreFiniteFloatLimits::new(
                limits.max_content_code_points(),
                limits.max_float_coefficient_digits(),
                limits.max_float_exponent_digits(),
            );
            match convert_core_finite_float(input, conversion_limits) {
                Ok(value) => {
                    let result = make_scalar(
                        presentation,
                        ResolvedScalarTag::CoreFloat,
                        explicit_tag,
                        ResolvedScalarValue::FiniteFloat(value),
                    );
                    proof {
                        reveal(resolve_classified_scalar_spec);
                    }
                    Ok(result)
                },
                Err(error) => {
                    let code_point_index = error.code_point_index();
                    let byte_offset = scalar_value_anchor(&presentation, node, code_point_index);
                    let mapped = ScalarValueError::at(
                        finite_float_error_kind(error.kind()),
                        byte_offset,
                    );
                    proof {
                        reveal(resolve_classified_scalar_spec);
                    }
                    Err(mapped)
                },
            }
        },
        CorePlainScalarClass::Infinity { negative } => {
            let value = if negative {
                ResolvedScalarValue::NegativeInfinity
            } else {
                ResolvedScalarValue::PositiveInfinity
            };
            let result = make_scalar(
                presentation,
                ResolvedScalarTag::CoreFloat,
                explicit_tag,
                value,
            );
            proof {
                reveal(resolve_classified_scalar_spec);
            }
            Ok(result)
        },
        CorePlainScalarClass::NotANumber => {
            let result = make_scalar(
                presentation,
                ResolvedScalarTag::CoreFloat,
                explicit_tag,
                ResolvedScalarValue::NotANumber,
            );
            proof {
                reveal(resolve_classified_scalar_spec);
            }
            Ok(result)
        },
        CorePlainScalarClass::String => {
            let result = make_scalar(
                presentation,
                ResolvedScalarTag::CoreString,
                explicit_tag,
                ResolvedScalarValue::String,
            );
            proof {
                reveal(resolve_classified_scalar_spec);
            }
            Ok(result)
        },
    }
}

fn classify_scalar(
    input: &[u32],
    presentation: &DecodedCstScalar,
    node: &CstNode,
    limits: ScalarValueLimits,
) -> (result: Result<CorePlainScalarClass, ScalarValueError>)
    ensures
        match crate::resolve::classify_core_plain_scalar_spec(
            input@,
            crate::resolve::CoreScalarLimitsView {
                max_code_points: limits@.max_content_code_points,
            },
        ) {
            Ok(value) => result == Ok(value),
            Err(error) => match result {
                Err(mapped) => mapped@ == (ScalarValueErrorView {
                    kind: ScalarValueErrorKind::ScalarClassificationLimitExceeded,
                    byte_offset: scalar_value_anchor_spec(
                        presentation@,
                        node@,
                        error.code_point_index,
                    ),
                }),
                Ok(_) => false,
            },
        },
{
    let classifier_limits = CoreScalarLimits::new(limits.max_content_code_points());
    match classify_core_plain_scalar(input, classifier_limits) {
        Ok(class) => Ok(class),
        Err(error) => {
            let byte_offset = scalar_value_anchor(presentation, node, error.code_point_index());
            let mapped = ScalarValueError::at(
                match error.kind() {
                    CoreScalarErrorKind::InputLimitExceeded => {
                        ScalarValueErrorKind::ScalarClassificationLimitExceeded
                    },
                },
                byte_offset,
            );
            Err(mapped)
        },
    }
}

fn resolve_decoded_scalar_value(
    presentation: DecodedCstScalar,
    explicit_tag: Option<ResolvedTagProperty>,
    node: &CstNode,
    node_index: u64,
    limits: ScalarValueLimits,
) -> (result: Result<ResolvedScalar, ScalarValueError>)
    ensures
        resolve_decoded_scalar_value_spec(
            presentation@,
            match explicit_tag {
                Some(ref property) => Some(property@),
                None => None,
            },
            node@,
            node_index,
            limits@,
        ) == match result {
            Ok(value) => Ok(value@),
            Err(error) => Err(error@),
        },
{
    if !decoded_scalar_matches_node(&presentation, node, node_index) || !tag_matches_node(
        explicit_tag.as_ref(),
        node,
    ) {
        let error = ScalarValueError::at(
            ScalarValueErrorKind::InvalidScalarPresentation,
            node.byte_start(),
        );
        proof {
            reveal(resolve_decoded_scalar_value_spec);
        }
        return Err(error);
    }
    let input = match presentation.decoded() {
        Some(decoded) => decoded_content_code_points(decoded.content()),
        None => {
            let empty = Vec::new();
            proof {
                reveal(decoded_scalar_code_points_spec);
            }
            empty
        },
    };
    proof {
        reveal(decoded_scalar_code_points_spec);
    }

    match explicit_tag {
        Some(tag) => {
            let tag_class = explicit_scalar_tag_class(&tag);
            match tag_class {
                ExplicitScalarTagClass::NonSpecific | ExplicitScalarTagClass::CoreString => {
                    let result = make_scalar(
                        presentation,
                        ResolvedScalarTag::CoreString,
                        Some(tag),
                        ResolvedScalarValue::String,
                    );
                    proof {
                        reveal(resolve_decoded_scalar_value_spec);
                    }
                    Ok(result)
                },
                ExplicitScalarTagClass::CustomGlobal => {
                    let result = make_scalar(
                        presentation,
                        ResolvedScalarTag::CustomGlobal,
                        Some(tag),
                        ResolvedScalarValue::String,
                    );
                    proof {
                        reveal(resolve_decoded_scalar_value_spec);
                    }
                    Ok(result)
                },
                ExplicitScalarTagClass::CustomLocal => {
                    let result = make_scalar(
                        presentation,
                        ResolvedScalarTag::CustomLocal,
                        Some(tag),
                        ResolvedScalarValue::String,
                    );
                    proof {
                        reveal(resolve_decoded_scalar_value_spec);
                    }
                    Ok(result)
                },
                ExplicitScalarTagClass::CoreSequence | ExplicitScalarTagClass::CoreMapping => {
                    let byte_offset = explicit_tag_anchor(&tag, node);
                    let error = ScalarValueError::at(
                        ScalarValueErrorKind::ScalarTagKindMismatch,
                        byte_offset,
                    );
                    proof {
                        reveal(resolve_decoded_scalar_value_spec);
                    }
                    Err(error)
                },
                _ => {
                    let class = match classify_scalar(&input, &presentation, node, limits) {
                        Ok(class) => class,
                        Err(error) => {
                            proof {
                                reveal(resolve_decoded_scalar_value_spec);
                            }
                            return Err(error);
                        },
                    };
                    if !explicit_scalar_tag_compatible(tag_class, class) {
                        let byte_offset = scalar_value_anchor(&presentation, node, 0);
                        let error = ScalarValueError::at(
                            ScalarValueErrorKind::InvalidExplicitScalarTagValue,
                            byte_offset,
                        );
                        proof {
                            reveal(resolve_decoded_scalar_value_spec);
                        }
                        return Err(error);
                    }
                    let result = resolve_classified_scalar(
                        &input,
                        class,
                        presentation,
                        Some(tag),
                        node,
                        limits,
                    );
                    proof {
                        reveal(resolve_decoded_scalar_value_spec);
                    }
                    result
                },
            }
        },
        None => {
            if node.kind() == CstNodeKind::Empty {
                let result = make_scalar(
                    presentation,
                    ResolvedScalarTag::CoreNull,
                    None,
                    ResolvedScalarValue::Null,
                );
                proof {
                    reveal(resolve_decoded_scalar_value_spec);
                }
                return Ok(result);
            }
            if node.style() == CstNodeStyle::Plain {
                let class = match classify_scalar(&input, &presentation, node, limits) {
                    Ok(class) => class,
                    Err(error) => {
                        proof {
                            reveal(resolve_decoded_scalar_value_spec);
                        }
                        return Err(error);
                    },
                };
                let result = resolve_classified_scalar(
                    &input,
                    class,
                    presentation,
                    None,
                    node,
                    limits,
                );
                proof {
                    reveal(resolve_decoded_scalar_value_spec);
                }
                result
            } else {
                let result = make_scalar(
                    presentation,
                    ResolvedScalarTag::CoreString,
                    None,
                    ResolvedScalarValue::String,
                );
                proof {
                    reveal(resolve_decoded_scalar_value_spec);
                }
                Ok(result)
            }
        },
    }
}

fn map_scalar_decode_error(error: CstScalarDecodeError) -> (mapped: ScalarValueError)
    ensures
        mapped@ == map_scalar_decode_error_spec(error@),
{
    let mapped = ScalarValueError::at(
        ScalarValueErrorKind::ScalarDecode(error.kind()),
        error.byte_offset(),
    );
    proof {
        reveal(map_scalar_decode_error_spec);
    }
    mapped
}

fn map_tag_resolution_error(error: TagResolutionError) -> (mapped: ScalarValueError)
    ensures
        mapped@ == map_tag_resolution_error_spec(error@),
{
    let mapped = ScalarValueError::at(
        ScalarValueErrorKind::TagResolution(error.kind()),
        error.byte_offset(),
    );
    proof {
        reveal(map_tag_resolution_error_spec);
    }
    mapped
}

#[expect(clippy::too_many_arguments, reason = "independent proof inputs remain explicit in the executable-to-spec contract")]  // Every authenticated producer remains an explicit input.
pub fn resolve_profile1_cst_node_scalar_value(
    atomized: &AtomizedSource,
    quoted: &QuotedScalarSource,
    plain: &PlainScalarSource,
    block: &BlockScalarSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    node_index: u64,
    limits: ScalarValueLimits,
) -> (result: Result<Option<ResolvedScalar>, ScalarValueError>)
    ensures
        resolve_profile1_cst_node_scalar_value_spec(
            atomized@,
            quoted@,
            plain@,
            block@,
            completed@,
            cst@,
            node_index,
            limits@,
        ) == match result {
            Ok(Some(value)) => Ok(Some(value@)),
            Ok(None) => Ok(None),
            Err(error) => Err(error@),
        },
{
    let decode_limits = CstScalarDecodeLimits::new(limits.max_content_code_points());
    let presentation = match decode_profile1_cst_node_scalar(
        atomized,
        quoted,
        plain,
        block,
        completed,
        cst,
        node_index,
        decode_limits,
    ) {
        Ok(Some(value)) => value,
        Ok(None) => {
            proof {
                reveal(resolve_profile1_cst_node_scalar_value_spec);
            }
            return Ok(None);
        },
        Err(error) => {
            let mapped = map_scalar_decode_error(error);
            proof {
                reveal(resolve_profile1_cst_node_scalar_value_spec);
            }
            return Err(mapped);
        },
    };

    let nodes = cst.nodes();
    if node_index >= nodes.len() as u64 {
        let error = ScalarValueError::at(
            ScalarValueErrorKind::InternalInvariantViolation,
            atomized.source_len_bytes(),
        );
        proof {
            reveal(resolve_profile1_cst_node_scalar_value_spec);
            reveal(crate::cst::cst_node_views_spec);
        }
        return Err(error);
    }
    let explicit_tag = match resolve_profile1_node_tag_property(
        atomized,
        completed,
        cst,
        node_index,
        TagResolutionLimits::new(limits.max_tag_code_points()),
    ) {
        Ok(tag) => tag,
        Err(error) => {
            let mapped = map_tag_resolution_error(error);
            proof {
                reveal(resolve_profile1_cst_node_scalar_value_spec);
            }
            return Err(mapped);
        },
    };

    let index = node_index as usize;
    let node = &nodes[index];
    proof {
        reveal(crate::cst::cst_node_views_spec);
        assert(cst@.nodes[node_index as int] == node@);
    }
    let resolved = resolve_decoded_scalar_value(
        presentation,
        explicit_tag,
        node,
        node_index,
        limits,
    );
    proof {
        reveal(resolve_profile1_cst_node_scalar_value_spec);
    }
    match resolved {
        Ok(value) => Ok(Some(value)),
        Err(error) => Err(error),
    }
}

} // verus!
