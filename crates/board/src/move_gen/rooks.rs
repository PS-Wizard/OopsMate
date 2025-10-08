use std::arch::x86_64::_pext_u64;

use crate::Position;
use raw::{ROOK_ATTACKS, ROOK_MASKS, THROUGH};
use types::moves::{Move, MoveCollector, MoveType::*};
use types::others::Piece::*;

impl Position {
    #[inline(always)]
    /// Takes in a mutable reference to the move collector, a pin mask and a check mask, and
    /// generates all valid moves for the rooks.
    pub fn generate_rook_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        let mut our_rooks = self.our(Rook).0;
        let blockers = self.all_pieces[0] | self.all_pieces[1];
        let friendly = self.us().0;
        let enemy = self.them().0;

        while our_rooks != 0 {
            let from = our_rooks.trailing_zeros() as usize;
            our_rooks &= our_rooks - 1; // Pop LSB

            let mask_idx = unsafe { _pext_u64(blockers.0, ROOK_MASKS[from]) as usize };
            let mut attacks = ROOK_ATTACKS[from][mask_idx] & !friendly;

            // Thy Rook Is Pinned Your Highness ( its 9 pm man gimme a break ive been at
            // this for like 9 hours now :< )
            // So when thy rook is pinned the path it can take is very trecherous,
            // limited to facing the opponent that is threatening the kings life.
            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }

            // Always apply the checkmask cause ... breh .. it just do be like that
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
mod rook_moves {
    use types::moves::MoveCollector;

    use crate::{Position, legality::attack_constraints::get_attack_constraints};

    #[test]
    fn generate_rook_moves() {
        // Initial game position should return 0 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_rook_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 0);
        mc.clear();

        // Expected 13 quiet moves, 2 captures
        let g = Position::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/8/R3KBNR w KQkq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_rook_moves(&mut mc, pinned, check_mask);
        assert_eq!(15, mc.len());
        mc.clear();

        // Expected 4 quiet moves, 2 captures = 6 total
        let g = Position::new_from_fen("rn2kbnr/pppppppp/8/8/8/7P/P1PPPPP1/Kb1R3q w kq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_rook_moves(&mut mc, pinned, check_mask);
        assert_eq!(6, mc.len());
        mc.clear();

        // 3 rooks all pinned by the different enemy sliders, expected:
        // expected 11 moves, 9 quiet, 2 captures
        let g = Position::new_from_fen("rn2k1nr/1ppppppp/5b2/8/8/7P/RRPPPPP1/KR4q1 w kq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_rook_moves(&mut mc, pinned, check_mask);
        assert_eq!(11, mc.len());
        mc.clear();

        // 3 rooks all pinned by the different enemy sliders, expected:
        // expected 10 moves, 8 quiet, 2 captures, 1 pin
        let g = Position::new_from_fen("rn2k1nr/1ppppppp/5b2/8/8/7P/RRPPPPP1/KRR3q1 w kq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_rook_moves(&mut mc, pinned, check_mask);
        assert_eq!(10, mc.len());
        mc.clear();

        // 3 rooks all pinned by the different enemy sliders, expected:
        // expected 12 moves, 8 quiet, 2 captures, 0 pins
        let g = Position::new_from_fen("rn2k1nr/1ppppppp/5b2/8/8/8/RRPPPPPP/KR2R1q1 w kq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_rook_moves(&mut mc, pinned, check_mask);
        assert_eq!(12, mc.len());
        mc.clear();
    }
}
