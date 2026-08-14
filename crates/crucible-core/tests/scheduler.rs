use crucible_core::{
    schedule_campaign, EngineClass, EngineStats, ProvenanceCredit, SchedulerError, SchedulerPolicy,
    MAX_SCHEDULER_CREDITS, MAX_SCHEDULER_SLOTS,
};

fn stats(engine: EngineClass) -> EngineStats {
    EngineStats {
        engine,
        enabled: true,
        capability_available: true,
        executions: 0,
        cpu_seconds: 0,
        cpu_nanoseconds: 0,
        new_coverage: 0,
        new_findings: 0,
        unique_states: 0,
        minimized_findings: 0,
        mutation_score_improvement: 0,
        new_oracle_failures: 0,
        corpus_quality_improvement: 0,
    }
}

fn all_stats() -> Vec<EngineStats> {
    EngineClass::ALL.into_iter().map(stats).collect()
}

fn policy() -> SchedulerPolicy {
    SchedulerPolicy {
        exploration_floor_percent: 2,
        adaptive_percent: 20,
        reallocation_interval: 10,
    }
}

#[test]
fn initial_allocation_is_the_documented_understandable_baseline() {
    let allocation = schedule_campaign(&all_stats(), &[], policy(), 0, 100).unwrap();
    let slots: Vec<_> = allocation.iter().map(|entry| entry.slots).collect();
    assert_eq!(slots, [35, 20, 15, 10, 10, 5, 5]);
    assert_eq!(allocation.iter().map(|entry| entry.slots).sum::<u64>(), 100);
}

#[test]
fn reallocation_is_periodic_and_uses_confirmed_quality_signals_per_cpu() {
    let mut measured = all_stats();
    measured[0].cpu_seconds = 3_600;
    measured[0].new_coverage = 10;
    measured[1].cpu_seconds = 60;
    measured[1].new_findings = 4;
    measured[1].new_oracle_failures = 2;

    let between_intervals = schedule_campaign(&measured, &[], policy(), 9, 100).unwrap();
    assert_eq!(
        between_intervals
            .iter()
            .map(|entry| entry.slots)
            .collect::<Vec<_>>(),
        [35, 20, 15, 10, 10, 5, 5]
    );

    let reallocated = schedule_campaign(&measured, &[], policy(), 10, 100).unwrap();
    assert!(reallocated[1].slots > between_intervals[1].slots);
    assert_eq!(
        reallocated.iter().map(|entry| entry.slots).sum::<u64>(),
        100
    );
    assert!(reallocated.iter().all(|entry| entry.slots >= 2));
}

#[test]
fn ancestry_and_delayed_confirmation_credit_both_contributing_engines() {
    let credits = [ProvenanceCredit {
        precursor: EngineClass::StatefulTesting,
        confirmer: EngineClass::FaultInjection,
        units: 50,
    }];
    let credited = schedule_campaign(&all_stats(), &credits, policy(), 10, 1_000).unwrap();
    let uncredited = schedule_campaign(&all_stats(), &[], policy(), 10, 1_000).unwrap();
    assert!(credited[2].utility_credit > uncredited[2].utility_credit);
    assert!(credited[4].utility_credit > uncredited[4].utility_credit);
    assert!(credited[2].slots > uncredited[2].slots);
    assert!(credited[4].slots > uncredited[4].slots);
}

#[test]
fn disabled_or_unavailable_engines_receive_no_work_and_enabled_engines_keep_the_floor() {
    let mut measured = all_stats();
    measured[5].capability_available = false;
    measured[6].enabled = false;
    let allocation = schedule_campaign(&measured, &[], policy(), 10, 100).unwrap();
    assert_eq!(allocation[5].slots, 0);
    assert_eq!(allocation[6].slots, 0);
    assert!(allocation[0..5].iter().all(|entry| entry.slots >= 2));
    assert_eq!(allocation.iter().map(|entry| entry.slots).sum::<u64>(), 100);
}

