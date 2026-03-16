use super::iid::try_iid;
use crate::evaluate::{apply_move, evaluate_with_probe, undo_move, EvalProbe};
use crate::search::ordering::{pick_next_move, score_move, MoveHistory};
use crate::search::params::{INFINITY, MAX_MOVES};
use crate::search::pruning::*;
use crate::search::qsearch::qsearch;
use crate::search::score::{checkmate_score, score_from_tt, score_to_tt};
use crate::search::SearchStats;
use crate::tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND};
use crate::{Move, MoveCollector, Position};

#[allow(clippy::too_many_arguments)]
pub fn negamax(
    pos: &mut Position,
    probe: &mut EvalProbe,
    mut depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    allow_null: bool,
    pv_node: bool,
    _cut_node: bool,
    excluded_move: Option<Move>,
    ply: usize,
    thread_id: usize,
) -> i32 {
    stats.nodes += 1;
    let alpha_start = alpha;

    if stats.should_stop() {
        return 0;
    }

    if pos.is_fifty_move_draw() || (ply > 0 && pos.is_repetition()) {
        return 0;
    }

    let hash = pos.hash();
    let tt_entry = tt.probe(hash).map(|mut entry| {
        entry.score = score_from_tt(entry.score, ply);
        entry
    });
    let tt_move = if let Some(entry) = tt_entry {
        if entry.depth >= depth && excluded_move.is_none() {
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
    };

    if depth == 0 {
        return qsearch(pos, probe, alpha, beta, stats, 0);
    }
    let in_check = pos.is_in_check();
    let static_eval = evaluate_with_probe(pos, probe);

    if let Some(score) = try_probcut(
        pos, probe, depth, beta, pv_node, in_check, allow_null, tt, history, stats, ply, thread_id,
    ) {
        return score;
    }

    if let Some(score) = try_razoring(
        pos,
        probe,
        depth,
        alpha,
        in_check,
        pv_node,
        static_eval,
        stats,
    ) {
        return score;
    }

    if can_use_reverse_futility(depth, in_check, pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(static_eval, beta, rfp_margin) {
            return static_eval - rfp_margin;
        }
    }

    if let Some(score) = try_null_move_pruning(
        pos,
        probe,
        depth,
        beta,
        allow_null,
        in_check,
        static_eval,
        tt,
        history,
        stats,
        ply,
        thread_id,
    ) {
        return score;
    }

    if !pv_node && excluded_move.is_none() && depth >= 8 && tt_move.is_some() && !in_check {
        if let Some(entry) = tt_entry {
            if entry.depth >= depth.saturating_sub(3) && entry.flag == LOWER_BOUND {
                let singular_beta = entry.score.saturating_sub(depth as i32 * 2);
                let singular_depth = depth / 2;
                let score = negamax(
                    pos,
                    probe,
                    singular_depth,
                    singular_beta - 1,
                    singular_beta,
                    tt,
                    history,
                    stats,
                    allow_null,
                    false,
                    true,
                    tt_move,
                    ply,
                    thread_id,
                );

                if score < singular_beta {
                    depth += 1;
                } else if score >= beta {
                    return singular_beta;
                }
            }
        }
    }

    let iid_move = try_iid(
        pos,
        probe,
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
    let tt_move = tt_move.or(iid_move);

    let use_futility = can_use_futility_pruning(depth, in_check, pv_node, alpha, beta);
    let (static_eval, futility_margin) = if use_futility {
        let margin = get_futility_margin(depth);
        (static_eval, margin)
    } else {
        (0, 0)
    };

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return if in_check { checkmate_score(ply) } else { 0 };
    }

    let move_count = moves.len();
    let mut move_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];

    for i in 0..move_count {
        move_list[i] = moves[i];
        scores[i] = score_move(moves[i], pos, tt_move, Some(history), ply);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    for i in 0..move_count {
        if stats.should_stop() {
            break;
        }

        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        if let Some(excluded) = excluded_move {
            if mv.0 == excluded.0 {
                continue;
            }
        }

        let delta = apply_move(probe, pos, mv);
        pos.make_move(mv);
        let gives_check = pos.is_in_check();
        let check_extension = if gives_check { 1 } else { 0 };

        if use_futility
            && i > 0
            && should_prune_futility(mv, gives_check, static_eval, alpha, futility_margin)
        {
            pos.unmake_move(mv);
            undo_move(probe, delta);
            continue;
        }

        let score = if i == 0 {
            -negamax(
                pos,
                probe,
                depth - 1,
                -beta,
                -alpha,
                tt,
                history,
                stats,
                true,
                pv_node,
                false,
                None,
                ply + 1,
                thread_id,
            )
        } else {
            let is_hash_move = tt_move.is_some_and(|tt_mv| mv.0 == tt_mv.0);
            let mut s = if should_reduce_lmr(depth, i, in_check, gives_check, mv, thread_id)
                && !is_hash_move
            {
                let reduction = calculate_lmr_reduction(depth, i, pv_node, mv, thread_id);
                let reduced_depth = depth
                    .saturating_sub(1 + reduction)
                    .saturating_add(check_extension);

                -negamax(
                    pos,
                    probe,
                    reduced_depth,
                    -alpha - 1,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    false,
                    true,
                    None,
                    ply + 1,
                    thread_id,
                )
            } else {
                -negamax(
                    pos,
                    probe,
                    depth - 1 + check_extension,
                    -alpha - 1,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    false,
                    true,
                    None,
                    ply + 1,
                    thread_id,
                )
            };

            if s > alpha && s < beta {
                s = -negamax(
                    pos,
                    probe,
                    depth - 1 + check_extension,
                    -beta,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    pv_node,
                    false,
                    None,
                    ply + 1,
                    thread_id,
                );
            }

            s
        };
        pos.unmake_move(mv);
        undo_move(probe, delta);

        if stats.should_stop() {
            return 0;
        }

        if score >= beta {
            if !mv.is_capture() && !mv.is_promotion() {
                history.killers.store(ply, mv);
                let bonus = (depth as i16 * depth as i16).min(400);
                history
                    .history
                    .update(pos.side_to_move, mv.from(), mv.to(), bonus);
            }

            tt.store(hash, mv, score_to_tt(beta, ply), depth, LOWER_BOUND);
            return beta;
        }

        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }
    }

    let flag = if best_score <= alpha_start {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(hash, best_move, score_to_tt(best_score, ply), depth, flag);

    best_score
}
