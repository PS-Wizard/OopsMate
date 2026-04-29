//! Shared engine data types.
//!
//! These types are intentionally compact and copy-friendly because they appear
//! throughout move generation, evaluation, hashing, and search.

mod bitboard;
mod castle;
mod color;
mod moves;
mod piece;

pub use bitboard::Bitboard;
pub use castle::CastleRights;
pub use color::Color;
pub use moves::{Move, MoveCollector, MoveType};
pub use piece::Piece;
