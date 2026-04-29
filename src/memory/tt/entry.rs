use crate::Move;

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

#[derive(Clone, Copy, Default)]
pub struct PackedTTEntry {
    pub(crate) data: u64,
    pub(crate) signature: u64,
}

/// Exact score stored in the transposition table.
pub const EXACT: u8 = 0;
/// Lower-bound score stored in the transposition table.
pub const LOWER_BOUND: u8 = 1;
/// Upper-bound score stored in the transposition table.
pub const UPPER_BOUND: u8 = 2;
