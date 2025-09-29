use std::arch::x86_64::_pext_u64;

use crate::Position;
use raw::{BISHOP_ATTACKS, BISHOP_MASKS};
use types::moves::{Move, MoveCollector, MoveType::*};
use types::others::Piece::*;

impl Position {
    #[inline(always)]
    pub fn generate_bishop_moves(&self, collector: &mut MoveCollector) {
        let mut our_bishops = self.our(Bishop).0;
        let blockers = self.all_pieces[0] | self.all_pieces[1];
        let friendly = self.us().0;
        let enemy = self.them().0;

        while our_bishops != 0 {
            let from = our_bishops.trailing_zeros() as usize;
            our_bishops &= our_bishops - 1; // Pop LSB

            let mask_idx = unsafe { _pext_u64(blockers.0, BISHOP_MASKS[from]) as usize };
            let attacks = BISHOP_ATTACKS[from][mask_idx] & !friendly;

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
mod bishop_moves {
    use types::moves::MoveCollector;

    use crate::Position;

    #[test]
    fn generate_bishop_moves() {
        // Initial game position should return 0 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        g.generate_rook_moves(&mut mc);
        assert_eq!(mc.len(), 0);

        // Expected 14 quiet moves, 3 captures = total 17
        let g =
            Position::new_from_fen("rnbqkbnr/pppppppp/8/3B4/8/4B3/PPP2PPP/RN1QK1NR w KQkq - 0 1");
        mc.clear();
        g.generate_bishop_moves(&mut mc);
        assert_eq!(17, mc.len());
        for i in 0..mc.len() {
            let m = mc[i];
            println!("Move: {}", m);
        }
    }
}
