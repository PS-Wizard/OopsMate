#![allow(dead_code)]
use handies::constants::BoardIdx::{self, *};

/// Holds The Game State
/// - boards: 12 bitboards for all piece types for each color
/// - board_map: a [Piece;64] array for fast lookup of the relevant board that has the piece
struct Game {
    boards: [u64; 12],
    board_map: [BoardIdx; 64],
    enpassant_square: u64,
    castling_rights: u8,
    turn: u8,
}

impl Game {
    fn new() -> Self {
        let mut boards: [u64; 12] = [0; 12];
        let mut board_map = [BoardIdx::Empty; 64];

        boards[WhitePawn.idx()] = 0x0000_0000_0000_FF00;
        boards[WhiteRook.idx()] = 0x0000_0000_0000_0081;
        boards[WhiteKnight.idx()] = 0x0000_0000_0000_0042;
        boards[WhiteBishop.idx()] = 0x0000_0000_0000_0024;
        boards[WhiteKing.idx()] = 0x0000_0000_0000_0010;
        boards[WhiteQueen.idx()] = 0x0000_0000_0000_0008;
        board_map[0] = WhiteRook;
        board_map[1] = WhiteKnight;
        board_map[2] = WhiteBishop;
        board_map[3] = WhiteQueen;
        board_map[4] = WhiteKing;
        board_map[5] = WhiteBishop;
        board_map[6] = WhiteKnight;
        board_map[7] = WhiteRook;
        board_map[8..16].fill(WhitePawn);

        boards[BlackPawn.idx()] = 0x00FF_0000_0000_0000;
        boards[BlackRook.idx()] = 0x8100_0000_0000_0000;
        boards[BlackKnight.idx()] = 0x4200_0000_0000_0000;
        boards[BlackBishop.idx()] = 0x2400_0000_0000_0000;
        boards[BlackKing.idx()] = 0x1000_0000_0000_0000;
        boards[BlackQueen.idx()] = 0x0800_0000_0000_0000;
        board_map[56] = BlackRook;
        board_map[57] = BlackKnight;
        board_map[58] = BlackBishop;
        board_map[59] = BlackQueen;
        board_map[60] = BlackKing;
        board_map[61] = BlackBishop;
        board_map[62] = BlackKnight;
        board_map[63] = BlackRook;
        board_map[48..56].fill(BlackPawn);

        Self {
            boards,
            board_map,
            castling_rights: 0b1111,
            enpassant_square: 0,
            turn: 0,
        }
    }

    fn get_board_of(&self, piece_at: usize) -> BoardIdx {
        self.board_map[piece_at]
    }
}

#[cfg(test)]
mod test_board {
    use std::time::Instant;

    use handies::constants::BoardIdx::*;

    use crate::game::Game;

    #[test]
    fn test_get_board() {
        let game = Game::new();
        assert_eq!(game.get_board_of(0), WhiteRook);
        assert_eq!(game.get_board_of(32), Empty);
        assert_eq!(game.get_board_of(56), BlackRook);
        assert_eq!(game.get_board_of(55), BlackPawn);
        assert_eq!(game.get_board_of(60), BlackKing);
        assert_eq!(game.get_board_of(4), WhiteKing);
    }

    #[test]
    fn bench_board_get() {
        let game = Game::new();
        let start = Instant::now();
        let mut something: i64 = 0;
        for square in 0..64 {
            something += game.get_board_of(square) as i64 - 1;
        }
        let duration = start.elapsed();
        println!(
            "Lookups for Corresponding table took: {:.3?} , total: 64, avg: {:.2} ns/lookup",
            duration,
            duration.as_nanos() / 64
        );
        std::hint::black_box(something);
    }
}
