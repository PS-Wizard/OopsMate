#![allow(dead_code)]

use crate::{
    board::Board,
    piece_kind::PieceKind::{self, *},
};

pub struct Game {
    pub boards: [Board; 12],
    pub board_map: [PieceKind; 64],
    pub enpassant_square: u64,
    pub castling_rights: u8,
    /// 0 for white, 1 for black
    pub turn: u8,
}

impl Game {
    pub fn new() -> Self {
        let mut boards: [Board; 12] = [Board::empty(); 12];
        let mut board_map = [Empty; 64];

        boards[WhitePawn.idx()] = Board(0x0000_0000_0000_FF00);
        boards[WhiteRook.idx()] = Board(0x0000_0000_0000_0081);
        boards[WhiteKnight.idx()] = Board(0x0000_0000_0000_0042);
        boards[WhiteBishop.idx()] = Board(0x0000_0000_0000_0024);
        boards[WhiteKing.idx()] = Board(0x0000_0000_0000_0010);
        boards[WhiteQueen.idx()] = Board(0x0000_0000_0000_0008);

        board_map[0] = WhiteRook;
        board_map[1] = WhiteKnight;
        board_map[2] = WhiteBishop;
        board_map[3] = WhiteQueen;
        board_map[4] = WhiteKing;
        board_map[5] = WhiteBishop;
        board_map[6] = WhiteKnight;
        board_map[7] = WhiteRook;
        board_map[8..16].fill(WhitePawn);

        boards[BlackPawn.idx()] = Board(0x00FF_0000_0000_0000);
        boards[BlackRook.idx()] = Board(0x8100_0000_0000_0000);
        boards[BlackKnight.idx()] = Board(0x4200_0000_0000_0000);
        boards[BlackBishop.idx()] = Board(0x2400_0000_0000_0000);
        boards[BlackKing.idx()] = Board(0x1000_0000_0000_0000);
        boards[BlackQueen.idx()] = Board(0x0800_0000_0000_0000);

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

    #[inline(always)]
    pub fn get_board_of(&self, square: usize) -> PieceKind {
        self.board_map[square]
    }

    #[inline(always)]
    pub fn flip_turn(&mut self) {
        self.turn ^= 1;
    }

    #[inline(always)]
    pub fn get_friendlies(&self) -> u64 {
        // since turn is either 0 or 1, the range becomes either 0 to 6, or 6 to 12.
        // 0 to 6 is the white pieces and 6 to 12 is the black pieces
        let start = (self.turn as usize) * 6;
        self.boards[start + 0].0
            | self.boards[start + 1].0
            | self.boards[start + 2].0
            | self.boards[start + 3].0
            | self.boards[start + 4].0
            | self.boards[start + 5].0
    }

    #[inline(always)]
    pub fn get_enemies(&self) -> u64 {
        let start = 6 * (1 - self.turn as usize);
        self.boards[start + 0].0
            | self.boards[start + 1].0
            | self.boards[start + 2].0
            | self.boards[start + 3].0
            | self.boards[start + 4].0
            | self.boards[start + 5].0
    }

    #[inline(always)]
    pub fn get_friendly_rooks(&self) -> u64 {
        self.boards[WhiteRook.idx() + (self.turn as usize) * 6].0
    }

    #[inline(always)]
    pub fn get_all(&self) -> u64 {
        self.get_friendlies() | self.get_enemies()
    }
}

#[cfg(test)]
mod test_board {
    use crate::{game::Game, piece_kind::PieceKind::*};
    use std::time::Instant;

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
    fn test_work() {
        let game = Game::new();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn bench_board_get() {
        let game = Game::new();
        let start = Instant::now();
        let mut something: i64 = 0;
        for square in 0..64 {
            something += game.get_board_of(square) as i64 - 1;
        }
        let duration = start.elapsed();
        println!(
            "Lookups for which board contained the piece took: {:.3?} , total: 64, avg: {:.2} ns/lookup",
            duration,
            duration.as_nanos() / 64
        );
        std::hint::black_box(something);
    }
}
