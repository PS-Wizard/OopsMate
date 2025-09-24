#![allow(dead_code)]
use types::{
    board::Board,
    piece_kind::{
        Piece,
        PieceKind::{self, *},
    },
};

pub struct Game {
    pub boards: [Board; 12],
    pub board_map: [PieceKind; 64],
    pub enpassant_sq: u8,
    pub castling_rights: u8,
    pub turn: u8,
}

impl Game {
    pub fn new() -> Self {
        let mut boards: [Board; 12] = [Board::empty(); 12];
        let mut board_map = [None; 64];

        boards[WhitePawn] = Board(0x0000_0000_0000_FF00);
        boards[WhiteRook] = Board(0x0000_0000_0000_0081);
        boards[WhiteKnight] = Board(0x0000_0000_0000_0042);
        boards[WhiteBishop] = Board(0x0000_0000_0000_0024);
        boards[WhiteKing] = Board(0x0000_0000_0000_0010);
        boards[WhiteQueen] = Board(0x0000_0000_0000_0008);

        board_map[0] = WhiteRook;
        board_map[1] = WhiteKnight;
        board_map[2] = WhiteBishop;
        board_map[3] = WhiteQueen;
        board_map[4] = WhiteKing;
        board_map[5] = WhiteBishop;
        board_map[6] = WhiteKnight;
        board_map[7] = WhiteRook;
        board_map[8..16].fill(WhitePawn);

        boards[BlackPawn] = Board(0x00FF_0000_0000_0000);
        boards[BlackRook] = Board(0x8100_0000_0000_0000);
        boards[BlackKnight] = Board(0x4200_0000_0000_0000);
        boards[BlackBishop] = Board(0x2400_0000_0000_0000);
        boards[BlackKing] = Board(0x1000_0000_0000_0000);
        boards[BlackQueen] = Board(0x0800_0000_0000_0000);

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
            enpassant_sq: 0,
            castling_rights: 0b1111,

            turn: 0,
        }
    }

    #[inline(always)]
    pub fn flip_turn(&mut self) {
        self.turn ^= 1;
    }

    #[inline(always)]
    pub fn friendly_board(&self, piece: Piece) -> u64 {
        self.boards[(piece as usize) + (self.turn as usize * 6)].0
    }

    #[inline(always)]
    pub fn piece_at(&self, idx: usize) -> PieceKind {
        self.board_map[idx]
    }

    #[inline(always)]
    pub fn friendly_occupied(&self) -> u64 {
        self.friendly_board(Piece::Pawns)
            | self.friendly_board(Piece::Rooks)
            | self.friendly_board(Piece::Knights)
            | self.friendly_board(Piece::Bishops)
            | self.friendly_board(Piece::Queens)
            | self.friendly_board(Piece::King)
    }

    #[inline(always)]
    pub fn enemy_occupied(&self) -> u64 {
        let enemy_turn = self.turn ^ 1; // 0->1, 1->0
        self.boards[(Piece::Pawns as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::Rooks as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::Knights as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::Bishops as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::Queens as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::King as usize) + (enemy_turn as usize * 6)].0
    }

    #[inline(always)]
    pub fn occupied(&self) -> u64 {
        self.friendly_occupied() | self.enemy_occupied()
    }
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod test_game {
    use crate::game::Game;
    use types::piece_kind::Piece::*;
    use utilities::board::PrintAsBoard;

    #[test]
    fn test_get_friendlies() {
        let mut g = Game::new();
        println!("Whites: -----");
        println!("Pawns:");
        g.friendly_board(Pawns).print();
        println!("Rooks:");
        g.friendly_board(Rooks).print();
        println!("Knights:");
        g.friendly_board(Knights).print();
        println!("Bishops:");
        g.friendly_board(Bishops).print();
        println!("Queens:");
        g.friendly_board(Queens).print();
        println!("King:");
        g.friendly_board(King).print();
        g.flip_turn();
        println!("Blacks: -----");
        println!("Pawns:");
        g.friendly_board(Pawns).print();
        println!("Rooks:");
        g.friendly_board(Rooks).print();
        println!("Knights:");
        g.friendly_board(Knights).print();
        println!("Bishops:");
        g.friendly_board(Bishops).print();
        println!("Queens:");
        g.friendly_board(Queens).print();
        println!("King:");
        g.friendly_board(King).print();
    }
}
