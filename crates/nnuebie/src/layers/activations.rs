use super::Layer;
use std::io::{self, Read};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Clips 32-bit activations into the `[0, 127]` range after scaling.
pub struct ClippedReLU {
    pub dims: usize,
}

impl ClippedReLU {
    pub fn new(dims: usize) -> Self {
        Self { dims }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn propagate_avx2(&self, input: &[i32], output: &mut [u8]) {
        let simd_len = self.dims / 8 * 8;
        for index in (0..simd_len).step_by(8) {
            let values = _mm256_load_si256(input.as_ptr().add(index) as *const _);
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

        for index in simd_len..self.dims {
            output[index] = (input[index] >> 6).clamp(0, 127) as u8;
        }
    }
}

impl Layer for ClippedReLU {
    type Input = i32;
    type Output = u8;

    #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
    fn propagate(&self, input: &[i32], output: &mut [u8]) {
        unsafe {
            self.propagate_avx2(input, output);
        }
    }

    #[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
    fn propagate(&self, input: &[i32], output: &mut [u8]) {
        for (index, &value) in input.iter().enumerate().take(self.dims) {
            output[index] = (value >> 6).clamp(0, 127) as u8;
        }
    }

    fn read_parameters<R: Read>(&mut self, _reader: &mut R) -> io::Result<()> {
        Ok(())
    }
}

/// Squares and clips activations for the first hidden activation stage.
pub struct SqrClippedReLU {
    pub dims: usize,
}

impl SqrClippedReLU {
    pub fn new(dims: usize) -> Self {
        Self { dims }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn propagate_avx2(&self, input: &[i32], output: &mut [u8]) {
        let simd_len = self.dims / 8 * 8;
        for index in (0..simd_len).step_by(8) {
            let values = _mm256_load_si256(input.as_ptr().add(index) as *const _);

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

        for index in simd_len..self.dims {
            let squared = (input[index] as i64) * (input[index] as i64);
            output[index] = (squared >> 19).clamp(0, 127) as u8;
        }
    }
}

impl Layer for SqrClippedReLU {
    type Input = i32;
    type Output = u8;

    #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
    fn propagate(&self, input: &[i32], output: &mut [u8]) {
        unsafe {
            self.propagate_avx2(input, output);
        }
    }

    #[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
    fn propagate(&self, input: &[i32], output: &mut [u8]) {
        for (index, &value) in input.iter().enumerate().take(self.dims) {
            let squared = (value as i64) * (value as i64);
            output[index] = (squared >> 19).clamp(0, 127) as u8;
        }
    }

    fn read_parameters<R: Read>(&mut self, _reader: &mut R) -> io::Result<()> {
        Ok(())
    }
}
