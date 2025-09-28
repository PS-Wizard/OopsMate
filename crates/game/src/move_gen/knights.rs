use pext::KNIGHT_ATTACKS;

use crate::{
    game::Game,
    move_gen::{Move, MoveGenerator},
    piece::Piece::*,
    pins_checks::move_type::mv_flags,
};

impl Game {
    pub fn generate_knight_moves(
        &self,
        pinned: u64,
        check_mask: u64,
        move_gen: &mut MoveGenerator,
    ) {
        // Knights can't move when pinned (except in very rare cases), so we filter them out
        let mut unpinned_knights = self.friendly_board(Knight) & !pinned;
        let friendly_pieces = self.get_all_friendlies();
        let enemy_pieces = self.get_all_enemies();

        while unpinned_knights != 0 {
            let from = unpinned_knights.trailing_zeros() as usize;
            unpinned_knights &= unpinned_knights - 1;

            // Get all knight attacks from this square that don't capture friendly pieces
            let attacks = KNIGHT_ATTACKS[from] & !friendly_pieces & check_mask;

            // Convert bitboard to individual moves
            add_knight_moves_from_bitboard(attacks, from, enemy_pieces, move_gen);
        }
    }
}

fn add_knight_moves_from_bitboard(
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
mod test_knights_legal {
    use crate::{
        game::Game,
        move_gen::{Move, MoveGenerator},
        pins_checks::{
            move_type::mv_flags::{CAPT, NONE},
            pin_check_finder::find_pins_n_checks,
        },
    };

    #[test]
    fn test_knight_legal() {
        let positions = [
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1",
            "rnb1k1nr/pppp1ppp/8/8/1b5q/8/PP2P1PP/RNBQKBNR w KQkq - 0 1",
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

            g.generate_knight_moves(pinned, check_mask, &mut move_gen);

            println!("Generated {} knight moves:", move_gen.count);
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
