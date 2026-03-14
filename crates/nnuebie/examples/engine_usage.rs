use nnuebie::{Color, MoveDelta, NNUEProbe, Piece, Square};
use std::io;

/// Helper function to parse a FEN string into a list of pieces and the side to move.
/// This adapts FEN characters to the library's internal `Piece` and `Color` types.
fn parse_fen(fen: &str) -> (Vec<(Piece, Square)>, Color) {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let board_str = parts[0];
    let side_str = parts[1];

    let mut pieces = Vec::new();
    let mut rank = 7;
    let mut file = 0;

    for c in board_str.chars() {
        if c == '/' {
            rank -= 1;
            file = 0;
        } else if c.is_digit(10) {
            file += c.to_digit(10).unwrap() as usize;
        } else {
            let piece = match c {
                'P' => Piece::WhitePawn,
                'N' => Piece::WhiteKnight,
                'B' => Piece::WhiteBishop,
                'R' => Piece::WhiteRook,
                'Q' => Piece::WhiteQueen,
                'K' => Piece::WhiteKing,
                'p' => Piece::BlackPawn,
                'n' => Piece::BlackKnight,
                'b' => Piece::BlackBishop,
                'r' => Piece::BlackRook,
                'q' => Piece::BlackQueen,
                'k' => Piece::BlackKing,
                _ => panic!("Invalid piece char: {}", c),
            };
            let sq = rank * 8 + file;
            pieces.push((piece, sq));
            file += 1;
        }
    }

    let side = if side_str == "w" {
        Color::White
    } else {
        Color::Black
    };
    (pieces, side)
}

fn main() -> io::Result<()> {
    // 1. Initialize the NNUE Probe with paths to network files
    // You must provide the paths to the Big and Small networks.
    let big_path = "archive/nnue/networks/nn-1c0000000000.nnue";
    let small_path = "archive/nnue/networks/nn-37f18f62d772.nnue";

    println!("Loading NNUE networks...");
    // The Probe wrapper handles state, accumulators, and loading.
    let mut probe = NNUEProbe::new(big_path, small_path)?;

    // 2. Set position from FEN
    // Example: Start Position
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    println!("Setting position: {}", fen);

    let (pieces, side) = parse_fen(fen);

    // `set_position` automatically handles a full refresh of the NNUE accumulators.
    // This is required when setting up a new board state from scratch.
    probe.set_position(&pieces, 0);

    // 3. Get Evaluation
    // `evaluate` returns the internal raw score from the perspective of `side`.
    let score_internal = probe.evaluate(side);

    // Convert to Centipawns (optional, usually preferred for UCI engines).
    // The conversion depends on total non-pawn material.
    let material_count = nnuebie::uci::calculate_material_from_pieces(&pieces);
    let score_cp = nnuebie::uci::to_centipawns(score_internal, material_count);

    println!("Internal Score: {}", score_internal);
    println!("Evaluation (centipawns): {}", score_cp);

    // 4. Incremental Update Example (e2 -> e4)
    println!("\nMaking move: e2e4");

    // Define the move (White Pawn from e2 to e4)
    let from_sq = 12; // e2
    let to_sq = 28; // e4
    let piece = Piece::WhitePawn;

    let mut delta = MoveDelta::new(0);
    delta.push_move(from_sq, to_sq, piece, piece).unwrap();
    probe.apply_delta(delta);

    // Evaluate new position (now Black to move)
    let side_to_move = Color::Black;
    let score_after_internal = probe.evaluate(side_to_move);

    // Note: Incremental updates to material count would also be handled by an engine here.
    // Since e2e4 captures nothing, material count is unchanged.
    let score_after_cp = nnuebie::uci::to_centipawns(score_after_internal, material_count);

    println!(
        "Evaluation after e2e4 (Black to move): {} cp",
        score_after_cp
    );

    Ok(())
}
