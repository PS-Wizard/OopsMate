#![allow(dead_code)]
use pext::{KNIGHT_ATTACKS, PAWN_ATTACKS};

use crate::{
    game::Game,
    piece::{Piece::*, PieceKind::*},
    pins_checks::{
        direction_consts::{
            BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, LEFT, RIGHT, TOP, TOP_LEFT, TOP_RIGHT,
        },
        gen_between_attacks::BETWEEN,
        gen_ray_attacks::RAY_ATTACKS,
    },
};

fn find_pins_n_checks(g: &Game) -> (u64, u64, u64) {
    let king_sq = g.friendly_board(King).trailing_zeros() as usize;
    let friendly = g.get_all_friendlies();
    let enemy = g.get_all_enemies();
    let all_pieces = friendly | enemy;

    let mut pinned_pieces = 0u64;
    let mut checking_pieces = 0u64;

    // Loop through all 8 directions, defined in `mod.rs`, basically loop thru TOP, TOP_RIGHT,
    // RIGHT, BOTTOM_RIGHT, BOTTOM ...
    for direction in 0..8 {
        let ray = RAY_ATTACKS[direction][king_sq];
        let pieces_along_the_ray = ray & all_pieces;
        if pieces_along_the_ray != 0 {
            // If the direction is less than or equal bottom, the ray goes from smaller values to larger
            // values, so closest will be the LSB
            let closest_piece_sq = if direction <= BOTTOM {
                pieces_along_the_ray.trailing_zeros() as usize
            } else {
                // If the direction is more than bottom, the ray goes from larger values to smaller
                // values, so closest will be the MSB
                63 - pieces_along_the_ray.leading_zeros() as usize
            };

            // Check if the piece is friendly or enemy
            // If Friendly:
            if (friendly >> closest_piece_sq) & 1 != 0 {
                // get the remainder of the ray excluding the closest piece
                let remaining_ray =
                    ray & !(BETWEEN[king_sq][closest_piece_sq] | (1u64 << closest_piece_sq));
                let enemy_sliders =
                    remaining_ray & enemy & get_slider_mask_for_direction(g, direction);

                // If Enemy Slider Exists, our friendly piece is pinned
                if enemy_sliders != 0 {
                    pinned_pieces |= 1 << closest_piece_sq;
                }
            } else {
                // If Enemy: the closest piece is the enemy, check to see if that enemy is a slider
                // that can attack in the `direction`
                if is_slider_attacking_in_direction(g, closest_piece_sq, direction) {
                    checking_pieces |= 1u64 << closest_piece_sq;
                }
            }
        }
    }
    // The above covers the pins & checks for sliders, now for stuff like knights & pawns
    // Because they cant really pin, its just a check caluclation for them

    // Basically get all the knight moves from the square of the **KING** and AND it with the
    // knight occupancy of the enemy, if something pops up, youre in check.
    checking_pieces |= g.enemy_board(Knight) & KNIGHT_ATTACKS[king_sq];

    // Pretty much the same thing for the pawn as well
    checking_pieces |= g.enemy_board(Pawn) & PAWN_ATTACKS[(1 - g.turn) as usize][king_sq];

    // Finally calculate the check mask
    let check_mask = if checking_pieces == 0 {
        // No Checks, all 1's the pieces are allowed to move freely
        0xFFFFFFFFFFFFFFFFu64
    } else if checking_pieces.count_ones() == 1 {
        let checker_sq = checking_pieces.trailing_zeros() as usize;
        // we add the `checking_piece` so the check mask now is anything in between the checking piece and the king
        // OR the capture of the checking piece itself
        BETWEEN[king_sq][checker_sq] | checking_pieces
    } else {
        // This is double check, and only legal move is to move the king himself.
        0
    };

    (pinned_pieces, checking_pieces, check_mask)
}

/// Checks if the piece at `piece_sq` can attack in the specified direction
fn is_slider_attacking_in_direction(game: &Game, piece_sq: usize, direction: usize) -> bool {
    let piece_kind = game.piece_at(piece_sq);
    match direction {
        TOP | BOTTOM | RIGHT | LEFT => {
            matches!(piece_kind, WhiteRook | BlackRook | WhiteQueen | BlackQueen)
        }
        TOP_RIGHT | BOTTOM_LEFT | TOP_LEFT | BOTTOM_RIGHT => {
            matches!(
                piece_kind,
                WhiteBishop | BlackBishop | WhiteQueen | BlackQueen
            )
        }
        _ => false,
    }
}

/// Gets the bitboard of pieces that can travel along a specified direction
fn get_slider_mask_for_direction(g: &Game, direction: usize) -> u64 {
    match direction {
        // Queens & Rooks can travel Vertically Or Horizontally
        TOP | BOTTOM | LEFT | RIGHT => g.enemy_board(Rook) | g.enemy_board(Queen),
        // Queens and bishops can travel tagonally
        TOP_RIGHT | TOP_LEFT | BOTTOM_RIGHT | BOTTOM_LEFT => {
            g.enemy_board(Bishop) | g.enemy_board(Queen)
        }
        _ => 0,
    }
}

#[cfg(test)]
mod test_find_pins_checks {
    use utilities::algebraic::Algebraic;
    use utilities::board::PrintAsBoard;

    use crate::game::Game;
    use crate::piece::PieceKind::*;
    use crate::pins_checks::pin_check_finder::find_pins_n_checks;

    #[test]
    fn test_find_checks_pins_mask() {
        let mut g = Game::new();
        g.remove_piece(WhitePawn, "e2".idx());
        g.remove_piece(BlackPawn, "e7".idx());
        g.move_piece(BlackQueen, "d8".idx(), "e7".idx());
        let (pinned, checkers, check_mask) = find_pins_n_checks(&g);
        println!("Pinned: ");
        pinned.print();
        println!("Checking Pieces:");
        checkers.print();
        println!("Check Mask:");
        check_mask.print();
    }
}
