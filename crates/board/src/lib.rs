mod board;
mod game;
mod piece;
mod utils;

#[cfg(test)]
mod tests {
    use std::{arch::x86_64::_pext_u64, time::Instant};

    use magics::{
        BISHOP_ATTACKS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MASKS, utils::enumerate_bit_variations,
    };

    #[test]
    fn board_state_test() {
        let blockers: u64 = 0b0010_1010_0000; // Example blocker bitboard
        let mask: u64 = 0b0011_1110_0000; // Mask representing relevant squares
        let index = unsafe { _pext_u64(blockers, mask) };
        println!("{:#b}", index);
    }

    #[test]
    fn benchmark_attack_init() {
        use std::time::Instant;

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

        let start = Instant::now();
        let mut bishop_count = 0u64;
        for square in 0u64..64 {
            let mask = BISHOP_MASKS[square as usize];
            let blocker_variants = enumerate_bit_variations(mask);
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
            let blocker_variants = enumerate_bit_variations(mask);
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
    }
}
