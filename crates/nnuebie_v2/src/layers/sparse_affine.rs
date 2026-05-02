use crate::aligned::CacheAligned;
use crate::arch::DENSE_CHUNK_SIZE;
use crate::constants::FC0_TOTAL_OUTPUTS;
use crate::network::DenseLayer;
use crate::simd256::{
    broadcast_i32_bytes32, dpbusd_i32x8, is_32_byte_aligned, load_bytes32_i8,
    load_bytes32_u8_aligned, load_bytes32_u8_unaligned, load_i32x8, store_i32x8, zero_bytes32,
    zero_lane_mask_u32x8, I32x8,
};

#[inline(always)]
pub fn sparse_affine_forward(
    layer: &DenseLayer,
    input: &[u8],
    output: &mut CacheAligned<[i32; FC0_TOTAL_OUTPUTS]>,
) {
    debug_assert_eq!(layer.output_dims, output.len());
    debug_assert!(input.len() >= layer.input_dims);
    debug_assert_eq!(layer.padded_input_dims % DENSE_CHUNK_SIZE, 0);
    debug_assert_eq!(
        layer.output_dims, 16,
        "fc_0 packed kernel expects 16 outputs"
    );
    debug_assert_eq!(layer.biases.as_ptr() as usize % 32, 0);
    debug_assert_eq!(layer.weights.as_ptr() as usize % 32, 0);
    debug_assert_eq!(output.as_ptr() as usize % 32, 0);

    #[cfg(target_arch = "x86_64")]
    unsafe {
        sparse_affine_forward_vnni256(layer, input, &mut output[..]);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
unsafe fn accumulate_chunk_vnni256(
    layer: &DenseLayer,
    weights: *const i8,
    chunk: usize,
    input_chunk: u32,
    acc0: &mut I32x8,
    acc1: &mut I32x8,
) {
    unsafe {
        // SAFETY: each chunk owns exactly two aligned 32-byte weight vectors for fc_0.
        let packed_input = broadcast_i32_bytes32(input_chunk as i32);
        let weight_base = chunk * layer.output_dims * DENSE_CHUNK_SIZE;
        let w0 = load_bytes32_i8(weights, weight_base);
        let w1 = load_bytes32_i8(weights, weight_base + 32);

        *acc0 = dpbusd_i32x8(*acc0, packed_input, w0);
        *acc1 = dpbusd_i32x8(*acc1, packed_input, w1);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
unsafe fn sparse_affine_forward_vnni256(layer: &DenseLayer, input: &[u8], output: &mut [i32]) {
    let (mut acc0, mut acc1): (I32x8, I32x8) = unsafe {
        // SAFETY: biases are 64-byte aligned; two aligned 256-bit loads cover all 16 outputs.
        (
            load_i32x8(layer.biases.as_ptr(), 0),
            load_i32x8(layer.biases.as_ptr(), 8),
        )
    };

    let input32 = input.as_ptr().cast::<u32>();
    let chunk_count = layer.padded_input_dims / DENSE_CHUNK_SIZE;
    let block_count = chunk_count / 8;
    let weights = layer.weights.as_ptr();
    let input_ptr = input.as_ptr();
    let zero = unsafe {
        // SAFETY: zero vector construction is pure register setup.
        zero_bytes32()
    };

    if block_count != 0 {
        let aligned_input = is_32_byte_aligned(input_ptr);

        for block in 0..block_count {
            let block_offset = block * 32;
            let input_block = unsafe {
                // SAFETY: every block covers exactly 32 bytes inside `input`.
                if aligned_input {
                    load_bytes32_u8_aligned(input_ptr, block_offset)
                } else {
                    load_bytes32_u8_unaligned(input_ptr, block_offset)
                }
            };
            let zero_mask = unsafe {
                // SAFETY: `input_block` and `zero` are valid AVX2 register values.
                zero_lane_mask_u32x8(input_block, zero)
            };
            let mut active_mask = (!zero_mask) & 0xff;

            while active_mask != 0 {
                let lane = active_mask.trailing_zeros() as usize;
                active_mask &= active_mask - 1;

                let chunk = block * 8 + lane;
                let input_chunk = unsafe {
                    // SAFETY: chunk index stays within the current loaded 32-byte block.
                    input32.add(chunk).read_unaligned()
                };
                unsafe {
                    // SAFETY: chunk index is in-bounds and `weights` points to aligned packed rows.
                    accumulate_chunk_vnni256(
                        layer,
                        weights,
                        chunk,
                        input_chunk,
                        &mut acc0,
                        &mut acc1,
                    );
                }
            }
        }
    }

    for chunk in (block_count * 8)..chunk_count {
        let input_chunk = unsafe {
            // SAFETY: input has at least padded_input_dims bytes, which is chunk_count * 4.
            input32.add(chunk).read_unaligned()
        };
        if input_chunk == 0 {
            continue;
        }

        unsafe {
            // SAFETY: chunk index is in-bounds and `weights` points to aligned packed rows.
            accumulate_chunk_vnni256(layer, weights, chunk, input_chunk, &mut acc0, &mut acc1);
        }
    }

    unsafe {
        // SAFETY: output scratch is 64-byte aligned; two aligned 256-bit stores write exactly that.
        store_i32x8(output.as_mut_ptr(), 0, acc0);
        store_i32x8(output.as_mut_ptr(), 8, acc1);
    }
}

#[cfg(test)]
fn sparse_affine_forward_scalar(layer: &DenseLayer, input: &[u8], output: &mut [i32]) {
    output.copy_from_slice(&layer.biases);

    let chunk_count = layer.padded_input_dims / DENSE_CHUNK_SIZE;
    for chunk in 0..chunk_count {
        let input_base = chunk * DENSE_CHUNK_SIZE;
        let chunk_bytes = [
            input[input_base],
            input[input_base + 1],
            input[input_base + 2],
            input[input_base + 3],
        ];
        if chunk_bytes == [0; DENSE_CHUNK_SIZE] {
            continue;
        }

        let weight_base = chunk * layer.output_dims * DENSE_CHUNK_SIZE;
        let v0 = i32::from(chunk_bytes[0]);
        let v1 = i32::from(chunk_bytes[1]);
        let v2 = i32::from(chunk_bytes[2]);
        let v3 = i32::from(chunk_bytes[3]);

        for out_index in 0..layer.output_dims {
            let offset = weight_base + out_index * DENSE_CHUNK_SIZE;
            output[out_index] += i32::from(layer.weights[offset]) * v0
                + i32::from(layer.weights[offset + 1]) * v1
                + i32::from(layer.weights[offset + 2]) * v2
                + i32::from(layer.weights[offset + 3]) * v3;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{sparse_affine_forward, sparse_affine_forward_scalar};
    use crate::aligned::{AlignedSlice, CacheAligned};
    use crate::network::DenseLayer;

    #[test]
    fn vnni256_sparse_kernel_matches_scalar_reference() {
        let layer = DenseLayer {
            input_dims: 8,
            padded_input_dims: 8,
            output_dims: 16,
            biases: AlignedSlice::from_vec((0..16).map(|v| v * 17 - 100).collect::<Vec<_>>()),
            weights: AlignedSlice::from_vec(
                (-64..0).chain(0..64).take(16 * 8).collect::<Vec<i8>>(),
            ),
        };
        let input = [0u8, 3, 255, 0, 9, 0, 7, 2];
        let mut simd = CacheAligned::new([0i32; 16]);
        let mut scalar = [0i32; 16];

        sparse_affine_forward(&layer, &input, &mut simd);
        sparse_affine_forward_scalar(&layer, &input, &mut scalar);

        assert_eq!(*simd, scalar);
    }

    #[test]
    fn vnni256_sparse_block_scan_matches_scalar_reference() {
        let layer = DenseLayer {
            input_dims: 32,
            padded_input_dims: 32,
            output_dims: 16,
            biases: AlignedSlice::from_vec((0..16).map(|v| v * 9 - 70).collect::<Vec<_>>()),
            weights: AlignedSlice::from_vec((-64..64).cycle().take(16 * 32).collect::<Vec<i8>>()),
        };
        let input = CacheAligned::new([
            0, 0, 0, 0, 5, 0, 7, 9, 0, 0, 0, 0, 11, 3, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 13, 2, 4, 6,
            0, 0, 0, 0,
        ]);
        let mut simd = CacheAligned::new([0i32; 16]);
        let mut scalar = [0i32; 16];

        sparse_affine_forward(&layer, &input[..], &mut simd);
        sparse_affine_forward_scalar(&layer, &input[..], &mut scalar);

        assert_eq!(*simd, scalar);
    }
}
