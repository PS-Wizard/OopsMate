use board::Position;
use types::moves::{Move, MoveCollector};

pub struct MoveParser;

impl MoveParser {
    /// Apply a sequence of UCI move strings to a position in-place
    pub fn apply_moves(position: &mut Position, move_strs: &[&str]) -> Result<(), String> {
        for move_str in move_strs {
            Self::apply_single_move(position, move_str)?;
        }
        Ok(())
    }

    /// Applies a single move on the Position
    fn apply_single_move(position: &mut Position, move_str: &str) -> Result<(), String> {
        let mut collector = MoveCollector::new();
        position.generate_moves(&mut collector);

        for i in 0..collector.len() {
            let m = collector[i];
            if Self::move_matches(m, move_str) {
                // Apply move in-place; store UndoInfo if needed
                let _undo = position.make_move(m);
                return Ok(());
            }
        }

        Err(format!("Illegal or invalid move: {}", move_str))
    }

    /// Checks if a move matches a UCI string
    fn move_matches(m: Move, move_str: &str) -> bool {
        let from = m.from();
        let to = m.to();

        let from_sq = format!("{}{}", (b'a' + (from % 8) as u8) as char, (from / 8) + 1);
        let to_sq = format!("{}{}", (b'a' + (to % 8) as u8) as char, (to / 8) + 1);
        let move_string = format!("{}{}", from_sq, to_sq);

        // Exact match for basic move
        if move_str == move_string {
            return true;
        }

        // Match with promotion
        if move_str.len() > 4 {
            let expected_move = format!("{}{}", move_string, &move_str[4..5]);
            return move_str == expected_move;
        }

        false
    }
}
