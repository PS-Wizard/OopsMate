//! Search entrypoints and internal search modules.

mod api;
mod context;
mod features;
mod heuristics;
mod limits;
mod node;
mod ordering;
mod output;
mod params;
pub(crate) mod qsearch;
mod root;
mod score;

pub use api::{search, search_with_eval, SearchInfo};
pub use heuristics::init_lmr;
pub use limits::SearchLimits;

pub(crate) use api::search_with_stop_signal;

#[cfg(test)]
mod tests {
    use super::limits::should_stop_next_iteration;
    use super::score::{checkmate_score, score_from_tt, score_to_tt};
    use super::*;
    use crate::tpt::TranspositionTable;
    use crate::Position;
    use std::thread;
    use std::time::Instant;

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

            let depth = 18;
            let pos = Position::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            )
            .unwrap_or_default();

            let mut tt = TranspositionTable::new_mb(512);
            init_lmr();

            println!("Starting iterative deepening search to depth {}...", depth);
            let start = std::time::Instant::now();

            let result = search(&pos, depth, None, &mut tt);

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

    #[test]
    fn movetime_waits_until_near_hard_limit() {
        let start = Instant::now() - std::time::Duration::from_millis(350);
        assert!(!should_stop_next_iteration(
            SearchLimits::movetime(490),
            start,
            100
        ));
        assert!(should_stop_next_iteration(
            SearchLimits::movetime(490),
            start,
            150
        ));
    }

    #[test]
    fn clock_limits_remain_predictive() {
        let start = Instant::now() - std::time::Duration::from_millis(220);
        assert!(should_stop_next_iteration(
            SearchLimits::clock(300, 360),
            start,
            50
        ));
        assert!(!should_stop_next_iteration(
            SearchLimits::clock(400, 500),
            start,
            50
        ));
    }
}
