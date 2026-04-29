use crate::{memory::MoveHistory, search::features, Move, Piece, Position};

pub(crate) const SCORE_TT_MOVE: i32 = 1_000_000;
const SCORE_GOOD_CAPTURE: i32 = 100_000;
pub(crate) const SCORE_PROMOTION: i32 = 90_000;
const SCORE_KILLER_PRIMARY: i32 = 20_000;
const SCORE_KILLER_SECONDARY: i32 = 15_000;
const SCORE_BAD_CAPTURE: i32 = 5_000;
const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20_000];

#[inline(always)]
pub(crate) const fn score_capture_from_see(see_score: i32) -> i32 {
    if see_score >= 0 {
        SCORE_GOOD_CAPTURE + see_score
    } else {
        SCORE_BAD_CAPTURE + see_score
    }
}

#[inline(always)]
fn piece_value(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize]
}

#[inline(always)]
fn score_capture_from_mvv_lva(m: Move, pos: &Position) -> i32 {
    let victim = pos.piece_at(m.to()).map(|(piece, _)| piece_value(piece));
    let attacker = pos.piece_at(m.from()).map(|(piece, _)| piece_value(piece));

    match (victim, attacker) {
        (Some(victim), Some(attacker)) => victim * 10 - attacker,
        _ => 0,
    }
}

#[inline(always)]
pub(crate) fn score_capture(m: Move, pos: &Position) -> i32 {
    // see capture ordering: prefer captures that win material after the full exchange sequence.
    if features::SEE {
        score_capture_from_see(pos.see(&m))
    } else {
        score_capture_from_mvv_lva(m, pos)
    }
}

#[inline(always)]
pub(crate) fn score_move(
    m: Move,
    pos: &Position,
    tt_move: Option<Move>,
    history: Option<&MoveHistory>,
    ply: usize,
) -> i32 {
    // move ordering: tt move first, then captures/promotions, then killer and history quiets.
    if features::TT_MOVE_ORDERING {
        if let Some(tt_mv) = tt_move {
            if m.0 == tt_mv.0 {
                return SCORE_TT_MOVE;
            }
        }
    }

    if m.is_capture() {
        return score_capture(m, pos);
    }

    if m.is_promotion() {
        return SCORE_PROMOTION;
    }

    if let Some(h) = history {
        if features::KILLER_MOVES && h.killers.is_killer(ply, m) {
            return if Some(m) == h.killers.get_primary(ply) {
                SCORE_KILLER_PRIMARY
            } else {
                SCORE_KILLER_SECONDARY
            };
        }

        if features::HISTORY_HEURISTIC {
            return h.history.get(pos.side_to_move, m.from(), m.to());
        }
    }

    0
}
