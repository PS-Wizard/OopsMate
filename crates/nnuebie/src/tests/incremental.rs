use crate::types::Color;
use crate::types::Piece;

use super::common::{new_probe, parse_probe_fen, run_with_large_stack, to_cp};

#[test]
fn test_refresh_produces_same_result() {
    run_with_large_stack(|| {
        let mut probe1 = new_probe();
        let mut probe2 = new_probe();

        let fen = "r1bqkb1r/pppp1ppp/2n2n2/3Pp3/4P3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1";
        let (pieces, side) = parse_probe_fen(fen);

        probe1.set_position(&pieces, 0);
        let cp1 = to_cp(&pieces, side, probe1.evaluate(side));

        probe2.set_position(&pieces, 0);
        let cp2 = to_cp(&pieces, side, probe2.evaluate(side));

        assert_eq!(cp1, cp2, "Refresh should produce identical results");
    });
}

#[test]
fn test_probe_evaluation_basic() {
    run_with_large_stack(|| {
        let mut probe = new_probe();

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let (pieces, side) = parse_probe_fen(fen);
        probe.set_position(&pieces, 0);
        let cp = to_cp(&pieces, side, probe.evaluate(side));

        assert!(cp == 7, "Should favor White slightly");
    });
}

#[test]
fn test_probe_evaluation_middlegame() {
    run_with_large_stack(|| {
        let mut probe = new_probe();

        let fen = "r1bq1rk1/ppp1npbp/2np2p1/4p3/2P4N/2NP2P1/PP2PPBP/R1BQ1RK1 w - - 0 1";
        let (pieces, side) = parse_probe_fen(fen);
        probe.set_position(&pieces, 0);
        let cp = to_cp(&pieces, side, probe.evaluate(side));

        assert!(cp == 4, "Middlegame Should've been 4");
    });
}

#[test]
fn test_probe_evaluation_endgame() {
    run_with_large_stack(|| {
        let mut probe = new_probe();

        let fen = "3r1rk1/5ppp/8/8/8/8/8/3R1RK1 w - - 0 1";
        let (pieces, side) = parse_probe_fen(fen);
        probe.set_position(&pieces, 0);
        let cp = to_cp(&pieces, side, probe.evaluate(side));

        assert!(cp == -429, "White is loosin");
    });
}

#[test]
fn test_side_to_move_affects_score() {
    run_with_large_stack(|| {
        let mut probe_white = new_probe();
        let mut probe_black = new_probe();

        let fen_w = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen_b = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";

        let (p_w, s_w) = parse_probe_fen(fen_w);
        let (p_b, s_b) = parse_probe_fen(fen_b);

        probe_white.set_position(&p_w, 0);
        probe_black.set_position(&p_b, 0);

        let _cp_w = to_cp(&p_w, s_w, probe_white.evaluate(s_w));
        let _cp_b = to_cp(&p_b, s_b, probe_black.evaluate(s_b));
    });
}

#[test]
fn incremental_update_test() {
    run_with_large_stack(|| {
        let (start, _) =
            parse_probe_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let inc_cp = {
            let mut inc = new_probe();
            inc.set_position(&start, 0);
            inc.update(&[(Piece::WhitePawn, 12)], &[(Piece::WhitePawn, 28)]);
            let inc_internal = inc.evaluate(Color::Black);
            to_cp(&start, Color::Black, inc_internal)
        };

        let (moved, moved_side) =
            parse_probe_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
        let full_cp = {
            let mut full = new_probe();
            full.set_position(&moved, 0);
            let full_internal = full.evaluate(Color::Black);
            to_cp(&moved, moved_side, full_internal)
        };

        let diff = (inc_cp - full_cp).abs();
        assert!(
            diff == 0,
            "Incremental should match full within 5 cp (diff={})",
            diff
        );
    });
}

#[test]
fn castling_test() {
    run_with_large_stack(|| {
        let before_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1";
        let after_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 w kq - 0 1";

        let before = {
            let mut p = new_probe();
            let (pieces, side) = parse_probe_fen(before_fen);
            p.set_position(&pieces, 0);
            to_cp(&pieces, side, p.evaluate(side))
        };
        let after = {
            let mut p = new_probe();
            let (pieces, side) = parse_probe_fen(after_fen);
            p.set_position(&pieces, 0);
            to_cp(&pieces, side, p.evaluate(side))
        };

        assert!(
            before == -562 && after == -502,
            "White is down a couple pieces"
        );
    });
}

#[test]
fn incremental_add_piece_test() {
    run_with_large_stack(|| {
        let (empty, _) = parse_probe_fen("6k1/8/8/8/8/8/8/3K4 w - - 0 1");
        let inc_cp = {
            let mut inc = new_probe();
            inc.set_position(&empty, 0);
            inc.update(&[], &[(Piece::WhitePawn, 8)]);
            inc.update(&[], &[(Piece::WhitePawn, 9)]);
            let inc_internal = inc.evaluate(Color::White);
            to_cp(&empty, Color::White, inc_internal)
        };

        let (pawn, side) = parse_probe_fen("6k1/8/8/8/8/8/PP6/3K4 w - - 0 1");
        let full_cp = {
            let mut full = new_probe();
            full.set_position(&pawn, 0);
            let full_internal = full.evaluate(Color::White);
            to_cp(&pawn, side, full_internal)
        };

        let diff = (inc_cp - full_cp).abs();
        assert!(diff == 0, "Should be close (diff={})", diff);
    });
}
