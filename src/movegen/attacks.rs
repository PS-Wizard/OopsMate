use crate::{Color, Piece, Position};

use strikes::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks};

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

    if knight_attacks(sq) & pos.pieces[Piece::Knight as usize].0 & attackers != 0 {
        return true;
    }

    if king_attacks(sq) & pos.pieces[Piece::King as usize].0 & attackers != 0 {
        return true;
    }

    if pawn_attacks(by.flip() as usize, sq) & pos.pieces[Piece::Pawn as usize].0 & attackers != 0 {
        return true;
    }

    if bishop_attacks(sq, blockers)
        & (pos.pieces[Piece::Bishop as usize].0 | pos.pieces[Piece::Queen as usize].0)
        & attackers
        != 0
    {
        return true;
    }

    if rook_attacks(sq, blockers)
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
