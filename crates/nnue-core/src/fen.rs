//! FEN string parsing
//!
//! FEN (Forsyth-Edwards Notation) is a standard way to describe chess positions.
//! Example: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//!
//! Format: <board> <side> <castling> <en-passant> <halfmove> <fullmove>
//!
//! Board is described rank by rank from rank 8 (top) to rank 1 (bottom),
//! with '/' separating ranks. Digits indicate empty squares.

use crate::types::{Color, Piece};

/// Parse a FEN string into pieces, side to move, and rule50 counter
pub fn parse_fen(fen: &str) -> (Vec<(Piece, usize)>, Color, i32) {
    let parts: Vec<&str> = fen.split_whitespace().collect();

    // Part 0: Board position
    let board_str = parts[0];

    // Part 1: Side to move ('w' or 'b')
    let side_str = parts.get(1).unwrap_or(&"w");

    // Part 4: Halfmove clock (rule50)
    let rule50_str = parts.get(4).unwrap_or(&"0");
    let rule50: i32 = rule50_str.parse().unwrap_or(0);

    let mut pieces = Vec::new();

    // FEN starts from rank 8 (index 7) and goes down
    let mut rank = 7i32;
    let mut file = 0usize;

    for c in board_str.chars() {
        match c {
            '/' => {
                // Move to next rank (going down)
                rank -= 1;
                file = 0;
            }
            '1'..='8' => {
                // Skip empty squares
                file += c.to_digit(10).unwrap() as usize;
            }
            _ => {
                // It's a piece character
                let piece = char_to_piece(c);
                if piece != Piece::None {
                    // Square index: rank * 8 + file
                    // rank 0 = rank 1, file 0 = file a
                    let sq = (rank as usize) * 8 + file;
                    pieces.push((piece, sq));
                }
                file += 1;
            }
        }
    }

    let side = if *side_str == "w" {
        Color::White
    } else {
        Color::Black
    };

    (pieces, side, rule50)
}

/// Convert a FEN piece character to a Piece enum
fn char_to_piece(c: char) -> Piece {
    match c {
        'P' => Piece::WhitePawn,
        'N' => Piece::WhiteKnight,
        'B' => Piece::WhiteBishop,
        'R' => Piece::WhiteRook,
        'Q' => Piece::WhiteQueen,
        'K' => Piece::WhiteKing,
        'p' => Piece::BlackPawn,
        'n' => Piece::BlackKnight,
        'b' => Piece::BlackBishop,
        'r' => Piece::BlackRook,
        'q' => Piece::BlackQueen,
        'k' => Piece::BlackKing,
        _ => Piece::None,
    }
}
