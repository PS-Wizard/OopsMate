use crate::eval::EvalProvider;
use crate::search::{search_with_eval, SearchInfo, SearchLimits};
use crate::tpt::TranspositionTable;
use crate::Position;

/// Engine state parameterized by the chosen evaluation provider.
pub struct Engine<E: EvalProvider> {
    pub position: Position,
    pub tt: TranspositionTable,
    pub eval: E,
}

impl<E: EvalProvider> Engine<E> {
    /// Creates a new engine initialized to the standard starting position.
    pub fn new(eval: E) -> Self {
        Self {
            position: Position::new(),
            tt: TranspositionTable::new_mb(256),
            eval,
        }
    }

    /// Searches the current position with the configured evaluation provider.
    pub fn search(&mut self, max_depth: u8, max_time_ms: Option<u64>) -> Option<SearchInfo> {
        search_with_eval(
            &self.position,
            max_depth,
            SearchLimits::from_max_time(max_time_ms),
            &mut self.tt,
            self.eval.clone(),
        )
    }
}
