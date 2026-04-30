pub use crate::{Color, Piece, Position};
use crate::MoveType;

pub type Bitboard = u64;
pub const EMPTY_SQUARE: u8 = 0;
pub const MAX_POSITION_HISTORY: usize = 1024;

pub trait ColorExt {
    fn index(self) -> usize;
}

impl ColorExt for Color {
    #[inline(always)]
    fn index(self) -> usize {
        self as usize
    }
}

pub trait PieceExt {
    fn index(self) -> usize;
}

impl PieceExt for Piece {
    #[inline(always)]
    fn index(self) -> usize {
        self as usize
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Square(u8);

impl Square {
    pub const NONE: Self = Self(64);

    #[inline(always)]
    #[must_use]
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    #[inline(always)]
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[inline(always)]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 < 64
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 >= 64
    }

    #[inline(always)]
    #[must_use]
    pub fn bit(self) -> Bitboard {
        debug_assert!(self.is_valid());
        1u64 << self.0
    }

    #[inline(always)]
    #[must_use]
    pub const fn file(self) -> u8 {
        self.0 & 7
    }

    #[inline(always)]
    #[must_use]
    pub const fn rank(self) -> u8 {
        self.0 >> 3
    }

    #[inline(always)]
    #[must_use]
    pub const fn from_file_rank(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self(rank * 8 + file))
        } else {
            None
        }
    }

    #[must_use]
    pub fn from_algebraic(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 2 {
            return None;
        }

        let file = match bytes[0] {
            b'a'..=b'h' => bytes[0] - b'a',
            b'A'..=b'H' => bytes[0] - b'A',
            _ => return None,
        };

        let rank = match bytes[1] {
            b'1'..=b'8' => bytes[1] - b'1',
            _ => return None,
        };

        Self::from_file_rank(file, rank)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MoveKind {
    #[default]
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

impl MoveKind {
    #[inline(always)]
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Quiet,
            1 => Self::DoublePush,
            2 => Self::Castle,
            3 => Self::EnPassant,
            4 => Self::Capture,
            8 => Self::PromotionKnight,
            9 => Self::PromotionBishop,
            10 => Self::PromotionRook,
            11 => Self::PromotionQueen,
            12 => Self::CapturePromotionKnight,
            13 => Self::CapturePromotionBishop,
            14 => Self::CapturePromotionRook,
            15 => Self::CapturePromotionQueen,
            _ => panic!("invalid move kind"),
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_capture(self) -> bool {
        (self as u8 & 0x4) != 0
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_promotion(self) -> bool {
        (self as u8 & 0x8) != 0
    }

    #[inline(always)]
    #[must_use]
    pub const fn promotion_piece(self) -> Option<Piece> {
        match self {
            Self::PromotionKnight | Self::CapturePromotionKnight => Some(Piece::Knight),
            Self::PromotionBishop | Self::CapturePromotionBishop => Some(Piece::Bishop),
            Self::PromotionRook | Self::CapturePromotionRook => Some(Piece::Rook),
            Self::PromotionQueen | Self::CapturePromotionQueen => Some(Piece::Queen),
            _ => None,
        }
    }
}

impl From<MoveType> for MoveKind {
    #[inline(always)]
    fn from(value: MoveType) -> Self {
        match value {
            MoveType::Quiet => Self::Quiet,
            MoveType::DoublePush => Self::DoublePush,
            MoveType::Castle => Self::Castle,
            MoveType::EnPassant => Self::EnPassant,
            MoveType::Capture => Self::Capture,
            MoveType::PromotionKnight => Self::PromotionKnight,
            MoveType::PromotionBishop => Self::PromotionBishop,
            MoveType::PromotionRook => Self::PromotionRook,
            MoveType::PromotionQueen => Self::PromotionQueen,
            MoveType::CapturePromotionKnight => Self::CapturePromotionKnight,
            MoveType::CapturePromotionBishop => Self::CapturePromotionBishop,
            MoveType::CapturePromotionRook => Self::CapturePromotionRook,
            MoveType::CapturePromotionQueen => Self::CapturePromotionQueen,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Move(pub u16);

impl Move {
    pub const NULL: Self = Self(0);

    #[inline(always)]
    #[must_use]
    pub const fn new(from: Square, to: Square, kind: MoveKind) -> Self {
        Self((from.raw() as u16) | ((to.raw() as u16) << 6) | ((kind as u16) << 12))
    }

    #[inline(always)]
    #[must_use]
    pub const fn from(self) -> Square {
        Square::from_raw((self.0 & 0x3f) as u8)
    }

    #[inline(always)]
    #[must_use]
    pub const fn to(self) -> Square {
        Square::from_raw(((self.0 >> 6) & 0x3f) as u8)
    }

    #[inline(always)]
    #[must_use]
    pub const fn kind(self) -> MoveKind {
        MoveKind::from_u8((self.0 >> 12) as u8)
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_capture(self) -> bool {
        ((self.0 >> 12) & 0x4) != 0
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_promotion(self) -> bool {
        ((self.0 >> 12) & 0x8) != 0
    }
}

impl From<crate::Move> for Move {
    #[inline(always)]
    fn from(value: crate::Move) -> Self {
        Self(value.0)
    }
}

impl From<Move> for crate::Move {
    #[inline(always)]
    fn from(value: Move) -> Self {
        Self(value.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Board<'a> {
    position: &'a Position,
}

impl<'a> Board<'a> {
    #[inline(always)]
    #[must_use]
    pub const fn piece_bb(&self, piece: Piece) -> Bitboard {
        self.position.pieces[piece as usize].0
    }

    #[inline(always)]
    #[must_use]
    pub const fn color_bb(&self, color: Color) -> Bitboard {
        self.position.colors[color as usize].0
    }

    #[inline(always)]
    #[must_use]
    pub const fn occupied(&self) -> Bitboard {
        self.position.colors[0].0 | self.position.colors[1].0
    }

    #[inline(always)]
    #[must_use]
    pub fn piece_code_at(&self, square: Square) -> u8 {
        debug_assert!(square.is_valid());
        match self.position.board[square.index()] {
            Some((piece, color)) => encode_piece(piece, color),
            None => EMPTY_SQUARE,
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn king_square(&self, color: Color) -> Square {
        Square::from_raw(self.position.king_sq[color as usize])
    }
}

pub trait PositionExt {
    fn board(&self) -> Board<'_>;
    fn side_to_move(&self) -> Color;
    fn rule50(&self) -> u16;
}

impl PositionExt for Position {
    #[inline(always)]
    fn board(&self) -> Board<'_> {
        Board { position: self }
    }

    #[inline(always)]
    fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    fn rule50(&self) -> u16 {
        self.halfmove
    }

}

#[inline(always)]
#[must_use]
pub const fn encode_piece(piece: Piece, color: Color) -> u8 {
    1 + piece as u8 + (color as u8 * 6)
}

#[inline(always)]
#[must_use]
pub const fn piece_from_code(code: u8) -> Piece {
    match code {
        1 | 7 => Piece::Pawn,
        2 | 8 => Piece::Knight,
        3 | 9 => Piece::Bishop,
        4 | 10 => Piece::Rook,
        5 | 11 => Piece::Queen,
        6 | 12 => Piece::King,
        _ => panic!("invalid piece code"),
    }
}

#[inline(always)]
#[must_use]
pub const fn color_from_code(code: u8) -> Color {
    match code {
        1..=6 => Color::White,
        7..=12 => Color::Black,
        _ => panic!("invalid piece code"),
    }
}

