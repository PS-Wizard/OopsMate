use std::sync::LazyLock;

use crate::{
    bishops::{generate_bishop_attacks, generate_bishop_masks},
    king::generate_king_attacks,
    knights::generate_knight_attacks,
    rooks::{generate_rook_attacks, generate_rook_masks},
};

mod bishops;
mod bob;
mod king;
mod knights;
pub mod pawns;
mod rooks;

static BISHOP_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(generate_bishop_masks).collect());
static BISHOP_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(generate_bishop_attacks, generate_bishop_masks));
static ROOK_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(generate_rook_masks).collect());
static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(generate_rook_attacks, generate_rook_masks));
static KING_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| generate_king_attacks());
static KNIGHT_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| generate_knight_attacks());

pub fn warmup_attack_tables() {
    use std::arch::x86_64::_pext_u64;

    // Force LazyLock initialization for all tables
    let _ = BISHOP_MASKS.len();
    let _ = BISHOP_ATTACKS.len();
    let _ = ROOK_MASKS.len();
    let _ = ROOK_ATTACKS.len();
    let _ = KING_ATTACKS.len();
    let _ = KNIGHT_ATTACKS.len();

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

#[inline(always)]
pub fn get_king_attacks(from: usize) -> u64 {
    KING_ATTACKS[from]
}

#[inline(always)]
pub fn get_knight_attacks(from: usize) -> u64 {
    KNIGHT_ATTACKS[from]
}

#[inline(always)]
pub fn get_bishop_attacks(from: usize, enemies: u64) -> u64 {
    let mask = BISHOP_MASKS[from];
    let idx = unsafe { std::arch::x86_64::_pext_u64(enemies, mask) };
    BISHOP_ATTACKS[from][idx as usize]
}

#[inline(always)]
pub fn get_rook_attacks(from: usize, enemies: u64) -> u64 {
    let mask = ROOK_MASKS[from];
    let idx = unsafe { std::arch::x86_64::_pext_u64(enemies, mask) };
    ROOK_ATTACKS[from][idx as usize]
}

#[inline(always)]
pub fn get_queen_attacks(from: usize, enemies: u64) -> u64 {
    get_bishop_attacks(from, enemies) | get_rook_attacks(from, enemies)
}

#[cfg(test)]
mod test {

    use crate::pawns::get_pawn_attacks;

    use super::*;
    use handies::bits::EnumerateVariations;
    use handies::{algebraic::Algebraic, board::PrintAsBoard};
    use std::arch::x86_64::_pext_u64;
    use std::time::Instant;

    #[test]
    fn test_attacks() {
        let sq = "a1".idx() as usize;

        // rook attack
        let pext_r = unsafe { _pext_u64("a2".place(), ROOK_MASKS[sq]) };
        let rook_attack = ROOK_ATTACKS[sq][pext_r as usize];
        rook_attack.print();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn benchmark_init() {
        let start = Instant::now();
        warmup_attack_tables();
        let duration = start.elapsed();
        #[cfg(debug_assertions)]
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

        // pawns
        let white_pawns = 0x000000000000FF00_u64;
        let black_pawns = 0x00FF000000000000_u64;

        let mut pawn_sink = 0u64;
        let mut total_ops = 0usize;

        // Benchmark white pawns
        let start = Instant::now();
        for i in 0..1000 {
            // repeat to get measurable time
            let wp = white_pawns.rotate_left((i % 8) * 8); // rotate ranks for variety
            let bp = black_pawns.rotate_right((i % 8) * 8);
            let moves_white = get_pawn_attacks(wp, bp, 0, 0);
            let moves_black = get_pawn_attacks(bp, wp, 1, 0);
            pawn_sink ^= moves_white ^ moves_black;
            total_ops += 2;
        }
        let duration = start.elapsed();
        println!(
            "Pawn movegen varied: {} ops, total {:?}, avg {:.2} ns/op",
            total_ops,
            duration,
            duration.as_nanos() as f64 / total_ops as f64
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
        std::hint::black_box((
            bishop_sink,
            rook_sink,
            king_sink,
            knight_sink,
            queen_sink,
            pawn_sink,
        ));
    }
}
