pub(crate) const FEATURE_INPUT_DIMS: usize = 22_528;
pub(crate) const BIG_HALF_DIMS: usize = 3_072;
pub(crate) const SMALL_HALF_DIMS: usize = 128;

pub(crate) const PSQT_BUCKET_COUNT: usize = 8;
pub(crate) const LAYER_STACK_COUNT: usize = PSQT_BUCKET_COUNT;

pub(crate) const FC0_HIDDEN_DIMS: usize = 15;
pub(crate) const FC0_OUTPUT_DIMS: usize = FC0_HIDDEN_DIMS + 1;
pub(crate) const FC1_LAYER_INPUT_DIMS: usize = FC0_HIDDEN_DIMS * 2;
pub(crate) const FC1_SCRATCH_DIMS: usize = FC1_LAYER_INPUT_DIMS + 2;
pub(crate) const FC1_OUTPUT_DIMS: usize = 32;

pub(crate) const PAWN_VALUE: i32 = 208;
pub(crate) const KNIGHT_VALUE: i32 = 781;
pub(crate) const BISHOP_VALUE: i32 = 825;
pub(crate) const ROOK_VALUE: i32 = 1276;
pub(crate) const QUEEN_VALUE: i32 = 2538;
