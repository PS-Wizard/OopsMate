use crate::utils::enumerate_bit_variations;

/// Takes in functions to generate the attacks and the mask returns the attack table, indexed via
/// PEXT, and the Masks array.
pub fn generate_attack_table<AG, MG>(attack_generator: AG, mask_generator: MG) -> Vec<Vec<u64>>
where
    AG: Fn(u64, u64) -> u64,
    MG: Fn(u64) -> u64,
{
    let mut moves: Vec<Vec<u64>> = Vec::with_capacity(64); // one entry per square

    for square in 0u64..64 {
        let mask = mask_generator(square);
        let blocker_variants = enumerate_bit_variations(mask);

        let attacks: Vec<u64> = blocker_variants
            .iter()
            .map(|&blocker| attack_generator(square, blocker))
            .collect();

        moves.push(attacks); // store attacks for this square
    }

    moves
}

#[cfg(test)]
mod test_bob {
    use std::arch::x86_64::_pext_u64;

    use crate::{
        bishops::{get_bishop_attacks, get_bishop_masks},
        bob::generate_attack_table,
        rooks::{get_rook_attacks, get_rook_masks},
        utils::{BitBoardPrinter, StrToNotation},
    };

    #[test]
    fn test_build_bishops() {
        let bishop_attacks = generate_attack_table(get_bishop_attacks, get_bishop_masks);

        let mut total_bytes = 0;
        for attacks in &bishop_attacks {
            total_bytes += size_of_val(attacks.as_slice())
        }

        println!(
            "Bishop attacks: {} bytes (~{:.2} KB, {:.2} MB)",
            total_bytes,
            total_bytes as f64 / 1024.0,
            total_bytes as f64 / (1024.0 * 1024.0)
        );

        let pext = unsafe { _pext_u64("g2,d5".to_blockers(), get_bishop_masks("e4".to_idx())) };
        let attack = bishop_attacks["e4".to_idx() as usize][pext as usize];
        attack.print_board();
    }
    #[test]
    fn test_build_rooks() {
        let rook_attacks = generate_attack_table(get_rook_attacks, get_rook_masks);

        let mut total_bytes = 0;
        for attacks in &rook_attacks {
            total_bytes += size_of_val(attacks.as_slice())
        }

        println!(
            "Rook attacks: {} bytes (~{:.2} KB, {:.2} MB)",
            total_bytes,
            total_bytes as f64 / 1024.0,
            total_bytes as f64 / (1024.0 * 1024.0)
        );

        let pext = unsafe { _pext_u64("c4,e2,e7".to_blockers(), get_rook_masks("e4".to_idx())) };
        let attack = rook_attacks["e4".to_idx() as usize][pext as usize];
        attack.print_board();
    }
}
