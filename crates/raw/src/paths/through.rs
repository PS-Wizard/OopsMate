use crate::{BETWEEN, LINE};

// Add this to your statics
pub const fn generate_line() -> [[u64; 64]; 64] {
    let mut line = [[0u64; 64]; 64];
    let mut from = 0;
    while from < 64 {
        let mut to = 0;
        while to < 64 {
            if from != to {
                line[from][to] = calculate_line(from, to);
            }
            to += 1;
        }
        from += 1;
    }
    line
}

const fn calculate_line(from: usize, to: usize) -> u64 {
    let between = BETWEEN[from][to]; // Your existing function

    if between == 0 && from != to {
        // Check if they're adjacent on a line
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

        if rank_diff == 0 && file_diff == 0 {
            return 0; // Same square
        }

        if rank_diff != 0
            && file_diff != 0
            && (to_rank as isize - from_rank as isize).abs()
                != (to_file as isize - from_file as isize).abs()
        {
            return 0; // Not on line
        }
    } else if between == 0 {
        return 0; // Not on same line
    }

    // Extend in both directions
    extend_ray_bidirectional(from, to)
}

const fn extend_ray_bidirectional(from: usize, to: usize) -> u64 {
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

    let mut result = 0u64;

    // Start from one end and go all the way to the other end of the board
    // Find the furthest square in the backward direction
    let mut rank = from_rank as isize;
    let mut file = from_file as isize;

    // Go backwards to board edge
    while rank >= 0 && rank < 8 && file >= 0 && file < 8 {
        result |= 1u64 << (rank * 8 + file);
        rank -= rank_diff;
        file -= file_diff;
    }

    // Now go forward from 'from' to board edge
    let mut rank = from_rank as isize + rank_diff;
    let mut file = from_file as isize + file_diff;
    while rank >= 0 && rank < 8 && file >= 0 && file < 8 {
        result |= 1u64 << (rank * 8 + file);
        rank += rank_diff;
        file += file_diff;
    }

    result
}

#[inline(always)]
pub fn ray_through(sq1: usize, sq2: usize) -> u64 {
    LINE[sq1][sq2]
}

#[cfg(test)]
mod line {
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    use crate::line_through;

    #[test]
    fn test_through() {
        line_through("e1".idx(), "c3".idx()).print();
        line_through("e1".idx(), "e2".idx()).print();
        line_through("e1".idx(), "f2".idx()).print();
        line_through("e4".idx(), "c3".idx()).print();
    }
}
