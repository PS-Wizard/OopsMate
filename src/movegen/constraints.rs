use crate::{position::Position, types::Piece};
use strikes::{line_between, BISHOP_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS};

pub(super) fn get_constraints(pos: &Position) -> (u64, u64) {
    let king_sq = pos.our(Piece::King).0.trailing_zeros() as usize;
    let us = pos.us().0;
    let them = pos.them().0;
    let occupied = us | them;

    let mut pinned = 0u64;
    let mut checkers = 0u64;

    let enemy_bishops_queens = pos.their(Piece::Bishop).0 | pos.their(Piece::Queen).0;
    let enemy_rooks_queens = pos.their(Piece::Rook).0 | pos.their(Piece::Queen).0;

    let bishop_rays = BISHOP_ATTACKS[king_sq][0];
    let rook_rays = ROOK_ATTACKS[king_sq][0];

    let mut potential = (bishop_rays & enemy_bishops_queens) | (rook_rays & enemy_rooks_queens);
    while potential != 0 {
        let sq = potential.trailing_zeros() as usize;
        potential &= potential - 1;

        let between = line_between(king_sq, sq);
        let pieces_between = between & occupied;

        if pieces_between == 0 {
            checkers |= 1u64 << sq;
        } else if pieces_between.count_ones() == 1 && (pieces_between & us) != 0 {
            pinned |= pieces_between;
        }
    }

    checkers |= pos.their(Piece::Knight).0 & KNIGHT_ATTACKS[king_sq];
    checkers |= pos.their(Piece::Pawn).0 & PAWN_ATTACKS[pos.side_to_move as usize][king_sq];

    let check_mask = if checkers == 0 {
        !0u64
    } else if checkers.count_ones() == 1 {
        let checker_sq = checkers.trailing_zeros() as usize;
        line_between(king_sq, checker_sq) | checkers
    } else {
        0
    };

    (pinned, check_mask)
}
