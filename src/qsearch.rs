use crate::evaluate::evaluate;
use crate::move_ordering::{pick_next_move, score_move_qsearch};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

const MAX_CAPTURES: usize = 64; // Reduced from 256 since we only generate captures

/// Quiescence search;  searches captures until position is "quiet"
pub fn qsearch(
    pos: &Position,
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

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning
    const QUEEN_VALUE: i32 = 900;
    if stand_pat + QUEEN_VALUE + 200 < alpha {
        return alpha;
    }

    // Generate ONLY captures
    let mut collector = MoveCollector::new();
    pos.generate_captures(&mut collector);

    let captures = collector.as_slice();
    if captures.is_empty() {
        return stand_pat;
    }

    let capture_count = captures.len().min(MAX_CAPTURES);

    // Stack arrays for move ordering
    let mut capture_list = [Move(0); MAX_CAPTURES];
    let mut scores = [0i32; MAX_CAPTURES];

    for i in 0..capture_count {
        capture_list[i] = captures[i];
        scores[i] = score_move_qsearch(captures[i], pos);
    }

    // Search captures in MVV-LVA order
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
