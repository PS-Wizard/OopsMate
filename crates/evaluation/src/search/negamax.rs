use crate::evaluation::evaluate::Evaluator;
use tpt::{Bound, TranspositionTable};
use types::moves::MoveCollector;

use board::Position;
use types::moves::Move;

/// Trait for searching chess positions
pub trait Searcher {
    /// Search for the best move at a given depth
    /// Returns (best_move, score in centipawns)
    fn search(&mut self, depth: u8, tt: &mut TranspositionTable) -> (Option<Move>, i32);

    /// Negamax search with alpha-beta pruning
    fn negamax(&mut self, depth: u8, alpha: i32, beta: i32, tt: &mut TranspositionTable) -> i32;
}

impl Searcher for Position {
    fn search(&mut self, depth: u8, tt: &mut TranspositionTable) -> (Option<Move>, i32) {
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        // Try TT move first if available
        let mut tt_move = None;
        if let Some(entry) = tt.probe(self.hash) {
            // If stored entry is greater than or equal to the current depth
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return (Some(entry.best_move), entry.score),
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => _ = (), // beta cutoff handled in negamax
                }
            }
            tt_move = Some(entry.best_move)
        }

        // Check TT moves first
        if let Some(tt_m) = tt_move {
            if collector.contains(tt_m) {
                let undo = self.make_move(tt_m);
                let mut score = -self.negamax(depth - 1, -beta, -alpha, tt);

                // Bonus for giving check
                if score > 50000 && self.is_in_check() {
                    score += 1;
                }

                self.unmake_move(tt_m, undo);

                if score > best_score {
                    best_score = score;
                    best_move = Some(tt_m);
                    alpha = alpha.max(score);
                }
            }
        }

        // Search Remaining
        for i in 0..collector.len() {
            let m = collector[i];

            // Skip TT move (already searched)
            if Some(m) == tt_move {
                continue;
            }

            let undo = self.make_move(m);
            // The order of alpha & Beta is swapped cause from the opponent's perspective,
            // our alpha becomes their -beta (our lower bound becomes their upper bound, negated)
            let mut score = -self.negamax(depth - 1, -beta, -alpha, tt);

            // If we're winning big (likely mate), prefer checks
            if score > 50000 {
                if self.is_in_check() {
                    score += 1; // bonus for giving check
                }
            }

            self.unmake_move(m, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
                alpha = alpha.max(score);
            }

            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        if let Some(mv) = best_move {
            tt.store(self.hash, mv, best_score, depth, bound);
        }

        (best_move, best_score)
    }

    fn negamax(&mut self, depth: u8, mut alpha: i32, beta: i32, tt: &mut TranspositionTable) -> i32 {
        let original_alpha = alpha;

        // TT probe
        if let Some(entry) = tt.probe(self.hash) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                }

                // Early cutoff
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        // Terminal Node
        if depth == 0 {
            return self.evaluate();
        }

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        // No legal moves - checkmate or stalemate
        if collector.len() == 0 {
            if self.is_in_check() {
                return -100000 - depth as i32; // Checkmate 
            } else {
                return 0; // Stalemate
            }
        }

        let mut best_score = i32::MIN;
        let mut best_move = Move::NULL;

        let tt_move = tt.probe(self.hash).map(|e| e.best_move);

        // Try TT Move First
        if let Some(tt_m) = tt_move {
            if collector.contains(tt_m) {
                let undo = self.make_move(tt_m);
                let score = -self.negamax(depth - 1, -beta, -alpha, tt);
                self.unmake_move(tt_m, undo);

                if score > best_score {
                    best_score = score;
                    best_move = tt_m;
                }

                alpha = alpha.max(score);
                if alpha >= beta {
                    // Store and return
                    tt.store(self.hash, best_move, best_score, depth, Bound::Lower);
                    return best_score;
                }
            }
        }

        for i in 0..collector.len() {
            let m = collector[i];
            if Some(m) == tt_move {
                continue; // TT Move Already searched above
            }

            let undo = self.make_move(m);
            let score = -self.negamax(depth - 1, -beta, -alpha, tt);
            self.unmake_move(m, undo);

            if score > best_score {
                best_score = score;
                best_move = m;
            }
            // Alpha-beta pruning
            alpha = alpha.max(score);
            if alpha >= beta {
                break; // Beta cutoff
            }
        }

        let bound = if best_score <= original_alpha {
            Bound::Upper // All moves failed low
        } else if best_score >= beta {
            Bound::Lower // Beta cutoff
        } else {
            Bound::Exact // Exact score
        };

        tt.store(self.hash, best_move, best_score, depth, bound);

        best_score
    }
}

#[cfg(test)]
mod search_benchmarks {

    use std::time::Instant;

    use board::Position;

    struct SearchBenchmark {
        name: &'static str,
        fen: &'static str,
        max_depth: u8,
    }

