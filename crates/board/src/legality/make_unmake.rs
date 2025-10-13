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

/// Stores all information needed to undo a move
#[derive(Clone, Copy, Debug)]
pub struct UndoInfo {
    pub captured_piece: Option<(Piece, Color)>,
    pub castling_rights: types::others::CastleRights,
    pub en_passant: Option<u8>,
    pub half_clock: u8,

    // Store the captured square for en passant (different from move.to())
    pub ep_capture_square: Option<usize>,
}

impl Position {
    /// Makes a move in place and returns undo information
    pub fn make_move(&mut self, m: Move) -> UndoInfo {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        // Save state for undo
        let mut undo = UndoInfo {
            captured_piece: None,
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            half_clock: self.half_clock,
            ep_capture_square: None,
        };

        let (moving_piece, moving_color) = self.piece_at(from).expect("No piece at from square");

        match move_type {
            Quiet => {
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);
            }
            Capture => {
                undo.captured_piece = self.piece_at(to);
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
                undo.captured_piece = self.piece_at(captured_sq);
                undo.ep_capture_square = Some(captured_sq);

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
                undo.captured_piece = self.piece_at(to);
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

        undo
    }

    /// Unmakes a move using the undo information
    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        // Switch turns back
        self.side_to_move = self.side_to_move.flip();

        // Restore clocks
        self.half_clock = undo.half_clock;
        if self.side_to_move == Black {
            self.full_clock -= 1;
        }

        let (moving_piece, moving_color) = self.piece_at(to).unwrap_or_else(|| {
            // For promotions, the piece at 'to' is the promoted piece, not the original
            match move_type {
                PromotionQueen | CapturePromotionQueen => (Queen, self.side_to_move),
                PromotionRook | CapturePromotionRook => (Rook, self.side_to_move),
                PromotionBishop | CapturePromotionBishop => (Bishop, self.side_to_move),
                PromotionKnight | CapturePromotionKnight => (Knight, self.side_to_move),
                _ => panic!("No piece at destination square {}", to),
            }
        });

        match move_type {
            Quiet => {
                self.remove_piece(to);
                self.add_piece(from, moving_color, moving_piece);
            }
            Capture => {
                self.remove_piece(to);
                self.add_piece(from, moving_color, moving_piece);
                if let Some((piece, color)) = undo.captured_piece {
                    self.add_piece(to, color, piece);
                }
            }
            DoublePush => {
                self.remove_piece(to);
                self.add_piece(from, moving_color, moving_piece);
            }
            EnPassant => {
                self.remove_piece(to);
                self.add_piece(from, moving_color, Pawn);

                if let Some(captured_sq) = undo.ep_capture_square {
                    if let Some((piece, color)) = undo.captured_piece {
                        self.add_piece(captured_sq, color, piece);
                    }
                }
            }
            Castle => {
                // Unmove king
                self.remove_piece(to);
                self.add_piece(from, moving_color, King);

                // Unmove rook
                let (rook_from, rook_to) = match to {
                    6 => (7, 5),    // White kingside
                    2 => (0, 3),    // White queenside
                    62 => (63, 61), // Black kingside
                    58 => (56, 59), // Black queenside
                    _ => panic!("Invalid castle destination: {}", to),
                };
                self.remove_piece(rook_to);
                self.add_piece(rook_from, moving_color, Rook);
            }
            PromotionQueen | PromotionRook | PromotionBishop | PromotionKnight => {
                self.remove_piece(to);
                self.add_piece(from, moving_color, Pawn);
            }
            CapturePromotionQueen
            | CapturePromotionRook
            | CapturePromotionBishop
            | CapturePromotionKnight => {
                self.remove_piece(to);
                self.add_piece(from, moving_color, Pawn);
                if let Some((piece, color)) = undo.captured_piece {
                    self.add_piece(to, color, piece);
                }
            }
        }

        // Restore state
        self.castling_rights = undo.castling_rights;
        self.en_passant = undo.en_passant;
    }

    /// Helper function to get the promotion piece depending on the flag
    fn get_promotion_piece(&self, move_type: MoveType) -> Piece {
        match move_type {
            PromotionQueen | CapturePromotionQueen => Queen,
            PromotionRook | CapturePromotionRook => Rook,
            PromotionBishop | CapturePromotionBishop => Bishop,
            PromotionKnight | CapturePromotionKnight => Knight,
            _ => panic!("Not a promotion move"),
        }
    }

    /// Updates castling rights depending on if the king moved or the rooks did
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

#[cfg(test)]
mod make_unmake_tests {
    use crate::Position;
    use types::moves::MoveCollector;

    #[test]
    fn test_make_unmake_reversible() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        ];

        for fen in positions {
            let mut pos = Position::from_fen(fen).unwrap();
            let original = pos.clone();

            let mut collector = MoveCollector::new();
            pos.generate_moves(&mut collector);

            for i in 0..collector.len() {
                let m = collector[i];
                let undo = pos.make_move(m);
                pos.unmake_move(m, undo);

                assert_eq!(
                    pos, original,
                    "Position not restored after make/unmake for move {}",
                    m
                );
            }
        }
    }
}
