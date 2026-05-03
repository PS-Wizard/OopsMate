use crate::eval::EvalProvider;
use crate::movegen::{analyze, generate_all_with_analysis};
use crate::search::api::SearchInfo;
use crate::search::context::SearchContext;
use crate::search::limits::{should_stop_next_iteration, SearchLimits};
use crate::search::output::print_uci_info;
use crate::search::root::aspiration::search_with_aspiration;
use crate::{MoveCollector, Position};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub(crate) fn run_search<E: EvalProvider>(
    pos: &Position,
    max_depth: u8,
    limits: SearchLimits,
    tt: &mut crate::tpt::TranspositionTable,
    stop_signal: Arc<AtomicBool>,
    eval: &E,
) -> Option<SearchInfo> {
    let mut pos = pos.clone();
    let start_time = Instant::now();
    let mut ctx = SearchContext::new(
        &pos,
        eval,
        tt,
        stop_signal.clone(),
        limits.hard_time_ms(),
        start_time,
    );

    let mut collector = MoveCollector::new();
    let analysis = analyze(&pos);
    generate_all_with_analysis(&pos, &analysis, &mut collector);
    let moves = collector.as_slice();
    if moves.is_empty() {
        return None;
    }

    let mut best_move = Some(moves[0]);
    let mut best_score = 0;
    let mut completed_depth = 0;

    for depth in 1..=max_depth {
        let depth_start = Instant::now();
        if ctx.stats.should_stop() {
            break;
        }

        let (iteration_best_score, iteration_best_move) =
            search_with_aspiration(&mut pos, &mut ctx, depth, best_score);

        if ctx.stats.should_stop() {
            break;
        }

        best_move = Some(iteration_best_move);
        best_score = iteration_best_score;
        completed_depth = depth;

        print_uci_info(depth, best_score, &ctx.stats, ctx.tt, &iteration_best_move);

        let current_depth_time = depth_start.elapsed().as_millis() as u64;
        if should_stop_next_iteration(limits, start_time, current_depth_time) {
            stop_signal.store(true, Ordering::Relaxed);
            break;
        }

        if let Some(max_time) = limits.hard_time_ms() {
            if start_time.elapsed().as_millis() as u64 >= max_time {
                stop_signal.store(true, Ordering::Relaxed);
                break;
            }
        }
    }

    if completed_depth == 0 {
        if let Some(mv) = best_move {
            print_uci_info(0, best_score, &ctx.stats, ctx.tt, &mv);
        }
    }

    best_move.map(|mv| SearchInfo {
        best_move: mv,
        score: best_score,
        depth: completed_depth,
        nodes: ctx.stats.nodes,
        time_ms: ctx.stats.elapsed_ms(),
        tt_hits: ctx.stats.tt_hits,
    })
}
