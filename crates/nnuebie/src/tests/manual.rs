use super::common::{
    new_probe, parse_probe_fen, run_with_large_stack, run_with_large_stack_ret, to_cp,
};

fn probe(fen: &str) -> i32 {
    let fen = fen.to_string();
    run_with_large_stack_ret(move || {
        let mut p = new_probe();
        let (pieces, side) = parse_probe_fen(&fen);
        p.set_position(&pieces, 0);
        let internal = p.evaluate(side);
        to_cp(&pieces, side, internal)
    })
}

#[test]
fn verify_vs_stockfish() {
    run_with_large_stack(|| {
        let positions = vec![
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "Startpos",
            ),
            (
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
                "e4",
            ),
            (
                "r1bqkb1r/pppp1ppp/2n2n2/3Pp3/4P3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1",
                "QGA",
            ),
            (
                "r1bq1rk1/ppp1npbp/2np2p1/4p3/2P4N/2NP2P1/PP2PPBP/R1BQ1RK1 w - - 0 1",
                "Middlegame",
            ),
        ];

        for (fen, _name) in positions {
            let _score = probe(fen);
        }
    });
}
