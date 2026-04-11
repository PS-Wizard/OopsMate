//! HalfKA Feature Indexing
//!
//! NNUE uses the HalfKA (Half King-All) feature set. Each feature represents:
//! "From perspective P, there is piece X on square S, and my king is on square K"
//!
//! The feature space is organized as:
//! - 32 king buckets (8 ranks × 4 mirrored files)
//! - 11 piece types (5 white pieces + 5 black pieces + our king)
//! - 64 squares
//!
//! Total: 32 × 11 × 64 = 22,528 features
//!
//! Key insight: We compute features from BOTH perspectives (White and Black),
//! and we apply transformations to normalize the board view:
//! - Vertical flip (^56) for Black's perspective
//! - Horizontal mirror (^7) when king is on queenside (files a-d)

use crate::types::{square_name, Piece};

// Perspective indices
pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

// Architecture constants
pub const HALF_DIMS: usize = 3072; // Accumulator size per perspective
pub const FEATURE_DIMS: usize = 22_528; // Total feature count (32 * 11 * 64)
pub const PSQT_BUCKETS: usize = 8; // PSQT bucket count

/// Transformation constants for ORIENT_TBL
/// These combine vertical flip and horizontal mirror operations
const NO_FLIP: usize = 0; // No transformation
const FLIP_HORIZONTAL: usize = 7; // Mirror horizontally (a↔h, b↔g, etc.)
const FLIP_VERTICAL: usize = 56; // Flip vertically (rank 1↔8, 2↔7, etc.)
const FLIP_BOTH: usize = 63; // Both flips combined

/// XOR transformation masks indexed by [perspective][king_square].
/// Applies vertical flip for Black + horizontal mirror when king is on files a-d.
#[rustfmt::skip]
const ORIENT_TBL: [[usize; 64]; 2] = [
    // White: mirror horizontally if king on left half (files a-d)
    [
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
        FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, FLIP_HORIZONTAL, NO_FLIP, NO_FLIP, NO_FLIP, NO_FLIP, 
    ],
    // Black: vertical flip always + horizontal mirror if king on left half
    [
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, 
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
        FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_BOTH, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL, FLIP_VERTICAL,
    ],
];

// Piece-square offset constants (Stockfish layout)
const PS_W_PAWN: usize = 0 * 64;
const PS_B_PAWN: usize = 1 * 64;
const PS_W_KNIGHT: usize = 2 * 64;
const PS_B_KNIGHT: usize = 3 * 64;
const PS_W_BISHOP: usize = 4 * 64;
const PS_B_BISHOP: usize = 5 * 64;
const PS_W_ROOK: usize = 6 * 64;
const PS_B_ROOK: usize = 7 * 64;
const PS_W_QUEEN: usize = 8 * 64;
const PS_B_QUEEN: usize = 9 * 64;
const PS_KING: usize = 10 * 64;
const PS_NONE: usize = 0;

/// Piece/square offset table indexed by perspective and Stockfish piece code.
/// piece code format: color bit (8) | piece_type (1-6)
/// White: 1=WP, 2=WN, 3=WB, 4=WR, 5=WQ, 6=WK
/// Black: 9=BP, 10=BN, 11=BB, 12=BR, 13=BQ, 14=BK
const PIECE_SQUARE_INDEX: [[usize; 16]; 2] = [
    // From White's perspective
    [
        PS_NONE,     // 0 = invalid
        PS_W_PAWN,   // 1 = White Pawn
        PS_W_KNIGHT, // 2 = White Knight
        PS_W_BISHOP, // 3 = White Bishop
        PS_W_ROOK,   // 4 = White Rook
        PS_W_QUEEN,  // 5 = White Queen
        PS_KING,     // 6 = White King
        PS_NONE,     // 7 = invalid
        PS_NONE,     // 8 = invalid
        PS_B_PAWN,   // 9 = Black Pawn
        PS_B_KNIGHT, // 10 = Black Knight
        PS_B_BISHOP, // 11 = Black Bishop
        PS_B_ROOK,   // 12 = Black Rook
        PS_B_QUEEN,  // 13 = Black Queen
        PS_KING,     // 14 = Black King
        PS_NONE,     // 15 = invalid
    ],
    // From Black's perspective (swapped colors)
    [
        PS_NONE,     // 0 = invalid
        PS_B_PAWN,   // 1 = White Pawn → treated as enemy pawn
        PS_B_KNIGHT, // 2 = White Knight → enemy knight
        PS_B_BISHOP, // 3 = White Bishop → enemy bishop
        PS_B_ROOK,   // 4 = White Rook → enemy rook
        PS_B_QUEEN,  // 5 = White Queen → enemy queen
        PS_KING,     // 6 = White King → enemy king
        PS_NONE,     // 7 = invalid
        PS_NONE,     // 8 = invalid
        PS_W_PAWN,   // 9 = Black Pawn → our pawn
        PS_W_KNIGHT, // 10 = Black Knight → our knight
        PS_W_BISHOP, // 11 = Black Bishop → our bishop
        PS_W_ROOK,   // 12 = Black Rook → our rook
        PS_W_QUEEN,  // 13 = Black Queen → our queen
        PS_KING,     // 14 = Black King → our king
        PS_NONE,     // 15 = invalid
    ],
];

