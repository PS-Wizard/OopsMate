use crate::types::*;

// ============================================================================
// POSITION
// ============================================================================

#[derive(Clone, PartialEq)]
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

        Ok(pos)
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
    pub fn make_move(&self, m: Move) -> Position {
        let mut new_pos = self.clone();
        new_pos.apply_move(m);
        new_pos
    }

    fn apply_move(&mut self, m: Move) {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        let (piece, color) = self.piece_at(from).expect("No piece at from");

        match move_type {
            MoveType::Quiet => {
                self.remove_piece(from);
                self.add_piece(to, color, piece);
            }
            MoveType::Capture => {
                self.remove_piece(to);
                self.remove_piece(from);
                self.add_piece(to, color, piece);
            }
            MoveType::DoublePush => {
                self.remove_piece(from);
                self.add_piece(to, color, piece);
                self.en_passant = Some(((from + to) / 2) as u8);
            }
            MoveType::EnPassant => {
                let captured = if color == Color::White {
                    to - 8
                } else {
                    to + 8
                };
                self.remove_piece(captured);
                self.remove_piece(from);
                self.add_piece(to, color, piece);
            }
            MoveType::Castle => {
                self.remove_piece(from);
                self.add_piece(to, color, piece);
                let (rook_from, rook_to) = match to {
                    6 => (7, 5),
                    2 => (0, 3),
                    62 => (63, 61),
                    58 => (56, 59),
                    _ => panic!("Invalid castle"),
                };
                self.remove_piece(rook_from);
                self.add_piece(rook_to, color, Piece::Rook);
            }
            MoveType::PromotionKnight
            | MoveType::PromotionBishop
            | MoveType::PromotionRook
            | MoveType::PromotionQueen => {
                self.remove_piece(from);
                let promoted = match move_type {
                    MoveType::PromotionKnight => Piece::Knight,
                    MoveType::PromotionBishop => Piece::Bishop,
                    MoveType::PromotionRook => Piece::Rook,
                    MoveType::PromotionQueen => Piece::Queen,
                    _ => unreachable!(),
                };
                self.add_piece(to, color, promoted);
            }
            MoveType::CapturePromotionKnight
            | MoveType::CapturePromotionBishop
            | MoveType::CapturePromotionRook
            | MoveType::CapturePromotionQueen => {
                self.remove_piece(to);
                self.remove_piece(from);
                let promoted = match move_type {
                    MoveType::CapturePromotionKnight => Piece::Knight,
                    MoveType::CapturePromotionBishop => Piece::Bishop,
                    MoveType::CapturePromotionRook => Piece::Rook,
                    MoveType::CapturePromotionQueen => Piece::Queen,
                    _ => unreachable!(),
                };
                self.add_piece(to, color, promoted);
            }
        }

        // Update castling rights
        if piece == Piece::King {
            self.castling_rights.remove_color(color);
        }
        self.castling_rights.remove_rook(from);
        self.castling_rights.remove_rook(to);

        // Update clocks
        if piece == Piece::Pawn || m.is_capture() {
            self.halfmove = 0;
        } else {
            self.halfmove += 1;
        }

        if self.side_to_move == Color::Black {
            self.fullmove += 1;
        }

        if move_type != MoveType::DoublePush {
            self.en_passant = None;
        }

        self.side_to_move = self.side_to_move.flip();
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}
