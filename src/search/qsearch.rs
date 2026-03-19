use crate::eval::EvalProvider;
use crate::search::features;
use crate::search::ordering::{pick_next_move, score_capture, SCORE_PROMOTION};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

pub(crate) fn qsearch<E: EvalProvider>(
    pos: &mut Position,
    eval: &E,
    eval_state: &mut E::State,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    ply: i32,
) -> i32 {
    stats.nodes += 1;

    if stats.should_stop() {
        return alpha;
    }

    if pos.is_fifty_move_draw() || pos.is_repetition() {
        return 0;
    }

    if ply >= 64 {
        return eval.eval(pos, eval_state);
    }

    let stand_pat = eval.eval(pos, eval_state);

    if stand_pat >= beta {
        return beta;
    }

    let original_alpha = alpha;
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    if !pos.is_in_check() {
        const QUEEN_VALUE: i32 = 900;
        if stand_pat + QUEEN_VALUE + 300 < original_alpha {
            return original_alpha;
        }
    }

    let mut collector = MoveCollector::new();
    pos.generate_captures(&mut collector);
    let moves = collector.as_slice();

    let mut capture_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];
    let mut capture_count = 0;

    for &m in moves {
        if m.is_capture() || m.is_promotion() {
            let score = if m.is_capture() {
                if features::SEE {
                    let see_score = pos.see(&m);
                    if see_score < 0 {
                        continue;
                    }
                }

                score_capture(m, pos)
            } else {
                SCORE_PROMOTION
            };

            if m.is_capture() {
                debug_assert!(score >= 0);
            }

            capture_list[capture_count] = m;
            scores[capture_count] = score;
            capture_count += 1;
        }
    }

    if capture_count == 0 {
        return stand_pat;
    }

    for i in 0..capture_count {
        if stats.should_stop() {
            break;
        }

        pick_next_move(
            &mut capture_list[..capture_count],
            &mut scores[..capture_count],
            i,
        );
        let mv = capture_list[i];

        let delta = eval.update_on_move(eval_state, pos, mv);
        pos.make_move(mv);
        let score = -qsearch(pos, eval, eval_state, -beta, -alpha, stats, ply + 1);
        pos.unmake_move(mv);
        eval.update_on_undo(eval_state, delta);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
