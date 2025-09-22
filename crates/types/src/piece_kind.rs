#![allow(dead_code)]

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum PieceKind {
    WhitePawn,
    WhiteRook,
    WhiteKnight,
    WhiteBishop,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackRook,
    BlackKnight,
    BlackBishop,
    BlackQueen,
    BlackKing,
    Empty,
}
