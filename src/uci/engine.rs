use crate::{tpt::TranspositionTable, Position};
use std::sync::Arc;

pub struct UciEngine {
    pub(crate) position: Position,
    pub(crate) tt: Arc<TranspositionTable>,
    pub(crate) threads: usize,
}

impl UciEngine {
    pub fn new() -> Self {
        UciEngine {
            position: Position::new(),
            tt: Arc::new(TranspositionTable::new_mb(256)),
            threads: 1,
        }
    }
}

impl Default for UciEngine {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
