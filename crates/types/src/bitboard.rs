
use std::ops::{BitAnd, BitOr, BitOrAssign};

/// A Tuple struct, just a wrapper around a u64, it is a transparently represented, just incase
/// We have to do FFI
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bitboard(pub u64);
impl Bitboard {
    /// Returns a new empty Bitboard
    pub fn new() -> Self {
        Bitboard(0)
    }

    /// Set bitboard from a raw u64
    pub fn from_raw(value: u64) -> Self {
        Bitboard(value)
    }

    /// Set Bit At A Specific Index
    pub fn set_bit(&mut self, idx: usize) {
        self.0 |= 1 << idx;
    }

    /// Removes Bit From A Specific Position
    pub fn remove_bit(&mut self, idx: usize) {
        self.0 &= !(1 << idx);
    }
}

/// Support OR-ing this custom Bitboard type
impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

/// Support shorthand OR-ing and assigning this custom Bitboard type
impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Support AND-ing this custom Bitboard type
impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}
