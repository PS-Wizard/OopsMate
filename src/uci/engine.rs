use crate::{eval::EvalProvider, tpt::TranspositionTable, Position};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

pub(crate) struct ActiveSearch {
    pub(crate) stop_signal: Arc<AtomicBool>,
    pub(crate) handle: JoinHandle<TranspositionTable>,
}

/// UCI-facing engine state.
pub struct UciEngine<E: EvalProvider> {
    pub(crate) position: Position,
    pub(crate) tt: Option<TranspositionTable>,
    pub(crate) eval: E,
    pub(crate) active_search: Option<ActiveSearch>,
}

impl<E: EvalProvider> UciEngine<E> {
    /// Creates a new engine initialized to the standard starting position.
    pub fn new(eval: E) -> Self {
        UciEngine {
            position: Position::new(),
            tt: Some(TranspositionTable::new_mb(256)),
            eval,
            active_search: None,
        }
    }
}

impl Default for UciEngine<crate::eval::NnueProvider> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(crate::eval::NnueProvider::new())
    }
}
