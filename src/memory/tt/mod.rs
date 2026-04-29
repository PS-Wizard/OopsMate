mod entry;
mod table;

pub use entry::{PackedTTEntry, TTEntry, EXACT, LOWER_BOUND, UPPER_BOUND};
pub use table::TranspositionTable;
