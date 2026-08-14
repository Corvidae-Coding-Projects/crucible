//! Verified, bounded, engine-neutral campaign scheduling policy.
use vstd::prelude::*;

verus! {

pub const ENGINE_CLASS_COUNT: usize = 7;

pub const MAX_SCHEDULER_CREDITS: usize = 4_096;

pub const MAX_SCHEDULER_SLOTS: u64 = 1_000_000;

pub const MAX_SCHEDULER_METRIC: u64 = 1_000_000_000_000;

const BASELINE_WEIGHTS: [u64; ENGINE_CLASS_COUNT] = [35, 20, 15, 10, 10, 5, 5];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineClass {
    CoverageFuzzing,
    PropertyTesting,
    StatefulTesting,
    MetamorphicTesting,
    FaultInjection,
    SymbolicTesting,
    Miscellaneous,
}

impl EngineClass {
    pub const ALL: [Self; ENGINE_CLASS_COUNT] = [
        Self::CoverageFuzzing,
        Self::PropertyTesting,
        Self::StatefulTesting,
        Self::MetamorphicTesting,
        Self::FaultInjection,
        Self::SymbolicTesting,
        Self::Miscellaneous,
    ];

    pub fn index(self) -> (index: usize)
        ensures
            index < ENGINE_CLASS_COUNT,
    {
        match self {
            Self::CoverageFuzzing => 0,
            Self::PropertyTesting => 1,
            Self::StatefulTesting => 2,
            Self::MetamorphicTesting => 3,
            Self::FaultInjection => 4,
            Self::SymbolicTesting => 5,
            Self::Miscellaneous => 6,
        }
    }

    pub fn from_index(index: usize) -> (engine: Self)
        requires
            index < ENGINE_CLASS_COUNT,
    {
        match index {
            0 => Self::CoverageFuzzing,
            1 => Self::PropertyTesting,
            2 => Self::StatefulTesting,
            3 => Self::MetamorphicTesting,
            4 => Self::FaultInjection,
            5 => Self::SymbolicTesting,
            _ => Self::Miscellaneous,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EngineStats {
    pub engine: EngineClass,
    pub enabled: bool,
    pub capability_available: bool,
    pub executions: u64,
    pub cpu_seconds: u64,
    pub cpu_nanoseconds: u32,
    pub new_coverage: u64,
    pub new_findings: u64,
    pub unique_states: u64,
    pub minimized_findings: u64,
    pub mutation_score_improvement: u64,
    pub new_oracle_failures: u64,
    pub corpus_quality_improvement: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProvenanceCredit {
    pub precursor: EngineClass,
    pub confirmer: EngineClass,
    pub units: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchedulerPolicy {
    pub exploration_floor_percent: u8,
    pub adaptive_percent: u8,
    pub reallocation_interval: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineAllocation {
    pub engine: EngineClass,
    pub enabled: bool,
    pub slots: u64,
    pub utility_credit: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerError {
    EngineSet,
    Policy,
    NoEnabledEngines,
    MetricLimit,
    CreditLimit,
    SlotLimit,
}

fn metrics_within_limit(stats: &EngineStats) -> (valid: bool) {
    stats.executions <= MAX_SCHEDULER_METRIC && stats.cpu_seconds <= MAX_SCHEDULER_METRIC
        && stats.cpu_nanoseconds < 1_000_000_000 && stats.new_coverage <= MAX_SCHEDULER_METRIC
        && stats.new_findings <= MAX_SCHEDULER_METRIC && stats.unique_states <= MAX_SCHEDULER_METRIC
        && stats.minimized_findings <= MAX_SCHEDULER_METRIC && stats.mutation_score_improvement
        <= MAX_SCHEDULER_METRIC && stats.new_oracle_failures <= MAX_SCHEDULER_METRIC
        && stats.corpus_quality_improvement <= MAX_SCHEDULER_METRIC
}

fn utility(stats: &EngineStats, provenance_credit: u64) -> (result: Result<u64, SchedulerError>) {
    let divisor = stats.cpu_seconds as u128 + 1;
    let hour = 3_600u128;
    let score = stats.new_findings as u128 * hour * 1_000 / divisor + stats.new_coverage as u128
        * hour * 5 / divisor + stats.unique_states as u128 * hour * 2 / divisor
        + stats.minimized_findings as u128 * hour * 100 / divisor
        + stats.mutation_score_improvement as u128 * hour * 50 / divisor
        + stats.new_oracle_failures as u128 * hour * 250 / divisor
        + stats.corpus_quality_improvement as u128 * hour * 10 / divisor + provenance_credit as u128
        * hour / divisor;
    if score > u64::MAX as u128 {
        Err(SchedulerError::MetricLimit)
    } else {
        Ok(score as u64)
    }
}

fn checked_product(left: u128, right: u128) -> (result: Result<u128, SchedulerError>) {
    match left.checked_mul(right) {
        Some(product) => Ok(product),
        None => Err(SchedulerError::MetricLimit),
    }
}

#[expect(
    clippy::manual_is_multiple_of,
    reason = "the pinned verifier proves modulo here but does not model the standard-library replacement"
)]
pub fn schedule_campaign(
    stats: &[EngineStats],
    credits: &[ProvenanceCredit],
    policy: SchedulerPolicy,
    epoch: u64,
    total_slots: u64,
) -> (result: Result<Vec<EngineAllocation>, SchedulerError>) {
    if stats.len() != ENGINE_CLASS_COUNT {
        return Err(SchedulerError::EngineSet);
    }
    if credits.len() > MAX_SCHEDULER_CREDITS {
        return Err(SchedulerError::CreditLimit);
    }
    if total_slots > MAX_SCHEDULER_SLOTS {
        return Err(SchedulerError::SlotLimit);
    }
    if policy.exploration_floor_percent > 100 || policy.adaptive_percent > 100
        || policy.reallocation_interval == 0 {
        return Err(SchedulerError::Policy);
    }
    let mut seen = [false;ENGINE_CLASS_COUNT];
    let mut enabled = [false;ENGINE_CLASS_COUNT];
    let mut source_index = [0usize;ENGINE_CLASS_COUNT];
    let mut enabled_count = 0u64;
    let mut index = 0;
    while index < stats.len()
        invariant
            index <= stats.len(),
            stats.len() == ENGINE_CLASS_COUNT,
            enabled_count <= index,
        decreases stats.len() - index,
    {
        let engine_index = stats[index].engine.index();
        if seen[engine_index] {
            return Err(SchedulerError::EngineSet);
        }
        if !metrics_within_limit(&stats[index]) {
            return Err(SchedulerError::MetricLimit);
        }
        seen[engine_index] = true;
        source_index[engine_index] = index;
        let available = stats[index].enabled && stats[index].capability_available;
        enabled[engine_index] = available;
        if available {
            enabled_count += 1;
        }
        index += 1;
    }
    index = 0;
    while index < ENGINE_CLASS_COUNT
        invariant
            index <= ENGINE_CLASS_COUNT,
        decreases ENGINE_CLASS_COUNT - index,
    {
        if !seen[index] {
            return Err(SchedulerError::EngineSet);
        }
        index += 1;
    }
    if enabled_count == 0 {
        return Err(SchedulerError::NoEnabledEngines);
    }
    let mut ancestry_credit = [0u64;ENGINE_CLASS_COUNT];
    index = 0;
    while index < credits.len()
        invariant
            index <= credits.len(),
            credits.len() <= MAX_SCHEDULER_CREDITS,
        decreases credits.len() - index,
    {
        let credit = credits[index];
        if credit.units > MAX_SCHEDULER_METRIC {
            return Err(SchedulerError::MetricLimit);
        }
        let precursor = credit.precursor.index();
        if ancestry_credit[precursor] > u64::MAX - credit.units {
            return Err(SchedulerError::MetricLimit);
        }
        ancestry_credit[precursor] += credit.units;
        let confirmer = credit.confirmer.index();
        if confirmer != precursor {
            if ancestry_credit[confirmer] > u64::MAX - credit.units {
                return Err(SchedulerError::MetricLimit);
            }
            ancestry_credit[confirmer] += credit.units;
        }
        index += 1;
    }

    let mut utilities = [0u64;ENGINE_CLASS_COUNT];
    let mut utility_total = 0u128;
    index = 0;
    while index < ENGINE_CLASS_COUNT
        invariant
            index <= ENGINE_CLASS_COUNT,
        decreases ENGINE_CLASS_COUNT - index,
    {
        if enabled[index] {
            let selected = source_index[index];
            if selected >= stats.len() {
                return Err(SchedulerError::EngineSet);
            }
            let value = utility(&stats[selected], ancestry_credit[index])?;
            utilities[index] = value;
            if utility_total > u128::MAX - value as u128 {
                return Err(SchedulerError::MetricLimit);
            }
            utility_total += value as u128;
        }
        index += 1;
    }

    let floor_each_wide = checked_product(
        total_slots as u128,
        policy.exploration_floor_percent as u128,
    )? / 100;
    if floor_each_wide > u64::MAX as u128 {
        return Err(SchedulerError::Policy);
    }
    let floor_each = floor_each_wide as u64;
    let floor_total_wide = checked_product(floor_each as u128, enabled_count as u128)?;
    if floor_total_wide > u64::MAX as u128 {
        return Err(SchedulerError::Policy);
    }
    let floor_total = floor_total_wide as u64;
    if floor_total > total_slots {
        return Err(SchedulerError::Policy);
    }
    let mut slots = [0u64;ENGINE_CLASS_COUNT];
    let mut baseline_weight = 0u128;
    index = 0;
    while index < ENGINE_CLASS_COUNT
        invariant
            index <= ENGINE_CLASS_COUNT,
        decreases ENGINE_CLASS_COUNT - index,
    {
        if enabled[index] {
            slots[index] = floor_each;
            if baseline_weight > u128::MAX - BASELINE_WEIGHTS[index] as u128 {
                return Err(SchedulerError::Policy);
            }
            baseline_weight += BASELINE_WEIGHTS[index] as u128;
        }
        index += 1;
    }
    if baseline_weight == 0 {
        return Err(SchedulerError::NoEnabledEngines);
    }
    let remaining = total_slots - floor_total;
    let periodic = epoch > 0 && epoch % policy.reallocation_interval == 0;
    let adaptive_pool = if periodic && utility_total > 0 {
        let wide = checked_product(remaining as u128, policy.adaptive_percent as u128)? / 100;
        if wide > u64::MAX as u128 {
            return Err(SchedulerError::Policy);
        }
        wide as u64
    } else {
        0
    };
    if adaptive_pool > total_slots {
        return Err(SchedulerError::Policy);
    }
    let baseline_budget = total_slots - adaptive_pool;
    let mut assigned = floor_total as u128;
    index = 0;
    while index < ENGINE_CLASS_COUNT
        invariant
            index <= ENGINE_CLASS_COUNT,
            assigned <= total_slots as u128,
        decreases ENGINE_CLASS_COUNT - index,
    {
        if enabled[index] {
            if baseline_weight == 0 {
                return Err(SchedulerError::NoEnabledEngines);
            }
            let baseline_target_wide = checked_product(
                baseline_budget as u128,
                BASELINE_WEIGHTS[index] as u128,
            )? / baseline_weight;
            if baseline_target_wide > u64::MAX as u128 {
                return Err(SchedulerError::Policy);
            }
            let baseline_target = baseline_target_wide as u64;
            if baseline_target < floor_each {
                return Err(SchedulerError::Policy);
            }
            let baseline = baseline_target - floor_each;
            let adaptive = if adaptive_pool > 0 {
                if utility_total == 0 {
                    return Err(SchedulerError::MetricLimit);
                }
                let wide = checked_product(adaptive_pool as u128, utilities[index] as u128)?
                    / utility_total;
                if wide > u64::MAX as u128 {
                    return Err(SchedulerError::MetricLimit);
                }
                wide as u64
            } else {
                0
            };
            let addition = baseline as u128 + adaptive as u128;
            let updated = slots[index] as u128 + addition;
            if updated > u64::MAX as u128 || assigned + addition > total_slots as u128 {
                return Err(SchedulerError::Policy);
            }
            slots[index] = updated as u64;
            assigned += addition;
        }
        index += 1;
    }
    if assigned > total_slots as u128 {
        return Err(SchedulerError::Policy);
    }
    let unassigned = total_slots - assigned as u64;
    index = 0;
    while index < ENGINE_CLASS_COUNT && !enabled[index]
        invariant
            index <= ENGINE_CLASS_COUNT,
        decreases ENGINE_CLASS_COUNT - index,
    {
        index += 1;
    }
    if index == ENGINE_CLASS_COUNT {
        return Err(SchedulerError::NoEnabledEngines);
    }
    if slots[index] > u64::MAX - unassigned {
        return Err(SchedulerError::Policy);
    }
    slots[index] += unassigned;

    let mut allocation = Vec::new();
    index = 0;
    while index < ENGINE_CLASS_COUNT
        invariant
            index <= ENGINE_CLASS_COUNT,
            allocation.len() == index,
        decreases ENGINE_CLASS_COUNT - index,
    {
        allocation.push(
            EngineAllocation {
                engine: EngineClass::from_index(index),
                enabled: enabled[index],
                slots: slots[index],
                utility_credit: utilities[index],
            },
        );
        index += 1;
    }
    Ok(allocation)
}

} // verus!