#[test]
fn scheduler_rejects_ambiguous_or_memory_amplifying_inputs() {
    let mut duplicate = all_stats();
    duplicate[6].engine = EngineClass::CoverageFuzzing;
    assert_eq!(
        schedule_campaign(&duplicate, &[], policy(), 0, 100),
        Err(SchedulerError::EngineSet)
    );

    let excessive_credits = (0..=MAX_SCHEDULER_CREDITS)
        .map(|_| ProvenanceCredit {
            precursor: EngineClass::CoverageFuzzing,
            confirmer: EngineClass::PropertyTesting,
            units: 1,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        schedule_campaign(&all_stats(), &excessive_credits, policy(), 10, 100),
        Err(SchedulerError::CreditLimit)
    );
    assert_eq!(
        schedule_campaign(&all_stats(), &[], policy(), 0, MAX_SCHEDULER_SLOTS + 1),
        Err(SchedulerError::SlotLimit)
    );
}

#[test]
fn scheduling_is_deterministic_for_identical_stats_seed_epoch_and_credit() {
    let measured = all_stats();
    let credits = [ProvenanceCredit {
        precursor: EngineClass::MetamorphicTesting,
        confirmer: EngineClass::SymbolicTesting,
        units: 9,
    }];
    let left = schedule_campaign(&measured, &credits, policy(), 20, 137).unwrap();
    let right = schedule_campaign(&measured, &credits, policy(), 20, 137).unwrap();
    assert_eq!(left, right);
}

#[test]
fn nightly_metamorphic_stats_reordering_preserves_engine_allocations() {
    let mut measured = all_stats();
    measured[1].new_findings = 11;
    measured[4].new_oracle_failures = 3;
    let expected = schedule_campaign(&measured, &[], policy(), 10, 997).unwrap();
    measured.reverse();
    let reordered = schedule_campaign(&measured, &[], policy(), 10, 997).unwrap();
    assert_eq!(reordered, expected);
}

#[test]
fn nightly_differential_baseline_matches_an_independent_integer_reference() {
    const WEIGHTS: [u64; 7] = [35, 20, 15, 10, 10, 5, 5];
    for total_slots in 100..=257 {
        let allocation = schedule_campaign(&all_stats(), &[], policy(), 0, total_slots).unwrap();
        let mut reference = WEIGHTS.map(|weight| total_slots * weight / 100);
        let assigned = reference.iter().sum::<u64>();
        reference[0] += total_slots - assigned;
        assert_eq!(
            allocation
                .iter()
                .map(|entry| entry.slots)
                .collect::<Vec<_>>(),
            reference,
            "differential mismatch for {total_slots} slots"
        );
    }
}

#[test]
fn weekly_symbolic_exploration_covers_every_capability_mask_and_boundary_epoch() {
    for mask in 1u8..=127 {
        for total_slots in [100, 101, 137, 1_000] {
            for epoch in [0, 9, 10, 20] {
                let mut measured = all_stats();
                for (index, item) in measured.iter_mut().enumerate() {
                    item.capability_available = mask & (1 << index) != 0;
                    item.new_coverage = index as u64;
                    item.new_findings = (6 - index) as u64;
                    item.cpu_seconds = index as u64 + 1;
                }
                let allocation =
                    schedule_campaign(&measured, &[], policy(), epoch, total_slots).unwrap();
                assert_eq!(allocation.len(), 7);
                assert_eq!(
                    allocation.iter().map(|entry| entry.slots).sum::<u64>(),
                    total_slots
                );
                for (index, entry) in allocation.iter().enumerate() {
                    assert_eq!(entry.enabled, mask & (1 << index) != 0);
                    if entry.enabled {
                        assert!(entry.slots >= total_slots * 2 / 100);
                    } else {
                        assert_eq!(entry.slots, 0);
                    }
                }
            }
        }
    }
}
