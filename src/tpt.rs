use crate::Move;

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,
    pub best_move: Move,
    pub score: i32,
    pub depth: u8,
    pub flag: u8, // 0=EXACT, 1=LOWER, 2=UPPER
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            best_move: Move(0),
            score: 0,
            depth: 0,
            flag: 0,
        }
    }
}

pub struct TranspositionTable {
    table: Vec<TTEntry>,
    size: usize,
}

impl TranspositionTable {
    pub fn new_mb(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();
        let size = bytes / entry_size;

        TranspositionTable {
            table: vec![TTEntry::default(); size],
            size,
        }
    }

    #[inline(always)]
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.table[(hash as usize) % self.size];
        if entry.key == hash {
            Some(entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn store(&mut self, hash: u64, best_move: Move, score: i32, depth: u8, flag: u8) {
        let idx = (hash as usize) % self.size;
        let entry = &mut self.table[idx];

        // Always replace (simple scheme)
        // Later you can add depth-preferred replacement
        entry.key = hash;
        entry.best_move = best_move;
        entry.score = score;
        entry.depth = depth;
        entry.flag = flag;
    }

    pub fn clear(&mut self) {
        for entry in &mut self.table {
            *entry = TTEntry::default();
        }
    }
}

// TT flags
pub const EXACT: u8 = 0;
pub const LOWER_BOUND: u8 = 1; // Beta cutoff
pub const UPPER_BOUND: u8 = 2; // Alpha cutoff
