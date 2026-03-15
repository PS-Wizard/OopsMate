use crate::types::*;

/// Snapshot of reversible state saved before a move is made.
#[derive(Clone, Copy, Debug)]
pub struct GameState {
    /// Castling rights before the move.
    pub castling_rights: CastleRights,
    /// En passant square before the move, if any.
    pub en_passant: Option<u8>,
    /// Halfmove clock before the move.
    pub halfmove: u16,
    /// Zobrist hash before the move.
    pub hash: u64,
    /// Captured piece type, used during unmake.
    pub captured_piece: Option<Piece>,
}

/// The engine's board representation.
///
/// The position stores piece bitboards, color bitboards, an indexed board array,
/// the side to move, irreversible state, and a compact history stack used for
/// unmake and repetition detection.
#[derive(Clone, Debug)]
pub struct Position {
    /// Piece bitboards indexed by `Piece`.
    pub pieces: [Bitboard; 6],
    /// Color bitboards indexed by `Color`.
    pub colors: [Bitboard; 2],
    /// Piece lookup table indexed by square.
    pub board: [Option<(Piece, Color)>; 64],
    /// Side to move.
    pub side_to_move: Color,
    /// Current castling rights.
    pub castling_rights: CastleRights,
    /// En passant square, if available.
    pub en_passant: Option<u8>,
    /// Halfmove clock used for the fifty-move rule.
    pub halfmove: u16,
    /// Fullmove counter from the FEN state.
    pub fullmove: u16,
    /// Incrementally maintained Zobrist hash.
    pub hash: u64,
    /// Reversible state stack used by make/unmake.
    pub history: Vec<GameState>,
}

impl Position {
    /// Creates the standard chess starting position.
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Invalid starting FEN")
    }

    #[inline(always)]
    /// Returns the current Zobrist hash.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    /// Returns the current side's pieces of the requested type.
    pub const fn our(&self, piece: Piece) -> Bitboard {
        Bitboard(self.pieces[piece as usize].0 & self.colors[self.side_to_move as usize].0)
    }

    #[inline(always)]
    /// Returns the opponent's pieces of the requested type.
    pub const fn their(&self, piece: Piece) -> Bitboard {
        Bitboard(self.pieces[piece as usize].0 & self.colors[self.side_to_move.flip() as usize].0)
    }

    #[inline(always)]
    /// Returns the current side's occupancy bitboard.
    pub const fn us(&self) -> Bitboard {
        self.colors[self.side_to_move as usize]
    }

    #[inline(always)]
    /// Returns the opponent's occupancy bitboard.
    pub const fn them(&self) -> Bitboard {
        self.colors[self.side_to_move.flip() as usize]
    }

    #[inline(always)]
    /// Returns the board occupancy bitboard.
    pub const fn occupied(&self) -> Bitboard {
        Bitboard(self.colors[0].0 | self.colors[1].0)
    }

    #[inline(always)]
    /// Returns the piece and color on `sq`, if the square is occupied.
    pub fn piece_at(&self, sq: usize) -> Option<(Piece, Color)> {
        self.board[sq]
    }

    #[inline(always)]
    /// Inserts a piece onto a square and updates all board views.
    pub fn add_piece(&mut self, sq: usize, color: Color, piece: Piece) {
        self.pieces[piece as usize].set(sq);
        self.colors[color as usize].set(sq);
        self.board[sq] = Some((piece, color));
    }

    #[inline(always)]
    /// Moves a piece between squares without changing its type or color.
    pub fn move_piece(&mut self, from: usize, to: usize, color: Color, piece: Piece) {
        self.pieces[piece as usize].clear(from);
        self.pieces[piece as usize].set(to);
        self.colors[color as usize].clear(from);
        self.colors[color as usize].set(to);
        self.board[from] = None;
        self.board[to] = Some((piece, color));
    }

    #[inline(always)]
    /// Removes the piece currently occupying `sq`.
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
