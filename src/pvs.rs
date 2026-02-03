/// Principal Variation Search (PVS) Module
/// PVS is a refinement of alpha-beta that assumes the first move
/// searched is likely to be the best. It searches the first move with
/// a full window, then searches subsequent moves with a null window.
/// If a null window search fails high, it re-searches with a full window.
use crate::{
    lmr::{calculate_reduction, should_reduce},
    move_history::KillerTable,
    negamax::negamax,
    search::SearchStats,
    tpt::TranspositionTable,
    Move, Position,
};

/// Search a move with PVS and LMR
/// - First move (move_num == 0): uses full window
/// - Later moves: uses null window, re-searches if it fails high
/// - Applies LMR reductions based on move characteristics
#[inline(always)]
pub fn search_move(
    pos: &Position,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    pv_node: bool,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
    ply: usize,
) -> i32 {
    // First move gets full window search
    if move_num == 0 {
        return -negamax(
            &*pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            killers,
            stats,
            gives_check,
            pv_node,
            ply + 1,
        );
    }

    // Later moves: try LMR + null window, re-search if needed
    let do_lmr = should_reduce(depth, move_num, in_check, gives_check, mv);

    let mut score = if do_lmr {
        // LMR: search with reduced depth and null window
        let reduction = calculate_reduction(depth, move_num, pv_node, mv);
        let reduced_depth = depth.saturating_sub(1 + reduction);

        -negamax(
            &*pos,
            reduced_depth,
            -alpha - 1,
            -alpha,
            tt,
            killers,
            stats,
            gives_check,
            false,
            ply + 1,
        )
    } else {
        // No LMR: just null window at full depth
        -negamax(
            &*pos,
            depth - 1,
            -alpha - 1,
            -alpha,
            tt,
            killers,
            stats,
            gives_check,
            false,
            ply + 1,
        )
    };

    // Re-search with full window if null window failed high
    if score > alpha && score < beta {
        score = -negamax(
            &*pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            killers,
            stats,
            gives_check,
            pv_node,
            ply + 1,
        );
    }

    score
}

#[cfg(test)]
mod test_pvs {
    use super::*;
    use crate::{tpt::TranspositionTable, Position};

    #[test]
    fn test_pvs_first_move() {
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let mut tt = TranspositionTable::new_mb(16);
        let mut killers = KillerTable::new();
        let mut stats = SearchStats::new();

        let mut collector = crate::MoveCollector::new();
        pos.generate_moves(&mut collector);
        let moves = collector.as_slice();

        if let Some(&mv) = moves.first() {
            let score = search_move(
                &pos,
                mv,
                3,
                -1000,
                1000,
                0, // First move
                false,
                false,
                true,
                &mut tt,
                &mut killers,
                &mut stats,
                0,
            );

            assert!(score.abs() < 10000);
        }
    }

    #[test]
    fn test_pvs_later_move() {
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let mut tt = TranspositionTable::new_mb(16);
        let mut killers = KillerTable::new();
        let mut stats = SearchStats::new();

        let mut collector = crate::MoveCollector::new();
        pos.generate_moves(&mut collector);
        let moves = collector.as_slice();

        if moves.len() > 1 {
            let score = search_move(
                &pos,
                moves[1],
                3,
                -1000,
                1000,
                1, // Second move
                false,
                false,
                true,
                &mut tt,
                &mut killers,
                &mut stats,
                0,
            );

            assert!(score.abs() < 10000);
        }
    }
}
