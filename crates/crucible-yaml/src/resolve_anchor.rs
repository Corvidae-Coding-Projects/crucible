//! Verified document-scoped YAML anchor and alias binding.
use crate::atom::{AtomizedSource, LexicalAtom};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::atom::{AtomizedSourceView, LexicalAtomView};
use crate::cst::{CstDocument, CstSource, CstSyntaxOwner, CstSyntaxOwnerKind};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::cst::{CstDocumentView, CstSourceView, CstSyntaxOwnerView};
use crate::token::{
    CompletedToken, CompletedTokenKind, CompletedTokenPart, CompletedTokenPartKind,
    CompletedTokenSource,
};
#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use crate::token::{CompletedTokenPartView, CompletedTokenSourceView, CompletedTokenView};
use vstd::prelude::*;

verus! {

pub const ANCHOR_ALIAS_RESOLUTION_VERSION: u16 = 1;

pub const MAX_PROFILE1_ANCHOR_DECLARATIONS: u64 = 1_048_576;

pub const MAX_PROFILE1_ALIAS_BINDINGS: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorAliasLimits {
    max_anchors: u64,
    max_aliases: u64,
}

#[verifier::ext_equal]
pub struct AnchorAliasLimitsView {
    pub max_anchors: u64,
    pub max_aliases: u64,
}

impl View for AnchorAliasLimits {
    type V = AnchorAliasLimitsView;

    closed spec fn view(&self) -> AnchorAliasLimitsView {
        AnchorAliasLimitsView { max_anchors: self.max_anchors, max_aliases: self.max_aliases }
    }
}

impl AnchorAliasLimits {
    pub fn new(max_anchors: u64, max_aliases: u64) -> (limits: Self)
        ensures
            limits@ == (AnchorAliasLimitsView { max_anchors, max_aliases }),
    {
        Self { max_anchors, max_aliases }
    }

    pub fn max_anchors(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_anchors,
    {
        self.max_anchors
    }

    pub fn max_aliases(&self) -> (maximum: u64)
        ensures
            maximum == self@.max_aliases,
    {
        self.max_aliases
    }
}

pub open spec fn effective_anchor_limit_spec(limits: AnchorAliasLimitsView) -> u64 {
    if limits.max_anchors < MAX_PROFILE1_ANCHOR_DECLARATIONS {
        limits.max_anchors
    } else {
        MAX_PROFILE1_ANCHOR_DECLARATIONS
    }
}

pub open spec fn effective_alias_limit_spec(limits: AnchorAliasLimitsView) -> u64 {
    if limits.max_aliases < MAX_PROFILE1_ALIAS_BINDINGS {
        limits.max_aliases
    } else {
        MAX_PROFILE1_ALIAS_BINDINGS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorDeclaration {
    document_index: u64,
    node_index: u64,
    token_index: u64,
    name_start_atom_index: u64,
    name_end_atom_index: u64,
    name_byte_start: u64,
    name_byte_end: u64,
}

#[verifier::ext_equal]
pub struct AnchorDeclarationView {
    pub document_index: u64,
    pub node_index: u64,
    pub token_index: u64,
    pub name_start_atom_index: u64,
    pub name_end_atom_index: u64,
    pub name_byte_start: u64,
    pub name_byte_end: u64,
}

impl View for AnchorDeclaration {
    type V = AnchorDeclarationView;

    closed spec fn view(&self) -> AnchorDeclarationView {
        AnchorDeclarationView {
            document_index: self.document_index,
            node_index: self.node_index,
            token_index: self.token_index,
            name_start_atom_index: self.name_start_atom_index,
            name_end_atom_index: self.name_end_atom_index,
            name_byte_start: self.name_byte_start,
            name_byte_end: self.name_byte_end,
        }
    }
}

impl AnchorDeclaration {
    fn new(
        document_index: u64,
        node_index: u64,
        token_index: u64,
        part: &CompletedTokenPart,
    ) -> (declaration: Self)
        ensures
            declaration@ == (AnchorDeclarationView {
                document_index,
                node_index,
                token_index,
                name_start_atom_index: part@.start_atom_index,
                name_end_atom_index: part@.end_atom_index,
                name_byte_start: part@.byte_start,
                name_byte_end: part@.byte_end,
            }),
    {
        Self {
            document_index,
            node_index,
            token_index,
            name_start_atom_index: part.start_atom_index(),
            name_end_atom_index: part.end_atom_index(),
            name_byte_start: part.byte_start(),
            name_byte_end: part.byte_end(),
        }
    }

    pub fn document_index(&self) -> (index: u64)
        ensures
            index == self@.document_index,
    {
        self.document_index
    }

    pub fn node_index(&self) -> (index: u64)
        ensures
            index == self@.node_index,
    {
        self.node_index
    }

    pub fn token_index(&self) -> (index: u64)
        ensures
            index == self@.token_index,
    {
        self.token_index
    }

    pub fn name_start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.name_start_atom_index,
    {
        self.name_start_atom_index
    }

    pub fn name_end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.name_end_atom_index,
    {
        self.name_end_atom_index
    }

    pub fn name_byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.name_byte_start,
    {
        self.name_byte_start
    }

    pub fn name_byte_end(&self) -> (offset: u64)
        ensures
            offset == self@.name_byte_end,
    {
        self.name_byte_end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AliasBinding {
    document_index: u64,
    alias_node_index: u64,
    alias_token_index: u64,
    name_start_atom_index: u64,
    name_end_atom_index: u64,
    name_byte_start: u64,
    name_byte_end: u64,
    target_anchor_index: u64,
    target_node_index: u64,
}

#[verifier::ext_equal]
pub struct AliasBindingView {
    pub document_index: u64,
    pub alias_node_index: u64,
    pub alias_token_index: u64,
    pub name_start_atom_index: u64,
    pub name_end_atom_index: u64,
    pub name_byte_start: u64,
    pub name_byte_end: u64,
    pub target_anchor_index: u64,
    pub target_node_index: u64,
}

impl View for AliasBinding {
    type V = AliasBindingView;

    closed spec fn view(&self) -> AliasBindingView {
        AliasBindingView {
            document_index: self.document_index,
            alias_node_index: self.alias_node_index,
            alias_token_index: self.alias_token_index,
            name_start_atom_index: self.name_start_atom_index,
            name_end_atom_index: self.name_end_atom_index,
            name_byte_start: self.name_byte_start,
            name_byte_end: self.name_byte_end,
            target_anchor_index: self.target_anchor_index,
            target_node_index: self.target_node_index,
        }
    }
}

impl AliasBinding {
    fn new(
        document_index: u64,
        alias_node_index: u64,
        alias_token_index: u64,
        part: &CompletedTokenPart,
        target_anchor_index: u64,
        target_node_index: u64,
    ) -> (binding: Self)
        ensures
            binding@ == (AliasBindingView {
                document_index,
                alias_node_index,
                alias_token_index,
                name_start_atom_index: part@.start_atom_index,
                name_end_atom_index: part@.end_atom_index,
                name_byte_start: part@.byte_start,
                name_byte_end: part@.byte_end,
                target_anchor_index,
                target_node_index,
            }),
    {
        Self {
            document_index,
            alias_node_index,
            alias_token_index,
            name_start_atom_index: part.start_atom_index(),
            name_end_atom_index: part.end_atom_index(),
            name_byte_start: part.byte_start(),
            name_byte_end: part.byte_end(),
            target_anchor_index,
            target_node_index,
        }
    }

    pub fn document_index(&self) -> (index: u64)
        ensures
            index == self@.document_index,
    {
        self.document_index
    }

    pub fn alias_node_index(&self) -> (index: u64)
        ensures
            index == self@.alias_node_index,
    {
        self.alias_node_index
    }

    pub fn alias_token_index(&self) -> (index: u64)
        ensures
            index == self@.alias_token_index,
    {
        self.alias_token_index
    }

    pub fn name_start_atom_index(&self) -> (index: u64)
        ensures
            index == self@.name_start_atom_index,
    {
        self.name_start_atom_index
    }

    pub fn name_end_atom_index(&self) -> (index: u64)
        ensures
            index == self@.name_end_atom_index,
    {
        self.name_end_atom_index
    }

    pub fn name_byte_start(&self) -> (offset: u64)
        ensures
            offset == self@.name_byte_start,
    {
        self.name_byte_start
    }

    pub fn name_byte_end(&self) -> (offset: u64)
        ensures
            offset == self@.name_byte_end,
    {
        self.name_byte_end
    }

    pub fn target_anchor_index(&self) -> (index: u64)
        ensures
            index == self@.target_anchor_index,
    {
        self.target_anchor_index
    }

    pub fn target_node_index(&self) -> (index: u64)
        ensures
            index == self@.target_node_index,
    {
        self.target_node_index
    }
}

pub open spec fn anchor_declaration_views_spec(values: Seq<AnchorDeclaration>) -> Seq<
    AnchorDeclarationView,
> {
    Seq::new(values.len(), |index: int| values[index]@)
}

pub open spec fn alias_binding_views_spec(values: Seq<AliasBinding>) -> Seq<AliasBindingView> {
    Seq::new(values.len(), |index: int| values[index]@)
}

proof fn lemma_anchor_declaration_views_push(
    values: Seq<AnchorDeclaration>,
    value: AnchorDeclaration,
)
    ensures
        anchor_declaration_views_spec(values.push(value)) == anchor_declaration_views_spec(
            values,
        ).push(value@),
{
    reveal(anchor_declaration_views_spec);
    assert(anchor_declaration_views_spec(values.push(value)) =~= anchor_declaration_views_spec(
        values,
    ).push(value@));
}

proof fn lemma_alias_binding_views_push(values: Seq<AliasBinding>, value: AliasBinding)
    ensures
        alias_binding_views_spec(values.push(value)) == alias_binding_views_spec(values).push(
            value@,
        ),
{
    reveal(alias_binding_views_spec);
    assert(alias_binding_views_spec(values.push(value)) =~= alias_binding_views_spec(values).push(
        value@,
    ));
}

#[derive(Debug, PartialEq, Eq)]
pub struct AnchorAliasSource {
    profile_version: u16,
    transformation_version: u16,
    source_len_bytes: u64,
    input_token_transformation_version: u16,
    input_cst_transformation_version: u16,
    anchors: Vec<AnchorDeclaration>,
    aliases: Vec<AliasBinding>,
}

#[verifier::ext_equal]
pub struct AnchorAliasSourceView {
    pub profile_version: u16,
    pub transformation_version: u16,
    pub source_len_bytes: u64,
    pub input_token_transformation_version: u16,
    pub input_cst_transformation_version: u16,
    pub anchors: Seq<AnchorDeclarationView>,
    pub aliases: Seq<AliasBindingView>,
}

impl View for AnchorAliasSource {
    type V = AnchorAliasSourceView;

    closed spec fn view(&self) -> AnchorAliasSourceView {
        AnchorAliasSourceView {
            profile_version: self.profile_version,
            transformation_version: self.transformation_version,
            source_len_bytes: self.source_len_bytes,
            input_token_transformation_version: self.input_token_transformation_version,
            input_cst_transformation_version: self.input_cst_transformation_version,
            anchors: anchor_declaration_views_spec(self.anchors@),
            aliases: alias_binding_views_spec(self.aliases@),
        }
    }
}

impl AnchorAliasSource {
    fn new(
        profile_version: u16,
        source_len_bytes: u64,
        input_token_transformation_version: u16,
        input_cst_transformation_version: u16,
        anchors: Vec<AnchorDeclaration>,
        aliases: Vec<AliasBinding>,
    ) -> (source: Self)
        ensures
            source@ == (AnchorAliasSourceView {
                profile_version,
                transformation_version: ANCHOR_ALIAS_RESOLUTION_VERSION,
                source_len_bytes,
                input_token_transformation_version,
                input_cst_transformation_version,
                anchors: anchor_declaration_views_spec(anchors@),
                aliases: alias_binding_views_spec(aliases@),
            }),
    {
        Self {
            profile_version,
            transformation_version: ANCHOR_ALIAS_RESOLUTION_VERSION,
            source_len_bytes,
            input_token_transformation_version,
            input_cst_transformation_version,
            anchors,
            aliases,
        }
    }

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

    pub fn anchors(&self) -> (values: &[AnchorDeclaration])
        ensures
            anchor_declaration_views_spec(values@) == self@.anchors,
    {
        self.anchors.as_slice()
    }

    pub fn aliases(&self) -> (values: &[AliasBinding])
        ensures
            alias_binding_views_spec(values@) == self@.aliases,
    {
        self.aliases.as_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum AnchorAliasErrorKind {
    InputCompletedTokenMismatch,
    InputCstMismatch,
    InvalidDocumentRange,
    InvalidSyntaxOwner,
    InvalidAnchorToken,
    InvalidAliasToken,
    UnresolvedAlias,
    AnchorLimitExceeded,
    AliasLimitExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorAliasError {
    kind: AnchorAliasErrorKind,
    byte_offset: u64,
}

#[verifier::ext_equal]
pub struct AnchorAliasErrorView {
    pub kind: AnchorAliasErrorKind,
    pub byte_offset: u64,
}

impl View for AnchorAliasError {
    type V = AnchorAliasErrorView;

    closed spec fn view(&self) -> AnchorAliasErrorView {
        AnchorAliasErrorView { kind: self.kind, byte_offset: self.byte_offset }
    }
}

impl AnchorAliasError {
    fn at(kind: AnchorAliasErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (AnchorAliasErrorView { kind, byte_offset }),
    {
        Self { kind, byte_offset }
    }

    pub fn kind(&self) -> (kind: AnchorAliasErrorKind)
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

pub open spec fn atom_code_points_spec(atoms: Seq<LexicalAtomView>) -> Seq<u32> {
    Seq::new(atoms.len(), |index: int| atoms[index].code_point)
}

pub open spec fn anchor_name_ranges_match_spec(
    atoms: Seq<u32>,
    left_start: u64,
    left_end: u64,
    right_start: u64,
    right_end: u64,
) -> bool {
    left_start < left_end <= atoms.len() && right_start < right_end <= atoms.len() && left_end
        - left_start == right_end - right_start && forall|offset: int|
        0 <= offset < left_end - left_start ==> #[trigger] atoms[left_start as int + offset]
            == atoms[right_start as int + offset]
}

pub open spec fn latest_matching_anchor_spec(
    atoms: Seq<u32>,
    anchors: Seq<AnchorDeclarationView>,
    document_index: u64,
    before_token_index: u64,
    alias_start_atom_index: u64,
    alias_end_atom_index: u64,
    fuel: nat,
) -> Option<int>
    decreases fuel,
{
    if fuel == 0 || fuel > anchors.len() {
        None
    } else {
        let index = fuel - 1;
        let anchor = anchors[index as int];
        if anchor.document_index == document_index && anchor.token_index < before_token_index
            && anchor_name_ranges_match_spec(
            atoms,
            anchor.name_start_atom_index,
            anchor.name_end_atom_index,
            alias_start_atom_index,
            alias_end_atom_index,
        ) {
            Some(index as int)
        } else {
            latest_matching_anchor_spec(
                atoms,
                anchors,
                document_index,
                before_token_index,
                alias_start_atom_index,
                alias_end_atom_index,
                (fuel - 1) as nat,
            )
        }
    }
}

#[verifier::ext_equal]
pub struct AnchorAliasBuildView {
    pub anchors: Seq<AnchorDeclarationView>,
    pub aliases: Seq<AliasBindingView>,
}

pub open spec fn anchor_alias_part_valid_spec(
    atoms: Seq<LexicalAtomView>,
    token: CompletedTokenView,
    part: CompletedTokenPartView,
    expected_kind: CompletedTokenPartKind,
) -> bool {
    part.kind == expected_kind && part.start_atom_index < part.end_atom_index && part.end_atom_index
        <= atoms.len() && token.start_atom_index <= part.start_atom_index && part.end_atom_index
        <= token.end_atom_index && token.byte_start <= part.byte_start && part.byte_end
        <= token.byte_end && part.byte_start
        == atoms[part.start_atom_index as int].span.start.byte_offset && part.byte_end == atoms[(
    part.end_atom_index - 1) as int].span.end.byte_offset
}

pub open spec fn process_anchor_alias_token_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
    nodes: Seq<crate::cst::CstNodeView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    document_index: u64,
    token_index: u64,
    state: AnchorAliasBuildView,
    limits: AnchorAliasLimitsView,
) -> Result<AnchorAliasBuildView, AnchorAliasErrorView> {
    if token_index >= tokens.len() {
        Err(
            AnchorAliasErrorView {
                kind: AnchorAliasErrorKind::InvalidDocumentRange,
                byte_offset: 0,
            },
        )
    } else {
        let token = tokens[token_index as int];
        if token.kind == CompletedTokenKind::AnchorProperty {
            if token_index >= owners.len() || owners[token_index as int].is_none() {
                Err(
                    AnchorAliasErrorView {
                        kind: AnchorAliasErrorKind::InvalidSyntaxOwner,
                        byte_offset: token.byte_start,
                    },
                )
            } else {
                let owner = owners[token_index as int].unwrap();
                if owner.token_index != token_index || owner.kind
                    != CstSyntaxOwnerKind::NodeProperty || owner.record_index >= nodes.len()
                    || nodes[owner.record_index as int].anchor_property_token != Some(token_index) {
                    Err(
                        AnchorAliasErrorView {
                            kind: AnchorAliasErrorKind::InvalidSyntaxOwner,
                            byte_offset: token.byte_start,
                        },
                    )
                } else if token.parts.len() != 1 || !anchor_alias_part_valid_spec(
                    atoms,
                    token,
                    token.parts[0],
                    CompletedTokenPartKind::AnchorName,
                ) {
                    Err(
                        AnchorAliasErrorView {
                            kind: AnchorAliasErrorKind::InvalidAnchorToken,
                            byte_offset: token.byte_start,
                        },
                    )
                } else if state.anchors.len() >= effective_anchor_limit_spec(limits) {
                    Err(
                        AnchorAliasErrorView {
                            kind: AnchorAliasErrorKind::AnchorLimitExceeded,
                            byte_offset: token.byte_start,
                        },
                    )
                } else {
                    let part = token.parts[0];
                    Ok(
                        AnchorAliasBuildView {
                            anchors: state.anchors.push(
                                AnchorDeclarationView {
                                    document_index,
                                    node_index: owner.record_index,
                                    token_index,
                                    name_start_atom_index: part.start_atom_index,
                                    name_end_atom_index: part.end_atom_index,
                                    name_byte_start: part.byte_start,
                                    name_byte_end: part.byte_end,
                                },
                            ),
                            aliases: state.aliases,
                        },
                    )
                }
            }
        } else if token.kind == CompletedTokenKind::Alias {
            if token_index >= owners.len() || owners[token_index as int].is_none() {
                Err(
                    AnchorAliasErrorView {
                        kind: AnchorAliasErrorKind::InvalidSyntaxOwner,
                        byte_offset: token.byte_start,
                    },
                )
            } else {
                let owner = owners[token_index as int].unwrap();
                if owner.token_index != token_index || owner.kind != CstSyntaxOwnerKind::NodeContent
                    || owner.record_index >= nodes.len()
                    || nodes[owner.record_index as int].scalar_or_alias_token != Some(token_index) {
                    Err(
                        AnchorAliasErrorView {
                            kind: AnchorAliasErrorKind::InvalidSyntaxOwner,
                            byte_offset: token.byte_start,
                        },
                    )
                } else if token.parts.len() != 1 || !anchor_alias_part_valid_spec(
                    atoms,
                    token,
                    token.parts[0],
                    CompletedTokenPartKind::AliasName,
                ) {
                    Err(
                        AnchorAliasErrorView {
                            kind: AnchorAliasErrorKind::InvalidAliasToken,
                            byte_offset: token.byte_start,
                        },
                    )
                } else {
                    let part = token.parts[0];
                    let target = latest_matching_anchor_spec(
                        atom_code_points_spec(atoms),
                        state.anchors,
                        document_index,
                        token_index,
                        part.start_atom_index,
                        part.end_atom_index,
                        state.anchors.len() as nat,
                    );
                    if target.is_none() {
                        Err(
                            AnchorAliasErrorView {
                                kind: AnchorAliasErrorKind::UnresolvedAlias,
                                byte_offset: token.byte_start,
                            },
                        )
                    } else if state.aliases.len() >= effective_alias_limit_spec(limits) {
                        Err(
                            AnchorAliasErrorView {
                                kind: AnchorAliasErrorKind::AliasLimitExceeded,
                                byte_offset: token.byte_start,
                            },
                        )
                    } else {
                        let target_index = target.unwrap();
                        Ok(
                            AnchorAliasBuildView {
                                anchors: state.anchors,
                                aliases: state.aliases.push(
                                    AliasBindingView {
                                        document_index,
                                        alias_node_index: owner.record_index,
                                        alias_token_index: token_index,
                                        name_start_atom_index: part.start_atom_index,
                                        name_end_atom_index: part.end_atom_index,
                                        name_byte_start: part.byte_start,
                                        name_byte_end: part.byte_end,
                                        target_anchor_index: target_index as u64,
                                        target_node_index: state.anchors[target_index].node_index,
                                    },
                                ),
                            },
                        )
                    }
                }
            }
        } else {
            Ok(state)
        }
    }
}

pub open spec fn scan_anchor_alias_document_tokens_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
    nodes: Seq<crate::cst::CstNodeView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    document_index: u64,
    token_index: u64,
    token_end: u64,
    fuel: nat,
    state: AnchorAliasBuildView,
    limits: AnchorAliasLimitsView,
) -> Result<AnchorAliasBuildView, AnchorAliasErrorView>
    decreases fuel,
{
    if fuel == 0 || token_index >= token_end {
        Ok(state)
    } else {
        match process_anchor_alias_token_spec(
            atoms,
            tokens,
            nodes,
            owners,
            document_index,
            token_index,
            state,
            limits,
        ) {
            Err(error) => Err(error),
            Ok(next) => scan_anchor_alias_document_tokens_spec(
                atoms,
                tokens,
                nodes,
                owners,
                document_index,
                (token_index + 1) as u64,
                token_end,
                (fuel - 1) as nat,
                next,
                limits,
            ),
        }
    }
}

pub open spec fn scan_anchor_alias_documents_spec(
    atoms: Seq<LexicalAtomView>,
    tokens: Seq<CompletedTokenView>,
    documents: Seq<CstDocumentView>,
    nodes: Seq<crate::cst::CstNodeView>,
    owners: Seq<Option<CstSyntaxOwnerView>>,
    document_index: u64,
    fuel: nat,
    state: AnchorAliasBuildView,
    limits: AnchorAliasLimitsView,
) -> Result<AnchorAliasBuildView, AnchorAliasErrorView>
    decreases fuel,
{
    if fuel == 0 || document_index >= documents.len() {
        Ok(state)
    } else {
        let document = documents[document_index as int];
        if document.root_token_start > document.root_token_end || document.root_token_end
            > tokens.len() {
            Err(
                AnchorAliasErrorView {
                    kind: AnchorAliasErrorKind::InvalidDocumentRange,
                    byte_offset: document.byte_start,
                },
            )
        } else {
            match scan_anchor_alias_document_tokens_spec(
                atoms,
                tokens,
                nodes,
                owners,
                document_index,
                document.root_token_start,
                document.root_token_end,
                (document.root_token_end - document.root_token_start) as nat,
                state,
                limits,
            ) {
                Err(error) => Err(error),
                Ok(next) => scan_anchor_alias_documents_spec(
                    atoms,
                    tokens,
                    documents,
                    nodes,
                    owners,
                    (document_index + 1) as u64,
                    (fuel - 1) as nat,
                    next,
                    limits,
                ),
            }
        }
    }
}

pub open spec fn resolve_profile1_anchor_aliases_spec(
    atomized: AtomizedSourceView,
    completed: CompletedTokenSourceView,
    cst: CstSourceView,
    limits: AnchorAliasLimitsView,
) -> Result<AnchorAliasSourceView, AnchorAliasErrorView> {
    if completed.profile_version != atomized.profile_version
        || completed.input_transformation_version != atomized.transformation_version
        || completed.transformation_version != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION
        || completed.source_len_bytes != atomized.source_len_bytes || completed.bom_bytes
        != atomized.bom_bytes || completed.input_atom_count != atomized.atoms.len() {
        Err(
            AnchorAliasErrorView {
                kind: AnchorAliasErrorKind::InputCompletedTokenMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else if cst.profile_version != completed.profile_version
        || cst.input_token_transformation_version != completed.transformation_version
        || cst.transformation_version != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes != completed.source_len_bytes || cst.input_token_count
        != completed.tokens.len() || cst.syntax_owners.len() != completed.tokens.len() {
        Err(
            AnchorAliasErrorView {
                kind: AnchorAliasErrorKind::InputCstMismatch,
                byte_offset: atomized.bom_bytes,
            },
        )
    } else {
        match scan_anchor_alias_documents_spec(
            atomized.atoms,
            completed.tokens,
            cst.documents,
            cst.nodes,
            cst.syntax_owners,
            0,
            cst.documents.len() as nat,
            AnchorAliasBuildView { anchors: Seq::empty(), aliases: Seq::empty() },
            limits,
        ) {
            Err(error) => Err(error),
            Ok(state) => Ok(
                AnchorAliasSourceView {
                    profile_version: atomized.profile_version,
                    transformation_version: ANCHOR_ALIAS_RESOLUTION_VERSION,
                    source_len_bytes: atomized.source_len_bytes,
                    input_token_transformation_version: completed.transformation_version,
                    input_cst_transformation_version: cst.transformation_version,
                    anchors: state.anchors,
                    aliases: state.aliases,
                },
            ),
        }
    }
}

fn anchor_name_ranges_match(
    atoms: &[LexicalAtom],
    left_start_atom: u64,
    left_end_atom: u64,
    right_start_atom: u64,
    right_end_atom: u64,
) -> (matches: bool)
    ensures
        matches == anchor_name_ranges_match_spec(
            atom_code_points_spec(crate::atom::lexical_atom_views_spec(atoms@)),
            left_start_atom,
            left_end_atom,
            right_start_atom,
            right_end_atom,
        ),
{
    if left_start_atom >= left_end_atom || left_end_atom > atoms.len() as u64 || right_start_atom
        >= right_end_atom || right_end_atom > atoms.len() as u64 {
        proof {
            reveal(anchor_name_ranges_match_spec);
            reveal(atom_code_points_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return false;
    }
    let left_start = left_start_atom as usize;
    let right_start = right_start_atom as usize;
    let left_len = (left_end_atom - left_start_atom) as usize;
    let right_len = (right_end_atom - right_start_atom) as usize;
    assert(left_len as u64 == left_end_atom - left_start_atom);
    assert(right_len as u64 == right_end_atom - right_start_atom);
    if left_len != right_len {
        proof {
            reveal(anchor_name_ranges_match_spec);
            reveal(atom_code_points_spec);
            reveal(crate::atom::lexical_atom_views_spec);
        }
        return false;
    }
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost points = atom_code_points_spec(atom_views);
    let mut offset = 0usize;
    while offset < left_len
        invariant
            atoms@.len() <= usize::MAX,
            left_start_atom as int == left_start as int,
            right_start_atom as int == right_start as int,
            left_start + left_len <= atoms@.len(),
            right_start + right_len <= atoms@.len(),
            left_len == right_len,
            left_len as u64 == left_end_atom - left_start_atom,
            right_len as u64 == right_end_atom - right_start_atom,
            offset <= left_len,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            points == atom_code_points_spec(atom_views),
            forall|prior: int|
                0 <= prior < offset ==> #[trigger] points[left_start as int + prior]
                    == points[right_start as int + prior],
        decreases left_len - offset,
    {
        assert(left_start <= usize::MAX - offset);
        assert(right_start <= usize::MAX - offset);
        assert(left_start + offset < atoms@.len());
        assert(right_start + offset < atoms@.len());
        assert((left_start + offset) as int == left_start as int + offset as int);
        assert((right_start + offset) as int == right_start as int + offset as int);
        assert(atom_views[(left_start + offset) as int] == atoms[(left_start + offset) as int]@)
            by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        assert(atom_views[(right_start + offset) as int] == atoms[(right_start + offset) as int]@)
            by {
            reveal(crate::atom::lexical_atom_views_spec);
        }
        proof {
            reveal(atom_code_points_spec);
            assert(points[left_start as int + offset as int] == atom_views[(left_start
                + offset) as int].code_point);
            assert(points[right_start as int + offset as int] == atom_views[(right_start
                + offset) as int].code_point);
        }
        if atoms[left_start + offset].code_point() != atoms[right_start + offset].code_point() {
            proof {
                reveal(anchor_name_ranges_match_spec);
                assert(points[left_start as int + offset as int] != points[right_start as int
                    + offset as int]);
                assert(!(forall|prior: int|
                    0 <= prior < left_end_atom - left_start_atom
                        ==> #[trigger] points[left_start_atom as int + prior]
                        == points[right_start_atom as int + prior])) by {
                    assert(0 <= offset as int);
                    assert(offset < left_len);
                    assert(left_len as int == left_end_atom - left_start_atom);
                    assert(left_start_atom as int + offset as int == left_start as int
                        + offset as int);
                    assert(right_start_atom as int + offset as int == right_start as int
                        + offset as int);
                }
            }
            return false;
        }
        offset += 1;
    }
    proof {
        reveal(anchor_name_ranges_match_spec);
    }
    true
}

fn latest_matching_anchor(
    atoms: &[LexicalAtom],
    anchors: &[AnchorDeclaration],
    document_index: u64,
    before_token_index: u64,
    alias_start_atom_index: u64,
    alias_end_atom_index: u64,
) -> (found: Option<usize>)
    ensures
        latest_matching_anchor_spec(
            atom_code_points_spec(crate::atom::lexical_atom_views_spec(atoms@)),
            anchor_declaration_views_spec(anchors@),
            document_index,
            before_token_index,
            alias_start_atom_index,
            alias_end_atom_index,
            anchors@.len() as nat,
        ) == match found {
            Some(index) => Some(index as int),
            None => None,
        },
        match found {
            Some(index) => index < anchors@.len(),
            None => true,
        },
{
    let ghost atom_points = atom_code_points_spec(crate::atom::lexical_atom_views_spec(atoms@));
    let ghost anchor_views = anchor_declaration_views_spec(anchors@);
    let ghost expected = latest_matching_anchor_spec(
        atom_points,
        anchor_views,
        document_index,
        before_token_index,
        alias_start_atom_index,
        alias_end_atom_index,
        anchors@.len() as nat,
    );
    let mut fuel = anchors.len();
    while fuel > 0
        invariant
            fuel <= anchors@.len(),
            anchor_views == anchor_declaration_views_spec(anchors@),
            atom_points == atom_code_points_spec(crate::atom::lexical_atom_views_spec(atoms@)),
            expected == latest_matching_anchor_spec(
                atom_points,
                anchor_views,
                document_index,
                before_token_index,
                alias_start_atom_index,
                alias_end_atom_index,
                anchors@.len() as nat,
            ),
            expected == latest_matching_anchor_spec(
                atom_points,
                anchor_views,
                document_index,
                before_token_index,
                alias_start_atom_index,
                alias_end_atom_index,
                fuel as nat,
            ),
        decreases fuel,
    {
        let index = fuel - 1;
        assert(anchor_views[index as int] == anchors[index as int]@) by {
            reveal(anchor_declaration_views_spec);
        }
        let anchor = &anchors[index];
        if anchor.document_index() == document_index && anchor.token_index() < before_token_index
            && anchor_name_ranges_match(
            atoms,
            anchor.name_start_atom_index(),
            anchor.name_end_atom_index(),
            alias_start_atom_index,
            alias_end_atom_index,
        ) {
            proof {
                reveal(latest_matching_anchor_spec);
                assert(expected == Some(index as int));
            }
            return Some(index);
        }
        proof {
            reveal(latest_matching_anchor_spec);
        }
        fuel -= 1;
    }
    proof {
        reveal(latest_matching_anchor_spec);
    }
    None
}

fn anchor_alias_part_valid(
    atoms: &[LexicalAtom],
    token: &CompletedToken,
    part: &CompletedTokenPart,
    expected_kind: CompletedTokenPartKind,
) -> (valid: bool)
    ensures
        valid == anchor_alias_part_valid_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            token@,
            part@,
            expected_kind,
        ),
{
    let start = part.start_atom_index();
    let end = part.end_atom_index();
    let valid = part.kind() == expected_kind && start < end && end <= atoms.len() as u64
        && token.start_atom_index() <= start && end <= token.end_atom_index() && token.byte_start()
        <= part.byte_start() && part.byte_end() <= token.byte_end() && part.byte_start()
        == atoms[start as usize].span().start().byte_offset() && part.byte_end() == atoms[(end
        - 1) as usize].span().end().byte_offset();
    proof {
        reveal(anchor_alias_part_valid_spec);
        reveal(crate::atom::lexical_atom_views_spec);
    }
    valid
}

#[expect(clippy::too_many_arguments, reason = "independent proof inputs remain explicit in the executable-to-spec contract")]  // The verified transition names every authenticated input.
fn process_anchor_alias_token(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    nodes: &[crate::cst::CstNode],
    owners: &[Option<CstSyntaxOwner>],
    document_index: u64,
    token_index: u64,
    anchors: &mut Vec<AnchorDeclaration>,
    aliases: &mut Vec<AliasBinding>,
    limits: AnchorAliasLimits,
) -> (result: Result<(), AnchorAliasError>)
    ensures
        process_anchor_alias_token_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::token::completed_token_views_spec(tokens@),
            crate::cst::cst_node_views_spec(nodes@),
            crate::cst::cst_syntax_owner_views_spec(owners@),
            document_index,
            token_index,
            AnchorAliasBuildView {
                anchors: anchor_declaration_views_spec(old(anchors)@),
                aliases: alias_binding_views_spec(old(aliases)@),
            },
            limits@,
        ) == match result {
            Ok(()) => Ok(
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(final(anchors)@),
                    aliases: alias_binding_views_spec(final(aliases)@),
                },
            ),
            Err(error) => Err(error@),
        },
        result.is_err() ==> final(anchors)@ == old(anchors)@ && final(aliases)@ == old(aliases)@,
{
    let ghost original_anchors = anchors@;
    let ghost original_aliases = aliases@;
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost node_views = crate::cst::cst_node_views_spec(nodes@);
    let ghost owner_views = crate::cst::cst_syntax_owner_views_spec(owners@);
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::cst::cst_node_views_spec);
        reveal(crate::cst::cst_syntax_owner_views_spec);
        assert(token_views.len() == tokens@.len());
        assert(node_views.len() == nodes@.len());
        assert(owner_views.len() == owners@.len());
    }
    if token_index >= tokens.len() as u64 {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::InvalidDocumentRange, 0);
        proof {
            reveal(process_anchor_alias_token_spec);
            crate::token::lemma_completed_token_views_len(tokens@);
        }
        return Err(error);
    }
    let runtime_index = token_index as usize;
    let token = &tokens[runtime_index];
    let kind = token.kind();
    let byte_start = token.byte_start();
    proof {
        crate::token::lemma_completed_token_view_at(tokens@, runtime_index as int);
        assert(token_views[runtime_index as int] == token@);
    }
    if kind != CompletedTokenKind::AnchorProperty && kind != CompletedTokenKind::Alias {
        proof {
            reveal(process_anchor_alias_token_spec);
        }
        return Ok(());
    }
    if token_index >= owners.len() as u64 || owners[runtime_index].is_none() {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::InvalidSyntaxOwner, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
            reveal(crate::cst::cst_syntax_owner_views_spec);
        }
        return Err(error);
    }
    let owner = owners[runtime_index].as_ref().unwrap();
    proof {
        crate::cst::lemma_cst_syntax_owner_view_at(owners@, runtime_index as int);
        assert(owner_views[runtime_index as int] == Some(owner@));
    }
    let node_index = owner.record_index();
    let expected_owner_kind = if kind == CompletedTokenKind::AnchorProperty {
        CstSyntaxOwnerKind::NodeProperty
    } else {
        CstSyntaxOwnerKind::NodeContent
    };
    if owner.token_index() != token_index || owner.kind() != expected_owner_kind || node_index
        >= nodes.len() as u64 {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::InvalidSyntaxOwner, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
        }
        return Err(error);
    }
    let node = &nodes[node_index as usize];
    proof {
        crate::cst::lemma_cst_node_view_at(nodes@, node_index as int);
        assert(node_views[node_index as int] == node@);
    }
    if (kind == CompletedTokenKind::AnchorProperty && node.anchor_property_token() != Some(
        token_index,
    )) || (kind == CompletedTokenKind::Alias && node.scalar_or_alias_token() != Some(token_index)) {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::InvalidSyntaxOwner, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
        }
        return Err(error);
    }
    let parts = token.parts();
    proof {
        crate::token::lemma_completed_token_part_views_len(parts@);
    }
    if parts.len() != 1 {
        let error_kind = if kind == CompletedTokenKind::AnchorProperty {
            AnchorAliasErrorKind::InvalidAnchorToken
        } else {
            AnchorAliasErrorKind::InvalidAliasToken
        };
        let error = AnchorAliasError::at(error_kind, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
        }
        return Err(error);
    }
    proof {
        crate::token::lemma_completed_token_part_view_at(parts@, 0);
        assert(token@.parts[0] == parts@[0]@);
    }
    let expected_part_kind = if kind == CompletedTokenKind::AnchorProperty {
        CompletedTokenPartKind::AnchorName
    } else {
        CompletedTokenPartKind::AliasName
    };
    if !anchor_alias_part_valid(atoms, token, &parts[0], expected_part_kind) {
        let error_kind = if kind == CompletedTokenKind::AnchorProperty {
            AnchorAliasErrorKind::InvalidAnchorToken
        } else {
            AnchorAliasErrorKind::InvalidAliasToken
        };
        let error = AnchorAliasError::at(error_kind, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
        }
        return Err(error);
    }
    if kind == CompletedTokenKind::AnchorProperty {
        let effective_limit = if limits.max_anchors < MAX_PROFILE1_ANCHOR_DECLARATIONS {
            limits.max_anchors
        } else {
            MAX_PROFILE1_ANCHOR_DECLARATIONS
        };
        if anchors.len() as u64 >= effective_limit {
            let error = AnchorAliasError::at(AnchorAliasErrorKind::AnchorLimitExceeded, byte_start);
            proof {
                reveal(process_anchor_alias_token_spec);
                reveal(effective_anchor_limit_spec);
            }
            return Err(error);
        }
        let declaration = AnchorDeclaration::new(
            document_index,
            node_index,
            token_index,
            &parts[0],
        );
        proof {
            lemma_anchor_declaration_views_push(anchors@, declaration);
            reveal(process_anchor_alias_token_spec);
            reveal(effective_anchor_limit_spec);
        }
        anchors.push(declaration);
        return Ok(());
    }
    let alias_start = parts[0].start_atom_index();
    let alias_end = parts[0].end_atom_index();
    let target = latest_matching_anchor(
        atoms,
        anchors.as_slice(),
        document_index,
        token_index,
        alias_start,
        alias_end,
    );
    if target.is_none() {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::UnresolvedAlias, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
        }
        return Err(error);
    }
    let effective_limit = if limits.max_aliases < MAX_PROFILE1_ALIAS_BINDINGS {
        limits.max_aliases
    } else {
        MAX_PROFILE1_ALIAS_BINDINGS
    };
    if aliases.len() as u64 >= effective_limit {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::AliasLimitExceeded, byte_start);
        proof {
            reveal(process_anchor_alias_token_spec);
            reveal(effective_alias_limit_spec);
        }
        return Err(error);
    }
    let target_index = target.unwrap();
    let target_node_index = anchors[target_index].node_index();
    let binding = AliasBinding::new(
        document_index,
        node_index,
        token_index,
        &parts[0],
        target_index as u64,
        target_node_index,
    );
    proof {
        lemma_alias_binding_views_push(aliases@, binding);
        reveal(process_anchor_alias_token_spec);
        reveal(effective_alias_limit_spec);
        assert(anchor_declaration_views_spec(anchors@)[target_index as int]
            == anchors[target_index as int]@) by {
            reveal(anchor_declaration_views_spec);
        }
    }
    aliases.push(binding);
    Ok(())
}

#[expect(clippy::too_many_arguments, reason = "independent proof inputs remain explicit in the executable-to-spec contract")]  // Keeps the loop contract explicit and independently reusable.
fn scan_anchor_alias_document_tokens(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    nodes: &[crate::cst::CstNode],
    owners: &[Option<CstSyntaxOwner>],
    document_index: u64,
    start: usize,
    end: usize,
    anchors: &mut Vec<AnchorDeclaration>,
    aliases: &mut Vec<AliasBinding>,
    limits: AnchorAliasLimits,
) -> (result: Result<(), AnchorAliasError>)
    requires
        start <= end <= tokens@.len(),
        end <= u64::MAX,
    ensures
        scan_anchor_alias_document_tokens_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::token::completed_token_views_spec(tokens@),
            crate::cst::cst_node_views_spec(nodes@),
            crate::cst::cst_syntax_owner_views_spec(owners@),
            document_index,
            start as u64,
            end as u64,
            (end - start) as nat,
            AnchorAliasBuildView {
                anchors: anchor_declaration_views_spec(old(anchors)@),
                aliases: alias_binding_views_spec(old(aliases)@),
            },
            limits@,
        ) == match result {
            Ok(()) => Ok(
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(final(anchors)@),
                    aliases: alias_binding_views_spec(final(aliases)@),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost node_views = crate::cst::cst_node_views_spec(nodes@);
    let ghost owner_views = crate::cst::cst_syntax_owner_views_spec(owners@);
    let ghost expected = scan_anchor_alias_document_tokens_spec(
        atom_views,
        token_views,
        node_views,
        owner_views,
        document_index,
        start as u64,
        end as u64,
        (end - start) as nat,
        AnchorAliasBuildView {
            anchors: anchor_declaration_views_spec(anchors@),
            aliases: alias_binding_views_spec(aliases@),
        },
        limits@,
    );
    let mut index = start;
    let mut _fuel = end - start;
    while index < end
        invariant
            start <= index <= end <= tokens@.len(),
            end <= u64::MAX,
            _fuel == end - index,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            token_views == crate::token::completed_token_views_spec(tokens@),
            node_views == crate::cst::cst_node_views_spec(nodes@),
            owner_views == crate::cst::cst_syntax_owner_views_spec(owners@),
            expected == scan_anchor_alias_document_tokens_spec(
                atom_views,
                token_views,
                node_views,
                owner_views,
                document_index,
                start as u64,
                end as u64,
                (end - start) as nat,
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(old(anchors)@),
                    aliases: alias_binding_views_spec(old(aliases)@),
                },
                limits@,
            ),
            expected == scan_anchor_alias_document_tokens_spec(
                atom_views,
                token_views,
                node_views,
                owner_views,
                document_index,
                index as u64,
                end as u64,
                _fuel as nat,
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(anchors@),
                    aliases: alias_binding_views_spec(aliases@),
                },
                limits@,
            ),
        decreases end - index,
    {
        let step = process_anchor_alias_token(
            atoms,
            tokens,
            nodes,
            owners,
            document_index,
            index as u64,
            anchors,
            aliases,
            limits,
        );
        if let Err(error) = step {
            proof {
                reveal(scan_anchor_alias_document_tokens_spec);
            }
            return Err(error);
        }
        proof {
            reveal(scan_anchor_alias_document_tokens_spec);
        }
        index += 1;
        _fuel -= 1;
    }
    proof {
        reveal(scan_anchor_alias_document_tokens_spec);
    }
    Ok(())
}

#[expect(clippy::too_many_arguments, reason = "independent proof inputs remain explicit in the executable-to-spec contract")]  // Mirrors the complete pure document-scan state.
fn scan_anchor_alias_documents(
    atoms: &[LexicalAtom],
    tokens: &[CompletedToken],
    documents: &[CstDocument],
    nodes: &[crate::cst::CstNode],
    owners: &[Option<CstSyntaxOwner>],
    anchors: &mut Vec<AnchorDeclaration>,
    aliases: &mut Vec<AliasBinding>,
    limits: AnchorAliasLimits,
) -> (result: Result<(), AnchorAliasError>)
    ensures
        scan_anchor_alias_documents_spec(
            crate::atom::lexical_atom_views_spec(atoms@),
            crate::token::completed_token_views_spec(tokens@),
            crate::cst::cst_document_views_spec(documents@),
            crate::cst::cst_node_views_spec(nodes@),
            crate::cst::cst_syntax_owner_views_spec(owners@),
            0,
            documents@.len() as nat,
            AnchorAliasBuildView {
                anchors: anchor_declaration_views_spec(old(anchors)@),
                aliases: alias_binding_views_spec(old(aliases)@),
            },
            limits@,
        ) == match result {
            Ok(()) => Ok(
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(final(anchors)@),
                    aliases: alias_binding_views_spec(final(aliases)@),
                },
            ),
            Err(error) => Err(error@),
        },
{
    let ghost atom_views = crate::atom::lexical_atom_views_spec(atoms@);
    let ghost token_views = crate::token::completed_token_views_spec(tokens@);
    let ghost document_views = crate::cst::cst_document_views_spec(documents@);
    let ghost node_views = crate::cst::cst_node_views_spec(nodes@);
    let ghost owner_views = crate::cst::cst_syntax_owner_views_spec(owners@);
    let ghost expected = scan_anchor_alias_documents_spec(
        atom_views,
        token_views,
        document_views,
        node_views,
        owner_views,
        0,
        documents@.len() as nat,
        AnchorAliasBuildView {
            anchors: anchor_declaration_views_spec(anchors@),
            aliases: alias_binding_views_spec(aliases@),
        },
        limits@,
    );
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::cst::cst_document_views_spec);
    }
    let mut document_index = 0usize;
    let mut _fuel = documents.len();
    while document_index < documents.len()
        invariant
            document_index <= documents@.len(),
            documents@.len() <= u64::MAX,
            _fuel == documents@.len() - document_index,
            atom_views == crate::atom::lexical_atom_views_spec(atoms@),
            token_views == crate::token::completed_token_views_spec(tokens@),
            token_views.len() == tokens@.len(),
            document_views == crate::cst::cst_document_views_spec(documents@),
            document_views.len() == documents@.len(),
            node_views == crate::cst::cst_node_views_spec(nodes@),
            owner_views == crate::cst::cst_syntax_owner_views_spec(owners@),
            expected == scan_anchor_alias_documents_spec(
                atom_views,
                token_views,
                document_views,
                node_views,
                owner_views,
                0,
                documents@.len() as nat,
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(old(anchors)@),
                    aliases: alias_binding_views_spec(old(aliases)@),
                },
                limits@,
            ),
            expected == scan_anchor_alias_documents_spec(
                atom_views,
                token_views,
                document_views,
                node_views,
                owner_views,
                document_index as u64,
                _fuel as nat,
                AnchorAliasBuildView {
                    anchors: anchor_declaration_views_spec(anchors@),
                    aliases: alias_binding_views_spec(aliases@),
                },
                limits@,
            ),
        decreases documents.len() - document_index,
    {
        assert(document_views[document_index as int] == documents[document_index as int]@) by {
            crate::cst::lemma_cst_document_view_at(documents@, document_index as int);
        }
        let document = &documents[document_index];
        let start = document.root_token_start();
        let end = document.root_token_end();
        if start > end || end > tokens.len() as u64 {
            let error = AnchorAliasError::at(
                AnchorAliasErrorKind::InvalidDocumentRange,
                document.byte_start(),
            );
            proof {
                reveal(scan_anchor_alias_documents_spec);
            }
            return Err(error);
        }
        assert(start <= usize::MAX as u64);
        assert(end <= usize::MAX as u64);
        assert((start as usize) as u64 == start);
        assert((end as usize) as u64 == end);
        let scan = scan_anchor_alias_document_tokens(
            atoms,
            tokens,
            nodes,
            owners,
            document_index as u64,
            start as usize,
            end as usize,
            anchors,
            aliases,
            limits,
        );
        if let Err(error) = scan {
            proof {
                reveal(scan_anchor_alias_documents_spec);
            }
            return Err(error);
        }
        proof {
            reveal(scan_anchor_alias_documents_spec);
        }
        assert(document_index < usize::MAX);
        assert((document_index + 1) as u64 == document_index as u64 + 1);
        document_index += 1;
        _fuel -= 1;
    }
    proof {
        reveal(scan_anchor_alias_documents_spec);
    }
    Ok(())
}

