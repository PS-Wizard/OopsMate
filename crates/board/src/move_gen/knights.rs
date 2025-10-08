use crate::Position;
use raw::KNIGHT_ATTACKS;
use types::moves::{Move, MoveCollector, MoveType::*};
use types::others::Piece::*;

impl Position {
    #[inline(always)]
    /// Takes in a pin mask, a check mask and a mutable reference to the move collector, generats
    /// all valid **legal** moves for the knights
    pub fn generate_knight_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let mut our_knights = self.our(Knight).0 & !pinned;
        let friendly = self.us().0;
        let enemy = self.them().0;

        while our_knights != 0 {
            let from = our_knights.trailing_zeros() as usize;
            our_knights &= our_knights - 1; // Pop LSB

            let mut attacks = KNIGHT_ATTACKS[from] & !friendly;

            attacks &= check_mask;

            // Split attacks into captures and quiet moves
            let captures = attacks & enemy;
            let quiets = attacks & !enemy;

            // Generate capture moves
            let mut capture_bb = captures;
            while capture_bb != 0 {
                let to = capture_bb.trailing_zeros() as usize;
                capture_bb &= capture_bb - 1;
                collector.push(Move::new(from, to, Capture));
            }

            // Generate quiet moves
            let mut quiet_bb = quiets;
            while quiet_bb != 0 {
                let to = quiet_bb.trailing_zeros() as usize;
                quiet_bb &= quiet_bb - 1;
                collector.push(Move::new(from, to, Quiet));
            }
        }
    }
}

#[cfg(test)]
mod knight_moves {
    use types::moves::MoveCollector;

    use crate::{Position, legality::attack_constraints::get_attack_constraints};

    #[test]
    fn generate_knight_moves() {
        // Initial game position should return 4 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_knight_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 4);
        mc.clear();

        // Expected 7 quiet moves, 3 captures = total 10
        let g = Position::new_from_fen(
            "rnbqk1nr/ppp2pp1/8/2p1p1p1/8/2N2N2/PP1PPPPP/R1BQKBbb w Qkq - 0 1",
        );
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_knight_moves(&mut mc, pinned, check_mask);
        assert_eq!(10, mc.len());
        mc.clear();

        // Expected 3 moves
        let g = Position::new_from_fen(
            "rnb1k1n1/ppppqppp/8/5N2/7b/3N2N1/PPPP2PP/r1N1KB1R w Kq - 0 1",
        );
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_knight_moves(&mut mc, pinned, check_mask);
        assert_eq!(3, mc.len());
        mc.clear();



        // Expected 3 moves
        let g = Position::new_from_fen(
            "rnb1k1n1/pppp1ppp/8/8/7b/3NN1NP/PPPP1NP1/r3KB1R w Kq - 0 1",
        );
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_knight_moves(&mut mc, pinned, check_mask);
        assert_eq!(3, mc.len());
        mc.clear();


        // Expected 3 moves
        let g = Position::new_from_fen(
            "1n2k1n1/pppp1ppp/b2b4/8/1NK1Nr2/2N5/PPPP1NPP/8 w - - 0 1",
        );
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_knight_moves(&mut mc, pinned, check_mask);
        assert_eq!(2, mc.len());
        mc.clear();
    }
}
