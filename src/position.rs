use crate::{
    types::*,
    zobrist::{CASTLE_KEYS, EP_KEYS, PIECE_KEYS, SIDE_KEY},
};

// ============================================================================
// POSITION
// ============================================================================
#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    // Piece bitboards [Pawn, Knight, Bishop, Rook, Queen, King]
    pub pieces: [Bitboard; 6],
    // Color occupancy [White, Black]
    pub colors: [Bitboard; 2],
    // Game state
    pub side_to_move: Color,
    pub castling_rights: CastleRights,
    pub en_passant: Option<u8>,
    pub halfmove: u16,
    pub fullmove: u16,
    pub hash: u64,
}

impl Position {
    /// Create starting position
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Invalid starting FEN")
    }

    /// Parse FEN string
    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Invalid FEN");
        }

        let mut pos = Position {
            pieces: [Bitboard::new(); 6],
            colors: [Bitboard::new(); 2],
            side_to_move: Color::White,
            castling_rights: CastleRights::NONE,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
            hash: 0,
        };

        // Parse board
        let mut sq = 56; // Start at a8
        for c in parts[0].chars() {
            match c {
                '/' => sq -= 16,
                '1'..='8' => sq += c.to_digit(10).unwrap() as usize,
                _ => {
                    let (piece, color) = match c {
                        'P' => (Piece::Pawn, Color::White),
                        'N' => (Piece::Knight, Color::White),
                        'B' => (Piece::Bishop, Color::White),
                        'R' => (Piece::Rook, Color::White),
                        'Q' => (Piece::Queen, Color::White),
                        'K' => (Piece::King, Color::White),
                        'p' => (Piece::Pawn, Color::Black),
                        'n' => (Piece::Knight, Color::Black),
                        'b' => (Piece::Bishop, Color::Black),
                        'r' => (Piece::Rook, Color::Black),
                        'q' => (Piece::Queen, Color::Black),
                        'k' => (Piece::King, Color::Black),
                        _ => return Err("Invalid piece"),
                    };
                    pos.pieces[piece as usize].set(sq);
                    pos.colors[color as usize].set(sq);
                    sq += 1;
                }
            }
        }

        // Side to move
        pos.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side to move"),
        };

        // Castling rights
        let mut rights = 0u8;
        for c in parts[2].chars() {
            rights |= match c {
                'K' => 1,
                'Q' => 2,
                'k' => 4,
                'q' => 8,
                '-' => 0,
                _ => return Err("Invalid castling"),
            };
        }
        pos.castling_rights = CastleRights(rights);

        // En passant
        if parts[3] != "-" {
            let file = (parts[3].as_bytes()[0] - b'a') as u8;
            let rank = (parts[3].as_bytes()[1] - b'1') as u8;
            pos.en_passant = Some(rank * 8 + file);
        }

        // Halfmove and fullmove
        if parts.len() >= 5 {
            pos.halfmove = parts[4].parse().unwrap_or(0);
        }
        if parts.len() >= 6 {
            pos.fullmove = parts[5].parse().unwrap_or(1);
        }

        pos.hash = pos.compute_hash();
        Ok(pos)
    }

    // ========================================================================
    // Zobrist Hashing
    // ========================================================================

    #[inline(always)]
    pub fn compute_hash(&self) -> u64 {
        let mut h = 0u64;

        // Hash all pieces
        for sq in 0..64 {
            if let Some((piece, color)) = self.piece_at(sq) {
                h ^= PIECE_KEYS[color as usize][piece as usize][sq];
            }
        }

        // Hash castling rights
        h ^= CASTLE_KEYS[self.castling_rights.0 as usize];

        // Hash en passant
        if let Some(ep) = self.en_passant {
            h ^= EP_KEYS[(ep % 8) as usize];
        }

        // Hash side to move
        if self.side_to_move == Color::Black {
            h ^= SIDE_KEY;
        }

        h
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    // ========================================================================
    // PIECE QUERIES
    // ========================================================================

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
        let mask = 1u64 << sq;
        for color in [Color::White, Color::Black] {
            if self.colors[color as usize].0 & mask != 0 {
                for piece in [
                    Piece::Pawn,
                    Piece::Knight,
                    Piece::Bishop,
                    Piece::Rook,
                    Piece::Queen,
                    Piece::King,
                ] {
                    if self.pieces[piece as usize].0 & mask != 0 {
                        return Some((piece, color));
                    }
                }
            }
        }
        None
    }

    // ========================================================================
    // PIECE MANIPULATION
    // ========================================================================

    #[inline(always)]
    pub fn add_piece(&mut self, sq: usize, color: Color, piece: Piece) {
        self.pieces[piece as usize].set(sq);
        self.colors[color as usize].set(sq);
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, sq: usize) {
        let mask = !(1u64 << sq);
        for i in 0..6 {
            self.pieces[i].0 &= mask;
        }
        self.colors[0].0 &= mask;
        self.colors[1].0 &= mask;
    }

    // ========================================================================
    // MOVE APPLICATION
    // ========================================================================

    #[inline(always)]
    pub fn make_move(&self, m: &Move) -> Position {
        let mut new_pos = self.clone();
        new_pos.apply_move(&m);
        new_pos
    }

    #[inline(always)]
    fn apply_move(&mut self, m: &Move) {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        let (piece, color) = self.piece_at(from).expect("No piece at from");

        // Remove old castling rights from hash
        self.hash ^= CASTLE_KEYS[self.castling_rights.0 as usize];

        // Remove old en passant from hash
        if let Some(ep) = self.en_passant {
            self.hash ^= EP_KEYS[(ep % 8) as usize];
        }

        match move_type {
            MoveType::Quiet => {
                // Remove piece from 'from'
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.remove_piece(from);

                // Add piece to 'to'
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.add_piece(to, color, piece);
            }
            MoveType::Capture => {
                let (captured_piece, captured_color) = self.piece_at(to).unwrap();

                // Remove captured piece
                self.hash ^= PIECE_KEYS[captured_color as usize][captured_piece as usize][to];
                self.remove_piece(to);

                // Remove moving piece from 'from'
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.remove_piece(from);

                // Add moving piece to 'to'
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.add_piece(to, color, piece);
            }
            MoveType::DoublePush => {
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.remove_piece(from);

                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.add_piece(to, color, piece);

                self.en_passant = Some(((from + to) / 2) as u8);
            }
            MoveType::EnPassant => {
                let captured = if color == Color::White {
                    to - 8
                } else {
                    to + 8
                };

                self.hash ^= PIECE_KEYS[color.flip() as usize][Piece::Pawn as usize][captured];
                self.remove_piece(captured);

                // Move our pawn
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.remove_piece(from);

                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.add_piece(to, color, piece);
            }
            MoveType::Castle => {
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.remove_piece(from);

                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.add_piece(to, color, piece);

                let (rook_from, rook_to) = match to {
                    6 => (7, 5),
                    2 => (0, 3),
                    62 => (63, 61),
                    58 => (56, 59),
                    _ => panic!("Invalid castle"),
                };

                self.hash ^= PIECE_KEYS[color as usize][Piece::Rook as usize][rook_from];
                self.remove_piece(rook_from);

                self.hash ^= PIECE_KEYS[color as usize][Piece::Rook as usize][rook_to];
                self.add_piece(rook_to, color, Piece::Rook);
            }
            MoveType::PromotionKnight
            | MoveType::PromotionBishop
            | MoveType::PromotionRook
            | MoveType::PromotionQueen => {
                self.hash ^= PIECE_KEYS[color as usize][Piece::Pawn as usize][from];
                self.remove_piece(from);

                let promoted = match move_type {
                    MoveType::PromotionKnight => Piece::Knight,
                    MoveType::PromotionBishop => Piece::Bishop,
                    MoveType::PromotionRook => Piece::Rook,
                    MoveType::PromotionQueen => Piece::Queen,
                    _ => unreachable!(),
                };

                self.hash ^= PIECE_KEYS[color as usize][promoted as usize][to];
                self.add_piece(to, color, promoted);
            }
            MoveType::CapturePromotionKnight
            | MoveType::CapturePromotionBishop
            | MoveType::CapturePromotionRook
            | MoveType::CapturePromotionQueen => {
                let (captured_piece, captured_color) = self.piece_at(to).unwrap();

                // Remove captured piece
                self.hash ^= PIECE_KEYS[captured_color as usize][captured_piece as usize][to];
                self.remove_piece(to);

                // Remove pawn from 'from'
                self.hash ^= PIECE_KEYS[color as usize][Piece::Pawn as usize][from];
                self.remove_piece(from);

                let promoted = match move_type {
                    MoveType::CapturePromotionKnight => Piece::Knight,
                    MoveType::CapturePromotionBishop => Piece::Bishop,
                    MoveType::CapturePromotionRook => Piece::Rook,
                    MoveType::CapturePromotionQueen => Piece::Queen,
                    _ => unreachable!(),
                };

                self.hash ^= PIECE_KEYS[color as usize][promoted as usize][to];
                self.add_piece(to, color, promoted);
            }
        }

        // Update castling rights
        if piece == Piece::King {
            self.castling_rights.remove_color(color);
        }
        self.castling_rights.remove_rook(from);
        self.castling_rights.remove_rook(to);

        // Add new castling rights to hash
        self.hash ^= CASTLE_KEYS[self.castling_rights.0 as usize];

        // Update clocks
        if piece == Piece::Pawn || m.is_capture() {
            self.halfmove = 0;
        } else {
            self.halfmove += 1;
        }

        if self.side_to_move == Color::Black {
            self.fullmove += 1;
        }

        // Clear en passant (unless double push, which set it above)
        if move_type != MoveType::DoublePush {
            self.en_passant = None;
        }

        // Add new en passant to hash
        if let Some(ep) = self.en_passant {
            self.hash ^= EP_KEYS[(ep % 8) as usize];
        }

        // Flip side to move
        self.hash ^= SIDE_KEY;
        self.side_to_move = self.side_to_move.flip();
    }
}

// Game State
impl Position {
    /// Check if the game is over (no legal moves)
    pub fn is_game_over(&self) -> bool {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);
        collector.is_empty()
    }

    /// Check if the current position is checkmate
    pub fn is_checkmate(&self) -> bool {
        self.is_in_check() && self.is_game_over()
    }

    /// Check if the current position is stalemate
    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check() && self.is_game_over()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkmate_detection() {
        // Fool's mate
        let pos =
            Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        assert!(pos.is_checkmate());
    }

    #[test]
    fn test_stalemate_detection() {
        let pos = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(pos.is_stalemate());
    }
}
