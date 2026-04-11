//! Evaluation provider abstraction and implementations.

mod nnue;
mod pesto;

use crate::{Move, Position};

pub use nnue::NnueProvider;
pub use pesto::PestoProvider;

/// Pluggable evaluation backend used by search and engine front-ends.
pub trait EvalProvider: Clone + Send + Sync + 'static {
    /// Mutable per-search state carried through make/unmake recursion.
    type State: Send + 'static;
    /// Provider-specific undo token for incremental move updates.
    type Undo: Copy + Default + Send + 'static;

    /// Creates a fresh evaluation state synchronized to `pos`.
    fn new_state(&self, pos: &Position) -> Self::State;

    /// Rebuilds `state` from `pos` when a full resync is required.
    fn sync(&self, state: &mut Self::State, pos: &Position);

    /// Returns a static evaluation in centipawns from the side-to-move perspective.
    fn eval(&self, pos: &Position, state: &mut Self::State) -> i32;

    /// Applies the eval-side effects of `mv` before the position is updated.
    #[inline(always)]
    fn update_on_move(&self, _state: &mut Self::State, _pos: &Position, _mv: Move) -> Self::Undo {
        Self::Undo::default()
    }

    /// Undoes a previous incremental move update.
    #[inline(always)]
    fn update_on_undo(&self, _state: &mut Self::State, _undo: Self::Undo) {}

    /// Applies a null-move update before the position is updated.
    #[inline(always)]
    fn update_on_null_move(&self, _state: &mut Self::State, _pos: &Position) {}

    /// Undoes a previous null-move update.
    #[inline(always)]
    fn update_on_undo_null(&self, _state: &mut Self::State) {}
}

#[cfg(test)]
mod tests {
    use super::{EvalProvider, PestoProvider};
    use crate::Position;

    #[test]
    fn pesto_start_position_eval_is_zero() {
        let provider = PestoProvider::new();
        let pos = Position::new();
        assert_eq!(provider.eval(&pos, &mut ()), 0);
    }

    #[test]
    fn pesto_restores_score_after_make_unmake() {
        let provider = PestoProvider::new();
        let mut pos = Position::new();
        let original = provider.eval(&pos, &mut ());

        let mv = crate::Move::new(12, 28, crate::MoveType::DoublePush);
        provider.update_on_move(&mut (), &pos, mv);
        pos.make_move(mv);
        pos.unmake_move(mv);
        provider.update_on_undo(&mut (), ());

        assert_eq!(provider.eval(&pos, &mut ()), original);
    }
}
