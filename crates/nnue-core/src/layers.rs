//! Layer operations (forward pass)
//!
//! AVX2-oriented implementations of the neural network layers with scalar fallback:
//! - Affine transformation: output = weights * input + biases
//! - Clipped ReLU: clamp to [0, 127] after scaling
//! - Squared Clipped ReLU: square then clamp to [0, 127]

use crate::network::AffineLayer;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
fn avx2_available() -> bool {
    std::arch::is_x86_feature_detected!("avx2")
}

#[cfg(target_arch = "x86_64")]
unsafe fn hsum_256(x: __m256i) -> i32 {
    let hi = _mm256_extracti128_si256(x, 1);
    let lo = _mm256_castsi256_si128(x);
    let sum = _mm_add_epi32(lo, hi);
    let sum = _mm_hadd_epi32(sum, sum);
    let sum = _mm_hadd_epi32(sum, sum);
    _mm_cvtsi128_si32(sum)
}

fn clipped_relu_scalar(input: &[i32], output: &mut [u8]) {
    for (i, &value) in input.iter().enumerate() {
        output[i] = (value >> 6).clamp(0, 127) as u8;
    }
}

fn sqr_clipped_relu_scalar(input: &[i32], output: &mut [u8]) {
    for (i, &value) in input.iter().enumerate() {
        let squared = (value as i64) * (value as i64);
        output[i] = (squared >> 19).clamp(0, 127) as u8;
    }
}

