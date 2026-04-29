use super::analysis::Analysis;
use super::attacks::is_square_attacked_with_blockers;
use super::stage::{include_captures, include_quiets};
use crate::{Color, Move, MoveCollector, MoveType, Position};
use strikes::KING_ATTACKS;

#[inline(always)]
pub(super) fn generate<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let king_sq = analysis.king_sq;
    let blockers_without_king = analysis.occ & !(1u64 << king_sq);
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut attacks = KING_ATTACKS[king_sq] & !analysis.us_occ & !enemy_king;

    while attacks != 0 {
        let to = attacks.trailing_zeros() as usize;
        attacks &= attacks - 1;

        if is_square_attacked_with_blockers(pos, to, analysis.them, blockers_without_king) {
            continue;
        }

        let is_capture = (analysis.them_occ >> to) & 1 != 0;
        if is_capture {
            if include_captures::<STAGE>() {
                collector.push(Move::new(king_sq, to, MoveType::Capture));
            }
        } else if include_quiets::<STAGE>() {
            collector.push(Move::new(king_sq, to, MoveType::Quiet));
        }
    }

    if include_quiets::<STAGE>() && !analysis.in_check() {
        generate_castling(pos, analysis, collector, king_sq);
    }
}

fn generate_castling(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
    king_sq: usize,
) {
    let occupied = analysis.occ;

    match pos.side_to_move {
        Color::White => {
            if pos.castling_rights.can_castle_kingside(Color::White)
                && (occupied & 0x60) == 0
                && !pos.is_square_attacked(5, analysis.them)
                && !pos.is_square_attacked(6, analysis.them)
            {
                collector.push(Move::new(king_sq, 6, MoveType::Castle));
            }
            if pos.castling_rights.can_castle_queenside(Color::White)
                && (occupied & 0x0E) == 0
                && !pos.is_square_attacked(3, analysis.them)
                && !pos.is_square_attacked(2, analysis.them)
            {
                collector.push(Move::new(king_sq, 2, MoveType::Castle));
            }
        }
        Color::Black => {
            if pos.castling_rights.can_castle_kingside(Color::Black)
                && (occupied & 0x6000000000000000) == 0
                && !pos.is_square_attacked(61, analysis.them)
                && !pos.is_square_attacked(62, analysis.them)
            {
                collector.push(Move::new(king_sq, 62, MoveType::Castle));
            }
            if pos.castling_rights.can_castle_queenside(Color::Black)
                && (occupied & 0x0E00000000000000) == 0
                && !pos.is_square_attacked(59, analysis.them)
                && !pos.is_square_attacked(58, analysis.them)
            {
                collector.push(Move::new(king_sq, 58, MoveType::Castle));
            }
        }
    }
}
