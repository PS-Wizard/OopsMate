use super::accumulator::{ensure_big_frame, ensure_small_frame};
use crate::eval::nnue::aligned::CacheAligned;
use crate::eval::nnue::constants::{
    FC0_OUTPUTS, FC0_TOTAL_OUTPUTS, FC1_OUTPUTS, FC1_PADDED_INPUT_DIMS, OUTPUT_SCALE,
    WEIGHT_SCALE_BITS,
};
use crate::eval::nnue::context::NnueContext;
use crate::eval::nnue::layers::{
    affine_forward, affine_forward_single_output, clipped_relu, sparse_affine_forward,
    sqr_clipped_relu,
};
use crate::eval::nnue::network::{LayerStack, LoadedNetwork};
use crate::eval::nnue::transform::transform_features;
use crate::eval::nnue::compat::{Color, ColorExt, Position};

pub(super) fn evaluate_big_network(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    side_to_move: Color,
    bucket: usize,
) -> (i32, i32) {
    ensure_big_frame(network, position, ctx);

    let depth = ctx.depth;
    let frame = &ctx.frames[depth];
    transform_features(
        [
            &frame.big_accumulation[0][..],
            &frame.big_accumulation[1][..],
        ],
        side_to_move,
        &mut ctx.big_transformed[..],
    );
    let positional_raw = propagate_loaded_network(
        &network.layer_stacks[bucket],
        &ctx.big_transformed[..],
        &mut ctx.fc0_out,
        &mut ctx.fc1_in,
        &mut ctx.fc1_out,
        &mut ctx.fc1_activated,
    );
    let stm = side_to_move.index();
    let opp = stm ^ 1;
    let psqt_raw = (frame.big_psqt[stm][bucket] - frame.big_psqt[opp][bucket]) / 2;
    (psqt_raw / OUTPUT_SCALE, positional_raw / OUTPUT_SCALE)
}

pub(super) fn evaluate_small_network(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    side_to_move: Color,
    bucket: usize,
) -> (i32, i32) {
    ensure_small_frame(network, position, ctx);

    let depth = ctx.depth;
    let frame = &ctx.frames[depth];
    transform_features(
        [
            &frame.small_accumulation[0][..],
            &frame.small_accumulation[1][..],
        ],
        side_to_move,
        &mut ctx.small_transformed[..],
    );
    let positional_raw = propagate_loaded_network(
        &network.layer_stacks[bucket],
        &ctx.small_transformed[..],
        &mut ctx.fc0_out,
        &mut ctx.fc1_in,
        &mut ctx.fc1_out,
        &mut ctx.fc1_activated,
    );
    let stm = side_to_move.index();
    let opp = stm ^ 1;
    let psqt_raw = (frame.small_psqt[stm][bucket] - frame.small_psqt[opp][bucket]) / 2;
    (psqt_raw / OUTPUT_SCALE, positional_raw / OUTPUT_SCALE)
}

fn propagate_loaded_network(
    stack: &LayerStack,
    transformed: &[u8],
    fc0_out: &mut CacheAligned<[i32; FC0_TOTAL_OUTPUTS]>,
    fc1_in: &mut CacheAligned<[u8; FC1_PADDED_INPUT_DIMS]>,
    fc1_out: &mut CacheAligned<[i32; FC1_OUTPUTS]>,
    fc1_activated: &mut CacheAligned<[u8; FC1_OUTPUTS]>,
) -> i32 {
    sparse_affine_forward(&stack.fc0, transformed, fc0_out);

    fc1_in.fill(0);
    sqr_clipped_relu(&fc0_out[..FC0_OUTPUTS], &mut fc1_in[..FC0_OUTPUTS]);
    clipped_relu(
        &fc0_out[..FC0_OUTPUTS],
        &mut fc1_in[FC0_OUTPUTS..FC0_OUTPUTS * 2],
    );

    affine_forward(&stack.fc1, fc1_in, fc1_out);
    clipped_relu(&fc1_out[..], &mut fc1_activated[..]);

    let positional = affine_forward_single_output(&stack.fc2, fc1_activated);

    let residual = fc0_out[FC0_OUTPUTS];
    let residual_forward = residual * (600 * OUTPUT_SCALE) / (127 * (1 << WEIGHT_SCALE_BITS));

    positional + residual_forward
}
