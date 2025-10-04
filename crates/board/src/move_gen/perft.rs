use types::moves::MoveCollector;
use types::others::Piece::*;

use crate::Position;

impl Position {
    /// Perft (performance test) - counts leaf nodes at a given depth
    pub fn perft(&self, depth: u8) -> u64 {
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
            let new_pos = self.make_move(m);

            let mut check_pos = new_pos.clone();
            check_pos.side_to_move = check_pos.side_to_move.flip();
            if check_pos.is_in_check() {
                continue;
            }

            nodes += new_pos.perft(depth - 1);
        }

        nodes
    }

    /// Perft divide - shows move breakdown at root
    pub fn perft_divide(&self, depth: u8) {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        let mut total = 0u64;

        for i in 0..collector.len() {
            let m = collector[i];
            let new_pos = self.make_move(m);

            // Fixed: Temporarily flip side on a clone to check if own king is left in check
            let mut check_pos = new_pos.clone();
            check_pos.side_to_move = check_pos.side_to_move.flip();
            if check_pos.is_in_check() {
                continue;
            }

            let count = if depth <= 1 {
                1
            } else {
                new_pos.perft(depth - 1)
            };

            println!("{}: {}", m, count);
            total += count;
        }

        println!("\nTotal: {}", total);
    }

    /// Generate all pseudo-legal moves
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

    pub fn is_other_side_in_check(&self) -> bool {
        let king_sq = self.their(King).0.trailing_zeros() as usize;
        self.is_square_attacked(king_sq)
    }
}

#[cfg(test)]
mod perft_tests {
    use crate::Position;
    use std::time::Instant;

    // Helper function to run perft with timing
    fn perft_with_timing(pos: &Position, depth: u8, expected: u64) {
        let start = Instant::now();
        let nodes = pos.perft(depth);
        let elapsed = start.elapsed();
        let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;

        println!(
            "Depth {}: {} nodes in {:.3}s ({} nodes/sec)",
            depth,
            nodes,
            elapsed.as_secs_f64(),
            nps
        );
        assert_eq!(nodes, expected, "Perft({}) mismatch", depth);
    }

    #[test]
    fn perft_position_1_starting() {
        println!("\n=== Position 1: Starting Position ===");
        let pos = Position::new();

        perft_with_timing(&pos, 1, 20);
        perft_with_timing(&pos, 2, 400);
        perft_with_timing(&pos, 3, 8_902);
        perft_with_timing(&pos, 4, 197_281);
        perft_with_timing(&pos, 5, 4_865_609);
        // perft_with_timing(&pos, 6, 119_060_324); // Uncomment for deeper test
    }

    #[test]
    fn perft_position_2_kiwipete() {
        println!("\n=== Position 2: Kiwipete (Complex Middle Game) ===");
        let pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        perft_with_timing(&pos, 1, 48);
        perft_with_timing(&pos, 2, 2_039);
        perft_with_timing(&pos, 3, 97_862);
        perft_with_timing(&pos, 4, 4_085_603);
        // perft_with_timing(&pos, 5, 193_690_690); // Takes longer
    }

    #[test]
    fn perft_position_3() {
        println!("\n=== Position 3: Endgame Position ===");
        let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

        perft_with_timing(&pos, 1, 14);
        perft_with_timing(&pos, 2, 191);
        perft_with_timing(&pos, 3, 2_812);
        perft_with_timing(&pos, 4, 43_238);
        perft_with_timing(&pos, 5, 674_624);
        // perft_with_timing(&pos, 6, 11_030_083);
    }

    #[test]
    fn perft_position_4_complex() {
        println!("\n=== Position 4: Complex Position with Castling ===");
        let pos =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();

        perft_with_timing(&pos, 1, 6);
        perft_with_timing(&pos, 2, 264);
        perft_with_timing(&pos, 3, 9_467);
        perft_with_timing(&pos, 4, 422_333);
        // perft_with_timing(&pos, 5, 15_833_292);
    }

    #[test]
    fn perft_position_5_mirrored() {
        println!("\n=== Position 5: Position 4 Mirrored ===");
        let pos =
            Position::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
                .unwrap();

        perft_with_timing(&pos, 1, 6);
        perft_with_timing(&pos, 2, 264);
        perft_with_timing(&pos, 3, 9_467);
        perft_with_timing(&pos, 4, 422_333);
        // perft_with_timing(&pos, 5, 15_833_292);
    }

    #[test]
    fn perft_position_6_promotions() {
        println!("\n=== Position 6: Promotion Heavy Position ===");
        let pos = Position::from_fen("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1").unwrap();

        perft_with_timing(&pos, 1, 24);
        perft_with_timing(&pos, 2, 496);
        perft_with_timing(&pos, 3, 9_483);
        perft_with_timing(&pos, 4, 182_838);
        perft_with_timing(&pos, 5, 3_605_103);
        // perft_with_timing(&pos, 6, 71_179_139);
    }

    #[test]
    fn perft_position_7_discovered_checks() {
        println!("\n=== Position 7: Discovered Checks ===");
        let pos = Position::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();

        perft_with_timing(&pos, 1, 46);
        perft_with_timing(&pos, 2, 2_079);
        perft_with_timing(&pos, 3, 89_890);
        perft_with_timing(&pos, 4, 3_894_594);
        // perft_with_timing(&pos, 5, 164_075_551);
    }

    #[test]
    fn perft_benchmark_suite() {
        println!("\n=== PERFT BENCHMARK SUITE ===\n");

        let positions = vec![
            (
                "Starting Position",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                5,
            ),
            (
                "Kiwipete",
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                4,
            ),
            ("Position 3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5),
            (
                "Position 4",
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                4,
            ),
        ];

        let mut total_nodes = 0u64;
        let mut total_time = 0.0f64;

        for (name, fen, depth) in positions {
            println!("Testing: {} (depth {})", name, depth);
            let pos = Position::from_fen(fen).unwrap();

            let start = Instant::now();
            let nodes = pos.perft(depth);
            let elapsed = start.elapsed();

            total_nodes += nodes;
            total_time += elapsed.as_secs_f64();

            let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;
            println!(
                "  {} nodes in {:.3}s ({} nodes/sec)\n",
                nodes,
                elapsed.as_secs_f64(),
                nps
            );
        }

        let avg_nps = (total_nodes as f64 / total_time) as u64;
        println!("=== BENCHMARK RESULTS ===");
        println!("Total nodes: {}", total_nodes);
        println!("Total time: {:.3}s", total_time);
        println!("Average speed: {} nodes/sec", avg_nps);
    }

    // Individual divide tests for debugging
    #[test]
    #[ignore] // Run with: cargo test -- --ignored --nocapture
    fn divide_starting_position() {
        println!("\n=== Divide: Starting Position ===");
        let pos = Position::new();
        pos.perft_divide(5);
    }

    #[test]
    #[ignore]
    fn divide_kiwipete() {
        println!("\n=== Divide: Kiwipete ===");
        let pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        pos.perft_divide(4);
    }

    #[test]
    #[ignore]
    fn divide_position_3_depth_5() {
        println!("\n=== Divide: Position 3 Depth 5 ===");
        let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        pos.perft_divide(5);
    }
}
