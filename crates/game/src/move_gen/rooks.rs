use std::arch::x86_64::_pext_u64;

use pext::{ROOK_ATTACKS, ROOK_MASKS};

use crate::{
    game::Game,
    move_gen::{Move, MoveGenerator},
    piece::Piece::*,
    pins_checks::move_type::mv_flags,
    pins_checks::{
        BETWEEN, RAY_ATTACKS,
        direction_consts::{BOTTOM, LEFT, RIGHT, TOP},
    },
};

impl Game {
    pub fn generate_rook_moves(&self, pinned: u64, check_mask: u64, move_gen: &mut MoveGenerator) {
        let mut rooks = self.friendly_board(Rook);
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        let friendly_pieces = self.get_all_friendlies();
        let enemy_pieces = self.get_all_enemies();

        while rooks != 0 {
            let from = rooks.trailing_zeros() as usize;
            rooks &= rooks - 1;

            // Get legal move squares for this rook
            let legal_moves = if (pinned >> from) & 1 != 0 {
                get_pin_ray_moves_for_rook(from, king_sq, all_pieces, friendly_pieces) & check_mask
            } else {
                let mask_idx = unsafe { _pext_u64(all_pieces, ROOK_MASKS[from]) };
                ROOK_ATTACKS[from][mask_idx as usize] & !friendly_pieces & check_mask
            };

            // Convert bitboard to individual moves
            add_moves_from_bitboard(legal_moves, from, enemy_pieces, move_gen);
        }
    }
}

fn get_pin_ray_moves_for_rook(
    rook_sq: usize,
    king_sq: usize,
    all_pieces: u64,
    friendly_pieces: u64,
) -> u64 {
    for direction in [TOP, RIGHT, BOTTOM, LEFT] {
        let ray = RAY_ATTACKS[direction][king_sq];
        if (ray >> rook_sq) & 1 != 0 {
            let opposite_direction = match direction {
                TOP => BOTTOM,
                BOTTOM => TOP,
                LEFT => RIGHT,
                RIGHT => LEFT,
                _ => direction,
            };

            return (get_sliding_attacks_in_direction(rook_sq, direction, all_pieces)
                | get_sliding_attacks_in_direction(rook_sq, opposite_direction, all_pieces))
                & !friendly_pieces;
        }
    }
    0
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
        TOP | RIGHT => true,
        BOTTOM | LEFT => false,
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
mod test_rooks_legal {
    use crate::{
        game::Game,
        move_gen::{Move, MoveGenerator},
        pins_checks::{
            move_type::mv_flags::{CAPT, NONE},
            pin_check_finder::find_pins_n_checks,
        },
    };

    #[test]
    fn test_rook_legal() {
        let positions = [
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1",
            "rnb1k1nr/pppp1ppp/8/8/1b5q/8/PP2P1PP/RNBQKBNR w KQkq - 0 1",
            "rnb1k1nr/pppp1ppp/8/8/1b6/8/PP2P1PP/K2Rq3 w kq - 0 1",
            "2Q2r1k/7P/8/8/8/8/8/8 b - - 0 1",
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

            g.generate_rook_moves(pinned, check_mask, &mut move_gen);

            println!("Generated {} rook moves:", move_gen.count);
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
