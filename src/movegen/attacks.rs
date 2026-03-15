use crate::{
    position::Position,
    types::{Color, Piece},
};
use std::arch::x86_64::_pext_u64;

use strikes::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS,
};

impl Position {
    #[inline(always)]
    pub fn is_square_attacked(&self, sq: usize, by: Color) -> bool {
        let blockers = self.occupied().0;
        self.is_square_attacked_with_blockers(sq, by, blockers)
    }

    #[inline(always)]
    pub fn is_square_attacked_with_blockers(&self, sq: usize, by: Color, blockers: u64) -> bool {
        let attackers = self.colors[by as usize].0;

        if KNIGHT_ATTACKS[sq] & self.pieces[Piece::Knight as usize].0 & attackers != 0 {
            return true;
        }

        if KING_ATTACKS[sq] & self.pieces[Piece::King as usize].0 & attackers != 0 {
            return true;
        }

        if PAWN_ATTACKS[by.flip() as usize][sq] & self.pieces[Piece::Pawn as usize].0 & attackers
            != 0
        {
            return true;
        }

        let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[sq]) as usize };
        if BISHOP_ATTACKS[sq][bishop_idx]
            & (self.pieces[Piece::Bishop as usize].0 | self.pieces[Piece::Queen as usize].0)
            & attackers
            != 0
        {
            return true;
        }

        let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[sq]) as usize };
        if ROOK_ATTACKS[sq][rook_idx]
            & (self.pieces[Piece::Rook as usize].0 | self.pieces[Piece::Queen as usize].0)
            & attackers
            != 0
        {
            return true;
        }

        false
    }

    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        self.is_square_attacked(king_sq, self.side_to_move.flip())
    }
}
