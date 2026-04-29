use crate::eval::EvalProvider;
use crate::movegen::analyze;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::ordering::{MovePicker, TtMode};
use crate::search::qsearch::evasions::qsearch_evasions;
use crate::tpt::{Bound, NO_STATIC_EVAL};
use crate::{Move, Position};

pub(crate) fn qsearch<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mut alpha: i32,
    beta: i32,
    ply: u8,
) -> i32 {
    ctx.stats.nodes += 1;

    if ctx.stats.should_stop() {
        return alpha;
    }

    if pos.is_fifty_move_draw() || pos.is_repetition() {
        return 0;
    }

    if ply >= 64 {
        return ctx.eval.eval(pos, &mut ctx.eval_state);
    }

    let hash = pos.hash();
    let tt_entry = if features::TT_CUTOFFS {
        ctx.tt.probe(hash, ply)
    } else {
        None
    };
    let tt_move = tt_entry.map(|entry| entry.best_move).filter(|mv| mv.0 != 0);

    if let Some(entry) = tt_entry {
        match entry.bound {
            Bound::Exact => return entry.score,
            Bound::Lower if entry.score >= beta => return entry.score,
            Bound::Upper if entry.score <= alpha => return entry.score,
            _ => {}
        }
    }

    let analysis = analyze(pos);
    if analysis.in_check() {
        return qsearch_evasions(pos, ctx, &analysis, tt_move, alpha, beta, ply);
    }

    let raw_static_eval = if let Some(entry) = tt_entry {
        if entry.static_eval != NO_STATIC_EVAL {
            i32::from(entry.static_eval)
        } else {
            ctx.eval.eval(pos, &mut ctx.eval_state)
        }
    } else {
        ctx.eval.eval(pos, &mut ctx.eval_state)
    };

    if raw_static_eval >= beta {
        if features::TT_CUTOFFS {
            ctx.tt.store(
                hash,
                ply,
                Move(0),
                raw_static_eval,
                static_eval_to_tt(raw_static_eval),
                0,
                Bound::Lower,
            );
        }
        return beta;
    }

    let original_alpha = alpha;
    if raw_static_eval > alpha {
        alpha = raw_static_eval;
    }

    const QUEEN_VALUE: i32 = 900;
    if raw_static_eval + QUEEN_VALUE + 300 < original_alpha {
        return original_alpha;
    }

    let mut picker = MovePicker::new(&analysis, tt_move, TtMode::ValidateInStage, false);
    let mut best_score = raw_static_eval;
    let mut best_move = Move(0);
    let mut saw_tactical = false;

    while let Some(mv) = picker.next_move(pos, &analysis, None, 0) {
        if ctx.stats.should_stop() {
            break;
        }

        if mv.is_capture() && features::SEE && pos.see(&mv) < 0 {
            continue;
        }

        saw_tactical = true;

        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);
        let score = -qsearch(pos, ctx, -beta, -alpha, ply + 1);
        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if score > best_score {
            best_score = score;
            best_move = mv;
        }

        if score >= beta {
            if features::TT_CUTOFFS {
                ctx.tt.store(
                    hash,
                    ply,
                    mv,
                    score,
                    static_eval_to_tt(raw_static_eval),
                    0,
                    Bound::Lower,
                );
            }
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    if !saw_tactical {
        if features::TT_CUTOFFS {
            ctx.tt.store(
                hash,
                ply,
                Move(0),
                raw_static_eval,
                static_eval_to_tt(raw_static_eval),
                0,
                if raw_static_eval <= original_alpha {
                    Bound::Upper
                } else {
                    Bound::Exact
                },
            );
        }
        return raw_static_eval;
    }

    if features::TT_CUTOFFS {
        ctx.tt.store(
            hash,
            ply,
            best_move,
            best_score,
            static_eval_to_tt(raw_static_eval),
            0,
            if best_score <= original_alpha {
                Bound::Upper
            } else {
                Bound::Exact
            },
        );
    }

    alpha
}

#[inline(always)]
fn static_eval_to_tt(score: i32) -> i16 {
    debug_assert!(score >= i16::MIN as i32 && score <= i16::MAX as i32);
    score as i16
}
