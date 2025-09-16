pub fn get_bishop_masks(square: u64) -> u64 {
    let mut mask = 0u64;
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;

    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    for (dr, df) in directions {
        let mut r = rank + dr;
        let mut f = file + df;

        while r > 0 && r < 7 && f > 0 && f < 7 {
            let sq = (r * 8 + f) as u64;
            mask |= 1 << sq;
            r += dr;
            f += df;
        }
    }

    mask
}

pub fn get_bishop_attacks(square: u64, blockers: u64) -> u64 {
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
mod test_bishops {
    use handies::{algebraic::Algebraic, board::PrintAsBoard};

    use crate::bishops::{get_bishop_attacks, get_bishop_masks};

    #[test]
    fn test_bishop_mask() {
        let mask = get_bishop_masks("e4".idx());
        mask.print()
    }

    #[test]
    fn test_bishop_attacks() {
        get_bishop_attacks("e4".idx(), "g2,c6,d3".place()).print();
        get_bishop_attacks("e4".idx(), 0).print();
    }
}
