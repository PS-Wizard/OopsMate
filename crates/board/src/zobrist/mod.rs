use types::others::{Color, Piece};
use zobrist::ZOBRIST;

use crate::Position;

impl Position {
    pub fn compute_hash(&self) -> u64 {
        let mut hash = 0u64;

        // Hash all pieces on the board
        for square in 0..64 {
            if let Some((piece, color)) = self.piece_map[square] {
                hash ^= ZOBRIST.piece(piece, color, square);
            }
        }

        // Hash castling rights
        hash ^= ZOBRIST.castling_key(self.castling_rights);

        // Hash en passant
        if let Some(ep_square) = self.en_passant {
            let file = (ep_square % 8) as usize;
            hash ^= ZOBRIST.en_passant_key(file);
        }

        // Hash side to move (only XOR if black)
        if self.side_to_move == Color::Black {
            hash ^= ZOBRIST.side_to_move;
        }

        hash
    }

    /// Remove piece WITHOUT updating hash (for unmake_move)
    #[inline(always)]
    pub fn remove_piece_silent(&mut self, idx: usize) {
        if let Some((piece, _color)) = self.piece_map[idx] {
            self.pieces[piece as usize].remove_bit(idx);
            self.all_pieces[_color as usize].remove_bit(idx);
            self.piece_map[idx] = None;
        }
    }

    /// Add piece WITHOUT updating hash (for unmake_move)
    #[inline(always)]
    pub fn add_piece_silent(&mut self, idx: usize, color: Color, piece: Piece) {
        self.pieces[piece as usize].set_bit(idx);
        self.all_pieces[color as usize].set_bit(idx);
        self.piece_map[idx] = Some((piece, color));
    }
}
