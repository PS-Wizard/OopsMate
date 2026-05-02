use nnuebie_v2::oopsmate_core::{Move, MoveKind, Position, Square};
use nnuebie_v2::{NnueContext, NnueNetworks};
use std::hint::black_box;
use std::time::Instant;

struct SectionResult {
    label: &'static str,
    full_nps: f64,
    raw_nps: f64,
}

fn sq(file: u8, rank: u8) -> Square {
    Square::from_file_rank(file - b'a', rank - 1).expect("valid square")
}

#[inline(always)]
fn evaluate<const RAW: bool>(
    networks: &NnueNetworks,
    position: &Position,
    ctx: &mut NnueContext,
) -> i32 {
    if RAW {
        networks.evaluate_raw(position, ctx)
    } else {
        networks.evaluate(position, ctx).final_raw
    }
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

fn report_pair(label: &'static str, full_nps: f64, raw_nps: f64) -> SectionResult {
    println!("{label} [full]:");
    println!("  {:.2} evals/sec", full_nps);
    println!("{label} [raw]:");
    println!("  {:.2} evals/sec", raw_nps);
    let delta_pct = (raw_nps / full_nps - 1.0) * 100.0;
    println!("{label} delta: {delta_pct:+.2}% raw vs full\n");

    SectionResult {
        label,
        full_nps,
        raw_nps,
    }
}

fn bench_full_refresh<const RAW: bool>(
    networks: &NnueNetworks,
    positions: &[Position],
    eval_target: usize,
) -> f64 {
    let fen_count = positions.len().max(1);
    let loops = (eval_target / fen_count).max(1);
    let evals = loops * fen_count;
    let mut ctx = NnueContext::new();

    let start = Instant::now();
    for _ in 0..loops {
        for position in positions {
            networks.reset_context(position, &mut ctx);
            black_box(evaluate::<RAW>(networks, position, &mut ctx));
        }
    }

    report_section(
        if RAW {
            "Full Refresh (FEN corpus, raw path)"
        } else {
            "Full Refresh (FEN corpus, full path)"
        },
        evals,
        start,
    )
}

fn bench_incremental<const RAW: bool>(
    networks: &NnueNetworks,
    base_fen: &str,
    toggles: &[Move],
    cycles: usize,
) -> f64 {
    let mut position = Position::from_fen(base_fen).expect("valid base FEN");
    let mut ctx = NnueContext::new();
    networks.reset_context(&position, &mut ctx);
    let evals = cycles * toggles.len() * 2;

    let start = Instant::now();
    for _ in 0..cycles {
        for &mv in toggles {
            ctx.push_move(&position, mv);
            position.make_move(mv);
            black_box(evaluate::<RAW>(networks, &position, &mut ctx));
            position.unmake_move(mv);
            ctx.pop();
            black_box(evaluate::<RAW>(networks, &position, &mut ctx));
        }
    }

    report_section(
        if RAW {
            "Incremental (mixed moves, raw path)"
        } else {
            "Incremental (mixed moves, full path)"
        },
        evals,
        start,
    )
}

fn bench_king_refresh<const RAW: bool>(
    networks: &NnueNetworks,
    base_fen: &str,
    king_move: Move,
    cycles: usize,
) -> f64 {
    let mut position = Position::from_fen(base_fen).expect("valid base FEN");
    let mut ctx = NnueContext::new();
    networks.reset_context(&position, &mut ctx);
    let evals = cycles * 2;

    let start = Instant::now();
    for _ in 0..cycles {
        ctx.push_move(&position, king_move);
        position.make_move(king_move);
        black_box(evaluate::<RAW>(networks, &position, &mut ctx));
        position.unmake_move(king_move);
        ctx.pop();
        black_box(evaluate::<RAW>(networks, &position, &mut ctx));
    }

    report_section(
        if RAW {
            "King Refresh (toggle, raw path)"
        } else {
            "King Refresh (toggle, full path)"
        },
        evals,
        start,
    )
}

fn main() {
    println!("Loading networks...");
    let networks = NnueNetworks::load_default().expect("load default networks");

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

    let positions: Vec<Position> = fen_list
        .iter()
        .map(|fen| Position::from_fen(fen).expect("valid benchmark FEN"))
        .collect();

    if let Some(first) = positions.first() {
        let mut full_ctx = NnueContext::new();
        networks.reset_context(first, &mut full_ctx);
        for _ in 0..100 {
            black_box(evaluate::<false>(&networks, first, &mut full_ctx));
        }

        let mut raw_ctx = NnueContext::new();
        networks.reset_context(first, &mut raw_ctx);
        for _ in 0..100 {
            black_box(evaluate::<true>(&networks, first, &mut raw_ctx));
        }
    }

    println!("Benchmarking with {} FENs...", positions.len());

    let base_fen = "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4";
    let toggles = [
        Move::new(sq(b'a', 2), sq(b'a', 3), MoveKind::Quiet),
        Move::new(sq(b'f', 3), sq(b'g', 5), MoveKind::Quiet),
        Move::new(sq(b'b', 5), sq(b'c', 6), MoveKind::Capture),
        Move::new(sq(b'e', 1), sq(b'f', 1), MoveKind::Quiet),
        Move::new(sq(b'a', 6), sq(b'a', 5), MoveKind::Quiet),
        Move::new(sq(b'b', 7), sq(b'b', 6), MoveKind::Quiet),
    ];
    let king_move = Move::new(sq(b'e', 1), sq(b'f', 1), MoveKind::Quiet);

    let results = [
        report_pair(
            "Full Refresh",
            bench_full_refresh::<false>(&networks, &positions, 2_000_000),
            bench_full_refresh::<true>(&networks, &positions, 2_000_000),
        ),
        report_pair(
            "Incremental",
            bench_incremental::<false>(&networks, base_fen, &toggles, 1_000_000),
            bench_incremental::<true>(&networks, base_fen, &toggles, 1_000_000),
        ),
        report_pair(
            "King Refresh",
            bench_king_refresh::<false>(&networks, base_fen, king_move, 2_500_000),
            bench_king_refresh::<true>(&networks, base_fen, king_move, 2_500_000),
        ),
    ];

    println!("Summary:");
    for result in results {
        let delta_pct = (result.raw_nps / result.full_nps - 1.0) * 100.0;
        println!(
            "- {}: full {:.2} evals/sec | raw {:.2} evals/sec | delta {:+.2}%",
            result.label, result.full_nps, result.raw_nps, delta_pct
        );
    }
}
