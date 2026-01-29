use chess_engine::{MoveCollector, Position};
use std::time::Instant;

fn perft(pos: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);

    if depth == 1 {
        return collector.len() as u64;
    }

    let mut nodes = 0;
    for i in 0..collector.len() {
        let m = collector.get(i);
        let new_pos = pos.make_move(m);
        nodes += perft(&new_pos, depth - 1);
    }
    nodes
}

fn main() {
    let positions = [
        (
            "Starting",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            6,
            119_060_324,
        ),
        (
            "Kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            5,
            193_690_690,
        ),
        (
            "Position 3",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            7,
            178_633_661,
        ),
        (
            "Position 4",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            5,
            15_833_292,
        ),
    ];

    println!("----------------------------------------------");
    println!("MOVE GENERATION BENCHMARK");
    println!("----------------------------------------------");

    for (name, fen, depth, expected) in positions {
        let pos = Position::from_fen(fen).unwrap();

        println!("Position: {}", name);
        println!("Depth: {}", depth);

        let start = Instant::now();
        let nodes = perft(&pos, depth);
        let elapsed = start.elapsed();

        let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;

        println!("  Nodes: {}", format_number(nodes));
        println!("  Expected: {}", format_number(expected));
        println!("  Time: {:.3}s", elapsed.as_secs_f64());
        println!("  NPS: {}", format_number(nps));

        if nodes == expected {
            println!("CORRECT");
        } else {
            println!(
                "INCORRECT (diff: {})",
                (nodes as i64 - expected as i64).abs()
            );
        }
        println!();
    }
}

fn format_number(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|x| std::str::from_utf8(x).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}
