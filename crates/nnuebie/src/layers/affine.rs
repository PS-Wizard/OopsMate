use super::Layer;
use crate::aligned::AlignedBuffer;
use crate::loader::{read_i32_array, read_i8_array};
use std::io::{self, Read};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
unsafe fn hsum_256(x: __m256i) -> i32 {
    let hi = _mm256_extracti128_si256(x, 1);
    let lo = _mm256_castsi256_si128(x);
    let sum = _mm_add_epi32(lo, hi);
    let sum = _mm_hadd_epi32(sum, sum);
    let sum = _mm_hadd_epi32(sum, sum);
    _mm_cvtsi128_si32(sum)
}

/// Dense affine layer used by the later NNUE stages.
pub struct AffineTransform {
    pub biases: AlignedBuffer<i32>,
    pub weights: AlignedBuffer<i8>,
    #[cfg_attr(all(target_arch = "x86_64", feature = "simd_avx2"), allow(dead_code))]
    pub input_dims: usize,
    pub output_dims: usize,
    pub padded_input_dims: usize,
}

impl AffineTransform {
    pub fn new(input_dims: usize, output_dims: usize) -> Self {
        let padded_input_dims = input_dims.div_ceil(32) * 32;
        Self {
            biases: AlignedBuffer::new(output_dims),
            weights: AlignedBuffer::new(output_dims * padded_input_dims),
            input_dims,
            output_dims,
            padded_input_dims,
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn propagate_avx2_32x32(&self, input: &[u8], output: &mut [i32]) {
        debug_assert_eq!(self.output_dims, 32);
        debug_assert_eq!(self.padded_input_dims, 32);

        let input_vec = _mm256_load_si256(input.as_ptr() as *const _);
        let ones = _mm256_set1_epi16(1);
        let w_ptr = self.weights.as_ptr();

        for row in (0..32).step_by(4) {
            let base = row * 32;

            let w0 = _mm256_load_si256(w_ptr.add(base) as *const _);
            let w1 = _mm256_load_si256(w_ptr.add(base + 32) as *const _);
            let w2 = _mm256_load_si256(w_ptr.add(base + 64) as *const _);
            let w3 = _mm256_load_si256(w_ptr.add(base + 96) as *const _);

            let p0 = _mm256_maddubs_epi16(input_vec, w0);
            let p1 = _mm256_maddubs_epi16(input_vec, w1);
            let p2 = _mm256_maddubs_epi16(input_vec, w2);
            let p3 = _mm256_maddubs_epi16(input_vec, w3);

            output[row] = hsum_256(_mm256_madd_epi16(p0, ones)) + self.biases[row];
            output[row + 1] = hsum_256(_mm256_madd_epi16(p1, ones)) + self.biases[row + 1];
            output[row + 2] = hsum_256(_mm256_madd_epi16(p2, ones)) + self.biases[row + 2];
            output[row + 3] = hsum_256(_mm256_madd_epi16(p3, ones)) + self.biases[row + 3];
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn propagate_avx2(&self, input: &[u8], output: &mut [i32]) {
        if self.output_dims == 32 && self.padded_input_dims == 32 {
            self.propagate_avx2_32x32(input, output);
            return;
        }

        let num_chunks = self.padded_input_dims / 32;

        if self.output_dims == 1 {
            let mut acc = _mm256_setzero_si256();
            let weights = self.weights.as_ptr();
            let ones = _mm256_set1_epi16(1);

            for chunk in 0..num_chunks {
                let input_vec = _mm256_load_si256(input.as_ptr().add(chunk * 32) as *const _);
                let weight_vec = _mm256_load_si256(weights.add(chunk * 32) as *const _);
                let partial = _mm256_maddubs_epi16(input_vec, weight_vec);
                acc = _mm256_add_epi32(acc, _mm256_madd_epi16(partial, ones));
            }

            output[0] = hsum_256(acc) + self.biases[0];
            return;
        }

        for row in (0..self.output_dims).step_by(4) {
            let left = self.output_dims - row;
            if left >= 4 {
                let mut acc0 = _mm256_setzero_si256();
                let mut acc1 = _mm256_setzero_si256();
                let mut acc2 = _mm256_setzero_si256();
                let mut acc3 = _mm256_setzero_si256();

                let w0 = self.weights.as_ptr().add(row * self.padded_input_dims);
                let w1 = w0.add(self.padded_input_dims);
                let w2 = w1.add(self.padded_input_dims);
                let w3 = w2.add(self.padded_input_dims);
                let ones = _mm256_set1_epi16(1);

                for chunk in 0..num_chunks {
                    let input_vec = _mm256_load_si256(input.as_ptr().add(chunk * 32) as *const _);

                    let p0 = _mm256_maddubs_epi16(
                        input_vec,
                        _mm256_load_si256(w0.add(chunk * 32) as *const _),
                    );
                    let p1 = _mm256_maddubs_epi16(
                        input_vec,
                        _mm256_load_si256(w1.add(chunk * 32) as *const _),
                    );
                    let p2 = _mm256_maddubs_epi16(
                        input_vec,
                        _mm256_load_si256(w2.add(chunk * 32) as *const _),
                    );
                    let p3 = _mm256_maddubs_epi16(
                        input_vec,
                        _mm256_load_si256(w3.add(chunk * 32) as *const _),
                    );

                    acc0 = _mm256_add_epi32(acc0, _mm256_madd_epi16(p0, ones));
                    acc1 = _mm256_add_epi32(acc1, _mm256_madd_epi16(p1, ones));
                    acc2 = _mm256_add_epi32(acc2, _mm256_madd_epi16(p2, ones));
                    acc3 = _mm256_add_epi32(acc3, _mm256_madd_epi16(p3, ones));
                }

                output[row] = hsum_256(acc0) + self.biases[row];
                output[row + 1] = hsum_256(acc1) + self.biases[row + 1];
                output[row + 2] = hsum_256(acc2) + self.biases[row + 2];
                output[row + 3] = hsum_256(acc3) + self.biases[row + 3];
            } else {
                for offset in 0..left {
                    let row_idx = row + offset;
                    let mut acc = _mm256_setzero_si256();
                    let weights = self.weights.as_ptr().add(row_idx * self.padded_input_dims);
                    let ones = _mm256_set1_epi16(1);

                    for chunk in 0..num_chunks {
                        let input_vec =
                            _mm256_load_si256(input.as_ptr().add(chunk * 32) as *const _);
                        let partial = _mm256_maddubs_epi16(
                            input_vec,
                            _mm256_load_si256(weights.add(chunk * 32) as *const _),
                        );
                        acc = _mm256_add_epi32(acc, _mm256_madd_epi16(partial, ones));
                    }

                    output[row_idx] = hsum_256(acc) + self.biases[row_idx];
                }
            }
        }
    }
}

impl Layer for AffineTransform {
    type Input = u8;
    type Output = i32;

    #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
    fn propagate(&self, input: &[u8], output: &mut [i32]) {
        unsafe {
            self.propagate_avx2(input, output);
        }
    }

    #[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
    fn propagate(&self, input: &[u8], output: &mut [i32]) {
        output.copy_from_slice(&self.biases);

        for (input_idx, &input_value) in input.iter().enumerate().take(self.input_dims) {
            if input_value == 0 {
                continue;
            }

            let input_value = input_value as i32;
            for (row, out) in output.iter_mut().enumerate().take(self.output_dims) {
                let weight_idx = row * self.padded_input_dims + input_idx;
                *out += self.weights[weight_idx] as i32 * input_value;
            }
        }
    }

    fn read_parameters<R: Read>(&mut self, reader: &mut R) -> io::Result<()> {
        self.biases = AlignedBuffer::from_vec(read_i32_array(reader, self.output_dims)?);
        self.weights = AlignedBuffer::from_vec(read_i8_array(
            reader,
            self.output_dims * self.padded_input_dims,
        )?);
        Ok(())
    }
}
