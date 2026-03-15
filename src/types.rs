//! Shared engine data types.
//!
//! These types are intentionally compact and copy-friendly because they appear
//! throughout move generation, evaluation, hashing, and search.

use std::mem::MaybeUninit;

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

/// Side to move or piece color.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    /// Returns the opposite color.
    pub const fn flip(self) -> Color {
        unsafe { std::mem::transmute(self as u8 ^ 1) }
    }
}

/// Chess piece kind.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

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

/// Encoded move classification.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MoveType {
    Quiet = 0,
    DoublePush = 1,
    Castle = 2,
    EnPassant = 3,
    Capture = 4,
    PromotionKnight = 8,
    PromotionBishop = 9,
    PromotionRook = 10,
    PromotionQueen = 11,
    CapturePromotionKnight = 12,
    CapturePromotionBishop = 13,
    CapturePromotionRook = 14,
    CapturePromotionQueen = 15,
}

/// Compact 16-bit move encoding.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Move(pub u16);

impl Move {
    #[inline(always)]
    /// Packs a move from its origin, destination, and type.
    pub const fn new(from: usize, to: usize, move_type: MoveType) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((move_type as u16) << 12))
    }

    #[inline(always)]
    /// Returns the origin square index.
    pub const fn from(self) -> usize {
        (self.0 & 0x3F) as usize
    }

    #[inline(always)]
    /// Returns the destination square index.
    pub const fn to(self) -> usize {
        ((self.0 >> 6) & 0x3F) as usize
    }

    #[inline(always)]
    /// Returns the encoded move type.
    pub const fn move_type(self) -> MoveType {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }

    #[inline(always)]
    /// Returns `true` if the move captures a piece.
    pub const fn is_capture(self) -> bool {
        (self.0 >> 12) & 0x4 != 0
    }

    #[inline(always)]
    /// Returns `true` if the move promotes a pawn.
    pub const fn is_promotion(self) -> bool {
        (self.0 >> 12) & 0x8 != 0
    }

    /// Formats the move in long UCI form such as `e2e4` or `a7a8q`.
    pub fn to_uci(self) -> String {
        let from = self.from();
        let to = self.to();
        let mut uci = String::with_capacity(5);

        uci.push((b'a' + (from % 8) as u8) as char);
        uci.push((b'1' + (from / 8) as u8) as char);
        uci.push((b'a' + (to % 8) as u8) as char);
        uci.push((b'1' + (to / 8) as u8) as char);

        if let Some(promotion) = self.promotion_suffix() {
            uci.push(promotion);
        }

        uci
    }

    const fn promotion_suffix(self) -> Option<char> {
        match self.move_type() {
            MoveType::PromotionQueen | MoveType::CapturePromotionQueen => Some('q'),
            MoveType::PromotionRook | MoveType::CapturePromotionRook => Some('r'),
            MoveType::PromotionBishop | MoveType::CapturePromotionBishop => Some('b'),
            MoveType::PromotionKnight | MoveType::CapturePromotionKnight => Some('n'),
            _ => None,
        }
    }
}

/// Fixed-capacity move buffer used by move generation.
#[derive(Clone, Copy)]
pub struct MoveCollector {
    moves: [MaybeUninit<Move>; 256],
    count: usize,
}

impl MoveCollector {
    #[inline(always)]
    /// Creates an empty collector.
    pub fn new() -> Self {
        MoveCollector {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            count: 0,
        }
    }

    #[inline(always)]
    /// Appends a move to the collector.
    pub fn push(&mut self, m: Move) {
        unsafe {
            self.moves.get_unchecked_mut(self.count).write(m);
        }
        self.count += 1;
    }

    #[inline(always)]
    /// Returns the number of collected moves.
    pub const fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    /// Returns `true` when no moves have been collected.
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    /// Resets the collector length to zero.
    pub fn clear(&mut self) {
        self.count = 0;
    }

    #[inline(always)]
    /// Returns the move at `index` without bounds checks in release builds.
    pub fn get(&self, index: usize) -> Move {
        debug_assert!(index < self.count);
        unsafe { self.moves.get_unchecked(index).assume_init() }
    }

    #[inline(always)]
    /// Returns the initialized prefix as a slice.
    pub fn as_slice(&self) -> &[Move] {
        unsafe { std::slice::from_raw_parts(self.moves.as_ptr() as *const Move, self.count) }
    }
}

impl Default for MoveCollector {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
