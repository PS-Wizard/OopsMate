use super::params::MATE_VALUE;

/// Score when the side to move is checkmated at `ply`.
/// More negative = mated sooner. e.g. mated in 1 = -48_999, mated in 2 = -48_998.
#[inline(always)]
pub(crate) const fn checkmate_score(ply: usize) -> i32 {
    -MATE_VALUE + ply as i32
}
