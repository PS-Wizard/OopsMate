use super::analysis::Analysis;
use super::stage::{include_captures, include_quiets};
use crate::{Move, MoveCollector, MoveType, Piece, Position};

use strikes::{bishop_attacks, queen_attacks, rook_attacks};

#[inline(always)]
pub(super) fn generate<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    generate_bishops::<STAGE>(pos, analysis, collector);
    generate_rooks::<STAGE>(pos, analysis, collector);
    generate_queens::<STAGE>(pos, analysis, collector);
}

#[inline(always)]
fn generate_bishops<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut bb = pos.our(Piece::Bishop).0;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let mut attacks = bishop_attacks(from, analysis.occ) & !analysis.us_occ & !enemy_king;
        if analysis.is_pinned(from) {
            attacks &= analysis.pin_ray(from);
        }
        attacks &= analysis.check_mask;
        emit::<STAGE>(collector, from, attacks, analysis.them_occ);
    }
}

#[inline(always)]
fn generate_rooks<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut bb = pos.our(Piece::Rook).0;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let mut attacks = rook_attacks(from, analysis.occ) & !analysis.us_occ & !enemy_king;
        if analysis.is_pinned(from) {
            attacks &= analysis.pin_ray(from);
        }
        attacks &= analysis.check_mask;
        emit::<STAGE>(collector, from, attacks, analysis.them_occ);
    }
}

#[inline(always)]
fn generate_queens<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut bb = pos.our(Piece::Queen).0;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let mut attacks = queen_attacks(from, analysis.occ) & !analysis.us_occ & !enemy_king;
        if analysis.is_pinned(from) {
            attacks &= analysis.pin_ray(from);
        }
        attacks &= analysis.check_mask;
        emit::<STAGE>(collector, from, attacks, analysis.them_occ);
    }
}

#[inline(always)]
fn emit<const STAGE: u8>(
    collector: &mut MoveCollector,
    from: usize,
    mut attacks: u64,
    enemies: u64,
) {
    while attacks != 0 {
        let to = attacks.trailing_zeros() as usize;
        attacks &= attacks - 1;

        let is_capture = (enemies >> to) & 1 != 0;
        if is_capture {
            if include_captures::<STAGE>() {
                collector.push(Move::new(from, to, MoveType::Capture));
            }
        } else if include_quiets::<STAGE>() {
            collector.push(Move::new(from, to, MoveType::Quiet));
        }
    }
}
