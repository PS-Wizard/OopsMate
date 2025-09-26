#![allow(dead_code)]
use crate::{
    board::Board,
    piece::{Piece, PieceKind},
};
pub struct Game {
    /// Bitboards for each piece type (6 white + 6 black pieces), they map exactly in the order of
    /// the PieceKind
    pub boards: [Board; 12],
    pub white_occupied: u64,
    pub black_occupied: u64,

    /// Quick lookup array for piece at each square
    pub piece_map: [PieceKind; 64],

    /// 0 = white, 1 = black
    pub turn: u8,

    /// Bits: KQkq (white king/queen side, black king/queen side)
    pub castling_rights: u8,

    /// En passant target square (prolly finna be 0 if none)
    pub en_passant: u8,

    /// Halfmove clock for 50-move rule
    pub halfmove_clock: u16,

    /// Fullmove number
    pub fullmove: u16,
}
impl Game {
    /// Sets up a new game with the initial position
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    /// Sets up a new game with the provided FEN
    pub fn from_position(fen: &str) -> Self {
        Self::from_fen(fen)
    }

    /// Takes in a u8 square, returns the corresponding Piece at that square
    #[inline(always)]
    pub fn piece_at(&self, square: usize) -> PieceKind {
        self.piece_map[square]
    }

    /// Flips the turn
    #[inline(always)]
    pub fn flip_turn(&mut self) {
        self.turn ^= 1;
    }

    /// Returns The Friendly Board For A Given `Piece`
    #[inline(always)]
    pub fn friendly_board(&self, piece: Piece) -> u64 {
        self.boards[self.turn as usize * 6 + piece as usize].0
    }

    /// Returns The Enemy Board For A Given `Piece`
    #[inline(always)]
    pub fn enemy_board(&self, piece: Piece) -> u64 {
        self.boards[(1 - self.turn) as usize * 6 + piece as usize].0
    }

    /// Returns All Friendly Pieces
    #[inline(always)]
    pub fn get_all_friendlies(&self) -> u64 {
        if self.turn == 0 {
            self.white_occupied
        } else {
            self.black_occupied
        }
    }

    /// Returns All Enemy Pieces
    #[inline(always)]
    pub fn get_all_enemies(&self) -> u64 {
        if self.turn == 0 {
            self.black_occupied
        } else {
            self.white_occupied
        }
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, board: PieceKind, idx: usize) {
        self.boards[board as usize].remove_bit(idx);
        self.piece_map[idx] = PieceKind::None;

        let bit = 1u64 << idx;
        if (board as usize) < 6 {
            self.white_occupied &= !bit;  
        } else {
            self.black_occupied &= !bit; 
        }
    }

    #[inline(always)]
    pub fn add_piece(&mut self, board: PieceKind, idx: usize) {
        self.boards[board as usize].set_bit(idx as u8);
        self.piece_map[idx] = board;

        let bit = 1u64 << idx;
        if (board as usize) < 6 {
            self.white_occupied |= bit;   // Add to white
        } else {
            self.black_occupied |= bit;   // Add to black
        }
    }

    #[inline(always)]
    pub fn move_piece(&mut self, board: PieceKind, from: usize, to: usize) {
        self.remove_piece(board,from);
        self.add_piece(board,to);
    }

}
