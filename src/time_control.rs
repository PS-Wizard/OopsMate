//! Time allocation helpers for the UCI front-end.

use std::time::{Duration, Instant};

const MIN_SEARCH_BUDGET_MS: u64 = 1;
pub const DEFAULT_MOVE_OVERHEAD_MS: u64 = 25;

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
    calculate_clock_limits(our_time, our_inc, moves_to_go, DEFAULT_MOVE_OVERHEAD_MS).0
}

/// Derives soft and hard search limits from UCI clock data.
pub fn calculate_clock_limits(
    our_time: u64,
    our_inc: u64,
    moves_to_go: Option<u32>,
    move_overhead_ms: u64,
) -> (u64, u64) {
    let available = our_time
        .saturating_sub(move_overhead_ms)
        .max(MIN_SEARCH_BUDGET_MS);
    let moves_left = moves_to_go
        .filter(|&mtg| mtg > 0)
        .map(u64::from)
        .unwrap_or_else(|| match our_time {
            0..=1_000 => 12,
            1_001..=5_000 => 18,
            5_001..=20_000 => 24,
            _ => 30,
        });

    let base = (available / moves_left).max(MIN_SEARCH_BUDGET_MS);
    let soft = base.saturating_add((our_inc * 3) / 4);
    let mut hard_cap = base.saturating_mul(4).min((available * 35) / 100);

    if available < 300 {
        hard_cap = hard_cap.min((available * 15) / 100);
    } else if available < 1_000 {
        hard_cap = hard_cap.min(available / 4);
    }

    let hard = hard_cap.max(MIN_SEARCH_BUDGET_MS).min(available);
    (soft.min(hard).max(MIN_SEARCH_BUDGET_MS), hard)
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

/// Keeps `movetime` close to the requested limit while preserving a tiny
/// scheduling reserve so the engine does not routinely flag on time.
pub fn clamp_movetime_budget(limit_ms: u64) -> u64 {
    clamp_movetime_budget_with_overhead(limit_ms, DEFAULT_MOVE_OVERHEAD_MS)
}

/// Keeps `movetime` inside the requested limit while preserving UCI overhead.
pub fn clamp_movetime_budget_with_overhead(limit_ms: u64, move_overhead_ms: u64) -> u64 {
    let reserve = move_overhead_ms.min(limit_ms.saturating_sub(MIN_SEARCH_BUDGET_MS));
    limit_ms.saturating_sub(reserve).max(MIN_SEARCH_BUDGET_MS)
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_clock_limits, calculate_time_allocation, clamp_movetime_budget,
        clamp_movetime_budget_with_overhead, clamp_search_budget,
    };

    #[test]
    fn sudden_death_uses_more_than_old_fraction_at_five_seconds() {
        let allocation = calculate_time_allocation(5_000, 0, None);
        assert!(allocation >= 250);
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

    #[test]
    fn movetime_budget_uses_most_of_requested_time() {
        assert_eq!(clamp_movetime_budget(500), 475);
        assert_eq!(clamp_movetime_budget(20), 1);
        assert_eq!(clamp_movetime_budget_with_overhead(500, 10), 490);
    }

    #[test]
    fn clock_limits_keep_hard_cap_below_remaining_time() {
        let (soft, hard) = calculate_clock_limits(1_000, 0, None, 25);
        assert!(soft <= hard);
        assert!(hard <= 975 / 4);
    }
}
