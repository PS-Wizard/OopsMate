use crate::eval::nnue::constants::{BIG_HALF_DIMS, SMALL_HALF_DIMS};
use crate::eval::nnue::context::NnueContext;
use crate::eval::nnue::finny::refresh_from_finny;
use crate::eval::nnue::network::LoadedNetwork;
use crate::eval::nnue::compat::{Color, ColorExt, Position};

#[cfg(test)]
use super::accumulator::apply_feature_add;
#[cfg(test)]
use crate::eval::nnue::compat::PositionExt;
#[cfg(test)]
use crate::eval::nnue::constants::PSQT_BUCKETS;
#[cfg(test)]
use crate::eval::nnue::context::AccumulatorFrame;
#[cfg(test)]
use crate::eval::nnue::features::feature_index_from_piece_code;
#[cfg(test)]
use crate::eval::nnue::network::FeatureTransformer;
#[cfg(test)]
use crate::eval::nnue::compat::Square;

pub(super) fn refresh_frame_big(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    depth: usize,
) {
    refresh_big_side(network, position, ctx, depth, Color::White);
    refresh_big_side(network, position, ctx, depth, Color::Black);
}

pub(super) fn refresh_frame_small(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    depth: usize,
) {
    refresh_small_side(network, position, ctx, depth, Color::White);
    refresh_small_side(network, position, ctx, depth, Color::Black);
}

pub(super) fn refresh_big_side(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    depth: usize,
    perspective: Color,
) {
    let p = perspective.index();
    let frame = &mut ctx.frames[depth];
    refresh_from_finny::<{ BIG_HALF_DIMS }>(
        &network.feature_transformer,
        position,
        &mut ctx.finny.big,
        perspective,
        &mut frame.big_accumulation[p],
        &mut frame.big_psqt[p],
    );
    frame.big_computed[p] = true;
}

pub(super) fn refresh_small_side(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    depth: usize,
    perspective: Color,
) {
    let p = perspective.index();
    let frame = &mut ctx.frames[depth];
    refresh_from_finny::<{ SMALL_HALF_DIMS }>(
        &network.feature_transformer,
        position,
        &mut ctx.finny.small,
        perspective,
        &mut frame.small_accumulation[p],
        &mut frame.small_psqt[p],
    );
    frame.small_computed[p] = true;
}

#[cfg(test)]
pub(super) fn full_refresh_frame_big(
    network: &LoadedNetwork,
    position: &Position,
    frame: &mut AccumulatorFrame,
) {
    full_refresh_big_side(network, position, frame, Color::White);
    full_refresh_big_side(network, position, frame, Color::Black);
}

#[cfg(test)]
pub(super) fn full_refresh_frame_small(
    network: &LoadedNetwork,
    position: &Position,
    frame: &mut AccumulatorFrame,
) {
    full_refresh_small_side(network, position, frame, Color::White);
    full_refresh_small_side(network, position, frame, Color::Black);
}

#[cfg(test)]
fn full_refresh_big_side(
    network: &LoadedNetwork,
    position: &Position,
    frame: &mut AccumulatorFrame,
    perspective: Color,
) {
    let p = perspective.index();
    full_refresh_side::<{ BIG_HALF_DIMS }>(
        &network.feature_transformer,
        position,
        perspective,
        &mut frame.big_accumulation[p],
        &mut frame.big_psqt[p],
    );
    frame.big_computed[p] = true;
}

#[cfg(test)]
fn full_refresh_small_side(
    network: &LoadedNetwork,
    position: &Position,
    frame: &mut AccumulatorFrame,
    perspective: Color,
) {
    let p = perspective.index();
    full_refresh_side::<{ SMALL_HALF_DIMS }>(
        &network.feature_transformer,
        position,
        perspective,
        &mut frame.small_accumulation[p],
        &mut frame.small_psqt[p],
    );
    frame.small_computed[p] = true;
}

#[cfg(test)]
fn full_refresh_side<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    position: &Position,
    perspective: Color,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    accumulation.copy_from_slice(&feature_transformer.biases);
    psqt.fill(0);

    let board = position.board();
    let king_square = board.king_square(perspective);

    for raw in 0u8..64 {
        let square = Square::from_raw(raw);
        let piece_code = board.piece_code_at(square);
        if piece_code == crate::eval::nnue::compat::EMPTY_SQUARE {
            continue;
        }

        let feature_index =
            feature_index_from_piece_code(perspective, piece_code, square, king_square) as usize;
        apply_feature_add::<HALF_DIMS>(feature_transformer, feature_index, accumulation, psqt);
    }
}
