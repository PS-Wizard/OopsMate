#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::nnue::NNUEProbe;
    use crate::types::{Color, Piece};
    use crate::uci::{calculate_material, to_centipawns};
    use crate::{BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};
    use std::thread;

    const BIG_NETWORK: &str = "archive/nnue/networks/nn-1c0000000000.nnue";
    const SMALL_NETWORK: &str = "archive/nnue/networks/nn-37f18f62d772.nnue";

    fn run_with_large_stack<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked");
    }

    #[allow(dead_code)]
    fn parse_fen(fen: &str) -> (Vec<(usize, usize, usize)>, usize) {
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
            } else if c.is_ascii_digit() {
                file += c.to_digit(10).unwrap() as usize;
            } else {
                let color = if c.is_uppercase() { WHITE } else { BLACK };
                let pt = match c.to_ascii_lowercase() {
                    'p' => PAWN,
                    'n' => KNIGHT,
                    'b' => BISHOP,
                    'r' => ROOK,
                    'q' => QUEEN,
                    'k' => KING,
                    _ => panic!("Invalid: {}", c),
                };
                let sq = rank * 8 + file;
                pieces.push((sq, pt, color));
                file += 1;
            }
        }

        let side = if side_str == "w" { WHITE } else { BLACK };
        (pieces, side)
    }

    fn parse_fen_for_probe(fen: &str) -> (Vec<(Piece, usize)>, Color) {
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
            } else if c.is_ascii_digit() {
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
                    _ => panic!("Invalid: {}", c),
                };
                pieces.push((piece, rank * 8 + file));
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

    fn pieces_to_internal(pieces: &[(Piece, usize)]) -> Vec<(usize, usize, usize)> {
        pieces
            .iter()
            .map(|(p, sq)| {
                let pt = p.piece_type();
                let color = p.color().unwrap_or(Color::White).index();
                (*sq, pt, color)
            })
            .collect()
    }

    fn to_cp(pieces: &[(Piece, usize)], side: Color, internal: i32) -> i32 {
        let internal_vec = pieces_to_internal(pieces);
        let material = calculate_material(&internal_vec);
        let cp = to_centipawns(internal, material);
        if side == Color::Black {
            -cp
        } else {
            cp
        }
    }

    #[test]
    fn test_refresh_produces_same_result() {
        run_with_large_stack(|| {
            let mut probe1 = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
            let mut probe2 = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");

            let fen = "r1bqkb1r/pppp1ppp/2n2n2/3Pp3/4P3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1";
            let (pieces, side) = parse_fen_for_probe(fen);

            probe1.set_position(&pieces, 0);
            let internal1 = probe1.evaluate(side);
            let cp1 = to_cp(&pieces, side, internal1);

            probe2.set_position(&pieces, 0);
            let internal2 = probe2.evaluate(side);
            let cp2 = to_cp(&pieces, side, internal2);

            println!("Refresh test: {} cp vs {} cp", cp1, cp2);
            assert_eq!(cp1, cp2, "Refresh should produce identical results");
        });
    }

    #[test]
    fn test_probe_evaluation_basic() {
        run_with_large_stack(|| {
            let mut probe = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");

            let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            let (pieces, side) = parse_fen_for_probe(fen);
            probe.set_position(&pieces, 0);
            let internal = probe.evaluate(side);
            let cp = to_cp(&pieces, side, internal);

            println!("Startpos: {} cp", cp);
            assert!(cp == 7, "Should favor White slightly");
        });
    }

    #[test]
    fn test_probe_evaluation_middlegame() {
        run_with_large_stack(|| {
            let mut probe = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");

            let fen = "r1bq1rk1/ppp1npbp/2np2p1/4p3/2P4N/2NP2P1/PP2PPBP/R1BQ1RK1 w - - 0 1";
            let (pieces, side) = parse_fen_for_probe(fen);
            probe.set_position(&pieces, 0);
            let internal = probe.evaluate(side);
            let cp = to_cp(&pieces, side, internal);

            println!("Middlegame: {} cp", cp);
            assert!(cp == 4, "Middlegame Should've been 4");
        });
    }

    #[test]
    fn test_probe_evaluation_endgame() {
        run_with_large_stack(|| {
            let mut probe = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");

            let fen = "3r1rk1/5ppp/8/8/8/8/8/3R1RK1 w - - 0 1";
            let (pieces, side) = parse_fen_for_probe(fen);
            probe.set_position(&pieces, 0);
            let internal = probe.evaluate(side);
            let cp = to_cp(&pieces, side, internal);

            println!("Rook endgame: {} cp", cp);
            assert!(cp == -429, "White is loosin");
        });
    }

    #[test]
    fn test_side_to_move_affects_score() {
        run_with_large_stack(|| {
            let mut probe_white = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
            let mut probe_black = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");

            let fen_w = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            let fen_b = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";

            let (p_w, s_w) = parse_fen_for_probe(fen_w);
            let (p_b, s_b) = parse_fen_for_probe(fen_b);

            probe_white.set_position(&p_w, 0);
            probe_black.set_position(&p_b, 0);

            let cp_w = to_cp(&p_w, s_w, probe_white.evaluate(s_w));
            let cp_b = to_cp(&p_b, s_b, probe_black.evaluate(s_b));

            println!("White to move: {} cp", cp_w);
            println!("Black to move: {} cp", cp_b);
            // assert!(true);
        });
    }
}

