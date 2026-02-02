/// Principal Variation Search (PVS) Module
/// PVS is a refinement of alpha-beta that assumes the first move
/// searched is likely to be the best. It searches the first move with
/// a full window, then searches subsequent moves with a null window.
/// If a null window search fails high, it re-searches with a full window.

use crate::{
    move_history::KillerTable, negamax::negamax, search::SearchStats, tpt::TranspositionTable,
    Move, Position,
};

/// Search the first move with potentially reduced depth
#[inline(always)]
pub fn search_pv_first_move(
    pos: &Position,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    should_reduce: bool,
    reduction: u8,
    pv_node: bool,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
    ply: usize,
) -> i32 {
    let new_pos = pos.make_move(&mv);

    if should_reduce {
        let reduced_depth = depth.saturating_sub(1 + reduction);

        // Try reduced search with null window first
        let mut score = -negamax(
            &new_pos,
            reduced_depth,
            -alpha - 1,
            -alpha,
            tt,
            killers,
            stats,
            true,
            false,
            ply + 1,
        );

        // Re-search with full depth and window if it looks good
        if score > alpha {
            score = -negamax(
                &new_pos,
                depth - 1,
                -beta,
                -alpha,
                tt,
                killers,
                stats,
                true,
                pv_node,
                ply + 1,
            );
        }

        score
    } else {
        // Full depth, full window
        -negamax(
            &new_pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            killers,
            stats,
            true,
            pv_node,
            ply + 1,
        )
    }
}

/// Search subsequent moves using PVS with null window
#[inline(always)]
pub fn search_pv_later_move(
    pos: &Position,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    should_reduce: bool,
    reduction: u8,
    pv_node: bool,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
    ply: usize,
) -> i32 {
    let new_pos = pos.make_move(&mv);

    // Try null window search (possibly reduced)
    let mut score = if should_reduce {
        let reduced_depth = depth.saturating_sub(1 + reduction);
        -negamax(
            &new_pos,
            reduced_depth,
            -alpha - 1,
            -alpha,
            tt,
            killers,
            stats,
            true,
            false,
            ply + 1,
        )
    } else {
        -negamax(
            &new_pos,
            depth - 1,
            -alpha - 1,
            -alpha,
            tt,
            killers,
            stats,
            true,
            false,
            ply + 1,
        )
    };

    // Re-search with full window if the null window search failed high
    if score > alpha && score < beta {
        score = -negamax(
            &new_pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            killers,
            stats,
            true,
            pv_node,
            ply + 1,
        );
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tpt::TranspositionTable, Position};

    #[test]
    fn test_pvs_basic() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        let mut tt = TranspositionTable::new_mb(16);
        let mut killers = KillerTable::new();
        let mut stats = SearchStats::new();

        // Generate a legal move
        let mut collector = crate::MoveCollector::new();
        pos.generate_moves(&mut collector);
        let moves = collector.as_slice();

        if let Some(&mv) = moves.first() {
            let score = search_pv_first_move(
                &pos,
                mv,
                3,
                -1000,
                1000,
                false,
                0,
                true,
                &mut tt,
                &mut killers,
                &mut stats,
                1,
            );

            // Score should be within reasonable bounds
            assert!(score.abs() < 10000);
        }
    }
}
