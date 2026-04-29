use super::Color;

/// Castling-right bitfield.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CastleRights(pub u8);

impl CastleRights {
    /// No castling rights for either side.
    pub const NONE: Self = CastleRights(0);
    const WHITE_KING: u8 = 1;
    const WHITE_QUEEN: u8 = 2;
    const BLACK_KING: u8 = 4;
    const BLACK_QUEEN: u8 = 8;

    #[inline(always)]
    /// Returns `true` if `color` can castle kingside.
    pub const fn can_castle_kingside(self, color: Color) -> bool {
        match color {
            Color::White => self.0 & Self::WHITE_KING != 0,
            Color::Black => self.0 & Self::BLACK_KING != 0,
        }
    }

    #[inline(always)]
    /// Returns `true` if `color` can castle queenside.
    pub const fn can_castle_queenside(self, color: Color) -> bool {
        match color {
            Color::White => self.0 & Self::WHITE_QUEEN != 0,
            Color::Black => self.0 & Self::BLACK_QUEEN != 0,
        }
    }

    #[inline(always)]
    /// Removes both castling rights for `color`.
    pub fn remove_color(&mut self, color: Color) {
        match color {
            Color::White => self.0 &= !(Self::WHITE_KING | Self::WHITE_QUEEN),
            Color::Black => self.0 &= !(Self::BLACK_KING | Self::BLACK_QUEEN),
        }
    }

    #[inline(always)]
    /// Removes castling rights affected by a rook move or capture on `sq`.
    pub fn remove_rook(&mut self, sq: usize) {
        match sq {
            0 => self.0 &= !Self::WHITE_QUEEN,
            7 => self.0 &= !Self::WHITE_KING,
            56 => self.0 &= !Self::BLACK_QUEEN,
            63 => self.0 &= !Self::BLACK_KING,
            _ => {}
        }
    }
}
