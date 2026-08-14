use crucible_core::derive_replay_seeds;

#[test]
fn every_stochastic_domain_gets_a_stable_distinct_recorded_seed() {
    let seeds = derive_replay_seeds(7);
    assert_eq!(seeds.campaign, 7);
    assert_eq!(seeds.engine, 7 ^ 0x454e_4749_4e45_0001);
    assert_eq!(seeds.experiment, 7 ^ 0x4558_5045_5249_0001);
    assert_eq!(seeds.scheduling, 7 ^ 0x5343_4845_4455_0001);
    assert_eq!(seeds.fault, 7 ^ 0x4641_554c_5453_0001);
    assert_ne!(seeds.engine, seeds.experiment);
    assert_ne!(seeds.experiment, seeds.scheduling);
    assert_ne!(seeds.scheduling, seeds.fault);
}
