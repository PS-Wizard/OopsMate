use board::Position;
use types::moves::{Move, MoveCollector};

pub struct MoveParser;

impl MoveParser {
    /// Apply a sequence of UCI move strings to a position
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

        // Find matching move
        for i in 0..collector.len() {
            let m = collector[i];
            if Self::move_matches(m, move_str) {
                *position = position.make_move(m);
                return Ok(());
            }
        }

        Err(format!("Illegal or invalid move: {}", move_str))
    }

    /// Checks if the a move matches the one passed in as a notation from the UCI
    fn move_matches(m: Move, move_str: &str) -> bool {
        let from = m.from();
        let to = m.to();

        let from_sq = format!("{}{}", (b'a' + (from % 8) as u8) as char, (from / 8) + 1);
        let to_sq = format!("{}{}", (b'a' + (to % 8) as u8) as char, (to / 8) + 1);
        let move_string = format!("{}{}", from_sq, to_sq);

        // Check basic move
        if move_str == move_string {
            return true;
        }

        // Check with promotion
        if move_str.len() > 4 {
            // move_string is the basic move: "e2e4", "e7e8", etc.
            // For promotions, UCI format adds a letter at the end:
            //   e7e8q = promote to queen
            //   e7e8r = promote to rook
            //   e7e8b = promote to bishop
            //   e7e8n = promote to knight
            // &move_str[4..5] extracts that promotion piece character
            let expected_move = format!("{}{}", move_string, &move_str[4..5]);
            return move_str == expected_move;
        }

        false
    }
}
