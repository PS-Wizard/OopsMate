use types::moves::MoveCollector;
use types::others::Piece::*;

use crate::Position;

impl Position {
    /// Perft (performance test) - counts leaf nodes at a given depth
    pub fn perft(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        if depth == 1 {
            return collector.len() as u64;
        }

        let mut nodes = 0u64;
        for i in 0..collector.len() {
            let m = collector[i];
            let new_pos = self.make_move(m);

            // Fixed: Temporarily flip side on a clone to check if own king is left in check
            let mut check_pos = new_pos.clone();
            check_pos.side_to_move = check_pos.side_to_move.flip();
            if check_pos.is_in_check() {
                continue;
            }

            nodes += new_pos.perft(depth - 1);
        }

        nodes
    }

    /// Perft divide - shows move breakdown at root
    pub fn perft_divide(&self, depth: u8) {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        let mut total = 0u64;

        for i in 0..collector.len() {
            let m = collector[i];
            let new_pos = self.make_move(m);

            // Fixed: Temporarily flip side on a clone to check if own king is left in check
            let mut check_pos = new_pos.clone();
            check_pos.side_to_move = check_pos.side_to_move.flip();
            if check_pos.is_in_check() {
                continue;
            }

            let count = if depth <= 1 {
                1
            } else {
                new_pos.perft(depth - 1)
            };

            println!("{}: {}", m, count);
            total += count;
        }

        println!("\nTotal: {}", total);
    }

    /// Generate all pseudo-legal moves
    fn generate_moves(&self, collector: &mut MoveCollector) {
        let (pinned, _checking, check_mask) =
            crate::legality::attack_constraints::get_attack_constraints(self);

        self.generate_pawn_moves(collector, pinned, check_mask);
        self.generate_knight_moves(collector, pinned, check_mask);
        self.generate_bishop_moves(collector, pinned, check_mask);
        self.generate_rook_moves(collector, pinned, check_mask);
        self.generate_queen_moves(collector, pinned, check_mask);
        self.generate_king_moves(collector);
    }

    pub fn is_other_side_in_check(&self) -> bool {
        let king_sq = self.their(King).0.trailing_zeros() as usize;
        self.is_square_attacked(king_sq)
    }
}

#[cfg(test)]
mod perft_tests {
    use crate::Position;

    #[test]
    fn perft_starting_position() {
        let pos = Position::new();
        pos.perft_divide(6);

        // assert_eq!(pos.perft(1), 20);
        // assert_eq!(pos.perft(2), 400);
        // assert_eq!(pos.perft(3), 8_902);
        // assert_eq!(pos.perft(4), 197_281);
        // assert_eq!(pos.perft(5), 4_865_609); // Takes a bit longer
    }
}
