use super::Position;
use crate::MoveCollector;

impl Position {
    pub fn is_game_over(&self) -> bool {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);
        collector.is_empty()
    }

    pub fn is_checkmate(&self) -> bool {
        self.is_in_check() && self.is_game_over()
    }

    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check() && self.is_game_over()
    }

    #[inline(always)]
    pub const fn is_fifty_move_draw(&self) -> bool {
        self.halfmove >= 100
    }

    pub fn is_repetition(&self) -> bool {
        if self.halfmove < 4 {
            return false;
        }

        let hash = self.hash;
        let max_back = (self.halfmove as usize).min(self.history.len());

        for prev in self
            .history
            .iter()
            .rev()
            .skip(1)
            .take(max_back.saturating_sub(1))
            .step_by(2)
        {
            if prev.hash == hash {
                return true;
            }
        }

        false
    }
}
