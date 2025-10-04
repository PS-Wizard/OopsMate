#![allow(dead_code)]

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// White / Black Color Enum, maps to the same order stored in the side, in `Position` struct
///
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn flip(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Piece type, this maps exactly to the order in which the board is stored in the Position struct
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Debug)]
/// Castling rights as bitflags
/// bits: WK, WQ, BK, BQ
pub struct CastleRights(pub u8);

impl CastleRights {
    pub const NONE: Self = CastleRights(0);
    pub const WHITE_KING: Self = CastleRights(1);
    pub const WHITE_QUEEN: Self = CastleRights(2);
    pub const BLACK_KING: Self = CastleRights(4);
    pub const BLACK_QUEEN: Self = CastleRights(8);

    #[inline(always)]
    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.0 & Self::WHITE_KING.0 != 0,
            Color::Black => self.0 & Self::BLACK_KING.0 != 0,
        }
    }

    #[inline(always)]
    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.0 & Self::WHITE_QUEEN.0 != 0,
            Color::Black => self.0 & Self::BLACK_QUEEN.0 != 0,
        }
    }

    #[inline(always)]
    pub fn remove_white_castling(&mut self) {
        self.0 &= !(Self::WHITE_KING.0 | Self::WHITE_QUEEN.0);
    }

    #[inline(always)]
    pub fn remove_black_castling(&mut self) {
        self.0 &= !(Self::BLACK_KING.0 | Self::BLACK_QUEEN.0);
    }

    #[inline(always)]
    pub fn remove_white_kingside(&mut self) {
        self.0 &= !Self::WHITE_KING.0;
    }

    #[inline(always)]
    pub fn remove_white_queenside(&mut self) {
        self.0 &= !Self::WHITE_QUEEN.0;
    }

    #[inline(always)]
    pub fn remove_black_kingside(&mut self) {
        self.0 &= !Self::BLACK_KING.0;
    }

    #[inline(always)]
    pub fn remove_black_queenside(&mut self) {
        self.0 &= !Self::BLACK_QUEEN.0;
    }
}
