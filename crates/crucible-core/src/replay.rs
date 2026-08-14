//! Deterministic domain-separated seed derivation for recorded replay metadata.
use vstd::prelude::*;

verus! {

pub const ENGINE_SEED_DOMAIN: u64 = 0x454e_4749_4e45_0001;

pub const EXPERIMENT_SEED_DOMAIN: u64 = 0x4558_5045_5249_0001;

pub const SCHEDULING_SEED_DOMAIN: u64 = 0x5343_4845_4455_0001;

pub const FAULT_SEED_DOMAIN: u64 = 0x4641_554c_5453_0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplaySeeds {
    pub campaign: u64,
    pub engine: u64,
    pub experiment: u64,
    pub scheduling: u64,
    pub fault: u64,
}

pub fn derive_replay_seeds(campaign: u64) -> (seeds: ReplaySeeds)
    ensures
        seeds.campaign == campaign,
        seeds.engine == campaign ^ ENGINE_SEED_DOMAIN,
        seeds.experiment == campaign ^ EXPERIMENT_SEED_DOMAIN,
        seeds.scheduling == campaign ^ SCHEDULING_SEED_DOMAIN,
        seeds.fault == campaign ^ FAULT_SEED_DOMAIN,
{
    ReplaySeeds {
        campaign,
        engine: campaign ^ ENGINE_SEED_DOMAIN,
        experiment: campaign ^ EXPERIMENT_SEED_DOMAIN,
        scheduling: campaign ^ SCHEDULING_SEED_DOMAIN,
        fault: campaign ^ FAULT_SEED_DOMAIN,
    }
}

} // verus!
