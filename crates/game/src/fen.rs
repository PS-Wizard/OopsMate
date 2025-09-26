#![allow(dead_code)]

use crate::{board::Board, game::Game, piece::PieceKind};
use utilities::algebraic::Algebraic;

impl Game { /// Parses FEN notation into a new `Game`
    pub fn from_fen(fen: &str) -> Self {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        let mut game = Self {
            boards: [Board::empty(); 12],
            piece_map: [PieceKind::None; 64],
            turn: if parts[1] == "w" { 0 } else { 1 },
            castling_rights: Self::parse_castling_rights(parts[2]),
            en_passant: Self::parse_en_passant(parts[3]),
            halfmove_clock: parts[4].parse().unwrap_or(0),
            fullmove: parts[5].parse().unwrap_or(1),
            white_occupied: 0,
            black_occupied: 0,
        };

        // Parse board position
        game.parse_position(parts[0]);
        game.update_occupied_boards();
        game
    }

    /// Parses FEN Position
    fn parse_position(&mut self, position: &str) {
        let mut rank = 7u8; 
        let mut file = 0u8;

        for ch in position.chars() {
            match ch {
                '/' => {
                    rank = rank.saturating_sub(1);
                    file = 0;
                }
                '1'..='8' => file += ch.to_digit(10).unwrap() as u8,
                piece_char => {
                    let square = rank * 8 + file;
                    let piece = PieceKind::from_char(piece_char);
                    self.piece_map[square as usize] = piece;

                    let board_idx = piece as usize;
                    if board_idx < 12 {
                        self.boards[board_idx].set_bit(square);
                    }
                    file += 1;
                }
            }
        }
    }

    /// Updates the white_occupied & black_occupied bitboards
    fn update_occupied_boards(&mut self) {
        let white_occupied = (0..6).fold(0, |acc, i| acc | self.boards[i].0);
        let black_occupied = (6..12).fold(0, |acc, i| acc | self.boards[i].0);

        if self.turn == 0 {
            // White to move
            self.white_occupied = white_occupied; // White = friendly
            self.black_occupied = black_occupied; // Black = enemy
        } else {
            // Black to move
            self.white_occupied = black_occupied; // Black = friendly
            self.black_occupied = white_occupied; // White = enemy
        }
    }

    /// Pretty Self Explainatory
    pub fn parse_castling_rights(castling: &str) -> u8 {
        let mut rights = 0u8;
        if castling.contains('K') {
            rights |= 0b0001;
        } // White kingside
        if castling.contains('Q') {
            rights |= 0b0010;
        } // White queenside
        if castling.contains('k') {
            rights |= 0b0100;
        } // Black kingside
        if castling.contains('q') {
            rights |= 0b1000;
        } // Black queenside
        rights
    }

    /// Pretty Self Explainatory
    pub fn parse_en_passant(en_passant: &str) -> u8 {
        if en_passant == "-" {
            0
        } else {
            en_passant.idx() as u8
        }
    }
}

#[cfg(test)]
mod test_fen {
    use utilities::board::PrintAsBoard;
    use crate::game::Game;

    #[test]
    #[cfg(debug_assertions)]
    fn test_initial() {
        let g = Game::new();
        g.white_occupied.print();
        g.black_occupied.print();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_from_position() {
        let g = Game::from_position("rnbqk1nr/pppp2pp/5p2/4p3/1b2P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3");
        println!("Enemy:");
        g.black_occupied.print();
        println!("Friendly:");
        g.white_occupied.print();
    }
}
