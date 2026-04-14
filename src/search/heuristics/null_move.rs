use crate::eval::EvalProvider;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::node::{search_node, NodeState};
use crate::{Piece, Position};

#[inline(always)]
pub fn try_null_move_pruning<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    depth: u8,
    beta: i32,
    allow_null: bool,
    in_check: bool,
    static_eval: i32,
    ply: usize,
) -> Option<i32> {
    // null move pruning: if passing still holds beta, this node is probably already a cutoff.
    if !features::NULL_MOVE || !allow_null || in_check || depth < 3 || static_eval < beta {
        return None;
    }

    let has_pieces = (pos.our(Piece::Knight).0
        | pos.our(Piece::Bishop).0
        | pos.our(Piece::Rook).0
        | pos.our(Piece::Queen).0)
        != 0;

    if !has_pieces {
        return None;
    }

    ctx.eval.update_on_null_move(&mut ctx.eval_state, pos);
    pos.make_null_move();

    let eval_excess = (static_eval.saturating_sub(beta)).max(0);
    let reduction_bonus = (eval_excess / 200).min(4);
    let base = depth / 3 + 3;
    let reduction = base + reduction_bonus as u8;
    let null_depth = depth.saturating_sub(reduction);

    let null_score = -search_node(
        pos,
        ctx,
        null_depth,
        -beta,
        -beta + 1,
        NodeState::new(false, false, None, ply + 1),
    );

    pos.unmake_null_move();
    ctx.eval.update_on_undo_null(&mut ctx.eval_state);

    if null_score >= beta {
        Some(beta)
    } else {
        None
    }
}
