use crate::{
    attacks::{
        bishops::{generate_bishop_attacks, generate_bishop_masks},
        kings::generate_king_attacks,
        knights::generate_knight_attacks,
        pawns::generate_pawn_attacks,
        rooks::{generate_rook_attacks, generate_rook_masks},
    },
    paths::{between::generate_between, through::generate_line},
    table_builder::generate_attack_table,
};
use std::sync::LazyLock;

mod attacks;
mod enumerate;
mod paths;
mod table_builder;

// Attacks & Masks
pub static PAWN_ATTACKS: [[u64; 64]; 2] = generate_pawn_attacks();
pub static KING_ATTACKS: [u64; 64] = generate_king_attacks();
pub static KNIGHT_ATTACKS: [u64; 64] = generate_knight_attacks();
pub static ROOK_MASKS: [u64; 64] = generate_rook_masks();
pub static BISHOP_MASKS: [u64; 64] = generate_bishop_masks();
pub static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| generate_attack_table(generate_rook_attacks, &ROOK_MASKS));
pub static BISHOP_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| generate_attack_table(generate_bishop_attacks, &BISHOP_MASKS));

// Ray between 2 given indices
pub static BETWEEN: [[u64; 64]; 64] = generate_between();
pub static THROUGH: [[u64; 64]; 64] = generate_line();

/// Gets all indices containing to given square, i.e
/// line_between("a2","c2") -> "b2"
#[inline(always)]
pub fn line_between(from: usize, to: usize) -> u64 {
    BETWEEN[from][to]
}

/// Gets all indices containing to given square, i.e
/// line_through("a1","b1") -> "a1,b1,c1,d1,e1,...,h1"
#[inline(always)]
pub fn line_through(sq1: usize, sq2: usize) -> u64 {
    THROUGH[sq1][sq2]
}

