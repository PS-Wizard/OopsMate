use crate::buildgen::alignment_step;

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

    let to_rank = (to / 8) as i32;
    let to_file = (to % 8) as i32;
    let mut rank = (from / 8) as i32 + dr;
    let mut file = (from % 8) as i32 + df;
    let mut mask = 0u64;

    while rank != to_rank || file != to_file {
        let square = (rank * 8 + file) as usize;
        mask |= 1u64 << square;
        rank += dr;
        file += df;
    }

    mask & !(1u64 << to)
}
