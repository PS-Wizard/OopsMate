use crate::Position;
use raw::{BISHOP_ATTACKS, BISHOP_MASKS, LINE};
use std::arch::x86_64::_pext_u64;
use types::moves::{Move, MoveCollector, MoveType::*};
use types::others::Piece::*;

impl Position {
    #[inline(always)]
    pub fn generate_bishop_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let mut our_bishops = self.our(Bishop).0;
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        let blockers = self.all_pieces[0] | self.all_pieces[1];
        let friendly = self.us().0;
        let enemy = self.them().0;

        while our_bishops != 0 {
            let from = our_bishops.trailing_zeros() as usize;
            our_bishops &= our_bishops - 1; // Pop LSB

            let mask_idx = unsafe { _pext_u64(blockers.0, BISHOP_MASKS[from]) as usize };
            let mut attacks = BISHOP_ATTACKS[from][mask_idx] & !friendly;

            if (pinned >> from) & 1 != 0 {
                // The bishop is pinned, limit its movement to between the king and itself because
                // a bishop pinned can only move between the king and itself diagonally
                attacks &= LINE[king_sq][from];
            }

            // Apply check mask (must block or capture checker)
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
mod bishop_moves {
    use crate::{Position, legality::attack_constraints::get_attack_constraints};
    use types::moves::MoveCollector;
    use utilities::board::PrintAsBoard;

    #[test]
    fn generate_bishop_moves() {
        println!("=============");
        // Initial game position should return 0 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_bishop_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 0);
        mc.clear();
        println!("=============");

        println!("=============");
        // Expected 14 quiet moves, 3 captures = total 17
        let g =
            Position::new_from_fen("rnbqkbnr/pppppppp/8/3B4/8/4B3/PPP2PPP/RN1QK1NR w KQkq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_bishop_moves(&mut mc, pinned, check_mask);
        assert_eq!(17, mc.len());
        mc.clear();
        println!("=============");

        println!("=============");
        // both bishop are pinned expected 0 moves
        let g = Position::new_from_fen("rnb1kbn1/ppppqppp/8/8/8/4B2P/PP4P1/RN2KB1r w Qq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        pinned.print();
        check_mask.print();
        g.generate_bishop_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 0);
        mc.clear();
        println!("=============");
        
        // Position too compilcated to describe just throw it in lichess board editor, but 
        // expectd: 5 moves
        println!("=============");
        let g = Position::new_from_fen("rnb1k1n1/pppp1ppp/8/8/3q3b/7P/PP1BrBP1/RN1KR3 w KQq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        pinned.print();
        check_mask.print();
        g.generate_bishop_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 5);
        mc.clear();
        println!("=============");

        println!("=============");
        // Position too compilcated to describe just throw it in lichess board editor, but 
        // expectd: 13 moves
        let g = Position::new_from_fen("rnb1k1n1/pppp1ppp/8/q7/7b/2B4P/PP1B1BP1/R3K3 w Qq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        pinned.print();
        check_mask.print();
        g.generate_bishop_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 13);
        mc.clear();
        println!("=============");

    }
}