/// Function to warm up attack tables, move stuff into cpu cache
pub fn warmup_attack_tables() {
    use std::arch::x86_64::_pext_u64;

    // Force LazyLock initialization for all tables
    let _ = BISHOP_ATTACKS.len();
    let _ = ROOK_ATTACKS.len();

    let mut sink = 0u64;

    // --- Warm up masks explicitly ---
    for &mask in BISHOP_MASKS.iter() {
        sink ^= mask;
    }
    for &mask in ROOK_MASKS.iter() {
        sink ^= mask;
    }

    // --- Warm up fixed tables ---
    for sq in 0..64 {
        sink ^= KING_ATTACKS[sq];
        sink ^= KNIGHT_ATTACKS[sq];
    }

    // --- Warm up bishop attacks ---
    for sq in 0..64 {
        let mask = BISHOP_MASKS[sq];
        let table = &BISHOP_ATTACKS[sq];
        for idx in 0..table.len() {
            // simulate real _pext usage
            let blockers = idx as u64;
            let _ = unsafe { _pext_u64(blockers, mask) };
            sink ^= table[idx];
        }
    }

    // --- Warm up rook attacks ---
    for sq in 0..64 {
        let mask = ROOK_MASKS[sq];
        let table = &ROOK_ATTACKS[sq];
        for idx in 0..table.len() {
            let blockers = idx as u64;
            let _ = unsafe { _pext_u64(blockers, mask) };
            sink ^= table[idx];
        }
    }

    // Prevent optimizer from nuking everything
    std::hint::black_box(sink);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::arch::x86_64::_pext_u64;
    use std::hint::black_box;
    use std::time::Instant;

    // Helper to run a benchmark loop and return average ns per op
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

    #[test]
    fn test_1_attack_lookup_speed() {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║               ATTACK TABLE LOOKUP SPEED                    ║");
        println!("╚════════════════════════════════════════════════════════════╝");

        warmup_attack_tables();

        // 1. Sliding Pieces 
        // We simulate real lookups: Fetch Mask -> PEXT(blockers, mask) -> Table[index]
        let iterations = 10_000_000;

        // Setup some dummy blockers to prevent constant folding
        let dummy_blockers = 0x00FF_00FF_00FF_00FFu64;

        bench_op("Rook (PEXT)", iterations, || {
            let sq = black_box(36); // e5
            let mask = ROOK_MASKS[sq];
            let idx = unsafe { _pext_u64(dummy_blockers, mask) };
            let _ = black_box(ROOK_ATTACKS[sq][idx as usize]);
        });

        bench_op("Bishop (PEXT)", iterations, || {
            let sq = black_box(36); // e5
            let mask = BISHOP_MASKS[sq];
            let idx = unsafe { _pext_u64(dummy_blockers, mask) };
            let _ = black_box(BISHOP_ATTACKS[sq][idx as usize]);
        });

        // 2. Leapers (Direct Array Access)
        bench_op("Knight", iterations, || {
            let sq = black_box(36);
            let _ = black_box(KNIGHT_ATTACKS[sq]);
        });

        bench_op("King", iterations, || {
            let sq = black_box(36);
            let _ = black_box(KING_ATTACKS[sq]);
        });

        bench_op("Pawn (White)", iterations, || {
            let sq = black_box(36);
            let _ = black_box(PAWN_ATTACKS[0][sq]);
        });
    }

    #[test]
    fn test_2_memory_footprint() {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║               ATTACK TABLE MEMORY USAGE                    ║");
        println!("╚════════════════════════════════════════════════════════════╝");

        // Force initialization of lazy statics
        let _ = BISHOP_ATTACKS.len();
        let _ = ROOK_ATTACKS.len();

        let mut total_bytes = 0;

        let calc_table_size = |table: &Vec<Vec<u64>>| -> usize {
            let top_level = table.capacity() * std::mem::size_of::<Vec<u64>>();
            let data_level: usize = table.iter().map(|v| v.capacity() * 8).sum();
            top_level + data_level
        };

        // 1. Sliding Pieces
        let rook_sz = calc_table_size(&ROOK_ATTACKS);
        let bishop_sz = calc_table_size(&BISHOP_ATTACKS);

        // 2. Fixed Tables (Compile time)
        let rook_mask_sz = std::mem::size_of_val(&ROOK_MASKS);
        let bishop_mask_sz = std::mem::size_of_val(&BISHOP_MASKS);
        let king_sz = std::mem::size_of_val(&KING_ATTACKS);
        let knight_sz = std::mem::size_of_val(&KNIGHT_ATTACKS);
        let pawn_sz = std::mem::size_of_val(&PAWN_ATTACKS);

        total_bytes +=
            rook_sz + bishop_sz + rook_mask_sz + bishop_mask_sz + king_sz + knight_sz + pawn_sz;

        // --- Output ---
        let to_kb = |b: usize| b as f64 / 1024.0;
        let to_mb = |b: usize| b as f64 / (1024.0 * 1024.0);

        println!(
            "{:<15} | {:>10.2} KB | {:>10.2} MB",
            "Rook Table",
            to_kb(rook_sz),
            to_mb(rook_sz)
        );
        println!(
            "{:<15} | {:>10.2} KB | {:>10.2} MB",
            "Bishop Table",
            to_kb(bishop_sz),
            to_mb(bishop_sz)
        );
        println!("{:<15} | {:>10.2} KB |", "Rook Masks", to_kb(rook_mask_sz));
        println!(
            "{:<15} | {:>10.2} KB |",
            "Bishop Masks",
            to_kb(bishop_mask_sz)
        );
        println!("{:<15} | {:>10.2} KB |", "King Table", to_kb(king_sz));
        println!("{:<15} | {:>10.2} KB |", "Knight Table", to_kb(knight_sz));
        println!("{:<15} | {:>10.2} KB |", "Pawn Table", to_kb(pawn_sz));

        println!("--------------------------------------------------");
        println!(
            "{:<15} | {:>10.2} KB | {:>10.2} MB",
            "TOTAL",
            to_kb(total_bytes),
            to_mb(total_bytes)
        );
    }
}
