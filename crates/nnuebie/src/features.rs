//! Feature indexing tables for the Stockfish NNUE layout.

/// Number of board squares.
pub const SQUARE_NB: usize = 64;
/// White perspective index.
pub const WHITE: usize = 0;
/// Black perspective index.
pub const BLACK: usize = 1;

/// Pawn piece-type index.
pub const PAWN: usize = 1;
/// Knight piece-type index.
pub const KNIGHT: usize = 2;
/// Bishop piece-type index.
pub const BISHOP: usize = 3;
/// Rook piece-type index.
pub const ROOK: usize = 4;
/// Queen piece-type index.
pub const QUEEN: usize = 5;
/// King piece-type index.
pub const KING: usize = 6;

const PS_W_PAWN: usize = 0;
const PS_B_PAWN: usize = SQUARE_NB;
const PS_W_KNIGHT: usize = 2 * SQUARE_NB;
const PS_B_KNIGHT: usize = 3 * SQUARE_NB;
const PS_W_BISHOP: usize = 4 * SQUARE_NB;
const PS_B_BISHOP: usize = 5 * SQUARE_NB;
const PS_W_ROOK: usize = 6 * SQUARE_NB;
const PS_B_ROOK: usize = 7 * SQUARE_NB;
const PS_W_QUEEN: usize = 8 * SQUARE_NB;
const PS_B_QUEEN: usize = 9 * SQUARE_NB;
const PS_KING: usize = 10 * SQUARE_NB;
const PS_NONE: usize = 0;

/// Piece/square offset table indexed by perspective and Stockfish piece code.
pub const PIECE_SQUARE_INDEX: [[usize; 16]; 2] = [
    [
        PS_NONE,
        PS_W_PAWN,
        PS_W_KNIGHT,
        PS_W_BISHOP,
        PS_W_ROOK,
        PS_W_QUEEN,
        PS_KING,
        PS_NONE,
        PS_NONE,
        PS_B_PAWN,
        PS_B_KNIGHT,
        PS_B_BISHOP,
        PS_B_ROOK,
        PS_B_QUEEN,
        PS_KING,
        PS_NONE,
    ],
    [
        PS_NONE,
        PS_B_PAWN,
        PS_B_KNIGHT,
        PS_B_BISHOP,
        PS_B_ROOK,
        PS_B_QUEEN,
        PS_KING,
        PS_NONE,
        PS_NONE,
        PS_W_PAWN,
        PS_W_KNIGHT,
        PS_W_BISHOP,
        PS_W_ROOK,
        PS_W_QUEEN,
        PS_KING,
        PS_NONE,
    ],
];

const PS_NB: usize = 11 * 64;
const fn b(v: usize) -> usize {
    v * PS_NB
}

/// King-bucket offsets indexed by perspective and king square.
#[rustfmt::skip]
pub const KING_BUCKETS: [[usize; 64]; 2] = [
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

// XOR masks for square transformations
const FLIP_HORIZONTAL: usize = 7; // Mirror files: a -> h, b -> g, c -> f, d -> e
const FLIP_VERTICAL: usize = 56; // Flip ranks: 1 -> 8, 2 -> 7, etc.
const FLIP_BOTH: usize = 63; // Both transforms (56 | 7)
const NO_FLIP: usize = 0; // No transformation

#[rustfmt::skip]
/// XOR transformation masks indexed by [perspective][king_square].
/// Applies vertical flip for Black + horizontal mirror when king is on files a-d.
pub const ORIENT_TBL: [[usize; 64]; 2] = [
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

/// Builds a sparse feature index from perspective, square, piece, and king square.
pub fn make_index(perspective: usize, s: usize, pc: usize, ksq: usize) -> usize {
    let orient = ORIENT_TBL[perspective][ksq];
    let piece_offset = PIECE_SQUARE_INDEX[perspective][pc];
    let bucket_offset = KING_BUCKETS[perspective][ksq];

    (s ^ orient) + piece_offset + bucket_offset
}
