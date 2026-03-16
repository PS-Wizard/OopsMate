use crate::feature_transformer::FeatureTransformer;
use crate::finny_tables::{update_accumulator_refresh_cache, FinnyTables};

use super::stack::AccumulatorStack;
use super::state::AccumulatorState;

impl AccumulatorStack {
    pub fn update_incremental<F>(
        &mut self,
        king_squares: [usize; 2],
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
        caches: &mut FinnyTables,
        bitboards_provider: F,
    ) where
        F: FnOnce() -> ([u64; 2], [u64; 6]),
    {
        let mut bitboards: Option<([u64; 2], [u64; 6])> = None;
        let mut provider = Some(bitboards_provider);

        self.evaluate_side::<0, F>(
            king_squares[0],
            ft_big,
            ft_small,
            caches,
            &mut bitboards,
            &mut provider,
        );
        self.evaluate_side::<1, F>(
            king_squares[1],
            ft_big,
            ft_small,
            caches,
            &mut bitboards,
            &mut provider,
        );
    }

    fn evaluate_side<const P: usize, F>(
        &mut self,
        ksq: usize,
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
        caches: &mut FinnyTables,
        bitboards: &mut Option<([u64; 2], [u64; 6])>,
        provider: &mut Option<F>,
    ) where
        F: FnOnce() -> ([u64; 2], [u64; 6]),
    {
        let last_usable = self.find_last_usable_accumulator(P);

        if self.stack[last_usable].computed[P] {
            self.forward_update_incremental::<P>(last_usable, ksq, ft_big, ft_small);
        } else {
            let (current_color_bb, current_type_bb) = Self::get_bitboards(bitboards, provider);
            let current = &mut self.stack[self.current_idx - 1];

            update_accumulator_refresh_cache(
                ft_big,
                &mut current.acc_big,
                &mut caches.cache_big,
                P,
                ksq,
                current_color_bb,
                current_type_bb,
            );

            update_accumulator_refresh_cache(
                ft_small,
                &mut current.acc_small,
                &mut caches.cache_small,
                P,
                ksq,
                current_color_bb,
                current_type_bb,
            );

            current.computed[P] = true;
            self.backward_update_incremental::<P>(last_usable, ksq, ft_big, ft_small);
        }
    }

    fn get_bitboards<'a, F>(
        bitboards: &'a mut Option<([u64; 2], [u64; 6])>,
        provider: &mut Option<F>,
    ) -> (&'a [u64; 2], &'a [u64; 6])
    where
        F: FnOnce() -> ([u64; 2], [u64; 6]),
    {
        if bitboards.is_none() {
            let build = provider
                .take()
                .expect("bitboards provider should be used at most once");
            *bitboards = Some(build());
        }
        let (ref color, ref typ) = bitboards.as_ref().expect("bitboards should be available");
        (color, typ)
    }

    fn find_last_usable_accumulator(&self, perspective: usize) -> usize {
        for idx in (1..self.current_idx).rev() {
            if self.stack[idx].computed[perspective] {
                return idx;
            }
            if self.requires_refresh(idx, perspective) {
                return idx;
            }
        }
        0
    }

    fn requires_refresh(&self, idx: usize, perspective: usize) -> bool {
        let dp = &self.stack[idx].dirty_piece;
        for i in 0..dp.dirty_num {
            let pc = dp.piece_to[i];
            if perspective == 0 && pc == 6 {
                return true;
            }
            if perspective == 1 && pc == 14 {
                return true;
            }
        }
        false
    }

    fn forward_update_incremental<const P: usize>(
        &mut self,
        begin: usize,
        ksq: usize,
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
    ) {
        for i in (begin + 1)..self.current_idx {
            let (left, right) = self.stack.split_at_mut(i);
            let prev = &left[i - 1];
            let curr = &mut right[0];

            Self::apply_update::<P, true>(prev, curr, ksq, ft_big, ft_small);
        }
    }

    fn backward_update_incremental<const P: usize>(
        &mut self,
        end: usize,
        ksq: usize,
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
    ) {
        for i in (end..self.current_idx - 1).rev() {
            let (left, right) = self.stack.split_at_mut(i + 1);
            let target = &mut left[i];
            let source = &right[0];

            Self::apply_update::<P, false>(source, target, ksq, ft_big, ft_small);
        }
    }

    fn apply_update<const P: usize, const FORWARD: bool>(
        source: &AccumulatorState,
        target: &mut AccumulatorState,
        ksq: usize,
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
    ) {
        let dp = if FORWARD {
            &target.dirty_piece
        } else {
            &source.dirty_piece
        };

        let mut added: [(usize, usize); 3] = [(0, 0); 3];
        let mut removed: [(usize, usize); 3] = [(0, 0); 3];
        let mut a_cnt = 0;
        let mut r_cnt = 0;

        for i in 0..dp.dirty_num {
            if dp.piece_from[i] != 0 {
                removed[r_cnt] = (dp.from[i], dp.piece_from[i]);
                r_cnt += 1;
            }
            if dp.piece_to[i] != 0 {
                added[a_cnt] = (dp.to[i], dp.piece_to[i]);
                a_cnt += 1;
            }
        }

        let (eff_added, eff_removed) = if FORWARD {
            (&added[..a_cnt], &removed[..r_cnt])
        } else {
            (&removed[..r_cnt], &added[..a_cnt])
        };

        target.acc_big.update_incremental_perspective::<P>(
            &source.acc_big,
            eff_added,
            eff_removed,
            ksq,
            ft_big,
        );

        target.acc_small.update_incremental_perspective::<P>(
            &source.acc_small,
            eff_added,
            eff_removed,
            ksq,
            ft_small,
        );

        target.computed[P] = true;
    }
}
