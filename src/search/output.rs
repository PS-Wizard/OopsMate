use super::context::SearchStats;
use crate::{tpt::TranspositionTable, Move};
use std::io::Write;

pub(crate) fn print_uci_info(
    depth: u8,
    score: i32,
    stats: &SearchStats,
    tt: &TranspositionTable,
    mv: &Move,
) {
    let elapsed = stats.elapsed_ms();
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
        mv.to_uci()
    );

    let _ = std::io::stdout().flush();
}
