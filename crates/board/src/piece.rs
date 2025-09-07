#![allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

// Each Piece Is 00000000, where first 3 bits is the piece type, the 4th bit is the color.
// i.e 000<color_(1)><piece_type_(3)>
pub struct Piece(u8);

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self((piece_type as u8) | ((color as u8) << 3))
    }

    pub fn info(&self) -> Option<(PieceType, Color)> {
        let piece_val = self.0 & 0b111; // Extracts the last 3 bits
        let color_val = (self.0 >> 3) & 0b1; // extracts the 4th bit

        let piece_type = match piece_val {
            0 => PieceType::Pawn,
            1 => PieceType::Rook,
            2 => PieceType::Knight,
            3 => PieceType::Bishop,
            4 => PieceType::Queen,
            5 => PieceType::King,
            _ => return None, // invalid piece
        };

        let color = match color_val {
            0 => Color::White,
            1 => Color::Black,
            _ => return None, // theoretically unreachable
        };

        Some((piece_type, color))
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((piece_type, color)) = self.info() {
            let symbol = match (piece_type, color) {
                (PieceType::Pawn, Color::Black) => "",
                (PieceType::Rook, Color::Black) => "",
                (PieceType::Knight, Color::Black) => "",
                (PieceType::Bishop, Color::Black) => "",
                (PieceType::Queen, Color::Black) => "",
                (PieceType::King, Color::Black) => "",

                (PieceType::Pawn, Color::White) => "󰡙",
                (PieceType::Rook, Color::White) => "󰡛",
                (PieceType::Knight, Color::White) => "󰡘",
                (PieceType::Bishop, Color::White) => "󰡜",
                (PieceType::Queen, Color::White) => "󰡚",
                (PieceType::King, Color::White) => "󰡗",
            };

            write!(f, "{}", symbol)
        } else {
            write!(f, "?")
        }
    }
}

#[cfg(test)]
mod piece_tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn test_piece_creation_and_info() {
        let p = Piece::new(PieceType::Pawn, Color::White);
        let info = p.info().unwrap();
        assert!(matches!(info.0, PieceType::Pawn));
        assert!(matches!(info.1, Color::White));

        let p2 = Piece::new(PieceType::King, Color::Black);
        let info2 = p2.info().unwrap();
        assert!(matches!(info2.0, PieceType::King));
        assert!(matches!(info2.1, Color::Black));
    }

    #[test]
    fn test_piece_display_symbols() {
        let tests = [
            (PieceType::Pawn, Color::White, "󰡙"),
            (PieceType::Rook, Color::White, "󰡛"),
            (PieceType::Knight, Color::White, "󰡘"),
            (PieceType::Bishop, Color::White, "󰡜"),
            (PieceType::Queen, Color::White, "󰡚"),
            (PieceType::King, Color::White, "󰡗"),
            (PieceType::Pawn, Color::Black, ""),
            (PieceType::Rook, Color::Black, ""),
            (PieceType::Knight, Color::Black, ""),
            (PieceType::Bishop, Color::Black, ""),
            (PieceType::Queen, Color::Black, ""),
            (PieceType::King, Color::Black, ""),
        ];

        for (pt, color, expected) in tests {
            let piece = Piece::new(pt, color);
            let mut s = String::new();
            write!(s, "{}", piece).unwrap();
            assert_eq!(s, expected);
        }
    }

    #[test]
    fn test_invalid_piece_bits() {
        // Directly make an invalid piece
        let invalid = Piece(0b1111_1111);
        assert!(invalid.info().is_none());
    }
}
