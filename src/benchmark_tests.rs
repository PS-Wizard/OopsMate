use crate::search::init_lmr;
use crate::search::search;
use crate::tpt::TranspositionTable;
use crate::Position;
use std::thread;
use std::time::Instant;

struct BenchPos {
    name: &'static str,
    fen: &'static str,
    depth: u8,
}

const BENCH_STACK_SIZE_BYTES: usize = 32 * 1024 * 1024;

fn run_with_large_stack<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    thread::Builder::new()
        .stack_size(BENCH_STACK_SIZE_BYTES)
        .spawn(f)
        .expect("failed to spawn benchmark thread")
        .join()
        .expect("benchmark thread panicked");
}

#[test]
#[ignore = "Long running benchmark"]
fn run_benchmark_suite() {
    run_with_large_stack(|| {
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
                depth: 13,
            },
        ];

        let mut total_nodes = 0;
        let mut total_time_ms = 0;

        println!("\n{:=^80}", " BENCHMARK SUITE ");

        for pos_def in &positions {
            let pos = Position::from_fen(pos_def.fen)
                .unwrap_or_else(|_| panic!("Invalid FEN: {}", pos_def.fen));
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
                println!("Best Move: {}", info.best_move.to_uci());
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
    });
}
