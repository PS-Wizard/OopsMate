#![allow(dead_code)]

/// A trait for printing a `u64` as a chessboard.
///
/// This trait is meant for bitboards, where each bit in the `u64`
/// represents a square on the chessboard. The least significant
/// bit corresponds to square `a1` and the most significant bit to `h8`.
///
/// # Example
///
/// let bb: u64 = 0x8100000000000081; // corners set
/// bb.print();
pub trait PrintAsBoard {
    /// Prints the u64 as a chessboard.
    ///
    /// Each set bit (`1`) is printed as `X`, each unset bit (`0`) as `.`.
    /// Ranks are printed from 8 down to 1, files from `a` to `h`
    fn print(&self) {}
}

impl PrintAsBoard for u64 {
    fn print(&self) {
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



