pub mod position;
pub mod types;

pub mod evaluate;
pub mod movegen;

pub mod see;

pub mod move_history;
pub mod move_ordering;
pub mod pruning;
pub mod qsearch;
pub mod search;
pub mod time_control;
pub mod tpt;
pub mod uci;
pub mod zobrist;

pub use position::Position;
pub use types::*;

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use crate::pruning::init_lmr;
    use crate::search::search;
    use crate::tpt::TranspositionTable;
    use std::time::Instant;

    struct BenchPos {
        name: &'static str,
        fen: &'static str,
        depth: u8,
    }

    #[test]
    #[ignore = "Long running benchmark"]
    fn run_benchmark_suite() {
        init_lmr();
        
        let positions = [
            BenchPos {
                name: "Start Position",
                fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                depth: 14,
            },
            BenchPos {
                name: "KiwiPete (Tricky)",
                fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                depth: 14,
            },
            BenchPos {
                name: "Middlegame (Giuoco Piano)",
                fen: "r4rk1/1pp1qppp/p1np1n2/4p3/2BPP1b1/2P2N2/P1P2PPP/R1BQR1K1 w - - 0 1",
                depth: 14,
            },
            BenchPos {
                name: "Endgame (Rook+Pawn)",
                fen: "8/8/2p5/4k3/1r6/8/2R2K2/8 w - - 0 1",
                depth: 16,
            },
            BenchPos {
                name: "Tactical (WAC-2)",
                fen: "2rr3k/pp3pp1/1nnqbN1p/3p4/2pP4/2P3Q1/PPB2PPP/R1B1R1K1 w - - 0 1",
                depth: 13, // Fast tactical search
            }
        ];

        let mut total_nodes = 0;
        let mut total_time_ms = 0;

        println!("\n{:=^80}", " BENCHMARK SUITE ");
        
        for pos_def in &positions {
            let pos = Position::from_fen(pos_def.fen)
                .unwrap_or_else(|_| panic!("Invalid FEN: {}", pos_def.fen));
            
            // 256MB TT
            let mut tt = TranspositionTable::new_mb(256);
            
            println!("\nRunning: {}", pos_def.name);
            println!("FEN: {}", pos_def.fen);
            
            let start = Instant::now();
            let result = search(&pos, pos_def.depth, None, &mut tt);
            let duration = start.elapsed();
            
            if let Some(info) = result {
                let time_ms = duration.as_millis() as u64;
                let nps = if time_ms > 0 {
                    (info.nodes * 1000) / time_ms
                } else {
                    0
                };
                
                total_nodes += info.nodes;
                total_time_ms += time_ms;

                println!("{:<80}", "");
                println!("Best Move: {}", move_to_uci(&info.best_move));
                println!("Score:     {:<10} (cp)", info.score);
                println!("Depth:     {:<10}", info.depth);
                println!("Nodes:     {:<10}", info.nodes);
                println!("Time:      {:.3}s", duration.as_secs_f64());
                println!("NPS:       {}", nps);
                println!("TT Hits:   {}", info.tt_hits);
            } else {
                println!("NO RESULT FOUND");
            }
        }

        let total_nps = if total_time_ms > 0 {
            (total_nodes * 1000) / total_time_ms
        } else {
            0
        };

        println!("\n{:=^80}", " SUMMARY ");
        println!("Total Nodes: {}", total_nodes);
        println!("Total Time:  {:.3}s", total_time_ms as f64 / 1000.0);
        println!("Overall NPS: {}", total_nps);
        println!("{:=^80}\n", "");
    }

    fn move_to_uci(m: &crate::Move) -> String {
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
}
