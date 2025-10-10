use crate::Position;
use types::moves::MoveCollector;

impl Position {
    pub fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        if depth == 1 {
            return collector.len() as u64;
        }

        let mut nodes = 0u64;
        for i in 0..collector.len() {
            let m = collector[i];
            let undo = self.make_move(m);
            nodes += self.perft(depth - 1);
            self.unmake_move(m, undo);
        }
        nodes
    }

    pub fn perft_divide(&mut self, depth: u8) {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);
        let mut total = 0u64;

        for i in 0..collector.len() {
            let m = collector[i];
            let undo = self.make_move(m);
            let count = if depth <= 1 { 1 } else { self.perft(depth - 1) };
            self.unmake_move(m, undo);

            println!("{}: {}", m, count);
            total += count;
        }

        println!("Total: {}", total);
    }

    pub fn generate_moves(&self, collector: &mut MoveCollector) {
        let (pinned, _checking, check_mask) =
            crate::legality::attack_constraints::get_attack_constraints(self);

        self.generate_pawn_moves(collector, pinned, check_mask);
        self.generate_knight_moves(collector, pinned, check_mask);
        self.generate_bishop_moves(collector, pinned, check_mask);
        self.generate_rook_moves(collector, pinned, check_mask);
        self.generate_queen_moves(collector, pinned, check_mask);
        self.generate_king_moves(collector);
    }
}

#[cfg(test)]
mod perft_tests {

    use crate::Position;
    use std::time::Instant;

    struct PerftTest {
        name: &'static str,
        fen: &'static str,
        depths: &'static [(u8, u64)],
    }

    const PERFT_SUITE: &[PerftTest] = &[
        PerftTest {
            name: "Starting",
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            depths: &[(5, 4_865_609), (6, 119_060_324)],
        },
        PerftTest {
            name: "Kiwipete",
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            depths: &[(5, 193_690_690)],
        },
        PerftTest {
            name: "Position 3",
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            depths: &[(6, 11_030_083), (7, 178_633_661)],
        },
        PerftTest {
            name: "Position 4",
            fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            depths: &[(5, 15_833_292)],
        },
        PerftTest {
            name: "Position 5",
            fen: "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
            depths: &[(5, 15_833_292)],
        },
        PerftTest {
            name: "Position 6",
            fen: "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
            depths: &[(6, 71_179_139)],
        },
    ];

    #[test]
    fn perft_correctness() {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║         PERFT CORRECTNESS TESTS            ║");
        println!("╚════════════════════════════════════════════╝\n");

        for test in PERFT_SUITE {
            println!("Position: {}", test.name);
            let mut pos = Position::from_fen(test.fen).unwrap();

            for &(depth, expected) in test.depths {
                let nodes = pos.perft(depth);
                assert_eq!(
                    nodes, expected,
                    "{} depth {} failed: got {}, expected {}",
                    test.name, depth, nodes, expected
                );
                println!("  ✓ Depth {}: {} nodes", depth, format_num(nodes));
            }
            println!();
        }
    }

    #[test]
    #[ignore]
    fn perft_stress_test() {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║          PERFT STRESS TEST                 ║");
        println!("╚════════════════════════════════════════════╝\n");

        let test_positions = [
            (
                "Starting",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                6,
            ),
            (
                "Kiwipete",
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                5,
            ),
            ("Position 3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6),
            (
                "Position 4",
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                5,
            ),
            ("Position 6", "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6),
        ];

        let mut total_nodes = 0u64;
        let mut total_time = 0.0;

        for (name, fen, depth) in test_positions {
            println!("Testing: {} (depth {})", name, depth);
            let mut pos = Position::from_fen(fen).unwrap();

            let start = Instant::now();
            let nodes = pos.perft(depth);
            let elapsed = start.elapsed().as_secs_f64();

            total_nodes += nodes;
            total_time += elapsed;

            let nps = (nodes as f64 / elapsed) as u64;
            println!("  {} nodes in {:.2}s", format_num(nodes), elapsed);
            println!("  {} nodes/sec\n", format_num(nps));
        }

        let avg_nps = (total_nodes as f64 / total_time) as u64;
        println!("╔════════════════════════════════════════════╗");
        println!("║              FINAL RESULTS                 ║");
        println!("╠════════════════════════════════════════════╣");
        println!("║ Total nodes:  {:>28} ║", format_num(total_nodes));
        println!("║ Total time:   {:>24.2}s ║", total_time);
        println!("║ Average NPS:  {:>28} ║", format_num(avg_nps));
        println!("╚════════════════════════════════════════════╝");
    }

    #[test]
    #[ignore]
    fn perft_single_benchmark() {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║       SINGLE POSITION BENCHMARK            ║");
        println!("╚════════════════════════════════════════════╝\n");

        let mut pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        let depth = 5;
        let runs = 3;

        println!("Position: Kiwipete");
        println!("Depth: {}", depth);
        println!("Runs: {}\n", runs);

        let mut times = Vec::new();
        let mut nodes_result = 0u64;

        for run in 1..=runs {
            let start = Instant::now();
            let nodes = pos.perft(depth);
            let elapsed = start.elapsed().as_secs_f64();
            times.push(elapsed);
            nodes_result = nodes;

            let nps = (nodes as f64 / elapsed) as u64;
            println!("Run {}: {:.3}s ({} nps)", run, elapsed, format_num(nps));
        }

        let avg_time = times.iter().sum::<f64>() / runs as f64;
        let avg_nps = (nodes_result as f64 / avg_time) as u64;

        println!("\n────────────────────────────────────────────");
        println!("Average: {:.3}s", avg_time);
        println!("Average NPS: {}", format_num(avg_nps));
        println!("Total nodes: {}", format_num(nodes_result));
        println!("────────────────────────────────────────────");
    }

    #[test]
    #[ignore]
    fn divide_debug() {
        println!("\n=== Divide Debug ===");
        let mut pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        pos.perft_divide(4);
    }

    fn format_num(n: u64) -> String {
        n.to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|x| std::str::from_utf8(x).unwrap())
            .collect::<Vec<_>>()
            .join(",")
    }
}
