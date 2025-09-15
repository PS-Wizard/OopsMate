//! Utilities For Chess Engine
//! Contains :
//! - Helpers to Print u64 as if it were a chess board
//! - Notation To index
//! - Notation To Blockers

/// Trait To Implement u64 printing like a chess board
pub trait BitBoardPrinter {
    fn print_board(&self);
}

impl BitBoardPrinter for u64 {
    /// Takes a u64 and prints it out like a chess board
    ///
    /// # Usage
    ///
    /// let some_bitboard = 0b101101;
    /// some_bitboard.print_board();
    fn print_board(&self) {
        for rank in (0..8).rev() {
            print!("{} ", rank + 1); // ranks 1..8
            for file in 0..8 {
                let square = rank * 8 + file;
                let bit = 1u64 << square;
                if self & bit != 0 {
                    print!("X ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
    }
}
// impl BitBoardPrinter for u64 {
//     fn print_board(&self) {
//
//         for rank in (0..8).rev() {
//             print!("{} ", rank + 1); // ranks 1..8
//             for file in 0..8 {
//                 let square = rank * 8 + file;
//                 let bit = 1u64 << square;
//                 if self & bit != 0 {
//                     print!(" 1 ");
//                 } else {
//                     print!(" . ");
//                 }
//             }
//             println!(" ");
//         }
//         println!("    a  b  c  d  e  f  g  h");
//     }
// }

// --- Printing Stuff ---
// Generates Variations for a bit patter, i.e
// for 1010:
// - 0000
// - 0010
// - 1000
// - 1010
pub fn enumerate_bit_variations(source: u64) -> Vec<u64> {
    let mut blockers = Vec::with_capacity(1 << source.count_ones());
    let mut n = 0u64;

    loop {
        blockers.push(n);
        n = (n.wrapping_sub(source)) & source; // `wrapping_sub` is safer than `n - source`
        if n == 0 {
            break;
        }
    }

    blockers
}

/// Trait To Implement Notation Operations On Strings
pub trait StrToNotation {
    fn to_blockers(&self) -> u64;
    fn to_idx(&self) -> u64;
}

impl StrToNotation for str {
    /// Conversts a comma seperated notation `&str` into a `u64` bitboard of blockers
    /// # Usage
    ///
    /// let blocker = "a2,a4,a6".to_blockers();
    /// blocker.print_board();
    fn to_blockers(&self) -> u64 {
        let mut board = 0u64;
        for sq in self.split(',') {
            let sq = sq.trim(); // Remove whitespace
            if sq.is_empty() {
                continue;
            }

            // Ensure the square notation is valid (e.g., "a2")
            let bytes = sq.as_bytes();
            if bytes.len() < 2 {
                continue; // Skip invalid squares
            }

            // Convert file (a-h) to index (0-7)
            let file_char = bytes[0] as char;
            if !('a'..='h').contains(&file_char.to_ascii_lowercase()) {
                continue; // Skip invalid files
            }
            let file = file_char.to_ascii_lowercase() as u8 - b'a';

            // Convert rank (1-8) to index (0-7)
            let rank_char = bytes[1] as char;
            if let Some(rank) = rank_char.to_digit(10) {
                if rank < 1 || rank > 8 {
                    continue; // Skip invalid ranks
                }
                let rank = rank - 1; // 1-based to 0-based
                let idx = rank * 8 + file as u32;
                board |= 1u64 << idx;
            }
        }
        board
    }

    fn to_idx(&self) -> u64 {
        let bytes = self.as_bytes();
        let file = (bytes[0] as char).to_ascii_lowercase() as u8 - b'a'; // 0..7
        let rank = (bytes[1] as char).to_digit(10).unwrap() - 1; // 0..7
        (rank * 8 + file as u32) as u64
    }
}

#[cfg(test)]
mod test_utils {
    use crate::utils::enumerate_bit_variations;

    #[test]
    fn test_enumeration_of_bits() {
        let ens = enumerate_bit_variations(0b1010);
        for ele in ens {
            println!("{:#b}", ele);
        }
    }
}
