use super::MoveDelta;
use crate::accumulator_stack::{AccumulatorStack, DirtyPiece};
use crate::finny_tables::FinnyTables;
use crate::network::{
    NnueNetworks, ScratchBuffer, BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE,
};
use crate::piece_list::{collect_pieces_from, PieceList, PIECE_LIST_CAPACITY};
use crate::types::{Color, Piece, Square};
use std::io;
use std::sync::Arc;

/// Stateful NNUE evaluator backed by shared immutable network weights.
pub struct NNUEProbe {
    networks: Arc<NnueNetworks>,
    scratch_big: ScratchBuffer,
    scratch_small: ScratchBuffer,
    pieces: [Piece; 64],
    king_squares: [Square; 2],
    piece_count: usize,
    pawn_count: [i32; 2],
    non_pawn_material: [i32; 2],
    by_color_bb: [u64; 2],
    by_type_bb: [u64; 6],
    accumulator_stack: AccumulatorStack,
    finny_tables: FinnyTables,
}

impl NNUEProbe {
    /// Loads both networks from disk and creates a probe around them.
    pub fn new(big_path: &str, small_path: &str) -> io::Result<Self> {
        let networks = Arc::new(NnueNetworks::new(big_path, small_path)?);
        Ok(Self::from_networks(networks))
    }

    /// Builds a probe from already-loaded shared network weights.
    pub fn from_networks(networks: Arc<NnueNetworks>) -> Self {
        let scratch_big = ScratchBuffer::new(networks.big_net.feature_transformer.half_dims);
        let scratch_small = ScratchBuffer::new(networks.small_net.feature_transformer.half_dims);

        let mut finny_tables = FinnyTables::new();
        finny_tables.clear(
            &networks.big_net.feature_transformer.biases,
            &networks.small_net.feature_transformer.biases,
        );

        Self {
            networks,
            scratch_big,
            scratch_small,
            pieces: [Piece::None; 64],
            king_squares: [0; 2],
            piece_count: 0,
            pawn_count: [0; 2],
            non_pawn_material: [0; 2],
            by_color_bb: [0; 2],
            by_type_bb: [0; 6],
            accumulator_stack: AccumulatorStack::new(),
            finny_tables,
        }
    }

    pub fn with_networks(networks: Arc<NnueNetworks>) -> io::Result<Self> {
        Ok(Self::from_networks(networks))
    }

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

    fn add_piece_internal(&mut self, piece: Piece, square: Square) {
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

    fn remove_piece_internal(&mut self, square: Square) -> Piece {
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

    fn piece_value(&self, piece: Piece) -> i32 {
        match piece {
            Piece::WhiteKnight | Piece::BlackKnight => KNIGHT_VALUE,
            Piece::WhiteBishop | Piece::BlackBishop => BISHOP_VALUE,
            Piece::WhiteRook | Piece::BlackRook => ROOK_VALUE,
            Piece::WhiteQueen | Piece::BlackQueen => QUEEN_VALUE,
            _ => 0,
        }
    }

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
    }

    fn refresh_accumulators(&mut self) {
        let mut pieces_idx = PieceList::new();
        collect_pieces_from(&self.pieces, &mut pieces_idx);

        self.accumulator_stack.refresh(
            pieces_idx.as_slice(),
            self.king_squares,
            &self.networks.big_net.feature_transformer,
            &self.networks.small_net.feature_transformer,
        );
    }

    /// Evaluates the current position from the side-to-move perspective.
    pub fn evaluate(&mut self, side_to_move: Color) -> i32 {
        let stm = side_to_move.index();
        let simple_eval = PAWN_VALUE * (self.pawn_count[stm] - self.pawn_count[1 - stm])
            + (self.non_pawn_material[stm] - self.non_pawn_material[1 - stm]);
        let use_small = simple_eval.abs() > 962;

        let bucket = if self.piece_count > 0 {
            (self.piece_count - 1) / 4
        } else {
            0
        }
        .min(7);

        let latest = self.accumulator_stack.latest();

        let (mut nnue_val, psqt_val, positional_val) = if use_small {
            let (psqt, pos) = self.networks.small_net.evaluate(
                &latest.acc_small,
                bucket,
                stm,
                &mut self.scratch_small,
            );
            let mut score = (125 * psqt + 131 * pos) / 128;

            if score.abs() < 236 {
                let (big_psqt, big_pos) = self.networks.big_net.evaluate(
                    &latest.acc_big,
                    bucket,
                    stm,
                    &mut self.scratch_big,
                );
                score = (125 * big_psqt + 131 * big_pos) / 128;
                (score, big_psqt, big_pos)
            } else {
                (score, psqt, pos)
            }
        } else {
            let (psqt, pos) =
                self.networks
                    .big_net
                    .evaluate(&latest.acc_big, bucket, stm, &mut self.scratch_big);
            ((125 * psqt + 131 * pos) / 128, psqt, pos)
        };

        let nnue_complexity = (psqt_val - positional_val).abs();
        nnue_val -= nnue_val * nnue_complexity / 18000;

        let material = 535 * (self.pawn_count[0] + self.pawn_count[1])
            + (self.non_pawn_material[0] + self.non_pawn_material[1]);
        let mut score = nnue_val * (77777 + material) / 77777;

        score -= score * latest.rule50 / 212;
        score.clamp(-31753, 31753)
    }
}
