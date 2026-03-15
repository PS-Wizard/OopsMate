use super::params::{MATE_VALUE, MAX_DEPTH};

const TT_MATE_THRESHOLD: i32 = MATE_VALUE - MAX_DEPTH as i32;

#[inline(always)]
pub(crate) const fn checkmate_score(ply: usize) -> i32 {
    -MATE_VALUE + ply as i32
}

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
