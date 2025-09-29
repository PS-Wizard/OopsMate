// Cargo Workspace Structure:
//
// Cargo.toml (workspace root)
// ├── types/           (foundational types)
// ├── attacks/         (attack generation)
// ├── board/           (position representation)
// ├── movegen/         (move generation)
// ├── search/          (search & eval)
// ├── uci/             (UCI protocol)
// └── engine/          (main binary)

// ============================================
// ROOT Cargo.toml
// ============================================
/*
[workspace]
members = [
    "types",
    "attacks", 
    "board",
    "movegen",
    "search",
    "uci",
    "engine"
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
# Shared dependencies
*/

// ============================================
// types/src/lib.rs - Foundation types
// ============================================

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Bitboard(0);
    pub const ALL: Self = Bitboard(u64::MAX);
    
    #[inline(always)]
    pub fn contains(self, sq: Square) -> bool {
        (self.0 & (1u64 << sq.0)) != 0
    }
    
    #[inline(always)]
    pub fn set(&mut self, sq: Square) {
        self.0 |= 1u64 << sq.0;
    }
    
    #[inline(always)]
    pub fn clear(&mut self, sq: Square) {
        self.0 &= !(1u64 << sq.0);
    }
    
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Square {
        let sq = Square(self.0.trailing_zeros() as u8);
        self.0 &= self.0 - 1;
        sq
    }
    
    #[inline(always)]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self { Bitboard(self.0 & rhs.0) }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { Bitboard(self.0 | rhs.0) }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self { Bitboard(self.0 ^ rhs.0) }
}

impl std::ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self { Bitboard(!self.0) }
}

// ============================================

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Square(pub u8);  // 0-63

impl Square {
    pub const A1: Square = Square(0);
    pub const H8: Square = Square(63);
    // ... define all squares
    
    #[inline(always)]
    pub fn new(rank: u8, file: u8) -> Self {
        Square(rank * 8 + file)
    }
    
    #[inline(always)]
    pub fn rank(self) -> u8 { self.0 / 8 }
    
    #[inline(always)]
    pub fn file(self) -> u8 { self.0 % 8 }
    
    #[inline(always)]
    pub fn flip(self) -> Self {
        Square(self.0 ^ 56)  // Flip vertically
    }
}

// ============================================

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

pub const PIECE_COUNT: usize = 6;

// ============================================

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    pub fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub const COLOR_COUNT: usize = 2;

// ============================================

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CastleRights(pub u8);

impl CastleRights {
    pub const NONE: Self = CastleRights(0);
    pub const WHITE_KING: Self = CastleRights(1);
    pub const WHITE_QUEEN: Self = CastleRights(2);
    pub const BLACK_KING: Self = CastleRights(4);
    pub const BLACK_QUEEN: Self = CastleRights(8);
    pub const WHITE_BOTH: Self = CastleRights(3);
    pub const BLACK_BOTH: Self = CastleRights(12);
    pub const ALL: Self = CastleRights(15);
    
    #[inline(always)]
    pub fn has(self, right: Self) -> bool {
        (self.0 & right.0) != 0
    }
    
    #[inline(always)]
    pub fn remove(&mut self, right: Self) {
        self.0 &= !right.0;
    }
}

// ============================================

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Move {
    data: u16,
    // bits 0-5:   from square
    // bits 6-11:  to square
    // bits 12-15: flags (promotion piece, capture, castle, en passant)
}

impl Move {
    #[inline(always)]
    pub fn new(from: Square, to: Square, flags: u8) -> Self {
        Move {
            data: (from.0 as u16) | ((to.0 as u16) << 6) | ((flags as u16) << 12)
        }
    }
    
    #[inline(always)]
    pub fn from(self) -> Square {
        Square((self.data & 0x3F) as u8)
    }
    
    #[inline(always)]
    pub fn to(self) -> Square {
        Square(((self.data >> 6) & 0x3F) as u8)
    }
    
    #[inline(always)]
    pub fn flags(self) -> u8 {
        (self.data >> 12) as u8
    }
}

// Move flags
pub const FLAG_NONE: u8 = 0;
pub const FLAG_CAPTURE: u8 = 1;
pub const FLAG_DOUBLE_PUSH: u8 = 2;
pub const FLAG_EN_PASSANT: u8 = 3;
pub const FLAG_CASTLE: u8 = 4;
pub const FLAG_PROMOTION_KNIGHT: u8 = 8;
pub const FLAG_PROMOTION_BISHOP: u8 = 9;
pub const FLAG_PROMOTION_ROOK: u8 = 10;
pub const FLAG_PROMOTION_QUEEN: u8 = 11;

// ============================================
// board/src/lib.rs - Position representation
// ============================================

use types::*;

#[derive(Copy, Clone)]
pub struct Position {
    // Bitboard representation
    pub pieces: [Bitboard; PIECE_COUNT],  // Index by Piece enum
    pub colors: [Bitboard; COLOR_COUNT],   // Index by Color enum
    
