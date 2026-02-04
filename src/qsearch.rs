use crate::evaluate::evaluate;
use crate::move_ordering::{pick_next_move, score_move};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

/// Quiescence search; searches captures until position is "quiet"
pub fn qsearch(
    pos: &Position,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    ply: i32,
) -> i32 {
    stats.nodes += 1;

    // MAX QSEARCH DEPTH
    if ply >= 64 {
        return evaluate(pos);
    }

    let stand_pat = evaluate(pos);

    if stand_pat >= beta {
        return beta;
    }

    let original_alpha = alpha;
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning: if stand_pat + queen value can't raise alpha, prune
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
        // Only Captures and Promotions
        if m.is_capture() || m.is_promotion() {
            // SEE PRUNING
            if m.is_capture() {
                let see_score = pos.see(&m);

                // If the exchange loses material (SEE < 0), prune it.
                if see_score < 0 {
                    continue;
                }
            }

            capture_list[capture_count] = m;

            scores[capture_count] = score_move(m, pos, None, None, 0);
            capture_count += 1;
        }
    }

    // If no moves passed SEE or generation, return the static eval
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

        let new_pos = pos.make_move(&mv);
        let score = -qsearch(&new_pos, -beta, -alpha, stats, ply + 1);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
