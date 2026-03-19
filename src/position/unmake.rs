use super::Position;
use crate::{
    types::{Move, MoveType, Piece},
    Color,
};

impl Position {
    #[inline(always)]
    /// Reverts the most recently made null move.
    pub fn unmake_null_move(&mut self) {
        let state = self.history.pop().expect("No history to unmake null move");
        self.en_passant = state.en_passant;
        self.halfmove = state.halfmove;
        self.hash = state.hash;
        self.castling_rights = state.castling_rights;
        self.side_to_move = self.side_to_move.flip();
    }

    #[inline(always)]
    /// Reverts the most recently made legal move.
    pub fn unmake_move(&mut self, m: Move) {
        let state = self.history.pop().expect("No history to unmake");

        self.castling_rights = state.castling_rights;
        self.en_passant = state.en_passant;
        self.halfmove = state.halfmove;
        self.hash = state.hash;

        let to = m.to();
        let from = m.from();
        let move_type = m.move_type();

        self.side_to_move = self.side_to_move.flip();
        let color = self.side_to_move;

        if color == Color::Black {
            self.fullmove -= 1;
        }

        match move_type {
            MoveType::Quiet => {
                let (piece, _) = self.piece_at(to).unwrap();
                self.move_piece(to, from, color, piece);
            }
            MoveType::Capture => {
                let (piece, _) = self.piece_at(to).unwrap();
                self.move_piece(to, from, color, piece);

                let captured = state.captured_piece.unwrap();
                self.add_piece(to, color.flip(), captured);
            }
            MoveType::DoublePush => {
                let (piece, _) = self.piece_at(to).unwrap();
                self.move_piece(to, from, color, piece);
            }
            MoveType::EnPassant => {
                let (piece, _) = self.piece_at(to).unwrap();
                self.move_piece(to, from, color, piece);

                let capture_sq = if color == Color::White {
                    to - 8
                } else {
                    to + 8
                };
                self.add_piece(capture_sq, color.flip(), Piece::Pawn);
            }
            MoveType::Castle => {
                let (piece, _) = self.piece_at(to).unwrap();
                self.move_piece(to, from, color, piece);

                let (rook_from, rook_to) = match to {
                    6 => (7, 5),
                    2 => (0, 3),
                    62 => (63, 61),
                    58 => (56, 59),
                    _ => panic!("Invalid castle"),
                };
                self.move_piece(rook_to, rook_from, color, Piece::Rook);
            }
            MoveType::PromotionKnight
            | MoveType::PromotionBishop
            | MoveType::PromotionRook
            | MoveType::PromotionQueen => {
                self.remove_piece(to);
                self.add_piece(from, color, Piece::Pawn);
            }
            MoveType::CapturePromotionKnight
            | MoveType::CapturePromotionBishop
            | MoveType::CapturePromotionRook
            | MoveType::CapturePromotionQueen => {
                self.remove_piece(to);
                self.add_piece(from, color, Piece::Pawn);

                let captured = state.captured_piece.unwrap();
                self.add_piece(to, color.flip(), captured);
            }
        }
    }
}
