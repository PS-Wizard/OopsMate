use crate::search::params::MAX_DEPTH;
use crate::{types::Color, Move};

const KILLERS_PER_PLY: usize = 2;
const MAX_HISTORY: i32 = 50_000;

pub struct KillerTable {
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
        if mv.0 == 0 { None } else { Some(mv) }
    }
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HistoryTable {
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

pub struct MoveHistory {
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

