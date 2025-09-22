pub const fn generate_bishop_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let mut square = 0;

    while square < 64 {
        let mut mask = 0u64;
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;

        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        let mut dir_idx = 0;
        while dir_idx < 4 {
            let (dr, df) = directions[dir_idx];
            let mut r = rank + dr;
            let mut f = file + df;

            while r > 0 && r < 7 && f > 0 && f < 7 {
                let sq = (r * 8 + f) as u64;
                mask |= 1 << sq;
                r += dr;
                f += df;
            }

            dir_idx += 1;
        }

        masks[square] = mask;
        square += 1;
    }

    masks
}

pub fn generate_bishop_attacks(square: u64, blockers: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;

    // Directions: (dr, df)
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    for (dr, df) in directions {
        let mut r = rank + dr;
        let mut f = file + df;

        while r >= 0 && r < 8 && f >= 0 && f < 8 {
            let sq = (r * 8 + f) as u64;
            attacks |= 1 << sq;

            if (blockers & (1 << sq)) != 0 {
                break; // stop ray if blocked
            }

            r += dr;
            f += df;
        }
    }

    attacks
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod test_bishops {
    use super::*;
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    #[test]
    fn test_bishop_mask() {
        let mask = generate_bishop_masks();
        let idxs = ["h4", "a1", "a8", "h8", "h1", "a4", "e5"];
        for idx in idxs {
            println!("For: {idx}");
            let mask = mask[idx.idx() as usize];
            mask.print();
            println!("---");
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_bishop_attacks() {
        let idxs = ["h4", "a1", "a8", "h8", "h1", "a4", "e5"];
        let blockers = "d4,d3,a2,h2,f4,g7,e6";
        for idx in idxs {
            println!("For Bishop on: {idx} & Blockers on {blockers}");
            generate_bishop_attacks(idx.idx(), blockers.place()).print();
            println!("---");
        }
    }
}
