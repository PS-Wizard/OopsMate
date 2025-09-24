#![allow(dead_code)]
/// Trait intended to provide some niceties for `&str` types to help when working with some of the
/// algebric notation stuff
pub trait Algebraic {
    /// Takes in a `str` of comma seperated squares in algebric notation and returns a u64 bitboard
    /// with bits set in those squares. Skips Invalid Squares.
    /// # Example
    /// let blockers = "g2,g4,g5".place();
    /// Outputs:
    /// 8 . . . . . . . .
    /// 7 . . . . . . . .
    /// 6 . . . . . . . .
    /// 5 . . . . . . X .
    /// 4 . . . . . . X .
    /// 3 . . . . . . . .
    /// 2 . . . . . . X .
    /// 1 . . . . . . . .
    /// + a b c d e f g h
    fn place(&self) -> u64 {
        unimplemented!("place() only for str");
    }
    /// Takes in a `str` for a square in algebric notation and returns the corresponding index in
    /// a 0-63 indexed bitboard
    /// # Example
    ///
    /// let idx = "g2".idx();
    /// assert_eq(idx,14);
    fn idx(&self) -> usize {
        unimplemented!("idx() only for str");
    }

    fn notation(&self) -> String {
        unimplemented!("notation() is only for u64")
    }
}

impl Algebraic for str {
    fn place(&self) -> u64 {
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

    fn idx(&self) -> usize {
        let file = (self.as_bytes()[0].to_ascii_lowercase() - b'a') as u64; // 0..7
        let rank = (self.as_bytes()[1] - b'1') as u64; // '1'..'8' → 0..7
        (rank * 8 + file) as usize
    }
}

impl Algebraic for u64 {
    fn notation(&self) -> String {
        let mut result = String::new();
        let mut bb = *self;

        while bb != 0 {
            let sq = bb.trailing_zeros() as u64;
            bb &= bb - 1;

            let file = (sq % 8) as u8;
            let rank = (sq / 8) as u8;

            if !result.is_empty() {
                result.push(',');
            }
            result.push((b'a' + file) as char);
            result.push((b'1' + rank) as char);
        }

        result
    }
}
