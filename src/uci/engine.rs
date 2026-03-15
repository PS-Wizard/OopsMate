use crate::{tpt::TranspositionTable, Position};
use std::sync::Arc;

/// UCI-facing engine state.
pub struct UciEngine {
    pub(crate) position: Position,
    pub(crate) tt: Arc<TranspositionTable>,
    pub(crate) threads: usize,
}

impl UciEngine {
    /// Creates a new engine initialized to the standard starting position.
    pub fn new() -> Self {
        UciEngine {
            position: Position::new(),
            tt: Arc::new(TranspositionTable::new_mb(256)),
            threads: 4,
        }
    }
}

impl Default for UciEngine {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
