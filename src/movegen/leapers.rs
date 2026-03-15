use crate::{
    position::Position,
    types::{Move, MoveCollector, MoveType, Piece},
};
use strikes::KNIGHT_ATTACKS;

impl Position {
    #[inline(always)]
    pub(super) fn gen_knight_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let knights = self.our(Piece::Knight).0 & !pinned;
        let us = self.us().0;
        let them = self.them().0;
        let enemy_king = self.their(Piece::King).0;

        let mut bb = knights;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let mut attacks = KNIGHT_ATTACKS[from] & !us & !enemy_king & check_mask;
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
