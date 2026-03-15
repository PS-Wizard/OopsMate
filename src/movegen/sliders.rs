use crate::{
    position::Position,
    types::{Move, MoveCollector, MoveType, Piece},
};
use std::arch::x86_64::_pext_u64;

use strikes::{BISHOP_ATTACKS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MASKS, THROUGH};

impl Position {
    #[inline(always)]
    pub(super) fn gen_bishop_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let bishops = self.our(Piece::Bishop).0;
        let blockers = self.occupied().0;
        let us = self.us().0;
        let them = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;

        let mut bb = bishops;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[from]) as usize };
            let mut attacks = BISHOP_ATTACKS[from][idx] & !us & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }

    #[inline(always)]
    pub(super) fn gen_rook_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let rooks = self.our(Piece::Rook).0;
        let blockers = self.occupied().0;
        let us = self.us().0;
        let them = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;

        let mut bb = rooks;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let idx = unsafe { _pext_u64(blockers, ROOK_MASKS[from]) as usize };
            let mut attacks = ROOK_ATTACKS[from][idx] & !us & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }

    #[inline(always)]
    pub(super) fn gen_queen_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let queens = self.our(Piece::Queen).0;
        let blockers = self.occupied().0;
        let us = self.us().0;
        let them = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;

        let mut bb = queens;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[from]) as usize };
            let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[from]) as usize };
            let mut attacks = (BISHOP_ATTACKS[from][bishop_idx] | ROOK_ATTACKS[from][rook_idx])
                & !us
                & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }
}
