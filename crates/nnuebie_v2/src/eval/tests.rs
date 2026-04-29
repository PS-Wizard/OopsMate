use super::EvalOutput;
use super::refresh::{full_refresh_frame_big, full_refresh_frame_small};
use crate::context::AccumulatorFrame;
use crate::oopsmate_core::{Color, Move, MoveKind, Position, Square};
use crate::{NnueContext, NnueNetworks};
use oops_mate::movegen::generate_all;
use oops_mate::MoveCollector;
use std::sync::OnceLock;

static NETWORKS: OnceLock<NnueNetworks> = OnceLock::new();

fn networks() -> &'static NnueNetworks {
    NETWORKS.get_or_init(|| NnueNetworks::load_default().expect("load default networks"))
}

fn white_side_cp(output: EvalOutput, position: &Position) -> i32 {
    output.white_side_cp(position.side_to_move())
}

fn sq(text: &str) -> Square {
    Square::from_algebraic(text).unwrap()
}

fn assert_incremental_matches_full(
    networks: &NnueNetworks,
    position: &Position,
    incremental_ctx: &mut NnueContext,
) {
    let incremental = networks.evaluate(position, incremental_ctx);
    let incremental_raw = networks.evaluate_raw(position, incremental_ctx);

    let mut full_ctx = NnueContext::new();
    networks.reset_context(position, &mut full_ctx);
    let full = networks.evaluate(position, &mut full_ctx);
    let full_raw = networks.evaluate_raw(position, &mut full_ctx);

    assert_eq!(incremental.psqt, full.psqt);
    assert_eq!(incremental.positional, full.positional);
    assert_eq!(incremental.final_raw, full.final_raw);
    assert_eq!(incremental.final_raw, incremental_raw);
    assert_eq!(full.final_raw, full_raw);
    assert_eq!(incremental.final_cp, full.final_cp);
    assert_eq!(incremental.used_smallnet, full.used_smallnet);
}

fn walk_limited_tree(
    networks: &NnueNetworks,
    position: &mut Position,
    incremental_ctx: &mut NnueContext,
    depth: usize,
    branch_limit: usize,
) {
    assert_incremental_matches_full(networks, position, incremental_ctx);

    if depth == 0 {
        return;
    }

    let mut moves = MoveCollector::new();
    generate_all(position.as_host(), &mut moves);

    for &host_mv in moves.as_slice().iter().take(branch_limit) {
        let mv = Move(host_mv.0);
        incremental_ctx.push_move(position, mv);
        position.make_move(mv);
        walk_limited_tree(networks, position, incremental_ctx, depth - 1, branch_limit);
        position.unmake_move(mv);
        incremental_ctx.pop();
    }
}

#[test]
fn validate_reference_positions() {
    let cases = [
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

    let networks = networks();
    let mut ctx = NnueContext::new();

    for (name, fen, expected_cp) in cases {
        let position = Position::from_fen(fen).expect(name);
        let output = networks.evaluate(&position, &mut ctx);
        assert_eq!(white_side_cp(output, &position), expected_cp, "{name}");
    }
}

#[test]
fn finny_refresh_matches_full_refresh_on_reused_root_context() {
    let networks = networks();
    let mut ctx = NnueContext::new();

    let cases = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - - 0 1",
        "8/8/3k4/8/8/4K3/8/8 w - - 0 1",
        "r1bq1rk1/ppp1npbp/2np2p1/4p3/2P4N/2NP2P1/PP2PPBP/R1BQ1RK1 w - - 0 1",
        "r1bq1rk1/1pp2pbN/2np4/4p3/7N/3P2P1/1P2PPBP/R1BQ1RK1 w - - 0 1",
    ];

    for fen in cases {
        let position = Position::from_fen(fen).unwrap();
        networks.reset_context(&position, &mut ctx);

        let mut full = AccumulatorFrame::new();
        full_refresh_frame_big(&networks.big, &position, &mut full);
        full_refresh_frame_small(&networks.small, &position, &mut full);

        assert_eq!(
            ctx.frames[0].big_accumulation, full.big_accumulation,
            "{fen} big accum"
        );
        assert_eq!(ctx.frames[0].big_psqt, full.big_psqt, "{fen} big psqt");
        assert_eq!(
            ctx.frames[0].small_accumulation, full.small_accumulation,
            "{fen} small accum"
        );
        assert_eq!(
            ctx.frames[0].small_psqt, full.small_psqt,
            "{fen} small psqt"
        );
    }
}

#[test]
fn white_side_cp_flips_black_positions() {
    let output = EvalOutput {
        final_cp: 37,
        ..EvalOutput::ZERO
    };

    assert_eq!(output.white_side_cp(Color::White), 37);
    assert_eq!(output.white_side_cp(Color::Black), -37);
}

#[test]
fn incremental_matches_full_on_limited_startpos_tree() {
    let networks = networks();
    let mut position = Position::startpos();
    let mut incremental_ctx = NnueContext::new();
    networks.reset_context(&position, &mut incremental_ctx);

    walk_limited_tree(networks, &mut position, &mut incremental_ctx, 3, 6);
}

#[test]
fn incremental_matches_full_on_castle_en_passant_and_promotion_sequences() {
    let networks = networks();

    let sequences = [
        (
            "castle",
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            vec![Move::new(sq("e1"), sq("g1"), MoveKind::Castle)],
        ),
        (
            "en-passant",
            "8/8/8/3pP3/8/8/8/4K2k w - d6 0 1",
            vec![Move::new(sq("e5"), sq("d6"), MoveKind::EnPassant)],
        ),
        (
            "promotion",
            "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            vec![Move::new(sq("a7"), sq("a8"), MoveKind::PromotionQueen)],
        ),
        (
            "capture-promotion",
            "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            vec![Move::new(
                sq("a7"),
                sq("b8"),
                MoveKind::CapturePromotionQueen,
            )],
        ),
    ];

    for (name, fen, sequence) in sequences {
        let mut position = Position::from_fen(fen).expect(name);
        let mut incremental_ctx = NnueContext::new();
        networks.reset_context(&position, &mut incremental_ctx);
        assert_incremental_matches_full(networks, &position, &mut incremental_ctx);

        for mv in sequence {
            incremental_ctx.push_move(&position, mv);
            position.make_move(mv);
            assert_incremental_matches_full(networks, &position, &mut incremental_ctx);
            position.unmake_move(mv);
            incremental_ctx.pop();
            assert_incremental_matches_full(networks, &position, &mut incremental_ctx);
        }
    }
}
