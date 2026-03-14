use crate::accumulator::Accumulator;
use crate::feature_transformer::{FeatureTransformer, PSQT_BUCKETS};
use crate::features::make_index;
use crate::types::Piece;
use std::mem::MaybeUninit;

#[repr(align(64))]
#[derive(Clone)]
pub struct AlignedI16<const N: usize> {
    data: [i16; N],
}

impl<const N: usize> AlignedI16<N> {
    fn new() -> Self {
        Self { data: [0; N] }
    }

    fn as_slice(&self) -> &[i16] {
        &self.data
    }

    fn as_mut_slice(&mut self) -> &mut [i16] {
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
pub struct AccumulatorCacheEntry<const SIZE: usize> {
    pub accumulation: AlignedI16<SIZE>,
    pub psqt_accumulation: [i32; PSQT_BUCKETS],
    pub by_color_bb: [u64; 2],
    pub by_type_bb: [u64; 6], // Pawn to King (0-5)
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

pub struct AccumulatorCache<const SIZE: usize> {
    // entries[king_sq][perspective]
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
    pub cache_big: AccumulatorCache<3072>,
    pub cache_small: AccumulatorCache<128>,
}

impl Default for FinnyTables {
    fn default() -> Self {
        Self::new()
    }
}

impl FinnyTables {
    pub fn new() -> Self {
        Self {
            cache_big: AccumulatorCache::new(),
            cache_small: AccumulatorCache::new(),
        }
    }

    pub fn clear(&mut self, biases_big: &[i16], biases_small: &[i16]) {
        self.cache_big.clear(biases_big);
        self.cache_small.clear(biases_small);
    }

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

// Helper to pop lsb from u64
fn pop_lsb(b: &mut u64) -> usize {
    let s = b.trailing_zeros();
    *b &= *b - 1;
    s as usize
}

fn build_bitboards(pieces: &[(usize, usize)]) -> ([u64; 2], [u64; 6]) {
    let mut current_color_bb = [0u64; 2];
    let mut current_type_bb = [0u64; 6];

    for &(sq, pc_idx) in pieces {
        let piece = Piece::from_index(pc_idx);
        if let Some(color) = piece.color() {
            let pt = piece.piece_type();
            if pt > 0 {
                current_color_bb[color.index()] |= 1u64 << sq;
                current_type_bb[pt - 1] |= 1u64 << sq;
            }
        }
    }

    (current_color_bb, current_type_bb)
}

/// Updates the accumulator cache entry and the target accumulator
pub fn update_accumulator_refresh_cache<const SIZE: usize>(
    ft: &FeatureTransformer,
    accumulator: &mut Accumulator<SIZE>,
    cache: &mut AccumulatorCache<SIZE>,
    perspective: usize,
    ksq: usize,
    current_color_bb: &[u64; 2],
    current_type_bb: &[u64; 6],
) {
    let entry = &mut cache.entries[ksq][perspective];

    if entry.by_color_bb == *current_color_bb && entry.by_type_bb == *current_type_bb {
        accumulator.accumulation[perspective].copy_from_slice(entry.accumulation.as_slice());
        accumulator.psqt_accumulation[perspective] = entry.psqt_accumulation;
        accumulator.computed[perspective] = true;
        return;
    }

    let current_color_bb = *current_color_bb;
    let current_type_bb = *current_type_bb;

    let mut added: [MaybeUninit<usize>; 32] = [MaybeUninit::uninit(); 32];
    let mut removed: [MaybeUninit<usize>; 32] = [MaybeUninit::uninit(); 32];
    let mut added_count = 0;
    let mut removed_count = 0;

    for (c, current_bb) in current_color_bb.iter().enumerate() {
        for (pt, current_type) in current_type_bb.iter().enumerate() {
            let piece_idx = if c == 0 { pt + 1 } else { pt + 9 };
            let old_bb = entry.by_color_bb[c] & entry.by_type_bb[pt];
            let new_bb = current_bb & current_type;
            let mut to_remove = old_bb & !new_bb;
            let mut to_add = new_bb & !old_bb;

            while to_remove != 0 {
                let sq = pop_lsb(&mut to_remove);
                removed[removed_count].write(make_index(perspective, sq, piece_idx, ksq));
                removed_count += 1;
            }

            while to_add != 0 {
                let sq = pop_lsb(&mut to_add);
                added[added_count].write(make_index(perspective, sq, piece_idx, ksq));
                added_count += 1;
            }
        }
    }

    debug_assert!(added_count <= 32);
    debug_assert!(removed_count <= 32);

    let added_slice =
        unsafe { std::slice::from_raw_parts(added.as_ptr() as *const usize, added_count) };
    let removed_slice =
        unsafe { std::slice::from_raw_parts(removed.as_ptr() as *const usize, removed_count) };

    let mut updated_accumulation = false;

    #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
    unsafe {
        if SIZE == 3072 {
            crate::accumulator_refresh::update_and_copy_avx2_3072(
                entry.accumulation.as_mut_slice(),
                accumulator.accumulation[perspective].as_mut_slice(),
                &ft.weights,
                added_slice,
                removed_slice,
            );
            updated_accumulation = true;
        } else if SIZE == 128 {
            crate::accumulator_refresh::update_and_copy_avx2_128(
                entry.accumulation.as_mut_slice(),
                accumulator.accumulation[perspective].as_mut_slice(),
                &ft.weights,
                added_slice,
                removed_slice,
            );
            updated_accumulation = true;
        }
    }

    if !updated_accumulation {
        let entry_acc = entry.accumulation.as_mut_slice();

        for &feat_idx in removed_slice {
            let offset = feat_idx * SIZE;
            let w = &ft.weights[offset..offset + SIZE];
            for j in 0..SIZE {
                entry_acc[j] -= w[j];
            }
        }
        for &feat_idx in added_slice {
            let offset = feat_idx * SIZE;
            let w = &ft.weights[offset..offset + SIZE];
            for j in 0..SIZE {
                entry_acc[j] += w[j];
            }
        }
        accumulator.accumulation[perspective].copy_from_slice(entry_acc);
    }

    for &feat_idx in removed_slice {
        let offset = feat_idx * PSQT_BUCKETS;
        let pq = &ft.psqt_weights[offset..offset + PSQT_BUCKETS];
        for (j, &val) in pq.iter().enumerate() {
            entry.psqt_accumulation[j] -= val;
        }
    }
    for &feat_idx in added_slice {
        let offset = feat_idx * PSQT_BUCKETS;
        let pq = &ft.psqt_weights[offset..offset + PSQT_BUCKETS];
        for (j, &val) in pq.iter().enumerate() {
            entry.psqt_accumulation[j] += val;
        }
    }

    entry.by_color_bb = current_color_bb;
    entry.by_type_bb = current_type_bb;

    accumulator.psqt_accumulation[perspective] = entry.psqt_accumulation;
    accumulator.computed[perspective] = true;
}
