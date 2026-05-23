use oops_mate::search::{init_lmr, search};
use oops_mate::tpt::TranspositionTable;
use oops_mate::Position;
use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};
use strikes::warmup_attack_tables;

const BENCH_STACK_SIZE_BYTES: usize = 32 * 1024 * 1024;
const TT_MB: usize = 256;

struct BenchPosition {
    name: &'static str,
    fen: &'static str,
    depth: u8,
}

const POSITIONS: &[BenchPosition] = &[
    BenchPosition {
        name: "startpos",
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        depth: 8,
    },
    BenchPosition {
        name: "kiwipete",
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        depth: 8,
    },
    BenchPosition {
        name: "middlegame",
        fen: "r4rk1/1pp1qppp/p1np1n2/4p3/2BPP1b1/2P2N2/P1P2PPP/R1BQR1K1 w - - 0 1",
        depth: 8,
    },
    BenchPosition {
        name: "tactical",
        fen: "2rr3k/pp3pp1/1nnqbN1p/3p4/2pP4/2P3Q1/PPB2PPP/R1B1R1K1 w - - 0 1",
        depth: 7,
    },
];

fn main() {
    thread::Builder::new()
        .name("search-perf".to_owned())
        .stack_size(BENCH_STACK_SIZE_BYTES)
        .spawn(run)
        .expect("failed to spawn benchmark thread")
        .join()
        .expect("benchmark thread panicked");
}

fn run() {
    warmup_attack_tables();
    init_lmr();

    let mut total_nodes = 0_u64;
    let mut total_elapsed = Duration::ZERO;

    println!(
        "\n{:<14} {:>5} {:>14} {:>10} {:>14}  bestmove",
        "position", "depth", "nodes", "time_ms", "nps"
    );
    println!("{:-<84}", "");

    for bench in POSITIONS {
        let pos = Position::from_fen(bench.fen)
            .unwrap_or_else(|_| panic!("invalid benchmark FEN: {}", bench.fen));
        let mut tt = TranspositionTable::new_mb(TT_MB);

        let start = Instant::now();
        let info = search(black_box(&pos), bench.depth, None, black_box(&mut tt))
            .unwrap_or_else(|| panic!("search returned no result for {}", bench.name));
        let elapsed = start.elapsed();

        let elapsed_ms = elapsed.as_millis() as u64;
        let nps = nodes_per_second(info.nodes, elapsed);

        total_nodes += info.nodes;
        total_elapsed += elapsed;

        println!(
            "{:<14} {:>5} {:>14} {:>10} {:>14}  {}",
            bench.name,
            info.depth,
            info.nodes,
            elapsed_ms,
            nps,
            info.best_move.to_uci(),
        );
    }

    println!("{:-<84}", "");
    println!(
        "{:<14} {:>5} {:>14} {:>10} {:>14}",
        "total",
        "-",
        total_nodes,
        total_elapsed.as_millis(),
        nodes_per_second(total_nodes, total_elapsed),
    );
}

fn nodes_per_second(nodes: u64, elapsed: Duration) -> u64 {
    let nanos = elapsed.as_nanos() as u64;
    if nanos == 0 {
        return 0;
    }

    nodes.saturating_mul(1_000_000_000) / nanos
}
