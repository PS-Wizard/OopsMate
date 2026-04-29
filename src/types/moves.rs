use std::mem::MaybeUninit;

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
