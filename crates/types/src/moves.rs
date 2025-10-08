use crate::others::Piece;
use std::mem::MaybeUninit;

/// A struct holding a fixed array of side 256 (max legal moves in any position) to avoid
/// allocating memory when generating moves and a count usize that represents the number of moves
/// in the array. Elements of the array are `MaybeUninit`  for better performance
pub struct MoveCollector {
    pub moves: [MaybeUninit<Move>; 256],
    count: usize,
}

impl MoveCollector {
    /// Returns a New Movecollector
    pub fn new() -> Self {
        MoveCollector {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            count: 0,
        }
    }

    #[inline(always)]
    /// Pushes a Move type into the array
    pub fn push(&mut self, m: Move) {
        self.moves[self.count].write(m);
        self.count += 1;
    }

    #[inline(always)]
    /// Returns the number of moves in the arrray
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    /// Returns true if there are no moves in the array
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    /// Clears the array, by setting the count to 0 so the next pass of generating moves overwrites
    /// the previous values
    pub fn clear(&mut self) {
        self.count = 0;
    }
}

/// Utility to directly index the MoveCollector's array
impl std::ops::Index<usize> for MoveCollector {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.count);
        unsafe { self.moves[index].assume_init_ref() }
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
