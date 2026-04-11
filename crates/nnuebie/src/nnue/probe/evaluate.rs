use super::NNUEProbe;
use crate::architecture::{PAWN_VALUE, PSQT_BUCKET_COUNT};
use crate::types::Color;

impl NNUEProbe<'_> {
    /// Evaluates the current position from the side-to-move perspective.
    pub fn evaluate(&mut self, side_to_move: Color) -> i32 {
        let stm = side_to_move.index();
        let simple_eval = PAWN_VALUE * (self.pawn_count[stm] - self.pawn_count[1 - stm])
            + (self.non_pawn_material[stm] - self.non_pawn_material[1 - stm]);
        let use_small = simple_eval.abs() > 962;

        let bucket = if self.piece_count > 0 {
            (self.piece_count - 1) / 4
        } else {
            0
        }
        .min(PSQT_BUCKET_COUNT - 1);

        let latest = self.accumulator_stack.latest();
        let networks = self.networks.as_ref();

        let (mut nnue_val, psqt_val, positional_val) = if use_small {
            let (psqt, pos) = networks.small_net.evaluate(
                &latest.acc_small,
                bucket,
                stm,
                &mut self.scratch_small,
            );
            let mut score = (125 * psqt + 131 * pos) / 128;

            if score.abs() < 236 {
                let (big_psqt, big_pos) =
                    networks
                        .big_net
                        .evaluate(&latest.acc_big, bucket, stm, &mut self.scratch_big);
                score = (125 * big_psqt + 131 * big_pos) / 128;
                (score, big_psqt, big_pos)
            } else {
                (score, psqt, pos)
            }
        } else {
            let (psqt, pos) =
                networks
                    .big_net
                    .evaluate(&latest.acc_big, bucket, stm, &mut self.scratch_big);
            ((125 * psqt + 131 * pos) / 128, psqt, pos)
        };

        let nnue_complexity = (psqt_val - positional_val).abs();
        nnue_val -= nnue_val * nnue_complexity / 18000;

        let material = 535 * (self.pawn_count[0] + self.pawn_count[1])
            + (self.non_pawn_material[0] + self.non_pawn_material[1]);
        let mut score = nnue_val * (77777 + material) / 77777;

        score -= score * latest.rule50 / 212;
        score.clamp(-31753, 31753)
    }
}
