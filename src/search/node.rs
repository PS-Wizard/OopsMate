use super::context::SearchContext;
use super::features;
use super::heuristics::{
    calculate_lmr_reduction, can_use_futility_pruning, can_use_reverse_futility,
    get_futility_margin, get_rfp_margin, should_prune_futility, should_reduce_lmr,
    should_rfp_prune, try_iid, try_null_move_pruning, try_probcut, try_razoring,
};
use super::ordering::{pick_next_move, score_move};
use super::params::{INFINITY, MAX_MOVES};
use super::qsearch::qsearch;
use super::score::{checkmate_score, score_from_tt, score_to_tt};
use crate::eval::EvalProvider;
use crate::tpt::{EXACT, LOWER_BOUND, UPPER_BOUND};
use crate::{Move, MoveCollector, Position};

#[derive(Clone, Copy)]
pub(crate) struct NodeState {
    pub(crate) allow_null: bool,
    pub(crate) pv_node: bool,
    pub(crate) excluded_move: Option<Move>,
    pub(crate) ply: usize,
}

impl NodeState {
    pub(crate) const fn new(
        allow_null: bool,
        pv_node: bool,
        excluded_move: Option<Move>,
        ply: usize,
    ) -> Self {
        Self {
            allow_null,
            pv_node,
            excluded_move,
            ply,
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "move search keeps the hot-path branching explicit"
)]
#[inline(always)]
fn search_with_lmr<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_index: usize,
    in_check: bool,
    gives_check: bool,
    check_extension: u8,
    node: NodeState,
) -> i32 {
    // lmr: reduce late moves first, then re-search at full depth only if they improve alpha.
    let do_lmr = should_reduce_lmr(depth, move_index, in_check, gives_check, mv);

    if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_index, node.pv_node, mv);
        let reduced_depth = depth
            .saturating_sub(1 + reduction)
            .saturating_add(check_extension);

        let reduced_score = -search_node(
            pos,
            ctx,
            reduced_depth,
            -beta,
            -alpha,
            NodeState::new(true, node.pv_node, None, node.ply + 1),
        );

        if reduced_score > alpha {
            return -search_node(
                pos,
                ctx,
                depth - 1 + check_extension,
                -beta,
                -alpha,
                NodeState::new(true, node.pv_node, None, node.ply + 1),
            );
        }

        return reduced_score;
    }

    -search_node(
        pos,
        ctx,
        depth - 1 + check_extension,
        -beta,
        -alpha,
        NodeState::new(true, node.pv_node, None, node.ply + 1),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "move search keeps the hot-path branching explicit"
)]
#[inline(always)]
fn search_with_pvs<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_index: usize,
    in_check: bool,
    gives_check: bool,
    check_extension: u8,
    node: NodeState,
    is_hash_move: bool,
) -> i32 {
    if move_index == 0 {
        // pvs: the first move is searched with the full alpha-beta window.
        return -search_node(
            pos,
            ctx,
            depth - 1 + check_extension,
            -beta,
            -alpha,
            NodeState::new(true, node.pv_node, None, node.ply + 1),
        );
    }

    // pvs: later moves get a null-window scout search before any full re-search.
    let do_lmr = should_reduce_lmr(depth, move_index, in_check, gives_check, mv) && !is_hash_move;

    let mut score = if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_index, node.pv_node, mv);
        let reduced_depth = depth
            .saturating_sub(1 + reduction)
            .saturating_add(check_extension);

        -search_node(
            pos,
            ctx,
            reduced_depth,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, node.ply + 1),
        )
    } else {
        -search_node(
            pos,
            ctx,
            depth - 1 + check_extension,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, node.ply + 1),
        )
    };

    // pvs re-search: only widen the window when the scout search proves the move is real.
    if score > alpha && score < beta {
        score = -search_node(
            pos,
            ctx,
            depth - 1 + check_extension,
            -beta,
            -alpha,
            NodeState::new(true, node.pv_node, None, node.ply + 1),
        );
    }

    score
}

