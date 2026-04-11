//! Basic types for NNUE evaluation

/// Piece colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(idx: usize) -> Self {
        if idx == 0 {
            Color::White
        } else {
            Color::Black
        }
    }
}

/// Chess pieces (includes color information)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    None = 0,
    WhitePawn = 1,
    WhiteKnight = 2,
    WhiteBishop = 3,
    WhiteRook = 4,
    WhiteQueen = 5,
    WhiteKing = 6,
    BlackPawn = 7,
    BlackKnight = 8,
    BlackBishop = 9,
    BlackRook = 10,
    BlackQueen = 11,
    BlackKing = 12,
}

impl Piece {
    /// Get the piece type (1=pawn, 2=knight, ..., 6=king)
    pub fn piece_type(self) -> usize {
        match self {
            Piece::None => 0,
            Piece::WhitePawn | Piece::BlackPawn => 1,
            Piece::WhiteKnight | Piece::BlackKnight => 2,
            Piece::WhiteBishop | Piece::BlackBishop => 3,
            Piece::WhiteRook | Piece::BlackRook => 4,
            Piece::WhiteQueen | Piece::BlackQueen => 5,
            Piece::WhiteKing | Piece::BlackKing => 6,
        }
    }

    /// Get the color of this piece
    pub fn color(self) -> Option<Color> {
        match self {
            Piece::None => None,
            Piece::WhitePawn
            | Piece::WhiteKnight
            | Piece::WhiteBishop
            | Piece::WhiteRook
            | Piece::WhiteQueen
            | Piece::WhiteKing => Some(Color::White),
            _ => Some(Color::Black),
        }
    }

    /// Is this piece a king?
    pub fn is_king(self) -> bool {
        matches!(self, Piece::WhiteKing | Piece::BlackKing)
    }

    /// Get the Stockfish piece code for feature indexing.
    /// Format: color bit (8) | piece_type (1-6)
    /// White: 1=WP, 2=WN, 3=WB, 4=WR, 5=WQ, 6=WK
    /// Black: 9=BP, 10=BN, 11=BB, 12=BR, 13=BQ, 14=BK
    pub fn stockfish_code(self) -> usize {
        match self {
            Piece::None => 0,
            Piece::WhitePawn => 1,
            Piece::WhiteKnight => 2,
            Piece::WhiteBishop => 3,
            Piece::WhiteRook => 4,
            Piece::WhiteQueen => 5,
            Piece::WhiteKing => 6,
            Piece::BlackPawn => 9,
            Piece::BlackKnight => 10,
            Piece::BlackBishop => 11,
            Piece::BlackRook => 12,
            Piece::BlackQueen => 13,
            Piece::BlackKing => 14,
        }
    }

    /// Get the internal index used for feature calculation
    /// This maps to: 0-4 = white pieces (pawn..queen), 5-9 = black pieces, 10 = white king
    pub fn index(self) -> usize {
        match self {
            Piece::None => 0,
            Piece::WhitePawn => 0,
            Piece::WhiteKnight => 1,
            Piece::WhiteBishop => 2,
            Piece::WhiteRook => 3,
            Piece::WhiteQueen => 4,
            Piece::WhiteKing => 10,
            Piece::BlackPawn => 5,
            Piece::BlackKnight => 6,
            Piece::BlackBishop => 7,
            Piece::BlackRook => 8,
            Piece::BlackQueen => 9,
            Piece::BlackKing => 11,
        }
    }

    /// Human-readable name
    pub fn name(self) -> &'static str {
        match self {
            Piece::None => "None",
            Piece::WhitePawn => "White Pawn",
            Piece::WhiteKnight => "White Knight",
            Piece::WhiteBishop => "White Bishop",
            Piece::WhiteRook => "White Rook",
            Piece::WhiteQueen => "White Queen",
            Piece::WhiteKing => "White King",
            Piece::BlackPawn => "Black Pawn",
            Piece::BlackKnight => "Black Knight",
            Piece::BlackBishop => "Black Bishop",
            Piece::BlackRook => "Black Rook",
            Piece::BlackQueen => "Black Queen",
            Piece::BlackKing => "Black King",
        }
    }

    /// Short notation (P, N, B, R, Q, K for white; p, n, b, r, q, k for black)
    pub fn symbol(self) -> char {
        match self {
            Piece::None => '.',
            Piece::WhitePawn => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook => 'R',
            Piece::WhiteQueen => 'Q',
            Piece::WhiteKing => 'K',
            Piece::BlackPawn => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook => 'r',
            Piece::BlackQueen => 'q',
            Piece::BlackKing => 'k',
        }
    }
}

/// Convert a square index (0-63) to algebraic notation (a1-h8)
/// Square 0 = a1, Square 7 = h1, Square 56 = a8, Square 63 = h8
pub fn square_name(sq: usize) -> String {
    let file = (sq % 8) as u8;
    let rank = (sq / 8) as u8;
    format!("{}{}", (b'a' + file) as char, (b'1' + rank) as char)
}

/// Convert algebraic notation to square index
pub fn square_from_name(name: &str) -> Option<usize> {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() != 2 {
        return None;
    }
    let file = chars[0] as usize;
    let rank = chars[1] as usize;
    if file < 'a' as usize || file > 'h' as usize {
        return None;
    }
    if rank < '1' as usize || rank > '8' as usize {
        return None;
    }
    Some((rank - '1' as usize) * 8 + (file - 'a' as usize))
}

// Material values for centipawn conversion
pub const PAWN_VALUE: i32 = 208;
pub const KNIGHT_VALUE: i32 = 781;
pub const BISHOP_VALUE: i32 = 825;
pub const ROOK_VALUE: i32 = 1276;
pub const QUEEN_VALUE: i32 = 2538;

/// Get material value of a piece
pub fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::WhitePawn | Piece::BlackPawn => PAWN_VALUE,
        Piece::WhiteKnight | Piece::BlackKnight => KNIGHT_VALUE,
        Piece::WhiteBishop | Piece::BlackBishop => BISHOP_VALUE,
        Piece::WhiteRook | Piece::BlackRook => ROOK_VALUE,
        Piece::WhiteQueen | Piece::BlackQueen => QUEEN_VALUE,
        _ => 0,
    }
}