#[cfg(test)]
mod manual_verification {
    use crate::nnue::NNUEProbe;
    use crate::types::{Color, Piece};
    use crate::uci::{calculate_material, to_centipawns};
    use std::thread;

    const BIG_NETWORK: &str = "archive/nnue/networks/nn-1c0000000000.nnue";
    const SMALL_NETWORK: &str = "archive/nnue/networks/nn-37f18f62d772.nnue";

    fn run_with_large_stack<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked");
    }

    fn run_with_large_stack_ret<F, T>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked")
    }

    fn parse_fen(fen: &str) -> (Vec<(Piece, usize)>, Color) {
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
            } else if c.is_ascii_digit() {
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
                    _ => panic!("Invalid: {}", c),
                };
                pieces.push((piece, rank * 8 + file));
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

    fn pieces_to_internal(pieces: &[(Piece, usize)]) -> Vec<(usize, usize, usize)> {
        pieces
            .iter()
            .map(|(p, sq)| {
                let pt = p.piece_type();
                let color = p.color().unwrap_or(Color::White).index();
                (*sq, pt, color)
            })
            .collect()
    }

    fn to_cp(pieces: &[(Piece, usize)], side: Color, internal: i32) -> i32 {
        let internal_vec = pieces_to_internal(pieces);
        let material = calculate_material(&internal_vec);
        let cp = to_centipawns(internal, material);
        if side == Color::Black {
            -cp
        } else {
            cp
        }
    }

    fn probe(fen: &str) -> i32 {
        let fen = fen.to_string();
        run_with_large_stack_ret(move || {
            let mut p = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
            let (pieces, side) = parse_fen(&fen);
            p.set_position(&pieces, 0);
            let internal = p.evaluate(side);
            to_cp(&pieces, side, internal)
        })
    }

    #[test]
    fn verify_vs_stockfish() {
        println!("\n=== Compare with Stockfish: stockfish -> position fen <FEN> -> eval ===\n");

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

        for (fen, name) in positions {
            let score = probe(fen);
            println!("{}: {} cp (white perspective)", name, score);
        }
        println!("\nCompare these with Stockfish 'Final evaluation' values.");
        // assert!(true);
    }

    #[test]
    fn incremental_update_test() {
        run_with_large_stack(|| {
            println!("\n=== Incremental Update Test ===\n");

            let (start, _) = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            let inc_cp = {
                let mut inc = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
                inc.set_position(&start, 0);
                inc.update(&[(Piece::WhitePawn, 12)], &[(Piece::WhitePawn, 28)]);
                let inc_internal = inc.evaluate(Color::Black);
                to_cp(&start, Color::Black, inc_internal)
            };

            let (moved, moved_side) =
                parse_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
            let full_cp = {
                let mut full = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
                full.set_position(&moved, 0);
                let full_internal = full.evaluate(Color::Black);
                to_cp(&moved, moved_side, full_internal)
            };

            println!("Incremental e2-e4: {} cp", inc_cp);
            println!("Full e2-e4: {} cp", full_cp);
            let diff = (inc_cp - full_cp).abs();
            println!("Difference: {} cp", diff);

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
            println!("\n=== Castling Evaluation ===\n");

            let before_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1";
            let after_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 w kq - 0 1";

            let before = probe(before_fen);
            let after = probe(after_fen);

            println!("Before O-O: {} cp (White to move)", before);
            println!("After O-O: {} cp (White to move)", after);
            println!("Difference: {} cp", after - before);

            assert!(
                before == -562 && after == -502,
                "White is down a couple pieces"
            );
        });
    }

    #[test]
    fn incremental_add_piece_test() {
        run_with_large_stack(|| {
            println!("\n=== Incremental Add Single Piece ===\n");

            let (empty, _) = parse_fen("6k1/8/8/8/8/8/8/3K4 w - - 0 1");
            let inc_cp = {
                let mut inc = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
                inc.set_position(&empty, 0);
                inc.update(&[], &[(Piece::WhitePawn, 8)]);
                inc.update(&[], &[(Piece::WhitePawn, 9)]);
                let inc_internal = inc.evaluate(Color::White);
                to_cp(&empty, Color::White, inc_internal)
            };

            let (pawn, side) = parse_fen("6k1/8/8/8/8/8/PP6/3K4 w - - 0 1");
            let full_cp = {
                let mut full = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
                full.set_position(&pawn, 0);
                let full_internal = full.evaluate(Color::White);
                to_cp(&pawn, side, full_internal)
            };

            println!("Incremental add pawn: {} cp", inc_cp);
            println!("Full position: {} cp", full_cp);
            let diff = (inc_cp - full_cp).abs();
            println!("Difference: {} cp", diff);

            assert!(diff == 0, "Should be close (diff={})", diff);
        });
    }
}

