use super::SearchStats;
use super::ordering::{score_move, pick_next_move, MoveHistory};
use super::pruning::*;
use super::params::{INFINITY, MATE_VALUE, MAX_MOVES, IID_MIN_DEPTH};
use crate::evaluate::evaluate;
use crate::qsearch::qsearch;
use crate::tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND};
use crate::{Move, MoveCollector, Position};

// ============================================================================
//  NEGAMAX
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn negamax(
    pos: &Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    allow_null: bool,
    pv_node: bool,
    ply: usize,
    thread_id: usize,
) -> i32 {
    stats.nodes += 1;

    // Check stop signal
    if stats.should_stop() {
        return 0; // Return neutral score or handle abort
    }

    let hash = pos.hash();

    // Probe transposition table - check for early cutoff
    let tt_move = {
        let tt_entry = tt.probe(hash);
        if let Some(entry) = tt_entry {
            if entry.depth >= depth {
                stats.tt_hits += 1;
                match entry.flag {
                    EXACT => return entry.score,
                    LOWER_BOUND if entry.score >= beta => return entry.score,
                    UPPER_BOUND if entry.score <= alpha => return entry.score,
                    _ => {}
                }
            }
            Some(entry.best_move)
        } else {
            None
        }
    };

    // Base case - drop into quiescence search
    if depth == 0 {
        return qsearch(pos, alpha, beta, stats, 0);
    }
    let in_check = pos.is_in_check();

    // Static evaluation for pruning decisions
    let static_eval = evaluate(pos);

    // PROBCUT
    if let Some(score) = try_probcut(
        pos, depth, beta, pv_node, in_check, allow_null, tt, history, stats, ply, thread_id
    ) {
        return score;
    }

    // RAZORING
    if let Some(score) = try_razoring(pos, depth, alpha, in_check, pv_node, static_eval, stats) {
        return score;
    }

    // Reverse futility pruning
    if can_use_reverse_futility(depth, in_check, pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(static_eval, beta, rfp_margin) {
            return static_eval - rfp_margin;
        }
    }

    // Try null move pruning
    if let Some(score) = try_null_move_pruning(
        pos, depth, beta, allow_null, in_check, tt, history, stats, ply, thread_id
    ) {
        return score;
    }

    // Internal Iterative Deepening
    let iid_move = try_iid(
        pos,
        depth,
        alpha,
        beta,
        pv_node,
        tt_move.is_some(),
        in_check,
        tt,
        history,
        stats,
        ply,
        thread_id,
    );

    // Use IID move if we found one and don't have a TT move
    let tt_move = tt_move.or(iid_move);

    // Futility pruning setup
    let use_futility = can_use_futility_pruning(depth, in_check, pv_node, alpha, beta);
    let (static_eval, futility_margin) = if use_futility {
        let margin = get_futility_margin(depth);
        (static_eval, margin)
    } else {
        (0, 0)
    };

    // Generate and order moves
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // Checkmate / Stalemate detection
    if moves.is_empty() {
        return if in_check {
            -MATE_VALUE - (depth as i32)
        } else {
            0
        };
    }

    let move_count = moves.len();
    let mut move_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];

    // Score moves for ordering
    for i in 0..move_count {
        move_list[i] = moves[i];
        scores[i] = score_move(moves[i], pos, tt_move, Some(history), ply);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    // Search all moves
    for i in 0..move_count {
        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        let new_pos = pos.make_move(&mv);
        let gives_check = new_pos.is_in_check();
        let check_extension = if gives_check { 1 } else { 0 };

        // Futility pruning
        if use_futility && i > 0 {
            if should_prune_futility(mv, gives_check, static_eval, alpha, futility_margin) {
                continue;
            }
        }

        let score = if i == 0 {
            // First move: full depth, full window
            -negamax(
                &new_pos,
                depth - 1,
                -beta,
                -alpha,
                tt,
                history,
                stats,
                true,
                pv_node,
                ply + 1,
                thread_id,
            )
        } else {
            // PVS for subsequent moves
            let is_hash_move = tt_move.map_or(false, |tt_mv| mv.0 == tt_mv.0);
            let mut s = if should_reduce_lmr(depth, i, in_check, gives_check, mv, thread_id) & !is_hash_move {
                let reduction = calculate_lmr_reduction(depth, i, pv_node, mv, thread_id);
                let reduced_depth = depth
                    .saturating_sub(1 + reduction)
                    .saturating_add(check_extension);

                -negamax(
                    &new_pos,
                    reduced_depth,
                    -alpha - 1,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    false,
                    ply + 1,
                    thread_id,
                )
            } else {
                // Null window search
                -negamax(
                    &new_pos,
                    depth - 1 + check_extension,
                    -alpha - 1,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    false,
                    ply + 1,
                    thread_id,
                )
            };

            if s > alpha && s < beta {
                s = -negamax(
                    &new_pos,
                    depth - 1 + check_extension,
                    -beta,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    pv_node,
                    ply + 1,
                    thread_id,
                );
            }

            s
        };

        if stats.should_stop() {
            return 0; // Abort
        }

        // Beta cutoff
        if score >= beta {
            if !mv.is_capture() && !mv.is_promotion() {
                history.killers.store(ply, mv);
                let bonus = (depth as i16 * depth as i16).min(400);
                history.history.update(pos.side_to_move, mv.from(), mv.to(), bonus);
            }

            tt.store(hash, mv, beta, depth, LOWER_BOUND);
            return beta;
        }

        // Update best move
        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }
    }

    let flag = if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(hash, best_move, best_score, depth, flag);

    best_score
}

