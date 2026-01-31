pub mod position;
pub mod types;

pub mod evaluate;
pub mod movegen;
pub mod search;

pub mod tpt;
pub mod zobrist;

pub mod time_control;
pub mod uci;

pub mod lmr;
pub mod move_ordering;
pub mod qsearch;

pub mod move_history;

pub use position::Position;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position() {
        let pos = Position::new();
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);
        assert_eq!(collector.len(), 20);
    }

    #[test]
    fn test_make_move() {
        let mut pos = Position::new();
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);

        let m = collector.get(0);

        // Make the move
        let undo = pos.make_move(&m);

        // After making move, side to move should be Black
        assert_eq!(pos.side_to_move, Color::Black);

        // Unmake the move
        pos.unmake_move(&m, &undo);

        // After unmaking, should be back to White
        assert_eq!(pos.side_to_move, Color::White);
    }
}
