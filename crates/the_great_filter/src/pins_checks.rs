#![allow(dead_code)]
use game::{
    game::Game,
    piece::{Piece, PieceKind},
};
use pext::{BISHOP_ATTACKS, BISHOP_MASKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS, ROOK_MASKS};
use std::arch::x86_64::_pext_u64;

use crate::{
    BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, LEFT, RIGHT, TOP, TOP_LEFT, TOP_RIGHT, between::BETWEEN,
    ray_attacks::RAY_ATTACKS,
};

pub fn find_pins_and_checks(game: &Game) -> (u64, u64, u64) {
    let king_sq = find_king_square(game);
    let our_pieces = game.get_all_friendlies();
    let enemy_pieces = game.get_all_enemies();
    let all_pieces = our_pieces | enemy_pieces;

    let mut pinned_pieces = 0u64;
    let mut checking_pieces = 0u64;

    // Check all 8 directions from king
    for direction in 0..8 {
        let ray = RAY_ATTACKS[direction][king_sq];
        let pieces_on_ray = ray & all_pieces;

        if pieces_on_ray != 0 {
            let closest_piece_sq = if direction <= BOTTOM {
                pieces_on_ray.trailing_zeros() as usize
            } else {
                63 - pieces_on_ray.leading_zeros() as usize
            };

            // Our piece - check for pin
            if (our_pieces >> closest_piece_sq) & 1 != 0 {
                // Gets the remainder of the ray, excluding the closest piece
                let remaining_ray =
                    ray & !(BETWEEN[king_sq][closest_piece_sq] | (1u64 << closest_piece_sq));

                let enemy_sliders =
                    remaining_ray & enemy_pieces & get_slider_mask_for_direction(game, direction);

                if enemy_sliders != 0 {
                    pinned_pieces |= 1u64 << closest_piece_sq;
                }
            } else {
                // Enemy piece - check for check
                if is_slider_attacking_in_direction(game, closest_piece_sq, direction) {
                    checking_pieces |= 1u64 << closest_piece_sq;
                }
            }
        }
    }

    // Knight and pawn checks
    checking_pieces |= find_knight_checks(game, king_sq);
    checking_pieces |= find_pawn_checks(game, king_sq);

    // Calculate check mask
    let check_mask = if checking_pieces == 0 {
        0xFFFFFFFFFFFFFFFFu64
    } else if checking_pieces.count_ones() == 1 {
        let checker_sq = checking_pieces.trailing_zeros() as usize;
        // we add the `checking_piece` so the check mask now is anything in between the checking piece and the king
        // OR the capture of the checking piece itself
        BETWEEN[king_sq][checker_sq] | checking_pieces
    } else {
        0u64 // Double check
    };

    (pinned_pieces, checking_pieces, check_mask)
}

#[inline(always)]
fn find_king_square(game: &Game) -> usize {
    game.friendly_board(Piece::King).trailing_zeros() as usize
}

#[inline(always)]
fn get_slider_mask_for_direction(game: &Game, direction: usize) -> u64 {
    match direction {
        TOP | BOTTOM => game.enemy_board(Piece::Rook) | game.enemy_board(Piece::Queen),
        RIGHT | LEFT => game.enemy_board(Piece::Rook) | game.enemy_board(Piece::Queen),
        TOP_RIGHT | BOTTOM_LEFT => game.enemy_board(Piece::Bishop) | game.enemy_board(Piece::Queen),
        TOP_LEFT | BOTTOM_RIGHT => game.enemy_board(Piece::Bishop) | game.enemy_board(Piece::Queen),
        _ => 0,
    }
}

#[inline(always)]
fn is_slider_attacking_in_direction(game: &Game, piece_sq: usize, direction: usize) -> bool {
    let piece_kind = game.piece_at(piece_sq);
    match direction {
        TOP | BOTTOM | RIGHT | LEFT => {
            matches!(
                piece_kind,
                PieceKind::WhiteRook
                    | PieceKind::BlackRook
                    | PieceKind::WhiteQueen
                    | PieceKind::BlackQueen
            )
        }
        TOP_RIGHT | BOTTOM_LEFT | TOP_LEFT | BOTTOM_RIGHT => {
            matches!(
                piece_kind,
                PieceKind::WhiteBishop
                    | PieceKind::BlackBishop
                    | PieceKind::WhiteQueen
                    | PieceKind::BlackQueen
            )
        }
        _ => false,
    }
}

#[inline(always)]
fn find_knight_checks(game: &Game, king_sq: usize) -> u64 {
    let enemy_knights = game.enemy_board(Piece::Knight);
    let king_knight_attacks = KNIGHT_ATTACKS[king_sq];
    enemy_knights & king_knight_attacks
}

#[inline(always)]
fn find_pawn_checks(game: &Game, king_sq: usize) -> u64 {
    let enemy_pawns = game.enemy_board(Piece::Pawn);
    let enemy_turn = 1 - game.turn; // Flip turn
    let king_pawn_attacks = PAWN_ATTACKS[enemy_turn as usize][king_sq];
    enemy_pawns & king_pawn_attacks
}

// Helper functions for move generation
#[inline(always)]
pub fn get_rook_attacks(square: usize, blockers: u64) -> u64 {
    let mask = ROOK_MASKS[square];
    let idx = unsafe { _pext_u64(blockers, mask) as usize };
    ROOK_ATTACKS[square][idx]
}

#[inline(always)]
pub fn get_bishop_attacks(square: usize, blockers: u64) -> u64 {
    let mask = BISHOP_MASKS[square];
    let idx = unsafe { _pext_u64(blockers, mask) as usize };
    BISHOP_ATTACKS[square][idx]
}

#[inline(always)]
pub fn get_queen_attacks(square: usize, blockers: u64) -> u64 {
    get_rook_attacks(square, blockers) | get_bishop_attacks(square, blockers)
}

#[cfg(test)]
mod test_pins_checks {
    use game::{game::Game, piece::PieceKind};
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    use crate::pins_checks::find_pins_and_checks;

    #[test]
    fn test_pins_n_checks() {
        let mut g = Game::new();
        g.remove_piece(PieceKind::WhitePawn, "e2".idx());
        g.remove_piece(PieceKind::BlackPawn, "e7".idx());
        g.add_piece(PieceKind::BlackQueen, "e7".idx());

        let (pinned, checking, checkmask) = find_pins_and_checks(&g);
        pinned.print();
        checking.print();
        checkmask.print();
    }
}
