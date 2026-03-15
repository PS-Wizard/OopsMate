//! Legal move generation.
//!
//! The implementation is split by piece family and by search use-case so that
//! full generation and capture-only generation stay easy to follow.

mod attacks;
mod captures;
mod constraints;
mod generation;
mod king;
mod leapers;
mod pawn_captures;
mod pawns;
mod sliders;
