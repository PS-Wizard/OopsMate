use crate::eval::EvalProvider;
use crate::search::qsearch::qsearch;
use crate::search::SearchStats;
use crate::Position;

#[rustfmt::skip]
const RAZOR_MARGINS: [i32; 4] = [
    0,
    300,
    400,
    500,
];

#[inline(always)]
pub fn try_razoring<E: EvalProvider>(
    pos: &mut Position,
    eval: &E,
    eval_state: &mut E::State,
    depth: u8,
    alpha: i32,
    in_check: bool,
    pv_node: bool,
    static_eval: i32,
    stats: &mut SearchStats,
) -> Option<i32> {
    if depth == 0 || depth > 3 {
        return None;
    }

    if in_check || pv_node {
        return None;
    }

    const MATE_BOUND: i32 = 40_000;
    if alpha.abs() > MATE_BOUND {
        return None;
    }

    let margin = RAZOR_MARGINS[depth as usize];

    if static_eval + margin < alpha {
        let razor_score = qsearch(
            pos,
            eval,
            eval_state,
            alpha - margin,
            alpha - margin + 1,
            stats,
            0,
        );

        if razor_score < alpha - margin {
            return Some(razor_score);
        }
    }

    None
}
