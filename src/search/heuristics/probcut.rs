use crate::eval::EvalProvider;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::node::{search_node, NodeState};
use crate::Position;

const PROBCUT_MARGIN: i32 = 150;
const PROBCUT_MIN_DEPTH: u8 = 5;

pub fn try_probcut<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    depth: u8,
    beta: i32,
    pv_node: bool,
    in_check: bool,
    ply: usize,
) -> Option<i32> {
    // probcut: use a reduced tactical probe to prove this node will likely fail high.
    if !features::PROBCUT || depth < PROBCUT_MIN_DEPTH || in_check || pv_node {
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
        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);

        let score = -search_node(
            pos,
            ctx,
            probcut_depth,
            -probcut_beta,
            -probcut_beta + 1,
            NodeState::new(true, false, None, ply + 1),
        );

        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if score >= probcut_beta {
            return Some(beta);
        }
    }

    None
}
