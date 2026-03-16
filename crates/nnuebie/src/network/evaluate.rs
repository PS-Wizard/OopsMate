use super::{Accumulator, Network, ScratchBuffer};
use crate::layers::Layer;
use crate::{OUTPUT_SCALE, WEIGHT_SCALE_BITS};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

impl Network {
    #[cfg(all(target_arch = "x86_64", feature = "simd_avx2"))]
    fn transform_features<const SIZE: usize>(
        &self,
        accumulator: &Accumulator<SIZE>,
        scratch: &mut ScratchBuffer,
        us: usize,
        them: usize,
    ) {
        unsafe {
            self.transform_features_avx2(accumulator, scratch, us, them);
        }
    }

    #[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
    fn transform_features<const SIZE: usize>(
        &self,
        accumulator: &Accumulator<SIZE>,
        scratch: &mut ScratchBuffer,
        us: usize,
        them: usize,
    ) {
        let half_dims = self.feature_transformer.half_dims;
        debug_assert_eq!(half_dims, SIZE);

        for p in 0..2 {
            let perspective = if p == 0 { us } else { them };
            let offset = (half_dims / 2) * p;

            for j in 0..(half_dims / 2) {
                let sum0 = accumulator.accumulation[perspective][j].clamp(0, 127 * 2) as i32;
                let sum1 = accumulator.accumulation[perspective][j + half_dims / 2]
                    .clamp(0, 127 * 2) as i32;

                scratch.transformed_features[offset + j] = ((sum0 * sum1) / 512) as u8;
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn transform_features_avx2<const SIZE: usize>(
        &self,
        accumulator: &Accumulator<SIZE>,
        scratch: &mut ScratchBuffer,
        us: usize,
        them: usize,
    ) {
        let half_dims = self.feature_transformer.half_dims;
        debug_assert_eq!(half_dims, SIZE);
        let output_ptr = scratch.transformed_features.as_mut_ptr();

        for p in 0..2 {
            let perspective = if p == 0 { us } else { them };
            let offset = (half_dims / 2) * p;
            let acc_ptr = accumulator.accumulation[perspective].as_ptr();

            let chunk_size = 32;
            let n = (half_dims / 2) / chunk_size * chunk_size;

            let min = _mm256_set1_epi16(254);
            let max = _mm256_setzero_si256();

            for j in (0..n).step_by(chunk_size) {
                let v0a = _mm256_load_si256(acc_ptr.add(j) as *const _);
                let v0b = _mm256_load_si256(acc_ptr.add(j + 16) as *const _);

                let offset_high = half_dims / 2;
                let v1a = _mm256_load_si256(acc_ptr.add(offset_high + j) as *const _);
                let v1b = _mm256_load_si256(acc_ptr.add(offset_high + j + 16) as *const _);

                let v0a_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v0a));
                let v0b_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v0b));

                let v1a_c = _mm256_min_epi16(min, v1a);
                let v1b_c = _mm256_min_epi16(min, v1b);

                let sum0a = _mm256_slli_epi16(v0a_c, 7);
                let sum0b = _mm256_slli_epi16(v0b_c, 7);

                let pa = _mm256_mulhi_epi16(sum0a, v1a_c);
                let pb = _mm256_mulhi_epi16(sum0b, v1b_c);

                let packed = _mm256_packus_epi16(pa, pb);
                _mm256_store_si256(output_ptr.add(offset + j) as *mut _, packed);
            }

            for j in n..(half_dims / 2) {
                let sum0 = accumulator.accumulation[perspective][j].clamp(0, 127 * 2) as i32;
                let sum1 = accumulator.accumulation[perspective][j + half_dims / 2]
                    .clamp(0, 127 * 2) as i32;
                *output_ptr.add(offset + j) = ((sum0 * sum1) / 512) as u8;
            }
        }
    }

    pub fn evaluate<const SIZE: usize>(
        &self,
        accumulator: &Accumulator<SIZE>,
        bucket: usize,
        side_to_move: usize,
        scratch: &mut ScratchBuffer,
    ) -> (i32, i32) {
        let us = side_to_move;
        let them = 1 - us;

        let psqt = (accumulator.psqt_accumulation[us][bucket]
            - accumulator.psqt_accumulation[them][bucket])
            / 2;

        self.transform_features(accumulator, scratch, us, them);

        let fc_0 = &self.fc_0[bucket];
        let fc_1 = &self.fc_1[bucket];
        let fc_2 = &self.fc_2[bucket];

        fc_0.propagate(&scratch.transformed_features, &mut scratch.fc_0_out);

        self.ac_sqr_0
            .propagate(&scratch.fc_0_out, &mut scratch.fc_1_in[0..16]);

        self.ac_0
            .propagate(&scratch.fc_0_out, &mut scratch.fc_1_in[15..31]);
        scratch.fc_1_in[30..32].fill(0);

        fc_1.propagate(&scratch.fc_1_in, &mut scratch.fc_1_out);
        self.ac_1
            .propagate(&scratch.fc_1_out, &mut scratch.ac_1_out);
        fc_2.propagate(&scratch.ac_1_out, &mut scratch.fc_2_out);

        let residual = scratch.fc_0_out[15];
        let fwd_out = (residual as i64 * (600 * OUTPUT_SCALE) as i64
            / (127 * (1 << WEIGHT_SCALE_BITS)) as i64) as i32;

        let positional = scratch.fc_2_out[0] + fwd_out;

        (psqt / OUTPUT_SCALE, positional / OUTPUT_SCALE)
    }
}
