use crate::{move_history::KillerTable, Move, Position};

// Piece values for MVV-LVA
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
const SCORE_WINNING_CAPTURE: i32 = 100_000;
const SCORE_KILLER_PRIMARY: i32 = 9_000;
const SCORE_KILLER_SECONDARY: i32 = 8_000;
const SCORE_PROMOTION: i32 = 5_000;

/// Score a move for ordering in main search
#[inline(always)]
pub fn score_move(
    m: Move,
    pos: &Position,
    tt_move: Option<Move>,
    killers: &KillerTable,
    ply: usize,
) -> i32 {
    // TT move gets highest priority
    if let Some(tt_mv) = tt_move {
        if m.0 == tt_mv.0 {
            return SCORE_TT_MOVE;
        }
    }

    // Promotions
    if m.is_promotion() {
        if m.is_capture() {
            // Capture promotion
            if let (Some((victim_piece, _)), Some(_)) =
                (pos.piece_at(m.to()), pos.piece_at(m.from()))
            {
                return SCORE_WINNING_CAPTURE + 900 + PIECE_VALUES[victim_piece as usize] * 10;
            }
        } else {
            return SCORE_PROMOTION; // Quiet promotion
        }
    }

    // Then captures...
    // MVV-LVA for captures
    if m.is_capture() {
        if let (Some((victim_piece, _)), Some((attacker_piece, _))) =
            (pos.piece_at(m.to()), pos.piece_at(m.from()))
        {
            return SCORE_WINNING_CAPTURE + PIECE_VALUES[victim_piece as usize] * 10
                - PIECE_VALUES[attacker_piece as usize];
        }
    }

    // Killer moves
    if killers.is_killer(ply, m) {
        return if Some(m) == killers.get_primary(ply) {
            SCORE_KILLER_PRIMARY
        } else {
            SCORE_KILLER_SECONDARY
        };
    }

    0
}

/// Order moves in-place using partial selection sort
/// Only sorts the next move to search, making it O(n) per move
#[inline(always)]
pub fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
    if index >= moves.len() {
        return;
    }

    let mut best_idx = index;
    let mut best_score = scores[index];

    for i in (index + 1)..moves.len() {
        if scores[i] > best_score {
            best_score = scores[i];
            best_idx = i;
        }
    }

    if best_idx != index {
        moves.swap(index, best_idx);
        scores.swap(index, best_idx);
    }
}
