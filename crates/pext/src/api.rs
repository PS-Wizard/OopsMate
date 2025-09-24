use crate::{BISHOP_ATTACKS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MASKS};
use crate::{KING_ATTACKS, KNIGHT_ATTACKS};
use std::arch::x86_64::_pext_u64;

#[inline(always)]
pub fn get_king_attacks(from: usize) -> u64 {
    KING_ATTACKS[from]
}

#[inline(always)]
pub fn get_knight_attacks(from: usize) -> u64 {
    KNIGHT_ATTACKS[from]
}

#[inline(always)]
pub fn get_bishop_attacks(from: usize, blockers: u64) -> u64 {
    let mask = BISHOP_MASKS[from];
    let idx = unsafe { _pext_u64(blockers, mask) as usize };
    BISHOP_ATTACKS[from][idx]
}

#[inline(always)]
pub fn get_rook_attacks(from: usize, blockers: u64) -> u64 {
    let mask = ROOK_MASKS[from];
    let idx = unsafe { _pext_u64(blockers, mask) as usize };
    ROOK_ATTACKS[from][idx]
}

#[inline(always)]
pub fn get_queen_attacks(from: usize, blockers: u64) -> u64 {
    let bishop = get_bishop_attacks(from, blockers);
    let rook = get_rook_attacks(from, blockers);
    bishop | rook
}

#[inline(always)]
pub fn get_pawn_moves(pawns: u64, blockers: u64, turn: u64) -> u64 {
    // masks
    let file_a: u64 = 0x0101010101010101;
    let file_h: u64 = 0x8080808080808080;
    let rank_3: u64 = 0x0000000000FF0000; // after white single push
    let rank_6: u64 = 0x0000FF0000000000; // after black single push

    let empty = !blockers;

    // white pushes
    let w_single = (pawns << 8) & empty;
    let w_double = ((w_single & rank_3) << 8) & empty;

    // black pushes
    let b_single = (pawns >> 8) & empty;
    let b_double = ((b_single & rank_6) >> 8) & empty;

    // captures
    let w_captures = ((pawns << 7) & !file_a | (pawns << 9) & !file_h) & blockers;
    let b_captures = ((pawns >> 9) & !file_a | (pawns >> 7) & !file_h) & blockers;

    // branchless turn selection
    let turn = turn.wrapping_sub(1);
    let white_moves = (w_single | w_double | w_captures) & turn;
    let black_moves = (b_single | b_double | b_captures) & !turn;

    white_moves | black_moves
}

#[cfg(test)]
mod test_pext_api {
    use utilities::{algebraic::Algebraic, board::PrintAsBoard};

    use crate::api::{
        get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_moves,
        get_queen_attacks, get_rook_attacks,
    };

    #[test]
    fn test_api() {
        let blockers = "h2,a2,d4,c8,f6,g7".place();
        let from = ["e4", "a1", "h1", "h8", "a8"];
        for starting in from {
            println!("Knight Attacks from {starting}");
            get_knight_attacks(starting.idx()).print();

            println!("King Attacks from {starting}");
            get_king_attacks(starting.idx()).print();

            println!("Bishop Attacks from {starting}");
            get_bishop_attacks(starting.idx(), blockers).print();

            println!("Rook Attacks from {starting}");
            get_rook_attacks(starting.idx(), blockers).print();

            println!("Queen Attacks from {starting}");
            get_queen_attacks(starting.idx(), blockers).print();
        }
    }

    #[test]
    fn test_pawns() {
        let blockers = "d5,a7,h2,e6".place();

        // white pawns
        let white_pawns = "e3".place();
        let white_turn = 0;

        println!("White pawn moves from e2, d4:");
        get_pawn_moves(white_pawns, blockers, white_turn).print();

        // black pawns
        let black_pawns = "d7,f7".place();
        let black_turn = 1;

        println!("Black pawn moves from d7, f7:");
        get_pawn_moves(black_pawns, blockers, black_turn).print();
    }
}
