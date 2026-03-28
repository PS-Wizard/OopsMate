//! Transposition table storage.
//!
//! The table stores one packed entry per bucket and uses a simple depth-and-age
//! replacement policy tuned for search throughput.

use crate::Move;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

/// Decoded transposition-table entry returned by probes.
#[derive(Copy, Clone)]
pub struct TTEntry {
    /// Full position hash used for validation.
    pub key: u64,
    /// Best move stored for the position.
    pub best_move: Move,
    /// Stored score after TT normalization.
    pub score: i32,
    /// Search depth associated with the entry.
    pub depth: u8,
    /// Bound type for the stored score.
    pub flag: u8,
    /// Table generation used for aging decisions.
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

/// Lock-free transposition table with one packed entry per bucket.
pub struct TranspositionTable {
    table: Vec<PackedTTEntry>,
    mask: u64,
    generation: AtomicU8,
}

const ENTRY_SIZE_BYTES: usize = 24;
const FLAG_SHIFT: u64 = 56;
const AGE_SHIFT: u64 = 58;
const DEPTH_SHIFT: u64 = 48;

#[inline(always)]
fn unpack_entry(hash: u64, data: u64, signature: u64) -> Option<TTEntry> {
    if (data ^ signature) != hash {
        return None;
    }

    Some(TTEntry {
        key: hash,
        best_move: Move(((data >> 32) & 0xFFFF) as u16),
        score: (data as u32) as i32,
        depth: ((data >> DEPTH_SHIFT) & 0xFF) as u8,
        flag: ((data >> FLAG_SHIFT) & 0x3) as u8,
        age: ((data >> AGE_SHIFT) & 0x3F) as u8,
    })
}

#[inline(always)]
fn pack_entry(best_move: Move, score: i32, depth: u8, flag: u8, age: u8) -> u64 {
    (score as u32) as u64
        | ((best_move.0 as u64) << 32)
        | ((depth as u64) << DEPTH_SHIFT)
        | ((flag as u64) << FLAG_SHIFT)
        | ((age as u64) << AGE_SHIFT)
}

impl TranspositionTable {
    /// Allocates a transposition table sized in megabytes.
    pub fn new_mb(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        let entries = bytes / ENTRY_SIZE_BYTES;

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

    /// Starts a new search generation for aging decisions.
    #[inline(always)]
    pub fn new_search(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Looks up a position by hash.
    #[inline(always)]
    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = (hash & self.mask) as usize;

        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::_mm_prefetch;
            let ptr = self.table.as_ptr().add(idx) as *const i8;
            _mm_prefetch::<3>(ptr);
        }

        let entry = unsafe { self.table.get_unchecked(idx) };
        let data = entry.data.load(Ordering::Relaxed);
        let signature = entry.signature.load(Ordering::Relaxed);

        unpack_entry(hash, data, signature)
    }

    /// Stores a search result for `hash` if the replacement policy allows it.
    #[inline(always)]
    pub fn store(&self, hash: u64, best_move: Move, score: i32, depth: u8, flag: u8) {
        let idx = (hash & self.mask) as usize;
        let entry = unsafe { self.table.get_unchecked(idx) };

        let old_data = entry.data.load(Ordering::Relaxed);
        let old_signature = entry.signature.load(Ordering::Relaxed);
        let old_hash = old_data ^ old_signature;

        let mut replace = false;
        let gen = self.generation.load(Ordering::Relaxed);

        if old_hash == 0 || old_hash == hash {
            replace = true;
        } else {
            let old_age = ((old_data >> AGE_SHIFT) & 0x3F) as u8;
            let current_age_bits = gen & 0x3F;

            if old_age != current_age_bits && depth >= ((old_data >> DEPTH_SHIFT) & 0xFF) as u8 {
                replace = true;
            }
        }

        if replace {
            let new_data = pack_entry(best_move, score, depth, flag, gen & 0x3F);
            let new_signature = hash ^ new_data;

            entry.data.store(new_data, Ordering::Relaxed);
            entry.signature.store(new_signature, Ordering::Relaxed);
        }
    }

    /// Clears all entries and resets the generation counter.
    pub fn clear(&self) {
        for entry in &self.table {
            entry.data.store(0, Ordering::Relaxed);
            entry.signature.store(0, Ordering::Relaxed);
        }
        self.generation.store(0, Ordering::Relaxed);
    }

    /// Returns hash table occupancy in permille, matching the UCI `hashfull` convention.
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
/// Exact score stored in the transposition table.
pub const EXACT: u8 = 0;
/// Lower-bound score stored in the transposition table.
pub const LOWER_BOUND: u8 = 1;
/// Upper-bound score stored in the transposition table.
pub const UPPER_BOUND: u8 = 2;
