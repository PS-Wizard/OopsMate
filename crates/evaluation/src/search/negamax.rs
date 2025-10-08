use crate::evaluation::evaluate::Evaluator;
use std::f32::INFINITY;
use types::moves::MoveCollector;

use board::Position;
use types::moves::Move;

/// Trait for searching chess positions
pub trait Searcher {
    /// Search for the best move at a given depth
    /// Returns (best_move, score)
    fn search(&self, depth: u8) -> (Option<Move>, f32);

    /// Classic negamax search with no alpha beta pruning atm.
    fn negamax(&self, depth: u8, _alpha: f32, _beta: f32) -> f32;
}

impl Searcher for Position {
    fn search(&self, depth: u8) -> (Option<Move>, f32) {
        let mut best_move = None;
        let mut best_score = -INFINITY;

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        for i in 0..collector.len() {
            let m = collector[i];
            let new_pos = self.make_move(m);
            let mut score = -new_pos.negamax(depth - 1, -INFINITY, INFINITY);

            // Penalize Repetitions
            if new_pos == *self {
                score -= 50.0;
            }

            // If we're winning big (likely mate), prefer checks
            if score > 50000.0 {
                if new_pos.is_in_check() {
                    score += 1.0; // Tiny bonus for giving check
                }
            }

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
        }

        (best_move, best_score)
    }

    fn negamax(&self, depth: u8, alpha: f32, beta: f32) -> f32 {
        if depth == 0 {
            return self.evaluate() as f32;
        }

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        // No legal moves - checkmate or stalemate
        if collector.len() == 0 {
            if self.is_in_check() {
                return -100000.0 - depth as f32; // Checkmate
            } else {
                return 0.0; // Stalemate
            }
        }

        let mut best_score = -INFINITY;

        for i in 0..collector.len() {
            let m = collector[i];
            let new_pos = self.make_move(m);

            let score = -new_pos.negamax(depth - 1, -beta, -alpha);
            best_score = best_score.max(score);
        }

        best_score
    }
}

#[cfg(test)]
mod negamax_tests {
    use std::time::Instant;

    use board::Position;

    use crate::search::negamax::Searcher;

    #[test]
    fn test_search_starting_position() {
        println!("\n=== Search: Starting Position ===");
        let pos = Position::new();

        for depth in 1..=4 {
            let start = Instant::now();
            let (best_move, score) = pos.search(depth);
            let elapsed = start.elapsed();

            if let Some(best_move) = best_move {
                println!(
                    "Depth {}: Best move: from: {:?}, to: {:?}, Score: {:.2}, Time: {:.3}s",
                    depth,
                    best_move.from(),
                    best_move.to(),
                    score,
                    elapsed.as_secs_f64()
                );
            }

            assert!(best_move.is_some(), "Should find a move at depth {}", depth);
        }
    }

    #[test]
    fn test_mate_in_one() {
        println!("\n=== Mate in One ===");
        // Back rank mate: Ra8#
        let pos = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1").unwrap();

        let (best_move, score) = pos.search(3);
        if let Some(best_move) = best_move {
            println!(
                "Best move: from: {:?}, to: {:?}",
                best_move.from(),
                best_move.to(),
            );
        }

        assert!(best_move.is_some());
        assert!(score > 50000.0, "Should detect mate with high score");
    }

    #[test]
    fn test_avoid_checkmate() {
        println!("\n=== Avoid Checkmate ===");
        // Black is threatened with back rank mate, must do something
        // Pretty much the same position as above but playing as black instead
        let pos = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/1R4K1 b - - 0 1").unwrap();

        let (best_move, _score) = pos.search(3);
        if let Some(best_move) = best_move {
            println!(
                "Best move: from: {:?}, to: {:?}",
                best_move.from(),
                best_move.to(),
            );
        }

        assert!(best_move.is_some());
    }

    #[test]
    fn test_capture_hanging_piece() {
        println!("\n=== Capture Hanging Queen ===");
        let pos =
            Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/3qP3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();

        let (best_move, score) = pos.search(4);
        if let Some(best_move) = best_move {
            println!(
                "Best move: from: {:?}, to: {:?}",
                best_move.from(),
                best_move.to(),
            );
        }

        assert!(best_move.is_some());
        assert!(score > 500.0, "Should recognize winning the queen");
    }

