//! Time allocation helpers for the UCI front-end.

use std::time::{Duration, Instant};

const MIN_SEARCH_BUDGET_MS: u64 = 1;

/// Soft and hard limits for a single search allocation.
pub struct TimeControl {
    start_time: Instant,
    allocated_time: Duration,
}

impl TimeControl {
    /// Creates a finite time control from a millisecond allocation.
    pub fn new(allocated_ms: u64) -> Self {
        let allocated = Duration::from_millis(allocated_ms);

        TimeControl {
            start_time: Instant::now(),
            allocated_time: allocated,
        }
    }

    /// Creates an effectively unbounded time control.
    pub fn infinite() -> Self {
        TimeControl {
            start_time: Instant::now(),
            allocated_time: Duration::from_secs(u64::MAX),
        }
    }

    /// Returns `true` once the soft limit has been reached.
    #[inline(always)]
    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed() >= self.allocated_time
    }
}

/// Derives a practical move allocation from remaining time, increment, and
/// optional moves-to-go information.
pub fn calculate_time_allocation(our_time: u64, our_inc: u64, moves_to_go: Option<u32>) -> u64 {
    if let Some(mtg) = moves_to_go.filter(|&mtg| mtg > 0) {
        let base = our_time / mtg as u64;
        let inc_share = (our_inc * 3) / 4;
        let allocation = base.saturating_add(inc_share);
        let max_share = (our_time / 2).max(MIN_SEARCH_BUDGET_MS);
        return allocation.min(max_share).max(MIN_SEARCH_BUDGET_MS);
    }

    let moves_left = match our_time {
        0..=1_000 => 10,
        1_001..=5_000 => 16,
        5_001..=20_000 => 22,
        _ => 28,
    };
    let base_time = our_time / moves_left;
    let allocated = base_time + (our_inc * 7) / 8;
    let max_share = match our_time {
        0..=1_000 => our_time / 2,
        1_001..=5_000 => our_time / 3,
        _ => our_time / 4,
    }
    .max(MIN_SEARCH_BUDGET_MS);

    allocated.min(max_share).max(MIN_SEARCH_BUDGET_MS)
}

/// Shrinks an external time limit into an internal search budget that leaves a
/// safety buffer for overshoot, scheduling, and I/O latency.
pub fn clamp_search_budget(limit_ms: u64) -> u64 {
    let reserve = match limit_ms {
        0..=50 => 5,
        51..=100 => 15,
        101..=250 => 50,
        251..=1000 => 75,
        _ => (limit_ms / 20).clamp(50, 250),
    };

    limit_ms.saturating_sub(reserve).max(MIN_SEARCH_BUDGET_MS)
}

#[cfg(test)]
mod tests {
    use super::{calculate_time_allocation, clamp_search_budget};

    #[test]
    fn sudden_death_uses_more_than_old_fraction_at_five_seconds() {
        let allocation = calculate_time_allocation(5_000, 0, None);
        assert!(allocation >= 300);
    }

    #[test]
    fn increment_increases_allocation() {
        let without_inc = calculate_time_allocation(5_000, 0, None);
        let with_inc = calculate_time_allocation(5_000, 100, None);
        assert!(with_inc > without_inc);
    }

    #[test]
    fn clamp_budget_keeps_a_small_reserve() {
        assert_eq!(clamp_search_budget(200), 150);
        assert_eq!(clamp_search_budget(20), 15);
    }
}
