use super::move_search::search_move;
use crate::eval::EvalProvider;
use crate::search::ordering::{pick_next_move, score_move, MoveHistory, SCORE_TT_MOVE};
use crate::search::params::{INFINITY, MAX_MOVES};
use crate::search::score::score_to_tt;
use crate::search::SearchStats;
use crate::tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND};
use crate::{Move, Position};

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub fn search_root<E: EvalProvider>(
    pos: &mut Position,
    eval: &E,
    eval_state: &mut E::State,
    moves: &mut [Move],
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    thread_id: usize,
) -> (i32, Move) {
    let in_check = pos.is_in_check();
    let alpha_start = alpha;
    let tt_move = tt.probe(pos.hash()).map(|e| e.best_move);
    let move_count = moves.len();
    let mut scores = [0i32; MAX_MOVES];

    for i in 0..move_count {
        scores[i] = score_move(moves[i], pos, tt_move, Some(history), 0);
    }

    if thread_id > 0 && move_count > 1 {
        for (i, score) in scores.iter_mut().enumerate().take(move_count) {
            if *score != SCORE_TT_MOVE {
                let noise = ((i + thread_id) * 987654321) % 4000;
                *score = (*score).saturating_add(noise as i32);
            }
        }
    }

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    for i in 0..move_count {
        if stats.should_stop() {
            break;
        }

        pick_next_move(moves, &mut scores, i);
        let mv = moves[i];

        let delta = eval.update_on_move(eval_state, pos, mv);
        pos.make_move(mv);
        let gives_check = pos.is_in_check();

        let score = if i == 0 {
            search_move(
                pos,
                eval,
                eval_state,
                mv,
                depth,
                alpha,
                beta,
                i,
                in_check,
                gives_check,
                true,
                tt,
                history,
                stats,
                0,
                thread_id,
            )
        } else {
            let s = search_move(
                pos,
                eval,
                eval_state,
                mv,
                depth,
                alpha,
                alpha + 1,
                i,
                in_check,
                gives_check,
                true,
                tt,
                history,
                stats,
                0,
                thread_id,
            );
            if s > alpha && s < beta {
                search_move(
                    pos,
                    eval,
                    eval_state,
                    mv,
                    depth,
                    alpha,
                    beta,
                    i,
                    in_check,
                    gives_check,
                    true,
                    tt,
                    history,
                    stats,
                    0,
                    thread_id,
                )
            } else {
                s
            }
        };
        pos.unmake_move(mv);
        eval.update_on_undo(eval_state, delta);

        if stats.should_stop() {
            return (best_score, best_move);
        }

        if i == 0 && score <= alpha {
            return (score, mv);
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    break;
                }
            }
        }
    }

    let flag = if best_score >= beta {
        LOWER_BOUND
    } else if best_score <= alpha_start {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(
        pos.hash(),
        best_move,
        score_to_tt(best_score, 0),
        depth,
        flag,
    );

    (best_score, best_move)
}
