//! Network structure and loading
//!
//! The NNUE network consists of:
//! 1. Feature Transformer - converts sparse input features to dense accumulator
//! 2. FC layers (fc_0, fc_1, fc_2) - dense fully connected layers
//!
//! Architecture:
//! - Input: 22,528 features (HalfKA)
//! - Feature Transformer: 22,528 → 3072 or 128 (per perspective)
//! - FC0: half_dims → 16
//! - FC1: 30 → 32 (after dual activation split)
//! - FC2: 32 → 1 (final output)

use crate::features::{FEATURE_DIMS, PSQT_BUCKETS};
use crate::loader::{
    read_i32_array, read_i8_array, read_leb128_i16, read_leb128_i16_checked, read_leb128_i32,
    read_u32,
};
use std::fs::File;
use std::io::{self, BufReader, Read};

// Architecture constants
pub const BIG_HALF_DIMS: usize = 3072;
pub const SMALL_HALF_DIMS: usize = 128;
const FC0_OUTPUT_DIMS: usize = 16;
const FC1_INPUT_DIMS: usize = 30; // 15 squared + 15 linear (with overlap)
const FC1_OUTPUT_DIMS: usize = 32;
const LAYER_STACK_COUNT: usize = 8; // One set of FC layers per PSQT bucket

// Expected network version
const NNUE_VERSION: u32 = 0x7AF32F20;

/// Feature transformer: converts sparse features to dense accumulator
pub struct FeatureTransformer {
    pub input_dims: usize,
    pub half_dims: usize,
    pub biases: Vec<i16>,       // [half_dims]
    pub weights: Vec<i16>,      // [input_dims * half_dims]
    pub psqt_weights: Vec<i32>, // [input_dims * PSQT_BUCKETS]
}

/// Affine layer (fully connected): output = weights * input + biases
pub struct AffineLayer {
    pub input_dims: usize,
    pub output_dims: usize,
    pub biases: Vec<i32>, // [output_dims]
    pub weights: Vec<i8>, // [output_dims * padded_input_dims]
    pub padded_input_dims: usize,
}

/// Complete NNUE network
pub struct Network {
    pub feature_transformer: FeatureTransformer,
    pub fc_0: Vec<AffineLayer>, // One per bucket
    pub fc_1: Vec<AffineLayer>, // One per bucket
    pub fc_2: Vec<AffineLayer>, // One per bucket
}

impl FeatureTransformer {
    fn new(input_dims: usize, half_dims: usize) -> Self {
        Self {
            input_dims,
            half_dims,
            biases: Vec::new(),
            weights: Vec::new(),
            psqt_weights: Vec::new(),
        }
    }

    fn read_parameters<R: Read>(
        &mut self,
        reader: &mut R,
        skip_first_magic: bool,
    ) -> io::Result<()> {
        // Read biases and weights (LEB128 compressed)
        let mut biases = read_leb128_i16_checked(reader, self.half_dims, !skip_first_magic)?;
        let mut weights = read_leb128_i16(reader, self.half_dims * self.input_dims)?;
        let psqt_weights = read_leb128_i32(reader, PSQT_BUCKETS * self.input_dims)?;

        // The serialized feature-transformer layout follows the SIMD path's pack order.
        permute_weights(&mut biases);
        permute_weights(&mut weights);

        // Scale weights by 2
        for b in &mut biases {
            *b = b.wrapping_mul(2);
        }
        for w in &mut weights {
            *w = w.wrapping_mul(2);
        }

        self.biases = biases;
        self.weights = weights;
        self.psqt_weights = psqt_weights;

        Ok(())
    }
}

impl AffineLayer {
    fn new(input_dims: usize, output_dims: usize) -> Self {
        let padded_input_dims = ((input_dims + 31) / 32) * 32;
        Self {
            input_dims,
            output_dims,
            biases: Vec::new(),
            weights: Vec::new(),
            padded_input_dims,
        }
    }

    fn read_parameters<R: Read>(&mut self, reader: &mut R) -> io::Result<()> {
        self.biases = read_i32_array(reader, self.output_dims)?;
        self.weights = read_i8_array(reader, self.output_dims * self.padded_input_dims)?;
        Ok(())
    }

