use super::analysis::Analysis;
use super::stage::{include_captures, include_quiets};
use crate::{Color, Move, MoveCollector, MoveType, Position, Piece};
use std::arch::x86_64::_pext_u64;

use strikes::{PAWN_ATTACKS, ROOK_ATTACKS, ROOK_MASKS};

#[inline(always)]
pub(super) fn generate<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    match pos.side_to_move {
        Color::White => generate_white::<STAGE>(pos, analysis, collector),
        Color::Black => generate_black::<STAGE>(pos, analysis, collector),
    }
}

#[inline(always)]
fn generate_white<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let pawns = pos.our(Piece::Pawn).0;
    let empty = !analysis.occ;
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut bb = pawns;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let pin_ray = if analysis.is_pinned(from) { analysis.pin_ray(from) } else { !0u64 };

        let to = from + 8;
        if to < 64 && (empty >> to) & 1 != 0 {
            let target = 1u64 << to;
            if (target & pin_ray & analysis.check_mask) != 0 {
                if to >= 56 {
                    emit_promotion_push::<STAGE>(collector, from, to);
                } else if include_quiets::<STAGE>() {
                    collector.push(Move::new(from, to, MoveType::Quiet));
                }
            }
        }

        if include_quiets::<STAGE>() && (8..16).contains(&from) {
            let to2 = from + 16;
            let single_to = from + 8;
            let target2 = 1u64 << to2;
            if (empty >> single_to) & 1 != 0
                && (empty >> to2) & 1 != 0
                && (target2 & pin_ray & analysis.check_mask) != 0
            {
                collector.push(Move::new(from, to2, MoveType::DoublePush));
            }
        }

        let mut attacks = PAWN_ATTACKS[Color::White as usize][from]
            & analysis.them_occ
            & !enemy_king
            & pin_ray
            & analysis.check_mask;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;
            emit_pawn_capture::<STAGE>(collector, from, to, true);
        }
    }

    if let Some(ep_sq) = pos.en_passant {
        generate_en_passant::<STAGE>(pos, analysis, collector, ep_sq as usize);
    }
}

#[inline(always)]
fn generate_black<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let pawns = pos.our(Piece::Pawn).0;
    let empty = !analysis.occ;
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut bb = pawns;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let pin_ray = if analysis.is_pinned(from) { analysis.pin_ray(from) } else { !0u64 };

        if from >= 8 {
            let to = from - 8;
            if (empty >> to) & 1 != 0 {
                let target = 1u64 << to;
                if (target & pin_ray & analysis.check_mask) != 0 {
                    if to < 8 {
                        emit_promotion_push::<STAGE>(collector, from, to);
                    } else if include_quiets::<STAGE>() {
                        collector.push(Move::new(from, to, MoveType::Quiet));
                    }
                }
            }
        }

        if include_quiets::<STAGE>() && (48..56).contains(&from) {
            let to2 = from - 16;
            let single_to = from - 8;
            let target2 = 1u64 << to2;
            if (empty >> single_to) & 1 != 0
                && (empty >> to2) & 1 != 0
                && (target2 & pin_ray & analysis.check_mask) != 0
            {
                collector.push(Move::new(from, to2, MoveType::DoublePush));
            }
        }

        let mut attacks = PAWN_ATTACKS[Color::Black as usize][from]
            & analysis.them_occ
            & !enemy_king
            & pin_ray
            & analysis.check_mask;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;
            emit_pawn_capture::<STAGE>(collector, from, to, false);
        }
    }

    if let Some(ep_sq) = pos.en_passant {
        generate_en_passant::<STAGE>(pos, analysis, collector, ep_sq as usize);
    }
}

#[inline(always)]
fn emit_promotion_push<const STAGE: u8>(collector: &mut MoveCollector, from: usize, to: usize) {
    if include_quiets::<STAGE>() || include_captures::<STAGE>() {
        collector.push(Move::new(from, to, MoveType::PromotionQueen));
        if include_quiets::<STAGE>() {
            collector.push(Move::new(from, to, MoveType::PromotionRook));
            collector.push(Move::new(from, to, MoveType::PromotionBishop));
            collector.push(Move::new(from, to, MoveType::PromotionKnight));
        }
    }
}

#[inline(always)]
fn emit_pawn_capture<const STAGE: u8>(
    collector: &mut MoveCollector,
    from: usize,
    to: usize,
    white: bool,
) {
    if !include_captures::<STAGE>() {
        return;
    }

    let promote = if white { to >= 56 } else { to < 8 };
    if promote {
        collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
        collector.push(Move::new(from, to, MoveType::CapturePromotionRook));
        collector.push(Move::new(from, to, MoveType::CapturePromotionBishop));
        collector.push(Move::new(from, to, MoveType::CapturePromotionKnight));
    } else {
        collector.push(Move::new(from, to, MoveType::Capture));
    }
}

fn generate_en_passant<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
    ep_sq: usize,
) {
    if !include_captures::<STAGE>() {
        return;
    }

    let captured_sq = if pos.side_to_move == Color::White { ep_sq - 8 } else { ep_sq + 8 };
    let ep_target = 1u64 << ep_sq;
    let captured_bit = 1u64 << captured_sq;

    if (ep_target & analysis.check_mask) == 0 && (captured_bit & analysis.check_mask) == 0 {
        return;
    }

    let mut bb = pos.our(Piece::Pawn).0;
    let color_idx = pos.side_to_move as usize;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        if (PAWN_ATTACKS[color_idx][from] & ep_target) == 0 {
            continue;
        }

        if analysis.is_pinned(from) && (ep_target & analysis.pin_ray(from)) == 0 {
            continue;
        }

        let king_rank = analysis.king_sq / 8;
        let from_rank = from / 8;
        if king_rank == from_rank && from_rank == captured_sq / 8 {
            let after_ep = analysis.occ & !(1u64 << from) & !captured_bit | ep_target;
            let rook_idx = unsafe { _pext_u64(after_ep, ROOK_MASKS[analysis.king_sq]) as usize };
            let rook_attacks = ROOK_ATTACKS[analysis.king_sq][rook_idx];
            let enemy_rooks_queens = pos.their(Piece::Rook).0 | pos.their(Piece::Queen).0;
            if (rook_attacks & enemy_rooks_queens) != 0 {
                continue;
            }
        }

        collector.push(Move::new(from, ep_sq, MoveType::EnPassant));
    }
}
