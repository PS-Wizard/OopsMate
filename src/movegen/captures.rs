use crate::{
    position::Position,
    types::{Move, MoveCollector, MoveType, Piece},
};
use std::arch::x86_64::_pext_u64;

use strikes::{BISHOP_ATTACKS, BISHOP_MASKS, KNIGHT_ATTACKS, ROOK_ATTACKS, ROOK_MASKS, THROUGH};

impl Position {
    #[inline(always)]
    pub(super) fn gen_piece_captures<const PIECE: usize>(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        enemies: u64,
    ) {
        let pieces = self
            .our(unsafe { std::mem::transmute::<u8, Piece>(PIECE as u8) })
            .0;

        if PIECE == Piece::Knight as usize {
            let pieces = pieces & !pinned;
            let mut bb = pieces;
            while bb != 0 {
                let from = bb.trailing_zeros() as usize;
                bb &= bb - 1;

                let mut attacks = KNIGHT_ATTACKS[from] & enemies & check_mask;
                while attacks != 0 {
                    let to = attacks.trailing_zeros() as usize;
                    attacks &= attacks - 1;
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            }
            return;
        }

        let blockers = self.occupied().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy_king = self.their(Piece::King).0;

        let mut bb = pieces;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let mut attacks = match PIECE {
                2 => {
                    let idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[from]) as usize };
                    BISHOP_ATTACKS[from][idx]
                }
                3 => {
                    let idx = unsafe { _pext_u64(blockers, ROOK_MASKS[from]) as usize };
                    ROOK_ATTACKS[from][idx]
                }
                4 => {
                    let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[from]) as usize };
                    let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[from]) as usize };
                    BISHOP_ATTACKS[from][bishop_idx] | ROOK_ATTACKS[from][rook_idx]
                }
                _ => unreachable!(),
            };

            attacks &= enemies & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;
                collector.push(Move::new(from, to, MoveType::Capture));
            }
        }
    }
}
