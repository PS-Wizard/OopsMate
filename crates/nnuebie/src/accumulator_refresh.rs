use crate::aligned::AlignedBuffer;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Tiled AVX2 Refresh implementation for Accumulators
///
/// This implements the Stockfish "phase 2.1" optimization:
/// Instead of iterating pieces and updating the full accumulator for each,
/// we iterate accumulator TILES (e.g. 128-256 elements) and process ALL pieces for that tile.
///
/// This keeps the accumulator values in registers (reducing memory traffic by ~30x).
#[cfg(target_arch = "x86_64")]
#[allow(clippy::identity_op, clippy::erasing_op)]
/// # Safety
/// Requires AVX2 and 32-byte aligned `acc`, `biases`, and `weights` slices sized for 3072 dims.
pub unsafe fn refresh_avx2_3072(
    acc: &mut [i16],
    biases: &[i16],
    weights: &AlignedBuffer<i16>,
    feature_indices: &[usize],
) {
    // 3072 dimensions
    // Tile size: 16 AVX2 registers (16 * 16 i16s = 256 elements)
    // 3072 / 256 = 12 tiles
    // This perfectly fills all 16 YMM registers, leaving none for scratch (but we don't need any for add)

    let acc_ptr = acc.as_mut_ptr();
    let bias_ptr = biases.as_ptr();
    let weight_base = weights.as_ptr();

    // Iterate over 12 tiles of 256 elements each
    for tile_idx in 0..12 {
        let offset = tile_idx * 256;
        let tile_bias_ptr = bias_ptr.add(offset);

        // 1. Load Biases into 16 registers
        let mut r00 = _mm256_load_si256(tile_bias_ptr.add(0 * 16) as *const _);
        let mut r01 = _mm256_load_si256(tile_bias_ptr.add(1 * 16) as *const _);
        let mut r02 = _mm256_load_si256(tile_bias_ptr.add(2 * 16) as *const _);
        let mut r03 = _mm256_load_si256(tile_bias_ptr.add(3 * 16) as *const _);
        let mut r04 = _mm256_load_si256(tile_bias_ptr.add(4 * 16) as *const _);
        let mut r05 = _mm256_load_si256(tile_bias_ptr.add(5 * 16) as *const _);
        let mut r06 = _mm256_load_si256(tile_bias_ptr.add(6 * 16) as *const _);
        let mut r07 = _mm256_load_si256(tile_bias_ptr.add(7 * 16) as *const _);
        let mut r08 = _mm256_load_si256(tile_bias_ptr.add(8 * 16) as *const _);
        let mut r09 = _mm256_load_si256(tile_bias_ptr.add(9 * 16) as *const _);
        let mut r10 = _mm256_load_si256(tile_bias_ptr.add(10 * 16) as *const _);
        let mut r11 = _mm256_load_si256(tile_bias_ptr.add(11 * 16) as *const _);
        let mut r12 = _mm256_load_si256(tile_bias_ptr.add(12 * 16) as *const _);
        let mut r13 = _mm256_load_si256(tile_bias_ptr.add(13 * 16) as *const _);
        let mut r14 = _mm256_load_si256(tile_bias_ptr.add(14 * 16) as *const _);
        let mut r15 = _mm256_load_si256(tile_bias_ptr.add(15 * 16) as *const _);

        // 2. Add features for all pieces to these registers
        for &idx in feature_indices {
            // Calculate pointer to this feature's weights for this tile
            // Full weights array is [NumFeatures * 3072]
            // We want feature[idx] at offset
            let w_ptr = weight_base.add(idx * 3072 + offset);

            r00 = _mm256_add_epi16(r00, _mm256_load_si256(w_ptr.add(0 * 16) as *const _));
            r01 = _mm256_add_epi16(r01, _mm256_load_si256(w_ptr.add(1 * 16) as *const _));
            r02 = _mm256_add_epi16(r02, _mm256_load_si256(w_ptr.add(2 * 16) as *const _));
            r03 = _mm256_add_epi16(r03, _mm256_load_si256(w_ptr.add(3 * 16) as *const _));
            r04 = _mm256_add_epi16(r04, _mm256_load_si256(w_ptr.add(4 * 16) as *const _));
            r05 = _mm256_add_epi16(r05, _mm256_load_si256(w_ptr.add(5 * 16) as *const _));
            r06 = _mm256_add_epi16(r06, _mm256_load_si256(w_ptr.add(6 * 16) as *const _));
            r07 = _mm256_add_epi16(r07, _mm256_load_si256(w_ptr.add(7 * 16) as *const _));
            r08 = _mm256_add_epi16(r08, _mm256_load_si256(w_ptr.add(8 * 16) as *const _));
            r09 = _mm256_add_epi16(r09, _mm256_load_si256(w_ptr.add(9 * 16) as *const _));
            r10 = _mm256_add_epi16(r10, _mm256_load_si256(w_ptr.add(10 * 16) as *const _));
            r11 = _mm256_add_epi16(r11, _mm256_load_si256(w_ptr.add(11 * 16) as *const _));
            r12 = _mm256_add_epi16(r12, _mm256_load_si256(w_ptr.add(12 * 16) as *const _));
            r13 = _mm256_add_epi16(r13, _mm256_load_si256(w_ptr.add(13 * 16) as *const _));
            r14 = _mm256_add_epi16(r14, _mm256_load_si256(w_ptr.add(14 * 16) as *const _));
            r15 = _mm256_add_epi16(r15, _mm256_load_si256(w_ptr.add(15 * 16) as *const _));
        }

        // 3. Store tile back to accumulator
        let tile_acc_ptr = acc_ptr.add(offset);
        _mm256_store_si256(tile_acc_ptr.add(0 * 16) as *mut _, r00);
        _mm256_store_si256(tile_acc_ptr.add(1 * 16) as *mut _, r01);
        _mm256_store_si256(tile_acc_ptr.add(2 * 16) as *mut _, r02);
        _mm256_store_si256(tile_acc_ptr.add(3 * 16) as *mut _, r03);
        _mm256_store_si256(tile_acc_ptr.add(4 * 16) as *mut _, r04);
        _mm256_store_si256(tile_acc_ptr.add(5 * 16) as *mut _, r05);
        _mm256_store_si256(tile_acc_ptr.add(6 * 16) as *mut _, r06);
        _mm256_store_si256(tile_acc_ptr.add(7 * 16) as *mut _, r07);
        _mm256_store_si256(tile_acc_ptr.add(8 * 16) as *mut _, r08);
        _mm256_store_si256(tile_acc_ptr.add(9 * 16) as *mut _, r09);
        _mm256_store_si256(tile_acc_ptr.add(10 * 16) as *mut _, r10);
        _mm256_store_si256(tile_acc_ptr.add(11 * 16) as *mut _, r11);
        _mm256_store_si256(tile_acc_ptr.add(12 * 16) as *mut _, r12);
        _mm256_store_si256(tile_acc_ptr.add(13 * 16) as *mut _, r13);
        _mm256_store_si256(tile_acc_ptr.add(14 * 16) as *mut _, r14);
        _mm256_store_si256(tile_acc_ptr.add(15 * 16) as *mut _, r15);
    }
}

