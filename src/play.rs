use crate::position::Position;
use crate::types::{Move, MoveCollector};
use rand::Rng;

impl Position {
    /// Select a random legal move from the current position
    pub fn random_move(&self) -> Option<Move> {
        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        if collector.is_empty() {
            return None;
        }

        let idx = rand::thread_rng().gen_range(0..collector.len());
        Some(collector.get(idx))
    }

    /// Play a random move and return the new position
    pub fn play_random(&self) -> Option<Position> {
        self.random_move().map(|m| self.make_move(m))
    }

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
    fn test_random_move() {
        let pos = Position::new();
        let m = pos.random_move();
        assert!(m.is_some(), "Starting position should have legal moves");
    }

    #[test]
    fn test_checkmate_detection() {
        // Fool's mate
        let pos = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
            .unwrap();
        assert!(pos.is_checkmate());
    }

    #[test]
    fn test_stalemate_detection() {
        let pos = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(pos.is_stalemate());
    }

    #[test]
    fn test_play_random_game() {
        let mut pos = Position::new();
        let mut moves = 0;

        // Play until game over or 100 moves
        while let Some(new_pos) = pos.play_random() {
            pos = new_pos;
            moves += 1;
            if moves >= 100 {
                break;
            }
        }

        assert!(moves > 0, "Should play at least one move");
    }
}
