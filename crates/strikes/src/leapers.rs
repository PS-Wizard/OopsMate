#[inline(always)]
#[must_use]
pub const fn pawn_attacks(color: usize, square: usize) -> u64 {
    crate::PAWN_ATTACKS[color][square]
}

#[inline(always)]
#[must_use]
pub const fn knight_attacks(square: usize) -> u64 {
    crate::KNIGHT_ATTACKS[square]
}

#[inline(always)]
#[must_use]
pub const fn king_attacks(square: usize) -> u64 {
    crate::KING_ATTACKS[square]
}
