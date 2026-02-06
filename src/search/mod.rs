pub mod alphabeta;
pub mod ordering;
pub mod parallel;
pub mod params;
pub mod pruning;

pub use alphabeta::*;
pub use ordering::*;
pub use params::*;
pub use pruning::*;
pub use parallel::*;

use crate::{
    tpt::TranspositionTable,
    Move, Position,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::io::Write;

// ============================================================================
//  STRUCTS
// ============================================================================

pub struct SearchStats {
    pub nodes: u64,
    pub tt_hits: u64,
    pub stop_signal: Option<Arc<AtomicBool>>,
}

impl SearchStats {
    pub fn new(stop_signal: Option<Arc<AtomicBool>>) -> Self {
        SearchStats {
            nodes: 0,
            tt_hits: 0,
            stop_signal,
        }
    }

    #[inline(always)]
    pub fn should_stop(&self) -> bool {
        // Check periodically (every 2048 nodes)
        if self.nodes & 2047 == 0 {
            if let Some(signal) = &self.stop_signal {
                return signal.load(Ordering::Relaxed);
            }
        }
        false
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

// ============================================================================
//  MAIN SEARCH ENTRY POINT
// ============================================================================

/// Main search function with iterative deepening and aspiration windows
pub fn search(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: Arc<TranspositionTable>,
    threads: usize,
) -> Option<SearchInfo> {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let threads = threads.max(1);

    // Mark new search for TT aging - only done once by master
    tt.new_search();

    // Spawn helper threads
    let mut handles = Vec::new();
    if threads > 1 {
        for id in 1..threads {
            let pos_clone = *pos; 
            let tt_clone = tt.clone();
            let signal_clone = stop_signal.clone();
            
            handles.push(std::thread::spawn(move || {
                parallel::search_driver(
                    &pos_clone,
                    max_depth,
                    None, // Helpers don't manage time directly
                    &tt_clone,
                    signal_clone,
                    id,
                )
            }));
        }
    }

    // Run master search
    let info = parallel::search_driver(
        pos,
        max_depth,
        max_time_ms,
        &tt,
        stop_signal.clone(),
        0,
    );

    // Signal helpers to stop
    stop_signal.store(true, Ordering::Relaxed);

    // Join helpers
    for handle in handles {
        let _ = handle.join();
    }

    info
}

// ============================================================================
//  HELPERS
// ============================================================================

pub fn print_uci_info(
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

pub fn should_stop_search(
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

pub fn move_to_uci(m: &Move) -> String {
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

// ============================================================================
//  TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Overflows On Debug / Need Release"]
    fn test_iterative_deepening() {
        use crate::search::init_lmr;
        use std::sync::Arc;

        let depth = 18;
        // let pos = Position::new();
        let pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap_or_default();

        let tt = Arc::new(TranspositionTable::new_mb(512));
        init_lmr();

        println!("Starting iterative deepening search to depth {}...", depth);
        let start = std::time::Instant::now();

        let result = search(&pos, depth, None, tt.clone(), 1);

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
