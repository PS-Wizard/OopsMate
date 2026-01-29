use crate::position::Position;
use crate::types::MoveCollector;

impl Position {
    /// Check if the game is over (no legal moves)
    pub fn is_game_over(&self) -> bool {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);
        collector.is_empty()
    }

    /// Check if the current position is checkmate
    pub fn is_checkmate(&self) -> bool {
        self.is_in_check() && self.is_game_over()
    }

    /// Check if the current position is stalemate
    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check() && self.is_game_over()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkmate_detection() {
        // Fool's mate
        let pos =
            Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        assert!(pos.is_checkmate());
    }

    #[test]
    fn test_stalemate_detection() {
        let pos = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(pos.is_stalemate());
    }
}
