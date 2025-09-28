#![allow(dead_code)]
use crate::pins_checks::move_type::Move;

pub struct MoveGenerator {
    pub moves: [Move; 256],
    pub count: usize,
}

mod bishops;
mod king;
mod knights;
mod rooks;
mod queens;
mod pawns;
