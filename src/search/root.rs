use super::api::SearchInfo;
use super::context::SearchContext;
use super::features;
use super::limits::{should_stop_next_iteration, SearchLimits};
use super::node::{search_node, NodeState};
use super::ordering::{pick_next_move, score_move};
use super::output::print_uci_info;
use super::params::{ASPIRATION_DEPTH, INFINITY, MAX_MOVES};
use super::score::{checkmate_score, score_to_tt};
use crate::eval::EvalProvider;
use crate::tpt::{EXACT, LOWER_BOUND, UPPER_BOUND};
use crate::{Move, MoveCollector, Position};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

const INITIAL_ASPIRATION_DELTA: i32 = 25;
const MAX_ASPIRATION_DELTA: i32 = 1_000;

#[derive(Clone, Copy)]
struct RootMoveState {
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    pv_node: bool,
    ply: usize,
}

pub(crate) fn run_search<E: EvalProvider>(
    pos: &Position,
    max_depth: u8,
    limits: SearchLimits,
    tt: &mut crate::tpt::TranspositionTable,
    stop_signal: Arc<AtomicBool>,
    eval: &E,
) -> Option<SearchInfo> {
    let mut pos = pos.clone();
    let start_time = Instant::now();
    let mut ctx = SearchContext::new(
        &pos,
        eval,
        tt,
        stop_signal.clone(),
        limits.hard_time_ms(),
        start_time,
    );

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    let mut best_move = Some(moves[0]);
    let mut best_score = 0;
    let mut completed_depth = 0;

    // iterative deepening: search depth 1..N and keep the best fully completed result.
    for depth in 1..=max_depth {
        let depth_start = Instant::now();

        if ctx.stats.should_stop() {
            break;
        }

        let (iteration_best_score, iteration_best_move) =
            search_with_aspiration(&mut pos, &mut ctx, depth, best_score);

        if ctx.stats.should_stop() {
            break;
        }

        best_move = Some(iteration_best_move);
        best_score = iteration_best_score;
        completed_depth = depth;

        print_uci_info(depth, best_score, &ctx.stats, ctx.tt, &iteration_best_move);

        let current_depth_time = depth_start.elapsed().as_millis() as u64;
        if should_stop_next_iteration(limits, start_time, current_depth_time) {
            stop_signal.store(true, Ordering::Relaxed);
            break;
        }

        if let Some(max_time) = limits.hard_time_ms() {
            if start_time.elapsed().as_millis() as u64 >= max_time {
                stop_signal.store(true, Ordering::Relaxed);
                break;
            }
        }
    }

    best_move.map(|mv| SearchInfo {
        best_move: mv,
        score: best_score,
        depth: completed_depth,
        nodes: ctx.stats.nodes,
        time_ms: ctx.stats.elapsed_ms(),
        tt_hits: ctx.stats.tt_hits,
    })
}

#[inline(always)]
fn search_with_aspiration<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    depth: u8,
    prev_score: i32,
) -> (i32, Move) {
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);

    if collector.as_slice().is_empty() {
        return if pos.is_in_check() {
            (checkmate_score(0), Move(0))
        } else {
            (0, Move(0))
        };
    }

    let mut moves = [Move(0); MAX_MOVES];
    let count = collector.len();
    moves[..count].copy_from_slice(&collector.as_slice()[..count]);
    let moves_slice = &mut moves[..count];

    // aspiration windows: search around the previous iteration score before widening.
    if !features::ASPIRATION_WINDOWS || depth < ASPIRATION_DEPTH {
        return search_root(pos, ctx, moves_slice, depth, -INFINITY, INFINITY);
    }

    let mut delta = INITIAL_ASPIRATION_DELTA;
    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        let (score, best_move) = search_root(pos, ctx, moves_slice, depth, alpha, beta);

        if ctx.stats.should_stop() {
            return (score, best_move);
        }

        if score > alpha && score < beta {
            return (score, best_move);
        }

        if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = alpha.saturating_sub(delta);
            delta += delta / 2;
        } else if score >= beta {
            alpha = (alpha + beta) / 2;
            beta = beta.saturating_add(delta);
            delta += delta / 2;
        }

        if delta > MAX_ASPIRATION_DELTA {
            alpha = -INFINITY;
            beta = INFINITY;
        }
    }
}

