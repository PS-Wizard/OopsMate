use nnuebie_v2::oopsmate_core::Position;
use nnuebie_v2::{NnueContext, NnueNetworks};

fn main() {
    let networks = NnueNetworks::load_default().expect("load default networks");
    let mut ctx = NnueContext::new();

    let test_cases = [
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

    for (name, fen, expected_cp) in test_cases {
        let position = Position::from_fen(fen).expect("valid FEN");
        let output = networks.evaluate(&position, &mut ctx);
        let white_side_cp = output.white_side_cp(position.side_to_move());
        let diff = white_side_cp - expected_cp;

        println!("Position: {name}");
        println!("FEN: {fen}");
        println!("PSQT: {}", output.psqt);
        println!("Positional: {}", output.positional);
        println!("Internal score (STM): {}", output.final_raw);
        println!("Centipawns (STM): {}", output.final_cp);
        println!("Centipawns (White side): {white_side_cp}");
        println!("Expected CP: {expected_cp}");
        println!("Result: {}", if diff == 0 { "PASS" } else { "FAIL" });
        println!("Diff: {diff}");
        println!("--------------------------------------------------");
    }
}
