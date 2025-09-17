use std::sync::LazyLock;

use crate::{
    bishops::{get_bishop_attacks, get_bishop_masks},
    king::generate_king_moves,
    knights::generate_knight_moves,
    rooks::{get_rook_attacks, get_rook_masks},
};

mod bishops;
mod bob;
mod king;
mod knights;
mod rooks;

pub static BISHOP_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(get_bishop_masks).collect());
pub static BISHOP_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(get_bishop_attacks, get_bishop_masks));

pub static ROOK_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(get_rook_masks).collect());
pub static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(get_rook_attacks, get_rook_masks));
pub static KING_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| generate_king_moves());
pub static KNIGHT_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| generate_knight_moves());

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
        let _ = KING_ATTACKS.len();
        let _ = KNIGHT_ATTACKS.len();

        let duration = start.elapsed();
        println!("Attack tables initialized in: {:.3?}", duration);
    }

    #[test]
    fn benchmark_attack_lookups() {
        // force initialization
        let _ = BISHOP_MASKS.len();
        let _ = BISHOP_ATTACKS.len();
        let _ = ROOK_MASKS.len();
        let _ = ROOK_ATTACKS.len();
        let _ = KING_ATTACKS.len();
        let _ = KNIGHT_ATTACKS.len();

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
}
