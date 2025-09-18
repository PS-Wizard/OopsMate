#![allow(dead_code)]

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Empty = 0,
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
}

impl PieceKind {
    #[inline(always)]
    pub fn idx(self) -> usize {
        debug_assert!(self != PieceKind::Empty, "Tried to index with Empty Piece");
        (self as usize) - 1
    }
}

