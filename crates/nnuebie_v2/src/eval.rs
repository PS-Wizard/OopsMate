pub(crate) mod accumulator;
pub(crate) mod pipeline;
pub(crate) mod refresh;
pub(crate) mod score;

#[cfg(test)]
mod tests;

use self::pipeline::{evaluate_big_network, evaluate_small_network};
use self::refresh::{refresh_frame_big, refresh_frame_small};
use self::score::{blended_nnue, simple_eval, to_centipawns, total_material_for_scaling};
use crate::constants::{
    BIG_NET_RECHECK_THRESHOLD, PSQT_BUCKETS, SMALLNET_SIMPLE_EVAL_THRESHOLD,
    VALUE_TB_LOSS_IN_MAX_PLY, VALUE_TB_WIN_IN_MAX_PLY,
};
use crate::context::NnueContext;
use crate::network::NnueNetworks;
use crate::oopsmate_core::{Color, Position};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvalOutput {
    pub psqt: i32,
    pub positional: i32,
    pub final_raw: i32,
    pub final_cp: i32,
    pub used_smallnet: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RawEvalOutput {
    psqt: i32,
    positional: i32,
    final_raw: i32,
    used_smallnet: bool,
}

impl EvalOutput {
    pub const ZERO: Self = Self {
        psqt: 0,
        positional: 0,
        final_raw: 0,
        final_cp: 0,
        used_smallnet: false,
    };

    #[inline(always)]
    #[must_use]
    pub const fn white_side_cp(self, side_to_move: Color) -> i32 {
        if matches!(side_to_move, Color::Black) {
            -self.final_cp
        } else {
            self.final_cp
        }
    }
}

impl RawEvalOutput {
    const ZERO: Self = Self {
        psqt: 0,
        positional: 0,
        final_raw: 0,
        used_smallnet: false,
    };
}

impl NnueNetworks {
    pub fn reset_context(&self, position: &Position, ctx: &mut NnueContext) {
        ctx.finny.prepare(
            &self.big.feature_transformer,
            &self.small.feature_transformer,
        );
        ctx.reset_root_state(position);
        refresh_frame_big(&self.big, position, ctx, 0);
        refresh_frame_small(&self.small, position, ctx, 0);
    }

    #[inline(always)]
    fn evaluate_raw_output(&self, position: &Position, ctx: &mut NnueContext) -> RawEvalOutput {
        let inputs = self.position_inputs(position);
        if inputs.piece_count == 0 {
            return RawEvalOutput::ZERO;
        }

        if !ctx.initialized {
            debug_assert!(
                ctx.depth == 0,
                "NNUE context is uninitialized while inside the move stack"
            );
            self.reset_context(position, ctx);
        } else if ctx.depth == 0 && ctx.root_hash != position.hash() {
            self.reset_context(position, ctx);
        }

        let bucket = ((usize::from(inputs.piece_count) - 1) / 4).min(PSQT_BUCKETS - 1);
        let mut used_smallnet =
            simple_eval(position, inputs.side_to_move).abs() > SMALLNET_SIMPLE_EVAL_THRESHOLD;

        let (mut psqt, mut positional) = if used_smallnet {
            evaluate_small_network(&self.small, position, ctx, inputs.side_to_move, bucket)
        } else {
            evaluate_big_network(&self.big, position, ctx, inputs.side_to_move, bucket)
        };

        let mut nnue = blended_nnue(psqt, positional);

        if used_smallnet && nnue.abs() < BIG_NET_RECHECK_THRESHOLD {
            (psqt, positional) =
                evaluate_big_network(&self.big, position, ctx, inputs.side_to_move, bucket);
            nnue = blended_nnue(psqt, positional);
            used_smallnet = false;
        }

        let complexity = (psqt - positional).abs();
        nnue -= nnue * complexity / 18_000;

        let material = total_material_for_scaling(position);
        let mut final_raw = nnue * (77_777 + material) / 77_777;
        final_raw -= final_raw * i32::from(inputs.rule50) / 212;
        final_raw = final_raw.clamp(VALUE_TB_LOSS_IN_MAX_PLY + 1, VALUE_TB_WIN_IN_MAX_PLY - 1);

        RawEvalOutput {
            psqt,
            positional,
            final_raw,
            used_smallnet,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn evaluate_raw(&self, position: &Position, ctx: &mut NnueContext) -> i32 {
        self.evaluate_raw_output(position, ctx).final_raw
    }

    #[inline(always)]
    #[must_use]
    pub fn raw_to_cp(&self, value: i32, position: &Position) -> i32 {
        to_centipawns(value, position)
    }

    #[must_use]
    pub fn evaluate(&self, position: &Position, ctx: &mut NnueContext) -> EvalOutput {
        let raw = self.evaluate_raw_output(position, ctx);

        EvalOutput {
            psqt: raw.psqt,
            positional: raw.positional,
            final_raw: raw.final_raw,
            final_cp: self.raw_to_cp(raw.final_raw, position),
            used_smallnet: raw.used_smallnet,
        }
    }
}
