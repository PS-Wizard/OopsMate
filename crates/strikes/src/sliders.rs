use crate::backend::pext::slider_index;

#[inline(always)]
#[must_use]
pub fn rook_attacks(square: usize, occupied: u64) -> u64 {
    // PEXT compresses just the relevant blocker bits into a dense table index,
    // which is why the generated tables are flat + offset-based instead of nested.
    let index = slider_index(occupied, crate::ROOK_MASKS[square]);
    crate::ROOK_ATTACKS[crate::ROOK_OFFSETS[square] as usize + index]
}

#[inline(always)]
#[must_use]
pub fn bishop_attacks(square: usize, occupied: u64) -> u64 {
    let index = slider_index(occupied, crate::BISHOP_MASKS[square]);
    crate::BISHOP_ATTACKS[crate::BISHOP_OFFSETS[square] as usize + index]
}

#[inline(always)]
#[must_use]
pub fn queen_attacks(square: usize, occupied: u64) -> u64 {
    rook_attacks(square, occupied) | bishop_attacks(square, occupied)
}
