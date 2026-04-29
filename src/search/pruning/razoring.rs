use crate::eval::EvalProvider;
use crate::search::context::SearchContext;
use crate::search::features;
use crate::search::qsearch::qsearch;
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
    ctx: &mut SearchContext<'_, E>,
    depth: u8,
    alpha: i32,
    in_check: bool,
    pv_node: bool,
    static_eval: i32,
) -> Option<i32> {
    // razoring: drop to qsearch early when a shallow node already looks hopeless versus alpha.
    if !features::RAZORING || depth == 0 || depth > 3 || in_check || pv_node {
        return None;
    }

    const MATE_BOUND: i32 = 40_000;
    if alpha.abs() > MATE_BOUND {
        return None;
    }

    let margin = RAZOR_MARGINS[depth as usize];

    if static_eval + margin < alpha {
        let razor_score = qsearch(pos, ctx, alpha - margin, alpha - margin + 1, 0);

        if razor_score < alpha - margin {
            return Some(razor_score);
        }
    }

    None
}
