use super::{simd, Accumulator, FeatureUpdateFn, RefreshFn, UpdateSinglePassFn};
use crate::aligned::AlignedBuffer;
use crate::feature_transformer::{FeatureTransformer, PSQT_BUCKETS};
use crate::features::{self, make_index};

impl<const SIZE: usize> Accumulator<SIZE> {
    /// Creates an empty accumulator wired to the best available update kernels.
    pub fn new() -> Self {
        #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
        let (add_fn, remove_fn, update_fn, refresh_fn) = (
            simd::add_feature_avx2 as FeatureUpdateFn,
            simd::remove_feature_avx2 as FeatureUpdateFn,
            simd::update_accumulators_single_pass_avx2 as UpdateSinglePassFn,
            refresh_kernel::<SIZE>(),
        );

        #[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
        let (add_fn, remove_fn, update_fn, refresh_fn) = (
            simd::add_feature_scalar as FeatureUpdateFn,
            simd::remove_feature_scalar as FeatureUpdateFn,
            simd::update_accumulators_single_pass_scalar as UpdateSinglePassFn,
            None,
        );

        Self {
            accumulation: [AlignedBuffer::new(SIZE), AlignedBuffer::new(SIZE)],
            psqt_accumulation: [[0; PSQT_BUCKETS]; 2],
            computed: [false, false],
            add_feature_fn: add_fn,
            remove_feature_fn: remove_fn,
            update_single_pass_fn: update_fn,
            refresh_fn,
        }
    }

    /// Rebuilds both perspectives from the current piece list.
    pub fn refresh(&mut self, pieces: &[(usize, usize)], ksq: [usize; 2], ft: &FeatureTransformer) {
        debug_assert_eq!(
            ft.half_dims, SIZE,
            "FeatureTransformer dims mismatch Accumulator size"
        );

        if let Some(refresh_kernel) = self.refresh_fn {
            let mut indices_w = [0usize; 32];
            let mut indices_b = [0usize; 32];
            let mut count = 0;

            for (slot, &(sq, pc)) in pieces.iter().take(32).enumerate() {
                indices_w[slot] = make_index(features::WHITE, sq, pc, ksq[features::WHITE]);
                indices_b[slot] = make_index(features::BLACK, sq, pc, ksq[features::BLACK]);
                count += 1;
            }

            unsafe {
                refresh_kernel(
                    self.accumulation[features::WHITE].as_mut_slice(),
                    &ft.biases,
                    &ft.weights,
                    &indices_w[..count],
                );
                refresh_kernel(
                    self.accumulation[features::BLACK].as_mut_slice(),
                    &ft.biases,
                    &ft.weights,
                    &indices_b[..count],
                );
            }

            self.psqt_accumulation[features::WHITE].fill(0);
            self.psqt_accumulation[features::BLACK].fill(0);

            for &(sq, pc) in pieces {
                let white_idx = make_index(features::WHITE, sq, pc, ksq[features::WHITE]);
                self.update_psqt(features::WHITE, white_idx, ft, true);

                let black_idx = make_index(features::BLACK, sq, pc, ksq[features::BLACK]);
                self.update_psqt(features::BLACK, black_idx, ft, true);
            }

            self.computed = [true, true];
            return;
        }

        for perspective in 0..2 {
            self.accumulation[perspective].copy_from_slice(&ft.biases);
            self.psqt_accumulation[perspective].fill(0);
            self.computed[perspective] = true;
        }

        for &(sq, pc) in pieces {
            let white_idx = make_index(features::WHITE, sq, pc, ksq[features::WHITE]);
            self.add_feature(features::WHITE, white_idx, ft);

            let black_idx = make_index(features::BLACK, sq, pc, ksq[features::BLACK]);
            self.add_feature(features::BLACK, black_idx, ft);
        }
    }

