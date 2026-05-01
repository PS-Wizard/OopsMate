use crate::buildgen::on_board;

pub fn generate_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];

    for square in 0..64 {
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;
        let mut mask = 0u64;

        for (dr, df) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut r = rank + dr;
            let mut f = file + df;
            while on_board(r, f) {
                let next_r = r + dr;
                let next_f = f + df;
                if !on_board(next_r, next_f) {
                    break;
                }
                mask |= 1u64 << (r * 8 + f);
                r = next_r;
                f = next_f;
            }
        }

        masks[square] = mask;
    }

    masks
}

pub fn attacks(square: usize, blockers: u64) -> u64 {
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;
    let mut attacks = 0u64;

    for (dr, df) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut r = rank + dr;
        let mut f = file + df;
        while on_board(r, f) {
            let target = (r * 8 + f) as usize;
            attacks |= 1u64 << target;
            if blockers & (1u64 << target) != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }

    attacks
}
