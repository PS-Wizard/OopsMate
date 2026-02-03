use crate::{
    move_history::KillerTable,
    move_ordering::{pick_next_move, score_move},
    pvs::search_move,
    tpt::{TranspositionTable, EXACT},
    Move, MoveCollector, Position,
};
use std::{io::Write, time::Instant};

const INFINITY: i32 = 50_000;
const MAX_MOVES: usize = 256;

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

pub struct SearchInfo {
    pub best_move: Move,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
    pub time_ms: u64,
    pub tt_hits: u64,
}

/// Main search function with iterative deepening
pub fn search(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: &mut TranspositionTable,
) -> Option<SearchInfo> {
    let start_time = Instant::now();
    let mut stats = SearchStats::new();
    let mut killers = KillerTable::new();
    let mut best_move = None;
    let mut best_score = -INFINITY;

    // Mark new search for TT aging
    tt.new_search();

    // Generate moves once
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    let move_count = moves.len();
    let in_check = pos.is_in_check();

    // Iterative deepening loop
    for depth in 1..=max_depth {
        let depth_start = Instant::now();
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        // Get TT move from previous iteration
        let tt_move = tt.probe(pos.hash()).map(|e| e.best_move);

        let mut move_list = [Move(0); MAX_MOVES];
        let mut scores = [0i32; MAX_MOVES];

        for i in 0..move_count {
            move_list[i] = moves[i];
            scores[i] = score_move(moves[i], pos, tt_move, Some(&killers), 0);
        }

        let mut iteration_best_move = None;
        let mut iteration_best_score = -INFINITY;

        // Search moves in order
        for i in 0..move_count {
            pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
            let mv = move_list[i];
            let newpos = pos.make_move(&mv);
            let gives_check = newpos.is_in_check();
            let score = search_move(
                &newpos,
                mv,
                depth,
                alpha,
                beta,
                i, // move number
                in_check,
                gives_check,
                true, // PV node at root
                tt,
                &mut killers,
                &mut stats,
                0, // ply = 0 at root
            );

            if score > iteration_best_score {
                iteration_best_score = score;
                iteration_best_move = Some(mv);

                if score > alpha {
                    alpha = score;
                }
            }

            // Check time during search
            if let Some(max_time) = max_time_ms {
                if start_time.elapsed().as_millis() as u64 >= max_time {
                    if iteration_best_move.is_none() && best_move.is_some() {
                        return best_move.map(|mv| SearchInfo {
                            best_move: mv,
                            score: best_score,
                            depth: (depth - 1).max(1),
                            nodes: stats.nodes,
                            time_ms: start_time.elapsed().as_millis() as u64,
                            tt_hits: stats.tt_hits,
                        });
                    }
                }
            }
        }

        // Update best move for this depth
        if let Some(mv) = iteration_best_move {
            best_move = Some(mv);
            best_score = iteration_best_score;

            // Store in TT
            tt.store(pos.hash(), mv, best_score, depth, EXACT);

            // Print UCI info
            print_uci_info(depth, best_score, &stats, start_time, tt, &mv);

            let current_depth_time = depth_start.elapsed().as_millis() as u64;

            // Time management - break if we're likely to run out of time
            if should_stop_search(max_time_ms, start_time, current_depth_time) {
                break;
            }
        } else {
            break;
        }
    }

    best_move.map(|mv| SearchInfo {
        best_move: mv,
        score: best_score,
        depth: max_depth,
        nodes: stats.nodes,
        time_ms: start_time.elapsed().as_millis() as u64,
        tt_hits: stats.tt_hits,
    })
}

fn print_uci_info(
    depth: u8,
    score: i32,
    stats: &SearchStats,
    start_time: Instant,
    tt: &TranspositionTable,
    mv: &Move,
) {
    let elapsed = start_time.elapsed().as_millis() as u64;
    let nps = if elapsed > 0 {
        (stats.nodes * 1000) / elapsed
    } else {
        0
    };

    println!(
        "info depth {} score cp {} nodes {} time {} nps {} hashfull {} pv {}",
        depth,
        score,
        stats.nodes,
        elapsed,
        nps,
        tt.hashfull(),
        move_to_uci(mv)
    );

    let _ = std::io::stdout().flush();
}

fn should_stop_search(
    max_time_ms: Option<u64>,
    start_time: Instant,
    current_depth_time: u64,
) -> bool {
    if let Some(max_time) = max_time_ms {
        let elapsed_total = start_time.elapsed().as_millis() as u64;
        let time_remaining = max_time.saturating_sub(elapsed_total);
        let predicted_next_depth = current_depth_time.saturating_mul(4);

        predicted_next_depth > time_remaining || elapsed_total * 10 > max_time * 7
    } else {
        false
    }
}

fn move_to_uci(m: &Move) -> String {
    let from = m.from();
    let to = m.to();

    let from_sq = format!(
        "{}{}",
        (b'a' + (from % 8) as u8) as char,
        (b'1' + (from / 8) as u8) as char
    );
    let to_sq = format!(
        "{}{}",
        (b'a' + (to % 8) as u8) as char,
        (b'1' + (to / 8) as u8) as char
    );

    if m.is_promotion() {
        let promo = match m.move_type() {
            crate::types::MoveType::PromotionQueen
            | crate::types::MoveType::CapturePromotionQueen => 'q',
            crate::types::MoveType::PromotionRook
            | crate::types::MoveType::CapturePromotionRook => 'r',
            crate::types::MoveType::PromotionBishop
            | crate::types::MoveType::CapturePromotionBishop => 'b',
            crate::types::MoveType::PromotionKnight
            | crate::types::MoveType::CapturePromotionKnight => 'n',
            _ => unreachable!(),
        };
        format!("{}{}{}", from_sq, to_sq, promo)
    } else {
        format!("{}{}", from_sq, to_sq)
    }
}

#[cfg(test)]
mod test_search {
    use super::*;
    use crate::{lmr::init, Position};

    #[test]
    #[ignore = "Overflows On Debug / Need Release"]
    fn test_iterative_deepening() {
        let depth = 18;
        let pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap_or_default();
        let mut tt = TranspositionTable::new_mb(256);
        init();

        println!("Starting iterative deepening search to depth {}...", depth);
        let start = std::time::Instant::now();

        let result = search(&pos, depth, None, &mut tt);

        let duration = start.elapsed();

        if let Some(info) = result {
            println!(
                "Best move: {} (depth {}, score {}, nodes {}, time {:.3}s, nps {})",
                move_to_uci(&info.best_move),
                info.depth,
                info.score,
                info.nodes,
                duration.as_secs_f64(),
                if duration.as_millis() > 0 {
                    (info.nodes * 1000) / duration.as_millis() as u64
                } else {
                    0
                }
            );
        } else {
            println!("No move found");
        }
    }
}
