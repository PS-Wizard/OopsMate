use super::Layer;
use crate::aligned::AlignedBuffer;
use crate::loader::{read_i32_array, read_i8_array};
use std::io::{self, Read};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn permuted_fc_column(index: usize) -> usize {
    let chunk = index / 32;
    let byte = index % 32;
    let block = chunk / 2;
    let row = chunk % 2;

    let (block_a, block_b) = if row == 0 {
        (4 * block, 4 * block + 2)
    } else {
        (4 * block + 1, 4 * block + 3)
    };

    if byte < 8 {
        block_a * 16 + byte
    } else if byte < 16 {
        block_b * 16 + (byte - 8)
    } else if byte < 24 {
        block_a * 16 + (byte - 16) + 8
    } else {
        block_b * 16 + (byte - 24) + 8
    }
}

fn scrambled_weight_index(index: usize, padded_input_dims: usize, output_dims: usize) -> usize {
    let chunk_size = 4;
    (index / chunk_size) % (padded_input_dims / chunk_size) * output_dims * chunk_size
        + index / padded_input_dims * chunk_size
        + index % chunk_size
}

/// Sparse-input affine layer used directly after feature transformation.
pub struct AffineTransformSparseInput {
    pub biases: AlignedBuffer<i32>,
    pub weights: AlignedBuffer<i8>,
    pub input_dims: usize,
    pub output_dims: usize,
    pub padded_input_dims: usize,
}

impl AffineTransformSparseInput {
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
    unsafe fn add_dpbusd(acc: __m256i, a: __m256i, b: __m256i) -> __m256i {
        let product = _mm256_maddubs_epi16(a, b);
        let summed = _mm256_madd_epi16(product, _mm256_set1_epi16(1));
        _mm256_add_epi32(acc, summed)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn propagate_avx2(&self, input: &[u8], output: &mut [i32]) {
        debug_assert_eq!(input.len(), self.input_dims);
        debug_assert_eq!(output.len(), self.output_dims);
        debug_assert_eq!(self.output_dims % 8, 0);

        let num_chunks = self.padded_input_dims / 4;
        let input32 = input.as_ptr() as *const i32;

        let bias_ptr = self.biases.as_ptr() as *const __m256i;
        let mut acc0 = _mm256_load_si256(bias_ptr);
        let mut acc1 = _mm256_load_si256(bias_ptr.add(1));

        let weights = self.weights.as_ptr();
        let block_stride = self.output_dims * 4;

        for chunk in 0..num_chunks {
            let input_value = *input32.add(chunk);
            if input_value == 0 {
                continue;
            }

            let input_vec = _mm256_set1_epi32(input_value);
            let column = weights.add(chunk * block_stride);

            acc0 = Self::add_dpbusd(acc0, input_vec, _mm256_load_si256(column as *const __m256i));
            acc1 = Self::add_dpbusd(
                acc1,
                input_vec,
                _mm256_load_si256(column.add(32) as *const __m256i),
            );
        }

        let out_ptr = output.as_mut_ptr() as *mut __m256i;
        _mm256_store_si256(out_ptr, acc0);
        _mm256_store_si256(out_ptr.add(1), acc1);
    }
}

impl Layer for AffineTransformSparseInput {
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
                let weight_idx = scrambled_weight_index(
                    row * self.padded_input_dims + input_idx,
                    self.padded_input_dims,
                    self.output_dims,
                );
                *out += self.weights[weight_idx] as i32 * input_value;
            }
        }
    }

    fn read_parameters<R: Read>(&mut self, reader: &mut R) -> io::Result<()> {
        self.biases = AlignedBuffer::from_vec(read_i32_array(reader, self.output_dims)?);

        let raw_weights = read_i8_array(reader, self.output_dims * self.padded_input_dims)?;
        let mut scrambled = vec![0i8; self.output_dims * self.padded_input_dims];

        for row in 0..self.output_dims {
            let row_offset = row * self.padded_input_dims;
            for column in 0..self.padded_input_dims {
                let source_column = if column < self.input_dims {
                    permuted_fc_column(column)
                } else {
                    column
                };
                let source_idx = row_offset + source_column;
                let dest_idx = scrambled_weight_index(
                    row_offset + column,
                    self.padded_input_dims,
                    self.output_dims,
                );
                scrambled[dest_idx] = raw_weights[source_idx];
            }
        }

        self.weights = AlignedBuffer::from_vec(scrambled);
        Ok(())
    }
}
