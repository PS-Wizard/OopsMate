use crate::types::*;

#[derive(Clone, Copy, Debug)]
pub struct GameState {
    pub castling_rights: CastleRights,
    pub en_passant: Option<u8>,
    pub halfmove: u16,
    pub hash: u64,
    pub captured_piece: Option<Piece>,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub pieces: [Bitboard; 6],
    pub colors: [Bitboard; 2],
    pub board: [Option<(Piece, Color)>; 64],
    pub side_to_move: Color,
    pub castling_rights: CastleRights,
    pub en_passant: Option<u8>,
    pub halfmove: u16,
    pub fullmove: u16,
    pub hash: u64,
    pub history: Vec<GameState>,
}

impl Position {
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Invalid starting FEN")
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    pub const fn our(&self, piece: Piece) -> Bitboard {
        Bitboard(self.pieces[piece as usize].0 & self.colors[self.side_to_move as usize].0)
    }

    #[inline(always)]
    pub const fn their(&self, piece: Piece) -> Bitboard {
        Bitboard(self.pieces[piece as usize].0 & self.colors[self.side_to_move.flip() as usize].0)
    }

    #[inline(always)]
    pub const fn us(&self) -> Bitboard {
        self.colors[self.side_to_move as usize]
    }

    #[inline(always)]
    pub const fn them(&self) -> Bitboard {
        self.colors[self.side_to_move.flip() as usize]
    }

    #[inline(always)]
    pub const fn occupied(&self) -> Bitboard {
        Bitboard(self.colors[0].0 | self.colors[1].0)
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: usize) -> Option<(Piece, Color)> {
        self.board[sq]
    }

    #[inline(always)]
    pub fn add_piece(&mut self, sq: usize, color: Color, piece: Piece) {
        self.pieces[piece as usize].set(sq);
        self.colors[color as usize].set(sq);
        self.board[sq] = Some((piece, color));
    }

    #[inline(always)]
    pub fn move_piece(&mut self, from: usize, to: usize, color: Color, piece: Piece) {
        self.pieces[piece as usize].clear(from);
        self.pieces[piece as usize].set(to);
        self.colors[color as usize].clear(from);
        self.colors[color as usize].set(to);
        self.board[from] = None;
        self.board[to] = Some((piece, color));
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, sq: usize) {
        if let Some((piece, color)) = self.board[sq] {
            self.pieces[piece as usize].clear(sq);
            self.colors[color as usize].clear(sq);
            self.board[sq] = None;
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}