#[cfg(target_arch = "x86_64")]
#[allow(clippy::identity_op, clippy::erasing_op)]
/// # Safety
/// Requires AVX2 and 32-byte aligned `acc`, `biases`, and `weights` slices sized for 128 dims.
pub unsafe fn refresh_avx2_128(
    acc: &mut [i16],
    biases: &[i16],
    weights: &AlignedBuffer<i16>,
    feature_indices: &[usize],
) {
    // 128 dimensions
    // Tile size: 8 AVX2 registers (8 * 16 i16s = 128 elements)
    // 128 / 128 = 1 tile (Single pass!)

    let acc_ptr = acc.as_mut_ptr();
    let bias_ptr = biases.as_ptr();
    let weight_base = weights.as_ptr();

    // Load all biases (128 elements = 8 registers)
    let mut r00 = _mm256_load_si256(bias_ptr.add(0 * 16) as *const _);
    let mut r01 = _mm256_load_si256(bias_ptr.add(1 * 16) as *const _);
    let mut r02 = _mm256_load_si256(bias_ptr.add(2 * 16) as *const _);
    let mut r03 = _mm256_load_si256(bias_ptr.add(3 * 16) as *const _);
    let mut r04 = _mm256_load_si256(bias_ptr.add(4 * 16) as *const _);
    let mut r05 = _mm256_load_si256(bias_ptr.add(5 * 16) as *const _);
    let mut r06 = _mm256_load_si256(bias_ptr.add(6 * 16) as *const _);
    let mut r07 = _mm256_load_si256(bias_ptr.add(7 * 16) as *const _);

    // Add features
    for &idx in feature_indices {
        let w_ptr = weight_base.add(idx * 128); // 128 stride for small net

        r00 = _mm256_add_epi16(r00, _mm256_load_si256(w_ptr.add(0 * 16) as *const _));
        r01 = _mm256_add_epi16(r01, _mm256_load_si256(w_ptr.add(1 * 16) as *const _));
        r02 = _mm256_add_epi16(r02, _mm256_load_si256(w_ptr.add(2 * 16) as *const _));
        r03 = _mm256_add_epi16(r03, _mm256_load_si256(w_ptr.add(3 * 16) as *const _));
        r04 = _mm256_add_epi16(r04, _mm256_load_si256(w_ptr.add(4 * 16) as *const _));
        r05 = _mm256_add_epi16(r05, _mm256_load_si256(w_ptr.add(5 * 16) as *const _));
        r06 = _mm256_add_epi16(r06, _mm256_load_si256(w_ptr.add(6 * 16) as *const _));
        r07 = _mm256_add_epi16(r07, _mm256_load_si256(w_ptr.add(7 * 16) as *const _));
    }

    // Store result
    _mm256_store_si256(acc_ptr.add(0 * 16) as *mut _, r00);
    _mm256_store_si256(acc_ptr.add(1 * 16) as *mut _, r01);
    _mm256_store_si256(acc_ptr.add(2 * 16) as *mut _, r02);
    _mm256_store_si256(acc_ptr.add(3 * 16) as *mut _, r03);
    _mm256_store_si256(acc_ptr.add(4 * 16) as *mut _, r04);
    _mm256_store_si256(acc_ptr.add(5 * 16) as *mut _, r05);
    _mm256_store_si256(acc_ptr.add(6 * 16) as *mut _, r06);
    _mm256_store_si256(acc_ptr.add(7 * 16) as *mut _, r07);
}

