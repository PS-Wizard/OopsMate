use super::constraints::get_constraints;
use crate::{
    position::Position,
    types::{MoveCollector, Piece},
};

impl Position {
    #[inline(always)]
    pub fn generate_moves(&self, collector: &mut MoveCollector) {
        let (pinned, check_mask) = get_constraints(self);

        self.gen_pawn_moves(collector, pinned, check_mask);
        self.gen_knight_moves(collector, pinned, check_mask);
        self.gen_bishop_moves(collector, pinned, check_mask);
        self.gen_rook_moves(collector, pinned, check_mask);
        self.gen_queen_moves(collector, pinned, check_mask);
        self.gen_king_moves(collector);
    }

    #[inline(always)]
    pub fn generate_captures(&self, collector: &mut MoveCollector) {
        let (pinned, check_mask) = get_constraints(self);
        let enemies = self.them().0;

        self.gen_pawn_captures(collector, pinned, check_mask, enemies);
        self.gen_piece_captures::<{ Piece::Knight as usize }>(
            collector, pinned, check_mask, enemies,
        );
        self.gen_piece_captures::<{ Piece::Bishop as usize }>(
            collector, pinned, check_mask, enemies,
        );
        self.gen_piece_captures::<{ Piece::Rook as usize }>(collector, pinned, check_mask, enemies);
        self.gen_piece_captures::<{ Piece::Queen as usize }>(
            collector, pinned, check_mask, enemies,
        );
        self.gen_king_captures(collector, enemies);
    }
}
