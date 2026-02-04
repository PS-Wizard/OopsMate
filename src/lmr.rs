use crate::Move;
use std::sync::OnceLock;

const LMR_MIN_DEPTH: u8 = 3;
const LMR_FULL_DEPTH_MOVES: usize = 3;

const MAX_DEPTH: usize = 64;
const MAX_MOVES: usize = 256;

// aggressive base formula constants
const QUIET_BASE: f32 = 0.85;
const QUIET_DIVISOR: f32 = 2.25;
const CAPTURE_BASE: f32 = 0.10;
const CAPTURE_DIVISOR: f32 = 2.85;

// PV reduction offset
const PV_REDUCTION: u8 = 1; // Can increase to 0 for even more aggressive PV reductions

static LMR_TABLE: OnceLock<[[u8; MAX_MOVES]; MAX_DEPTH]> = OnceLock::new();

/// Initialize the LMR lookup table with aggressive reduction formula
fn init_lmr_table() -> [[u8; MAX_MOVES]; MAX_DEPTH] {
    let mut table = [[0u8; MAX_MOVES]; MAX_DEPTH];

    for depth in 1..MAX_DEPTH {
        for move_num in 1..MAX_MOVES {
            if depth >= LMR_MIN_DEPTH as usize && move_num >= LMR_FULL_DEPTH_MOVES {
                // More aggressive formula: lower base, smaller divisor
                let d = (depth as f32).ln();
                let m = (move_num as f32).ln();
                let reduction = QUIET_BASE + (d * m) / QUIET_DIVISOR;

                // Allow deeper reductions (depth - 1 instead of depth - 2)
                let max_reduction = (depth as i32 - 1).max(0) as f32;
                table[depth][move_num] = reduction.min(max_reduction).max(0.0) as u8;
            }
        }
    }

    table
}

/// Initialize LMR tables at startup
pub fn init() {
    LMR_TABLE.get_or_init(init_lmr_table);
}

#[inline(always)]
fn get_lmr_table() -> &'static [[u8; MAX_MOVES]; MAX_DEPTH] {
    LMR_TABLE
        .get()
        .expect("LMR table not initialized - call lmr::init() at startup")
}

/// Determine if a move should be reduced - more aggressive conditions
#[inline(always)]
pub fn should_reduce(
    depth: u8,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    mv: Move,
) -> bool {
    // Reduced minimum depth from 3 to allow earlier reductions if needed
    if depth < LMR_MIN_DEPTH {
        return false;
    }

    // Start reducing after move 3 instead of 4
    if move_num < LMR_FULL_DEPTH_MOVES {
        return false;
    }

    // Don't reduce check moves
    if in_check || gives_check {
        return false;
    }

    // Don't reduce promotions
    if mv.is_promotion() {
        return false;
    }

    true
}

/// Calculate aggressive reduction amount
#[inline(always)]
pub fn calculate_reduction(depth: u8, move_num: usize, pv_node: bool, mv: Move) -> u8 {
    if depth < LMR_MIN_DEPTH || move_num < LMR_FULL_DEPTH_MOVES {
        return 0;
    }

    let table = get_lmr_table();
    let depth_idx = (depth as usize).min(MAX_DEPTH - 1);
    let move_idx = move_num.min(MAX_MOVES - 1);

    // Get base reduction from table
    let mut reduction = unsafe { *table.get_unchecked(depth_idx).get_unchecked(move_idx) };

    // More aggressive capture reductions
    if mv.is_capture() {
        let d = (depth as f32).ln();
        let m = (move_num as f32).ln();
        let capture_reduction = CAPTURE_BASE + (d * m) / CAPTURE_DIVISOR;
        let max_reduction = (depth as i32 - 1).max(0) as f32; // Allow deeper reductions
        reduction = capture_reduction.min(max_reduction).max(0.0) as u8;
    }

    // Reduce less in PV nodes (but not as much less)
    if pv_node && reduction > PV_REDUCTION {
        reduction = reduction.saturating_sub(PV_REDUCTION);
    }

    // More aggressive: allow reductions up to depth - 1
    reduction.min(depth.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggressive_reductions() {
        init();
        let quiet = Move::new(0, 8, crate::types::MoveType::Quiet);

        // Test that we get larger reductions than conservative settings
        let r1 = calculate_reduction(10, 20, false, quiet);

        // At depth 10, move 20, we should see significant reduction
        println!("Depth 10, Move 20: reduction = {}", r1);
        assert!(r1 >= 3, "Expected aggressive reduction >= 3, got {}", r1);

        // Deep search with late move should be heavily reduced
        let r2 = calculate_reduction(15, 30, false, quiet);
        println!("Depth 15, Move 30: reduction = {}", r2);
        assert!(r2 >= 4, "Expected aggressive reduction >= 4, got {}", r2);
    }

    #[test]
    fn test_early_reduction_trigger() {
        let mv = Move::new(0, 8, crate::types::MoveType::Quiet);

        // Should start reducing at move 3 instead of 4
        assert!(should_reduce(5, 3, false, false, mv));
        assert!(!should_reduce(5, 2, false, false, mv));
    }

    #[test]
    fn test_capture_aggressiveness() {
        init();
        let capture = Move::new(0, 8, crate::types::MoveType::Capture);

        // Captures should still be reduced, just less than quiet moves
        let r_capture = calculate_reduction(10, 20, false, capture);
        println!("Capture reduction: {}", r_capture);
        assert!(
            r_capture >= 2,
            "Expected capture reduction >= 2, got {}",
            r_capture
        );
    }
}
