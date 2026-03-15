use crate::evaluate::{apply_move, evaluate_with_probe, undo_move, EvalProbe};
use crate::search::ordering::{pick_next_move, score_capture_from_see, SCORE_PROMOTION};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

pub(crate) fn qsearch(
    pos: &mut Position,
    probe: &mut EvalProbe,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    ply: i32,
) -> i32 {
    stats.nodes += 1;

    if pos.is_fifty_move_draw() || pos.is_repetition() {
        return 0;
    }

    if ply >= 64 {
        return evaluate_with_probe(pos, probe);
    }

    let stand_pat = evaluate_with_probe(pos, probe);

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
                let see_score = pos.see(&m);
                if see_score < 0 {
                    continue;
                }
                score_capture_from_see(see_score)
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
        pick_next_move(
            &mut capture_list[..capture_count],
            &mut scores[..capture_count],
            i,
        );
        let mv = capture_list[i];

        let delta = apply_move(probe, pos, mv);
        pos.make_move(mv);
        let score = -qsearch(pos, probe, -beta, -alpha, stats, ply + 1);
        pos.unmake_move(mv);
        undo_move(probe, delta);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
