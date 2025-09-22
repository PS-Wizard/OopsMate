use utilities::bits::EnumerateVariations;

/// Takes in functions to generate the attacks and the mask returns the attack table, indexed via
/// PEXT, and the Masks array.
pub fn generate_attack_table<AG>(attack_generator: AG, masks: &[u64; 64]) -> Vec<Vec<u64>>
where
    AG: Fn(u64, u64) -> u64,
{
    use utilities::bits::EnumerateVariations; // Make sure this trait is in scope

    let mut moves: Vec<Vec<u64>> = Vec::with_capacity(64);

    for square in 0u64..64 {
        let mask = masks[square as usize];
        let blocker_variants = mask.enumerate();
        let attacks: Vec<u64> = blocker_variants
            .iter()
            .map(|&blocker| attack_generator(square, blocker))
            .collect();
        moves.push(attacks);
    }

    moves
}

