use crate::{Color, Piece, Position};
use strikes::{bishop_attacks, knight_attacks, line_between, line_through, pawn_attacks, rook_attacks};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Analysis {
    pub us: Color,
    pub them: Color,
    pub king_sq: usize,
    pub us_occ: u64,
    pub them_occ: u64,
    pub occ: u64,
    pub pinned: u64,
    pub checkers: u64,
    pub check_mask: u64,
}

impl Analysis {
    #[inline(always)]
    pub const fn in_check(self) -> bool {
        self.checkers != 0
    }

    #[inline(always)]
    pub const fn double_check(self) -> bool {
        self.checkers.count_ones() > 1
    }

    #[inline(always)]
    pub const fn is_pinned(self, sq: usize) -> bool {
        (self.pinned >> sq) & 1 != 0
    }

    #[inline(always)]
    pub fn pin_ray(self, sq: usize) -> u64 {
        line_through(self.king_sq, sq)
    }
}

#[inline(always)]
pub fn analyze(pos: &Position) -> Analysis {
    let us = pos.side_to_move;
    let them = us.flip();
    let king_sq = pos.our_king_sq();
    let us_occ = pos.us().0;
    let them_occ = pos.them().0;
    let occ = us_occ | them_occ;

    let enemy_bishops_queens = pos.their(Piece::Bishop).0 | pos.their(Piece::Queen).0;
    let enemy_rooks_queens = pos.their(Piece::Rook).0 | pos.their(Piece::Queen).0;

    let mut pinned = 0u64;
    let mut checkers = 0u64;

    let mut potential = (bishop_attacks(king_sq, 0) & enemy_bishops_queens)
        | (rook_attacks(king_sq, 0) & enemy_rooks_queens);

    while potential != 0 {
        let sq = potential.trailing_zeros() as usize;
        potential &= potential - 1;

        let between = line_between(king_sq, sq);
        let blockers = between & occ;

        if blockers == 0 {
            checkers |= 1u64 << sq;
        } else if blockers.count_ones() == 1 && (blockers & us_occ) != 0 {
            pinned |= blockers;
        }
    }

    checkers |= pos.their(Piece::Knight).0 & knight_attacks(king_sq);
    checkers |= pos.their(Piece::Pawn).0 & pawn_attacks(us as usize, king_sq);

    let check_mask = if checkers == 0 {
        !0u64
    } else if checkers.count_ones() == 1 {
        let checker_sq = checkers.trailing_zeros() as usize;
        line_between(king_sq, checker_sq) | checkers
    } else {
        0
    };

    Analysis {
        us,
        them,
        king_sq,
        us_occ,
        them_occ,
        occ,
        pinned,
        checkers,
        check_mask,
    }
}
