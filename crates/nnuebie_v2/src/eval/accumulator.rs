use super::refresh::{refresh_big_side, refresh_small_side};
use crate::constants::{BIG_HALF_DIMS, PSQT_BUCKETS, SMALL_HALF_DIMS};
use crate::context::{AccumulatorFrame, DirtyPiece, NnueContext};
use crate::features::feature_index_from_piece_code;
use crate::network::{FeatureTransformer, LoadedNetwork};
use crate::update::{
    accum_add, accum_add1_sub1_into, accum_add1_sub2_into, accum_add2_sub1_into,
    accum_add2_sub2_into, accum_sub, psqt_add, psqt_add1_sub1_into, psqt_add1_sub2_into,
    psqt_add2_sub1_into, psqt_add2_sub2_into, psqt_sub,
};
use crate::oopsmate_core::{Color, Position, Square};

pub(crate) fn ensure_big_frame(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
) {
    ensure_big_perspective(network, position, ctx, Color::White);
    ensure_big_perspective(network, position, ctx, Color::Black);
}

pub(crate) fn ensure_small_frame(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
) {
    ensure_small_perspective(network, position, ctx, Color::White);
    ensure_small_perspective(network, position, ctx, Color::Black);
}

fn ensure_big_perspective(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    perspective: Color,
) {
    let p = perspective.index();
    let current = ctx.depth;
    if ctx.frames[current].big_computed[p] {
        return;
    }

    match find_last_usable_big(ctx, perspective) {
        LastUsable::Computed(begin) => {
            let king_square = position.board().king_square(perspective);
            for next in (begin + 1)..=current {
                let (left, right) = ctx.frames.split_at_mut(next);
                let source = &left[next - 1];
                let target = &mut right[0];
                forward_update_big(
                    &network.feature_transformer,
                    perspective,
                    king_square,
                    source,
                    target,
                );
            }
        }
        LastUsable::RefreshBoundary(boundary) => {
            refresh_big_side(network, position, ctx, current, perspective);

            if boundary < current {
                let king_square = position.board().king_square(perspective);
                for prev in (boundary..current).rev() {
                    let (left, right) = ctx.frames.split_at_mut(prev + 1);
                    let target = &mut left[prev];
                    let source = &right[0];
                    backward_update_big(
                        &network.feature_transformer,
                        perspective,
                        king_square,
                        source,
                        target,
                    );
                }
            }
        }
    }
}

