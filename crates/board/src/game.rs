#![allow(dead_code)]
use crate::{
    board::Board,
    piece::{Color, PieceType},
    utils,
};

pub struct Game {
    pub white_pawns: Board,
    pub white_rooks: Board,
    pub white_knights: Board,
    pub white_bishops: Board,
    pub white_queens: Board,
    pub white_king: Board,

    pub black_pawns: Board,
    pub black_rooks: Board,
    pub black_knights: Board,
    pub black_bishops: Board,
    pub black_queens: Board,
    pub black_king: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            white_pawns: Board(0x0000_0000_0000_FF00),
            white_knights: Board(0x0000_0000_0000_0042),
            white_bishops: Board(0x0000_0000_0000_0024),
            white_rooks: Board(0x0000_0000_0000_0081),
            white_queens: Board(0x0000_0000_0000_0008),
            white_king: Board(0x0000_0000_0000_0010),

            // Black pieces on ranks 7 and 8
            black_pawns: Board(0x00FF_0000_0000_0000),
            black_knights: Board(0x4200_0000_0000_0000),
            black_bishops: Board(0x2400_0000_0000_0000),
            black_rooks: Board(0x8100_0000_0000_0000),
            black_queens: Board(0x0800_0000_0000_0000),
            black_king: Board(0x1000_0000_0000_0000),
        }
    }

    pub fn black_pieces(&self) -> Board {
        self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king
    }

    pub fn white_pieces(&self) -> Board {
        self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king
    }

    pub fn all_pieces(&self) -> Board {
        self.black_pieces() | self.white_pieces()
    }

    pub fn get_piece_at(&self, pos: usize) -> Option<(PieceType, Color)> {
        if pos >= 64 {
            return None;
        }

        let boards = [
            (&self.white_pawns, PieceType::Pawn, Color::White),
            (&self.white_rooks, PieceType::Rook, Color::White),
            (&self.white_knights, PieceType::Knight, Color::White),
            (&self.white_bishops, PieceType::Bishop, Color::White),
            (&self.white_queens, PieceType::Queen, Color::White),
            (&self.white_king, PieceType::King, Color::White),
            (&self.black_pawns, PieceType::Pawn, Color::Black),
            (&self.black_rooks, PieceType::Rook, Color::Black),
            (&self.black_knights, PieceType::Knight, Color::Black),
            (&self.black_bishops, PieceType::Bishop, Color::Black),
            (&self.black_queens, PieceType::Queen, Color::Black),
            (&self.black_king, PieceType::King, Color::Black),
        ];

        for (board, piece_type, color) in boards {
            if (board.0 >> pos) & 1 != 0 {
                return Some((piece_type, color));
            }
        }

        None
    }

    pub fn make_move(&mut self, from: usize, to: usize) -> Result<(), &str> {
        if from >= 64 || to >= 64 {
            return Err("Square out of bounds");
        }

        let piece_opt = self.get_piece_at(from);
        if piece_opt.is_none() {
            return Err("No piece at source square");
        }
        let (piece_type, color) = piece_opt.unwrap();
        let board = utils::get_board_mut(self, piece_type, color);

        board.0 &= !(1u64 << from); // clear from
        board.0 |= 1u64 << to; // set to

        let enemy_boards = match color {
            Color::White => [
                &mut self.black_pawns,
                &mut self.black_rooks,
                &mut self.black_knights,
                &mut self.black_bishops,
                &mut self.black_queens,
                &mut self.black_king,
            ],
            Color::Black => [
                &mut self.white_pawns,
                &mut self.white_rooks,
                &mut self.white_knights,
                &mut self.white_bishops,
                &mut self.white_queens,
                &mut self.white_king,
            ],
        };

        for b in enemy_boards {
            b.0 &= !(1u64 << to);
        }

        Ok(())
    }

    pub fn print_board(&self) {
        for rank in (0..8).rev() {
            // ranks 8..1
            print!("{} ", rank + 1);
            for file in 0..8 {
                // files a..h
                let sq = rank * 8 + file;
                if let Some((pt, color)) = self.get_piece_at(sq) {
                    let piece = crate::piece::Piece::new(pt, color);
                    print!("{} ", piece);
                } else {
                    print!(". "); // empty square
                }
            }
            println!();
        }
        println!("  a b c d e f g h\n");
    }
}
#[cfg(test)]
mod game_tests {
    use std::time::Instant;

    use super::{Color, Game, PieceType};

    #[test]
    fn test_initial_positions() {
        let g = Game::new();

        // White pawns on rank 2 (bits 8..15)
        for i in 8..16 {
            assert_eq!(g.get_piece_at(i), Some((PieceType::Pawn, Color::White)));
        }

        // Black pawns on rank 7 (bits 48..55)
        for i in 48..56 {
            assert_eq!(g.get_piece_at(i), Some((PieceType::Pawn, Color::Black)));
        }

        // Kings at correct squares
        assert_eq!(g.get_piece_at(4), Some((PieceType::King, Color::White)));
        assert_eq!(g.get_piece_at(60), Some((PieceType::King, Color::Black)));

        g.print_board();
    }
    #[test]
    fn test_bench_speed() {
        use std::hint::black_box;
        use std::time::Instant;

        let g = Game::new();
        let mut acc: u64 = 0; // accumulate something so compiler can't remove the loop

        let start = Instant::now();
        for sq in 0..64 {
            if let Some((piece_type, color)) = g.get_piece_at(sq) {
                // combine values in a trivial way
                acc += piece_type as u64 + color as u64;
            }
        }
        // prevent compiler from optimizing away the loop entirely
        black_box(acc);

        let duration = start.elapsed();
        println!(
            "Lookups for Corresponding table took: {:.3?}, total: 64, avg: {:.2} ns/lookup",
            duration,
            duration.as_nanos() as f64 / 64.0
        );
    }

    #[test]
    fn test_make_move_basic() {
        let mut g = Game::new();
        g.print_board();

        // Move white pawn from e2 (12) to e4 (28)
        g.make_move(12, 28).unwrap();
        assert_eq!(g.get_piece_at(12), None);
        assert_eq!(g.get_piece_at(28), Some((PieceType::Pawn, Color::White)));
        g.print_board();

        // Move black pawn from e7 (52) to e5 (36)
        g.make_move(52, 36).unwrap();
        assert_eq!(g.get_piece_at(52), None);
        assert_eq!(g.get_piece_at(36), Some((PieceType::Pawn, Color::Black)));
        g.print_board();
    }

    #[test]
    fn test_capture() {
        let mut g = Game::new();
        g.print_board();

        // White pawn captures black pawn
        g.make_move(12, 52).unwrap();
        assert_eq!(g.get_piece_at(12), None);
        assert_eq!(g.get_piece_at(52), Some((PieceType::Pawn, Color::White)));
        g.print_board();
    }

    #[test]
    fn test_no_piece_move() {
        let mut g = Game::new();
        assert!(g.make_move(20, 21).is_err()); // empty square
    }
}
