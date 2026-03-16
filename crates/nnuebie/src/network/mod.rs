use crate::accumulator::Accumulator;
use crate::aligned::AlignedBuffer;
use crate::architecture::{FC0_OUTPUT_DIMS, FC1_OUTPUT_DIMS, FC1_SCRATCH_DIMS};
use crate::feature_transformer::FeatureTransformer;
use crate::layers::{AffineTransform, AffineTransformSparseInput, ClippedReLU, SqrClippedReLU};

mod evaluate;
mod load;

/// Shared immutable big/small NNUE network pair.
pub struct NnueNetworks {
    pub(crate) big_net: Network,
    pub(crate) small_net: Network,
}

/// Loaded NNUE network together with its dense layers.
pub(crate) struct Network {
    pub feature_transformer: FeatureTransformer,
    pub fc_0: Vec<AffineTransformSparseInput>,
    pub fc_1: Vec<AffineTransform>,
    pub fc_2: Vec<AffineTransform>,
    pub ac_sqr_0: SqrClippedReLU,
    pub ac_0: ClippedReLU,
    pub ac_1: ClippedReLU,
}

/// Scratch buffers reused across evaluations to avoid per-call allocation.
pub(crate) struct ScratchBuffer {
    pub transformed_features: AlignedBuffer<u8>,
    pub fc_0_out: AlignedBuffer<i32>,
    pub fc_1_in: AlignedBuffer<u8>,
    pub fc_1_out: AlignedBuffer<i32>,
    pub ac_1_out: AlignedBuffer<u8>,
    pub fc_2_out: AlignedBuffer<i32>,
}

impl ScratchBuffer {
    pub(crate) fn new(half_dims: usize) -> Self {
        Self {
            transformed_features: AlignedBuffer::new(half_dims),
            fc_0_out: AlignedBuffer::new(FC0_OUTPUT_DIMS),
            fc_1_in: AlignedBuffer::new(FC1_SCRATCH_DIMS),
            fc_1_out: AlignedBuffer::new(FC1_OUTPUT_DIMS),
            ac_1_out: AlignedBuffer::new(FC1_OUTPUT_DIMS),
            fc_2_out: AlignedBuffer::new(1),
        }
    }
}
impl NnueNetworks {
    /// Loads the standard big and small Stockfish-style NNUE networks.
    pub fn new(big_path: &str, small_path: &str) -> std::io::Result<Self> {
        let big_net = Network::load(big_path, true)?;
        let small_net = Network::load(small_path, false)?;
        Ok(Self { big_net, small_net })
    }
}
