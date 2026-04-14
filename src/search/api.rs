use super::limits::SearchLimits;
use super::root::run_search;
use crate::{eval::EvalProvider, tpt::TranspositionTable, Move, Position};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Result returned by a completed search.
pub struct SearchInfo {
    /// Best move found at the completed search depth.
    pub best_move: Move,
    /// Score in centipawns or mate-score space.
    pub score: i32,
    /// Deepest fully completed root depth.
    pub depth: u8,
    /// Total nodes visited.
    pub nodes: u64,
    /// Elapsed time in milliseconds.
    pub time_ms: u64,
    /// Number of transposition table hits.
    pub tt_hits: u64,
}

/// Runs an iterative-deepening search from `pos` and returns the best completed result.
pub fn search(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: &mut TranspositionTable,
) -> Option<SearchInfo> {
    search_with_eval(
        pos,
        max_depth,
        SearchLimits::from_max_time(max_time_ms),
        tt,
        crate::eval::NnueProvider::new(),
    )
}

/// Runs an iterative-deepening search using the supplied evaluation provider.
pub fn search_with_eval<E: EvalProvider>(
    pos: &Position,
    max_depth: u8,
    limits: SearchLimits,
    tt: &mut TranspositionTable,
    eval: E,
) -> Option<SearchInfo> {
    let stop_signal = Arc::new(AtomicBool::new(false));
    search_with_stop_signal(pos, max_depth, limits, tt, stop_signal, eval)
}

pub(crate) fn search_with_stop_signal<E: EvalProvider>(
    pos: &Position,
    max_depth: u8,
    limits: SearchLimits,
    tt: &mut TranspositionTable,
    stop_signal: Arc<AtomicBool>,
    eval: E,
) -> Option<SearchInfo> {
    tt.new_search();
    run_search(pos, max_depth, limits, tt, stop_signal, &eval)
}
