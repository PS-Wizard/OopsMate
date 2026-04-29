use crate::constants::FEATURE_DIMS;
#[cfg(test)]
use crate::oopsmate_core::{Board, Position};
use crate::oopsmate_core::{Color, EMPTY_SQUARE, Piece, Square, color_from_code, piece_from_code};

const NO_FLIP: usize = 0;
const FLIP_HORIZONTAL: usize = 7;
const FLIP_VERTICAL: usize = 56;
const FLIP_BOTH: usize = 63;

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
const PS_NB: usize = 11 * 64;

const fn bucket(v: usize) -> usize {
    v * PS_NB
}

#[rustfmt::skip]
const ORIENT_TBL: [[usize; 64]; 2] = [
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

#[rustfmt::skip]
const KING_BUCKETS: [[usize; 64]; 2] = [
    [
        bucket(28), bucket(29), bucket(30), bucket(31), bucket(31), bucket(30), bucket(29), bucket(28),
        bucket(24), bucket(25), bucket(26), bucket(27), bucket(27), bucket(26), bucket(25), bucket(24),
        bucket(20), bucket(21), bucket(22), bucket(23), bucket(23), bucket(22), bucket(21), bucket(20),
        bucket(16), bucket(17), bucket(18), bucket(19), bucket(19), bucket(18), bucket(17), bucket(16),
        bucket(12), bucket(13), bucket(14), bucket(15), bucket(15), bucket(14), bucket(13), bucket(12),
        bucket(8),  bucket(9),  bucket(10), bucket(11), bucket(11), bucket(10), bucket(9),  bucket(8),
        bucket(4),  bucket(5),  bucket(6),  bucket(7),  bucket(7),  bucket(6),  bucket(5),  bucket(4),
        bucket(0),  bucket(1),  bucket(2),  bucket(3),  bucket(3),  bucket(2),  bucket(1),  bucket(0),
    ],
    [
        bucket(0),  bucket(1),  bucket(2),  bucket(3),  bucket(3),  bucket(2),  bucket(1),  bucket(0),
        bucket(4),  bucket(5),  bucket(6),  bucket(7),  bucket(7),  bucket(6),  bucket(5),  bucket(4),
        bucket(8),  bucket(9),  bucket(10), bucket(11), bucket(11), bucket(10), bucket(9),  bucket(8),
        bucket(12), bucket(13), bucket(14), bucket(15), bucket(15), bucket(14), bucket(13), bucket(12),
        bucket(16), bucket(17), bucket(18), bucket(19), bucket(19), bucket(18), bucket(17), bucket(16),
        bucket(20), bucket(21), bucket(22), bucket(23), bucket(23), bucket(22), bucket(21), bucket(20),
        bucket(24), bucket(25), bucket(26), bucket(27), bucket(27), bucket(26), bucket(25), bucket(24),
        bucket(28), bucket(29), bucket(30), bucket(31), bucket(31), bucket(30), bucket(29), bucket(28),
    ],
];

#[inline(always)]
fn piece_square_index(perspective: usize, piece: Piece, color: Color) -> usize {
    match (perspective, color, piece) {
        (0, Color::White, Piece::Pawn) => PS_W_PAWN,
        (0, Color::White, Piece::Knight) => PS_W_KNIGHT,
        (0, Color::White, Piece::Bishop) => PS_W_BISHOP,
        (0, Color::White, Piece::Rook) => PS_W_ROOK,
        (0, Color::White, Piece::Queen) => PS_W_QUEEN,
        (0, Color::White, Piece::King) => PS_KING,
        (0, Color::Black, Piece::Pawn) => PS_B_PAWN,
        (0, Color::Black, Piece::Knight) => PS_B_KNIGHT,
        (0, Color::Black, Piece::Bishop) => PS_B_BISHOP,
        (0, Color::Black, Piece::Rook) => PS_B_ROOK,
        (0, Color::Black, Piece::Queen) => PS_B_QUEEN,
        (0, Color::Black, Piece::King) => PS_KING,
        (1, Color::White, Piece::Pawn) => PS_B_PAWN,
        (1, Color::White, Piece::Knight) => PS_B_KNIGHT,
        (1, Color::White, Piece::Bishop) => PS_B_BISHOP,
        (1, Color::White, Piece::Rook) => PS_B_ROOK,
        (1, Color::White, Piece::Queen) => PS_B_QUEEN,
        (1, Color::White, Piece::King) => PS_KING,
        (1, Color::Black, Piece::Pawn) => PS_W_PAWN,
        (1, Color::Black, Piece::Knight) => PS_W_KNIGHT,
        (1, Color::Black, Piece::Bishop) => PS_W_BISHOP,
        (1, Color::Black, Piece::Rook) => PS_W_ROOK,
        (1, Color::Black, Piece::Queen) => PS_W_QUEEN,
        (1, Color::Black, Piece::King) => PS_KING,
        _ => unreachable!("invalid perspective"),
    }
}

#[inline(always)]
#[must_use]
pub fn feature_index_from_piece_code(
    perspective: Color,
    piece_code: u8,
    square: Square,
    king_square: Square,
) -> u32 {
    debug_assert!(square.is_valid());
    debug_assert!(king_square.is_valid());
    debug_assert_ne!(piece_code, EMPTY_SQUARE);

    let perspective_index = perspective.index();
    let square_index = square.raw() as usize;
    let king_square_index = king_square.raw() as usize;
    let piece = piece_from_code(piece_code);
    let color = color_from_code(piece_code);
    let index = (square_index ^ ORIENT_TBL[perspective_index][king_square_index])
        + piece_square_index(perspective_index, piece, color)
        + KING_BUCKETS[perspective_index][king_square_index];

    debug_assert!(index < FEATURE_DIMS);
    index as u32
}

#[cfg(test)]
#[inline(always)]
pub fn enumerate_active_features(
    position: &Position,
    out: &mut [[u32; crate::constants::MAX_ACTIVE_FEATURES]; 2],
    lengths: &mut [usize; 2],
) -> usize {
    lengths.fill(0);

    let board = position.board();
    push_perspective_features(
        &board,
        Color::White,
        board.king_square(Color::White),
        &mut out[0],
        &mut lengths[0],
    );
    push_perspective_features(
        &board,
        Color::Black,
        board.king_square(Color::Black),
        &mut out[1],
        &mut lengths[1],
    );

    debug_assert_eq!(lengths[0], lengths[1]);
    debug_assert!(lengths[0] <= crate::constants::MAX_ACTIVE_FEATURES);
    lengths[0]
}

#[cfg(test)]
fn push_perspective_features(
    board: &Board,
    perspective: Color,
    king_square: Square,
    out: &mut [u32; crate::constants::MAX_ACTIVE_FEATURES],
    len: &mut usize,
) {
    for raw in 0u8..64 {
        let square = Square::from_raw(raw);
        let piece_code = board.piece_code_at(square);
        if piece_code == EMPTY_SQUARE {
            continue;
        }

        out[*len] = feature_index_from_piece_code(perspective, piece_code, square, king_square);
        *len += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{enumerate_active_features, feature_index_from_piece_code};
    use crate::constants::MAX_ACTIVE_FEATURES;
    use crate::oopsmate_core::{Color, Piece, Position, Square, encode_piece};

    #[test]
    fn startpos_enumerates_32_features_per_perspective() {
        let position = Position::startpos();
        let mut indices = [[0u32; MAX_ACTIVE_FEATURES]; 2];
        let mut lengths = [0usize; 2];

        let piece_count = enumerate_active_features(&position, &mut indices, &mut lengths);

        assert_eq!(piece_count, 32);
        assert_eq!(lengths, [32, 32]);
    }

    #[test]
    fn direct_feature_index_matches_startpos_white_king_piece() {
        let position = Position::startpos();
        let board = position.board();
        let index = feature_index_from_piece_code(
            Color::White,
            encode_piece(Piece::King, Color::White),
            Square::from_algebraic("e1").unwrap(),
            board.king_square(Color::White),
        );

        assert!(index < crate::constants::FEATURE_DIMS as u32);
    }
}
