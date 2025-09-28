use std::arch::x86_64::_pext_u64;

use pext::{BISHOP_ATTACKS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MASKS};
use utilities::board::PrintAsBoard;

use crate::{
    game::Game,
    piece::Piece::*,
    pins_checks::{
        BETWEEN, RAY_ATTACKS,
        direction_consts::{
            BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, LEFT, RIGHT, TOP, TOP_LEFT, TOP_RIGHT,
        },
    },
};

impl Game {
    fn generate_queen_moves(&self, pinned: u64, check_mask: u64) {
        // We cant filter out the queen like self.friendly(Queen) & !pinned because for
        // sliding pieces they can still move along the check_mask even if pinned
        let mut queens = self.friendly_board(Queen);
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        let friendly_pieces = self.get_all_friendlies();

        while queens != 0 {
            let from = queens.trailing_zeros() as usize;
            queens &= queens - 1;

            // check if this queen is pinned
            // if so, it can only move along the pin ray
            let legal_moves = if (pinned >> from) & 1 != 0 {
                get_pin_ray_moves_for_queen(from, king_sq, all_pieces, friendly_pieces) & check_mask
            } else {
                // If not it could still be in check or can just be a normal move
                // Combine rook and bishop attacks
                let rook_mask_idx = unsafe { _pext_u64(all_pieces, ROOK_MASKS[from]) };
                let bishop_mask_idx = unsafe { _pext_u64(all_pieces, BISHOP_MASKS[from]) };

                (ROOK_ATTACKS[from][rook_mask_idx as usize]
                    | BISHOP_ATTACKS[from][bishop_mask_idx as usize])
                    & !friendly_pieces
                    & check_mask
            };

            println!("Queen Moves for queen on {from}:");
            legal_moves.print();
        }
    }
}

fn get_pin_ray_moves_for_queen(
    queen_sq: usize,
    king_sq: usize,
    all_pieces: u64,
    friendly_pieces: u64,
) -> u64 {
    // Find which direction the pin is in by checking which ray from the king contains the queen
    // Queens can be pinned along any of the 8 directions
    for direction in 0..8 {
        let ray = RAY_ATTACKS[direction][king_sq];
        if (ray >> queen_sq) & 1 != 0 {
            // The piece is on this ray, so it can move along this ray
            // Queen is pinned along this axis - it can move in BOTH directions along this axis
            let opposite_direction = get_opposite_direction(direction);
            //
            // Filter out the friendly_pieces cause get_sliding_attacks_in_direction returns upto
            // and INCLUDING the closest piece, which could've been a friendly
            return (get_sliding_attacks_in_direction(queen_sq, direction, all_pieces)
                | get_sliding_attacks_in_direction(queen_sq, opposite_direction, all_pieces))
                & !friendly_pieces;
        }
    }
    // If we get here, something's wrong - the queen should be on one of the above rays
    0
}

fn get_opposite_direction(direction: usize) -> usize {
    match direction {
        TOP => BOTTOM,
        BOTTOM => TOP,
        LEFT => RIGHT,
        RIGHT => LEFT,
        TOP_LEFT => BOTTOM_RIGHT,
        BOTTOM_RIGHT => TOP_LEFT,
        TOP_RIGHT => BOTTOM_LEFT,
        BOTTOM_LEFT => TOP_RIGHT,
        _ => direction,
    }
}

fn get_sliding_attacks_in_direction(from: usize, direction: usize, all_pieces: u64) -> u64 {
    let ray = RAY_ATTACKS[direction][from];
    let pieces_on_the_ray = all_pieces & ray;
    if pieces_on_the_ray == 0 {
        return ray;
    }

    // Find the FIRST piece in the direction we're moving from the queen
    let blocking_piece_sq = if is_direction_increasing(direction, from) {
        pieces_on_the_ray.trailing_zeros() as usize // First piece in increasing direction
    } else {
        63 - pieces_on_the_ray.leading_zeros() as usize // First piece in decreasing direction
    };

    BETWEEN[from][blocking_piece_sq] | (1u64 << blocking_piece_sq)
}

fn is_direction_increasing(direction: usize, _from: usize) -> bool {
    match direction {
        TOP | TOP_RIGHT | RIGHT | TOP_LEFT => true, // These go to higher square numbers
        BOTTOM | BOTTOM_LEFT | LEFT | BOTTOM_RIGHT => false, // These go to lower square numbers
        _ => true,
    }
}

#[cfg(test)]
mod test_queens_legal {
    use utilities::board::PrintAsBoard;

    use crate::{game::Game, pins_checks::pin_check_finder::find_pins_n_checks};

    #[test]
    fn test_queen_legal() {
        // Test positions with queen pins and checks
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1", // Queen can move
            "8/8/3k4/8/8/3K4/3Q4/8 w - - 0 1",                          // Simple queen test
            "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1", // Queen in starting position
            "8/8/6k1/3q4/8/8/8/3K4 b - - 0 1", // Black queen vs white king
        ];

        for position in positions {
            println!("================");
            let g = Game::from_fen(position);
            let (pinned, _checking, check_mask) = find_pins_n_checks(&g);
            println!("Position: {}", position);
            println!("Pinned:");
            pinned.print();
            println!("Checking:");
            _checking.print();
            println!("CheckMask:");
            check_mask.print();
            g.generate_queen_moves(pinned, check_mask);
            println!("================");
        }
    }
}