// ============================================================================
//  PVS
// ============================================================================

#[inline(always)]
pub fn search_move(
    pos: &Position,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    pv_node: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
    thread_id: usize,
) -> i32 {
    if move_num == 0 {
        return -negamax(
            &*pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            pv_node,
            ply + 1,
            thread_id,
        );
    }

    let do_lmr = should_reduce_lmr(depth, move_num, in_check, gives_check, mv, thread_id);

    let mut score = if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_num, pv_node, mv, thread_id);
        let reduced_depth = depth.saturating_sub(1 + reduction);

        -negamax(
            &*pos,
            reduced_depth,
            -alpha - 1,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            false,
            ply + 1,
            thread_id,
        )
    } else {
        -negamax(
            &*pos,
            depth - 1,
            -alpha - 1,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            false,
            ply + 1,
            thread_id,
        )
    };

    if score > alpha && score < beta {
        score = -negamax(
            &*pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            pv_node,
            ply + 1,
            thread_id,
        );
    }

    score
}

#[inline(always)]
pub fn search_root(
    pos: &Position,
    moves: &mut [Move],
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    thread_id: usize,
) -> (i32, Move) {
    let in_check = pos.is_in_check();
    let tt_move = tt.probe(pos.hash()).map(|e| e.best_move);
    let move_count = moves.len();
    let mut scores = [0i32; MAX_MOVES];

    // Score moves
    for i in 0..move_count {
        scores[i] = score_move(moves[i], pos, tt_move, Some(history), 0);
    }

    // DIVERSIFICATION: Perturb scores for helper threads
    if thread_id > 0 && move_count > 1 {
        for i in 0..move_count {
             // If it's not the TT move
             if scores[i] != super::ordering::SCORE_TT_MOVE {
                 let noise = ((i + thread_id) * 987654321) % 4000;
                 scores[i] = scores[i].saturating_add(noise as i32);
             }
        }
    }

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    for i in 0..move_count {
        pick_next_move(moves, &mut scores, i);
        let mv = moves[i];
        let newpos = pos.make_move(&mv);
        let gives_check = newpos.is_in_check();

        let score = if i == 0 {
            search_move(
                &newpos,
                mv,
                depth,
                alpha,
                beta,
                i,
                in_check,
                gives_check,
                true,
                tt,
                history,
                stats,
                0,
                thread_id,
            )
        } else {
            // PVS for other moves
            let s = search_move(
                &newpos,
                mv,
                depth,
                alpha,
                alpha + 1,
                i,
                in_check,
                gives_check,
                true,
                tt,
                history,
                stats,
                0,
                thread_id,
            );
            if s > alpha && s < beta {
                search_move(
                    &newpos,
                    mv,
                    depth,
                    alpha,
                    beta,
                    i,
                    in_check,
                    gives_check,
                    true,
                    tt,
                    history,
                    stats,
                    0,
                    thread_id,
                )
            } else {
                s
            }
        };

        if stats.should_stop() {
            return (best_score, best_move);
        }

        // crafty's optimization
        if i == 0 && score <= alpha {
            return (score, mv);
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    break; // Beta Cutoff
                }
            }
        }
    }

    // Store result
    let flag = if best_score >= beta {
        LOWER_BOUND
    } else if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(pos.hash(), best_move, best_score, depth, flag);

    (best_score, best_move)
}

// ============================================================================
//  INTERNAL ITERATIVE DEEPENING (IID)
// ============================================================================

#[inline(always)]
fn iid_reduction(depth: u8, pv_node: bool) -> u8 {
    if pv_node {
        (depth / 4).max(2)
    } else {
        (depth / 3).max(1)
    }
}

#[inline(always)]
pub fn try_iid(
    pos: &crate::Position,
    depth: u8,
    alpha: i32,
    beta: i32,
    pv_node: bool,
    has_tt_move: bool,
    in_check: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
    thread_id: usize,
) -> Option<Move> {
    if has_tt_move || depth < IID_MIN_DEPTH || in_check {
        return None;
    }

    if !pv_node && depth < IID_MIN_DEPTH + 2 {
        return None;
    }

    let reduction = iid_reduction(depth, pv_node);
    let iid_depth = depth.saturating_sub(reduction);

    negamax(
        pos, iid_depth, alpha, beta, tt, history, stats, true, 
        pv_node, ply, thread_id,
    );

    tt.probe(pos.hash()).map(|entry| entry.best_move)
}