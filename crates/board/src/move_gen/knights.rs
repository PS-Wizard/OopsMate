use crate::Position;
use raw::KNIGHT_ATTACKS;
use types::moves::{Move, MoveCollector, MoveType::*};
use types::others::Piece::*;

impl Position {
    #[inline(always)]
    pub fn generate_knight_moves(&self, collector: &mut MoveCollector) {
        let mut our_knights = self.our(Knight).0;
        let friendly = self.us().0;
        let enemy = self.them().0;

        while our_knights != 0 {
            let from = our_knights.trailing_zeros() as usize;
            our_knights &= our_knights - 1; // Pop LSB

            let attacks = KNIGHT_ATTACKS[from] & !friendly;

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

    use crate::Position;

    #[test]
    fn generate_knight_moves() {
        // Initial game position should return 4 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        g.generate_knight_moves(&mut mc);
        assert_eq!(mc.len(), 4);

        // Expected 7 quiet moves, 3 captures = total 10
        let g =
            Position::new_from_fen("rnbqk1nr/ppp2pp1/8/2p1p1p1/8/2N2N2/PP1PPPPP/R1BQKBbb w Qkq - 0 1");
        mc.clear();
        g.generate_knight_moves(&mut mc);
        assert_eq!(10, mc.len());
        for i in 0..mc.len() {
            let m = mc[i];
            println!("Move: {}", m);
        }
    }
}
