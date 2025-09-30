use raw::PAWN_ATTACKS;
use types::moves::MoveCollector;
use types::moves::{Move, MoveType::*};
use types::others::Color::*;
use types::others::Piece::*;

use crate::Position;

impl Position {
    #[inline(always)]
    pub fn generate_pawn_moves(&self, collector: &mut MoveCollector) {
        if self.side_to_move == White {
            self.generate_white_pawn_moves(collector);
        } else {
            self.generate_black_pawn_moves(collector);
        }
    }

    fn generate_white_pawn_moves(&self, collector: &mut MoveCollector) {
        let pawns = self.our(Pawn).0;
        let empty = !(self.all_pieces[0].0 | self.all_pieces[1].0);
        let enemies = self.them().0;

        // Single pushes
        let push_targets = (pawns << 8) & empty;
        self.add_pawn_pushes(push_targets, -8, collector);

        // Double pushes (from rank 2 to rank 4)
        let rank_3 = push_targets & 0x0000000000FF0000;
        let double_targets = (rank_3 << 8) & empty;
        self.add_pawn_double_pushes(double_targets, -16, collector);

        // Captures using attack table
        let mut pawn_bb = pawns;
        while pawn_bb != 0 {
            let from = pawn_bb.trailing_zeros() as usize;
            pawn_bb &= pawn_bb - 1;

            let attacks = PAWN_ATTACKS[0][from] & enemies;
            self.add_pawn_captures_from_square(attacks, from, collector);
        }

        // En passant
        if let Some(ep_sq) = self.en_passant {
            let ep_target = 1u64 << ep_sq;
            let mut pawn_bb = pawns;
            while pawn_bb != 0 {
                let from = pawn_bb.trailing_zeros() as usize;
                pawn_bb &= pawn_bb - 1;

                if PAWN_ATTACKS[0][from] & ep_target != 0 {
                    collector.push(Move::new(from, ep_sq as usize, EnPassant));
                }
            }
        }
    }

    fn generate_black_pawn_moves(&self, collector: &mut MoveCollector) {
        let pawns = self.our(Pawn).0;
        let empty = !(self.all_pieces[0].0 | self.all_pieces[1].0);
        let enemies = self.them().0;

        // Single pushes
        let push_targets = (pawns >> 8) & empty;
        self.add_pawn_pushes(push_targets, 8, collector);

        // Double pushes (from rank 7 to rank 5)
        let rank_6 = push_targets & 0x0000FF0000000000;
        let double_targets = (rank_6 >> 8) & empty;
        self.add_pawn_double_pushes(double_targets, 16, collector);

        // Captures using attack table
        let mut pawn_bb = pawns;
        while pawn_bb != 0 {
            let from = pawn_bb.trailing_zeros() as usize;
            pawn_bb &= pawn_bb - 1;

            let attacks = PAWN_ATTACKS[1][from] & enemies;
            self.add_pawn_captures_from_square(attacks, from, collector);
        }

        // En passant
        if let Some(ep_sq) = self.en_passant {
            let ep_target = 1u64 << ep_sq;
            let mut pawn_bb = pawns;
            while pawn_bb != 0 {
                let from = pawn_bb.trailing_zeros() as usize;
                pawn_bb &= pawn_bb - 1;

                if PAWN_ATTACKS[1][from] & ep_target != 0 {
                    collector.push(Move::new(from, ep_sq as usize, EnPassant));
                }
            }
        }
    }

    // Helper: add pushes (with promotions on rank 8/1)
    fn add_pawn_pushes(&self, targets: u64, offset: i32, collector: &mut MoveCollector) {
        let promo_rank = if self.side_to_move == White {
            0xFF00000000000000
        } else {
            0x00000000000000FF
        };
        let promotions = targets & promo_rank;
        let non_promotions = targets & !promo_rank;

        // Regular pushes
        let mut bb = non_promotions;
        while bb != 0 {
            let to = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            let from = (to as i32 + offset) as usize;
            collector.push(Move::new(from, to, Quiet));
        }

        // Promotion pushes
        let mut bb = promotions;
        while bb != 0 {
            let to = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            let from = (to as i32 + offset) as usize;
            collector.push(Move::new(from, to, PromotionQueen));
            collector.push(Move::new(from, to, PromotionRook));
            collector.push(Move::new(from, to, PromotionBishop));
            collector.push(Move::new(from, to, PromotionKnight));
        }
    }

    // Helper: add double pushes
    fn add_pawn_double_pushes(&self, mut targets: u64, offset: i32, collector: &mut MoveCollector) {
        while targets != 0 {
            let to = targets.trailing_zeros() as usize;
            targets &= targets - 1;
            let from = (to as i32 + offset) as usize;
            collector.push(Move::new(from, to, DoublePush));
        }
    }

    // Helper: add captures from a single square (with promotions)
    fn add_pawn_captures_from_square(
        &self,
        targets: u64,
        from: usize,
        collector: &mut MoveCollector,
    ) {
        let promo_rank = if self.side_to_move == White {
            0xFF00000000000000
        } else {
            0x00000000000000FF
        };
        let promotions = targets & promo_rank;
        let non_promotions = targets & !promo_rank;

        // Regular captures
        let mut bb = non_promotions;
        while bb != 0 {
            let to = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            collector.push(Move::new(from, to, Capture));
        }

        // Capture promotions
        let mut bb = promotions;
        while bb != 0 {
            let to = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            collector.push(Move::new(from, to, CapturePromotionQueen));
            collector.push(Move::new(from, to, CapturePromotionRook));
            collector.push(Move::new(from, to, CapturePromotionBishop));
            collector.push(Move::new(from, to, CapturePromotionKnight));
        }
    }
}

#[cfg(test)]
mod pawns {
    use types::moves::MoveCollector;

    use crate::Position;

    #[test]
    fn test() {
        let g = Position::new();
        let mut mc = MoveCollector::new();
        g.generate_pawn_moves(&mut mc);
        // initial position should be 16 moves
        assert_eq!(16, mc.len());
        mc.clear();

        // enpassant on b6, expected 11 moves
        let g =
            Position::new_from_fen("rn2k1nr/p1ppp1pp/8/1pP5/8/7P/PP2P1P1/RNBQK2R w KQkq b6 0 1");
        g.generate_pawn_moves(&mut mc);
        assert_eq!(11, mc.len());
        mc.clear();

        // enpassant on b6, available promotion, so capture on h8 -> 4 moves, promo to a queen,
        // bishop, knight, rook: expected 14 moves
        let g =
            Position::new_from_fen("rn2k1nr/p1ppp1Pp/8/1pP5/8/8/PP2P1P1/RNBQKp1R w KQkq b6 0 1");
        g.generate_pawn_moves(&mut mc);
        assert_eq!(14, mc.len());
        mc.clear();

        let g = Position::new_from_fen(
            "rnbqkbnr/pp4pp/2p5/2PpppP1/8/8/PP1P1P1P/RNBQKBNR w KQkq f6 0 2",
        );
        g.generate_pawn_moves(&mut mc);
        assert_eq!(12, mc.len());
    }
}
