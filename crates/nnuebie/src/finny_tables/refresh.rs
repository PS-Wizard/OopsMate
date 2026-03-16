use crate::accumulator::Accumulator;
use crate::architecture::{BIG_HALF_DIMS, SMALL_HALF_DIMS};
use crate::feature_transformer::{FeatureTransformer, PSQT_BUCKETS};
use crate::features::make_index;
use crate::types::Piece;
use std::mem::MaybeUninit;

use super::cache::AccumulatorCache;

fn pop_lsb(b: &mut u64) -> usize {
    let s = b.trailing_zeros();
    *b &= *b - 1;
    s as usize
}

pub fn build_bitboards(pieces: &[(usize, usize)]) -> ([u64; 2], [u64; 6]) {
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
        if SIZE == BIG_HALF_DIMS {
            crate::accumulator_refresh::update_and_copy_avx2_3072(
                entry.accumulation.as_mut_slice(),
                accumulator.accumulation[perspective].as_mut_slice(),
                &ft.weights,
                added_slice,
                removed_slice,
            );
            updated_accumulation = true;
        } else if SIZE == SMALL_HALF_DIMS {
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
