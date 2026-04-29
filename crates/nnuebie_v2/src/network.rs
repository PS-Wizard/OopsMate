use crate::aligned::AlignedSlice;
use crate::constants::{
    BIG_FEATURE_TRANSFORMER_HASH, BIG_HALF_DIMS, BIG_LAYER_STACK_HASH, BIG_NETWORK_HASH,
    DEFAULT_BIG_NETWORK_PATH, DEFAULT_SMALL_NETWORK_PATH, FC0_TOTAL_OUTPUTS, FC1_INPUT_DIMS,
    FC1_OUTPUTS, FEATURE_DIMS, LAYER_STACKS, NNUE_VERSION, SMALL_FEATURE_TRANSFORMER_HASH,
    SMALL_HALF_DIMS, SMALL_LAYER_STACK_HASH, SMALL_NETWORK_HASH,
};
use crate::layout::{repack_dense_weights, repack_feature_transformer};
use crate::loader::{
    read_i8_array, read_i32_array, read_leb128_i16_array, read_leb128_i32_array, read_u32,
};
use crate::oopsmate_core::{Color, Position, Square};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionInputs {
    pub side_to_move: Color,
    pub rule50: u16,
    pub piece_count: u8,
    pub white_king: Square,
    pub black_king: Square,
}

#[derive(Debug)]
pub(crate) struct FeatureTransformer {
    pub(crate) biases: AlignedSlice<i16>,
    pub(crate) weights: AlignedSlice<i16>,
    pub(crate) psqt_weights: AlignedSlice<i32>,
}

#[derive(Debug)]
pub(crate) struct DenseLayer {
    pub(crate) input_dims: usize,
    pub(crate) padded_input_dims: usize,
    pub(crate) output_dims: usize,
    pub(crate) biases: AlignedSlice<i32>,
    pub(crate) weights: AlignedSlice<i8>,
}

#[derive(Debug)]
pub(crate) struct LayerStack {
    pub(crate) fc0: DenseLayer,
    pub(crate) fc1: DenseLayer,
    pub(crate) fc2: DenseLayer,
}

#[derive(Debug)]
pub(crate) struct LoadedNetwork {
    #[allow(dead_code)]
    pub(crate) half_dims: usize,
    pub(crate) feature_transformer: FeatureTransformer,
    pub(crate) layer_stacks: Box<[LayerStack]>,
}

#[derive(Debug)]
pub struct NnueNetworks {
    pub(crate) big: LoadedNetwork,
    pub(crate) small: LoadedNetwork,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NetworkKind {
    Big,
    Small,
}

impl NetworkKind {
    #[inline(always)]
    const fn network_hash(self) -> u32 {
        match self {
            Self::Big => BIG_NETWORK_HASH,
            Self::Small => SMALL_NETWORK_HASH,
        }
    }

    #[inline(always)]
    const fn feature_transformer_hash(self) -> u32 {
        match self {
            Self::Big => BIG_FEATURE_TRANSFORMER_HASH,
            Self::Small => SMALL_FEATURE_TRANSFORMER_HASH,
        }
    }

    #[inline(always)]
    const fn layer_stack_hash(self) -> u32 {
        match self {
            Self::Big => BIG_LAYER_STACK_HASH,
            Self::Small => SMALL_LAYER_STACK_HASH,
        }
    }

    #[inline(always)]
    const fn half_dims(self) -> usize {
        match self {
            Self::Big => BIG_HALF_DIMS,
            Self::Small => SMALL_HALF_DIMS,
        }
    }
}

impl NnueNetworks {
    pub fn load_default() -> io::Result<Self> {
        Self::load(DEFAULT_BIG_NETWORK_PATH, DEFAULT_SMALL_NETWORK_PATH)
    }

    pub fn load<P: AsRef<Path>, Q: AsRef<Path>>(big_path: P, small_path: Q) -> io::Result<Self> {
        Ok(Self {
            big: LoadedNetwork::load_from_path(big_path.as_ref(), NetworkKind::Big)?,
            small: LoadedNetwork::load_from_path(small_path.as_ref(), NetworkKind::Small)?,
        })
    }

    #[inline(always)]
    #[must_use]
    pub fn position_inputs(&self, position: &Position) -> PositionInputs {
        let board = position.board();

        PositionInputs {
            side_to_move: position.side_to_move(),
            rule50: position.rule50(),
            piece_count: board.occupied().count_ones() as u8,
            white_king: board.king_square(Color::White),
            black_king: board.king_square(Color::Black),
        }
    }
}

impl LoadedNetwork {
    fn load_from_path(path: &Path, kind: NetworkKind) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::load_from_reader(&mut reader, kind)
    }

