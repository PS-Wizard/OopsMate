pub const fn generate_pawn_attacks() -> [[u64; 64]; 2] {
    let mut attacks = [[0u64; 64]; 2];
    let mut square = 0;

    while square < 64 {
        let rank = square / 8;
        let file = square % 8;

        // White pawn attacks (up-left and up-right)
        if rank < 7 {
            if file > 0 {
                attacks[0][square] |= 1u64 << (square + 7); // up-left
            }
            if file < 7 {
                attacks[0][square] |= 1u64 << (square + 9); // up-right
            }
        }

        // Black pawn attacks (down-left and down-right)
        if rank > 0 {
            if file > 0 {
                attacks[1][square] |= 1u64 << (square - 9); // down-left
            }
            if file < 7 {
                attacks[1][square] |= 1u64 << (square - 7); // down-right
            }
        }

        square += 1;
    }

    attacks
}
