use super::{SearchInfo, SearchStats, print_uci_info, should_stop_search};
use super::alphabeta::search_root;
use super::ordering::MoveHistory;
use super::params::{ASPIRATION_DEPTH, INFINITY, MAX_MOVES};
use crate::{Move, MoveCollector, Position};
use crate::tpt::TranspositionTable;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Driver for the iterative deepening loop
pub fn search_driver(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: &TranspositionTable,
    stop_signal: Arc<AtomicBool>,
    thread_id: usize,
) -> Option<SearchInfo> {
    let start_time = Instant::now();
    let mut stats = SearchStats::new(Some(stop_signal.clone()));
    let mut history = MoveHistory::new();
    let mut best_move = None;
    let mut best_score = 0; // Initialize to 0 for first aspiration window
    let is_master = thread_id == 0;

    // Generate moves once to check for game over
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    // DIVERSIFICATION: Depth Offset
    // Some threads can start searching at higher depths to avoid redundant low-depth work
    // But usually we want all to start at 1 to fill TT.
    // Let's keep it simple: all start at 1, but maybe helpers skip printing info.
    let start_depth = 1;

    // Iterative deepening loop
    for depth in start_depth..=max_depth {
        let depth_start = Instant::now();

        // Check stop signal before starting depth
        if stop_signal.load(Ordering::Relaxed) {
            break;
        }

        // search_aspiration handles both shallow (full window) and deep (aspiration) searches
        let (iteration_best_score, iteration_best_move) =
            search_aspiration(pos, depth, best_score, tt, &mut history, &mut stats, thread_id);

        // Check if we stopped during search
        if stop_signal.load(Ordering::Relaxed) {
            break;
        }

        // Update best move for this depth
        best_move = Some(iteration_best_move);
        best_score = iteration_best_score;

        // Print UCI info (Master only)
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

        // Time management - break if we're likely to run out of time (Master only)
        if is_master {
            if should_stop_search(max_time_ms, start_time, current_depth_time) {
                stop_signal.store(true, Ordering::Relaxed);
                break;
            }

            // Check time during search
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
        depth: max_depth,
        nodes: stats.nodes,
        time_ms: start_time.elapsed().as_millis() as u64,
        tt_hits: stats.tt_hits,
    })
}

// ============================================================================
//  ASPIRATION WINDOWS
// ============================================================================

#[inline(always)]
fn search_aspiration(
    pos: &Position,
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
            (-49_000 - depth as i32, Move(0))
        } else {
            (0, Move(0))
        };
    }

    let mut moves = [Move(0); MAX_MOVES];
    let count = collector.len();
    for i in 0..count {
        moves[i] = collector.as_slice()[i];
    }
    let moves_slice = &mut moves[..count];

    // If shallow, just search full window
    if depth < ASPIRATION_DEPTH {
        return search_root(
            pos,
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

    // Aspiration Loop
    // DIVERSIFICATION: Aspiration Window Variation
    let mut delta = match thread_id % 4 {
        0 => 25,   // Narrow (master) - standard
        1 => 50,   // Medium
        2 => 100,  // Wide
        _ => 200,  // Very wide
    };

    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        // We pass 'depth' to let search_root know if should use the optimization
        let (score, best_move) =
            search_root(pos, moves_slice, depth, alpha, beta, tt, history, stats, thread_id);

        if stats.should_stop() {
            return (score, best_move);
        }

        // Success Inside window
        if score > alpha && score < beta {
            return (score, best_move);
        }

        // Fail Low: Score <= Alpha
        if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = alpha.saturating_sub(delta);
            delta += delta / 2;
        }
        // Fail High: Score >= Beta
        else if score >= beta {
            alpha = (alpha + beta) / 2;
            beta = beta.saturating_add(delta);
            delta += delta / 2;
        }

        // If window gets too huge, give up and search infinite
        if delta > 1000 {
            alpha = -INFINITY;
            beta = INFINITY;
        }
    }
}