    fn read_fc0_parameters<R: Read>(&mut self, reader: &mut R) -> io::Result<()> {
        self.biases = read_i32_array(reader, self.output_dims)?;
        let raw_weights = read_i8_array(reader, self.output_dims * self.padded_input_dims)?;
        let mut weights = vec![0i8; self.output_dims * self.padded_input_dims];

        for row in 0..self.output_dims {
            let row_offset = row * self.padded_input_dims;
            for column in 0..self.padded_input_dims {
                let source_column = if column < self.input_dims {
                    permuted_fc_column(column)
                } else {
                    column
                };
                weights[row_offset + column] = raw_weights[row_offset + source_column];
            }
        }

        self.weights = weights;
        Ok(())
    }
}

impl Network {
    pub fn load(path: &str, half_dims: usize) -> io::Result<Self> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);

        // Read and verify header
        let version = read_u32(&mut reader)?;
        if version != NNUE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid NNUE version: {:x}, expected {:x}",
                    version, NNUE_VERSION
                ),
            ));
        }

        let _hash = read_u32(&mut reader)?;

        // Skip description string
        let desc_len = read_u32(&mut reader)? as usize;
        io::copy(&mut reader.by_ref().take(desc_len as u64), &mut io::sink())?;

        // Skip feature transformer hash
        let _hash_ft = read_u32(&mut reader)?;

        // Verify LEB128 magic
        let mut check = [0u8; 17];
        reader.read_exact(&mut check)?;
        if &check != b"COMPRESSED_LEB128" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid LEB128 magic: {:?}",
                    String::from_utf8_lossy(&check)
                ),
            ));
        }

        // Load feature transformer
        let mut ft = FeatureTransformer::new(FEATURE_DIMS, half_dims);
        ft.read_parameters(&mut reader, true)?;

        // Load FC layers (one set per bucket)
        let mut fc_0s = Vec::with_capacity(LAYER_STACK_COUNT);
        let mut fc_1s = Vec::with_capacity(LAYER_STACK_COUNT);
        let mut fc_2s = Vec::with_capacity(LAYER_STACK_COUNT);

        for _ in 0..LAYER_STACK_COUNT {
            let _hash_stack = read_u32(&mut reader)?;

            // FC0: half_dims → 16
            let mut fc_0 = AffineLayer::new(half_dims, FC0_OUTPUT_DIMS);
            fc_0.read_fc0_parameters(&mut reader)?;

            // FC1: 30 → 32
            let mut fc_1 = AffineLayer::new(FC1_INPUT_DIMS, FC1_OUTPUT_DIMS);
            fc_1.read_parameters(&mut reader)?;

            // FC2: 32 → 1
            let mut fc_2 = AffineLayer::new(FC1_OUTPUT_DIMS, 1);
            fc_2.read_parameters(&mut reader)?;

            fc_0s.push(fc_0);
            fc_1s.push(fc_1);
            fc_2s.push(fc_2);
        }

        Ok(Self {
            feature_transformer: ft,
            fc_0: fc_0s,
            fc_1: fc_1s,
            fc_2: fc_2s,
        })
    }
}

// Weight permutation helpers for the serialized SIMD-oriented FT layout.
const PACKUS_EPI16_ORDER: [usize; 8] = [0, 2, 1, 3, 4, 6, 5, 7];

fn permute_weights(data: &mut [i16]) {
    const BLOCK_SIZE: usize = 16;
    const ORDER_SIZE: usize = 8;
    const CHUNK_SIZE: usize = BLOCK_SIZE * ORDER_SIZE;

    if data.len() < CHUNK_SIZE {
        return;
    }

    let mut buffer = [0i16; CHUNK_SIZE];
    let mut i = 0;

    while i + CHUNK_SIZE <= data.len() {
        for j in 0..ORDER_SIZE {
            let src = i + PACKUS_EPI16_ORDER[j] * BLOCK_SIZE;
            buffer[j * BLOCK_SIZE..(j + 1) * BLOCK_SIZE]
                .copy_from_slice(&data[src..src + BLOCK_SIZE]);
        }
        data[i..i + CHUNK_SIZE].copy_from_slice(&buffer);
        i += CHUNK_SIZE;
    }
}

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
