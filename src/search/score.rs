use super::params::{MATE_VALUE, MAX_DEPTH};

/// Scores above this magnitude are mate scores, not centipawn evals.
const TT_MATE_THRESHOLD: i32 = MATE_VALUE - MAX_DEPTH as i32;

/// Score when the side to move is checkmated at `ply`.
/// More negative = mated sooner. e.g. mated in 1 = -48_999, mated in 2 = -48_998.
#[inline(always)]
pub(crate) const fn checkmate_score(ply: usize) -> i32 {
    -MATE_VALUE + ply as i32
}

/// Adjust a mate score before storing in the TT.
/// Mate scores encode distance-from-root, but the TT needs distance-from-position.
/// Strips the current ply out so the score is position-relative.
/// Normal centipawn scores pass through unchanged.
#[inline(always)]
pub(crate) const fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score >= TT_MATE_THRESHOLD {
        score + ply as i32
    } else if score <= -TT_MATE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}

/// Adjust a mate score after retrieving from the TT.
/// Re-adds the current ply so the score is root-relative again.
/// Normal centipawn scores pass through unchanged.
#[inline(always)]
pub(crate) const fn score_from_tt(score: i32, ply: usize) -> i32 {
    if score >= TT_MATE_THRESHOLD {
        score - ply as i32
    } else if score <= -TT_MATE_THRESHOLD {
        score + ply as i32
    } else {
        score
    }
}
