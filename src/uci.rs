//! UCI protocol front-end.
//!
//! The UCI module owns the long-lived engine state used by the command loop and
//! translates text commands into search requests and move applications.

mod engine;
mod parse;
mod run;

/// Stateful UCI engine front-end.
pub use engine::UciEngine;
