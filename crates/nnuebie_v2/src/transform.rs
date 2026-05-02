#[cfg(test)]
use crate::arch::{FT_PERMUTE_BLOCK_I16S, FT_PERMUTE_GROUP_I16S, FT_PERMUTE_INVERSE_ORDER};
use crate::oopsmate_core::Color;
use crate::simd256::{
    is_32_byte_aligned, load_i16x16, max_i16x16, min_i16x16, mulhi_i16x16, packus_i16x16,
    shl_i16x16, splat_i16x16, store_bytes32_u8, zero_i16x16,
};

const FT_CLAMP_MAX: i16 = 127 * 2;
const FT_SIMD_OUTPUT_CHUNK: usize = 32;
const FT_SIMD_SHIFT: i32 = 7;

#[inline(always)]
fn transform_perspective(accumulation: &[i16], output: &mut [u8]) {
    debug_assert_eq!(accumulation.len() % 2, 0);
    let half = accumulation.len() / 2;
    debug_assert_eq!(output.len(), half);

    #[cfg(test)]
    if half % FT_SIMD_OUTPUT_CHUNK != 0
        || !is_32_byte_aligned(accumulation.as_ptr())
        || !is_32_byte_aligned(unsafe { accumulation.as_ptr().add(half) })
        || !is_32_byte_aligned(output.as_ptr())
    {
        transform_perspective_scalar(accumulation, output);
        return;
    }

    debug_assert_eq!(half % FT_SIMD_OUTPUT_CHUNK, 0);
    #[cfg(target_arch = "x86_64")]
    {
        debug_assert!(is_32_byte_aligned(accumulation.as_ptr()));
        debug_assert!(is_32_byte_aligned(unsafe {
            accumulation.as_ptr().add(half)
        }));
        debug_assert!(is_32_byte_aligned(output.as_ptr()));
        unsafe {
            transform_perspective_avx2(accumulation, output);
        }
    }
}

#[inline(always)]
#[cfg(test)]
fn transform_perspective_scalar(accumulation: &[i16], output: &mut [u8]) {
    let half = accumulation.len() / 2;

    if half % FT_PERMUTE_GROUP_I16S != 0 {
        for index in 0..half {
            let left = i32::from(accumulation[index]).clamp(0, i32::from(FT_CLAMP_MAX));
            let right = i32::from(accumulation[index + half]).clamp(0, i32::from(FT_CLAMP_MAX));
            output[index] = ((left * right) >> 9) as u8;
        }
        return;
    }

    for group_start in (0..half).step_by(FT_PERMUTE_GROUP_I16S) {
        for logical_block in 0..FT_PERMUTE_INVERSE_ORDER.len() {
            let packed_block = FT_PERMUTE_INVERSE_ORDER[logical_block];
            let packed_start = group_start + packed_block * FT_PERMUTE_BLOCK_I16S;
            let output_start = group_start + logical_block * FT_PERMUTE_BLOCK_I16S;

            for lane in 0..FT_PERMUTE_BLOCK_I16S {
                let index = packed_start + lane;
                let left = i32::from(accumulation[index]).clamp(0, i32::from(FT_CLAMP_MAX));
                let right = i32::from(accumulation[index + half]).clamp(0, i32::from(FT_CLAMP_MAX));
                output[output_start + lane] = ((left * right) >> 9) as u8;
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn transform_perspective_avx2(accumulation: &[i16], output: &mut [u8]) {
    let half = accumulation.len() / 2;
    let chunk_count = half / FT_SIMD_OUTPUT_CHUNK;
    let zero = unsafe {
        // SAFETY: zero vector construction is pure register setup.
        zero_i16x16()
    };
    let one = unsafe {
        // SAFETY: splat vector construction is pure register setup.
        splat_i16x16(FT_CLAMP_MAX)
    };
    let input = accumulation.as_ptr();
    let out = output.as_mut_ptr();

    for chunk in 0..chunk_count {
        let left_offset = chunk * FT_SIMD_OUTPUT_CHUNK;
        let right_offset = half + left_offset;
        let packed = unsafe {
            // SAFETY: caller guarantees both halves and output are 32-byte aligned and sized in 32-byte chunks.
            let left0 = shl_i16x16::<FT_SIMD_SHIFT>(max_i16x16(
                min_i16x16(load_i16x16(input, left_offset), one),
                zero,
            ));
            let left1 = shl_i16x16::<FT_SIMD_SHIFT>(max_i16x16(
                min_i16x16(load_i16x16(input, left_offset + 16), one),
                zero,
            ));
            let right0 = min_i16x16(load_i16x16(input, right_offset), one);
            let right1 = min_i16x16(load_i16x16(input, right_offset + 16), one);

            packus_i16x16(mulhi_i16x16(left0, right0), mulhi_i16x16(left1, right1))
        };

        unsafe {
            // SAFETY: `out` is 32-byte aligned and chunked in 32-byte output groups.
            store_bytes32_u8(out, left_offset, packed);
        }
    }
}

pub fn transform_features(accumulation: [&[i16]; 2], side_to_move: Color, output: &mut [u8]) {
    let half = output.len() / 2;
    let stm = side_to_move.index();
    let opp = stm ^ 1;

    transform_perspective(accumulation[stm], &mut output[..half]);
    transform_perspective(accumulation[opp], &mut output[half..]);
}

#[cfg(test)]
mod tests {
    use super::{transform_features, transform_perspective_scalar};
    use crate::aligned::CacheAligned;
    use crate::oopsmate_core::Color;

    #[test]
    fn transform_clamps_and_multiplies_pairs() {
        let white = [128i16, 64, 64, 128];
        let black = [254i16, 100, 160, 2];
        let mut output = [0u8; 4];

        transform_features([&white, &black], Color::White, &mut output);

        assert_eq!(output[0], 16);
        assert_eq!(output[1], 16);
        assert_eq!(output[2], 79);
        assert_eq!(output[3], 0);
    }

    #[test]
    fn avx2_transform_matches_scalar_reference() {
        let white: CacheAligned<[i16; 128]> = CacheAligned::new(std::array::from_fn(|idx| {
            let lane = idx as i16;
            ((lane * 19) % 401) - 96
        }));
        let black: CacheAligned<[i16; 128]> = CacheAligned::new(std::array::from_fn(|idx| {
            let lane = idx as i16;
            220 - ((lane * 23) % 377)
        }));
        let mut simd = CacheAligned::new([0u8; 128]);
        let mut scalar = [0u8; 128];
        let mut expected_white = [0u8; 64];
        let mut expected_black = [0u8; 64];

        transform_features([&white[..], &black[..]], Color::White, &mut simd[..]);
        transform_perspective_scalar(&white[..], &mut expected_white);
        transform_perspective_scalar(&black[..], &mut expected_black);
        scalar[..64].copy_from_slice(&expected_white);
        scalar[64..].copy_from_slice(&expected_black);

        assert_eq!(*simd, scalar);
    }
}