    pub fn update_incremental_perspective<const P: usize>(
        &mut self,
        prev: &Accumulator<SIZE>,
        added: &[(usize, usize)],
        removed: &[(usize, usize)],
        ksq: usize,
        ft: &FeatureTransformer,
    ) {
        self.psqt_accumulation[P] = prev.psqt_accumulation[P];

        for &(sq, pc) in removed {
            self.update_psqt(P, make_index(P, sq, pc, ksq), ft, false);
        }
        for &(sq, pc) in added {
            self.update_psqt(P, make_index(P, sq, pc, ksq), ft, true);
        }

        let mut added_ptrs = [std::ptr::null(); 3];
        let mut removed_ptrs = [std::ptr::null(); 3];
        let added_count = added.len().min(3);
        let removed_count = removed.len().min(3);
        let weights_ptr = ft.weights.as_ptr();

        for (slot, &(sq, pc)) in added.iter().take(added_count).enumerate() {
            added_ptrs[slot] = unsafe { weights_ptr.add(make_index(P, sq, pc, ksq) * SIZE) };
        }
        for (slot, &(sq, pc)) in removed.iter().take(removed_count).enumerate() {
            removed_ptrs[slot] = unsafe { weights_ptr.add(make_index(P, sq, pc, ksq) * SIZE) };
        }

        unsafe {
            (self.update_single_pass_fn)(
                &prev.accumulation[P],
                &mut self.accumulation[P],
                &added_ptrs[..added_count],
                &removed_ptrs[..removed_count],
            );
        }

        self.computed[P] = true;
    }

    pub fn update_with_ksq(
        &mut self,
        added: &[(usize, usize)],
        removed: &[(usize, usize)],
        ksq: [usize; 2],
        ft: &FeatureTransformer,
    ) {
        debug_assert_eq!(
            ft.half_dims, SIZE,
            "FeatureTransformer dims mismatch Accumulator size"
        );

        for &(sq, pc) in removed {
            self.remove_feature(
                features::WHITE,
                make_index(features::WHITE, sq, pc, ksq[features::WHITE]),
                ft,
            );
            self.remove_feature(
                features::BLACK,
                make_index(features::BLACK, sq, pc, ksq[features::BLACK]),
                ft,
            );
        }

        for &(sq, pc) in added {
            self.add_feature(
                features::WHITE,
                make_index(features::WHITE, sq, pc, ksq[features::WHITE]),
                ft,
            );
            self.add_feature(
                features::BLACK,
                make_index(features::BLACK, sq, pc, ksq[features::BLACK]),
                ft,
            );
        }
    }

    pub fn add_feature(&mut self, perspective: usize, feature_idx: usize, ft: &FeatureTransformer) {
        unsafe {
            (self.add_feature_fn)(
                self.accumulation[perspective].as_mut_slice(),
                feature_weights::<SIZE>(ft, feature_idx),
            );
        }
        self.update_psqt(perspective, feature_idx, ft, true);
    }

    fn remove_feature(&mut self, perspective: usize, feature_idx: usize, ft: &FeatureTransformer) {
        unsafe {
            (self.remove_feature_fn)(
                self.accumulation[perspective].as_mut_slice(),
                feature_weights::<SIZE>(ft, feature_idx),
            );
        }
        self.update_psqt(perspective, feature_idx, ft, false);
    }

    #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
    fn update_psqt(
        &mut self,
        perspective: usize,
        feature_idx: usize,
        ft: &FeatureTransformer,
        add: bool,
    ) {
        unsafe {
            simd::update_psqt_avx2(
                &mut self.psqt_accumulation[perspective],
                psqt_weights(ft, feature_idx),
                add,
            );
        }
    }

    #[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
    fn update_psqt(
        &mut self,
        perspective: usize,
        feature_idx: usize,
        ft: &FeatureTransformer,
        add: bool,
    ) {
        simd::update_psqt_scalar(
            &mut self.psqt_accumulation[perspective],
            psqt_weights(ft, feature_idx),
            add,
        );
    }
}

#[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
fn refresh_kernel<const SIZE: usize>() -> Option<RefreshFn> {
    if SIZE == 3072 {
        Some(crate::accumulator_refresh::refresh_avx2_3072 as RefreshFn)
    } else if SIZE == 128 {
        Some(crate::accumulator_refresh::refresh_avx2_128 as RefreshFn)
    } else {
        None
    }
}

fn feature_weights<const SIZE: usize>(ft: &FeatureTransformer, feature_idx: usize) -> &[i16] {
    let offset = feature_idx * SIZE;
    &ft.weights[offset..offset + SIZE]
}

fn psqt_weights(ft: &FeatureTransformer, feature_idx: usize) -> &[i32] {
    let offset = feature_idx * PSQT_BUCKETS;
    &ft.psqt_weights[offset..offset + PSQT_BUCKETS]
}
