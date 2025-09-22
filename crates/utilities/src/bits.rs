#![allow(dead_code)]
/// A trait intended to be used on a integer (binary) to generate every possible variation of it.
pub trait EnumerateVariations {
    /// A carry rippler function that generates all variations for a given binary.
    /// # Example
    /// let som_b = 0b0101;
    /// some_b.enumerate()
    /// Outputs: 0000, 0001, 0100, 0101
    fn enumerate(&self) -> Vec<u64>;
}

impl EnumerateVariations for u64 {
    fn enumerate(&self) -> Vec<u64> {
        let mut blockers = Vec::with_capacity(1 << self.count_ones());
        let mut n = 0u64;
        loop {
            blockers.push(n);
            n = (n.wrapping_sub(*self)) & self;
            if n == 0 {
                break;
            }
        }

        blockers
    }
}
