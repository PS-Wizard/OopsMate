use super::Position;
use crate::zobrist::{CASTLE_KEYS, EP_KEYS, PIECE_KEYS, SIDE_KEY};

impl Position {
    #[inline(always)]
    /// Recomputes the full Zobrist hash from board state.
    pub fn compute_hash(&self) -> u64 {
        let mut h = 0u64;
        for (sq, entry) in self.board.iter().enumerate() {
            if let Some((piece, color)) = *entry {
                h ^= PIECE_KEYS[color as usize][piece as usize][sq];
            }
        }
        h ^= CASTLE_KEYS[self.castling_rights.0 as usize];
        if let Some(ep) = self.en_passant {
            h ^= EP_KEYS[(ep % 8) as usize];
        }
        if self.side_to_move == crate::Color::Black {
            h ^= SIDE_KEY;
        }
        h
    }
}
