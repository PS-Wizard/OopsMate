use super::alphabeta::root::search_root;
use super::ordering::MoveHistory;
use super::params::{ASPIRATION_DEPTH, INFINITY, MAX_MOVES};
use super::score::checkmate_score;
use super::{print_uci_info, should_stop_search, SearchInfo, SearchStats};
use crate::evaluate::{new_probe, EvalProbe};
use crate::tpt::TranspositionTable;
use crate::{Move, MoveCollector, Position};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub fn search_driver(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: &TranspositionTable,
    stop_signal: Arc<AtomicBool>,
    thread_id: usize,
) -> Option<SearchInfo> {
    let mut pos = pos.clone();
    let start_time = Instant::now();
    let mut probe = Box::new(new_probe(&pos));
    let mut stats = SearchStats::new(Some(stop_signal.clone()), start_time, max_time_ms);
    let mut history = MoveHistory::new();
    let mut best_score = 0;
    let mut completed_depth = 0;
    let is_master = thread_id == 0;

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    let mut best_move = Some(moves[0]);

    let start_depth = 1;

    for depth in start_depth..=max_depth {
        let depth_start = Instant::now();

        if stats.should_stop() {
            break;
        }

        let (iteration_best_score, iteration_best_move) = search_aspiration(
            &mut pos,
            &mut probe,
            depth,
            best_score,
            tt,
            &mut history,
            &mut stats,
            thread_id,
        );

        if stats.should_stop() {
            break;
        }

        best_move = Some(iteration_best_move);
        best_score = iteration_best_score;
        completed_depth = depth;

        if is_master {
            print_uci_info(
                depth,
                best_score,
                &stats,
                start_time,
                tt,
                &iteration_best_move,
            );
        }

        let current_depth_time = depth_start.elapsed().as_millis() as u64;

        if is_master {
            if should_stop_search(max_time_ms, start_time, current_depth_time) {
                stop_signal.store(true, Ordering::Relaxed);
                break;
            }

            if let Some(max_time) = max_time_ms {
                if start_time.elapsed().as_millis() as u64 >= max_time {
                    stop_signal.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    }

    best_move.map(|mv| SearchInfo {
        best_move: mv,
        score: best_score,
        depth: completed_depth,
        nodes: stats.nodes,
        time_ms: start_time.elapsed().as_millis() as u64,
        tt_hits: stats.tt_hits,
    })
}

#[inline(always)]
fn search_aspiration(
    pos: &mut Position,
    probe: &mut EvalProbe,
    depth: u8,
    prev_score: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    thread_id: usize,
) -> (i32, Move) {
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);

    if collector.as_slice().is_empty() {
        return if pos.is_in_check() {
            (checkmate_score(0), Move(0))
        } else {
            (0, Move(0))
        };
    }

    let mut moves = [Move(0); MAX_MOVES];
    let count = collector.len();
    moves[..count].copy_from_slice(&collector.as_slice()[..count]);
    let moves_slice = &mut moves[..count];

    if depth < ASPIRATION_DEPTH {
        return search_root(
            pos,
            probe,
            moves_slice,
            depth,
            -INFINITY,
            INFINITY,
            tt,
            history,
            stats,
            thread_id,
        );
    }

    let mut delta = match thread_id % 4 {
        0 => 25,
        1 => 50,
        2 => 100,
        _ => 200,
    };

    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        let (score, best_move) = search_root(
            pos,
            probe,
            moves_slice,
            depth,
            alpha,
            beta,
            tt,
            history,
            stats,
            thread_id,
        );

        if stats.should_stop() {
            return (score, best_move);
        }

        if score > alpha && score < beta {
            return (score, best_move);
        }

        if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = alpha.saturating_sub(delta);
            delta += delta / 2;
        } else if score >= beta {
            alpha = (alpha + beta) / 2;
            beta = beta.saturating_add(delta);
            delta += delta / 2;
        }

        if delta > 1000 {
            alpha = -INFINITY;
            beta = INFINITY;
        }
    }
}
