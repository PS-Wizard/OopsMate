use crate::search::features;
use crate::Move;

const MAX_RFP_DEPTH: u8 = 7;

#[rustfmt::skip]
const RFP_MARGINS: [i32; 8] = [
    0,
    100,
    200,
    300,
    400,
    450,
    550,
    650,
];

#[inline(always)]
pub fn can_use_reverse_futility(depth: u8, in_check: bool, pv_node: bool, beta: i32) -> bool {
    // reverse futility pruning: skip deeper search when static eval is already far above beta.
    if !features::REVERSE_FUTILITY {
        return false;
    }

    if in_check || pv_node || depth == 0 || depth > MAX_RFP_DEPTH {
        return false;
    }

    const MATE_BOUND: i32 = 40_000;
    beta.abs() <= MATE_BOUND
}

#[inline(always)]
pub fn get_rfp_margin(depth: u8) -> i32 {
    if depth as usize >= RFP_MARGINS.len() {
        return RFP_MARGINS[RFP_MARGINS.len() - 1];
    }

    RFP_MARGINS[depth as usize]
}

#[inline(always)]
pub fn should_rfp_prune(static_eval: i32, beta: i32, margin: i32) -> bool {
    static_eval - margin >= beta
}

const MAX_FUTILITY_DEPTH: u8 = 7;

#[rustfmt::skip]
const FUTILITY_MARGINS: [i32; 8] = [
    0,
    90,
    180,
    270,
    360,
    450,
    540,
    630,
];

#[inline(always)]
pub fn can_use_futility_pruning(
    depth: u8,
    in_check: bool,
    pv_node: bool,
    alpha: i32,
    beta: i32,
) -> bool {
    // futility pruning: enable quiet-move pruning only in shallow, stable non-pv positions.
    if !features::FUTILITY {
        return false;
    }

    if in_check || pv_node || depth == 0 || depth > MAX_FUTILITY_DEPTH {
        return false;
    }

    const MATE_BOUND: i32 = 40_000;
    alpha.abs() <= MATE_BOUND && beta.abs() <= MATE_BOUND
}

#[inline(always)]
pub fn get_futility_margin(depth: u8) -> i32 {
    if depth as usize >= FUTILITY_MARGINS.len() {
        return FUTILITY_MARGINS[FUTILITY_MARGINS.len() - 1];
    }

    FUTILITY_MARGINS[depth as usize]
}

#[inline(always)]
pub fn should_prune_futility(
    mv: Move,
    gives_check: bool,
    static_eval: i32,
    alpha: i32,
    margin: i32,
) -> bool {
    // futility pruning test: reject quiet moves that cannot realistically improve alpha.
    if mv.is_capture() || mv.is_promotion() || gives_check {
        return false;
    }

    static_eval + margin <= alpha
}
