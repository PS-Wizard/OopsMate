use types::moves::Move;

/// Bound type for the stored score
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bound {
    Exact,
    Upper,
    Lower,
}

// Entry in the transposition table
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,        // Full Zobrist hash for verification
    pub best_move: Move, // Best move found (or null move)
    pub score: i32,      // Evaluation score
    pub depth: u8,       // Search depth
    pub bound: Bound,    // Type of bound
    pub age: u8,         // Generation/age for replacement
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            key: 0,
            best_move: Move::NULL,
            score: 0,
            depth: 0,
            bound: Bound::Exact,
            age: 0,
        }
    }
}

pub struct TranspositionTable {
    table: Vec<TTEntry>,
    size: usize,
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        Self {
            table: vec![TTEntry::default(); num_entries],
            size: num_entries,
            age: 0,
        }
    }

    /// Get index from hash
    #[inline(always)]
    fn index(&self, hash: u64) -> usize {
        // Zobrist hash is a big normal, using modulo to normalize it to 0 and whatever the size is
        (hash as usize) % self.size
    }

    /// Query the table for a position
    #[inline(always)]
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.table[self.index(hash)];

        // Verify the key matches
        if entry.key == hash { Some(entry) } else { None }
    }

    pub fn store(&mut self, hash: u64, best_move: Move, score: i32, depth: u8, bound: Bound) {
        let idx = self.index(hash);
        let existing = &self.table[idx];

        // Replacement strategy: always replace if
        // 1. Slot is empty (key == 0)
        // 2. Same position (key matches)
        // 3. Entry is from older generation
        // 4. New search is deeper
        let should_replace = existing.key == 0
            || existing.key == hash
            || existing.age != self.age
            || depth >= existing.depth;

        if should_replace {
            self.table[idx] = TTEntry {
                key: hash,
                best_move,
                score,
                depth,
                bound,
                age: self.age,
            };
        }
    }

    /// Clear the table
    pub fn clear(&mut self) {
        self.table.fill(TTEntry::default());
        self.age = 0
    }

    /// Get table usage statistics
    pub fn usage(&self) -> f64 {
        let used = self.table.iter().filter(|e| e.key != 0).count();
        (used as f64 / self.size as f64) * 100.0
    }
}