const PS_NB: usize = 11 * 64; // Features per king bucket

const fn b(v: usize) -> usize {
    v * PS_NB
}

/// King-bucket offsets indexed by perspective and king square.
#[rustfmt::skip]
const KING_BUCKETS: [[usize; 64]; 2] = [
    // From White's perspective (king on ranks 1-8, bottom to top)
    [
        b(28), b(29), b(30), b(31), b(31), b(30), b(29), b(28),
        b(24), b(25), b(26), b(27), b(27), b(26), b(25), b(24),
        b(20), b(21), b(22), b(23), b(23), b(22), b(21), b(20),
        b(16), b(17), b(18), b(19), b(19), b(18), b(17), b(16),
        b(12), b(13), b(14), b(15), b(15), b(14), b(13), b(12),
        b(8), b(9), b(10), b(11), b(11), b(10), b(9), b(8),
        b(4), b(5), b(6), b(7), b(7), b(6), b(5), b(4),
        b(0), b(1), b(2), b(3), b(3), b(2), b(1), b(0),
    ],
    // From Black's perspective (flipped vertically)
    [
        b(0), b(1), b(2), b(3), b(3), b(2), b(1), b(0),
        b(4), b(5), b(6), b(7), b(7), b(6), b(5), b(4),
        b(8), b(9), b(10), b(11), b(11), b(10), b(9), b(8),
        b(12), b(13), b(14), b(15), b(15), b(14), b(13), b(12),
        b(16), b(17), b(18), b(19), b(19), b(18), b(17), b(16),
        b(20), b(21), b(22), b(23), b(23), b(22), b(21), b(20),
        b(24), b(25), b(26), b(27), b(27), b(26), b(25), b(24),
        b(28), b(29), b(30), b(31), b(31), b(30), b(29), b(28),
    ],
];

/// Calculate the feature index for a piece on a square, from a given perspective.
///
/// This matches nnuebie exactly:
/// - `perspective`: 0=White, 1=Black
/// - `s`: square index 0-63
/// - `pc`: Stockfish piece code (1-6 for white pieces, 9-14 for black pieces)
/// - `ksq`: king square for this perspective
pub fn make_index(perspective: usize, s: usize, pc: usize, ksq: usize) -> usize {
    let orient = ORIENT_TBL[perspective][ksq];
    let piece_offset = PIECE_SQUARE_INDEX[perspective][pc];
    let bucket_offset = KING_BUCKETS[perspective][ksq];

    (s ^ orient) + piece_offset + bucket_offset
}

/// Human-readable explanation of a feature index
pub fn explain_feature(perspective: usize, square: usize, piece: Piece, king_sq: usize) -> String {
    let persp_name = if perspective == WHITE {
        "White"
    } else {
        "Black"
    };
    let pc = piece.stockfish_code();
    let orient = ORIENT_TBL[perspective][king_sq];
    let transformed_sq = square ^ orient;
    let bucket_idx = KING_BUCKETS[perspective][king_sq] / PS_NB;
    let index = make_index(perspective, square, pc, king_sq);

    format!(
        "  {} perspective: {} on {} (king on {}) → orient={}, sq'={}, ksq'={}, bucket={}, index={}",
        persp_name,
        piece.symbol(),
        square_name(square),
        square_name(king_sq),
        orient,
        square_name(transformed_sq),
        square_name(king_sq),
        bucket_idx,
        index
    )
}
