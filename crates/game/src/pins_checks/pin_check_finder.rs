#![allow(dead_code)]
use pext::{KNIGHT_ATTACKS, PAWN_ATTACKS};

use crate::{
    game::Game,
    piece::{Piece::*, PieceKind::*},
    pins_checks::{
        BETWEEN, RAY_ATTACKS,
        direction_consts::{
            BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, LEFT, RIGHT, TOP, TOP_LEFT, TOP_RIGHT,
        },
    },
};

pub fn find_pins_n_checks(g: &Game) -> (u64, u64, u64) {
    let king_sq = g.friendly_board(King).trailing_zeros() as usize;
    let friendly = g.get_all_friendlies();
    let enemy = g.get_all_enemies();
    let all_pieces = friendly | enemy;

    let mut pinned_pieces = 0u64;
    let mut checking_pieces = 0u64;

    // Loop through all 8 directions
    for direction in 0..8 {
        let ray = RAY_ATTACKS[direction][king_sq];
        let pieces_along_the_ray = ray & all_pieces;
        if pieces_along_the_ray != 0 {
            let closest_piece_sq = find_closest_piece_on_ray(pieces_along_the_ray, king_sq);

            // Check if the piece is friendly or enemy
            if (friendly >> closest_piece_sq) & 1 != 0 {
                // Friendly piece - check for pins
                let remaining_ray =
                    ray & !(BETWEEN[king_sq][closest_piece_sq] | (1u64 << closest_piece_sq));
                let potential_sliders =
                    remaining_ray & enemy & get_slider_mask_for_direction(g, direction);

                // Check each potential slider to see if it has a clear line of sight
                let mut actual_pinning_pieces = 0u64;
                let mut temp_sliders = potential_sliders;

                while temp_sliders != 0 {
                    let slider_sq = temp_sliders.trailing_zeros() as usize;
                    temp_sliders &= temp_sliders - 1;

                    // Check if this slider has a clear path to the friendly piece
                    let between_slider_and_friendly = BETWEEN[slider_sq][closest_piece_sq];
                    if (between_slider_and_friendly & all_pieces) == 0 {
                        // Clear path - this slider is actually pinning
                        actual_pinning_pieces |= 1u64 << slider_sq;
                    }
                }

                if actual_pinning_pieces != 0 {
                    pinned_pieces |= 1 << closest_piece_sq;
                }
            } else {
                // Enemy piece - check if it's giving check
                if is_slider_attacking_in_direction(g, closest_piece_sq, direction) {
                    checking_pieces |= 1u64 << closest_piece_sq;
                }
            }
        }
    }

    // Handle knight and pawn checks (they can't pin)
    checking_pieces |= g.enemy_board(Knight) & KNIGHT_ATTACKS[king_sq];
    checking_pieces |= g.enemy_board(Pawn) & PAWN_ATTACKS[(1 - g.turn) as usize][king_sq];

    // Calculate check mask
    let check_mask = if checking_pieces == 0 {
        // No checks - all moves allowed
        0xFFFFFFFFFFFFFFFFu64
    } else if checking_pieces.count_ones() == 1 {
        let checker_sq = checking_pieces.trailing_zeros() as usize;
        // Can block or capture the checking piece
        BETWEEN[king_sq][checker_sq] | checking_pieces
    } else {
        // Double check - only king moves allowed
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
        // Queens and bishops can travel diagonally
        TOP_RIGHT | TOP_LEFT | BOTTOM_RIGHT | BOTTOM_LEFT => {
            g.enemy_board(Bishop) | g.enemy_board(Queen)
        }
        _ => 0,
    }
}

/// Finds the closest piece to the king on a ray by checking both ends
pub fn find_closest_piece_on_ray(pieces_along_ray: u64, king_sq: usize) -> usize {
    if pieces_along_ray == 0 {
        return 0; // No pieces on ray
    }

    // Get the lowest and highest square numbers with pieces
    let lowest_sq = pieces_along_ray.trailing_zeros() as usize;
    let highest_sq = 63 - pieces_along_ray.leading_zeros() as usize;

    // Calculate distances from king to both ends
    let distance_to_lowest = square_distance(king_sq, lowest_sq);
    let distance_to_highest = square_distance(king_sq, highest_sq);

    // Return the square that's closer to the king
    if distance_to_lowest <= distance_to_highest {
        lowest_sq
    } else {
        highest_sq
    }
}

/// Calculate the distance between two squares (Chebyshev distance)
fn square_distance(sq1: usize, sq2: usize) -> usize {
    let (rank1, file1) = (sq1 / 8, sq1 % 8);
    let (rank2, file2) = (sq2 / 8, sq2 % 8);

    let rank_diff = if rank1 > rank2 {
        rank1 - rank2
    } else {
        rank2 - rank1
    };
    let file_diff = if file1 > file2 {
        file1 - file2
    } else {
        file2 - file1
    };

    // Chebyshev distance (max of rank and file differences)
    if rank_diff > file_diff {
        rank_diff
    } else {
        file_diff
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
