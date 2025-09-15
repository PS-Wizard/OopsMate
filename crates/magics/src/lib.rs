use std::sync::LazyLock;

use crate::{
    bishops::{get_bishop_attacks, get_bishop_masks},
    rooks::{get_rook_attacks, get_rook_masks},
};

mod bishops;
mod bob;
mod rooks;
pub mod utils;

pub static BISHOP_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(get_bishop_masks).collect());
pub static BISHOP_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(get_bishop_attacks, get_bishop_masks));

pub static ROOK_MASKS: LazyLock<Vec<u64>> =
    LazyLock::new(|| (0u64..64).map(get_rook_masks).collect());
pub static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> =
    LazyLock::new(|| bob::generate_attack_table(get_rook_attacks, get_rook_masks));

#[cfg(test)]
mod test {

    use std::arch::x86_64::_pext_u64;

    use crate::utils::{BitBoardPrinter, StrToNotation};

    use super::*;
    #[test]
    fn test_magics() {
        let sq = "e4".to_idx() as usize;
        let pext = unsafe { _pext_u64("g2,d5".to_blockers(), BISHOP_MASKS[sq]) };
        let attack = BISHOP_ATTACKS[sq][pext as usize];
        attack.print_board();

        let pext = unsafe { _pext_u64("e3,d4".to_blockers(), ROOK_MASKS[sq]) };
        let attack = ROOK_ATTACKS[sq][pext as usize];
        attack.print_board();
    }
}
