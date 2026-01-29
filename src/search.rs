use crate::{
    move_ordering::{pick_next_move, score_move},
    qsearch::qsearch,
    tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND},
    Move, MoveCollector, Position,
};

const INFINITY: i32 = 50_000;
const MATE_VALUE: i32 = 49_000;

pub struct SearchStats {
    pub nodes: u64,
    pub tt_hits: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        SearchStats {
            nodes: 0,
            tt_hits: 0,
        }
    }
}

pub fn search(pos: &Position, depth: u8, tt: &mut TranspositionTable) -> Option<Move> {
    let mut stats = SearchStats::new();
    let mut best_move = None;
    let mut alpha = -INFINITY;
    let beta = INFINITY;

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    // Get TT move
    let tt_move = tt.probe(pos.hash()).map(|e| e.best_move);

    // Score all moves
    let mut move_list: Vec<Move> = moves.to_vec();
    let mut scores: Vec<i32> = move_list
        .iter()
        .map(|m| score_move(*m, pos, tt_move))
        .collect();

    let mut best_score = -INFINITY;

    // Search moves in order
    for i in 0..move_list.len() {
        pick_next_move(&mut move_list, &mut scores, i);
        let mv = move_list[i];

        let new_pos = pos.make_move(&mv);
        let score = -negamax(&new_pos, depth - 1, -beta, -alpha, tt, &mut stats);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
            if score > alpha {
                alpha = score;
            }
        }
    }

    println!(
        "Search complete. Nodes: {} TT Hits: {} Best Score: {}",
        stats.nodes, stats.tt_hits, best_score
    );

    if let Some(mv) = best_move {
        tt.store(pos.hash(), mv, best_score, depth, EXACT);
    }

    best_move
}

fn negamax(
    pos: &Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &mut TranspositionTable,
    stats: &mut SearchStats,
) -> i32 {
    stats.nodes += 1;

    let hash = pos.hash();

    // TT probe
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
    }

    // Base case;
    if depth == 0 {
        // return evaluate(pos);
        // enter quiescence search instead of just evaluating
        return qsearch(pos, alpha, beta, stats);
    }

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // Checkmate / Stalemate detection
    if moves.is_empty() {
        if pos.is_in_check() {
            return -MATE_VALUE + (depth as i32);
        } else {
            return 0; // Stalemate
        }
    }

    // Get TT move
    let tt_move = tt_entry.map(|e| e.best_move);

    // Score all moves
    let mut move_list: Vec<Move> = moves.to_vec();
    let mut scores: Vec<i32> = move_list
        .iter()
        .map(|m| score_move(*m, pos, tt_move))
        .collect();

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    // Search moves in order
    for i in 0..move_list.len() {
        pick_next_move(&mut move_list, &mut scores, i);
        let mv = move_list[i];

        let new_pos = pos.make_move(&mv);
        let score = -negamax(&new_pos, depth - 1, -beta, -alpha, tt, stats);

        if score >= beta {
            tt.store(hash, mv, beta, depth, LOWER_BOUND);
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

    let flag = if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };

    tt.store(hash, best_move, best_score, depth, flag);

    best_score
}

#[cfg(test)]
mod test_search {
    use std::time::Instant;
    use utilities::algebraic::Algebraic;

    use super::*;
    use crate::Position;

    #[test]
    fn test_search_with_qsearch() {
        let depth = 1;
        let pos =
            Position::from_fen("rnb1kbnr/pppp1p1p/6p1/4p2q/4P3/8/PPPPQPPP/RNB1KBNR w KQkq - 0 1")
                .unwrap();
        let mut tt = TranspositionTable::new_mb(1);

        println!("Starting search at depth {}...", depth);
        let start = Instant::now();

        let move_result = search(&pos, depth, &mut tt);

        let duration = start.elapsed();

        if let Some(m) = move_result {
            println!(
                "Depth: {}, From: {}, To: {}",
                depth,
                m.from().single_notation(),
                m.to().single_notation()
            );
        } else {
            println!("No Move");
        }

        println!("Time elapsed: {:.4}s", duration.as_secs_f64());
    }

    #[test]
    fn test_search_with_tt() {
        let depth = 10;
        let pos = Position::new();
        let mut tt = TranspositionTable::new_mb(64);

        println!("Starting search at depth {}...", depth);
        let start = Instant::now();

        let move_result = search(&pos, depth, &mut tt);

        let duration = start.elapsed();

        if let Some(m) = move_result {
            println!(
                "Depth: {}, From: {}, To: {}",
                depth,
                m.from().single_notation(),
                m.to().single_notation()
            );
        } else {
            println!("No Move");
        }

        println!("Time elapsed: {:.4}s", duration.as_secs_f64());
    }
}
