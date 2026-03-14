use crate::accumulator::Accumulator;
use crate::aligned::AlignedBuffer;
use crate::feature_transformer::FeatureTransformer;
use crate::layers::{
    AffineTransform, AffineTransformSparseInput, ClippedReLU, Layer, SqrClippedReLU,
};
use crate::{OUTPUT_SCALE, WEIGHT_SCALE_BITS};
use std::fs::File;
use std::io::{self, BufReader, Read};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub const LAYER_STACKS: usize = 8;

pub const PAWN_VALUE: i32 = 208;
pub const KNIGHT_VALUE: i32 = 781;
pub const BISHOP_VALUE: i32 = 825;
pub const ROOK_VALUE: i32 = 1276;
pub const QUEEN_VALUE: i32 = 2538;

/// Shared immutable big/small NNUE network pair.
pub struct NnueNetworks {
    pub big_net: Network,
    pub small_net: Network,
}

impl NnueNetworks {
    /// Loads the standard big and small Stockfish-style NNUE networks.
    pub fn new(big_path: &str, small_path: &str) -> io::Result<Self> {
        let big_net = Network::load(big_path, true)?;
        let small_net = Network::load(small_path, false)?;
        Ok(Self { big_net, small_net })
    }
}

pub struct Network {
    pub feature_transformer: FeatureTransformer,
    pub fc_0: Vec<AffineTransformSparseInput>,
    pub fc_1: Vec<AffineTransform>,
    pub fc_2: Vec<AffineTransform>,
    pub ac_sqr_0: SqrClippedReLU,
    pub ac_0: ClippedReLU,
    pub ac_1: ClippedReLU,
    pub is_big: bool,
}

pub struct ScratchBuffer {
    pub transformed_features: AlignedBuffer<u8>,
    pub fc_0_out: AlignedBuffer<i32>,
    pub fc_1_in: AlignedBuffer<u8>,
    pub fc_1_out: AlignedBuffer<i32>,
    pub ac_1_out: AlignedBuffer<u8>,
    pub fc_2_out: AlignedBuffer<i32>,
}

impl ScratchBuffer {
    pub fn new(half_dims: usize) -> Self {
        Self {
            transformed_features: AlignedBuffer::new(half_dims),
            fc_0_out: AlignedBuffer::new(16), // L2 + 1
            fc_1_in: AlignedBuffer::new(32),  // L2 * 2, padded to 32 for AVX2
            fc_1_out: AlignedBuffer::new(32), // L3
            ac_1_out: AlignedBuffer::new(32), // L3
            fc_2_out: AlignedBuffer::new(1),  // 1
        }
    }
}

