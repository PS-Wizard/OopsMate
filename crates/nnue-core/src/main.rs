mod accumulator;
mod evaluate;
mod features;
mod fen;
mod layers;
mod loader;
mod network;
mod types;

use types::Color;

fn main() {
    // Load the networks
    let big_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../nnuebie/archive/nnue/networks/nn-1c0000000000.nnue"
    );
    let small_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../nnuebie/archive/nnue/networks/nn-37f18f62d772.nnue"
    );

    println!("LOADING NETWORK");
    println!("{}", "-".repeat(70));
    
    let (big_network, small_network) = match (
        network::Network::load(big_path, network::BIG_HALF_DIMS),
        network::Network::load(small_path, network::SMALL_HALF_DIMS),
    ) {
        (Ok(big), Ok(small)) => {
            println!("    Networks loaded successfully!");
            println!(
                "    - Big feature transformer: {} input dims, {} half dims",
                big.feature_transformer.input_dims, big.feature_transformer.half_dims
            );
            println!(
                "    - Small feature transformer: {} input dims, {} half dims",
                small.feature_transformer.input_dims, small.feature_transformer.half_dims
            );
            println!("    - FC0: half_dims → 16");
            println!("    - FC1: {} → {}", 30, 32);
            println!("    - FC2: {} → {}", 32, 1);
            println!("    - PSQT buckets: 8");
            (big, small)
        }
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("Failed to load network: {}", e);
            return;
        }
    };
    println!();

    // Test cases from validate.rs
    let test_cases = vec![
        (
            "Startpos",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            7,
        ),
        (
            "King Triggers Refresh",
            "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            -20,
        ),
        (
            "e4",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            37,
        ),
        (
            "No Queen",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1",
            -522,
        ),
        (
            "Opening",
            "r1bqkb1r/pppp1ppp/2n2n2/3Pp3/4P3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1",
            113,
        ),
        (
            "Middlegame 1",
            "r1bq1rk1/ppp1npbp/2np2p1/4p3/2P4N/2NP2P1/PP2PPBP/R1BQ1RK1 w - - 0 1",
            4,
        ),
        (
            "Middlegame 2",
            "r1bq1rk1/1pp2pbN/2np4/4p3/7N/3P2P1/1P2PPBP/R1BQ1RK1 w - - 0 1",
            389,
        ),
    ];

    let mut pass_count = 0;
    let mut fail_count = 0;

    for (name, fen_str, expected_cp) in &test_cases {
        println!("{}", "=".repeat(70));
        println!("[2] PARSING FEN: {}", name);
        println!("{}", "-".repeat(70));
        println!("    FEN: {}", fen_str);

        let (pieces, side_to_move, rule50) = fen::parse_fen(fen_str);
        println!(
            "    Side to move: {}",
            if side_to_move == Color::White {
                "White"
            } else {
                "Black"
            }
        );
        println!("    Rule50: {}", rule50);
        println!("    Pieces found: {}", pieces.len());

        // Show pieces in readable format
        println!();
        println!("    Piece list:");
        for (piece, square) in &pieces {
            println!("      {} on {}", piece.name(), types::square_name(*square));
        }
        println!();

        // Run the full evaluation
        let (score_cp_white, _details) = evaluate::evaluate_position(
            &big_network,
            &small_network,
            &pieces,
            side_to_move,
            rule50,
            true,
        );

        // Compare with expected
        let diff = (score_cp_white - expected_cp).abs();
        let passed = diff == 0;

        println!();
        println!("[RESULT] Position: {}", name);
        println!("    Computed (White): {} cp", score_cp_white);
        println!("    Expected (White): {} cp", expected_cp);
        println!("    Difference: {} cp", diff);

        if passed {
            println!("    Status: PASS");
            pass_count += 1;
        } else {
            println!("    Status: FAIL");
            fail_count += 1;
        }
        println!();
    }

    println!("{}", "=".repeat(70));
    println!("  SUMMARY: {} passed, {} failed", pass_count, fail_count);
    println!("{}", "=".repeat(70));
}
