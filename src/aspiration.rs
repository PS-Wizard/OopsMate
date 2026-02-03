use crate::{
    move_history::KillerTable,
    move_ordering::{pick_next_move, score_move},
    pvs::search_move,
    search::SearchStats,
    tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND},
    Move, MoveCollector, Position,
};

const INFINITY: i32 = 50_000;
const MAX_MOVES: usize = 256;
const INITIAL_WINDOW: i32 = 25;
const ASPIRATION_DEPTH: u8 = 8;

#[inline(always)]
pub fn search_aspiration(
    pos: &Position,
    depth: u8,
    prev_score: i32,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
) -> (i32, Move) {
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);

    if collector.as_slice().is_empty() {
        return if pos.is_in_check() {
            (-49_000 - depth as i32, Move(0))
        } else {
            (0, Move(0))
        };
    }

    let mut moves = [Move(0); MAX_MOVES];
    let count = collector.len();
    for i in 0..count {
        moves[i] = collector.as_slice()[i];
    }
    let moves_slice = &mut moves[..count];

    // If shallow, just search full window
    if depth < ASPIRATION_DEPTH {
        return search_root(
            pos,
            moves_slice,
            depth,
            -INFINITY,
            INFINITY,
            tt,
            killers,
            stats,
        );
    }

    // Aspiration Loop
    let mut delta = INITIAL_WINDOW;
    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        // We pass 'depth' to let search_root know if it should use the optimization
        let (score, best_move) =
            search_root(pos, moves_slice, depth, alpha, beta, tt, killers, stats);

        // Success Inside window
        if score > alpha && score < beta {
            return (score, best_move);
        }

        // Fail Low: Score <= Alpha
        if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = alpha.saturating_sub(delta);
            delta += delta / 2;
        }
        // Fail High: Score >= Beta
        else if score >= beta {
            alpha = (alpha + beta) / 2;
            beta = beta.saturating_add(delta);
            delta += delta / 2;
        }

        // If window gets too huge, give up and search infinite
        if delta > 1000 {
            alpha = -INFINITY;
            beta = INFINITY;
        }
    }
}

#[inline(always)]
fn search_root(
    pos: &Position,
    moves: &mut [Move],
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &mut TranspositionTable,
    killers: &mut KillerTable,
    stats: &mut SearchStats,
) -> (i32, Move) {
    let in_check = pos.is_in_check();
    let tt_move = tt.probe(pos.hash()).map(|e| e.best_move);
    let move_count = moves.len();
    let mut scores = [0i32; MAX_MOVES];

    // Score moves
    for i in 0..move_count {
        scores[i] = score_move(moves[i], pos, tt_move, Some(killers), 0);
    }

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    for i in 0..move_count {
        pick_next_move(moves, &mut scores, i);
        let mv = moves[i];
        let newpos = pos.make_move(&mv);
        let gives_check = newpos.is_in_check();

        let score = if i == 0 {
            search_move(
                &newpos,
                mv,
                depth,
                alpha,
                beta,
                i,
                in_check,
                gives_check,
                true,
                tt,
                killers,
                stats,
                0,
            )
        } else {
            // PVS for other moves
            let s = search_move(
                &newpos,
                mv,
                depth,
                alpha,
                alpha + 1,
                i,
                in_check,
                gives_check,
                true,
                tt,
                killers,
                stats,
                0,
            );
            if s > alpha && s < beta {
                search_move(
                    &newpos,
                    mv,
                    depth,
                    alpha,
                    beta,
                    i,
                    in_check,
                    gives_check,
                    true,
                    tt,
                    killers,
                    stats,
                    0,
                )
            } else {
                s
            }
        };

        // crafty's optimization
        // If the PV move (i=0) fails low (score <= alpha), abort immediately.
        // We do not waste time searching the remaining moves against this invalid alpha.
        // https://www.chessprogramming.org/PVS_and_Aspiration
        if i == 0 && score <= alpha {
            return (score, mv);
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    break; // Beta Cutoff
                }
            }
        }
    }

    // Store result
    let flag = if best_score >= beta {
        LOWER_BOUND
    } else if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(pos.hash(), best_move, best_score, depth, flag);

    (best_score, best_move)
}