/// Update and copy AVX2 - for Finny Table updates
/// Updates `entry` in-place using `added` and `removed`, and copies result to `acc`.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::identity_op, clippy::erasing_op)]
/// # Safety
/// Requires AVX2 and 32-byte aligned `entry`, `acc`, and `weights` slices sized for 3072 dims.
pub unsafe fn update_and_copy_avx2_3072(
    entry: &mut [i16],
    acc: &mut [i16],
    weights: &AlignedBuffer<i16>,
    added: &[usize],
    removed: &[usize],
) {
    let entry_ptr = entry.as_mut_ptr();
    let acc_ptr = acc.as_mut_ptr();
    let weight_base = weights.as_ptr();

    for tile_idx in 0..12 {
        let offset = tile_idx * 256;
        let tile_entry_ptr = entry_ptr.add(offset);
        let tile_acc_ptr = acc_ptr.add(offset);

        // 1. Load Entry (1 tile)
        let mut r00 = _mm256_load_si256(tile_entry_ptr.add(0 * 16) as *const _);
        let mut r01 = _mm256_load_si256(tile_entry_ptr.add(1 * 16) as *const _);
        let mut r02 = _mm256_load_si256(tile_entry_ptr.add(2 * 16) as *const _);
        let mut r03 = _mm256_load_si256(tile_entry_ptr.add(3 * 16) as *const _);
        let mut r04 = _mm256_load_si256(tile_entry_ptr.add(4 * 16) as *const _);
        let mut r05 = _mm256_load_si256(tile_entry_ptr.add(5 * 16) as *const _);
        let mut r06 = _mm256_load_si256(tile_entry_ptr.add(6 * 16) as *const _);
        let mut r07 = _mm256_load_si256(tile_entry_ptr.add(7 * 16) as *const _);
        let mut r08 = _mm256_load_si256(tile_entry_ptr.add(8 * 16) as *const _);
        let mut r09 = _mm256_load_si256(tile_entry_ptr.add(9 * 16) as *const _);
        let mut r10 = _mm256_load_si256(tile_entry_ptr.add(10 * 16) as *const _);
        let mut r11 = _mm256_load_si256(tile_entry_ptr.add(11 * 16) as *const _);
        let mut r12 = _mm256_load_si256(tile_entry_ptr.add(12 * 16) as *const _);
        let mut r13 = _mm256_load_si256(tile_entry_ptr.add(13 * 16) as *const _);
        let mut r14 = _mm256_load_si256(tile_entry_ptr.add(14 * 16) as *const _);
        let mut r15 = _mm256_load_si256(tile_entry_ptr.add(15 * 16) as *const _);

        // 2. Remove features
        for &idx in removed {
            let w_ptr = weight_base.add(idx * 3072 + offset);
            r00 = _mm256_sub_epi16(r00, _mm256_load_si256(w_ptr.add(0 * 16) as *const _));
            r01 = _mm256_sub_epi16(r01, _mm256_load_si256(w_ptr.add(1 * 16) as *const _));
            r02 = _mm256_sub_epi16(r02, _mm256_load_si256(w_ptr.add(2 * 16) as *const _));
            r03 = _mm256_sub_epi16(r03, _mm256_load_si256(w_ptr.add(3 * 16) as *const _));
            r04 = _mm256_sub_epi16(r04, _mm256_load_si256(w_ptr.add(4 * 16) as *const _));
            r05 = _mm256_sub_epi16(r05, _mm256_load_si256(w_ptr.add(5 * 16) as *const _));
            r06 = _mm256_sub_epi16(r06, _mm256_load_si256(w_ptr.add(6 * 16) as *const _));
            r07 = _mm256_sub_epi16(r07, _mm256_load_si256(w_ptr.add(7 * 16) as *const _));
            r08 = _mm256_sub_epi16(r08, _mm256_load_si256(w_ptr.add(8 * 16) as *const _));
            r09 = _mm256_sub_epi16(r09, _mm256_load_si256(w_ptr.add(9 * 16) as *const _));
            r10 = _mm256_sub_epi16(r10, _mm256_load_si256(w_ptr.add(10 * 16) as *const _));
            r11 = _mm256_sub_epi16(r11, _mm256_load_si256(w_ptr.add(11 * 16) as *const _));
            r12 = _mm256_sub_epi16(r12, _mm256_load_si256(w_ptr.add(12 * 16) as *const _));
            r13 = _mm256_sub_epi16(r13, _mm256_load_si256(w_ptr.add(13 * 16) as *const _));
            r14 = _mm256_sub_epi16(r14, _mm256_load_si256(w_ptr.add(14 * 16) as *const _));
            r15 = _mm256_sub_epi16(r15, _mm256_load_si256(w_ptr.add(15 * 16) as *const _));
        }

        // 3. Add features
        for &idx in added {
            let w_ptr = weight_base.add(idx * 3072 + offset);
            r00 = _mm256_add_epi16(r00, _mm256_load_si256(w_ptr.add(0 * 16) as *const _));
            r01 = _mm256_add_epi16(r01, _mm256_load_si256(w_ptr.add(1 * 16) as *const _));
            r02 = _mm256_add_epi16(r02, _mm256_load_si256(w_ptr.add(2 * 16) as *const _));
            r03 = _mm256_add_epi16(r03, _mm256_load_si256(w_ptr.add(3 * 16) as *const _));
            r04 = _mm256_add_epi16(r04, _mm256_load_si256(w_ptr.add(4 * 16) as *const _));
            r05 = _mm256_add_epi16(r05, _mm256_load_si256(w_ptr.add(5 * 16) as *const _));
            r06 = _mm256_add_epi16(r06, _mm256_load_si256(w_ptr.add(6 * 16) as *const _));
            r07 = _mm256_add_epi16(r07, _mm256_load_si256(w_ptr.add(7 * 16) as *const _));
            r08 = _mm256_add_epi16(r08, _mm256_load_si256(w_ptr.add(8 * 16) as *const _));
            r09 = _mm256_add_epi16(r09, _mm256_load_si256(w_ptr.add(9 * 16) as *const _));
            r10 = _mm256_add_epi16(r10, _mm256_load_si256(w_ptr.add(10 * 16) as *const _));
            r11 = _mm256_add_epi16(r11, _mm256_load_si256(w_ptr.add(11 * 16) as *const _));
            r12 = _mm256_add_epi16(r12, _mm256_load_si256(w_ptr.add(12 * 16) as *const _));
            r13 = _mm256_add_epi16(r13, _mm256_load_si256(w_ptr.add(13 * 16) as *const _));
            r14 = _mm256_add_epi16(r14, _mm256_load_si256(w_ptr.add(14 * 16) as *const _));
            r15 = _mm256_add_epi16(r15, _mm256_load_si256(w_ptr.add(15 * 16) as *const _));
        }

        // 4. Store to Entry
        _mm256_store_si256(tile_entry_ptr.add(0 * 16) as *mut _, r00);
        _mm256_store_si256(tile_entry_ptr.add(1 * 16) as *mut _, r01);
        _mm256_store_si256(tile_entry_ptr.add(2 * 16) as *mut _, r02);
        _mm256_store_si256(tile_entry_ptr.add(3 * 16) as *mut _, r03);
        _mm256_store_si256(tile_entry_ptr.add(4 * 16) as *mut _, r04);
        _mm256_store_si256(tile_entry_ptr.add(5 * 16) as *mut _, r05);
        _mm256_store_si256(tile_entry_ptr.add(6 * 16) as *mut _, r06);
        _mm256_store_si256(tile_entry_ptr.add(7 * 16) as *mut _, r07);
        _mm256_store_si256(tile_entry_ptr.add(8 * 16) as *mut _, r08);
        _mm256_store_si256(tile_entry_ptr.add(9 * 16) as *mut _, r09);
        _mm256_store_si256(tile_entry_ptr.add(10 * 16) as *mut _, r10);
        _mm256_store_si256(tile_entry_ptr.add(11 * 16) as *mut _, r11);
        _mm256_store_si256(tile_entry_ptr.add(12 * 16) as *mut _, r12);
        _mm256_store_si256(tile_entry_ptr.add(13 * 16) as *mut _, r13);
        _mm256_store_si256(tile_entry_ptr.add(14 * 16) as *mut _, r14);
        _mm256_store_si256(tile_entry_ptr.add(15 * 16) as *mut _, r15);

        // 5. Store to Target Accumulator
        _mm256_store_si256(tile_acc_ptr.add(0 * 16) as *mut _, r00);
        _mm256_store_si256(tile_acc_ptr.add(1 * 16) as *mut _, r01);
        _mm256_store_si256(tile_acc_ptr.add(2 * 16) as *mut _, r02);
        _mm256_store_si256(tile_acc_ptr.add(3 * 16) as *mut _, r03);
        _mm256_store_si256(tile_acc_ptr.add(4 * 16) as *mut _, r04);
        _mm256_store_si256(tile_acc_ptr.add(5 * 16) as *mut _, r05);
        _mm256_store_si256(tile_acc_ptr.add(6 * 16) as *mut _, r06);
        _mm256_store_si256(tile_acc_ptr.add(7 * 16) as *mut _, r07);
        _mm256_store_si256(tile_acc_ptr.add(8 * 16) as *mut _, r08);
        _mm256_store_si256(tile_acc_ptr.add(9 * 16) as *mut _, r09);
        _mm256_store_si256(tile_acc_ptr.add(10 * 16) as *mut _, r10);
        _mm256_store_si256(tile_acc_ptr.add(11 * 16) as *mut _, r11);
        _mm256_store_si256(tile_acc_ptr.add(12 * 16) as *mut _, r12);
        _mm256_store_si256(tile_acc_ptr.add(13 * 16) as *mut _, r13);
        _mm256_store_si256(tile_acc_ptr.add(14 * 16) as *mut _, r14);
        _mm256_store_si256(tile_acc_ptr.add(15 * 16) as *mut _, r15);
    }
}

