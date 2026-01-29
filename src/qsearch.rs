use crate::evaluate::evaluate;
use crate::move_ordering::{pick_next_move, score_move};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

/// Quiescence search; searches captures until position is "quiet"
/// This prevents the horizon effect where we stop searching mid-capture sequence
pub fn qsearch(pos: &Position, mut alpha: i32, beta: i32, stats: &mut SearchStats) -> i32 {
    stats.nodes += 1;

    // Stand pat ; the score if we make no capture
    // This represents doing nothing and just evaluating the position
    let stand_pat = evaluate(pos);

    // Beta cutoff ; this position is already too good for the opponent
    // They won't let us reach this position
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha ; we can always choose to not capture
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning ; if we're so far behind that even capturing the best piece
    // won't bring us back, don't bother searching
    const QUEEN_VALUE: i32 = 900;
    if stand_pat + QUEEN_VALUE + 200 < alpha {
        return alpha;
    }

    // Generate all moves and filter to captures/promotions
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // Only look at tactical moves (captures and promotions)
    let captures: Vec<Move> = moves
        .iter()
        .filter(|m| m.is_capture() || m.is_promotion())
        .copied()
        .collect();

    // If no captures available, return the stand pat score
    if captures.is_empty() {
        return stand_pat;
    }

    // Score captures with MVV-LVA (no TT move in qsearch)
    let mut capture_list = captures;
    let mut scores: Vec<i32> = capture_list
        .iter()
        .map(|m| score_move(*m, pos, None))
        .collect();

    // Search captures in MVV-LVA order
    for i in 0..capture_list.len() {
        pick_next_move(&mut capture_list, &mut scores, i);
        let mv = capture_list[i];

        let new_pos = pos.make_move(&mv);
        let score = -qsearch(&new_pos, -beta, -alpha, stats);

        if score >= beta {
            return beta; // Beta cutoff
        }

        if score > alpha {
            alpha = score; // Raise alpha
        }
    }

    alpha
}
