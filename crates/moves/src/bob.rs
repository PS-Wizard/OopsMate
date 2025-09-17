use handies::bits::EnumerateVariations;

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
        let blocker_variants = mask.enumerate();

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
    use crate::{
        bishops::{get_bishop_attacks, get_bishop_masks},
        bob::generate_attack_table,
        rooks::{get_rook_attacks, get_rook_masks},
    };

    #[test]
    fn test_size_rook() {
        let rook_attacks = generate_attack_table(get_rook_attacks, get_rook_masks);
        let rook_masks: Vec<u64> = (0..64).map(|sq| get_rook_masks(sq)).collect();

        // Size of attack table
        let mut total_attack_bytes = 0;
        for attacks in &rook_attacks {
            total_attack_bytes += std::mem::size_of_val(attacks.as_slice());
        }

        // Size of masks
        let total_mask_bytes = std::mem::size_of_val(rook_masks.as_slice());

        println!(
            "Rook attacks: {} bytes (~{:.2} KB, {:.2} MB)",
            total_attack_bytes,
            total_attack_bytes as f64 / 1024.0,
            total_attack_bytes as f64 / (1024.0 * 1024.0)
        );

        println!(
            "Rook masks: {} bytes (~{:.2} KB, {:.2} MB)",
            total_mask_bytes,
            total_mask_bytes as f64 / 1024.0,
            total_mask_bytes as f64 / (1024.0 * 1024.0)
        );
    }

    #[test]
    fn test_size_bishop() {
        let bishop_attacks = generate_attack_table(get_bishop_attacks, get_bishop_masks);
        let bishop_masks: Vec<u64> = (0..64).map(|sq| get_bishop_masks(sq)).collect();

        // Size of attack table
        let mut total_attack_bytes = 0;
        for attacks in &bishop_attacks {
            total_attack_bytes += std::mem::size_of_val(attacks.as_slice());
        }

        // Size of masks
        let total_mask_bytes = std::mem::size_of_val(bishop_masks.as_slice());

        println!(
            "Bishop attacks: {} bytes (~{:.2} KB, {:.2} MB)",
            total_attack_bytes,
            total_attack_bytes as f64 / 1024.0,
            total_attack_bytes as f64 / (1024.0 * 1024.0)
        );

        println!(
            "Bishop masks: {} bytes (~{:.2} KB, {:.2} MB)",
            total_mask_bytes,
            total_mask_bytes as f64 / 1024.0,
            total_mask_bytes as f64 / (1024.0 * 1024.0)
        );
    }
}
