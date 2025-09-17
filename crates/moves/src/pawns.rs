#![allow(dead_code)]

const RANK_2: u64 = 0x000000000000FF00;
const RANK_7: u64 = 0x00FF000000000000;

const FILE_A: u64 = 0x0101010101010101;
const FILE_H: u64 = 0x8080808080808080;

pub fn generate_pawn_moves(pawns: u64, enemies: u64, turn: u64, en_passant_square: u64) -> u64 {
    let empty = !(pawns | enemies);

    // --- Single pushes ---
    let single_pushes_white = (pawns << 8) & empty;
    let single_pushes_black = (pawns >> 8) & empty;

    // --- Double pushes ---
    let double_pushes_white = ((pawns & RANK_2) << 8 & empty) << 8 & empty;
    let double_pushes_black = ((pawns & RANK_7) >> 8 & empty) >> 8 & empty;

    // --- Normal captures ---
    let captures_white = (((pawns << 7) & !FILE_H) | ((pawns << 9) & !FILE_A)) & enemies;
    let captures_black = (((pawns >> 9) & !FILE_H) | ((pawns >> 7) & !FILE_A)) & enemies;

    // --- En passant captures ---
    let ep_captures_white = (((pawns & 0x000000FF00000000) << 7 & !FILE_H)
        | ((pawns & 0x000000FF00000000) << 9 & !FILE_A))
        & en_passant_square;

    let ep_captures_black = (((pawns & 0x00000000FF000000) >> 9 & !FILE_H)
        | ((pawns & 0x00000000FF000000) >> 7 & !FILE_A))
        & en_passant_square;

    // --- Branchless turn selection ---
    let mask = 0u64.wrapping_sub(turn); // 0 = white, 0xFFFF.. = black

    let single_pushes = (single_pushes_white & !mask) | (single_pushes_black & mask);
    let double_pushes = (double_pushes_white & !mask) | (double_pushes_black & mask);
    let captures = (captures_white & !mask) | (captures_black & mask);
    let ep_captures = (ep_captures_white & !mask) | (ep_captures_black & mask);

    // --- Combine all moves ---
    single_pushes | double_pushes | captures | ep_captures
}

#[cfg(test)]
mod test_pawns {
    use handies::board::PrintAsBoard;

    use crate::pawns::{RANK_2, generate_pawn_moves};

    #[test]
    fn test_generate_pawn_moves() {
        let moves = generate_pawn_moves(RANK_2, 0, 0, 0);
        println!("{}", moves.count_ones());
        moves.print();
    }
}
