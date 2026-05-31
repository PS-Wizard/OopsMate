pub const fn generate_knight_attacks() -> [u64; 64] {
    let mut moves = [0u64; 64];
    let mut square = 0;

    while square < 64 {
        let mut mask = 0u64;
        let file = square % 8;
        let rank = square / 8;

        // All 8 knight moves as (rank_delta, file_delta)
        let knight_moves = [
            (-2, -1),
            (-2, 1), // Up 2, Left/Right 1
            (-1, -2),
            (-1, 2), // Up 1, Left/Right 2
            (1, -2),
            (1, 2), // Down 1, Left/Right 2
            (2, -1),
            (2, 1), // Down 2, Left/Right 1
        ];

        let mut i = 0;
        while i < 8 {
            let (dr, df) = knight_moves[i];
            let new_rank = rank as i32 + dr;
            let new_file = file as i32 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_square = (new_rank * 8 + new_file) as u64;
                mask |= 1u64 << target_square;
            }

            i += 1;
        }

        moves[square] = mask;
        square += 1;
    }

    moves
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod test_knight {
    use utilities::board::PrintAsBoard;

    use crate::attacks::knights::generate_knight_attacks;

    #[test]
    fn test_knight_attacks() {
        let moves = generate_knight_attacks();
        for ele in moves {
            ele.print();
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_size_knight() {
        let moves = generate_knight_attacks();
        let total_bytes = moves.len() * std::mem::size_of::<u64>();
        println!(
            "Knight table size: {} bytes (~{:.2} KB, {:.2} MB)",
            total_bytes,
            total_bytes as f64 / 1024.0,
            total_bytes as f64 / (1024.0 * 1024.0)
        );
    }
}
