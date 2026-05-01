use crate::buildgen::{alignment_step, on_board};

pub fn generate() -> [[u64; 64]; 64] {
    let mut table = [[0u64; 64]; 64];

    for from in 0..64 {
        for to in 0..64 {
            table[from][to] = mask(from, to);
        }
    }

    table
}

pub fn mask(from: usize, to: usize) -> u64 {
    let Some((dr, df)) = alignment_step(from, to) else {
        return 0;
    };

    let mut mask = 0u64;

    let mut rank = (from / 8) as i32;
    let mut file = (from % 8) as i32;
    while on_board(rank, file) {
        mask |= 1u64 << (rank * 8 + file);
        rank -= dr;
        file -= df;
    }

    let mut rank = (from / 8) as i32 + dr;
    let mut file = (from % 8) as i32 + df;
    while on_board(rank, file) {
        mask |= 1u64 << (rank * 8 + file);
        rank += dr;
        file += df;
    }

    mask
}
