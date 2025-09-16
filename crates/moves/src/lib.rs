use std::sync::LazyLock;

use crate::{
    bishops::{get_bishop_attacks, get_bishop_masks},
    rooks::{get_rook_attacks, get_rook_masks},
};

mod bishops;
mod bob;
mod rooks;

pub static BISHOP_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(get_bishop_masks).collect());
pub static BISHOP_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(get_bishop_attacks, get_bishop_masks));

pub static ROOK_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(get_rook_masks).collect());
pub static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(get_rook_attacks, get_rook_masks));

#[cfg(test)]
mod test {

    use super::*;
    use handies::bits::EnumerateVariations;
    use handies::{algebraic::Algebraic, board::PrintAsBoard};
    use std::arch::x86_64::_pext_u64;
    use std::time::Instant;

    #[test]
    fn test_attacks() {
        let sq = "e4".idx() as usize;

        // bishop attack
        let pext_b = unsafe { _pext_u64("g2,d5".place(), BISHOP_MASKS[sq]) };
        let bishop_attack = BISHOP_ATTACKS[sq][pext_b as usize];
        bishop_attack.print();
        "g2,d5".place().print();

        // rook attack
        let pext_r = unsafe { _pext_u64("e3,d4".place(), ROOK_MASKS[sq]) };
        let rook_attack = ROOK_ATTACKS[sq][pext_r as usize];
        rook_attack.print();
        "e3,d4".place().print();

        // queen attack = rook | bishop
        let queen_attack = rook_attack | bishop_attack;
        println!("queen:");
        queen_attack.print();
        "g2,d5,e3,d4".place().print();
    }

    #[test]
    fn benchmark_init() {
        let start = Instant::now();

        // Force initialization
        let _ = BISHOP_MASKS.len();
        let _ = BISHOP_ATTACKS.len();
        let _ = ROOK_MASKS.len();
        let _ = ROOK_ATTACKS.len();

        let duration = start.elapsed();
        println!("Attack tables initialized in: {:.3?}", duration);
    }

    #[test]
    fn benchmark_attack_lookups() {
        // Force initialization
        let _ = BISHOP_MASKS.len();
        let _ = BISHOP_ATTACKS.len();
        let _ = ROOK_MASKS.len();
        let _ = ROOK_ATTACKS.len();

        // Bishops
        let start = Instant::now();
        let mut bishop_count = 0u64;
        for square in 0u64..64 {
            let mask = BISHOP_MASKS[square as usize];
            let blocker_variants = mask.enumerate();
            for &blockers in &blocker_variants {
                let idx = unsafe { _pext_u64(blockers, mask) };
                let attack = BISHOP_ATTACKS[square as usize][idx as usize];
                bishop_count += attack.count_ones() as u64;
            }
        }
        let bishop_duration = start.elapsed();
        let bishop_avg = bishop_duration.as_nanos() as f64 / bishop_count as f64;
        println!(
            "Bishop lookup: {} variants, total {:.3?}, avg {:.2} ns/lookup",
            bishop_count, bishop_duration, bishop_avg
        );

        // Rooks
        let start = Instant::now();
        let mut rook_count = 0u64;
        for square in 0u64..64 {
            let mask = ROOK_MASKS[square as usize];
            let blocker_variants = mask.enumerate();
            for &blockers in &blocker_variants {
                let idx = unsafe { _pext_u64(blockers, mask) };
                let attack = ROOK_ATTACKS[square as usize][idx as usize];
                rook_count += attack.count_ones() as u64;
            }
        }
        let rook_duration = start.elapsed();
        let rook_avg = rook_duration.as_nanos() as f64 / rook_count as f64;
        println!(
            "Rook lookup: {} variants, total {:.3?}, avg {:.2} ns/lookup",
            rook_count, rook_duration, rook_avg
        );

        // Queens
        let start = Instant::now();
        let mut queen_count = 0u64;
        for square in 0u64..64 {
            let bmask = BISHOP_MASKS[square as usize];
            let rmask = ROOK_MASKS[square as usize];
            let b_variants = bmask.enumerate();
            let r_variants = rmask.enumerate();

            // Cartesian product of bishop/rook blocker sets
            for &bblockers in &b_variants {
                for &rblockers in &r_variants {
                    let bidx = unsafe { _pext_u64(bblockers, bmask) };
                    let ridx = unsafe { _pext_u64(rblockers, rmask) };
                    let battack = BISHOP_ATTACKS[square as usize][bidx as usize];
                    let rattack = ROOK_ATTACKS[square as usize][ridx as usize];
                    let qattack = battack | rattack;
                    queen_count += qattack.count_ones() as u64;
                }
            }
        }
        let queen_duration = start.elapsed();
        let queen_avg = queen_duration.as_nanos() as f64 / queen_count as f64;
        println!(
            "Queen lookup: {} combos, total {:.3?}, avg {:.2} ns/lookup",
            queen_count, queen_duration, queen_avg
        );
    }
}
