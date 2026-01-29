use crate::{Move, Position};

// Piece values for MVV-LVA
pub const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    320,   // Knight
    330,   // Bishop
    500,   // Rook
    900,   // Queen
    20000, // King (should never be captured, but just in case)
];

/// Score a move for ordering purposes
/// Higher scores = search first
#[inline(always)]
pub fn score_move(m: Move, pos: &Position, tt_move: Option<Move>) -> i32 {
    // TT move gets highest priority
    if let Some(tt_mv) = tt_move {
        if m.0 == tt_mv.0 {
            return 1_000_000;
        }
    }

    // MVV-LVA for captures
    if m.is_capture() {
        let victim = pos.piece_at(m.to()).map(|(p, _)| p);
        let attacker = pos.piece_at(m.from()).map(|(p, _)| p);

        if let (Some(victim_piece), Some(attacker_piece)) = (victim, attacker) {
            // MVV-LVA: (victim_value * 10) - attacker_value
            // This ensures we try high-value captures with low-value attackers first
            return PIECE_VALUES[victim_piece as usize] * 10
                - PIECE_VALUES[attacker_piece as usize];
        }
    }

    // Promotions (after captures but before quiet moves)
    if m.is_promotion() {
        return 5000;
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
