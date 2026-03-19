use crate::eval::EvalProvider;
use crate::search::alphabeta::negamax::negamax;
use crate::search::ordering::MoveHistory;
use crate::search::SearchStats;
use crate::tpt::TranspositionTable;
use crate::Position;

const PROBCUT_MARGIN: i32 = 150;
const PROBCUT_MIN_DEPTH: u8 = 5;

#[allow(clippy::too_many_arguments)]
pub fn try_probcut<E: EvalProvider>(
    pos: &mut Position,
    eval: &E,
    eval_state: &mut E::State,
    depth: u8,
    beta: i32,
    pv_node: bool,
    in_check: bool,
    allow_null: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
    thread_id: usize,
) -> Option<i32> {
    if depth < PROBCUT_MIN_DEPTH || in_check || pv_node || !allow_null {
        return None;
    }

    const MATE_BOUND: i32 = 40_000;
    if beta.abs() > MATE_BOUND {
        return None;
    }

    let probcut_beta = beta + PROBCUT_MARGIN;
    let probcut_depth = depth - 5;

    let mut collector = crate::MoveCollector::new();
    pos.generate_captures(&mut collector);
    let moves = collector.as_slice();

    for &mv in moves {
        let delta = eval.update_on_move(eval_state, pos, mv);
        pos.make_move(mv);

        let score = -negamax(
            pos,
            eval,
            eval_state,
            probcut_depth,
            -probcut_beta,
            -probcut_beta + 1,
            tt,
            history,
            stats,
            true,
            false,
            true,
            None,
            ply + 1,
            thread_id,
        );

        pos.unmake_move(mv);
        eval.update_on_undo(eval_state, delta);

        if score >= probcut_beta {
            return Some(beta);
        }
    }

    None
}