fn affine_forward_scalar(layer: &AffineLayer, input: &[u8], output: &mut [i32]) {
    output[..layer.output_dims].copy_from_slice(&layer.biases);

    for (input_idx, &input_value) in input.iter().enumerate().take(layer.input_dims) {
        if input_value == 0 {
            continue;
        }

        let input_val = input_value as i32;
        for row in 0..layer.output_dims {
            let weight_idx = row * layer.padded_input_dims + input_idx;
            output[row] += layer.weights[weight_idx] as i32 * input_val;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn clipped_relu_avx2(input: &[i32], output: &mut [u8]) {
    let simd_len = output.len() / 8 * 8;
    for index in (0..simd_len).step_by(8) {
        let values = _mm256_loadu_si256(input.as_ptr().add(index) as *const __m256i);
        let scaled = _mm256_srai_epi32(values, 6);

        let lo = _mm256_castsi256_si128(scaled);
        let hi = _mm256_extracti128_si256(scaled, 1);
        let packed32 = _mm_packus_epi32(lo, hi);
        let packed16 = _mm_packus_epi16(packed32, packed32);
        let clamped = _mm_min_epu8(packed16, _mm_set1_epi8(127));

        std::ptr::write_unaligned(
            output.as_mut_ptr().add(index) as *mut i64,
            _mm_cvtsi128_si64(clamped),
        );
    }

    clipped_relu_scalar(&input[simd_len..], &mut output[simd_len..]);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sqr_clipped_relu_avx2(input: &[i32], output: &mut [u8]) {
    let simd_len = output.len() / 8 * 8;
    for index in (0..simd_len).step_by(8) {
        let values = _mm256_loadu_si256(input.as_ptr().add(index) as *const __m256i);

        let even_sq = _mm256_mul_epi32(values, values);
        let even_res = _mm256_srli_epi64(even_sq, 19);

        let odd_values = _mm256_shuffle_epi32(values, 0xF5);
        let odd_sq = _mm256_mul_epi32(odd_values, odd_values);
        let odd_res = _mm256_srli_epi64(odd_sq, 19);

        let even = _mm256_shuffle_epi32(even_res, 0xD8);
        let odd = _mm256_shuffle_epi32(odd_res, 0xD8);
        let unpacked = _mm256_unpacklo_epi32(even, odd);

        let lo = _mm256_castsi256_si128(unpacked);
        let hi = _mm256_extracti128_si256(unpacked, 1);
        let packed32 = _mm_packus_epi32(lo, hi);
        let packed16 = _mm_packus_epi16(packed32, packed32);
        let clamped = _mm_min_epu8(packed16, _mm_set1_epi8(127));

        std::ptr::write_unaligned(
            output.as_mut_ptr().add(index) as *mut i64,
            _mm_cvtsi128_si64(clamped),
        );
    }

    sqr_clipped_relu_scalar(&input[simd_len..], &mut output[simd_len..]);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn affine_forward_avx2(layer: &AffineLayer, input: &[u8], output: &mut [i32]) {
    if layer.output_dims == 32 && layer.padded_input_dims == 32 {
        let input_vec = _mm256_loadu_si256(input.as_ptr() as *const __m256i);
        let ones = _mm256_set1_epi16(1);
        let w_ptr = layer.weights.as_ptr();

        for row in (0..32).step_by(4) {
            let base = row * 32;

            let w0 = _mm256_loadu_si256(w_ptr.add(base) as *const __m256i);
            let w1 = _mm256_loadu_si256(w_ptr.add(base + 32) as *const __m256i);
            let w2 = _mm256_loadu_si256(w_ptr.add(base + 64) as *const __m256i);
            let w3 = _mm256_loadu_si256(w_ptr.add(base + 96) as *const __m256i);

            let p0 = _mm256_maddubs_epi16(input_vec, w0);
            let p1 = _mm256_maddubs_epi16(input_vec, w1);
            let p2 = _mm256_maddubs_epi16(input_vec, w2);
            let p3 = _mm256_maddubs_epi16(input_vec, w3);

            output[row] = hsum_256(_mm256_madd_epi16(p0, ones)) + layer.biases[row];
            output[row + 1] = hsum_256(_mm256_madd_epi16(p1, ones)) + layer.biases[row + 1];
            output[row + 2] = hsum_256(_mm256_madd_epi16(p2, ones)) + layer.biases[row + 2];
            output[row + 3] = hsum_256(_mm256_madd_epi16(p3, ones)) + layer.biases[row + 3];
        }
        return;
    }

    let num_chunks = layer.padded_input_dims / 32;

    if layer.output_dims == 1 {
        let mut acc = _mm256_setzero_si256();
        let weights = layer.weights.as_ptr();
        let ones = _mm256_set1_epi16(1);

        for chunk in 0..num_chunks {
            let input_vec = _mm256_loadu_si256(input.as_ptr().add(chunk * 32) as *const __m256i);
            let weight_vec = _mm256_loadu_si256(weights.add(chunk * 32) as *const __m256i);
            let partial = _mm256_maddubs_epi16(input_vec, weight_vec);
            acc = _mm256_add_epi32(acc, _mm256_madd_epi16(partial, ones));
        }

        output[0] = hsum_256(acc) + layer.biases[0];
        return;
    }

    for row in (0..layer.output_dims).step_by(4) {
        let left = layer.output_dims - row;
        if left >= 4 {
            let mut acc0 = _mm256_setzero_si256();
            let mut acc1 = _mm256_setzero_si256();
            let mut acc2 = _mm256_setzero_si256();
            let mut acc3 = _mm256_setzero_si256();

            let w0 = layer.weights.as_ptr().add(row * layer.padded_input_dims);
            let w1 = w0.add(layer.padded_input_dims);
            let w2 = w1.add(layer.padded_input_dims);
            let w3 = w2.add(layer.padded_input_dims);
            let ones = _mm256_set1_epi16(1);

            for chunk in 0..num_chunks {
                let input_vec =
                    _mm256_loadu_si256(input.as_ptr().add(chunk * 32) as *const __m256i);

                let p0 = _mm256_maddubs_epi16(
                    input_vec,
                    _mm256_loadu_si256(w0.add(chunk * 32) as *const __m256i),
                );
                let p1 = _mm256_maddubs_epi16(
                    input_vec,
                    _mm256_loadu_si256(w1.add(chunk * 32) as *const __m256i),
                );
                let p2 = _mm256_maddubs_epi16(
                    input_vec,
                    _mm256_loadu_si256(w2.add(chunk * 32) as *const __m256i),
                );
                let p3 = _mm256_maddubs_epi16(
                    input_vec,
                    _mm256_loadu_si256(w3.add(chunk * 32) as *const __m256i),
                );

                acc0 = _mm256_add_epi32(acc0, _mm256_madd_epi16(p0, ones));
                acc1 = _mm256_add_epi32(acc1, _mm256_madd_epi16(p1, ones));
                acc2 = _mm256_add_epi32(acc2, _mm256_madd_epi16(p2, ones));
                acc3 = _mm256_add_epi32(acc3, _mm256_madd_epi16(p3, ones));
            }

            output[row] = hsum_256(acc0) + layer.biases[row];
            output[row + 1] = hsum_256(acc1) + layer.biases[row + 1];
            output[row + 2] = hsum_256(acc2) + layer.biases[row + 2];
            output[row + 3] = hsum_256(acc3) + layer.biases[row + 3];
        } else {
            for offset in 0..left {
                let row_idx = row + offset;
                let mut acc = _mm256_setzero_si256();
                let weights = layer
                    .weights
                    .as_ptr()
                    .add(row_idx * layer.padded_input_dims);
                let ones = _mm256_set1_epi16(1);

                for chunk in 0..num_chunks {
                    let input_vec =
                        _mm256_loadu_si256(input.as_ptr().add(chunk * 32) as *const __m256i);
                    let partial = _mm256_maddubs_epi16(
                        input_vec,
                        _mm256_loadu_si256(weights.add(chunk * 32) as *const __m256i),
                    );
                    acc = _mm256_add_epi32(acc, _mm256_madd_epi16(partial, ones));
                }

                output[row_idx] = hsum_256(acc) + layer.biases[row_idx];
            }
        }
    }
}

/// Clipped ReLU activation: scale down and clamp to [0, 127]
/// Input is i32, output is u8
/// Formula: clamp(value >> 6, 0, 127)
pub fn clipped_relu(input: &[i32], output: &mut [u8]) {
    if avx2_available() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            clipped_relu_avx2(input, output);
            return;
        }
    }

    clipped_relu_scalar(input, output);
}

/// Squared Clipped ReLU: square, scale down, and clamp to [0, 127]
/// Input is i32, output is u8
/// Formula: clamp((value * value) >> 19, 0, 127)
pub fn sqr_clipped_relu(input: &[i32], output: &mut [u8]) {
    if avx2_available() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            sqr_clipped_relu_avx2(input, output);
            return;
        }
    }

    sqr_clipped_relu_scalar(input, output);
}

/// Affine layer forward pass.
/// output[i] = bias[i] + sum(weights[i][j] * input[j])
pub fn affine_forward(layer: &AffineLayer, input: &[u8], output: &mut [i32]) {
    if avx2_available() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            affine_forward_avx2(layer, input, output);
            return;
        }
    }

    affine_forward_scalar(layer, input, output);
}

