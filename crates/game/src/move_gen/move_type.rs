#![allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Move(u16);

pub mod mv_flags {
    pub const NONE: u16 = 0;
    pub const PROMO: u16 = 1; // promotion
    pub const CAPT: u16 = 2; // capture
    pub const ENPASS: u16 = 4; // en-passant
    pub const CASTLE: u16 = 8; // castling
}

impl Move {
    // bit layout (LSB -> MSB):
    // bits  0..=5   : from  (6 bits)
    // bits  6..=11  : to    (6 bits)
    // bits 12..=15  : flags (4 bits)
    //
    // masks & shifts:
    const FROM_MASK: u16 = 0x003F; // 6 bits
    const TO_MASK: u16 = 0x0FC0; // bits 6..11
    const FLAGS_MASK: u16 = 0xF000; // bits 12..15

    const TO_SHIFT: u16 = 6;
    const FLAGS_SHIFT: u16 = 12;

    /// Create from components (panics on out-of-range values in debug).
    #[inline]
    pub const fn new(from: u16, to: u16, flags: u16) -> Self {
        debug_assert!(from < 64, "from must be 0..63");
        debug_assert!(to < 64, "to must be 0..63");
        debug_assert!(flags < 16, "flags must be 0..15");

        let raw = (from & Self::FROM_MASK)
            | ((to << Self::TO_SHIFT) & Self::TO_MASK)
            | ((flags << Self::FLAGS_SHIFT) & Self::FLAGS_MASK);
        Move(raw)
    }

    /// Pack raw u16 into Move (no checks).
    #[inline]
    pub const fn from_u16(raw: u16) -> Self {
        Move(raw)
    }

    /// Unpack to raw u16.
    #[inline]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn from_sq(self) -> u16 {
        self.0 & Self::FROM_MASK
    }

    #[inline]
    pub const fn to_sq(self) -> u16 {
        (self.0 & Self::TO_MASK) >> Self::TO_SHIFT
    }

    #[inline]
    pub const fn flags(self) -> u16 {
        (self.0 & Self::FLAGS_MASK) >> Self::FLAGS_SHIFT
    }

    /// Replace flags quickly
    #[inline]
    pub const fn with_flags(self, flags: u16) -> Self {
        debug_assert!(flags < 16);
        let raw =
            (self.0 & !(Self::FLAGS_MASK)) | ((flags << Self::FLAGS_SHIFT) & Self::FLAGS_MASK);
        Move(raw)
    }
}
