/// Returns a array of attacks for a king from any given square.
pub const fn generate_king_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut square = 0;

    while square < 64 {
        let mut mask = 0u64;
        let file = square % 8;
        let rank = square / 8;

        // All 8 directions: N, NE, E, SE, S, SW, W, NW
        let directions = [
            (-1, 0),  // North
            (-1, 1),  // Northeast
            (0, 1),   // East
            (1, 1),   // Southeast
            (1, 0),   // South
            (1, -1),  // Southwest
            (0, -1),  // West
            (-1, -1), // Northwest
        ];

        let mut i = 0;
        while i < 8 {
            let (dr, df) = directions[i];
            let new_rank = rank as i32 + dr;
            let new_file = file as i32 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_square = (new_rank * 8 + new_file) as u64;
                mask |= 1u64 << target_square;
            }

            i += 1;
        }

        attacks[square] = mask;
        square += 1;
    }

    attacks
}

#[cfg(test)]
mod test_king {
    use utilities::board::PrintAsBoard;

    use crate::attacks::kings::generate_king_attacks;

    #[test]
    fn test_king_attacks() {
        let moves = generate_king_attacks();
        for ele in moves {
            ele.print();
        }
    }

    #[test]
    fn test_size_king() {
        let moves = generate_king_attacks();
        let total_bytes = moves.len() * std::mem::size_of::<u64>();
        println!(
            "King table size: {} bytes (~{:.2} KB, {:.2} MB)",
            total_bytes,
            total_bytes as f64 / 1024.0,
            total_bytes as f64 / (1024.0 * 1024.0)
        );
    }
}
