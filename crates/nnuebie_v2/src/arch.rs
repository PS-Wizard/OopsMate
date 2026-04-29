pub(crate) const DENSE_CHUNK_SIZE: usize = 4;
pub(crate) const FT_PERMUTE_ORDER: [usize; 8] = [0, 2, 1, 3, 4, 6, 5, 7];
pub(crate) const FT_PERMUTE_BLOCK_I16S: usize = 8;
pub(crate) const FT_PERMUTE_GROUP_I16S: usize = FT_PERMUTE_BLOCK_I16S * FT_PERMUTE_ORDER.len();
pub(crate) const FT_SCALE: i16 = 2;

#[cfg(test)]
const fn invert_order(order: [usize; 8]) -> [usize; 8] {
    let mut inverse = [0usize; 8];
    let mut idx = 0;
    while idx < 8 {
        inverse[order[idx]] = idx;
        idx += 1;
    }
    inverse
}

#[cfg(test)]
pub(crate) const FT_PERMUTE_INVERSE_ORDER: [usize; 8] = invert_order(FT_PERMUTE_ORDER);
