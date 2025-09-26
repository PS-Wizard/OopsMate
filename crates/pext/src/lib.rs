use crate::{
    attacks::{
        bishops::{generate_bishop_attacks, generate_bishop_masks},
        kings::generate_king_attacks,
        knights::generate_knight_attacks,
        pawns::generate_pawn_attacks,
        rooks::{generate_rook_attacks, generate_rook_masks},
    },
    table_builder::generate_attack_table,
};
use std::sync::LazyLock;

pub mod api;
mod attacks;
mod enumerate;
mod table_builder;

pub static PAWN_ATTACKS: [[u64; 64]; 2] = generate_pawn_attacks();
pub static KING_ATTACKS: [u64; 64] = generate_king_attacks();
pub static KNIGHT_ATTACKS: [u64; 64] = generate_knight_attacks();
pub static ROOK_MASKS: [u64; 64] = generate_rook_masks();
pub static BISHOP_MASKS: [u64; 64] = generate_bishop_masks();
pub static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| generate_attack_table(generate_rook_attacks, &ROOK_MASKS));
pub static BISHOP_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| generate_attack_table(generate_bishop_attacks, &BISHOP_MASKS));

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
mod test {
    use crate::enumerate::EnumerateVariations;

    use super::*;
    use std::arch::x86_64::_pext_u64;
    use std::time::Instant;
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    #[test]
    fn test_attacks() {
        let sq = "a1".idx() as usize;

        // rook attack
        let pext_r = unsafe { _pext_u64("a2".place(), ROOK_MASKS[sq]) };
        let rook_attack = ROOK_ATTACKS[sq][pext_r as usize];
        rook_attack.print();

        // bishop attack
        let pext_b = unsafe { _pext_u64("b2,c3".place(), BISHOP_MASKS[sq]) };
        let bishop_attack = BISHOP_ATTACKS[sq][pext_b as usize];
        bishop_attack.print();

        // king attack
        let king_attack = KING_ATTACKS[sq];
        king_attack.print();

        // knight attack
        let knight_attack = KNIGHT_ATTACKS[sq];
        knight_attack.print();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn benchmark_init() {
        let start = Instant::now();
        warmup_attack_tables();
        let duration = start.elapsed();
        println!("Attack tables initialized & warmed up in: {:.3?}", duration);
    }

    #[test]
    fn benchmark_attacks() {
        warmup_attack_tables();

        // bishops
        let start = Instant::now();
        let mut bishop_ops = 0usize;
        let mut bishop_sink = 0u64;
        for square in 0..64 {
            let mask = BISHOP_MASKS[square];
            let blocker_variants = mask.enumerate();
            bishop_ops += blocker_variants.len();
            for &blockers in &blocker_variants {
                let idx = unsafe { _pext_u64(blockers, mask) };
                bishop_sink ^= BISHOP_ATTACKS[square][idx as usize];
            }
        }
        let bishop_duration = start.elapsed();
        println!(
            "Bishop lookup: {} lookups, total {:.3?}, avg {:.2} ns/lookup",
            bishop_ops,
            bishop_duration,
            bishop_duration.as_nanos() as f64 / bishop_ops as f64
        );

        // rooks
        let start = Instant::now();
        let mut rook_ops = 0usize;
        let mut rook_sink = 0u64;
        for square in 0..64 {
            let mask = ROOK_MASKS[square];
            let blocker_variants = mask.enumerate();
            rook_ops += blocker_variants.len();
            for &blockers in &blocker_variants {
                let idx = unsafe { _pext_u64(blockers, mask) };
                rook_sink ^= ROOK_ATTACKS[square][idx as usize];
            }
        }
        let rook_duration = start.elapsed();
        println!(
            "Rook lookup: {} lookups, total {:.3?}, avg {:.2} ns/lookup",
            rook_ops,
            rook_duration,
            rook_duration.as_nanos() as f64 / rook_ops as f64
        );

        // kings
        let start = Instant::now();
        let mut king_sink = 0u64;
        for square in 0..64 {
            king_sink ^= KING_ATTACKS[square];
        }
        let king_duration = start.elapsed();
        println!(
            "King lookup: 64 lookups, total {:.3?}, avg {:.2} ns/lookup",
            king_duration,
            king_duration.as_nanos() as f64 / 64.0
        );

        // knights
        let start = Instant::now();
        let mut knight_sink = 0u64;
        for square in 0..64 {
            knight_sink ^= KNIGHT_ATTACKS[square];
        }
        let knight_duration = start.elapsed();
        println!(
            "Knight lookup: 64 lookups, total {:.3?}, avg {:.2} ns/lookup",
            knight_duration,
            knight_duration.as_nanos() as f64 / 64.0
        );

        // queens (cartesian product)
        let start = Instant::now();
        let mut queen_ops = 0usize;
        let mut queen_sink = 0u64;
        for square in 0..64 {
            let bmask = BISHOP_MASKS[square];
            let rmask = ROOK_MASKS[square];
            let b_variants = bmask.enumerate();
            let r_variants = rmask.enumerate();

            for &bblockers in &b_variants {
                let bidx = unsafe { _pext_u64(bblockers, bmask) };
                for &rblockers in &r_variants {
                    let ridx = unsafe { _pext_u64(rblockers, rmask) };
                    let battack = BISHOP_ATTACKS[square][bidx as usize];
                    let rattack = ROOK_ATTACKS[square][ridx as usize];
                    queen_sink ^= battack | rattack;
                    queen_ops += 1;
                }
            }
        }
        let queen_duration = start.elapsed();
        println!(
            "Queen lookup: {} lookups, total {:.3?}, avg {:.2} ns/lookup",
            queen_ops,
            queen_duration,
            queen_duration.as_nanos() as f64 / queen_ops as f64
        );

        // prevent optimizer from nuking everything
        std::hint::black_box((bishop_sink, rook_sink, king_sink, knight_sink, queen_sink));
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_table_sizes() {
        // Force initialization
        let _ = &*ROOK_ATTACKS;
        let _ = &*BISHOP_ATTACKS;

        // Calculate memory usage
        let mut total_rook_bytes = 0;
        for attacks in ROOK_ATTACKS.iter() {
            total_rook_bytes += attacks.len() * std::mem::size_of::<u64>();
        }

        let mut total_bishop_bytes = 0;
        for attacks in BISHOP_ATTACKS.iter() {
            total_bishop_bytes += attacks.len() * std::mem::size_of::<u64>();
        }

        let rook_mask_bytes = std::mem::size_of_val(&ROOK_MASKS);
        let bishop_mask_bytes = std::mem::size_of_val(&BISHOP_MASKS);
        let king_bytes = std::mem::size_of_val(&KING_ATTACKS);
        let knight_bytes = std::mem::size_of_val(&KNIGHT_ATTACKS);

        println!("=== ATTACK TABLE MEMORY USAGE ===");
        println!(
            "Rook attacks: {} bytes ({:.2} MB)",
            total_rook_bytes,
            total_rook_bytes as f64 / (1024.0 * 1024.0)
        );
        println!(
            "Bishop attacks: {} bytes ({:.2} MB)",
            total_bishop_bytes,
            total_bishop_bytes as f64 / (1024.0 * 1024.0)
        );
        println!(
            "Rook masks: {} bytes ({:.2} KB)",
            rook_mask_bytes,
            rook_mask_bytes as f64 / 1024.0
        );
        println!(
            "Bishop masks: {} bytes ({:.2} KB)",
            bishop_mask_bytes,
            bishop_mask_bytes as f64 / 1024.0
        );
        println!(
            "King attacks: {} bytes ({:.2} KB)",
            king_bytes,
            king_bytes as f64 / 1024.0
        );
        println!(
            "Knight attacks: {} bytes ({:.2} KB)",
            knight_bytes,
            knight_bytes as f64 / 1024.0
        );

        let total_bytes = total_rook_bytes
            + total_bishop_bytes
            + rook_mask_bytes
            + bishop_mask_bytes
            + king_bytes
            + knight_bytes;
        println!(
            "Total: {} bytes ({:.2} MB)",
            total_bytes,
            total_bytes as f64 / (1024.0 * 1024.0)
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_compile_time_vs_runtime() {
        println!("=== COMPILE-TIME vs RUNTIME ===");
        println!("Compile-time generated:");
        println!(
            "  - KING_ATTACKS: {} bytes",
            std::mem::size_of_val(&KING_ATTACKS)
        );
        println!(
            "  - KNIGHT_ATTACKS: {} bytes",
            std::mem::size_of_val(&KNIGHT_ATTACKS)
        );
        println!(
            "  - ROOK_MASKS: {} bytes",
            std::mem::size_of_val(&ROOK_MASKS)
        );
        println!(
            "  - BISHOP_MASKS: {} bytes",
            std::mem::size_of_val(&BISHOP_MASKS)
        );

        println!("Runtime generated (LazyLock):");
        println!("  - ROOK_ATTACKS: dynamic size");
        println!("  - BISHOP_ATTACKS: dynamic size");
    }
}
