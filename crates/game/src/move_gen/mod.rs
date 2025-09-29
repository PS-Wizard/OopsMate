#![allow(dead_code)]

mod bishops;
mod king;
mod knights;
mod pawns;
mod queens;
mod rooks;
mod make_unmake;

use crate::{
    game::Game,
    pins_checks::{move_type::Move, pin_check_finder::find_pins_n_checks},
};

pub struct MoveGenerator {
    pub moves: [Move; 256],
    pub count: usize,
}

impl MoveGenerator {
    fn new() -> Self {
        MoveGenerator {
            moves: [Move::default(); 256],
            count: 0,
        }
    }

    fn get_count(&self) -> usize {
        self.count
    }
}


impl Game {
    pub fn generate_legal_moves(&self, mg: &mut MoveGenerator) {
        mg.count = 0;
        let (pinned, _checking, check_mask) = find_pins_n_checks(self);
        self.generate_king_moves(mg);
        self.generate_bishop_moves(pinned, check_mask, mg);
        self.generate_knight_moves(pinned, check_mask, mg);
        self.generate_pawn_moves(pinned, check_mask, mg);
        self.generate_queen_moves(pinned, check_mask, mg);
        self.generate_rook_moves(pinned, check_mask, mg);
    }
}

#[cfg(test)]
mod test_move_gen {
    use crate::{game::Game, move_gen::MoveGenerator};

    #[test]
    fn test_generate_all() {
        let g = Game::new();
        let mut mg = MoveGenerator::new();
        g.generate_legal_moves(&mut mg);
        println!("Got Legal Moves: {}", mg.get_count());
    }

}
