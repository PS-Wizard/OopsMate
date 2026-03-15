use crate::{
    position::Position,
    types::{Color, Move, MoveCollector, MoveType, Piece},
};
use std::arch::x86_64::_pext_u64;

use strikes::{PAWN_ATTACKS, ROOK_ATTACKS, ROOK_MASKS, THROUGH};

impl Position {
    #[inline(always)]
    pub(super) fn gen_pawn_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        match self.side_to_move {
            Color::White => self.gen_white_pawns(collector, pinned, check_mask),
            Color::Black => self.gen_black_pawns(collector, pinned, check_mask),
        }
    }

    #[inline(always)]
    fn gen_white_pawns(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let pawns = self.our(Piece::Pawn).0;
        let empty = !self.occupied().0;
        let enemies = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;

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
            if to < 64 && (empty >> to) & 1 != 0 {
                let target = 1u64 << to;
                if (target & pin_ray & check_mask) != 0 {
                    if to >= 56 {
                        collector.push(Move::new(from, to, MoveType::PromotionQueen));
                        collector.push(Move::new(from, to, MoveType::PromotionRook));
                        collector.push(Move::new(from, to, MoveType::PromotionBishop));
                        collector.push(Move::new(from, to, MoveType::PromotionKnight));
                    } else {
                        collector.push(Move::new(from, to, MoveType::Quiet));
                    }
                }
            }

            if (8..16).contains(&from) {
                let to2 = from + 16;
                let target2 = 1u64 << to2;
                let single_to = from + 8;
                if (empty >> single_to) & 1 != 0
                    && (empty >> to2) & 1 != 0
                    && (target2 & pin_ray & check_mask) != 0
                {
                    collector.push(Move::new(from, to2, MoveType::DoublePush));
                }
            }

            let mut attacks = PAWN_ATTACKS[0][from] & enemies & !enemy_king & pin_ray & check_mask;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                if to >= 56 {
                    collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionRook));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionBishop));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionKnight));
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
    fn gen_black_pawns(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let pawns = self.our(Piece::Pawn).0;
        let empty = !self.occupied().0;
        let enemies = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;

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
                if (empty >> to) & 1 != 0 {
                    let target = 1u64 << to;
                    if (target & pin_ray & check_mask) != 0 {
                        if to < 8 {
                            collector.push(Move::new(from, to, MoveType::PromotionQueen));
                            collector.push(Move::new(from, to, MoveType::PromotionRook));
                            collector.push(Move::new(from, to, MoveType::PromotionBishop));
                            collector.push(Move::new(from, to, MoveType::PromotionKnight));
                        } else {
                            collector.push(Move::new(from, to, MoveType::Quiet));
                        }
                    }
                }
            }

            if (48..56).contains(&from) {
                let to2 = from - 16;
                let target2 = 1u64 << to2;
                let single_to = from - 8;
                if (empty >> single_to) & 1 != 0
                    && (empty >> to2) & 1 != 0
                    && (target2 & pin_ray & check_mask) != 0
                {
                    collector.push(Move::new(from, to2, MoveType::DoublePush));
                }
            }

            let mut attacks = PAWN_ATTACKS[1][from] & enemies & !enemy_king & pin_ray & check_mask;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                if to < 8 {
                    collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionRook));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionBishop));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionKnight));
                } else {
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            }
        }

        if let Some(ep_sq) = self.en_passant {
            self.gen_en_passant(collector, pinned, check_mask, ep_sq as usize);
        }
    }

    pub(super) fn gen_en_passant(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        ep_sq: usize,
    ) {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let captured_sq = if self.side_to_move == Color::White {
            ep_sq - 8
        } else {
            ep_sq + 8
        };

        let ep_target = 1u64 << ep_sq;
        let captured_bit = 1u64 << captured_sq;

        if (ep_target & check_mask) == 0 && (captured_bit & check_mask) == 0 {
            return;
        }

        let pawns = self.our(Piece::Pawn).0;
        let color_idx = self.side_to_move as usize;

        let mut bb = pawns;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            if (PAWN_ATTACKS[color_idx][from] & ep_target) == 0 {
                continue;
            }

            if (pinned >> from) & 1 != 0 && (ep_target & THROUGH[king_sq][from]) == 0 {
                continue;
            }

            let king_rank = king_sq / 8;
            let from_rank = from / 8;

            if king_rank == from_rank && from_rank == captured_sq / 8 {
                let occupied = self.occupied().0;
                let after_ep = occupied & !(1u64 << from) & !(1u64 << captured_sq) | ep_target;

                let rook_idx = unsafe { _pext_u64(after_ep, ROOK_MASKS[king_sq]) as usize };
                let rook_attacks = ROOK_ATTACKS[king_sq][rook_idx];
                let enemy_rooks_queens = self.their(Piece::Rook).0 | self.their(Piece::Queen).0;

                if (rook_attacks & enemy_rooks_queens) != 0 {
                    continue;
                }
            }

            collector.push(Move::new(from, ep_sq, MoveType::EnPassant));
        }
    }
}
