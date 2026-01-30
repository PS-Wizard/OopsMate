use crate::evaluate::evaluate;
use crate::move_ordering::{pick_next_move, score_move};
use crate::search::SearchStats;
use crate::{Move, MoveCollector, Position};

const MAX_MOVES: usize = 256;

/// Quiescence search; searches captures until position is "quiet"
pub fn qsearch(pos: &Position, mut alpha: i32, beta: i32, stats: &mut SearchStats) -> i32 {
    stats.nodes += 1;

    // Stand pat; the score if we make no capture
    let stand_pat = evaluate(pos);

    // Beta cutoff;  position is already too good for opponent
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha;  we can always choose to not capture
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning; if we're so far behind that even capturing queen won't help
    const QUEEN_VALUE: i32 = 900;
    if stand_pat + QUEEN_VALUE + 200 < alpha {
        return alpha;
    }

    // Generate all moves
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // Filter to tactical moves and score them on stack
    let mut capture_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];
    let mut capture_count = 0;

    for &m in moves {
        if m.is_capture() || m.is_promotion() {
            capture_list[capture_count] = m;
            scores[capture_count] = score_move(m, pos, None);
            capture_count += 1;
        }
    }

    // If no captures available, return stand pat
    if capture_count == 0 {
        return stand_pat;
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
