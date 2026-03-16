//! Helpers for converting raw NNUE scores into UCI-facing values.

use crate::features::{BISHOP, KNIGHT, PAWN, QUEEN, ROOK};
use crate::{Piece, Square};

/// Parameters of the fitted Stockfish win-rate model.
pub struct WinRateParams {
    pub a: f64,
    pub b: f64,
}

/// Returns the fitted win-rate parameters for a given material count.
pub fn win_rate_params(material: i32) -> WinRateParams {
    let m = (material.clamp(17, 78) as f64) / 58.0;

    let as_coeffs = [-13.50030198, 40.92780883, -36.82753545, 386.83004070];
    let bs_coeffs = [96.53354896, -165.79058388, 90.89679019, 49.29561889];

    let a = (((as_coeffs[0] * m + as_coeffs[1]) * m + as_coeffs[2]) * m) + as_coeffs[3];
    let b = (((bs_coeffs[0] * m + bs_coeffs[1]) * m + bs_coeffs[2]) * m) + bs_coeffs[3];

    WinRateParams { a, b }
}

/// Converts an internal NNUE score into a centipawn value.
pub fn to_centipawns(value: i32, material: i32) -> i32 {
    let params = win_rate_params(material);
    (100.0 * (value as f64) / params.a).round() as i32
}

/// Computes coarse material from `(square, piece_type, color)` tuples.
pub fn calculate_material(pieces: &[(usize, usize, usize)]) -> i32 {
    let mut material = 0;
    for &(_, pt, _) in pieces {
        match pt {
            PAWN => material += 1,
            KNIGHT => material += 3,
            BISHOP => material += 3,
            ROOK => material += 5,
            QUEEN => material += 9,
            _ => {}
        }
    }
    material
}

/// Computes coarse material directly from `(piece, square)` tuples.
pub fn calculate_material_from_pieces(pieces: &[(Piece, Square)]) -> i32 {
    let mut material = 0;
    for &(piece, _) in pieces {
        match piece {
            Piece::WhitePawn | Piece::BlackPawn => material += 1,
            Piece::WhiteKnight | Piece::BlackKnight => material += 3,
            Piece::WhiteBishop | Piece::BlackBishop => material += 3,
            Piece::WhiteRook | Piece::BlackRook => material += 5,
            Piece::WhiteQueen | Piece::BlackQueen => material += 9,
            _ => {}
        }
    }
    material
}