pub fn resolve_profile1_anchor_aliases(
    atomized: &AtomizedSource,
    completed: &CompletedTokenSource,
    cst: &CstSource,
    limits: AnchorAliasLimits,
) -> (result: Result<AnchorAliasSource, AnchorAliasError>)
    ensures
        resolve_profile1_anchor_aliases_spec(atomized@, completed@, cst@, limits@) == match result {
            Ok(source) => Ok(source@),
            Err(error) => Err(error@),
        },
{
    let bom_bytes = atomized.bom_bytes();
    let atoms = atomized.atoms();
    let tokens = completed.tokens();
    let documents = cst.documents();
    let nodes = cst.nodes();
    let owners = cst.syntax_owners();
    proof {
        crate::token::lemma_completed_token_views_len(tokens@);
        reveal(crate::atom::lexical_atom_views_spec);
        reveal(crate::cst::cst_document_views_spec);
        reveal(crate::cst::cst_node_views_spec);
        reveal(crate::cst::cst_syntax_owner_views_spec);
        assert(atomized@.atoms == crate::atom::lexical_atom_views_spec(atoms@));
        assert(completed@.tokens == crate::token::completed_token_views_spec(tokens@));
        assert(cst@.documents == crate::cst::cst_document_views_spec(documents@));
        assert(cst@.nodes == crate::cst::cst_node_views_spec(nodes@));
        assert(cst@.syntax_owners == crate::cst::cst_syntax_owner_views_spec(owners@));
        assert(atomized@.atoms.len() == atoms@.len());
        assert(completed@.tokens.len() == tokens@.len());
        assert(cst@.documents.len() == documents@.len());
        assert(cst@.nodes.len() == nodes@.len());
        assert(cst@.syntax_owners.len() == owners@.len());
    }
    if completed.profile_version() != atomized.profile_version()
        || completed.input_transformation_version() != atomized.transformation_version()
        || completed.transformation_version()
        != crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION || completed.source_len_bytes()
        != atomized.source_len_bytes() || completed.bom_bytes() != bom_bytes
        || completed.input_atom_count() != atoms.len() as u64 {
        let error = AnchorAliasError::at(
            AnchorAliasErrorKind::InputCompletedTokenMismatch,
            bom_bytes,
        );
        proof {
            reveal(resolve_profile1_anchor_aliases_spec);
        }
        return Err(error);
    }
    if cst.profile_version() != completed.profile_version()
        || cst.input_token_transformation_version() != completed.transformation_version()
        || cst.transformation_version() != crate::cst::CST_TRANSFORMATION_VERSION
        || cst.source_len_bytes() != completed.source_len_bytes() || cst.input_token_count()
        != tokens.len() as u64 || owners.len() != tokens.len() {
        let error = AnchorAliasError::at(AnchorAliasErrorKind::InputCstMismatch, bom_bytes);
        proof {
            reveal(resolve_profile1_anchor_aliases_spec);
        }
        return Err(error);
    }
    proof {
        assert(completed@.profile_version == atomized@.profile_version);
        assert(completed@.input_transformation_version == atomized@.transformation_version);
        assert(completed@.transformation_version
            == crate::token::COMPLETED_TOKEN_TRANSFORMATION_VERSION);
        assert(completed@.source_len_bytes == atomized@.source_len_bytes);
        assert(completed@.bom_bytes == atomized@.bom_bytes);
        assert(completed@.input_atom_count == atomized@.atoms.len());
        assert(cst@.profile_version == completed@.profile_version);
        assert(cst@.input_token_transformation_version == completed@.transformation_version);
        assert(cst@.transformation_version == crate::cst::CST_TRANSFORMATION_VERSION);
        assert(cst@.source_len_bytes == completed@.source_len_bytes);
        assert(cst@.input_token_count == completed@.tokens.len());
        assert(cst@.syntax_owners.len() == completed@.tokens.len());
    }
    let mut anchors = Vec::new();
    let mut aliases = Vec::new();
    let ghost initial_anchor_views = anchor_declaration_views_spec(anchors@);
    let ghost initial_alias_views = alias_binding_views_spec(aliases@);
    proof {
        reveal(anchor_declaration_views_spec);
        reveal(alias_binding_views_spec);
        assert(initial_anchor_views == Seq::<AnchorDeclarationView>::empty());
        assert(initial_alias_views == Seq::<AliasBindingView>::empty());
    }
    let ghost expected_scan = scan_anchor_alias_documents_spec(
        atomized@.atoms,
        completed@.tokens,
        cst@.documents,
        cst@.nodes,
        cst@.syntax_owners,
        0,
        cst@.documents.len() as nat,
        AnchorAliasBuildView { anchors: initial_anchor_views, aliases: initial_alias_views },
        limits@,
    );
    let scan = scan_anchor_alias_documents(
        atoms,
        tokens,
        documents,
        nodes,
        owners,
        &mut anchors,
        &mut aliases,
        limits,
    );
    match scan {
        Err(error) => {
            proof {
                reveal(anchor_declaration_views_spec);
                reveal(alias_binding_views_spec);
                assert(expected_scan == Err(error@));
                reveal(resolve_profile1_anchor_aliases_spec);
                assert(resolve_profile1_anchor_aliases_spec(atomized@, completed@, cst@, limits@)
                    == Err(error@));
            }
            Err(error)
        },
        Ok(()) => {
            let source = AnchorAliasSource::new(
                atomized.profile_version(),
                atomized.source_len_bytes(),
                completed.transformation_version(),
                cst.transformation_version(),
                anchors,
                aliases,
            );
            proof {
                reveal(resolve_profile1_anchor_aliases_spec);
            }
            Ok(source)
        },
    }
}

} // verus!
