pub fn generate_knight_moves() -> [u64; 64] {
    let mut moves = [0u64; 64];

    for square in 0..64 {
        let from = 1u64 << square;
        let mut mask = 0u64;

        // Check file and rank to prevent wrapping
        let file = square % 8;
        let rank = square / 8;

        // Knight moves: two squares in one direction, one in the perpendicular
        // Up 2, Left 1
        if rank <= 5 && file >= 1 {
            mask |= from << 15;
        }
        // Up 2, Right 1
        if rank <= 5 && file <= 6 {
            mask |= from << 17;
        }
        // Right 2, Up 1
        if file <= 5 && rank <= 6 {
            mask |= from << 10;
        }
        // Right 2, Down 1
        if file <= 5 && rank >= 1 {
            mask |= from >> 6;
        }
        // Down 2, Right 1
        if rank >= 2 && file <= 6 {
            mask |= from >> 15;
        }
        // Down 2, Left 1
        if rank >= 2 && file >= 1 {
            mask |= from >> 17;
        }
        // Left 2, Down 1
        if file >= 2 && rank >= 1 {
            mask |= from >> 10;
        }
        // Left 2, Up 1
        if file >= 2 && rank <= 6 {
            mask |= from << 6;
        }

        moves[square as usize] = mask;
    }

    moves
}

#[cfg(test)]
mod test_knight {
    use handies::board::PrintAsBoard;

    use crate::knights::generate_knight_moves;

    #[test]
    fn test_knight_attacks() {
        let moves = generate_knight_moves();
        for ele in moves {
            ele.print();
        }
    }

    #[test]
    fn test_size_knight() {
        let moves = generate_knight_moves();
        let total_bytes = moves.len() * std::mem::size_of::<u64>();
        println!(
            "Knight table size: {} bytes (~{:.2} KB, {:.2} MB)",
            total_bytes,
            total_bytes as f64 / 1024.0,
            total_bytes as f64 / (1024.0 * 1024.0)
        );
    }
}
