use crate::search::nnue_negamax::NNUESearcher;
use crate::search::types::{SearchLimits, SearchResult};
use board::Position;
use std::time::Instant;
use tpt::TranspositionTable;

/// Iterative deepening with NNUE
pub struct NNUEIterativeSearch {
    start_time: Instant,
    limits: SearchLimits,
    nodes: u64,
    tt: TranspositionTable,
}

impl NNUEIterativeSearch {
    pub fn new(limits: SearchLimits, tt_size_mb: usize) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            tt: TranspositionTable::new(tt_size_mb),
        }
    }

    pub fn with_tt(limits: SearchLimits, tt: TranspositionTable) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            tt,
        }
    }

    pub fn search(&mut self, position: &mut Position) -> SearchResult {
        let mut best_move = None;
        let mut best_score = 0;
        let mut completed_depth = 0;

        let max_depth = self.limits.max_depth.unwrap_or(20);

        for depth in 1..=max_depth {
            if self.should_stop(depth) {
                break;
            }

            // Search with NNUE
            let (mv, score) = position.search_nnue(depth, &mut self.tt);

            if let Some(m) = mv {
                best_move = Some(m);
                best_score = score;
                completed_depth = depth;

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

                if self.should_stop_after_depth(depth, score) {
                    break;
                }
            } else {
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

    fn should_stop(&self, next_depth: u8) -> bool {
        if let Some(max) = self.limits.max_depth {
            if next_depth > max {
                return true;
            }
        }

        if let Some(hard) = self.limits.hard_limit {
            if self.start_time.elapsed() >= hard {
                return true;
            }
        }

        if let Some(soft) = self.limits.max_time {
            let elapsed = self.start_time.elapsed();
            let estimated_next = elapsed * 3;
            if elapsed + estimated_next > soft {
                return true;
            }
        }

        false
    }

    fn should_stop_after_depth(&self, _depth: u8, score: i32) -> bool {
        if score.abs() > 50000 {
            return true;
        }

        if let Some(soft) = self.limits.max_time {
            if self.start_time.elapsed() >= soft {
                return true;
            }
        }

        if let Some(hard) = self.limits.hard_limit {
            if self.start_time.elapsed() >= hard {
                return true;
            }
        }

        false
    }

    pub fn tt(&self) -> &TranspositionTable {
        &self.tt
    }

    pub fn tt_mut(&mut self) -> &mut TranspositionTable {
        &mut self.tt
    }

    pub fn into_tt(self) -> TranspositionTable {
        self.tt
    }
}

/// Extension trait for Position
pub trait NNUEIterativeSearcher {
    fn search_nnue_iterative(&mut self, limits: SearchLimits) -> SearchResult;
    fn search_nnue_iterative_with_tt(
        &mut self,
        limits: SearchLimits,
        tt: &mut TranspositionTable,
    ) -> SearchResult;
}

impl NNUEIterativeSearcher for Position {
    fn search_nnue_iterative(&mut self, limits: SearchLimits) -> SearchResult {
        let mut search = NNUEIterativeSearch::new(limits, 64);
        search.search(self)
    }

    fn search_nnue_iterative_with_tt(
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
            if should_stop_before_depth(&limits, &start_time, depth) {
                break;
            }

            let (mv, score) = self.search_nnue(depth, tt);

            if let Some(m) = mv {
                best_move = Some(m);
                best_score = score;
                completed_depth = depth;

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

                if should_stop_after_depth(&limits, &start_time, score) {
                    break;
                }
            } else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::nnue_eval::init_nnue;

    #[test]
    fn test_nnue_iterative() {
        init_nnue("assets/nn-04cf2b4ed1da.nnue").unwrap();

        let mut pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let limits = SearchLimits::from_depth(6);
        let result = pos.search_nnue_iterative(limits);

        assert!(result.best_move.is_some());
        println!("Result: {:?} at depth {}", result.best_move, result.depth);
    }
}
