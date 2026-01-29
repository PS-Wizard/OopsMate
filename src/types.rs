use std::mem::MaybeUninit;

// ============================================================================
// BITBOARD
// ============================================================================

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub const fn new() -> Self {
        Bitboard(0)
    }

    #[inline(always)]
    pub const fn from_raw(value: u64) -> Self {
        Bitboard(value)
    }

    #[inline(always)]
    pub fn set(&mut self, idx: usize) {
        self.0 |= 1 << idx;
    }

    #[inline(always)]
    pub fn clear(&mut self, idx: usize) {
        self.0 &= !(1 << idx);
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
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

// ============================================================================
// COLOR & PIECE
// ============================================================================

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    pub const fn flip(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

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

// ============================================================================
// CASTLING RIGHTS
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CastleRights(pub u8);

impl CastleRights {
    pub const NONE: Self = CastleRights(0);
    const WHITE_KING: u8 = 1;
    const WHITE_QUEEN: u8 = 2;
    const BLACK_KING: u8 = 4;
    const BLACK_QUEEN: u8 = 8;

    #[inline(always)]
    pub const fn can_castle_kingside(self, color: Color) -> bool {
        match color {
            Color::White => self.0 & Self::WHITE_KING != 0,
            Color::Black => self.0 & Self::BLACK_KING != 0,
        }
    }

    #[inline(always)]
    pub const fn can_castle_queenside(self, color: Color) -> bool {
        match color {
            Color::White => self.0 & Self::WHITE_QUEEN != 0,
            Color::Black => self.0 & Self::BLACK_QUEEN != 0,
        }
    }

    #[inline(always)]
    pub fn remove_color(&mut self, color: Color) {
        match color {
            Color::White => self.0 &= !(Self::WHITE_KING | Self::WHITE_QUEEN),
            Color::Black => self.0 &= !(Self::BLACK_KING | Self::BLACK_QUEEN),
        }
    }

    #[inline(always)]
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

// ============================================================================
// MOVE
// ============================================================================

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Move(pub u16);

impl Move {
    #[inline(always)]
    pub const fn new(from: usize, to: usize, move_type: MoveType) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((move_type as u16) << 12))
    }

    #[inline(always)]
    pub const fn from(self) -> usize {
        (self.0 & 0x3F) as usize
    }

    #[inline(always)]
    pub const fn to(self) -> usize {
        ((self.0 >> 6) & 0x3F) as usize
    }

    #[inline(always)]
    pub const fn move_type(self) -> MoveType {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }

    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self.0 >> 12) & 0x4 != 0
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.0 >> 12) & 0x8 != 0
    }
}

// ============================================================================
// MOVE COLLECTOR
// ============================================================================

pub struct MoveCollector {
    moves: [MaybeUninit<Move>; 256],
    count: usize,
}

impl MoveCollector {
    #[inline(always)]
    pub fn new() -> Self {
        MoveCollector {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            count: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        unsafe {
            self.moves.get_unchecked_mut(self.count).write(m);
        }
        self.count += 1;
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.count = 0;
    }

    #[inline(always)]
    pub fn get(&self, index: usize) -> Move {
        debug_assert!(index < self.count);
        unsafe { self.moves.get_unchecked(index).assume_init() }
    }
}