impl Network {
    pub fn load(path: &str, is_big: bool) -> io::Result<Self> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);

        // Read Header
        let version = crate::loader::read_little_endian_u32(&mut reader)?;
        if version != crate::VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid version: {:x}, expected {:x}",
                    version,
                    crate::VERSION
                ),
            ));
        }
        let _hash = crate::loader::read_little_endian_u32(&mut reader)?;

        let desc_len = crate::loader::read_little_endian_u32(&mut reader)? as usize;

        let mut desc = vec![0u8; desc_len];
        reader.read_exact(&mut desc)?;

        // Feature Transformer
        let _hash_ft = crate::loader::read_little_endian_u32(&mut reader)?;

        // Peek/Consume Magic String
        let mut check = [0u8; 17];
        reader.read_exact(&mut check)?;
        if &check != b"COMPRESSED_LEB128" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid LEB128 magic string: {:?} (Ascii: {})",
                    check,
                    String::from_utf8_lossy(&check)
                ),
            ));
        }

        // Determine dims
        let (input_dims, half_dims, l2, l3) = if is_big {
            (22528, 3072, 15, 32)
        } else {
            (22528, 128, 15, 32)
        };

        let mut ft = FeatureTransformer::new(input_dims, half_dims);
        ft.read_parameters(&mut reader, true)?;

        // Layers
        let mut fc_0s = Vec::with_capacity(LAYER_STACKS);
        let mut fc_1s = Vec::with_capacity(LAYER_STACKS);
        let mut fc_2s = Vec::with_capacity(LAYER_STACKS);

        for _ in 0..LAYER_STACKS {
            let _hash_stack = crate::loader::read_little_endian_u32(&mut reader)?;

            // fc_0
            let mut fc_0_layer = AffineTransformSparseInput::new(half_dims, l2 + 1);
            fc_0_layer.read_parameters(&mut reader)?;

            // fc_1
            let mut fc_1_layer = AffineTransform::new(l2 * 2, l3);
            fc_1_layer.read_parameters(&mut reader)?;

            // fc_2
            let mut fc_2_layer = AffineTransform::new(l3, 1);
            fc_2_layer.read_parameters(&mut reader)?;

            fc_0s.push(fc_0_layer);
            fc_1s.push(fc_1_layer);
            fc_2s.push(fc_2_layer);
        }

        Ok(Self {
            feature_transformer: ft,
            fc_0: fc_0s,
            fc_1: fc_1s,
            fc_2: fc_2s,
            ac_sqr_0: SqrClippedReLU::new(l2 + 1),
            ac_0: ClippedReLU::new(l2 + 1),
            ac_1: ClippedReLU::new(l3),
            is_big,
        })
    }

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

            // Stockfish-style optimization:
            // Process 32 output elements (one AVX2 register) per iteration.
            // This consumes 64 input elements (4 AVX2 registers) per perspective.
            let chunk_size = 32;
            let n = (half_dims / 2) / chunk_size * chunk_size;

            // Constants: max=0, min=127*2=254. Weights were scaled by 2 at load time.
            let min = _mm256_set1_epi16(254);
            let max = _mm256_setzero_si256();

            for j in (0..n).step_by(chunk_size) {
                // Load 32 elements for 'us' (in two 16-element vectors)
                // Access pattern corresponds to Stockfish's in0[j*2] and in0[j*2+1]
                let v0a = _mm256_load_si256(acc_ptr.add(j) as *const _);
                let v0b = _mm256_load_si256(acc_ptr.add(j + 16) as *const _);

                // Load 32 elements for 'them'
                let offset_high = half_dims / 2;
                let v1a = _mm256_load_si256(acc_ptr.add(offset_high + j) as *const _);
                let v1b = _mm256_load_si256(acc_ptr.add(offset_high + j + 16) as *const _);

                // Asymmetric clipping:
                // v0 (first op): Full clip [0, 254]
                let v0a_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v0a));
                let v0b_c = _mm256_max_epi16(max, _mm256_min_epi16(min, v0b));

                // v1 (second op): Min-only clip (-inf, 254].
                // Negative values are handled implicitly by packus later.
                let v1a_c = _mm256_min_epi16(min, v1a);
                let v1b_c = _mm256_min_epi16(min, v1b);

                // Multiply: (x * y) / 512
                // Shift left 7, then mulhi (>> 16) results in net >> 9.
                let sum0a = _mm256_slli_epi16(v0a_c, 7);
                let sum0b = _mm256_slli_epi16(v0b_c, 7);

                let pa = _mm256_mulhi_epi16(sum0a, v1a_c);
                let pb = _mm256_mulhi_epi16(sum0b, v1b_c);

                // Packus combines 2 vectors and clips negatives to 0
                let packed = _mm256_packus_epi16(pa, pb);

                _mm256_store_si256(output_ptr.add(offset + j) as *mut _, packed);
            }

            // Scalar fallback for remainder
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

        // Residual scaling
        let residual = scratch.fc_0_out[15];
        let fwd_out = (residual as i64 * (600 * OUTPUT_SCALE) as i64
            / (127 * (1 << WEIGHT_SCALE_BITS)) as i64) as i32;

        let positional = scratch.fc_2_out[0] + fwd_out;

        (psqt / OUTPUT_SCALE, positional / OUTPUT_SCALE)
    }
}
