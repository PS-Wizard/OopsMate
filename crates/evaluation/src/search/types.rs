// evaluation/src/search/types.rs
// Shared types for both HCE and NNUE search

use std::time::Duration;
use types::moves::Move;

/// Controls when to stop the iterative deepening search
#[derive(Clone)]
pub struct SearchLimits {
    /// Maximum depth to search
    pub max_depth: Option<u8>,
    /// Maximum time to spend on this move
    pub max_time: Option<Duration>,
    /// Hard time limit (never exceed this)
    pub hard_limit: Option<Duration>,
    /// Search indefinitely
    pub infinite: bool,
}

impl SearchLimits {
    pub fn new() -> Self {
        Self {
            max_depth: None,
            max_time: None,
            hard_limit: None,
            infinite: false,
        }
    }

    pub fn from_depth(depth: u8) -> Self {
        Self {
            max_depth: Some(depth),
            max_time: None,
            hard_limit: None,
            infinite: false,
        }
    }

    pub fn from_time(soft_ms: u64, hard_ms: u64) -> Self {
        Self {
            max_depth: None,
            max_time: Some(Duration::from_millis(soft_ms)),
            hard_limit: Some(Duration::from_millis(hard_ms)),
            infinite: false,
        }
    }

    pub fn infinite() -> Self {
        Self {
            max_depth: Some(20), // Still have a max to prevent infinite loops
            max_time: None,
            hard_limit: None,
            infinite: true,
        }
    }
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of an iterative deepening search
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
    pub time_ms: u64,
}
