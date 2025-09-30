#![allow(dead_code, unused_variables)]
use std::arch::x86_64::_pext_u64;

use crate::Position;
use raw::{
    BISHOP_ATTACKS, BISHOP_MASKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS, ROOK_MASKS,
    line_between,
};
use types::others::Piece::*;

/// Returns a tuple of (pinned, checking, check_mask) pieces
pub fn get_attack_constraints(g: &Position) -> (u64, u64, u64) {
    let king_sq = g.our(King).0.trailing_zeros() as usize;
    let friendly_pieces = g.us();
    let enemy_pieces = g.them();
    let everyone = friendly_pieces | enemy_pieces;

    let mut pinned_pieces = 0u64;
    let mut checking_pieces = 0u64;

    // Get all potential pinners/checkers (use EMPTY board to see all sliders)
    let rook_mask_idx_empty = unsafe { _pext_u64(0, ROOK_MASKS[king_sq]) as usize };
    let bishop_mask_idx_empty = unsafe { _pext_u64(0, BISHOP_MASKS[king_sq]) as usize };
    let rook_rays = ROOK_ATTACKS[king_sq][rook_mask_idx_empty];
    let bishop_rays = BISHOP_ATTACKS[king_sq][bishop_mask_idx_empty];

    let enemy_rooks_queens = g.their(Rook).0 | g.their(Queen).0;
    let enemy_bishops_queens = g.their(Bishop).0 | g.their(Queen).0;

    // Find potential pinners on rook rays
    let potential_rook_pinners = rook_rays & enemy_rooks_queens;
    let potential_bishop_pinners = bishop_rays & enemy_bishops_queens;

    // Check each potential pinner
    let mut temp = potential_rook_pinners | potential_bishop_pinners;
    while temp != 0 {
        let pinner_sq = temp.trailing_zeros() as usize;
        temp &= temp - 1;

        // Get pieces between king and potential pinner
        let between = line_between(king_sq, pinner_sq);
        let pieces_between = between & everyone.0;

        if pieces_between == 0 {
            // No pieces between - it's giving check
            checking_pieces |= 1u64 << pinner_sq;
        } else if pieces_between.count_ones() == 1 {
            // Exactly one piece between - check if it's friendly (pin)
            if (pieces_between & friendly_pieces.0) != 0 {
                pinned_pieces |= pieces_between;
            }
        }
        // If more than 1 piece between, no pin/check
    }

    // Handle knight and pawn checks
    checking_pieces |= g.their(Knight).0 & KNIGHT_ATTACKS[king_sq];
    checking_pieces |= g.their(Pawn).0 & PAWN_ATTACKS[g.side_to_move as usize][king_sq];

    // Calculate check mask
    let check_mask = if checking_pieces == 0 {
        0xFFFFFFFFFFFFFFFFu64
    } else if checking_pieces.count_ones() == 1 {
        let checker_sq = checking_pieces.trailing_zeros() as usize;
        line_between(king_sq, checker_sq) | checking_pieces
    } else {
        0 // Double check
    };

    (pinned_pieces, checking_pieces, check_mask)
}

#[cfg(test)]
mod attack_constraints {
    use utilities::board::PrintAsBoard;

    use crate::{Position, legality::attack_constraints::get_attack_constraints};

    #[test]
    fn test_attack_constraints() {
        // Initial position expected no pins & no checks
        let g = Position::new();
        let (pins, checking, check_mask) = get_attack_constraints(&g);
        pins.print();
        checking.print();
        check_mask.print();
        println!("----");

        // Queen checking king, expected a vertical checkmask from e2 to e7, no pins.
        let g =
            Position::new_from_fen("rnb1kbnr/ppppq1pp/8/8/8/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1");
        let (pins, checking, check_mask) = get_attack_constraints(&g);
        assert_eq!(pins, 0);
        pins.print();
        checking.print();
        check_mask.print();

        println!("----");
        // Double Check, expected checking on h4 and b4, checkmask should be 0.
        let g = Position::new_from_fen("rnb1k1nr/pppp2pp/8/8/1b5q/8/PPP3PP/RNBQKB1R w KQkq - 0 1");
        let (pins, checking, check_mask) = get_attack_constraints(&g);
        assert_eq!(check_mask, 0);
        assert_eq!(pins, 0);
        pins.print();
        checking.print();
        check_mask.print();

        println!("----");
        let g = Position::new_from_fen("rnb1kbn1/ppppqppp/8/8/8/4B2P/PP4P1/RN2KB1r w Qq - 0 1");
        let (pins, checking, check_mask) = get_attack_constraints(&g);
        assert_eq!(checking, 0);
        pins.print();
        checking.print();
        check_mask.print();
    }
}
