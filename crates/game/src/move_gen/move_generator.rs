use crate::{game::Game, move_gen::move_type::Move};

pub struct MoveGenerator {
    pub moves: [Move; 256],
    pub count: usize,
}
impl Game {
    pub fn generate_legal_moves(&mut self, mg: &MoveGenerator) {}
    fn generate_knight_moves(&mut self, mg: &MoveGenerator) {}
    fn generate_bishop_moves(&mut self, mg: &MoveGenerator) {}
    fn generate_rook_moves(&mut self, mg: &MoveGenerator) {}
    fn generate_queen_moves(&mut self, mg: &MoveGenerator) {}
    fn generate_pawn_moves(&mut self, mg: &MoveGenerator) {}
    fn generate_king_moves(&mut self, mg: &MoveGenerator) {}
}
