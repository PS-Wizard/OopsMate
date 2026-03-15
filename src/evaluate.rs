//! Incremental NNUE evaluation support.
//!
//! This module owns probe construction, full position synchronization, and the
//! move deltas used to keep evaluation updates cheap during search.

mod delta;
mod mapping;
mod networks;
mod probe;
#[cfg(test)]
mod tests;

/// Applies a move to an evaluation probe and returns the delta needed to undo it.
pub use delta::{apply_move, apply_null_move, undo_move, undo_null_move};
/// Probe construction and full-position evaluation helpers.
pub use probe::{evaluate, evaluate_with_probe, new_probe, sync_probe};

/// Thread-local or per-search NNUE probe type used by the engine.
pub type EvalProbe = nnuebie::NNUEProbe;
/// Incremental NNUE delta returned by move application helpers.
pub type EvalMoveDelta = nnuebie::MoveDelta;
