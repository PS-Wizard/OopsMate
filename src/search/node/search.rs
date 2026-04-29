use crate::eval::EvalProvider;
use crate::movegen::analyze;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::node::child::{search_with_lmr, search_with_pvs};
use crate::search::node::NodeState;
use crate::search::ordering::{MovePicker, TtMode};
use crate::search::params::INFINITY;
use crate::search::pruning::{
    can_use_futility_pruning, can_use_reverse_futility, get_futility_margin, get_rfp_margin,
    should_prune_futility, should_rfp_prune, try_iid, try_null_move_pruning, try_probcut,
    try_razoring,
};
use crate::search::qsearch::qsearch;
use crate::search::score::checkmate_score;
use crate::tpt::{Bound, NO_STATIC_EVAL};
use crate::{Move, Position};

#[inline(always)]
pub(crate) fn search_node<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mut depth: u8,
    mut alpha: i32,
    beta: i32,
    node: NodeState,
) -> i32 {
    if depth == 0 {
        return qsearch(pos, ctx, alpha, beta, node.ply as u8);
    }

    ctx.stats.nodes += 1;
    let alpha_start = alpha;

    if ctx.stats.should_stop() {
        return 0;
    }

    if pos.is_fifty_move_draw() || (node.ply > 0 && pos.is_repetition()) {
        return 0;
    }

    let hash = pos.hash();
    let tt_entry = if features::TT_CUTOFFS {
        ctx.tt.probe(hash, node.ply as u8)
    } else {
        None
    };

    let tt_move = if let Some(entry) = tt_entry {
        if entry.depth >= depth && node.excluded_move.is_none() {
            ctx.stats.tt_hits += 1;
            match entry.bound {
                Bound::Exact => return entry.score,
                Bound::Lower if entry.score >= beta => return entry.score,
                Bound::Upper if entry.score <= alpha => return entry.score,
                _ => {}
            }
        }
        Some(entry.best_move)
    } else {
        None
    };

    let analysis = analyze(pos);
    let in_check = analysis.in_check();
    let raw_static_eval = if let Some(entry) = tt_entry {
        if entry.static_eval != NO_STATIC_EVAL {
            i32::from(entry.static_eval)
        } else {
            ctx.eval.eval(pos, &mut ctx.eval_state)
        }
    } else {
        ctx.eval.eval(pos, &mut ctx.eval_state)
    };

    if let Some(score) = try_probcut(pos, ctx, depth, beta, node.pv_node, in_check, node.ply) {
        return score;
    }

    if let Some(score) = try_razoring(
        pos,
        ctx,
        depth,
        alpha,
        in_check,
        node.pv_node,
        raw_static_eval,
    ) {
        return score;
    }

    if can_use_reverse_futility(depth, in_check, node.pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(raw_static_eval, beta, rfp_margin) {
            return raw_static_eval - rfp_margin;
        }
    }

    if let Some(score) = try_null_move_pruning(
        pos,
        ctx,
        depth,
        beta,
        node.allow_null,
        in_check,
        raw_static_eval,
        node.ply,
    ) {
        return score;
    }

    if features::SINGULAR_EXTENSIONS
        && !node.pv_node
        && node.excluded_move.is_none()
        && depth >= 8
        && tt_move.is_some()
        && !in_check
    {
        if let Some(entry) = tt_entry {
            if entry.depth >= depth.saturating_sub(3) && entry.bound == Bound::Lower {
                let singular_beta = entry.score.saturating_sub(depth as i32 * 2);
                let singular_depth = depth / 2;

                let score = search_node(
                    pos,
                    ctx,
                    singular_depth,
                    singular_beta - 1,
                    singular_beta,
                    NodeState::new(node.allow_null, false, tt_move, node.ply),
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
        ctx,
        depth,
        alpha,
        beta,
        node.pv_node,
        tt_move.is_some(),
        in_check,
        node.ply,
    );
    let tt_move = tt_move.or(iid_move);

    let use_futility = can_use_futility_pruning(depth, in_check, node.pv_node, alpha, beta);
    let (futility_static_eval, futility_margin) = if use_futility {
        (raw_static_eval, get_futility_margin(depth))
    } else {
        (0, 0)
    };

    let mut picker = MovePicker::new(
        &analysis,
        tt_move,
        if in_check {
            TtMode::ValidateInStage
        } else {
            TtMode::BlindTrust
        },
        true,
    );

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);
    let mut saw_legal_move = false;
    let mut move_index = 0usize;

    while let Some(mv) = picker.next_move(pos, &analysis, Some(&ctx.history), node.ply) {
        if ctx.stats.should_stop() {
            break;
        }

        saw_legal_move = true;

        if let Some(excluded) = node.excluded_move {
            if mv.0 == excluded.0 {
                move_index += 1;
                continue;
            }
        }

        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);
        let gives_check = pos.is_in_check();
        let check_extension = if features::CHECK_EXTENSIONS && gives_check {
            1
        } else {
            0
        };

        if use_futility
            && move_index > 0
            && should_prune_futility(
                mv,
                gives_check,
                futility_static_eval,
                alpha,
                futility_margin,
            )
        {
            pos.unmake_move(mv);
            ctx.eval.update_on_undo(&mut ctx.eval_state, delta);
            move_index += 1;
            continue;
        }

        let score = if !features::PVS {
            search_with_lmr(
                pos,
                ctx,
                mv,
                depth,
                alpha,
                beta,
                move_index,
                in_check,
                gives_check,
                check_extension,
                node,
            )
        } else {
            let is_hash_move = tt_move.is_some_and(|tt_mv| mv.0 == tt_mv.0);
            search_with_pvs(
                pos,
                ctx,
                mv,
                depth,
                alpha,
                beta,
                move_index,
                in_check,
                gives_check,
                check_extension,
                node,
                is_hash_move,
            )
        };

        move_index += 1;

        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if ctx.stats.should_stop() {
            return 0;
        }

        if score >= beta {
            if !mv.is_capture() && !mv.is_promotion() {
                if features::KILLER_MOVES {
                    ctx.history.killers.store(node.ply, mv);
                }
                if features::HISTORY_HEURISTIC {
                    let bonus = (depth as i16 * depth as i16).min(400);
                    ctx.history
                        .history
                        .update(pos.side_to_move, mv.from(), mv.to(), bonus);
                }
            }

            if features::TT_CUTOFFS {
                ctx.tt.store(
                    hash,
                    node.ply as u8,
                    mv,
                    score,
                    static_eval_to_tt(raw_static_eval),
                    depth,
                    Bound::Lower,
                );
            }
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

    if !saw_legal_move {
        return if in_check {
            checkmate_score(node.ply)
        } else {
            0
        };
    }

    let bound = if best_score <= alpha_start {
        Bound::Upper
    } else {
        Bound::Exact
    };

    if features::TT_CUTOFFS {
        ctx.tt.store(
            hash,
            node.ply as u8,
            best_move,
            best_score,
            static_eval_to_tt(raw_static_eval),
            depth,
            bound,
        );
    }

    best_score
}

#[inline(always)]
fn static_eval_to_tt(score: i32) -> i16 {
    debug_assert!(score >= i16::MIN as i32 && score <= i16::MAX as i32);
    score as i16
}
