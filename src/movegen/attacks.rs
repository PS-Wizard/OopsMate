use crate::{Color, Piece, Position};
use std::arch::x86_64::_pext_u64;

use strikes::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS,
};

#[inline(always)]
pub fn is_square_attacked(pos: &Position, sq: usize, by: Color) -> bool {
    is_square_attacked_with_blockers(pos, sq, by, pos.occupied().0)
}

#[inline(always)]
pub fn is_square_attacked_with_blockers(
    pos: &Position,
    sq: usize,
    by: Color,
    blockers: u64,
) -> bool {
    let attackers = pos.colors[by as usize].0;

    if KNIGHT_ATTACKS[sq] & pos.pieces[Piece::Knight as usize].0 & attackers != 0 {
        return true;
    }

    if KING_ATTACKS[sq] & pos.pieces[Piece::King as usize].0 & attackers != 0 {
        return true;
    }

    if PAWN_ATTACKS[by.flip() as usize][sq] & pos.pieces[Piece::Pawn as usize].0 & attackers != 0 {
        return true;
    }

    let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[sq]) as usize };
    if BISHOP_ATTACKS[sq][bishop_idx]
        & (pos.pieces[Piece::Bishop as usize].0 | pos.pieces[Piece::Queen as usize].0)
        & attackers
        != 0
    {
        return true;
    }

    let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[sq]) as usize };
    if ROOK_ATTACKS[sq][rook_idx]
        & (pos.pieces[Piece::Rook as usize].0 | pos.pieces[Piece::Queen as usize].0)
        & attackers
        != 0
    {
        return true;
    }

    false
}

impl Position {
    #[inline(always)]
    pub fn is_square_attacked(&self, sq: usize, by: Color) -> bool {
        is_square_attacked(self, sq, by)
    }

    #[inline(always)]
    pub fn is_square_attacked_with_blockers(&self, sq: usize, by: Color, blockers: u64) -> bool {
        is_square_attacked_with_blockers(self, sq, by, blockers)
    }

    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        is_square_attacked(self, self.our_king_sq(), self.side_to_move.flip())
    }
}
