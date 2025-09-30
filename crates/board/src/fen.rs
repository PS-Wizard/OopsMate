use crate::Position;
use types::{
    bitboard::Bitboard,
    others::{CastleRights, Color, Piece},
};

impl Position {
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();

        if parts.len() != 6 {
            return Err("FEN string must have exactly 6 parts".to_string());
        }

        // Initialize empty position
        let mut position = Position {
            pieces: [Bitboard::new(); 6],
            all_pieces: [Bitboard::new(); 2],
            piece_map: [None; 64],
            side_to_move: Color::White,
            castling_rights: CastleRights(0),
            en_passant: None,
            half_clock: 0,
            full_clock: 1,
            hash: 0,
        };

        // Parse piece placement (part 0)
        Self::parse_piece_placement(&mut position, parts[0])?;

        // Parse side to move (part 1)
        position.side_to_move = Self::parse_side_to_move(parts[1])?;

        // Parse castling rights (part 2)
        position.castling_rights = Self::parse_castling_rights(parts[2])?;

        // Parse en passant square (part 3)
        position.en_passant = Self::parse_en_passant(parts[3])?;

        // Parse halfmove clock (part 4)
        position.half_clock = parts[4].parse().map_err(|_| "Invalid halfmove clock")?;

        // Parse fullmove number (part 5)
        position.full_clock = parts[5].parse().map_err(|_| "Invalid fullmove number")?;

        Ok(position)
    }

    fn parse_piece_placement(position: &mut Position, placement: &str) -> Result<(), String> {
        let ranks: Vec<&str> = placement.split('/').collect();

        if ranks.len() != 8 {
            return Err("Piece placement must have 8 ranks".to_string());
        }

        for (rank_idx, rank) in ranks.iter().enumerate() {
            let mut file_idx = 0;

            for ch in rank.chars() {
                if file_idx >= 8 {
                    return Err("Too many pieces/squares in rank".to_string());
                }

                let square_idx = (7 - rank_idx) * 8 + file_idx;

                if ch.is_ascii_digit() {
                    // Empty squares
                    let empty_count = ch.to_digit(10).unwrap() as usize;
                    if file_idx + empty_count > 8 {
                        return Err("Invalid empty square count".to_string());
                    }
                    file_idx += empty_count;
                } else {
                    // Piece
                    let (piece, color) = Self::char_to_piece(ch)?;

                    // Set piece in piece_map
                    position.piece_map[square_idx] = Some((piece, color));

                    // Set bit in appropriate bitboards
                    let square_bit = 1u64 << square_idx;
                    position.pieces[piece as usize] |= Bitboard::from_raw(square_bit);
                    position.all_pieces[color as usize] |= Bitboard::from_raw(square_bit);

                    file_idx += 1;
                }
            }

            if file_idx != 8 {
                return Err("Rank doesn't have exactly 8 squares".to_string());
            }
        }

        Ok(())
    }

    fn char_to_piece(ch: char) -> Result<(Piece, Color), String> {
        let (piece, color) = match ch {
            'P' => (Piece::Pawn, Color::White),
            'p' => (Piece::Pawn, Color::Black),
            'N' => (Piece::Knight, Color::White),
            'n' => (Piece::Knight, Color::Black),
            'B' => (Piece::Bishop, Color::White),
            'b' => (Piece::Bishop, Color::Black),
            'R' => (Piece::Rook, Color::White),
            'r' => (Piece::Rook, Color::Black),
            'Q' => (Piece::Queen, Color::White),
            'q' => (Piece::Queen, Color::Black),
            'K' => (Piece::King, Color::White),
            'k' => (Piece::King, Color::Black),
            _ => return Err(format!("Invalid piece character: {}", ch)),
        };

        Ok((piece, color))
    }

    fn parse_side_to_move(side_str: &str) -> Result<Color, String> {
        match side_str {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err("Side to move must be 'w' or 'b'".to_string()),
        }
    }

    fn parse_castling_rights(castle_str: &str) -> Result<CastleRights, String> {
        if castle_str == "-" {
            return Ok(CastleRights(0));
        }

        let mut rights = 0u8;

        for ch in castle_str.chars() {
            match ch {
                'K' => rights |= 0b0001, // White kingside
                'Q' => rights |= 0b0010, // White queenside
                'k' => rights |= 0b0100, // Black kingside
                'q' => rights |= 0b1000, // Black queenside
                _ => return Err(format!("Invalid castling right: {}", ch)),
            }
        }

        Ok(CastleRights(rights))
    }

    fn parse_en_passant(ep_str: &str) -> Result<Option<u8>, String> {
        if ep_str == "-" {
            return Ok(None);
        }

        if ep_str.len() != 2 {
            return Err("En passant square must be 2 characters".to_string());
        }

        let chars: Vec<char> = ep_str.chars().collect();
        let file = chars[0] as u8;
        let rank = chars[1] as u8;

        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return Err("Invalid en passant square".to_string());
        }

        let square_idx = (rank - b'1') * 8 + (file - b'a');
        Ok(Some(square_idx))
    }
}

// Example usage:
#[cfg(test)]
mod fen {
    use types::others::Color::*;
    use utilities::algebraic::Algebraic;

    use super::*;

    #[test]
    fn test_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let position = Position::from_fen(fen).unwrap();

        // Test that white king is on e1
        assert_eq!(
            position.piece_map["e1".idx()],
            Some((Piece::King, Color::White))
        );

        // Test black King on e8
        assert_eq!(
            position.piece_map["e8".idx()],
            Some((Piece::King, Color::Black))
        );

        // Test that white rook is on a1
        assert_eq!(
            position.piece_map["a1".idx()],
            Some((Piece::Rook, Color::White))
        );

        // Test that black rook is on a8
        assert_eq!(
            position.piece_map["a8".idx()],
            Some((Piece::Rook, Color::Black))
        );

        // Test side to move
        assert_eq!(position.side_to_move, White);

        // Test castling rights (all available)
        assert_eq!(position.castling_rights.0, 0b1111);

        // Test no en passant
        assert_eq!(position.en_passant, None);

        // // Test Verify Visually
        // for board in position.pieces {
        //     board.0.print();
        // }

        // Test get only white
        // for board in position.pieces {
        //     (board & position.side[White as usize]).0.print();
        // }
    }
}
