pub fn generate() -> [[u64; 64]; 2] {
    let mut attacks = [[0u64; 64]; 2];

    for square in 0..64 {
        let rank = square / 8;
        let file = square % 8;

        let mut white = 0u64;
        if rank < 7 {
            if file > 0 {
                white |= 1u64 << (square + 7);
            }
            if file < 7 {
                white |= 1u64 << (square + 9);
            }
        }

        let mut black = 0u64;
        if rank > 0 {
            if file > 0 {
                black |= 1u64 << (square - 9);
            }
            if file < 7 {
                black |= 1u64 << (square - 7);
            }
        }

        attacks[0][square] = white;
        attacks[1][square] = black;
    }

    attacks
}
