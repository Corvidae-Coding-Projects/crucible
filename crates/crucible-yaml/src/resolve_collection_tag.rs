//! Verified semantic-tag resolution for authenticated YAML collection nodes.
use crate::atom::AtomizedSource;
#[allow(unused_imports)]
use crate::atom::AtomizedSourceView;
use crate::cst::{CstNode, CstNodeKind, CstSource};
#[allow(unused_imports)]
use crate::cst::{CstNodeView, CstSourceView};
use crate::resolve_scalar_value::ExplicitScalarTagClass;
use crate::resolve_tag::{
    resolve_profile1_node_tag_property, ResolvedTagProperty, TagResolutionError,
    TagResolutionErrorKind, TagResolutionLimits,
};
#[allow(unused_imports)]
use crate::resolve_tag::{
    ResolvedTagPropertyView, TagResolutionErrorView, TagResolutionLimitsView,
};
use crate::token::CompletedTokenSource;
#[allow(unused_imports)]
use crate::token::CompletedTokenSourceView;
use vstd::prelude::*;

verus! {

pub const COLLECTION_TAG_RESOLUTION_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectionTagLimits {
    max_tag_code_points: u64,
}

#[verifier::ext_equal]
pub struct CollectionTagLimitsView {
    pub max_tag_code_points: u64,
}

impl View for CollectionTagLimits {
    type V = CollectionTagLimitsView;

    closed spec fn view(&self) -> CollectionTagLimitsView {
        CollectionTagLimitsView { max_tag_code_points: self.max_tag_code_points }
    }
}

impl CollectionTagLimits {
    pub fn new(max_tag_code_points: u64) -> (limits: Self)
        ensures
            limits@ == (CollectionTagLimitsView { max_tag_code_points }),
    {
        Self { max_tag_code_points }
    }

    pub fn max_tag_code_points(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_tag_code_points,
    {
        self.max_tag_code_points
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
pub enum ResolvedCollectionTag {
    CoreSequence,
    CoreMapping,
    CustomGlobal,
    CustomLocal,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedCollection {
    node_index: u64,
    kind: CstNodeKind,
    tag: ResolvedCollectionTag,
    explicit_tag: Option<ResolvedTagProperty>,
}

#[verifier::ext_equal]
pub struct ResolvedCollectionView {
    pub node_index: u64,
    pub kind: CstNodeKind,
    pub tag: ResolvedCollectionTag,
    pub explicit_tag: Option<ResolvedTagPropertyView>,
}

impl View for ResolvedCollection {
    type V = ResolvedCollectionView;

    closed spec fn view(&self) -> ResolvedCollectionView {
        ResolvedCollectionView {
            node_index: self.node_index,
            kind: self.kind,
            tag: self.tag,
            explicit_tag: match self.explicit_tag {
                Some(ref property) => Some(property@),
                None => None,
            },
        }
    }
}

impl ResolvedCollection {
    fn new(
        node_index: u64,
        kind: CstNodeKind,
        tag: ResolvedCollectionTag,
        explicit_tag: Option<ResolvedTagProperty>,
    ) -> (resolved: Self)
        ensures
            resolved@ == (ResolvedCollectionView {
                node_index,
                kind,
                tag,
                explicit_tag: match explicit_tag {
                    Some(ref property) => Some(property@),
                    None => None,
                },
            }),
    {
        Self { node_index, kind, tag, explicit_tag }
    }

    pub fn node_index(&self) -> (index: u64)
        ensures
            index == self@.node_index,
    {
        self.node_index
    }

    pub fn kind(&self) -> (kind: CstNodeKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn tag(&self) -> (tag: ResolvedCollectionTag)
        ensures
            tag == self@.tag,
    {
        self.tag
    }

    pub fn explicit_tag(&self) -> (tag: Option<&ResolvedTagProperty>)
        ensures
            match tag {
                Some(property) => self@.explicit_tag == Some(property@),
                None => self@.explicit_tag.is_none(),
            },
    {
        self.explicit_tag.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum CollectionTagErrorKind {
    TagResolution(TagResolutionErrorKind),
    InvalidCollectionNode,
    CollectionTagKindMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectionTagError {
    kind: CollectionTagErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct CollectionTagErrorView {
    pub kind: CollectionTagErrorKind,
    pub byte_offset: u64,
}

impl View for CollectionTagError {
    type V = CollectionTagErrorView;

    closed spec fn view(&self) -> CollectionTagErrorView {
        CollectionTagErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl CollectionTagError {
    fn at(kind: CollectionTagErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (CollectionTagErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: CollectionTagErrorKind)
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

pub open spec fn map_tag_resolution_error_spec(
    error: TagResolutionErrorView,
) -> CollectionTagErrorView {
    CollectionTagErrorView {
        kind: CollectionTagErrorKind::TagResolution(error.kind),
        byte_offset: error.byte_offset,
    }
}

pub open spec fn implicit_collection_tag_spec(kind: CstNodeKind) -> ResolvedCollectionTag {
    if kind == CstNodeKind::Sequence {
        ResolvedCollectionTag::CoreSequence
    } else {
        ResolvedCollectionTag::CoreMapping
    }
}

pub open spec fn collection_tag_anchor_spec(
    tag: ResolvedTagPropertyView,
    node: CstNodeView,
) -> u64 {
    crate::resolve_scalar_value::explicit_tag_anchor_spec(tag, node)
}

pub open spec fn resolve_collection_tag_spec(
    explicit_tag: Option<ResolvedTagPropertyView>,
    node: CstNodeView,
    node_index: u64,
) -> Result<ResolvedCollectionView, CollectionTagErrorView> {
    if node.kind != CstNodeKind::Sequence && node.kind != CstNodeKind::Mapping {
        Err(
            CollectionTagErrorView {
                kind: CollectionTagErrorKind::InvalidCollectionNode,
                byte_offset: node.byte_start,
            },
        )
    } else if !crate::resolve_scalar_value::tag_matches_node_spec(explicit_tag, node) {
        Err(
            CollectionTagErrorView {
                kind: CollectionTagErrorKind::InvalidCollectionNode,
                byte_offset: node.byte_start,
            },
        )
    } else {
        match explicit_tag {
            None => Ok(
                ResolvedCollectionView {
                    node_index,
                    kind: node.kind,
                    tag: implicit_collection_tag_spec(node.kind),
                    explicit_tag: None,
                },
            ),
            Some(tag) => {
                let class = crate::resolve_scalar_value::explicit_scalar_tag_class_spec(tag);
                let semantic_tag = match class {
                    ExplicitScalarTagClass::NonSpecific => Some(
                        implicit_collection_tag_spec(node.kind),
                    ),
                    ExplicitScalarTagClass::CoreSequence => if node.kind == CstNodeKind::Sequence {
                        Some(ResolvedCollectionTag::CoreSequence)
                    } else {
                        None
                    },
                    ExplicitScalarTagClass::CoreMapping => if node.kind == CstNodeKind::Mapping {
                        Some(ResolvedCollectionTag::CoreMapping)
                    } else {
                        None
                    },
                    ExplicitScalarTagClass::CustomGlobal => {
                        Some(ResolvedCollectionTag::CustomGlobal)
                    },
                    ExplicitScalarTagClass::CustomLocal => {
                        Some(ResolvedCollectionTag::CustomLocal)
                    },
                    _ => None,
                };
                match semantic_tag {
                    Some(resolved_tag) => Ok(
                        ResolvedCollectionView {
                            node_index,
                            kind: node.kind,
                            tag: resolved_tag,
                            explicit_tag: Some(tag),
                        },
                    ),
                    None => Err(
                        CollectionTagErrorView {
                            kind: CollectionTagErrorKind::CollectionTagKindMismatch,
                            byte_offset: collection_tag_anchor_spec(tag, node),
                        },
                    ),
                }
            },
        }
    }
}

pub open spec fn resolve_profile1_cst_node_collection_tag_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    node_index: u64,
    limits: CollectionTagLimitsView,
) -> Result<Option<ResolvedCollectionView>, CollectionTagErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            CollectionTagErrorView {
                kind: CollectionTagErrorKind::TagResolution(
                    TagResolutionErrorKind::InputCompletedTokenMismatch,
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() {
        Err(
            CollectionTagErrorView {
                kind: CollectionTagErrorKind::TagResolution(
                    TagResolutionErrorKind::InputCstMismatch,
                ),
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if node_index >= cst.nodes.len() {
        Err(
            CollectionTagErrorView {
                kind: CollectionTagErrorKind::TagResolution(
                    TagResolutionErrorKind::NodeIndexOutOfRange,
                ),
                byte_offset: atomized.source_len_bytes,
            },
        )
    } else {
        let node = cst.nodes[node_index as int];
        if node.kind != CstNodeKind::Sequence && node.kind != CstNodeKind::Mapping {
            Ok(None)
        } else {
            match crate::resolve_tag::resolve_profile1_node_tag_property_spec(
                atomized,
                completed,
                cst,
                node_index,
                TagResolutionLimitsView { max_tag_code_points: limits.max_tag_code_points },
            ) {
                Err(error) => Err(map_tag_resolution_error_spec(error)),
                Ok(tag) => match resolve_collection_tag_spec(tag, node, node_index) {
                    Ok(resolved) => Ok(Some(resolved)),
                    Err(error) => Err(error),
                },
            }
        }
    }
}

fn map_tag_resolution_error(error: TagResolutionError) -> (mapped: CollectionTagError)
    ensures
        mapped@ == map_tag_resolution_error_spec(error@),
{
    let mapped = CollectionTagError::at(
        CollectionTagErrorKind::TagResolution(error.kind()),
        error.byte_offset(),
    );
    proof {
        reveal(map_tag_resolution_error_spec);
    }
    mapped
}

fn tag_matches_node(tag: Option<&ResolvedTagProperty>, node: &CstNode) -> (matches: bool)
    ensures
        matches == crate::resolve_scalar_value::tag_matches_node_spec(
            match tag {
                Some(property) => Some(property@),
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
        reveal(crate::resolve_scalar_value::tag_matches_node_spec);
    }
    matches
}

fn implicit_collection_tag(kind: CstNodeKind) -> (tag: ResolvedCollectionTag)
    requires
        kind == CstNodeKind::Sequence || kind == CstNodeKind::Mapping,
    ensures
        tag == implicit_collection_tag_spec(kind),
{
    let tag = if kind == CstNodeKind::Sequence {
        ResolvedCollectionTag::CoreSequence
    } else {
        ResolvedCollectionTag::CoreMapping
    };
    proof {
        reveal(implicit_collection_tag_spec);
    }
    tag
}

fn resolve_collection_tag(
    explicit_tag: Option<ResolvedTagProperty>,
    node: &CstNode,
    node_index: u64,
) -> (result: Result<ResolvedCollection, CollectionTagError>)
    ensures
        resolve_collection_tag_spec(
            match explicit_tag {
                Some(ref property) => Some(property@),
                None => None,
            },
            node@,
            node_index,
        ) == match result {
            Ok(resolved) => Ok(resolved@),
            Err(error) => Err(error@),
        },
{
    let kind = node.kind();
    if kind != CstNodeKind::Sequence && kind != CstNodeKind::Mapping {
        let error = CollectionTagError::at(
            CollectionTagErrorKind::InvalidCollectionNode,
            node.byte_start(),
        );
        proof {
            reveal(resolve_collection_tag_spec);
        }
        return Err(error);
    }
    if !tag_matches_node(explicit_tag.as_ref(), node) {
        let error = CollectionTagError::at(
            CollectionTagErrorKind::InvalidCollectionNode,
            node.byte_start(),
        );
        proof {
            reveal(resolve_collection_tag_spec);
        }
        return Err(error);
    }
    let tag = match explicit_tag {
        None => {
            let semantic = implicit_collection_tag(kind);
            let resolved = ResolvedCollection::new(node_index, kind, semantic, None);
            proof {
                reveal(resolve_collection_tag_spec);
            }
            return Ok(resolved);
        },
        Some(tag) => tag,
    };
    let class = crate::resolve_scalar_value::explicit_scalar_tag_class(&tag);
    let semantic_tag = match class {
        ExplicitScalarTagClass::NonSpecific => Some(implicit_collection_tag(kind)),
        ExplicitScalarTagClass::CoreSequence => if kind == CstNodeKind::Sequence {
            Some(ResolvedCollectionTag::CoreSequence)
        } else {
            None
        },
        ExplicitScalarTagClass::CoreMapping => if kind == CstNodeKind::Mapping {
            Some(ResolvedCollectionTag::CoreMapping)
        } else {
            None
        },
        ExplicitScalarTagClass::CustomGlobal => Some(ResolvedCollectionTag::CustomGlobal),
        ExplicitScalarTagClass::CustomLocal => Some(ResolvedCollectionTag::CustomLocal),
        _ => None,
    };
    match semantic_tag {
        Some(resolved_tag) => {
            let resolved = ResolvedCollection::new(node_index, kind, resolved_tag, Some(tag));
            proof {
                reveal(resolve_collection_tag_spec);
            }
            Ok(resolved)
        },
        None => {
            let byte_offset = crate::resolve_scalar_value::explicit_tag_anchor(&tag, node);
            let error = CollectionTagError::at(
                CollectionTagErrorKind::CollectionTagKindMismatch,
                byte_offset,
            );
            proof {
                reveal(collection_tag_anchor_spec);
                reveal(resolve_collection_tag_spec);
            }
            Err(error)
        },
    }
}

pub fn resolve_profile1_cst_node_collection_tag(
    atomized: &AtomizedSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    node_index: u64,
    limits: CollectionTagLimits,
) -> (result: Result<Option<ResolvedCollection>, CollectionTagError>)
    ensures
        resolve_profile1_cst_node_collection_tag_spec(
            atomized@,
            completed@,
            cst@,
            node_index,
            limits@,
        ) == match result {
            Ok(Some(resolved)) => Ok(Some(resolved@)),
            Ok(None) => Ok(None),
            Err(error) => Err(error@),
        },
{
    let bom_bytes = atomized.bom_bytes();
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let nodes = cst.nodes();
    proof {
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(crate::cst::cst_node_views_spec);
        crate::token::lemma_completed_token_views_len(tokens@);
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens.len() == tokens@.len());
        assert(cst@.nodes.len() == nodes@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != bom_bytes
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = CollectionTagError::at(
            CollectionTagErrorKind::TagResolution(
                TagResolutionErrorKind::InputCompletedTokenMismatch,
            ),
            bom_bytes,
        );
        proof {
            reveal(resolve_profile1_cst_node_collection_tag_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 {
        let error = CollectionTagError::at(
            CollectionTagErrorKind::TagResolution(TagResolutionErrorKind::InputCstMismatch),
            bom_bytes,
        );
        proof {
            reveal(resolve_profile1_cst_node_collection_tag_spec);
        }
        return Err(error);
    }
    if node_index >= nodes.len() as u64 {
        let error = CollectionTagError::at(
            CollectionTagErrorKind::TagResolution(TagResolutionErrorKind::NodeIndexOutOfRange),
            atomized.source_len_bytes(),
        );
        proof {
            reveal(resolve_profile1_cst_node_collection_tag_spec);
        }
        return Err(error);
    }
    let index = node_index as usize;
    let node = &nodes[index];
    proof {
        reveal(crate::cst::cst_node_views_spec);
        assert(cst@.nodes[node_index as int] == node@);
    }
    let kind = node.kind();
    if kind != CstNodeKind::Sequence && kind != CstNodeKind::Mapping {
        proof {
            reveal(resolve_profile1_cst_node_collection_tag_spec);
        }
        return Ok(None);
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
                reveal(resolve_profile1_cst_node_collection_tag_spec);
            }
            return Err(mapped);
        },
    };
    let resolved = resolve_collection_tag(explicit_tag, node, node_index);
    proof {
        reveal(resolve_profile1_cst_node_collection_tag_spec);
    }
    match resolved {
        Ok(value) => Ok(Some(value)),
        Err(error) => Err(error),
    }
}

} // verus!
