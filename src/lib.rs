//! `oops_mate` is a UCI chess engine focused on a small, explicit core.
//!
//! The crate is organized by engine subsystem so that board state, move generation,
//! evaluation, search, and protocol handling can evolve independently without
//! obscuring the hot paths that matter most.

/// Engine state wrapper parameterized by eval provider.
pub mod engine;
/// Evaluation providers and shared eval abstraction.
pub mod eval;
/// Legal move generation and attack detection.
pub mod movegen;
/// Board state, FEN parsing, hashing, and make/unmake logic.
pub mod position;
/// Search orchestration, pruning, and root reporting.
pub mod search;
/// Static exchange evaluation.
pub mod see;
/// Time allocation helpers used by the UCI front-end.
pub mod time_control;
/// Transposition table storage.
pub mod tpt;
/// Shared low-level engine types.
pub mod types;
/// UCI protocol driver.
pub mod uci;
/// Zobrist hashing keys.
pub mod zobrist;

/// Generic engine wrapper.
pub use engine::Engine;
/// Evaluation providers and trait.
pub use eval::{EvalProvider, NnueProvider, PestoProvider};
/// The engine board representation.
pub use position::Position;
/// Common engine types re-exported at the crate root.
pub use types::*;

#[cfg(test)]
mod benchmark_tests;
