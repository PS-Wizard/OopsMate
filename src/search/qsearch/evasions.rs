use crate::eval::EvalProvider;
use crate::movegen::Analysis;
use crate::search::context::SearchContext;
use crate::search::ordering::{MovePicker, TtMode};
use crate::search::qsearch::search::qsearch;
use crate::search::score::checkmate_score;
use crate::{Move, Position};

pub(super) fn qsearch_evasions<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    analysis: &Analysis,
    tt_move: Option<Move>,
    mut alpha: i32,
    beta: i32,
    ply: u8,
) -> i32 {
    let mut picker = MovePicker::new(analysis, tt_move, TtMode::ValidateInStage, false);
    let mut saw_legal_move = false;

    while let Some(mv) = picker.next_move(pos, analysis, None, 0) {
        if ctx.stats.should_stop() {
            break;
        }

        saw_legal_move = true;

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

    if !saw_legal_move {
        return checkmate_score(ply as usize);
    }

    alpha
}
