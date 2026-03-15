use super::Position;
use crate::types::{Bitboard, CastleRights, Color, Piece};

impl Position {
    /// Builds a position from a FEN string.
    ///
    /// The parser accepts the standard six-field FEN form. If the halfmove or
    /// fullmove counters are omitted, they default to `0` and `1` respectively.
    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Invalid FEN");
        }

        let mut pos = Position {
            pieces: [Bitboard::new(); 6],
            colors: [Bitboard::new(); 2],
            board: [None; 64],
            side_to_move: Color::White,
            castling_rights: CastleRights::NONE,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
            hash: 0,
            history: Vec::with_capacity(1024),
        };

        let mut sq = 56;
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
                    pos.add_piece(sq, color, piece);
                    sq += 1;
                }
            }
        }

        pos.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side to move"),
        };

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

        if parts[3] != "-" {
            let file = parts[3].as_bytes()[0] - b'a';
            let rank = parts[3].as_bytes()[1] - b'1';
            pos.en_passant = Some(rank * 8 + file);
        }

        if parts.len() >= 5 {
            pos.halfmove = parts[4].parse().unwrap_or(0);
        }
        if parts.len() >= 6 {
            pos.fullmove = parts[5].parse().unwrap_or(1);
        }

        pos.hash = pos.compute_hash();
        Ok(pos)
    }
}