#[inline(always)]
pub(crate) fn search_root<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    moves: &mut [Move],
    depth: u8,
    mut alpha: i32,
    beta: i32,
) -> (i32, Move) {
    let in_check = pos.is_in_check();
    let alpha_start = alpha;

    // tt move ordering: seed root move ordering from the transposition table.
    let tt_move = if features::TT_MOVE_ORDERING {
        ctx.tt.probe(pos.hash()).map(|entry| entry.best_move)
    } else {
        None
    };

    let move_count = moves.len();
    let mut scores = [0i32; MAX_MOVES];

    // root move ordering: score all root moves before iterative pick-next selection.
    for i in 0..move_count {
        scores[i] = score_move(moves[i], pos, tt_move, Some(&ctx.history), 0);
    }

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    // root pvs: first move gets a full window, later moves get a scout search first.
    for i in 0..move_count {
        if ctx.stats.should_stop() {
            break;
        }

        pick_next_move(moves, &mut scores, i);
        let mv = moves[i];

        let delta = ctx.eval.update_on_move(&mut ctx.eval_state, pos, mv);
        pos.make_move(mv);
        let gives_check = pos.is_in_check();

        let score = search_root_child(
            pos,
            ctx,
            mv,
            depth,
            alpha,
            beta,
            RootMoveState {
                move_num: i,
                in_check,
                gives_check,
                pv_node: true,
                ply: 0,
            },
        );

        pos.unmake_move(mv);
        ctx.eval.update_on_undo(&mut ctx.eval_state, delta);

        if ctx.stats.should_stop() {
            return (best_score, best_move);
        }

        if i == 0 && score <= alpha {
            return (score, mv);
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    break;
                }
            }
        }
    }

    // tt store: keep the root result so later iterations can reuse the score and move.
    let flag = if best_score >= beta {
        LOWER_BOUND
    } else if best_score <= alpha_start {
        UPPER_BOUND
    } else {
        EXACT
    };

    if features::TT_CUTOFFS {
        ctx.tt.store(
            pos.hash(),
            best_move,
            score_to_tt(best_score, 0),
            depth,
            flag,
        );
    }

    (best_score, best_move)
}

#[inline(always)]
fn search_root_child<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    state: RootMoveState,
) -> i32 {
    if state.move_num == 0 || !features::PVS {
        let do_lmr = super::heuristics::should_reduce_lmr(
            depth,
            state.move_num,
            state.in_check,
            state.gives_check,
            mv,
        );

        if do_lmr {
            let reduction = super::heuristics::calculate_lmr_reduction(
                depth,
                state.move_num,
                state.pv_node,
                mv,
            );
            let reduced_depth = depth.saturating_sub(1 + reduction);
            let reduced_score = -search_node(
                pos,
                ctx,
                reduced_depth,
                -beta,
                -alpha,
                NodeState::new(true, state.pv_node, None, state.ply + 1),
            );

            if reduced_score > alpha {
                return -search_node(
                    pos,
                    ctx,
                    depth - 1,
                    -beta,
                    -alpha,
                    NodeState::new(true, state.pv_node, None, state.ply + 1),
                );
            }

            return reduced_score;
        }

        return -search_node(
            pos,
            ctx,
            depth - 1,
            -beta,
            -alpha,
            NodeState::new(true, state.pv_node, None, state.ply + 1),
        );
    }

    let do_lmr = super::heuristics::should_reduce_lmr(
        depth,
        state.move_num,
        state.in_check,
        state.gives_check,
        mv,
    );

    let mut score = if do_lmr {
        let reduction =
            super::heuristics::calculate_lmr_reduction(depth, state.move_num, state.pv_node, mv);
        let reduced_depth = depth.saturating_sub(1 + reduction);

        -search_node(
            pos,
            ctx,
            reduced_depth,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, state.ply + 1),
        )
    } else {
        -search_node(
            pos,
            ctx,
            depth - 1,
            -alpha - 1,
            -alpha,
            NodeState::new(true, false, None, state.ply + 1),
        )
    };

    if score > alpha && score < beta {
        score = -search_node(
            pos,
            ctx,
            depth - 1,
            -beta,
            -alpha,
            NodeState::new(true, state.pv_node, None, state.ply + 1),
        );
    }

    score
}
