use crate::search::params::MATE_VALUE;

const TT_MATE_THRESHOLD: i32 = MATE_VALUE - 255;

#[inline(always)]
pub const fn normalize_score(score: i32, ply: u8) -> i32 {
    if score >= TT_MATE_THRESHOLD {
        score + ply as i32
    } else if score <= -TT_MATE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}

#[inline(always)]
pub const fn denormalize_score(score: i32, ply: u8) -> i32 {
    if score >= TT_MATE_THRESHOLD {
        score - ply as i32
    } else if score <= -TT_MATE_THRESHOLD {
        score + ply as i32
    } else {
        score
    }
}
