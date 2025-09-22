use std::ops::{Index, IndexMut};

use crate::piece_kind::PieceKind;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Board(pub u64);

impl Board {
    #[inline(always)]
    pub fn empty() -> Self {
        Board(0)
    }

    #[inline(always)]
    pub fn set(&mut self, sq: usize) {
        self.0 |= 1 << sq;
    }

    #[inline(always)]
    pub fn remove(&mut self, sq: usize) {
        self.0 &= !(1 << sq);
    }
}

impl Index<PieceKind> for [Board; 12] {
    type Output = Board;

    fn index(&self, piece: PieceKind) -> &Self::Output {
        &self[piece as usize]
    }
}

impl IndexMut<PieceKind> for [Board; 12] {
    fn index_mut(&mut self, piece: PieceKind) -> &mut Self::Output {
        &mut self[piece as usize]
    }
}
