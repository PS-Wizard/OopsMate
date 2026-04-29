/// Bitboard wrapper used throughout the engine.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    /// Returns an empty bitboard.
    pub const fn new() -> Self {
        Bitboard(0)
    }

    #[inline(always)]
    /// Wraps a raw `u64` bit pattern.
    pub const fn from_raw(value: u64) -> Self {
        Bitboard(value)
    }

    #[inline(always)]
    /// Sets the bit for `idx`.
    pub fn set(&mut self, idx: usize) {
        self.0 |= 1 << idx;
    }

    #[inline(always)]
    /// Clears the bit for `idx`.
    pub fn clear(&mut self, idx: usize) {
        self.0 &= !(1 << idx);
    }

    #[inline(always)]
    /// Returns `true` when no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Default for Bitboard {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}
