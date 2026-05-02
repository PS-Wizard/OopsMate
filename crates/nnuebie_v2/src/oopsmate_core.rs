use oops_mate as host;
use std::ops::{Deref, DerefMut};

pub type Bitboard = u64;
pub const EMPTY_SQUARE: u8 = 0;
pub const MAX_POSITION_HISTORY: usize = 1024;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

impl From<host::Color> for Color {
    #[inline(always)]
    fn from(value: host::Color) -> Self {
        match value {
            host::Color::White => Self::White,
            host::Color::Black => Self::Black,
        }
    }
}

impl From<Color> for host::Color {
    #[inline(always)]
    fn from(value: Color) -> Self {
        match value {
            Color::White => Self::White,
            Color::Black => Self::Black,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Piece {
    #[default]
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    #[inline(always)]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Pawn),
            1 => Some(Self::Knight),
            2 => Some(Self::Bishop),
            3 => Some(Self::Rook),
            4 => Some(Self::Queen),
            5 => Some(Self::King),
            _ => None,
        }
    }
}

impl From<host::Piece> for Piece {
    #[inline(always)]
    fn from(value: host::Piece) -> Self {
        match value {
            host::Piece::Pawn => Self::Pawn,
            host::Piece::Knight => Self::Knight,
            host::Piece::Bishop => Self::Bishop,
            host::Piece::Rook => Self::Rook,
            host::Piece::Queen => Self::Queen,
            host::Piece::King => Self::King,
        }
    }
}

impl From<Piece> for host::Piece {
    #[inline(always)]
    fn from(value: Piece) -> Self {
        match value {
            Piece::Pawn => Self::Pawn,
            Piece::Knight => Self::Knight,
            Piece::Bishop => Self::Bishop,
            Piece::Rook => Self::Rook,
            Piece::Queen => Self::Queen,
            Piece::King => Self::King,
        }
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

impl From<host::MoveType> for MoveKind {
    #[inline(always)]
    fn from(value: host::MoveType) -> Self {
        match value {
            host::MoveType::Quiet => Self::Quiet,
            host::MoveType::DoublePush => Self::DoublePush,
            host::MoveType::Castle => Self::Castle,
            host::MoveType::EnPassant => Self::EnPassant,
            host::MoveType::Capture => Self::Capture,
            host::MoveType::PromotionKnight => Self::PromotionKnight,
            host::MoveType::PromotionBishop => Self::PromotionBishop,
            host::MoveType::PromotionRook => Self::PromotionRook,
            host::MoveType::PromotionQueen => Self::PromotionQueen,
            host::MoveType::CapturePromotionKnight => Self::CapturePromotionKnight,
            host::MoveType::CapturePromotionBishop => Self::CapturePromotionBishop,
            host::MoveType::CapturePromotionRook => Self::CapturePromotionRook,
            host::MoveType::CapturePromotionQueen => Self::CapturePromotionQueen,
        }
    }
}

impl From<MoveKind> for host::MoveType {
    #[inline(always)]
    fn from(value: MoveKind) -> Self {
        match value {
            MoveKind::Quiet => Self::Quiet,
            MoveKind::DoublePush => Self::DoublePush,
            MoveKind::Castle => Self::Castle,
            MoveKind::EnPassant => Self::EnPassant,
            MoveKind::Capture => Self::Capture,
            MoveKind::PromotionKnight => Self::PromotionKnight,
            MoveKind::PromotionBishop => Self::PromotionBishop,
            MoveKind::PromotionRook => Self::PromotionRook,
            MoveKind::PromotionQueen => Self::PromotionQueen,
            MoveKind::CapturePromotionKnight => Self::CapturePromotionKnight,
            MoveKind::CapturePromotionBishop => Self::CapturePromotionBishop,
            MoveKind::CapturePromotionRook => Self::CapturePromotionRook,
            MoveKind::CapturePromotionQueen => Self::CapturePromotionQueen,
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

impl From<host::Move> for Move {
    #[inline(always)]
    fn from(value: host::Move) -> Self {
        Self(value.0)
    }
}

impl From<Move> for host::Move {
    #[inline(always)]
    fn from(value: Move) -> Self {
        Self(value.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Board<'a> {
    position: &'a host::Position,
}

impl<'a> Board<'a> {
    #[inline(always)]
    #[must_use]
    pub const fn piece_bb(&self, piece: Piece) -> Bitboard {
        self.position.pieces[piece.index()].0
    }

    #[inline(always)]
    #[must_use]
    pub const fn color_bb(&self, color: Color) -> Bitboard {
        self.position.colors[color.index()].0
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
            Some((piece, color)) => encode_piece(piece.into(), color.into()),
            None => EMPTY_SQUARE,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.position.board[square.index()].map(|(piece, color)| (piece.into(), color.into()))
    }

    #[inline(always)]
    #[must_use]
    pub const fn king_square(&self, color: Color) -> Square {
        Square::from_raw(self.position.king_sq[color.index()])
    }
}

#[derive(Clone, Debug)]
pub struct Position(host::Position);

impl Position {
    #[inline(always)]
    #[must_use]
    pub fn startpos() -> Self {
        Self(host::Position::new())
    }

    #[inline(always)]
    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        host::Position::from_fen(fen).map(Self)
    }

    #[inline(always)]
    #[must_use]
    pub const fn board(&self) -> Board<'_> {
        Board { position: &self.0 }
    }

    #[inline(always)]
    #[must_use]
    pub fn side_to_move(&self) -> Color {
        self.0.side_to_move.into()
    }

    #[inline(always)]
    #[must_use]
    pub const fn rule50(&self) -> u16 {
        self.0.halfmove
    }

    #[inline(always)]
    #[must_use]
    pub const fn fullmove(&self) -> u16 {
        self.0.fullmove
    }

    #[inline(always)]
    #[must_use]
    pub const fn hash(&self) -> u64 {
        self.0.hash
    }

    #[inline(always)]
    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.0
            .piece_at(square.index())
            .map(|(piece, color)| (piece.into(), color.into()))
    }

    #[inline(always)]
    pub fn make_move(&mut self, mv: Move) {
        self.0.make_move(mv.into());
    }

    #[inline(always)]
    pub fn unmake_move(&mut self, mv: Move) {
        self.0.unmake_move(mv.into());
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_host(&self) -> &host::Position {
        &self.0
    }

    #[inline(always)]
    #[must_use]
    pub fn as_host_mut(&mut self) -> &mut host::Position {
        &mut self.0
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::startpos()
    }
}

impl Deref for Position {
    type Target = host::Position;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Position {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

#[inline(always)]
#[must_use]
pub const fn decode_piece(code: u8) -> Option<(Piece, Color)> {
    if code == EMPTY_SQUARE {
        None
    } else {
        Some((piece_from_code(code), color_from_code(code)))
    }
}
