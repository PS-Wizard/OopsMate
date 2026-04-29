use crate::aligned::CacheAligned;
use crate::arch::DENSE_CHUNK_SIZE;
use crate::constants::{FC1_OUTPUTS, FC1_PADDED_INPUT_DIMS};
use crate::network::DenseLayer;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm_add_epi32, _mm_cvtsi128_si32, _mm_shuffle_epi32, _mm256_castsi256_si128,
    _mm256_dpbusd_epi32, _mm256_extracti128_si256, _mm256_load_si256, _mm256_set1_epi32,
    _mm256_setzero_si256, _mm256_store_si256,
};

#[inline(always)]
pub fn affine_forward(
    layer: &DenseLayer,
    input: &CacheAligned<[u8; FC1_PADDED_INPUT_DIMS]>,
    output: &mut CacheAligned<[i32; FC1_OUTPUTS]>,
) {
    debug_assert_eq!(layer.output_dims, output.len());
    debug_assert!(input.len() >= layer.input_dims);
    debug_assert_eq!(layer.padded_input_dims % DENSE_CHUNK_SIZE, 0);
    debug_assert_eq!(
        layer.output_dims, FC1_OUTPUTS,
        "fc_1 kernel expects 32 outputs"
    );
    debug_assert_eq!(layer.biases.as_ptr() as usize % 32, 0);
    debug_assert_eq!(layer.weights.as_ptr() as usize % 32, 0);
    debug_assert_eq!(input.as_ptr() as usize % 32, 0);
    debug_assert_eq!(output.as_ptr() as usize % 32, 0);

    #[cfg(target_arch = "x86_64")]
    unsafe {
        affine_forward_vnni256(layer, &input[..], &mut output[..]);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
unsafe fn affine_forward_vnni256(layer: &DenseLayer, input: &[u8], output: &mut [i32]) {
    let (mut acc0, mut acc1, mut acc2, mut acc3) = unsafe {
        // SAFETY: biases are 64-byte aligned and contain 32 i32 outputs.
        (
            _mm256_load_si256(layer.biases.as_ptr().cast::<__m256i>()),
            _mm256_load_si256(layer.biases.as_ptr().add(8).cast::<__m256i>()),
            _mm256_load_si256(layer.biases.as_ptr().add(16).cast::<__m256i>()),
            _mm256_load_si256(layer.biases.as_ptr().add(24).cast::<__m256i>()),
        )
    };

    let input32 = input.as_ptr().cast::<u32>();
    let chunk_count = layer.padded_input_dims / DENSE_CHUNK_SIZE;
    let weights = layer.weights.as_ptr();

    for chunk in 0..chunk_count {
        let input_chunk = unsafe {
            // SAFETY: `input` contains at least `padded_input_dims` bytes; each chunk is 4 bytes.
            *input32.add(chunk)
        };
        if input_chunk == 0 {
            continue;
        }

        let packed_input = _mm256_set1_epi32(input_chunk as i32);
        let weight_base = unsafe { weights.add(chunk * layer.output_dims * DENSE_CHUNK_SIZE) };
        let (w0, w1, w2, w3) = unsafe {
            // SAFETY: each chunk owns 32 * 4 = 128 packed weights, exposed as four aligned 32-byte loads.
            (
                _mm256_load_si256(weight_base.cast::<__m256i>()),
                _mm256_load_si256(weight_base.add(32).cast::<__m256i>()),
                _mm256_load_si256(weight_base.add(64).cast::<__m256i>()),
                _mm256_load_si256(weight_base.add(96).cast::<__m256i>()),
            )
        };

        acc0 = _mm256_dpbusd_epi32(acc0, packed_input, w0);
        acc1 = _mm256_dpbusd_epi32(acc1, packed_input, w1);
        acc2 = _mm256_dpbusd_epi32(acc2, packed_input, w2);
        acc3 = _mm256_dpbusd_epi32(acc3, packed_input, w3);
    }

    unsafe {
        // SAFETY: output scratch is 64-byte aligned and stores exactly 32 i32s.
        _mm256_store_si256(output.as_mut_ptr().cast::<__m256i>(), acc0);
        _mm256_store_si256(output.as_mut_ptr().add(8).cast::<__m256i>(), acc1);
        _mm256_store_si256(output.as_mut_ptr().add(16).cast::<__m256i>(), acc2);
        _mm256_store_si256(output.as_mut_ptr().add(24).cast::<__m256i>(), acc3);
    }
}

#[inline(always)]
pub fn affine_forward_single_output(
    layer: &DenseLayer,
    input: &CacheAligned<[u8; FC1_OUTPUTS]>,
) -> i32 {
    debug_assert_eq!(layer.input_dims, FC1_OUTPUTS);
    debug_assert_eq!(layer.padded_input_dims, FC1_OUTPUTS);
    debug_assert_eq!(layer.output_dims, 1, "fc_2 kernel expects 1 output");
    debug_assert_eq!(layer.weights.len(), FC1_OUTPUTS);
    debug_assert_eq!(layer.biases.len(), 1);
    debug_assert_eq!(layer.weights.as_ptr() as usize % 32, 0);
    debug_assert_eq!(input.as_ptr() as usize % 32, 0);

    #[cfg(target_arch = "x86_64")]
    unsafe {
        return affine_forward_single_output_vnni256(layer, &input[..]);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
unsafe fn affine_forward_single_output_vnni256(layer: &DenseLayer, input: &[u8]) -> i32 {
    let sum = unsafe {
        // SAFETY: fc_2 input and weights are 32-byte aligned and each exactly 32 bytes wide.
        let input_vec = _mm256_load_si256(input.as_ptr().cast::<__m256i>());
        let weight_vec = _mm256_load_si256(layer.weights.as_ptr().cast::<__m256i>());
        _mm256_dpbusd_epi32(_mm256_setzero_si256(), input_vec, weight_vec)
    };

    horizontal_add_i32x8(sum) + layer.biases[0]
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn horizontal_add_i32x8(sum: __m256i) -> i32 {
    unsafe {
        // SAFETY: pure register shuffles/adds on the eight accumulated i32 lanes.
        let mut sum128 = _mm_add_epi32(
            _mm256_castsi256_si128(sum),
            _mm256_extracti128_si256(sum, 1),
        );
        sum128 = _mm_add_epi32(sum128, _mm_shuffle_epi32(sum128, 0b10_11_00_01));
        sum128 = _mm_add_epi32(sum128, _mm_shuffle_epi32(sum128, 0b01_00_11_10));
        _mm_cvtsi128_si32(sum128)
    }
}

#[cfg(test)]
fn affine_forward_scalar(layer: &DenseLayer, input: &[u8], output: &mut [i32]) {
    output.copy_from_slice(&layer.biases);

    let chunk_count = layer.padded_input_dims / DENSE_CHUNK_SIZE;
    for chunk in 0..chunk_count {
        let input_base = chunk * DENSE_CHUNK_SIZE;
        let weight_base = chunk * layer.output_dims * DENSE_CHUNK_SIZE;
        let v0 = i32::from(input[input_base]);
        let v1 = i32::from(input[input_base + 1]);
        let v2 = i32::from(input[input_base + 2]);
        let v3 = i32::from(input[input_base + 3]);

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
    use super::{affine_forward, affine_forward_scalar, affine_forward_single_output};
    use crate::aligned::{AlignedSlice, CacheAligned};
    use crate::constants::{FC1_OUTPUTS, FC1_PADDED_INPUT_DIMS};
    use crate::network::DenseLayer;

    #[test]
    fn vnni256_affine_kernel_matches_scalar_reference() {
        let layer = DenseLayer {
            input_dims: 30,
            padded_input_dims: 32,
            output_dims: 32,
            biases: AlignedSlice::from_vec((0..32).map(|v| v * 13 - 90).collect::<Vec<_>>()),
            weights: AlignedSlice::from_vec((-64..64).cycle().take(32 * 32).collect::<Vec<i8>>()),
        };
        let mut input = CacheAligned::new([0u8; FC1_PADDED_INPUT_DIMS]);
        for (idx, slot) in input.iter_mut().enumerate() {
            *slot = ((idx * 7 + 3) % 255) as u8;
        }
        let mut simd = CacheAligned::new([0i32; FC1_OUTPUTS]);
        let mut scalar = [0i32; FC1_OUTPUTS];

        affine_forward(&layer, &input, &mut simd);
        affine_forward_scalar(&layer, &input[..], &mut scalar);

        assert_eq!(*simd, scalar);
    }

    #[test]
    fn vnni256_single_output_kernel_matches_scalar_reference() {
        let layer = DenseLayer {
            input_dims: 32,
            padded_input_dims: 32,
            output_dims: 1,
            biases: AlignedSlice::from_vec(vec![137]),
            weights: AlignedSlice::from_vec((-16..16).collect::<Vec<i8>>()),
        };
        let mut input = CacheAligned::new([0u8; FC1_OUTPUTS]);
        for (idx, slot) in input.iter_mut().enumerate() {
            *slot = ((idx * 11 + 5) % 255) as u8;
        }
        let mut scalar = [0i32; 1];

        affine_forward_scalar(&layer, &input[..], &mut scalar);

        assert_eq!(affine_forward_single_output(&layer, &input), scalar[0]);
    }
}
