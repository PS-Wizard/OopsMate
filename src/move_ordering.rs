use crate::{move_history::MoveHistory, Move, Position};

pub const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    320,   // Knight
    330,   // Bishop
    500,   // Rook
    900,   // Queen
    20000, // King
];

// Move ordering priority scores
const SCORE_TT_MOVE: i32 = 1_000_000;
const SCORE_GOOD_CAPTURE: i32 = 100_000; // SEE >= 0
const SCORE_PROMOTION: i32 = 90_000; // Quiet promotion
const SCORE_KILLER_PRIMARY: i32 = 20_000;
const SCORE_KILLER_SECONDARY: i32 = 15_000;
const SCORE_BAD_CAPTURE: i32 = 5_000; // SEE < 0 (Still better than random quiet moves?)

/// Score a move for ordering in main search
#[inline(always)]
pub fn score_move(
    m: Move,
    pos: &Position,
    tt_move: Option<Move>,
    history: Option<&MoveHistory>,
    ply: usize,
) -> i32 {
    // TT move gets highest priority
    if let Some(tt_mv) = tt_move {
        if m.0 == tt_mv.0 {
            return SCORE_TT_MOVE;
        }
    }

    // Captures & Promotions (Resolved via SEE)
    if m.is_capture() {
        let see_score = pos.see(&m);

        if see_score >= 0 {
            // Good capture: Prioritize by SEE score (Winning Queen > Winning Pawn)
            return SCORE_GOOD_CAPTURE + see_score;
        } else {
            // Bad capture: Lose material.
            return SCORE_BAD_CAPTURE + see_score;
        }
    }

    // 3. Quiet Promotions
    if m.is_promotion() {
        return SCORE_PROMOTION;
    }

    if let Some(h) = history {
        // 4. Killer moves
        if h.killers.is_killer(ply, m) {
            return if Some(m) == h.killers.get_primary(ply) {
                SCORE_KILLER_PRIMARY
            } else {
                SCORE_KILLER_SECONDARY
            };
        }
        
        // 5. History Heuristic
        return h.history.get(pos.side_to_move, m.from(), m.to());
    }

    0
}

#[inline(always)]
pub fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
    if index >= moves.len() {
        return;
    }

    let mut best_idx = index;
    let mut best_score = unsafe { *scores.get_unchecked(index) };

    for i in (index + 1)..moves.len() {
        let score = unsafe { *scores.get_unchecked(i) };
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    if best_idx != index {
        moves.swap(index, best_idx);
        scores.swap(index, best_idx);
    }
}
