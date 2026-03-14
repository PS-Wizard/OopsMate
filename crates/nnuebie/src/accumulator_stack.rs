use crate::accumulator::Accumulator;
use crate::feature_transformer::FeatureTransformer;
use crate::finny_tables::FinnyTables;

const MAX_PLY: usize = 128;

#[derive(Clone, Debug)]
pub struct DirtyPiece {
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
pub struct AccumulatorState {
    pub acc_big: Accumulator<3072>,
    pub acc_small: Accumulator<128>,
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

pub struct AccumulatorStack {
    stack: Vec<AccumulatorState>,
    current_idx: usize,
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

    pub fn push(&mut self, dirty_piece: &DirtyPiece, rule50: i32) {
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

        crate::finny_tables::update_accumulator_refresh_cache(
            ft_big,
            &mut root.acc_big,
            &mut caches.cache_big,
            0,
            king_squares[0],
            &current_color_bb,
            &current_type_bb,
        );
        crate::finny_tables::update_accumulator_refresh_cache(
            ft_big,
            &mut root.acc_big,
            &mut caches.cache_big,
            1,
            king_squares[1],
            &current_color_bb,
            &current_type_bb,
        );

        crate::finny_tables::update_accumulator_refresh_cache(
            ft_small,
            &mut root.acc_small,
            &mut caches.cache_small,
            0,
            king_squares[0],
            &current_color_bb,
            &current_type_bb,
        );
        crate::finny_tables::update_accumulator_refresh_cache(
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

    /// Update incrementally using the Finny Tables cache if needed
    /// This is the main update entry point, equivalent to Stockfish's evaluate
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
            // Need refresh from cache
            let (current_color_bb, current_type_bb) = Self::get_bitboards(bitboards, provider);
            let current = &mut self.stack[self.current_idx - 1];

            // Big network
            crate::finny_tables::update_accumulator_refresh_cache(
                ft_big,
                &mut current.acc_big,
                &mut caches.cache_big,
                P,
                ksq,
                current_color_bb,
                current_type_bb,
            );

            // Small network
            crate::finny_tables::update_accumulator_refresh_cache(
                ft_small,
                &mut current.acc_small,
                &mut caches.cache_small,
                P,
                ksq,
                current_color_bb,
                current_type_bb,
            );

            current.computed[P] = true;

            // Backward propagation to fill gaps
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
            // Check if king moved
            // In Stockfish: requires_refresh(dirtyPiece, Perspective)
            if self.requires_refresh(idx, perspective) {
                return idx;
            }
        }
        0 // Root
    }

    fn requires_refresh(&self, idx: usize, perspective: usize) -> bool {
        let dp = &self.stack[idx].dirty_piece;
        for i in 0..dp.dirty_num {
            // Check if king moved
            // White King = 6, Black King = 14
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
            // Borrow checker gymnastics
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
            // i is target, i+1 is source (computed)
            let (left, right) = self.stack.split_at_mut(i + 1);
            let target = &mut left[i];
            let source = &right[0];

            Self::apply_update::<P, false>(source, target, ksq, ft_big, ft_small);
        }
    }

    // Generic update helper
    // FORWARD=true: prev -> curr
    // FORWARD=false: curr -> prev (backward)
    fn apply_update<const P: usize, const FORWARD: bool>(
        source: &AccumulatorState,
        target: &mut AccumulatorState,
        ksq: usize,
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
    ) {
        // Determine dirty piece.
        // If Forward: use target's dirty piece (which represents move prev->curr)
        // If Backward: use source's dirty piece (which represents move target->source)
        let dp = if FORWARD {
            &target.dirty_piece
        } else {
            &source.dirty_piece
        };

        // Prepare lists
        let mut added: [(usize, usize); 3] = [(0, 0); 3];
        let mut removed: [(usize, usize); 3] = [(0, 0); 3];
        let mut a_cnt = 0;
        let mut r_cnt = 0;

        for i in 0..dp.dirty_num {
            if dp.piece_from[i] != 0 {
                // Removal
                // We actually need indices for the helper, but Accumulator::update_incremental
                // takes (Square, Piece) and calls make_index internally.
                removed[r_cnt] = (dp.from[i], dp.piece_from[i]);
                r_cnt += 1;
            }
            if dp.piece_to[i] != 0 {
                // Addition
                added[a_cnt] = (dp.to[i], dp.piece_to[i]);
                a_cnt += 1;
            }
        }

        // If Backward: Swap added/removed!
        // Because "undoing" an addition is a removal, and "undoing" a removal is an addition.
        let (eff_added, eff_removed) = if FORWARD {
            (&added[..a_cnt], &removed[..r_cnt])
        } else {
            (&removed[..r_cnt], &added[..a_cnt])
        };

        // Need to update both networks
        // Since we are inside generic <P>, we only update perspective P
        // But Accumulator::update_incremental updates BOTH perspectives if we pass the array.
        // Wait, Accumulator::update_incremental updates both 0 and 1.
        // But here we are inside evaluate_side<P>.
        // We only want to update accumulator[P].
        // Our Accumulator struct has [AlignedBuffer; 2].
        // update_incremental updates BOTH.

        // Stockfish's update_accumulator_incremental is templated on Perspective.
        // My Accumulator::update_incremental is not.
        // I should probably fix Accumulator::update_incremental to take a perspective or update both.
        // But here we only computed P for the King Square of P.
        // The other perspective might have a DIFFERENT King Square!
        // So we CANNOT use Accumulator::update_incremental as is, because it assumes same ksq array for both?
        // Actually it takes `ksq: [usize; 2]`.
        // But here we only know ksq for P. The ksq for 1-P might be different and might have moved!

        // So we MUST use a perspective-specific update.
        // I need to add `update_incremental_perspective` to Accumulator.

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
