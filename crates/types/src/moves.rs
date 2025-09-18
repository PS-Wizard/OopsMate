#![allow(dead_code)]

use crate::{flags::*, piece_kind::PieceKind};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move(u32);

impl Move {
    pub fn new(
        from: u8,
        to: u8,
        piece: PieceKind,
        capture: PieceKind,
        promotion: Promotion,
        flags: MoveFlags,
    ) -> Self {
        let val = (from as u32)
            | ((to as u32) << 6)
            | ((piece as u32) << 12)
            | ((capture as u32) << 16)
            | ((promotion as u32) << 20)
            | ((flags as u32) << 24);
        Move(val)
    }

    pub fn from(self) -> u8 {
        (self.0 & 0x3F) as u8
    }

    pub fn to(self) -> u8 {
        ((self.0 >> 6) & 0x3F) as u8
    }

    pub fn piece(self) -> u8 {
        ((self.0 >> 12) & 0xF) as u8
    }

    pub fn capture(self) -> u8 {
        ((self.0 >> 16) & 0xF) as u8
    }

    pub fn promotion(self) -> u8 {
        ((self.0 >> 20) & 0xF) as u8
    }

    pub fn flags(self) -> u8 {
        ((self.0 >> 24) & 0xF) as u8
    }

    pub fn additional_flags(self) -> u8 {
        ((self.0 >> 28) & 0xF) as u8
    }
}

#[cfg(test)]
mod test_moves {

    use handies::algebraic::Algebraic;

    use crate::{flags::*, moves::Move, piece_kind::PieceKind::*};

    #[test]
    fn test_create_move() {
        let some_move = Move::new(
            "d4".idx() as u8,
            "d6".idx() as u8,
            WhiteRook,
            BlackPawn,
            Promotion::None,
            MoveFlags::None,
        );

        #[cfg(debug_assertions)]
        println!(
            "From: {}, To: {}, Piece is of type: {}, It captures: {}, Promotion: {}, Flags: {}, Additional Flags: {}",
            some_move.from(),
            some_move.to(),
            some_move.piece(),
            some_move.capture(),
            some_move.promotion(),
            some_move.flags(),
            some_move.additional_flags(),
        );
    }
}
