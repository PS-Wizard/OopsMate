use crate::accumulator::Accumulator;
use crate::architecture::{BIG_HALF_DIMS, SMALL_HALF_DIMS};

#[derive(Clone, Debug)]
pub(crate) struct DirtyPiece {
    pub dirty_num: usize,
    pub from: [usize; 3],
    pub to: [usize; 3],
    pub piece_from: [usize; 3],
    pub piece_to: [usize; 3],
}

impl DirtyPiece {
    pub fn new() -> Self {
        Self {
            dirty_num: 0,
            from: [0; 3],
            to: [0; 3],
            piece_from: [0; 3],
            piece_to: [0; 3],
        }
    }

    pub fn reset(&mut self) {
        self.dirty_num = 0;
    }

    pub fn add_change(&mut self, from_sq: usize, to_sq: usize, piece_from: usize, piece_to: usize) {
        if self.dirty_num < 3 {
            self.from[self.dirty_num] = from_sq;
            self.to[self.dirty_num] = to_sq;
            self.piece_from[self.dirty_num] = piece_from;
            self.piece_to[self.dirty_num] = piece_to;
            self.dirty_num += 1;
        }
    }
}

impl Default for DirtyPiece {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub(crate) struct AccumulatorState {
    pub acc_big: Accumulator<BIG_HALF_DIMS>,
    pub acc_small: Accumulator<SMALL_HALF_DIMS>,
    pub dirty_piece: DirtyPiece,
    pub computed: [bool; 2],
    pub rule50: i32,
}

impl Default for AccumulatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl AccumulatorState {
    pub fn new() -> Self {
        Self {
            acc_big: Accumulator::new(),
            acc_small: Accumulator::new(),
            dirty_piece: DirtyPiece::new(),
            computed: [false, false],
            rule50: 0,
        }
    }

    pub fn reset(&mut self, dp: &DirtyPiece, rule50: i32) {
        self.dirty_piece = dp.clone();
        self.computed = [false, false];
        self.rule50 = rule50;
    }

    pub fn clear_root(&mut self, rule50: i32) {
        self.dirty_piece.reset();
        self.computed = [false, false];
        self.rule50 = rule50;
        self.acc_big.computed = [false, false];
        self.acc_small.computed = [false, false];
    }
}
