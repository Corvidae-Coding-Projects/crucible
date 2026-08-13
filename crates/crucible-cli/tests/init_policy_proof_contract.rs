use crucible_cli::{database_snapshot_is_exact, DatabaseSnapshot};
#[allow(unused_imports)]
use vstd::prelude::*;

verus! {

#[allow(dead_code)]
fn exact_runtime_policy_matches_its_pure_model(snapshot: &DatabaseSnapshot) -> (exact: bool)
    ensures
        exact == crucible_cli::database_snapshot_is_exact_spec(snapshot@),
{
    database_snapshot_is_exact(snapshot)
}

proof fn exact_database_identity_cannot_be_laundered(
    left: DatabaseSnapshot,
    right: DatabaseSnapshot,
)
    requires
        crucible_cli::database_snapshot_is_exact_spec(left@),
        crucible_cli::database_snapshot_is_exact_spec(right@),
    ensures
        left@ == right@,
{
    crucible_cli::lemma_exact_database_snapshot_is_unique(left, right);
}

proof fn version_one_and_current_database_profiles_are_disjoint(snapshot: DatabaseSnapshot)
    requires
        crucible_cli::database_snapshot_is_exact_v1_spec(snapshot@),
    ensures
        !crucible_cli::database_snapshot_is_exact_spec(snapshot@),
{
    crucible_cli::lemma_database_profiles_are_disjoint(snapshot);
}

} // verus!
#[test]
fn proof_contract_is_compiled() {
    assert_eq!(crucible_cli::WORKSPACE_SCHEMA_VERSION, 2);
}
