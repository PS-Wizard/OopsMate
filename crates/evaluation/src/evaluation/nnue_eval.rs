use board::Position;
use nnueffi::{CColor, CPiece, NNUEData, NNUEProbe};
use std::sync::OnceLock;
use types::others::{Color, Piece};

// Use OnceLock instead of Once + static mut for thread-safe initialization
static NNUE_PROBE: OnceLock<NNUEProbe> = OnceLock::new();

/// Initialize NNUE probe (call once at startup)
pub fn init_nnue(eval_file: &str) -> Result<(), String> {
    NNUE_PROBE.get_or_init(|| {
        let mut probe = NNUEProbe::new();
        if let Err(e) = probe.init(eval_file) {
            eprintln!("Failed to initialize NNUE: {}", e);
        }
        probe
    });

    Ok(())
}

/// Get reference to initialized NNUE probe
fn get_probe() -> Result<&'static NNUEProbe, String> {
    NNUE_PROBE
        .get()
        .ok_or_else(|| "NNUE not initialized".to_string())
}

/// Check if NNUE is initialized
pub fn is_nnue_initialized() -> bool {
    NNUE_PROBE.get().is_some()
}

/// Trait for NNUE evaluation
pub trait NNUEEvaluator {
    /// Evaluate using NNUE with incremental updates
    fn evaluate_nnue(&self, nnue_data: &mut [*mut NNUEData; 3]) -> Result<i32, String>;

    /// Evaluate using NNUE without incremental updates (slower)
    fn evaluate_nnue_simple(&self) -> Result<i32, String>;

    /// Convert position to NNUE pieces and squares arrays
    fn to_nnue_arrays(&self) -> (Vec<i32>, Vec<i32>);
}

impl NNUEEvaluator for Position {
    fn evaluate_nnue(&self, nnue_data: &mut [*mut NNUEData; 3]) -> Result<i32, String> {
        let probe = get_probe()?;
        let (mut pieces, mut squares) = self.to_nnue_arrays();
        let player = match self.side_to_move {
            Color::White => CColor::White,
            Color::Black => CColor::Black,
        };

        probe.evaluate_incremental(player, &mut pieces, &mut squares, nnue_data)
    }

    fn evaluate_nnue_simple(&self) -> Result<i32, String> {
        let probe = get_probe()?;
        let (mut pieces, mut squares) = self.to_nnue_arrays();
        let player = match self.side_to_move {
            Color::White => CColor::White,
            Color::Black => CColor::Black,
        };

        probe.evaluate(player, &mut pieces, &mut squares)
    }

    fn to_nnue_arrays(&self) -> (Vec<i32>, Vec<i32>) {
        let mut pieces = Vec::with_capacity(33);
        let mut squares = Vec::with_capacity(33);

        // Helper to convert our piece/color to CPiece
        let to_cpiece = |piece: Piece, color: Color| -> CPiece {
            match (piece, color) {
                (Piece::King, Color::White) => CPiece::WKing,
                (Piece::Queen, Color::White) => CPiece::WQueen,
                (Piece::Rook, Color::White) => CPiece::WRook,
                (Piece::Bishop, Color::White) => CPiece::WBishop,
                (Piece::Knight, Color::White) => CPiece::WKnight,
                (Piece::Pawn, Color::White) => CPiece::WPawn,
                (Piece::King, Color::Black) => CPiece::BKing,
                (Piece::Queen, Color::Black) => CPiece::BQueen,
                (Piece::Rook, Color::Black) => CPiece::BRook,
                (Piece::Bishop, Color::Black) => CPiece::BBishop,
                (Piece::Knight, Color::Black) => CPiece::BKnight,
                (Piece::Pawn, Color::Black) => CPiece::BPawn,
            }
        };

        // Find king squares
        let mut wking_sq = None;
        let mut bking_sq = None;

        for sq in 0..64 {
            if let Some((piece, color)) = self.piece_map[sq] {
                if piece == Piece::King {
                    if color == Color::White {
                        wking_sq = Some(sq);
                    } else {
                        bking_sq = Some(sq);
                    }
                }
            }
        }

        // Kings must be first two entries
        if let Some(sq) = wking_sq {
            pieces.push(CPiece::WKing as i32);
            squares.push(sq as i32);
        }

        if let Some(sq) = bking_sq {
            pieces.push(CPiece::BKing as i32);
            squares.push(sq as i32);
        }

        // Add all other pieces
        for sq in 0..64 {
            if let Some((piece, color)) = self.piece_map[sq] {
                if piece != Piece::King {
                    pieces.push(to_cpiece(piece, color) as i32);
                    squares.push(sq as i32);
                }
            }
        }

        // Add terminator
        pieces.push(0);
        squares.push(0);

        (pieces, squares)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nnue_initialization() {
        // This will fail if NNUE file doesn't exist, but shows the pattern
        let result = init_nnue("assets/nn-04cf2b4ed1da.nnue");
        assert!(result.is_ok());
        assert!(is_nnue_initialized());
    }

    #[test]
    fn test_piece_array_conversion() {
        let pos = Position::new();
        let (pieces, squares) = pos.to_nnue_arrays();

        // Should have: 2 kings + 30 other pieces + 1 terminator = 33 entries
        assert_eq!(pieces.len(), 33);
        assert_eq!(squares.len(), 33);

        // First two should be kings
        assert_eq!(pieces[0], CPiece::WKing as i32);
        assert_eq!(pieces[1], CPiece::BKing as i32);

        // Last entry should be terminator
        assert_eq!(pieces[32], 0);
        assert_eq!(squares[32], 0);
    }
}
