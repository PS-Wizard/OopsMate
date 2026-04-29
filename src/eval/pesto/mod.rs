mod eg_tables;
mod evaluate;
mod mg_tables;

use super::EvalProvider;
use crate::{Move, Position};

#[derive(Clone, Copy, Default)]
pub struct PestoProvider;

impl PestoProvider {
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }
}

impl EvalProvider for PestoProvider {
    type State = ();
    type Undo = ();

    #[inline(always)]
    fn new_state(&self, _pos: &Position) -> Self::State {}

    #[inline(always)]
    fn sync(&self, _state: &mut Self::State, _pos: &Position) {}

    #[inline(always)]
    fn eval(&self, pos: &Position, _state: &mut Self::State) -> i32 {
        evaluate::evaluate(pos)
    }

    #[inline(always)]
    fn update_on_move(&self, _state: &mut Self::State, _pos: &Position, _mv: Move) -> Self::Undo {}
}

#[cfg(test)]
mod tests {
    use super::evaluate::evaluate;
    use crate::Position;

    #[test]
    fn start_position_is_equal() {
        assert_eq!(evaluate(&Position::new()), 0);
    }

    #[test]
    fn side_to_move_flips_sign() {
        let white = Position::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/4p3/2BPP1b1/2P2N2/P1P2PPP/R1BQR1K1 w - - 0 1",
        )
        .unwrap();
        let black = Position::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/4p3/2BPP1b1/2P2N2/P1P2PPP/R1BQR1K1 b - - 0 1",
        )
        .unwrap();
        assert_eq!(evaluate(&white), -evaluate(&black));
    }
}