fn ensure_small_perspective(
    network: &LoadedNetwork,
    position: &Position,
    ctx: &mut NnueContext,
    perspective: Color,
) {
    let p = perspective.index();
    let current = ctx.depth;
    if ctx.frames[current].small_computed[p] {
        return;
    }

    match find_last_usable_small(ctx, perspective) {
        LastUsable::Computed(begin) => {
            let king_square = position.board().king_square(perspective);
            for next in (begin + 1)..=current {
                let (left, right) = ctx.frames.split_at_mut(next);
                let source = &left[next - 1];
                let target = &mut right[0];
                forward_update_small(
                    &network.feature_transformer,
                    perspective,
                    king_square,
                    source,
                    target,
                );
            }
        }
        LastUsable::RefreshBoundary(boundary) => {
            refresh_small_side(network, position, ctx, current, perspective);

            if boundary < current {
                let king_square = position.board().king_square(perspective);
                for prev in (boundary..current).rev() {
                    let (left, right) = ctx.frames.split_at_mut(prev + 1);
                    let target = &mut left[prev];
                    let source = &right[0];
                    backward_update_small(
                        &network.feature_transformer,
                        perspective,
                        king_square,
                        source,
                        target,
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LastUsable {
    Computed(usize),
    RefreshBoundary(usize),
}

fn find_last_usable_big(ctx: &NnueContext, perspective: Color) -> LastUsable {
    let p = perspective.index();
    for idx in (1..=ctx.depth).rev() {
        if ctx.frames[idx].big_computed[p] {
            return LastUsable::Computed(idx);
        }
        if ctx.frames[idx].dirty.requires_refresh(perspective) {
            return LastUsable::RefreshBoundary(idx);
        }
    }

    if ctx.frames[0].big_computed[p] {
        LastUsable::Computed(0)
    } else {
        LastUsable::RefreshBoundary(0)
    }
}

fn find_last_usable_small(ctx: &NnueContext, perspective: Color) -> LastUsable {
    let p = perspective.index();
    for idx in (1..=ctx.depth).rev() {
        if ctx.frames[idx].small_computed[p] {
            return LastUsable::Computed(idx);
        }
        if ctx.frames[idx].dirty.requires_refresh(perspective) {
            return LastUsable::RefreshBoundary(idx);
        }
    }

    if ctx.frames[0].small_computed[p] {
        LastUsable::Computed(0)
    } else {
        LastUsable::RefreshBoundary(0)
    }
}

fn forward_update_big(
    feature_transformer: &FeatureTransformer,
    perspective: Color,
    king_square: Square,
    source: &AccumulatorFrame,
    target: &mut AccumulatorFrame,
) {
    apply_dirty_update::<BIG_HALF_DIMS>(
        feature_transformer,
        target.dirty,
        perspective,
        king_square,
        &source.big_accumulation[perspective.index()],
        &source.big_psqt[perspective.index()],
        &mut target.big_accumulation[perspective.index()],
        &mut target.big_psqt[perspective.index()],
        true,
    );
    target.big_computed[perspective.index()] = true;
}

fn backward_update_big(
    feature_transformer: &FeatureTransformer,
    perspective: Color,
    king_square: Square,
    source: &AccumulatorFrame,
    target: &mut AccumulatorFrame,
) {
    apply_dirty_update::<BIG_HALF_DIMS>(
        feature_transformer,
        source.dirty,
        perspective,
        king_square,
        &source.big_accumulation[perspective.index()],
        &source.big_psqt[perspective.index()],
        &mut target.big_accumulation[perspective.index()],
        &mut target.big_psqt[perspective.index()],
        false,
    );
    target.big_computed[perspective.index()] = true;
}

fn forward_update_small(
    feature_transformer: &FeatureTransformer,
    perspective: Color,
    king_square: Square,
    source: &AccumulatorFrame,
    target: &mut AccumulatorFrame,
) {
    apply_dirty_update::<SMALL_HALF_DIMS>(
        feature_transformer,
        target.dirty,
        perspective,
        king_square,
        &source.small_accumulation[perspective.index()],
        &source.small_psqt[perspective.index()],
        &mut target.small_accumulation[perspective.index()],
        &mut target.small_psqt[perspective.index()],
        true,
    );
    target.small_computed[perspective.index()] = true;
}

fn backward_update_small(
    feature_transformer: &FeatureTransformer,
    perspective: Color,
    king_square: Square,
    source: &AccumulatorFrame,
    target: &mut AccumulatorFrame,
) {
    apply_dirty_update::<SMALL_HALF_DIMS>(
        feature_transformer,
        source.dirty,
        perspective,
        king_square,
        &source.small_accumulation[perspective.index()],
        &source.small_psqt[perspective.index()],
        &mut target.small_accumulation[perspective.index()],
        &mut target.small_psqt[perspective.index()],
        false,
    );
    target.small_computed[perspective.index()] = true;
}

fn apply_dirty_update<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    dirty: DirtyPiece,
    perspective: Color,
    king_square: Square,
    source_accumulation: &[i16; HALF_DIMS],
    source_psqt: &[i32; PSQT_BUCKETS],
    target_accumulation: &mut [i16; HALF_DIMS],
    target_psqt: &mut [i32; PSQT_BUCKETS],
    forward: bool,
) {
    let mut removed = [0usize; 2];
    let mut added = [0usize; 2];
    let mut removed_len = 0usize;
    let mut added_len = 0usize;

    for idx in 0..dirty.len {
        let piece_code = dirty.piece_codes[idx];
        let removed_square = if forward {
            dirty.from[idx]
        } else {
            dirty.to[idx]
        };
        let added_square = if forward {
            dirty.to[idx]
        } else {
            dirty.from[idx]
        };

        if removed_square.is_valid() {
            removed[removed_len] =
                feature_index_from_piece_code(perspective, piece_code, removed_square, king_square)
                    as usize;
            removed_len += 1;
        }

        if added_square.is_valid() {
            added[added_len] =
                feature_index_from_piece_code(perspective, piece_code, added_square, king_square)
                    as usize;
            added_len += 1;
        }
    }

    match (added_len, removed_len) {
        (0, 0) => {
            target_accumulation.copy_from_slice(source_accumulation);
            target_psqt.copy_from_slice(source_psqt);
        }
        (1, 1) => {
            let add_row =
                &feature_transformer.weights[added[0] * HALF_DIMS..(added[0] + 1) * HALF_DIMS];
            let sub_row =
                &feature_transformer.weights[removed[0] * HALF_DIMS..(removed[0] + 1) * HALF_DIMS];
            let add_psqt = &feature_transformer.psqt_weights
                [added[0] * PSQT_BUCKETS..(added[0] + 1) * PSQT_BUCKETS];
            let sub_psqt = &feature_transformer.psqt_weights
                [removed[0] * PSQT_BUCKETS..(removed[0] + 1) * PSQT_BUCKETS];
            accum_add1_sub1_into(source_accumulation, add_row, sub_row, target_accumulation);
            psqt_add1_sub1_into(source_psqt, add_psqt, sub_psqt, target_psqt);
        }
        (1, 2) => {
            let add_row =
                &feature_transformer.weights[added[0] * HALF_DIMS..(added[0] + 1) * HALF_DIMS];
            let sub0_row =
                &feature_transformer.weights[removed[0] * HALF_DIMS..(removed[0] + 1) * HALF_DIMS];
            let sub1_row =
                &feature_transformer.weights[removed[1] * HALF_DIMS..(removed[1] + 1) * HALF_DIMS];
            let add_psqt = &feature_transformer.psqt_weights
                [added[0] * PSQT_BUCKETS..(added[0] + 1) * PSQT_BUCKETS];
            let sub0_psqt = &feature_transformer.psqt_weights
                [removed[0] * PSQT_BUCKETS..(removed[0] + 1) * PSQT_BUCKETS];
            let sub1_psqt = &feature_transformer.psqt_weights
                [removed[1] * PSQT_BUCKETS..(removed[1] + 1) * PSQT_BUCKETS];
            accum_add1_sub2_into(
                source_accumulation,
                add_row,
                sub0_row,
                sub1_row,
                target_accumulation,
            );
            psqt_add1_sub2_into(source_psqt, add_psqt, sub0_psqt, sub1_psqt, target_psqt);
        }
        (2, 1) => {
            let add0_row =
                &feature_transformer.weights[added[0] * HALF_DIMS..(added[0] + 1) * HALF_DIMS];
            let add1_row =
                &feature_transformer.weights[added[1] * HALF_DIMS..(added[1] + 1) * HALF_DIMS];
            let sub_row =
                &feature_transformer.weights[removed[0] * HALF_DIMS..(removed[0] + 1) * HALF_DIMS];
            let add0_psqt = &feature_transformer.psqt_weights
                [added[0] * PSQT_BUCKETS..(added[0] + 1) * PSQT_BUCKETS];
            let add1_psqt = &feature_transformer.psqt_weights
                [added[1] * PSQT_BUCKETS..(added[1] + 1) * PSQT_BUCKETS];
            let sub_psqt = &feature_transformer.psqt_weights
                [removed[0] * PSQT_BUCKETS..(removed[0] + 1) * PSQT_BUCKETS];
            accum_add2_sub1_into(
                source_accumulation,
                add0_row,
                add1_row,
                sub_row,
                target_accumulation,
            );
            psqt_add2_sub1_into(source_psqt, add0_psqt, add1_psqt, sub_psqt, target_psqt);
        }
        (2, 2) => {
            let add0_row =
                &feature_transformer.weights[added[0] * HALF_DIMS..(added[0] + 1) * HALF_DIMS];
            let add1_row =
                &feature_transformer.weights[added[1] * HALF_DIMS..(added[1] + 1) * HALF_DIMS];
            let sub0_row =
                &feature_transformer.weights[removed[0] * HALF_DIMS..(removed[0] + 1) * HALF_DIMS];
            let sub1_row =
                &feature_transformer.weights[removed[1] * HALF_DIMS..(removed[1] + 1) * HALF_DIMS];
            let add0_psqt = &feature_transformer.psqt_weights
                [added[0] * PSQT_BUCKETS..(added[0] + 1) * PSQT_BUCKETS];
            let add1_psqt = &feature_transformer.psqt_weights
                [added[1] * PSQT_BUCKETS..(added[1] + 1) * PSQT_BUCKETS];
            let sub0_psqt = &feature_transformer.psqt_weights
                [removed[0] * PSQT_BUCKETS..(removed[0] + 1) * PSQT_BUCKETS];
            let sub1_psqt = &feature_transformer.psqt_weights
                [removed[1] * PSQT_BUCKETS..(removed[1] + 1) * PSQT_BUCKETS];
            accum_add2_sub2_into(
                source_accumulation,
                add0_row,
                add1_row,
                sub0_row,
                sub1_row,
                target_accumulation,
            );
            psqt_add2_sub2_into(
                source_psqt,
                add0_psqt,
                add1_psqt,
                sub0_psqt,
                sub1_psqt,
                target_psqt,
            );
        }
        _ => {
            target_accumulation.copy_from_slice(source_accumulation);
            target_psqt.copy_from_slice(source_psqt);
            for &feature_index in &removed[..removed_len] {
                apply_feature_sub::<HALF_DIMS>(
                    feature_transformer,
                    feature_index,
                    target_accumulation,
                    target_psqt,
                );
            }
            for &feature_index in &added[..added_len] {
                apply_feature_add::<HALF_DIMS>(
                    feature_transformer,
                    feature_index,
                    target_accumulation,
                    target_psqt,
                );
            }
        }
    }
}

#[inline(always)]
pub(super) fn apply_feature_add<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    feature_index: usize,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let weight_row = feature_index * HALF_DIMS;
    let psqt_row = feature_index * PSQT_BUCKETS;

    accum_add(
        accumulation,
        &feature_transformer.weights[weight_row..weight_row + HALF_DIMS],
    );
    psqt_add(
        psqt,
        &feature_transformer.psqt_weights[psqt_row..psqt_row + PSQT_BUCKETS],
    );
}

#[inline(always)]
fn apply_feature_sub<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    feature_index: usize,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let weight_row = feature_index * HALF_DIMS;
    let psqt_row = feature_index * PSQT_BUCKETS;

    accum_sub(
        accumulation,
        &feature_transformer.weights[weight_row..weight_row + HALF_DIMS],
    );
    psqt_sub(
        psqt,
        &feature_transformer.psqt_weights[psqt_row..psqt_row + PSQT_BUCKETS],
    );
}
