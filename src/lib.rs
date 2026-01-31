pub mod position;
pub mod types;

pub mod evaluate;
pub mod movegen;
pub mod search;

pub mod tpt;
pub mod zobrist;

pub mod uci;
pub mod time_control;

pub mod move_ordering;
pub mod qsearch;
pub mod lmr;

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
        let pos = Position::new();
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);

        // Make e2e4
        let m = collector.get(0);
        let new_pos = pos.make_move(&m);

        // assert_ne!(pos, new_pos);
        assert_eq!(new_pos.side_to_move, Color::Black);
    }
}
