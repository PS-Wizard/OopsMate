#![allow(dead_code)]

pub static BETWEEN: [[u64; 64]; 64] = generate_between();

const fn generate_between() -> [[u64; 64]; 64] {
    let mut between = [[0u64; 64]; 64];

    let mut from = 0;
    while from < 64 {
        let mut to = 0;
        while to < 64 {
            if from != to {
                between[from][to] = calculate_between(from, to);
            }
            to += 1;
        }
        from += 1;
    }

    between
}

const fn calculate_between(from: usize, to: usize) -> u64 {
    let from_rank = from / 8;
    let from_file = from % 8;
    let to_rank = to / 8;
    let to_file = to % 8;

    let rank_diff = if to_rank > from_rank {
        1
    } else if to_rank < from_rank {
        -1
    } else {
        0
    };
    let file_diff = if to_file > from_file {
        1
    } else if to_file < from_file {
        -1
    } else {
        0
    };

    // Only calculate if squares are on same rank, file, or diagonal
    if rank_diff == 0 && file_diff == 0 {
        return 0; // Same square
    }

    if rank_diff != 0
        && file_diff != 0
        && (to_rank as isize - from_rank as isize).abs()
            != (to_file as isize - from_file as isize).abs()
    {
        return 0; // Not on same diagonal
    }

    let mut result = 0u64;
    let mut current_rank = from_rank as isize + rank_diff;
    let mut current_file = from_file as isize + file_diff;

    while current_rank != to_rank as isize || current_file != to_file as isize {
        if current_rank >= 0 && current_rank < 8 && current_file >= 0 && current_file < 8 {
            let square = (current_rank * 8 + current_file) as usize;
            result |= 1u64 << square;
        }
        current_rank += rank_diff;
        current_file += file_diff;
    }

    result
}

#[cfg(test)]
mod test_between {
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    use crate::between::BETWEEN;

    #[test]
    fn test_between_stuff() {
        BETWEEN["e5".idx()]["c3".idx()].print();
        BETWEEN["a1".idx()]["a8".idx()].print();
        BETWEEN["b1".idx()]["a1".idx()].print();
        BETWEEN["h1".idx()]["h8".idx()].print();
    }
}
