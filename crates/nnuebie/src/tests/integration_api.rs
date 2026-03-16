use crate::{Color, MoveDelta, NNUEProbe, Piece};

use super::common::{new_probe, parse_probe_fen, run_with_large_stack};

fn eval_internal(
    probe: &mut NNUEProbe,
    pieces: &[(Piece, usize)],
    side: Color,
    rule50: i32,
) -> i32 {
    probe.set_position(pieces, rule50);
    probe.evaluate(side)
}

fn assert_delta_matches(
    before_fen: &str,
    before_rule50: i32,
    after_fen: &str,
    after_rule50: i32,
    delta: MoveDelta,
) {
    let (before_pieces, before_side) = parse_probe_fen(before_fen);
    let (after_pieces, after_side) = parse_probe_fen(after_fen);

    let mut inc = new_probe();
    inc.set_position(&before_pieces, before_rule50);
    let original = inc.evaluate(before_side);

    inc.apply_delta(delta);
    let incremental = inc.evaluate(after_side);

    let mut full = new_probe();
    let refreshed = eval_internal(&mut full, &after_pieces, after_side, after_rule50);

    assert_eq!(
        incremental, refreshed,
        "incremental delta must match full refresh"
    );

    inc.undo_delta(delta);
    let restored = inc.evaluate(before_side);
    assert_eq!(
        restored, original,
        "undo_delta must restore the original evaluation"
    );
    assert_eq!(
        inc.rule50(),
        before_rule50,
        "undo_delta must restore rule50"
    );
}

#[test]
fn castling_delta_matches_full_refresh() {
    run_with_large_stack(|| {
        let mut delta = MoveDelta::new(1);
        delta
            .push_move(4, 6, Piece::WhiteKing, Piece::WhiteKing)
            .unwrap();
        delta
            .push_move(7, 5, Piece::WhiteRook, Piece::WhiteRook)
            .unwrap();

        assert_delta_matches(
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            0,
            "r3k2r/8/8/8/8/8/8/R4RK1 b kq - 1 1",
            1,
            delta,
        );
    });
}

#[test]
fn en_passant_delta_matches_full_refresh() {
    run_with_large_stack(|| {
        let mut delta = MoveDelta::new(0);
        delta
            .push_move(36, 43, Piece::WhitePawn, Piece::WhitePawn)
            .unwrap();
        delta.push_removal(35, Piece::BlackPawn).unwrap();

        assert_delta_matches(
            "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
            0,
            "4k3/8/3P4/8/8/8/8/4K3 b - - 0 1",
            0,
            delta,
        );
    });
}

#[test]
fn promotion_delta_matches_full_refresh() {
    run_with_large_stack(|| {
        let mut delta = MoveDelta::new(0);
        delta
            .push_move(48, 56, Piece::WhitePawn, Piece::WhiteQueen)
            .unwrap();

        assert_delta_matches(
            "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            0,
            "Q3k3/8/8/8/8/8/8/4K3 b - - 0 1",
            0,
            delta,
        );
    });
}

#[test]
fn capture_promotion_delta_matches_full_refresh() {
    run_with_large_stack(|| {
        let mut delta = MoveDelta::new(0);
        delta
            .push_move(48, 57, Piece::WhitePawn, Piece::WhiteQueen)
            .unwrap();
        delta.push_removal(57, Piece::BlackRook).unwrap();

        assert_delta_matches(
            "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            0,
            "1Q2k3/8/8/8/8/8/8/4K3 b - - 0 1",
            0,
            delta,
        );
    });
}

#[test]
fn null_move_matches_full_refresh() {
    run_with_large_stack(|| {
        let fen = "4k3/8/8/8/3P4/8/8/4K3 w - - 17 1";
        let (pieces, _) = parse_probe_fen(fen);

        let mut inc = new_probe();
        inc.set_position(&pieces, 17);
        let original = inc.evaluate(Color::White);

        inc.make_null_move();
        let null_eval = inc.evaluate(Color::Black);

        let mut full = new_probe();
        full.set_position(&pieces, 18);
        let refreshed = full.evaluate(Color::Black);

        assert_eq!(null_eval, refreshed, "null move must match full refresh");
        assert_eq!(inc.rule50(), 18, "null move must increment rule50");

        inc.unmake_null_move();
        assert_eq!(
            inc.evaluate(Color::White),
            original,
            "null move undo must restore eval"
        );
        assert_eq!(inc.rule50(), 17, "null move undo must restore rule50");
    });
}

#[test]
fn promotion_resets_rule50_in_make_move() {
    run_with_large_stack(|| {
        let (pieces, _) = parse_probe_fen("4k3/P7/8/8/8/8/8/4K3 w - - 73 1");
        let mut probe = new_probe();
        probe.set_position(&pieces, 73);

        probe.make_move(48, 56, Piece::WhiteQueen);

        assert_eq!(probe.rule50(), 0, "promotion should reset rule50");
        probe.unmake_move(48, 56, Piece::WhitePawn, None);
        assert_eq!(
            probe.rule50(),
            73,
            "unmake must restore pre-promotion rule50"
        );
    });
}
