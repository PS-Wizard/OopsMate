use crate::aligned::CacheAligned;
use crate::arch::DENSE_CHUNK_SIZE;
use crate::constants::FC0_TOTAL_OUTPUTS;
use crate::network::DenseLayer;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_castsi256_ps, _mm256_cmpeq_epi32, _mm256_dpbusd_epi32, _mm256_load_si256,
    _mm256_loadu_si256, _mm256_movemask_ps, _mm256_set1_epi32, _mm256_setzero_si256,
    _mm256_store_si256,
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
#[inline(always)]
fn is_32_byte_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize & 31) == 0
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
unsafe fn accumulate_chunk_vnni256(
    layer: &DenseLayer,
    weights: *const i8,
    chunk: usize,
    input_chunk: u32,
    acc0: &mut __m256i,
    acc1: &mut __m256i,
) {
    let packed_input = _mm256_set1_epi32(input_chunk as i32);
    let (w0, w1) = unsafe {
        // SAFETY: each chunk owns output_dims * 4 packed weights; fc_0 has 16 outputs,
        // so two 32-byte loads cover the full 64-byte chunk payload.
        let weight_base = weights.add(chunk * layer.output_dims * DENSE_CHUNK_SIZE);
        (
            _mm256_load_si256(weight_base.cast::<__m256i>()),
            _mm256_load_si256(weight_base.add(32).cast::<__m256i>()),
        )
    };

    *acc0 = _mm256_dpbusd_epi32(*acc0, packed_input, w0);
    *acc1 = _mm256_dpbusd_epi32(*acc1, packed_input, w1);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
unsafe fn sparse_affine_forward_vnni256(layer: &DenseLayer, input: &[u8], output: &mut [i32]) {
    let (mut acc0, mut acc1) = unsafe {
        // SAFETY: biases are 64-byte aligned; two aligned 256-bit loads cover all 16 outputs.
        (
            _mm256_load_si256(layer.biases.as_ptr().cast::<__m256i>()),
            _mm256_load_si256(layer.biases.as_ptr().add(8).cast::<__m256i>()),
        )
    };

    let input32 = input.as_ptr().cast::<u32>();
    let chunk_count = layer.padded_input_dims / DENSE_CHUNK_SIZE;
    let block_count = chunk_count / 8;
    let weights = layer.weights.as_ptr();
    let input_ptr = input.as_ptr();
    let zero = _mm256_setzero_si256();

    if block_count != 0 {
        let aligned_input = is_32_byte_aligned(input_ptr);

        for block in 0..block_count {
            let block_ptr = unsafe { input_ptr.add(block * 32) };
            let input_block = unsafe {
                // SAFETY: every block covers exactly 32 bytes inside `input`; aligned load is only
                // used when the caller-provided input pointer is 32-byte aligned.
                if aligned_input {
                    _mm256_load_si256(block_ptr.cast::<__m256i>())
                } else {
                    _mm256_loadu_si256(block_ptr.cast::<__m256i>())
                }
            };
            let zero_mask =
                _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpeq_epi32(input_block, zero)))
                    as u32;
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
            accumulate_chunk_vnni256(layer, weights, chunk, input_chunk, &mut acc0, &mut acc1);
        }
    }

    unsafe {
        // SAFETY: output scratch is 64-byte aligned; two aligned 256-bit stores write exactly that.
        _mm256_store_si256(output.as_mut_ptr().cast::<__m256i>(), acc0);
        _mm256_store_si256(output.as_mut_ptr().add(8).cast::<__m256i>(), acc1);
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
