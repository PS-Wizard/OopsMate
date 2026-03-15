use super::evaluate;
use crate::Position;
use std::thread;

fn run_with_large_stack<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(f)
        .expect("failed to spawn test thread")
        .join()
        .expect("test thread panicked");
}

#[test]
#[ignore = "NNUE evaluation requires release build"]
fn start_position_eval_is_small_white_edge() {
    run_with_large_stack(|| {
        let pos = Position::new();
        let score = evaluate(&pos);
        assert_eq!(score, 7);
    });
}
