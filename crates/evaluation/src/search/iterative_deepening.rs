use crate::search::negamax::Searcher;
use board::Position;
use std::time::{Duration, Instant};
use tpt::TranspositionTable;
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

/// Performs iterative deepening search with transposition table
pub struct IterativeSearch {
    start_time: Instant,
    limits: SearchLimits,
    nodes: u64,
    tt: TranspositionTable,
}

impl IterativeSearch {
    pub fn new(limits: SearchLimits, tt_size_mb: usize) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            tt: TranspositionTable::new(tt_size_mb),
        }
    }

    /// Create with existing transposition table
    pub fn with_tt(limits: SearchLimits, tt: TranspositionTable) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            tt,
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

            // Search with TT
            let (mv, score) = position.search(depth, &mut self.tt);

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

    /// Get reference to the transposition table (for reuse across searches)
    pub fn tt(&self) -> &TranspositionTable {
        &self.tt
    }

    /// Get mutable reference to the transposition table
    pub fn tt_mut(&mut self) -> &mut TranspositionTable {
        &mut self.tt
    }

    /// Consume and return the transposition table
    pub fn into_tt(self) -> TranspositionTable {
        self.tt
    }
}

/// Extension trait to add iterative deepening to Position
pub trait IterativeSearcher {
    fn search_iterative(&mut self, limits: SearchLimits) -> SearchResult;
    fn search_iterative_with_tt(
        &mut self,
        limits: SearchLimits,
        tt: &mut TranspositionTable,
    ) -> SearchResult;
}

impl IterativeSearcher for Position {
    /// Search with a new transposition table (64MB default)
    fn search_iterative(&mut self, limits: SearchLimits) -> SearchResult {
        let mut search = IterativeSearch::new(limits, 64);
        search.search(self)
    }

    /// Search with an existing transposition table (more efficient for repeated searches)
    fn search_iterative_with_tt(
        &mut self,
        limits: SearchLimits,
        tt: &mut TranspositionTable,
    ) -> SearchResult {
        let mut best_move = None;
        let mut best_score = 0;
        let mut completed_depth = 0;
        let nodes = 0u64;

        let max_depth = limits.max_depth.unwrap_or(20);
        let start_time = Instant::now();

        for depth in 1..=max_depth {
            // Check time before starting new depth
            if should_stop_before_depth(&limits, &start_time, depth) {
                break;
            }

            // Search with TT
            let (mv, score) = self.search(depth, tt);

            if let Some(m) = mv {
                best_move = Some(m);
                best_score = score;
                completed_depth = depth;

                // Print info for UCI
                let elapsed = start_time.elapsed().as_millis() as u64;
                let nps = if elapsed > 0 {
                    (nodes * 1000) / elapsed
                } else {
                    0
                };

                println!(
                    "info depth {} score cp {} nodes {} nps {} time {} pv {}",
                    depth, score, nodes, nps, elapsed, m
                );

                // Check if we should stop after completing this depth
                if should_stop_after_depth(&limits, &start_time, score) {
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
            nodes,
            time_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}

// Helper functions for time management
fn should_stop_before_depth(limits: &SearchLimits, start_time: &Instant, next_depth: u8) -> bool {
    if let Some(max) = limits.max_depth {
        if next_depth > max {
            return true;
        }
    }

    if let Some(hard) = limits.hard_limit {
        if start_time.elapsed() >= hard {
            return true;
        }
    }

    if let Some(soft) = limits.max_time {
        let elapsed = start_time.elapsed();
        let estimated_next = elapsed * 3;
        if elapsed + estimated_next > soft {
            return true;
        }
    }

    false
}

fn should_stop_after_depth(limits: &SearchLimits, start_time: &Instant, score: i32) -> bool {
    if score.abs() > 50000 {
        return true;
    }

    if let Some(soft) = limits.max_time {
        if start_time.elapsed() >= soft {
            return true;
        }
    }

    if let Some(hard) = limits.hard_limit {
        if start_time.elapsed() >= hard {
            return true;
        }
    }

    false
}
