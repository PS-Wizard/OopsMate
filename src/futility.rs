use crate::Move;

/// Maximum depth at which futility pruning is applied
const MAX_FUTILITY_DEPTH: u8 = 7;

const FUTILITY_MARGINS: [i32; 8] = [
    0,   // depth 0 (not used, handled by qsearch)
    100, // depth 1
    200, // depth 2
    300, 400, 500, 600, 700,
];

/// Check if we can apply futility pruning to this position/search state
#[inline(always)]
pub fn can_use_futility_pruning(
    depth: u8,
    in_check: bool,
    pv_node: bool,
    alpha: i32,
    beta: i32,
) -> bool {
    // Don't prune in check (tactical)
    if in_check {
        return false;
    }

    // Don't prune in PV nodes (need exact scores)
    if pv_node {
        return false;
    }

    // Only apply at low depths
    if depth == 0 || depth > MAX_FUTILITY_DEPTH {
        return false;
    }

    // Don't prune near mate scores (margins don't apply)
    const MATE_BOUND: i32 = 40_000;
    if alpha.abs() > MATE_BOUND || beta.abs() > MATE_BOUND {
        return false;
    }

    true
}

/// Get the futility margin for a given depth
#[inline(always)]
pub fn get_futility_margin(depth: u8) -> i32 {
    if depth as usize >= FUTILITY_MARGINS.len() {
        return FUTILITY_MARGINS[FUTILITY_MARGINS.len() - 1];
    }
    FUTILITY_MARGINS[depth as usize]
}

/// Check if a move should be pruned based on futility
/// Returns true if the move can be safely skipped
#[inline(always)]
pub fn should_prune_move(
    mv: Move,
    gives_check: bool,
    static_eval: i32,
    alpha: i32,
    margin: i32,
) -> bool {

    // Never prune tactical moves
    if mv.is_capture() {
        return false;
    }

    // This can change the val like ~900 centipawns from a queen promo
    if mv.is_promotion() {
        return false;
    }

    if gives_check {
        return false;
    }

    // Prune if static_eval + margin <= alpha
    static_eval + margin <= alpha
}
