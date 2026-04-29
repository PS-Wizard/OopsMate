use std::mem::size_of;

use super::entry::{Bound, TtEntry, TtHit};
use crate::Move;

const MIB_BYTES: usize = 1024 * 1024;
const HASHFULL_SAMPLE_ENTRIES: usize = 1000;

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug)]
pub(crate) struct Bucket {
    entries: [TtEntry; 4],
}

impl Bucket {
    pub(crate) const EMPTY: Self = Self {
        entries: [TtEntry::EMPTY; 4],
    };
}

#[derive(Debug)]
pub struct TranspositionTable {
    buckets: Box<[Bucket]>,
    bucket_mask: usize,
    generation: u8,
}

impl TranspositionTable {
    #[must_use]
    #[inline(always)]
    pub fn new(mebibytes: usize) -> Self {
        let buckets = allocate_buckets(mebibytes);
        let bucket_mask = buckets.len() - 1;

        Self {
            buckets,
            bucket_mask,
            generation: 0,
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn new_mb(mebibytes: usize) -> Self {
        Self::new(mebibytes)
    }

    #[inline(always)]
    pub fn resize(&mut self, mebibytes: usize) {
        self.buckets = allocate_buckets(mebibytes);
        self.bucket_mask = self.buckets.len() - 1;
        self.generation = 0;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.buckets.fill(Bucket::EMPTY);
        self.generation = 0;
    }

    #[inline(always)]
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    #[must_use]
    #[inline(always)]
    pub fn probe(&self, hash: u64, ply: u8) -> Option<TtHit> {
        let key32 = (hash >> 32) as u32;
        let bucket = &self.buckets[self.bucket_index(hash)];

        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::_mm_prefetch;
            _mm_prefetch::<3>(bucket as *const Bucket as *const i8);
        }

        for entry in &bucket.entries {
            if entry.matches_key(key32) {
                return Some(entry.to_hit(ply));
            }
        }

        None
    }

    #[inline(always)]
    pub fn store(
        &mut self,
        hash: u64,
        ply: u8,
        best_move: Move,
        score: i32,
        static_eval: i16,
        depth: u8,
        bound: Bound,
    ) {
        let key32 = (hash >> 32) as u32;
        let generation = self.generation;
        let index = self.bucket_index(hash);
        let bucket = &mut self.buckets[index];

        for entry in &mut bucket.entries {
            if entry.matches_key(key32) {
                *entry = TtEntry::from_values(
                    key32,
                    best_move,
                    score,
                    static_eval,
                    depth,
                    generation,
                    bound,
                    ply,
                );
                return;
            }
        }

        for entry in &mut bucket.entries {
            if entry.is_empty() {
                *entry = TtEntry::from_values(
                    key32,
                    best_move,
                    score,
                    static_eval,
                    depth,
                    generation,
                    bound,
                    ply,
                );
                return;
            }
        }

        let mut victim_index = 0usize;
        let mut victim_value = i16::MAX;
        for (index, entry) in bucket.entries.iter().copied().enumerate() {
            let value = entry.replacement_value(generation);
            if value < victim_value {
                victim_value = value;
                victim_index = index;
            }
        }

        bucket.entries[victim_index] = TtEntry::from_values(
            key32,
            best_move,
            score,
            static_eval,
            depth,
            generation,
            bound,
            ply,
        );
    }

    #[must_use]
    #[inline(always)]
    pub fn size_mib(&self) -> usize {
        self.buckets.len() * size_of::<Bucket>() / MIB_BYTES
    }

    #[must_use]
    #[inline(always)]
    pub fn hashfull(&self) -> usize {
        let sample_buckets = (HASHFULL_SAMPLE_ENTRIES / 4).min(self.buckets.len());
        let mut used = 0usize;
        let mut total = 0usize;

        for bucket in &self.buckets[..sample_buckets] {
            for entry in &bucket.entries {
                total += 1;
                if !entry.is_empty() {
                    used += 1;
                }
            }
        }

        if total == 0 {
            return 0;
        }

        (used * 1000) / total
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    #[inline(always)]
    fn bucket_index(&self, hash: u64) -> usize {
        hash as usize & self.bucket_mask
    }
}

fn allocate_buckets(mebibytes: usize) -> Box<[Bucket]> {
    let requested_bytes = mebibytes.saturating_mul(MIB_BYTES);
    let raw_bucket_count = requested_bytes / size_of::<Bucket>();
    let bucket_count = floor_power_of_two(raw_bucket_count);
    vec![Bucket::EMPTY; bucket_count].into_boxed_slice()
}

#[inline(always)]
fn floor_power_of_two(value: usize) -> usize {
    if value <= 1 {
        return 1;
    }

    1usize << (usize::BITS - 1 - value.leading_zeros())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_mb_still_allocates_one_bucket() {
        let tt = TranspositionTable::new(0);
        assert_eq!(tt.bucket_count(), 1);
    }

    #[test]
    fn hashfull_is_zero_for_empty_table() {
        let tt = TranspositionTable::new(1);
        assert_eq!(tt.hashfull(), 0);
    }

    #[test]
    fn static_eval_sentinel_roundtrips() {
        let mut tt = TranspositionTable::new(1);
        tt.store(
            0x1234_5678_9abc_def0,
            0,
            Move(1),
            42,
            super::super::NO_STATIC_EVAL,
            5,
            Bound::Exact,
        );
        let hit = tt.probe(0x1234_5678_9abc_def0, 0).unwrap();
        assert_eq!(hit.static_eval, super::super::NO_STATIC_EVAL);
    }
}
