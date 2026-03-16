//! Finny-table cache storage.

use super::refresh::{build_bitboards, update_accumulator_refresh_cache};
use crate::accumulator::Accumulator;
use crate::architecture::{BIG_HALF_DIMS, SMALL_HALF_DIMS};
use crate::feature_transformer::FeatureTransformer;
use crate::feature_transformer::PSQT_BUCKETS;

#[repr(align(64))]
#[derive(Clone)]
pub(crate) struct AlignedI16<const N: usize> {
    data: [i16; N],
}

impl<const N: usize> AlignedI16<N> {
    fn new() -> Self {
        Self { data: [0; N] }
    }

    pub(crate) fn as_slice(&self) -> &[i16] {
        &self.data
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [i16] {
        &mut self.data
    }

    fn copy_from_slice(&mut self, src: &[i16]) {
        self.data.copy_from_slice(src);
    }
}

impl<const N: usize> Default for AlignedI16<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub(crate) struct AccumulatorCacheEntry<const SIZE: usize> {
    pub accumulation: AlignedI16<SIZE>,
    pub psqt_accumulation: [i32; PSQT_BUCKETS],
    pub by_color_bb: [u64; 2],
    pub by_type_bb: [u64; 6],
}

impl<const SIZE: usize> Default for AccumulatorCacheEntry<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> AccumulatorCacheEntry<SIZE> {
    pub fn new() -> Self {
        Self {
            accumulation: AlignedI16::new(),
            psqt_accumulation: [0; PSQT_BUCKETS],
            by_color_bb: [0; 2],
            by_type_bb: [0; 6],
        }
    }

    pub fn clear(&mut self, biases: &[i16]) {
        self.accumulation.copy_from_slice(biases);
        self.psqt_accumulation.fill(0);
        self.by_color_bb.fill(0);
        self.by_type_bb.fill(0);
    }
}

pub(crate) struct AccumulatorCache<const SIZE: usize> {
    pub entries: [[AccumulatorCacheEntry<SIZE>; 2]; 64],
}

impl<const SIZE: usize> Default for AccumulatorCache<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> AccumulatorCache<SIZE> {
    pub fn new() -> Self {
        Self {
            entries: std::array::from_fn(|_| std::array::from_fn(|_| AccumulatorCacheEntry::new())),
        }
    }

    pub fn clear(&mut self, biases: &[i16]) {
        for sq in 0..64 {
            for c in 0..2 {
                self.entries[sq][c].clear(biases);
            }
        }
    }

    pub fn prepopulate(
        &mut self,
        pieces: &[(usize, usize)],
        ft: &FeatureTransformer,
        _king_squares: [usize; 2],
    ) {
        let (current_color_bb, current_type_bb) = build_bitboards(pieces);
        let mut temp_acc = Accumulator::<SIZE>::new();
        for king_sq in 0..64 {
            for c in 0..2 {
                self.entries[king_sq][c].clear(&ft.biases);
                temp_acc.accumulation[c].copy_from_slice(&ft.biases);
                update_accumulator_refresh_cache(
                    ft,
                    &mut temp_acc,
                    self,
                    c,
                    king_sq,
                    &current_color_bb,
                    &current_type_bb,
                );
            }
        }
    }
}

pub struct FinnyTables {
    pub(crate) cache_big: AccumulatorCache<BIG_HALF_DIMS>,
    pub(crate) cache_small: AccumulatorCache<SMALL_HALF_DIMS>,
}

impl Default for FinnyTables {
    fn default() -> Self {
        Self::new()
    }
}

impl FinnyTables {
    /// Creates empty caches for the big and small networks.
    pub fn new() -> Self {
        Self {
            cache_big: AccumulatorCache::new(),
            cache_small: AccumulatorCache::new(),
        }
    }

    /// Resets both caches to their feature-transformer biases.
    pub fn clear(&mut self, biases_big: &[i16], biases_small: &[i16]) {
        self.cache_big.clear(biases_big);
        self.cache_small.clear(biases_small);
    }

    /// Warms cache entries for all king squares from the current piece list.
    pub fn prepopulate(
        &mut self,
        pieces: &[(usize, usize)],
        ft_big: &FeatureTransformer,
        ft_small: &FeatureTransformer,
        king_squares: [usize; 2],
    ) {
        self.cache_big.prepopulate(pieces, ft_big, king_squares);
        self.cache_small.prepopulate(pieces, ft_small, king_squares);
    }
}
