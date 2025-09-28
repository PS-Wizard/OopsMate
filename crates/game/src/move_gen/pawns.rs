use pext::PAWN_ATTACKS;
use utilities::board::PrintAsBoard;

use crate::{
    game::Game,
    piece::Piece::*,
    pins_checks::{
        RAY_ATTACKS,
        direction_consts::{BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, TOP, TOP_LEFT, TOP_RIGHT},
    },
};

impl Game {
    fn generate_pawn_moves(&self, pinned: u64, check_mask: u64) {
        let mut pawns = self.friendly_board(Pawn);
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        let enemy_pieces = self.get_all_enemies();

        println!("=== Pawn Move Generation ===");

        while pawns != 0 {
            let from = pawns.trailing_zeros() as usize;
            pawns &= pawns - 1;

            let mut _legal_moves = 0u64;

            if (pinned >> from) & 1 != 0 {
                // Pawn is pinned - very restricted movement
                _legal_moves =
                    get_pinned_pawn_moves(from, king_sq, all_pieces, enemy_pieces, self.turn);
            } else {
                // Pawn is not pinned - normal pawn moves
                _legal_moves = get_normal_pawn_moves(from, all_pieces, enemy_pieces, self.turn);

                // Add en passant moves if available
                if self.en_passant != 0 {
                    _legal_moves |=
                        get_en_passant_moves(from, self.en_passant as usize, self.turn, self);
                }
            }

            // Apply check mask
            _legal_moves &= check_mask;

            if _legal_moves != 0 {
                println!("Pawn moves for pawn on {from}:");
                _legal_moves.print();

                // Check for promotion moves
                let promotion_moves = get_promotion_moves(_legal_moves, self.turn);
                if promotion_moves != 0 {
                    println!("Promotion squares:");
                    promotion_moves.print();
                }
            }
        }
    }
}

fn get_normal_pawn_moves(from: usize, all_pieces: u64, enemy_pieces: u64, turn: u8) -> u64 {
    let mut moves = 0u64;

    let (forward_offset, starting_rank, _double_move_rank) = if turn == 0 {
        (8, 1, 3) // White: move up, start on rank 2, double move to rank 4
    } else {
        (-8i8 as u8, 6, 4) // Black: move down, start on rank 7, double move to rank 5
    };

    let rank = from / 8;
    let forward_sq = (from as i8 + forward_offset as i8) as usize;

    // Single square forward (if not blocked)
    if forward_sq < 64 && (all_pieces & (1u64 << forward_sq)) == 0 {
        moves |= 1u64 << forward_sq;

        // Double square forward from starting position (if not blocked)
        if rank == starting_rank {
            let double_forward_sq = (forward_sq as i8 + forward_offset as i8) as usize;
            if double_forward_sq < 64 && (all_pieces & (1u64 << double_forward_sq)) == 0 {
                moves |= 1u64 << double_forward_sq;
            }
        }
    }

    // Diagonal captures
    moves |= PAWN_ATTACKS[turn as usize][from] & enemy_pieces;

    moves
}

fn get_pinned_pawn_moves(
    pawn_sq: usize,
    king_sq: usize,
    all_pieces: u64,
    enemy_pieces: u64,
    turn: u8,
) -> u64 {
    // Find which direction the pin is along
    let pin_direction = find_pin_direction(pawn_sq, king_sq);
    if pin_direction.is_none() {
        return 0;
    }

    let direction = pin_direction.unwrap();
    let mut moves = 0u64;

    match direction {
        // Vertical pin (file pin) - pawn can move forward/backward along the file
        TOP | BOTTOM => {
            moves |= get_forward_pawn_moves_along_pin(pawn_sq, all_pieces, turn);
        }
        // Diagonal pin - pawn might be able to capture along the pin diagonal
        TOP_LEFT | TOP_RIGHT | BOTTOM_LEFT | BOTTOM_RIGHT => {
            moves |= get_diagonal_pawn_moves_along_pin(pawn_sq, direction, enemy_pieces, turn);
        }
        _ => {} // Horizontal pins block all pawn movement
    }

    moves
}

