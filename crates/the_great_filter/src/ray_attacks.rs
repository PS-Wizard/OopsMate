#![allow(dead_code)]

use crate::{BOTTOM, BOTTOM_LEFT, BOTTOM_RIGHT, LEFT, RIGHT, TOP, TOP_LEFT, TOP_RIGHT};

pub static RAY_ATTACKS: [[u64; 64]; 8] = generate_ray_attacks();

const fn generate_ray_attacks() -> [[u64; 64]; 8] {
    let mut rays = [[0u64; 64]; 8];

    let mut square = 0;
    while square < 64 {
        let rank = square / 8;
        let file = square % 8;

        // NORTH (up)
        rays[TOP][square] = generate_ray(rank, file, 1, 0);

        // NORTH_EAST (up-right)
        rays[TOP_RIGHT][square] = generate_ray(rank, file, 1, 1);

        // EAST (right)
        rays[RIGHT][square] = generate_ray(rank, file, 0, 1);

        // SOUTH_EAST (down-right)
        rays[BOTTOM_RIGHT][square] = generate_ray(rank, file, -1, 1);

        // SOUTH (down)
        rays[BOTTOM][square] = generate_ray(rank, file, -1, 0);

        // SOUTH_WEST (down-left)
        rays[BOTTOM_LEFT][square] = generate_ray(rank, file, -1, -1);

        // WEST (left)
        rays[LEFT][square] = generate_ray(rank, file, 0, -1);

        // NORTH_WEST (up-left)
        rays[TOP_LEFT][square] = generate_ray(rank, file, 1, -1);

        square += 1;
    }

    rays
}

const fn generate_ray(
    start_rank: usize,
    start_file: usize,
    rank_delta: isize,
    file_delta: isize,
) -> u64 {
    let mut ray = 0u64;
    let mut rank = start_rank as isize + rank_delta;
    let mut file = start_file as isize + file_delta;

    while rank >= 0 && rank < 8 && file >= 0 && file < 8 {
        let square = (rank * 8 + file) as usize;
        ray |= 1u64 << square;
        rank += rank_delta;
        file += file_delta;
    }

    ray
}

#[cfg(test)]
mod test_ray_attacks {
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    use crate::ray_attacks::{RAY_ATTACKS, RIGHT, TOP, TOP_LEFT, TOP_RIGHT};

    #[test]
    fn test_ray_attack_thingy() {
        RAY_ATTACKS[TOP_RIGHT]["f4".idx()].print();
        RAY_ATTACKS[TOP_LEFT]["h4".idx()].print();
        RAY_ATTACKS[TOP_RIGHT]["h6".idx()].print();
        RAY_ATTACKS[RIGHT]["d6".idx()].print();
        RAY_ATTACKS[TOP]["c7".idx()].print();
    }
}
