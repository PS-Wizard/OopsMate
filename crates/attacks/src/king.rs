/// Returns a array of attacks for a king from any given square.
pub fn generate_king_attacks() -> [u64; 64] {
    let mut moves = [0u64; 64];
    for square in 0..64 {
        let from = 1u64 << square;
        let mut mask = 0u64;

        // Check if not on A-file (left edge)
        if square % 8 != 0 {
            mask |= from >> 1; // Left
            if square < 56 {
                mask |= from << 7; // Up-left
            }
            if square >= 8 {
                mask |= from >> 9; // Down-left
            }
        }

        // Check if not on H-file (right edge)
        if square % 8 != 7 {
            mask |= from << 1; // Right
            if square < 56 {
                mask |= from << 9; // Up-right
            }
            if square >= 8 {
                mask |= from >> 7; // Down-right
            }
        }

        // Up
        if square < 56 {
            mask |= from << 8;
        }

        // Down
        if square >= 8 {
            mask |= from >> 8;
        }

        moves[square as usize] = mask;
    }
    moves
}

#[cfg(test)]
mod test_king {
    use handies::board::PrintAsBoard;

    use crate::king::generate_king_attacks;

    #[test]
    fn test_king_attacks() {
        let moves = generate_king_attacks();
        for ele in moves {
            ele.print();
        }
    }

    #[test]
    #[cfg(debug_assertions)]
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
