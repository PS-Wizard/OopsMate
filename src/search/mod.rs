//! Search orchestration.
//!
//! This module owns iterative deepening, aspiration windows, pruning setup, and
//! the public `search` entry point used by the UCI driver.

mod alphabeta;
mod ordering;
mod parallel;
mod params;
mod pruning;
pub(crate) mod qsearch;
mod score;

/// Initializes late-move-reduction tables used by the search.
pub use pruning::init_lmr;

use crate::{tpt::TranspositionTable, Move, Position};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Mutable counters and stop state carried through a single search.
pub(crate) struct SearchStats {
    pub(crate) nodes: u64,
    pub(crate) tt_hits: u64,
    stop_signal: Option<Arc<AtomicBool>>,
}

impl SearchStats {
    pub(crate) fn new(stop_signal: Option<Arc<AtomicBool>>) -> Self {
        Self {
            nodes: 0,
            tt_hits: 0,
            stop_signal,
        }
    }

    #[inline(always)]
    pub(crate) fn should_stop(&self) -> bool {
        if self.nodes & 2047 == 0 {
            if let Some(signal) = &self.stop_signal {
                return signal.load(Ordering::Relaxed);
            }
        }
        false
    }
}

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
    tt: Arc<TranspositionTable>,
    threads: usize,
) -> Option<SearchInfo> {
    let stop_signal = Arc::new(AtomicBool::new(false));
    search_with_stop_signal(pos, max_depth, max_time_ms, tt, threads, stop_signal)
}

pub(crate) fn search_with_stop_signal(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: Arc<TranspositionTable>,
    threads: usize,
    stop_signal: Arc<AtomicBool>,
) -> Option<SearchInfo> {
    let threads = threads.max(1);

    tt.new_search();

    let mut handles = Vec::new();
    if threads > 1 {
        for id in 1..threads {
            let pos_clone = pos.clone();
            let tt_clone = tt.clone();
            let signal_clone = stop_signal.clone();

            handles.push(std::thread::spawn(move || {
                parallel::search_driver(&pos_clone, max_depth, None, &tt_clone, signal_clone, id)
            }));
        }
    }

    let info = parallel::search_driver(pos, max_depth, max_time_ms, &tt, stop_signal.clone(), 0);

    stop_signal.store(true, Ordering::Relaxed);

    for handle in handles {
        let _ = handle.join();
    }

    info
}

fn print_uci_info(
    depth: u8,
    score: i32,
    stats: &SearchStats,
    start_time: Instant,
    tt: &TranspositionTable,
    mv: &Move,
) {
    let elapsed = start_time.elapsed().as_millis() as u64;
    let nps = if elapsed > 0 {
        (stats.nodes * 1000) / elapsed
    } else {
        0
    };

    println!(
        "info depth {} score cp {} nodes {} time {} nps {} hashfull {} pv {}",
        depth,
        score,
        stats.nodes,
        elapsed,
        nps,
        tt.hashfull(),
        mv.to_uci()
    );

    let _ = std::io::stdout().flush();
}

fn should_stop_search(
    max_time_ms: Option<u64>,
    start_time: Instant,
    current_depth_time: u64,
) -> bool {
    if let Some(max_time) = max_time_ms {
        let elapsed_total = start_time.elapsed().as_millis() as u64;
        let time_remaining = max_time.saturating_sub(elapsed_total);
        let predicted_next_depth = current_depth_time.saturating_mul(4);

        predicted_next_depth > time_remaining || elapsed_total * 10 > max_time * 7
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::score::{checkmate_score, score_from_tt, score_to_tt};
    use super::*;
    use std::thread;

    fn run_with_large_stack<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked");
    }

    #[test]
    #[ignore = "Long-running search validation"]
    fn test_iterative_deepening() {
        run_with_large_stack(|| {
            use crate::search::init_lmr;
            use std::sync::Arc;

            let depth = 18;
            let pos = Position::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            )
            .unwrap_or_default();

            let tt = Arc::new(TranspositionTable::new_mb(512));
            init_lmr();

            println!("Starting iterative deepening search to depth {}...", depth);
            let start = std::time::Instant::now();

            let result = search(&pos, depth, None, tt.clone(), 1);

            let duration = start.elapsed();

            if let Some(info) = result {
                println!(
                    "Best move: {} (depth {}, score {}, nodes {}, time {:.3}s, nps {})",
                    info.best_move.to_uci(),
                    info.depth,
                    info.score,
                    info.nodes,
                    duration.as_secs_f64(),
                    if duration.as_millis() > 0 {
                        (info.nodes * 1000) / duration.as_millis() as u64
                    } else {
                        0
                    }
                );
            } else {
                println!("No move found");
            }
        });
    }

    #[test]
    fn tt_mate_scores_roundtrip_across_ply() {
        let mate_in_three = 48_997;
        let stored = score_to_tt(mate_in_three, 5);
        assert_eq!(score_from_tt(stored, 5), mate_in_three);

        let getting_mated = -48_994;
        let stored = score_to_tt(getting_mated, 6);
        assert_eq!(score_from_tt(stored, 6), getting_mated);
    }

    #[test]
    fn checkmate_scores_prefer_shorter_lines() {
        assert!(checkmate_score(1) < checkmate_score(5));
        assert_eq!(-checkmate_score(1), 48_999);
    }
}
