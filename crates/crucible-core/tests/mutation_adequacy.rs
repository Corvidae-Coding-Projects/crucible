use crucible_core::{schedule_campaign, EngineClass, EngineStats, SchedulerPolicy};

fn empty_stats(engine: EngineClass) -> EngineStats {
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

#[test]
fn checked_in_scheduler_mutants_are_all_killed_by_the_policy_oracle() {
    let stats = EngineClass::ALL.map(empty_stats);
    let actual = schedule_campaign(
        &stats,
        &[],
        SchedulerPolicy {
            exploration_floor_percent: 2,
            adaptive_percent: 20,
            reallocation_interval: 10,
        },
        0,
        100,
    )
    .unwrap()
    .into_iter()
    .map(|entry| entry.slots)
    .collect::<Vec<_>>();

    let seeded_mutants = [
        [34, 21, 15, 10, 10, 5, 5],
        [35, 20, 14, 11, 10, 5, 5],
        [35, 20, 15, 10, 9, 6, 5],
        [35, 20, 15, 10, 10, 4, 6],
        [36, 19, 15, 10, 10, 5, 5],
        [20, 35, 15, 10, 10, 5, 5],
    ];
    assert_eq!(actual, [35, 20, 15, 10, 10, 5, 5]);
    for mutant in seeded_mutants {
        assert_ne!(actual, mutant, "a seeded scheduler mutant survived");
    }
}
