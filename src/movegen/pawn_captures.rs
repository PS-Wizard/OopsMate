use crate::{
    position::Position,
    types::{Color, Move, MoveCollector, MoveType, Piece},
};
use strikes::{PAWN_ATTACKS, THROUGH};

impl Position {
    #[inline(always)]
    pub(super) fn gen_pawn_captures(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        enemies: u64,
    ) {
        match self.side_to_move {
            Color::White => self.gen_white_pawn_captures(collector, pinned, check_mask, enemies),
            Color::Black => self.gen_black_pawn_captures(collector, pinned, check_mask, enemies),
        }
    }

    #[inline(always)]
    fn gen_white_pawn_captures(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        enemies: u64,
    ) {
        let pawns = self.our(Piece::Pawn).0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;
        let empty = !self.occupied().0;

        let mut bb = pawns;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let pin_ray = if (pinned >> from) & 1 != 0 {
                THROUGH[king_sq][from]
            } else {
                !0u64
            };

            let to = from + 8;
            if (56..64).contains(&to) && (empty >> to) & 1 != 0 {
                let target = 1u64 << to;
                if (target & pin_ray & check_mask) != 0 {
                    collector.push(Move::new(from, to, MoveType::PromotionQueen));
                }
            }

            let mut attacks = PAWN_ATTACKS[0][from] & enemies & !enemy_king & pin_ray & check_mask;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                if to >= 56 {
                    collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
                } else {
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            }
        }

        if let Some(ep_sq) = self.en_passant {
            self.gen_en_passant(collector, pinned, check_mask, ep_sq as usize);
        }
    }

    #[inline(always)]
    fn gen_black_pawn_captures(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        enemies: u64,
    ) {
        let pawns = self.our(Piece::Pawn).0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;
        let empty = !self.occupied().0;

        let mut bb = pawns;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let pin_ray = if (pinned >> from) & 1 != 0 {
                THROUGH[king_sq][from]
            } else {
                !0u64
            };

            if from >= 8 {
                let to = from - 8;
                if to < 8 && (empty >> to) & 1 != 0 {
                    let target = 1u64 << to;
                    if (target & pin_ray & check_mask) != 0 {
                        collector.push(Move::new(from, to, MoveType::PromotionQueen));
                    }
                }
            }

            let mut attacks = PAWN_ATTACKS[1][from] & enemies & !enemy_king & pin_ray & check_mask;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                if to < 8 {
                    collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
                } else {
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            }
        }

        if let Some(ep_sq) = self.en_passant {
            self.gen_en_passant(collector, pinned, check_mask, ep_sq as usize);
        }
    }
}
