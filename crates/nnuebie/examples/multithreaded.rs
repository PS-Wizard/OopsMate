use nnuebie::{Color, MoveDelta, NNUEProbe, NnueNetworks, Piece};
use std::sync::{Arc, Barrier};
use std::thread;

fn main() {
    // 1. Load the networks ONCE in the main thread.
    // This performs the heavy I/O and parsing.
    // The resulting structure is wrapped in an Arc for thread-safe shared ownership.
    let big_path = "archive/nnue/networks/nn-1c0000000000.nnue";
    let small_path = "archive/nnue/networks/nn-37f18f62d772.nnue";

    println!("Loading networks...");
    // Robust loading handling for running from different directories
    let networks = NnueNetworks::new(big_path, small_path)
        .or_else(|_| {
            NnueNetworks::new(
                "../../archive/nnue/networks/nn-1c0000000000.nnue",
                "../../archive/nnue/networks/nn-37f18f62d772.nnue",
            )
        })
        .expect("Failed to load networks");

    let shared_networks = Arc::new(networks);

    // 2. Spawn multiple worker threads (simulating LazySMP or parallel search)
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    println!("Spawning {} threads...", num_threads);

    for i in 0..num_threads {
        let networks_clone = shared_networks.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // 3. Create a thread-local NNUEProbe using the shared networks.
            // This is lightweight and allocates only the thread-local accumulators/scratch buffers.
            let mut probe = NNUEProbe::with_networks(networks_clone)
                .expect("Failed to create thread-local probe");

            // Example Position (Startpos)
            // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
            let startpos = vec![
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
            ];

            // 4. Set Position
            probe.set_position(&startpos, 0);

            // Synchronize threads for demo purposes
            barrier_clone.wait();

            // 5. Perform Evaluations
            let score = probe.evaluate(Color::White);
            println!("Thread {} initialized. Startpos Eval: {}", i, score);

            // 6. Perform Incremental Updates (e2 -> e4)
            // Remove White Pawn at 12 (e2), Add White Pawn at 28 (e4)
            let mut delta = MoveDelta::new(0);
            delta
                .push_move(12, 28, Piece::WhitePawn, Piece::WhitePawn)
                .unwrap();
            probe.apply_delta(delta);

            let score_after = probe.evaluate(Color::Black); // Black to move
            println!("Thread {} after e2e4. Eval: {}", i, score_after);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads finished successfully.");
}
