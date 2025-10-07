#![allow(dead_code)]

use types::{
    bitboard::Bitboard,
    others::{CastleRights, Color, Piece},
};

mod fen;
mod legality;
mod move_gen;

#[derive(Clone, Debug)]
pub struct Position {
    pub pieces: [Bitboard; 6],
    pub all_pieces: [Bitboard; 2],
    pub piece_map: [Option<(Piece, Color)>; 64],
    pub side_to_move: Color,
    pub castling_rights: CastleRights,
    pub en_passant: Option<u8>,
    pub half_clock: u8,
    pub full_clock: u16,
    pub hash: u64, // Zobrist Hash Later On
}

impl Position {
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap_or_else(|err| panic!("Failed to parse starting FEN: {}", err))
    }

    #[inline(always)]
    pub fn new_from_fen(fen: &str) -> Self {
        Self::from_fen(fen).unwrap_or_else(|err| panic!("Failed to parse starting FEN: {}", err))
    }

    #[inline(always)]
    pub fn us(&self) -> Bitboard {
        self.all_pieces[self.side_to_move as usize]
    }

    #[inline(always)]
    pub fn them(&self) -> Bitboard {
        self.all_pieces[self.side_to_move.flip() as usize]
    }

    #[inline(always)]
    pub fn our(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize] & self.us()
    }

    #[inline(always)]
    pub fn their(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize] & self.them()
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, idx: usize) {
        if let Some((piece, color)) = self.piece_map[idx] {
            self.pieces[piece as usize].remove_bit(idx);
            self.all_pieces[color as usize].remove_bit(idx);
            self.piece_map[idx] = None;
        }
    }

    #[inline(always)]
    pub fn add_piece(&mut self, idx: usize, color: Color, board: Piece) {
        self.pieces[board as usize].set_bit(idx);
        self.all_pieces[color as usize].set_bit(idx);
        self.piece_map[idx] = Some((board, color));
    }

    #[inline(always)]
    pub fn is_square_attacked(&self, sq: usize) -> bool {
        self.is_square_attacked_by(sq, self.side_to_move.flip())
    }

    #[inline(always)]
    pub fn piece_at(&self, idx: usize) -> Option<(Piece, Color)> {
        self.piece_map[idx]
    }

}

#[cfg(test)]
mod position {
    use crate::Position;
    use types::others::Color::*;
    use types::others::Piece::*;
    use utilities::algebraic::Algebraic;
    use utilities::board::PrintAsBoard;

    #[test]
    fn set_remove() {
        let mut pos = Position::new();
        pos.add_piece("a5".idx(), White, Rook);
        pos.remove_piece("a1".idx());
        pos.all_pieces[White as usize].0.print();
        pos.all_pieces[Black as usize].0.print();
    }
}
