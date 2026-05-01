#[inline(always)]
#[must_use]
pub const fn line_between(from: usize, to: usize) -> u64 {
    crate::BETWEEN[from][to]
}

#[inline(always)]
#[must_use]
pub const fn line_through(from: usize, to: usize) -> u64 {
    crate::THROUGH[from][to]
}
