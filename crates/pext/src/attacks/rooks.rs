pub const fn generate_rook_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let mut square = 0;

    while square < 64 {
        let mut mask = 0u64;
        let rank = square / 8;
        let file = square % 8;

        // Directions: up, down, left, right
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        let mut dir_idx = 0;
        while dir_idx < 4 {
            let (dr, df) = directions[dir_idx];
            let mut r = rank as i32 + dr;
            let mut f = file as i32 + df;

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
            dir_idx += 1;
        }

        masks[square] = mask;
        square += 1;
    }

    masks
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

    use crate::attacks::rooks::{generate_rook_attacks, generate_rook_masks};
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    #[test]
    fn test_rook_mask() {
        let masks = generate_rook_masks();
        let idxs = ["h4", "a1", "a8", "h8", "h1", "a4", "e5"];
        for idx in idxs {
            println!("For: {idx}");
            let mask = masks[idx.idx() as usize];
            mask.print();
            println!("---");
        }
    }

    #[test]
    fn test_rook_attacks() {
        let idxs = ["h4", "a1", "a8", "h8", "h1", "a4", "e5"];
        let blockers = "d4,d3,a2,h2,f4,g7,e6";
        for idx in idxs {
            println!("For Rook on: {idx} & Blockers on {blockers}");
            generate_rook_attacks(idx.idx(), blockers.place()).print();
            println!("---");
        }
    }
}
