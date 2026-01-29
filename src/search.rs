use crate::{evaluate::evaluate, Move, MoveCollector, Position};

const INFINITY: i32 = 50_000;
const MATE_VALUE: i32 = 49_000;

pub struct SearchStats {
    pub nodes: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        SearchStats { nodes: 0 }
    }
}

pub fn search(pos: &Position, depth: u8) -> Option<Move> {
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

    let mut best_score = -INFINITY;

    for mv in moves {
        let new_pos = pos.make_move(mv);

        let score = -negamax(&new_pos, depth - 1, -beta, -alpha, &mut stats);

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);

            if score > alpha {
                alpha = score;
            }
        }
    }

    println!(
        "Search complete. Nodes: {} Best Score: {}",
        stats.nodes, best_score
    );
    best_move
}

fn negamax(pos: &Position, depth: u8, mut alpha: i32, beta: i32, stats: &mut SearchStats) -> i32 {
    stats.nodes += 1;

    // Base Case
    if depth == 0 {
        return evaluate(pos);
    }

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // 2. Checkmate / Stalemate Detection
    if moves.is_empty() {
        if pos.is_in_check() {
            // Include distance to mate so engine prefers faster kills
            return -MATE_VALUE + (depth as i32);
        } else {
            return 0; // Stalemate
        }
    }

    let mut best_score = -INFINITY;

    // 3. Recursive Search
    for mv in moves {
        let new_pos = pos.make_move(mv);
        let score = -negamax(&new_pos, depth - 1, -beta, -alpha, stats);

        if score >= beta {
            return beta; // Fail-high / Beta cutoff
        }

        if score > best_score {
            best_score = score;
            if score > alpha {
                alpha = score;
            }
        }
    }

    best_score
}

#[cfg(test)]
mod test_search {
    use std::time::Instant;

    use utilities::algebraic::Algebraic;

    use super::*;
    use crate::Position;

    #[test]
    fn test_search_ab() {
        let depth = 5;
        let pos = Position::new();

        println!("Starting search at depth {}...", depth);

        let start = Instant::now(); // Start timer
        let move_result = search(&pos, depth);
        let duration = start.elapsed(); // Stop timer

        if let Some(m) = move_result {
            println!(
                "Depth: {}, From: {}, To: {} ",
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
