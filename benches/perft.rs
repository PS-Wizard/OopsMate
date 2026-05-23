use oops_mate::{movegen::generate_all, MoveCollector, Position};
use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};
use strikes::warmup_attack_tables;

const BENCH_STACK_SIZE_BYTES: usize = 32 * 1024 * 1024;

struct PerftPosition {
    name: &'static str,
    fen: &'static str,
    depth: u8,
}

const POSITIONS: &[PerftPosition] = &[
    PerftPosition {
        name: "startpos",
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        depth: 6,
    },
    PerftPosition {
        name: "kiwipete",
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        depth: 5,
    },
    PerftPosition {
        name: "middlegame",
        fen: "r4rk1/1pp1qppp/p1np1n2/4p3/2BPP1b1/2P2N2/P1P2PPP/R1BQR1K1 w - - 0 1",
        depth: 5,
    },
    PerftPosition {
        name: "tactical",
        fen: "2rr3k/pp3pp1/1nnqbN1p/3p4/2pP4/2P3Q1/PPB2PPP/R1B1R1K1 w - - 0 1",
        depth: 5,
    },
];

fn main() {
    thread::Builder::new()
        .name("perft".to_owned())
        .stack_size(BENCH_STACK_SIZE_BYTES)
        .spawn(run)
        .expect("failed to spawn perft thread")
        .join()
        .expect("perft thread panicked");
}

fn run() {
    warmup_attack_tables();

    let mut total_nodes = 0_u64;
    let mut total_elapsed = Duration::ZERO;

    println!(
        "\n{:<14} {:>5} {:>16} {:>10} {:>14}",
        "position", "depth", "nodes", "time_ms", "nps"
    );
    println!("{:-<66}", "");

    for bench in POSITIONS {
        let mut pos = Position::from_fen(bench.fen)
            .unwrap_or_else(|_| panic!("invalid perft FEN: {}", bench.fen));

        let start = Instant::now();
        let nodes = perft(black_box(&mut pos), bench.depth);
        let elapsed = start.elapsed();

        total_nodes += nodes;
        total_elapsed += elapsed;

        println!(
            "{:<14} {:>5} {:>16} {:>10} {:>14}",
            bench.name,
            bench.depth,
            nodes,
            elapsed.as_millis(),
            nodes_per_second(nodes, elapsed),
        );
    }

    println!("{:-<66}", "");
    println!(
        "{:<14} {:>5} {:>16} {:>10} {:>14}",
        "total",
        "-",
        total_nodes,
        total_elapsed.as_millis(),
        nodes_per_second(total_nodes, total_elapsed),
    );
}

fn perft(pos: &mut Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveCollector::new();
    generate_all(pos, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for mv in moves.as_slice() {
        pos.make_move(*mv);
        nodes += perft(pos, depth - 1);
        pos.unmake_move(*mv);
    }

    nodes
}

fn nodes_per_second(nodes: u64, elapsed: Duration) -> u64 {
    let nanos = elapsed.as_nanos() as u64;
    if nanos == 0 {
        return 0;
    }

    nodes.saturating_mul(1_000_000_000) / nanos
}
