use std::arch::x86_64::_pext_u64;
use std::hint::black_box;
use std::time::Instant;

use strikes::{
    BISHOP_ATTACKS, BISHOP_MASKS, BISHOP_OFFSETS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS,
    ROOK_ATTACKS, ROOK_MASKS, ROOK_OFFSETS,
};

fn bench_op<F>(name: &str, iterations: u64, mut op: F)
where
    F: FnMut(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        op();
    }
    let duration = start.elapsed();
    let ns_per_op = duration.as_nanos() as f64 / iterations as f64;

    println!(
        "{:<15} | Total: {:<10.3?} | Avg: {:.3} ns/op | Ops: {}",
        name, duration, ns_per_op, iterations
    );
}

fn main() {
    println!("old-style strikes lookup benchmark");
    let iterations = 10_000_000;
    let dummy_blockers = 0x00FF_00FF_00FF_00FFu64;

    bench_op("Rook (PEXT)", iterations, || {
        let sq = black_box(36usize);
        let mask = ROOK_MASKS[sq];
        let idx = unsafe { _pext_u64(dummy_blockers, mask) as usize };
        let _ = black_box(ROOK_ATTACKS[ROOK_OFFSETS[sq] as usize + idx]);
    });

    bench_op("Bishop (PEXT)", iterations, || {
        let sq = black_box(36usize);
        let mask = BISHOP_MASKS[sq];
        let idx = unsafe { _pext_u64(dummy_blockers, mask) as usize };
        let _ = black_box(BISHOP_ATTACKS[BISHOP_OFFSETS[sq] as usize + idx]);
    });

    bench_op("Knight", iterations, || {
        let sq = black_box(36usize);
        let _ = black_box(KNIGHT_ATTACKS[sq]);
    });

    bench_op("King", iterations, || {
        let sq = black_box(36usize);
        let _ = black_box(KING_ATTACKS[sq]);
    });

    bench_op("Pawn (White)", iterations, || {
        let sq = black_box(36usize);
        let _ = black_box(PAWN_ATTACKS[0][sq]);
    });
}
