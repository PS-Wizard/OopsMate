use crate::buildgen::on_board;

pub fn generate() -> [u64; 64] {
    const DELTAS: [(i32, i32); 8] = [
        (2, 1),
        (1, 2),
        (-1, 2),
        (-2, 1),
        (-2, -1),
        (-1, -2),
        (1, -2),
        (2, -1),
    ];

    let mut attacks = [0u64; 64];

    for square in 0..64 {
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;
        let mut mask = 0u64;

        for (dr, df) in DELTAS {
            let target_rank = rank + dr;
            let target_file = file + df;
            if on_board(target_rank, target_file) {
                let target = (target_rank * 8 + target_file) as usize;
                mask |= 1u64 << target;
            }
        }

        attacks[square] = mask;
    }

    attacks
}
