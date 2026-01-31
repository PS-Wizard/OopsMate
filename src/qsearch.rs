use crate::evaluate::evaluate;
use crate::move_history::KillerTable;
use crate::move_ordering::{pick_next_move, score_move, PIECE_VALUES};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

/// Quiescence search with make/unmake
pub fn qsearch(
    pos: &mut Position,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    ply: i32,
) -> i32 {
    stats.nodes += 1;

    const MAX_QSEARCH_PLY: i32 = 64;
    if ply >= MAX_QSEARCH_PLY {
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

    // Delta pruning
    const QUEEN_VALUE: i32 = 900;
    if stand_pat + QUEEN_VALUE + 200 < original_alpha {
        return alpha;
    }

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    let mut capture_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];
    let mut capture_count = 0;

    let killers = KillerTable::new();

    for &m in moves {
        if m.is_capture() || m.is_promotion() {
            if m.is_capture() {
                let victim_value = pos
                    .piece_at(m.to())
                    .map(|(p, _)| PIECE_VALUES[p as usize])
                    .unwrap_or(0);
                let attacker_value = pos
                    .piece_at(m.from())
                    .map(|(p, _)| PIECE_VALUES[p as usize])
                    .unwrap_or(0);

                if victim_value < attacker_value / 2 && ply > 0 {
                    continue;
                }
            }

            capture_list[capture_count] = m;
            scores[capture_count] = score_move(m, pos, None, &killers, ply as usize);
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

        let undo = pos.make_move(&mv);
        let score = -qsearch(pos, -beta, -alpha, stats, ply + 1);
        pos.unmake_move(&mv, &undo);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
