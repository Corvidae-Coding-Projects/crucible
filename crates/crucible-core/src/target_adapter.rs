//! Verified ownership and state transitions shared by target adapters.
use crate::{RunAttemptId, TargetBuildId, TargetId};
use vstd::prelude::*;

verus! {

pub const MAX_TARGET_INSTANCE_ORDINAL: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetAdapterKind {
    Cli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetAdapterIdentity {
    kind: TargetAdapterKind,
    version: u64,
}

#[verifier::ext_equal]
pub struct TargetAdapterIdentityView {
    pub kind: TargetAdapterKind,
    pub version: u64,
}

impl View for TargetAdapterIdentity {
    type V = TargetAdapterIdentityView;

    closed spec fn view(&self) -> TargetAdapterIdentityView {
        TargetAdapterIdentityView { kind: self.kind, version: self.version }
    }
}

pub open spec fn target_adapter_identity_well_formed_spec(
    identity: TargetAdapterIdentityView,
) -> bool {
    identity.version > 0
}

impl TargetAdapterIdentity {
    pub fn new(kind: TargetAdapterKind, version: u64) -> (result: Result<
        Self,
        TargetLifecycleError,
    >)
        ensures
            match &result {
                Ok(identity) => target_adapter_identity_well_formed_spec(identity@)
                    && identity@.kind == kind && identity@.version == version,
                Err(TargetLifecycleError::InvalidAdapterVersion) => version == 0,
                Err(_) => false,
            },
    {
        if version == 0 {
            Err(TargetLifecycleError::InvalidAdapterVersion)
        } else {
            Ok(Self { kind, version })
        }
    }

    pub fn kind(&self) -> (kind: TargetAdapterKind)
        ensures
            kind == self@.kind,
    {
        self.kind
    }

    pub fn version(&self) -> (version: u64)
        ensures
            version == self@.version,
    {
        self.version
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetLifecycleState {
    Allocated,
    Prepared,
    Executing,
    ResetRequired,
    Cleaned,
    Discarded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetLifecycleAction {
    PrepareSucceeded,
    PrepareFailed,
    BeginExecute,
    FinishExecute,
    ResetSucceeded,
    ResetUncertain,
    CleanupSucceeded,
    CleanupUncertain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetLifecycleError {
    InvalidAdapterVersion,
    EmptyTargetId,
    EmptyTargetBuildId,
    EmptyOwnerAttemptId,
    InstanceOrdinalOutOfRange,
    InvalidTransition,
}

/// Linear ownership token for one prepared target instance.
///
/// The token deliberately does not implement `Clone`; every transition consumes the prior token.
///
/// ```compile_fail
/// use crucible_core::TargetInstanceLifecycle;
///
/// fn duplicate(instance: TargetInstanceLifecycle) {
///     let _second_owner = instance.clone();
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct TargetInstanceLifecycle {
    adapter: TargetAdapterIdentity,
    target_id: TargetId,
    target_build_id: TargetBuildId,
    owner_attempt_id: RunAttemptId,
    instance_ordinal: u64,
    state: TargetLifecycleState,
}

#[verifier::ext_equal]
pub struct TargetInstanceLifecycleView {
    pub adapter: TargetAdapterIdentityView,
    pub target_id: Seq<char>,
    pub target_build_id: Seq<char>,
    pub owner_attempt_id: Seq<char>,
    pub instance_ordinal: u64,
    pub state: TargetLifecycleState,
}

impl View for TargetInstanceLifecycle {
    type V = TargetInstanceLifecycleView;

    closed spec fn view(&self) -> TargetInstanceLifecycleView {
        TargetInstanceLifecycleView {
            adapter: self.adapter@,
            target_id: self.target_id@,
            target_build_id: self.target_build_id@,
            owner_attempt_id: self.owner_attempt_id@,
            instance_ordinal: self.instance_ordinal,
            state: self.state,
        }
    }
}

pub open spec fn target_instance_lifecycle_well_formed_spec(
    lifecycle: TargetInstanceLifecycleView,
) -> bool {
    target_adapter_identity_well_formed_spec(lifecycle.adapter) && lifecycle.target_id.len() > 0
        && lifecycle.target_build_id.len() > 0 && lifecycle.owner_attempt_id.len() > 0 && 0
        < lifecycle.instance_ordinal <= MAX_TARGET_INSTANCE_ORDINAL
}

pub open spec fn same_target_instance_identity_spec(
    left: TargetInstanceLifecycleView,
    right: TargetInstanceLifecycleView,
) -> bool {
    left.adapter == right.adapter && left.target_id == right.target_id && left.target_build_id
        == right.target_build_id && left.owner_attempt_id == right.owner_attempt_id
        && left.instance_ordinal == right.instance_ordinal
}

pub open spec fn target_lifecycle_transition_spec(
    current: TargetLifecycleState,
    action: TargetLifecycleAction,
) -> Option<TargetLifecycleState> {
    match (current, action) {
        (TargetLifecycleState::Allocated, TargetLifecycleAction::PrepareSucceeded) => {
            Some(TargetLifecycleState::Prepared)
        },
        (TargetLifecycleState::Allocated, TargetLifecycleAction::PrepareFailed) => {
            Some(TargetLifecycleState::Discarded)
        },
        (TargetLifecycleState::Prepared, TargetLifecycleAction::BeginExecute) => {
            Some(TargetLifecycleState::Executing)
        },
        (TargetLifecycleState::Executing, TargetLifecycleAction::FinishExecute) => {
            Some(TargetLifecycleState::ResetRequired)
        },
        (TargetLifecycleState::ResetRequired, TargetLifecycleAction::ResetSucceeded) => {
            Some(TargetLifecycleState::Prepared)
        },
        (TargetLifecycleState::ResetRequired, TargetLifecycleAction::ResetUncertain) => {
            Some(TargetLifecycleState::Discarded)
        },
        (
            TargetLifecycleState::Prepared
            | TargetLifecycleState::ResetRequired,
            TargetLifecycleAction::CleanupSucceeded,
        ) => Some(TargetLifecycleState::Cleaned),
        (
            TargetLifecycleState::Prepared
            | TargetLifecycleState::ResetRequired,
            TargetLifecycleAction::CleanupUncertain,
        ) => Some(TargetLifecycleState::Discarded),
        _ => None,
    }
}

impl TargetInstanceLifecycle {
    pub fn new(
        adapter: TargetAdapterIdentity,
        target_id: TargetId,
        target_build_id: TargetBuildId,
        owner_attempt_id: RunAttemptId,
        instance_ordinal: u64,
    ) -> (result: Result<Self, TargetLifecycleError>)
        ensures
            match &result {
                Ok(lifecycle) => target_instance_lifecycle_well_formed_spec(lifecycle@)
                    && lifecycle@.adapter == adapter@ && lifecycle@.target_id == target_id@
                    && lifecycle@.target_build_id == target_build_id@ && lifecycle@.owner_attempt_id
                    == owner_attempt_id@ && lifecycle@.instance_ordinal == instance_ordinal
                    && lifecycle@.state == TargetLifecycleState::Allocated,
                Err(TargetLifecycleError::InvalidAdapterVersion) => {
                    !target_adapter_identity_well_formed_spec(adapter@)
                },
                Err(TargetLifecycleError::EmptyTargetId) => target_id@.len() == 0,
                Err(TargetLifecycleError::EmptyTargetBuildId) => target_id@.len() > 0
                    && target_build_id@.len() == 0,
                Err(TargetLifecycleError::EmptyOwnerAttemptId) => target_id@.len() > 0
                    && target_build_id@.len() > 0 && owner_attempt_id@.len() == 0,
                Err(TargetLifecycleError::InstanceOrdinalOutOfRange) => target_id@.len() > 0
                    && target_build_id@.len() > 0 && owner_attempt_id@.len() > 0 && (
                instance_ordinal == 0 || instance_ordinal > MAX_TARGET_INSTANCE_ORDINAL),
                Err(TargetLifecycleError::InvalidTransition) => false,
            },
    {
        if adapter.version == 0 {
            return Err(TargetLifecycleError::InvalidAdapterVersion);
        }
        if target_id.as_str().is_empty() {
            return Err(TargetLifecycleError::EmptyTargetId);
        }
        if target_build_id.as_str().is_empty() {
            return Err(TargetLifecycleError::EmptyTargetBuildId);
        }
        if owner_attempt_id.as_str().is_empty() {
            return Err(TargetLifecycleError::EmptyOwnerAttemptId);
        }
        if instance_ordinal == 0 || instance_ordinal > MAX_TARGET_INSTANCE_ORDINAL {
            return Err(TargetLifecycleError::InstanceOrdinalOutOfRange);
        }
        Ok(
            Self {
                adapter,
                target_id,
                target_build_id,
                owner_attempt_id,
                instance_ordinal,
                state: TargetLifecycleState::Allocated,
            },
        )
    }

    pub fn adapter(&self) -> (adapter: &TargetAdapterIdentity)
        ensures
            adapter@ == self@.adapter,
    {
        &self.adapter
    }

    pub fn target_id(&self) -> (target_id: &TargetId)
        ensures
            target_id@ == self@.target_id,
    {
        &self.target_id
    }

    pub fn target_build_id(&self) -> (target_build_id: &TargetBuildId)
        ensures
            target_build_id@ == self@.target_build_id,
    {
        &self.target_build_id
    }

    pub fn owner_attempt_id(&self) -> (owner_attempt_id: &RunAttemptId)
        ensures
            owner_attempt_id@ == self@.owner_attempt_id,
    {
        &self.owner_attempt_id
    }

    pub fn instance_ordinal(&self) -> (instance_ordinal: u64)
        ensures
            instance_ordinal == self@.instance_ordinal,
    {
        self.instance_ordinal
    }

    pub fn state(&self) -> (state: TargetLifecycleState)
        ensures
            state == self@.state,
    {
        self.state
    }
}

pub fn advance_target_instance_lifecycle(
    mut lifecycle: TargetInstanceLifecycle,
    action: TargetLifecycleAction,
) -> (result: Result<TargetInstanceLifecycle, TargetLifecycleError>)
    requires
        target_instance_lifecycle_well_formed_spec(lifecycle@),
    ensures
        match &result {
            Ok(next) => target_instance_lifecycle_well_formed_spec(next@)
                && target_lifecycle_transition_spec(lifecycle@.state, action) == Some(next@.state)
                && same_target_instance_identity_spec(lifecycle@, next@),
            Err(TargetLifecycleError::InvalidTransition) => {
                target_lifecycle_transition_spec(lifecycle@.state, action) is None
            },
            Err(_) => false,
        },
{
    let next = match (lifecycle.state, action) {
        (TargetLifecycleState::Allocated, TargetLifecycleAction::PrepareSucceeded) => {
            TargetLifecycleState::Prepared
        },
        (TargetLifecycleState::Allocated, TargetLifecycleAction::PrepareFailed) => {
            TargetLifecycleState::Discarded
        },
        (TargetLifecycleState::Prepared, TargetLifecycleAction::BeginExecute) => {
            TargetLifecycleState::Executing
        },
        (TargetLifecycleState::Executing, TargetLifecycleAction::FinishExecute) => {
            TargetLifecycleState::ResetRequired
        },
        (TargetLifecycleState::ResetRequired, TargetLifecycleAction::ResetSucceeded) => {
            TargetLifecycleState::Prepared
        },
        (TargetLifecycleState::ResetRequired, TargetLifecycleAction::ResetUncertain) => {
            TargetLifecycleState::Discarded
        },
        (
            TargetLifecycleState::Prepared
            | TargetLifecycleState::ResetRequired,
            TargetLifecycleAction::CleanupSucceeded,
        ) => TargetLifecycleState::Cleaned,
        (
            TargetLifecycleState::Prepared
            | TargetLifecycleState::ResetRequired,
            TargetLifecycleAction::CleanupUncertain,
        ) => TargetLifecycleState::Discarded,
        _ => return Err(TargetLifecycleError::InvalidTransition),
    };
    lifecycle.state = next;
    Ok(lifecycle)
}

} // verus!
