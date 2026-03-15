use crate::evaluate::{apply_null_move, undo_null_move, EvalProbe};
use crate::search::alphabeta::negamax::negamax;
use crate::search::ordering::MoveHistory;
use crate::search::SearchStats;
use crate::tpt::TranspositionTable;
use crate::{Piece, Position};

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub fn try_null_move_pruning(
    pos: &mut Position,
    probe: &mut EvalProbe,
    depth: u8,
    beta: i32,
    allow_null: bool,
    in_check: bool,
    static_eval: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
    thread_id: usize,
) -> Option<i32> {
    if !allow_null || in_check || depth < 3 {
        return None;
    }

    if static_eval < beta {
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

    apply_null_move(probe, pos);
    pos.make_null_move();

    let eval_excess = (static_eval.saturating_sub(beta)).max(0);
    let reduction_bonus = (eval_excess / 200).min(4);
    let base = depth / 3 + 3;
    let reduction = base + reduction_bonus as u8;
    let null_depth = depth.saturating_sub(reduction);

    let null_score = -negamax(
        pos,
        probe,
        null_depth,
        -beta,
        -beta + 1,
        tt,
        history,
        stats,
        false,
        false,
        true,
        None,
        ply + 1,
        thread_id,
    );

    pos.unmake_null_move();
    undo_null_move(probe);

    if null_score >= beta {
        Some(beta)
    } else {
        None
    }
}
