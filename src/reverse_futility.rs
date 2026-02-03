/// Maximum depth at which reverse futility pruning is applied
const MAX_RFP_DEPTH: u8 = 7;

/// Reverse futility margins by depth
/// These are more aggressive than forward futility since we're betting our position is winning
#[rustfmt::skip]
const RFP_MARGINS: [i32; 8] = [
    0,   // depth 0 (not used)
    150, // depth 1
    250, // depth 2
    350, // depth 3
    450,
    500,
    600,
    700,
];

/// Check if we can apply reverse futility pruning to this position
#[inline(always)]
pub fn can_use_reverse_futility(depth: u8, in_check: bool, pv_node: bool, beta: i32) -> bool {
    // Don't prune in check (tactical)
    if in_check {
        return false;
    }

    // Don't prune in PV nodes (need exact scores)
    if pv_node {
        return false;
    }

    // Only apply at low depths
    if depth == 0 || depth > MAX_RFP_DEPTH {
        return false;
    }

    // Don't prune near mate scores
    const MATE_BOUND: i32 = 40_000;
    if beta.abs() > MATE_BOUND {
        return false;
    }

    true
}

/// Get the reverse futility margin for a given depth
#[inline(always)]
pub fn get_rfp_margin(depth: u8) -> i32 {
    if depth as usize >= RFP_MARGINS.len() {
        return RFP_MARGINS[RFP_MARGINS.len() - 1];
    }
    RFP_MARGINS[depth as usize]
}

/// Check if we should prune this node with reverse futility
/// Returns true if static_eval - margin >= beta (our position is too good)
#[inline(always)]
pub fn should_rfp_prune(static_eval: i32, beta: i32, margin: i32) -> bool {
    static_eval - margin >= beta
}

#[cfg(test)]
mod test_reverse_futility {
    use super::*;

    #[test]
    fn test_can_use_reverse_futility() {
        // Should work at shallow depths, non-PV, not in check
        assert!(can_use_reverse_futility(3, false, false, 100));

        // Don't use in check
        assert!(!can_use_reverse_futility(3, true, false, 100));

        // Don't use in PV nodes
        assert!(!can_use_reverse_futility(3, false, true, 100));

        // Don't use at depth 0
        assert!(!can_use_reverse_futility(0, false, false, 100));

        // Don't use too deep
        assert!(!can_use_reverse_futility(8, false, false, 100));

        // Don't use near mate
        assert!(!can_use_reverse_futility(3, false, false, 45_000));
    }

    #[test]
    fn test_rfp_margins() {
        assert_eq!(get_rfp_margin(1), 150);
        assert_eq!(get_rfp_margin(2), 250);
        assert_eq!(get_rfp_margin(3), 350);
    }

    #[test]
    fn test_should_rfp_prune() {
        // Position is good enough to prune
        assert!(should_rfp_prune(500, 200, 150)); // 500 - 150 = 350 >= 200

        // Position not good enough
        assert!(!should_rfp_prune(300, 200, 150)); // 300 - 150 = 150 < 200

        // Exactly at threshold
        assert!(should_rfp_prune(350, 200, 150)); // 350 - 150 = 200 >= 200
    }
}