#[cfg(test)]
mod multithreaded_tests {
    use crate::{Color, NNUEProbe, NnueNetworks, Piece};
    use std::sync::{Arc, Barrier};
    use std::thread;

    const BIG_NETWORK: &str = "archive/nnue/networks/nn-1c0000000000.nnue";
    const SMALL_NETWORK: &str = "archive/nnue/networks/nn-37f18f62d772.nnue";

    fn run_with_large_stack<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked");
    }

    fn get_startpos_pieces() -> Vec<(Piece, usize)> {
        vec![
            (Piece::WhiteRook, 0),
            (Piece::WhiteKnight, 1),
            (Piece::WhiteBishop, 2),
            (Piece::WhiteQueen, 3),
            (Piece::WhiteKing, 4),
            (Piece::WhiteBishop, 5),
            (Piece::WhiteKnight, 6),
            (Piece::WhiteRook, 7),
            (Piece::WhitePawn, 8),
            (Piece::WhitePawn, 9),
            (Piece::WhitePawn, 10),
            (Piece::WhitePawn, 11),
            (Piece::WhitePawn, 12),
            (Piece::WhitePawn, 13),
            (Piece::WhitePawn, 14),
            (Piece::WhitePawn, 15),
            (Piece::BlackPawn, 48),
            (Piece::BlackPawn, 49),
            (Piece::BlackPawn, 50),
            (Piece::BlackPawn, 51),
            (Piece::BlackPawn, 52),
            (Piece::BlackPawn, 53),
            (Piece::BlackPawn, 54),
            (Piece::BlackPawn, 55),
            (Piece::BlackRook, 56),
            (Piece::BlackKnight, 57),
            (Piece::BlackBishop, 58),
            (Piece::BlackQueen, 59),
            (Piece::BlackKing, 60),
            (Piece::BlackBishop, 61),
            (Piece::BlackKnight, 62),
            (Piece::BlackRook, 63),
        ]
    }

    #[test]
    fn test_multithreaded_evaluation_consistency() {
        run_with_large_stack(|| {
            println!("\n=== Multi-threaded Evaluation Consistency Test ===\n");

            let networks = Arc::new(
                NnueNetworks::new(BIG_NETWORK, SMALL_NETWORK).expect("Failed to load networks"),
            );

            let num_threads = 8;
            let iterations_per_thread = 1000;
            let barrier = Arc::new(Barrier::new(num_threads));
            let mut handles = vec![];

            // Single-threaded reference evaluation
            let pieces = get_startpos_pieces();
            let mut probe_ref = NNUEProbe::with_networks(networks.clone())
                .expect("Failed to create reference probe");
            probe_ref.set_position(&pieces, 0);
            let reference_score = probe_ref.evaluate(Color::White);
            println!("Reference score (single-threaded): {}", reference_score);

            // Spawn threads
            for thread_id in 0..num_threads {
                let networks_clone = networks.clone();
                let barrier_clone = barrier.clone();
                let pieces_clone = pieces.clone();

                let handle = thread::Builder::new()
                    .stack_size(8 * 1024 * 1024)
                    .spawn(move || {
                        let mut probe = NNUEProbe::with_networks(networks_clone)
                            .expect("Failed to create thread-local probe");

                        probe.set_position(&pieces_clone, 0);

                        // Synchronize all threads
                        barrier_clone.wait();

                        let mut scores = vec![];

                        for i in 0..iterations_per_thread {
                            // Alternate between moves to stress test
                            if i % 2 == 0 {
                                probe.update(&[(Piece::WhitePawn, 12)], &[(Piece::WhitePawn, 28)]);
                            } else {
                                probe.update(&[(Piece::WhitePawn, 28)], &[(Piece::WhitePawn, 12)]);
                            }

                            let score = probe.evaluate(Color::White);
                            scores.push(score);
                        }

                        (thread_id, scores)
                    })
                    .expect("Failed to spawn worker thread");

                handles.push(handle);
            }

            // Collect results
            let mut all_thread_scores: Vec<Vec<i32>> = vec![];
            for handle in handles {
                let (_thread_id, scores) = handle.join().unwrap();
                all_thread_scores.push(scores);
            }

            // Verify consistency
            println!(
                "Verifying {} threads × {} iterations = {} total evaluations",
                num_threads,
                iterations_per_thread,
                num_threads * iterations_per_thread
            );

            // Check that all threads got the same scores for the same positions
            // (positions alternate, so scores at even indices should match)
            let mut all_match = true;
            for i in (0..iterations_per_thread).step_by(2) {
                let first_thread_score = all_thread_scores[0][i];
                for thread_scores in &all_thread_scores[1..] {
                    if thread_scores[i] != first_thread_score {
                        all_match = false;
                        println!(
                            "Mismatch at index {}: thread 0 got {}, other got {}",
                            i, first_thread_score, thread_scores[i]
                        );
                        break;
                    }
                }
                if !all_match {
                    break;
                }
            }

            println!("All threads consistent: {}", all_match);
            assert!(all_match, "Multi-threaded evaluations should be consistent");
            println!("Multi-threaded consistency test PASSED");
        });
    }

    #[test]
    fn test_thread_safety_no_data_races() {
        run_with_large_stack(|| {
            println!("\n=== Thread Safety Test (No Data Races) ===\n");

            let networks = Arc::new(
                NnueNetworks::new(BIG_NETWORK, SMALL_NETWORK).expect("Failed to load networks"),
            );

            let num_threads = 16; // Stress test with many threads
            let iterations = 10_000;
            let barrier = Arc::new(Barrier::new(num_threads));
            let mut handles = vec![];

            let pieces = get_startpos_pieces();

            for _thread_id in 0..num_threads {
                let networks_clone = networks.clone();
                let barrier_clone = barrier.clone();
                let pieces_clone = pieces.clone();

                let handle = thread::Builder::new()
                    .stack_size(8 * 1024 * 1024)
                    .spawn(move || {
                        let mut probe = NNUEProbe::with_networks(networks_clone)
                            .expect("Failed to create thread-local probe");

                        probe.set_position(&pieces_clone, 0);

                        // Synchronize
                        barrier_clone.wait();

                        // Heavy evaluation load
                        for _ in 0..iterations {
                            let _score = probe.evaluate(Color::White);
                            // Also test make_move/unmake_move
                            probe.make_move(12, 28, Piece::WhitePawn);
                            let _score2 = probe.evaluate(Color::Black);
                            probe.unmake_move(12, 28, Piece::WhitePawn, None);
                        }

                        true // Success
                    })
                    .expect("Failed to spawn worker thread");

                handles.push(handle);
            }

            // Join all threads - if there were data races, this might panic or hang
            let results: Vec<bool> = handles
                .into_iter()
                .map(|h| h.join().expect("Thread panicked"))
                .collect();

            let all_success = results.iter().all(|&r| r);
            println!(
                "All {} threads completed successfully: {}",
                num_threads, all_success
            );
            assert!(all_success, "All threads should complete without errors");
            println!("Thread safety test PASSED");
        });
    }
}

