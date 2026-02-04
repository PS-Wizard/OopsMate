/// Internal Iterative Deepening (IID)
///
/// When we reach a node without a TT move, we perform a reduced-depth
/// search to find a good move for ordering. This improves cutoff rates
/// and search efficiency, especially in PV nodes where move ordering matters most.
use crate::{
    move_history::KillerTable, negamax::negamax, search::SearchStats, tpt::TranspositionTable, Move,
};

/// Minimum depth to trigger IID
const IID_MIN_DEPTH: u8 = 4;

/// Depth reduction for IID search
/// For PV nodes: reduce by depth/4 + 2
/// For non-PV nodes: reduce by depth/3 + 1
#[inline(always)]
fn iid_reduction(depth: u8, pv_node: bool) -> u8 {
    if pv_node {
        (depth / 4).max(2)
    } else {
        (depth / 3).max(1)
    }
}

/// Perform internal iterative deepening to find a good move
/// Returns the best move found, or None if IID shouldn't be performed
#[inline(always)]
pub fn try_iid(
    pos: &crate::Position,
    depth: u8,
    alpha: i32,
    beta: i32,
    pv_node: bool,
    has_tt_move: bool,
    in_check: bool,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
    ply: usize,
) -> Option<Move> {
    if has_tt_move || depth < IID_MIN_DEPTH || in_check {
        return None;
    }

    // For non-PV nodes, require even deeper to justify the cost
    if !pv_node && depth < IID_MIN_DEPTH + 2 {
        return None;
    }

    let reduction = iid_reduction(depth, pv_node);
    let iid_depth = depth.saturating_sub(reduction);

    // Perform reduced search
    negamax(
        pos, iid_depth, alpha, beta, tt, killers, stats, true, // allow_null
        pv_node, ply,
    );

    // After search, probe TT for the move it found
    tt.probe(pos.hash()).map(|entry| entry.best_move)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iid_reduction() {
        // PV nodes: more aggressive reduction
        assert_eq!(iid_reduction(4, true), 2);
        assert_eq!(iid_reduction(8, true), 2);
        assert_eq!(iid_reduction(12, true), 3);
        assert_eq!(iid_reduction(16, true), 4);

        // Non-PV nodes: less aggressive
        assert_eq!(iid_reduction(4, false), 1);
        assert_eq!(iid_reduction(9, false), 3);
        assert_eq!(iid_reduction(12, false), 4);
    }

    #[test]
    fn test_iid_depth_threshold() {
        use crate::{tpt::TranspositionTable, Position};

        let pos = Position::new();
        let mut tt = TranspositionTable::new_mb(16);
        let mut killers = KillerTable::new();
        let mut stats = SearchStats::new();

        // Should not trigger at shallow depths
        let result = try_iid(
            &pos,
            3,
            -1000,
            1000,
            true,
            false,
            false,
            &mut tt,
            &mut killers,
            &mut stats,
            0,
        );
        assert!(result.is_none());

        // Should trigger at sufficient depth for PV
        let result = try_iid(
            &pos,
            4,
            -1000,
            1000,
            true,
            false,
            false,
            &mut tt,
            &mut killers,
            &mut stats,
            0,
        );
        assert!(result.is_some());
    }
}
