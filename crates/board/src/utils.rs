use crate::game::Game;
use crate::{board::Board, piece::Color, piece::PieceType};

pub fn get_board_mut<'a>(game: &'a mut Game, piece_type: PieceType, color: Color) -> &'a mut Board {
    match (piece_type, color) {
        (PieceType::Pawn, Color::White) => &mut game.white_pawns,
        (PieceType::Rook, Color::White) => &mut game.white_rooks,
        (PieceType::Knight, Color::White) => &mut game.white_knights,
        (PieceType::Bishop, Color::White) => &mut game.white_bishops,
        (PieceType::Queen, Color::White) => &mut game.white_queens,
        (PieceType::King, Color::White) => &mut game.white_king,
        (PieceType::Pawn, Color::Black) => &mut game.black_pawns,
        (PieceType::Rook, Color::Black) => &mut game.black_rooks,
        (PieceType::Knight, Color::Black) => &mut game.black_knights,
        (PieceType::Bishop, Color::Black) => &mut game.black_bishops,
        (PieceType::Queen, Color::Black) => &mut game.black_queens,
        (PieceType::King, Color::Black) => &mut game.black_king,
    }
}
