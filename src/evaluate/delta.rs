use super::{
    mapping::{map_piece, promotion_piece},
    EvalMoveDelta, EvalProbe,
};
use crate::{Color, Move, MoveType, Piece, Position};

#[inline(always)]
pub fn apply_move(probe: &mut EvalProbe, pos: &Position, mv: Move) -> EvalMoveDelta {
    let delta = build_move_delta(pos, mv);
    probe.apply_delta(delta);
    delta
}

#[inline(always)]
pub fn undo_move(probe: &mut EvalProbe, delta: EvalMoveDelta) {
    probe.undo_delta(delta);
}

#[inline(always)]
pub fn apply_null_move(probe: &mut EvalProbe, pos: &Position) {
    probe.make_null_move_with_rule50(pos.halfmove as i32 + 1);
}

#[inline(always)]
pub fn undo_null_move(probe: &mut EvalProbe) {
    probe.unmake_null_move();
}

fn build_move_delta(pos: &Position, mv: Move) -> EvalMoveDelta {
    let from = mv.from();
    let to = mv.to();
    let move_type = mv.move_type();
    let (piece, color) = pos
        .piece_at(from)
        .expect("expected moving piece on source square");
    let moving_piece = map_piece(piece, color);
    let capture_resets = mv.is_capture();
    let next_rule50 = if piece == Piece::Pawn || capture_resets {
        0
    } else {
        pos.halfmove as i32 + 1
    };
    let mut delta = EvalMoveDelta::new(next_rule50);

    match move_type {
        MoveType::Quiet | MoveType::DoublePush => {
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("quiet move should fit inside MoveDelta");
        }
        MoveType::Capture => {
            let (captured, captured_color) = pos
                .piece_at(to)
                .expect("expected captured piece on destination square");
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("capture move should fit inside MoveDelta");
            delta
                .push_removal(to, map_piece(captured, captured_color))
                .expect("capture move should fit inside MoveDelta");
        }
        MoveType::EnPassant => {
            let capture_sq = if color == Color::White {
                to - 8
            } else {
                to + 8
            };
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("en passant move should fit inside MoveDelta");
            delta
                .push_removal(capture_sq, map_piece(Piece::Pawn, color.flip()))
                .expect("en passant move should fit inside MoveDelta");
        }
        MoveType::Castle => {
            let (rook_from, rook_to) = match to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                58 => (56, 59),
                _ => panic!("invalid castle move"),
            };
            let rook_piece = map_piece(Piece::Rook, color);
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("castle move should fit inside MoveDelta");
            delta
                .push_move(rook_from, rook_to, rook_piece, rook_piece)
                .expect("castle move should fit inside MoveDelta");
        }
        MoveType::PromotionKnight
        | MoveType::PromotionBishop
        | MoveType::PromotionRook
        | MoveType::PromotionQueen => {
            delta
                .push_move(from, to, moving_piece, promotion_piece(move_type, color))
                .expect("promotion move should fit inside MoveDelta");
        }
        MoveType::CapturePromotionKnight
        | MoveType::CapturePromotionBishop
        | MoveType::CapturePromotionRook
        | MoveType::CapturePromotionQueen => {
            let (captured, captured_color) = pos
                .piece_at(to)
                .expect("expected captured piece on destination square");
            delta
                .push_move(from, to, moving_piece, promotion_piece(move_type, color))
                .expect("capture promotion move should fit inside MoveDelta");
            delta
                .push_removal(to, map_piece(captured, captured_color))
                .expect("capture promotion move should fit inside MoveDelta");
        }
    }

    delta
}
