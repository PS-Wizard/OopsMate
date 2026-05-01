use std::hint::black_box;
use std::time::Instant;

use strikes::{
    BLACK, WHITE, bishop_attacks, king_attacks, knight_attacks, line_between, pawn_attacks,
    queen_attacks, rook_attacks,
};

const ITERATIONS: usize = 20_000_000;
const OCCUPANCIES: [u64; 8] = [
    0x0000_0000_0000_0000,
    0x0000_0008_1000_0000,
    0x0000_1818_2418_0000,
    0x00ff_0000_0000_ff00,
    0x8142_2400_0024_4281,
    0x00ff_2400_1800_ff00,
    0x55aa_55aa_55aa_55aa,
    0xffff_ffff_ffff_ffff,
];

fn main() {
    println!("strikes lookup benchmark");
    println!("iterations per case: {ITERATIONS}");
    println!();

    bench_slider("rook", rook_attacks);
    bench_slider("bishop", bishop_attacks);
    bench_slider("queen", queen_attacks);
    bench_leaper("knight", knight_attacks);
    bench_leaper("king", king_attacks);
    bench_pawn("pawn white", WHITE);
    bench_pawn("pawn black", BLACK);
    bench_geometry();
}

fn bench_slider(name: &str, attack_fn: fn(usize, u64) -> u64) {
    let mut sink = 0u64;
    let start = Instant::now();

    for i in 0..ITERATIONS {
        let square = i & 63;
        let occupied = OCCUPANCIES[i & (OCCUPANCIES.len() - 1)] ^ ((i as u64) << (square & 15));
        sink ^= black_box(attack_fn(black_box(square), black_box(occupied)));
    }

    report(name, start.elapsed(), sink);
}

fn bench_leaper(name: &str, attack_fn: fn(usize) -> u64) {
    let mut sink = 0u64;
    let start = Instant::now();

    for i in 0..ITERATIONS {
        let square = i & 63;
        sink ^= black_box(attack_fn(black_box(square)));
    }

    report(name, start.elapsed(), sink);
}

fn bench_pawn(name: &str, color: usize) {
    let mut sink = 0u64;
    let start = Instant::now();

    for i in 0..ITERATIONS {
        let square = i & 63;
        sink ^= black_box(pawn_attacks(black_box(color), black_box(square)));
    }

    report(name, start.elapsed(), sink);
}

fn bench_geometry() {
    let mut sink = 0u64;
    let start = Instant::now();

    for i in 0..ITERATIONS {
        let from = i & 63;
        let to = (i.wrapping_mul(37)) & 63;
        sink ^= black_box(line_between(black_box(from), black_box(to)));
    }

    report("line_between", start.elapsed(), sink);
}

fn report(name: &str, elapsed: std::time::Duration, sink: u64) {
    let total_ns = elapsed.as_nanos() as f64;
    let ns_per_op = total_ns / ITERATIONS as f64;
    let mops = ITERATIONS as f64 / elapsed.as_secs_f64() / 1_000_000.0;

    println!(
        "{name:12}  {:>8.3} ns/op  {:>8.2} Mops/s  sink=0x{sink:016x}",
        ns_per_op, mops
    );
}