#[cfg(target_arch = "x86_64")]
#[allow(clippy::identity_op, clippy::erasing_op)]
/// # Safety
/// Requires AVX2 and 32-byte aligned `entry`, `acc`, and `weights` slices sized for 128 dims.
pub unsafe fn update_and_copy_avx2_128(
    entry: &mut [i16],
    acc: &mut [i16],
    weights: &AlignedBuffer<i16>,
    added: &[usize],
    removed: &[usize],
) {
    let entry_ptr = entry.as_mut_ptr();
    let acc_ptr = acc.as_mut_ptr();
    let weight_base = weights.as_ptr();

    // 1. Load Entry (1 tile)
    let mut r00 = _mm256_load_si256(entry_ptr.add(0 * 16) as *const _);
    let mut r01 = _mm256_load_si256(entry_ptr.add(1 * 16) as *const _);
    let mut r02 = _mm256_load_si256(entry_ptr.add(2 * 16) as *const _);
    let mut r03 = _mm256_load_si256(entry_ptr.add(3 * 16) as *const _);
    let mut r04 = _mm256_load_si256(entry_ptr.add(4 * 16) as *const _);
    let mut r05 = _mm256_load_si256(entry_ptr.add(5 * 16) as *const _);
    let mut r06 = _mm256_load_si256(entry_ptr.add(6 * 16) as *const _);
    let mut r07 = _mm256_load_si256(entry_ptr.add(7 * 16) as *const _);

    // 2. Remove features
    for &idx in removed {
        let w_ptr = weight_base.add(idx * 128);
        r00 = _mm256_sub_epi16(r00, _mm256_load_si256(w_ptr.add(0 * 16) as *const _));
        r01 = _mm256_sub_epi16(r01, _mm256_load_si256(w_ptr.add(1 * 16) as *const _));
        r02 = _mm256_sub_epi16(r02, _mm256_load_si256(w_ptr.add(2 * 16) as *const _));
        r03 = _mm256_sub_epi16(r03, _mm256_load_si256(w_ptr.add(3 * 16) as *const _));
        r04 = _mm256_sub_epi16(r04, _mm256_load_si256(w_ptr.add(4 * 16) as *const _));
        r05 = _mm256_sub_epi16(r05, _mm256_load_si256(w_ptr.add(5 * 16) as *const _));
        r06 = _mm256_sub_epi16(r06, _mm256_load_si256(w_ptr.add(6 * 16) as *const _));
        r07 = _mm256_sub_epi16(r07, _mm256_load_si256(w_ptr.add(7 * 16) as *const _));
    }

    // 3. Add features
    for &idx in added {
        let w_ptr = weight_base.add(idx * 128);
        r00 = _mm256_add_epi16(r00, _mm256_load_si256(w_ptr.add(0 * 16) as *const _));
        r01 = _mm256_add_epi16(r01, _mm256_load_si256(w_ptr.add(1 * 16) as *const _));
        r02 = _mm256_add_epi16(r02, _mm256_load_si256(w_ptr.add(2 * 16) as *const _));
        r03 = _mm256_add_epi16(r03, _mm256_load_si256(w_ptr.add(3 * 16) as *const _));
        r04 = _mm256_add_epi16(r04, _mm256_load_si256(w_ptr.add(4 * 16) as *const _));
        r05 = _mm256_add_epi16(r05, _mm256_load_si256(w_ptr.add(5 * 16) as *const _));
        r06 = _mm256_add_epi16(r06, _mm256_load_si256(w_ptr.add(6 * 16) as *const _));
        r07 = _mm256_add_epi16(r07, _mm256_load_si256(w_ptr.add(7 * 16) as *const _));
    }

    // 4. Store to Entry
    _mm256_store_si256(entry_ptr.add(0 * 16) as *mut _, r00);
    _mm256_store_si256(entry_ptr.add(1 * 16) as *mut _, r01);
    _mm256_store_si256(entry_ptr.add(2 * 16) as *mut _, r02);
    _mm256_store_si256(entry_ptr.add(3 * 16) as *mut _, r03);
    _mm256_store_si256(entry_ptr.add(4 * 16) as *mut _, r04);
    _mm256_store_si256(entry_ptr.add(5 * 16) as *mut _, r05);
    _mm256_store_si256(entry_ptr.add(6 * 16) as *mut _, r06);
    _mm256_store_si256(entry_ptr.add(7 * 16) as *mut _, r07);

    // 5. Store to Target Accumulator
    _mm256_store_si256(acc_ptr.add(0 * 16) as *mut _, r00);
    _mm256_store_si256(acc_ptr.add(1 * 16) as *mut _, r01);
    _mm256_store_si256(acc_ptr.add(2 * 16) as *mut _, r02);
    _mm256_store_si256(acc_ptr.add(3 * 16) as *mut _, r03);
    _mm256_store_si256(acc_ptr.add(4 * 16) as *mut _, r04);
    _mm256_store_si256(acc_ptr.add(5 * 16) as *mut _, r05);
    _mm256_store_si256(acc_ptr.add(6 * 16) as *mut _, r06);
    _mm256_store_si256(acc_ptr.add(7 * 16) as *mut _, r07);
}
