use crate::{
    evaluate::evaluate,
    futility::{can_use_futility_pruning, get_futility_margin, should_prune_move},
    lmr::{calculate_reduction, should_reduce},
    move_history::KillerTable,
    move_ordering::{pick_next_move, score_move},
    null_move::try_null_move_pruning,
    qsearch::qsearch,
    reverse_futility::{can_use_reverse_futility, get_rfp_margin, should_rfp_prune},
    search::SearchStats,
    tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND},
    Move, MoveCollector, Position,
};

const INFINITY: i32 = 50_000;
const MATE_VALUE: i32 = 49_000;
const MAX_MOVES: usize = 256;

#[allow(clippy::too_many_arguments)]
pub fn negamax(
    pos: &Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
    allow_null: bool,
    pv_node: bool,
    ply: usize,
) -> i32 {
    stats.nodes += 1;

    let hash = pos.hash();

    // Probe transposition table - check for early cutoff
    let tt_move = {
        let tt_entry = tt.probe(hash);
        if let Some(entry) = tt_entry {
            if entry.depth >= depth {
                stats.tt_hits += 1;
                match entry.flag {
                    EXACT => return entry.score,
                    LOWER_BOUND if entry.score >= beta => return entry.score,
                    UPPER_BOUND if entry.score <= alpha => return entry.score,
                    _ => {}
                }
            }
            Some(entry.best_move)
        } else {
            None
        }
    };

    // Base case - drop into quiescence search
    if depth == 0 {
        return qsearch(pos, alpha, beta, stats, 0);
    }

    let in_check = pos.is_in_check();

    // Try null move pruning
    if let Some(score) = try_null_move_pruning(
        pos, depth, beta, allow_null, in_check, tt, killers, stats, ply,
    ) {
        return score;
    }

    // Static evaluation for pruning decisions
    let static_eval = evaluate(pos);

    // Reverse futility pruning
    if can_use_reverse_futility(depth, in_check, pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(static_eval, beta, rfp_margin) {
            return static_eval - rfp_margin;
        }
    }

    // Futility pruning setup
    let use_futility = can_use_futility_pruning(depth, in_check, pv_node, alpha, beta);
    let (static_eval, futility_margin) = if use_futility {
        let margin = get_futility_margin(depth);
        (static_eval, margin)
    } else {
        (0, 0)
    };

    // Generate and order moves
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // Checkmate / Stalemate detection
    if moves.is_empty() {
        return if in_check {
            -MATE_VALUE - (depth as i32)
        } else {
            0
        };
    }

    let move_count = moves.len();
    let mut move_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];

    // Score moves for ordering
    for i in 0..move_count {
        move_list[i] = moves[i];
        scores[i] = score_move(moves[i], pos, tt_move, killers, ply);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    // Search all moves
    for i in 0..move_count {
        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        let new_pos = pos.make_move(&mv);
        let gives_check = new_pos.is_in_check();

        // Futility pruning - skip quiet moves in losing positions
        if use_futility && i > 0 {
            if should_prune_move(mv, gives_check, static_eval, alpha, futility_margin) {
                continue;
            }
        }

        let score = if i == 0 {
            // First move: full depth, full window (possibly reduced)
            if should_reduce(depth, i, in_check, gives_check, mv) {
                let reduction = calculate_reduction(depth, i, pv_node, mv);
                let reduced_depth = depth.saturating_sub(1 + reduction);

                let mut s = -negamax(
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

                if s > alpha {
                    s = -negamax(
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
                s
            } else {
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
        } else {
            // PVS for subsequent moves
            let mut s = if should_reduce(depth, i, in_check, gives_check, mv) {
                let reduction = calculate_reduction(depth, i, pv_node, mv);
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
                // Null window search
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
            if s > alpha && s < beta {
                s = -negamax(
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

            s
        };

        // Beta cutoff
        if score >= beta {
            // Store killer move if it's quiet
            if !mv.is_capture() && !mv.is_promotion() {
                killers.store(ply, mv);
            }

            tt.store(hash, mv, beta, depth, LOWER_BOUND);
            return beta;
        }

        // Update best move
        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }
    }

    // Store result in transposition table
    let flag = if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(hash, best_move, best_score, depth, flag);

    best_score
}
