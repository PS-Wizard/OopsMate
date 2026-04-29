//! Position state and state transitions.
//!
//! `Position` owns the engine's authoritative board representation together with
//! hashing, repetition history, FEN parsing, and make/unmake support.

mod draw;
mod fen;
mod hash;
mod make;
mod state;
#[cfg(test)]
mod tests;
mod unmake;

/// Public position types exported by the position subsystem.
pub use state::{GameState, Position};
