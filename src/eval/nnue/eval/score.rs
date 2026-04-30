use crate::eval::nnue::constants::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::eval::nnue::compat::{Color, Piece, Position, PositionExt};

#[inline(always)]
pub(super) fn blended_nnue(psqt: i32, positional: i32) -> i32 {
    (125 * psqt + 131 * positional) / 128
}

pub(super) fn simple_eval(position: &Position, side_to_move: Color) -> i32 {
    PAWN_VALUE * (pawn_count(position, side_to_move) - pawn_count(position, side_to_move.flip()))
        + (non_pawn_material(position, side_to_move)
            - non_pawn_material(position, side_to_move.flip()))
}

pub(super) fn total_material_for_scaling(position: &Position) -> i32 {
    let board = position.board();
    535 * board.piece_bb(Piece::Pawn).count_ones() as i32
        + board.piece_bb(Piece::Knight).count_ones() as i32 * KNIGHT_VALUE
        + board.piece_bb(Piece::Bishop).count_ones() as i32 * BISHOP_VALUE
        + board.piece_bb(Piece::Rook).count_ones() as i32 * ROOK_VALUE
        + board.piece_bb(Piece::Queen).count_ones() as i32 * QUEEN_VALUE
}

pub(super) fn to_centipawns(value: i32, position: &Position) -> i32 {
    let material = coarse_material_for_cp(position);
    let m = material.clamp(17, 78) as f64 / 58.0;
    let a = (((-13.50030198 * m + 40.92780883) * m - 36.82753545) * m) + 386.83004070;

    (100.0 * f64::from(value) / a).round() as i32
}

fn pawn_count(position: &Position, color: Color) -> i32 {
    let board = position.board();
    (board.piece_bb(Piece::Pawn) & board.color_bb(color)).count_ones() as i32
}

fn non_pawn_material(position: &Position, color: Color) -> i32 {
    let board = position.board();
    let color_bb = board.color_bb(color);

    ((board.piece_bb(Piece::Knight) & color_bb).count_ones() as i32 * KNIGHT_VALUE)
        + ((board.piece_bb(Piece::Bishop) & color_bb).count_ones() as i32 * BISHOP_VALUE)
        + ((board.piece_bb(Piece::Rook) & color_bb).count_ones() as i32 * ROOK_VALUE)
        + ((board.piece_bb(Piece::Queen) & color_bb).count_ones() as i32 * QUEEN_VALUE)
}

fn coarse_material_for_cp(position: &Position) -> i32 {
    let board = position.board();
    board.piece_bb(Piece::Pawn).count_ones() as i32
        + 3 * board.piece_bb(Piece::Knight).count_ones() as i32
        + 3 * board.piece_bb(Piece::Bishop).count_ones() as i32
        + 5 * board.piece_bb(Piece::Rook).count_ones() as i32
        + 9 * board.piece_bb(Piece::Queen).count_ones() as i32
}
