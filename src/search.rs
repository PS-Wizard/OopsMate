use crate::{
    aspiration::search_aspiration, move_history::KillerTable, tpt::TranspositionTable, Move,
    MoveCollector, Position,
};
use std::{io::Write, time::Instant};

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

/// Main search function with iterative deepening and aspiration windows
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
    let mut best_score = 0; // Initialize to 0 for first aspiration window

    // Mark new search for TT aging
    tt.new_search();

    // Generate moves once to check for game over
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    // Iterative deepening loop
    for depth in 1..=max_depth {
        let depth_start = Instant::now();

        // search_aspiration handles both shallow (full window) and deep (aspiration) searches
        // It also handles TT storage internally, so we don't need to store again here
        let (iteration_best_score, iteration_best_move) =
            search_aspiration(pos, depth, best_score, tt, &mut killers, &mut stats);

        // Update best move for this depth
        best_move = Some(iteration_best_move);
        best_score = iteration_best_score;

        // NOTE: We do NOT store in TT here because search_aspiration/search_with_window
        // already stored the result with the correct bound flag

        // Print UCI info
        print_uci_info(
            depth,
            best_score,
            &stats,
            start_time,
            tt,
            &iteration_best_move,
        );

        let current_depth_time = depth_start.elapsed().as_millis() as u64;

        // Time management - break if we're likely to run out of time
        if should_stop_search(max_time_ms, start_time, current_depth_time) {
            break;
        }

        // Check time during search
        if let Some(max_time) = max_time_ms {
            if start_time.elapsed().as_millis() as u64 >= max_time {
                break;
            }
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
        let pos = Position::new();
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