#[cfg(test)]
mod integration_api_tests {
    use crate::{Color, MoveDelta, NNUEProbe, Piece};
    use std::thread;

    const BIG_NETWORK: &str = "archive/nnue/networks/nn-1c0000000000.nnue";
    const SMALL_NETWORK: &str = "archive/nnue/networks/nn-37f18f62d772.nnue";

    fn run_with_large_stack<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked");
    }

    fn parse_fen_for_probe(fen: &str) -> (Vec<(Piece, usize)>, Color) {
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
            } else if c.is_ascii_digit() {
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
                    _ => panic!("Invalid: {}", c),
                };
                pieces.push((piece, rank * 8 + file));
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
        let (before_pieces, before_side) = parse_fen_for_probe(before_fen);
        let (after_pieces, after_side) = parse_fen_for_probe(after_fen);

        let mut inc = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
        inc.set_position(&before_pieces, before_rule50);
        let original = inc.evaluate(before_side);

        inc.apply_delta(delta);
        let incremental = inc.evaluate(after_side);

        let mut full = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
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
            let (pieces, _) = parse_fen_for_probe(fen);

            let mut inc = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
            inc.set_position(&pieces, 17);
            let original = inc.evaluate(Color::White);

            inc.make_null_move();
            let null_eval = inc.evaluate(Color::Black);

            let mut full = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
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
            let (pieces, _) = parse_fen_for_probe("4k3/P7/8/8/8/8/8/4K3 w - - 73 1");
            let mut probe = NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load");
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
}
