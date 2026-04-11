use super::negamax::negamax;
use crate::eval::EvalProvider;
use crate::search::features;
use crate::search::ordering::MoveHistory;
use crate::search::pruning::{calculate_lmr_reduction, should_reduce_lmr};
use crate::search::SearchStats;
use crate::tpt::TranspositionTable;
use crate::{Move, Position};

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub fn search_move<E: EvalProvider>(
    pos: &mut Position,
    eval: &E,
    eval_state: &mut E::State,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    pv_node: bool,
    tt: &mut TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
) -> i32 {
    if move_num == 0 || !features::PVS {
        let do_lmr = should_reduce_lmr(depth, move_num, in_check, gives_check, mv);

        if do_lmr {
            let reduction = calculate_lmr_reduction(depth, move_num, pv_node, mv);
            let reduced_depth = depth.saturating_sub(1 + reduction);
            let reduced_score = -negamax(
                pos,
                eval,
                eval_state,
                reduced_depth,
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
            );

            if reduced_score > alpha {
                return -negamax(
                    pos,
                    eval,
                    eval_state,
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
                );
            }

            return reduced_score;
        }

        return -negamax(
            pos,
            eval,
            eval_state,
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
        );
    }

    let do_lmr = should_reduce_lmr(depth, move_num, in_check, gives_check, mv);

    let mut score = if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_num, pv_node, mv);
        let reduced_depth = depth.saturating_sub(1 + reduction);

        -negamax(
            pos,
            eval,
            eval_state,
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
        )
    } else {
        -negamax(
            pos,
            eval,
            eval_state,
            depth - 1,
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
        )
    };

    if score > alpha && score < beta {
        score = -negamax(
            pos,
            eval,
            eval_state,
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
        );
    }

    score
}
