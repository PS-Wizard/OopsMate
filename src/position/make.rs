use super::{GameState, Position};
use crate::{
    types::{Move, MoveType, Piece},
    zobrist::{CASTLE_KEYS, EP_KEYS, PIECE_KEYS, SIDE_KEY},
    Color,
};

impl Position {
    #[inline(always)]
    pub fn make_move(&mut self, m: Move) {
        let from = m.from();
        let to = m.to();
        let move_type = m.move_type();
        let (piece, color) = self.piece_at(from).expect("No piece at from");
        let (captured_piece, _) = if m.is_capture() {
            if move_type == MoveType::EnPassant {
                (Some(Piece::Pawn), Color::White)
            } else {
                let (p, c) = self.piece_at(to).unwrap();
                (Some(p), c)
            }
        } else {
            (None, Color::White)
        };

        self.history.push(GameState {
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove: self.halfmove,
            hash: self.hash,
            captured_piece,
        });

        self.hash ^= CASTLE_KEYS[self.castling_rights.0 as usize];
        if let Some(ep) = self.en_passant {
            self.hash ^= EP_KEYS[(ep % 8) as usize];
        }

        match move_type {
            MoveType::Quiet => {
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.move_piece(from, to, color, piece);
            }
            MoveType::Capture => {
                let (captured_piece, captured_color) = self.piece_at(to).unwrap();
                self.hash ^= PIECE_KEYS[captured_color as usize][captured_piece as usize][to];
                self.remove_piece(to);
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.move_piece(from, to, color, piece);
            }
            MoveType::DoublePush => {
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.move_piece(from, to, color, piece);
                self.en_passant = Some(((from + to) / 2) as u8);
            }
            MoveType::EnPassant => {
                let captured = if color == Color::White {
                    to - 8
                } else {
                    to + 8
                };
                self.hash ^= PIECE_KEYS[color.flip() as usize][Piece::Pawn as usize][captured];
                self.remove_piece(captured);
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.move_piece(from, to, color, piece);
            }
            MoveType::Castle => {
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][from];
                self.hash ^= PIECE_KEYS[color as usize][piece as usize][to];
                self.move_piece(from, to, color, piece);

                let (rook_from, rook_to) = match to {
                    6 => (7, 5),
                    2 => (0, 3),
                    62 => (63, 61),
                    58 => (56, 59),
                    _ => panic!("Invalid castle"),
                };
                self.hash ^= PIECE_KEYS[color as usize][Piece::Rook as usize][rook_from];
                self.hash ^= PIECE_KEYS[color as usize][Piece::Rook as usize][rook_to];
                self.move_piece(rook_from, rook_to, color, Piece::Rook);
            }
            MoveType::PromotionKnight
            | MoveType::PromotionBishop
            | MoveType::PromotionRook
            | MoveType::PromotionQueen => {
                self.hash ^= PIECE_KEYS[color as usize][Piece::Pawn as usize][from];
                self.remove_piece(from);
                let promoted = match move_type {
                    MoveType::PromotionKnight => Piece::Knight,
                    MoveType::PromotionBishop => Piece::Bishop,
                    MoveType::PromotionRook => Piece::Rook,
                    MoveType::PromotionQueen => Piece::Queen,
                    _ => unreachable!(),
                };
                self.hash ^= PIECE_KEYS[color as usize][promoted as usize][to];
                self.add_piece(to, color, promoted);
            }
            MoveType::CapturePromotionKnight
            | MoveType::CapturePromotionBishop
            | MoveType::CapturePromotionRook
            | MoveType::CapturePromotionQueen => {
                let (captured_piece, captured_color) = self.piece_at(to).unwrap();
                self.hash ^= PIECE_KEYS[captured_color as usize][captured_piece as usize][to];
                self.remove_piece(to);
                self.hash ^= PIECE_KEYS[color as usize][Piece::Pawn as usize][from];
                self.remove_piece(from);
                let promoted = match move_type {
                    MoveType::CapturePromotionKnight => Piece::Knight,
                    MoveType::CapturePromotionBishop => Piece::Bishop,
                    MoveType::CapturePromotionRook => Piece::Rook,
                    MoveType::CapturePromotionQueen => Piece::Queen,
                    _ => unreachable!(),
                };
                self.hash ^= PIECE_KEYS[color as usize][promoted as usize][to];
                self.add_piece(to, color, promoted);
            }
        }

        if piece == Piece::King {
            self.castling_rights.remove_color(color);
        }
        self.castling_rights.remove_rook(from);
        self.castling_rights.remove_rook(to);
        self.hash ^= CASTLE_KEYS[self.castling_rights.0 as usize];

        if piece == Piece::Pawn || m.is_capture() {
            self.halfmove = 0;
        } else {
            self.halfmove += 1;
        }

        if self.side_to_move == Color::Black {
            self.fullmove += 1;
        }

        if move_type != MoveType::DoublePush {
            self.en_passant = None;
        }
        if let Some(ep) = self.en_passant {
            self.hash ^= EP_KEYS[(ep % 8) as usize];
        }

        self.hash ^= SIDE_KEY;
        self.side_to_move = self.side_to_move.flip();
    }

    #[inline(always)]
    pub fn make_null_move(&mut self) {
        self.history.push(GameState {
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove: self.halfmove,
            hash: self.hash,
            captured_piece: None,
        });

        if let Some(ep) = self.en_passant {
            self.hash ^= EP_KEYS[(ep % 8) as usize];
            self.en_passant = None;
        }

        self.halfmove += 1;
        self.hash ^= SIDE_KEY;
        self.side_to_move = self.side_to_move.flip();
    }
}
