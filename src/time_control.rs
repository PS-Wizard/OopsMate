use std::time::{Duration, Instant};

pub struct TimeControl {
    start_time: Instant,
    allocated_time: Duration,
    hard_limit: Duration,
}

impl TimeControl {
    pub fn new(allocated_ms: u64) -> Self {
        let allocated = Duration::from_millis(allocated_ms);
        let hard_limit = Duration::from_millis((allocated_ms as f64 * 1.5) as u64);

        TimeControl {
            start_time: Instant::now(),
            allocated_time: allocated,
            hard_limit,
        }
    }

    pub fn infinite() -> Self {
        TimeControl {
            start_time: Instant::now(),
            allocated_time: Duration::from_secs(u64::MAX),
            hard_limit: Duration::from_secs(u64::MAX),
        }
    }

    #[inline(always)]
    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed() >= self.allocated_time
    }

    #[inline(always)]
    pub fn must_stop(&self) -> bool {
        self.start_time.elapsed() >= self.hard_limit
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

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
