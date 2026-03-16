use super::NNUEProbe;
use crate::architecture::{BISHOP_VALUE, KNIGHT_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::piece_list::{collect_pieces_from, PieceList, PIECE_LIST_CAPACITY};
use crate::types::{Piece, Square};

impl NNUEProbe {
    /// Replaces the current board state and refreshes both accumulators from scratch.
    pub fn set_position(&mut self, pieces: &[(Piece, Square)], rule50: i32) {
        self.pieces = [Piece::None; 64];
        self.piece_count = 0;
        self.pawn_count = [0; 2];
        self.non_pawn_material = [0; 2];
        self.king_squares = [0; 2];
        self.by_color_bb = [0; 2];
        self.by_type_bb = [0; 6];

        for &(piece, square) in pieces {
            self.add_piece_internal(piece, square);
        }

        self.accumulator_stack.reset_with_refresh(
            self.king_squares,
            &self.networks.big_net.feature_transformer,
            &self.networks.small_net.feature_transformer,
            &mut self.finny_tables,
            self.by_color_bb,
            self.by_type_bb,
            rule50,
        );

        self.debug_assert_consistent();
    }

    /// Pre-fills the king-square cache for the current position.
    pub fn prepopulate_cache(&mut self) {
        let mut pieces_idx = PieceList::new();
        collect_pieces_from(&self.pieces, &mut pieces_idx);

        self.finny_tables.prepopulate(
            pieces_idx.as_slice(),
            &self.networks.big_net.feature_transformer,
            &self.networks.small_net.feature_transformer,
            self.king_squares,
        );
    }

    pub(super) fn add_piece_internal(&mut self, piece: Piece, square: Square) {
        if piece == Piece::None {
            return;
        }

        if self.pieces[square] != Piece::None {
            self.remove_piece_internal(square);
        }

        self.pieces[square] = piece;
        self.piece_count += 1;

        if let Some(color) = piece.color() {
            let piece_type = piece.piece_type();
            if piece_type > 0 {
                let mask = 1u64 << square;
                self.by_color_bb[color.index()] |= mask;
                self.by_type_bb[piece_type - 1] |= mask;
            }
        }

        if let Some(color) = piece.color() {
            let side = color.index();
            if piece.piece_type() == 1 {
                self.pawn_count[side] += 1;
            } else if piece.is_king() {
                self.king_squares[side] = square;
            } else {
                self.non_pawn_material[side] += self.piece_value(piece);
            }
        }
    }

    pub(super) fn remove_piece_internal(&mut self, square: Square) -> Piece {
        let piece = self.pieces[square];
        if piece == Piece::None {
            return Piece::None;
        }

        self.pieces[square] = Piece::None;
        self.piece_count -= 1;

        if let Some(color) = piece.color() {
            let piece_type = piece.piece_type();
            if piece_type > 0 {
                let mask = !(1u64 << square);
                self.by_color_bb[color.index()] &= mask;
                self.by_type_bb[piece_type - 1] &= mask;
            }
        }

        if let Some(color) = piece.color() {
            let side = color.index();
            if piece.piece_type() == 1 {
                self.pawn_count[side] -= 1;
            } else if !piece.is_king() {
                self.non_pawn_material[side] -= self.piece_value(piece);
            }
        }

        piece
    }

    pub(super) fn piece_value(&self, piece: Piece) -> i32 {
        match piece {
            Piece::WhiteKnight | Piece::BlackKnight => KNIGHT_VALUE,
            Piece::WhiteBishop | Piece::BlackBishop => BISHOP_VALUE,
            Piece::WhiteRook | Piece::BlackRook => ROOK_VALUE,
            Piece::WhiteQueen | Piece::BlackQueen => QUEEN_VALUE,
            _ => 0,
        }
    }

    pub(super) fn refresh_accumulators(&mut self) {
        let mut pieces_idx = PieceList::new();
        collect_pieces_from(&self.pieces, &mut pieces_idx);

        self.accumulator_stack.refresh(
            pieces_idx.as_slice(),
            self.king_squares,
            &self.networks.big_net.feature_transformer,
            &self.networks.small_net.feature_transformer,
        );
    }

    /// Directly mutates the current position without touching the stack.
    pub fn update(&mut self, removed: &[(Piece, Square)], added: &[(Piece, Square)]) {
        if removed.len() > PIECE_LIST_CAPACITY || added.len() > PIECE_LIST_CAPACITY {
            let mut removed_mapped = Vec::with_capacity(removed.len());
            let mut added_mapped = Vec::with_capacity(added.len());
            let mut king_moved = false;

            for &(piece, square) in removed {
                self.remove_piece_internal(square);
                removed_mapped.push((square, piece.index()));
                king_moved |= piece.is_king();
            }

            for &(piece, square) in added {
                self.add_piece_internal(piece, square);
                added_mapped.push((square, piece.index()));
                king_moved |= piece.is_king();
            }

            if king_moved {
                self.refresh_accumulators();
            } else {
                let state = self.accumulator_stack.mut_latest();
                state.acc_big.update_with_ksq(
                    &added_mapped,
                    &removed_mapped,
                    self.king_squares,
                    &self.networks.big_net.feature_transformer,
                );
                state.acc_small.update_with_ksq(
                    &added_mapped,
                    &removed_mapped,
                    self.king_squares,
                    &self.networks.small_net.feature_transformer,
                );
            }
            self.debug_assert_consistent();
            return;
        }

        let mut removed_mapped = PieceList::new();
        let mut added_mapped = PieceList::new();
        let mut king_moved = false;

        for &(piece, square) in removed {
            self.remove_piece_internal(square);
            removed_mapped.push(square, piece.index());
            king_moved |= piece.is_king();
        }

        for &(piece, square) in added {
            self.add_piece_internal(piece, square);
            added_mapped.push(square, piece.index());
            king_moved |= piece.is_king();
        }

        if king_moved {
            self.refresh_accumulators();
        } else {
            let state = self.accumulator_stack.mut_latest();
            state.acc_big.update_with_ksq(
                added_mapped.as_slice(),
                removed_mapped.as_slice(),
                self.king_squares,
                &self.networks.big_net.feature_transformer,
            );
            state.acc_small.update_with_ksq(
                added_mapped.as_slice(),
                removed_mapped.as_slice(),
                self.king_squares,
                &self.networks.small_net.feature_transformer,
            );
        }

        self.debug_assert_consistent();
    }

    #[cfg(debug_assertions)]
    pub(super) fn debug_assert_consistent(&self) {
        let mut piece_count = 0usize;
        let mut pawn_count = [0; 2];
        let mut non_pawn_material = [0; 2];
        let mut by_color_bb = [0u64; 2];
        let mut by_type_bb = [0u64; 6];
        let mut king_squares = [None; 2];

        for (square, piece) in self.pieces.iter().copied().enumerate() {
            if piece == Piece::None {
                continue;
            }

            piece_count += 1;

            if let Some(color) = piece.color() {
                let side = color.index();
                let piece_type = piece.piece_type();
                let mask = 1u64 << square;

                by_color_bb[side] |= mask;
                if piece_type > 0 {
                    by_type_bb[piece_type - 1] |= mask;
                }

                if piece_type == 1 {
                    pawn_count[side] += 1;
                } else if piece.is_king() {
                    assert!(
                        king_squares[side].replace(square).is_none(),
                        "multiple kings for side {side}"
                    );
                } else {
                    non_pawn_material[side] += self.piece_value(piece);
                }
            }
        }

        assert_eq!(self.piece_count, piece_count, "piece count drifted");
        assert_eq!(self.pawn_count, pawn_count, "pawn counts drifted");
        assert_eq!(
            self.non_pawn_material, non_pawn_material,
            "material counts drifted"
        );
        assert_eq!(self.by_color_bb, by_color_bb, "color bitboards drifted");
        assert_eq!(self.by_type_bb, by_type_bb, "piece-type bitboards drifted");
        assert_eq!(
            self.king_squares[0],
            king_squares[0].expect("missing white king")
        );
        assert_eq!(
            self.king_squares[1],
            king_squares[1].expect("missing black king")
        );
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub(super) fn debug_assert_consistent(&self) {}
}
