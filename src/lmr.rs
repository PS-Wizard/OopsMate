use crate::Move;
use std::sync::OnceLock;

// LMR Configuration
const LMR_MIN_DEPTH: u8 = 3;
const LMR_FULL_DEPTH_MOVES: usize = 4;
const MAX_DEPTH: usize = 64;
const MAX_MOVES: usize = 256;

// Precomputed LMR reduction table [depth][move_number]
// table[5][10] = 2   // At depth 5, move #10 should be reduced by 2 plies
// table[10][20] = 3  // At depth 10, move #20 should be reduced by 3 plies
// table[3][3] = 0    // Too early, no reduction
// .. etc
static LMR_TABLE: OnceLock<[[u8; MAX_MOVES]; MAX_DEPTH]> = OnceLock::new();

/// Initialize the LMR lookup table using Weiss formula
fn init_lmr_table() -> [[u8; MAX_MOVES]; MAX_DEPTH] {
    let mut table = [[0u8; MAX_MOVES]; MAX_DEPTH];

    for depth in 1..MAX_DEPTH {
        for move_num in 1..MAX_MOVES {
            if depth >= LMR_MIN_DEPTH as usize && move_num >= LMR_FULL_DEPTH_MOVES {
                // Weiss formula for quiet moves: 1.35 + ln(depth) * ln(move) / 2.75
                let d = (depth as f32).ln();
                let m = (move_num as f32).ln();
                let reduction = 1.35 + (d * m) / 2.75;

                // Clamp to valid range
                let max_reduction = (depth as i32 - 2).max(0) as f32;
                table[depth][move_num] = reduction.min(max_reduction).max(0.0) as u8;
            }
        }
    }

    table
}

/// func to initialize LMR tables at startup (call this once in main)
pub fn init() {
    LMR_TABLE.get_or_init(init_lmr_table);
}

/// Get the LMR table (assumes init() was called)
#[inline(always)]
fn get_lmr_table() -> &'static [[u8; MAX_MOVES]; MAX_DEPTH] {
    // Safe because we call init() at startup
    LMR_TABLE
        .get()
        .expect("LMR table not initialized - call lmr::init() at startup")
}

/// Determine if a move should be reduced based on LMR heuristics
#[inline(always)]
pub fn should_reduce(
    depth: u8,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    mv: Move,
) -> bool {
    // Don't reduce if depth is too shallow
    if depth < LMR_MIN_DEPTH {
        return false;
    }

    // Don't reduce early moves (likely best moves after ordering)
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

/// Calculate the reduction amount for a move using Weiss-style formula
/// Returns the number of plies to reduce
#[inline(always)]
pub fn calculate_reduction(depth: u8, move_num: usize, pv_node: bool, mv: Move) -> u8 {
    if depth < LMR_MIN_DEPTH || move_num < LMR_FULL_DEPTH_MOVES {
        return 0;
    }

    let table = get_lmr_table();
    let depth_idx = (depth as usize).min(MAX_DEPTH - 1);
    let move_idx = move_num.min(MAX_MOVES - 1);

    // Get base reduction from table (quiet move formula)
    let mut reduction = table[depth_idx][move_idx];

    // Adjust for captures (reduce them less)
    // Weiss: 0.20 + ln(depth) * ln(moves) / 3.35 for captures
    // vs 1.35 + ln(depth) * ln(moves) / 2.75 for quiet moves
    if mv.is_capture() {
        // Captures: use different formula
        let d = (depth as f32).ln();
        let m = (move_num as f32).ln();
        let capture_reduction = 0.20 + (d * m) / 3.35;
        let max_reduction = (depth as i32 - 2).max(0) as f32;
        reduction = capture_reduction.min(max_reduction).max(0.0) as u8;
    }

    // Reduce less in PV nodes (more important positions)
    if pv_node && reduction > 0 {
        reduction = reduction.saturating_sub(1);
    }

    // Safety: never reduce to invalid depth
    reduction.min(depth.saturating_sub(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_reduce_depth() {
        let mv = Move::new(0, 8, crate::types::MoveType::Quiet);

        // Too shallow
        assert!(!should_reduce(2, 5, false, false, mv));

        // Deep enough
        assert!(should_reduce(3, 5, false, false, mv));
    }

    #[test]
    fn test_should_reduce_move_number() {
        let mv = Move::new(0, 8, crate::types::MoveType::Quiet);

        // Early moves
        for i in 0..LMR_FULL_DEPTH_MOVES {
            assert!(!should_reduce(5, i, false, false, mv));
        }

        // Later moves
        assert!(should_reduce(5, LMR_FULL_DEPTH_MOVES, false, false, mv));
    }

    #[test]
    fn test_should_reduce_tactical() {
        let quiet = Move::new(0, 8, crate::types::MoveType::Quiet);
        let capture = Move::new(0, 8, crate::types::MoveType::Capture);
        let promotion = Move::new(0, 8, crate::types::MoveType::PromotionQueen);

        // Quiet move should reduce
        assert!(should_reduce(5, 5, false, false, quiet));

        // Captures can reduce (but less than quiet moves)
        assert!(should_reduce(5, 5, false, false, capture));

        // Promotions shouldn't reduce
        assert!(!should_reduce(5, 5, false, false, promotion));

        // In check shouldn't reduce
        assert!(!should_reduce(5, 5, true, false, quiet));

        // Gives check shouldn't reduce
        assert!(!should_reduce(5, 5, false, true, quiet));
    }

    #[test]
    fn test_reduction_scaling() {
        init();
        let quiet = Move::new(0, 8, crate::types::MoveType::Quiet);
        let capture = Move::new(0, 8, crate::types::MoveType::Capture);

        // Shallow depth, early move
        let r1 = calculate_reduction(3, 4, false, quiet);

        // Deep depth, late move
        let r2 = calculate_reduction(10, 20, false, quiet);

        // Later moves at greater depth should have larger reductions
        assert!(r2 > r1);

        // PV nodes should reduce less
        let r_pv = calculate_reduction(10, 20, true, quiet);
        let r_non_pv = calculate_reduction(10, 20, false, quiet);
        assert!(r_pv <= r_non_pv);

        // Captures should reduce less than quiet moves
        let r_capture = calculate_reduction(10, 20, false, capture);
        let r_quiet = calculate_reduction(10, 20, false, quiet);
        assert!(r_capture < r_quiet);
    }

    #[test]
    fn test_reduction_bounds() {
        init();
        let quiet = Move::new(0, 8, crate::types::MoveType::Quiet);

        // Should never reduce more than depth - 2
        for depth in 3..20 {
            for moves in 4..50 {
                let reduction = calculate_reduction(depth, moves, false, quiet);
                assert!(reduction <= depth.saturating_sub(2));
            }
        }
    }
}
