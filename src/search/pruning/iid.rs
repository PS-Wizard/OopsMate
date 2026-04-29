use crate::eval::EvalProvider;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::node::{search_node, NodeState};
use crate::search::params::IID_MIN_DEPTH;
use crate::{Move, Position};

#[inline(always)]
fn iid_reduction(depth: u8, pv_node: bool) -> u8 {
    if pv_node {
        (depth / 4).max(2)
    } else {
        (depth / 3).max(1)
    }
}

#[inline(always)]
pub fn try_iid<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    depth: u8,
    alpha: i32,
    beta: i32,
    pv_node: bool,
    has_tt_move: bool,
    in_check: bool,
    ply: usize,
) -> Option<Move> {
    // iid: run a cheaper preliminary search to manufacture a good tt move for ordering.
    if !features::IID || has_tt_move || depth < IID_MIN_DEPTH || in_check {
        return None;
    }

    if !pv_node && depth < IID_MIN_DEPTH + 2 {
        return None;
    }

    let reduction = iid_reduction(depth, pv_node);
    let iid_depth = depth.saturating_sub(reduction);

    search_node(
        pos,
        ctx,
        iid_depth,
        alpha,
        beta,
        NodeState::new(true, pv_node, None, ply),
    );

    ctx.tt.probe(pos.hash()).map(|entry| entry.best_move)
}