/// FC0 now uses the same row-major AVX2-friendly affine path as the later layers.
pub fn sparse_affine_forward(layer: &AffineLayer, input: &[u8], output: &mut [i32]) {
    affine_forward(layer, input, output);
}

/// Transform features from accumulator to FC0 input
/// This does the element-wise multiplication of accumulator halves
/// and orders by side-to-move perspective
pub fn transform_features(
    accumulation: &[Vec<i16>; 2],
    side_to_move: usize,
    output: &mut [u8],
    verbose: bool,
) {
    if avx2_available() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            transform_features_avx2(accumulation, side_to_move, output, verbose);
            return;
        }
    }

    transform_features_scalar(accumulation, side_to_move, output, verbose);
}

fn transform_features_scalar(
    accumulation: &[Vec<i16>; 2],
    side_to_move: usize,
    output: &mut [u8],
    verbose: bool,
) {
    let half_dims = accumulation[0].len();
    let half = half_dims / 2;

    let us = side_to_move;
    let them = 1 - side_to_move;

    if verbose {
        println!();
        println!("[4] TRANSFORMING FEATURES");
        println!("{}", "-".repeat(70));
        println!(
            "    Side to move: {} (perspective order: us={}, them={})",
            if us == 0 { "White" } else { "Black" },
            us,
            them
        );
        println!("    Accumulator half_dims: {}", half_dims);
        println!(
            "    Output size: {} (2 perspectives × {} values)",
            half_dims, half
        );
    }

    let mut natural = vec![0u8; half_dims];

    // Build features in natural accumulator order first.
    for (p, perspective) in [us, them].iter().enumerate() {
        let offset = half * p; // First half = our perspective, second half = their perspective

        for j in 0..half {
            // Get values from first and second half of accumulator
            let sum0 = accumulation[*perspective][j].clamp(0, 254) as i32;
            let sum1 = accumulation[*perspective][j + half].clamp(0, 254) as i32;

            // Element-wise multiply and scale
            natural[offset + j] = ((sum0 * sum1) / 512) as u8;
        }
    }

    // Reorder into the byte layout produced by `_mm256_packus_epi16` in nnuebie's AVX2 path.
    // Each 32-byte block becomes [0..8, 16..24, 8..16, 24..32].
    for block in (0..half_dims).step_by(32) {
        output[block..block + 8].copy_from_slice(&natural[block..block + 8]);
        output[block + 8..block + 16].copy_from_slice(&natural[block + 16..block + 24]);
        output[block + 16..block + 24].copy_from_slice(&natural[block + 8..block + 16]);
        output[block + 24..block + 32].copy_from_slice(&natural[block + 24..block + 32]);
    }

    if verbose {
        println!("    Output (first 16 values): {:?}", &output[0..16]);
        println!(
            "    Output (last 16 values): {:?}",
            &output[half_dims - 16..half_dims]
        );
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn transform_features_avx2(
    accumulation: &[Vec<i16>; 2],
    side_to_move: usize,
    output: &mut [u8],
    verbose: bool,
) {
    let half_dims = accumulation[0].len();
    let half = half_dims / 2;
    let us = side_to_move;
    let them = 1 - side_to_move;

    if verbose {
        println!();
        println!("[4] TRANSFORMING FEATURES");
        println!("{}", "-".repeat(70));
        println!(
            "    Side to move: {} (perspective order: us={}, them={})",
            if us == 0 { "White" } else { "Black" },
            us,
            them
        );
        println!("    Accumulator half_dims: {}", half_dims);
        println!(
            "    Output size: {} (2 perspectives × {} values)",
            half_dims, half
        );
    }

    let min = _mm256_set1_epi16(254);
    let max = _mm256_setzero_si256();

    for (p, perspective) in [us, them].iter().enumerate() {
        let offset = half * p;
        let acc_ptr = accumulation[*perspective].as_ptr();
        let simd_len = half / 32 * 32;

        for j in (0..simd_len).step_by(32) {
            let v0a = _mm256_loadu_si256(acc_ptr.add(j) as *const __m256i);
            let v0b = _mm256_loadu_si256(acc_ptr.add(j + 16) as *const __m256i);
            let v1a = _mm256_loadu_si256(acc_ptr.add(half + j) as *const __m256i);
            let v1b = _mm256_loadu_si256(acc_ptr.add(half + j + 16) as *const __m256i);

            let v0a_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v0a));
            let v0b_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v0b));
            let v1a_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v1a));
            let v1b_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v1b));

            let pa = _mm256_mulhi_epi16(_mm256_slli_epi16(v0a_c, 7), v1a_c);
            let pb = _mm256_mulhi_epi16(_mm256_slli_epi16(v0b_c, 7), v1b_c);
            let packed = _mm256_packus_epi16(pa, pb);

            _mm256_storeu_si256(output.as_mut_ptr().add(offset + j) as *mut __m256i, packed);
        }

        for j in simd_len..half {
            let sum0 = accumulation[*perspective][j].clamp(0, 254) as i32;
            let sum1 = accumulation[*perspective][j + half].clamp(0, 254) as i32;
            output[offset + j] = ((sum0 * sum1) / 512) as u8;
        }
    }

    if verbose {
        println!("    Output (first 16 values): {:?}", &output[0..16]);
        println!(
            "    Output (last 16 values): {:?}",
            &output[half_dims - 16..half_dims]
        );
    }
}
