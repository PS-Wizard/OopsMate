#![allow(dead_code)]

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BoardIdx {
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

impl BoardIdx {
    pub fn idx(&self) -> usize {
        debug_assert!(*self != BoardIdx::Empty, "Tried to index with Empty Piece");
        (*self as usize) - 1
    }
}
