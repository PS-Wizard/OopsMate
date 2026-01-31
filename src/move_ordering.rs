use crate::{move_history::KillerTable, Move, Position};

// Piece values for MVV-LVA
pub const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    320,   // Knight
    330,   // Bishop
    500,   // Rook
    900,   // Queen
    20000, // King (should never be captured, but just in case)
];

// Move ordering priority scores
const SCORE_TT_MOVE: i32 = 1_000_000;
const SCORE_WINNING_CAPTURE: i32 = 100_000; // MVV-LVA base
const SCORE_KILLER_PRIMARY: i32 = 9_000;
const SCORE_KILLER_SECONDARY: i32 = 8_000;
const SCORE_PROMOTION: i32 = 5_000;
// Quiet moves and losing captures: 0

/// Score a move for ordering purposes
/// Higher scores = search first
///
/// Priority order:
/// - TT move 
/// - Winning captures via MVV-LVA 
/// - Killer moves 
/// - Promotions 
/// - Quiet moves 
///
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

    // MVV-LVA for captures
    if m.is_capture() {
        let victim = pos.piece_at(m.to()).map(|(p, _)| p);
        let attacker = pos.piece_at(m.from()).map(|(p, _)| p);

        if let (Some(victim_piece), Some(attacker_piece)) = (victim, attacker) {
            // MVV-LVA: (victim_value * 10) - attacker_value
            // This ensures we try high-value captures with low-value attackers first
            return SCORE_WINNING_CAPTURE + PIECE_VALUES[victim_piece as usize] * 10
                - PIECE_VALUES[attacker_piece as usize];
        }
    }

    // Promotions (after captures but before killers)
    if m.is_promotion() {
        return SCORE_PROMOTION;
    }

    // Killer moves (quiet moves that caused beta cutoffs at this ply)
    if killers.is_killer(ply, m) {
        if Some(m) == killers.get_primary(ply) {
            return SCORE_KILLER_PRIMARY;
        } else {
            return SCORE_KILLER_SECONDARY;
        }
    }

    // Quiet moves get lowest priority
    0
}

/// Order moves in-place using partial selection sort
/// Only sorts the next move to search, making it O(n) per move instead of O(n log n) total
#[inline(always)]
pub fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
    if index >= moves.len() {
        return;
    }

    // Find the best move from index..end
    let mut best_idx = index;
    let mut best_score = scores[index];

    for i in (index + 1)..moves.len() {
        if scores[i] > best_score {
            best_score = scores[i];
            best_idx = i;
        }
    }

    // Swap it to the current position
    if best_idx != index {
        moves.swap(index, best_idx);
        scores.swap(index, best_idx);
    }
}
