use crate::{types::Color, Move, Position};
use super::params::MAX_DEPTH;

const KILLERS_PER_PLY: usize = 2;
const MAX_HISTORY: i32 = 50_000; 

/// Killer move table
/// Stores the best quiet moves that caused beta cutoffs at each ply
pub struct KillerTable {
    /// killers[ply][slot] where slot 0 is primary, slot 1 is secondary
    killers: [[Move; KILLERS_PER_PLY]; MAX_DEPTH],
}

impl KillerTable {
    #[inline(always)]
    pub fn new() -> Self {
        KillerTable {
            killers: [[Move(0); KILLERS_PER_PLY]; MAX_DEPTH],
        }
    }

    /// Store a killer move at the given ply
    /// Maintains a sliding window: new killer becomes primary, old primary becomes secondary
    #[inline(always)]
    pub fn store(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_DEPTH {
            return;
        }

        let killers = &mut self.killers[ply];

        // Don't store if it's already the primary killer
        if killers[0].0 == mv.0 {
            return;
        }

        // Shift: secondary <- primary, primary <- new
        killers[1] = killers[0];
        killers[0] = mv;
    }

    /// Check if a move is a killer at the given ply
    #[inline(always)]
    pub fn is_killer(&self, ply: usize, mv: Move) -> bool {
        if ply >= MAX_DEPTH {
            return false;
        }

        let killers = &self.killers[ply];
        killers[0].0 == mv.0 || killers[1].0 == mv.0
    }

    /// Get the primary killer move for a ply (for use as hint in move ordering)
    #[inline(always)]
    pub fn get_primary(&self, ply: usize) -> Option<Move> {
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

    /// Get the secondary killer move for a ply
    #[inline(always)]
    pub fn get_secondary(&self, ply: usize) -> Option<Move> {
        if ply >= MAX_DEPTH {
            return None;
        }

        let mv = self.killers[ply][1];
        if mv.0 == 0 {
            None
        } else {
            Some(mv)
        }
    }

    /// Clear all killer moves (call at start of new search)
    #[inline(always)]
    pub fn clear(&mut self) {
        for ply in 0..MAX_DEPTH {
            self.killers[ply] = [Move(0); KILLERS_PER_PLY];
        }
    }
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

/// History Heuristic Table
/// Stores scores for quiet moves that caused beta cutoffs
pub struct HistoryTable {
    // [color][from][to]
    table: [[[i32; 64]; 64]; 2],
}

impl HistoryTable {
    #[inline(always)]
    pub fn new() -> Self {
        HistoryTable {
            table: [[[0; 64]; 64]; 2],
        }
    }

    #[inline(always)]
    pub fn update(&mut self, color: Color, from: usize, to: usize, bonus: i16) {
        let entry = &mut self.table[color as usize][from][to];
        
        // Simple saturation (FAST)
        // Ensure we stay within bounds without expensive division
        *entry = (*entry + bonus as i32).clamp(-MAX_HISTORY, MAX_HISTORY);
    }

    #[inline(always)]
    pub fn get(&self, color: Color, from: usize, to: usize) -> i32 {
        self.table[color as usize][from][to]
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.table = [[[0; 64]; 64]; 2];
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Container for all move history heuristics
pub struct MoveHistory {
    pub killers: KillerTable,
    pub history: HistoryTable,
}

impl MoveHistory {
    pub fn new() -> Self {
        MoveHistory {
            killers: KillerTable::new(),
            history: HistoryTable::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.killers.clear();
        self.history.clear();
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}

// Move ordering priority scores
pub const SCORE_TT_MOVE: i32 = 1_000_000;
pub const SCORE_GOOD_CAPTURE: i32 = 100_000; // SEE >= 0
pub const SCORE_PROMOTION: i32 = 90_000; // Quiet promotion
pub const SCORE_KILLER_PRIMARY: i32 = 20_000;
pub const SCORE_KILLER_SECONDARY: i32 = 15_000;
pub const SCORE_BAD_CAPTURE: i32 = 5_000; // SEE < 0 (Still better than random quiet moves?)

/// Score a move for ordering in main search
#[inline(always)]
pub fn score_move(
    m: Move,
    pos: &Position,
    tt_move: Option<Move>,
    history: Option<&MoveHistory>,
    ply: usize,
) -> i32 {
    // TT move gets highest priority
    if let Some(tt_mv) = tt_move {
        if m.0 == tt_mv.0 {
            return SCORE_TT_MOVE;
        }
    }

    // Captures & Promotions (Resolved via SEE)
    if m.is_capture() {
        let see_score = pos.see(&m);

        if see_score >= 0 {
            // Good capture: Prioritize by SEE score (Winning Queen > Winning Pawn)
            return SCORE_GOOD_CAPTURE + see_score;
        } else {
            // Bad capture: Lose material.
            return SCORE_BAD_CAPTURE + see_score;
        }
    }

    // 3. Quiet Promotions
    if m.is_promotion() {
        return SCORE_PROMOTION;
    }

    if let Some(h) = history {
        // 4. Killer moves
        if h.killers.is_killer(ply, m) {
            return if Some(m) == h.killers.get_primary(ply) {
                SCORE_KILLER_PRIMARY
            } else {
                SCORE_KILLER_SECONDARY
            };
        }
        
        // 5. History Heuristic
        return h.history.get(pos.side_to_move, m.from(), m.to());
    }

    0
}

#[inline(always)]
pub fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
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
