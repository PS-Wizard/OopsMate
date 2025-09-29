#![allow(dead_code)]

use std::{ops::{BitAnd, BitOr, BitOrAssign}};

/// A Tuple struct, just a wrapper around a u64, it is a transparently represented, just incase
/// i have to do FFI with some other language.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Bitboard(pub u64);
impl Bitboard {
    /// Returns a new empty Bitboard
    pub fn new() -> Self {
        Bitboard(0)
    }

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

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

