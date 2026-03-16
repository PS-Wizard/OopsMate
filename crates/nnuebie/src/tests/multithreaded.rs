use crate::{Color, NNUEProbe, Piece};
use std::sync::{Arc, Barrier};
use std::thread;

use super::common::{
    get_startpos_pieces, load_networks, run_with_large_stack, TEST_STACK_SIZE_BYTES,
};

#[test]
fn test_multithreaded_evaluation_consistency() {
    run_with_large_stack(|| {
        let networks = load_networks();

        let num_threads = 8;
        let iterations_per_thread = 1000;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];

        let pieces = get_startpos_pieces();
        let mut probe_ref =
            NNUEProbe::with_networks(networks.clone()).expect("Failed to create reference probe");
        probe_ref.set_position(&pieces, 0);
        let _reference_score = probe_ref.evaluate(Color::White);

        for thread_id in 0..num_threads {
            let networks_clone = networks.clone();
            let barrier_clone = barrier.clone();
            let pieces_clone = pieces.clone();

            let handle = thread::Builder::new()
                .stack_size(TEST_STACK_SIZE_BYTES)
                .spawn(move || {
                    let mut probe = NNUEProbe::with_networks(networks_clone)
                        .expect("Failed to create thread-local probe");

                    probe.set_position(&pieces_clone, 0);
                    barrier_clone.wait();

                    let mut scores = vec![];

                    for i in 0..iterations_per_thread {
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

        let mut all_thread_scores: Vec<Vec<i32>> = vec![];
        for handle in handles {
            let (_thread_id, scores) = handle.join().unwrap();
            all_thread_scores.push(scores);
        }

        let mut all_match = true;
        for i in (0..iterations_per_thread).step_by(2) {
            let first_thread_score = all_thread_scores[0][i];
            for thread_scores in &all_thread_scores[1..] {
                if thread_scores[i] != first_thread_score {
                    all_match = false;
                    break;
                }
            }
            if !all_match {
                break;
            }
        }

        assert!(all_match, "Multi-threaded evaluations should be consistent");
    });
}

#[test]
fn test_thread_safety_no_data_races() {
    run_with_large_stack(|| {
        let networks = load_networks();

        let num_threads = 16;
        let iterations = 10_000;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];
        let pieces = get_startpos_pieces();

        for _thread_id in 0..num_threads {
            let networks_clone = networks.clone();
            let barrier_clone = barrier.clone();
            let pieces_clone = pieces.clone();

            let handle = thread::Builder::new()
                .stack_size(TEST_STACK_SIZE_BYTES)
                .spawn(move || {
                    let mut probe = NNUEProbe::with_networks(networks_clone)
                        .expect("Failed to create thread-local probe");

                    probe.set_position(&pieces_clone, 0);
                    barrier_clone.wait();

                    for _ in 0..iterations {
                        let _score = probe.evaluate(Color::White);
                        probe.make_move(12, 28, Piece::WhitePawn);
                        let _score2 = probe.evaluate(Color::Black);
                        probe.unmake_move(12, 28, Piece::WhitePawn, None);
                    }

                    true
                })
                .expect("Failed to spawn worker thread");

            handles.push(handle);
        }

        let results: Vec<bool> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread panicked"))
            .collect();

        let all_success = results.iter().all(|&r| r);
        assert!(all_success, "All threads should complete without errors");
    });
}
