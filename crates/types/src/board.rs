#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Board(pub u64);

impl Board {
    #[inline(always)]
    pub const fn empty() -> Self {
        Board(0)
    }

    #[inline(always)]
    pub fn is_set(self, sq: usize) -> bool {
        (self.0 >> sq) & 1 != 0
    }

    #[inline(always)]
    pub fn set(&mut self, sq: usize) {
        self.0 |= 1 << sq;
    }

    #[inline(always)]
    pub fn clear(&mut self, sq: usize) {
        self.0 &= !(1 << sq);
    }
}
