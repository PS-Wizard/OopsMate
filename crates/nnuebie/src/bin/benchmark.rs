use nnuebie::{Color, NNUEProbe, Piece};
use std::time::Instant;

struct ParsedFen {
    pieces: Vec<(Piece, usize)>,
    side: Color,
    rule50: i32,
}

struct ToggleMove {
    from: usize,
    to: usize,
    piece: Piece,
    captured: Option<Piece>,
}

fn parse_fen(fen: &str) -> (Vec<(Piece, usize)>, Color, i32) {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let board_str = parts[0];
    let side_str = parts[1];
    let rule50_str = if parts.len() > 4 { parts[4] } else { "0" };
    let rule50: i32 = rule50_str.parse().unwrap_or(0);

    let mut pieces = Vec::with_capacity(32);
    let mut rank = 7;
    let mut file = 0;

    for c in board_str.chars() {
        if c == '/' {
            rank -= 1;
            file = 0;
        } else if c.is_ascii_digit() {
            file += c.to_digit(10).unwrap() as usize;
        } else {
            let piece = match (c.is_uppercase(), c.to_ascii_lowercase()) {
                (true, 'p') => Piece::WhitePawn,
                (true, 'n') => Piece::WhiteKnight,
                (true, 'b') => Piece::WhiteBishop,
                (true, 'r') => Piece::WhiteRook,
                (true, 'q') => Piece::WhiteQueen,
                (true, 'k') => Piece::WhiteKing,
                (false, 'p') => Piece::BlackPawn,
                (false, 'n') => Piece::BlackKnight,
                (false, 'b') => Piece::BlackBishop,
                (false, 'r') => Piece::BlackRook,
                (false, 'q') => Piece::BlackQueen,
                (false, 'k') => Piece::BlackKing,
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
    (pieces, side, rule50)
}

fn report_section(label: &str, evals: usize, start: Instant) -> f64 {
    let duration = start.elapsed();
    let nps = evals as f64 / duration.as_secs_f64();
    println!(
        "{}: {:.2} evals/sec ({} evals, {:.2}s)",
        label,
        nps,
        evals,
        duration.as_secs_f64()
    );
    nps
}

fn sq(file: u8, rank: u8) -> usize {
    let file_idx = (file - b'a') as usize;
    let rank_idx = (rank - 1) as usize;
    rank_idx * 8 + file_idx
}

fn main() {
    let big_path = "archive/nnue/networks/nn-1c0000000000.nnue";
    let small_path = "archive/nnue/networks/nn-37f18f62d772.nnue";

    println!("Loading networks...");

    let mut probe = match NNUEProbe::new(big_path, small_path) {
        Ok(p) => p,
        Err(_) => NNUEProbe::new(
            "../../archive/nnue/networks/nn-1c0000000000.nnue",
            "../../archive/nnue/networks/nn-37f18f62d772.nnue",
        )
        .expect("Failed to load networks"),
    };

    let fen_list = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
        "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1",
        "r1bqkb1r/pppp1ppp/2n2n2/3Pp3/4P3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1",
        "r1bq1rk1/ppp1npbp/2np2p1/4p3/2P4N/2NP2P1/PP2PPBP/R1BQ1RK1 w - - 0 1",
        "r1bq1rk1/1pp2pbN/2np4/4p3/7N/3P2P1/1P2PPBP/R1BQ1RK1 w - - 0 1",
        "4k3/8/8/8/8/8/4K3/8 w - - 0 1",
        "4k3/8/8/8/8/8/4K2P/8 w - - 0 1",
        "4k3/8/8/8/8/8/4K2R/8 w - - 0 1",
        "6k1/5ppp/8/8/8/8/5PPP/6K1 w - - 0 1",
        "4k3/8/8/8/8/3p4/4K3/8 b - - 0 1",
        "6k1/8/8/8/8/8/4K3/6R1 w - - 0 1",
    ];

    let parsed_fens: Vec<ParsedFen> = fen_list
        .iter()
        .map(|fen| {
            let (pieces, side, rule50) = parse_fen(fen);
            ParsedFen {
                pieces,
                side,
                rule50,
            }
        })
        .collect();

    // Warmup
    if let Some(first) = parsed_fens.first() {
        probe.set_position(&first.pieces, first.rule50);
        for _ in 0..100 {
            std::hint::black_box(probe.evaluate(first.side));
        }
    }

    println!("Benchmarking with {} FENs...", parsed_fens.len());

    let mut results: Vec<(&'static str, f64)> = Vec::new();

    // Section 1: Full refresh (set_position) across a small FEN corpus
    let full_refresh_target = 2_000_000usize;
    let fen_count = parsed_fens.len().max(1);
    let full_refresh_loops = (full_refresh_target / fen_count).max(1);
    let full_refresh_evals = full_refresh_loops * fen_count;

    let start = Instant::now();
    for _ in 0..full_refresh_loops {
        for entry in &parsed_fens {
            probe.set_position(&entry.pieces, entry.rule50);
            std::hint::black_box(probe.evaluate(entry.side));
        }
    }
    let nps = report_section("Full Refresh (FEN corpus)", full_refresh_evals, start);
    results.push(("Full Refresh (FEN corpus)", nps));

    // Section 2: Incremental mixed moves on a midgame-like position
    let base_fen = "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4";
    let (base_pieces, _base_side, base_rule50) = parse_fen(base_fen);
    probe.set_position(&base_pieces, base_rule50);

    let toggles = [
        ToggleMove {
            from: sq(b'a', 2),
            to: sq(b'a', 3),
            piece: Piece::WhitePawn,
            captured: None,
        },
        ToggleMove {
            from: sq(b'f', 3),
            to: sq(b'g', 5),
            piece: Piece::WhiteKnight,
            captured: None,
        },
        ToggleMove {
            from: sq(b'b', 5),
            to: sq(b'c', 6),
            piece: Piece::WhiteBishop,
            captured: Some(Piece::BlackKnight),
        },
        ToggleMove {
            from: sq(b'e', 1),
            to: sq(b'f', 1),
            piece: Piece::WhiteKing,
            captured: None,
        },
        ToggleMove {
            from: sq(b'a', 6),
            to: sq(b'a', 5),
            piece: Piece::BlackPawn,
            captured: None,
        },
        ToggleMove {
            from: sq(b'b', 7),
            to: sq(b'b', 6),
            piece: Piece::BlackPawn,
            captured: None,
        },
    ];

    let toggle_cycles = 1_000_000usize;
    let incremental_evals = toggle_cycles * toggles.len() * 2;

    let start = Instant::now();
    for _ in 0..toggle_cycles {
        for mv in &toggles {
            probe.make_move(mv.from, mv.to, mv.piece);
            let stm = mv.piece.color().unwrap_or(Color::White);
            std::hint::black_box(probe.evaluate(stm));
            probe.unmake_move(mv.from, mv.to, mv.piece, mv.captured);
            std::hint::black_box(probe.evaluate(stm));
        }
    }
    let nps = report_section("Incremental (mixed moves)", incremental_evals, start);
    results.push(("Incremental (mixed moves)", nps));

    // Section 3: King-move refresh cost (forces refresh)
    probe.set_position(&base_pieces, base_rule50);
    let king_cycles = 2_500_000usize;
    let king_evals = king_cycles * 2;
    let king_from = sq(b'e', 1);
    let king_to = sq(b'f', 1);

    let start = Instant::now();
    for _ in 0..king_cycles {
        probe.make_move(king_from, king_to, Piece::WhiteKing);
        std::hint::black_box(probe.evaluate(Color::White));
        probe.unmake_move(king_from, king_to, Piece::WhiteKing, None);
        std::hint::black_box(probe.evaluate(Color::White));
    }
    let nps = report_section("King Refresh (toggle)", king_evals, start);
    results.push(("King Refresh (toggle)", nps));

    println!("\nSummary:");
    for (label, nps) in results {
        println!("- {}: {:.2} evals/sec", label, nps);
    }
}
