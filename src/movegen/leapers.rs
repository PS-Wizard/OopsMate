use super::analysis::Analysis;
use super::stage::{include_captures, include_quiets};
use crate::{Move, MoveCollector, MoveType, Position, Piece};
use strikes::KNIGHT_ATTACKS;

#[inline(always)]
pub(super) fn generate<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    let knights = pos.our(Piece::Knight).0 & !analysis.pinned;
    let enemy_king = 1u64 << pos.their_king_sq();
    let mut bb = knights;

    while bb != 0 {
        let from = bb.trailing_zeros() as usize;
        bb &= bb - 1;

        let mut attacks = KNIGHT_ATTACKS[from] & !analysis.us_occ & !enemy_king & analysis.check_mask;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;

            let is_capture = (analysis.them_occ >> to) & 1 != 0;
            if is_capture {
                if include_captures::<STAGE>() {
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            } else if include_quiets::<STAGE>() {
                collector.push(Move::new(from, to, MoveType::Quiet));
            }
        }
    }
}
