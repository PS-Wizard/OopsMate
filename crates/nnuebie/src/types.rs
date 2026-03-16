//! Shared chess-domain types used by the probe API.

/// Chess side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// Returns the opposite side.
    pub fn flip(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Returns the side as a zero-based array index.
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Converts a side index into a color.
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Color::White,
            _ => Color::Black,
        }
    }
}

/// Chess piece encoded in the Stockfish NNUE layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    WhitePawn = 1,
    WhiteKnight = 2,
    WhiteBishop = 3,
    WhiteRook = 4,
    WhiteQueen = 5,
    WhiteKing = 6,
    BlackPawn = 9,
    BlackKnight = 10,
    BlackBishop = 11,
    BlackRook = 12,
    BlackQueen = 13,
    BlackKing = 14,
    None = 0,
}

impl Piece {
    /// Returns the color of the piece, or `None` for `Piece::None`.
    pub fn color(&self) -> Option<Color> {
        match self {
            Piece::WhitePawn
            | Piece::WhiteKnight
            | Piece::WhiteBishop
            | Piece::WhiteRook
            | Piece::WhiteQueen
            | Piece::WhiteKing => Some(Color::White),
            Piece::BlackPawn
            | Piece::BlackKnight
            | Piece::BlackBishop
            | Piece::BlackRook
            | Piece::BlackQueen
            | Piece::BlackKing => Some(Color::Black),
            Piece::None => None,
        }
    }

    /// Returns the piece type in the `1..=6` Stockfish layout.
    pub fn piece_type(&self) -> usize {
        match self {
            Piece::WhitePawn | Piece::BlackPawn => 1,
            Piece::WhiteKnight | Piece::BlackKnight => 2,
            Piece::WhiteBishop | Piece::BlackBishop => 3,
            Piece::WhiteRook | Piece::BlackRook => 4,
            Piece::WhiteQueen | Piece::BlackQueen => 5,
            Piece::WhiteKing | Piece::BlackKing => 6,
            Piece::None => 0,
        }
    }

    /// Returns `true` if the piece is a king.
    pub fn is_king(&self) -> bool {
        matches!(self, Piece::WhiteKing | Piece::BlackKing)
    }

    /// Returns the raw Stockfish piece index.
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Converts a raw Stockfish piece index into a piece value.
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => Piece::WhitePawn,
            2 => Piece::WhiteKnight,
            3 => Piece::WhiteBishop,
            4 => Piece::WhiteRook,
            5 => Piece::WhiteQueen,
            6 => Piece::WhiteKing,
            9 => Piece::BlackPawn,
            10 => Piece::BlackKnight,
            11 => Piece::BlackBishop,
            12 => Piece::BlackRook,
            13 => Piece::BlackQueen,
            14 => Piece::BlackKing,
            _ => Piece::None,
        }
    }
}

/// Board square index in `[0, 63]` using `A1 = 0`.
pub type Square = usize;

pub const A1: Square = 0;
pub const B1: Square = 1;
pub const C1: Square = 2;
pub const D1: Square = 3;
pub const E1: Square = 4;
pub const F1: Square = 5;
pub const G1: Square = 6;
pub const H1: Square = 7;

pub const A2: Square = 8;
pub const B2: Square = 9;
pub const C2: Square = 10;
pub const D2: Square = 11;
pub const E2: Square = 12;
pub const F2: Square = 13;
pub const G2: Square = 14;
pub const H2: Square = 15;

pub const A3: Square = 16;
pub const B3: Square = 17;
pub const C3: Square = 18;
pub const D3: Square = 19;
pub const E3: Square = 20;
pub const F3: Square = 21;
pub const G3: Square = 22;
pub const H3: Square = 23;

pub const A4: Square = 24;
pub const B4: Square = 25;
pub const C4: Square = 26;
pub const D4: Square = 27;
pub const E4: Square = 28;
pub const F4: Square = 29;
pub const G4: Square = 30;
pub const H4: Square = 31;

pub const A5: Square = 32;
pub const B5: Square = 33;
pub const C5: Square = 34;
pub const D5: Square = 35;
pub const E5: Square = 36;
pub const F5: Square = 37;
pub const G5: Square = 38;
pub const H5: Square = 39;

pub const A6: Square = 40;
pub const B6: Square = 41;
pub const C6: Square = 42;
pub const D6: Square = 43;
pub const E6: Square = 44;
pub const F6: Square = 45;
pub const G6: Square = 46;
pub const H6: Square = 47;

pub const A7: Square = 48;
pub const B7: Square = 49;
pub const C7: Square = 50;
pub const D7: Square = 51;
pub const E7: Square = 52;
pub const F7: Square = 53;
pub const G7: Square = 54;
pub const H7: Square = 55;

pub const A8: Square = 56;
pub const B8: Square = 57;
pub const C8: Square = 58;
pub const D8: Square = 59;
pub const E8: Square = 60;
pub const F8: Square = 61;
pub const G8: Square = 62;
pub const H8: Square = 63;
