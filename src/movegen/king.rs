use crate::{
    position::Position,
    types::{Color, Move, MoveCollector, MoveType, Piece},
};
use strikes::KING_ATTACKS;

impl Position {
    pub(super) fn gen_king_moves(&self, collector: &mut MoveCollector) {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let us = self.us().0;
        let them = self.them().0;
        let enemy = self.side_to_move.flip();

        let blockers_without_king = self.occupied().0 & !(1u64 << king_sq);
        let enemy_king = self.their(Piece::King).0;

        let mut attacks = KING_ATTACKS[king_sq] & !us & !enemy_king;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;

            if !self.is_square_attacked_with_blockers(to, enemy, blockers_without_king) {
                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(king_sq, to, move_type));
            }
        }

        if !self.is_in_check() {
            self.gen_castling(collector, king_sq, enemy);
        }
    }

    pub(super) fn gen_king_captures(&self, collector: &mut MoveCollector, enemies: u64) {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let enemy = self.side_to_move.flip();
        let blockers_without_king = self.occupied().0 & !(1u64 << king_sq);
        let enemy_king = self.their(Piece::King).0;

        let mut attacks = KING_ATTACKS[king_sq] & enemies & !enemy_king;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;

            if !self.is_square_attacked_with_blockers(to, enemy, blockers_without_king) {
                collector.push(Move::new(king_sq, to, MoveType::Capture));
            }
        }
    }

    fn gen_castling(&self, collector: &mut MoveCollector, king_sq: usize, enemy: Color) {
        let occupied = self.occupied().0;

        match self.side_to_move {
            Color::White => {
                if self.castling_rights.can_castle_kingside(Color::White)
                    && (occupied & 0x60) == 0
                    && !self.is_square_attacked(5, enemy)
                    && !self.is_square_attacked(6, enemy)
                {
                    collector.push(Move::new(king_sq, 6, MoveType::Castle));
                }
                if self.castling_rights.can_castle_queenside(Color::White)
                    && (occupied & 0x0E) == 0
                    && !self.is_square_attacked(3, enemy)
                    && !self.is_square_attacked(2, enemy)
                {
                    collector.push(Move::new(king_sq, 2, MoveType::Castle));
                }
            }
            Color::Black => {
                if self.castling_rights.can_castle_kingside(Color::Black)
                    && (occupied & 0x6000000000000000) == 0
                    && !self.is_square_attacked(61, enemy)
                    && !self.is_square_attacked(62, enemy)
                {
                    collector.push(Move::new(king_sq, 62, MoveType::Castle));
                }
                if self.castling_rights.can_castle_queenside(Color::Black)
                    && (occupied & 0x0E00000000000000) == 0
                    && !self.is_square_attacked(59, enemy)
                    && !self.is_square_attacked(58, enemy)
                {
                    collector.push(Move::new(king_sq, 58, MoveType::Castle));
                }
            }
        }
    }
}
