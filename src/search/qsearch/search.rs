use crate::eval::EvalProvider;
use crate::movegen::{analyze, generate_captures_with_analysis};
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::ordering::{pick_next_move, score_move};
use crate::search::qsearch::evasions::qsearch_evasions;
use crate::tpt::{Bound, NO_STATIC_EVAL};
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

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

    let mut collector = MoveCollector::new();
    generate_captures_with_analysis(pos, &analysis, &mut collector);
    let moves = collector.as_slice();

    let mut capture_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];
    let mut capture_count = 0;

    for &m in moves {
        if m.is_capture() || m.is_promotion() {
            if m.is_capture() && features::SEE {
                let see_score = pos.see(&m);
                if see_score < 0 {
                    continue;
                }
            }

            capture_list[capture_count] = m;
            scores[capture_count] = score_move(m, pos, tt_move, None, 0);
            capture_count += 1;
        }
    }

    if capture_count == 0 {
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

    let mut best_score = raw_static_eval;
    let mut best_move = Move(0);

    for i in 0..capture_count {
        if ctx.stats.should_stop() {
            break;
        }

        pick_next_move(
            &mut capture_list[..capture_count],
            &mut scores[..capture_count],
            i,
        );
        let mv = capture_list[i];

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
