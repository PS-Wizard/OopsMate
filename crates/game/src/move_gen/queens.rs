use std::arch::x86_64::_pext_u64;

use pext::{BISHOP_ATTACKS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MASKS};

use crate::{
    game::Game,
    move_gen::{Move, MoveGenerator},
    piece::Piece::*,
    pins_checks::move_type::mv_flags,
    pins_checks::{
        BETWEEN, RAY_ATTACKS,
        direction_consts::{
            BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, LEFT, RIGHT, TOP, TOP_LEFT, TOP_RIGHT,
        },
    },
};

impl Game {
    pub fn generate_queen_moves(&self, pinned: u64, check_mask: u64, move_gen: &mut MoveGenerator) {
        let mut queens = self.friendly_board(Queen);
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        let friendly_pieces = self.get_all_friendlies();
        let enemy_pieces = self.get_all_enemies();

        while queens != 0 {
            let from = queens.trailing_zeros() as usize;
            queens &= queens - 1;

            // Get legal move squares for this queen
            let legal_moves = if (pinned >> from) & 1 != 0 {
                get_pin_ray_moves_for_queen(from, king_sq, all_pieces, friendly_pieces) & check_mask
            } else {
                // Combine rook and bishop attacks
                let rook_mask_idx = unsafe { _pext_u64(all_pieces, ROOK_MASKS[from]) };
                let bishop_mask_idx = unsafe { _pext_u64(all_pieces, BISHOP_MASKS[from]) };

                (ROOK_ATTACKS[from][rook_mask_idx as usize]
                    | BISHOP_ATTACKS[from][bishop_mask_idx as usize])
                    & !friendly_pieces
                    & check_mask
            };

            // Convert bitboard to individual moves
            add_moves_from_bitboard(legal_moves, from, enemy_pieces, move_gen);
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

            return (get_sliding_attacks_in_direction(queen_sq, direction, all_pieces)
                | get_sliding_attacks_in_direction(queen_sq, opposite_direction, all_pieces))
                & !friendly_pieces;
        }
    }
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

    let blocking_piece_sq = if is_direction_increasing(direction, from) {
        pieces_on_the_ray.trailing_zeros() as usize
    } else {
        63 - pieces_on_the_ray.leading_zeros() as usize
    };

    BETWEEN[from][blocking_piece_sq] | (1u64 << blocking_piece_sq)
}

fn is_direction_increasing(direction: usize, _from: usize) -> bool {
    match direction {
        TOP | TOP_RIGHT | RIGHT | TOP_LEFT => true,
        BOTTOM | BOTTOM_LEFT | LEFT | BOTTOM_RIGHT => false,
        _ => true,
    }
}

fn add_moves_from_bitboard(
    moves_bitboard: u64,
    from_sq: usize,
    enemy_pieces: u64,
    move_gen: &mut MoveGenerator,
) {
    let mut moves = moves_bitboard;
    while moves != 0 {
        let to_sq = moves.trailing_zeros() as usize;
        moves &= moves - 1;

        // Determine flags based on move type
        let flags = if (enemy_pieces >> to_sq) & 1 != 0 {
            mv_flags::CAPT // Capture
        } else {
            mv_flags::NONE // Normal move
        };

        // Create and add the move
        let mv = Move::new(from_sq as u16, to_sq as u16, flags);
        move_gen.moves[move_gen.count] = mv;
        move_gen.count += 1;

        // Safety check to prevent buffer overflow
        if move_gen.count >= move_gen.moves.len() {
            break;
        }
    }
}

#[cfg(test)]
mod test_queens_legal {
    use crate::{
        game::Game,
        move_gen::{Move, MoveGenerator},
        pins_checks::{
            move_type::mv_flags::{CAPT, NONE},
            pin_check_finder::find_pins_n_checks,
        },
    };

    #[test]
    fn test_queen_legal() {
        let positions = [
            // starting expected 0 moves
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            // expected 9
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1", 
            // expected 17
            "8/8/3k4/8/8/3K4/3Q4/8 w - - 0 1",
            // expected 1
            "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
            // expected 27
            "8/8/6k1/3q4/8/8/8/3K4 b - - 0 1",
        ];

        for position in positions {
            println!("================");
            let g = Game::from_fen(position);
            let (pinned, _checking, check_mask) = find_pins_n_checks(&g);
            println!("Position: {}", position);

            let mut move_gen = MoveGenerator {
                moves: [Move::from_u16(0); 256],
                count: 0,
            };

            g.generate_queen_moves(pinned, check_mask, &mut move_gen);

            println!("Generated {} queen moves:", move_gen.count);
            for i in 0..move_gen.count {
                let mv = move_gen.moves[i];
                let flags_str = match mv.flags() {
                    CAPT => " (capture)",
                    NONE => "",
                    _ => " (other)",
                };
                println!("  {} -> {}{}", mv.from_sq(), mv.to_sq(), flags_str);
            }
            println!("================");
        }
    }
}
