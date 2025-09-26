#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board(pub u64);

impl Board {
    /// New Empty Board
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Self Explainatory
    #[inline(always)]
    pub fn set_bit(&mut self, square: u8) {
        self.0 |= 1u64 << square;
    }

    /// Self Explainatory
    #[inline(always)]
    pub fn remove_bit(&mut self, square: usize) {
        self.0 &= !(1u64 << square);
    }

    /// Self Explainatory
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// I'm hoping this will make it cleaner later on when i gota do get_lsb & pop_count
    #[inline(always)]
    pub fn count_bits(&self) -> u32 {
        self.0.count_ones()
    }

    /// I'm hoping this will make it cleaner later on when i gota do get_lsb & pop_count
    #[inline(always)]
    pub fn lsb(&self) -> u32 {
        self.0.trailing_zeros()
    }

    /// I'm hoping this will make it cleaner later on when i gota do get_lsb & pop_count
    #[inline(always)]
    pub fn pop_lsb(&mut self) {
        self.0 &= self.0 - 1;
    }
}