fn find_pin_direction(pawn_sq: usize, king_sq: usize) -> Option<usize> {
    use crate::pins_checks::direction_consts::*;

    // Check all 8 directions to find which ray the pawn is on
    for direction in [
        TOP,
        TOP_RIGHT,
        RIGHT,
        BOTTOM_RIGHT,
        BOTTOM,
        BOTTOM_LEFT,
        LEFT,
        TOP_LEFT,
    ] {
        let ray = RAY_ATTACKS[direction][king_sq];
        if (ray >> pawn_sq) & 1 != 0 {
            return Some(direction);
        }
    }
    None
}

fn get_forward_pawn_moves_along_pin(pawn_sq: usize, all_pieces: u64, turn: u8) -> u64 {
    let mut moves = 0u64;
    let forward_offset = if turn == 0 { 8 } else { -8i8 as u8 };
    let forward_sq = (pawn_sq as i8 + forward_offset as i8) as usize;

    // Single move forward (if not blocked)
    if forward_sq < 64 && (all_pieces & (1u64 << forward_sq)) == 0 {
        moves |= 1u64 << forward_sq;

        // Double move if on starting rank
        let starting_rank = if turn == 0 { 1 } else { 6 };
        if pawn_sq / 8 == starting_rank {
            let double_forward_sq = (forward_sq as i8 + forward_offset as i8) as usize;
            if double_forward_sq < 64 && (all_pieces & (1u64 << double_forward_sq)) == 0 {
                moves |= 1u64 << double_forward_sq;
            }
        }
    }

    moves
}

fn get_diagonal_pawn_moves_along_pin(
    pawn_sq: usize,
    pin_direction: usize,
    enemy_pieces: u64,
    turn: u8,
) -> u64 {
    // Pawn can only capture along the pin diagonal
    let pawn_attacks = PAWN_ATTACKS[turn as usize][pawn_sq] & enemy_pieces;

    // Filter to only include captures along the pin direction
    let mut valid_captures = 0u64;
    let mut attacks = pawn_attacks;

    while attacks != 0 {
        let capture_sq = attacks.trailing_zeros() as usize;
        attacks &= attacks - 1;

        // Check if this capture is along the pin direction
        let capture_direction = get_direction_between_squares(pawn_sq, capture_sq);
        if capture_direction == Some(pin_direction) {
            valid_captures |= 1u64 << capture_sq;
        }
    }

    valid_captures
}

fn get_direction_between_squares(from: usize, to: usize) -> Option<usize> {
    use crate::pins_checks::direction_consts::*;

    let from_rank = from / 8;
    let from_file = from % 8;
    let to_rank = to / 8;
    let to_file = to % 8;

    let rank_diff = to_rank as i8 - from_rank as i8;
    let file_diff = to_file as i8 - from_file as i8;

    match (rank_diff.signum(), file_diff.signum()) {
        (1, 0) => Some(TOP),
        (-1, 0) => Some(BOTTOM),
        (0, 1) => Some(RIGHT),
        (0, -1) => Some(LEFT),
        (1, 1) => Some(TOP_RIGHT),
        (1, -1) => Some(TOP_LEFT),
        (-1, 1) => Some(BOTTOM_RIGHT),
        (-1, -1) => Some(BOTTOM_LEFT),
        _ => None,
    }
}

fn get_en_passant_moves(pawn_sq: usize, en_passant_sq: usize, turn: u8, game: &Game) -> u64 {
    // Check if this pawn can capture en passant
    let pawn_attacks = PAWN_ATTACKS[turn as usize][pawn_sq];
    if (pawn_attacks >> en_passant_sq) & 1 == 0 {
        return 0;
    }

    // Calculate the square of the captured pawn
    let captured_pawn_sq = if turn == 0 {
        en_passant_sq - 8 // White captures black pawn one rank below
    } else {
        en_passant_sq + 8 // Black captures white pawn one rank above
    };

    // Simulate the en passant capture and check if king would be in check
    if is_en_passant_legal(pawn_sq, en_passant_sq, captured_pawn_sq, game) {
        1u64 << en_passant_sq
    } else {
        0
    }
}

fn is_en_passant_legal(
    pawn_sq: usize,
    en_passant_sq: usize,
    captured_pawn_sq: usize,
    game: &Game,
) -> bool {
    let king_sq = game.friendly_board(King).trailing_zeros() as usize;

    // Create a temporary board state after en passant
    let mut all_pieces = game.white_occupied | game.black_occupied;
    all_pieces &= !(1u64 << pawn_sq); // Remove capturing pawn
    all_pieces &= !(1u64 << captured_pawn_sq); // Remove captured pawn  
    all_pieces |= 1u64 << en_passant_sq; // Add pawn to new position

    // Check if king would be in check in this new position
    !is_square_attacked_by_enemy(king_sq, game.get_all_enemies(), all_pieces, game)
}

