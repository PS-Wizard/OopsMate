use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchLimits {
    Infinite,
    MoveTime {
        hard_time_ms: u64,
    },
    Clock {
        soft_time_ms: u64,
        hard_time_ms: u64,
    },
}

impl SearchLimits {
    pub const fn infinite() -> Self {
        Self::Infinite
    }

    pub const fn movetime(hard_time_ms: u64) -> Self {
        Self::MoveTime { hard_time_ms }
    }

    pub const fn clock(soft_time_ms: u64, hard_time_ms: u64) -> Self {
        Self::Clock {
            soft_time_ms,
            hard_time_ms,
        }
    }

    pub const fn from_max_time(max_time_ms: Option<u64>) -> Self {
        match max_time_ms {
            Some(hard_time_ms) => Self::MoveTime { hard_time_ms },
            None => Self::Infinite,
        }
    }

    pub const fn hard_time_ms(self) -> Option<u64> {
        match self {
            Self::Infinite => None,
            Self::MoveTime { hard_time_ms } => Some(hard_time_ms),
            Self::Clock { hard_time_ms, .. } => Some(hard_time_ms),
        }
    }
}

pub(crate) fn should_stop_next_iteration(
    limits: SearchLimits,
    start_time: Instant,
    current_depth_time: u64,
) -> bool {
    let elapsed_total = start_time.elapsed().as_millis() as u64;

    match limits {
        SearchLimits::Infinite => false,
        SearchLimits::MoveTime { hard_time_ms } => {
            let time_remaining = hard_time_ms.saturating_sub(elapsed_total);
            time_remaining == 0 || current_depth_time >= time_remaining
        }
        SearchLimits::Clock {
            soft_time_ms,
            hard_time_ms,
        } => {
            if elapsed_total >= hard_time_ms || elapsed_total >= soft_time_ms {
                return true;
            }

            let time_remaining = soft_time_ms.saturating_sub(elapsed_total);
            let predicted_next_depth = current_depth_time.saturating_mul(2);

            predicted_next_depth > time_remaining
        }
    }
}
