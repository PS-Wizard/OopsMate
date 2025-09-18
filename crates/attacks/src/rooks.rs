pub fn generate_rook_masks(square: u64) -> u64 {
    let mut mask = 0u64;
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;

    // Directions: (dr, df) for up, down, left, right
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (dr, df) in directions {
        let mut r = rank + dr;
        let mut f = file + df;

        // Add all squares in this direction except the final edge square
        while r >= 0 && r < 8 && f >= 0 && f < 8 {
            // Check if this is the final edge square in this direction
            let next_r = r + dr;
            let next_f = f + df;
            let is_final_edge = next_r < 0 || next_r >= 8 || next_f < 0 || next_f >= 8;

            if !is_final_edge {
                let sq = (r * 8 + f) as u64;
                mask |= 1 << sq;
            }

            r += dr;
            f += df;
        }
    }

    mask
}

pub fn generate_rook_attacks(square: u64, blockers: u64) -> u64 {
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

    use crate::rooks::{generate_rook_attacks, generate_rook_masks};
    use handies::{algebraic::Algebraic, board::PrintAsBoard};

    #[test]
    fn test_rook_mask() {
        let mask = generate_rook_masks("g4".idx());
        mask.print()
    }

    #[test]
    fn test_rook_attacks() {
        generate_rook_attacks("e4".idx(), "d4,e2,h4".place()).print();
        generate_rook_attacks("e4".idx(), 0).print();
    }
}
