use crate::types::Piece;

pub const PIECE_LIST_CAPACITY: usize = 32;

#[derive(Clone)]
pub struct PieceList {
    len: usize,
    data: [(usize, usize); PIECE_LIST_CAPACITY],
}

impl PieceList {
    pub fn new() -> Self {
        Self {
            len: 0,
            data: [(0, 0); PIECE_LIST_CAPACITY],
        }
    }

    pub fn push(&mut self, sq: usize, piece_idx: usize) {
        debug_assert!(self.len < PIECE_LIST_CAPACITY);
        if self.len < PIECE_LIST_CAPACITY {
            self.data[self.len] = (sq, piece_idx);
            self.len += 1;
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[(usize, usize)] {
        &self.data[..self.len]
    }
}

pub(crate) fn collect_pieces_from(pieces: &[Piece; 64], list: &mut PieceList) {
    list.clear();
    for (sq, p) in pieces.iter().copied().enumerate() {
        if p != Piece::None {
            list.push(sq, p.index());
        }
    }
}
