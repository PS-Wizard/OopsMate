#![allow(dead_code)]
const FILE_A: u64 = 0x0101010101010101;
const FILE_H: u64 = 0x8080808080808080;

pub const fn generate_pawn_attacks() -> [[u64; 64]; 2] {
    let mut attacks = [[0u64; 64]; 2];
    let mut square = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;

        // White pawn attacks (moving up the board)
        let mut white_attacks = 0u64;
        if rank < 7 {
            // Not on rank 8
            // Attack diagonally up-left
            if file > 0 {
                // Not on file A
                white_attacks |= 1u64 << (square + 7);
            }
            // Attack diagonally up-right
            if file < 7 {
                // Not on file H
                white_attacks |= 1u64 << (square + 9);
            }
        }
        attacks[0][square] = white_attacks;

        // Black pawn attacks (moving down the board)
        let mut black_attacks = 0u64;
        if rank > 0 {
            // Not on rank 1
            // Attack diagonally down-left (from black's POV)
            if file > 0 {
                // Not on file A
                black_attacks |= 1u64 << (square - 9);
            }
            // Attack diagonally down-right (from black's POV)
            if file < 7 {
                // Not on file H
                black_attacks |= 1u64 << (square - 7);
            }
        }
        attacks[1][square] = black_attacks;

        square += 1;
    }

    attacks
}

#[cfg(test)]
mod test_pawns {
    use utilities::board::PrintAsBoard;

    use crate::attacks::pawns::generate_pawn_attacks;

    #[test]
    fn test_pawn_attacks() {
        let pawn_attacks = generate_pawn_attacks();
        println!("For Black:");
        for sq in 0..64 {
            println!("For A Pawn On {sq}:");
            pawn_attacks[1][sq].print();
            println!("---");
        }
        println!("For White:");
        for sq in 0..64 {
            println!("For A Pawn On {sq}:");
            pawn_attacks[0][sq].print();
            println!("---");
        }
    }
}
