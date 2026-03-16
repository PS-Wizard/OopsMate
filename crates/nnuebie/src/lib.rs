//! Lean NNUE evaluation crate for chess engines using Stockfish-style `.nnue` files.
//!
//! The crate exposes two main entry points:
//! - [`NnueNetworks`], which loads a big/small network pair
//! - [`NNUEProbe`], which owns mutable board state and evaluates positions incrementally

#[cfg(all(feature = "simd_avx2", feature = "simd_scalar"))]
compile_error!("Enable either `simd_avx2` or `simd_scalar`, not both.");

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

/// Serialized network format version expected by the loader.
pub const VERSION: u32 = 0x7AF32F20;
/// Final NNUE output divisor used before centipawn conversion.
pub const OUTPUT_SCALE: i32 = 16;
/// Weight scaling shift used by the network format.
pub const WEIGHT_SCALE_BITS: i32 = 6;