#[inline(always)]
pub(crate) fn search_node<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mut depth: u8,
    mut alpha: i32,
    beta: i32,
    node: NodeState,
) -> i32 {
    ctx.stats.nodes += 1;
    let alpha_start = alpha;

    if ctx.stats.should_stop() {
        return 0;
    }

    if pos.is_fifty_move_draw() || (node.ply > 0 && pos.is_repetition()) {
        return 0;
    }

    // tt cutoffs: reuse a cached bound or exact score before doing any deeper work.
    let hash = pos.hash();
    let tt_entry = if features::TT_CUTOFFS {
        ctx.tt.probe(hash).map(|mut entry| {
            entry.score = score_from_tt(entry.score, node.ply);
            entry
        })
    } else {
        None
    };

    let tt_move = if let Some(entry) = tt_entry {
        if entry.depth >= depth && node.excluded_move.is_none() {
            ctx.stats.tt_hits += 1;
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
    };

    // quiescence search: stop full-width recursion and resolve tactical noise with captures only.
    if depth == 0 {
        return qsearch(pos, ctx, alpha, beta, 0);
    }

    let in_check = pos.is_in_check();
    let static_eval = ctx.eval.eval(pos, &mut ctx.eval_state);

    // forward pruning: try cheap cutoffs before generating and searching every move.
    if let Some(score) = try_probcut(pos, ctx, depth, beta, node.pv_node, in_check, node.ply) {
        return score;
    }

    if let Some(score) = try_razoring(pos, ctx, depth, alpha, in_check, node.pv_node, static_eval) {
        return score;
    }

    if can_use_reverse_futility(depth, in_check, node.pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(static_eval, beta, rfp_margin) {
            return static_eval - rfp_margin;
        }
    }

    if let Some(score) = try_null_move_pruning(
        pos,
        ctx,
        depth,
        beta,
        node.allow_null,
        in_check,
        static_eval,
        node.ply,
    ) {
        return score;
    }

    // singular extensions: extend depth when the tt move looks uniquely stronger than alternatives.
    if features::SINGULAR_EXTENSIONS
        && !node.pv_node
        && node.excluded_move.is_none()
        && depth >= 8
        && tt_move.is_some()
        && !in_check
    {
        if let Some(entry) = tt_entry {
            if entry.depth >= depth.saturating_sub(3) && entry.flag == LOWER_BOUND {
                let singular_beta = entry.score.saturating_sub(depth as i32 * 2);
                let singular_depth = depth / 2;

                let score = search_node(
                    pos,
                    ctx,
                    singular_depth,
                    singular_beta - 1,
                    singular_beta,
                    NodeState::new(node.allow_null, false, tt_move, node.ply),
                );

                if score < singular_beta {
                    depth += 1;
                } else if score >= beta {
                    return singular_beta;
                }
            }
        }
    }

    // iid: do a shallower search to discover a good tt move when none is cached yet.
    let iid_move = try_iid(
        pos,
        ctx,
        depth,
        alpha,
        beta,
        node.pv_node,
        tt_move.is_some(),
        in_check,
        node.ply,
    );
    let tt_move = tt_move.or(iid_move);
    let tt_order_move = if features::TT_MOVE_ORDERING {
        tt_move
    } else {
        None
    };

    // futility pruning setup: precompute the quiet-move margin used inside the move loop.
    let use_futility = can_use_futility_pruning(depth, in_check, node.pv_node, alpha, beta);
    let (static_eval, futility_margin) = if use_futility {
        (static_eval, get_futility_margin(depth))
    } else {
        (0, 0)
    };

    // move ordering: generate legal moves, then score them for tt/captures/killers/history.
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return if in_check {
            checkmate_score(node.ply)
        } else {
            0
        };
    }

    let move_count = moves.len();
    let mut move_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];

    for i in 0..move_count {
        move_list[i] = moves[i];
        scores[i] = score_move(moves[i], pos, tt_order_move, Some(&ctx.history), node.ply);
    }

    // move loop: search moves in order and let alpha-beta cut the rest when possible.
    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    for i in 0..move_count {
        if ctx.stats.should_stop() {
            break;
        }

        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        if let Some(excluded) = node.excluded_move {
            if mv.0 == excluded.0 {
                continue;
            }
        }

        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);
        let gives_check = pos.is_in_check();
        let check_extension = if features::CHECK_EXTENSIONS && gives_check {
            1
        } else {
            0
        };

        // futility pruning: skip quiet non-checking moves that cannot plausibly raise alpha.
        if use_futility
            && i > 0
            && should_prune_futility(mv, gives_check, static_eval, alpha, futility_margin)
        {
            pos.unmake_move(mv);
            ctx.eval.update_on_undo(&mut ctx.eval_state, delta);
            continue;
        }

        // per-move search: combine pvs with lmr so late moves start cheap and only earn a re-search.
        let score = if !features::PVS {
            search_with_lmr(
                pos,
                ctx,
                mv,
                depth,
                alpha,
                beta,
                i,
                in_check,
                gives_check,
                check_extension,
                node,
            )
        } else {
            let is_hash_move = tt_order_move.is_some_and(|tt_mv| mv.0 == tt_mv.0);
            search_with_pvs(
                pos,
                ctx,
                mv,
                depth,
                alpha,
                beta,
                i,
                in_check,
                gives_check,
                check_extension,
                node,
                is_hash_move,
            )
        };

        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if ctx.stats.should_stop() {
            return 0;
        }

        // beta cutoff: once this move is good enough, stop and record the quiet-move heuristics.
        if score >= beta {
            if !mv.is_capture() && !mv.is_promotion() {
                // killer moves: store this move as it caused a beta cutoff
                if features::KILLER_MOVES {
                    ctx.history.killers.store(node.ply, mv);
                }
                if features::HISTORY_HEURISTIC {
                    let bonus = (depth as i16 * depth as i16).min(400);
                    ctx.history
                        .history
                        .update(pos.side_to_move, mv.from(), mv.to(), bonus);
                }
            }

            if features::TT_CUTOFFS {
                ctx.tt
                    .store(hash, mv, score_to_tt(beta, node.ply), depth, LOWER_BOUND);
            }
            return beta;
        }

        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }
    }

    // tt store: cache the best result from this node as an exact score or upper bound.
    let flag = if best_score <= alpha_start {
        UPPER_BOUND
    } else {
        EXACT
    };

    if features::TT_CUTOFFS {
        ctx.tt.store(
            hash,
            best_move,
            score_to_tt(best_score, node.ply),
            depth,
            flag,
        );
    }

    best_score
}