// Helper function for en passant validation - checks if a square is attacked
fn is_square_attacked_by_enemy(
    square: usize,
    enemy_pieces: u64,
    all_pieces: u64,
    game: &Game,
) -> bool {
    // Check for enemy pawn attacks
    if (PAWN_ATTACKS[game.turn as usize][square] & game.enemy_board(Pawn)) != 0 {
        return true;
    }

    // Check for enemy knight attacks
    use pext::KNIGHT_ATTACKS;
    if (KNIGHT_ATTACKS[square] & game.enemy_board(Knight)) != 0 {
        return true;
    }

    // Check for enemy king attacks
    use pext::KING_ATTACKS;
    if (KING_ATTACKS[square] & game.enemy_board(King)) != 0 {
        return true;
    }

    // Check for enemy sliding piece attacks (rooks, bishops, queens)
    is_square_attacked_by_sliders(square, enemy_pieces, all_pieces, game)
}

fn is_direction_increasing_for_validation(direction: usize) -> bool {
    use crate::pins_checks::direction_consts::*;
    match direction {
        TOP | TOP_RIGHT | RIGHT | TOP_LEFT => true,
        BOTTOM | BOTTOM_LEFT | LEFT | BOTTOM_RIGHT => false,
        _ => true,
    }
}

fn is_square_attacked_by_sliders(
    square: usize,
    _enemy_pieces: u64,
    all_pieces: u64,
    game: &Game,
) -> bool {
    use crate::piece::PieceKind::*;
    use crate::pins_checks::{RAY_ATTACKS, direction_consts::*};

    // Check all 8 directions for attacking sliders
    for direction in [
        TOP,
        TOP_RIGHT,
        RIGHT,
        BOTTOM_RIGHT,
        BOTTOM,
        BOTTOM_LEFT,
        LEFT,
        TOP_LEFT,
    ] {
        let ray = RAY_ATTACKS[direction][square];
        let pieces_on_ray = ray & all_pieces;

        if pieces_on_ray != 0 {
            // Find the first piece in this direction
            let first_piece_sq = if is_direction_increasing_for_validation(direction) {
                pieces_on_ray.trailing_zeros() as usize
            } else {
                63 - pieces_on_ray.leading_zeros() as usize
            };

            // Check if it's an enemy slider that can attack in this direction
            let piece = game.piece_at(first_piece_sq);
            let is_enemy = match game.turn {
                0 => matches!(piece, BlackRook | BlackBishop | BlackQueen),
                1 => matches!(piece, WhiteRook | WhiteBishop | WhiteQueen),
                _ => false,
            };

            if is_enemy {
                let can_attack = match direction {
                    TOP | BOTTOM | RIGHT | LEFT => {
                        matches!(piece, WhiteRook | BlackRook | WhiteQueen | BlackQueen)
                    }
                    TOP_RIGHT | BOTTOM_LEFT | TOP_LEFT | BOTTOM_RIGHT => {
                        matches!(piece, WhiteBishop | BlackBishop | WhiteQueen | BlackQueen)
                    }
                    _ => false,
                };

                if can_attack {
                    return true;
                }
            }
        }
    }

    false
}

fn get_promotion_moves(moves: u64, turn: u8) -> u64 {
    let promotion_rank = if turn == 0 { 7 } else { 0 }; // 8th rank for white, 1st rank for black

    // Filter moves to only include those that reach the promotion rank
    moves & (0xFF << (promotion_rank * 8))
}

#[cfg(test)]
mod test_pawn_legal {
    use crate::{game::Game, pins_checks::pin_check_finder::find_pins_n_checks};
    use utilities::board::PrintAsBoard;

    #[test]
    fn test_pawn_legal() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", // En passant available
            "4k3/P7/8/8/8/8/7p/2K5 w - - 0 1",                          // Promotion test
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1", // Pinned pawns
            "5k2/8/8/1KpP1r2/8/8/8/8 w - - 0 1",                        // En passant capture
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
            g.generate_pawn_moves(pinned, check_mask);
            println!("================");
        }
    }
}
