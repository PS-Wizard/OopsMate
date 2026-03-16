use super::NNUEProbe;
use crate::accumulator_stack::DirtyPiece;
use crate::nnue::MoveDelta;
use crate::types::{Piece, Square};

impl NNUEProbe {
    #[inline(always)]
    pub fn rule50(&self) -> i32 {
        self.accumulator_stack.latest().rule50
    }

    #[inline(always)]
    fn apply_delta_internal(&mut self, delta: MoveDelta) {
        for change in delta.changes() {
            if change.piece_from != Piece::None {
                debug_assert_eq!(self.pieces[change.from], change.piece_from);
                self.remove_piece_internal(change.from);
            }
        }

        for change in delta.changes() {
            if change.piece_to != Piece::None {
                debug_assert_eq!(self.pieces[change.to], Piece::None);
                self.add_piece_internal(change.piece_to, change.to);
            }
        }

        let dirty = delta.to_dirty_piece();
        self.accumulator_stack.push(&dirty, delta.next_rule50());

        let color_bb = self.by_color_bb;
        let type_bb = self.by_type_bb;
        self.accumulator_stack.update_incremental(
            self.king_squares,
            &self.networks.big_net.feature_transformer,
            &self.networks.small_net.feature_transformer,
            &mut self.finny_tables,
            || (color_bb, type_bb),
        );
    }

    #[inline(always)]
    fn undo_delta_internal(&mut self, delta: MoveDelta) {
        for change in delta.changes().iter().rev() {
            if change.piece_to != Piece::None {
                debug_assert_eq!(self.pieces[change.to], change.piece_to);
                self.remove_piece_internal(change.to);
            }
        }

        for change in delta.changes().iter().rev() {
            if change.piece_from != Piece::None {
                debug_assert_eq!(self.pieces[change.from], Piece::None);
                self.add_piece_internal(change.piece_from, change.from);
            }
        }

        self.accumulator_stack.pop();
    }

    #[inline(always)]
    pub fn apply_delta(&mut self, delta: MoveDelta) {
        self.apply_delta_internal(delta);
    }

    #[inline(always)]
    pub fn undo_delta(&mut self, delta: MoveDelta) {
        self.undo_delta_internal(delta);
    }

    #[inline(always)]
    pub fn make_null_move(&mut self) {
        self.apply_delta_internal(MoveDelta::null(self.rule50() + 1));
    }

    #[inline(always)]
    pub fn make_null_move_with_rule50(&mut self, next_rule50: i32) {
        self.apply_delta_internal(MoveDelta::null(next_rule50));
    }

    #[inline(always)]
    pub fn unmake_null_move(&mut self) {
        self.accumulator_stack.pop();
    }

    /// Applies a simple move and computes the next accumulator state.
    pub fn make_move(&mut self, from_sq: Square, to_sq: Square, piece: Piece) {
        let mut dirty = DirtyPiece::new();
        let from_piece = self.pieces[from_sq];
        let to_piece = self.pieces[to_sq];

        self.remove_piece_internal(from_sq);
        self.add_piece_internal(piece, to_sq);

        dirty.add_change(from_sq, to_sq, from_piece.index(), piece.index());
        if to_piece != Piece::None {
            dirty.add_change(to_sq, to_sq, to_piece.index(), Piece::None.index());
        }

        let new_rule50 = if from_piece.piece_type() == 1 || to_piece != Piece::None {
            0
        } else {
            self.rule50() + 1
        };

        self.accumulator_stack.push(&dirty, new_rule50);

        let color_bb = self.by_color_bb;
        let type_bb = self.by_type_bb;
        self.accumulator_stack.update_incremental(
            self.king_squares,
            &self.networks.big_net.feature_transformer,
            &self.networks.small_net.feature_transformer,
            &mut self.finny_tables,
            || (color_bb, type_bb),
        );
    }

    /// Reverts a move previously applied with `make_move`.
    pub fn unmake_move(
        &mut self,
        from_sq: Square,
        to_sq: Square,
        from_piece: Piece,
        captured_piece: Option<Piece>,
    ) {
        self.remove_piece_internal(to_sq);

        if let Some(captured) = captured_piece {
            self.add_piece_internal(captured, to_sq);
        }

        self.add_piece_internal(from_piece, from_sq);
        self.accumulator_stack.pop();
    }
}
