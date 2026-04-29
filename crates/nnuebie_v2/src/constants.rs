pub const NNUE_VERSION: u32 = 0x7AF32F20;
pub const FEATURE_SET_HASH: u32 = 0x7F23_4CB8;

pub const FEATURE_DIMS: usize = 22_528;
pub(crate) const MAX_ACTIVE_FEATURES: usize = 32;
pub const PSQT_BUCKETS: usize = 8;
pub const LAYER_STACKS: usize = 8;

pub const BIG_HALF_DIMS: usize = 3_072;
pub const SMALL_HALF_DIMS: usize = 128;

pub const FC0_OUTPUTS: usize = 15;
pub const FC0_TOTAL_OUTPUTS: usize = FC0_OUTPUTS + 1;
pub const FC1_INPUT_DIMS: usize = FC0_OUTPUTS * 2;
pub const FC1_PADDED_INPUT_DIMS: usize = 32;
pub const FC1_OUTPUTS: usize = 32;

pub const OUTPUT_SCALE: i32 = 16;
pub const WEIGHT_SCALE_BITS: i32 = 6;

pub const SMALLNET_SIMPLE_EVAL_THRESHOLD: i32 = 962;
pub const BIG_NET_RECHECK_THRESHOLD: i32 = 236;

pub const PAWN_VALUE: i32 = 208;
pub const KNIGHT_VALUE: i32 = 781;
pub const BISHOP_VALUE: i32 = 825;
pub const ROOK_VALUE: i32 = 1_276;
pub const QUEEN_VALUE: i32 = 2_538;

pub const VALUE_TB_WIN_IN_MAX_PLY: i32 = 31_507;
pub const VALUE_TB_LOSS_IN_MAX_PLY: i32 = -31_507;

pub const DEFAULT_BIG_NETWORK_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/networks/nn-1c0000000000.nnue");
pub const DEFAULT_SMALL_NETWORK_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/networks/nn-37f18f62d772.nnue");

const fn affine_hash(prev_hash: u32, output_dims: u32) -> u32 {
    let mut hash = 0xCC03_DAE4u32.wrapping_add(output_dims);
    hash ^= prev_hash >> 1;
    hash ^= prev_hash << 31;
    hash
}

const fn activation_hash(prev_hash: u32) -> u32 {
    0x538D_24C7u32.wrapping_add(prev_hash)
}

const fn feature_transformer_hash(output_dims: u32) -> u32 {
    FEATURE_SET_HASH ^ (output_dims * 2)
}

const fn network_architecture_hash(transformed_dims: u32) -> u32 {
    let mut hash = 0xEC42_E90Du32 ^ (transformed_dims * 2);
    hash = affine_hash(hash, FC0_TOTAL_OUTPUTS as u32);
    hash = activation_hash(hash);
    hash = affine_hash(hash, FC1_OUTPUTS as u32);
    hash = activation_hash(hash);
    affine_hash(hash, 1)
}

pub const BIG_FEATURE_TRANSFORMER_HASH: u32 = feature_transformer_hash(BIG_HALF_DIMS as u32);
pub const SMALL_FEATURE_TRANSFORMER_HASH: u32 = feature_transformer_hash(SMALL_HALF_DIMS as u32);

pub const BIG_LAYER_STACK_HASH: u32 = network_architecture_hash(BIG_HALF_DIMS as u32);
pub const SMALL_LAYER_STACK_HASH: u32 = network_architecture_hash(SMALL_HALF_DIMS as u32);

pub const BIG_NETWORK_HASH: u32 = BIG_FEATURE_TRANSFORMER_HASH ^ BIG_LAYER_STACK_HASH;
pub const SMALL_NETWORK_HASH: u32 = SMALL_FEATURE_TRANSFORMER_HASH ^ SMALL_LAYER_STACK_HASH;