    fn load_from_reader<R: Read>(reader: &mut R, kind: NetworkKind) -> io::Result<Self> {
        let version = read_u32(reader)?;
        let header_hash = read_u32(reader)?;
        let description_len = read_u32(reader)? as usize;

        if version != NNUE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid NNUE version: {version:#x}"),
            ));
        }

        if header_hash != kind.network_hash() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected network hash: {header_hash:#x}"),
            ));
        }

        let mut description_bytes = vec![0u8; description_len];
        reader.read_exact(&mut description_bytes)?;
        let _description = String::from_utf8_lossy(&description_bytes).into_owned();

        let transformer_hash = read_u32(reader)?;
        if transformer_hash != kind.feature_transformer_hash() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected feature-transformer hash: {transformer_hash:#x}"),
            ));
        }

        let half_dims = kind.half_dims();
        let mut biases = read_leb128_i16_array(reader, half_dims)?.into_vec();
        let mut weights = read_leb128_i16_array(reader, FEATURE_DIMS * half_dims)?.into_vec();
        repack_feature_transformer(&mut biases, &mut weights);
        let feature_transformer = FeatureTransformer {
            biases: AlignedSlice::from_vec(biases),
            weights: AlignedSlice::from_vec(weights),
            psqt_weights: AlignedSlice::from_vec(
                read_leb128_i32_array(reader, FEATURE_DIMS * 8)?.into_vec(),
            ),
        };

        let mut layer_stacks = Vec::with_capacity(LAYER_STACKS);
        for _ in 0..LAYER_STACKS {
            let stack_hash = read_u32(reader)?;
            if stack_hash != kind.layer_stack_hash() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unexpected layer-stack hash: {stack_hash:#x}"),
                ));
            }

            let fc0 = DenseLayer::load(reader, half_dims, FC0_TOTAL_OUTPUTS)?;
            let fc1 = DenseLayer::load(reader, FC1_INPUT_DIMS, FC1_OUTPUTS)?;
            let fc2 = DenseLayer::load(reader, FC1_OUTPUTS, 1)?;
            layer_stacks.push(LayerStack { fc0, fc1, fc2 });
        }

        let mut trailing = [0u8; 1];
        if reader.read(&mut trailing)? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected trailing bytes after network payload",
            ));
        }

        Ok(Self {
            half_dims,
            feature_transformer,
            layer_stacks: layer_stacks.into_boxed_slice(),
        })
    }
}

impl DenseLayer {
    fn load<R: Read>(reader: &mut R, input_dims: usize, output_dims: usize) -> io::Result<Self> {
        let padded_input_dims = input_dims.next_multiple_of(32);
        let biases = read_i32_array(reader, output_dims)?.into_vec();
        let mut weights = read_i8_array(reader, output_dims * padded_input_dims)?.into_vec();
        repack_dense_weights(&mut weights, padded_input_dims, output_dims);

        Ok(Self {
            input_dims,
            padded_input_dims,
            output_dims,
            biases: AlignedSlice::from_vec(biases),
            weights: AlignedSlice::from_vec(weights),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::NnueNetworks;
    use crate::aligned::CACHELINE_BYTES;
    use crate::constants::{BIG_NETWORK_HASH, SMALL_NETWORK_HASH};
    use crate::oopsmate_core::{Color, Position, Square};

    #[test]
    fn phase1_reads_core_position_directly() {
        let position = Position::startpos();
        let networks = NnueNetworks::load_default().expect("load default networks");
        let inputs = networks.position_inputs(&position);

        assert_eq!(inputs.side_to_move, Color::White);
        assert_eq!(inputs.rule50, 0);
        assert_eq!(inputs.piece_count, 32);
        assert_eq!(inputs.white_king, Square::from_raw(4));
        assert_eq!(inputs.black_king, Square::from_raw(60));

        assert_eq!(BIG_NETWORK_HASH, 0x1c10_20f2);
        assert_eq!(SMALL_NETWORK_HASH, 0x1c10_3c92);
    }

    #[test]
    fn loads_both_sf17_networks() {
        let networks = NnueNetworks::load_default().expect("load default networks");

        assert_eq!(networks.big.half_dims, 3072);
        assert_eq!(networks.small.half_dims, 128);
        assert_eq!(networks.big.layer_stacks.len(), 8);
        assert_eq!(networks.small.layer_stacks.len(), 8);
    }

    #[test]
    fn loaded_network_hot_buffers_are_cache_aligned() {
        let networks = NnueNetworks::load_default().expect("load default networks");

        assert_eq!(
            networks.big.feature_transformer.biases.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
        assert_eq!(
            networks.big.feature_transformer.weights.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
        assert_eq!(
            networks.big.feature_transformer.psqt_weights.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
        assert_eq!(
            networks.big.layer_stacks[0].fc0.biases.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
        assert_eq!(
            networks.big.layer_stacks[0].fc0.weights.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
        assert_eq!(
            networks.small.layer_stacks[0].fc1.biases.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
        assert_eq!(
            networks.small.layer_stacks[0].fc1.weights.as_ptr() as usize % CACHELINE_BYTES,
            0
        );
    }
}
