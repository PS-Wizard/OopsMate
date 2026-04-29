use crate::eval::EvalProvider;
use crate::movegen::{analyze, generate_all_with_analysis};
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::params::{ASPIRATION_DEPTH, INFINITY, MAX_MOVES};
use crate::search::root::search::search_root;
use crate::search::score::checkmate_score;
use crate::{Move, MoveCollector, Position};

const INITIAL_ASPIRATION_DELTA: i32 = 25;
const MAX_ASPIRATION_DELTA: i32 = 1_000;

#[inline(always)]
pub(super) fn search_with_aspiration<E: EvalProvider>(
    pos: &mut Position,
    ctx: &mut SearchContext<'_, E>,
    depth: u8,
    prev_score: i32,
) -> (i32, Move) {
    let mut collector = MoveCollector::new();
    let analysis = analyze(pos);
    generate_all_with_analysis(pos, &analysis, &mut collector);

    if collector.as_slice().is_empty() {
        return if analysis.in_check() {
            (checkmate_score(0), Move(0))
        } else {
            (0, Move(0))
        };
    }

    let mut moves = [Move(0); MAX_MOVES];
    let count = collector.len();
    moves[..count].copy_from_slice(&collector.as_slice()[..count]);
    let moves_slice = &mut moves[..count];

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
