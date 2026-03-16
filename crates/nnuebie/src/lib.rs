//! Lean NNUE evaluation crate for chess engines using Stockfish-style `.nnue` files.

mod accumulator;
mod accumulator_refresh;
mod accumulator_stack;
mod aligned;
mod architecture;
mod feature_transformer;
mod features;
mod finny_tables;
mod layers;
mod loader;
mod network;
mod nnue;
mod piece_list;
pub mod types;
pub mod uci;

#[cfg(test)]
mod tests;

pub use network::NnueNetworks;
pub use nnue::{DeltaChange, DeltaError, MoveDelta, NNUEProbe};

pub use types::{Color, Piece, Square};

pub use features::{BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};

pub const VERSION: u32 = 0x7AF32F20;
pub const OUTPUT_SCALE: i32 = 16;
pub const WEIGHT_SCALE_BITS: i32 = 6;
