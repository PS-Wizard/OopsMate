use crate::eval::EvalProvider;
use crate::movegen::generate_evasions_with_analysis;
use crate::search::context::SearchContext;
use crate::search::ordering::{pick_next_move, score_move};
use crate::search::qsearch::search::qsearch;
use crate::search::score::checkmate_score;
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

pub(super) fn qsearch_evasions<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    analysis: crate::movegen::Analysis,
    mut alpha: i32,
    beta: i32,
    ply: i32,
) -> i32 {
    let mut collector = MoveCollector::new();
    generate_evasions_with_analysis(pos, &analysis, &mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return checkmate_score(ply as usize);
    }

    let move_count = moves.len();
    let mut move_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];
    for i in 0..move_count {
        move_list[i] = moves[i];
        scores[i] = score_move(moves[i], pos, None, None, 0);
    }

    for i in 0..move_count {
        if ctx.stats.should_stop() {
            break;
        }

        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);
        let score = -qsearch(pos, ctx, -beta, -alpha, ply + 1);
        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
