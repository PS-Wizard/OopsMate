use super::UciEngine;
use crate::{Move, Position};

impl<E: crate::eval::EvalProvider> UciEngine<E> {
    pub(crate) fn parse_move(move_str: &str, pos: &Position) -> Option<Move> {
        if move_str.len() < 4 {
            return None;
        }

        let from = Self::parse_square(&move_str[0..2])?;
        let to = Self::parse_square(&move_str[2..4])?;

        let (piece, _color) = pos.piece_at(from)?;

        use crate::types::{MoveType, Piece};

        let is_capture = pos.piece_at(to).is_some();

        if move_str.len() == 5 {
            let promo = move_str.chars().nth(4)?;
            let move_type = match (promo, is_capture) {
                ('q', false) => MoveType::PromotionQueen,
                ('r', false) => MoveType::PromotionRook,
                ('b', false) => MoveType::PromotionBishop,
                ('n', false) => MoveType::PromotionKnight,
                ('q', true) => MoveType::CapturePromotionQueen,
                ('r', true) => MoveType::CapturePromotionRook,
                ('b', true) => MoveType::CapturePromotionBishop,
                ('n', true) => MoveType::CapturePromotionKnight,
                _ => return None,
            };
            return Some(Move::new(from, to, move_type));
        }

        if piece == Piece::King && ((from as i32 - to as i32).abs() == 2) {
            return Some(Move::new(from, to, MoveType::Castle));
        }

        if piece == Piece::Pawn {
            if let Some(ep_sq) = pos.en_passant {
                if to == ep_sq as usize && !is_capture {
                    return Some(Move::new(from, to, MoveType::EnPassant));
                }
            }
            if (from as i32 - to as i32).abs() == 16 {
                return Some(Move::new(from, to, MoveType::DoublePush));
            }
        }

        let move_type = if is_capture {
            MoveType::Capture
        } else {
            MoveType::Quiet
        };

        Some(Move::new(from, to, move_type))
    }

    pub(crate) fn parse_square(s: &str) -> Option<usize> {
        if s.len() != 2 {
            return None;
        }

        let file = (s.as_bytes()[0] as i32 - b'a' as i32) as usize;
        let rank = (s.as_bytes()[1] as i32 - b'1' as i32) as usize;

        if file > 7 || rank > 7 {
            return None;
        }

        Some(rank * 8 + file)
    }
}
