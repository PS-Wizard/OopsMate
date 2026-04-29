/// Side to move or piece color.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    /// Returns the opposite color.
    pub const fn flip(self) -> Color {
        unsafe { std::mem::transmute(self as u8 ^ 1) }
    }
}
