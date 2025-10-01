use types::moves::{
    Move,
    MoveType::{self, *},
};
use types::others::Color;
use types::others::{
    Color::*,
    Piece::{self, *},
};

use crate::Position;

impl Position {
    #[inline(always)]
    pub fn piece_at(&self, idx: usize) -> Option<(Piece, Color)> {
        self.piece_map[idx]
    }

    pub fn make_move(&self, m: Move) -> Position {
        let mut new_pos = self.clone();
        new_pos.apply_move(m);
        new_pos
    }

    fn apply_move(&mut self, m: Move) {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        let (moving_piece, moving_color) = self.piece_at(from).expect("No piece at from square");

        match move_type {
            Quiet => {
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);
            }
            Capture => {
                self.remove_piece(to);
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);
            }
            DoublePush => {
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);
                // Set en passant square
                let ep_sq = if self.side_to_move == White {
                    from + 8
                } else {
                    from - 8
                };
                self.en_passant = Some(ep_sq as u8);
            }
            EnPassant => {
                let captured_sq = if self.side_to_move == White {
                    to - 8
                } else {
                    to + 8
                };
                self.remove_piece(captured_sq);
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);
            }
            Castle => {
                // Move king
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);

                // Move rook
                let (rook_from, rook_to) = match to {
                    6 => (7, 5),    // White kingside
                    2 => (0, 3),    // White queenside
                    62 => (63, 61), // Black kingside
                    58 => (56, 59), // Black queenside
                    _ => panic!("Invalid castle destination: {}", to),
                };
                // Move rook
                self.remove_piece(rook_from);
                self.add_piece(rook_to, moving_color, Rook);
            }
            PromotionQueen | PromotionRook | PromotionBishop | PromotionKnight => {
                self.remove_piece(from);
                let promoted = self.get_promotion_piece(move_type);
                self.add_piece(to, moving_color, promoted);
            }
            CapturePromotionQueen
            | CapturePromotionRook
            | CapturePromotionBishop
            | CapturePromotionKnight => {
                self.remove_piece(to);
                self.remove_piece(from);
                let promoted = self.get_promotion_piece(move_type);
                self.add_piece(to, moving_color, promoted);
            }
        }

        // Update castling rights
        self.update_castling_rights(from, to, moving_piece, moving_color);

        // Clear en passant if not a double push
        if move_type != DoublePush {
            self.en_passant = None;
        }

        // Switch turns
        self.side_to_move = self.side_to_move.flip();

        // Update clocks
        if matches!(moving_piece, Pawn)
            || matches!(
                move_type,
                Capture
                    | CapturePromotionQueen
                    | CapturePromotionRook
                    | CapturePromotionBishop
                    | CapturePromotionKnight
                    | EnPassant
            )
        {
            self.half_clock = 0;
        } else {
            self.half_clock += 1;
        }

        if self.side_to_move == White {
            self.full_clock += 1;
        }
    }

    fn get_promotion_piece(&self, move_type: MoveType) -> Piece {
        match move_type {
            PromotionQueen | CapturePromotionQueen => Queen,
            PromotionRook | CapturePromotionRook => Rook,
            PromotionBishop | CapturePromotionBishop => Bishop,
            PromotionKnight | CapturePromotionKnight => Knight,
            _ => panic!("Not a promotion move"),
        }
    }

    fn update_castling_rights(&mut self, from: usize, to: usize, piece: Piece, color: Color) {
        // King moves lose all castling rights for that color
        if piece == King {
            if color == White {
                self.castling_rights.remove_white_castling();
            } else {
                self.castling_rights.remove_black_castling();
            }
        }

        // Rook moves or captures lose specific castling rights
        match from {
            0 => self.castling_rights.remove_white_queenside(),
            7 => self.castling_rights.remove_white_kingside(),
            56 => self.castling_rights.remove_black_queenside(),
            63 => self.castling_rights.remove_black_kingside(),
            _ => {}
        }

        match to {
            0 => self.castling_rights.remove_white_queenside(),
            7 => self.castling_rights.remove_white_kingside(),
            56 => self.castling_rights.remove_black_queenside(),
            63 => self.castling_rights.remove_black_kingside(),
            _ => {}
        }
    }
}
