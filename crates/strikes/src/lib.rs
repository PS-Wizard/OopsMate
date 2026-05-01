#[cfg(not(target_arch = "x86_64"))]
compile_error!("strikes currently supports x86_64 only");

#[cfg(not(target_feature = "bmi2"))]
compile_error!("strikes currently requires BMI2 support for PEXT-based slider lookups");

// The large attack tables are generated at build time so runtime stays tiny and
// pays no initialization cost.
include!(concat!(env!("OUT_DIR"), "/tables.rs"));

mod backend;
mod geometry;
mod leapers;
mod sliders;

pub use geometry::{line_between, line_through};
pub use leapers::{king_attacks, knight_attacks, pawn_attacks};
pub use sliders::{bishop_attacks, queen_attacks, rook_attacks};

pub fn warmup_attack_tables() {
    let mut sink = 0u64;

    for &value in &PAWN_ATTACKS[0] {
        sink ^= value;
    }
    for &value in &PAWN_ATTACKS[1] {
        sink ^= value;
    }
    for &value in &KNIGHT_ATTACKS {
        sink ^= value;
    }
    for &value in &KING_ATTACKS {
        sink ^= value;
    }
    for &value in &ROOK_MASKS {
        sink ^= value;
    }
    for &value in &BISHOP_MASKS {
        sink ^= value;
    }
    for &value in &ROOK_ATTACKS {
        sink ^= value;
    }
    for &value in &BISHOP_ATTACKS {
        sink ^= value;
    }
    for row in &BETWEEN {
        for &value in row {
            sink ^= value;
        }
    }
    for row in &THROUGH {
        for &value in row {
            sink ^= value;
        }
    }

    std::hint::black_box(sink);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rook_attack_slow(square: usize, blockers: u64) -> u64 {
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;
        let mut attacks = 0u64;

        for (dr, df) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut r = rank + dr;
            let mut f = file + df;
            while (0..8).contains(&r) && (0..8).contains(&f) {
                let target = (r * 8 + f) as usize;
                attacks |= 1u64 << target;
                if blockers & (1u64 << target) != 0 {
                    break;
                }
                r += dr;
                f += df;
            }
        }

        attacks
    }

    fn bishop_attack_slow(square: usize, blockers: u64) -> u64 {
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;
        let mut attacks = 0u64;

        for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let mut r = rank + dr;
            let mut f = file + df;
            while (0..8).contains(&r) && (0..8).contains(&f) {
                let target = (r * 8 + f) as usize;
                attacks |= 1u64 << target;
                if blockers & (1u64 << target) != 0 {
                    break;
                }
                r += dr;
                f += df;
            }
        }

        attacks
    }

    #[test]
    fn slider_tables_match_slow_reference() {
        for square in 0..64 {
            let rook_mask = ROOK_MASKS[square];
            let mut blockers = 0u64;
            loop {
                assert_eq!(
                    rook_attacks(square, blockers),
                    rook_attack_slow(square, blockers)
                );
                blockers = blockers.wrapping_sub(rook_mask) & rook_mask;
                if blockers == 0 {
                    break;
                }
            }

            let bishop_mask = BISHOP_MASKS[square];
            let mut blockers = 0u64;
            loop {
                assert_eq!(
                    bishop_attacks(square, blockers),
                    bishop_attack_slow(square, blockers)
                );
                blockers = blockers.wrapping_sub(bishop_mask) & bishop_mask;
                if blockers == 0 {
                    break;
                }
            }
        }
    }

    #[test]
    fn leaper_tables_match_known_targets() {
        assert_eq!(knight_attacks(28), 0x0000_2844_0044_2800);
        assert_eq!(king_attacks(28), 0x0000_0038_2838_0000);
        assert_eq!(pawn_attacks(WHITE, 28), 0x0000_0028_0000_0000);
        assert_eq!(pawn_attacks(BLACK, 28), 0x0000_0000_0028_0000);
    }

    #[test]
    fn geometry_tables_match_expected_lines() {
        assert_eq!(line_between(4, 60), 0x0010_1010_1010_1000);
        assert_eq!(line_between(2, 20), 0x0000_0000_0000_0800);
        assert_eq!(line_between(0, 10), 0);
        assert_eq!(line_through(4, 60), 0x1010_1010_1010_1010);
        assert_eq!(line_through(0, 7), 0x0000_0000_0000_00ff);
        assert_eq!(line_through(0, 9), 0x8040_2010_0804_0201);
    }

    #[test]
    fn queen_attacks_is_union_of_rook_and_bishop() {
        let occupied = 0x00ff_2400_1800_ff00;
        for square in 0..64 {
            assert_eq!(
                queen_attacks(square, occupied),
                rook_attacks(square, occupied) | bishop_attacks(square, occupied)
            );
        }
    }
}
