use crate::{tpt::TranspositionTable, Position};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

pub(crate) struct ActiveSearch {
    pub(crate) stop_signal: Arc<AtomicBool>,
    pub(crate) handle: JoinHandle<()>,
}

/// UCI-facing engine state.
pub struct UciEngine {
    pub(crate) position: Position,
    pub(crate) tt: Arc<TranspositionTable>,
    pub(crate) threads: usize,
    pub(crate) active_search: Option<ActiveSearch>,
}

impl UciEngine {
    /// Creates a new engine initialized to the standard starting position.
    pub fn new() -> Self {
        UciEngine {
            position: Position::new(),
            tt: Arc::new(TranspositionTable::new_mb(256)),
            threads: 4,
            active_search: None,
        }
    }
}

impl Default for UciEngine {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
