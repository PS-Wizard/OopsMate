#![allow(dead_code)]

/// A Side Irrespective version of the PieceMap, only exists cause doing `self.boards[self.turn as usize * 6 + PieceKind as usize].0` felt weird 
/// as that would mean that I'd have to pass in a `PieceKind::WhitePawn` to get the index of the
/// BlackPawn, so that's confusion pro max ultra. Nuh uh.
#[repr(u8)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

/// Side Respective Piece Maps
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
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
    None,
}

impl PieceKind {
    /// For Parsing FEN
    pub fn from_char(ch: char) -> Self {
        match ch {
            'P' => Self::WhitePawn,
            'R' => Self::WhiteRook,
            'N' => Self::WhiteKnight,
            'B' => Self::WhiteBishop,
            'Q' => Self::WhiteQueen,
            'K' => Self::WhiteKing,
            'p' => Self::BlackPawn,
            'r' => Self::BlackRook,
            'n' => Self::BlackKnight,
            'b' => Self::BlackBishop,
            'q' => Self::BlackQueen,
            'k' => Self::BlackKing,
            _ => Self::None,
        }
    }

    /// Piece Values
    pub fn piece_value(&self) -> u32 {
        match self {
            Self::WhitePawn | Self::BlackPawn => 100,
            Self::WhiteKnight | Self::BlackKnight => 320,
            Self::WhiteBishop | Self::BlackBishop => 330,
            Self::WhiteRook | Self::BlackRook => 500,
            Self::WhiteQueen | Self::BlackQueen => 900,
            Self::WhiteKing | Self::BlackKing => 20000,
            Self::None => 0,
        }
    }
}