    // CRITICAL: Mailbox for O(1) "what piece is here?" lookups
    pub mailbox: [Option<(Piece, Color)>; 64],
    
    // Game state
    pub side_to_move: Color,
    pub castle_rights: CastleRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    
    // Zobrist hash
    pub hash: u64,
}

#[derive(Copy, Clone)]
pub struct UndoInfo {
    pub captured: Option<(Piece, Color)>,
    pub castle_rights: CastleRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub hash: u64,
}

impl Position {
    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        self.mailbox[sq.0 as usize]
    }
    
    #[inline(always)]
    pub fn occupied(&self) -> Bitboard {
        self.colors[Color::White as usize] | self.colors[Color::Black as usize]
    }
    
    #[inline(always)]
    pub fn us(&self) -> Bitboard {
        self.colors[self.side_to_move as usize]
    }
    
    #[inline(always)]
    pub fn them(&self) -> Bitboard {
        self.colors[self.side_to_move.flip() as usize]
    }
    
    #[inline(always)]
    pub fn our_pieces(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize] & self.us()
    }
    
    #[inline(always)]
    pub fn their_pieces(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize] & self.them()
    }
    
    #[inline(always)]
    fn add_piece(&mut self, sq: Square, piece: Piece, color: Color) {
        self.pieces[piece as usize].set(sq);
        self.colors[color as usize].set(sq);
        self.mailbox[sq.0 as usize] = Some((piece, color));
    }
    
    #[inline(always)]
    fn remove_piece(&mut self, sq: Square, piece: Piece, color: Color) {
        self.pieces[piece as usize].clear(sq);
        self.colors[color as usize].clear(sq);
        self.mailbox[sq.0 as usize] = None;
    }
    
    pub fn make_move(&mut self, m: Move) -> UndoInfo {
        let from = m.from();
        let to = m.to();
        let flags = m.flags();
        
        // Save state for unmake
        let undo = UndoInfo {
            captured: self.piece_at(to),
            castle_rights: self.castle_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };
        
        let (piece, color) = self.piece_at(from).unwrap();
        
        // Remove piece from 'from' square
        self.remove_piece(from, piece, color);
        // XOR out old position from hash
        self.hash ^= zobrist_piece(piece, color, from);
        
        // Handle capture
        if let Some((cap_piece, cap_color)) = undo.captured {
            self.remove_piece(to, cap_piece, cap_color);
            self.hash ^= zobrist_piece(cap_piece, cap_color, to);
        }
        
        // Place piece on 'to' square (might be promotion)
        let final_piece = if flags >= FLAG_PROMOTION_KNIGHT {
            // It's a promotion
            match flags {
                FLAG_PROMOTION_KNIGHT => Piece::Knight,
                FLAG_PROMOTION_BISHOP => Piece::Bishop,
                FLAG_PROMOTION_ROOK => Piece::Rook,
                FLAG_PROMOTION_QUEEN => Piece::Queen,
                _ => piece,
            }
        } else {
            piece
        };
        
        self.add_piece(to, final_piece, color);
        self.hash ^= zobrist_piece(final_piece, color, to);
        
        // Handle special moves (castling, en passant, etc.)
        // ... update castle rights, en passant square, etc.
        // ... XOR hash appropriately
        
        // Switch sides
        self.side_to_move = self.side_to_move.flip();
        self.hash ^= zobrist_side();
        
        undo
    }
    
    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        // This is the MIRROR of make_move
        // Restore everything from UndoInfo
        
        self.side_to_move = self.side_to_move.flip();
        self.castle_rights = undo.castle_rights;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.hash = undo.hash;
        
        // ... move piece back, restore captured piece, etc.
    }
    
    pub fn is_in_check(&self, color: Color) -> bool {
        let king_sq = (self.pieces[Piece::King as usize] & 
                       self.colors[color as usize]).pop_lsb();
        self.is_square_attacked(king_sq, color.flip())
    }
    
    pub fn is_square_attacked(&self, sq: Square, by_color: Color) -> bool {
        // Use your PEXT attack tables here
        // Check if any enemy piece attacks this square
        todo!()
    }
}

// Zobrist hash functions (implement in board/zobrist.rs)
fn zobrist_piece(piece: Piece, color: Color, sq: Square) -> u64 { todo!() }
fn zobrist_side() -> u64 { todo!() }

// ============================================
// Summary of key points:
// ============================================
// 1. mailbox[64] solves your O(1) lookup problem
// 2. pieces[6] + colors[2] is faster than pieces[12] for "all my pieces" queries
// 3. make_move updates mailbox, bitboards, AND hash together
// 4. UndoInfo makes unmake_move O(1) - no searching needed
// 5. Everything is Copy so you can do fast position copies for legality testing