    const BENCHMARK_SUITE: &[SearchBenchmark] = &[
        SearchBenchmark {
            name: "Starting Position",
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            max_depth: 5,
        },
        SearchBenchmark {
            name: "Kiwipete",
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            max_depth: 4,
        },
        SearchBenchmark {
            name: "Middlegame",
            fen: "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
            max_depth: 5,
        },
        SearchBenchmark {
            name: "Endgame (K+P vs K)",
            fen: "8/8/8/4k3/8/8/4P3/4K3 w - - 0 1",
            max_depth: 6,
        },
        SearchBenchmark {
            name: "Tactical (Scholar's Mate Setup)",
            fen: "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1",
            max_depth: 5,
        },
    ];

    #[test]
    #[ignore]
    fn search_benchmark_suite() {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║              SEARCH PERFORMANCE BENCHMARK SUITE                ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        for test in BENCHMARK_SUITE {
            println!("┌─ {} ─", test.name);
            println!("│  FEN: {}", test.fen);
            let mut pos = Position::from_fen(test.fen).unwrap();

            println!("│");
            println!("│  Depth │    Time    │   Nodes    │    NPS     │  Score │");
            println!("│  ──────┼────────────┼────────────┼────────────┼────────┤");

            for depth in 1..=test.max_depth {
                let (nodes, time, score) = benchmark_search(&mut pos, depth);
                let nps = if time > 0.0 {
                    (nodes as f64 / time) as u64
                } else {
                    0
                };

                println!(
                    "│  {:>6} │ {:>8.3}s │ {:>10} │ {:>10} │ {:>6} │",
                    depth,
                    time,
                    format_num(nodes),
                    format_num(nps),
                    score
                );
            }
            println!("└────────────────────────────────────────────────────────────────\n");
        }
    }

    #[test]
    #[ignore]
    fn compare_perft_vs_search() {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║              PERFT vs SEARCH COMPARISON                        ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        let positions = [
            (
                "Starting",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                4,
            ),
            (
                "Middlegame",
                "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
                4,
            ),
        ];

        for (name, fen, depth) in positions {
            println!("Position: {} (depth {})", name, depth);
            let mut pos = Position::from_fen(fen).unwrap();

            // Perft
            let start = Instant::now();
            let perft_nodes = pos.perft(depth);
            let perft_time = start.elapsed().as_secs_f64();
            let perft_nps = (perft_nodes as f64 / perft_time) as u64;

            // Search
            let (search_nodes, search_time, _) = benchmark_search(&mut pos, depth);
            let search_nps = if search_time > 0.0 {
                (search_nodes as f64 / search_time) as u64
            } else {
                0
            };

            println!("┌─ Perft");
            println!("│  Nodes: {}", format_num(perft_nodes));
            println!("│  Time:  {:.3}s", perft_time);
            println!("│  NPS:   {}", format_num(perft_nps));
            println!("│");
            println!("├─ Search");
            println!("│  Nodes: {}", format_num(search_nodes));
            println!("│  Time:  {:.3}s", search_time);
            println!("│  NPS:   {}", format_num(search_nps));
            println!("│");
            println!("└─ Overhead");
            println!("   Slowdown: {:.2}x", search_time / perft_time);
            println!(
                "   Evaluation overhead per node: {:.2}µs",
                ((search_time - perft_time) / search_nodes as f64) * 1_000_000.0
            );
            println!();
        }
    }

    fn benchmark_search(pos: &mut Position, depth: u8) -> (u64, f64, i32) {
        let mut node_count = 0u64;

        let start = Instant::now();
        let (_best_move, score) = search_with_node_count(pos, depth, &mut node_count);
        let elapsed = start.elapsed().as_secs_f64();

        (node_count, elapsed, score)
    }

    fn search_with_node_count(
        pos: &mut Position,
        depth: u8,
        node_count: &mut u64,
    ) -> (Option<types::moves::Move>, i32) {
        use types::moves::MoveCollector;

        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);

        for i in 0..collector.len() {
            let m = collector[i];
            let undo = pos.make_move(m);
            let score = -negamax_with_count(pos, depth - 1, -beta, -alpha, node_count);
            pos.unmake_move(m, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
                alpha = alpha.max(score);
            }
        }

        (best_move, best_score)
    }

    fn negamax_with_count(
        pos: &mut Position,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        node_count: &mut u64,
    ) -> i32 {
        use crate::evaluation::evaluate::Evaluator;
        use types::moves::MoveCollector;

        *node_count += 1;

        if depth == 0 {
            return pos.evaluate();
        }

        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);

        if collector.len() == 0 {
            return if pos.is_in_check() {
                -100000 - depth as i32
            } else {
                0
            };
        }

        let mut best_score = i32::MIN;

        for i in 0..collector.len() {
            let m = collector[i];
            let undo = pos.make_move(m);
            let score = -negamax_with_count(pos, depth - 1, -beta, -alpha, node_count);
            pos.unmake_move(m, undo);

            best_score = best_score.max(score);

            // Alpha-beta pruning
            alpha = alpha.max(score);
            if alpha >= beta {
                break; // Beta cutoff
            }
        }

        best_score
    }

    fn format_num(n: u64) -> String {
        n.to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|x| std::str::from_utf8(x).unwrap())
            .collect::<Vec<_>>()
            .join(",")
    }
}
