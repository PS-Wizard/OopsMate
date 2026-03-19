use super::params::MAX_DEPTH;
use crate::{search::features, types::Color, Move, Piece, Position};

const KILLERS_PER_PLY: usize = 2;
const MAX_HISTORY: i32 = 50_000;

pub(crate) struct KillerTable {
    killers: [[Move; KILLERS_PER_PLY]; MAX_DEPTH],
}

impl KillerTable {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            killers: [[Move(0); KILLERS_PER_PLY]; MAX_DEPTH],
        }
    }

    #[inline(always)]
    pub(crate) fn store(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_DEPTH {
            return;
        }

        let killers = &mut self.killers[ply];

        if killers[0].0 == mv.0 {
            return;
        }

        killers[1] = killers[0];
        killers[0] = mv;
    }

    #[inline(always)]
    pub(crate) fn is_killer(&self, ply: usize, mv: Move) -> bool {
        if ply >= MAX_DEPTH {
            return false;
        }

        let killers = &self.killers[ply];
        killers[0].0 == mv.0 || killers[1].0 == mv.0
    }

    #[inline(always)]
    pub(crate) fn get_primary(&self, ply: usize) -> Option<Move> {
        if ply >= MAX_DEPTH {
            return None;
        }

        let mv = self.killers[ply][0];
        if mv.0 == 0 {
            None
        } else {
            Some(mv)
        }
    }
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct HistoryTable {
    table: [[[i32; 64]; 64]; 2],
}

impl HistoryTable {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }

    #[inline(always)]
    pub(crate) fn update(&mut self, color: Color, from: usize, to: usize, bonus: i16) {
        let entry = &mut self.table[color as usize][from][to];
        *entry = (*entry + bonus as i32).clamp(-MAX_HISTORY, MAX_HISTORY);
    }

    #[inline(always)]
    pub(crate) fn get(&self, color: Color, from: usize, to: usize) -> i32 {
        self.table[color as usize][from][to]
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct MoveHistory {
    pub(crate) killers: KillerTable,
    pub(crate) history: HistoryTable,
}

impl MoveHistory {
    pub(crate) fn new() -> Self {
        Self {
            killers: KillerTable::new(),
            history: HistoryTable::new(),
        }
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}

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

#[inline(always)]
pub(crate) fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
    if index >= moves.len() {
        return;
    }

    let mut best_idx = index;
    let mut best_score = unsafe { *scores.get_unchecked(index) };

    for i in (index + 1)..moves.len() {
        let score = unsafe { *scores.get_unchecked(i) };
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    if best_idx != index {
        moves.swap(index, best_idx);
        scores.swap(index, best_idx);
    }
}
