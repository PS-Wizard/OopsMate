use crate::qsearch::qsearch;
use crate::search::SearchStats;
use crate::Position;

/// Razoring margins by depth
#[rustfmt::skip]
const RAZOR_MARGINS: [i32; 4] = [
    0,   // depth 0 (not used)
    300, // depth 1
    400, // depth 2
    500, // depth 3
];

/// Try razoring - return Some(score) if we can prune, None otherwise
#[inline(always)]
pub fn try_razoring(
    pos: &Position,
    depth: u8,
    alpha: i32,
    in_check: bool,
    pv_node: bool,
    static_eval: i32,
    stats: &mut SearchStats,
) -> Option<i32> {
    // Only razor at very low depths
    if depth == 0 || depth > 3 {
        return None;
    }

    // Don't razor in check or PV nodes
    if in_check || pv_node {
        return None;
    }

    // Don't razor near mate scores
    const MATE_BOUND: i32 = 40_000;
    if alpha.abs() > MATE_BOUND {
        return None;
    }

    let margin = RAZOR_MARGINS[depth as usize];

    // If static eval is way below alpha, even with margin
    if static_eval + margin < alpha {
        // Verify with qsearch
        let razor_score = qsearch(pos, alpha - margin, alpha - margin + 1, stats, 0);

        if razor_score < alpha - margin {
            return Some(razor_score);
        }
    }

    None
}
