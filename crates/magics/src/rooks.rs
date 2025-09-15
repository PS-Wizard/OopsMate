pub fn get_rook_masks(square: u64) -> u64 {
    let mut mask = 0u64;
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;

    // Directions: (dr, df) for up, down, left, right
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (dr, df) in directions {
        let mut r = rank + dr;
        let mut f = file + df;

        // Exclude edges (for magic mask generation)
        while r > 0 && r < 7 && f > 0 && f < 7 {
            let sq = (r * 8 + f) as u64;
            mask |= 1 << sq;
            r += dr;
            f += df;
        }
    }

    mask
}

pub fn get_rook_attacks(square: u64, blockers: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;

    // Directions: up, down, left, right
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

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
mod test_rooks {
    use crate::{
        rooks::{get_rook_attacks, get_rook_masks},
        utils::{BitBoardPrinter, StrToNotation},
    };

    #[test]
    fn test_rook_mask() {
        let mask = get_rook_masks("h4".to_idx());
        mask.print_board()
    }

    #[test]
    fn test_rook_attacks() {
        get_rook_attacks("e4".to_idx(), "d4,e2,h4".to_blockers()).print_board();
        get_rook_attacks("e4".to_idx(), 0).print_board();
    }
}
