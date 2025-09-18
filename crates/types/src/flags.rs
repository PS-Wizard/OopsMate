#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Promotion {
    None = 0,
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MoveFlags {
    None = 0,
    QueenSideCastle,
    KingSideCastle,
}
