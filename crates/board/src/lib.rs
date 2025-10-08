use types::{
    bitboard::Bitboard,
    others::{CastleRights, Color, Piece},
};

mod fen;
mod legality;
mod move_gen;

#[derive(Clone, Debug, PartialEq)]
/// A struct that represents the entire game state. Uses bitboards to represent pieces, 6 total,
/// one for each piece type, uses all_pieces to distguish between the piece of each side. Contains
/// a piece square mapping for O(1) identification of a piece.
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
    /// Returns a new position struct set to the initial game state.
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap_or_else(|err| panic!("Failed to parse starting FEN: {}", err))
    }

    #[inline(always)]
    /// Returns a new position struct set to the provided fen
    pub fn new_from_fen(fen: &str) -> Self {
        Self::from_fen(fen).unwrap_or_else(|err| panic!("Failed to parse starting FEN: {}", err))
    }

    #[inline(always)]
    /// Returns all friendly pieces based on the side to move.
    pub fn us(&self) -> Bitboard {
        self.all_pieces[self.side_to_move as usize]
    }

    #[inline(always)]
    /// Returns all enemy pieces based on the side to move.
    pub fn them(&self) -> Bitboard {
        self.all_pieces[self.side_to_move.flip() as usize]
    }

    #[inline(always)]
    /// Returns the bitboard of a specific piece of the current player
    pub fn our(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize] & self.us()
    }

    #[inline(always)]
    /// Returns the bitboard of a specific piece of the opponent
    pub fn their(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize] & self.them()
    }

    #[inline(always)]
    /// Removes piece at a given index, takes in the idx as a usize and a mutable reference to the
    /// Position struct
    pub fn remove_piece(&mut self, idx: usize) {
        if let Some((piece, color)) = self.piece_map[idx] {
            self.pieces[piece as usize].remove_bit(idx);
            self.all_pieces[color as usize].remove_bit(idx);
            self.piece_map[idx] = None;
        }
    }

    #[inline(always)]
    /// Takes in a mutable refernce to the position struct, an index , a Color and a piece type and
    /// sets the bit of the relevant board based on those provided values.
    pub fn add_piece(&mut self, idx: usize, color: Color, board: Piece) {
        self.pieces[board as usize].set_bit(idx);
        self.all_pieces[color as usize].set_bit(idx);
        self.piece_map[idx] = Some((board, color));
    }

    #[inline(always)]
    /// Given a square, returns true if the square is attacked by an enemy piece else false.
    pub fn is_square_attacked(&self, sq: usize) -> bool {
        self.is_square_attacked_by(sq, self.side_to_move.flip())
    }

    #[inline(always)]
    /// Given a square, returns an Optional Piece
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
