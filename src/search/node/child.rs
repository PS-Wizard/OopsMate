use crate::eval::EvalProvider;
use crate::search::context::SearchContext;
use crate::search::node::{search_node, NodeState};
use crate::search::pruning::{calculate_lmr_reduction, should_reduce_lmr};
use crate::{Move, Position};

#[expect(
    clippy::too_many_arguments,
    reason = "move search keeps the hot-path branching explicit"
)]
#[inline(always)]
pub(super) fn search_with_lmr<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_index: usize,
    in_check: bool,
    gives_check: bool,
    check_extension: u8,
    node: NodeState,
) -> i32 {
    let do_lmr = should_reduce_lmr(depth, move_index, in_check, gives_check, mv);

    if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_index, node.pv_node, mv);
        let reduced_depth = depth
            .saturating_sub(1 + reduction)
            .saturating_add(check_extension);

        let reduced_score = -search_node(
            pos,
            ctx,
            reduced_depth,
            -beta,
            -alpha,
            NodeState::new(true, node.pv_node, None, node.ply + 1),
        );

        if reduced_score > alpha {
            return -search_node(
                pos,
                ctx,
                depth - 1 + check_extension,
                -beta,
                -alpha,
                NodeState::new(true, node.pv_node, None, node.ply + 1),
            );
        }

        return reduced_score;
    }

    -search_node(
        pos,
        ctx,
        depth - 1 + check_extension,
        -beta,
        -alpha,
        NodeState::new(true, node.pv_node, None, node.ply + 1),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "move search keeps the hot-path branching explicit"
)]
#[inline(always)]
pub(super) fn search_with_pvs<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_index: usize,
    in_check: bool,
    gives_check: bool,
    check_extension: u8,
    node: NodeState,
    is_hash_move: bool,
) -> i32 {
    if move_index == 0 {
        return -search_node(
            pos,
            ctx,
            depth - 1 + check_extension,
            -beta,
            -alpha,
            NodeState::new(true, node.pv_node, None, node.ply + 1),
        );
    }

    let do_lmr = should_reduce_lmr(depth, move_index, in_check, gives_check, mv) && !is_hash_move;

    let mut score = if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_index, node.pv_node, mv);
        let reduced_depth = depth
            .saturating_sub(1 + reduction)
            .saturating_add(check_extension);

        -search_node(
            pos,
            ctx,
            reduced_depth,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, node.ply + 1),
        )
    } else {
        -search_node(
            pos,
            ctx,
            depth - 1 + check_extension,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, node.ply + 1),
        )
    };

    if score > alpha && score < beta {
        score = -search_node(
            pos,
            ctx,
            depth - 1 + check_extension,
            -beta,
            -alpha,
            NodeState::new(true, node.pv_node, None, node.ply + 1),
        );
    }

    score
}
