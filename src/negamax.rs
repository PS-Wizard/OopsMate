use crate::{
    evaluate::evaluate,
    futility::{can_use_futility_pruning, get_futility_margin, should_prune_move},
    iid::try_iid,
    lmr::{calculate_reduction, should_reduce},
    move_history::KillerTable,
    move_ordering::{pick_next_move, score_move},
    null_move::try_null_move_pruning,
    qsearch::qsearch,
    razoring::try_razoring,
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

    // Static evaluation for pruning decisions
    let static_eval = evaluate(pos);

    // RAZORING (depth 1-3, losing badly)
    // "Am I so far behind that even tactics can't save me?"
    if let Some(score) = try_razoring(pos, depth, alpha, in_check, pv_node, static_eval, stats) {
        return score;
    }

    // Reverse futility pruning
    // "Am I winning by so much I can just return?"
    if can_use_reverse_futility(depth, in_check, pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(static_eval, beta, rfp_margin) {
            return static_eval - rfp_margin;
        }
    }

    // Try null move pruning
    // "Let me verify I'm winning by giving opponent free move"
    if let Some(score) = try_null_move_pruning(
        pos, depth, beta, allow_null, in_check, tt, killers, stats, ply,
    ) {
        return score;
    }

    // Internal Iterative Deepening-  get a good move if we don't have a TT move
    // "Let me do a quick search to find the best move for ordering"
    let iid_move = try_iid(
        pos,
        depth,
        alpha,
        beta,
        pv_node,
        tt_move.is_some(),
        in_check,
        tt,
        killers,
        stats,
        ply,
    );

    // Use IID move if we found one and don't have a TT move
    let tt_move = tt_move.or(iid_move);

    // Futility pruning setup
    // "Prepare to skip hopeless quiet moves later"
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
        scores[i] = score_move(moves[i], pos, tt_move, Some(&killers), ply);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    // Search all moves
    for i in 0..move_count {
        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        let new_pos = pos.make_move(&mv);
        let gives_check = new_pos.is_in_check();
        let check_extension = if gives_check { 1 } else { 0 };

        // Futility pruning - skip quiet moves in losing positions
        // "This quiet move can't raise alpha, skip it"
        if use_futility && i > 0 {
            if should_prune_move(mv, gives_check, static_eval, alpha, futility_margin) {
                continue;
            }
        }

        let score = if i == 0 {
            // First move: full depth, full window
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
        } else {
            // PVS for subsequent moves
            let is_hash_move = tt_move.map_or(false, |tt_mv| mv.0 == tt_mv.0);
            let mut s = if should_reduce(depth, i, in_check, gives_check, mv) & !is_hash_move {
                let reduction = calculate_reduction(depth, i, pv_node, mv);
                let reduced_depth = depth
                    .saturating_sub(1 + reduction)
                    .saturating_add(check_extension);

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
                    depth - 1 + check_extension,
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
                    depth - 1 + check_extension,
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
