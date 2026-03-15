use crate::{Color, MoveType, Piece, Position};
use nnuebie::{Color as NnueColor, Piece as NnuePiece};

#[inline(always)]
pub fn map_color(color: Color) -> NnueColor {
    match color {
        Color::White => NnueColor::White,
        Color::Black => NnueColor::Black,
    }
}

#[inline(always)]
pub fn map_piece(piece: Piece, color: Color) -> NnuePiece {
    match (color, piece) {
        (Color::White, Piece::Pawn) => NnuePiece::WhitePawn,
        (Color::White, Piece::Knight) => NnuePiece::WhiteKnight,
        (Color::White, Piece::Bishop) => NnuePiece::WhiteBishop,
        (Color::White, Piece::Rook) => NnuePiece::WhiteRook,
        (Color::White, Piece::Queen) => NnuePiece::WhiteQueen,
        (Color::White, Piece::King) => NnuePiece::WhiteKing,
        (Color::Black, Piece::Pawn) => NnuePiece::BlackPawn,
        (Color::Black, Piece::Knight) => NnuePiece::BlackKnight,
        (Color::Black, Piece::Bishop) => NnuePiece::BlackBishop,
        (Color::Black, Piece::Rook) => NnuePiece::BlackRook,
        (Color::Black, Piece::Queen) => NnuePiece::BlackQueen,
        (Color::Black, Piece::King) => NnuePiece::BlackKing,
    }
}

#[inline(always)]
pub fn promotion_piece(move_type: MoveType, color: Color) -> NnuePiece {
    let promoted = match move_type {
        MoveType::PromotionKnight | MoveType::CapturePromotionKnight => Piece::Knight,
        MoveType::PromotionBishop | MoveType::CapturePromotionBishop => Piece::Bishop,
        MoveType::PromotionRook | MoveType::CapturePromotionRook => Piece::Rook,
        MoveType::PromotionQueen | MoveType::CapturePromotionQueen => Piece::Queen,
        _ => unreachable!("not a promotion move"),
    };

    map_piece(promoted, color)
}

pub fn collect_pieces(pos: &Position) -> Vec<(NnuePiece, usize)> {
    let mut pieces = Vec::with_capacity(32);
    for sq in 0..64 {
        if let Some((piece, color)) = pos.board[sq] {
            pieces.push((map_piece(piece, color), sq));
        }
    }
    pieces
}

#[inline(always)]
pub fn material_count(pos: &Position) -> i32 {
    let pawns = pos.pieces[Piece::Pawn as usize].0.count_ones() as i32;
    let knights = pos.pieces[Piece::Knight as usize].0.count_ones() as i32;
    let bishops = pos.pieces[Piece::Bishop as usize].0.count_ones() as i32;
    let rooks = pos.pieces[Piece::Rook as usize].0.count_ones() as i32;
    let queens = pos.pieces[Piece::Queen as usize].0.count_ones() as i32;

    pawns + 3 * knights + 3 * bishops + 5 * rooks + 9 * queens
}
