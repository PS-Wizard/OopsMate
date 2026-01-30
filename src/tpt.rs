use crate::Move;

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,
    pub best_move: Move,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
    pub age: u8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            best_move: Move(0),
            score: 0,
            depth: 0,
            flag: 0,
            age: 0,
        }
    }
}

pub struct TranspositionTable {
    table: Vec<TTEntry>,
    mask: u64, // For fast indexing with bitwise AND
    generation: u8,
}

impl TranspositionTable {
    pub fn new_mb(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();
        let entries = bytes / entry_size;

        // Round down to power of 2 for fast indexing
        let size = if entries.is_power_of_two() {
            entries
        } else {
            entries.next_power_of_two() / 2
        };

        TranspositionTable {
            table: vec![TTEntry::default(); size],
            mask: (size - 1) as u64,
            generation: 0,
        }
    }

    /// Increment generation (call this at the start of each new search)
    #[inline(always)]
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    #[inline(always)]
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let entry = unsafe { self.table.get_unchecked((hash & self.mask) as usize) };
        if entry.key == hash {
            Some(entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn store(&mut self, hash: u64, best_move: Move, score: i32, depth: u8, flag: u8) {
        let idx = (hash & self.mask) as usize;
        let entry = unsafe { self.table.get_unchecked_mut(idx) };

        // Depth-preferred replacement with aging
        let replace = entry.key == 0
            || entry.key == hash
            || entry.age != self.generation
            || depth >= entry.depth.saturating_add(3)
            || (depth >= entry.depth && entry.age == self.generation);

        if replace {
            entry.key = hash;
            entry.best_move = best_move;
            entry.score = score;
            entry.depth = depth;
            entry.flag = flag;
            entry.age = self.generation;
        }
    }

    pub fn clear(&mut self) {
        for entry in &mut self.table {
            *entry = TTEntry::default();
        }
        self.generation = 0;
    }

    /// Calculate hashfull (permill - parts per thousand)
    pub fn hashfull(&self) -> usize {
        let sample_size = 1000.min(self.table.len());
        let mut filled = 0;

        for i in 0..sample_size {
            if unsafe { self.table.get_unchecked(i).key } != 0 {
                filled += 1;
            }
        }

        (filled * 1000) / sample_size
    }
}

// TT flags
pub const EXACT: u8 = 0;
pub const LOWER_BOUND: u8 = 1;
pub const UPPER_BOUND: u8 = 2;
