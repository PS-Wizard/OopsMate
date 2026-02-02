use crate::{
    move_history::KillerTable, negamax::negamax, search::SearchStats, tpt::TranspositionTable,
    Piece, Position,
};

/// Attempts null move pruning - returns Some(score) if pruning succeeds, None otherwise
#[inline(always)]
pub fn try_null_move_pruning(
    pos: &Position,
    depth: u8,
    beta: i32,
    allow_null: bool,
    in_check: bool,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
    ply: usize,
) -> Option<i32> {
    // Don't do null move if:
    // - Not allowed (to prevent double null moves)
    // - In check (illegal to pass when in check)
    // - Not deep enough
    if !allow_null || in_check || depth < 3 {
        return None;
    }

    // Don't do null move in endgame positions without pieces
    // (zugzwang risk is too high)
    let has_pieces = (pos.our(Piece::Knight).0
        | pos.our(Piece::Bishop).0
        | pos.our(Piece::Rook).0
        | pos.our(Piece::Queen).0)
        != 0;

    if !has_pieces {
        return None;
    }

    // Create null move position
    let null_pos = make_null_move(pos);

    // Calculate reduction depth
    let reduction = if depth >= 7 { 3 } else { 2 };
    let null_depth = depth.saturating_sub(1 + reduction);

    // Search with null window
    let null_score = -negamax(
        &null_pos,
        null_depth,
        -beta,
        -beta + 1,
        tt,
        killers,
        stats,
        false, // Don't allow consecutive null moves
        false, // Not a PV node
        ply + 1,
    );

    // If null move fails high, we can prune this node
    if null_score >= beta {
        Some(beta)
    } else {
        None
    }
}

/// Creates a position after making a null move (passing the turn)
#[inline(always)]
fn make_null_move(pos: &Position) -> Position {
    let mut null_pos = *pos;
    null_pos.side_to_move = null_pos.side_to_move.flip();
    null_pos.hash ^= crate::zobrist::SIDE_KEY;
    null_pos.en_passant = None;
    null_pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_null_move() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();

        let null_pos = make_null_move(&pos);

        // Side to move should be flipped
        assert_ne!(pos.side_to_move, null_pos.side_to_move);

        // En passant should be cleared
        assert_eq!(null_pos.en_passant, None);

        // Hash should be different
        assert_ne!(pos.hash(), null_pos.hash());
    }
}
