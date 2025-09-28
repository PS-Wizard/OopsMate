use pext::KNIGHT_ATTACKS;
use utilities::board::PrintAsBoard;

use crate::{game::Game, piece::Piece::*};

impl Game {
    fn generate_knight_moves(&self, pinned: u64, check_mask: u64) {
        let mut unpinned_knights = self.friendly_board(Knight) & !pinned;
        while unpinned_knights != 0 {
            let from = unpinned_knights.trailing_zeros() as usize;
            unpinned_knights &= unpinned_knights - 1;
            let attacks = KNIGHT_ATTACKS[from] & check_mask;
            attacks.print();
        }
    }
}

#[cfg(test)]
mod test_knights_legal {
    use utilities::board::PrintAsBoard;

    use crate::{game::Game, pins_checks::pin_check_finder::find_pins_n_checks};

    #[test]
    fn test_knight_legal() {
        // Queen on e7 checking king on e1, there is no white pawn on e2 nor a black pawn on e7
        // Knight on c3 is pinned by a black bishop on b4
        // Double Check, King should move
        let positions = [
            "rnbqk1nr/pppp1ppp/8/8/1b6/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 1",
            "rnb1k1nr/pppp1ppp/8/8/1b5q/8/PP2P1PP/RNBQKBNR w KQkq - 0 1",
        ];
        for position in positions {
            let g = Game::from_fen(position);
            println!("================");
            let (pinned, _checking, check_mask) = find_pins_n_checks(&g);
            println!("Pinned:");
            pinned.print();
            println!("Checking:");
            _checking.print();
            println!("CheckMask:");
            check_mask.print();
            g.generate_knight_moves(pinned, check_mask);
        }
    }
}
