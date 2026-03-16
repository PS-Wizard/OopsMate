use crate::feature_transformer::FeatureTransformer;
use crate::finny_tables::{update_accumulator_refresh_cache, FinnyTables};

use super::state::{AccumulatorState, DirtyPiece};

const MAX_PLY: usize = 128;

pub struct AccumulatorStack {
    pub(super) stack: Vec<AccumulatorState>,
    pub(super) current_idx: usize,
}

impl Default for AccumulatorStack {
    fn default() -> Self {
        Self::new()
    }
}

impl AccumulatorStack {
    pub fn new() -> Self {
        let mut stack = Vec::with_capacity(MAX_PLY + 1);
        stack.push(AccumulatorState::new());
        Self {
            stack,
            current_idx: 1,
        }
    }

    pub fn latest(&self) -> &AccumulatorState {
        &self.stack[self.current_idx - 1]
    }

    pub fn mut_latest(&mut self) -> &mut AccumulatorState {
        &mut self.stack[self.current_idx - 1]
    }

    pub fn push(&mut self, dirty_piece: DirtyPiece, rule50: i32) {
        if self.current_idx >= self.stack.len() {
            self.stack.push(AccumulatorState::new());
        }
        self.stack[self.current_idx].reset(dirty_piece, rule50);
        self.current_idx += 1;
    }

    pub fn pop(&mut self) {
        if self.current_idx > 1 {
            self.current_idx -= 1;
        }
    }

    pub fn reset_with_refresh(
        &mut self,
        king_squares: [usize; 2],
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
        caches: &mut FinnyTables,
        current_color_bb: [u64; 2],
        current_type_bb: [u64; 6],
        rule50: i32,
    ) {
        self.current_idx = 1;
        self.stack[0].clear_root(rule50);

        let root = &mut self.stack[0];

        update_accumulator_refresh_cache(
            ft_big,
            &mut root.acc_big,
            &mut caches.cache_big,
            0,
            king_squares[0],
            &current_color_bb,
            &current_type_bb,
        );
        update_accumulator_refresh_cache(
            ft_big,
            &mut root.acc_big,
            &mut caches.cache_big,
            1,
            king_squares[1],
            &current_color_bb,
            &current_type_bb,
        );

        update_accumulator_refresh_cache(
            ft_small,
            &mut root.acc_small,
            &mut caches.cache_small,
            0,
            king_squares[0],
            &current_color_bb,
            &current_type_bb,
        );
        update_accumulator_refresh_cache(
            ft_small,
            &mut root.acc_small,
            &mut caches.cache_small,
            1,
            king_squares[1],
            &current_color_bb,
            &current_type_bb,
        );

        root.computed = [true, true];
    }

    pub fn refresh(
        &mut self,
        pieces: &[(usize, usize)],
        king_squares: [usize; 2],
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
    ) {
        let current = self.mut_latest();
        current.acc_big.refresh(pieces, king_squares, ft_big);
        current.acc_small.refresh(pieces, king_squares, ft_small);
        current.computed = [true, true];
    }
}
