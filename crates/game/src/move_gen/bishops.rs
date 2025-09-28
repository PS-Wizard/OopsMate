use std::arch::x86_64::_pext_u64;

use pext::{BISHOP_ATTACKS, BISHOP_MASKS};
use utilities::board::PrintAsBoard;

use crate::{
    game::Game,
    piece::Piece::*,
    pins_checks::{
        BETWEEN, RAY_ATTACKS,
        direction_consts::{BOTTOM_LEFT, BOTTOM_RIGHT, TOP_LEFT, TOP_RIGHT},
    },
};

impl Game {
    fn generate_bishop_moves(&self, pinned: u64, check_mask: u64) {
        // We cant filter out the bishop like self.friendly(Bishop) & !pinned because for
        // sliding pieces they can still move along the check_mask even if pinned
        let mut bishops = self.friendly_board(Bishop);
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        let friendly_pieces = self.get_all_friendlies();
        while bishops != 0 {
            let from = bishops.trailing_zeros() as usize;
            bishops &= bishops - 1;
            // check if this bishop is pinned
            // if so, it can only move along the pin ray
            let legal_moves = if (pinned >> from) & 1 != 0 {
                get_pin_ray_moves_for_bishop(from, king_sq, all_pieces, friendly_pieces) & check_mask
            } else {
                // If not it could still be in check or can just be a normal move
                let mask_idx = unsafe {
                    _pext_u64(self.white_occupied | self.black_occupied, BISHOP_MASKS[from])
                };
                // Filter out the moves that don't align with the check_mask
                BISHOP_ATTACKS[from][mask_idx as usize] & !friendly_pieces & check_mask
            };
            println!("Bishop Moves for a bishop on {from}:");
            legal_moves.print();
        }
    }
}

fn get_pin_ray_moves_for_bishop(
    bishop_sq: usize,
    king_sq: usize,
    all_pieces: u64,
    friendly_pieces: u64,
) -> u64 {
    // Find which direction the pin is in by checking which ray from the king contains the bishop
    for direction in [TOP_LEFT, TOP_RIGHT, BOTTOM_LEFT, BOTTOM_RIGHT] {
        let ray = RAY_ATTACKS[direction][king_sq];
        if (ray >> bishop_sq) & 1 != 0 {
            // The piece is on this ray, so it can move along this ray
            // Bishop is pinned along this axis - it can move in BOTH directions along this axis
            let opposite_direction = match direction {
                TOP_LEFT => BOTTOM_RIGHT,
                BOTTOM_RIGHT => TOP_LEFT,
                TOP_RIGHT => BOTTOM_LEFT,
                BOTTOM_LEFT => TOP_RIGHT,
                _ => direction,
            };
            //
            // Filter out the friendly_pieces cause get_sliding_attacks_in_direction returns upto
            // and INCLUDING the closest piece, which could've been a friendly
            return (get_sliding_attacks_in_direction(bishop_sq, direction, all_pieces)
                | get_sliding_attacks_in_direction(bishop_sq, opposite_direction, all_pieces))
                & !friendly_pieces;
        }
    }
    // If we get here, something's wrong - the bishop should be on one of the above rays
    0
}

fn get_sliding_attacks_in_direction(from: usize, direction: usize, all_pieces: u64) -> u64 {
    let ray = RAY_ATTACKS[direction][from];
    let pieces_on_the_ray = all_pieces & ray;
    if pieces_on_the_ray == 0 {
        return ray;
    }

    // Find the FIRST piece in the direction we're moving from the bishop
    let blocking_piece_sq = if is_direction_increasing(direction, from) {
        pieces_on_the_ray.trailing_zeros() as usize // First piece in increasing direction
    } else {
        63 - pieces_on_the_ray.leading_zeros() as usize // First piece in decreasing direction
    };

    BETWEEN[from][blocking_piece_sq] | (1u64 << blocking_piece_sq)
}

fn is_direction_increasing(direction: usize, _from: usize) -> bool {
    match direction {
        TOP_LEFT | TOP_RIGHT => true,        // These go to higher square numbers
        BOTTOM_LEFT | BOTTOM_RIGHT => false, // These go to lower square numbers
        _ => true,
    }
}

#[cfg(test)]
mod test_bishops_legal {
    use utilities::board::PrintAsBoard;

    use crate::{game::Game, pins_checks::pin_check_finder::find_pins_n_checks};

    #[test]
    fn test_bishop_legal() {
        // Test positions with bishop pins and checks
        let positions = [
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1", // Bishop on b4
            "rnbqkbnr/ppp2ppp/8/3pp3/8/3P1N2/PPP1PPPP/RNBQKB1R w KQkq - 0 1", // Normal position
            "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1", // Bishop on b5
            "8/8/3k4/8/8/3K4/3B4/8 w - - 0 1", // Simple bishop test
            "8/8/3k4/8/8/8/8/2KBr3 w - - 0 1" // bishop pinned
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
            g.generate_bishop_moves(pinned, check_mask);
            println!("================");
        }
    }
}
