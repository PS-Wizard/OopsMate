use std::time::{Duration, Instant};

/// Manages time control for search
pub struct TimeControl {
    start_time: Instant,
    allocated_time: Duration,
    hard_limit: Duration,
}

impl TimeControl {
    /// Create a new time control with allocated time
    pub fn new(allocated_ms: u64) -> Self {
        let allocated = Duration::from_millis(allocated_ms);
        let hard_limit = Duration::from_millis((allocated_ms as f64 * 1.5) as u64);

        TimeControl {
            start_time: Instant::now(),
            allocated_time: allocated,
            hard_limit,
        }
    }

    /// Create infinite time control
    pub fn infinite() -> Self {
        TimeControl {
            start_time: Instant::now(),
            allocated_time: Duration::from_secs(u64::MAX),
            hard_limit: Duration::from_secs(u64::MAX),
        }
    }

    /// Check if we should stop searching (soft limit)
    #[inline(always)]
    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed() >= self.allocated_time
    }

    /// Check if we must stop searching (hard limit)
    #[inline(always)]
    pub fn must_stop(&self) -> bool {
        self.start_time.elapsed() >= self.hard_limit
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

/// Calculate time allocation based on UCI go command parameters
pub fn calculate_time_allocation(our_time: u64, our_inc: u64, moves_to_go: Option<u32>) -> u64 {
    // If moves to go is specified, divide time accordingly
    if let Some(mtg) = moves_to_go {
        let base = our_time / (mtg as u64 + 1);
        return base + our_inc;
    }

    // Otherwise, assume ~40 moves left in game
    let moves_left = 40;
    let base_time = our_time / moves_left;
    let allocated = base_time + (our_inc * 3) / 4;

    // Never use more than 1/3 of remaining time
    allocated.min(our_time / 3)
}
