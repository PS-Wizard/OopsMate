use super::{
    AffineTransform, AffineTransformSparseInput, ClippedReLU, FeatureTransformer, Network,
    SqrClippedReLU,
};
use crate::architecture::{
    BIG_HALF_DIMS, FC0_OUTPUT_DIMS, FC1_LAYER_INPUT_DIMS, FC1_OUTPUT_DIMS, FEATURE_INPUT_DIMS,
    LAYER_STACK_COUNT, SMALL_HALF_DIMS,
};
use crate::layers::Layer;
use std::fs::File;
use std::io::{self, BufReader, Read};

impl Network {
    pub(crate) fn load(path: &str, is_big: bool) -> io::Result<Self> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);

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

        let _hash_ft = crate::loader::read_little_endian_u32(&mut reader)?;

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

        let (input_dims, half_dims) = if is_big {
            (FEATURE_INPUT_DIMS, BIG_HALF_DIMS)
        } else {
            (FEATURE_INPUT_DIMS, SMALL_HALF_DIMS)
        };

        let mut ft = FeatureTransformer::new(input_dims, half_dims);
        ft.read_parameters(&mut reader, true)?;

        let mut fc_0s = Vec::with_capacity(LAYER_STACK_COUNT);
        let mut fc_1s = Vec::with_capacity(LAYER_STACK_COUNT);
        let mut fc_2s = Vec::with_capacity(LAYER_STACK_COUNT);

        for _ in 0..LAYER_STACK_COUNT {
            let _hash_stack = crate::loader::read_little_endian_u32(&mut reader)?;

            let mut fc_0_layer = AffineTransformSparseInput::new(half_dims, FC0_OUTPUT_DIMS);
            fc_0_layer.read_parameters(&mut reader)?;

            let mut fc_1_layer = AffineTransform::new(FC1_LAYER_INPUT_DIMS, FC1_OUTPUT_DIMS);
            fc_1_layer.read_parameters(&mut reader)?;

            let mut fc_2_layer = AffineTransform::new(FC1_OUTPUT_DIMS, 1);
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
            ac_sqr_0: SqrClippedReLU::new(FC0_OUTPUT_DIMS),
            ac_0: ClippedReLU::new(FC0_OUTPUT_DIMS),
            ac_1: ClippedReLU::new(FC1_OUTPUT_DIMS),
        })
    }
}
