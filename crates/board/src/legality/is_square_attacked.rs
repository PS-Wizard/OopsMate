use std::arch::x86_64::_pext_u64;

use raw::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS,
};
use types::others::{Color, Piece::*};

use crate::Position;

impl Position {
    #[inline(always)]
    /// The underlying function behind is_square_attacked, is_in_check. Uses the raw attacks from
    /// PEXT attack tables to determine if a square is attacked
    pub fn is_square_attacked_by(&self, sq: usize, attacker: Color) -> bool {
        let blockers = self.all_pieces[0] | self.all_pieces[1];

        // Check knights
        if KNIGHT_ATTACKS[sq]
            & self.pieces[Knight as usize].0
            & self.all_pieces[attacker as usize].0
            != 0
        {
            return true;
        }

        // Check king
        if KING_ATTACKS[sq] & self.pieces[King as usize].0 & self.all_pieces[attacker as usize].0
            != 0
        {
            return true;
        }

        // Check bishops/queens
        let bishop_mask_idx = unsafe { _pext_u64(blockers.0, BISHOP_MASKS[sq]) as usize };
        let bishop_attacks = BISHOP_ATTACKS[sq][bishop_mask_idx];
        if bishop_attacks
            & (self.pieces[Bishop as usize].0 | self.pieces[Queen as usize].0)
            & self.all_pieces[attacker as usize].0
            != 0
        {
            return true;
        }

        // Check rooks/queens
        let rook_mask_idx = unsafe { _pext_u64(blockers.0, ROOK_MASKS[sq]) as usize };
        let rook_attacks = ROOK_ATTACKS[sq][rook_mask_idx];
        if rook_attacks
            & (self.pieces[Rook as usize].0 | self.pieces[Queen as usize].0)
            & self.all_pieces[attacker as usize].0
            != 0
        {
            return true;
        }

        // Check pawns
        if PAWN_ATTACKS[attacker.flip() as usize][sq]
            & self.pieces[Pawn as usize].0
            & self.all_pieces[attacker as usize].0
            != 0
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod is_square_attacked {
    use types::others::Color::*;
    use types::others::Piece::*;
    use utilities::algebraic::Algebraic;

    use crate::Position;

    #[test]
    fn test() {
        let mut g = Position::new();

        // a file should be un attacked
        assert_eq!(false, g.is_square_attacked("a1".idx()));
        assert_eq!(false, g.is_square_attacked("a2".idx()));

        // removing the pawn infront of the rook now the a2 square should be attacked
        g.remove_piece("a7".idx());
        assert_eq!(true, g.is_square_attacked("a2".idx()));

        // This should be unattacked
        assert_eq!(false, g.is_square_attacked("e2".idx()));
        // removing the pawn exposing the black king
        g.remove_piece("e7".idx());
        // removing placing a bishop inplace of the pawn the e2 square should still be unattacked
        g.add_piece("e7".idx(), Black, Bishop);
        assert_eq!(false, g.is_square_attacked("e2".idx()));

        // Replacing the bishop with a rook now the e2 should be attacked.
        g.remove_piece("e7".idx());
        g.add_piece("e7".idx(), Black, Rook);
        assert_eq!(true, g.is_square_attacked("e2".idx()));

        // replacing the rook with a queen the e2 pawn should still be attacked
        g.remove_piece("e7".idx());
        g.add_piece("e7".idx(), Black, Queen);
        assert_eq!(true, g.is_square_attacked("e2".idx()));

        // removing the e2 pawn now the king should be attacked
        g.remove_piece("e2".idx());
        assert_eq!(true, g.is_square_attacked("e1".idx()));
    }
}
