use crate::others::Piece;
use std::mem::MaybeUninit;

/// A struct that separates moves into captures and quiet moves for better move ordering
pub struct MoveCollector {
    pub captures: [MaybeUninit<Move>; 128],
    pub quiets: [MaybeUninit<Move>; 128],
    capture_count: usize,
    quiet_count: usize,
}

impl MoveCollector {
    /// Returns a new MoveCollector
    pub fn new() -> Self {
        MoveCollector {
            captures: unsafe { MaybeUninit::uninit().assume_init() },
            quiets: unsafe { MaybeUninit::uninit().assume_init() },
            capture_count: 0,
            quiet_count: 0,
        }
    }

    #[inline(always)]
    /// Pushes a move into the appropriate array (captures or quiets)
    pub fn push(&mut self, m: Move) {
        if m.is_capture() {
            self.captures[self.capture_count].write(m);
            self.capture_count += 1;
        } else {
            self.quiets[self.quiet_count].write(m);
            self.quiet_count += 1;
        }
    }

    #[inline(always)]
    /// Returns the total number of moves
    pub fn len(&self) -> usize {
        self.capture_count + self.quiet_count
    }

    #[inline(always)]
    /// Returns true if there are no moves
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of capture moves
    #[inline(always)]
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }

    /// Returns the number of quiet moves
    #[inline(always)]
    pub fn quiet_count(&self) -> usize {
        self.quiet_count
    }

    #[inline(always)]
    /// Clears both arrays
    pub fn clear(&mut self) {
        self.capture_count = 0;
        self.quiet_count = 0;
    }

    pub fn contains(&self, m: Move) -> bool {
        (0..self.capture_count).any(|i| unsafe { self.captures[i].assume_init() } == m)
            || (0..self.quiet_count).any(|i| unsafe { self.quiets[i].assume_init() } == m)
    }

    /// Get a move by index (captures first, then quiets)
    #[inline(always)]
    pub fn get(&self, index: usize) -> Move {
        if index < self.capture_count {
            unsafe { self.captures[index].assume_init() }
        } else {
            unsafe { self.quiets[index - self.capture_count].assume_init() }
        }
    }

    /// Iterate through captures first, then quiets
    pub fn iter_ordered(&self) -> impl Iterator<Item = Move> + '_ {
        let captures =
            (0..self.capture_count).map(move |i| unsafe { self.captures[i].assume_init() });
        let quiets = (0..self.quiet_count).map(move |i| unsafe { self.quiets[i].assume_init() });
        captures.chain(quiets)
    }
}

/// Utility to directly index the MoveCollector's array (for backward compat)
impl std::ops::Index<usize> for MoveCollector {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len());
        // SAFETY: We know the move is initialized because we wrote it in push()
        if index < self.capture_count {
            unsafe { self.captures[index].assume_init_ref() }
        } else {
            unsafe { self.quiets[index - self.capture_count].assume_init_ref() }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Move type enum representing the different move types, it is a wrapper around `u8`
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

impl MoveType {
    #[inline(always)]
    /// Check if this move type is a capture
    pub const fn is_capture(self) -> bool {
        (self as u8) & 0x4 != 0
    }

    /// Check if this move type is a promotion
    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self as u8) & 0x8 != 0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Move(pub u16);
impl Move {
    pub const NULL: Move = Move(0);

    #[inline(always)]
    /// Returns a new Move
    pub const fn new(from: usize, to: usize, move_type: MoveType) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((move_type as u16) << 12))
    }

    #[inline(always)]
    /// Takes in a move and pares the from square
    pub const fn from(self) -> usize {
        (self.0 & 0x3F) as usize
    }

    /// Get the to square
    #[inline(always)]
    /// Takes in a move and pares the to square
    pub const fn to(self) -> usize {
        ((self.0 >> 6) & 0x3F) as usize
    }

    #[inline(always)]
    /// Gets the move type
    pub const fn move_type(self) -> MoveType {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }

    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self.0 >> 12) & 0x4 != 0
    }

    /// Check if this is a promotion
    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.0 >> 12) & 0x8 != 0
    }

    #[inline(always)]
    /// checks if the move is a special move, i.e if a move causes 2 pieces to move around
    pub const fn is_special(self) -> bool {
        let mt = (self.0 >> 12) as u8;
        mt == MoveType::EnPassant as u8
            || mt == MoveType::Castle as u8
            || mt == MoveType::DoublePush as u8
    }

    #[inline(always)]
    pub fn promotion_piece(self) -> Option<Piece> {
        match self.move_type() {
            MoveType::PromotionKnight | MoveType::CapturePromotionKnight => Some(Piece::Knight),
            MoveType::PromotionBishop | MoveType::CapturePromotionBishop => Some(Piece::Bishop),
            MoveType::PromotionRook | MoveType::CapturePromotionRook => Some(Piece::Rook),
            MoveType::PromotionQueen | MoveType::CapturePromotionQueen => Some(Piece::Queen),
            _ => None,
        }
    }
}

/// Trait implementation to display the Move type
impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from();
        let to = self.to();
        write!(f, "{}{}", square_to_string(from), square_to_string(to))?;

        if let Some(piece) = self.promotion_piece() {
            let c = match piece {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                _ => unreachable!(),
            };
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

/// Takes in a square and return's its value as algabriac notation
pub fn square_to_string(sq: usize) -> String {
    let file = (b'a' + (sq % 8) as u8) as char;
    let rank = (b'1' + (sq / 8) as u8) as char;
    format!("{}{}", file, rank)
}
