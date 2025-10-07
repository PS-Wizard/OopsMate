use std::arch::x86_64::_pext_u64;

use crate::Position;
use raw::{BISHOP_ATTACKS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MASKS, THROUGH};
use types::moves::{Move, MoveCollector, MoveType::*};
use types::others::Piece::*;

impl Position {
    #[inline(always)]
    pub fn generate_queen_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let mut our_queens = self.our(Queen).0;
        let blockers = self.all_pieces[0] | self.all_pieces[1];
        let friendly = self.us().0;
        let enemy = self.them().0;
        let king_sq = self.our(King).0.trailing_zeros() as usize;

        while our_queens != 0 {
            let from = our_queens.trailing_zeros() as usize;
            our_queens &= our_queens - 1; // Pop LSB

            let bishop_idx = unsafe { _pext_u64(blockers.0, BISHOP_MASKS[from]) as usize };
            let rook_idx = unsafe { _pext_u64(blockers.0, ROOK_MASKS[from]) as usize };
            let mut attacks =
                (BISHOP_ATTACKS[from][bishop_idx] | ROOK_ATTACKS[from][rook_idx]) & !friendly;

            // Handle pin restriction
            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }

            // Apply check mask
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
mod queen_moves {

    use types::moves::MoveCollector;

    use crate::{Position, legality::attack_constraints::get_attack_constraints};

    #[test]
    fn generate_queen_moves() {
        // Initial game position should return 0 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(mc.len(), 0);
        mc.clear();

        // Expected 9 quiet moves, 1 captures = total 10
        let g = Position::new_from_fen(
            "rnbqk1nr/ppp2pp1/8/2p1p1p1/8/2N2N2/PP2PPPP/R1BQKB2 w Qkq - 0 1",
        );
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(10, mc.len());
        mc.clear();

        let g = Position::new_from_fen("rn2kbnr/pppppppp/8/8/b7/7P/PP1PPPP1/q2QKBNR w Kkq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(3, mc.len());
        mc.clear();

        let g = Position::new_from_fen("rn2kbnr/pppppppp/8/8/b7/7P/PP1PPPP1/q2QK2q w kq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(0, mc.len());
        mc.clear();

        let g = Position::new_from_fen("rn2kbnr/p1pppppp/1Q6/8/b7/7P/PP1PP1P1/q2QK2q w kq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(1, mc.len());
        mc.clear();

        let g = Position::new_from_fen("1n1rkbnr/p1p1pppp/8/Q7/b7/8/PPQ1P1PP/q1QKQ2q w k - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(2, mc.len());
        mc.clear();

        let g = Position::new_from_fen("1n1rkbnr/p1p1pppp/8/Q7/b7/3Q4/PPQ1P1PP/q1QKQ2q w k - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_queen_moves(&mut mc, pinned, check_mask);
        assert_eq!(28, mc.len());
        mc.clear();
    }
}
