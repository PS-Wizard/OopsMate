use super::eg_tables::{EG_TABLES, EG_VALUE};
use super::mg_tables::{GAME_PHASE_INC, MG_TABLES, MG_VALUE};
use crate::{Bitboard, Position};

pub(super) fn evaluate(pos: &Position) -> i32 {
    let mut mg = [0i32; 2];
    let mut eg = [0i32; 2];
    let mut game_phase = 0i32;

    let white_pieces = pos.colors[crate::Color::White as usize].0;
    let black_pieces = pos.colors[crate::Color::Black as usize].0;

    unsafe {
        eval_piece::<0>(&pos.pieces, white_pieces, black_pieces, &mut mg, &mut eg, &mut game_phase);
        eval_piece::<1>(&pos.pieces, white_pieces, black_pieces, &mut mg, &mut eg, &mut game_phase);
        eval_piece::<2>(&pos.pieces, white_pieces, black_pieces, &mut mg, &mut eg, &mut game_phase);
        eval_piece::<3>(&pos.pieces, white_pieces, black_pieces, &mut mg, &mut eg, &mut game_phase);
        eval_piece::<4>(&pos.pieces, white_pieces, black_pieces, &mut mg, &mut eg, &mut game_phase);
        eval_piece::<5>(&pos.pieces, white_pieces, black_pieces, &mut mg, &mut eg, &mut game_phase);
    }

    let side = pos.side_to_move as usize;
    let mg_score = mg[side] - mg[side ^ 1];
    let eg_score = eg[side] - eg[side ^ 1];
    let mg_phase = game_phase.min(24);
    let eg_phase = 24 - mg_phase;

    (mg_score * mg_phase + eg_score * eg_phase) / 24
}

#[inline(always)]
unsafe fn eval_piece<const PIECE: usize>(
    pieces: &[Bitboard; 6],
    white_pieces: u64,
    black_pieces: u64,
    mg: &mut [i32; 2],
    eg: &mut [i32; 2],
    game_phase: &mut i32,
) {
    let piece_bb = pieces.get_unchecked(PIECE).0;
    let mg_val = *MG_VALUE.get_unchecked(PIECE);
    let eg_val = *EG_VALUE.get_unchecked(PIECE);
    let phase_val = *GAME_PHASE_INC.get_unchecked(PIECE);
    let mg_pst = MG_TABLES.get_unchecked(PIECE);
    let eg_pst = EG_TABLES.get_unchecked(PIECE);

    let mut us = piece_bb & white_pieces;
    while us != 0 {
        let sq = us.trailing_zeros() as usize;
        us &= us.wrapping_sub(1);

        *mg.get_unchecked_mut(0) += mg_val + mg_pst.get_unchecked(sq);
        *eg.get_unchecked_mut(0) += eg_val + eg_pst.get_unchecked(sq);
        *game_phase += phase_val;
    }

    let mut them = piece_bb & black_pieces;
    while them != 0 {
        let sq = them.trailing_zeros() as usize;
        them &= them.wrapping_sub(1);
        let mirror_sq = sq ^ 56;

        *mg.get_unchecked_mut(1) += mg_val + mg_pst.get_unchecked(mirror_sq);
        *eg.get_unchecked_mut(1) += eg_val + eg_pst.get_unchecked(mirror_sq);
        *game_phase += phase_val;
    }
}
