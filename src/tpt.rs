use crate::Move;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

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

struct PackedTTEntry {
    data: AtomicU64,
    signature: AtomicU64,
}

impl PackedTTEntry {
    fn default() -> Self {
        PackedTTEntry {
            data: AtomicU64::new(0),
            signature: AtomicU64::new(0),
        }
    }
}

pub struct TranspositionTable {
    table: Vec<PackedTTEntry>,
    mask: u64, // For fast indexing with bitwise AND
    generation: AtomicU8,
}

impl TranspositionTable {
    pub fn new_mb(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        // Mimic old entry size (24 bytes) to keep table size identical for regression testing
        let entry_size = 24; 
        let entries = bytes / entry_size;

        // Round down to power of 2 for fast indexing
        let size = if entries.is_power_of_two() {
            entries
        } else {
            entries.next_power_of_two() / 2
        };

        let mut table = Vec::with_capacity(size);
        for _ in 0..size {
            table.push(PackedTTEntry::default());
        }

        TranspositionTable {
            table,
            mask: (size - 1) as u64,
            generation: AtomicU8::new(0),
        }
    }

    /// Increment generation (call this at the start of each new search)
    #[inline(always)]
    pub fn new_search(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = (hash & self.mask) as usize;

        // Prefetch the cache line
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::_mm_prefetch;
            let ptr = self.table.as_ptr().add(idx) as *const i8;
            _mm_prefetch::<3>(ptr); // _MM_HINT_T0
        }

        let entry = unsafe { self.table.get_unchecked(idx) };
        
        let data = entry.data.load(Ordering::Relaxed);
        let signature = entry.signature.load(Ordering::Relaxed);

        if (data ^ signature) == hash {
            // Unpack
            let score = (data as u32) as i32; // Sign extension relies on cast
            let best_move = Move(((data >> 32) & 0xFFFF) as u16);
            let depth = ((data >> 48) & 0xFF) as u8;
            let flag = ((data >> 56) & 0x3) as u8;
            let age = ((data >> 58) & 0x3F) as u8;

            Some(TTEntry {
                key: hash,
                best_move,
                score,
                depth,
                flag,
                age,
            })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn store(&self, hash: u64, best_move: Move, score: i32, depth: u8, flag: u8) {
        let idx = (hash & self.mask) as usize;
        let entry = unsafe { self.table.get_unchecked(idx) };

        let old_data = entry.data.load(Ordering::Relaxed);
        let old_signature = entry.signature.load(Ordering::Relaxed);
        let old_hash = old_data ^ old_signature;
        
        let mut replace = false;
        let gen = self.generation.load(Ordering::Relaxed);
        
        if old_hash == 0 {
            // Empty
            replace = true;
        } else if old_hash == hash {
            // Same position
            // The previous logic was `entry.key == hash`. It always replaced if key matched!
            replace = true; 
        } else {
            // Collision
            // Replace if old is from previous generation (age mismatch)
            // AND new depth >= old depth? 
            // Previous logic: `(entry.age != self.generation && depth >= entry.depth)`
            
            let old_age = ((old_data >> 58) & 0x3F) as u8;
            let current_age_bits = gen & 0x3F;
            
            if old_age != current_age_bits && depth >= ((old_data >> 48) & 0xFF) as u8 {
                replace = true;
            }
        }

        if replace {
            // Pack
            // Score: 32 bits (0-31)
            // Move: 16 bits (32-47)
            // Depth: 8 bits (48-55)
            // Flag: 2 bits (56-57)
            // Age: 6 bits (58-63)

            let score_bits = (score as u32) as u64;
            let move_bits = (best_move.0 as u64) << 32;
            let depth_bits = (depth as u64) << 48;
            let flag_bits = (flag as u64) << 56;
            let age_bits = ((gen & 0x3F) as u64) << 58;

            let new_data = score_bits | move_bits | depth_bits | flag_bits | age_bits;
            let new_signature = hash ^ new_data;

            // Store using Relaxed ordering
            entry.data.store(new_data, Ordering::Relaxed);
            entry.signature.store(new_signature, Ordering::Relaxed);
        }
    }

    pub fn clear(&self) {
        for entry in &self.table {
            entry.data.store(0, Ordering::Relaxed);
            entry.signature.store(0, Ordering::Relaxed);
        }
        self.generation.store(0, Ordering::Relaxed);
    }

    /// Calculate hashfull (permill - parts per thousand)
    pub fn hashfull(&self) -> usize {
        let sample_size = 1000.min(self.table.len());
        let mut filled = 0;

        for i in 0..sample_size {
            let entry = unsafe { self.table.get_unchecked(i) };
            let data = entry.data.load(Ordering::Relaxed);
            let signature = entry.signature.load(Ordering::Relaxed);
            
            if (data ^ signature) != 0 {
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
