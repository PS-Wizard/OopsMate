//! Time allocation helpers for the UCI front-end.

use std::time::{Duration, Instant};

const MIN_SEARCH_BUDGET_MS: u64 = 1;

/// Soft and hard limits for a single search allocation.
pub struct TimeControl {
    start_time: Instant,
    allocated_time: Duration,
    hard_limit: Duration,
}

impl TimeControl {
    /// Creates a finite time control from a millisecond allocation.
    pub fn new(allocated_ms: u64) -> Self {
        let allocated = Duration::from_millis(allocated_ms);
        let hard_limit = Duration::from_millis((allocated_ms as f64 * 1.5) as u64);

        TimeControl {
            start_time: Instant::now(),
            allocated_time: allocated,
            hard_limit,
        }
    }

    /// Creates an effectively unbounded time control.
    pub fn infinite() -> Self {
        TimeControl {
            start_time: Instant::now(),
            allocated_time: Duration::from_secs(u64::MAX),
            hard_limit: Duration::from_secs(u64::MAX),
        }
    }

    /// Returns `true` once the soft limit has been reached.
    #[inline(always)]
    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed() >= self.allocated_time
    }

    /// Returns `true` once the hard limit has been reached.
    #[inline(always)]
    pub fn must_stop(&self) -> bool {
        self.start_time.elapsed() >= self.hard_limit
    }

    /// Returns elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

/// Derives a practical move allocation from remaining time, increment, and
/// optional moves-to-go information.
pub fn calculate_time_allocation(our_time: u64, our_inc: u64, moves_to_go: Option<u32>) -> u64 {
    if let Some(mtg) = moves_to_go {
        let base = our_time / (mtg as u64 + 1);
        return base + our_inc;
    }

    let moves_left = 40;
    let base_time = our_time / moves_left;
    let allocated = base_time + (our_inc * 3) / 4;

    allocated.min(our_time / 3)
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
