use crate::search::negamax::Searcher;
use board::Position;
use std::time::{Duration, Instant};
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

/// Result of an iterative deepening search
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
    pub time_ms: u64,
}

/// Performs iterative deepening search
pub struct IterativeSearch {
    start_time: Instant,
    limits: SearchLimits,
    nodes: u64,
}

impl IterativeSearch {
    pub fn new(limits: SearchLimits) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
        }
    }

    /// Main iterative deepening loop
    pub fn search(&mut self, position: &mut Position) -> SearchResult {
        let mut best_move = None;
        let mut best_score = 0;
        let mut completed_depth = 0;

        let max_depth = self.limits.max_depth.unwrap_or(20);

        for depth in 1..=max_depth {
            // Check time before starting new depth
            if self.should_stop(depth) {
                break;
            }

            let (mv, score) = position.search(depth);

            if let Some(m) = mv {
                best_move = Some(m);
                best_score = score;
                completed_depth = depth;

                // Print info for UCI
                let elapsed = self.start_time.elapsed().as_millis() as u64;
                let nps = if elapsed > 0 {
                    (self.nodes * 1000) / elapsed
                } else {
                    0
                };

                println!(
                    "info depth {} score cp {} nodes {} nps {} time {} pv {}",
                    depth, score, self.nodes, nps, elapsed, m
                );

                // Check if we should stop after completing this depth
                if self.should_stop_after_depth(depth, score) {
                    break;
                }
            } else {
                // No legal moves
                break;
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            depth: completed_depth,
            nodes: self.nodes,
            time_ms: self.start_time.elapsed().as_millis() as u64,
        }
    }

    /// Check if we should stop before starting a new depth
    fn should_stop(&self, next_depth: u8) -> bool {
        // Already at max depth
        if let Some(max) = self.limits.max_depth {
            if next_depth > max {
                return true;
            }
        }

        // Check hard time limit
        if let Some(hard) = self.limits.hard_limit {
            if self.start_time.elapsed() >= hard {
                return true;
            }
        }

        // Check soft time limit with branching factor estimate
        // Don't start a new depth if we probably won't finish it
        if let Some(soft) = self.limits.max_time {
            let elapsed = self.start_time.elapsed();
            // Assume next depth takes ~3x as long (conservative branching factor)
            let estimated_next = elapsed * 3;
            if elapsed + estimated_next > soft {
                return true;
            }
        }

        false
    }

    /// Check if we should stop after completing a depth
    fn should_stop_after_depth(&self, _depth: u8, score: i32) -> bool {
        // Found a mate, no need to search deeper
        if score.abs() > 50000 {
            return true;
        }

        // Check if we've used our soft time
        if let Some(soft) = self.limits.max_time {
            if self.start_time.elapsed() >= soft {
                return true;
            }
        }

        // Check hard limit
        if let Some(hard) = self.limits.hard_limit {
            if self.start_time.elapsed() >= hard {
                return true;
            }
        }

        false
    }
}

/// Extension trait to add iterative deepening to Position
pub trait IterativeSearcher {
    fn search_iterative(&mut self, limits: SearchLimits) -> SearchResult;
}

impl IterativeSearcher for Position {
    fn search_iterative(&mut self, limits: SearchLimits) -> SearchResult {
        let mut search = IterativeSearch::new(limits);
        search.search(self)
    }
}