    #[test]
    fn test_stalemate_detection() {
        println!("\n=== Stalemate Position ===");
        let pos = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();

        let score = pos.negamax(1, f32::NEG_INFINITY, f32::INFINITY);
        println!("Stalemate score: {:.2}", score);

        assert_eq!(score, 0.0, "Stalemate should score 0");
    }

    #[test]
    fn test_checkmate_detection() {
        println!("\n=== Checkmate Position ===");
        let pos = Position::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();

        let score = pos.negamax(1, f32::NEG_INFINITY, f32::INFINITY);
        println!("Checkmate score: {:.2}", score);

        assert!(
            score < -50000.0,
            "Checkmate should return very negative score"
        );
    }

    #[test]
    fn test_tactical_position() {
        println!("\n=== Tactical Position ===");
        let pos = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
        )
        .unwrap();

        for depth in 1..=4 {
            let start = Instant::now();
            let (best_move, score) = pos.search(depth);
            let elapsed = start.elapsed();

            if let Some(best_move) = best_move {
                println!(
                    "Depth {}: Best move: from: {:?}, to: {:?}, Score: {:.2}, Time: {:.3}s",
                    depth,
                    best_move.from(),
                    best_move.to(),
                    score,
                    elapsed.as_secs_f64()
                );
            }
        }
    }

    #[test]
    fn test_endgame_position() {
        println!("\n=== Endgame Position ===");
        let pos = Position::from_fen("8/8/8/4k3/8/8/4P3/4K3 w - - 0 1").unwrap();

        let (best_move, _score) = pos.search(5);
        if let Some(best_move) = best_move {
            println!(
                "Best move: from: {:?}, to: {:?}",
                best_move.from(),
                best_move.to(),
            );
        }

        assert!(best_move.is_some());
    }

    #[test]
    fn test_search_consistency() {
        println!("\n=== Search Consistency Test ===");
        let pos = Position::new();

        // Search multiple times - should get same result
        let (move1, score1) = pos.search(3);
        let (move2, score2) = pos.search(3);

        println!("First search: {:?}, {:.2}", move1, score1);
        println!("Second search: {:?}, {:.2}", move2, score2);

        assert_eq!(move1, move2, "Should get same move");
        assert_eq!(score1, score2, "Should get same score");
    }

    #[test]
    fn test_search_depth_increases_score_accuracy() {
        println!("\n=== Depth Comparison ===");
        let pos =
            Position::from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();

        for depth in 1..=5 {
            let start = Instant::now();
            let (best_move, score) = pos.search(depth);
            let elapsed = start.elapsed();

            println!(
                "Depth {}: Move: {:?}, Score: {:.2}, Time: {:.3}s",
                depth,
                best_move,
                score,
                elapsed.as_secs_f64()
            );

            assert!(best_move.is_some());
            assert!(score.is_finite());
        }
    }

    #[test]
    fn test_forced_checkmate_sequence() {
        println!("\n=== Forced Mate in 2 ===");
        let pos = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1",
        )
        .unwrap();

        let (best_move, score) = pos.search(4);
        println!("Best move: {:?}, Score: {:.2}", best_move, score);

        if let Some(best_move) = best_move {
            println!(
                "Best move: from: {:?}, to: {:?}",
                best_move.from(),
                best_move.to(),
            );
        }
        assert!(score > 5000.0, "Should see forced mate");
    }

    #[test]
    #[ignore]
    fn benchmark_search_performance() {
        println!("\n=== Search Performance Benchmark ===");

        let positions = vec![
            (
                "Starting",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                4,
            ),
            (
                "Open",
                "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
                4,
            ),
            (
                "Tactical",
                "3rk2p/p1pb3r/1pn1pnb1/Pp1Pp1p1/4P2q/RBP2N2/1PB2PPP/1N2KQ2 w - - 0 1",
                4,
            ),
        ];

        for (name, fen, depth) in positions {
            let pos = Position::from_fen(fen).unwrap();

            let start = Instant::now();
            let (best_move, score) = pos.search(depth);
            let elapsed = start.elapsed();

            if let Some(best_move) = best_move {
                println!(
                    "name: {}, Depth {}: Best move: from: {:?}, to: {:?}, Score: {:.2}, Time: {:.3}s",
                    name,
                    depth,
                    best_move.from(),
                    best_move.to(),
                    score,
                    elapsed.as_secs_f64()
                );
            }
        }
    }
}
