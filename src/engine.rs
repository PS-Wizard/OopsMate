use crate::eval::EvalProvider;
use crate::search::{search_with_eval, SearchInfo, SearchLimits};
use crate::tpt::TranspositionTable;
use crate::Position;
use std::sync::Arc;

/// Engine state parameterized by the chosen evaluation provider.
pub struct Engine<E: EvalProvider> {
    pub position: Position,
    pub tt: Arc<TranspositionTable>,
    pub threads: usize,
    pub eval: E,
}

impl<E: EvalProvider> Engine<E> {
    /// Creates a new engine initialized to the standard starting position.
    pub fn new(eval: E) -> Self {
        Self {
            position: Position::new(),
            tt: Arc::new(TranspositionTable::new_mb(256)),
            threads: 1,
            eval,
        }
    }

    /// Searches the current position with the configured evaluation provider.
    pub fn search(&self, max_depth: u8, max_time_ms: Option<u64>) -> Option<SearchInfo> {
        search_with_eval(
            &self.position,
            max_depth,
            SearchLimits::from_max_time(max_time_ms),
            self.tt.clone(),
            self.threads,
            self.eval.clone(),
        )
    }
}
