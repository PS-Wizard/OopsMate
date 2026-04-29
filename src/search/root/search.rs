use crate::eval::EvalProvider;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::node::{search_node, NodeState};
use crate::search::ordering::{pick_next_move, score_move};
use crate::search::params::{INFINITY, MAX_MOVES};
use crate::search::pruning::{calculate_lmr_reduction, should_reduce_lmr};
use crate::search::score::score_to_tt;
use crate::tpt::{EXACT, LOWER_BOUND, UPPER_BOUND};
use crate::{Move, Position};

#[derive(Clone, Copy)]
struct RootMoveState {
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    pv_node: bool,
    ply: usize,
}

#[inline(always)]
pub(crate) fn search_root<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    moves: &mut [Move],
    depth: u8,
    mut alpha: i32,
    beta: i32,
) -> (i32, Move) {
    let in_check = pos.is_in_check();
    let alpha_start = alpha;
    let tt_move = if features::TT_MOVE_ORDERING {
        ctx.tt.probe(pos.hash()).map(|entry| entry.best_move)
    } else {
        None
    };

    let move_count = moves.len();
    let mut scores = [0i32; MAX_MOVES];
    for i in 0..move_count {
        scores[i] = score_move(moves[i], pos, tt_move, Some(&ctx.history), 0);
    }

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    for i in 0..move_count {
        if ctx.stats.should_stop() {
            break;
        }

        pick_next_move(moves, &mut scores, i);
        let mv = moves[i];

        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);
        let gives_check = pos.is_in_check();

        let score = search_root_child(
            pos,
            ctx,
            mv,
            depth,
            alpha,
            beta,
            RootMoveState {
                move_num: i,
                in_check,
                gives_check,
                pv_node: true,
                ply: 0,
            },
        );

        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if ctx.stats.should_stop() {
            return (best_score, best_move);
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    break;
                }
            }
        }
    }

    let flag = if best_score >= beta {
        LOWER_BOUND
    } else if best_score <= alpha_start {
        UPPER_BOUND
    } else {
        EXACT
    };

    if features::TT_CUTOFFS {
        ctx.tt.store(
            pos.hash(),
            best_move,
            score_to_tt(best_score, 0),
            depth,
            flag,
        );
    }

    (best_score, best_move)
}

#[inline(always)]
fn search_root_child<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    state: RootMoveState,
) -> i32 {
    if state.move_num == 0 || !features::PVS {
        let do_lmr = should_reduce_lmr(
            depth,
            state.move_num,
            state.in_check,
            state.gives_check,
            mv,
        );

        if do_lmr {
            let reduction = calculate_lmr_reduction(depth, state.move_num, state.pv_node, mv);
            let reduced_depth = depth.saturating_sub(1 + reduction);
            let reduced_score = -search_node(
                pos,
                ctx,
                reduced_depth,
                -beta,
                -alpha,
                NodeState::new(true, state.pv_node, None, state.ply + 1),
            );

            if reduced_score > alpha {
                return -search_node(
                    pos,
                    ctx,
                    depth - 1,
                    -beta,
                    -alpha,
                    NodeState::new(true, state.pv_node, None, state.ply + 1),
                );
            }

            return reduced_score;
        }

        return -search_node(
            pos,
            ctx,
            depth - 1,
            -beta,
            -alpha,
            NodeState::new(true, state.pv_node, None, state.ply + 1),
        );
    }

    let do_lmr = should_reduce_lmr(
        depth,
        state.move_num,
        state.in_check,
        state.gives_check,
        mv,
    );

    let mut score = if do_lmr {
        let reduction = calculate_lmr_reduction(depth, state.move_num, state.pv_node, mv);
        let reduced_depth = depth.saturating_sub(1 + reduction);

        -search_node(
            pos,
            ctx,
            reduced_depth,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, state.ply + 1),
        )
    } else {
        -search_node(
            pos,
            ctx,
            depth - 1,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, state.ply + 1),
        )
    };

    if score > alpha && score < beta {
        score = -search_node(
            pos,
            ctx,
            depth - 1,
            -beta,
            -alpha,
            NodeState::new(true, state.pv_node, None, state.ply + 1),
        );
    }

    score
}
