use super::negamax::negamax;
use crate::eval::EvalProvider;
use crate::search::features;
use crate::search::ordering::MoveHistory;
use crate::search::params::IID_MIN_DEPTH;
use crate::search::SearchStats;
use crate::tpt::TranspositionTable;
use crate::{Move, Position};

#[inline(always)]
fn iid_reduction(depth: u8, pv_node: bool) -> u8 {
    if pv_node {
        (depth / 4).max(2)
    } else {
        (depth / 3).max(1)
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub fn try_iid<E: EvalProvider>(
    pos: &mut Position,
    eval: &E,
    eval_state: &mut E::State,
    depth: u8,
    alpha: i32,
    beta: i32,
    pv_node: bool,
    has_tt_move: bool,
    in_check: bool,
    tt: &mut TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
) -> Option<Move> {
    if !features::IID {
        return None;
    }

    if has_tt_move || depth < IID_MIN_DEPTH || in_check {
        return None;
    }

    if !pv_node && depth < IID_MIN_DEPTH + 2 {
        return None;
    }

    let reduction = iid_reduction(depth, pv_node);
    let iid_depth = depth.saturating_sub(reduction);

    negamax(
        pos, eval, eval_state, iid_depth, alpha, beta, tt, history, stats, true, pv_node, false,
        None, ply,
    );

    tt.probe(pos.hash()).map(|entry| entry.best_move)
}
