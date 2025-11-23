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
            max_depth: Some(20),
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

/// Aspiration window configuration
struct AspirationWindow {
    /// Window size for depth 1-3
    initial_window: i32,
    /// Window size for depth 4+
    normal_window: i32,
    /// How much to expand when search fails outside window
    expansion_factor: i32,
}

impl AspirationWindow {
    fn new() -> Self {
        Self {
            initial_window: 50,    // Tight for early depths
            normal_window: 50,     // Standard window
            expansion_factor: 100, // Each retry: expand by 100cp
        }
    }

    fn get_window(&self, depth: u8) -> i32 {
        if depth <= 3 {
            self.initial_window
        } else {
            self.normal_window
        }
    }
}

/// Performs iterative deepening search with aspiration windows and transposition table
pub struct IterativeSearch {
    start_time: Instant,
    limits: SearchLimits,
    nodes: u64,
    tt: TranspositionTable,
    aspiration: AspirationWindow,
}

impl IterativeSearch {
    pub fn new(limits: SearchLimits, tt_size_mb: usize) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            tt: TranspositionTable::new(tt_size_mb),
            aspiration: AspirationWindow::new(),
        }
    }

    pub fn with_tt(limits: SearchLimits, tt: TranspositionTable) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            tt,
            aspiration: AspirationWindow::new(),
        }
    }

    /// Main iterative deepening loop with aspiration windows
    pub fn search(&mut self, position: &mut Position) -> SearchResult {
        let mut best_move = None;
        let mut best_score = 0;
        let mut completed_depth = 0;

        let max_depth = self.limits.max_depth.unwrap_or(20);

        for depth in 1..=max_depth {
            if self.should_stop(depth) {
                break;
            }

            // Search with aspiration window
            let (mv, score) = if depth <= 3 {
                // Depths 1-3: use full window (aspiration windows not worth it at shallow depths)
                position.search(depth, &mut self.tt)
            } else {
                // Depths 4+: use aspiration windows
                self.search_with_aspiration(position, depth, best_score)
            };

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

    /// Search with aspiration windows - retries with wider windows on failure
    fn search_with_aspiration(
        &mut self,
        position: &mut Position,
        depth: u8,
        prev_score: i32,
    ) -> (Option<Move>, i32) {
        let window = self.aspiration.get_window(depth);
        let mut alpha = prev_score - window;
        let mut beta = prev_score + window;

        let mut retries = 0;
        const MAX_RETRIES: usize = 4;

        loop {
            // Perform the search with current window
            let (best_move, score) = self.search_with_limits(position, depth, alpha, beta);

            // Check if search succeeded within the window
            if score > alpha && score < beta {
                // Success! Score is within the window
                return (best_move, score);
            }

            retries += 1;
            if retries >= MAX_RETRIES {
                // Give up on aspiration, return what we have
                return (best_move, score);
            }

            // Expand the window based on where we failed
            if score <= alpha {
                // Failed low: expand lower bound
                alpha = (alpha - self.aspiration.expansion_factor * (1 << retries) as i32)
                    .max(i32::MIN + 1);
            } else {
                // Failed high: expand upper bound
                beta =
                    (beta + self.aspiration.expansion_factor * (1 << retries) as i32).min(i32::MAX);
            }
        }
    }

    /// Helper to search with specific alpha-beta bounds
    fn search_with_limits(
        &mut self,
        position: &mut Position,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> (Option<Move>, i32) {
        use types::moves::MoveCollector;

        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut local_alpha = alpha;

        let mut collector = MoveCollector::new();
        position.generate_moves(&mut collector);

        if collector.len() == 0 {
            // No legal moves
            return (None, 0);
        }

        // Try TT move first if available
        let mut tt_move = None;
        if let Some(entry) = self.tt.probe(position.hash) {
            if entry.depth >= depth {
                match entry.bound {
                    tpt::Bound::Exact => return (Some(entry.best_move), entry.score),
                    tpt::Bound::Lower => local_alpha = local_alpha.max(entry.score),
                    tpt::Bound::Upper => {}
                }
            }
            tt_move = Some(entry.best_move);
        }

        // Search TT move first
        if let Some(tt_m) = tt_move {
            if collector.contains(tt_m) {
                let undo = position.make_move(tt_m);
                let mut score = -self.negamax_with_limits(position, depth - 1, -beta, -local_alpha);

                if score > 50000 && position.is_in_check() {
                    score += 1;
                }

                position.unmake_move(tt_m, undo);

                if score > best_score {
                    best_score = score;
                    best_move = Some(tt_m);
                    local_alpha = local_alpha.max(score);

                    if local_alpha >= beta {
                        self.tt
                            .store(position.hash, tt_m, best_score, depth, tpt::Bound::Lower);
                        return (best_move, best_score);
                    }
                }
            }
        }

        // Search remaining moves
        for i in 0..collector.len() {
            let m = collector[i];
            if Some(m) == tt_move {
                continue;
            }

            let undo = position.make_move(m);
            let mut score = -self.negamax_with_limits(position, depth - 1, -beta, -local_alpha);

            if score > 50000 {
                if position.is_in_check() {
                    score += 1;
                }
            }

            position.unmake_move(m, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
                local_alpha = local_alpha.max(score);

                if local_alpha >= beta {
                    self.tt
                        .store(position.hash, m, best_score, depth, tpt::Bound::Lower);
                    return (best_move, best_score);
                }
            }
        }

        let bound = if best_score <= alpha {
            tpt::Bound::Upper
        } else if best_score >= beta {
            tpt::Bound::Lower
        } else {
            tpt::Bound::Exact
        };

        if let Some(mv) = best_move {
            self.tt.store(position.hash, mv, best_score, depth, bound);
        }

        (best_move, best_score)
    }

    /// Negamax with alpha-beta bounds for aspiration window retries
    fn negamax_with_limits(
        &mut self,
        position: &mut Position,
        depth: u8,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        use crate::evaluation::evaluate::Evaluator;
        use types::moves::MoveCollector;

        let original_alpha = alpha;

        // TT probe
        if let Some(entry) = self.tt.probe(position.hash) {
            if entry.depth >= depth {
                match entry.bound {
                    tpt::Bound::Exact => return entry.score,
                    tpt::Bound::Lower => alpha = alpha.max(entry.score),
                    tpt::Bound::Upper => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                }

                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        if depth == 0 {
            return position.evaluate();
        }

        let mut collector = MoveCollector::new();
        position.generate_moves(&mut collector);

        if collector.len() == 0 {
            if position.is_in_check() {
                return -100000 - depth as i32;
            } else {
                return 0;
            }
        }

        let mut best_score = i32::MIN;
        let mut best_move = types::moves::Move::NULL;

        let tt_move = self.tt.probe(position.hash).map(|e| e.best_move);

        // Try TT move first
        if let Some(tt_m) = tt_move {
            if collector.contains(tt_m) {
                let undo = position.make_move(tt_m);
                let score = -self.negamax_with_limits(position, depth - 1, -beta, -alpha);
                position.unmake_move(tt_m, undo);

                if score > best_score {
                    best_score = score;
                    best_move = tt_m;
                }

                alpha = alpha.max(score);
                if alpha >= beta {
                    self.tt.store(
                        position.hash,
                        best_move,
                        best_score,
                        depth,
                        tpt::Bound::Lower,
                    );
                    return best_score;
                }
            }
        }

        for i in 0..collector.len() {
            let m = collector[i];
            if Some(m) == tt_move {
                continue;
            }

            let undo = position.make_move(m);
            let score = -self.negamax_with_limits(position, depth - 1, -beta, -alpha);
            position.unmake_move(m, undo);

            if score > best_score {
                best_score = score;
                best_move = m;
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= original_alpha {
            tpt::Bound::Upper
        } else if best_score >= beta {
            tpt::Bound::Lower
        } else {
            tpt::Bound::Exact
        };

        self.tt
            .store(position.hash, best_move, best_score, depth, bound);

        best_score
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
    fn search_iterative(&mut self, limits: SearchLimits) -> SearchResult {
        let mut search = IterativeSearch::new(limits, 64);
        search.search(self)
    }

    fn search_iterative_with_tt(
        &mut self,
        limits: SearchLimits,
        tt: &mut TranspositionTable,
    ) -> SearchResult {
        // Create a temporary TT and swap
        let temp_tt = TranspositionTable::new(64);
        let mut search = IterativeSearch::with_tt(limits, temp_tt);

        // Swap the provided TT into search
        std::mem::swap(&mut search.tt, tt);

        let result = search.search(self);

        // Swap back
        std::mem::swap(&mut search.tt, tt);
        result
    }
}
