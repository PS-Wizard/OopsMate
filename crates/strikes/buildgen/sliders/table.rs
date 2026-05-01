pub fn generate(masks: &[u64; 64], attack_fn: fn(usize, u64) -> u64) -> ([u32; 64], Vec<u64>) {
    let mut offsets = [0u32; 64];
    let mut attacks = Vec::new();

    for square in 0..64 {
        offsets[square] = attacks.len() as u32;
        let mask = masks[square];
        let mut blockers = 0u64;

        loop {
            attacks.push(attack_fn(square, blockers));
            blockers = blockers.wrapping_sub(mask) & mask;
            if blockers == 0 {
                break;
            }
        }
    }

    (offsets, attacks)
}
