use crate::search::features;
use crate::Move;
use std::sync::OnceLock;

const LMR_MIN_DEPTH: u8 = 2;
const LMR_FULL_DEPTH_MOVES: usize = 2;

const MAX_DEPTH: usize = 64;
const MAX_MOVES: usize = 256;

const QUIET_BASE: f32 = 0.85;
const QUIET_DIVISOR: f32 = 2.0;
const CAPTURE_BASE: f32 = 0.10;
const CAPTURE_DIVISOR: f32 = 2.85;
const PV_REDUCTION: u8 = 1;

static LMR_TABLE: OnceLock<[[u8; MAX_MOVES]; MAX_DEPTH]> = OnceLock::new();

fn init_lmr_table() -> [[u8; MAX_MOVES]; MAX_DEPTH] {
    let mut table = [[0u8; MAX_MOVES]; MAX_DEPTH];

    for (depth, row) in table.iter_mut().enumerate().take(MAX_DEPTH).skip(1) {
        for (move_num, cell) in row.iter_mut().enumerate().take(MAX_MOVES).skip(1) {
            if depth >= LMR_MIN_DEPTH as usize && move_num >= LMR_FULL_DEPTH_MOVES {
                let d = (depth as f32).ln();
                let m = (move_num as f32).ln();
                let reduction = QUIET_BASE + (d * m) / QUIET_DIVISOR;
                let max_reduction = (depth as i32 - 1).max(0) as f32;
                *cell = reduction.min(max_reduction).max(0.0) as u8;
            }
        }
    }

    table
}

pub fn init_lmr() {
    if features::LMR {
        LMR_TABLE.get_or_init(init_lmr_table);
    }
}

#[inline(always)]
fn get_lmr_table() -> &'static [[u8; MAX_MOVES]; MAX_DEPTH] {
    LMR_TABLE
        .get()
        .expect("LMR table not initialized - call pruning::init_lmr() at startup")
}

#[inline(always)]
pub fn should_reduce_lmr(
    depth: u8,
    move_num: usize,
    _in_check: bool,
    _gives_check: bool,
    mv: Move,
    thread_id: usize,
) -> bool {
    if !features::LMR {
        return false;
    }

    if depth < LMR_MIN_DEPTH {
        return false;
    }

    if move_num < LMR_FULL_DEPTH_MOVES {
        return false;
    }

    if mv.is_promotion() {
        return false;
    }

    if thread_id > 0 && move_num > 4 {
        return true;
    }

    true
}

#[inline(always)]
pub fn calculate_lmr_reduction(
    depth: u8,
    move_num: usize,
    pv_node: bool,
    mv: Move,
    thread_id: usize,
) -> u8 {
    if !features::LMR {
        return 0;
    }

    if depth < LMR_MIN_DEPTH || move_num < LMR_FULL_DEPTH_MOVES {
        return 0;
    }

    let table = get_lmr_table();
    let depth_idx = (depth as usize).min(MAX_DEPTH - 1);
    let move_idx = move_num.min(MAX_MOVES - 1);
    let mut reduction = unsafe { *table.get_unchecked(depth_idx).get_unchecked(move_idx) };

    if mv.is_capture() {
        let d = (depth as f32).ln();
        let m = (move_num as f32).ln();
        let capture_reduction = CAPTURE_BASE + (d * m) / CAPTURE_DIVISOR;
        let max_reduction = (depth as i32 - 1).max(0) as f32;
        reduction = capture_reduction.min(max_reduction).max(0.0) as u8;
    }

    if pv_node && reduction > PV_REDUCTION {
        reduction = reduction.saturating_sub(PV_REDUCTION);
    }

    if thread_id > 0 {
        let thread_mod = (thread_id % 3) as u8;
        reduction = reduction.saturating_add(thread_mod);

        if move_num > 6 {
            reduction = reduction.saturating_add(1);
        }
    }

    reduction.min(depth.saturating_sub(1))
}
