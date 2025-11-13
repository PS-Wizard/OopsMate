use types::moves::{
    Move,
    MoveType::{self, *},
};
use types::others::Color;
use types::others::{
    Color::*,
    Piece::{self, *},
};
use zobrist::ZOBRIST;

use crate::Position;

#[derive(Clone, Copy, Debug)]
pub struct UndoInfo {
    pub captured_piece: Option<(Piece, Color)>,
    pub castling_rights: types::others::CastleRights,
    pub en_passant: Option<u8>,
    pub half_clock: u8,
    pub ep_capture_square: Option<usize>,
    pub hash: u64,
}

impl Position {
    pub fn make_move(&mut self, m: Move) -> UndoInfo {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        let mut undo = UndoInfo {
            captured_piece: None,
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            half_clock: self.half_clock,
            ep_capture_square: None,
            hash: self.hash,
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

                if let Some(old_ep) = self.en_passant {
                    let file = (old_ep % 8) as usize;
                    self.hash ^= ZOBRIST.en_passant_key(file);
                }

                let ep_sq = if self.side_to_move == White {
                    from + 8
                } else {
                    from - 8
                };

                self.en_passant = Some(ep_sq as u8);
                let ep_file = (ep_sq % 8) as usize;
                self.hash ^= ZOBRIST.en_passant_key(ep_file);
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
                self.remove_piece(from);
                self.add_piece(to, moving_color, moving_piece);

                let (rook_from, rook_to) = match to {
                    6 => (7, 5),
                    2 => (0, 3),
                    62 => (63, 61),
                    58 => (56, 59),
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

        self.hash ^= ZOBRIST.castling_key(self.castling_rights);
        self.update_castling_rights(from, to, moving_piece, moving_color);
        self.hash ^= ZOBRIST.castling_key(self.castling_rights);

        if move_type != DoublePush {
            if let Some(old_ep) = self.en_passant {
                let file = (old_ep % 8) as usize;
                self.hash ^= ZOBRIST.en_passant_key(file);
            }
            self.en_passant = None;
        }

        self.hash ^= ZOBRIST.side_to_move;
        self.side_to_move = self.side_to_move.flip();

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

    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();

        self.side_to_move = self.side_to_move.flip();
        self.half_clock = undo.half_clock;
        if self.side_to_move == Black {
            self.full_clock -= 1;
        }

        let (moving_piece, moving_color) = self.piece_at(to).unwrap_or_else(|| match move_type {
            PromotionQueen | CapturePromotionQueen => (Queen, self.side_to_move),
            PromotionRook | CapturePromotionRook => (Rook, self.side_to_move),
            PromotionBishop | CapturePromotionBishop => (Bishop, self.side_to_move),
            PromotionKnight | CapturePromotionKnight => (Knight, self.side_to_move),
            _ => panic!("No piece at destination square {}", to),
        });

        match move_type {
            Quiet => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, moving_piece);
            }
            Capture => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, moving_piece);
                if let Some((piece, color)) = undo.captured_piece {
                    self.add_piece_silent(to, color, piece);
                }
            }
            DoublePush => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, moving_piece);
            }
            EnPassant => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, Pawn);

                if let Some(captured_sq) = undo.ep_capture_square {
                    if let Some((piece, color)) = undo.captured_piece {
                        self.add_piece_silent(captured_sq, color, piece);
                    }
                }
            }
            Castle => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, King);

                let (rook_from, rook_to) = match to {
                    6 => (7, 5),
                    2 => (0, 3),
                    62 => (63, 61),
                    58 => (56, 59),
                    _ => panic!("Invalid castle destination: {}", to),
                };
                self.remove_piece_silent(rook_to);
                self.add_piece_silent(rook_from, moving_color, Rook);
            }
            PromotionQueen | PromotionRook | PromotionBishop | PromotionKnight => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, Pawn);
            }
            CapturePromotionQueen
            | CapturePromotionRook
            | CapturePromotionBishop
            | CapturePromotionKnight => {
                self.remove_piece_silent(to);
                self.add_piece_silent(from, moving_color, Pawn);
                if let Some((piece, color)) = undo.captured_piece {
                    self.add_piece_silent(to, color, piece);
                }
            }
        }

        self.castling_rights = undo.castling_rights;
        self.en_passant = undo.en_passant;
        self.hash = undo.hash;
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
        if piece == King {
            if color == White {
                self.castling_rights.remove_white_castling();
            } else {
                self.castling_rights.remove_black_castling();
            }
        }

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

                println!("Zobrist For Original: {}",original.hash);
                assert_eq!(
                    pos, original,
                    "Position not restored after make/unmake for move {}",
                    m
                );
            }
        }
    }
}
